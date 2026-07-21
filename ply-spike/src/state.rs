//! Application state — replaces the egui-era `app.rs` struct.
//!
//! Held across frames in `main.rs`; mutated by input handling and worker
//! messages, read by the rendering closure.

use crate::alerts::Alert;
use crate::borders::Ring;
use crate::cache::Cache;
use crate::data::WorkerMessage;
use crate::model::{Product, ScanData};
use crate::nhc::{NhcBundle, NhcFetchState};
use crate::settings::Settings;
use crate::widgets::dropdown::DropdownState;
use ply_engine::prelude::Texture2D;
use std::collections::HashMap;
use std::sync::mpsc;
use tokio::sync::oneshot;

/// What's shown in the NHC product modal popup.
#[derive(Clone)]
pub enum NhcModal {
    /// No modal visible.
    None,
    /// Showing a text product (advisory, discussion, etc.).
    Text {
        title: String,
        content: String,
        url: String,
    },
    /// Showing a graphic product (image texture).
    Image { title: String, url: String },
}
/// Top-level state shared between the game loop, input handler, and worker.
pub struct AppState {
    /// Index into `geo::RADAR_SITES`.
    pub site_index: usize,
    /// Active radar product (Reflectivity or Velocity).
    pub product: Product,
    /// Pan offset in kilometres from scope centre.
    pub pan_km: (f32, f32),
    /// Zoom factor (1.0 = default).
    pub zoom: f32,
    /// Cached rasterised radar texture; rebuilt when `needs_reraster` is set.
    pub radar_texture: Option<Texture2D>,
    /// Dirty flag — re-rasterise the sweep on the next frame.
    pub needs_reraster: bool,

    // ── Persistence ────────────────────────────────────────────
    /// Ply storage cache handle (Clone — cheap to share).
    pub cache: Cache,
    /// Non-blocking load in flight, if any.
    pub pending_load: Option<oneshot::Receiver<Option<ScanData>>>,
    /// Pending site-preference load (first launch restores last site).
    pub pending_site_load: Option<oneshot::Receiver<Option<String>>>,

    // ── Real data (Stage 2 formalises caching/error states) ──────
    /// Latest decoded volume scan, if one has arrived.
    pub scan: Option<ScanData>,
    /// Current tilt index into `scan.sweeps(product)`.
    pub tilt_index: usize,
    /// Human-readable status line for the bottom bar.
    pub status_text: String,

    // ── Worker channels ──────────────────────────────────────────
    /// Receives `WorkerMessage`s from the background data thread.
    pub worker_rx: mpsc::Receiver<WorkerMessage>,
    /// Sends site-change requests to the background data thread.
    pub site_tx: mpsc::Sender<String>,

    // ── Custom controls ──────────────────────────────────────────
    /// Searchable radar-site selector state.
    pub site_dropdown: DropdownState,
    /// Available-elevation selector state.
    pub tilt_dropdown: DropdownState,

    // ── Borders & Alerts (Stage 4) ────────────────────────────────
    /// Cached state-boundary + coastline rings.
    pub borders: Vec<Ring>,
    /// Whether borders have been loaded (from cache or network).
    pub borders_loaded: bool,
    /// Whether the borders net requests have been fired.
    pub borders_fetch_fired: bool,
    /// Active NWS warnings/watches relevant to the current site.
    pub alerts: Vec<Alert>,
    /// Whether alerts have been fetched at least once.
    pub alerts_loaded: bool,
    /// Whether the alerts net request has been fired.
    pub alerts_fetch_fired: bool,
    /// Wall-clock time (seconds) of the last alerts refresh.
    pub last_alert_poll: f64,
    /// Toggle: show state borders on the scope.
    pub show_borders: bool,
    /// Toggle: show NWS alert polygons on the scope.
    pub show_alerts: bool,

    // ── NHC Tropical (Stage 5) ────────────────────────────────────
    /// NHC data bundle (storm metadata, GIS, text, images, contours).
    pub nhc_bundle: Option<NhcBundle>,
    /// NHC fetch state machine.
    pub nhc_fetch: NhcFetchState,
    /// Whether the NHC fetch has been started.
    pub nhc_fetch_fired: bool,
    /// Wall-clock time of the last NHC refresh.
    pub nhc_last_poll: f64,
    /// Whether the NHC detail panel is visible.
    pub nhc_show_panel: bool,
    /// Currently selected storm index into `nhc_bundle.metas`.
    pub nhc_selected_storm: usize,
    /// Storm selector dropdown state.
    pub nhc_storm_dropdown: DropdownState,
    /// Decoded image textures keyed by "storm_id:product_title".
    pub nhc_image_textures: HashMap<String, Texture2D>,
    /// NHC overlay toggle state.
    pub nhc_overlays: crate::scope::NhcOverlayState,
    /// Currently displayed NHC product modal.
    pub nhc_modal: NhcModal,
    /// Scroll offset (pixels) for NHC modal text content.
    pub nhc_modal_scroll: f32,

    /// Last mouse position for manual pan delta calculation.
    pub last_mouse_pos: Option<(f32, f32)>,

    // ── Stage 6: Observatory Look (animations / auto-hide) ──────────
    /// Wall-clock time at app start (for staggered entrance animation).
    pub start_time: f64,
    /// NHC panel open/close animation start time (0 = not animating).
    pub nhc_anim_start: f64,
    /// NHC panel animation progress at animation start (for easing).
    pub nhc_anim_from: f32,
    /// Wall-clock time of the last new-data arrival (for pulse animation).
    pub pulse_time: f64,
    /// Radar sweep line angle in degrees (optional visual flourish).
    pub sweep_angle: f32,
    /// Previous-frame hovered element IDs (for hover-glow styling).
    pub hovered_ids: Vec<ply_engine::prelude::Id>,

    // ── Double-click radar site selection ─────────────────────────
    /// Wall-clock time of the last single mouse click (for double-click).
    pub last_click_time: f64,
    /// Screen position of the last single mouse click.
    pub last_click_pos: (f32, f32),

    // ── Settings (Stage 7) ──────────────────────────────────────
    /// Current user settings (defaults until the persisted copy loads).
    pub settings: Settings,
    /// Pending settings load in flight (first launch: yields `None`).
    pub pending_settings_load: Option<oneshot::Receiver<Option<Settings>>>,
    /// Whether the settings panel modal is visible.
    pub show_settings_panel: bool,
}
