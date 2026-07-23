//! Keyboard shortcuts overlay — glass modal (Stage 7) listing all keybindings.
//!
//! Toggled with the ? key, displays a read-only modal with shortcuts grouped
//! by category (Products, Navigation, Overlays, General). Mirrors the
//! `settings.rs` glass-modal pattern.

use crate::widgets::glass_panel;
use ply_engine::prelude::*;

pub const BACKDROP_ID: &str = "shortcuts-modal-backdrop";
pub const CLOSE_ID: &str = "shortcuts-modal-close";

const ROW_HEIGHT: f32 = 24.0;
const TEXT_COLOR: u32 = 0xE8E0DC;
const MUTED_COLOR: u32 = 0x9E9590;
const ACCENT_COLOR: u32 = 0x0dc5b8;
const SECTION_BG: u32 = 0x1E1B1B;

/// Key-label font (Inter Regular), embedded so the binary doesn't depend on
/// a CWD-relative `assets/` directory (see issue #11).
static KEY_FONT: FontAsset = FontAsset::Bytes {
    file_name: "Inter-Regular.ttf",
    data: include_bytes!("../../assets/fonts/Inter-Regular.ttf"),
};

/// One shortcut row: key on the left, action on the right.
fn shortcut_row(ui: &mut Ui<'_, crate::widgets::ChartWidget>, key: &str, action: &str) {
    ui.element()
        .width(grow!())
        .height(fixed!(ROW_HEIGHT))
        .layout(|l| l.direction(LeftToRight).gap(16).align(Left, CenterY))
        .children(|ui| {
            // Key (monospace, accent color)
            ui.element()
                .width(fixed!(80.0))
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .children(|ui| {
                    ui.text(key, |t| t.font_size(11).font(&KEY_FONT).color(ACCENT_COLOR));
                });
            // Action
            ui.element()
                .width(grow!())
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .children(|ui| {
                    ui.text(action, |t| t.font_size(11).color(TEXT_COLOR));
                });
        });
}

/// Section header with light background.
fn section_header(ui: &mut Ui<'_, crate::widgets::ChartWidget>, title: &str) {
    ui.element()
        .width(grow!())
        .height(fixed!(28.0))
        .background_color(SECTION_BG)
        .corner_radius(4.0)
        .layout(|l| l.padding(8).align(Left, CenterY))
        .children(|ui| {
            ui.text(title, |t| t.font_size(12).color(MUTED_COLOR));
        });
}

/// Draw the keyboard shortcuts modal (backdrop + glass panel).
/// No-op if the caller doesn't want it shown — check `state.show_shortcuts` before calling.
pub fn draw(ui: &mut Ui<'_, crate::widgets::ChartWidget>) {
    let modal_w = 480.0;
    let modal_h = 520.0;
    let modal_x = (screen_width() - modal_w) / 2.0;
    let modal_y = (screen_height() - modal_h) / 2.0;

    // Semi-transparent backdrop (click to close).
    ui.element()
        .id(BACKDROP_ID)
        .width(fixed!(screen_width()))
        .height(fixed!(screen_height()))
        .background_color((0.0f32, 0.0f32, 0.0f32, 220.0f32))
        .floating(|f| f.offset((0.0, 0.0)).z_index(200).attach_root())
        .empty();

    glass_panel::glass(ui.element().width(fixed!(modal_w)).height(fixed!(modal_h)))
        .floating(|f| f.offset((modal_x, modal_y)).z_index(201).attach_root())
        .layout(|l| l.direction(TopToBottom).padding(0).gap(0))
        .children(|ui| {
            // Title bar.
            ui.element()
                .width(grow!())
                .height(fixed!(36.0))
                .background_color(0x1E1B1B)
                .corner_radius(8.0)
                .layout(|l| {
                    l.direction(LeftToRight)
                        .padding((0, 12, 0, 12))
                        .gap(8)
                        .align(Left, CenterY)
                })
                .children(|ui| {
                    ui.text("Keyboard Shortcuts", |t| t.font_size(14).color(TEXT_COLOR));
                    ui.element().width(grow!()).height(fixed!(1.0)).empty();
                    // Close button
                    ui.element()
                        .id(CLOSE_ID)
                        .width(fixed!(28.0))
                        .height(fixed!(28.0))
                        .background_color(0x3a1a1a)
                        .corner_radius(4.0)
                        .layout(|l| l.align(CenterX, CenterY))
                        .accessibility(|a| a.button("Close shortcuts"))
                        .children(|ui| {
                            ui.text(super::nf::CLOSE, |t| {
                                t.font_size(14).font(&super::SYMBOL_FONT).color(TEXT_COLOR)
                            });
                        });
                });

            // Content (scrollable).
            ui.element()
                .width(grow!())
                .height(grow!())
                .background_color(0x0a0d12)
                .layout(|l| l.padding(0).gap(0).direction(TopToBottom))
                .overflow(|o| {
                    o.scroll_y()
                        .scrollbar(|s| s.width(6.0).thumb_color(0x4a4a4a).track_color(0x1a1a1a))
                })
                .children(|ui| {
                    ui.element()
                        .width(grow!())
                        .height(fit!())
                        .layout(|l| l.padding(12).gap(8).direction(TopToBottom))
                        .children(|ui| {
                            // Products section
                            section_header(ui, "PRODUCTS");
                            shortcut_row(ui, "R", "Reflectivity");
                            shortcut_row(ui, "V", "Velocity");
                            shortcut_row(ui, "W", "Spectrum Width");
                            shortcut_row(ui, "Z", "Differential Reflectivity");
                            shortcut_row(ui, "C", "Correlation Coefficient");
                            shortcut_row(ui, "P", "Differential Phase");

                            // Navigation section
                            section_header(ui, "NAVIGATION");
                            shortcut_row(ui, "← / →", "Previous / Next site");
                            shortcut_row(ui, "T", "Next tilt");
                            shortcut_row(ui, "0", "Reset pan & zoom");

                            // Overlays section
                            section_header(ui, "OVERLAYS");
                            shortcut_row(ui, "B", "Toggle borders");
                            shortcut_row(ui, "W", "Toggle watches");
                            shortcut_row(ui, "A", "Toggle warnings");
                            shortcut_row(ui, "N", "Toggle tropical data");

                            // General section
                            section_header(ui, "GENERAL");
                            shortcut_row(ui, "?", "Show / hide shortcuts");
                            shortcut_row(ui, "Esc", "Close all modals");
                        });
                });
        });
}
