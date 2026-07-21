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

use crate::model::{RadialData, ScanData, SweepData};
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

    // ── Save (RLE-compressed) ──────────────────────────────────────

    /// Key used for a site's RLE-compressed cached scan.
    ///
    /// Separate key namespace from [`Self::scan_key`] — compressed and
    /// plain caches can coexist without clobbering each other.
    fn scan_key_rle(site: &str) -> String {
        format!("scan_rle_{site}")
    }

    /// Flatten `scan` to bytes, RLE-compress ([`crate::rle::compress`]),
    /// and persist in the background.
    ///
    /// Fire-and-forget, same pattern as [`Self::save_scan`].
    pub fn save_scan_compressed(&self, site: &str, scan: &ScanData) {
        let storage = self.storage.clone();
        let key = Self::scan_key_rle(site);
        let compressed = crate::rle::compress(&scan_to_bytes(scan));
        tokio::spawn(async move {
            let _ = storage.save_bytes(&key, &compressed).await;
        });
    }

    // ── Load (RLE-compressed) ───────────────────────────────────────

    /// Spawn a background task to load and decompress the RLE-compressed
    /// scan for `site`. Mirrors [`Self::load_scan`]; yields `None` if no
    /// compressed entry exists or the stored bytes are corrupt.
    pub fn load_scan_compressed(&self, site: &str) -> oneshot::Receiver<Option<ScanData>> {
        let storage = self.storage.clone();
        let key = Self::scan_key_rle(site);
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            let result = storage
                .load_bytes(&key)
                .await
                .ok()
                .flatten()
                .and_then(|bytes| crate::rle::decompress(&bytes).ok())
                .and_then(|bytes| bytes_to_scan(&bytes).ok());
            let _ = tx.send(result);
        });

        rx
    }

    // ── Site preference ─────────────────────────────────────────

    /// Key used for the persisted last-selected site ID.
    const SITE_KEY: &str = "selected_site";

    /// Persist the selected site ID (fire-and-forget).
    pub fn save_site(&self, site: &str) {
        let storage = self.storage.clone();
        let key = Self::SITE_KEY.to_string();
        let value = site.as_bytes().to_vec();
        tokio::spawn(async move {
            let _ = storage.save_bytes(&key, &value).await;
        });
    }

    /// Spawn a background task to load the persisted site ID.
    ///
    /// Returns a `oneshot::Receiver` that yields `None` when no
    /// preference has been saved (first launch).
    pub fn load_site(&self) -> oneshot::Receiver<Option<String>> {
        let storage = self.storage.clone();
        let key = Self::SITE_KEY.to_string();
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            let result = storage
                .load_bytes(&key)
                .await
                .ok()
                .flatten()
                .and_then(|bytes| String::from_utf8(bytes).ok());
            let _ = tx.send(result);
        });
        rx
    }

    // ── Settings (Stage 7) ──────────────────────────────────────

    /// Key used for the persisted [`crate::settings::Settings`] JSON blob.
    const SETTINGS_KEY: &str = "settings.json";

    /// Serialize `settings` to JSON and persist in the background.
    ///
    /// Fire-and-forget, same pattern as [`Self::save_scan`]/[`Self::save_site`].
    pub fn save_settings(&self, settings: &crate::settings::Settings) {
        let storage = self.storage.clone();
        let key = Self::SETTINGS_KEY.to_string();
        match serde_json::to_vec(settings) {
            Ok(json) => {
                tokio::spawn(async move {
                    let _ = storage.save_bytes(&key, &json).await;
                });
            }
            Err(_) => {
                // Serialization failure — settings stay at their in-memory value.
            }
        }
    }

    /// Spawn a background task to load the persisted [`crate::settings::Settings`].
    ///
    /// Returns a `oneshot::Receiver` that yields `None` when no settings
    /// have been saved yet (first launch) or the stored JSON is corrupt.
    pub fn load_settings(&self) -> oneshot::Receiver<Option<crate::settings::Settings>> {
        let storage = self.storage.clone();
        let key = Self::SETTINGS_KEY.to_string();
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

    /// Get a reference to the underlying Ply storage handle.
    pub fn storage(&self) -> Storage {
        self.storage.clone()
    }
}

// ── ScanData <-> bytes ──────────────────────────────────────────────
//
// A hand-rolled flat binary layout, not `serde_json`. RLE only pays off
// when identical bytes run consecutively, and JSON's `"null"` tokens
// don't (each is 4 different bytes). Encoding `None` gates as a single
// `0x00` tag byte gives the RLE compressor the long runs it needs — real
// volumes are mostly below-threshold gates.

/// Flatten a `ScanData` into bytes suitable for [`crate::rle::compress`].
fn scan_to_bytes(scan: &ScanData) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&scan.timestamp.timestamp_millis().to_le_bytes());
    encode_sweeps(&mut buf, &scan.reflectivity);
    encode_sweeps(&mut buf, &scan.velocity);
    buf
}

fn encode_sweeps(buf: &mut Vec<u8>, sweeps: &[SweepData]) {
    buf.extend_from_slice(&(sweeps.len() as u32).to_le_bytes());
    for sweep in sweeps {
        buf.extend_from_slice(&sweep.elevation_deg.to_le_bytes());
        buf.extend_from_slice(&(sweep.radials.len() as u32).to_le_bytes());
        for radial in &sweep.radials {
            buf.extend_from_slice(&radial.azimuth_deg.to_le_bytes());
            buf.extend_from_slice(&(radial.gates.len() as u32).to_le_bytes());
            for gate in &radial.gates {
                match gate {
                    Some(v) => {
                        buf.push(1);
                        buf.extend_from_slice(&v.to_le_bytes());
                    }
                    None => buf.push(0),
                }
            }
        }
    }
}

/// Reconstruct a `ScanData` from bytes produced by [`scan_to_bytes`].
///
/// Returns an error string (not a panic) on truncated/corrupt input, so a
/// bad cache entry degrades to a cache miss rather than crashing the app.
fn bytes_to_scan(bytes: &[u8]) -> Result<ScanData, String> {
    let mut r = Reader::new(bytes);
    let timestamp_millis = r.read_i64()?;
    let timestamp = chrono::DateTime::from_timestamp_millis(timestamp_millis)
        .ok_or("cache: invalid timestamp in compressed scan")?;
    let reflectivity = decode_sweeps(&mut r)?;
    let velocity = decode_sweeps(&mut r)?;
    Ok(ScanData {
        timestamp,
        reflectivity,
        velocity,
    })
}

fn decode_sweeps(r: &mut Reader) -> Result<Vec<SweepData>, String> {
    let count = r.read_u32()?;
    (0..count)
        .map(|_| {
            let elevation_deg = r.read_f32()?;
            let radial_count = r.read_u32()?;
            let radials = (0..radial_count)
                .map(|_| {
                    let azimuth_deg = r.read_f32()?;
                    let gate_count = r.read_u32()?;
                    let gates = (0..gate_count)
                        .map(|_| match r.read_u8()? {
                            0 => Ok(None),
                            1 => Ok(Some(r.read_f32()?)),
                            other => Err(format!("cache: bad gate tag byte {other}")),
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                    Ok(RadialData {
                        azimuth_deg,
                        gates,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(SweepData {
                elevation_deg,
                radials,
            })
        })
        .collect()
}

/// Minimal bounds-checked byte cursor for [`bytes_to_scan`].
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Reader { data, pos: 0 }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let slice = self
            .data
            .get(self.pos..self.pos + n)
            .ok_or("cache: truncated compressed scan")?;
        self.pos += n;
        Ok(slice)
    }

    fn read_u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }

    fn read_u32(&mut self) -> Result<u32, String> {
        let bytes: [u8; 4] = self.take(4)?.try_into().unwrap();
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_i64(&mut self) -> Result<i64, String> {
        let bytes: [u8; 8] = self.take(8)?.try_into().unwrap();
        Ok(i64::from_le_bytes(bytes))
    }

    fn read_f32(&mut self) -> Result<f32, String> {
        let bytes: [u8; 4] = self.take(4)?.try_into().unwrap();
        Ok(f32::from_le_bytes(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::{bytes_to_scan, scan_to_bytes};
    use crate::model::{RadialData, ScanData, SweepData};
    use chrono::Utc;

    fn sample_scan() -> ScanData {
        ScanData {
            timestamp: Utc::now(),
            reflectivity: vec![SweepData {
                elevation_deg: 0.5,
                radials: vec![RadialData {
                    azimuth_deg: 12.0,
                    gates: vec![None, Some(32.0), None, None, Some(-5.5)],
                }],
            }],
            velocity: vec![],
        }
    }

    #[test]
    fn scan_bytes_round_trip() {
        let scan = sample_scan();
        let bytes = scan_to_bytes(&scan);
        let restored = bytes_to_scan(&bytes).unwrap();

        assert_eq!(
            restored.timestamp.timestamp_millis(),
            scan.timestamp.timestamp_millis()
        );
        assert_eq!(restored.reflectivity.len(), 1);
        assert_eq!(restored.reflectivity[0].elevation_deg, 0.5);
        assert_eq!(
            restored.reflectivity[0].radials[0].gates,
            vec![None, Some(32.0), None, None, Some(-5.5)]
        );
        assert!(restored.velocity.is_empty());
    }

    #[test]
    fn compressed_round_trip_through_rle() {
        let scan = sample_scan();
        let bytes = scan_to_bytes(&scan);
        let compressed = crate::rle::compress(&bytes);
        let decompressed = crate::rle::decompress(&compressed).unwrap();
        assert_eq!(decompressed, bytes);
        let restored = bytes_to_scan(&decompressed).unwrap();
        assert_eq!(
            restored.reflectivity[0].radials[0].gates,
            scan.reflectivity[0].radials[0].gates
        );
    }

    #[test]
    fn bytes_to_scan_rejects_truncated_input() {
        let bytes = scan_to_bytes(&sample_scan());
        let truncated = &bytes[..bytes.len() - 3];
        assert!(bytes_to_scan(truncated).is_err());
    }

    #[test]
    fn realistic_volume_compresses_by_at_least_90_percent() {
        // A representative volume: 16 sweeps, 360 radials each, 500 gates
        // per radial, ~1-in-250 gates carrying a real value (the rest below
        // threshold / range folded) — sparse like a real WSR-88D volume,
        // where most of a tilt's range is clear air.
        let sweeps: Vec<SweepData> = (0..16)
            .map(|s| SweepData {
                elevation_deg: s as f32 * 0.5,
                radials: (0..360)
                    .map(|az| RadialData {
                        azimuth_deg: az as f32,
                        gates: (0..500)
                            .map(|g| if g % 250 == 0 { Some(g as f32 * 0.5) } else { None })
                            .collect(),
                    })
                    .collect(),
            })
            .collect();
        let scan = ScanData {
            timestamp: Utc::now(),
            reflectivity: sweeps,
            velocity: vec![],
        };

        let raw = scan_to_bytes(&scan);
        let compressed = crate::rle::compress(&raw);

        assert!(
            compressed.len() * 10 < raw.len(),
            "expected >=90% savings: {} -> {} bytes",
            raw.len(),
            compressed.len()
        );

        let restored = bytes_to_scan(&crate::rle::decompress(&compressed).unwrap()).unwrap();
        assert_eq!(
            restored.reflectivity[0].radials[0].gates,
            scan.reflectivity[0].radials[0].gates
        );
    }
}
