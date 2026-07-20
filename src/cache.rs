//! Disk cache for radar scans, NWS alerts, and NHC storm data.
//! Lives at `~/.rustywx/cache/` alongside the SQLite DB and border cache.
//! On startup, data is loaded from cache first so the UI has something to
//! show immediately; background workers then refresh from the network.

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Where all cache files live, under a given home directory.
fn cache_dir_under(home: &Path) -> PathBuf {
    home.join(".rustywx").join("cache")
}

fn cache_dir() -> Result<PathBuf> {
    let home =
        std::env::var("HOME").map_err(|_| anyhow::anyhow!("HOME not set"))?;
    Ok(cache_dir_under(Path::new(&home)))
}

fn ensure_dir() -> Result<PathBuf> {
    let dir = cache_dir()?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

// ── Radar scan cache ─────────────────────────────────────────────────────

/// Path to the cached radar scan for a given site.
pub fn radar_cache_path(site: &str) -> Result<PathBuf> {
    Ok(ensure_dir()?.join(format!("radar_{site}.json")))
}

/// Load a cached radar scan for `site`, if it exists and is valid JSON.
pub fn load_radar(site: &str) -> Option<crate::model::ScanData> {
    let path = radar_cache_path(site).ok()?;
    let json = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&json).ok()
}

/// Save a radar scan to the cache for `site`.
pub fn save_radar(site: &str, scan: &crate::model::ScanData) {
    if let Ok(path) = radar_cache_path(site)
        && let Ok(json) = serde_json::to_string(scan)
    {
        let _ = std::fs::write(&path, json);
    }
}

// ── NWS alerts cache ─────────────────────────────────────────────────────

/// Path to the cached NWS alerts.
fn alerts_cache_path() -> Result<PathBuf> {
    Ok(ensure_dir()?.join("nws_alerts.json"))
}

/// Load cached NWS alerts.
pub fn load_alerts() -> Option<Vec<crate::alerts::Alert>> {
    let path = alerts_cache_path().ok()?;
    let json = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&json).ok()
}

/// Save NWS alerts to the cache.
pub fn save_alerts(alerts: &[crate::alerts::Alert]) {
    if let Ok(path) = alerts_cache_path()
        && let Ok(json) = serde_json::to_string(alerts)
    {
        let _ = std::fs::write(&path, json);
    }
}

// ── NHC storm data cache ─────────────────────────────────────────────────

/// Path to the cached NHC storm data.
fn nhc_cache_path() -> Result<PathBuf> {
    Ok(ensure_dir()?.join("nhc_storms.json"))
}

/// Load cached NHC storm data.
pub fn load_nhc() -> Option<crate::nhc::NhcBundle> {
    let path = nhc_cache_path().ok()?;
    let json = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&json).ok()
}

/// Save NHC storm data to the cache.
pub fn save_nhc(bundle: &crate::nhc::NhcBundle) {
    if let Ok(path) = nhc_cache_path()
        && let Ok(json) = serde_json::to_string(bundle)
    {
        let _ = std::fs::write(&path, json);
    }
}
