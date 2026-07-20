//! Ply spike: radar scope rendering proof-of-concept.
//! Generates synthetic radar data and renders it with overlays.

mod colors;
mod geo;
mod model;
mod scope;

use model::{Product, RadialData, SweepData};
use ply_engine::prelude::*;

// ---------------------------------------------------------------------------
// Synthetic radar data
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
            // Storm cells
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
            window_title: "rustywx — Ply Radar Scope Spike".to_owned(),
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
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[macroquad::main(window_conf)]
async fn main() {
    static DEFAULT_FONT: FontAsset = FontAsset::Path("assets/fonts/DejaVuSansMono.ttf");
    let mut ply = Ply::<()>::new(&DEFAULT_FONT).await;

    let sweep = synthetic_sweep();
    let site = &geo::RADAR_SITES[0];

    let mut state = AppState {
        site_index: 0,
        product: Product::Reflectivity,
        pan_km: (0.0, 0.0),
        zoom: 1.0,
        radar_texture: None,
        needs_reraster: true,
    };

    loop {
        clear_background(BLACK);

        // Rasterize when needed
        if state.needs_reraster {
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
                        ui.text(
                            &format!("{} — {}", site.id, site.name),
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

                // ── Radar scope ────────────────────────────────────
                ui.element()
                    .width(grow!())
                    .height(grow!())
                    .image(scope_tex)
                    .empty();

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
                        ui.text(
                            "Synthetic sweep — 0.5° — 360 radials — drag to pan, scroll to zoom",
                            |t| t.font_size(11).color(0x9E9590),
                        );
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
        handle_input(&mut state, &ply, site);

        next_frame().await;
    }
}

fn handle_input(state: &mut AppState, ply: &Ply<()>, _site: &geo::RadarSite) {
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
    if is_key_pressed(KeyCode::Key0) {
        state.pan_km = (0.0, 0.0);
        state.zoom = 1.0;
    }
    if is_key_pressed(KeyCode::Right) {
        state.site_index = (state.site_index + 1) % geo::RADAR_SITES.len();
    }
    if is_key_pressed(KeyCode::Left) {
        state.site_index =
            (state.site_index + geo::RADAR_SITES.len() - 1) % geo::RADAR_SITES.len();
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
