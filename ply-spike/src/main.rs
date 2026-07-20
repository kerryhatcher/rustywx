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
use std::sync::mpsc;

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
        dropdown_open: false,
        dropdown_filter: String::new(),
        dropdown_scroll: 0,
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
                if let Some(ref s) = state.scan {
                    state.status_text = format!(
                        "{} — {} — {} tilt(s) [cached]",
                        s.timestamp.format("%Y-%m-%d %H:%M UTC"),
                        geo::RADAR_SITES[state.site_index].id,
                        s.sweeps(state.product).len(),
                    );
                }
            } else {
                state.status_text = "Waiting for data…".to_string();
            }
        }

        // ── Poll worker messages ──────────────────────────────────
        while let Ok(msg) = state.worker_rx.try_recv() {
            match msg {
                WorkerMessage::NewScan(scan) => {
                    // Persist to disk cache (fire-and-forget).
                    let site_id = geo::RADAR_SITES[state.site_index].id.to_string();
                    state.cache.save_scan(&site_id, &scan);

                    state.scan = Some(*scan);
                    state.tilt_index = 0;
                    state.needs_reraster = true;
                    if let Some(ref s) = state.scan {
                        state.status_text = format!(
                            "{} — {} — {} tilt(s)",
                            s.timestamp.format("%Y-%m-%d %H:%M UTC"),
                            geo::RADAR_SITES[state.site_index].id,
                            s.sweeps(state.product).len(),
                        );
                    }
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

        // ── Ply UI ─────────────────────────────────────────────────
        let mut ui = ply.begin();

        ui.element()
            .width(grow!())
            .height(grow!())
            .layout(|l| l.direction(TopToBottom))
            .children(|ui| {
                // ── Top controls bar ───────────────────────────────
                ui.element()
                    .width(grow!())
                    .height(fixed!(36.0))
                    .background_color(0x12161e)
                    .layout(|l| {
                        l.direction(LeftToRight)
                            .padding(8)
                            .gap(12)
                            .align(Left, CenterY)
                    })
                    .children(|ui| {
                        // Site selector dropdown button
                        ui.element()
                            .id("site-dropdown-btn")
                            .width(fit!())
                            .height(fixed!(24.0))
                            .background_color(0x1E1B1B)
                            .corner_radius(4.0)
                            .layout(|l| l.padding((0, 8, 0, 8)).align(CenterX, CenterY))
                            .children(|ui| {
                                ui.text(&format!("{} ▾", site.id), |t| {
                                    t.font_size(13).color(0xE8E0DC)
                                });
                            });

                        ui.text(&format!(" — {}", site.name), |t| {
                            t.font_size(14).color(0xE8E0DC)
                        });

                        // Reflectivity button
                        let refl_bg = if state.product == Product::Reflectivity {
                            0x3A3533
                        } else {
                            0x1E1B1B
                        };
                        ui.element()
                            .id("btn-refl")
                            .width(fit!())
                            .height(fixed!(24.0))
                            .background_color(refl_bg)
                            .corner_radius(4.0)
                            .layout(|l| l.padding((0, 8, 0, 8)).align(CenterX, CenterY))
                            .children(|ui| {
                                ui.text("Reflectivity", |t| t.font_size(12).color(0xE8E0DC));
                            });

                        // Velocity button
                        let vel_bg = if state.product == Product::Velocity {
                            0x3A3533
                        } else {
                            0x1E1B1B
                        };
                        ui.element()
                            .id("btn-vel")
                            .width(fit!())
                            .height(fixed!(24.0))
                            .background_color(vel_bg)
                            .corner_radius(4.0)
                            .layout(|l| l.padding((0, 8, 0, 8)).align(CenterX, CenterY))
                            .children(|ui| {
                                ui.text("Velocity", |t| t.font_size(12).color(0xE8E0DC));
                            });

                        ui.text(
                            &format!(
                                "Zoom: {:.1}x  Pan: ({:.0}, {:.0}) km",
                                state.zoom, state.pan_km.0, state.pan_km.1
                            ),
                            |t| t.font_size(11).color(0x9E9590),
                        );
                    });

                // ── Site dropdown panel ─────────────────────────────
                if state.dropdown_open {
                    let filter = state.dropdown_filter.to_lowercase();
                    let filtered: Vec<(usize, &geo::RadarSite)> = geo::RADAR_SITES
                        .iter()
                        .enumerate()
                        .filter(|(_, s)| {
                            filter.is_empty()
                                || s.id.to_lowercase().contains(&filter)
                                || s.name.to_lowercase().contains(&filter)
                        })
                        .collect();

                    let visible_count = 12usize;
                    let max_scroll = filtered.len().saturating_sub(visible_count);
                    let scroll = state.dropdown_scroll.min(max_scroll);
                    let visible = &filtered[scroll..(scroll + visible_count).min(filtered.len())];

                    ui.element()
                        .id("site-dropdown-panel")
                        .width(fixed!(220.0))
                        .height(fixed!(300.0))
                        .background_color(0xDD1A1D)
                        .corner_radius(6.0)
                        .floating(|f| f.offset((8.0, 44.0)).z_index(100))
                        .layout(|l| l.direction(TopToBottom).padding(4).gap(2))
                        .children(|ui| {
                            ui.text(&format!("Filter: {}_", state.dropdown_filter), |t| {
                                t.font_size(11).color(0x9E9590)
                            });
                            for &(idx, site) in visible {
                                let bg = if idx == state.site_index {
                                    0x3A3533
                                } else {
                                    0x00000000
                                };
                                ui.element()
                                    .id(("site-opt", idx as u32))
                                    .width(grow!())
                                    .height(fixed!(22.0))
                                    .background_color(bg)
                                    .corner_radius(3.0)
                                    .layout(|l| l.padding((0, 6, 0, 6)).align(Left, CenterY))
                                    .children(|ui| {
                                        ui.text(&format!("{} — {}", site.id, site.name), |t| {
                                            t.font_size(12).color(0xE8E0DC)
                                        });
                                    });
                            }
                            if filtered.len() > visible_count {
                                ui.text(
                                    &format!("{} more…", filtered.len() - visible_count),
                                    |t| t.font_size(10).color(0x5F8A6A),
                                );
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
                    .layout(|l| {
                        l.direction(LeftToRight)
                            .padding(8)
                            .gap(12)
                            .align(Left, CenterY)
                    })
                    .children(|ui| {
                        let has_real = state.scan.is_some();
                        let status_color = if has_real { 0x5F8A6A } else { 0x9E9590 };
                        ui.text(&state.status_text, |t| t.font_size(11).color(status_color));
                        // Color legend swatches
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
                        ui.text("dBZ", |t| t.font_size(10).color(0x5F8A6A));
                    });
            });

        ui.show(|_| {}).await;

        // ── Input handling ─────────────────────────────────────────
        handle_input(&mut state, &ply);

        next_frame().await;
    }
}

// ---------------------------------------------------------------------------
// Input handling
// ---------------------------------------------------------------------------

fn handle_input(state: &mut AppState, ply: &Ply<()>) {
    // ── Dropdown keyboard handling ────────────────────────────
    if state.dropdown_open {
        if let Some(c) = get_char_pressed()
            && (c.is_ascii_alphanumeric() || c == ' ' || c == '-')
        {
            state.dropdown_filter.push(c);
            state.dropdown_scroll = 0;
        }
        if is_key_pressed(KeyCode::Backspace) && !state.dropdown_filter.is_empty() {
            state.dropdown_filter.pop();
            state.dropdown_scroll = 0;
        }
        if is_key_pressed(KeyCode::Escape) {
            state.dropdown_open = false;
            state.dropdown_filter.clear();
        }
        if is_key_pressed(KeyCode::Down) {
            state.dropdown_scroll += 1;
        }
        if is_key_pressed(KeyCode::Up) {
            state.dropdown_scroll = state.dropdown_scroll.saturating_sub(1);
        }
        if is_key_pressed(KeyCode::Enter) {
            let filter = state.dropdown_filter.to_lowercase();
            if let Some((idx, _)) = geo::RADAR_SITES.iter().enumerate().find(|(_, s)| {
                filter.is_empty()
                    || s.id.to_lowercase().contains(&filter)
                    || s.name.to_lowercase().contains(&filter)
            }) {
                state.site_index = idx;
                let site_id = geo::RADAR_SITES[idx].id.to_string();
                let _ = state.site_tx.send(site_id.clone());
                state.scan = None;
                state.needs_reraster = true;
                state.status_text = format!("Switching to {}…", site_id);
                state.pending_load = Some(state.cache.load_scan(&site_id));
            }
            state.dropdown_open = false;
            state.dropdown_filter.clear();
        }
    }

    // ── Dropdown button click ─────────────────────────────────
    if ply.is_just_pressed("site-dropdown-btn") {
        state.dropdown_open = !state.dropdown_open;
        if state.dropdown_open {
            state.dropdown_filter.clear();
            state.dropdown_scroll = 0;
        }
    }

    // ── Dropdown option clicks ────────────────────────────────
    if state.dropdown_open {
        for (idx, _site) in geo::RADAR_SITES.iter().enumerate() {
            if ply.is_just_pressed(("site-opt", idx as u32)) {
                state.site_index = idx;
                let site_id = geo::RADAR_SITES[idx].id.to_string();
                let _ = state.site_tx.send(site_id.clone());
                state.scan = None;
                state.needs_reraster = true;
                state.status_text = format!("Switching to {}…", site_id);
                state.pending_load = Some(state.cache.load_scan(&site_id));
                state.dropdown_open = false;
                state.dropdown_filter.clear();
                break;
            }
        }
    }

    // ── Outside click closes dropdown ─────────────────────────
    if state.dropdown_open && is_mouse_button_pressed(MouseButton::Left) {
        let panel_pressed = ply.is_just_pressed("site-dropdown-panel");
        let btn_pressed = ply.is_just_pressed("site-dropdown-btn");
        if !panel_pressed && !btn_pressed {
            state.dropdown_open = false;
            state.dropdown_filter.clear();
        }
    }

    // Mouse drag for panning
    if is_mouse_button_down(MouseButton::Left) {
        let delta = mouse_delta_position();
        let side = screen_width().min(screen_height());
        let px_per_km = (side / 2.0) / scope::MAX_RANGE_KM * state.zoom;
        state.pan_km.0 += delta.x / px_per_km;
        state.pan_km.1 += delta.y / px_per_km;
    }

    // Scroll for zoom
    let scroll = mouse_wheel().1;
    if scroll != 0.0 {
        state.zoom = (state.zoom * (1.0 + scroll * 0.001)).clamp(0.05, 4.0);
    }

    // Keyboard shortcuts
    if is_key_pressed(KeyCode::R) {
        state.product = Product::Velocity;
        state.needs_reraster = true;
    }
    if is_key_pressed(KeyCode::V) {
        state.product = Product::Reflectivity;
        state.needs_reraster = true;
    }
    if is_key_pressed(KeyCode::T)
        && let Some(ref scan) = state.scan
    {
        let sweeps = scan.sweeps(state.product);
        if !sweeps.is_empty() {
            state.tilt_index = (state.tilt_index + 1) % sweeps.len();
            state.needs_reraster = true;
        }
    }
    if is_key_pressed(KeyCode::Key0) {
        state.pan_km = (0.0, 0.0);
        state.zoom = 1.0;
    }
    if is_key_pressed(KeyCode::Right) && !state.dropdown_open {
        state.site_index = (state.site_index + 1) % geo::RADAR_SITES.len();
        let site_id = geo::RADAR_SITES[state.site_index].id.to_string();
        let _ = state.site_tx.send(site_id.clone());
        state.scan = None;
        state.needs_reraster = true;
        state.status_text = format!("Switching to {}…", site_id);
        state.pending_load = Some(state.cache.load_scan(&site_id));
    }
    if is_key_pressed(KeyCode::Left) && !state.dropdown_open {
        state.site_index = (state.site_index + geo::RADAR_SITES.len() - 1) % geo::RADAR_SITES.len();
        let site_id = geo::RADAR_SITES[state.site_index].id.to_string();
        let _ = state.site_tx.send(site_id.clone());
        state.scan = None;
        state.needs_reraster = true;
        state.status_text = format!("Switching to {}…", site_id);
        state.pending_load = Some(state.cache.load_scan(&site_id));
    }

    // Ply button presses
    if ply.is_just_pressed("btn-refl") {
        state.product = Product::Reflectivity;
        state.needs_reraster = true;
    }
    if ply.is_just_pressed("btn-vel") {
        state.product = Product::Velocity;
        state.needs_reraster = true;
    }
}
