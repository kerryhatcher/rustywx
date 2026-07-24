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
pub const GAP_FILL_TOGGLE_ID: &str = "settings-toggle-gap-fill";
pub const MULTI_SCALE_TEXTURE_TOGGLE_ID: &str = "settings-toggle-multi-scale-texture";
pub const SUN_SPIKE_TOGGLE_ID: &str = "settings-toggle-sun-spike";
pub const MELTING_LAYER_TOGGLE_ID: &str = "settings-toggle-melting-layer";
pub const REFL_FLOOR_TOGGLE_ID: &str = "settings-toggle-refl-floor";
pub const VEL_SD_TOGGLE_ID: &str = "settings-toggle-vel-sd";
pub const VEL_DEALIAS_TOGGLE_ID: &str = "settings-toggle-vel-dealias";
pub const NONMET_FUZZY_TOGGLE_ID: &str = "settings-toggle-nonmet-fuzzy";
pub const LOCATION_INPUT_ID: &str = "settings-location-input";
pub const LOCATION_DETECT_ID: &str = "settings-location-detect";
pub const CENTER_TOGGLE_ID: &str = "settings-toggle-center";

// Threshold steppers: each pass's numeric knob, adjusted by [-]/[+] buttons
// (no continuous slider — the Ply layout doesn't expose element rects for
// drag mapping, and click steppers reuse the proven is_just_pressed path).
pub const CC_GATE_DEC_ID: &str = "settings-cc-gate-dec";
pub const CC_GATE_INC_ID: &str = "settings-cc-gate-inc";
pub const NONMET_DEC_ID: &str = "settings-nonmet-dec";
pub const NONMET_INC_ID: &str = "settings-nonmet-inc";
pub const REFL_FLOOR_DEC_ID: &str = "settings-refl-floor-dec";
pub const REFL_FLOOR_INC_ID: &str = "settings-refl-floor-inc";
pub const VEL_SD_DEC_ID: &str = "settings-vel-sd-dec";
pub const VEL_SD_INC_ID: &str = "settings-vel-sd-inc";

const ROW_HEIGHT: f32 = 32.0;
const TEXT_COLOR: u32 = 0xE8E0DC;
const MUTED_COLOR: u32 = 0x9E9590;
const ACTIVE_BG: u32 = 0x0dc5b8;
const INACTIVE_BG: u32 = 0x1E1B1B;

/// One row: a label on the left, a control on the right.
fn row(
    ui: &mut Ui<'_, crate::widgets::ChartWidget>,
    label: &str,
    children: impl FnOnce(&mut Ui<'_, crate::widgets::ChartWidget>),
) {
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

fn bool_toggle(
    ui: &mut Ui<'_, crate::widgets::ChartWidget>,
    id: &'static str,
    label: &str,
    value: bool,
) {
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

fn cycle_button(ui: &mut Ui<'_, crate::widgets::ChartWidget>, id: &'static str, label: &str) {
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

/// Lay a toggle and its threshold stepper side by side in one row's control
/// area (keeps the row count unchanged so the panel stays compact).
fn toggle_with_stepper(
    ui: &mut Ui<'_, crate::widgets::ChartWidget>,
    children: impl FnOnce(&mut Ui<'_, crate::widgets::ChartWidget>),
) {
    ui.element()
        .width(fit!())
        .height(grow!())
        .layout(|l| l.direction(LeftToRight).gap(10).align(Left, CenterY))
        .children(children);
}

/// A `[-] value [+]` numeric stepper. `dec_id`/`inc_id` are polled with
/// `ply.is_just_pressed` in `main.rs` to nudge the underlying setting.
/// Follows `bool_toggle`'s controlled-widget idiom: it renders `value_text`,
/// the caller owns the value and re-clamps on press.
fn stepper(
    ui: &mut Ui<'_, crate::widgets::ChartWidget>,
    dec_id: &'static str,
    inc_id: &'static str,
    label: &str,
    value_text: &str,
) {
    let button = |ui: &mut Ui<'_, crate::widgets::ChartWidget>, id: &'static str, glyph: &str| {
        ui.element()
            .id(id)
            .width(fixed!(24.0))
            .height(fixed!(24.0))
            .background_color(INACTIVE_BG)
            .corner_radius(4.0)
            .layout(|l| l.align(CenterX, CenterY))
            .accessibility(|a| a.button(&format!("{label}: {glyph}")))
            .children(|ui| {
                ui.text(glyph, |t| t.font_size(14).color(TEXT_COLOR));
            });
    };
    ui.element()
        .width(fit!())
        .height(grow!())
        .layout(|l| l.direction(LeftToRight).gap(6).align(Left, CenterY))
        .children(|ui| {
            button(ui, dec_id, "-");
            ui.element()
                .width(fixed!(40.0))
                .height(grow!())
                .layout(|l| l.align(CenterX, CenterY))
                .children(|ui| {
                    ui.text(value_text, |t| t.font_size(12).color(TEXT_COLOR));
                });
            button(ui, inc_id, "+");
        });
}

/// Draw the settings modal (backdrop + glass panel). No-op if the caller
/// doesn't want it shown — check `state.show_settings_panel` before calling.
#[allow(clippy::too_many_arguments)]
pub fn draw(
    ui: &mut Ui<'_, crate::widgets::ChartWidget>,
    settings: &Settings,
    current_site_id: &str,
    location_input: &str,
    location_focused: bool,
    location_status: &str,
    qc_report: crate::scope::QcReport,
    product_label: &str,
) {
    let modal_w = 420.0;
    let modal_h = 542.0;
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
                .overflow(|o| {
                    o.scroll_y()
                        .scrollbar(|s| s.width(6.0).thumb_color(0x4a4a4a).track_color(0x1a1a1a))
                })
                .layout(|l| l.padding(12).gap(4).direction(TopToBottom))
                .children(|ui| {
                    // Live QC feedback: how many gates the active passes
                    // removed from the current product on the last raster, so
                    // a toggle visibly confirms it did something.
                    let qc_line = if qc_report.gates_before == 0 {
                        "QC: no radar gates loaded".to_string()
                    } else {
                        format!(
                            "QC removed {} of {} {} gates",
                            qc_report.removed(),
                            qc_report.gates_before,
                            product_label,
                        )
                    };
                    ui.text(&qc_line, |t| t.font_size(11).color(ACTIVE_BG));
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
                        toggle_with_stepper(ui, |ui| {
                            bool_toggle(
                                ui,
                                CC_GATE_TOGGLE_ID,
                                "CC-gate reflectivity",
                                settings.cc_gate_enabled,
                            );
                            stepper(
                                ui,
                                CC_GATE_DEC_ID,
                                CC_GATE_INC_ID,
                                "CC threshold",
                                &format!("{:.2}", settings.cc_gate_threshold),
                            );
                        });
                    });
                    row(ui, "Fuzzy non-met filter", |ui| {
                        toggle_with_stepper(ui, |ui| {
                            bool_toggle(
                                ui,
                                NONMET_FUZZY_TOGGLE_ID,
                                "Fuzzy non-met filter",
                                settings.nonmet_fuzzy_enabled,
                            );
                            stepper(
                                ui,
                                NONMET_DEC_ID,
                                NONMET_INC_ID,
                                "Non-met threshold",
                                &format!("{:.2}", settings.nonmet_threshold),
                            );
                        });
                    });
                    row(ui, "Fill radial gaps", |ui| {
                        bool_toggle(
                            ui,
                            GAP_FILL_TOGGLE_ID,
                            "Fill radial gaps",
                            settings.refl_gap_fill_enabled,
                        );
                    });
                    row(ui, "Multi-scale TDBZ", |ui| {
                        bool_toggle(
                            ui,
                            MULTI_SCALE_TEXTURE_TOGGLE_ID,
                            "Multi-scale TDBZ",
                            settings.multi_scale_texture_enabled,
                        );
                    });
                    row(ui, "Sun-spike / RFI removal", |ui| {
                        bool_toggle(
                            ui,
                            SUN_SPIKE_TOGGLE_ID,
                            "Sun-spike / RFI removal",
                            settings.sun_spike_removal_enabled,
                        );
                    });
                    row(ui, "Melting-layer hint", |ui| {
                        bool_toggle(
                            ui,
                            MELTING_LAYER_TOGGLE_ID,
                            "Melting-layer hint",
                            settings.melting_layer_hint_enabled,
                        );
                    });
                    row(ui, "Noise-floor cut (dBZ)", |ui| {
                        toggle_with_stepper(ui, |ui| {
                            bool_toggle(
                                ui,
                                REFL_FLOOR_TOGGLE_ID,
                                "Noise-floor cut",
                                settings.refl_floor_enabled,
                            );
                            stepper(
                                ui,
                                REFL_FLOOR_DEC_ID,
                                REFL_FLOOR_INC_ID,
                                "Noise floor dBZ",
                                &format!("{:.0}", settings.refl_floor_dbz),
                            );
                        });
                    });
                    row(ui, "Velocity dealias", |ui| {
                        bool_toggle(
                            ui,
                            VEL_DEALIAS_TOGGLE_ID,
                            "Velocity dealias",
                            settings.vel_dealias_enabled,
                        );
                    });
                    row(ui, "Velocity SD censor", |ui| {
                        toggle_with_stepper(ui, |ui| {
                            bool_toggle(
                                ui,
                                VEL_SD_TOGGLE_ID,
                                "Velocity SD censor",
                                settings.vel_sd_censor_enabled,
                            );
                            stepper(
                                ui,
                                VEL_SD_DEC_ID,
                                VEL_SD_INC_ID,
                                "Velocity SD m/s",
                                &format!("{:.0}", settings.vel_sd_threshold),
                            );
                        });
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
