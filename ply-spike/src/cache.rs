//! Disk cache for radar scans via Ply `storage`.
//!
//! Replaces the old egui-era `cache.rs` (filesystem JSON) and `store.rs`
//! (rusqlite).  Ply `storage` is a key-value API backed by platform-
//! appropriate paths (`~/.local/share` on Linux, OPFS on WASM).
//!
//! ## Async pattern (from Spike S8)
//!
//! - **Save:** fire-and-forget `tokio::spawn` — never blocks a frame.
//! - **Load:** `tokio::spawn` + `oneshot::channel` — the game loop polls
//!   `try_recv()` each frame while showing "Loading…" UI.
//! - **Metadata** (<2 ms): direct `.await` is acceptable, but we keep the
//!   channel pattern for consistency since scan payloads can be 100 KB+.

use crate::model::ScanData;
use ply_engine::prelude::Storage;
use tokio::sync::oneshot;

/// Wraps a Ply `Storage` handle for radar-scan persistence.
///
/// `Storage` is `Clone` — spawned tasks receive a cheap clone so they can
/// `.await` without borrowing the game-loop copy.
#[derive(Clone)]
pub struct Cache {
    storage: Storage,
}

impl Cache {
    /// Open (or create) the `"rustywx"` storage namespace.
    ///
    /// Must be called inside an async context (the game loop is `async fn`).
    pub async fn new() -> Result<Self, String> {
        let storage = Storage::new("rustywx").await?;
        Ok(Cache { storage })
    }

    /// Key used for a site's cached scan.
    fn scan_key(site: &str) -> String {
        format!("scan_{site}")
    }

    // ── Save ────────────────────────────────────────────────────────

    /// Serialize `scan` to JSON and persist in the background.
    ///
    /// Fire-and-forget — errors are silently ignored (the next poll will
    /// re-fetch from the network anyway).
    pub fn save_scan(&self, site: &str, scan: &ScanData) {
        let storage = self.storage.clone();
        let key = Self::scan_key(site);
        match serde_json::to_vec(scan) {
            Ok(json) => {
                tokio::spawn(async move {
                    let _ = storage.save_bytes(&key, &json).await;
                });
            }
            Err(_) => {
                // Serialization failure — will retry on next poll.
            }
        }
    }

    // ── Load ────────────────────────────────────────────────────────

    /// Spawn a background task to load the cached scan for `site`.
    ///
    /// Returns a `oneshot::Receiver` that the game loop polls each frame
    /// with `try_recv()`.  The receiver yields `None` when no cached data
    /// exists for this site.
    pub fn load_scan(&self, site: &str) -> oneshot::Receiver<Option<ScanData>> {
        let storage = self.storage.clone();
        let key = Self::scan_key(site);
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            let result = storage
                .load_bytes(&key)
                .await
                .ok()
                .flatten()
                .and_then(|bytes| serde_json::from_slice(&bytes).ok());
            let _ = tx.send(result);
        });

        rx
    }

    // ── Helpers ─────────────────────────────────────────────────────

    /// Remove the cached scan for `site` (e.g. after a corrupt read).
    #[allow(dead_code)]
    pub async fn remove_scan(&self, site: &str) -> Result<(), String> {
        self.storage.remove(&Self::scan_key(site)).await
    }
}
