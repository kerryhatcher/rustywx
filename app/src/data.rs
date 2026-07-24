//! Background worker: polls the AWS archive bucket for the latest volume,
//! decodes it off the UI thread, and reports over an mpsc channel.
//! Adapted from rustywx/src/data.rs for Ply — no egui dependency.

use crate::demo::DemoRequest;
use crate::model::ScanData;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use nexrad_data::aws::archive::{Identifier, download_file, list_files};
use nexrad_data::volume::File as VolumeFile;
use nexrad_decode::messages::{Message, MessageContents};
use ply_engine::prelude::Storage;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

/// How often to check for a new volume scan.
pub const POLL_INTERVAL: Duration = Duration::from_secs(120);

/// Messages the worker sends to the UI thread.
pub enum WorkerMessage {
    NewScan { site: String, scan: Box<ScanData> },
    Status(String),
    Error(String),
}

/// Delay before the next poll given the number of consecutive failures:
/// `healthy_interval` when healthy, otherwise 30 s doubling, capped at 600 s.
fn retry_delay(consecutive_errors: u32, healthy_interval: Duration) -> Duration {
    if consecutive_errors == 0 {
        healthy_interval
    } else {
        let secs = 30u64.saturating_mul(2u64.saturating_pow(consecutive_errors - 1));
        Duration::from_secs(secs.min(600))
    }
}

/// Discard `_MDM` metadata objects; only real volume files remain.
fn volume_files(identifiers: Vec<Identifier>) -> Vec<Identifier> {
    identifiers
        .into_iter()
        .filter(|id| !id.name().ends_with("_MDM"))
        .collect()
}

/// Fetch and decode the most recent volume scan for `site`, unless it's the
/// same volume the caller already has.
pub async fn fetch_latest_scan(
    site: &str,
    last_seen: Option<DateTime<Utc>>,
) -> Result<Option<ScanData>> {
    let today = Utc::now().date_naive();
    let mut files = volume_files(
        list_files(site, &today)
            .await
            .map_err(|e| anyhow!("listing volumes for {site} {today}: {e}"))?,
    );

    if files.is_empty() {
        let yesterday = today - ChronoDuration::days(1);
        files = volume_files(
            list_files(site, &yesterday)
                .await
                .map_err(|e| anyhow!("listing volumes for {site} {yesterday}: {e}"))?,
        );
    }

    let identifier = files
        .into_iter()
        .max()
        .ok_or_else(|| anyhow!("no volume files found for {site}"))?;

    let timestamp = identifier
        .date_time()
        .ok_or_else(|| anyhow!("unparseable volume name: {}", identifier.name()))?;

    if last_seen == Some(timestamp) {
        return Ok(None);
    }

    let file = download_file(identifier)
        .await
        .map_err(|e| anyhow!("downloading volume: {e}"))?;

    Ok(Some(scan_from_volume(&file, timestamp)?))
}

/// Decode a volume file into `ScanData` — the shared tail of the live and
/// demo fetch paths, so the two cannot drift.
fn scan_from_volume(file: &VolumeFile, timestamp: DateTime<Utc>) -> Result<ScanData> {
    let scan = file.scan().map_err(|e| anyhow!("decoding volume: {e}"))?;
    let nyquist_by_elev = nyquist_by_elevation(file);

    let mut scan_data = ScanData::from_nexrad(&scan, timestamp, &nyquist_by_elev);
    // KDP isn't a decoded moment — derive it from the ΦDP sweeps just
    // decoded above. Empty ΦDP (legacy scan) derives to an empty KDP, same
    // convention as the other dual-pol products.
    scan_data.specific_differential_phase =
        crate::kdp::derive_kdp_sweeps(&scan_data.differential_phase);

    Ok(scan_data)
}

/// Ply storage key for a cached raw demo volume (bytes exactly as downloaded,
/// which may be gzip-wrapped for pre-2016 archives).
fn demo_cache_key(volume_name: &str) -> String {
    format!("demo_volume_{volume_name}")
}

/// Fetch and decode a demo volume: a curated archive event (downloaded once,
/// then served from the raw-bytes cache) or a local Level II file. Decodes
/// through the same `scan_from_volume` as the live path.
pub async fn fetch_demo_scan(req: &DemoRequest, storage: Option<&Storage>) -> Result<ScanData> {
    let volume_name = req.volume_name();
    let timestamp = Identifier::new(volume_name.clone())
        .date_time()
        .ok_or_else(|| anyhow!("no timestamp in demo volume name {volume_name:?}"))?;

    let (bytes, from_cache) = match req {
        DemoRequest::File(path) => (
            std::fs::read(path).map_err(|e| anyhow!("reading demo volume {path:?}: {e}"))?,
            false,
        ),
        DemoRequest::Event(ev) => {
            let key = demo_cache_key(ev.volume_name);
            let cached = match storage {
                Some(s) => s.load_bytes(&key).await.ok().flatten(),
                None => None,
            };
            match cached {
                Some(bytes) => (bytes, true),
                None => {
                    let file = download_file(Identifier::new(ev.volume_name.to_string()))
                        .await
                        .map_err(|e| anyhow!("downloading demo volume {}: {e}", ev.volume_name))?;
                    let bytes = file.data().to_vec();
                    if let Some(s) = storage {
                        let _ = s.save_bytes(&key, &bytes).await;
                    }
                    (bytes, false)
                }
            }
        }
    };

    // Pre-2016 archive volumes are gzip-wrapped; decompress() is a no-op for
    // modern files.
    let decoded = VolumeFile::new(bytes)
        .decompress()
        .map_err(|e| anyhow!("decompressing demo volume {volume_name}: {e}"))
        .and_then(|file| scan_from_volume(&file, timestamp));

    match decoded {
        Ok(scan) => Ok(scan),
        Err(e) => {
            // Self-heal a corrupt cache entry — whether it failed gzip
            // inflation or volume decode — so the next run re-downloads.
            if from_cache && let (Some(s), DemoRequest::Event(ev)) = (storage, req) {
                let _ = s.remove(&demo_cache_key(ev.volume_name)).await;
            }
            Err(e)
        }
    }
}

/// Nyquist velocity (m/s), by elevation number, recovered from Message 31
/// Radial Data (Constant) Blocks in the volume.
///
/// `file.scan()` drops this converting to `nexrad_model::Radial` — see
/// `docs/velocity-dealiasing-plan.md`. This is a second, cheap decode pass
/// over the already-in-memory volume bytes; negligible next to the network
/// fetch. Decode failures on individual records/messages are skipped — an
/// absent map entry means "unknown" downstream (readout shows "—", dealias
/// is a no-op), never a guess.
fn nyquist_by_elevation(file: &VolumeFile) -> HashMap<u8, f32> {
    let mut nyquist = HashMap::new();
    let Ok(records) = file.records() else {
        return nyquist;
    };
    for record in records {
        let record = if record.compressed() {
            match record.decompress() {
                Ok(r) => r,
                Err(_) => continue,
            }
        } else {
            record
        };
        let Ok(messages) = record.messages() else {
            continue;
        };
        collect_nyquist(&messages, &mut nyquist);
    }
    nyquist
}

/// Fold each Message 31's Radial Data Block Nyquist into `nyquist`, keyed by
/// elevation number. Nyquist is constant within a Doppler cut, so the first
/// value seen for an elevation number wins; a raw value of 0 means the block
/// was absent/unset and is skipped, not recorded as a real zero.
fn collect_nyquist(messages: &[Message<'_>], nyquist: &mut HashMap<u8, f32>) {
    for message in messages {
        let MessageContents::DigitalRadarData(m) = message.contents() else {
            continue;
        };
        let Some(block) = m.radial_data_block() else {
            continue;
        };
        let value = block.nyquist_velocity_raw() as f32 * 0.01;
        if value > 0.0 {
            nyquist
                .entry(m.header().elevation_number())
                .or_insert(value);
        }
    }
}

/// Spawn the background polling thread. It owns a current-thread tokio
/// runtime; all communication with the UI is via `tx`.
/// `site_rx` delivers site-change requests; `recv_timeout` is used so the
/// worker wakes immediately when the user selects a new radar.
/// `poll_interval` is the healthy-state delay between checks, in seconds —
/// a shared atomic the UI updates from `Settings.poll_interval_secs`, so a
/// changed setting takes effect on the next cycle (see `main.rs`).
pub fn spawn_worker(
    tx: Sender<WorkerMessage>,
    initial_site: String,
    site_rx: Receiver<String>,
    poll_interval: Arc<AtomicU64>,
) {
    std::thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                let _ = tx.send(WorkerMessage::Error(format!("tokio runtime: {e}")));
                return;
            }
        };

        let mut current_site = initial_site;
        let mut last_timestamp = None;
        let mut consecutive_errors: u32 = 0;
        // Start with a zero delay so the first fetch happens immediately.
        let mut delay = Duration::ZERO;

        loop {
            // Wait for either a site-change request or the poll delay.
            match site_rx.recv_timeout(delay) {
                Ok(new_site) => {
                    current_site = new_site;
                    last_timestamp = None;
                    consecutive_errors = 0;
                    delay = Duration::ZERO; // fetch immediately
                    continue;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Timeout elapsed — proceed with the poll.
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    return; // UI dropped the sender; shut down.
                }
            }

            let site = current_site.clone();
            let _ = tx.send(WorkerMessage::Status(format!(
                "Checking {site} for new data…"
            )));

            // Read the live healthy interval (floored at 1s) each cycle.
            let healthy = Duration::from_secs(poll_interval.load(Ordering::Relaxed).max(1));
            delay = match runtime.block_on(fetch_latest_scan(&site, last_timestamp)) {
                Ok(Some(scan)) => {
                    consecutive_errors = 0;
                    last_timestamp = Some(scan.timestamp);
                    let _ = tx.send(WorkerMessage::NewScan {
                        site: site.clone(),
                        scan: Box::new(scan),
                    });
                    retry_delay(consecutive_errors, healthy)
                }
                Ok(None) => {
                    consecutive_errors = 0;
                    let _ = tx.send(WorkerMessage::Status("Up to date".to_string()));
                    retry_delay(consecutive_errors, healthy)
                }
                Err(e) => {
                    consecutive_errors += 1;
                    let d = retry_delay(consecutive_errors, healthy);
                    let _ = tx.send(WorkerMessage::Error(format!(
                        "{e:#} — retrying in {}s",
                        d.as_secs()
                    )));
                    d
                }
            };
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_demo_scan_missing_file_errors() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let req = crate::demo::DemoRequest::File("/nonexistent/KTLX20130520_201643_V06".into());
        // `ScanData` isn't `Debug`, so `unwrap_err()` (which requires `T:
        // Debug`) doesn't compile here — match instead.
        let err = match rt.block_on(fetch_demo_scan(&req, None)) {
            Err(e) => e,
            Ok(_) => panic!("expected an error for a missing demo file"),
        };
        assert!(
            err.to_string().contains("reading demo volume"),
            "got: {err:#}"
        );
    }

    #[test]
    fn fetch_demo_scan_bad_filename_has_no_timestamp() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let path = std::env::temp_dir().join("not-a-volume.bin");
        std::fs::write(&path, b"junk").unwrap();
        let req = crate::demo::DemoRequest::File(path.clone());
        let err = match rt.block_on(fetch_demo_scan(&req, None)) {
            Err(e) => e,
            Ok(_) => panic!("expected an error for an unparseable volume name"),
        };
        assert!(err.to_string().contains("timestamp"), "got: {err:#}");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn retry_delay_uses_healthy_interval_when_no_errors() {
        let custom = Duration::from_secs(45);
        assert_eq!(retry_delay(0, custom), custom);
        assert_eq!(retry_delay(0, POLL_INTERVAL), POLL_INTERVAL);
    }

    #[test]
    fn retry_delay_backs_off_regardless_of_healthy_interval() {
        let custom = Duration::from_secs(45);
        assert_eq!(retry_delay(1, custom), Duration::from_secs(30));
        assert_eq!(retry_delay(2, custom), Duration::from_secs(60));
        // Capped at 600s even after many consecutive errors.
        assert_eq!(retry_delay(10, custom), Duration::from_secs(600));
    }

    /// A single real Message 31 (Digital Radar Data) frame, borrowed from
    /// `nexrad-decode`'s own test fixtures (`tests/data/messages/`,
    /// MIT-licensed) so this test exercises the real Radial Data Block
    /// layout rather than hand-rolled bytes.
    const DIGITAL_RADAR_DATA_FULL: &[u8] = include_bytes!("testdata/digital_radar_data_full.bin");

    #[test]
    fn collect_nyquist_reads_real_radial_data_block() {
        let messages = nexrad_decode::messages::decode_messages(DIGITAL_RADAR_DATA_FULL).unwrap();
        let mut nyquist = HashMap::new();
        collect_nyquist(&messages, &mut nyquist);

        assert_eq!(nyquist.len(), 1, "fixture carries exactly one radial");
        let (&elevation_number, &value) = nyquist.iter().next().unwrap();
        assert_eq!(elevation_number, 1);
        assert!(
            (value - 8.85).abs() < 0.01,
            "expected ~8.85 m/s, got {value}"
        );
    }

    #[test]
    fn collect_nyquist_skips_messages_without_radial_data_block() {
        // Empty input decodes to zero messages — nothing to fold, no panic.
        let messages = nexrad_decode::messages::decode_messages(&[0u8; 0]).unwrap_or_default();
        let mut nyquist = HashMap::new();
        collect_nyquist(&messages, &mut nyquist);
        assert!(nyquist.is_empty());
    }

    #[test]
    fn collect_nyquist_first_value_wins_per_elevation() {
        // Folding the same elevation number's radial into an existing map
        // entry a second time (simulating a later radial in the same cut)
        // must not disturb the value recorded by the first.
        let messages = nexrad_decode::messages::decode_messages(DIGITAL_RADAR_DATA_FULL).unwrap();
        let mut nyquist = HashMap::new();
        collect_nyquist(&messages, &mut nyquist);
        let first_value = *nyquist.values().next().unwrap();

        collect_nyquist(&messages, &mut nyquist);
        assert_eq!(nyquist.len(), 1);
        assert_eq!(*nyquist.values().next().unwrap(), first_value);
    }

    #[test]
    fn nyquist_by_elevation_returns_empty_map_on_bad_file() {
        // A gzip-compressed file can't be decoded without a `decompress()`
        // pass first (`fetch_latest_scan` doesn't do that here); `records()`
        // fails and the function degrades to "unknown" rather than
        // propagating an error or panicking.
        let file = VolumeFile::new(vec![0x1f, 0x8b, 0, 0]);
        assert!(file.compressed());
        assert!(nyquist_by_elevation(&file).is_empty());
    }
}
