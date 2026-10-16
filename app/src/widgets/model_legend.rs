//! Floating map legend for the spaghetti-models overlay: wind-category
//! swatches, line keys, data age, and the short "not an official forecast"
//! notice. Shown whenever model tracks are drawn.

use crate::adeck_style::{self, WindCategory};
use crate::widgets::{ChartWidget, SYMBOL_FONT, glass_panel, nf};
use ply_engine::prelude::*;

pub const WIDTH: f32 = 196.0;
/// Fixed so the caller can anchor the legend above the bottom bar. Tune by
/// eye if rows are added. Bumped for review S4/L1's new rows: the GIS
/// advisory track and cone swatches (split into two, fix-round Important 5
/// — the cone draws white, not the track's violet), and the source-credit
/// plus "consult nhc.noaa.gov" lines (each row ~13-16 px with its gap).
pub const HEIGHT: f32 = 324.0;

const TEXT: u32 = 0xE8E0DC;
const MUTED: u32 = 0x9E9590;
const WARN: u32 = 0xFFC140;

#[allow(clippy::too_many_arguments)]
pub fn draw(
    ui: &mut Ui<'_, ChartWidget>,
    x: f32,
    y: f32,
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
            .height(fixed!(HEIGHT)),
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
        for cat in WindCategory::LEGEND {
            swatch_row(ui, cat.hex(), 4.0, cat.label());
        }
        if ofcl_state == adeck_style::OfficialLegendState::Drawn {
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
            ui.text(adeck_style::official_legend_label(ofcl_state), |t| {
                t.font_size(10).color(MUTED)
            });
        }
        // Split into track and cone (fix-round Important 5: one row with a
        // violet swatch was wrong for the cone, which `draw_nhc_overlays`
        // actually draws in translucent white).
        swatch_row(
            ui,
            adeck_style::rgb_hex(adeck_style::GIS_ADVISORY_RGB),
            2.0,
            "NHC advisory track",
        );
        swatch_row(ui, 0xFFFFFF, 2.0, "NHC advisory cone");
        swatch_row(ui, 0xB0B0B0, 2.0, "CARQ  past track");
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
        // Review L1: the full disclaimer and "Source: NOAA/NHC" credit only
        // live in the Tropical card, which can be closed or sit off-screen
        // on mobile while the map (and this legend) stay visible.
        ui.text("Source: NOAA/NHC ATCF", |t| t.font_size(10).color(MUTED));
        ui.text("Consult nhc.noaa.gov", |t| t.font_size(10).color(MUTED));
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
