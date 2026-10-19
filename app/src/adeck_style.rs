//! Presentation rules for the spaghetti-models overlay: wind-category colours,
//! per-segment colour, line styles, the km-projection cache, cycle/age text,
//! the disclaimer, and intensity-chart scaling. Pure: no drawing calls.

use crate::adeck::{GuidanceStore, IntensitySeries, ModelGroup, StormGuidance};
use crate::geo;
use ply_engine::prelude::Vec2;

/// Saffir-Simpson wind category of a forecast point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindCategory {
    /// VMAX column was 0 (no value), e.g. TVCN.
    Unknown,
    Depression,
    Storm,
    Cat1,
    Cat2,
    Cat3,
    Cat4,
    Cat5,
}

impl WindCategory {
    /// Legend order: weakest to strongest, then "no intensity".
    pub const LEGEND: [WindCategory; 8] = [
        WindCategory::Depression,
        WindCategory::Storm,
        WindCategory::Cat1,
        WindCategory::Cat2,
        WindCategory::Cat3,
        WindCategory::Cat4,
        WindCategory::Cat5,
        WindCategory::Unknown,
    ];

    pub fn from_kt(kt: Option<u32>) -> Self {
        match kt {
            None => WindCategory::Unknown,
            Some(k) if k < 34 => WindCategory::Depression,
            Some(k) if k < 64 => WindCategory::Storm,
            Some(k) if k < 83 => WindCategory::Cat1,
            Some(k) if k < 96 => WindCategory::Cat2,
            Some(k) if k < 113 => WindCategory::Cat3,
            Some(k) if k < 137 => WindCategory::Cat4,
            Some(_) => WindCategory::Cat5,
        }
    }

    /// Standard Saffir-Simpson scale colours; grey for unknown.
    pub fn rgb(self) -> [u8; 3] {
        match self {
            WindCategory::Unknown => [0x9E, 0x95, 0x90],
            WindCategory::Depression => [0x5E, 0xBA, 0xFF],
            WindCategory::Storm => [0x00, 0xFA, 0xF4],
            WindCategory::Cat1 => [0xFF, 0xFF, 0xCC],
            WindCategory::Cat2 => [0xFF, 0xE7, 0x75],
            WindCategory::Cat3 => [0xFF, 0xC1, 0x40],
            WindCategory::Cat4 => [0xFF, 0x8F, 0x20],
            WindCategory::Cat5 => [0xFF, 0x60, 0x60],
        }
    }

    /// `0xRRGGBB` for Ply `background_color`.
    pub fn hex(self) -> u32 {
        rgb_hex(self.rgb())
    }

    pub fn label(self) -> &'static str {
        match self {
            WindCategory::Unknown => "No intensity",
            WindCategory::Depression => "TD  <34 kt",
            WindCategory::Storm => "TS  34-63 kt",
            WindCategory::Cat1 => "Cat 1  64-82 kt",
            WindCategory::Cat2 => "Cat 2  83-95 kt",
            WindCategory::Cat3 => "Cat 3  96-112 kt",
            WindCategory::Cat4 => "Cat 4  113-136 kt",
            WindCategory::Cat5 => "Cat 5  137+ kt",
        }
    }

    /// Lower bound in kt (intensity-chart bands); `None` for Unknown.
    pub fn min_kt(self) -> Option<u32> {
        match self {
            WindCategory::Unknown => None,
            WindCategory::Depression => Some(0),
            WindCategory::Storm => Some(34),
            WindCategory::Cat1 => Some(64),
            WindCategory::Cat2 => Some(83),
            WindCategory::Cat3 => Some(96),
            WindCategory::Cat4 => Some(113),
            WindCategory::Cat5 => Some(137),
        }
    }
}

/// `[r, g, b]` -> `0xRRGGBB`, shared by every swatch/outline colour that
/// needs both an RGB triplet (for draw calls that take separate channels)
/// and a packed hex (for Ply `background_color`), so the two never drift
/// apart.
pub fn rgb_hex(rgb: [u8; 3]) -> u32 {
    let [r, g, b] = rgb;
    (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
}

/// RGBA for the segment between two consecutive points: coloured by the
/// wind at its start (falling back to its end). CARQ (observed) is plain
/// grey; OFCL is opaque, models slightly translucent.
pub fn segment_rgba(group: ModelGroup, start_kt: Option<u32>, end_kt: Option<u32>) -> [u8; 4] {
    if group == ModelGroup::Observed {
        return [0xB0, 0xB0, 0xB0, 200];
    }
    let [r, g, b] = WindCategory::from_kt(start_kt.or(end_kt)).rgb();
    let a = if group == ModelGroup::Official {
        255
    } else {
        210
    };
    [r, g, b, a]
}

/// A repeating screen-space dash pattern for a track line (review S11):
/// draw `on_px`, skip `off_px`, repeat. Both lengths are constant SCREEN
/// pixels, never scaled by zoom or `px_per_km` — that's what keeps the
/// pattern from "exploding into tiny segments at high zoom" (the review's
/// own wording): a world-space dash length would shrink to a handful of
/// screen pixels at low zoom and need thousands of sub-segments at high
/// zoom, where a screen-space one always draws the same handful of marks
/// per visible pixel run regardless of how far the camera is zoomed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DashPattern {
    pub on_px: f32,
    pub off_px: f32,
}

impl DashPattern {
    /// Total length of one on+off repeat.
    pub fn cycle_px(self) -> f32 {
        self.on_px + self.off_px
    }
}

/// One drawn ("on") sub-span of a dashed segment, in distance along that
/// segment: `0.0` is the segment's start point, `end` <= the segment's full
/// screen-space length. Gaps are simply omitted from the list that
/// `dash_steps` returns — nothing needs to be drawn for them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DashSpan {
    pub start: f32,
    pub end: f32,
}

/// Steps a `pattern` across one screen-space segment of `length_px` pixels,
/// starting `phase_px` pixels into the pattern's repeating on/off cycle, and
/// returns the "on" sub-spans to draw plus the phase to carry into the next
/// segment of the same track (review S11: the phase must accumulate
/// continuously along a track's whole polyline, not reset to 0 at every
/// vertex, or the pattern visibly jumps at each forecast point).
///
/// Pure and allocation-light; no macroquad calls, so it's unit-testable
/// headless — unlike the `draw_line` calls that consume its output.
///
/// Degenerate inputs never loop forever or divide by zero: a non-positive
/// `length_px` draws nothing and passes `phase_px` through unchanged (nothing
/// moved, so there's nothing to re-phase); a pattern with no "on" length (or
/// a non-positive cycle) also draws nothing, and its returned phase is
/// irrelevant to any future draw, so it's reported as `0.0` rather than
/// carrying forward a value that could otherwise grow without bound across
/// a long polyline of all-degenerate segments.
/// Hard ceiling on a single `dash_steps` call's length, in px. Callers that
/// clip to the viewport first (`scope::draw_model_tracks`, review N1) never
/// pass anything near this, so it exists purely as defense in depth: an f32
/// `pos` stops advancing around 2^25 px when the step is small, so a
/// non-finite or absurdly large `length_px` could otherwise spin the loop
/// below forever (review N17).
const MAX_DASH_STEPS_LENGTH_PX: f32 = 1.0e6;

pub fn dash_steps(pattern: DashPattern, phase_px: f32, length_px: f32) -> (Vec<DashSpan>, f32) {
    if !length_px.is_finite() || length_px <= 0.0 {
        return (Vec::new(), phase_px);
    }
    let length_px = length_px.min(MAX_DASH_STEPS_LENGTH_PX);
    let cycle = pattern.cycle_px();
    if cycle <= 0.0 || pattern.on_px <= 0.0 {
        return (Vec::new(), 0.0);
    }
    let mut phase = phase_px.rem_euclid(cycle);
    let mut pos = 0.0_f32;
    let mut spans = Vec::new();
    while pos < length_px {
        let in_on = phase < pattern.on_px;
        let remaining_in_state = if in_on {
            pattern.on_px - phase
        } else {
            cycle - phase
        };
        let step = remaining_in_state.min(length_px - pos);
        if step <= 0.0 {
            // Unreachable given the guards above (cycle > 0 and on_px > 0
            // mean remaining_in_state is always > 0), but never spin.
            break;
        }
        if in_on {
            spans.push(DashSpan {
                start: pos,
                end: pos + step,
            });
        }
        pos += step;
        phase += step;
        if phase >= cycle {
            phase -= cycle;
        }
    }
    (spans, phase)
}

/// Advances a dash phase by `distance_px` without enumerating any spans —
/// an O(1) modular step, matching `dash_steps`'s own degenerate-input
/// conventions (a non-positive cycle or `on_px`, or a non-finite distance,
/// reports phase `0.0`). Used to skip over the part of a track segment
/// that `clip_segment_to_rect` clipped away as off-screen, so the dash
/// pattern stays continuous once the track re-enters view without ever
/// running `dash_steps`'s per-dash loop over a distance longer than what's
/// actually on screen — that loop, not just the resulting `draw_line`
/// calls, is what explodes at high zoom if it has to walk a whole
/// multi-thousand-pixel off-screen run (review N1).
pub fn advance_dash_phase(pattern: DashPattern, phase_px: f32, distance_px: f32) -> f32 {
    let cycle = pattern.cycle_px();
    if cycle <= 0.0 || pattern.on_px <= 0.0 || !distance_px.is_finite() {
        return 0.0;
    }
    (phase_px + distance_px.max(0.0)).rem_euclid(cycle)
}

/// Hard ceiling on spans drawn for one clipped segment. Clipping first
/// (review N1) already bounds `dash_steps`'s input to roughly the screen
/// diagonal, so this never bites for any pattern `track_style` actually
/// assigns — it's a cheap backstop against a pattern with an on/off pair
/// far smaller than those, which could still produce an excessive span
/// count even over a screen-bounded length.
pub const MAX_DASH_SPANS_PER_SEGMENT: usize = 2000;

/// Collapses `spans` to one solid span covering `[0, length_px]` if there
/// are more than `MAX_DASH_SPANS_PER_SEGMENT` of them, trading dash detail
/// for a bounded number of `draw_line` calls (review N1).
pub fn cap_dash_spans(spans: Vec<DashSpan>, length_px: f32) -> Vec<DashSpan> {
    if spans.len() > MAX_DASH_SPANS_PER_SEGMENT {
        vec![DashSpan {
            start: 0.0,
            end: length_px.max(0.0),
        }]
    } else {
        spans
    }
}

/// Liang-Barsky clip of the segment `(x0,y0)-(x1,y1)` against the
/// axis-aligned rect `[rx0,ry0]-[rx1,ry1]`. Returns the parametric range
/// `(t0, t1)` (`0.0..=1.0`, `t0 <= t1`) of the segment that lies inside the
/// rect, or `None` if the segment doesn't intersect it at all. Used to clip
/// each model-track segment to the viewport (plus a margin) before dashing
/// it, which is the actual fix for review N1: a track segment that's
/// thousands of screen pixels long off-screen no longer costs anything
/// beyond this one check, instead of walking its entire length through
/// `dash_steps`.
#[allow(clippy::too_many_arguments)]
pub fn clip_segment_to_rect(
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    rx0: f32,
    ry0: f32,
    rx1: f32,
    ry1: f32,
) -> Option<(f32, f32)> {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let mut t0 = 0.0_f32;
    let mut t1 = 1.0_f32;
    for (p, q) in [
        (-dx, x0 - rx0),
        (dx, rx1 - x0),
        (-dy, y0 - ry0),
        (dy, ry1 - y0),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return None; // parallel to this edge and entirely outside it
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                if r > t1 {
                    return None;
                }
                if r > t0 {
                    t0 = r;
                }
            } else {
                if r < t0 {
                    return None;
                }
                if r < t1 {
                    t1 = r;
                }
            }
        }
    }
    (t0 <= t1).then_some((t0, t1))
}

/// Line style per model group.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackStyle {
    pub width_px: f32,
    /// White outline drawn under the line (OFCL only), in px.
    pub underlay_px: Option<f32>,
    /// Draw the tech id at the track's last point.
    pub label: bool,
    /// Higher draws later (on top).
    pub draw_rank: u8,
    /// Screen-space dash/gap pattern (review S11); `None` is a solid line.
    pub dash: Option<DashPattern>,
}

/// Per-`ModelGroup` line style, including the dash pattern (review S11).
/// Patterns are assigned only to the groups that `classify_tech`
/// (`adeck.rs`) can actually mark `default_visible` — the ones realistically
/// on screen together: Official, Observed, Consensus, Hurricane, Global,
/// Ensemble, Ai. Every other group (Statistical, Baseline, Trajectory,
/// EnsembleMember, Late, Other) is never default-visible, so it falls to the
/// `_` arm; its dash choice is moot today but still has to be *something*,
/// and reusing the plain default (no dash) is the least surprising choice if
/// one is ever toggled visible later.
///
/// Scheme (all screen-space px, review S11's "fixed screen-space pattern"
/// constraint — see `dash_steps`):
/// - **Official**: solid. It already has its own dedicated 2.5px width and a
///   5px dark outline underlay (`underlay_px`), so a dash would be redundant
///   camouflage against the one track that must never be ambiguous.
/// - **Observed (CARQ)**: solid. This is the storm's own recorded past
///   track, not a forecast model — a dash pattern implies "one model's
///   extrapolation among several," which doesn't describe observed fixes.
///   It's already visually distinct (plain grey, unlabeled, thin).
/// - **Consensus**: long dash (12 on / 6 off) — the biggest, clearest marks
///   of the five model dash styles, fitting for the multi-model averages
///   that are usually the most-trusted non-official guidance.
/// - **Hurricane**: medium dash (8 on / 4 off) — the specialized hurricane
///   models, one size down from Consensus.
/// - **Ensemble**: short dash (5 on / 5 off), an even 1:1 mark:space ratio —
///   visually reads as "busier" than the two longer dashes above it.
/// - **Ai**: fine dash (3 on / 3 off) — same 1:1 ratio as Ensemble but at
///   half the scale, so the two are told apart by mark *size*, the same way
///   Consensus and Hurricane are.
/// - **Global**: dotted (1.5 on / 4 off) — short marks, wide gaps; the
///   lowest duty cycle of the set, reading as "dots" rather than "dashes".
pub fn track_style(group: ModelGroup) -> TrackStyle {
    match group {
        ModelGroup::Official => TrackStyle {
            width_px: 2.5,
            underlay_px: Some(5.0),
            label: true,
            draw_rank: 2,
            dash: None,
        },
        ModelGroup::Observed => TrackStyle {
            width_px: 1.5,
            underlay_px: None,
            label: false,
            draw_rank: 1,
            dash: None,
        },
        ModelGroup::Consensus => TrackStyle {
            width_px: 1.5,
            underlay_px: None,
            label: true,
            draw_rank: 0,
            dash: Some(DashPattern {
                on_px: 12.0,
                off_px: 6.0,
            }),
        },
        ModelGroup::Hurricane => TrackStyle {
            width_px: 1.5,
            underlay_px: None,
            label: true,
            draw_rank: 0,
            dash: Some(DashPattern {
                on_px: 8.0,
                off_px: 4.0,
            }),
        },
        ModelGroup::Ensemble => TrackStyle {
            width_px: 1.5,
            underlay_px: None,
            label: true,
            draw_rank: 0,
            dash: Some(DashPattern {
                on_px: 5.0,
                off_px: 5.0,
            }),
        },
        ModelGroup::Ai => TrackStyle {
            width_px: 1.5,
            underlay_px: None,
            label: true,
            draw_rank: 0,
            dash: Some(DashPattern {
                on_px: 3.0,
                off_px: 3.0,
            }),
        },
        ModelGroup::Global => TrackStyle {
            width_px: 1.5,
            underlay_px: None,
            label: true,
            draw_rank: 0,
            dash: Some(DashPattern {
                on_px: 1.5,
                off_px: 4.0,
            }),
        },
        _ => TrackStyle {
            width_px: 1.5,
            underlay_px: None,
            label: true,
            draw_rank: 0,
            dash: None,
        },
    }
}

/// Legend label for a model group's own dash-pattern swatch row (review
/// S11's item 3). Only the groups with a *distinct* dash pattern in
/// `track_style` get one here — Official and Observed are solid and already
/// have their own dedicated legend rows elsewhere (the OFCL swatch and the
/// "CARQ past track" row), so repeating them here would be redundant.
pub fn model_group_legend_label(group: ModelGroup) -> Option<&'static str> {
    match group {
        ModelGroup::Consensus => Some("Consensus models"),
        ModelGroup::Hurricane => Some("Hurricane models"),
        ModelGroup::Ensemble => Some("Ensemble means"),
        ModelGroup::Ai => Some("AI models"),
        ModelGroup::Global => Some("Global models"),
        _ => None,
    }
}

/// The five dash-styled model groups, in the order the legend lists them —
/// longest dash (most "trusted"/aggregated) to dotted, mirroring the
/// `track_style` doc comment's own size ordering.
pub const MODEL_GROUP_LEGEND_ORDER: [ModelGroup; 5] = [
    ModelGroup::Consensus,
    ModelGroup::Hurricane,
    ModelGroup::Ensemble,
    ModelGroup::Ai,
    ModelGroup::Global,
];

/// Tiles `pattern` across a `total_px`-wide legend swatch preview as
/// alternating (length, is_on) segments, starting "on" — used by
/// `model_legend.rs` to draw a miniature preview of each model group's real
/// dash pattern (review S11 item 3), rather than a plain solid colour bar
/// that no longer matches what's drawn on the map. `None` (solid) and any
/// degenerate pattern (non-positive cycle or "on" length) both fall back to
/// one single "on" segment spanning the whole preview width, so the legend
/// never renders a blank swatch.
///
/// This is deliberately a different, simpler shape than `dash_steps`:
/// `dash_steps` clips a pattern to *one real polyline segment* and carries a
/// phase across segment boundaries, which this preview has no use for (it's
/// one fixed-width swatch, always starting fresh at phase 0); this instead
/// needs the "off" gaps as their own explicit segments too, so the caller
/// can lay out alternating coloured/transparent elements in sequence.
pub fn dash_preview_segments(pattern: Option<DashPattern>, total_px: f32) -> Vec<(f32, bool)> {
    let solid = || vec![(total_px.max(0.0), true)];
    let Some(p) = pattern else {
        return solid();
    };
    if total_px <= 0.0 {
        return Vec::new();
    }
    let cycle = p.cycle_px();
    if cycle <= 0.0 || p.on_px <= 0.0 {
        return solid();
    }
    let mut segs = Vec::new();
    let mut pos = 0.0_f32;
    let mut on = true;
    while pos < total_px {
        let want = if on { p.on_px } else { p.off_px };
        let seg = want.min(total_px - pos);
        segs.push((seg, on));
        pos += seg;
        on = !on;
    }
    segs
}

/// OFCL's outline colour, drawn under its own wind-coloured line
/// (`scope.rs`'s `draw_model_tracks`, `underlay_px`). Dark instead of white
/// (review S4): the palest wind colours, used right where hurricane
/// strength starts — Cat 1 `#FFFFCC` and Cat 2 `#FFE775` — were nearly
/// invisible against a white 5 px outline. A dark outline gives
/// near-maximal contrast against every colour in the wind palette (all of
/// which are light or saturated), not just the two palest ones, so this
/// one change covers the whole scale rather than only re-saturating Cat
/// 1/2 in isolation.
pub const OFFICIAL_OUTLINE_RGB: [u8; 3] = [0x10, 0x10, 0x10];

/// Alpha for the outline underlay before cycle-age dimming
/// (`track_alpha_mult`) is applied to it, same base alpha as the white
/// outline it replaces.
pub const OFFICIAL_OUTLINE_ALPHA: u8 = 230;

/// `OFFICIAL_OUTLINE_RGB` as `0xRRGGBB`.
pub fn official_outline_hex() -> u32 {
    rgb_hex(OFFICIAL_OUTLINE_RGB)
}

/// Outline colour for the legend's OFCL swatch specifically — lighter than
/// `OFFICIAL_OUTLINE_RGB`, not the same value. The map's dark outline works
/// there because it sits against bright radar echo and pale wind colours;
/// inside the legend panel the background is itself dark
/// (`glass_panel::GLASS_BG`, roughly `#12161E`), so that same dark outline
/// would be invisible against the panel (review fix-round finding: the
/// swatch was indistinguishable from the plain Cat 3 row above it). A light
/// neutral grey isn't on the Saffir-Simpson palette (which has no
/// grey/white hue) and reads clearly against the dark panel.
pub const OFFICIAL_OUTLINE_LEGEND_RGB: [u8; 3] = [0xCF, 0xCF, 0xCF];

/// `OFFICIAL_OUTLINE_LEGEND_RGB` as `0xRRGGBB`.
pub fn official_outline_legend_hex() -> u32 {
    rgb_hex(OFFICIAL_OUTLINE_LEGEND_RGB)
}

/// Fill colours for the legend's OFCL swatch, drawn as adjacent segments
/// rather than one solid block (review fix-round finding: a single Cat 3
/// fill was visually identical to the plain Cat 3 swatch two rows above
/// it). Low/medium/high wind categories, so the swatch's own shape conveys
/// "this track's colour varies by segment" rather than relying on prose
/// alone to say so.
pub const OFFICIAL_SWATCH_SEGMENTS: [WindCategory; 3] =
    [WindCategory::Storm, WindCategory::Cat3, WindCategory::Cat5];

/// Colour for the pre-existing NHC advisory GIS track, drawn in
/// `scope.rs::draw_nhc_overlays` from `NhcBundle.gis_storms` — code that
/// predates this branch. Moved off the Saffir-Simpson palette (review S4:
/// the old `#FFCC66` sat close to Cat 3's `#FFC140` and wasn't in the
/// legend, so it could pass for a model track). No *other* overlay drawn
/// over a storm's own track uses violet, so it reads unambiguously as "the
/// other kind of forecast track" in that context — though the radar
/// palette itself comes close at the extremes (70 dBZ reflectivity is
/// `#9854C6`, and the range-folded colour is `#770077`; see `colors.rs`).
/// That's an acceptable, low-risk collision: those are sparse, high-
/// intensity-only echo colours on the radar layer underneath, not colours
/// any other *forecast overlay* uses, so a track segment sitting on top of
/// either briefly is a contrast issue at worst, not a "which overlay is
/// this" ambiguity. Shared by the GIS draw code and the legend swatch so
/// both stay in sync.
pub const GIS_ADVISORY_RGB: [u8; 3] = [0xB2, 0x66, 0xFF];

/// The legend's OFCL row has three distinct states, not two (review
/// fix-round 2, reopened Important 4). A storm's OFCL can be: actually
/// drawn; genuinely absent this cycle (`StormGuidance::has_official ==
/// false`, review S8); or present in the a-deck but *suppressed* because
/// the live NHC advisory GIS track is shown instead (review S4's fix).
/// Conflating the last two as one "not drawn" state made the legend claim
/// "No OFCL this cycle" directly above a legend row that correctly implies
/// an official forecast *is* shown — self-contradictory in exactly the
/// case S4's fix exists to clarify.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfficialLegendState {
    /// An OFCL track is actually drawn this frame.
    Drawn,
    /// OFCL exists for some shown storm but is suppressed in favour of that
    /// storm's live NHC advisory GIS track.
    SuppressedForGis,
    /// No shown storm has an OFCL track, and none was suppressed either —
    /// this cycle genuinely has no OFCL (most invests).
    NotIssued,
}

/// Picks the legend's OFCL state from two independently-computed facts:
/// whether any OFCL is actually drawn (`any_official_track_shown`) and
/// whether any OFCL was suppressed for GIS overlap
/// (`any_official_suppressed_by_gis`). `Drawn` wins if any track is drawn
/// at all, even if a *different* shown storm's OFCL was separately
/// suppressed — the legend is one aggregate panel, and "some official
/// guidance is on screen" takes priority over describing the suppressed
/// case when both are true.
pub fn official_legend_state(any_drawn: bool, any_suppressed: bool) -> OfficialLegendState {
    if any_drawn {
        OfficialLegendState::Drawn
    } else if any_suppressed {
        OfficialLegendState::SuppressedForGis
    } else {
        OfficialLegendState::NotIssued
    }
}

/// Legend text for the OFCL row, one line per `OfficialLegendState`.
/// `SuppressedForGis`'s wording deliberately doesn't claim "no OFCL" — that
/// would be false in this state, since OFCL does exist, it's just not the
/// track being drawn.
pub fn official_legend_label(state: OfficialLegendState) -> &'static str {
    match state {
        OfficialLegendState::Drawn => "OFCL  NHC official",
        OfficialLegendState::SuppressedForGis => "OFCL shown as advisory track",
        OfficialLegendState::NotIssued => "No OFCL this cycle",
    }
}

/// Appends " (stale)" to a legend line already coloured amber for
/// staleness. Colour alone is an accessibility gap (review L1) — the
/// model-guidance card already appends this suffix to its own age line;
/// this gives the legend the same text cue, not just the card.
pub fn stale_suffix(line: &str, stale: bool) -> String {
    if stale {
        format!("{line} (stale)")
    } else {
        line.to_string()
    }
}

/// Canonical two-letter ATCF basin code. `CurrentStorms.json`'s
/// `binNumber` (`StormMeta::bin_number`, e.g. `"AT2"`) prefixes the
/// Atlantic `"AT"`, but NHC's live ArcGIS MapServer (`StormGis::basin`)
/// returns `"AL"` for the same basin — confirmed against the real feed and
/// an archived Atlantic shapefile (`al142024`, Hurricane Milton) during
/// review (fix-round 2, reopened Important 1). EP and CP agree between the
/// two systems, so only the Atlantic pair needs aliasing; this normalizes
/// `"AT"` to `"AL"` so both sides compare equal.
fn canonical_basin(basin: &str) -> String {
    let basin = basin.trim().to_ascii_uppercase();
    if basin == "AT" {
        "AL".to_string()
    } else {
        basin
    }
}

/// True when `bin_number` (`StormMeta`, e.g. `"AT2"`) and `gis_basin`
/// (`StormGis::basin`, e.g. `"AL"` for Atlantic, `"EP"`/`"CP"` otherwise)
/// name the same ATCF basin, via `canonical_basin`. Required alongside the
/// name match in `gis_track_shown_for`: NHC basin-numbers invests and
/// short/common names from 1 within each basin, so a name match alone
/// can't rule out two different-basin storms.
///
/// Round 1 of this fix compared `bin_number`'s *prefix* directly against
/// `gis_basin` with no aliasing, which silently never matched any Atlantic
/// storm (`"AT9".starts_with("AL")` is false) — the basin check, and so
/// the whole OFCL-suppression fix, was dead code for the one basin this
/// app's primary use case cares about most. This takes `bin_number`'s
/// first two characters as its own basin code (rather than relying on
/// `starts_with`, which only happened to work before because the two
/// strings were being compared literally) and canonicalizes both sides
/// before comparing.
fn basin_matches(bin_number: &str, gis_basin: &str) -> bool {
    let gis_basin = gis_basin.trim();
    if gis_basin.is_empty() {
        return false;
    }
    let bin_basin: String = bin_number.trim().chars().take(2).collect();
    if bin_basin.is_empty() {
        return false;
    }
    canonical_basin(&bin_basin) == canonical_basin(gis_basin)
}

/// True when `storm_id`'s a-deck OFCL track already has a matching, live,
/// currently-drawn NHC advisory GIS track, so the OFCL track should be
/// suppressed rather than drawn as a second, possibly-diverging "official"
/// track (review S4: a-deck OFCL "can differ from official NHC products"
/// per the ATCF README, and can lag the live advisory — see review S8).
///
/// This does **not** reuse `nhc.rs`'s own GIS-layer-to-GIS-layer match
/// (`storms.iter_mut().find(|s| name.contains(&s.name))`, case-sensitive,
/// one-direction, basin-blind) — that rule exists to merge one storm's own
/// cone/track/points/watches layers onto each other, and reusing it
/// directly here would be unsafe: a plain substring match on
/// `CurrentStorms` names produces real false positives, since depression
/// names are often plain numbers (`"Four"` is a substring of
/// `"Fourteen-E"`, `"Nine"` of `"Nine-E"`, `"Seven"` of `"Seventeen-E"`).
/// Wrongly suppressing a brand-new depression's only official track because
/// an unrelated storm's GIS name happens to contain its name would be worse
/// than the bug this fix addresses, exactly in the new-storm window where
/// it matters most.
///
/// Instead this requires an **exact**, case-insensitive, trimmed name match
/// *and* a basin match (`basin_matches`), since `StormGis` carries no storm
/// id (only a bare name — there is no existing id-based accessor to reuse).
/// `storm_id` is looked up in `metas` (`StormMeta::id` is the same
/// CurrentStorms id `GuidanceStore` keys on) to get both its bare name and
/// its basin (`StormMeta::bin_number`).
///
/// `show_gis_track` should be the live `NhcOverlayState::show_track` toggle,
/// and the match also requires the candidate `StormGis::track` to be
/// non-empty: `draw_nhc_overlays` only actually draws the advisory track
/// when both hold, so without this check turning the GIS track layer off
/// (or a storm whose GIS track is still empty) would suppress OFCL while
/// drawing nothing official at all — worse than the original bug (review
/// fix-round finding).
pub fn gis_track_shown_for(
    storm_id: &str,
    metas: &[crate::nhc::StormMeta],
    gis_storms: &[crate::nhc::StormGis],
    show_gis_track: bool,
) -> bool {
    if !show_gis_track {
        return false;
    }
    let Some(meta) = metas.iter().find(|m| m.id.eq_ignore_ascii_case(storm_id)) else {
        return false;
    };
    let name = meta.name.trim();
    if name.is_empty() {
        return false;
    }
    gis_storms.iter().any(|g| {
        !g.track.is_empty()
            && g.name.trim().eq_ignore_ascii_case(name)
            && basin_matches(&meta.bin_number, &g.basin)
    })
}

/// True when `g`'s guidance has a default-visible, >=2-point
/// `ModelGroup::Official` track — i.e. it has *something* that would be an
/// OFCL candidate for `draw_model_tracks`/`project_tracks`, before
/// `gis_track_shown_for`'s suppression check. Shared by
/// `any_official_track_shown` and `any_official_suppressed_by_gis` so the
/// eligibility rule isn't duplicated twice over within this module.
///
/// This still hand-copies `project_tracks`'s own `default_visible &&
/// points.len() >= 2` filter rather than deriving from its output
/// (`ProjectedTrack`), which is a real, acknowledged risk of drift between
/// the two (review fix-round 2 flagged this). Computing these from
/// `state.model_projection`'s already-filtered tracks instead — the exact
/// input `draw_model_tracks` receives — would remove that risk, but
/// `ProjectedTrack` only carries `storm_id`, not which original
/// `StormGuidance` it came from with its full field set, so re-deriving
/// "which of *this* storm's tracks are OFCL-eligible" from a flat,
/// already-storm-and-tech-mixed `Vec<ProjectedTrack>` is a bigger,
/// cache-shape-sensitive change than this fix round's scope. Left as a
/// documented follow-up rather than risked here.
fn has_official_candidate(g: &StormGuidance) -> bool {
    g.tracks
        .iter()
        .any(|t| t.group == ModelGroup::Official && t.default_visible && t.points.len() >= 2)
}

/// True when at least one shown storm's OFCL track is actually drawn this
/// frame by `draw_model_tracks`: it has an OFCL candidate
/// (`has_official_candidate`), and `gis_track_shown_for` doesn't suppress
/// it. Mirrors that function's per-track condition independently (rather
/// than threading a return value back up through
/// `draw_scope_to_texture`'s void render call chain), so the legend's OFCL
/// row can describe what the map actually shows without coupling the
/// render path to the legend.
pub fn any_official_track_shown(
    store: &GuidanceStore,
    metas: &[crate::nhc::StormMeta],
    gis_storms: &[crate::nhc::StormGis],
    show_gis_track: bool,
) -> bool {
    store.iter().any(|g| {
        has_official_candidate(g)
            && !gis_track_shown_for(&g.storm_id, metas, gis_storms, show_gis_track)
    })
}

/// True when at least one shown storm has an OFCL candidate
/// (`has_official_candidate`) that `gis_track_shown_for` suppresses — i.e.
/// OFCL exists for this cycle but isn't what's drawn, because that storm's
/// live NHC advisory GIS track is shown instead (review S4's fix; review
/// fix-round 2's reopened Important 4 needs this distinguished from
/// "genuinely no OFCL" in the legend).
pub fn any_official_suppressed_by_gis(
    store: &GuidanceStore,
    metas: &[crate::nhc::StormMeta],
    gis_storms: &[crate::nhc::StormGis],
    show_gis_track: bool,
) -> bool {
    store.iter().any(|g| {
        has_official_candidate(g)
            && gis_track_shown_for(&g.storm_id, metas, gis_storms, show_gis_track)
    })
}

/// Full disclaimer and attribution, one panel line each (no auto-wrap).
/// Wording from `docs/research/spaghetti-models.md` section 4.
pub const DISCLAIMER_LINES: [&str; 6] = [
    "Model guidance from NHC ATCF (aid_public).",
    "Individual models are not official forecasts.",
    "The NHC forecast (OFCL) is the official track.",
    "Data are automated and may contain errors.",
    "Consult NHC and your local NWS office.",
    "Source: NOAA/NHC, public domain.",
];

/// NHC's model catalogue and guidance disclaimer.
pub const MODEL_SUMMARY_URL: &str = "https://www.nhc.noaa.gov/modelsummary.shtml";

/// Data older than this is labelled "(stale)": two missed 15-min refreshes.
/// This is a *fetch*-time threshold, for the secondary "checked N min ago"
/// line — it flags a fetcher that has stopped succeeding. The primary
/// freshness signal is the forecast cycle's own age; see
/// `CYCLE_STALE_AFTER_SECS` below (review S1).
pub const STALE_AFTER_SECS: i64 = 30 * 60;

/// Cycle age past which the cycle readout is flagged "(stale)". NHC issues
/// a new synoptic cycle every 6 h, and OFCL typically lands about 3 h after
/// the nominal time, so a cycle in normal, healthy operation sits at
/// roughly 0-9 h old. 12 h gives headroom above that before warning, while
/// still catching a stalled feed (review S1's "stuck re-fetching the same
/// old cycle") well before the 24 h dim threshold below.
pub const CYCLE_STALE_AFTER_SECS: i64 = 12 * 60 * 60;

/// Cycle age past which model tracks are visually dimmed on the map, so
/// stale guidance can't look current even to someone who never reads the
/// legend or card text. 24 h is two full missed synoptic cycles —
/// comfortably past any routine OFCL lag, but still well short of the
/// multi-day staleness review S3 describes for a storm the fetcher can no
/// longer reach.
pub const CYCLE_DIM_AFTER_SECS: i64 = 24 * 60 * 60;

/// `2026100912` -> `"12Z Oct 9"`. Arithmetic on the integer (shared with
/// `adeck::cycle_time_unix` via `adeck::cycle_digits`), no string slicing,
/// so malformed values just print `???`.
pub fn format_cycle(init: u32) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let (_year, month, day, hour) = crate::adeck::cycle_digits(init);
    let name = (month as usize)
        .checked_sub(1)
        .and_then(|i| MONTHS.get(i))
        .copied()
        .unwrap_or("???");
    format!("{hour:02}Z {name} {day}")
}

/// Human age of a timestamp: "just now", "7 min ago", "2 h 5 min ago". Used
/// both for fetch age (the secondary "checked" line) and cycle age (the
/// primary freshness line, review S1) — just feed it the right timestamp.
pub fn format_age(fetched_at_unix: i64, now_unix: i64) -> String {
    let secs = (now_unix - fetched_at_unix).max(0);
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{} min ago", secs / 60)
    } else {
        format!("{} h {} min ago", secs / 3600, (secs % 3600) / 60)
    }
}

/// Fetch-time staleness, for the secondary "checked N min ago" line.
pub fn is_stale(fetched_at_unix: i64, now_unix: i64) -> bool {
    now_unix - fetched_at_unix > STALE_AFTER_SECS
}

/// Cycle-age staleness, for the primary cycle line and the legend/card
/// "(stale)" marker (review S1) — this is the one that should drive the
/// user-facing "is this current?" signal, not fetch time.
pub fn is_stale_cycle(cycle_unix: i64, now_unix: i64) -> bool {
    now_unix - cycle_unix > CYCLE_STALE_AFTER_SECS
}

/// Alpha multiplier for map-drawn tracks, from a track's cycle age: full
/// strength (`1.0`) under `CYCLE_DIM_AFTER_SECS`, then fading linearly to a
/// `0.35` floor by twice that age. Never reaches 0 — a legitimate backoff
/// window shouldn't erase the last known guidance from the map, only make
/// it visually recede so it can't be mistaken for current (review S1).
pub fn cycle_alpha_mult(age_secs: i64) -> f32 {
    if age_secs <= CYCLE_DIM_AFTER_SECS {
        1.0
    } else {
        let extra = (age_secs - CYCLE_DIM_AFTER_SECS) as f32;
        let span = CYCLE_DIM_AFTER_SECS as f32;
        (1.0 - 0.65 * (extra / span).min(1.0)).max(0.35)
    }
}

/// A visible track pre-projected to km offsets from the radar site.
/// The caller (`main.rs`) rebuilds its cache of these only when the site or
/// a non-empty store's revision changes, so per frame the draw is a bbox
/// cull plus one multiply-add per vertex (review H5). When the store drains
/// to empty (e.g. the storm leaves `CurrentStorms.json`), the caller draws
/// nothing rather than reusing the last cached, now-stale tracks.
///
/// `cycle_unix` is a property of the data (the storm's selected forecast
/// cycle), not of "now", so it's safe to cache here even though the cache
/// itself only rebuilds on a site or revision change. The *age-to-alpha*
/// mapping is deliberately NOT baked in here — `track_alpha_mult` below
/// recomputes it from `cycle_unix` and the current time on every draw, so
/// map dimming keeps advancing even across a total fetch outage, when
/// nothing ever bumps the store's revision and this cache never rebuilds
/// (review S1 critical fix: dimming must not freeze during the exact
/// outage scenario it exists to catch).
#[derive(Clone, Debug)]
pub struct ProjectedTrack {
    /// The CurrentStorms id this track belongs to (`StormGuidance::storm_id`),
    /// so a draw-time pass can decide per storm whether to suppress its
    /// OFCL track (review S4, `gis_track_shown_for`).
    pub storm_id: String,
    pub tech: String,
    pub group: ModelGroup,
    pub km: Vec<Vec2>,
    pub vmax_kt: Vec<Option<u32>>,
    pub min_km: Vec2,
    pub max_km: Vec2,
    /// This track's storm's selected forecast-cycle timestamp
    /// (`adeck::cycle_time_unix`), or `None` if `init` didn't parse.
    pub cycle_unix: Option<i64>,
}

/// This track's current alpha multiplier, from its cycle age evaluated
/// against `now_unix` — not from anything cached at projection time. Call
/// this fresh every frame (it's one `Option` map and a scalar multiply, not
/// a re-projection) so dimming tracks real time even when `ProjectedTrack`s
/// themselves haven't been rebuilt in a while (review S1).
pub fn track_alpha_mult(track: &ProjectedTrack, now_unix: i64) -> f32 {
    track
        .cycle_unix
        .map(|c| cycle_alpha_mult((now_unix - c).max(0)))
        .unwrap_or(1.0)
}

/// Project every default-visible track, ordered by draw rank (models, then
/// CARQ, then OFCL on top).
pub fn project_tracks(
    store: &GuidanceStore,
    origin_lat: f64,
    origin_lon: f64,
) -> Vec<ProjectedTrack> {
    let mut out: Vec<ProjectedTrack> = store
        .iter()
        .flat_map(|g| {
            let cycle_unix = crate::adeck::cycle_time_unix(g.init);
            g.tracks
                .iter()
                .map(move |t| (g.storm_id.as_str(), t, cycle_unix))
        })
        .filter(|(_, t, _)| t.default_visible && t.points.len() >= 2)
        .map(|(storm_id, t, cycle_unix)| {
            let km: Vec<Vec2> = t
                .points
                .iter()
                .map(|p| geo::point_to_km_offset(origin_lat, origin_lon, (p.lat, p.lon)))
                .collect();
            let min_km = km.iter().copied().reduce(Vec2::min).unwrap_or(Vec2::ZERO);
            let max_km = km.iter().copied().reduce(Vec2::max).unwrap_or(Vec2::ZERO);
            ProjectedTrack {
                storm_id: storm_id.to_string(),
                tech: t.tech.clone(),
                group: t.group,
                vmax_kt: t.points.iter().map(|p| p.vmax_kt).collect(),
                km,
                min_km,
                max_km,
                cycle_unix,
            }
        })
        .collect();
    out.sort_by_key(|t| track_style(t.group).draw_rank);
    out
}

/// The cycle-freshness line shared by the model-guidance card and the map
/// legend, plus whether it should read as stale: `"{cycle} cycle, {age}"`
/// for a single cycle, or `"Mixed cycles, oldest {cycle}, {age}"` when the
/// caller's shown storms disagree on `init` (review S5) — the mixed case
/// still surfaces the oldest shown cycle's own age and staleness instead of
/// suppressing it (review S5 fix-round finding: "mixed" must not mean "no
/// freshness information"). `init`'s age and staleness come from
/// `adeck::cycle_time_unix` + `is_stale_cycle`; an unparseable `init` omits
/// the age and never reads as stale (nothing to measure against).
pub fn cycle_freshness_line(init: u32, mixed: bool, now_unix: i64) -> (String, bool) {
    let cycle = format_cycle(init);
    let prefix = if mixed { "Mixed cycles, oldest " } else { "" };
    match crate::adeck::cycle_time_unix(init) {
        Some(c) => (
            format!("{prefix}{cycle} cycle, {}", format_age(c, now_unix)),
            is_stale_cycle(c, now_unix),
        ),
        None => (format!("{prefix}{cycle} cycle"), false),
    }
}

/// Top of the intensity chart's kt axis.
pub const CHART_MAX_KT: u32 = 160;

/// X-axis extent: the longest series' last tau, rounded up to a whole day,
/// clamped to 24..=168 h. `adeck::parse_adeck_line` already rejects a tau
/// outside a sane range (review L6), but `max + 23` is kept saturating
/// here too as defense in depth against a near-`i32::MAX` tau built some
/// other way: plain addition panics on overflow in debug and test builds
/// (it wraps and still clamps in release, so only debug/test ever crashed).
pub fn chart_max_tau(series: &[IntensitySeries]) -> i32 {
    let max = series
        .iter()
        .flat_map(|s| s.points.iter().map(|p| p.0))
        .max()
        .unwrap_or(0);
    (max.saturating_add(23) / 24 * 24).clamp(24, 168)
}

/// Category bands (lo_kt, hi_kt, rgb), TD through Cat 5, capped at CHART_MAX_KT.
pub fn chart_bands() -> Vec<(u32, u32, [u8; 3])> {
    let cats = [
        WindCategory::Depression,
        WindCategory::Storm,
        WindCategory::Cat1,
        WindCategory::Cat2,
        WindCategory::Cat3,
        WindCategory::Cat4,
        WindCategory::Cat5,
    ];
    cats.iter()
        .enumerate()
        .map(|(i, c)| {
            let lo = c.min_kt().unwrap_or(0);
            let hi = cats
                .get(i + 1)
                .and_then(|n| n.min_kt())
                .unwrap_or(CHART_MAX_KT)
                .min(CHART_MAX_KT);
            (lo, hi, c.rgb())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adeck::ModelGroup;
    use crate::adeck::{AidTrack, GuidanceStore, StormGuidance, TrackPoint};

    const FIXTURE: &str = include_str!("testdata/aal092026_sample.dat");

    /// `now` far enough past every hand-built fixture `init` in this module
    /// that `GuidanceStore::insert`'s future-cycle guard never rejects a
    /// test insert as implausible.
    const TEST_NOW: i64 = 4_102_444_800; // 2100-01-01T00:00:00Z

    fn fixture_store() -> GuidanceStore {
        let records = crate::adeck::parse_adeck(FIXTURE);
        let mut store = GuidanceStore::default();
        store.insert(
            crate::adeck::build_guidance("al092026", &records, 2026100912),
            TEST_NOW,
        );
        store
    }

    /// `now` right at the fixture's cycle time: age 0, so `track_alpha_mult`
    /// is 1.0 and tests that aren't about dimming aren't affected by it.
    fn fresh_now() -> i64 {
        crate::adeck::cycle_time_unix(2026100912).unwrap()
    }

    #[test]
    fn project_tracks_orders_official_last() {
        let tracks = project_tracks(&fixture_store(), 27.0, -87.6);
        let techs: Vec<&str> = tracks.iter().map(|t| t.tech.as_str()).collect();
        assert_eq!(techs, ["AVNI", "HWFI", "TVCN", "CARQ", "OFCL"]);
        let ofcl = tracks.last().unwrap();
        assert!(ofcl.km[0].length() < 0.01, "OFCL tau 0 is at the origin");
        for t in &tracks {
            assert_eq!(t.km.len(), t.vmax_kt.len());
            assert_eq!(
                track_alpha_mult(t, fresh_now()),
                1.0,
                "fresh cycle is not dimmed"
            );
            for p in &t.km {
                assert!(p.x >= t.min_km.x && p.x <= t.max_km.x);
                assert!(p.y >= t.min_km.y && p.y <= t.max_km.y);
            }
        }
    }

    #[test]
    fn project_tracks_skips_hidden_models() {
        let mut store = fixture_store();
        let mut g = store.get("al092026").unwrap().clone();
        g.tracks.push(AidTrack {
            tech: "TABS".into(),
            group: ModelGroup::Trajectory,
            default_visible: false,
            points: g.tracks[0].points.clone(),
        });
        store.insert(g, TEST_NOW);
        assert!(
            project_tracks(&store, 27.0, -87.6)
                .iter()
                .all(|t| t.tech != "TABS")
        );
    }

    #[test]
    fn track_alpha_mult_dims_past_the_cycle_age_threshold() {
        let store = fixture_store();
        let tracks = project_tracks(&store, 27.0, -87.6);
        let stale_now = fresh_now() + CYCLE_DIM_AFTER_SECS + 1;
        assert!(
            tracks.iter().all(|t| track_alpha_mult(t, stale_now) < 1.0),
            "every track for this storm shares its one cycle age"
        );
    }

    #[test]
    fn track_alpha_mult_changes_with_now_independent_of_any_store_mutation() {
        // This is the review fix-round regression test: dimming must be
        // able to change as time advances even though the `ProjectedTrack`
        // itself (and the `GuidanceStore` it came from) never mutates —
        // exactly what happens during a total fetch outage, when nothing
        // ever bumps `GuidanceStore::revision` and the caller's projection
        // cache never rebuilds.
        let store = fixture_store();
        let tracks = project_tracks(&store, 27.0, -87.6);
        let track = &tracks[0];
        let cycle_unix = track.cycle_unix.expect("fixture cycle parses");

        assert_eq!(track_alpha_mult(track, cycle_unix), 1.0, "fresh");
        assert_eq!(
            track_alpha_mult(track, cycle_unix + CYCLE_DIM_AFTER_SECS),
            1.0,
            "right at the dim threshold"
        );
        let just_over = track_alpha_mult(track, cycle_unix + CYCLE_DIM_AFTER_SECS + 1);
        assert!(just_over < 1.0, "just past the dim threshold");
        let way_over = track_alpha_mult(track, cycle_unix + CYCLE_DIM_AFTER_SECS * 10);
        assert!(
            way_over < just_over,
            "dims further as the outage (or just elapsed time) continues"
        );
        assert!(way_over >= 0.35, "never fully hidden");
    }

    #[test]
    fn projection_is_continuous_across_antimeridian() {
        let point = |lon: f64| TrackPoint {
            tau_h: 0,
            lat: 20.0,
            lon,
            vmax_kt: Some(50),
        };
        let mut store = GuidanceStore::default();
        store.insert(
            StormGuidance {
                storm_id: "cp012026".into(),
                init: 2026100912,
                has_official: true,
                tracks: vec![AidTrack {
                    tech: "AVNI".into(),
                    group: ModelGroup::Global,
                    default_visible: true,
                    points: vec![point(179.5), point(-179.5)],
                }],
                intensity: Vec::new(),
                fetched_at_unix: 0,
            },
            TEST_NOW,
        );
        let tracks = project_tracks(&store, 20.0, 179.0);
        let gap = (tracks[0].km[1] - tracks[0].km[0]).length();
        assert!(
            gap < 200.0,
            "1 degree of longitude, not a 360-degree wrap: {gap} km"
        );
    }

    #[test]
    fn wind_category_thresholds_follow_saffir_simpson() {
        let cases = [
            (33, WindCategory::Depression),
            (34, WindCategory::Storm),
            (63, WindCategory::Storm),
            (64, WindCategory::Cat1),
            (82, WindCategory::Cat1),
            (83, WindCategory::Cat2),
            (95, WindCategory::Cat2),
            (96, WindCategory::Cat3),
            (112, WindCategory::Cat3),
            (113, WindCategory::Cat4),
            (136, WindCategory::Cat4),
            (137, WindCategory::Cat5),
        ];
        for (kt, cat) in cases {
            assert_eq!(WindCategory::from_kt(Some(kt)), cat, "{kt} kt");
        }
        assert_eq!(WindCategory::from_kt(None), WindCategory::Unknown);
    }

    #[test]
    fn legend_colors_are_distinct() {
        let mut colors: Vec<u32> = WindCategory::LEGEND.iter().map(|c| c.hex()).collect();
        colors.sort_unstable();
        colors.dedup();
        assert_eq!(colors.len(), 8);
        assert_eq!(WindCategory::Cat5.hex(), 0xFF6060);
    }

    #[test]
    fn segment_color_uses_start_wind_then_end() {
        let start = segment_rgba(ModelGroup::Global, Some(100), Some(20));
        assert_eq!(&start[..3], &WindCategory::Cat3.rgb());
        let fallback = segment_rgba(ModelGroup::Global, None, Some(40));
        assert_eq!(&fallback[..3], &WindCategory::Storm.rgb());
        let unknown = segment_rgba(ModelGroup::Consensus, None, None);
        assert_eq!(&unknown[..3], &WindCategory::Unknown.rgb());
    }

    #[test]
    fn observed_track_is_grey_and_official_is_opaque() {
        assert_eq!(
            segment_rgba(ModelGroup::Observed, Some(140), Some(140)),
            [0xB0, 0xB0, 0xB0, 200]
        );
        assert_eq!(segment_rgba(ModelGroup::Official, Some(50), None)[3], 255);
        assert!(segment_rgba(ModelGroup::Hurricane, Some(50), None)[3] < 255);
    }

    #[test]
    fn official_track_draws_last_and_widest() {
        let ofcl = track_style(ModelGroup::Official);
        let carq = track_style(ModelGroup::Observed);
        let model = track_style(ModelGroup::Hurricane);
        assert!(ofcl.draw_rank > carq.draw_rank && carq.draw_rank > model.draw_rank);
        assert!(ofcl.width_px > model.width_px);
        assert!(ofcl.underlay_px.is_some() && model.underlay_px.is_none());
        assert!(!carq.label && model.label && ofcl.label);
    }

    // ── S11: per-model-group dash patterns ──────────────────────────────

    #[test]
    fn official_and_observed_are_solid() {
        assert_eq!(track_style(ModelGroup::Official).dash, None);
        assert_eq!(track_style(ModelGroup::Observed).dash, None);
    }

    #[test]
    fn default_visible_model_groups_have_distinct_dash_patterns() {
        let groups = [
            ModelGroup::Consensus,
            ModelGroup::Hurricane,
            ModelGroup::Global,
            ModelGroup::Ensemble,
            ModelGroup::Ai,
        ];
        let patterns: Vec<DashPattern> = groups
            .iter()
            .map(|&g| {
                track_style(g)
                    .dash
                    .expect("every default-visible model group has a dash")
            })
            .collect();
        let mut dedup = patterns.clone();
        dedup.dedup();
        // No exact duplicate, and sorting shouldn't collapse any pair either
        // (dedup() only removes *adjacent* equal elements).
        let mut sorted: Vec<(u32, u32)> = patterns
            .iter()
            .map(|p| (p.on_px.to_bits(), p.off_px.to_bits()))
            .collect();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            groups.len(),
            "every group's pattern must differ"
        );
    }

    #[test]
    fn model_group_legend_order_matches_the_groups_with_labels() {
        for group in MODEL_GROUP_LEGEND_ORDER {
            assert!(
                model_group_legend_label(group).is_some(),
                "{group:?} is in the legend order but has no label"
            );
            assert!(track_style(group).dash.is_some());
        }
        assert_eq!(model_group_legend_label(ModelGroup::Official), None);
        assert_eq!(model_group_legend_label(ModelGroup::Observed), None);
    }

    // `dash_steps`: the core screen-space dash-stepping math (review S11
    // item 2). TDD per the task's instructions — written before wiring the
    // real draw calls in `scope.rs`.

    fn pattern() -> DashPattern {
        DashPattern {
            on_px: 10.0,
            off_px: 5.0,
        }
    }

    #[test]
    fn dash_steps_zero_length_segment_draws_nothing_and_keeps_phase() {
        let (spans, phase) = dash_steps(pattern(), 3.0, 0.0);
        assert!(spans.is_empty());
        assert_eq!(phase, 3.0, "nothing moved, so phase must not change");
    }

    #[test]
    fn dash_steps_negative_length_segment_draws_nothing_and_keeps_phase() {
        let (spans, phase) = dash_steps(pattern(), 3.0, -5.0);
        assert!(spans.is_empty());
        assert_eq!(phase, 3.0);
    }

    #[test]
    fn dash_steps_segment_shorter_than_one_dash_stays_entirely_on() {
        // Starting at phase 0 (mid-dash), a 4px segment never reaches the
        // 10px on/off boundary.
        let (spans, phase) = dash_steps(pattern(), 0.0, 4.0);
        assert_eq!(
            spans,
            vec![DashSpan {
                start: 0.0,
                end: 4.0
            }]
        );
        assert_eq!(phase, 4.0);
    }

    #[test]
    fn dash_steps_segment_shorter_than_one_gap_stays_entirely_off() {
        // Starting at phase 10 (exactly at the on/off boundary, i.e. the
        // start of a gap), a short segment never re-enters "on".
        let (spans, phase) = dash_steps(pattern(), 10.0, 3.0);
        assert!(spans.is_empty(), "entirely inside the gap");
        assert_eq!(phase, 13.0);
    }

    #[test]
    fn dash_steps_segment_spanning_many_cycles_produces_one_span_per_dash() {
        // cycle = 15px; a 47px segment starting fresh covers 3 full cycles
        // (30px) plus a 17px remainder: on(10) off(5) on(10) off(5)
        // on(10) off(5) on(2) -- i.e. 4 "on" spans, the last one partial.
        let (spans, phase) = dash_steps(pattern(), 0.0, 47.0);
        assert_eq!(
            spans,
            vec![
                DashSpan {
                    start: 0.0,
                    end: 10.0
                },
                DashSpan {
                    start: 15.0,
                    end: 25.0
                },
                DashSpan {
                    start: 30.0,
                    end: 40.0
                },
                DashSpan {
                    start: 45.0,
                    end: 47.0
                },
            ]
        );
        // 47 % 15 = 2, i.e. 2px into the next "on" run.
        assert_eq!(phase, 2.0);
    }

    #[test]
    fn dash_steps_phase_continues_across_segment_boundaries() {
        // Two consecutive 8px segments of the same track, fed through in
        // sequence, must behave exactly like one 16px segment: the ending
        // phase of segment N is segment N+1's starting phase (review S11:
        // "the pattern doesn't visibly reset at each vertex").
        let (first_spans, phase_after_first) = dash_steps(pattern(), 0.0, 8.0);
        let (second_spans, phase_after_second) = dash_steps(pattern(), phase_after_first, 8.0);
        let (whole_spans, phase_whole) = dash_steps(pattern(), 0.0, 16.0);

        assert_eq!(
            first_spans,
            vec![DashSpan {
                start: 0.0,
                end: 8.0
            }]
        );
        assert_eq!(phase_after_first, 8.0);
        // Second segment: phase starts at 8 (still "on", on_px=10): 2px
        // finishes the dash, then the 5px gap, then 1px re-enters the next
        // dash -- all within this one 8px segment.
        assert_eq!(
            second_spans,
            vec![
                DashSpan {
                    start: 0.0,
                    end: 2.0
                },
                DashSpan {
                    start: 7.0,
                    end: 8.0
                },
            ]
        );
        assert_eq!(phase_after_second, phase_whole, "matches one 16px segment");
        assert_eq!(
            whole_spans,
            vec![
                DashSpan {
                    start: 0.0,
                    end: 10.0
                },
                DashSpan {
                    start: 15.0,
                    end: 16.0
                },
            ],
            "one continuous dash/gap/dash spanning both segments, as a single 16px segment sees it"
        );
        // The total drawn ("on") length must match regardless of how the
        // 16px is split into segments -- that's the real continuity
        // guarantee, independent of either side's local coordinate frames.
        let on_len = |spans: &[DashSpan]| -> f32 { spans.iter().map(|s| s.end - s.start).sum() };
        assert_eq!(
            on_len(&first_spans) + on_len(&second_spans),
            on_len(&whole_spans)
        );
    }

    #[test]
    fn dash_steps_wraps_an_out_of_range_starting_phase() {
        // A starting phase larger than one cycle (15px) must behave as if
        // it had been wrapped first -- callers should never need to
        // pre-normalize.
        let (spans, phase) = dash_steps(pattern(), 15.0 + 0.0, 4.0);
        let (spans_zero, phase_zero) = dash_steps(pattern(), 0.0, 4.0);
        assert_eq!(spans, spans_zero);
        assert_eq!(phase, phase_zero);
    }

    #[test]
    fn dash_steps_degenerate_pattern_draws_nothing() {
        let zero_on = DashPattern {
            on_px: 0.0,
            off_px: 5.0,
        };
        let (spans, phase) = dash_steps(zero_on, 2.0, 20.0);
        assert!(spans.is_empty());
        assert_eq!(phase, 0.0);

        let negative_cycle = DashPattern {
            on_px: -1.0,
            off_px: -1.0,
        };
        let (spans2, phase2) = dash_steps(negative_cycle, 2.0, 20.0);
        assert!(spans2.is_empty());
        assert_eq!(phase2, 0.0);
    }

    #[test]
    fn dash_steps_rejects_non_finite_length_without_hanging() {
        // Review N17: a non-finite length_px must not spin the loop. Both
        // NaN and +/-infinity fail `is_finite()`, so both take the early
        // return — this test completing at all is the assertion.
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let (spans, phase) = dash_steps(pattern(), 3.0, bad);
            assert!(spans.is_empty());
            assert_eq!(phase, 3.0, "phase passes through unchanged");
        }
    }

    #[test]
    fn dash_steps_caps_a_huge_finite_length_instead_of_hanging() {
        // A length well past MAX_DASH_STEPS_LENGTH_PX must still return
        // promptly with a bounded span count (review N17) — reachable
        // only as defense in depth, since scope::draw_model_tracks clips
        // to the viewport before ever calling dash_steps (review N1).
        let (spans, _) = dash_steps(pattern(), 0.0, 1.0e9);
        assert!(
            (spans.len() as f32) < MAX_DASH_STEPS_LENGTH_PX / pattern().cycle_px() + 1.0,
            "span count must stay bounded by the length cap, got {}",
            spans.len()
        );
    }

    #[test]
    fn advance_dash_phase_matches_dash_steps_final_phase() {
        // advance_dash_phase is the O(1) shortcut for skipping a distance
        // without enumerating spans; it must agree with dash_steps's own
        // final phase for the same (pattern, start phase, distance).
        let (_, want) = dash_steps(pattern(), 2.0, 37.0);
        assert_eq!(advance_dash_phase(pattern(), 2.0, 37.0), want);
    }

    #[test]
    fn advance_dash_phase_matches_degenerate_pattern_convention() {
        let degenerate = DashPattern {
            on_px: 0.0,
            off_px: 5.0,
        };
        assert_eq!(advance_dash_phase(degenerate, 2.0, 37.0), 0.0);
        assert_eq!(advance_dash_phase(pattern(), 2.0, f32::NAN), 0.0);
    }

    #[test]
    fn cap_dash_spans_passes_through_under_the_limit() {
        let spans = vec![DashSpan {
            start: 0.0,
            end: 1.0,
        }];
        assert_eq!(cap_dash_spans(spans.clone(), 100.0), spans);
    }

    #[test]
    fn cap_dash_spans_collapses_to_one_solid_span_over_the_limit() {
        let spans: Vec<DashSpan> = (0..MAX_DASH_SPANS_PER_SEGMENT + 1)
            .map(|i| DashSpan {
                start: i as f32,
                end: i as f32 + 0.5,
            })
            .collect();
        let capped = cap_dash_spans(spans, 250.0);
        assert_eq!(
            capped,
            vec![DashSpan {
                start: 0.0,
                end: 250.0
            }]
        );
    }

    #[test]
    fn clip_segment_to_rect_keeps_a_fully_inside_segment_whole() {
        assert_eq!(
            clip_segment_to_rect(10.0, 10.0, 20.0, 10.0, 0.0, 0.0, 100.0, 100.0),
            Some((0.0, 1.0))
        );
    }

    #[test]
    fn clip_segment_to_rect_rejects_a_fully_outside_segment() {
        assert_eq!(
            clip_segment_to_rect(200.0, 200.0, 300.0, 200.0, 0.0, 0.0, 100.0, 100.0),
            None
        );
    }

    #[test]
    fn clip_segment_to_rect_trims_a_segment_that_exits_the_rect() {
        // Horizontal segment from x=50 (inside) to x=200 (outside); the
        // rect's right edge is at x=100, so only the first half is kept.
        let (t0, t1) =
            clip_segment_to_rect(50.0, 50.0, 200.0, 50.0, 0.0, 0.0, 100.0, 100.0).unwrap();
        assert_eq!(t0, 0.0);
        assert!((t1 - (50.0 / 150.0)).abs() < 1e-5, "t1={t1}");
    }

    #[test]
    fn clip_segment_to_rect_trims_a_diagonal_segment_from_both_ends() {
        // Diagonal from well above-left of the rect to well below-right of
        // it, passing straight through the middle.
        let (t0, t1) =
            clip_segment_to_rect(-100.0, -100.0, 200.0, 200.0, 0.0, 0.0, 100.0, 100.0).unwrap();
        assert!(t0 > 0.0 && t0 < 0.5, "t0={t0}");
        assert!(t1 > 0.5 && t1 < 1.0, "t1={t1}");
    }

    #[test]
    fn clip_segment_to_rect_bounds_dash_work_for_a_track_far_off_screen_at_high_zoom() {
        // Review N1's actual failure scenario: a model track segment that,
        // at high zoom, spans a million screen pixels — almost all of it
        // off-screen. Clipping first must bound the work regardless of
        // how long the real segment is.
        let full_len = 1_000_000.0_f32;
        let rect = (-50.0, -50.0, 1970.0, 1130.0); // 1920x1080 viewport + 50px margin
        let (t0, t1) =
            clip_segment_to_rect(0.0, 0.0, full_len, 0.0, rect.0, rect.1, rect.2, rect.3).unwrap();
        let clipped_len = (t1 - t0) * full_len;
        assert!(
            clipped_len < 2100.0,
            "clipped length must stay near the viewport size, got {clipped_len}"
        );
        // Global's pattern has the smallest cycle (1.5 on / 4 off = 5.5px),
        // the worst case for span count.
        let global = track_style(ModelGroup::Global).dash.unwrap();
        let (spans, _) = dash_steps(global, 0.0, clipped_len);
        assert!(
            spans.len() < 1000,
            "span count for a clipped segment must stay small, got {}",
            spans.len()
        );
    }

    #[test]
    fn dash_steps_many_short_segments_sum_to_one_long_segment() {
        // Simulates a real polyline: several short legs back-to-back must
        // produce the same total "on" length as one long leg of the same
        // overall distance, with phase carried through every step.
        let total = 123.0_f32;
        let leg = 7.0_f32;
        let mut phase = 0.0_f32;
        let mut on_total = 0.0_f32;
        let mut walked = 0.0_f32;
        while walked < total {
            let len = leg.min(total - walked);
            let (spans, next_phase) = dash_steps(pattern(), phase, len);
            on_total += spans.iter().map(|s| s.end - s.start).sum::<f32>();
            phase = next_phase;
            walked += len;
        }
        let (whole_spans, _) = dash_steps(pattern(), 0.0, total);
        let whole_on: f32 = whole_spans.iter().map(|s| s.end - s.start).sum();
        assert!(
            (on_total - whole_on).abs() < 1e-3,
            "segmented total {on_total} vs whole-track total {whole_on}"
        );
    }

    // `dash_preview_segments`: the legend swatch preview tiling (review S11
    // item 3).

    #[test]
    fn dash_preview_segments_solid_is_one_on_segment() {
        assert_eq!(dash_preview_segments(None, 24.0), vec![(24.0, true)]);
    }

    #[test]
    fn dash_preview_segments_degenerate_pattern_falls_back_to_solid() {
        let zero_on = DashPattern {
            on_px: 0.0,
            off_px: 5.0,
        };
        assert_eq!(
            dash_preview_segments(Some(zero_on), 24.0),
            vec![(24.0, true)]
        );
    }

    #[test]
    fn dash_preview_segments_tiles_and_clips_the_last_segment() {
        let p = DashPattern {
            on_px: 10.0,
            off_px: 5.0,
        };
        // 24px total: on(10) off(5) on(9, clipped from 10).
        assert_eq!(
            dash_preview_segments(Some(p), 24.0),
            vec![(10.0, true), (5.0, false), (9.0, true)]
        );
    }

    #[test]
    fn dash_preview_segments_zero_width_is_empty() {
        let p = DashPattern {
            on_px: 10.0,
            off_px: 5.0,
        };
        assert_eq!(
            dash_preview_segments(Some(p), 0.0),
            Vec::<(f32, bool)>::new()
        );
    }

    // ── S4: OFCL outline / GIS advisory colour / legend text ────────────

    #[test]
    fn rgb_hex_packs_channels() {
        assert_eq!(rgb_hex([0xB2, 0x66, 0xFF]), 0xB266FF);
        assert_eq!(rgb_hex([0, 0, 0]), 0);
        assert_eq!(rgb_hex([0xFF, 0xFF, 0xFF]), 0xFFFFFF);
    }

    #[test]
    fn official_outline_is_dark_not_white() {
        // Review S4: a white outline erased OFCL's own Cat 1/2 colours. A
        // dark outline contrasts with every colour in the wind palette.
        assert_eq!(OFFICIAL_OUTLINE_RGB, [0x10, 0x10, 0x10]);
        assert_eq!(official_outline_hex(), 0x101010);
    }

    #[test]
    fn official_outline_legend_variant_is_light_not_dark() {
        // Fix-round finding: a dark outline is invisible against the
        // legend panel's own dark background, unlike against the map.
        assert_eq!(OFFICIAL_OUTLINE_LEGEND_RGB, [0xCF, 0xCF, 0xCF]);
        assert_eq!(official_outline_legend_hex(), 0xCFCFCF);
        assert_ne!(OFFICIAL_OUTLINE_LEGEND_RGB, OFFICIAL_OUTLINE_RGB);
    }

    #[test]
    fn official_swatch_segments_are_distinct_wind_categories() {
        let hexes: Vec<u32> = OFFICIAL_SWATCH_SEGMENTS.iter().map(|c| c.hex()).collect();
        let mut sorted = hexes.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), hexes.len(), "segments must all differ");
    }

    #[test]
    fn gis_advisory_color_is_off_the_saffir_simpson_palette() {
        let gis_hex = rgb_hex(GIS_ADVISORY_RGB);
        for cat in WindCategory::LEGEND {
            assert_ne!(
                gis_hex,
                cat.hex(),
                "GIS advisory colour must not collide with a wind category"
            );
        }
        // Must also differ from the old pre-branch colour it replaces
        // (`#FFCC66`), which review S4 flagged as visually close to Cat 3.
        assert_ne!(gis_hex, 0xFFCC66);
    }

    #[test]
    fn official_legend_label_covers_all_three_states() {
        assert_eq!(
            official_legend_label(OfficialLegendState::Drawn),
            "OFCL  NHC official"
        );
        assert_eq!(
            official_legend_label(OfficialLegendState::SuppressedForGis),
            "OFCL shown as advisory track"
        );
        assert_eq!(
            official_legend_label(OfficialLegendState::NotIssued),
            "No OFCL this cycle"
        );
    }

    #[test]
    fn official_legend_state_prefers_drawn_over_suppressed() {
        assert_eq!(
            official_legend_state(true, true),
            OfficialLegendState::Drawn
        );
        assert_eq!(
            official_legend_state(true, false),
            OfficialLegendState::Drawn
        );
    }

    #[test]
    fn official_legend_state_distinguishes_suppressed_from_not_issued() {
        // Review fix-round 2, reopened Important 4: these two must read
        // differently — "suppressed" means OFCL exists but the GIS
        // advisory track is shown instead (not a false "No OFCL" claim).
        assert_eq!(
            official_legend_state(false, true),
            OfficialLegendState::SuppressedForGis
        );
        assert_eq!(
            official_legend_state(false, false),
            OfficialLegendState::NotIssued
        );
    }

    #[test]
    fn official_legend_labels_fit_the_legend_panel() {
        // Legend content width is WIDTH (196) minus 8px padding each side =
        // 180px; same ~5.5 px/char density `disclaimer_lines_fit_the_panel`
        // uses below, giving a budget of about 32 chars.
        for state in [
            OfficialLegendState::Drawn,
            OfficialLegendState::SuppressedForGis,
            OfficialLegendState::NotIssued,
        ] {
            let label = official_legend_label(state);
            assert!(
                label.chars().count() <= 32,
                "{label:?} too long for the legend"
            );
        }
    }

    #[test]
    fn stale_suffix_appends_only_when_stale() {
        assert_eq!(
            stale_suffix("12Z Oct 9 cycle, 3 h ago", false),
            "12Z Oct 9 cycle, 3 h ago"
        );
        assert_eq!(
            stale_suffix("12Z Oct 9 cycle, 3 h ago", true),
            "12Z Oct 9 cycle, 3 h ago (stale)"
        );
    }

    fn test_meta(id: &str, name: &str, bin_number: &str) -> crate::nhc::StormMeta {
        crate::nhc::StormMeta {
            id: id.to_string(),
            name: name.to_string(),
            classification: String::new(),
            intensity_kt: 0,
            pressure_mb: 0,
            lat: 0.0,
            lon: 0.0,
            movement_dir_deg: None,
            movement_speed_kt: None,
            last_update: String::new(),
            bin_number: bin_number.to_string(),
            advisory_num: String::new(),
            graphics_url: String::new(),
            graphics_issuance: String::new(),
            earliest_arrival_kmz: String::new(),
            most_likely_arrival_kmz: String::new(),
            wind_probs_34kt_kmz: String::new(),
            wind_probs_50kt_kmz: String::new(),
            wind_probs_64kt_kmz: String::new(),
        }
    }

    /// `has_track = false` models a GIS entry whose cone data arrived but
    /// whose track layer is still empty (review fix-round Important 2).
    fn test_gis(name: &str, basin: &str, has_track: bool) -> crate::nhc::StormGis {
        crate::nhc::StormGis {
            name: name.to_string(),
            storm_type: String::new(),
            basin: basin.to_string(),
            advisory_num: String::new(),
            advisory_date: String::new(),
            cone: Vec::new(),
            track: if has_track {
                vec![vec![(10.0, -80.0), (11.0, -81.0)]]
            } else {
                Vec::new()
            },
            points: Vec::new(),
            watches_warnings: Vec::new(),
        }
    }

    #[test]
    fn gis_track_shown_for_matches_exact_bare_name_case_insensitively() {
        // Real `CurrentStorms.json` / GIS cone-layer names are bare
        // ("Fausto"), not the points-layer's "Tropical Storm Fausto" style
        // (review fix-round Important 3: the old test used an unrealistic
        // input that never actually occurs for this comparison). East
        // Pacific uses the same two-letter code on both sides, so this one
        // doesn't need the Atlantic AL/AT alias — that gets its own test
        // below.
        let metas = [test_meta("ep092026", "Fausto", "EP9")];
        let gis = [test_gis("FAUSTO", "EP", true)];
        assert!(gis_track_shown_for("ep092026", &metas, &gis, true));
        // Storm id is matched case-insensitively too.
        assert!(gis_track_shown_for("EP092026", &metas, &gis, true));
    }

    #[test]
    fn gis_track_shown_for_matches_atlantic_storm_despite_al_at_basin_code_mismatch() {
        // Review fix-round 2, reopened Important 1: `CurrentStorms.json`
        // prefixes the Atlantic `bin_number` "AT" (e.g. "AT9"), but NHC's
        // live ArcGIS MapServer returns `"AL"` for `StormGis::basin` —
        // confirmed by the reviewer against the real feed and an archived
        // Atlantic shapefile (Hurricane Milton, al142024). Round 1's
        // `basin_matches` compared these with no aliasing and so never
        // matched a single real Atlantic storm; this is the regression
        // test that would have caught that.
        let metas = [test_meta("al142026", "Milton", "AT14")];
        let gis = [test_gis("Milton", "AL", true)];
        assert!(gis_track_shown_for("al142026", &metas, &gis, true));
    }

    #[test]
    fn gis_track_shown_for_exact_match_rejects_substring_false_positives() {
        // Review fix-round Important 1: depression names are often plain
        // numbers, so a substring rule would wrongly match "Four" inside
        // "Fourteen-E". The exact-match rule must not suppress here.
        // Realistic Atlantic basin codes on both sides (see the AL/AT test
        // above): "AT4" from CurrentStorms, "AL" from the live GIS feed.
        let metas = [test_meta("al042026", "Four", "AT4")];
        let gis = [test_gis("Fourteen-E", "AL", true)];
        assert!(!gis_track_shown_for("al042026", &metas, &gis, true));
    }

    #[test]
    fn gis_track_shown_for_false_when_basin_differs() {
        // Same exact name, genuinely different basin — must not match
        // (review fix-round Important 1: a name-only rule is basin-blind).
        // "EP" isn't aliased to anything, so this stays a real mismatch
        // even after the AL/AT fix.
        let metas = [test_meta("al142026", "Milton", "AT14")];
        let gis = [test_gis("Milton", "EP", true)];
        assert!(!gis_track_shown_for("al142026", &metas, &gis, true));
    }

    #[test]
    fn gis_track_shown_for_false_when_storm_unknown() {
        let metas = [test_meta("al142026", "Milton", "AT14")];
        let gis = [test_gis("Milton", "AL", true)];
        assert!(!gis_track_shown_for("ep092026", &metas, &gis, true));
    }

    #[test]
    fn gis_track_shown_for_false_when_no_gis_match() {
        let metas = [test_meta("al142026", "Milton", "AT14")];
        let gis = [test_gis("Helene", "AL", true)];
        assert!(!gis_track_shown_for("al142026", &metas, &gis, true));
    }

    #[test]
    fn gis_track_shown_for_false_with_no_gis_storms() {
        let metas = [test_meta("al142026", "Milton", "AT14")];
        assert!(!gis_track_shown_for("al142026", &metas, &[], true));
    }

    #[test]
    fn gis_track_shown_for_false_when_gis_track_layer_is_empty() {
        // Review fix-round Important 2: cone data without a track means
        // `draw_nhc_overlays` draws nothing for the track; suppressing
        // OFCL here would leave nothing official on screen at all.
        let metas = [test_meta("al142026", "Milton", "AT14")];
        let gis = [test_gis("Milton", "AL", false)];
        assert!(!gis_track_shown_for("al142026", &metas, &gis, true));
    }

    #[test]
    fn gis_track_shown_for_false_when_track_layer_toggled_off() {
        // Review fix-round Important 2: `draw_nhc_overlays` only draws the
        // track when `overlays.show_track` is on.
        let metas = [test_meta("al142026", "Milton", "AT14")];
        let gis = [test_gis("Milton", "AL", true)];
        assert!(!gis_track_shown_for("al142026", &metas, &gis, false));
    }

    #[test]
    fn gis_track_shown_for_is_independent_per_storm_in_a_mixed_frame() {
        // Review fix-round Important 3(c): one storm has a matching GIS
        // entry, a second doesn't — each storm's suppression decision must
        // be independently correct. Basin-consistent names on each side:
        // Milton (Atlantic) and Hector (East Pacific) are both real storm
        // names for their respective basins.
        let metas = [
            test_meta("al142026", "Milton", "AT14"),
            test_meta("ep082026", "Hector", "EP8"),
        ];
        let gis = [test_gis("Milton", "AL", true)];
        assert!(gis_track_shown_for("al142026", &metas, &gis, true));
        assert!(!gis_track_shown_for("ep082026", &metas, &gis, true));
    }

    #[test]
    fn project_tracks_carries_the_storm_id() {
        let tracks = project_tracks(&fixture_store(), 27.0, -87.6);
        assert!(tracks.iter().all(|t| t.storm_id == "al092026"));
    }

    #[test]
    fn any_official_track_shown_true_when_ofcl_exists_and_is_not_suppressed() {
        let store = fixture_store(); // has an al092026 OFCL track (see project_tracks_orders_official_last)
        assert!(any_official_track_shown(&store, &[], &[], true));
    }

    #[test]
    fn any_official_track_shown_false_when_the_only_ofcl_is_gis_suppressed() {
        let store = fixture_store(); // al092026
        let metas = [test_meta("al092026", "Jerry", "AT9")];
        let gis = [test_gis("Jerry", "AL", true)];
        assert!(!any_official_track_shown(&store, &metas, &gis, true));
    }

    #[test]
    fn any_official_track_shown_false_for_an_empty_store() {
        assert!(!any_official_track_shown(
            &GuidanceStore::default(),
            &[],
            &[],
            true
        ));
    }

    #[test]
    fn any_official_suppressed_by_gis_true_when_the_only_ofcl_is_suppressed() {
        // Mirror image of any_official_track_shown_false_when_the_only_
        // ofcl_is_gis_suppressed: the same inputs must report "suppressed",
        // not just "not drawn" (review fix-round 2, reopened Important 4).
        let store = fixture_store(); // al092026
        let metas = [test_meta("al092026", "Jerry", "AT9")];
        let gis = [test_gis("Jerry", "AL", true)];
        assert!(any_official_suppressed_by_gis(&store, &metas, &gis, true));
    }

    #[test]
    fn any_official_suppressed_by_gis_false_when_ofcl_is_drawn_normally() {
        let store = fixture_store();
        assert!(!any_official_suppressed_by_gis(&store, &[], &[], true));
    }

    #[test]
    fn any_official_suppressed_by_gis_false_for_an_empty_store() {
        assert!(!any_official_suppressed_by_gis(
            &GuidanceStore::default(),
            &[],
            &[],
            true
        ));
    }

    #[test]
    fn basin_matches_aliases_atlantic_at_and_al() {
        assert!(basin_matches("AT9", "AL"));
        assert!(basin_matches("AT9", "at")); // case-insensitive
        assert!(!basin_matches("AT9", "EP"));
        assert!(!basin_matches("EP5", "AL"));
    }

    #[test]
    fn basin_matches_ep_and_cp_need_no_alias() {
        assert!(basin_matches("EP5", "EP"));
        assert!(basin_matches("CP1", "CP"));
        assert!(!basin_matches("EP5", "CP"));
    }

    #[test]
    fn disclaimer_lines_fit_the_panel() {
        // NHC panel content is ~288 px; 10 px Inter is ~5.5 px/char.
        for line in DISCLAIMER_LINES {
            assert!(line.chars().count() <= 48, "{line:?} too long");
        }
        assert!(
            DISCLAIMER_LINES
                .iter()
                .any(|l| l.contains("not official forecasts"))
        );
    }

    #[test]
    fn format_cycle_reads_init_digits() {
        assert_eq!(format_cycle(2026100912), "12Z Oct 9");
        assert_eq!(format_cycle(2026010100), "00Z Jan 1");
        // Garbage never panics.
        assert_eq!(format_cycle(0), "00Z ??? 0");
        assert_eq!(format_cycle(2026130100), "00Z ??? 1");
    }

    #[test]
    fn format_age_buckets() {
        assert_eq!(format_age(1000, 1000), "just now");
        assert_eq!(format_age(1000, 1059), "just now");
        assert_eq!(format_age(1000, 1060), "1 min ago");
        assert_eq!(format_age(1000, 4599), "59 min ago");
        assert_eq!(format_age(1000, 4600), "1 h 0 min ago");
        assert_eq!(format_age(1000, 900), "just now", "clock skew clamps to 0");
    }

    #[test]
    fn is_stale_boundary() {
        assert!(!is_stale(0, STALE_AFTER_SECS));
        assert!(is_stale(0, STALE_AFTER_SECS + 1));
    }

    #[test]
    fn is_stale_cycle_boundary() {
        assert!(!is_stale_cycle(0, CYCLE_STALE_AFTER_SECS));
        assert!(is_stale_cycle(0, CYCLE_STALE_AFTER_SECS + 1));
        // A fresh-ish cycle (well under 12 h) must not read as stale, unlike
        // feeding the same age into the 30-min fetch threshold.
        assert!(!is_stale_cycle(0, 6 * 60 * 60));
    }

    #[test]
    fn cycle_alpha_mult_is_full_strength_under_the_dim_threshold() {
        assert_eq!(cycle_alpha_mult(0), 1.0);
        assert_eq!(cycle_alpha_mult(CYCLE_DIM_AFTER_SECS), 1.0);
        assert_eq!(cycle_alpha_mult(-10), 1.0, "clock skew doesn't dim");
    }

    #[test]
    fn cycle_alpha_mult_fades_past_the_dim_threshold_without_hiding() {
        let just_over = cycle_alpha_mult(CYCLE_DIM_AFTER_SECS + 1);
        assert!(just_over < 1.0 && just_over > 0.35);
        let at_double = cycle_alpha_mult(CYCLE_DIM_AFTER_SECS * 2);
        assert!((at_double - 0.35).abs() < 1e-6);
        let way_past = cycle_alpha_mult(CYCLE_DIM_AFTER_SECS * 10);
        assert!(
            (way_past - 0.35).abs() < 1e-6,
            "floor, never fully hidden: {way_past}"
        );
    }

    #[test]
    fn cycle_freshness_line_single_fresh_cycle() {
        let now = crate::adeck::cycle_time_unix(2026100912).unwrap();
        let (text, stale) = cycle_freshness_line(2026100912, false, now);
        assert_eq!(text, "12Z Oct 9 cycle, just now");
        assert!(!stale);
    }

    #[test]
    fn cycle_freshness_line_single_stale_cycle() {
        let cycle = crate::adeck::cycle_time_unix(2026100912).unwrap();
        let now = cycle + CYCLE_STALE_AFTER_SECS + 1;
        let (text, stale) = cycle_freshness_line(2026100912, false, now);
        assert!(text.starts_with("12Z Oct 9 cycle, "), "{text:?}");
        assert!(stale);
    }

    #[test]
    fn cycle_freshness_line_invalid_init_has_no_age_and_is_never_stale() {
        let (text, stale) = cycle_freshness_line(2026130100, false, 0);
        assert_eq!(text, "00Z ??? 1 cycle");
        assert!(!stale);
    }

    #[test]
    fn cycle_freshness_line_mixed_still_surfaces_the_oldest_shown_cycles_age() {
        // Fix-round regression: "mixed cycles" must not suppress
        // staleness. The oldest shown storm's own age and staleness still
        // come through, just with the "Mixed cycles, oldest" prefix.
        let cycle = crate::adeck::cycle_time_unix(2026100806).unwrap();
        let now = cycle + CYCLE_STALE_AFTER_SECS + 1;
        let (text, stale) = cycle_freshness_line(2026100806, true, now);
        assert!(
            text.starts_with("Mixed cycles, oldest 06Z Oct 8 cycle, "),
            "{text:?}"
        );
        assert!(
            stale,
            "a genuinely stale oldest cycle must still read stale"
        );
    }

    #[test]
    fn cycle_freshness_line_mixed_fresh_is_not_stale() {
        let now = crate::adeck::cycle_time_unix(2026100912).unwrap();
        let (text, stale) = cycle_freshness_line(2026100912, true, now);
        assert_eq!(text, "Mixed cycles, oldest 12Z Oct 9 cycle, just now");
        assert!(!stale);
    }

    use crate::adeck::IntensitySeries;

    fn series(max_tau: i32) -> IntensitySeries {
        IntensitySeries {
            tech: "AVNI".into(),
            official: false,
            points: vec![(0, 50), (max_tau, 60)],
        }
    }

    #[test]
    fn chart_max_tau_rounds_up_to_a_day_and_clamps() {
        assert_eq!(chart_max_tau(&[]), 24);
        assert_eq!(chart_max_tau(&[series(0)]), 24);
        assert_eq!(chart_max_tau(&[series(60)]), 72);
        assert_eq!(chart_max_tau(&[series(72)]), 72);
        assert_eq!(chart_max_tau(&[series(200)]), 168);
    }

    #[test]
    fn chart_max_tau_does_not_panic_on_a_tau_near_i32_max() {
        // `adeck::parse_adeck_line` rejects this before it ever reaches an
        // `IntensitySeries`, but `chart_max_tau` is defense in depth for a
        // series built some other way. Plain `max + 23` panics on overflow
        // in debug/test builds here (review L6); this test runs under
        // `cargo test`'s debug profile, so it would catch a regression.
        assert_eq!(chart_max_tau(&[series(i32::MAX)]), 168);
    }

    #[test]
    fn chart_bands_are_contiguous_and_capped() {
        let bands = chart_bands();
        assert_eq!(bands.len(), 7);
        assert_eq!(bands[0].0, 0);
        assert_eq!(bands.last().unwrap().1, CHART_MAX_KT);
        for pair in bands.windows(2) {
            assert_eq!(pair[0].1, pair[1].0, "no gaps between categories");
        }
        assert_eq!(bands[2], (64, 83, WindCategory::Cat1.rgb()));
    }
}
