//! Application state — replaces the egui-era `app.rs` struct.
//!
//! Held across frames in `main.rs`; mutated by input handling and worker
//! messages, read by the rendering closure.

use crate::cache::Cache;
use crate::data::WorkerMessage;
use crate::model::{Product, ScanData};
use crate::widgets::dropdown::DropdownState;
use ply_engine::prelude::Texture2D;
use std::sync::mpsc;
use tokio::sync::oneshot;

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
}
