//! Floating map legend for the spaghetti-models overlay: wind-category
//! swatches, line keys, data age, and the short "not an official forecast"
//! notice. Shown whenever model tracks are drawn.

use crate::adeck_style::{self, WindCategory};
use crate::widgets::{ChartWidget, SYMBOL_FONT, glass_panel, nf};
use ply_engine::prelude::*;

pub const WIDTH: f32 = 196.0;
/// Fixed so the caller can anchor the legend above the bottom bar. Tune by
/// eye if rows are added. Bumped for review S4/L1's two new rows: the GIS
/// advisory swatch and the source-credit line (each ~13-16 px with its gap).
pub const HEIGHT: f32 = 296.0;

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
    // Whether the shown guidance's selected cycle actually has an OFCL
    // forecast (review L1: invests often don't, and the legend used to show
    // the OFCL swatch regardless). Callers pass
    // `GuidanceStore::earliest_cycle()`'s `has_official`, the same storm
    // that drives `cycle_line` (review S5).
    has_official: bool,
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
        if has_official {
            // Matches how OFCL is actually drawn on the map — wind-coloured
            // with a dark outline, not plain solid white (review S4's
            // "swatch mismatch"). The fill shown here is Cat 3, a
            // representative mid-strength colour; the real track's colour
            // still varies by forecast intensity along its length.
            outlined_swatch_row(
                ui,
                WindCategory::Cat3.hex(),
                adeck_style::official_outline_hex(),
                adeck_style::official_legend_label(true),
            );
        } else {
            // Review L1: no OFCL swatch when this cycle has none (most
            // invests), instead of showing a swatch for guidance that was
            // never issued.
            ui.text(adeck_style::official_legend_label(false), |t| {
                t.font_size(10).color(MUTED)
            });
        }
        swatch_row(
            ui,
            adeck_style::rgb_hex(adeck_style::GIS_ADVISORY_RGB),
            2.0,
            "NHC advisory track/cone",
        );
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

/// Like `swatch_row`, but the swatch is a coloured bar inside a 2 px border
/// of `outline` — used only for the OFCL row, so the legend matches how
/// OFCL is really drawn: a wind-coloured line with an outline underneath it
/// (review S4's "swatch mismatch"; the plain solid-white swatch it replaces
/// matched neither the colour nor the outline-plus-fill structure).
fn outlined_swatch_row(ui: &mut Ui<'_, ChartWidget>, fill: u32, outline: u32, label: &str) {
    ui.element()
        .width(grow!())
        .height(fixed!(13.0))
        .layout(|l| l.direction(LeftToRight).gap(6).align(Left, CenterY))
        .children(|ui| {
            ui.element()
                .width(fixed!(22.0))
                .height(fixed!(8.0))
                .background_color(outline)
                .layout(|l| l.padding(2))
                .children(|ui| {
                    ui.element()
                        .width(fixed!(18.0))
                        .height(fixed!(4.0))
                        .background_color(fill)
                        .empty();
                });
            ui.text(label, |t| t.font_size(10).color(TEXT));
        });
}
