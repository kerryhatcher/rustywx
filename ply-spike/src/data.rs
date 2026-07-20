//! Background worker: polls the AWS archive bucket for the latest volume,
//! decodes it off the UI thread, and reports over an mpsc channel.
//! Adapted from rustywx/src/data.rs for Ply — no egui dependency.

use crate::model::ScanData;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use nexrad_data::aws::archive::{Identifier, download_file, list_files};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

/// How often to check for a new volume scan.
pub const POLL_INTERVAL: Duration = Duration::from_secs(120);

/// Messages the worker sends to the UI thread.
pub enum WorkerMessage {
    NewScan(Box<ScanData>),
    Status(String),
    Error(String),
}

/// Delay before the next poll given the number of consecutive failures:
/// normal interval when healthy, otherwise 30 s doubling, capped at 600 s.
fn retry_delay(consecutive_errors: u32) -> Duration {
    if consecutive_errors == 0 {
        POLL_INTERVAL
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

    let scan = file.scan().map_err(|e| anyhow!("decoding volume: {e}"))?;

    Ok(Some(ScanData::from_nexrad(&scan, timestamp)))
}

/// Spawn the background polling thread. It owns a current-thread tokio
/// runtime; all communication with the UI is via `tx`.
/// `site_rx` delivers site-change requests; `recv_timeout` is used so the
/// worker wakes immediately when the user selects a new radar.
pub fn spawn_worker(tx: Sender<WorkerMessage>, initial_site: String, site_rx: Receiver<String>) {
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

            delay = match runtime.block_on(fetch_latest_scan(&site, last_timestamp)) {
                Ok(Some(scan)) => {
                    consecutive_errors = 0;
                    last_timestamp = Some(scan.timestamp);
                    let _ = tx.send(WorkerMessage::NewScan(Box::new(scan)));
                    retry_delay(consecutive_errors)
                }
                Ok(None) => {
                    consecutive_errors = 0;
                    let _ = tx.send(WorkerMessage::Status("Up to date".to_string()));
                    retry_delay(consecutive_errors)
                }
                Err(e) => {
                    consecutive_errors += 1;
                    let d = retry_delay(consecutive_errors);
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
