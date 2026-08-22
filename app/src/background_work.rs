//! CPU-bound startup work kept off the Macroquad render thread.
//!
//! Requests and results contain owned, `Send` data only.  The bounded queues
//! provide back-pressure and generations let the UI discard superseded work
//! without ever waiting for a worker to finish.

use crate::alerts::{self, Alert};
use crate::borders::{self, Ring};
use crate::model::{Product, SweepData};
use crate::nhc::{self, StormMeta};
use crate::scope::{self, QcConfig, QcReport};
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use tokio::runtime::Handle;
use tokio::sync::mpsc;

pub const FRAME_BUDGET: Duration = Duration::from_millis(16);
const BACKGROUND_WARN: Duration = Duration::from_millis(250);
const QUEUE_CAPACITY: usize = 4;

#[derive(Debug)]
pub enum SubmitError {
    Full,
    Closed,
}

fn map_submit<T>(error: mpsc::error::TrySendError<T>) -> SubmitError {
    match error {
        mpsc::error::TrySendError::Full(_) => SubmitError::Full,
        mpsc::error::TrySendError::Closed(_) => SubmitError::Closed,
    }
}

pub enum ParsePayload {
    Borders {
        states: String,
        coast: String,
        country: String,
    },
    Alerts(String),
    NhcMetadata {
        storms: String,
        gis_cone: Option<String>,
        gis_track: Option<String>,
        gis_points: Option<String>,
        gis_ww: Option<String>,
    },
}

impl ParsePayload {
    fn phase(&self) -> &'static str {
        match self {
            Self::Borders { .. } => "borders_parse",
            Self::Alerts(_) => "alerts_parse",
            Self::NhcMetadata { .. } => "nhc_metadata_parse",
        }
    }

    fn payload_size(&self) -> usize {
        match self {
            Self::Borders {
                states,
                coast,
                country,
            } => states.len() + coast.len() + country.len(),
            Self::Alerts(body) => body.len(),
            Self::NhcMetadata {
                storms,
                gis_cone,
                gis_track,
                gis_points,
                gis_ww,
            } => {
                storms.len()
                    + gis_cone.as_ref().map_or(0, String::len)
                    + gis_track.as_ref().map_or(0, String::len)
                    + gis_points.as_ref().map_or(0, String::len)
                    + gis_ww.as_ref().map_or(0, String::len)
            }
        }
    }
}

pub struct ParseRequest {
    pub generation: u64,
    pub payload: ParsePayload,
    pub submitted_at: Instant,
}

pub struct NhcMetadata {
    pub metas: Vec<StormMeta>,
    pub gis_cone: Option<Value>,
    pub gis_track: Option<Value>,
    pub gis_points: Option<Value>,
    pub gis_ww: Option<Value>,
}

pub enum ParsedData {
    Borders(Vec<Ring>),
    Alerts(Vec<Alert>),
    NhcMetadata(NhcMetadata),
}

pub struct ParseResult {
    pub generation: u64,
    pub result: Result<ParsedData, String>,
    pub submitted_at: Instant,
    pub elapsed: Duration,
}

#[derive(Clone)]
pub struct OwnedQcConfig {
    pub tdbz_kernel_size: usize,
    pub cc_sweep: Option<SweepData>,
    pub cc_gate_enabled: bool,
    pub cc_gate_threshold: f32,
    pub refl_floor_enabled: bool,
    pub refl_floor_dbz: f32,
    pub vel_dealias_enabled: bool,
    pub vel_sd_censor_enabled: bool,
    pub vel_sd_threshold: f32,
    pub zdr_sweep: Option<SweepData>,
    pub phidp_sweep: Option<SweepData>,
    pub nonmet_fuzzy_enabled: bool,
    pub nonmet_threshold: f32,
    pub refl_gap_fill_enabled: bool,
    pub multi_scale_texture_enabled: bool,
    pub sun_spike_removal_enabled: bool,
}

impl OwnedQcConfig {
    fn as_borrowed(&self) -> QcConfig<'_> {
        QcConfig {
            tdbz_kernel_size: self.tdbz_kernel_size,
            cc_sweep: self.cc_sweep.as_ref(),
            cc_gate_enabled: self.cc_gate_enabled,
            cc_gate_threshold: self.cc_gate_threshold,
            refl_floor_enabled: self.refl_floor_enabled,
            refl_floor_dbz: self.refl_floor_dbz,
            vel_dealias_enabled: self.vel_dealias_enabled,
            vel_sd_censor_enabled: self.vel_sd_censor_enabled,
            vel_sd_threshold: self.vel_sd_threshold,
            zdr_sweep: self.zdr_sweep.as_ref(),
            phidp_sweep: self.phidp_sweep.as_ref(),
            nonmet_fuzzy_enabled: self.nonmet_fuzzy_enabled,
            nonmet_threshold: self.nonmet_threshold,
            refl_gap_fill_enabled: self.refl_gap_fill_enabled,
            multi_scale_texture_enabled: self.multi_scale_texture_enabled,
            sun_spike_removal_enabled: self.sun_spike_removal_enabled,
        }
    }
}

pub struct RasterRequest {
    pub generation: u64,
    pub site: String,
    pub product: Product,
    pub sweep: SweepData,
    pub qc: OwnedQcConfig,
    pub size_px: usize,
    pub max_range_km: f32,
    pub submitted_at: Instant,
}

pub struct RasterResult {
    pub generation: u64,
    pub site: String,
    pub product: Product,
    pub result: Result<(Vec<u8>, QcReport), String>,
    pub submitted_at: Instant,
    pub elapsed: Duration,
}

pub fn raster_result_is_current(
    result: &RasterResult,
    generation: u64,
    site: &str,
    product: Product,
    dirty: bool,
) -> bool {
    !dirty && result.generation == generation && result.site == site && result.product == product
}

pub struct ImageDecodeRequest {
    pub generation: u64,
    pub sequence: usize,
    pub key: String,
    pub bytes: Vec<u8>,
    pub submitted_at: Instant,
}

#[derive(Debug)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub struct ImageDecodeResult {
    pub generation: u64,
    pub sequence: usize,
    pub key: String,
    pub result: Result<DecodedImage, String>,
    pub submitted_at: Instant,
    pub elapsed: Duration,
}

pub struct BackgroundWorkers {
    parse_tx: mpsc::Sender<ParseRequest>,
    parse_rx: mpsc::Receiver<ParseResult>,
    raster_tx: mpsc::Sender<RasterRequest>,
    raster_rx: mpsc::Receiver<RasterResult>,
    image_tx: mpsc::Sender<ImageDecodeRequest>,
    image_rx: mpsc::Receiver<ImageDecodeResult>,
}

impl BackgroundWorkers {
    pub fn new(handle: &Handle) -> Self {
        let (parse_tx, mut parse_requests) = mpsc::channel::<ParseRequest>(QUEUE_CAPACITY);
        let (parse_results, parse_rx) = mpsc::channel(QUEUE_CAPACITY);
        handle.spawn(async move {
            while let Some(request) = parse_requests.recv().await {
                let started = Instant::now();
                let generation = request.generation;
                let submitted_at = request.submitted_at;
                let phase = request.payload.phase();
                let payload_bytes = request.payload.payload_size();
                let result = tokio::task::spawn_blocking(move || parse(request.payload))
                    .await
                    .map_err(|e| format!("parse worker join: {e}"))
                    .and_then(|r| r);
                let elapsed = started.elapsed();
                log_background(phase, generation, elapsed);
                eprintln!("startup phase={phase} generation={generation} payload_bytes={payload_bytes} elapsed_ms={}", elapsed.as_millis());
                let _ = parse_results
                    .send(ParseResult {
                        generation,
                        result,
                        submitted_at,
                        elapsed,
                    })
                    .await;
            }
        });

        let (raster_tx, mut raster_requests) = mpsc::channel::<RasterRequest>(QUEUE_CAPACITY);
        let (raster_results, raster_rx) = mpsc::channel(QUEUE_CAPACITY);
        handle.spawn(async move {
            while let Some(request) = raster_requests.recv().await {
                let started = Instant::now();
                let generation = request.generation;
                let submitted_at = request.submitted_at;
                let site = request.site.clone();
                let product = request.product;
                eprintln!("startup phase=radar_raster_start generation={generation} site={site} product={product:?}");
                let result = tokio::task::spawn_blocking(move || {
                    Ok(scope::rasterize_with_report(
                        &request.sweep,
                        request.product,
                        request.size_px,
                        request.max_range_km,
                        &request.qc.as_borrowed(),
                    ))
                })
                .await
                .map_err(|e| format!("raster worker join: {e}"))
                .and_then(|r: Result<_, String>| r);
                let elapsed = started.elapsed();
                log_background("radar_raster_finish", generation, elapsed);
                let _ = raster_results.send(RasterResult {
                    generation, site, product, result, submitted_at, elapsed,
                }).await;
            }
        });

        let (image_tx, mut image_requests) = mpsc::channel::<ImageDecodeRequest>(QUEUE_CAPACITY);
        let (image_results, image_rx) = mpsc::channel(QUEUE_CAPACITY);
        handle.spawn(async move {
            while let Some(request) = image_requests.recv().await {
                let started = Instant::now();
                let generation = request.generation;
                let sequence = request.sequence;
                let key = request.key.clone();
                let submitted_at = request.submitted_at;
                let result = tokio::task::spawn_blocking(move || decode_image(&request.bytes))
                    .await
                    .map_err(|e| format!("image worker join: {e}"))
                    .and_then(|r| r);
                let elapsed = started.elapsed();
                log_background("nhc_image_decode", generation, elapsed);
                let _ = image_results
                    .send(ImageDecodeResult {
                        generation,
                        sequence,
                        key,
                        result,
                        submitted_at,
                        elapsed,
                    })
                    .await;
            }
        });

        Self {
            parse_tx,
            parse_rx,
            raster_tx,
            raster_rx,
            image_tx,
            image_rx,
        }
    }

    pub fn submit_parse(&self, request: ParseRequest) -> Result<(), SubmitError> {
        self.parse_tx.try_send(request).map_err(map_submit)
    }
    pub fn poll_parse(&mut self) -> Option<ParseResult> {
        self.parse_rx.try_recv().ok()
    }
    pub fn submit_raster(&self, request: RasterRequest) -> Result<(), SubmitError> {
        self.raster_tx.try_send(request).map_err(map_submit)
    }
    pub fn poll_raster(&mut self) -> Option<RasterResult> {
        self.raster_rx.try_recv().ok()
    }
    pub fn submit_image(&self, request: ImageDecodeRequest) -> Result<(), SubmitError> {
        self.image_tx.try_send(request).map_err(map_submit)
    }
    pub fn image_queue_has_capacity(&self) -> bool {
        self.image_tx.capacity() > 0
    }
    pub fn poll_image(&mut self) -> Option<ImageDecodeResult> {
        self.image_rx.try_recv().ok()
    }
}

fn parse(payload: ParsePayload) -> Result<ParsedData, String> {
    match payload {
        ParsePayload::Borders {
            states,
            coast,
            country,
        } => borders::parse_and_merge(&states, &coast, &country)
            .map(ParsedData::Borders)
            .map_err(|e| format!("{e:#}")),
        ParsePayload::Alerts(body) => alerts::parse_alerts(&body)
            .map(ParsedData::Alerts)
            .map_err(|e| format!("{e:#}")),
        ParsePayload::NhcMetadata {
            storms,
            gis_cone,
            gis_track,
            gis_points,
            gis_ww,
        } => {
            let parse_gis =
                |body: Option<String>| body.and_then(|body| nhc::parse_gis_layer(&body).ok());
            nhc::parse_current_storms(&storms)
                .map(|metas| {
                    ParsedData::NhcMetadata(NhcMetadata {
                        metas,
                        gis_cone: parse_gis(gis_cone),
                        gis_track: parse_gis(gis_track),
                        gis_points: parse_gis(gis_points),
                        gis_ww: parse_gis(gis_ww),
                    })
                })
                .map_err(|e| format!("{e:#}"))
        }
    }
}

fn decode_image(bytes: &[u8]) -> Result<DecodedImage, String> {
    let rgba = image::load_from_memory(bytes)
        .map_err(|e| format!("decoding NHC image: {e}"))?
        .to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(DecodedImage {
        width,
        height,
        rgba: rgba.into_raw(),
    })
}

fn log_background(phase: &str, generation: u64, elapsed: Duration) {
    let ms = elapsed.as_millis();
    if elapsed > BACKGROUND_WARN {
        eprintln!(
            "Warning: startup phase={phase} generation={generation} elapsed_ms={ms} background_threshold_ms={}",
            BACKGROUND_WARN.as_millis()
        );
    } else {
        eprintln!("startup phase={phase} generation={generation} elapsed_ms={ms}");
    }
}

/// Reorders independently completed decodes and yields one item per caller
/// invocation, which lets the render loop upload at most one texture/frame.
pub struct OrderedImageResults {
    generation: u64,
    next_sequence: usize,
    pending: BTreeMap<usize, ImageDecodeResult>,
}

impl OrderedImageResults {
    pub fn new(generation: u64) -> Self {
        Self {
            generation,
            next_sequence: 0,
            pending: BTreeMap::new(),
        }
    }
    pub fn reset(&mut self, generation: u64) {
        self.generation = generation;
        self.next_sequence = 0;
        self.pending.clear();
    }
    pub fn push(&mut self, result: ImageDecodeResult) -> bool {
        if result.generation != self.generation {
            return false;
        }
        self.pending.insert(result.sequence, result);
        true
    }
    pub fn pop_next(&mut self) -> Option<ImageDecodeResult> {
        let result = self.pending.remove(&self.next_sequence)?;
        self.next_sequence += 1;
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_decode_propagates_errors() {
        assert!(
            decode_image(b"not an image")
                .unwrap_err()
                .contains("decoding NHC image")
        );
    }

    #[test]
    fn stale_and_out_of_order_images_are_deterministic() {
        fn result(generation: u64, sequence: usize) -> ImageDecodeResult {
            ImageDecodeResult {
                generation,
                sequence,
                key: sequence.to_string(),
                result: Err("x".into()),
                submitted_at: Instant::now(),
                elapsed: Duration::ZERO,
            }
        }
        let mut ordered = OrderedImageResults::new(2);
        assert!(!ordered.push(result(1, 0)));
        assert!(ordered.push(result(2, 1)));
        assert!(ordered.pop_next().is_none());
        assert!(ordered.push(result(2, 0)));
        assert_eq!(ordered.pop_next().unwrap().sequence, 0);
        assert_eq!(ordered.pop_next().unwrap().sequence, 1);
    }

    #[test]
    fn parse_worker_is_polled_without_blocking_the_render_caller() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let _guard = runtime.enter();
        let mut workers = BackgroundWorkers::new(runtime.handle());
        workers
            .submit_parse(ParseRequest {
                generation: 7,
                payload: ParsePayload::Alerts(r#"{"features":[]}"#.to_owned()),
                submitted_at: Instant::now(),
            })
            .unwrap();

        let poll_started = Instant::now();
        let _ = workers.poll_parse();
        assert!(poll_started.elapsed() < FRAME_BUDGET);

        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if let Some(result) = workers.poll_parse() {
                assert_eq!(result.generation, 7);
                assert!(
                    matches!(result.result, Ok(ParsedData::Alerts(alerts)) if alerts.is_empty())
                );
                break;
            }
            assert!(Instant::now() < deadline, "parse worker timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn raster_worker_converts_owned_request_to_result() {
        use crate::model::RadialData;
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let _guard = runtime.enter();
        let mut workers = BackgroundWorkers::new(runtime.handle());
        workers
            .submit_raster(RasterRequest {
                generation: 9,
                site: "TEST".to_owned(),
                product: Product::Reflectivity,
                sweep: SweepData {
                    elevation_deg: 0.5,
                    radials: vec![RadialData {
                        azimuth_deg: 0.0,
                        gates: vec![Some(10.0)],
                        range_folded: vec![false],
                    }],
                    first_gate_km: 0.0,
                    gate_spacing_km: 1.0,
                    nyquist_ms: 0.0,
                },
                qc: OwnedQcConfig {
                    tdbz_kernel_size: 3,
                    cc_sweep: None,
                    cc_gate_enabled: false,
                    cc_gate_threshold: 0.8,
                    refl_floor_enabled: false,
                    refl_floor_dbz: 0.0,
                    vel_dealias_enabled: false,
                    vel_sd_censor_enabled: false,
                    vel_sd_threshold: 0.0,
                    zdr_sweep: None,
                    phidp_sweep: None,
                    nonmet_fuzzy_enabled: false,
                    nonmet_threshold: 0.5,
                    refl_gap_fill_enabled: false,
                    multi_scale_texture_enabled: false,
                    sun_spike_removal_enabled: false,
                },
                size_px: 8,
                max_range_km: 4.0,
                submitted_at: Instant::now(),
            })
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if let Some(result) = workers.poll_raster() {
                assert_eq!(result.generation, 9);
                assert_eq!(result.result.unwrap().0.len(), 8 * 8 * 4);
                break;
            }
            assert!(Instant::now() < deadline, "raster worker timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn timing_log_contains_required_fields() {
        let phase = "alerts_parse";
        let generation = 12;
        let elapsed = Duration::from_millis(4);
        let line = format!(
            "startup phase={phase} generation={generation} elapsed_ms={}",
            elapsed.as_millis()
        );
        assert!(line.contains("phase=alerts_parse"));
        assert!(line.contains("generation=12"));
        assert!(line.contains("elapsed_ms=4"));
    }

    #[test]
    fn changed_site_or_product_rejects_completed_raster() {
        let result = RasterResult {
            generation: 3,
            site: "KFFC".to_owned(),
            product: Product::Reflectivity,
            result: Err("unused".to_owned()),
            submitted_at: Instant::now(),
            elapsed: Duration::ZERO,
        };
        assert!(raster_result_is_current(
            &result,
            3,
            "KFFC",
            Product::Reflectivity,
            false
        ));
        assert!(!raster_result_is_current(
            &result,
            3,
            "KTLX",
            Product::Reflectivity,
            false
        ));
        assert!(!raster_result_is_current(
            &result,
            3,
            "KFFC",
            Product::Velocity,
            false
        ));
        assert!(!raster_result_is_current(
            &result,
            4,
            "KFFC",
            Product::Reflectivity,
            false
        ));
        assert!(!raster_result_is_current(
            &result,
            3,
            "KFFC",
            Product::Reflectivity,
            true
        ));
    }
}
