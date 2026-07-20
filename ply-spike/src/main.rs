//! rustywx — Ply radar scope (Stage 2: Live Data).
//!
//! Boots a window, fetches real NEXRAD radar data via a background worker,
//! caches scans to disk with Ply storage, and renders the scope with
//! pan/zoom and keyboard controls.

use ply_engine::prelude::*;
use rustywx::cache::Cache;
use rustywx::colors;
use rustywx::data::{self, WorkerMessage};
use rustywx::geo;
use rustywx::model::{Product, RadialData, SweepData};
use rustywx::scope;
use rustywx::state::AppState;
use rustywx::widgets::dropdown::{DropdownConfig, DropdownOption, DropdownState};
use rustywx::widgets::toggle::{self, ToggleOption};
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
        scope::draw_scope_to_texture(state.radar_texture.as_ref(), site, state.pan_km, state.zoom);

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
                    });

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
        let delta = mouse_delta_position();
        let side = screen_width().min(screen_height());
        let px_per_km = (side / 2.0) / scope::MAX_RANGE_KM * state.zoom;
        state.pan_km.0 += delta.x / px_per_km;
        state.pan_km.1 += delta.y / px_per_km;
    }

    if !dropdown_open {
        let scroll = mouse_wheel().1;
        if scroll != 0.0 {
            state.zoom = (state.zoom * (1.0 + scroll * 0.001)).clamp(0.05, 4.0);
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
    }

    if let Some(product) = toggle::pressed(ply, &PRODUCT_OPTIONS) {
        select_product(state, product);
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
