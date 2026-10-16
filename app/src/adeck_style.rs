//! Presentation rules for the spaghetti-models overlay: wind-category colours,
//! per-segment colour, line styles, the km-projection cache, cycle/age text,
//! the disclaimer, and intensity-chart scaling. Pure: no drawing calls.

use crate::adeck::{GuidanceStore, IntensitySeries, ModelGroup};
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
}

pub fn track_style(group: ModelGroup) -> TrackStyle {
    match group {
        ModelGroup::Official => TrackStyle {
            width_px: 2.5,
            underlay_px: Some(5.0),
            label: true,
            draw_rank: 2,
        },
        ModelGroup::Observed => TrackStyle {
            width_px: 1.5,
            underlay_px: None,
            label: false,
            draw_rank: 1,
        },
        _ => TrackStyle {
            width_px: 1.5,
            underlay_px: None,
            label: true,
            draw_rank: 0,
        },
    }
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

/// Legend text for the OFCL row. `any_official_drawn` should be computed by
/// `any_official_track_shown` below — i.e. this reflects what
/// `draw_model_tracks` actually puts on screen this frame (an OFCL track
/// exists for some shown storm, and isn't suppressed by
/// `gis_track_shown_for`), not merely `StormGuidance::has_official` (a
/// cycle-selection flag from review S8: whether OFCL made it into the
/// *selected cycle* at all). Those two can disagree — a storm can have
/// `has_official == false` while its live NHC advisory GIS track is drawn,
/// or have `has_official == true` yet have its OFCL suppressed by S4's GIS
/// overlap fix — so basing the legend on `has_official` would let it
/// contradict the map (review fix-round finding). The wording also avoids
/// "No official forecast", which would overclaim when a GIS advisory track
/// is in fact showing; "No OFCL this cycle" only claims what's true of the
/// a-deck OFCL specifically.
pub fn official_legend_label(any_official_drawn: bool) -> &'static str {
    if any_official_drawn {
        "OFCL  NHC official"
    } else {
        "No OFCL this cycle"
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

/// True when `bin_number` (`StormMeta`, e.g. `"AT2"`) and `gis_basin`
/// (`StormGis::basin`, e.g. `"AT"`) name the same ATCF basin. Required
/// alongside the name match in `gis_track_shown_for`: NHC basin-numbers
/// invests and short/common names from 1 within each basin, so a name
/// match alone can't rule out two different-basin storms.
fn basin_matches(bin_number: &str, gis_basin: &str) -> bool {
    let gis_basin = gis_basin.trim();
    !gis_basin.is_empty()
        && bin_number
            .trim()
            .to_ascii_uppercase()
            .starts_with(&gis_basin.to_ascii_uppercase())
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

/// True when at least one shown storm's OFCL track is actually drawn this
/// frame by `draw_model_tracks`: it has a default-visible, >=2-point
/// Official-group track, and `gis_track_shown_for` doesn't suppress it.
/// Mirrors that function's per-track condition independently (rather than
/// threading a return value back up through `draw_scope_to_texture`'s void
/// render call chain), so the legend's OFCL row can describe what the map
/// actually shows (review fix-round finding on `official_legend_label`)
/// without coupling the render path to the legend.
pub fn any_official_track_shown(
    store: &GuidanceStore,
    metas: &[crate::nhc::StormMeta],
    gis_storms: &[crate::nhc::StormGis],
    show_gis_track: bool,
) -> bool {
    store.iter().any(|g| {
        let has_ofcl_track = g
            .tracks
            .iter()
            .any(|t| t.group == ModelGroup::Official && t.default_visible && t.points.len() >= 2);
        has_ofcl_track && !gis_track_shown_for(&g.storm_id, metas, gis_storms, show_gis_track)
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
/// clamped to 24..=168 h.
pub fn chart_max_tau(series: &[IntensitySeries]) -> i32 {
    let max = series
        .iter()
        .flat_map(|s| s.points.iter().map(|p| p.0))
        .max()
        .unwrap_or(0);
    (((max + 23) / 24) * 24).clamp(24, 168)
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
    fn official_legend_label_reflects_whether_ofcl_is_drawn() {
        assert_eq!(official_legend_label(true), "OFCL  NHC official");
        assert_eq!(official_legend_label(false), "No OFCL this cycle");
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
        // input that never actually occurs for this comparison).
        let metas = [test_meta("al092026", "Fausto", "AT9")];
        let gis = [test_gis("FAUSTO", "AT", true)];
        assert!(gis_track_shown_for("al092026", &metas, &gis, true));
        // Storm id is matched case-insensitively too.
        assert!(gis_track_shown_for("AL092026", &metas, &gis, true));
    }

    #[test]
    fn gis_track_shown_for_exact_match_rejects_substring_false_positives() {
        // Review fix-round Important 1: depression names are often plain
        // numbers, so a substring rule would wrongly match "Four" inside
        // "Fourteen-E". The exact-match rule must not suppress here.
        let metas = [test_meta("al042026", "Four", "AT4")];
        let gis = [test_gis("Fourteen-E", "AT", true)];
        assert!(!gis_track_shown_for("al042026", &metas, &gis, true));
    }

    #[test]
    fn gis_track_shown_for_false_when_basin_differs() {
        // Same exact name, different basin — must not match (review
        // fix-round Important 1: a name-only rule is basin-blind).
        let metas = [test_meta("al092026", "Fausto", "AT9")];
        let gis = [test_gis("Fausto", "EP", true)];
        assert!(!gis_track_shown_for("al092026", &metas, &gis, true));
    }

    #[test]
    fn gis_track_shown_for_false_when_storm_unknown() {
        let metas = [test_meta("al092026", "Fausto", "AT9")];
        let gis = [test_gis("Fausto", "AT", true)];
        assert!(!gis_track_shown_for("ep182026", &metas, &gis, true));
    }

    #[test]
    fn gis_track_shown_for_false_when_no_gis_match() {
        let metas = [test_meta("al092026", "Fausto", "AT9")];
        let gis = [test_gis("Helene", "AT", true)];
        assert!(!gis_track_shown_for("al092026", &metas, &gis, true));
    }

    #[test]
    fn gis_track_shown_for_false_with_no_gis_storms() {
        let metas = [test_meta("al092026", "Fausto", "AT9")];
        assert!(!gis_track_shown_for("al092026", &metas, &[], true));
    }

    #[test]
    fn gis_track_shown_for_false_when_gis_track_layer_is_empty() {
        // Review fix-round Important 2: cone data without a track means
        // `draw_nhc_overlays` draws nothing for the track; suppressing
        // OFCL here would leave nothing official on screen at all.
        let metas = [test_meta("al092026", "Fausto", "AT9")];
        let gis = [test_gis("Fausto", "AT", false)];
        assert!(!gis_track_shown_for("al092026", &metas, &gis, true));
    }

    #[test]
    fn gis_track_shown_for_false_when_track_layer_toggled_off() {
        // Review fix-round Important 2: `draw_nhc_overlays` only draws the
        // track when `overlays.show_track` is on.
        let metas = [test_meta("al092026", "Fausto", "AT9")];
        let gis = [test_gis("Fausto", "AT", true)];
        assert!(!gis_track_shown_for("al092026", &metas, &gis, false));
    }

    #[test]
    fn gis_track_shown_for_is_independent_per_storm_in_a_mixed_frame() {
        // Review fix-round Important 3(c): one storm has a matching GIS
        // entry, a second doesn't — each storm's suppression decision must
        // be independently correct.
        let metas = [
            test_meta("al092026", "Fausto", "AT9"),
            test_meta("ep182026", "Helene", "EP8"),
        ];
        let gis = [test_gis("Fausto", "AT", true)];
        assert!(gis_track_shown_for("al092026", &metas, &gis, true));
        assert!(!gis_track_shown_for("ep182026", &metas, &gis, true));
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
        let store = fixture_store();
        let metas = [test_meta("al092026", "Fausto", "AT9")];
        let gis = [test_gis("Fausto", "AT", true)];
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
