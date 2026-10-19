//! Floating map legend for the spaghetti-models overlay: wind-category
//! swatches, line keys, data age, and the short "not an official forecast"
//! notice. Shown whenever model tracks are drawn.
//!
//! Review S10: the panel's height is sized from its own content
//! (`fit!()`), not a hand-counted constant, and its `y` offset is clamped so
//! it can never sit above the top bar or collide with the bottom status
//! bar even on a short or mobile window. `legend_mode` and `legend_y` are
//! the two pure, unit-tested decisions behind that; `draw` just follows
//! whichever `LegendMode` the caller (`main.rs`) picked.

use crate::adeck_style::{self, WindCategory};
use crate::widgets::{ChartWidget, SYMBOL_FONT, glass_panel, nf};
use ply_engine::prelude::*;

pub const WIDTH: f32 = 196.0;

/// Rough estimate of the full-mode panel's rendered height, used only to
/// compute where to anchor it from the bottom of the screen (`legend_y`) —
/// NOT as the panel's actual `.height()`, which is `fit!()` (content-sized)
/// so a wrong estimate here can only shift the panel a few px, never clip
/// or overflow it (review S10). Tune by eye if rows are added; doesn't need
/// to be exact.
pub const FULL_HEIGHT_ESTIMATE: f32 = 420.0;

/// Same estimate, for `LegendMode::Compact`'s much shorter content (review
/// S10 item 3): header, cycle/age freshness lines, the OFCL-state line, and
/// the two-line disclaimer — the safety-critical rows, nothing else.
pub const COMPACT_HEIGHT_ESTIMATE: f32 = 130.0;

/// Below this window height, or on the mobile layout, `legend_mode` returns
/// `Compact` (review S10 item 3). Derived from the SAME numbers `legend_y`
/// clamps with (`FULL_HEIGHT_ESTIMATE` plus the top-bar and bottom-margin
/// pixels), so the mode decision and the y-clamp can never disagree about
/// whether the Full legend actually fits — a flat, independently-chosen
/// constant here previously left windows roughly 400-490px tall in Full
/// mode even though the Full legend didn't fit them (review S10 fix-round
/// follow-up: the two decisions must share one source of truth).
pub fn compact_height_trigger_px(top_bar_px: f32, bottom_margin_px: f32) -> f32 {
    FULL_HEIGHT_ESTIMATE + top_bar_px + bottom_margin_px
}

const TEXT: u32 = 0xE8E0DC;
const MUTED: u32 = 0x9E9590;
const WARN: u32 = 0xFFC140;

/// Which legend layout to draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegendMode {
    /// Every row: wind-category swatches, OFCL/GIS/CARQ keys, the
    /// per-model-group dash-pattern key, freshness, and the full
    /// disclaimer/attribution block.
    Full,
    /// Only the safety-critical rows (review S10 item 3): the freshness
    /// lines from review S1 (cycle age / fetch age) and the OFCL-state text
    /// from review S4, plus the short "not an official forecast" warning.
    /// Everything else — wind-category swatches, the GIS/CARQ rows, the
    /// per-model dash key, and the source/consult lines — is dropped to fit
    /// a short or mobile window.
    Compact,
}

/// Picks `Full` or `Compact` from two independent window-shape signals
/// (review S10): a short window, one where the Full legend genuinely
/// wouldn't fit between the top and bottom bars
/// (`compact_height_trigger_px`), can't fit the full legend no matter the
/// platform; and the app's own mobile breakpoint (`is_mobile`, `main.rs`'s
/// `screen_width() < 900.0`, which already switches to larger touch
/// targets elsewhere) implies a layout where full-screen panels cover the
/// map and the legend has much less room to work with regardless of
/// height. Either alone is enough to trigger `Compact`. `top_bar_px` and
/// `bottom_margin_px` must be the same values passed to `legend_y`, so the
/// two can't disagree about whether Full actually fits.
pub fn legend_mode(
    screen_height_px: f32,
    is_mobile: bool,
    top_bar_px: f32,
    bottom_margin_px: f32,
) -> LegendMode {
    if is_mobile || screen_height_px < compact_height_trigger_px(top_bar_px, bottom_margin_px) {
        LegendMode::Compact
    } else {
        LegendMode::Full
    }
}

/// The legend's floating `y` offset, anchored above the bottom status bar
/// like before, but clamped so it can never sit above the top bar
/// (`top_bar_px`, 36px elsewhere in `main.rs`) or collide with the bottom
/// bar — the review S10 failure mode on a window shorter than about
/// 320-400px, where the old fixed-height calculation could go negative or
/// overlap both bars at once.
///
/// `content_h_estimate` only needs to be in the right ballpark (see
/// `FULL_HEIGHT_ESTIMATE` / `COMPACT_HEIGHT_ESTIMATE`): the panel's real
/// `.height(fit!())` means a mis-estimate shifts the panel's position by a
/// few px, it never truncates or overflows the panel itself.
pub fn legend_y(
    screen_height_px: f32,
    top_bar_px: f32,
    bottom_margin_px: f32,
    content_h_estimate: f32,
) -> f32 {
    let from_bottom = screen_height_px - bottom_margin_px - content_h_estimate;
    from_bottom.max(top_bar_px)
}

#[allow(clippy::too_many_arguments)]
pub fn draw(
    ui: &mut Ui<'_, ChartWidget>,
    x: f32,
    y: f32,
    mode: LegendMode,
    cycle_line: &str,
    cycle_stale: bool,
    age_line: &str,
    fetch_stale: bool,
    // Which of the three OFCL states applies this frame (review L1 +
    // fix-round Important 4; `adeck_style::OfficialLegendState` doc
    // explains why two states aren't enough). Callers compute this from
    // `adeck_style::official_legend_state(any_official_track_shown(...),
    // any_official_suppressed_by_gis(...))`, not from
    // `StormGuidance::has_official` (a cycle-selection flag from review
    // S8, which can disagree with what's actually drawn).
    ofcl_state: adeck_style::OfficialLegendState,
) {
    glass_panel::glass(
        ui.element()
            .id("model-legend")
            .width(fixed!(WIDTH))
            // Review S10: content-sized, not a hand-counted fixed height —
            // the old `HEIGHT` constant had to be re-tuned by eye every
            // time a row was added or removed, and a wrong guess either
            // clipped rows or left dead space. `legend_y` below only needs
            // a rough height *estimate* to anchor the panel from the
            // bottom, not an exact one, because `fit!()` means the panel
            // always sizes to its real content regardless.
            .height(fit!()),
    )
    .floating(|f| f.offset((x, y)).z_index(40).attach_root())
    .layout(|l| l.direction(TopToBottom).padding(8).gap(3))
    .children(|ui| {
        ui.element()
            .width(grow!())
            .height(fit!())
            .layout(|l| l.direction(LeftToRight).gap(6).align(Left, CenterY))
            .children(|ui| {
                ui.text(nf::HURRICANE, |t| {
                    t.font_size(12).font(&SYMBOL_FONT).color(0x0dc5b8)
                });
                ui.text("Model tracks", |t| t.font_size(12).color(TEXT));
            });

        if mode == LegendMode::Full {
            for cat in WindCategory::LEGEND {
                swatch_row(ui, cat.hex(), 4.0, cat.label());
            }
        }

        if ofcl_state == adeck_style::OfficialLegendState::Drawn && mode == LegendMode::Full {
            // Matches how OFCL is actually drawn on the map — wind-coloured
            // with a dark outline (review S4's "swatch mismatch"). Shown as
            // a few adjacent wind-category segments rather than one solid
            // fill (fix-round Important 5: a single Cat 3 block was
            // indistinguishable from the plain Cat 3 row above it), using a
            // lighter outline than the map's own (the legend panel's
            // background is itself dark, so the map's dark outline would be
            // invisible here).
            outlined_swatch_row(
                ui,
                &adeck_style::OFFICIAL_SWATCH_SEGMENTS.map(WindCategory::hex),
                adeck_style::official_outline_legend_hex(),
                adeck_style::official_legend_label(ofcl_state),
            );
        } else {
            // Review L1 + fix-round Important 4: distinct text for "OFCL
            // genuinely absent this cycle" vs. "OFCL exists but is shown as
            // the NHC advisory track instead" — conflating them into one
            // "not drawn" state let the legend claim "No OFCL" directly
            // above a row that implies an official forecast *is* shown.
            // Review S10: this plain-text form (as opposed to the outlined
            // swatch above) is also what `Compact` mode shows even when
            // `Drawn` — the OFCL-state text itself is kept in both modes
            // (it's one of the safety-critical rows), only its fancier
            // swatch rendering is full-mode-only.
            ui.text(adeck_style::official_legend_label(ofcl_state), |t| {
                t.font_size(10).color(MUTED)
            });
        }

        if mode == LegendMode::Full {
            // Split into track and cone (fix-round Important 5: one row
            // with a violet swatch was wrong for the cone, which
            // `draw_nhc_overlays` actually draws in translucent white).
            swatch_row(
                ui,
                adeck_style::rgb_hex(adeck_style::GIS_ADVISORY_RGB),
                2.0,
                "NHC advisory track",
            );
            swatch_row(ui, 0xFFFFFF, 2.0, "NHC advisory cone");
            swatch_row(ui, 0xB0B0B0, 2.0, "CARQ  past track");
            // Review S11 item 3: each default-visible model group's own
            // dash pattern, so the legend key matches what's really drawn
            // on the map rather than implying every model line looks the
            // same. (L15 — matching the real line *width* — stays out of
            // scope; only the dash pattern is updated here.)
            for group in adeck_style::MODEL_GROUP_LEGEND_ORDER {
                let Some(label) = adeck_style::model_group_legend_label(group) else {
                    continue;
                };
                let dash = adeck_style::track_style(group).dash;
                dash_swatch_row(ui, dash, label);
            }
        }

        // Review S1/S10: freshness is safety-critical, so these two lines
        // are shown in every mode, not just `Full`.
        ui.text(cycle_line, |t| {
            t.font_size(10)
                .color(if cycle_stale { WARN } else { MUTED })
        });
        ui.text(age_line, |t| {
            t.font_size(10)
                .color(if fetch_stale { WARN } else { MUTED })
        });
        ui.text("Guidance only.", |t| t.font_size(10).color(WARN));
        ui.text("Not an official forecast.", |t| t.font_size(10).color(WARN));

        if mode == LegendMode::Full {
            // Review L1: the full disclaimer and "Source: NOAA/NHC" credit
            // only live in the Tropical card, which can be closed or sit
            // off-screen on mobile while the map (and this legend) stay
            // visible. Dropped in `Compact` mode (review S10): it's
            // attribution, not a freshness or official-forecast signal.
            ui.text("Source: NOAA/NHC ATCF", |t| t.font_size(10).color(MUTED));
            ui.text("Consult nhc.noaa.gov", |t| t.font_size(10).color(MUTED));
        }
    });
}

fn swatch_row(ui: &mut Ui<'_, ChartWidget>, color: u32, thickness: f32, label: &str) {
    ui.element()
        .width(grow!())
        .height(fixed!(13.0))
        .layout(|l| l.direction(LeftToRight).gap(6).align(Left, CenterY))
        .children(|ui| {
            ui.element()
                .width(fixed!(18.0))
                .height(fixed!(thickness))
                .background_color(color)
                .empty();
            ui.text(label, |t| t.font_size(10).color(TEXT));
        });
}

/// Like `swatch_row`, but the swatch is a row of adjacent `fills` segments
/// inside a 2 px border of `outline` — used only for the OFCL row, so the
/// legend matches how OFCL is really drawn: a wind-coloured line (varying
/// by segment) with an outline underneath it (review S4's "swatch
/// mismatch"; the solid single-colour swatches this replaces matched
/// neither the colour variation nor the outline-plus-fill structure).
fn outlined_swatch_row(ui: &mut Ui<'_, ChartWidget>, fills: &[u32], outline: u32, label: &str) {
    ui.element()
        .width(grow!())
        .height(fixed!(13.0))
        .layout(|l| l.direction(LeftToRight).gap(6).align(Left, CenterY))
        .children(|ui| {
            ui.element()
                .width(fixed!(22.0))
                .height(fixed!(8.0))
                .background_color(outline)
                .layout(|l| l.direction(LeftToRight).padding(2).gap(1))
                .children(|ui| {
                    for &fill in fills {
                        ui.element()
                            .width(grow!())
                            .height(fixed!(4.0))
                            .background_color(fill)
                            .empty();
                    }
                });
            ui.text(label, |t| t.font_size(10).color(TEXT));
        });
}

/// Like `swatch_row`, but the swatch previews `dash`'s real on/off pattern
/// (review S11 item 3) instead of one solid bar: `adeck_style::
/// dash_preview_segments` tiles the pattern across a fixed preview width as
/// alternating (length, is_on) pieces, and this lays each piece out as its
/// own fixed-width element — a transparent one for "off", so the panel's own
/// background shows through as the gap — in the same `LeftToRight` row the
/// "on" pieces sit in, with no absolute positioning needed.
fn dash_swatch_row(
    ui: &mut Ui<'_, ChartWidget>,
    dash: Option<adeck_style::DashPattern>,
    label: &str,
) {
    const PREVIEW_PX: f32 = 28.0;
    let segments = adeck_style::dash_preview_segments(dash, PREVIEW_PX);
    ui.element()
        .width(grow!())
        .height(fixed!(13.0))
        .layout(|l| l.direction(LeftToRight).gap(6).align(Left, CenterY))
        .children(|ui| {
            ui.element()
                .width(fixed!(PREVIEW_PX))
                .height(fixed!(13.0))
                .layout(|l| l.direction(LeftToRight).align(Left, CenterY))
                .children(|ui| {
                    for (len, on) in segments {
                        let mut el = ui.element().width(fixed!(len)).height(fixed!(2.0));
                        if on {
                            el = el.background_color(TEXT);
                        }
                        el.empty();
                    }
                });
            ui.text(label, |t| t.font_size(10).color(TEXT));
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── S10: compact/full legend-mode selection ─────────────────────────

    #[test]
    fn legend_mode_full_on_a_tall_desktop_window() {
        assert_eq!(legend_mode(1080.0, false, 36.0, 54.0), LegendMode::Full);
    }

    #[test]
    fn legend_mode_compact_below_the_height_trigger() {
        let trigger = compact_height_trigger_px(36.0, 54.0);
        assert_eq!(
            legend_mode(trigger - 1.0, false, 36.0, 54.0),
            LegendMode::Compact
        );
    }

    #[test]
    fn legend_mode_full_right_at_the_height_trigger() {
        // The trigger is a strict "under", so the boundary itself is still
        // Full — matches `is_stale`/`is_stale_cycle`'s own boundary
        // convention elsewhere in this overlay (`adeck_style.rs`).
        let trigger = compact_height_trigger_px(36.0, 54.0);
        assert_eq!(legend_mode(trigger, false, 36.0, 54.0), LegendMode::Full);
    }

    #[test]
    fn legend_mode_trigger_matches_where_legend_y_would_clamp() {
        // Review S10 fix-round follow-up: the mode decision and the y-clamp
        // must agree on whether Full actually fits, so a window just above
        // the trigger must NOT need clamping, and just below it must.
        // At the trigger itself `from_bottom` lands exactly on `top_bar_px`
        // (clamped or not, the value's identical), so "just above" needs a
        // height strictly greater than the trigger to prove no clamping.
        let trigger = compact_height_trigger_px(36.0, 54.0);
        let y_above = legend_y(trigger + 1.0, 36.0, 54.0, FULL_HEIGHT_ESTIMATE);
        assert!(
            y_above > 36.0,
            "just above the trigger, Full must not clamp"
        );
        let y_below = legend_y(trigger - 1.0, 36.0, 54.0, FULL_HEIGHT_ESTIMATE);
        assert_eq!(y_below, 36.0, "just below the trigger, Full would clamp");
    }

    #[test]
    fn legend_mode_compact_on_mobile_even_with_a_tall_window() {
        // Review S10: mobile triggers compact regardless of height — a
        // tall phone in portrait is still the cramped mobile layout, not a
        // roomy desktop one.
        assert_eq!(legend_mode(1200.0, true, 36.0, 54.0), LegendMode::Compact);
    }

    #[test]
    fn legend_mode_compact_when_both_signals_say_compact() {
        assert_eq!(legend_mode(200.0, true, 36.0, 54.0), LegendMode::Compact);
    }

    // ── S10: y-position clamping ─────────────────────────────────────────

    #[test]
    fn legend_y_anchors_above_the_bottom_bar_on_a_tall_window() {
        let y = legend_y(1080.0, 36.0, 54.0, FULL_HEIGHT_ESTIMATE);
        assert_eq!(y, 1080.0 - 54.0 - FULL_HEIGHT_ESTIMATE);
        assert!(y > 36.0, "plenty of room above the top bar too");
    }

    #[test]
    fn legend_y_clamps_to_the_top_bar_on_a_short_window() {
        // A window short enough that bottom-anchoring would put the panel
        // above the top bar (or off-screen entirely) must instead sit right
        // at the top bar's margin (review S10's "under ~320-400px tall").
        let y = legend_y(300.0, 36.0, 54.0, FULL_HEIGHT_ESTIMATE);
        assert_eq!(y, 36.0);
    }

    #[test]
    fn legend_y_never_goes_negative_or_above_the_top_bar() {
        for screen_h in [0.0, 50.0, 150.0, 320.0, 400.0] {
            let y = legend_y(screen_h, 36.0, 54.0, FULL_HEIGHT_ESTIMATE);
            assert!(y >= 36.0, "y={y} for screen_h={screen_h}");
        }
    }

    #[test]
    fn legend_y_compact_estimate_fits_more_screens_before_clamping() {
        // At a height where the full estimate would already clamp, the
        // much shorter compact estimate should still have room to
        // bottom-anchor normally.
        let full_y = legend_y(420.0, 36.0, 54.0, FULL_HEIGHT_ESTIMATE);
        let compact_y = legend_y(420.0, 36.0, 54.0, COMPACT_HEIGHT_ESTIMATE);
        assert_eq!(full_y, 36.0, "full estimate clamps here");
        assert!(
            compact_y > 36.0,
            "compact estimate has room to bottom-anchor at the same height"
        );
    }
}
