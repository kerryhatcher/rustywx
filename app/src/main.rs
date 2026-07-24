//! rustywx — Ply radar scope (Stage 2: Live Data).
//!
//! Boots a window, fetches real NEXRAD radar data via a background worker,
//! caches scans to disk with Ply storage, and renders the scope with
//! pan/zoom and keyboard controls.

use ply_engine::prelude::*;
use ply_engine::render_commands::{RenderCommand, RenderCommandConfig};
use rustywx::alerts;
use rustywx::borders;
use rustywx::cache::Cache;
use rustywx::colors;
use rustywx::data::{self, WorkerMessage};
use rustywx::forecast;
use rustywx::geo;
use rustywx::melting_layer;
use rustywx::model::{Product, RadialData, SweepData, format_nyquist_velocity, vcp_mode_label};
use rustywx::nhc;
use rustywx::scope;
use rustywx::settings::{AnimationLevel, Settings};
use rustywx::state::ViewMode;
use rustywx::state::{AlertModal, AppState, NhcModal};
use rustywx::widgets::dropdown::{DropdownConfig, DropdownOption, DropdownState};
use rustywx::widgets::glass_panel;
use rustywx::widgets::settings as settings_widget;
use rustywx::widgets::shortcuts as shortcuts_widget;
use rustywx::widgets::toast as toast_widget;
use rustywx::widgets::toggle::{self, ToggleOption};
use rustywx::widgets::{ChartWidget, SYMBOL_FONT, nf};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
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

// Product buttons stack vertically in the radar side panel (TopToBottom
// layout, panel scrolls if needed) rather than sitting in a horizontal row,
// so seven buttons just add rows — no width-clipping concern.
const PRODUCT_OPTIONS: [ToggleOption<Product>; 7] = [
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
    ToggleOption {
        id: "btn-sw",
        label: "Spectrum Width",
        value: Product::SpectrumWidth,
    },
    ToggleOption {
        id: "btn-zdr",
        label: "ZDR",
        value: Product::DifferentialReflectivity,
    },
    ToggleOption {
        id: "btn-cc",
        label: "CC",
        value: Product::CorrelationCoefficient,
    },
    ToggleOption {
        id: "btn-phidp",
        label: "PhiDP",
        value: Product::DifferentialPhase,
    },
    ToggleOption {
        id: "btn-kdp",
        label: "KDP",
        value: Product::SpecificDifferentialPhase,
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

const NHC_MODAL_LINE_HEIGHT: f32 = 14.0;
const NHC_MODAL_TEXT_COLUMNS: usize = 82;

/// Build the full alert-detail modal body: headline, full NWS description, and
/// precautionary instruction, skipping any part the feed left empty.
fn alert_modal_body(alert: &rustywx::alerts::Alert) -> String {
    let mut parts: Vec<String> = Vec::new();
    if !alert.headline.is_empty() {
        parts.push(alert.headline.clone());
    }
    if !alert.description.is_empty() {
        parts.push(alert.description.clone());
    }
    if !alert.instruction.is_empty() {
        parts.push(format!(
            "PRECAUTIONARY/PREPAREDNESS ACTIONS:\n\n{}",
            alert.instruction
        ));
    }
    parts.join("\n\n")
}

fn wrap_modal_text(content: &str) -> Vec<String> {
    let mut output = Vec::new();
    for source_line in content.lines() {
        if source_line.is_empty() {
            output.push(String::new());
            continue;
        }

        let mut remaining = source_line;
        while remaining.chars().count() > NHC_MODAL_TEXT_COLUMNS {
            let byte_limit = remaining
                .char_indices()
                .nth(NHC_MODAL_TEXT_COLUMNS)
                .map(|(index, _)| index)
                .unwrap_or(remaining.len());
            let candidate = &remaining[..byte_limit];
            let split = candidate
                .rfind(char::is_whitespace)
                .filter(|&index| index > 0)
                .unwrap_or(byte_limit);
            output.push(remaining[..split].trim_end().to_string());
            remaining = remaining[split..].trim_start();
        }
        output.push(remaining.to_string());
    }
    output
}

fn nhc_modal_text_metrics(state: &AppState) -> Option<(usize, usize, f32)> {
    let NhcModal::Text { content, .. } = &state.nhc_modal else {
        return None;
    };
    let modal_h = screen_height() * 0.7;
    let content_h = modal_h - 36.0 - 40.0 - 24.0;
    let lines = wrap_modal_text(content);
    let visible = (content_h / NHC_MODAL_LINE_HEIGHT).floor().max(1.0) as usize;
    let max_first = lines.len().saturating_sub(visible);
    Some((
        lines.len(),
        visible,
        max_first as f32 * NHC_MODAL_LINE_HEIGHT,
    ))
}

fn alert_modal_text_metrics(state: &AppState) -> Option<(usize, usize, f32)> {
    let alert_modal = state.alert_modal.as_ref()?;
    let modal_h = screen_height() * 0.7;
    let content_h = modal_h - 36.0 - 24.0;
    let lines = wrap_modal_text(&alert_modal.content);
    let visible = (content_h / NHC_MODAL_LINE_HEIGHT).floor().max(1.0) as usize;
    let max_first = lines.len().saturating_sub(visible);
    Some((
        lines.len(),
        visible,
        max_first as f32 * NHC_MODAL_LINE_HEIGHT,
    ))
}

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
            range_folded: vec![],
        });
    }
    SweepData {
        elevation_deg: 0.5,
        radials,
        first_gate_km: scope::FIRST_GATE_KM,
        gate_spacing_km: scope::GATE_SPACING_KM,
        nyquist_ms: 0.0,
    }
}

// ---------------------------------------------------------------------------
// Stage 6: Observatory visual helpers
// ---------------------------------------------------------------------------

/// Ply custom-draw handler for the hourly rain-chance chart. Invoked by
/// `ui.show(draw_rain_chart)` in z-order alongside the rest of the UI, so it
/// draws directly with macroquad (real `draw_text` axis labels, crisp lines)
/// instead of the old rasterize-to-texture approach.
fn draw_rain_chart(cmd: &RenderCommand<ChartWidget>) {
    let RenderCommandConfig::Custom(c) = &cmd.config else {
        return;
    };
    let ChartWidget::RainChart(hours) = &c.data else {
        return;
    };
    if hours.is_empty() {
        return;
    }
    let n = hours.len();
    let bb = cmd.bounding_box;

    let bg = MacroquadColor::new(0.118, 0.106, 0.106, 1.0);
    let grid = MacroquadColor::new(1.0, 1.0, 1.0, 0.10);
    let gray = MacroquadColor::new(0.6, 0.58, 0.56, 1.0);
    let fill = MacroquadColor::new(0.31, 0.56, 0.88, 0.45);
    let line_color = MacroquadColor::new(0.75, 0.86, 1.0, 1.0);
    let teal = MacroquadColor::new(0.05, 0.77, 0.72, 0.6);

    draw_rectangle(bb.x, bb.y, bb.width, bb.height, bg);

    // Plot area inset: room for % labels (left), day labels (top), hour
    // ticks (bottom).
    let left = bb.x + 30.0;
    let right = bb.x + bb.width - 10.0;
    let top = bb.y + 16.0;
    let bottom = bb.y + bb.height - 18.0;
    let plot_w = (right - left).max(1.0);
    let plot_h = (bottom - top).max(1.0);

    // Gridlines + % labels.
    for pct in [0i64, 25, 50, 75, 100] {
        let y = bottom - (pct as f32 / 100.0) * plot_h;
        draw_line(left, y, right, y, 1.0, grid);
        let label = format!("{pct}");
        let dims = measure_text(&label, None, 11, 1.0);
        draw_text(
            &label,
            left - 6.0 - dims.width,
            y + dims.height / 2.0 - 2.0,
            11.0,
            gray,
        );
    }

    // x/y for hour i (n==1 guarded: single point centered on the plot).
    let x_at = |i: usize| -> f32 {
        if n > 1 {
            left + (i as f32 / (n - 1) as f32) * plot_w
        } else {
            left + plot_w / 2.0
        }
    };
    let y_at = |pct: i64| -> f32 {
        let p = pct.clamp(0, 100) as f32;
        bottom - (p / 100.0) * plot_h
    };
    let baseline = y_at(0);

    // Area fill under the curve.
    for i in 0..n.saturating_sub(1) {
        let (x0, y0) = (x_at(i), y_at(hours[i].precip_pct));
        let (x1, y1) = (x_at(i + 1), y_at(hours[i + 1].precip_pct));
        draw_triangle(
            Vec2::new(x0, y0),
            Vec2::new(x1, y1),
            Vec2::new(x0, baseline),
            fill,
        );
        draw_triangle(
            Vec2::new(x1, y1),
            Vec2::new(x1, baseline),
            Vec2::new(x0, baseline),
            fill,
        );
    }

    // Line + dots.
    for i in 0..n.saturating_sub(1) {
        let (x0, y0) = (x_at(i), y_at(hours[i].precip_pct));
        let (x1, y1) = (x_at(i + 1), y_at(hours[i + 1].precip_pct));
        draw_line(x0, y0, x1, y1, 2.5, line_color);
    }
    #[allow(clippy::needless_range_loop)] // i also feeds x_at(i)
    for i in 0..n {
        draw_circle(x_at(i), y_at(hours[i].precip_pct), 2.5, line_color);
    }

    // Day-boundary verticals + day labels.
    for i in 0..n {
        let new_day = i == 0 || hours[i].date != hours[i - 1].date;
        if new_day {
            let x = x_at(i);
            if i != 0 {
                draw_line(x, top, x, bottom, 1.0, teal);
            }
            let wd = forecast::weekday_from_iso(&hours[i].date);
            draw_text(&wd, x.max(left), top - 4.0, 12.0, teal);
        }
    }

    // Sparse hour ticks.
    #[allow(clippy::needless_range_loop)] // k also feeds x_at(k)
    for k in 0..n {
        if k % 6 == 0 || k == n - 1 {
            let x = x_at(k);
            let dims = measure_text(&hours[k].label, None, 11, 1.0);
            draw_text(
                &hours[k].label,
                x - dims.width / 2.0,
                bottom + 14.0,
                11.0,
                gray,
            );
        }
    }
}

/// Draw a subtle dark radial-gradient background behind the scope.
/// Uses concentric filled circles from a brighter centre to darker edges,
/// approximating the observatory mockup's radial gradient without a custom
/// GLSL shader in the macroquad layer.
fn draw_observatory_background() {
    let w = screen_width();
    let h = screen_height();
    let cx = w / 2.0;
    let cy = h / 2.0;
    let max_r = (w * w + h * h).sqrt() / 2.0;
    let steps = 24;
    for i in (0..steps).rev() {
        let t = i as f32 / steps as f32;
        let r = max_r * (1.0 + t * 0.04);
        let c = MacroquadColor::new(
            0.024 + (0.063 - 0.024) * (1.0 - t),
            0.031 + (0.078 - 0.031) * (1.0 - t),
            0.047 + (0.114 - 0.047) * (1.0 - t),
            1.0,
        );
        draw_circle(cx, cy, r, c);
    }
}

/// Draw the optional radar sweep line — a thin rotating teal beam with a
/// trailing fade, radiating from the scope centre.
fn draw_radar_sweep(pan_km: (f32, f32), zoom: f32, angle_deg: f32, fade: f32) {
    let side = screen_width().min(screen_height());
    let px_per_km = (side / 2.0) / scope::MAX_RANGE_KM * zoom;
    let cx = screen_width() / 2.0 + pan_km.0 * px_per_km;
    let cy = screen_height() / 2.0 + pan_km.1 * px_per_km;
    let len = side * zoom / 2.0;
    for i in 0..18 {
        let a = angle_deg - i as f32 * 2.0;
        let ar = a.to_radians();
        let bx = cx + ar.cos() * len;
        let by = cy - ar.sin() * len;
        let alpha = (18 - i) as f32 / 18.0 * 40.0 * fade;
        draw_line(
            cx,
            cy,
            bx,
            by,
            1.5,
            MacroquadColor::new(0.051, 0.773, 0.722, alpha / 255.0),
        );
    }
}

/// Linearly blend two hex colours (i32/u32) by factor t in [0,1].
fn blend_hex(a: i32, b: i32, t: f32) -> i32 {
    let ar = ((a >> 16) & 0xff) as f32;
    let ag = ((a >> 8) & 0xff) as f32;
    let ab = (a & 0xff) as f32;
    let br = ((b >> 16) & 0xff) as f32;
    let bg = ((b >> 8) & 0xff) as f32;
    let bb = (b & 0xff) as f32;
    let r = ar + (br - ar) * t;
    let g = ag + (bg - ag) * t;
    let bl = ab + (bb - ab) * t;
    ((r as i32) << 16) | ((g as i32) << 8) | (bl as i32)
}

/// Return an accent-tinted background when the element `id` is hovered
/// (using previous-frame pointer-over state), otherwise `idle`.
fn hover_tint(hovered: &[Id], id: &str, active: i32, _idle: i32) -> i32 {
    let is_hovered = hovered.iter().any(|i| i.string_id.as_str() == id);
    if is_hovered {
        blend_hex(active, 0x0dc5b8, 0.35)
    } else {
        active
    }
}

// ---------------------------------------------------------------------------
// Window config
// ---------------------------------------------------------------------------

/// Window icon, decoded from the rendered PNGs at startup.
/// Source of truth is `assets/icon/rustywx.svg`; regenerate the PNGs with
/// `rsvg-convert -w N -h N rustywx.svg -o icon_N.png`.
fn app_icon() -> miniquad::conf::Icon {
    fn rgba<const N: usize>(png: &[u8]) -> [u8; N] {
        let img = image::load_from_memory(png).expect("icon png").to_rgba8();
        let mut out = [0u8; N];
        out.copy_from_slice(&img);
        out
    }
    miniquad::conf::Icon {
        small: rgba(include_bytes!("../assets/icon/icon_16.png")),
        medium: rgba(include_bytes!("../assets/icon/icon_32.png")),
        big: rgba(include_bytes!("../assets/icon/icon_64.png")),
    }
}

fn window_conf() -> macroquad::conf::Conf {
    // Optional startup size override for window-size / HiDPI testing:
    // RUSTYWX_WIN_W / RUSTYWX_WIN_H.
    let env_i32 = |k: &str, default: i32| {
        std::env::var(k)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    };
    macroquad::conf::Conf {
        miniquad_conf: miniquad::conf::Conf {
            window_title: "rustywx — NEXRAD Radar Scope".to_owned(),
            window_width: env_i32("RUSTYWX_WIN_W", 900),
            window_height: env_i32("RUSTYWX_WIN_H", 960),
            high_dpi: true,
            sample_count: 4,
            icon: Some(app_icon()),
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
    // ── Demo mode (--demo <event|path>) ────────────────────────
    let demo_request = match rustywx::demo::parse_args(std::env::args().skip(1)) {
        Ok(req) => req,
        Err(msg) => {
            eprintln!("{msg}");
            std::process::exit(2);
        }
    };

    // Tokio runtime for cache I/O and other async background work.
    // The game loop is driven by macroquad's async executor; this
    // runtime lives alongside it so `tokio::spawn` works everywhere.
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let _rt_guard = rt.enter();

    static DEFAULT_FONT: FontAsset = FontAsset::Bytes {
        file_name: "Inter-Regular.ttf",
        data: include_bytes!("../assets/fonts/Inter-Regular.ttf"),
    };
    static INTER_BOLD: FontAsset = FontAsset::Bytes {
        file_name: "Inter-Bold.ttf",
        data: include_bytes!("../assets/fonts/Inter-Bold.ttf"),
    };
    static MONO_FONT: FontAsset = FontAsset::Bytes {
        file_name: "Inter-Regular.ttf",
        data: include_bytes!("../assets/fonts/Inter-Regular.ttf"),
    };
    // Optional dyslexia-friendly body font (accessibility). The default (body)
    // font is fixed at Ply::new, so switching means rebuilding Ply (see loop).
    static DYSLEXIC_FONT: FontAsset = FontAsset::Bytes {
        file_name: "OpenDyslexicNerdFont-Regular.otf",
        data: include_bytes!("../assets/fonts/OpenDyslexicNerdFont-Regular.otf"),
    };
    static DYSLEXIC_BOLD: FontAsset = FontAsset::Bytes {
        file_name: "OpenDyslexicNerdFont-Bold.otf",
        data: include_bytes!("../assets/fonts/OpenDyslexicNerdFont-Bold.otf"),
    };
    let mut ply = Ply::<rustywx::widgets::ChartWidget>::new(&DEFAULT_FONT).await;
    // Which body font Ply was last built with, so we can detect setting changes.
    let mut active_dyslexic = false;

    // Secondary fonts (Inter Bold, JetBrains Mono) are lazy-loaded on
    // first use via .font(&ASSET) on TextConfig — no explicit loading needed.

    // ── Channels for the background data worker ───────────────
    let (worker_tx, worker_rx) = mpsc::channel();
    let (site_tx, site_rx) = mpsc::channel();

    // Default to KFFC (Atlanta, GA) on first launch. The persisted
    // site preference (loaded below) may override this once it arrives.
    let mut default_site_index = geo::RADAR_SITES
        .iter()
        .position(|s| s.id == "KFFC")
        .unwrap_or(0);
    if let Some(req) = &demo_request {
        let site_id = req.site_id().unwrap_or_else(|msg| {
            eprintln!("{msg}");
            std::process::exit(2);
        });
        default_site_index = geo::RADAR_SITES
            .iter()
            .position(|s| s.id == site_id)
            .unwrap_or_else(|| {
                eprintln!("demo site {site_id} not in the radar site table");
                std::process::exit(2);
            });
    }
    let initial_site = geo::RADAR_SITES[default_site_index].id.to_string();

    // ── Open disk cache ────────────────────────────────────────
    let cache = Cache::new().await.expect("Ply storage initialisation");

    // ── Kick off the demo volume load, if requested ─────────────
    // `DemoRequest` is moved into the task rather than shared, so the
    // startup-load block below can check `demo_label` without touching a
    // borrowed/consumed `demo_request`.
    let mut pending_demo = None;
    let demo_label = demo_request.as_ref().map(|r| r.label());
    if let Some(req) = demo_request {
        let storage = cache.storage();
        let (tx, rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let result = data::fetch_demo_scan(&req, Some(&storage))
                .await
                .map_err(|e| format!("{e:#}"));
            let _ = tx.send(result);
        });
        pending_demo = Some(rx);
    }

    // Healthy poll interval (seconds), shared with the worker thread so the
    // persisted setting — and any live change from the settings panel — takes
    // effect without a blocking read here. (Awaiting the settings oneshot on
    // the macroquad executor panics: "does not support waking futures".) The
    // worker starts on the default; the game loop stores the live value from
    // `state.settings` each frame once the async settings load resolves.
    let poll_interval = Arc::new(AtomicU64::new(data::POLL_INTERVAL.as_secs()));
    data::spawn_worker(
        worker_tx,
        initial_site.clone(),
        site_rx,
        Arc::clone(&poll_interval),
    );

    // Kick off a non-blocking load of the last-cached scan for the
    // initial site so we have something to show before the first
    // network fetch completes. Skipped in demo mode: it would show a
    // stale live scan for the pinned site before the demo volume arrives.
    let pending_load = if demo_label.is_some() {
        None
    } else {
        Some(cache.load_scan(&initial_site))
    };

    // Load the persisted site preference (None on very first launch).
    // Skipped in demo mode: it would yank the site away from the demo scene.
    let pending_site_load = if demo_label.is_some() {
        None
    } else {
        Some(cache.load_site())
    };

    // Load persisted settings (None on very first launch — defaults apply).
    let pending_settings_load = Some(cache.load_settings());

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
        site_index: default_site_index,
        product: Product::Reflectivity,
        pan_km: (0.0, 0.0),
        zoom: 1.0,
        radar_texture: None,
        radar_texture_is_value: false,
        palette_material: None,
        dbz_lut_tex: None,
        needs_reraster: true,
        melting_layer_hint: None,
        qc_report: scope::QcReport::default(),
        scan: None,
        tilt_index: 0,
        status_text: match &demo_label {
            Some(label) => format!("DEMO — loading {label}…"),
            None => "Loading cached data…".to_string(),
        },
        worker_rx,
        site_tx,
        cache,
        pending_load,
        pending_site_load,
        demo: demo_label.clone(),
        pending_demo,
        site_dropdown: DropdownState::default(),
        tilt_dropdown: DropdownState::default(),
        borders: Vec::new(),
        borders_loaded: false,
        borders_fetch_fired: false,
        alerts: Vec::new(),
        alerts_loaded: false,
        alerts_fetch_fired: false,
        last_alert_poll: 0.0,
        radar_panel_open: false,
        radar_anim_start: 0.0,
        show_radar_data: true,
        show_nhc_data: true,
        show_borders: true,
        show_watches: true,
        show_warnings: true,
        fullscreen: false,
        nhc_bundle: None,
        nhc_fetch: nhc::NhcFetchState::new(),
        nhc_fetch_fired: false,
        nhc_last_poll: 0.0,
        nhc_show_panel: false,
        nhc_selected_storm: 0,
        nhc_storm_dropdown: DropdownState::default(),
        nhc_image_textures: HashMap::new(),
        nhc_overlays: scope::NhcOverlayState::default(),
        nhc_modal: NhcModal::None,
        nhc_modal_scroll: 0.0,
        alert_modal: None,
        alert_modal_scroll: 0.0,
        last_mouse_pos: None,
        start_time: get_time(),
        nhc_anim_start: 0.0,
        nhc_anim_from: 0.0,
        pulse_time: 0.0,
        sweep_angle: 0.0,
        hovered_ids: Vec::new(),
        last_click_time: 0.0,
        last_click_pos: (0.0, 0.0),
        settings: Settings::default(),
        pending_settings_load,
        show_settings_panel: false,
        show_shortcuts: false,
        toast: None,
        // Seeded from settings once the async load resolves (see the
        // settings_applied block below) — no network call at startup.
        user_location: None,
        show_location: false,
        location_resolver: rustywx::location::LocationResolver::new(),
        location_input_focused: false,
        location_error_shown: false,
        center_pending: false,
        view_mode: ViewMode::Radar,
        forecast: None,
        forecast_coords: None,
        forecast_fetch_fired: false,
        forecast_error: None,
        forecast_place: String::new(),
        fc_search_text: String::new(),
        fc_search_focused: false,
        fc_geo_hits: Vec::new(),
        fc_geo_fired: false,
    };

    // Boot-time bookkeeping for applying loaded settings exactly once, and
    // only overriding the default site if no explicit site preference was
    // found (see the settings-apply block below). Local to `main`, not
    // `AppState` — this isn't state the rest of the app needs to see.
    let mut had_explicit_site_pref = false;
    let mut site_pref_resolved = false;
    let mut settings_applied = false;

    loop {
        clear_background(MacroquadColor::new(0.031, 0.039, 0.059, 1.0));

        // Rebuild Ply if the dyslexia-friendly body font was toggled — the
        // default font is bound at construction and has no runtime setter.
        if state.settings.dyslexic_font != active_dyslexic {
            active_dyslexic = state.settings.dyslexic_font;
            ply = Ply::<rustywx::widgets::ChartWidget>::new(if active_dyslexic {
                &DYSLEXIC_FONT
            } else {
                &DEFAULT_FONT
            })
            .await;
        }

        let now = get_time();
        // `None` renders the settled/final state immediately; `Subtle` and
        // `Full` both ease in (Subtle just skips the sweep line below).
        let entrance = if state.settings.animation_level == AnimationLevel::None {
            1.0
        } else {
            ease_out_cubic(((now - state.start_time) / 0.6).clamp(0.0, 1.0) as f32)
        };

        // Stage 6: animation timing + hover tracking.
        state.sweep_angle = (state.sweep_angle + 0.6) % 360.0;

        state.hovered_ids = ply.pointer_over_ids();

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

        // ── Poll persisted site preference (first launch restore) ──
        // `.take()` moves the receiver out so there's no borrow conflict
        // with the later `select_site(&mut state, …)` call. If the data
        // isn't ready yet, put the receiver back to poll again next frame.
        let restore_site = if let Some(mut rx) = state.pending_site_load.take() {
            match rx.try_recv() {
                Ok(Some(site_id)) => {
                    site_pref_resolved = true;
                    had_explicit_site_pref = true;
                    geo::RADAR_SITES
                        .iter()
                        .position(|s| s.id == site_id)
                        .filter(|&i| i != state.site_index)
                }
                Ok(None) => {
                    site_pref_resolved = true;
                    None
                }
                Err(_) => {
                    state.pending_site_load = Some(rx);
                    None
                }
            }
        } else {
            None
        };
        if let Some(index) = restore_site {
            select_site(&mut state, index);
        }

        // ── Poll persisted settings (Stage 7) ───────────────────────
        if let Some(mut rx) = state.pending_settings_load.take() {
            match rx.try_recv() {
                Ok(Some(loaded)) => state.settings = loaded,
                Ok(None) => {}
                Err(_) => state.pending_settings_load = Some(rx),
            }
        }

        // Keep the worker's healthy poll interval in sync with the current
        // setting (floored at 1s to avoid a busy loop). Cheap relaxed store.
        poll_interval.store(state.settings.poll_interval_secs.max(1), Ordering::Relaxed);

        // Apply settings-seeded defaults exactly once, after both the site
        // preference and settings loads have resolved (order between the
        // two is not guaranteed — both are independent oneshot loads).
        if !settings_applied && site_pref_resolved && state.pending_settings_load.is_none() {
            settings_applied = true;
            if !had_explicit_site_pref
                && let Some(index) = geo::RADAR_SITES
                    .iter()
                    .position(|s| s.id == state.settings.default_site)
                && index != state.site_index
            {
                select_site(&mut state, index);
            }
            state.show_borders = state.settings.show_borders;
            state.show_watches = state.settings.show_watches;
            state.show_warnings = state.settings.show_warnings;
            state.nhc_show_panel = state.settings.show_nhc;
            state.radar_panel_open = state.settings.show_radar;
            state.show_radar_data = state.settings.show_radar_data;
            state.show_nhc_data = state.settings.show_nhc_data;
            state.show_location = state.settings.show_location;
            // Restore last known location from settings without any
            // network call — the resolver chain runs only on Detect.
            state.user_location = match (state.settings.user_lat, state.settings.user_lon) {
                (Some(lat), Some(lon)) => Some(rustywx::location::Coords { lat, lon }),
                _ => None,
            };
            if state.settings.center_on_location && state.user_location.is_some() {
                recenter_on_user(&mut state);
            }
            // The initial raster (if any) used the default TDBZ kernel size
            // before this load resolved — redo it with the loaded setting.
            state.needs_reraster = true;
        }

        // ── Poll worker messages ──────────────────────────────────
        while let Ok(msg) = state.worker_rx.try_recv() {
            // Demo mode: the worker still polls the pinned site's *live* data —
            // drop everything it says so it can't overwrite the demo scene.
            if state.demo.is_some() {
                continue;
            }
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
                    state.pulse_time = now;
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
                    // `e` already carries the raw anyhow detail + retry
                    // countdown for the status bar; the toast stays short
                    // and friendly (see widgets::toast).
                    state.status_text = format!("Error: {e}");
                    show_toast(&mut state, now, toast_widget::ErrorKind::RadarData);
                }
            }
        }

        // ── Poll pending demo volume load ─────────────────────────
        if let Some(rx) = &mut state.pending_demo
            && let Ok(result) = rx.try_recv()
        {
            state.pending_demo = None;
            match result {
                Ok(scan) => {
                    state.scan = Some(scan);
                    state.tilt_index = 0;
                    state.needs_reraster = true;
                    state.pulse_time = now;
                    update_scan_status(&mut state, "");
                }
                Err(e) => {
                    let label = state.demo.clone().unwrap_or_default();
                    state.status_text = format!("DEMO — {label} — load failed: {e}");
                    show_toast(&mut state, now, toast_widget::ErrorKind::RadarData);
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
                    show_toast(&mut state, now, toast_widget::ErrorKind::Network);
                }
            }
        }

        // ── Fire alerts fetch if not yet started ──────────────────
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
                    show_toast(&mut state, now, toast_widget::ErrorKind::Network);
                }
            }
        }

        // ── Fire NHC fetch if not yet started or refresh interval elapsed ─
        if !state.nhc_fetch_fired
            && (state.nhc_bundle.is_none()
                || now - state.nhc_last_poll > state.settings.nhc_refresh_secs as f64)
        {
            state.nhc_fetch.start();
            state.nhc_fetch_fired = true;
            state.nhc_last_poll = now;
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
                    show_toast(&mut state, now, toast_widget::ErrorKind::Network);
                }
            }
        }

        // ── Poll the location resolver (Detect in progress) ───────
        if let Some(coords) = state.location_resolver.poll(now) {
            state.user_location = Some(coords);
            state.settings.user_lat = Some(coords.lat);
            state.settings.user_lon = Some(coords.lon);
            state.cache.save_settings(&state.settings);
            if state.settings.center_on_location || state.center_pending {
                recenter_on_user(&mut state);
            }
            state.center_pending = false;
        }
        // Surface resolver failures as a toast (once, until the next detect).
        if !state.location_error_shown {
            let msg = match state.location_resolver.status() {
                rustywx::location::LocationStatus::Offline => Some("Location: offline"),
                rustywx::location::LocationStatus::Denied => Some("Location permission denied"),
                rustywx::location::LocationStatus::NotFound => Some("ZIP not found"),
                _ => None,
            };
            if let Some(msg) = msg {
                state.toast = Some(toast_widget::Toast::new(msg, now));
                state.location_error_shown = true;
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

        // For CC-gating (and the fuzzy non-met classifier below): find the CC
        // sweep at the nearest elevation to the REF sweep we are about to
        // rasterize. Only needed for Reflectivity; None for every other
        // product (and when there is no dual-pol CC volume). The fuzzy
        // classifier also wants CC even when the plain CC gate is off, since
        // fuzzy supersedes it — so it's fetched whenever either is enabled.
        let cc_sweep: Option<SweepData> = if state.product == Product::Reflectivity
            && (state.settings.cc_gate_enabled || state.settings.nonmet_fuzzy_enabled)
        {
            state.scan.as_ref().and_then(|scan| {
                let cc = &scan.correlation_coefficient;
                cc.iter()
                    .min_by(|a, b| {
                        (a.elevation_deg - sweep.elevation_deg)
                            .abs()
                            .total_cmp(&(b.elevation_deg - sweep.elevation_deg).abs())
                    })
                    .cloned()
            })
        } else {
            None
        };

        // ZDR/ΦDP sweeps for the fuzzy non-met classifier, selected by
        // nearest elevation exactly like `cc_sweep` above. Only needed when
        // the classifier is on; fails open to None otherwise (and if the
        // volume lacks the moment), same as `cc_sweep`.
        let zdr_sweep: Option<SweepData> =
            if state.product == Product::Reflectivity && state.settings.nonmet_fuzzy_enabled {
                state.scan.as_ref().and_then(|scan| {
                    let zdr = &scan.differential_reflectivity;
                    zdr.iter()
                        .min_by(|a, b| {
                            (a.elevation_deg - sweep.elevation_deg)
                                .abs()
                                .total_cmp(&(b.elevation_deg - sweep.elevation_deg).abs())
                        })
                        .cloned()
                })
            } else {
                None
            };
        let phidp_sweep: Option<SweepData> =
            if state.product == Product::Reflectivity && state.settings.nonmet_fuzzy_enabled {
                state.scan.as_ref().and_then(|scan| {
                    let phidp = &scan.differential_phase;
                    phidp
                        .iter()
                        .min_by(|a, b| {
                            (a.elevation_deg - sweep.elevation_deg)
                                .abs()
                                .total_cmp(&(b.elevation_deg - sweep.elevation_deg).abs())
                        })
                        .cloned()
                })
            } else {
                None
            };

        // Rasterize when needed
        if state.needs_reraster {
            state.needs_reraster = false;

            // Melting-layer hint (annotation overlay, not part of the QC
            // pipeline above) — recomputed alongside the raster so it
            // stays in step with new scans and the settings toggle without
            // its own dirty-flag plumbing.
            state.melting_layer_hint = if state.settings.melting_layer_hint_enabled {
                state
                    .scan
                    .as_ref()
                    .and_then(|scan| melting_layer::detect(&scan.correlation_coefficient))
            } else {
                None
            };
            let (rgba, qc_report) = scope::rasterize_with_report(
                &sweep,
                state.product,
                scope::RASTER_SIZE_PX,
                scope::MAX_RANGE_KM,
                &scope::QcConfig {
                    tdbz_kernel_size: state.settings.tdbz_kernel.size() as usize,
                    cc_sweep: cc_sweep.as_ref(),
                    cc_gate_enabled: state.settings.cc_gate_enabled,
                    cc_gate_threshold: state.settings.cc_gate_threshold,
                    refl_floor_enabled: state.settings.refl_floor_enabled,
                    refl_floor_dbz: state.settings.refl_floor_dbz,
                    vel_dealias_enabled: state.settings.vel_dealias_enabled,
                    vel_sd_censor_enabled: state.settings.vel_sd_censor_enabled,
                    vel_sd_threshold: state.settings.vel_sd_threshold,
                    zdr_sweep: zdr_sweep.as_ref(),
                    phidp_sweep: phidp_sweep.as_ref(),
                    nonmet_fuzzy_enabled: state.settings.nonmet_fuzzy_enabled,
                    nonmet_threshold: state.settings.nonmet_threshold,
                    refl_gap_fill_enabled: state.settings.refl_gap_fill_enabled,
                    multi_scale_texture_enabled: state.settings.multi_scale_texture_enabled,
                    sun_spike_removal_enabled: state.settings.sun_spike_removal_enabled,
                },
            );
            let tex = Texture2D::from_rgba8(
                scope::RASTER_SIZE_PX as u16,
                scope::RASTER_SIZE_PX as u16,
                &rgba,
            );
            // Reflectivity is a value field colorized by the GPU palette shader:
            // Linear so the GPU interpolates the dBZ value across screen pixels,
            // then the Nearest-filtered LUT snaps to discrete colors — crisp
            // bands, smooth edges at any zoom. Other products are CPU-colorized
            // RGBA; Nearest keeps their band steps crisp.
            let is_value = state.product == Product::Reflectivity;
            tex.set_filter(if is_value {
                macroquad::texture::FilterMode::Linear
            } else {
                macroquad::texture::FilterMode::Nearest
            });
            state.radar_texture_is_value = is_value;
            state.radar_texture = Some(tex);

            // Build the palette material + LUT once, now that the GL context is
            // live (can't be done at State construction).
            if state.palette_material.is_none() {
                let lut = Texture2D::from_rgba8(256, 1, &colors::dbz_lut());
                lut.set_filter(macroquad::texture::FilterMode::Nearest);
                state.dbz_lut_tex = Some(lut);
                state.palette_material = Some(scope::load_palette_material());
            }
            state.qc_report = qc_report;
        }

        // Draw scope + overlays directly to screen (avoids render_to_texture
        // coordinate flip — framebuffer bottom-left origin + Ply .image() display
        // causes a 180° rotation of the content).
        draw_observatory_background();
        if state.view_mode == ViewMode::Radar {
            // Bind the LUT and hand the palette material to the draw when the
            // radar texture is a reflectivity value field; None => CPU-colorized
            // product drawn plainly.
            let palette = if state.radar_texture_is_value {
                match (state.palette_material.as_ref(), state.dbz_lut_tex.as_ref()) {
                    (Some(mat), Some(lut)) => {
                        mat.set_texture("Palette", lut.clone());
                        Some(mat)
                    }
                    _ => None,
                }
            } else {
                None
            };
            scope::draw_scope_to_texture(
                if state.show_radar_data {
                    state.radar_texture.as_ref()
                } else {
                    None
                },
                site,
                state.pan_km,
                state.zoom,
                Some((&state.borders, state.show_borders)),
                Some((&state.alerts, state.show_watches, state.show_warnings)),
                if state.show_nhc_data {
                    state.nhc_bundle.as_ref().map(|b| (b, &state.nhc_overlays))
                } else {
                    None
                },
                if state.show_location {
                    state.user_location
                } else {
                    None
                },
                state.radar_panel_open, // show_sites arg — markers shown only while the Radar panel is open
                state.settings.show_scope_rings,
                state.melting_layer_hint.as_ref(),
                palette,
            );

            // Radar sweep line (optional observatory visual flourish) — gated on
            // its own setting, independent of the range rings/crosshairs.
            if state.settings.show_sweep {
                draw_radar_sweep(state.pan_km, state.zoom, state.sweep_angle, entrance);
            }
        }

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
        let is_mobile = screen_width() < 900.0;
        let mut ui = ply.begin();

        ui.element()
            .width(grow!())
            .height(grow!())
            .layout(|layout| layout.direction(TopToBottom))
            .children(|ui| {
                // ── Controls bar (frosted glass) ────────────────────
                // Height fits content so the bar can grow to multiple rows at
                // narrow widths (layout.wrap) instead of squeezing controls to
                // slivers / spilling text. Wide desktop stays a single row.
                glass_panel::glass(ui.element().width(grow!()).height(fit!()))
                    .layout(|layout| {
                        layout
                            .direction(LeftToRight)
                            .padding(8)
                            .gap(12)
                            .align(Left, CenterY)
                            .wrap()
                            .wrap_gap(6)
                    })
                    .children(|ui| {
                        // ── Panels group ──────────────────────────────────
                        // Menu-openers for the slide-in side panels. Chevron
                        // affordance, no ✓ — panel-open is not data-visible.
                        let radar_panel_bg = hover_tint(
                            &state.hovered_ids,
                            "btn-radar",
                            if state.radar_panel_open {
                                0x0dc5b8
                            } else {
                                0x1E1B1B
                            },
                            0x1E1B1B,
                        );
                        ui.element()
                            .id("btn-radar")
                            .width(fit!())
                            .height(fixed!(if is_mobile { 44.0 } else { 24.0 }))
                            .background_color(radar_panel_bg)
                            .corner_radius(4.0)
                            .layout(|layout| {
                                layout
                                    .direction(LeftToRight)
                                    .gap(6)
                                    .padding((0, 8, 0, 8))
                                    .align(CenterX, CenterY)
                            })
                            .accessibility(|a| {
                                a.button("Open radar panel").checked(state.radar_panel_open)
                            })
                            .children(|ui| {
                                ui.text("Radar", |text| text.font_size(12).color(0xE8E0DC));
                                ui.text(nf::CHEVRON_RIGHT, |text| {
                                    text.font_size(10).font(&SYMBOL_FONT).color(0xE8E0DC)
                                });
                            });

                        let tropical_panel_bg = hover_tint(
                            &state.hovered_ids,
                            "btn-tropical",
                            if state.nhc_show_panel {
                                0x0dc5b8
                            } else {
                                0x1E1B1B
                            },
                            0x1E1B1B,
                        );
                        ui.element()
                            .id("btn-tropical")
                            .width(fit!())
                            .height(fixed!(if is_mobile { 44.0 } else { 24.0 }))
                            .background_color(tropical_panel_bg)
                            .corner_radius(4.0)
                            .layout(|layout| {
                                layout
                                    .direction(LeftToRight)
                                    .gap(6)
                                    .padding((0, 8, 0, 8))
                                    .align(CenterX, CenterY)
                            })
                            .accessibility(|a| {
                                a.button("Open tropical panel")
                                    .checked(state.nhc_show_panel)
                            })
                            .children(|ui| {
                                ui.text("Tropical", |text| text.font_size(12).color(0xE8E0DC));
                                ui.text(nf::CHEVRON_RIGHT, |text| {
                                    text.font_size(10).font(&SYMBOL_FONT).color(0xE8E0DC)
                                });
                            });

                        let forecast_active = state.view_mode == ViewMode::Forecast;
                        let forecast_bg = hover_tint(
                            &state.hovered_ids,
                            "btn-forecast",
                            if forecast_active { 0x0dc5b8 } else { 0x1E1B1B },
                            0x1E1B1B,
                        );
                        let forecast_label = if forecast_active {
                            "Forecast ✓"
                        } else {
                            "Forecast"
                        };
                        ui.element()
                            .id("btn-forecast")
                            .width(fit!())
                            .height(fixed!(if is_mobile { 44.0 } else { 24.0 }))
                            .background_color(forecast_bg)
                            .corner_radius(4.0)
                            .layout(|layout| {
                                layout
                                    .direction(LeftToRight)
                                    .gap(6)
                                    .padding((0, 8, 0, 8))
                                    .align(CenterX, CenterY)
                            })
                            .accessibility(|a| a.button(forecast_label).checked(forecast_active))
                            .children(|ui| {
                                ui.text(forecast_label, |text| text.font_size(12).color(0xE8E0DC));
                            });

                        // Map-layer toggles (Radar/Tropical/Borders/Watches/
                        // Warnings/Location) are irrelevant in Forecast mode.
                        if !forecast_active {
                            // Divider between Panels and Layers groups.
                            ui.element()
                                .width(fixed!(1.0))
                                .height(fixed!(if is_mobile { 28.0 } else { 16.0 }))
                                .background_color((1.0f32, 1.0f32, 1.0f32, 40.0f32))
                                .empty();

                            // ── Layers group ──────────────────────────────────
                            // Pure visibility toggles (✓ idiom).
                            let radar_data_bg = hover_tint(
                                &state.hovered_ids,
                                "btn-radar-data",
                                if state.show_radar_data {
                                    0x0dc5b8
                                } else {
                                    0x1E1B1B
                                },
                                0x1E1B1B,
                            );
                            let radar_data_label = if state.show_radar_data {
                                "Radar ✓"
                            } else {
                                "Radar"
                            };
                            ui.element()
                                .id("btn-radar-data")
                                .width(fit!())
                                .height(fixed!(if is_mobile { 44.0 } else { 24.0 }))
                                .background_color(radar_data_bg)
                                .corner_radius(4.0)
                                .layout(|layout| {
                                    layout.padding((0, 8, 0, 8)).align(CenterX, CenterY)
                                })
                                .accessibility(|a| {
                                    a.button(radar_data_label).checked(state.show_radar_data)
                                })
                                .children(|ui| {
                                    ui.text(radar_data_label, |text| {
                                        text.font_size(12).color(0xE8E0DC)
                                    });
                                });

                            // ── Tropical data toggle (Layers group) ──────────
                            let tropical_data_bg = hover_tint(
                                &state.hovered_ids,
                                "btn-tropical-data",
                                if state.show_nhc_data {
                                    0x0dc5b8
                                } else {
                                    0x1E1B1B
                                },
                                0x1E1B1B,
                            );
                            let tropical_data_label = if state.show_nhc_data {
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
                                .id("btn-tropical-data")
                                .width(fit!())
                                .height(fixed!(if is_mobile { 44.0 } else { 24.0 }))
                                .background_color(tropical_data_bg)
                                .corner_radius(4.0)
                                .layout(|layout| {
                                    layout.padding((0, 8, 0, 8)).align(CenterX, CenterY)
                                })
                                .accessibility(|a| {
                                    a.button(tropical_data_label).checked(state.show_nhc_data)
                                })
                                .children(|ui| {
                                    ui.text(&format!("{tropical_data_label}{nhc_badge}"), |text| {
                                        text.font_size(12).color(0xE8E0DC)
                                    });
                                });

                            // Zoom/Pan readout lives in the bottom status bar.

                            // ── Overlay toggles (Stage 4) ──────────────────
                            let borders_active = state.show_borders;
                            let borders_bg = if borders_active { 0x0dc5b8 } else { 0x1E1B1B };
                            let borders_label = if borders_active {
                                "Borders ✓"
                            } else {
                                "Borders"
                            };
                            ui.element()
                                .id("btn-borders")
                                .width(fit!())
                                .height(fixed!(if is_mobile { 44.0 } else { 24.0 }))
                                .background_color(borders_bg)
                                .corner_radius(4.0)
                                .layout(|layout| {
                                    layout.padding((0, 8, 0, 8)).align(CenterX, CenterY)
                                })
                                .accessibility(|a| a.button(borders_label).checked(borders_active))
                                .children(|ui| {
                                    ui.text(borders_label, |text| {
                                        text.font_size(12).color(0xE8E0DC)
                                    });
                                });

                            // Per-category counts for the Watches/Warnings buttons.
                            let (watch_n, warn_n) =
                                state.alerts.iter().fold((0, 0), |(w, a), al| {
                                    if alerts::is_watch(&al.event) {
                                        (w + 1, a)
                                    } else {
                                        (w, a + 1)
                                    }
                                });
                            let count_suffix = |n: usize| {
                                if state.alerts_loaded {
                                    format!(" ({n})")
                                } else if state.alerts_fetch_fired {
                                    " (…)".to_string()
                                } else {
                                    String::new()
                                }
                            };

                            let watches_bg = hover_tint(
                                &state.hovered_ids,
                                "btn-watches",
                                if state.show_watches {
                                    0x0dc5b8
                                } else {
                                    0x1E1B1B
                                },
                                0x1E1B1B,
                            );
                            let watches_label = if state.show_watches {
                                "Watches ✓"
                            } else {
                                "Watches"
                            };
                            ui.element()
                                .id("btn-watches")
                                .width(fit!())
                                .height(fixed!(if is_mobile { 44.0 } else { 24.0 }))
                                .background_color(watches_bg)
                                .corner_radius(4.0)
                                .layout(|layout| {
                                    layout.padding((0, 8, 0, 8)).align(CenterX, CenterY)
                                })
                                .accessibility(|a| {
                                    a.button(watches_label).checked(state.show_watches)
                                })
                                .children(|ui| {
                                    ui.text(
                                        &format!("{watches_label}{}", count_suffix(watch_n)),
                                        |text| text.font_size(12).color(0xE8E0DC),
                                    );
                                });

                            let warnings_bg = hover_tint(
                                &state.hovered_ids,
                                "btn-warnings",
                                if state.show_warnings {
                                    0x0dc5b8
                                } else {
                                    0x1E1B1B
                                },
                                0x1E1B1B,
                            );
                            let warnings_label = if state.show_warnings {
                                "Warnings ✓"
                            } else {
                                "Warnings"
                            };
                            ui.element()
                                .id("btn-warnings")
                                .width(fit!())
                                .height(fixed!(if is_mobile { 44.0 } else { 24.0 }))
                                .background_color(warnings_bg)
                                .corner_radius(4.0)
                                .layout(|layout| {
                                    layout.padding((0, 8, 0, 8)).align(CenterX, CenterY)
                                })
                                .accessibility(|a| {
                                    a.button(warnings_label).checked(state.show_warnings)
                                })
                                .children(|ui| {
                                    ui.text(
                                        &format!("{warnings_label}{}", count_suffix(warn_n)),
                                        |text| text.font_size(12).color(0xE8E0DC),
                                    );
                                });

                            // ── Location toggle button (Task 9) ──────────────
                            let loc_bg = hover_tint(
                                &state.hovered_ids,
                                "btn-location",
                                if state.show_location {
                                    0x0dc5b8
                                } else {
                                    0x1E1B1B
                                },
                                0x1E1B1B,
                            );
                            let loc_label = if state.show_location {
                                "Location ✓"
                            } else {
                                "Location"
                            };
                            ui.element()
                                .id("btn-location")
                                .width(fit!())
                                .height(fixed!(if is_mobile { 44.0 } else { 24.0 }))
                                .background_color(loc_bg)
                                .corner_radius(4.0)
                                .layout(|layout| {
                                    layout.padding((0, 8, 0, 8)).align(CenterX, CenterY)
                                })
                                .accessibility(|a| a.button(loc_label).checked(state.show_location))
                                .children(|ui| {
                                    ui.text(loc_label, |text| text.font_size(12).color(0xE8E0DC));
                                });
                        }

                        // Settings gear lives in the bottom status bar.

                        // Spacer pushes window controls to the right edge.
                        ui.element()
                            .width(grow!())
                            .height(fixed!(0.0))
                            .children(|_| {});

                        // ── Window controls (right side, kept together) ──
                        let ctrl_h = if is_mobile { 44.0 } else { 24.0 };
                        let ctrl_w = fixed!(if is_mobile { 44.0 } else { 30.0 });
                        let (fs_glyph, fs_label) = if state.fullscreen {
                            (nf::COMPRESS, "Exit fullscreen")
                        } else {
                            (nf::EXPAND, "Fullscreen")
                        };
                        let fs_bg =
                            hover_tint(&state.hovered_ids, "btn-fullscreen", 0x1E1B1B, 0x2A2727);
                        let close_bg =
                            hover_tint(&state.hovered_ids, "btn-close", 0x1E1B1B, 0xC03535);
                        ui.element()
                            .width(fit!())
                            .height(fixed!(ctrl_h))
                            .layout(|layout| {
                                layout.direction(LeftToRight).gap(6).align(Left, CenterY)
                            })
                            .children(|ui| {
                                ui.element()
                                    .id("btn-fullscreen")
                                    .width(ctrl_w)
                                    .height(fixed!(ctrl_h))
                                    .background_color(fs_bg)
                                    .corner_radius(4.0)
                                    .layout(|layout| layout.align(CenterX, CenterY))
                                    .accessibility(|a| a.button(fs_label))
                                    .children(|ui| {
                                        ui.text(fs_glyph, |text| {
                                            text.font_size(13).font(&SYMBOL_FONT).color(0xE8E0DC)
                                        });
                                    });
                                ui.element()
                                    .id("btn-close")
                                    .width(ctrl_w)
                                    .height(fixed!(ctrl_h))
                                    .background_color(close_bg)
                                    .corner_radius(4.0)
                                    // The fa-times glyph sits ~4px right / ~1.5px
                                    // high of the cell centre in Symbols Nerd Font
                                    // Mono (ply draws all glyphs at the Latin
                                    // baseline). Asymmetric padding recentres it.
                                    // ponytail: per-glyph nudge; the real fix is a
                                    // ply-engine metrics patch.
                                    .layout(|layout| {
                                        layout.padding((3, 8, 0, 0)).align(CenterX, CenterY)
                                    })
                                    .accessibility(|a| a.button("Close"))
                                    .children(|ui| {
                                        ui.text(nf::CLOSE, |text| {
                                            text.font_size(13).font(&SYMBOL_FONT).color(0xE8E0DC)
                                        });
                                    });
                            });
                    });

                // ── Forecast full-screen view ───────────────────────────
                if state.view_mode == ViewMode::Forecast {
                    glass_panel::glass(ui.element().width(grow!()).height(grow!()))
                        .layout(|layout| {
                            layout
                                .direction(TopToBottom)
                                .padding(24)
                                .gap(20)
                                .align(CenterX, Top)
                        })
                        .children(|ui| {
                            // Search row.
                            let search_bg = hover_tint(
                                &state.hovered_ids,
                                "fc-search",
                                if state.fc_search_focused {
                                    0x2A2727
                                } else {
                                    0x1E1B1B
                                },
                                0x2A2727,
                            );
                            let search_display = if state.fc_search_text.is_empty() {
                                "Search a city…".to_string()
                            } else {
                                state.fc_search_text.clone()
                            };
                            ui.element()
                                .id("fc-search")
                                .width(fixed!(360.0))
                                .height(fixed!(32.0))
                                .background_color(search_bg)
                                .corner_radius(6.0)
                                .layout(|layout| {
                                    layout.padding((0, 10, 0, 10)).align(Left, CenterY)
                                })
                                .accessibility(|a| a.button(&search_display))
                                .children(|ui| {
                                    ui.text(&search_display, |t| t.font_size(14).color(0xE8E0DC));
                                });

                            // Geocode results dropdown (if any).
                            for (idx, hit) in state.fc_geo_hits.iter().enumerate() {
                                let hit_id: Id = ("fc-hit", idx as u32).into();
                                let hit_hovered = state.hovered_ids.contains(&hit_id);
                                let hit_bg = if hit_hovered {
                                    blend_hex(0x1E1B1B, 0x0dc5b8, 0.35)
                                } else {
                                    0x1E1B1B
                                };
                                ui.element()
                                    .id(hit_id.clone())
                                    .width(fixed!(360.0))
                                    .height(fixed!(26.0))
                                    .background_color(hit_bg)
                                    .corner_radius(4.0)
                                    .layout(|layout| {
                                        layout.padding((0, 10, 0, 10)).align(Left, CenterY)
                                    })
                                    .accessibility(|a| a.button(&hit.label))
                                    .children(|ui| {
                                        ui.text(&hit.label, |t| t.font_size(13).color(0xC8C0BC));
                                    });
                            }

                            // Body: current conditions + 7-day strip, or a status line.
                            if let Some(err) = &state.forecast_error {
                                ui.text(&format!("Couldn't load forecast: {err}"), |t| {
                                    t.font_size(14).color(0xE08080)
                                });
                            } else if let Some(fc) = &state.forecast {
                                // Current conditions.
                                let (glyph, label) =
                                    forecast::wmo_icon(fc.current.code, fc.current.is_day);
                                ui.text(&fc.place, |t| t.font_size(20).color(0xE8E0DC));
                                ui.element()
                                    .width(fit!())
                                    .height(fit!())
                                    .layout(|layout| {
                                        layout
                                            .direction(LeftToRight)
                                            .gap(16)
                                            .align(CenterX, CenterY)
                                    })
                                    .children(|ui| {
                                        ui.text(glyph, |t| {
                                            t.font_size(48).font(&SYMBOL_FONT).color(0xE8E0DC)
                                        });
                                        ui.text(&format!("{:.0}°F", fc.current.temp), |t| {
                                            t.font_size(48).color(0xE8E0DC)
                                        });
                                    });
                                ui.text(
                                    &format!(
                                        "{label}   Feels {:.0}°   Wind {:.0} mph   Humidity {}%",
                                        fc.current.feels_like, fc.current.wind, fc.current.humidity
                                    ),
                                    |t| t.font_size(14).color(0xC8C0BC),
                                );

                                // 7-day strip.
                                ui.element()
                                    .width(fit!())
                                    .height(fit!())
                                    .layout(|layout| {
                                        layout.direction(LeftToRight).gap(12).align(CenterX, Top)
                                    })
                                    .children(|ui| {
                                        for day in &fc.days {
                                            let (dglyph, _dlabel) =
                                                forecast::wmo_icon(day.code, true);
                                            ui.element()
                                                .width(fixed!(72.0))
                                                .height(fit!())
                                                .background_color(0x1E1B1B)
                                                .corner_radius(6.0)
                                                .layout(|layout| {
                                                    layout
                                                        .direction(TopToBottom)
                                                        .padding(8)
                                                        .gap(6)
                                                        .align(CenterX, Top)
                                                })
                                                .children(|ui| {
                                                    ui.text(&day.weekday, |t| {
                                                        t.font_size(13).color(0xE8E0DC)
                                                    });
                                                    ui.text(dglyph, |t| {
                                                        t.font_size(22)
                                                            .font(&SYMBOL_FONT)
                                                            .color(0xE8E0DC)
                                                    });
                                                    ui.text(&format!("{:.0}°", day.hi), |t| {
                                                        t.font_size(14).color(0xE8E0DC)
                                                    });
                                                    ui.text(&format!("{:.0}°", day.lo), |t| {
                                                        t.font_size(13).color(0x9A9490)
                                                    });
                                                    ui.text(&format!("{}%", day.precip_pct), |t| {
                                                        t.font_size(12).color(0x6F9FE0)
                                                    });
                                                });
                                        }
                                    });

                                // Hourly rain chance (next 24h) as a line graph:
                                // each dot is an hour, its height the rain %.
                                // Drawn directly via macroquad in `draw_rain_chart`,
                                // through Ply's custom-draw hook (see `ui.show(...)`).
                                if !fc.hours.is_empty() {
                                    ui.text("Hourly rain chance", |t| {
                                        t.font_size(14).color(0xC8C0BC)
                                    });
                                    ui.element()
                                        .width(fixed!(600.0))
                                        .height(fixed!(150.0))
                                        .custom_element(ChartWidget::RainChart(fc.hours.clone()))
                                        .empty();
                                }
                            } else if state.user_location.is_none() {
                                ui.text("Detecting location…", |t| {
                                    t.font_size(14).color(0xC8C0BC)
                                });
                            } else {
                                ui.text("Loading forecast…", |t| t.font_size(14).color(0xC8C0BC));
                            }
                        });
                }

                // ── Radar slide-in panel ────────────────────────────────
                if state.radar_panel_open {
                    if state.radar_anim_start == 0.0 {
                        state.radar_anim_start = now;
                    }
                    let panel_w = if is_mobile { screen_width() } else { 320.0 };
                    let panel_h = if is_mobile {
                        screen_height()
                    } else {
                        screen_height() - 60.0
                    };
                    let panel_top = if is_mobile { 0.0 } else { 36.0 };
                    let slide_t = ((now - state.radar_anim_start) / 0.5).clamp(0.0, 1.0) as f32;
                    let slide = match state.settings.animation_level {
                        AnimationLevel::Full => ease_out_elastic(slide_t),
                        AnimationLevel::Subtle => ease_out_cubic(slide_t),
                        AnimationLevel::None => 1.0,
                    };
                    let final_x = if is_mobile {
                        0.0
                    } else {
                        screen_width() - panel_w - 8.0
                    };
                    let panel_x = final_x + (1.0 - slide) * (panel_w + 16.0);

                    // Dropdown popups float in absolute screen space via
                    // `attach_root`, so anchor them just left of the panel (a
                    // flyout) near their button. Mobile has no room left, so
                    // the popup opens over the panel at the left edge.
                    // ponytail: hardcoded popup y per the fixed panel layout;
                    // compute from measured element rects if it goes dynamic.
                    let flyout_x = |popup_w: f32| {
                        if is_mobile {
                            8.0
                        } else {
                            screen_width() - panel_w - popup_w - 16.0
                        }
                    };
                    // y ≈ each button's row: header(28)+name(18)+gaps put the
                    // site button ~64px below the panel top; the 3 stacked
                    // product buttons push tilt ~120px lower.
                    let site_cfg = DropdownConfig {
                        panel_offset: (flyout_x(SITE_DROPDOWN.width), panel_top + 64.0),
                        ..SITE_DROPDOWN
                    };
                    let tilt_cfg = DropdownConfig {
                        panel_offset: (flyout_x(TILT_DROPDOWN.width), panel_top + 184.0),
                        ..TILT_DROPDOWN
                    };

                    glass_panel::glass(
                        ui.element()
                            .id("radar-panel")
                            .width(fixed!(panel_w))
                            .height(fixed!(panel_h)),
                    )
                    .floating(|floating| {
                        floating
                            .offset((panel_x, panel_top))
                            .z_index(50)
                            .attach_root()
                    })
                    .layout(|layout| layout.direction(TopToBottom).padding(8).gap(6))
                    .overflow(|o| {
                        o.scroll_y()
                            .scrollbar(|s| s.width(6.0).thumb_color(0x4a4a4a).track_color(0x1a1a1a))
                    })
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
                                ui.text(nf::RADAR, |text| {
                                    text.font_size(15).font(&SYMBOL_FONT).color(0x0dc5b8)
                                });
                                let bold = if active_dyslexic {
                                    &DYSLEXIC_BOLD
                                } else {
                                    &INTER_BOLD
                                };
                                ui.text("Radar", |text| {
                                    text.font_size(14).font(bold).color(0xE8E0DC)
                                });
                            });

                        ui.text(&format!("— {}", site.name), |text| {
                            text.font_size(14).color(0xE8E0DC)
                        });

                        state.site_dropdown.draw(
                            ui,
                            site_cfg,
                            site.id,
                            &site_options,
                            Some(state.site_index),
                        );

                        toggle::draw(ui, state.product, &PRODUCT_OPTIONS);

                        state.tilt_dropdown.draw(
                            ui,
                            tilt_cfg,
                            tilt_label,
                            &tilt_options,
                            tilt_options
                                .get(state.tilt_index)
                                .map(|option| option.source_index),
                        );
                    });
                }

                // ── NHC slide-in panel (Stage 5) ────────────────────────
                if state.nhc_show_panel {
                    // Spring slide-in: start off-screen right, ease into place.
                    if state.nhc_anim_start == 0.0 {
                        state.nhc_anim_start = now;
                    }
                    let panel_w = if is_mobile { screen_width() } else { 320.0 };
                    let panel_h = if is_mobile {
                        screen_height()
                    } else {
                        screen_height() - 60.0
                    };
                    let slide_t = ((now - state.nhc_anim_start) / 0.5).clamp(0.0, 1.0) as f32;
                    // Full = bouncy spring; Subtle = damped (no overshoot);
                    // None = appear instantly in the final position.
                    let slide = match state.settings.animation_level {
                        AnimationLevel::Full => ease_out_elastic(slide_t),
                        AnimationLevel::Subtle => ease_out_cubic(slide_t),
                        AnimationLevel::None => 1.0,
                    };
                    let final_x = if is_mobile {
                        0.0
                    } else {
                        screen_width() - panel_w - 8.0
                    };
                    let panel_x = final_x + (1.0 - slide) * (panel_w + 16.0);
                    glass_panel::glass(
                        ui.element()
                            .id("nhc-panel")
                            .width(fixed!(panel_w))
                            .height(fixed!(panel_h)),
                    )
                    .floating(|floating| {
                        floating
                            .offset((panel_x, if is_mobile { 0.0 } else { 36.0 }))
                            .z_index(50)
                            .attach_root()
                    })
                    .layout(|layout| layout.direction(TopToBottom).padding(8).gap(6))
                    .overflow(|o| {
                        o.scroll_y()
                            .scrollbar(|s| s.width(6.0).thumb_color(0x4a4a4a).track_color(0x1a1a1a))
                    })
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
                                ui.text(nf::HURRICANE, |text| {
                                    text.font_size(15).font(&SYMBOL_FONT).color(0x0dc5b8)
                                });
                                let bold = if active_dyslexic {
                                    &DYSLEXIC_BOLD
                                } else {
                                    &INTER_BOLD
                                };
                                ui.text("NHC Tropical Cyclones", |text| {
                                    text.font_size(14).font(bold).color(0xE8E0DC)
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
                                        ui.text(&format!("Advisory: {}", meta.advisory_num), |t| {
                                            t.font_size(11).color(0x9E9590)
                                        });
                                        ui.text(&format!("Updated: {}", meta.last_update), |t| {
                                            t.font_size(11).color(0x9E9590)
                                        });
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
                                            layout
                                                .padding((0, 8, 0, 8))
                                                .gap(5)
                                                .align(CenterX, CenterY)
                                        })
                                        .accessibility(|a| a.link("NHC Graphics Page"))
                                        .children(|ui| {
                                            ui.text("NHC Graphics Page", |t| {
                                                t.font_size(11).color(0x4a90d9)
                                            });
                                            ui.text(nf::EXTERNAL_LINK, |t| {
                                                t.font_size(11).font(&SYMBOL_FONT).color(0x4a90d9)
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
                                            "btn-nhc-earliest",
                                            "Earliest Arrival (34kt)",
                                            state.nhc_overlays.show_earliest_arrival,
                                        );
                                        nhc_toggle_button(
                                            ui,
                                            "btn-nhc-likely",
                                            "Most Likely Arrival (34kt)",
                                            state.nhc_overlays.show_most_likely_arrival,
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
                                            for (img_idx, img) in images.iter().enumerate() {
                                                let key = format!("{}:{}", meta.id, img.title);
                                                // Skip products that failed to download (404)
                                                if !state.nhc_image_textures.contains_key(&key) {
                                                    continue;
                                                }
                                                let tex =
                                                    state.nhc_image_textures.get(&key).unwrap();
                                                // Clickable row: thumbnail + title
                                                ui.element()
                                                    .id(("btn-nhc-img", img_idx as u32))
                                                    .width(grow!())
                                                    .height(fixed!(56.0))
                                                    .background_color(0x1E1B1B)
                                                    .corner_radius(3.0)
                                                    .layout(|layout| {
                                                        layout
                                                            .direction(LeftToRight)
                                                            .padding((4, 6, 4, 6))
                                                            .gap(8)
                                                            .align(Left, CenterY)
                                                    })
                                                    .children(|ui| {
                                                        // Thumbnail image (75x48, NHC aspect ratio)
                                                        ui.element()
                                                            .width(fixed!(75.0))
                                                            .height(fixed!(48.0))
                                                            .image(tex.clone())
                                                            .empty();
                                                        // Product title
                                                        ui.text(&img.title, |t| {
                                                            t.font_size(11).color(0xE8E0DC)
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
                                        // Entire preview is one clickable element
                                        ui.element()
                                            .id(("btn-nhc-text", product.title.len() as u32))
                                            .width(grow!())
                                            .height(fit!())
                                            .background_color(0x171A1F)
                                            .corner_radius(4.0)
                                            .layout(|layout| layout.direction(TopToBottom).gap(2))
                                            .children(|ui| {
                                                // Title bar
                                                ui.element()
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
                                                        ui.text(&product.title, |t| {
                                                            t.font_size(11).color(0xE8E0DC)
                                                        });
                                                    });
                                                // Truncated content preview
                                                ui.element()
                                                    .width(grow!())
                                                    .height(fixed!(80.0))
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
                                            });
                                    }
                                }
                            }
                        }
                    });
                }

                // ── NHC product modal (Stage 5) ────────────────────────────
                if !matches!(state.nhc_modal, NhcModal::None) {
                    let modal_w = 640.0;
                    let modal_h = screen_height() * 0.7;
                    let modal_x = (screen_width() - modal_w) / 2.0;
                    let modal_y = (screen_height() - modal_h) / 2.0;
                    let content_h = modal_h - 36.0 - 40.0 - 24.0;

                    // Semi-transparent backdrop (click to close)
                    ui.element()
                        .id("nhc-modal-backdrop")
                        .width(fixed!(screen_width()))
                        .height(fixed!(screen_height()))
                        .background_color((0.0f32, 0.0f32, 0.0f32, 220.0f32))
                        .floating(|f| f.offset((0.0, 0.0)).z_index(200).attach_root())
                        .empty();

                    // Modal panel (frosted glass)
                    glass_panel::glass(ui.element().width(fixed!(modal_w)).height(fixed!(modal_h)))
                        .floating(|f| f.offset((modal_x, modal_y)).z_index(201).attach_root())
                        .layout(|l| l.direction(TopToBottom).padding(0).gap(0))
                        .children(|ui| {
                            // Title bar
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
                                    let title = match &state.nhc_modal {
                                        NhcModal::Text { title, .. } => title.clone(),
                                        NhcModal::Image { title, .. } => format!("🌐 {title}"),
                                        _ => String::new(),
                                    };
                                    ui.text(&title, |t| t.font_size(14).color(0xE8E0DC));
                                    ui.element().width(grow!()).height(fixed!(1.0)).empty();
                                    // Close button
                                    ui.element()
                                        .id("nhc-modal-close")
                                        .width(fixed!(28.0))
                                        .height(fixed!(28.0))
                                        .background_color(0x3a1a1a)
                                        .corner_radius(4.0)
                                        .layout(|l| l.align(CenterX, CenterY))
                                        .children(|ui| {
                                            ui.text(nf::CLOSE, |t| {
                                                t.font_size(14).font(&SYMBOL_FONT).color(0xE8E0DC)
                                            });
                                        });
                                });

                            // Content area
                            ui.element()
                                .width(grow!())
                                .height(grow!())
                                .background_color(0x0a0d12)
                                .layout(|l| l.padding(12).gap(8).direction(TopToBottom))
                                .children(|ui| {
                                    match &state.nhc_modal {
                                        NhcModal::Text { content, .. } => {
                                            let lines = wrap_modal_text(content);
                                            let visible_count = (content_h / NHC_MODAL_LINE_HEIGHT)
                                                .floor()
                                                .max(1.0)
                                                as usize;
                                            let max_first =
                                                lines.len().saturating_sub(visible_count);
                                            let first = (state.nhc_modal_scroll
                                                / NHC_MODAL_LINE_HEIGHT)
                                                .floor()
                                                as usize;
                                            let first = first.min(max_first);
                                            let last = (first + visible_count).min(lines.len());
                                            let window = lines[first..last].join("\n");
                                            ui.text(&window, |t| {
                                                t.font_size(11)
                                                    .color(0x9E9590)
                                                    .line_height(NHC_MODAL_LINE_HEIGHT as u16)
                                                    .wrap_mode(ply_engine::text::WrapMode::Newline)
                                            });
                                        }
                                        NhcModal::Image { .. } => {
                                            // Image drawn via macroquad after ui.show()
                                        }
                                        _ => {}
                                    }
                                });

                            // Bottom bar with "Open in browser" button
                            ui.element()
                                .width(grow!())
                                .height(fixed!(40.0))
                                .background_color(0x1E1B1B)
                                .corner_radius(8.0)
                                .layout(|l| l.padding((0, 12, 0, 12)).align(Right, CenterY))
                                .children(|ui| {
                                    ui.element()
                                        .id("nhc-modal-browser")
                                        .width(fit!())
                                        .height(fixed!(28.0))
                                        .background_color(0x2a3a5a)
                                        .corner_radius(4.0)
                                        .layout(|l| {
                                            l.padding((0, 12, 0, 12)).gap(6).align(CenterX, CenterY)
                                        })
                                        .children(|ui| {
                                            ui.text("Open in browser", |t| {
                                                t.font_size(12).color(0x88aaff)
                                            });
                                            ui.text(nf::EXTERNAL_LINK, |t| {
                                                t.font_size(12).font(&SYMBOL_FONT).color(0x88aaff)
                                            });
                                        });
                                });
                        });
                }

                // ── Alert detail modal (click-for-detail) ───────────────────
                if let Some(alert_modal) = &state.alert_modal {
                    let modal_w = 640.0;
                    let modal_h = screen_height() * 0.7;
                    let modal_x = (screen_width() - modal_w) / 2.0;
                    let modal_y = (screen_height() - modal_h) / 2.0;
                    let content_h = modal_h - 36.0 - 24.0;

                    // Semi-transparent backdrop (click to close)
                    ui.element()
                        .id("alert-modal-backdrop")
                        .width(fixed!(screen_width()))
                        .height(fixed!(screen_height()))
                        .background_color((0.0f32, 0.0f32, 0.0f32, 220.0f32))
                        .floating(|f| f.offset((0.0, 0.0)).z_index(200).attach_root())
                        .empty();

                    // Modal panel (frosted glass)
                    glass_panel::glass(ui.element().width(fixed!(modal_w)).height(fixed!(modal_h)))
                        .floating(|f| f.offset((modal_x, modal_y)).z_index(201).attach_root())
                        .layout(|l| l.direction(TopToBottom).padding(0).gap(0))
                        .children(|ui| {
                            // Title bar
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
                                    ui.text(&alert_modal.title, |t| {
                                        t.font_size(14).color(0xE8E0DC)
                                    });
                                    ui.element().width(grow!()).height(fixed!(1.0)).empty();
                                    // Close button
                                    ui.element()
                                        .id("alert-modal-close")
                                        .width(fixed!(28.0))
                                        .height(fixed!(28.0))
                                        .background_color(0x3a1a1a)
                                        .corner_radius(4.0)
                                        .layout(|l| l.align(CenterX, CenterY))
                                        .children(|ui| {
                                            ui.text(nf::CLOSE, |t| {
                                                t.font_size(14).font(&SYMBOL_FONT).color(0xE8E0DC)
                                            });
                                        });
                                });

                            // Content area
                            ui.element()
                                .width(grow!())
                                .height(grow!())
                                .background_color(0x0a0d12)
                                .layout(|l| l.padding(12).gap(8).direction(TopToBottom))
                                .children(|ui| {
                                    let lines = wrap_modal_text(&alert_modal.content);
                                    let visible_count =
                                        (content_h / NHC_MODAL_LINE_HEIGHT).floor().max(1.0)
                                            as usize;
                                    let max_first = lines.len().saturating_sub(visible_count);
                                    let first = (state.alert_modal_scroll / NHC_MODAL_LINE_HEIGHT)
                                        .floor()
                                        as usize;
                                    let first = first.min(max_first);
                                    let last = (first + visible_count).min(lines.len());
                                    let window = lines[first..last].join("\n");
                                    ui.text(&window, |t| {
                                        t.font_size(11)
                                            .color(0x9E9590)
                                            .line_height(NHC_MODAL_LINE_HEIGHT as u16)
                                            .wrap_mode(ply_engine::text::WrapMode::Newline)
                                    });
                                });
                        });
                }

                // ── Settings modal (Stage 7) ────────────────────────────
                if state.show_settings_panel {
                    let site = &geo::RADAR_SITES[state.site_index];
                    let loc_status = match state.location_resolver.status() {
                        rustywx::location::LocationStatus::Idle => match state.user_location {
                            Some(c) => format!("{:.4}, {:.4}", c.lat, c.lon),
                            None => "Not set".to_string(),
                        },
                        rustywx::location::LocationStatus::Detecting => "Detecting…".to_string(),
                        rustywx::location::LocationStatus::Resolved(c, src) => {
                            format!("{:.4}, {:.4} ({:?})", c.lat, c.lon, src)
                        }
                        rustywx::location::LocationStatus::Denied => {
                            "Permission denied".to_string()
                        }
                        rustywx::location::LocationStatus::Offline => "Offline".to_string(),
                        rustywx::location::LocationStatus::NotFound => "ZIP not found".to_string(),
                        rustywx::location::LocationStatus::Invalid => "Invalid input".to_string(),
                    };
                    settings_widget::draw(
                        ui,
                        &state.settings,
                        site.id,
                        &state.settings.location_input,
                        state.location_input_focused,
                        &loc_status,
                        state.qc_report,
                        state.product.label(),
                    );
                }

                // ── Keyboard shortcuts overlay (Stage 7) ────────────────────
                if state.show_shortcuts {
                    shortcuts_widget::draw(ui);
                }

                // ── Error toast banner (Stage 7 error recovery) ─────────────
                if let Some(ref toast) = state.toast
                    && let Some(opacity) = toast.opacity(now)
                {
                    toast_widget::draw(ui, toast, opacity);
                }

                // ── Radar scope (transparent — drawn directly to screen) ──
                // Loading skeleton: pulsing indicator while first scan loads.
                if state.scan.is_none() {
                    ui.element()
                        .width(grow!())
                        .height(grow!())
                        .layout(|l| l.gap(8).align(CenterX, CenterY))
                        .children(|ui| {
                            let pulse = (0.5 + 0.5 * (now * 2.0).sin()) as f32;
                            let c = blend_hex(0x9E9590, 0x0dc5b8, pulse);
                            ui.text(nf::RADAR, |t| t.font_size(18).font(&SYMBOL_FONT).color(c));
                            ui.text("Loading radar data…", |t| {
                                t.font_size(18).font(&MONO_FONT).color(c)
                            });
                        });
                } else {
                    ui.element().width(grow!()).height(grow!()).empty();
                }

                // ── Bottom status bar: 2 rows (color key on top, data below);
                //    settings gear spans both rows on the right ────────────
                glass_panel::glass(ui.element().width(grow!()).height(fixed!(46.0)))
                    .layout(|layout| {
                        layout
                            .direction(LeftToRight)
                            .padding(6)
                            .gap(8)
                            .align(Left, CenterY)
                    })
                    .children(|ui| {
                        let has_real = state.scan.is_some();
                        let base_status = if has_real { 0x5F8A6A } else { 0x9E9590 };
                        // Pulse toward accent colour for ~1.2s after new data.
                        // Subtle halves the intensity; None disables it.
                        let raw_pulse =
                            (1.2 - (now - state.pulse_time).max(0.0)).clamp(0.0, 1.0) as f32;
                        let pulse = match state.settings.animation_level {
                            AnimationLevel::Full => raw_pulse,
                            AnimationLevel::Subtle => raw_pulse * 0.5,
                            AnimationLevel::None => 0.0,
                        };
                        let status_color = if pulse > 0.0 && state.pulse_time > 0.0 {
                            blend_hex(base_status, 0x0dc5b8, pulse)
                        } else {
                            base_status
                        };
                        let legend: &[(f32, [u8; 4])] = match state.product {
                            Product::Reflectivity => colors::DBZ_LEGEND,
                            Product::Velocity => colors::VELOCITY_LEGEND,
                            Product::SpectrumWidth => colors::SPECTRUM_WIDTH_LEGEND,
                            Product::DifferentialReflectivity => colors::ZDR_LEGEND,
                            Product::CorrelationCoefficient => colors::CC_LEGEND,
                            Product::DifferentialPhase => colors::PHIDP_LEGEND,
                            Product::SpecificDifferentialPhase => colors::KDP_LEGEND,
                        };

                        // Left: two stacked rows (color key + radar readout).
                        // Both are map-only — hidden in Forecast mode; a grow
                        // spacer keeps the gear pinned to the right edge.
                        if state.view_mode == ViewMode::Forecast {
                            ui.element().width(grow!()).height(grow!()).empty();
                        } else {
                            ui.element()
                                .width(grow!())
                                .height(grow!())
                                .layout(|layout| {
                                    layout.direction(TopToBottom).gap(4).align(Left, CenterY)
                                })
                                .children(|ui| {
                                    // Row 1 — color key.
                                    ui.element()
                                        .width(grow!())
                                        .height(fit!())
                                        .layout(|layout| {
                                            layout
                                                .direction(LeftToRight)
                                                .gap(8)
                                                .align(Left, CenterY)
                                        })
                                        .children(|ui| {
                                            for &(_threshold, color) in legend.iter().step_by(2) {
                                                let hex = (color[0] as u32) << 16
                                                    | (color[1] as u32) << 8
                                                    | (color[2] as u32);
                                                ui.element()
                                                    .width(fixed!(14.0))
                                                    .height(fixed!(10.0))
                                                    .background_color(hex)
                                                    .empty();
                                            }
                                            ui.text(state.product.units(), |text| {
                                                text.font_size(10).font(&MONO_FONT).color(0x5F8A6A)
                                            });

                                            // ── Alert legend (watches/warnings key) ─────
                                            if (state.show_watches || state.show_warnings)
                                                && !state.alerts.is_empty()
                                            {
                                                const ALERT_LEGEND_CAP: usize = 4;
                                                let mut seen: Vec<(&str, [u8; 4])> = Vec::new();
                                                for a in &state.alerts {
                                                    if !seen.iter().any(|(e, _)| *e == a.event) {
                                                        seen.push((a.event.as_str(), a.color));
                                                    }
                                                }
                                                let total = seen.len();
                                                for &(event, color) in
                                                    seen.iter().take(ALERT_LEGEND_CAP)
                                                {
                                                    let hex = (color[0] as u32) << 16
                                                        | (color[1] as u32) << 8
                                                        | (color[2] as u32);
                                                    ui.element()
                                                        .width(fixed!(14.0))
                                                        .height(fixed!(10.0))
                                                        .background_color(hex)
                                                        .empty();
                                                    ui.text(event, |text| {
                                                        text.font_size(10)
                                                            .font(&MONO_FONT)
                                                            .color(0x9E9590)
                                                    });
                                                }
                                                if total > ALERT_LEGEND_CAP {
                                                    ui.text(
                                                        &format!(
                                                            "+{} more",
                                                            total - ALERT_LEGEND_CAP
                                                        ),
                                                        |text| {
                                                            text.font_size(10)
                                                                .font(&MONO_FONT)
                                                                .color(0x9E9590)
                                                        },
                                                    );
                                                }
                                            }
                                        });
                                    // Row 2 — status + zoom/pan.
                                    ui.element()
                                        .width(grow!())
                                        .height(fit!())
                                        .layout(|layout| {
                                            layout
                                                .direction(LeftToRight)
                                                .gap(12)
                                                .align(Left, CenterY)
                                        })
                                        .children(|ui| {
                                            ui.text(&state.status_text, |text| {
                                                text.font_size(11)
                                                    .font(&MONO_FONT)
                                                    .color(status_color)
                                            });
                                            ui.text(
                                                &format!(
                                                    "Zoom: {:.1}x  Pan: ({:.0}, {:.0}) km",
                                                    state.zoom, state.pan_km.0, state.pan_km.1
                                                ),
                                                |text| {
                                                    text.font_size(11)
                                                        .font(&MONO_FONT)
                                                        .color(0x9E9590)
                                                },
                                            );
                                            if let Some(hint) = &state.melting_layer_hint {
                                                ui.text(
                                                    &format!(
                                                        "Melting layer: ~{:.1} km ({:.1}° tilt)",
                                                        hint.height_km, hint.elevation_deg
                                                    ),
                                                    |text| {
                                                        text.font_size(11)
                                                            .font(&MONO_FONT)
                                                            .color(0x9E9590)
                                                    },
                                                );
                                            }
                                        });
                                });
                        }

                        // Settings gear — spans both rows (full bar height).
                        let gear_bg = hover_tint(
                            &state.hovered_ids,
                            "btn-settings",
                            if state.show_settings_panel {
                                0x0dc5b8
                            } else {
                                0x1E1B1B
                            },
                            0x1E1B1B,
                        );
                        ui.element()
                            .id("btn-settings")
                            .width(fixed!(34.0))
                            .height(grow!())
                            .background_color(gear_bg)
                            .corner_radius(4.0)
                            // fa-gear sits ~5px right / ~1.5px high of centre in
                            // Symbols Nerd Font Mono — same quirk as btn-close.
                            // ponytail: per-glyph nudge, not a metrics fix.
                            .layout(|layout| layout.padding((3, 10, 0, 0)).align(CenterX, CenterY))
                            .accessibility(|a| a.button("Settings"))
                            .children(|ui| {
                                ui.text(nf::GEAR, |text| {
                                    text.font_size(18).font(&SYMBOL_FONT).color(0xE8E0DC)
                                });
                            });
                    });
            });

        ui.show(draw_rain_chart).await;

        // ── Draw NHC modal image ───────────────────────────────────
        // Text products are rendered exclusively by Ply inside the modal.
        // Rendering them here as well caused every line to appear twice.
        if let NhcModal::Image { title, .. } = &state.nhc_modal
            && let Some(ref bundle) = state.nhc_bundle
            && let Some(meta) = bundle.metas.get(state.nhc_selected_storm)
        {
            let key = format!("{}:{}", meta.id, title);
            if let Some(tex) = state.nhc_image_textures.get(&key) {
                let modal_w = 640.0;
                let modal_h = screen_height() * 0.7;
                let modal_x = (screen_width() - modal_w) / 2.0;
                let modal_y = (screen_height() - modal_h) / 2.0;
                let content_x = modal_x + 12.0;
                let content_y = modal_y + 36.0 + 12.0;
                let content_w = modal_w - 24.0;
                let content_h = modal_h - 36.0 - 40.0 - 24.0;
                let tex_w = tex.width();
                let tex_h = tex.height();
                let scale = (content_w / tex_w).min(content_h / tex_h).min(1.0);
                let draw_w = tex_w * scale;
                let draw_h = tex_h * scale;
                let draw_x = content_x + (content_w - draw_w) / 2.0;
                let draw_y = content_y + (content_h - draw_h) / 2.0;
                draw_texture_ex(
                    tex,
                    draw_x,
                    draw_y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(draw_w, draw_h)),
                        ..Default::default()
                    },
                );
            }
        }

        // ── Input handling ─────────────────────────────────────────
        handle_input(&mut state, &ply, &site_options, &tilt_options);

        next_frame().await;
    }
}

// ---------------------------------------------------------------------------
// NHC panel helpers
// ---------------------------------------------------------------------------

/// Draw a compact toggle button for an NHC overlay layer.
fn nhc_toggle_button(
    ui: &mut Ui<'_, rustywx::widgets::ChartWidget>,
    id: &'static str,
    label: &str,
    active: bool,
) {
    let bg = if active { 0x0dc5b8 } else { 0x1E1B1B };
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
        .accessibility(|a| a.checkbox(label).checked(active))
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
    ply: &Ply<rustywx::widgets::ChartWidget>,
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
    let modal_open = !matches!(state.nhc_modal, NhcModal::None)
        || state.alert_modal.is_some()
        || state.show_settings_panel
        || state.show_shortcuts;
    let over_nhc_panel = state.nhc_show_panel && ply.pointer_over("nhc-panel");
    let over_radar_panel = state.radar_panel_open && ply.pointer_over("radar-panel");

    // Scope-specific input (pan/zoom, hit-testing, radar keyboard shortcuts)
    // only applies while the radar scope is actually visible. In Forecast
    // mode the scope is hidden, so this whole region is gated off.
    if state.view_mode == ViewMode::Radar {
        if !dropdown_open
            && !modal_open
            && !over_nhc_panel
            && !over_radar_panel
            && is_mouse_button_down(MouseButton::Left)
        {
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

        if !dropdown_open && !modal_open && !over_nhc_panel {
            let scroll = mouse_wheel().1;
            if scroll != 0.0 {
                // 0.05 per unit = ~5-25% per wheel notch (vs old 0.001 = 0.1-0.5%)
                state.zoom = (state.zoom * (1.0 + scroll * 0.05)).clamp(0.05, 8.0);
            }

            // ── Double-click on a radar site marker to select it ──────
            if is_mouse_button_pressed(MouseButton::Left) {
                let (mx, my) = mouse_position();
                let now = get_time();
                let dt = now - state.last_click_time;
                let (lx, ly) = state.last_click_pos;
                let moved = (mx - lx).abs() + (my - ly).abs();
                // Double-click: second press within 400ms and 10px of the first.
                if dt < 0.4 && moved < 10.0 {
                    let center = &geo::RADAR_SITES[state.site_index];
                    // Hit-test all radar site markers (12px radius).
                    let hit_radius = 14.0;
                    let mut best: Option<(usize, f32)> = None;
                    for (i, other) in geo::RADAR_SITES.iter().enumerate() {
                        if i == state.site_index {
                            continue;
                        }
                        let (sx, sy) = scope::project_site(
                            other.lat,
                            other.lon,
                            center,
                            state.pan_km,
                            state.zoom,
                        );
                        let dist = ((mx - sx).powi(2) + (my - sy).powi(2)).sqrt();
                        if dist < hit_radius && best.is_none_or(|(_, d)| dist < d) {
                            best = Some((i, dist));
                        }
                    }
                    if let Some((index, _)) = best {
                        select_site(state, index);
                    } else {
                        // No site marker hit — double-click on an alert polygon
                        // opens its detail modal. (Single-click stays free for
                        // panning, so map drags don't pop modals accidentally.)
                        let side = screen_width().min(screen_height());
                        let px_per_km = (side / 2.0) / scope::MAX_RANGE_KM * state.zoom;
                        let center_x = screen_width() / 2.0 + state.pan_km.0 * px_per_km;
                        let center_y = screen_height() / 2.0 + state.pan_km.1 * px_per_km;
                        if let Some(alert) = alerts::hit_test(
                            &state.alerts,
                            state.show_watches,
                            state.show_warnings,
                            center,
                            (mx, my),
                            (center_x, center_y),
                            px_per_km,
                        ) {
                            state.alert_modal = Some(AlertModal {
                                title: alert.event.clone(),
                                content: alert_modal_body(alert),
                            });
                        }
                    }
                    // Reset so a third click doesn't re-trigger.
                    state.last_click_time = 0.0;
                } else {
                    state.last_click_time = now;
                    state.last_click_pos = (mx, my);
                }
            }
        }

        if !dropdown_open && !modal_open {
            if is_key_pressed(KeyCode::R) {
                select_product(state, Product::Reflectivity);
            }
            if is_key_pressed(KeyCode::V) {
                select_product(state, Product::Velocity);
            }
            if is_key_pressed(KeyCode::W) {
                select_product(state, Product::SpectrumWidth);
            }
            if is_key_pressed(KeyCode::Z) {
                select_product(state, Product::DifferentialReflectivity);
            }
            if is_key_pressed(KeyCode::C) {
                select_product(state, Product::CorrelationCoefficient);
            }
            if is_key_pressed(KeyCode::P) {
                select_product(state, Product::DifferentialPhase);
            }
            if is_key_pressed(KeyCode::K) {
                select_product(state, Product::SpecificDifferentialPhase);
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
            if is_key_pressed(KeyCode::W) {
                state.show_watches = !state.show_watches;
            }
            if is_key_pressed(KeyCode::A) {
                state.show_warnings = !state.show_warnings;
            }
        }
    } // state.view_mode == ViewMode::Radar

    if let Some(product) = toggle::pressed(ply, &PRODUCT_OPTIONS) {
        select_product(state, product);
    }

    // ── Overlay toggle button presses (Stage 4) ──────────────────
    if ply.is_just_pressed("btn-borders") {
        state.show_borders = !state.show_borders;
    }
    if ply.is_just_pressed("btn-watches") {
        state.show_watches = !state.show_watches;
    }
    if ply.is_just_pressed("btn-warnings") {
        state.show_warnings = !state.show_warnings;
    }
    if ply.is_just_pressed("btn-radar-data") {
        state.show_radar_data = !state.show_radar_data;
        state.settings.show_radar_data = state.show_radar_data;
        state.cache.save_settings(&state.settings);
    }
    if ply.is_just_pressed("btn-tropical-data") {
        state.show_nhc_data = !state.show_nhc_data;
        state.settings.show_nhc_data = state.show_nhc_data;
        state.cache.save_settings(&state.settings);
    }
    if ply.is_just_pressed("btn-location") {
        state.show_location = !state.show_location;
        state.settings.show_location = state.show_location;
        state.cache.save_settings(&state.settings);
        // Turning the marker on triggers a snap-to-location: recenter now if
        // we already have a fix, otherwise kick off auto-resolution (system →
        // IP) and snap once it lands (see the resolver-poll block).
        if state.show_location {
            if state.user_location.is_some() {
                recenter_on_user(state);
            } else {
                state.location_error_shown = false;
                state.location_resolver.detect("", get_time());
                state.center_pending = true;
            }
        } else {
            // Toggled off: cancel any pending snap so a late fix doesn't move
            // the map after the marker is hidden.
            state.center_pending = false;
        }
    }

    // ── Window controls: fullscreen toggle + close ───────────────
    if ply.is_just_pressed("btn-fullscreen")
        || (!dropdown_open && !state.location_input_focused && is_key_pressed(KeyCode::F))
    {
        state.fullscreen = !state.fullscreen;
        miniquad::window::set_fullscreen(state.fullscreen);
    }
    if ply.is_just_pressed("btn-close") {
        miniquad::window::order_quit();
    }

    // ── Radar toggle button ──────────────────────────────────────
    if ply.is_just_pressed("btn-radar") {
        state.view_mode = ViewMode::Radar;
        state.radar_panel_open = !state.radar_panel_open;
        // Replay the slide-in next open; drop open dropdowns on close so
        // their floating popups don't linger.
        state.radar_anim_start = 0.0;
        if state.radar_panel_open {
            state.nhc_show_panel = false; // right-edge panels are exclusive
        } else {
            state.site_dropdown.close();
            state.tilt_dropdown.close();
        }
        state.settings.show_radar = state.radar_panel_open;
        state.cache.save_settings(&state.settings);
    }

    // ── Tropical panel toggle button (Stage 5) ────────────────────
    if ply.is_just_pressed("btn-tropical") {
        state.view_mode = ViewMode::Radar;
        state.nhc_show_panel = !state.nhc_show_panel;
        if !state.nhc_show_panel {
            state.nhc_anim_start = 0.0;
        } else {
            state.radar_panel_open = false; // right-edge panels are exclusive
        }
    }

    // ── Forecast view toggle ──────────────────────────────────────
    if ply.is_just_pressed("btn-forecast") {
        state.view_mode = if state.view_mode == ViewMode::Forecast {
            ViewMode::Radar
        } else {
            // Entering forecast: close the scope side panels (exclusive).
            state.radar_panel_open = false;
            state.nhc_show_panel = false;
            ViewMode::Forecast
        };
    }
    // `N` toggles the tropical *data* layer (not the panel), matching the
    // B/W/A data-toggle family.
    if !dropdown_open && !state.location_input_focused && is_key_pressed(KeyCode::N) {
        state.show_nhc_data = !state.show_nhc_data;
        state.settings.show_nhc_data = state.show_nhc_data;
        state.cache.save_settings(&state.settings);
    }

    // ── Settings gear button and modal (Stage 7) ──────────────────
    if ply.is_just_pressed("btn-settings") {
        state.show_settings_panel = !state.show_settings_panel;
    }
    if state.show_settings_panel {
        if ply.is_just_pressed(settings_widget::CLOSE_ID)
            || ply.is_just_pressed(settings_widget::BACKDROP_ID)
            || is_key_pressed(KeyCode::Escape)
        {
            state.show_settings_panel = false;
        }
        if ply.is_just_pressed(settings_widget::BORDERS_TOGGLE_ID) {
            state.settings.show_borders = !state.settings.show_borders;
            state.cache.save_settings(&state.settings);
        }
        if ply.is_just_pressed(settings_widget::DYSLEXIC_TOGGLE_ID) {
            state.settings.dyslexic_font = !state.settings.dyslexic_font;
            state.cache.save_settings(&state.settings);
        }
        if ply.is_just_pressed(settings_widget::WATCHES_TOGGLE_ID) {
            state.settings.show_watches = !state.settings.show_watches;
            state.cache.save_settings(&state.settings);
        }
        if ply.is_just_pressed(settings_widget::WARNINGS_TOGGLE_ID) {
            state.settings.show_warnings = !state.settings.show_warnings;
            state.cache.save_settings(&state.settings);
        }
        if ply.is_just_pressed(settings_widget::NHC_TOGGLE_ID) {
            state.settings.show_nhc = !state.settings.show_nhc;
            state.cache.save_settings(&state.settings);
        }
        if ply.is_just_pressed(settings_widget::SWEEP_TOGGLE_ID) {
            state.settings.show_sweep = !state.settings.show_sweep;
            state.cache.save_settings(&state.settings);
        }
        if ply.is_just_pressed(settings_widget::SCOPE_RINGS_TOGGLE_ID) {
            state.settings.show_scope_rings = !state.settings.show_scope_rings;
            state.cache.save_settings(&state.settings);
        }
        if ply.is_just_pressed(settings_widget::CC_GATE_TOGGLE_ID) {
            state.settings.cc_gate_enabled = !state.settings.cc_gate_enabled;
            state.cache.save_settings(&state.settings);
            state.needs_reraster = true;
        }
        if ply.is_just_pressed(settings_widget::NONMET_FUZZY_TOGGLE_ID) {
            state.settings.nonmet_fuzzy_enabled = !state.settings.nonmet_fuzzy_enabled;
            state.cache.save_settings(&state.settings);
            state.needs_reraster = true;
        }
        if ply.is_just_pressed(settings_widget::REFL_FLOOR_TOGGLE_ID) {
            state.settings.refl_floor_enabled = !state.settings.refl_floor_enabled;
            state.cache.save_settings(&state.settings);
            state.needs_reraster = true;
        }
        if ply.is_just_pressed(settings_widget::VEL_DEALIAS_TOGGLE_ID) {
            state.settings.vel_dealias_enabled = !state.settings.vel_dealias_enabled;
            state.cache.save_settings(&state.settings);
            state.needs_reraster = true;
        }
        if ply.is_just_pressed(settings_widget::VEL_SD_TOGGLE_ID) {
            state.settings.vel_sd_censor_enabled = !state.settings.vel_sd_censor_enabled;
            state.cache.save_settings(&state.settings);
            state.needs_reraster = true;
        }
        if ply.is_just_pressed(settings_widget::GAP_FILL_TOGGLE_ID) {
            state.settings.refl_gap_fill_enabled = !state.settings.refl_gap_fill_enabled;
            state.cache.save_settings(&state.settings);
            state.needs_reraster = true;
        }
        if ply.is_just_pressed(settings_widget::MULTI_SCALE_TEXTURE_TOGGLE_ID) {
            state.settings.multi_scale_texture_enabled =
                !state.settings.multi_scale_texture_enabled;
            state.cache.save_settings(&state.settings);
            state.needs_reraster = true;
        }
        if ply.is_just_pressed(settings_widget::SUN_SPIKE_TOGGLE_ID) {
            state.settings.sun_spike_removal_enabled = !state.settings.sun_spike_removal_enabled;
            state.cache.save_settings(&state.settings);
            state.needs_reraster = true;
        }
        if ply.is_just_pressed(settings_widget::MELTING_LAYER_TOGGLE_ID) {
            state.settings.melting_layer_hint_enabled = !state.settings.melting_layer_hint_enabled;
            state.cache.save_settings(&state.settings);
            // Not a raster QC pass — this just re-triggers the hint
            // recompute that lives alongside the raster (see above).
            state.needs_reraster = true;
        }
        // Threshold steppers: nudge each QC knob, clamp to a sane range,
        // persist, and re-raster so the effect is visible immediately.
        {
            let s = &mut state.settings;
            let mut qc_changed = false;
            if ply.is_just_pressed(settings_widget::CC_GATE_DEC_ID) {
                s.cc_gate_threshold = (s.cc_gate_threshold - 0.05).clamp(0.0, 1.0);
                qc_changed = true;
            }
            if ply.is_just_pressed(settings_widget::CC_GATE_INC_ID) {
                s.cc_gate_threshold = (s.cc_gate_threshold + 0.05).clamp(0.0, 1.0);
                qc_changed = true;
            }
            if ply.is_just_pressed(settings_widget::NONMET_DEC_ID) {
                s.nonmet_threshold = (s.nonmet_threshold - 0.05).clamp(0.0, 1.0);
                qc_changed = true;
            }
            if ply.is_just_pressed(settings_widget::NONMET_INC_ID) {
                s.nonmet_threshold = (s.nonmet_threshold + 0.05).clamp(0.0, 1.0);
                qc_changed = true;
            }
            if ply.is_just_pressed(settings_widget::REFL_FLOOR_DEC_ID) {
                s.refl_floor_dbz = (s.refl_floor_dbz - 1.0).clamp(0.0, 40.0);
                qc_changed = true;
            }
            if ply.is_just_pressed(settings_widget::REFL_FLOOR_INC_ID) {
                s.refl_floor_dbz = (s.refl_floor_dbz + 1.0).clamp(0.0, 40.0);
                qc_changed = true;
            }
            if ply.is_just_pressed(settings_widget::VEL_SD_DEC_ID) {
                s.vel_sd_threshold = (s.vel_sd_threshold - 1.0).clamp(0.0, 20.0);
                qc_changed = true;
            }
            if ply.is_just_pressed(settings_widget::VEL_SD_INC_ID) {
                s.vel_sd_threshold = (s.vel_sd_threshold + 1.0).clamp(0.0, 20.0);
                qc_changed = true;
            }
            if qc_changed {
                state.cache.save_settings(&state.settings);
                state.needs_reraster = true;
            }
        }
        if ply.is_just_pressed(settings_widget::ANIMATION_CYCLE_ID) {
            state.settings.animation_level = state.settings.animation_level.next();
            state.cache.save_settings(&state.settings);
        }
        if ply.is_just_pressed(settings_widget::TDBZ_CYCLE_ID) {
            state.settings.tdbz_kernel = state.settings.tdbz_kernel.next();
            state.cache.save_settings(&state.settings);
            state.needs_reraster = true;
        }
        if ply.is_just_pressed(settings_widget::USE_CURRENT_SITE_ID) {
            state.settings.default_site = geo::RADAR_SITES[state.site_index].id.to_string();
            state.cache.save_settings(&state.settings);
        }

        // Focus the location field on click; blur when clicking elsewhere.
        if ply.is_just_pressed(settings_widget::LOCATION_INPUT_ID) {
            state.location_input_focused = true;
        }
        // Capture typing while focused (typing and Escape key off disjoint
        // KeyCodes, so their relative ordering doesn't matter).
        if state.location_input_focused {
            let now = get_time();
            while let Some(ch) = get_char_pressed() {
                if !ch.is_control() && state.settings.location_input.len() < 32 {
                    state.settings.location_input.push(ch);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                state.settings.location_input.pop();
            }
            if is_key_pressed(KeyCode::Enter) {
                state.location_input_focused = false;
                state.location_error_shown = false;
                if let Some(c) = state
                    .location_resolver
                    .detect(&state.settings.location_input, now)
                {
                    state.user_location = Some(c);
                    state.settings.user_lat = Some(c.lat);
                    state.settings.user_lon = Some(c.lon);
                    // Relocating clears the stale place label so the Forecast
                    // fetch block recomputes it from the new coordinates.
                    state.forecast_place = String::new();
                    if state.settings.center_on_location {
                        recenter_on_user(state);
                    }
                }
                state.cache.save_settings(&state.settings);
            }
        }
        if ply.is_just_pressed(settings_widget::LOCATION_DETECT_ID) {
            let now = get_time();
            state.location_input_focused = false;
            state.location_error_shown = false;
            if let Some(c) = state
                .location_resolver
                .detect(&state.settings.location_input, now)
            {
                state.user_location = Some(c);
                state.settings.user_lat = Some(c.lat);
                state.settings.user_lon = Some(c.lon);
                // Relocating clears the stale place label so the Forecast
                // fetch block recomputes it from the new coordinates.
                state.forecast_place = String::new();
                if state.settings.center_on_location {
                    recenter_on_user(state);
                }
            }
            state.cache.save_settings(&state.settings);
        }
        if ply.is_just_pressed(settings_widget::CENTER_TOGGLE_ID) {
            state.settings.center_on_location = !state.settings.center_on_location;
            state.cache.save_settings(&state.settings);
            if state.settings.center_on_location {
                recenter_on_user(state);
            }
        }
    }

    // ── Forecast view: fetch, poll, and search input ────────────
    if state.view_mode == ViewMode::Forecast {
        // Target coords: a picked search hit sets forecast_coords directly
        // (handled below); otherwise follow the app's user_location.
        if let Some(target) = state.user_location {
            let stale = state.forecast_coords != Some(target);
            if stale && !state.forecast_fetch_fired {
                // Derive a place label if we don't already have one.
                if state.forecast_place.is_empty() {
                    state.forecast_place = if !state.settings.location_input.is_empty() {
                        state.settings.location_input.clone()
                    } else {
                        format!("{:.2}, {:.2}", target.lat, target.lon)
                    };
                }
                forecast::fire_forecast(target);
                state.forecast_fetch_fired = true;
                state.forecast_error = None;
                // Remember which coords we're fetching for.
                state.forecast_coords = Some(target);
            }
        }
        // Poll whichever coords we last fired for.
        if state.forecast_fetch_fired
            && let Some(coords) = state.forecast_coords
        {
            match forecast::poll_forecast(coords) {
                Some(Ok(mut fc)) => {
                    fc.place = state.forecast_place.clone();
                    state.forecast = Some(fc);
                    state.forecast_fetch_fired = false;
                }
                Some(Err(e)) => {
                    state.forecast_error = Some(e.to_string());
                    state.forecast_fetch_fired = false;
                }
                None => {}
            }
        }

        // Search box focus (click) + typing.
        if ply.is_just_pressed("fc-search") {
            state.fc_search_focused = true;
        }
        if state.fc_search_focused {
            while let Some(ch) = get_char_pressed() {
                if !ch.is_control() && state.fc_search_text.len() < 48 {
                    state.fc_search_text.push(ch);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                state.fc_search_text.pop();
            }
            if is_key_pressed(KeyCode::Enter) {
                state.fc_search_focused = false;
                let q = state.fc_search_text.trim().to_string();
                if !q.is_empty() {
                    forecast::fire_geo(&q);
                    state.fc_geo_fired = true;
                }
            }
            if is_key_pressed(KeyCode::Escape) {
                state.fc_search_focused = false;
            }
        }
        // Poll geocode results.
        if state.fc_geo_fired {
            let q = state.fc_search_text.trim().to_string();
            if !q.is_empty() {
                match forecast::poll_geo(&q) {
                    Some(Ok(hits)) => {
                        state.fc_geo_hits = hits;
                        state.fc_geo_fired = false;
                    }
                    Some(Err(_)) => {
                        state.fc_geo_hits = Vec::new();
                        state.fc_geo_fired = false;
                    }
                    None => {}
                }
            }
        }
        // Pick a search result.
        for idx in 0..state.fc_geo_hits.len() {
            if ply.is_just_pressed(("fc-hit", idx as u32)) {
                let hit = state.fc_geo_hits[idx].clone();
                state.user_location = Some(hit.coords);
                state.forecast_place = hit.label;
                state.forecast_coords = None; // force refetch for new coords
                state.forecast_fetch_fired = false;
                state.forecast = None;
                state.fc_geo_hits = Vec::new();
                state.fc_search_text.clear();
                break;
            }
        }
    }

    // ── Keyboard shortcuts overlay (Stage 7) ─────────────────────────
    // ? key (Shift+/) toggles the shortcuts overlay. If both settings and
    // shortcuts would be open, close settings first for cleaner UX.
    if !dropdown_open && is_key_pressed(KeyCode::Slash) && is_key_down(KeyCode::LeftShift) {
        if state.show_settings_panel {
            state.show_settings_panel = false;
        }
        state.show_shortcuts = !state.show_shortcuts;
    }
    if state.show_shortcuts
        && (ply.is_just_pressed(shortcuts_widget::CLOSE_ID)
            || ply.is_just_pressed(shortcuts_widget::BACKDROP_ID)
            || is_key_pressed(KeyCode::Escape))
    {
        state.show_shortcuts = false;
    }

    // ── Error toast dismissal (Stage 7) ──────────────────────────────
    // Click-to-dismiss, or drop it once fully faded (see `Toast::opacity`).
    if state.toast.is_some()
        && (ply.is_just_pressed(toast_widget::DISMISS_ID)
            || state
                .toast
                .as_ref()
                .is_some_and(|t| t.opacity(get_time()).is_none()))
    {
        state.toast = None;
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
    if ply.is_just_pressed("btn-nhc-earliest") {
        state.nhc_overlays.show_earliest_arrival = !state.nhc_overlays.show_earliest_arrival;
    }
    if ply.is_just_pressed("btn-nhc-likely") {
        state.nhc_overlays.show_most_likely_arrival = !state.nhc_overlays.show_most_likely_arrival;
    }

    // ── NHC modal scrolling / close / browser buttons ─────────────
    if modal_open {
        if let Some((_, visible_lines, max_scroll)) = nhc_modal_text_metrics(state) {
            let page = visible_lines.saturating_sub(2).max(1) as f32 * NHC_MODAL_LINE_HEIGHT;
            let wheel = mouse_wheel().1;
            if wheel != 0.0 {
                state.nhc_modal_scroll -= wheel * NHC_MODAL_LINE_HEIGHT * 3.0;
            }
            if is_key_pressed(KeyCode::Down) {
                state.nhc_modal_scroll += NHC_MODAL_LINE_HEIGHT;
            }
            if is_key_pressed(KeyCode::Up) {
                state.nhc_modal_scroll -= NHC_MODAL_LINE_HEIGHT;
            }
            if is_key_pressed(KeyCode::PageDown) {
                state.nhc_modal_scroll += page;
            }
            if is_key_pressed(KeyCode::PageUp) {
                state.nhc_modal_scroll -= page;
            }
            if is_key_pressed(KeyCode::Home) {
                state.nhc_modal_scroll = 0.0;
            }
            if is_key_pressed(KeyCode::End) {
                state.nhc_modal_scroll = max_scroll;
            }
            state.nhc_modal_scroll = state.nhc_modal_scroll.clamp(0.0, max_scroll);
        }

        if ply.is_just_pressed("nhc-modal-close") || ply.is_just_pressed("nhc-modal-backdrop") {
            state.nhc_modal = NhcModal::None;
            state.nhc_modal_scroll = 0.0;
        }
        if ply.is_just_pressed("nhc-modal-browser") {
            let url = match &state.nhc_modal {
                NhcModal::Text { url, .. } => url.clone(),
                NhcModal::Image { url, .. } => url.clone(),
                _ => String::new(),
            };
            if !url.is_empty() {
                let _ = webbrowser::open(&url);
            }
        }
        if is_key_pressed(KeyCode::Escape) {
            state.nhc_modal = NhcModal::None;
            state.nhc_modal_scroll = 0.0;
        }
    }

    // ── Alert modal scrolling / close ──────────────────────────────
    if state.alert_modal.is_some() {
        if let Some((_, visible_lines, max_scroll)) = alert_modal_text_metrics(state) {
            let page = visible_lines.saturating_sub(2).max(1) as f32 * NHC_MODAL_LINE_HEIGHT;
            let wheel = mouse_wheel().1;
            if wheel != 0.0 {
                state.alert_modal_scroll -= wheel * NHC_MODAL_LINE_HEIGHT * 3.0;
            }
            if is_key_pressed(KeyCode::Down) {
                state.alert_modal_scroll += NHC_MODAL_LINE_HEIGHT;
            }
            if is_key_pressed(KeyCode::Up) {
                state.alert_modal_scroll -= NHC_MODAL_LINE_HEIGHT;
            }
            if is_key_pressed(KeyCode::PageDown) {
                state.alert_modal_scroll += page;
            }
            if is_key_pressed(KeyCode::PageUp) {
                state.alert_modal_scroll -= page;
            }
            if is_key_pressed(KeyCode::Home) {
                state.alert_modal_scroll = 0.0;
            }
            if is_key_pressed(KeyCode::End) {
                state.alert_modal_scroll = max_scroll;
            }
            state.alert_modal_scroll = state.alert_modal_scroll.clamp(0.0, max_scroll);
        }

        if ply.is_just_pressed("alert-modal-close") || ply.is_just_pressed("alert-modal-backdrop") {
            state.alert_modal = None;
            state.alert_modal_scroll = 0.0;
        }
        if is_key_pressed(KeyCode::Escape) {
            state.alert_modal = None;
            state.alert_modal_scroll = 0.0;
        }
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
        for (img_idx, img) in images.iter().enumerate() {
            let id = ("btn-nhc-img", img_idx as u32);
            if ply.is_just_pressed(id) {
                state.nhc_modal = NhcModal::Image {
                    title: img.title.clone(),
                    url: img.url.clone(),
                };
                state.nhc_modal_scroll = 0.0;
            }
        }
        if let Some((_, texts)) = bundle.text_products.iter().find(|(id, _)| *id == meta.id) {
            for product in texts {
                let id = ("btn-nhc-text", product.title.len() as u32);
                if ply.is_just_pressed(id) {
                    state.nhc_modal = NhcModal::Text {
                        title: product.title.clone(),
                        content: product.content.clone(),
                        url: product.url.clone(),
                    };
                    state.nhc_modal_scroll = 0.0;
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

/// Surface a short, friendly error banner (Stage 7 error recovery). The raw
/// error detail stays in the `eprintln!` log at the call site — only the
/// canned [`toast_widget::friendly_message`] text reaches the user.
fn show_toast(state: &mut AppState, now: f64, kind: toast_widget::ErrorKind) {
    state.toast = Some(toast_widget::Toast::new(
        toast_widget::friendly_message(kind),
        now,
    ));
}

/// Set pan so the user's location sits at screen center. No-op if unknown.
fn recenter_on_user(state: &mut AppState) {
    if let Some(user) = state.user_location {
        let site = &geo::RADAR_SITES[state.site_index];
        let off = geo::point_to_km_offset(site.lat, site.lon, (user.lat, user.lon));
        state.pan_km = (-off.x, -off.y);
    }
}

fn select_site(state: &mut AppState, index: usize) {
    if index >= geo::RADAR_SITES.len() || index == state.site_index {
        return;
    }

    state.site_index = index;
    // Switching sites exits demo mode — the worker resumes live polling.
    state.demo = None;
    state.pending_demo = None;
    state.tilt_index = 0;
    state.tilt_dropdown.close();
    let site_id = geo::RADAR_SITES[index].id.to_string();
    let _ = state.site_tx.send(site_id.clone());
    // Persist the selection so the last-chosen site is restored on next launch.
    state.cache.save_site(&site_id);
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
        let vcp_num_enum = nexrad_model::data::VCPNumber::from_number(scan.vcp_number);
        let vcp_mode = vcp_mode_label(vcp_num_enum);
        // Nyquist is a velocity-product quantity; show it for the current
        // tilt regardless of which product is on screen (matches the
        // elevation/VCP context info alongside it). `state.tilt_index`
        // indexes the currently-displayed product's sweep list, not
        // Velocity's — the per-product lists are built and
        // sort_and_dedup'd independently and can diverge on split-cut
        // VCPs — so look up the Velocity sweep by nearest elevation_deg
        // instead of by index (mirrors the `cc_sweep` lookup above).
        let nyquist_ms = sweeps
            .get(state.tilt_index)
            .and_then(|sweep| {
                scan.sweeps(Product::Velocity).iter().min_by(|a, b| {
                    (a.elevation_deg - sweep.elevation_deg)
                        .abs()
                        .total_cmp(&(b.elevation_deg - sweep.elevation_deg).abs())
                })
            })
            .map(|sweep| sweep.nyquist_ms)
            .unwrap_or(0.0);
        let nyquist = format_nyquist_velocity(nyquist_ms);
        state.status_text = format!(
            "{} — {} — {} tilt(s){} — VCP {} — {} — {}{}",
            scan.timestamp.format("%Y-%m-%d %H:%M UTC"),
            geo::RADAR_SITES[state.site_index].id,
            sweeps.len(),
            elevation,
            scan.vcp_number,
            vcp_mode,
            nyquist,
            suffix,
        );
    }
    if let Some(label) = &state.demo
        && !state.status_text.starts_with("DEMO — ")
    {
        state.status_text = format!("DEMO — {label} — {}", state.status_text);
    }
}
