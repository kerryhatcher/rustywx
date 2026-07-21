//! Settings panel — glass modal (Stage 6 `glass_panel`) exposing
//! [`crate::settings::Settings`] via toggle/cycle buttons.
//!
//! Follows the `toggle.rs` idiom: `draw()` declares the elements each frame,
//! and the caller checks `ply.is_just_pressed(ID)` after `ply.show()` to
//! react to presses (see the button IDs below, used from `main.rs`).

use crate::settings::Settings;
use crate::widgets::glass_panel;
use ply_engine::prelude::*;

pub const BACKDROP_ID: &str = "settings-modal-backdrop";
pub const CLOSE_ID: &str = "settings-modal-close";
pub const BORDERS_TOGGLE_ID: &str = "settings-toggle-borders";
pub const WATCHES_TOGGLE_ID: &str = "settings-toggle-watches";
pub const WARNINGS_TOGGLE_ID: &str = "settings-toggle-warnings";
pub const NHC_TOGGLE_ID: &str = "settings-toggle-nhc";
pub const ANIMATION_CYCLE_ID: &str = "settings-cycle-animation";
pub const TDBZ_CYCLE_ID: &str = "settings-cycle-tdbz";
pub const USE_CURRENT_SITE_ID: &str = "settings-use-current-site";
pub const DYSLEXIC_TOGGLE_ID: &str = "settings-toggle-dyslexic";
pub const SWEEP_TOGGLE_ID: &str = "settings-toggle-sweep";
pub const SCOPE_RINGS_TOGGLE_ID: &str = "settings-toggle-scope-rings";
pub const CC_GATE_TOGGLE_ID: &str = "settings-toggle-cc-gate";
pub const LOCATION_INPUT_ID: &str = "settings-location-input";
pub const LOCATION_DETECT_ID: &str = "settings-location-detect";
pub const CENTER_TOGGLE_ID: &str = "settings-toggle-center";

const ROW_HEIGHT: f32 = 32.0;
const TEXT_COLOR: u32 = 0xE8E0DC;
const MUTED_COLOR: u32 = 0x9E9590;
const ACTIVE_BG: u32 = 0x0dc5b8;
const INACTIVE_BG: u32 = 0x1E1B1B;

/// One row: a label on the left, a control on the right.
fn row(ui: &mut Ui<'_, ()>, label: &str, children: impl FnOnce(&mut Ui<'_, ()>)) {
    ui.element()
        .width(grow!())
        .height(fixed!(ROW_HEIGHT))
        .layout(|l| l.direction(LeftToRight).gap(8).align(Left, CenterY))
        .children(|ui| {
            ui.element()
                .width(fixed!(160.0))
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .children(|ui| {
                    ui.text(label, |t| t.font_size(12).color(MUTED_COLOR));
                });
            ui.element()
                .width(grow!())
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .children(children);
        });
}

fn bool_toggle(ui: &mut Ui<'_, ()>, id: &'static str, label: &str, value: bool) {
    ui.element()
        .id(id)
        .width(fit!())
        .height(fixed!(24.0))
        .background_color(if value { ACTIVE_BG } else { INACTIVE_BG })
        .corner_radius(4.0)
        .layout(|l| l.padding((0, 10, 0, 10)).align(CenterX, CenterY))
        .accessibility(|a| a.checkbox(label).checked(value))
        .children(|ui| {
            ui.text(if value { "On" } else { "Off" }, |t| {
                t.font_size(12).color(TEXT_COLOR)
            });
        });
}

fn cycle_button(ui: &mut Ui<'_, ()>, id: &'static str, label: &str) {
    ui.element()
        .id(id)
        .width(fit!())
        .height(fixed!(24.0))
        .background_color(INACTIVE_BG)
        .corner_radius(4.0)
        .layout(|l| l.padding((0, 10, 0, 10)).gap(6).align(CenterX, CenterY))
        .accessibility(|a| a.button(label))
        .children(|ui| {
            ui.text(label, |t| t.font_size(12).color(TEXT_COLOR));
            ui.text(super::nf::REFRESH, |t| {
                t.font_size(12).font(&super::SYMBOL_FONT).color(TEXT_COLOR)
            });
        });
}

/// Draw the settings modal (backdrop + glass panel). No-op if the caller
/// doesn't want it shown — check `state.show_settings_panel` before calling.
pub fn draw(
    ui: &mut Ui<'_, ()>,
    settings: &Settings,
    current_site_id: &str,
    location_input: &str,
    location_focused: bool,
    location_status: &str,
) {
    let modal_w = 420.0;
    let modal_h = 510.0;
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
                    ui.text("Settings", |t| t.font_size(14).color(TEXT_COLOR));
                    ui.element().width(grow!()).height(fixed!(1.0)).empty();
                    ui.element()
                        .id(CLOSE_ID)
                        .width(fixed!(28.0))
                        .height(fixed!(28.0))
                        .background_color(0x3a1a1a)
                        .corner_radius(4.0)
                        .layout(|l| l.align(CenterX, CenterY))
                        .accessibility(|a| a.button("Close settings"))
                        .children(|ui| {
                            ui.text(super::nf::CLOSE, |t| {
                                t.font_size(14).font(&super::SYMBOL_FONT).color(TEXT_COLOR)
                            });
                        });
                });

            // Content.
            ui.element()
                .width(grow!())
                .height(grow!())
                .background_color(0x0a0d12)
                .layout(|l| l.padding(12).gap(4).direction(TopToBottom))
                .children(|ui| {
                    row(ui, "Default site", |ui| {
                        ui.text(&settings.default_site, |t| {
                            t.font_size(12).color(TEXT_COLOR)
                        });
                    });
                    row(ui, "", |ui| {
                        let use_current_site_label =
                            format!("Use current site ({current_site_id})");
                        ui.element()
                            .id(USE_CURRENT_SITE_ID)
                            .width(fit!())
                            .height(fixed!(24.0))
                            .background_color(INACTIVE_BG)
                            .corner_radius(4.0)
                            .layout(|l| l.padding((0, 10, 0, 10)).align(CenterX, CenterY))
                            .accessibility(|a| a.button(&use_current_site_label))
                            .children(|ui| {
                                ui.text(&use_current_site_label, |t| {
                                    t.font_size(11).color(TEXT_COLOR)
                                });
                            });
                    });
                    row(ui, "Show borders on startup", |ui| {
                        bool_toggle(
                            ui,
                            BORDERS_TOGGLE_ID,
                            "Show borders on startup",
                            settings.show_borders,
                        );
                    });
                    row(ui, "Show watches on startup", |ui| {
                        bool_toggle(
                            ui,
                            WATCHES_TOGGLE_ID,
                            "Show watches on startup",
                            settings.show_watches,
                        );
                    });
                    row(ui, "Show warnings on startup", |ui| {
                        bool_toggle(
                            ui,
                            WARNINGS_TOGGLE_ID,
                            "Show warnings on startup",
                            settings.show_warnings,
                        );
                    });
                    row(ui, "Show tropical panel on startup", |ui| {
                        bool_toggle(
                            ui,
                            NHC_TOGGLE_ID,
                            "Show tropical panel on startup",
                            settings.show_nhc,
                        );
                    });
                    row(ui, "Dyslexia-friendly font", |ui| {
                        bool_toggle(
                            ui,
                            DYSLEXIC_TOGGLE_ID,
                            "Dyslexia-friendly font",
                            settings.dyslexic_font,
                        );
                    });
                    row(ui, "Radar sweep animation", |ui| {
                        bool_toggle(
                            ui,
                            SWEEP_TOGGLE_ID,
                            "Radar sweep animation",
                            settings.show_sweep,
                        );
                    });
                    row(ui, "Scope rings & crosshairs", |ui| {
                        bool_toggle(
                            ui,
                            SCOPE_RINGS_TOGGLE_ID,
                            "Scope rings & crosshairs",
                            settings.show_scope_rings,
                        );
                    });
                    row(ui, "CC-gate reflectivity", |ui| {
                        bool_toggle(
                            ui,
                            CC_GATE_TOGGLE_ID,
                            "CC-gate reflectivity",
                            settings.cc_gate_enabled,
                        );
                    });
                    // ── My Location ──────────────────────────────
                    ui.text("My Location", |t| t.font_size(12).color(TEXT_COLOR));
                    row(ui, "Coords or ZIP", |ui| {
                        let bg = if location_focused { 0x1a2730 } else { 0x11151c };
                        let shown = if location_input.is_empty() && !location_focused {
                            "e.g. 33.75, -84.39 or 30301"
                        } else {
                            location_input
                        };
                        ui.element()
                            .id(LOCATION_INPUT_ID)
                            .width(grow!())
                            .height(fixed!(24.0))
                            .background_color(bg)
                            .corner_radius(4.0)
                            .layout(|l| l.padding((0, 8, 0, 8)).align(Left, CenterY))
                            .accessibility(|a| a.button("Edit location"))
                            .children(|ui| {
                                ui.text(shown, |t| {
                                    t.font_size(11).color(
                                        if location_input.is_empty() && !location_focused {
                                            MUTED_COLOR
                                        } else {
                                            TEXT_COLOR
                                        },
                                    )
                                });
                            });
                    });
                    row(ui, "", |ui| {
                        ui.element()
                            .id(LOCATION_DETECT_ID)
                            .width(fit!())
                            .height(fixed!(24.0))
                            .background_color(INACTIVE_BG)
                            .corner_radius(4.0)
                            .layout(|l| l.padding((0, 10, 0, 10)).align(CenterX, CenterY))
                            .accessibility(|a| a.button("Detect location"))
                            .children(|ui| {
                                ui.text("Detect", |t| t.font_size(11).color(TEXT_COLOR));
                            });
                    });
                    row(ui, "Center map on my location", |ui| {
                        bool_toggle(
                            ui,
                            CENTER_TOGGLE_ID,
                            "Center map on my location",
                            settings.center_on_location,
                        );
                    });
                    ui.text(location_status, |t| t.font_size(10).color(MUTED_COLOR));
                    row(ui, "Animation level", |ui| {
                        cycle_button(ui, ANIMATION_CYCLE_ID, settings.animation_level.label());
                    });
                    row(ui, "TDBZ clutter kernel", |ui| {
                        cycle_button(ui, TDBZ_CYCLE_ID, settings.tdbz_kernel.label());
                    });
                    ui.text(
                        &format!(
                            "Radar poll: {}s   NHC refresh: {}s",
                            settings.poll_interval_secs, settings.nhc_refresh_secs
                        ),
                        |t| t.font_size(10).color(MUTED_COLOR),
                    );
                });
        });
}
