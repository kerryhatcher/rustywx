//! rustywx — Ply radar scope (Stage 2: Live Data).
//!
//! Boots a window, fetches real NEXRAD radar data via a background worker,
//! caches scans to disk with Ply storage, and renders the scope with
//! pan/zoom and keyboard controls.

use ply_engine::prelude::*;
use rustywx::alerts;
use rustywx::borders;
use rustywx::cache::Cache;
use rustywx::colors;
use rustywx::data::{self, WorkerMessage};
use rustywx::geo;
use rustywx::model::{Product, RadialData, SweepData};
use rustywx::nhc;
use rustywx::scope;
use rustywx::state::AppState;
use rustywx::widgets::dropdown::{DropdownConfig, DropdownOption, DropdownState};
use rustywx::widgets::toggle::{self, ToggleOption};
use std::collections::HashMap;
use std::sync::mpsc;

const SITE_DROPDOWN: DropdownConfig = DropdownConfig {
    button_id: "site-dropdown-btn",
    panel_id: "site-dropdown-panel",
    option_id: "site-dropdown-option",
    width: 260.0,
    visible_rows: 12,
    panel_offset: (8.0, 44.0),
    searchable: true,
};

const TILT_DROPDOWN: DropdownConfig = DropdownConfig {
    button_id: "tilt-dropdown-btn",
    panel_id: "tilt-dropdown-panel",
    option_id: "tilt-dropdown-option",
    width: 100.0,
    visible_rows: 10,
    panel_offset: (610.0, 44.0),
    searchable: false,
};

const PRODUCT_OPTIONS: [ToggleOption<Product>; 2] = [
    ToggleOption {
        id: "btn-refl",
        label: "Reflectivity",
        value: Product::Reflectivity,
    },
    ToggleOption {
        id: "btn-vel",
        label: "Velocity",
        value: Product::Velocity,
    },
];

const STORM_DROPDOWN: DropdownConfig = DropdownConfig {
    button_id: "storm-dropdown-btn",
    panel_id: "storm-dropdown-panel",
    option_id: "storm-dropdown-option",
    width: 200.0,
    visible_rows: 8,
    panel_offset: (8.0, 44.0),
    searchable: true,
};

// ---------------------------------------------------------------------------
// Synthetic radar data (fallback until real data arrives)
// ---------------------------------------------------------------------------

fn synthetic_sweep() -> SweepData {
    let mut radials = Vec::new();
    for az_idx in 0..360 {
        let azimuth = az_idx as f32;
        let mut gates = Vec::new();
        let num_gates =
            ((scope::MAX_RANGE_KM - scope::FIRST_GATE_KM) / scope::GATE_SPACING_KM) as usize;
        for g in 0..num_gates {
            let range_km = scope::FIRST_GATE_KM + g as f32 * scope::GATE_SPACING_KM;
            let angle = (azimuth + range_km * 2.0).to_radians();
            let base = 30.0 + 20.0 * angle.sin();
            let cell1 = if (azimuth - 90.0).abs() < 15.0 && (range_km - 80.0).abs() < 20.0 {
                25.0
            } else {
                0.0
            };
            let cell2 = if (azimuth - 270.0).abs() < 10.0 && (range_km - 120.0).abs() < 15.0 {
                30.0
            } else {
                0.0
            };
            let value = base + cell1 + cell2;
            gates.push(if value > 5.0 { Some(value) } else { None });
        }
        radials.push(RadialData {
            azimuth_deg: azimuth,
            gates,
        });
    }
    SweepData {
        elevation_deg: 0.5,
        radials,
    }
}

// ---------------------------------------------------------------------------
// Window config
// ---------------------------------------------------------------------------

fn window_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: miniquad::conf::Conf {
            window_title: "rustywx — NEXRAD Radar Scope".to_owned(),
            window_width: 900,
            window_height: 960,
            high_dpi: true,
            sample_count: 4,
            platform: miniquad::conf::Platform {
                webgl_version: miniquad::conf::WebGLVersion::WebGL2,
                ..Default::default()
            },
            ..Default::default()
        },
        draw_call_vertex_capacity: 100_000,
        draw_call_index_capacity: 100_000,
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[macroquad::main(window_conf)]
async fn main() {
    // Tokio runtime for cache I/O and other async background work.
    // The game loop is driven by macroquad's async executor; this
    // runtime lives alongside it so `tokio::spawn` works everywhere.
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let _rt_guard = rt.enter();

    static DEFAULT_FONT: FontAsset = FontAsset::Path("assets/fonts/DejaVuSansMono.ttf");
    let mut ply = Ply::<()>::new(&DEFAULT_FONT).await;

    // ── Channels for the background data worker ───────────────
    let (worker_tx, worker_rx) = mpsc::channel();
    let (site_tx, site_rx) = mpsc::channel();

    // Spawn the background NEXRAD data worker.
    let initial_site = geo::RADAR_SITES[0].id.to_string();
    data::spawn_worker(worker_tx, initial_site.clone(), site_rx);

    // ── Open disk cache ────────────────────────────────────────
    let cache = Cache::new().await.expect("Ply storage initialisation");

    // Kick off a non-blocking load of the last-cached scan for the
    // initial site so we have something to show before the first
    // network fetch completes.
    let pending_load = Some(cache.load_scan(&initial_site));

    // ── Load cached borders (if any) ───────────────────────────
    let mut pending_borders = {
        let storage = cache.storage();
        let (tx, rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let result = borders::load_cached(&storage).await;
            let _ = tx.send(result);
        });
        rx
    };

    let mut state = AppState {
        site_index: 0,
        product: Product::Reflectivity,
        pan_km: (0.0, 0.0),
        zoom: 1.0,
        radar_texture: None,
        needs_reraster: true,
        scan: None,
        tilt_index: 0,
        status_text: "Loading cached data…".to_string(),
        worker_rx,
        site_tx,
        cache,
        pending_load,
        site_dropdown: DropdownState::default(),
        tilt_dropdown: DropdownState::default(),
        borders: Vec::new(),
        borders_loaded: false,
        borders_fetch_fired: false,
        alerts: Vec::new(),
        alerts_loaded: false,
        alerts_fetch_fired: false,
        last_alert_poll: 0.0,
        show_borders: true,
        show_alerts: true,
        nhc_bundle: None,
        nhc_fetch: nhc::NhcFetchState::new(),
        nhc_fetch_fired: false,
        nhc_last_poll: 0.0,
        nhc_show_panel: false,
        nhc_selected_storm: 0,
        nhc_storm_dropdown: DropdownState::default(),
        nhc_image_textures: HashMap::new(),
        nhc_overlays: scope::NhcOverlayState::default(),
        last_mouse_pos: None,
    };

    loop {
        clear_background(BLACK);

        // ── Poll cache load ────────────────────────────────────
        if let Some(ref mut rx) = state.pending_load
            && let Ok(cached) = rx.try_recv()
        {
            state.pending_load = None;
            if let Some(scan) = cached {
                state.scan = Some(scan);
                state.needs_reraster = true;
                update_scan_status(&mut state, " [cached]");
            } else {
                state.status_text = "Waiting for data…".to_string();
            }
        }

        // ── Poll worker messages ──────────────────────────────────
        while let Ok(msg) = state.worker_rx.try_recv() {
            match msg {
                WorkerMessage::NewScan { site, scan } => {
                    let current_site = geo::RADAR_SITES[state.site_index].id.to_string();

                    // Discard scans that arrived after the user switched sites.
                    if site != current_site {
                        continue;
                    }

                    // Persist to disk cache (fire-and-forget).
                    state.cache.save_scan(&current_site, &scan);

                    state.scan = Some(*scan);
                    state.tilt_index = 0;
                    state.needs_reraster = true;
                    update_scan_status(&mut state, "");
                }
                WorkerMessage::Status(s) => {
                    // Don't overwrite a real scan status with transient
                    // "Checking…" messages.
                    if state.scan.is_none() {
                        state.status_text = s;
                    }
                }
                WorkerMessage::Error(e) => {
                    state.status_text = format!("Error: {e}");
                }
            }
        }

        // ── Poll cached borders load ──────────────────────────────
        if let Ok(result) = pending_borders.try_recv() {
            match result {
                Ok(Some(rings)) => {
                    state.borders = rings;
                    state.borders_loaded = true;
                }
                Ok(None) => {
                    // No cache — will fetch from network
                }
                Err(e) => {
                    eprintln!("Warning: failed to load cached borders: {e}");
                }
            }
        }

        // ── Fire border fetch if not yet started and no cache ─────
        if !state.borders_loaded && !state.borders_fetch_fired {
            borders::fire_fetch_all();
            state.borders_fetch_fired = true;
        }

        // ── Poll borders net responses ────────────────────────────
        if state.borders_fetch_fired
            && !state.borders_loaded
            && let Some(result) = borders::poll_and_merge()
        {
            state.borders_fetch_fired = false;
            match result {
                Ok(rings) => {
                    // Save to cache
                    borders::save_cached(&state.cache.storage(), &rings);
                    state.borders = rings;
                    state.borders_loaded = true;
                }
                Err(e) => {
                    eprintln!("Warning: border fetch failed: {e}");
                    // Will retry on next frame (borders_fetch_fired is false)
                }
            }
        }

        // ── Fire alerts fetch if not yet started ──────────────────
        let now = get_time();
        if !state.alerts_fetch_fired
            && (!state.alerts_loaded
                || now - state.last_alert_poll > alerts::POLL_INTERVAL.as_secs() as f64)
        {
            alerts::fire_fetch();
            state.alerts_fetch_fired = true;
            state.last_alert_poll = now;
        }

        // ── Poll alerts net response ──────────────────────────────
        if state.alerts_fetch_fired
            && let Some(result) = alerts::poll_response()
        {
            state.alerts_fetch_fired = false;
            match result {
                Ok(alerts_list) => {
                    state.alerts = alerts_list;
                    state.alerts_loaded = true;
                }
                Err(e) => {
                    eprintln!("Warning: alert fetch failed: {e}");
                }
            }
        }

        // ── Fire NHC fetch if not yet started or refresh interval elapsed ─
        let now_nhc = get_time();
        if !state.nhc_fetch_fired
            && (state.nhc_bundle.is_none()
                || now_nhc - state.nhc_last_poll > nhc::POLL_INTERVAL.as_secs() as f64)
        {
            state.nhc_fetch.start();
            state.nhc_fetch_fired = true;
            state.nhc_last_poll = now_nhc;
        }

        // ── Poll NHC fetch state machine ──────────────────────────
        if state.nhc_fetch_fired
            && let Some(result) = state.nhc_fetch.poll()
        {
            state.nhc_fetch_fired = false;
            match result {
                Ok(bundle) => {
                    // Decode image products to textures.
                    for (storm_id, images) in &bundle.image_products {
                        for img in images {
                            if let Some(ref data) = img.data
                                && let Ok(rgba) = image::load_from_memory(data)
                            {
                                let rgba = rgba.to_rgba8();
                                let (w, h) = rgba.dimensions();
                                let tex = Texture2D::from_rgba8(w as u16, h as u16, &rgba);
                                let key = format!("{}:{}", storm_id, img.title);
                                state.nhc_image_textures.insert(key, tex);
                            }
                        }
                    }
                    // Clamp selected storm index.
                    let storm_count = bundle.metas.len();
                    if state.nhc_selected_storm >= storm_count && storm_count > 0 {
                        state.nhc_selected_storm = 0;
                    }
                    state.nhc_bundle = Some(bundle);
                }
                Err(e) => {
                    eprintln!("Warning: NHC fetch failed: {e:#}");
                }
            }
        }

        // ── Get current sweep ─────────────────────────────────────
        let sweep: SweepData = if let Some(ref scan) = state.scan {
            let sweeps = scan.sweeps(state.product);
            if sweeps.is_empty() {
                synthetic_sweep()
            } else {
                let idx = state.tilt_index.min(sweeps.len() - 1);
                sweeps[idx].clone()
            }
        } else {
            synthetic_sweep()
        };

        let site = &geo::RADAR_SITES[state.site_index];

        // Rasterize when needed
        if state.needs_reraster {
            state.needs_reraster = false;
            let rgba = scope::rasterize(
                &sweep,
                state.product,
                scope::RASTER_SIZE_PX,
                scope::MAX_RANGE_KM,
            );
            let tex = Texture2D::from_rgba8(
                scope::RASTER_SIZE_PX as u16,
                scope::RASTER_SIZE_PX as u16,
                &rgba,
            );
            state.radar_texture = Some(tex);
        }

        // Draw scope + overlays directly to screen (avoids render_to_texture
        // coordinate flip — framebuffer bottom-left origin + Ply .image() display
        // causes a 180° rotation of the content).
        scope::draw_scope_to_texture(
            state.radar_texture.as_ref(),
            site,
            state.pan_km,
            state.zoom,
            Some((&state.borders, state.show_borders)),
            Some((&state.alerts, state.show_alerts)),
            state.nhc_bundle.as_ref().map(|b| (b, &state.nhc_overlays)),
        );

        // Build frame-local control options before borrowing Ply for layout.
        let site_options: Vec<DropdownOption> = geo::RADAR_SITES
            .iter()
            .enumerate()
            .map(|(index, site)| DropdownOption {
                source_index: index,
                label: format!("{} — {}", site.id, site.name),
                search_text: format!("{} {}", site.id, site.name),
            })
            .collect();
        let tilt_options: Vec<DropdownOption> = state
            .scan
            .as_ref()
            .map(|scan| {
                scan.sweeps(state.product)
                    .iter()
                    .enumerate()
                    .map(|(index, sweep)| DropdownOption {
                        source_index: index,
                        label: format!("{:.1}°", sweep.elevation_deg),
                        search_text: format!("{:.1}", sweep.elevation_deg),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let tilt_label = tilt_options
            .get(state.tilt_index)
            .map(|option| option.label.as_str())
            .unwrap_or("No tilts");

        // ── Ply UI ─────────────────────────────────────────────────
        let mut ui = ply.begin();

        ui.element()
            .width(grow!())
            .height(grow!())
            .layout(|layout| layout.direction(TopToBottom))
            .children(|ui| {
                // ── Top controls bar ───────────────────────────────
                ui.element()
                    .width(grow!())
                    .height(fixed!(36.0))
                    .background_color(0x12161e)
                    .layout(|layout| {
                        layout
                            .direction(LeftToRight)
                            .padding(8)
                            .gap(12)
                            .align(Left, CenterY)
                    })
                    .children(|ui| {
                        state.site_dropdown.draw(
                            ui,
                            SITE_DROPDOWN,
                            site.id,
                            &site_options,
                            Some(state.site_index),
                        );

                        ui.text(&format!("— {}", site.name), |text| {
                            text.font_size(14).color(0xE8E0DC)
                        });

                        toggle::draw(ui, state.product, &PRODUCT_OPTIONS);

                        state.tilt_dropdown.draw(
                            ui,
                            TILT_DROPDOWN,
                            tilt_label,
                            &tilt_options,
                            tilt_options
                                .get(state.tilt_index)
                                .map(|option| option.source_index),
                        );

                        ui.text(
                            &format!(
                                "Zoom: {:.1}x  Pan: ({:.0}, {:.0}) km",
                                state.zoom, state.pan_km.0, state.pan_km.1
                            ),
                            |text| text.font_size(11).color(0x9E9590),
                        );

                        // ── Overlay toggles (Stage 4) ──────────────────
                        let borders_bg = if state.show_borders {
                            0x3F684A
                        } else {
                            0x1E1B1B
                        };
                        let borders_label = if state.show_borders {
                            "Borders ✓"
                        } else {
                            "Borders"
                        };
                        ui.element()
                            .id("btn-borders")
                            .width(fit!())
                            .height(fixed!(24.0))
                            .background_color(borders_bg)
                            .corner_radius(4.0)
                            .layout(|layout| layout.padding((0, 8, 0, 8)).align(CenterX, CenterY))
                            .children(|ui| {
                                ui.text(borders_label, |text| text.font_size(12).color(0xE8E0DC));
                            });

                        let alerts_bg = if state.show_alerts {
                            0x3F684A
                        } else {
                            0x1E1B1B
                        };
                        let alerts_label = if state.show_alerts {
                            "Alerts ✓"
                        } else {
                            "Alerts"
                        };
                        let alerts_count = if state.alerts_loaded {
                            format!(" ({})", state.alerts.len())
                        } else if state.alerts_fetch_fired {
                            " (…)".to_string()
                        } else {
                            String::new()
                        };
                        ui.element()
                            .id("btn-alerts")
                            .width(fit!())
                            .height(fixed!(24.0))
                            .background_color(alerts_bg)
                            .corner_radius(4.0)
                            .layout(|layout| layout.padding((0, 8, 0, 8)).align(CenterX, CenterY))
                            .children(|ui| {
                                ui.text(&format!("{alerts_label}{alerts_count}"), |text| {
                                    text.font_size(12).color(0xE8E0DC)
                                });
                            });

                        // ── NHC toggle button (Stage 5) ──────────────────
                        let nhc_bg = if state.nhc_show_panel {
                            0x3F684A
                        } else {
                            0x1E1B1B
                        };
                        let nhc_label = if state.nhc_show_panel {
                            "Tropical ✓"
                        } else {
                            "Tropical"
                        };
                        let storm_count = state
                            .nhc_bundle
                            .as_ref()
                            .map(|b| b.metas.len())
                            .unwrap_or(0);
                        let nhc_badge = if storm_count > 0 {
                            format!(" ({storm_count})")
                        } else if state.nhc_fetch_fired {
                            " (…)".to_string()
                        } else {
                            String::new()
                        };
                        ui.element()
                            .id("btn-nhc")
                            .width(fit!())
                            .height(fixed!(24.0))
                            .background_color(nhc_bg)
                            .corner_radius(4.0)
                            .layout(|layout| layout.padding((0, 8, 0, 8)).align(CenterX, CenterY))
                            .children(|ui| {
                                ui.text(&format!("{nhc_label}{nhc_badge}"), |text| {
                                    text.font_size(12).color(0xE8E0DC)
                                });
                            });
                    });

                // ── NHC slide-in panel (Stage 5) ────────────────────────
                if state.nhc_show_panel {
                    let panel_w = 320.0;
                    let panel_h = screen_height() - 60.0;
                    ui.element()
                        .width(fixed!(panel_w))
                        .height(fixed!(panel_h))
                        .background_color(0x12161e)
                        .corner_radius(6.0)
                        .floating(|floating| {
                            floating
                                .offset((screen_width() - panel_w - 8.0, 36.0))
                                .z_index(50)
                                .attach_root()
                        })
                        .layout(|layout| layout.direction(TopToBottom).padding(8).gap(6))
                        .children(|ui| {
                            // Panel header
                            ui.element()
                                .width(grow!())
                                .height(fixed!(28.0))
                                .background_color(0x1E1B1B)
                                .corner_radius(4.0)
                                .layout(|layout| {
                                    layout
                                        .padding((0, 8, 0, 8))
                                        .direction(LeftToRight)
                                        .gap(8)
                                        .align(Left, CenterY)
                                })
                                .children(|ui| {
                                    ui.text("🌀 NHC Tropical Cyclones", |text| {
                                        text.font_size(14).color(0xE8E0DC)
                                    });
                                });

                            let bundle = state.nhc_bundle.as_ref();

                            let has_storms = bundle
                                .as_ref()
                                .map(|b| !b.metas.is_empty())
                                .unwrap_or(false);
                            if !has_storms {
                                // No active storms
                                ui.element()
                                    .width(grow!())
                                    .height(fixed!(40.0))
                                    .layout(|layout| layout.align(CenterX, CenterY))
                                    .children(|ui| {
                                        let msg = if state.nhc_fetch_fired {
                                            "Loading NHC data…"
                                        } else {
                                            "No active storms"
                                        };
                                        ui.text(msg, |text| text.font_size(13).color(0x9E9590));
                                    });
                            } else if let Some(bundle) = bundle.as_ref() {
                                // Storm selector dropdown
                                let storm_options: Vec<DropdownOption> = bundle
                                    .metas
                                    .iter()
                                    .enumerate()
                                    .map(|(i, m)| DropdownOption {
                                        source_index: i,
                                        label: format!("{} — {}", m.name, m.classification),
                                        search_text: format!("{} {}", m.name, m.classification),
                                    })
                                    .collect();
                                let selected_storm = bundle
                                    .metas
                                    .get(state.nhc_selected_storm)
                                    .map(|m| m.name.as_str())
                                    .unwrap_or("—");
                                state.nhc_storm_dropdown.draw(
                                    ui,
                                    STORM_DROPDOWN,
                                    selected_storm,
                                    &storm_options,
                                    Some(state.nhc_selected_storm),
                                );

                                // Storm stats
                                if let Some(meta) = bundle.metas.get(state.nhc_selected_storm) {
                                    ui.element()
                                        .width(grow!())
                                        .height(fit!())
                                        .background_color(0x171A1F)
                                        .corner_radius(4.0)
                                        .layout(|layout| {
                                            layout.direction(TopToBottom).padding(8).gap(4)
                                        })
                                        .children(|ui| {
                                            ui.text(
                                                &format!("{} — {}", meta.name, meta.classification),
                                                |t| t.font_size(13).color(0xE8E0DC),
                                            );
                                            ui.text(
                                                &format!("Intensity: {} kt", meta.intensity_kt),
                                                |t| t.font_size(11).color(0x9E9590),
                                            );
                                            ui.text(
                                                &format!("Pressure: {} mb", meta.pressure_mb),
                                                |t| t.font_size(11).color(0x9E9590),
                                            );
                                            ui.text(
                                                &format!(
                                                    "Position: {:.1}°N, {:.1}°W",
                                                    meta.lat, -meta.lon
                                                ),
                                                |t| t.font_size(11).color(0x9E9590),
                                            );
                                            if let (Some(dir), Some(spd)) =
                                                (meta.movement_dir_deg, meta.movement_speed_kt)
                                            {
                                                ui.text(
                                                    &format!("Movement: {}° at {} kt", dir, spd),
                                                    |t| t.font_size(11).color(0x9E9590),
                                                );
                                            }
                                            ui.text(
                                                &format!("Advisory: {}", meta.advisory_num),
                                                |t| t.font_size(11).color(0x9E9590),
                                            );
                                            ui.text(
                                                &format!("Updated: {}", meta.last_update),
                                                |t| t.font_size(11).color(0x9E9590),
                                            );
                                        });

                                    // Graphics page link
                                    if !meta.graphics_url.is_empty() {
                                        ui.element()
                                            .id("btn-nhc-graphics")
                                            .width(fit!())
                                            .height(fixed!(24.0))
                                            .background_color(0x1E1B1B)
                                            .corner_radius(4.0)
                                            .layout(|layout| {
                                                layout.padding((0, 8, 0, 8)).align(CenterX, CenterY)
                                            })
                                            .children(|ui| {
                                                ui.text("🌐 NHC Graphics Page", |t| {
                                                    t.font_size(11).color(0x4a90d9)
                                                });
                                            });
                                    }

                                    // Map overlay toggles
                                    ui.element()
                                        .width(grow!())
                                        .height(fit!())
                                        .background_color(0x171A1F)
                                        .corner_radius(4.0)
                                        .layout(|layout| {
                                            layout.direction(TopToBottom).padding(8).gap(4)
                                        })
                                        .children(|ui| {
                                            ui.text("Map Overlays", |t| {
                                                t.font_size(12).color(0xE8E0DC)
                                            });
                                            nhc_toggle_button(
                                                ui,
                                                "btn-nhc-cone",
                                                "Forecast Cone",
                                                state.nhc_overlays.show_cone,
                                            );
                                            nhc_toggle_button(
                                                ui,
                                                "btn-nhc-track",
                                                "Track",
                                                state.nhc_overlays.show_track,
                                            );
                                            nhc_toggle_button(
                                                ui,
                                                "btn-nhc-points",
                                                "Points",
                                                state.nhc_overlays.show_points,
                                            );
                                            nhc_toggle_button(
                                                ui,
                                                "btn-nhc-ww",
                                                "Watches/Warnings",
                                                state.nhc_overlays.show_watches_warnings,
                                            );
                                            nhc_toggle_button(
                                                ui,
                                                "btn-nhc-wp",
                                                "Wind Probabilities",
                                                state.nhc_overlays.show_wind_probs,
                                            );
                                            nhc_toggle_button(
                                                ui,
                                                "btn-nhc-at",
                                                "Arrival Times",
                                                state.nhc_overlays.show_arrival_times,
                                            );
                                        });

                                    // Graphics products (thumbnails)
                                    ui.element()
                                        .width(grow!())
                                        .height(fit!())
                                        .background_color(0x171A1F)
                                        .corner_radius(4.0)
                                        .layout(|layout| {
                                            layout.direction(TopToBottom).padding(8).gap(4)
                                        })
                                        .children(|ui| {
                                            ui.text("Graphics Products", |t| {
                                                t.font_size(12).color(0xE8E0DC)
                                            });
                                            if let Some((_, images)) = bundle
                                                .image_products
                                                .iter()
                                                .find(|(id, _)| *id == meta.id)
                                            {
                                                for img in images {
                                                    let key = format!("{}:{}", meta.id, img.title);
                                                    let has_tex =
                                                        state.nhc_image_textures.contains_key(&key);
                                                    ui.element()
                                                        .width(grow!())
                                                        .height(fixed!(28.0))
                                                        .background_color(0x1E1B1B)
                                                        .corner_radius(3.0)
                                                        .layout(|layout| {
                                                            layout
                                                                .direction(LeftToRight)
                                                                .padding((0, 6, 0, 6))
                                                                .gap(6)
                                                                .align(Left, CenterY)
                                                        })
                                                        .children(|ui| {
                                                            let status =
                                                                if has_tex { "✓" } else { "…" };
                                                            ui.text(status, |t| {
                                                                t.font_size(11).color(0x9E9590)
                                                            });
                                                            ui.text(&img.title, |t| {
                                                                t.font_size(11).color(0xE8E0DC)
                                                            });
                                                            ui.element()
                                                                .width(grow!())
                                                                .height(fixed!(1.0))
                                                                .empty();
                                                            ui.element()
                                                                .id((
                                                                    "btn-nhc-img",
                                                                    img.title.len() as u32,
                                                                ))
                                                                .width(fit!())
                                                                .height(fixed!(20.0))
                                                                .background_color(0x2a2a2a)
                                                                .corner_radius(3.0)
                                                                .layout(|layout| {
                                                                    layout
                                                                        .padding((0, 4, 0, 4))
                                                                        .align(CenterX, CenterY)
                                                                })
                                                                .children(|ui| {
                                                                    ui.text("🔗", |t| {
                                                                        t.font_size(10)
                                                                            .color(0x4a90d9)
                                                                    });
                                                                });
                                                        });
                                                }
                                            } else {
                                                ui.text("No graphics available", |t| {
                                                    t.font_size(11).color(0x9E9590)
                                                });
                                            }
                                        });

                                    // Text products (collapsible)
                                    if let Some((_, texts)) =
                                        bundle.text_products.iter().find(|(id, _)| *id == meta.id)
                                    {
                                        for product in texts {
                                            ui.element()
                                                .width(grow!())
                                                .height(fit!())
                                                .background_color(0x171A1F)
                                                .corner_radius(4.0)
                                                .layout(|layout| layout.direction(TopToBottom))
                                                .children(|ui| {
                                                    ui.element()
                                                        .id((
                                                            "btn-nhc-text",
                                                            product.title.len() as u32,
                                                        ))
                                                        .width(grow!())
                                                        .height(fixed!(24.0))
                                                        .background_color(0x1E1B1B)
                                                        .corner_radius(3.0)
                                                        .layout(|layout| {
                                                            layout
                                                                .padding((0, 8, 0, 8))
                                                                .align(Left, CenterY)
                                                        })
                                                        .children(|ui| {
                                                            let arrow =
                                                                if false { '▾' } else { '▸' };
                                                            ui.text(
                                                                &format!(
                                                                    "{arrow} {}",
                                                                    product.title
                                                                ),
                                                                |t| t.font_size(11).color(0xE8E0DC),
                                                            );
                                                        });
                                                    // Show truncated content
                                                    ui.element()
                                                        .width(grow!())
                                                        .height(fixed!(100.0))
                                                        .background_color(0x12161e)
                                                        .corner_radius(3.0)
                                                        .layout(|layout| layout.padding(6))
                                                        .children(|ui| {
                                                            let truncated =
                                                                if product.content.len() > 300 {
                                                                    &product.content[..300]
                                                                } else {
                                                                    &product.content
                                                                };
                                                            ui.text(truncated, |t| {
                                                                t.font_size(9).color(0x9E9590)
                                                            });
                                                        });
                                                    ui.element()
                                                        .id((
                                                            "btn-nhc-open",
                                                            product.title.len() as u32,
                                                        ))
                                                        .width(fit!())
                                                        .height(fixed!(20.0))
                                                        .background_color(0x2a2a2a)
                                                        .corner_radius(3.0)
                                                        .layout(|layout| {
                                                            layout
                                                                .padding((0, 4, 0, 4))
                                                                .align(CenterX, CenterY)
                                                        })
                                                        .children(|ui| {
                                                            ui.text("🔗 Open in browser", |t| {
                                                                t.font_size(9).color(0x4a90d9)
                                                            });
                                                        });
                                                });
                                        }
                                    }
                                }
                            }
                        });
                }

                // ── Radar scope (transparent — drawn directly to screen) ──
                ui.element().width(grow!()).height(grow!()).empty();

                // ── Bottom status bar ──────────────────────────────
                ui.element()
                    .width(grow!())
                    .height(fixed!(24.0))
                    .background_color(0x12161e)
                    .layout(|layout| {
                        layout
                            .direction(LeftToRight)
                            .padding(8)
                            .gap(12)
                            .align(Left, CenterY)
                    })
                    .children(|ui| {
                        let has_real = state.scan.is_some();
                        let status_color = if has_real { 0x5F8A6A } else { 0x9E9590 };
                        ui.text(&state.status_text, |text| {
                            text.font_size(11).color(status_color)
                        });
                        for &(_threshold, color) in colors::DBZ_LEGEND.iter().step_by(2) {
                            let hex = (color[0] as u32) << 16
                                | (color[1] as u32) << 8
                                | (color[2] as u32);
                            ui.element()
                                .width(fixed!(14.0))
                                .height(fixed!(10.0))
                                .background_color(hex)
                                .empty();
                        }
                        ui.text("dBZ", |text| text.font_size(10).color(0x5F8A6A));
                    });
            });

        ui.show(|_| {}).await;

        // ── Input handling ─────────────────────────────────────────
        handle_input(&mut state, &ply, &site_options, &tilt_options);

        next_frame().await;
    }
}

// ---------------------------------------------------------------------------
// NHC panel helpers
// ---------------------------------------------------------------------------

/// Draw a compact toggle button for an NHC overlay layer.
fn nhc_toggle_button(ui: &mut Ui<'_, ()>, id: &'static str, label: &str, active: bool) {
    let bg = if active { 0x3F684A } else { 0x1E1B1B };
    let marker = if active { "✓" } else { " " };
    ui.element()
        .id(id)
        .width(grow!())
        .height(fixed!(22.0))
        .background_color(bg)
        .corner_radius(3.0)
        .layout(|layout| {
            layout
                .direction(LeftToRight)
                .padding((0, 6, 0, 6))
                .gap(6)
                .align(Left, CenterY)
        })
        .children(|ui| {
            ui.text(marker, |t| t.font_size(11).color(0xE8E0DC));
            ui.text(label, |t| t.font_size(11).color(0xE8E0DC));
        });
}

// ---------------------------------------------------------------------------
// Input handling
// ---------------------------------------------------------------------------

fn handle_input(
    state: &mut AppState,
    ply: &Ply<()>,
    site_options: &[DropdownOption],
    tilt_options: &[DropdownOption],
) {
    // The open dropdown gets first chance to consume keyboard and wheel input.
    let site_was_open = state.site_dropdown.is_open();
    let tilt_was_open = state.tilt_dropdown.is_open();

    if let Some(index) = state
        .site_dropdown
        .handle_input(ply, SITE_DROPDOWN, site_options)
    {
        state.tilt_dropdown.close();
        select_site(state, index);
    }
    if let Some(index) = state
        .tilt_dropdown
        .handle_input(ply, TILT_DROPDOWN, tilt_options)
    {
        state.site_dropdown.close();
        select_tilt(state, index);
    }

    if !site_was_open && state.site_dropdown.is_open() {
        state.tilt_dropdown.close();
    }
    if !tilt_was_open && state.tilt_dropdown.is_open() {
        state.site_dropdown.close();
    }

    let dropdown_open = state.site_dropdown.is_open() || state.tilt_dropdown.is_open();

    if !dropdown_open && is_mouse_button_down(MouseButton::Left) {
        let (mx, my) = mouse_position();
        if let Some((lx, ly)) = state.last_mouse_pos {
            let dx = mx - lx;
            let dy = my - ly;
            let side = screen_width().min(screen_height());
            let px_per_km = (side / 2.0) / scope::MAX_RANGE_KM * state.zoom;
            // Drag right moves content right; drag down moves content down.
            state.pan_km.0 += dx / px_per_km;
            state.pan_km.1 += dy / px_per_km;
        }
        state.last_mouse_pos = Some((mx, my));
    } else {
        state.last_mouse_pos = None;
    }

    if !dropdown_open {
        let scroll = mouse_wheel().1;
        if scroll != 0.0 {
            // 0.05 per unit = ~5-25% per wheel notch (vs old 0.001 = 0.1-0.5%)
            state.zoom = (state.zoom * (1.0 + scroll * 0.05)).clamp(0.05, 8.0);
        }
    }

    if !dropdown_open {
        if is_key_pressed(KeyCode::R) {
            select_product(state, Product::Reflectivity);
        }
        if is_key_pressed(KeyCode::V) {
            select_product(state, Product::Velocity);
        }
        if is_key_pressed(KeyCode::T) {
            let tilt_count = state
                .scan
                .as_ref()
                .map(|scan| scan.sweeps(state.product).len())
                .unwrap_or(0);
            if tilt_count > 0 {
                select_tilt(state, (state.tilt_index + 1) % tilt_count);
            }
        }
        if is_key_pressed(KeyCode::Key0) {
            state.pan_km = (0.0, 0.0);
            state.zoom = 1.0;
        }
        if is_key_pressed(KeyCode::Right) {
            select_site(state, (state.site_index + 1) % geo::RADAR_SITES.len());
        }
        if is_key_pressed(KeyCode::Left) {
            select_site(
                state,
                (state.site_index + geo::RADAR_SITES.len() - 1) % geo::RADAR_SITES.len(),
            );
        }
        if is_key_pressed(KeyCode::B) {
            state.show_borders = !state.show_borders;
        }
        if is_key_pressed(KeyCode::A) {
            state.show_alerts = !state.show_alerts;
        }
    }

    if let Some(product) = toggle::pressed(ply, &PRODUCT_OPTIONS) {
        select_product(state, product);
    }

    // ── Overlay toggle button presses (Stage 4) ──────────────────
    if ply.is_just_pressed("btn-borders") {
        state.show_borders = !state.show_borders;
    }
    if ply.is_just_pressed("btn-alerts") {
        state.show_alerts = !state.show_alerts;
    }

    // ── NHC toggle button and keyboard shortcut (Stage 5) ────────
    if ply.is_just_pressed("btn-nhc") {
        state.nhc_show_panel = !state.nhc_show_panel;
    }
    if !dropdown_open && is_key_pressed(KeyCode::N) {
        state.nhc_show_panel = !state.nhc_show_panel;
    }

    // ── NHC storm selector dropdown ──────────────────────────────
    if state.nhc_show_panel
        && let Some(ref bundle) = state.nhc_bundle
        && !bundle.metas.is_empty()
    {
        let storm_options: Vec<DropdownOption> = bundle
            .metas
            .iter()
            .enumerate()
            .map(|(i, m)| DropdownOption {
                source_index: i,
                label: format!("{} — {}", m.name, m.classification),
                search_text: format!("{} {}", m.name, m.classification),
            })
            .collect();
        if let Some(index) =
            state
                .nhc_storm_dropdown
                .handle_input(ply, STORM_DROPDOWN, &storm_options)
        {
            state.nhc_selected_storm = index;
        }
    }

    // ── NHC overlay toggle button presses ────────────────────────
    if ply.is_just_pressed("btn-nhc-cone") {
        state.nhc_overlays.show_cone = !state.nhc_overlays.show_cone;
    }
    if ply.is_just_pressed("btn-nhc-track") {
        state.nhc_overlays.show_track = !state.nhc_overlays.show_track;
    }
    if ply.is_just_pressed("btn-nhc-points") {
        state.nhc_overlays.show_points = !state.nhc_overlays.show_points;
    }
    if ply.is_just_pressed("btn-nhc-ww") {
        state.nhc_overlays.show_watches_warnings = !state.nhc_overlays.show_watches_warnings;
    }
    if ply.is_just_pressed("btn-nhc-wp") {
        state.nhc_overlays.show_wind_probs = !state.nhc_overlays.show_wind_probs;
    }
    if ply.is_just_pressed("btn-nhc-at") {
        state.nhc_overlays.show_arrival_times = !state.nhc_overlays.show_arrival_times;
    }

    // ── NHC external link buttons ────────────────────────────────
    if ply.is_just_pressed("btn-nhc-graphics")
        && let Some(ref bundle) = state.nhc_bundle
        && let Some(meta) = bundle.metas.get(state.nhc_selected_storm)
        && !meta.graphics_url.is_empty()
    {
        let _ = webbrowser::open(&meta.graphics_url);
    }
    if let Some(ref bundle) = state.nhc_bundle
        && let Some(meta) = bundle.metas.get(state.nhc_selected_storm)
        && let Some((_, images)) = bundle.image_products.iter().find(|(id, _)| *id == meta.id)
    {
        for img in images {
            let id = ("btn-nhc-img", img.title.len() as u32);
            if ply.is_just_pressed(id) {
                let _ = webbrowser::open(&img.url);
            }
        }
        if let Some((_, texts)) = bundle.text_products.iter().find(|(id, _)| *id == meta.id) {
            for product in texts {
                let id = ("btn-nhc-open", product.title.len() as u32);
                if ply.is_just_pressed(id) {
                    let _ = webbrowser::open(&product.url);
                }
            }
        }
    }
}

fn select_product(state: &mut AppState, product: Product) {
    if state.product != product {
        state.product = product;
        state.tilt_index = 0;
        state.needs_reraster = true;
        update_scan_status(state, "");
    }
}

fn select_tilt(state: &mut AppState, index: usize) {
    let tilt_count = state
        .scan
        .as_ref()
        .map(|scan| scan.sweeps(state.product).len())
        .unwrap_or(0);
    if index < tilt_count && state.tilt_index != index {
        state.tilt_index = index;
        state.needs_reraster = true;
        update_scan_status(state, "");
    }
}

fn select_site(state: &mut AppState, index: usize) {
    if index >= geo::RADAR_SITES.len() || index == state.site_index {
        return;
    }

    state.site_index = index;
    state.tilt_index = 0;
    state.tilt_dropdown.close();
    let site_id = geo::RADAR_SITES[index].id.to_string();
    let _ = state.site_tx.send(site_id.clone());
    state.scan = None;
    state.needs_reraster = true;
    state.status_text = format!("Switching to {site_id}…");
    state.pending_load = Some(state.cache.load_scan(&site_id));
}

fn update_scan_status(state: &mut AppState, suffix: &str) {
    if let Some(scan) = &state.scan {
        let sweeps = scan.sweeps(state.product);
        let elevation = sweeps
            .get(state.tilt_index)
            .map(|sweep| format!(" — {:.1}°", sweep.elevation_deg))
            .unwrap_or_default();
        state.status_text = format!(
            "{} — {} — {} tilt(s){}{}",
            scan.timestamp.format("%Y-%m-%d %H:%M UTC"),
            geo::RADAR_SITES[state.site_index].id,
            sweeps.len(),
            elevation,
            suffix,
        );
    }
}
