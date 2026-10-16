//! Floating map legend for the spaghetti-models overlay: wind-category
//! swatches, line keys, data age, and the short "not an official forecast"
//! notice. Shown whenever model tracks are drawn.

use crate::adeck_style::WindCategory;
use crate::widgets::{ChartWidget, SYMBOL_FONT, glass_panel, nf};
use ply_engine::prelude::*;

pub const WIDTH: f32 = 196.0;
/// Fixed so the caller can anchor the legend above the bottom bar. Tune by
/// eye if rows are added.
pub const HEIGHT: f32 = 262.0;

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
        swatch_row(ui, 0xFFFFFF, 4.0, "OFCL  NHC official");
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
