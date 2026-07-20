//! Ply spike: radar scope rendering with real NEXRAD data.
//! Falls back to synthetic data until the first real scan arrives.

mod colors;
mod data;
mod geo;
mod model;
mod scope;

use model::{Product, ScanData, SweepData};
use ply_engine::prelude::*;
use ply_engine::shaders::ShaderAsset;
use std::sync::mpsc;
use tokio::sync::oneshot;

// ---------------------------------------------------------------------------
// Custom blur shader for frosted glass effect (Spike S1)
// ---------------------------------------------------------------------------

static BLUR_SHADER: ShaderAsset = ShaderAsset::Source {
    file_name: "blur",
    fragment: include_str!("../assets/shaders/blur.frag"),
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
        radials.push(model::RadialData {
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
            window_title: "rustywx — Ply Radar Scope Spike (Live Data)".to_owned(),
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
        draw_call_vertex_capacity: 100000,
        draw_call_index_capacity: 100000,
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

struct AppState {
    site_index: usize,
    product: Product,
    pan_km: (f32, f32),
    zoom: f32,
    radar_texture: Option<Texture2D>,
    needs_reraster: bool,
    // Real data
    scan: Option<ScanData>,
    tilt_index: usize,
    status_text: String,
    // Worker channels
    worker_rx: mpsc::Receiver<data::WorkerMessage>,
    site_tx: mpsc::Sender<String>,
    // Glass panel toggle (Spike S1)
    show_glass: bool,
    // Dropdown state (Spike S3)
    dropdown_open: bool,
    dropdown_filter: String,
    dropdown_scroll: usize,
    // Texture stress test (Spike S4)
    stress_test: bool,
    stress_frame_count: u32,
    // Storage test (Spike S8)
    storage_test_result: Option<String>,
    pending_storage_load: Option<oneshot::Receiver<Option<Vec<u8>>>>,
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[macroquad::main(window_conf)]
async fn main() {
    static DEFAULT_FONT: FontAsset = FontAsset::Path("assets/fonts/DejaVuSansMono.ttf");
    let mut ply = Ply::<()>::new(&DEFAULT_FONT).await;

    let initial_site = &geo::RADAR_SITES[0];

    // Spawn background worker
    let (worker_tx, worker_rx) = mpsc::channel();
    let (site_tx, site_rx) = mpsc::channel();
    data::spawn_worker(worker_tx, initial_site.id.to_string(), site_rx);

    let mut state = AppState {
        site_index: 0,
        product: Product::Reflectivity,
        pan_km: (0.0, 0.0),
        zoom: 1.0,
        radar_texture: None,
        needs_reraster: true,
        scan: None,
        tilt_index: 0,
        status_text: "Starting…".to_string(),
        worker_rx,
        site_tx,
        show_glass: true,
        dropdown_open: false,
        dropdown_filter: String::new(),
        dropdown_scroll: 0,
        stress_test: false,
        stress_frame_count: 0,
        storage_test_result: None,
        pending_storage_load: None,
    };

    loop {
        clear_background(BLACK);

        // ── Poll worker messages ──────────────────────────────────
        while let Ok(msg) = state.worker_rx.try_recv() {
            match msg {
                data::WorkerMessage::NewScan(scan) => {
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
                data::WorkerMessage::Status(s) => {
                    state.status_text = s;
                }
                data::WorkerMessage::Error(e) => {
                    state.status_text = format!("Error: {e}");
                }
            }
        }

        // ── Poll net responses (Spike S6) ─────────────────────────
        for id in ["spike-test-a", "spike-test-b", "spike-test-c"] {
            if let Some(req) = ply_engine::net::request(id) {
                match req.response() {
                    None => { /* still loading */ }
                    Some(Ok(resp)) => {
                        let status = resp.status();
                        if status == 200 {
                            let body = resp.text();
                            let preview: String = body.chars().take(60).collect();
                            state.status_text = format!(
                                "Net {id}: {status} — {preview}…"
                            );
                        } else {
                            state.status_text = format!("Net {id}: HTTP {status}");
                        }
                    }
                    Some(Err(e)) => {
                        state.status_text = format!("Net {id}: error — {e:?}");
                    }
                }
            }
        }

        // ── Poll storage load result (Spike S8) ───────────────────
        if let Some(rx) = &mut state.pending_storage_load {
            match rx.try_recv() {
                Ok(Some(data)) => {
                    let data_str = String::from_utf8_lossy(&data).into_owned();
                    state.storage_test_result = Some(format!("Loaded: {}", data_str));
                    state.status_text = "Storage test: SUCCESS ✓".to_string();
                    state.pending_storage_load = None;
                }
                Ok(None) => {
                    state.storage_test_result = Some("Load failed".to_string());
                    state.status_text = "Storage test: FAILED ✗".to_string();
                    state.pending_storage_load = None;
                }
                Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {
                    // Still loading
                    state.status_text = "Storage test: loading...".to_string();
                }
                Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                    state.storage_test_result = Some("Channel closed".to_string());
                    state.status_text = "Storage test: channel closed".to_string();
                    state.pending_storage_load = None;
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

        // Rasterize when needed (or every frame in stress test)
        if state.needs_reraster || state.stress_test {
            state.needs_reraster = false;
            if state.stress_test {
                state.stress_frame_count += 1;
            }
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
            state.needs_reraster = false;
        }

        // Render scope + overlays to texture
        let scope_tex = render_to_texture(screen_width(), screen_height(), || {
            scope::draw_scope_to_texture(
                state.radar_texture.as_ref(),
                site,
                state.pan_km,
                state.zoom,
            );
        });

        // ── Ply UI ─────────────────────────────────────────────────
        let mut ui = ply.begin();

        ui.element()
            .width(grow!())
            .height(grow!())
            .background_color(0x06090e)
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
                        // ── Site selector dropdown button ────────
                        ui.element()
                            .id("site-dropdown-btn")
                            .width(fit!())
                            .height(fixed!(24.0))
                            .background_color(0x1E1B1B)
                            .corner_radius(4.0)
                            .layout(|l| l.padding((0, 8, 0, 8)).align(CenterX, CenterY))
                            .children(|ui| {
                                ui.text(
                                    &format!("{} ▾", site.id),
                                    |t| t.font_size(13).color(0xE8E0DC),
                                );
                            });

                        ui.text(
                            &format!(" — {}", site.name),
                            |t| t.font_size(14).color(0xE8E0DC),
                        );

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

                // ── Site dropdown panel (Spike S3) ────────────
                if state.dropdown_open {
                    // Build filtered site list
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
                        .floating(|f| {
                            f.offset((8.0, 44.0)).z_index(100)
                        })
                        .layout(|l| {
                            l.direction(TopToBottom)
                                .padding(4)
                                .gap(2)
                        })
                        .children(|ui| {
                            // Filter hint
                            ui.text(
                                &format!("Filter: {}_", state.dropdown_filter),
                                |t| t.font_size(11).color(0x9E9590),
                            );
                            // Site list
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
                                        ui.text(
                                            &format!("{} — {}", site.id, site.name),
                                            |t| t.font_size(12).color(0xE8E0DC),
                                        );
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

                // ── Radar scope ────────────────────────────────────
                ui.element()
                    .width(grow!())
                    .height(grow!())
                    .image(scope_tex)
                    .empty();

                // ── Frosted glass test panel (Spike S1) ───────────
                // Floating panel over the radar scope with blur shader
                if state.show_glass {
                    ui.element()
                        .id("glass-panel")
                        .width(fixed!(220.0))
                        .height(fixed!(160.0))
                        .background_color(0x1A_FFFFFF) // semi-transparent white
                        .corner_radius(12.0)
                        .shader(&BLUR_SHADER, |s| {
                            s.uniform("u_radius", 8.0);
                        })
                        .floating(|f| {
                            f.offset((20.0, 60.0)).z_index(10)
                        })
                        .layout(|l| {
                            l.direction(TopToBottom)
                                .padding(16)
                                .gap(8)
                                .align(Left, Top)
                            })
                        .children(|ui| {
                            ui.text("Frosted Glass Test", |t| {
                                t.font_size(14).color(0xFFFFFF)
                            });
                            ui.text("This panel uses a custom GLSL", |t| {
                                t.font_size(11).color(0xCC_DDDDD)
                            });
                            ui.text("Gaussian blur shader to create", |t| {
                                t.font_size(11).color(0xCC_DDDDD)
                            });
                            ui.text("the frosted glass effect.", |t| {
                                t.font_size(11).color(0xCC_DDDDD)
                            });
                            ui.text("Press G to toggle glass panel", |t| {
                                t.font_size(10).color(0x889999)
                            });
                        });
                }

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
                        let stress_info = if state.stress_test {
                            format!(" [STRESS: {} frames]", state.stress_frame_count)
                        } else {
                            String::new()
                        };
                        ui.text(
                            &format!("{}{}", state.status_text, stress_info),
                            |t| t.font_size(11).color(status_color),
                        );
                        // Color legend swatches
                        for &(_threshold, color) in colors::DBZ_LEGEND.iter().step_by(2) {
                            let hex =
                                (color[0] as u32) << 16 | (color[1] as u32) << 8 | (color[2] as u32);
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
        handle_input(&mut state, &ply, site);

        next_frame().await;
    }
}

fn handle_input(state: &mut AppState, ply: &Ply<()>, _site: &geo::RadarSite) {
    // ── Dropdown keyboard handling ────────────────────────────
    if state.dropdown_open {
        // Character input for filter
        if let Some(c) = get_char_pressed() {
            if c.is_ascii_alphanumeric() || c == ' ' || c == '-' {
                state.dropdown_filter.push(c);
                state.dropdown_scroll = 0;
            }
        }
        // Backspace
        if is_key_pressed(KeyCode::Backspace) && !state.dropdown_filter.is_empty() {
            state.dropdown_filter.pop();
            state.dropdown_scroll = 0;
        }
        // Escape to close
        if is_key_pressed(KeyCode::Escape) {
            state.dropdown_open = false;
            state.dropdown_filter.clear();
        }
        // Arrow keys for scroll
        if is_key_pressed(KeyCode::Down) {
            state.dropdown_scroll += 1;
        }
        if is_key_pressed(KeyCode::Up) {
            state.dropdown_scroll = state.dropdown_scroll.saturating_sub(1);
        }
        // Enter to select first visible
        if is_key_pressed(KeyCode::Enter) {
            let filter = state.dropdown_filter.to_lowercase();
            if let Some((idx, _)) = geo::RADAR_SITES.iter().enumerate().find(|(_, s)| {
                filter.is_empty()
                    || s.id.to_lowercase().contains(&filter)
                    || s.name.to_lowercase().contains(&filter)
            }) {
                state.site_index = idx;
                let _ = state.site_tx.send(geo::RADAR_SITES[idx].id.to_string());
                state.scan = None;
                state.needs_reraster = true;
                state.status_text = format!("Switching to {}…", geo::RADAR_SITES[idx].id);
            }
            state.dropdown_open = false;
            state.dropdown_filter.clear();
        }
        // Don't process other input while dropdown is open
        // (except mouse for clicking options)
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
                let _ = state.site_tx.send(geo::RADAR_SITES[idx].id.to_string());
                state.scan = None;
                state.needs_reraster = true;
                state.status_text = format!("Switching to {}…", geo::RADAR_SITES[idx].id);
                state.dropdown_open = false;
                state.dropdown_filter.clear();
                break;
            }
        }
    }

    // ── Outside click closes dropdown ─────────────────────────
    if state.dropdown_open && is_mouse_button_pressed(MouseButton::Left) {
        // Check if click is on the dropdown panel or button
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
        state.product = Product::Reflectivity;
        state.needs_reraster = true;
    }
    if is_key_pressed(KeyCode::V) {
        state.product = Product::Velocity;
        state.needs_reraster = true;
    }
    if is_key_pressed(KeyCode::T) {
        // Cycle tilt
        if let Some(ref scan) = state.scan {
            let sweeps = scan.sweeps(state.product);
            if !sweeps.is_empty() {
                state.tilt_index = (state.tilt_index + 1) % sweeps.len();
                state.needs_reraster = true;
            }
        }
    }
    if is_key_pressed(KeyCode::G) {
        state.show_glass = !state.show_glass;
    }
    // Texture stress test toggle (Spike S4)
    if is_key_pressed(KeyCode::F5) {
        state.stress_test = !state.stress_test;
        state.stress_frame_count = 0;
        state.needs_reraster = true;
    }
    // Storage test (Spike S8) — press F6 to save/load test
    if is_key_pressed(KeyCode::F6) {
        // Test Pattern 1: Direct await for small data (settings)
        // This is acceptable because settings are <2ms
        state.status_text = "Storage test: saving...".to_string();

        // Spawn a storage task to demonstrate the channel pattern
        let (tx, rx) = oneshot::channel::<Option<Vec<u8>>>();
        tokio::spawn(async move {
            match Storage::new("rustywx-spike/test").await {
                Ok(storage) => {
                    // Save test
                    let test_data = b"test radar scan metadata {\"site\":\"KJGX\",\"time\":\"2025-07-19T12:00:00Z\"}";
                    match storage.save_bytes("test-scan", test_data).await {
                        Ok(_) => {
                            // Load test
                            match storage.load_bytes("test-scan").await {
                                Ok(Some(loaded)) => {
                                    let loaded_str = String::from_utf8_lossy(&loaded).into_owned();
                                    let _ = tx.send(Some(loaded));
                                    log::info!("Storage test: saved and loaded: {}", loaded_str);
                                }
                                Ok(None) => {
                                    log::warn!("Storage test: key not found");
                                    let _ = tx.send(None);
                                }
                                Err(e) => {
                                    log::error!("Storage test load error: {:?}", e);
                                    let _ = tx.send(None);
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Storage test save error: {:?}", e);
                            let _ = tx.send(None);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Storage test init error: {:?}", e);
                    let _ = tx.send(None);
                }
            }
        });
        state.pending_storage_load = Some(rx);
    }
    // Net concurrent test (Spike S6) — press F7 to fire 3 requests
    if is_key_pressed(KeyCode::F7) {
        ply_engine::net::get("spike-test-a", "https://httpbin.org/delay/0", |r| r);
        ply_engine::net::get("spike-test-b", "https://httpbin.org/delay/1", |r| r);
        ply_engine::net::get("spike-test-c", "https://httpbin.org/status/404", |r| r);
        state.status_text = "Net test: fired 3 concurrent requests".to_string();
    }
    if is_key_pressed(KeyCode::Key0) {
        state.pan_km = (0.0, 0.0);
        state.zoom = 1.0;
    }
    if is_key_pressed(KeyCode::Right) {
        state.site_index = (state.site_index + 1) % geo::RADAR_SITES.len();
        let _ = state
            .site_tx
            .send(geo::RADAR_SITES[state.site_index].id.to_string());
        state.scan = None;
        state.needs_reraster = true;
        state.status_text = format!(
            "Switching to {}…",
            geo::RADAR_SITES[state.site_index].id
        );
    }
    if is_key_pressed(KeyCode::Left) {
        state.site_index =
            (state.site_index + geo::RADAR_SITES.len() - 1) % geo::RADAR_SITES.len();
        let _ = state
            .site_tx
            .send(geo::RADAR_SITES[state.site_index].id.to_string());
        state.scan = None;
        state.needs_reraster = true;
        state.status_text = format!(
            "Switching to {}…",
            geo::RADAR_SITES[state.site_index].id
        );
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
