# State Boundary Overlay Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Draw US state border lines (Georgia, Alabama, South Carolina, Florida) on the radarscope, sourced from the Census Bureau's TIGERweb REST API and cached locally at `~/.rustywx/state_borders.geojson`.

**Architecture:** A new one-shot background thread (`src/borders.rs`) checks for a local GeoJSON cache file, fetching it from TIGERweb on first run if absent, and reports parsed polygon rings to the UI thread over an `mpsc` channel — mirroring the existing NEXRAD worker's message-passing pattern but without a polling loop. `scope::draw_scope` projects each ring's vertices with the existing `geo::range_bearing`/`geo::polar_to_offset` functions and draws them as line segments, clipped to the 230 km display radius.

**Tech Stack:** Rust, `ureq` (blocking HTTPS GET), `serde_json` (GeoJSON parsing via `Value`), existing `egui`/`eframe`/`anyhow` stack.

## Global Constraints

- Rust edition 2024 (existing project requirement).
- New dependencies: `ureq = "3"` (default features — rustls + gzip, no extra flags needed) and `serde_json = "1"`. No other new dependencies (no `geojson` crate, no `dirs` crate, no `tempfile` crate).
- TIGERweb query URL (verified live against the real service — returns 4 features: SC, GA, AL, FL; FL is `MultiPolygon`, the others are `Polygon`):
  `https://tigerweb.geo.census.gov/arcgis/rest/services/TIGERweb/State_County/MapServer/0/query?where=STUSAB+IN+(%27GA%27%2C%27AL%27%2C%27SC%27%2C%27FL%27)&outFields=STUSAB,NAME&f=geojson`
- Cache file path: `$HOME/.rustywx/state_borders.geojson` (via the `HOME` env var directly — no cross-platform home-dir crate, matching the approved spec).
- GeoJSON coordinates are `[lon, lat]`; this codebase's convention (see `geo::CITIES`, `geo::range_bearing`) is `(lat, lon)`. All parsing must flip the order — this is the single easiest bug to introduce in this feature.
- A ring is only drawn between two consecutive vertices if **both** are within `scope::MAX_RANGE_KM` (230 km) of KJGX — same threshold city markers already use. No partial-segment clipping.
- Border line color: `Color32::from_rgb(0x8a, 0x6d, 0x4a)` (muted brown), distinct from the existing grid green (`0x2a3a2f`) and city-marker pale yellow (`0xddddaa`).
- On fetch failure, the app must still run normally with no borders drawn — no panics, no crash, no retry within the same run.

---

### Task 1: GeoJSON ring parsing

**Files:**
- Modify: `Cargo.toml`
- Create: `src/borders.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Produces: `pub type Ring = Vec<(f64, f64)>;` and `fn parse_geojson_rings(json: &str) -> anyhow::Result<Vec<Ring>>` (private to `borders.rs`, used internally and by this task's own tests).

- [ ] **Step 1: Add the `serde_json` dependency**

Edit `Cargo.toml`, adding this line under `[dependencies]` (alongside the existing `anyhow`/`chrono` lines):

```toml
serde_json = "1"
```

- [ ] **Step 2: Create `src/borders.rs` with failing tests**

```rust
//! Loading and caching US state boundary data for the radarscope overlay.
//!
//! Boundaries come from the Census Bureau's TIGERweb REST API as GeoJSON.
//! GeoJSON coordinates are `[lon, lat]`; everything in this module flips
//! that to `(lat, lon)` to match the rest of the codebase's convention
//! (see `geo::CITIES`, `geo::range_bearing`).

use anyhow::{Result, anyhow};
use serde_json::Value;

/// One polygon ring: a closed sequence of (lat, lon) vertices in degrees.
pub type Ring = Vec<(f64, f64)>;

/// Parse a GeoJSON `FeatureCollection` of Polygon/MultiPolygon features into
/// a flat list of rings. Ring winding order (exterior vs. hole) and feature
/// properties are ignored — every ring is drawn as a plain border line.
fn parse_geojson_rings(json: &str) -> Result<Vec<Ring>> {
    let root: Value = serde_json::from_str(json).map_err(|e| anyhow!("parsing GeoJSON: {e}"))?;
    let features = root
        .get("features")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("GeoJSON has no \"features\" array"))?;

    let mut rings = Vec::new();
    for feature in features {
        let geometry = feature
            .get("geometry")
            .ok_or_else(|| anyhow!("feature has no \"geometry\""))?;
        let geom_type = geometry
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("geometry has no \"type\""))?;
        let coordinates = geometry
            .get("coordinates")
            .ok_or_else(|| anyhow!("geometry has no \"coordinates\""))?;

        match geom_type {
            "Polygon" => rings.extend(polygon_rings(coordinates)?),
            "MultiPolygon" => {
                let polygons = coordinates
                    .as_array()
                    .ok_or_else(|| anyhow!("MultiPolygon coordinates are not an array"))?;
                for polygon in polygons {
                    rings.extend(polygon_rings(polygon)?);
                }
            }
            other => return Err(anyhow!("unsupported geometry type: {other}")),
        }
    }
    Ok(rings)
}

/// Convert a Polygon's `coordinates` value (an array of rings, each an
/// array of `[lon, lat]` pairs) into `Ring`s.
fn polygon_rings(coordinates: &Value) -> Result<Vec<Ring>> {
    let rings_json = coordinates
        .as_array()
        .ok_or_else(|| anyhow!("Polygon coordinates are not an array"))?;

    rings_json
        .iter()
        .map(|ring| {
            let points = ring
                .as_array()
                .ok_or_else(|| anyhow!("ring is not an array"))?;
            points
                .iter()
                .map(|point| {
                    let pair = point
                        .as_array()
                        .ok_or_else(|| anyhow!("coordinate pair is not an array"))?;
                    let lon = pair
                        .first()
                        .and_then(Value::as_f64)
                        .ok_or_else(|| anyhow!("missing longitude"))?;
                    let lat = pair
                        .get(1)
                        .and_then(Value::as_f64)
                        .ok_or_else(|| anyhow!("missing latitude"))?;
                    Ok((lat, lon))
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLYGON_FIXTURE: &str = r#"{
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "properties": {"STUSAB": "GA"},
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[[-83.0, 32.0], [-83.1, 32.1], [-83.2, 32.0], [-83.0, 32.0]]]
                }
            }
        ]
    }"#;

    const MULTIPOLYGON_FIXTURE: &str = r#"{
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "properties": {"STUSAB": "FL"},
                "geometry": {
                    "type": "MultiPolygon",
                    "coordinates": [
                        [[[-82.0, 27.0], [-82.1, 27.1], [-82.2, 27.0], [-82.0, 27.0]]],
                        [[[-80.0, 25.0], [-80.1, 25.1], [-80.2, 25.0], [-80.0, 25.0]]]
                    ]
                }
            }
        ]
    }"#;

    #[test]
    fn parses_polygon_ring_as_lat_lon_pairs() {
        let rings = parse_geojson_rings(POLYGON_FIXTURE).unwrap();
        assert_eq!(rings.len(), 1);
        assert_eq!(
            rings[0],
            vec![(32.0, -83.0), (32.1, -83.1), (32.0, -83.2), (32.0, -83.0)]
        );
    }

    #[test]
    fn parses_multipolygon_as_one_ring_per_part() {
        let rings = parse_geojson_rings(MULTIPOLYGON_FIXTURE).unwrap();
        assert_eq!(rings.len(), 2);
        assert_eq!(rings[0][0], (27.0, -82.0));
        assert_eq!(rings[1][0], (25.0, -80.0));
    }

    #[test]
    fn rejects_malformed_json() {
        assert!(parse_geojson_rings("not json").is_err());
    }

    #[test]
    fn rejects_missing_features_array() {
        assert!(parse_geojson_rings(r#"{"type": "FeatureCollection"}"#).is_err());
    }
}
```

- [ ] **Step 3: Register the module**

Edit `src/lib.rs`, adding `borders` to the alphabetically-sorted module list:

```rust
pub mod app;
pub mod borders;
pub mod colors;
pub mod data;
pub mod geo;
pub mod model;
pub mod scope;
```

- [ ] **Step 4: Run the tests to confirm they compile and pass**

Run: `cargo test --lib borders::`
Expected: 4 tests pass (`parses_polygon_ring_as_lat_lon_pairs`, `parses_multipolygon_as_one_ring_per_part`, `rejects_malformed_json`, `rejects_missing_features_array`).

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock src/borders.rs src/lib.rs
git commit -m "feat: add GeoJSON ring parsing for state boundary overlay"
```

---

### Task 2: Cache path resolution and load-from-cache

**Files:**
- Modify: `src/borders.rs`

**Interfaces:**
- Consumes: `Ring` type and `parse_geojson_rings` from Task 1 (same file, private).
- Produces: `pub fn load_or_fetch(path: &std::path::Path) -> anyhow::Result<Vec<Ring>>` — the network-fetch branch is stubbed to return an error in this task and completed in Task 3.

- [ ] **Step 1: Add a failing test for cache-path resolution**

Add to the `tests` module in `src/borders.rs`:

```rust
    #[test]
    fn cache_path_is_under_dot_rustywx() {
        let path = cache_path_under(std::path::Path::new("/home/example"));
        assert_eq!(
            path,
            std::path::Path::new("/home/example/.rustywx/state_borders.geojson")
        );
    }
```

- [ ] **Step 2: Run it to confirm it fails to compile**

Run: `cargo test --lib borders::cache_path_is_under_dot_rustywx`
Expected: FAIL — `cannot find function \`cache_path_under\` in this scope`

- [ ] **Step 3: Implement `cache_path_under` and `cache_path`**

Add above the `#[cfg(test)]` block in `src/borders.rs`:

```rust
use std::path::{Path, PathBuf};

/// Where the borders cache lives, under a given home directory. Split out
/// from `cache_path` so it's testable without touching the real `$HOME` env
/// var (mutating env vars in tests is racy across parallel test threads).
fn cache_path_under(home: &Path) -> PathBuf {
    home.join(".rustywx").join("state_borders.geojson")
}

/// Where the borders cache lives on this machine.
fn cache_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").map_err(|_| anyhow!("HOME environment variable is not set"))?;
    Ok(cache_path_under(Path::new(&home)))
}
```

- [ ] **Step 4: Run it to confirm it passes**

Run: `cargo test --lib borders::cache_path_is_under_dot_rustywx`
Expected: PASS

- [ ] **Step 5: Add a failing test for reading an existing cache file**

Add to the `tests` module:

```rust
    fn unique_temp_path(name: &str) -> PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("rustywx-test-{}-{n}-{name}", std::process::id()))
    }

    #[test]
    fn load_or_fetch_reads_existing_cache_without_network() {
        let path = unique_temp_path("cache-hit.geojson");
        std::fs::write(&path, POLYGON_FIXTURE).unwrap();

        let rings = load_or_fetch(&path).unwrap();

        assert_eq!(rings.len(), 1);
        std::fs::remove_file(&path).unwrap();
    }
```

- [ ] **Step 6: Run it to confirm it fails to compile**

Run: `cargo test --lib borders::load_or_fetch_reads_existing_cache_without_network`
Expected: FAIL — `cannot find function \`load_or_fetch\` in this scope`

- [ ] **Step 7: Implement `load_or_fetch` with a stubbed fetch branch**

Add above the `#[cfg(test)]` block:

```rust
/// Load cached state-boundary rings from `path`, fetching and caching them
/// first if the file doesn't exist yet. Malformed or unreadable existing
/// files are returned as errors rather than silently overwritten — a
/// corrupt cache is left for a human to investigate.
pub fn load_or_fetch(path: &Path) -> Result<Vec<Ring>> {
    match std::fs::read_to_string(path) {
        Ok(json) => parse_geojson_rings(&json),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let json = fetch_geojson()?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| anyhow!("creating {}: {e}", parent.display()))?;
            }
            std::fs::write(path, &json).map_err(|e| anyhow!("writing {}: {e}", path.display()))?;
            parse_geojson_rings(&json)
        }
        Err(e) => Err(anyhow!("reading {}: {e}", path.display())),
    }
}

/// Placeholder for Task 3, which replaces this with a real TIGERweb fetch.
fn fetch_geojson() -> Result<String> {
    Err(anyhow!("state border fetch not yet implemented"))
}
```

- [ ] **Step 8: Run it to confirm it passes**

Run: `cargo test --lib borders::load_or_fetch_reads_existing_cache_without_network`
Expected: PASS (this test only exercises the cache-hit branch, so the stubbed `fetch_geojson` is never called)

- [ ] **Step 9: Run the full unit test suite**

Run: `cargo test --lib`
Expected: all tests pass, including the 4 from Task 1 and the 2 new ones from this task.

- [ ] **Step 10: Commit**

```bash
git add src/borders.rs
git commit -m "feat: add state-boundary cache-file read path"
```

---

### Task 3: Real TIGERweb fetch

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/borders.rs`
- Create: `tests/borders_network.rs`

**Interfaces:**
- Consumes: `load_or_fetch` from Task 2 (replaces its stubbed `fetch_geojson`).
- Produces: nothing new for other tasks — `fetch_geojson` stays private; the live test drives it through the already-`pub` `load_or_fetch`.

- [ ] **Step 1: Add the `ureq` dependency**

Edit `Cargo.toml`, adding this line under `[dependencies]`:

```toml
ureq = "3"
```

- [ ] **Step 2: Replace the stubbed `fetch_geojson` with the real implementation**

In `src/borders.rs`, replace:

```rust
/// Placeholder for Task 3, which replaces this with a real TIGERweb fetch.
fn fetch_geojson() -> Result<String> {
    Err(anyhow!("state border fetch not yet implemented"))
}
```

with:

```rust
/// Georgia plus its neighbors whose border could plausibly fall within the
/// scope's 230 km display radius from KJGX: Alabama, South Carolina, Florida.
/// Verified live against the TIGERweb service: returns 4 features (SC, GA,
/// AL as `Polygon`, FL as `MultiPolygon`).
const TIGERWEB_URL: &str = "https://tigerweb.geo.census.gov/arcgis/rest/services/TIGERweb/State_County/MapServer/0/query?where=STUSAB+IN+(%27GA%27%2C%27AL%27%2C%27SC%27%2C%27FL%27)&outFields=STUSAB,NAME&f=geojson";

/// Fetch the GA/AL/SC/FL state boundaries from the Census Bureau's TIGERweb
/// REST service as GeoJSON. A single blocking request — unlike
/// `data::fetch_latest_scan`, this has no polling loop, since boundary data
/// doesn't change on a timescale this app cares about.
fn fetch_geojson() -> Result<String> {
    ureq::get(TIGERWEB_URL)
        .call()
        .map_err(|e| anyhow!("fetching state borders: {e}"))?
        .body_mut()
        .read_to_string()
        .map_err(|e| anyhow!("reading state borders response: {e}"))
}
```

- [ ] **Step 3: Write the live-network integration test**

Create `tests/borders_network.rs`:

```rust
//! Live-network test for the TIGERweb state-boundary fetch, excluded from
//! normal runs. Execute explicitly with:
//! `cargo test --test borders_network -- --ignored`

#[test]
#[ignore = "requires network access to the Census TIGERweb REST API"]
fn fetches_and_caches_state_borders() {
    let path = std::env::temp_dir().join(format!(
        "rustywx-live-borders-test-{}.geojson",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path); // ensure a clean slate

    let rings = rustywx::borders::load_or_fetch(&path).expect("fetch+parse should succeed");

    assert!(!rings.is_empty(), "should return at least one border ring");
    assert!(path.exists(), "should have written a local cache file");

    std::fs::remove_file(&path).ok();
}
```

- [ ] **Step 4: Run the unit tests (no network needed) to confirm nothing broke**

Run: `cargo test --lib`
Expected: all tests still pass (the stub replacement doesn't affect the cache-hit test from Task 2).

- [ ] **Step 5: Run the live test to confirm the real fetch works**

Run: `cargo test --test borders_network -- --ignored`
Expected: PASS — `fetches_and_caches_state_borders` succeeds against the real TIGERweb API.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/borders.rs tests/borders_network.rs
git commit -m "feat: fetch state boundaries from Census TIGERweb API"
```

---

### Task 4: Wire the border loader into the app

**Files:**
- Modify: `src/borders.rs`
- Modify: `src/app.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: `load_or_fetch`, `cache_path` (Task 2/3, same file); `Ring` (Task 1).
- Produces: `pub enum BorderMessage { Loaded(Vec<Ring>), Error(String) }` and `pub fn spawn_border_loader(tx: std::sync::mpsc::Sender<BorderMessage>, egui_ctx: egui::Context)`, both consumed by `app.rs`/`main.rs` and by Task 5 (`scope.rs` only needs `Ring`, not these).

This task has no new unit tests of its own — `BorderMessage`/`spawn_border_loader`/the `RadarApp` wiring are thin plumbing around already-tested logic (`load_or_fetch`), and `RadarApp` has no existing unit tests to extend (verified by manual run in Step 6, same as the rest of `app.rs`).

- [ ] **Step 1: Add `BorderMessage` and `spawn_border_loader` to `src/borders.rs`**

Add at the end of `src/borders.rs`, above the `#[cfg(test)]` block:

```rust
/// Messages the border-loader thread sends to the UI thread.
pub enum BorderMessage {
    Loaded(Vec<Ring>),
    Error(String),
}

/// Spawn a one-shot thread that loads (fetching and caching first, if
/// needed) the state-boundary rings and reports them once. Unlike
/// `data::spawn_worker`, this thread does its work once and exits — there's
/// no polling loop, since state borders aren't expected to change.
pub fn spawn_border_loader(tx: std::sync::mpsc::Sender<BorderMessage>, egui_ctx: egui::Context) {
    std::thread::spawn(move || {
        let message = match cache_path().and_then(|path| load_or_fetch(&path)) {
            Ok(rings) => BorderMessage::Loaded(rings),
            Err(e) => BorderMessage::Error(format!("{e:#}")),
        };
        let _ = tx.send(message);
        egui_ctx.request_repaint();
    });
}
```

- [ ] **Step 2: Add the `egui` dependency reference**

`egui` is already a dependency of this crate (used throughout `app.rs`/`scope.rs`), so no `Cargo.toml` change is needed here — `src/borders.rs` can use `egui::Context` directly since it's declared as a normal `use` inside the crate.

- [ ] **Step 3: Wire the receiver and state into `RadarApp`**

In `src/app.rs`, update the imports and struct:

```rust
use crate::borders::BorderMessage;
use crate::data::{SITE, WorkerMessage};
use crate::model::{Product, ScanData};
use crate::scope;
use chrono::{DateTime, Utc};
use egui::{Color32, TextureHandle, TextureOptions, Ui};
use std::sync::mpsc::Receiver;

pub struct RadarApp {
    rx: Receiver<WorkerMessage>,
    border_rx: Receiver<BorderMessage>,
    scan: Option<ScanData>,
    product: Product,
    tilt_index: usize,
    status: String,
    texture: Option<TextureHandle>,
    texture_key: Option<(DateTime<Utc>, Product, usize)>,
    borders: Vec<crate::borders::Ring>,
}
```

Update the constructor:

```rust
impl RadarApp {
    pub fn new(rx: Receiver<WorkerMessage>, border_rx: Receiver<BorderMessage>) -> Self {
        Self {
            rx,
            border_rx,
            scan: None,
            product: Product::Reflectivity,
            tilt_index: 0,
            status: format!("Starting up — fetching latest {SITE} volume…"),
            texture: None,
            texture_key: None,
            borders: Vec::new(),
        }
    }
```

Update `drain_messages` to also drain the border channel:

```rust
    fn drain_messages(&mut self) {
        while let Ok(message) = self.rx.try_recv() {
            match message {
                WorkerMessage::NewScan(scan) => {
                    self.status = format!(
                        "Scan {} UTC ({} local)",
                        scan.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        scan.timestamp
                            .with_timezone(&chrono::Local)
                            .format("%H:%M:%S")
                    );
                    self.scan = Some(*scan);
                }
                WorkerMessage::Status(text) => self.status = text,
                WorkerMessage::Error(text) => self.status = format!("Error: {text}"),
            }
        }
        while let Ok(message) = self.border_rx.try_recv() {
            match message {
                BorderMessage::Loaded(rings) => self.borders = rings,
                BorderMessage::Error(text) => {
                    self.status = format!("State borders unavailable: {text}");
                }
            }
        }
    }
```

- [ ] **Step 4: Wire the new channel and thread in `src/main.rs`**

Replace the body of `main.rs` with:

```rust
fn main() -> eframe::Result {
    let (tx, rx) = std::sync::mpsc::channel();
    let (border_tx, border_rx) = std::sync::mpsc::channel();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 960.0])
            .with_title("rustywx — KJGX radarscope (Macon, GA)"),
        ..Default::default()
    };

    eframe::run_native(
        "rustywx",
        options,
        Box::new(move |cc| {
            rustywx::data::spawn_worker(tx, cc.egui_ctx.clone());
            rustywx::borders::spawn_border_loader(border_tx, cc.egui_ctx.clone());
            Ok(Box::new(rustywx::app::RadarApp::new(rx, border_rx)))
        }),
    )
}
```

- [ ] **Step 5: Confirm it builds**

Run: `cargo build`
Expected: builds successfully (this task changes signatures, not behavior visible in tests — `scope::draw_scope`'s call site is updated in Task 5, so don't run `cargo run` until then).

- [ ] **Step 6: Commit**

```bash
git add src/borders.rs src/app.rs src/main.rs
git commit -m "feat: wire state-boundary loader into the app"
```

---

### Task 5: Draw the borders on the scope

**Files:**
- Modify: `src/scope.rs`
- Modify: `src/app.rs`

**Interfaces:**
- Consumes: `crate::borders::Ring` (Task 1); `geo::range_bearing`, `geo::polar_to_offset`, `geo::KJGX_LAT`, `geo::KJGX_LON` (existing, unchanged); `self.borders: Vec<Ring>` (Task 4, `app.rs`).

- [ ] **Step 1: Add the border color constant and update `draw_scope`'s signature**

In `src/scope.rs`, add near the top (alongside the other `pub const` declarations):

```rust
/// Muted brown for state border overlays, distinct from the grid green
/// (`0x2a3a2f`) and the pale-yellow city markers (`0xddddaa`).
const BORDER_COLOR: Color32 = Color32::from_rgb(0x8a, 0x6d, 0x4a);
```

Change the `draw_scope` signature from:

```rust
pub fn draw_scope(
    ui: &mut egui::Ui,
    texture: Option<&egui::TextureHandle>,
    scan: Option<&crate::model::ScanData>,
    product: Product,
) {
```

to:

```rust
pub fn draw_scope(
    ui: &mut egui::Ui,
    texture: Option<&egui::TextureHandle>,
    scan: Option<&crate::model::ScanData>,
    product: Product,
    borders: &[crate::borders::Ring],
) {
```

- [ ] **Step 2: Draw the border rings**

In `src/scope.rs`, inside `draw_scope`, insert this block right after the cardinal-spokes `for` loop (which ends with the closing `}` after `);` for the spoke labels) and before the `// Station marker at scope center.` comment:

```rust
    // State border outlines, drawn only where both endpoints of a segment
    // are within the display radius (same rule city markers use).
    for ring in borders {
        for pair in ring.windows(2) {
            let (lat1, lon1) = pair[0];
            let (lat2, lon2) = pair[1];
            let (range1, bearing1) = geo::range_bearing(geo::KJGX_LAT, geo::KJGX_LON, lat1, lon1);
            let (range2, bearing2) = geo::range_bearing(geo::KJGX_LAT, geo::KJGX_LON, lat2, lon2);
            if range1 as f32 > MAX_RANGE_KM || range2 as f32 > MAX_RANGE_KM {
                continue;
            }
            let (dx1, dy1) = geo::polar_to_offset(bearing1 as f32, range1 as f32, px_per_km);
            let (dx2, dy2) = geo::polar_to_offset(bearing2 as f32, range2 as f32, px_per_km);
            painter.line_segment(
                [center + vec2(dx1, dy1), center + vec2(dx2, dy2)],
                Stroke::new(1.2, BORDER_COLOR),
            );
        }
    }
```

- [ ] **Step 3: Update the call site in `src/app.rs`**

Change:

```rust
                scope::draw_scope(ui, self.texture.as_ref(), self.scan.as_ref(), self.product);
```

to:

```rust
                scope::draw_scope(
                    ui,
                    self.texture.as_ref(),
                    self.scan.as_ref(),
                    self.product,
                    &self.borders,
                );
```

- [ ] **Step 4: Run the full unit test suite**

Run: `cargo test --lib`
Expected: all existing `scope::` tests (`rasterizes_quadrants_to_expected_colors`, `center_and_beyond_range_are_transparent`, `gates_beyond_radial_data_are_transparent`, `nearest_radial_wraps_around_north`) still pass unchanged — `draw_scope` itself has no unit tests (it requires a live `egui::Ui`, consistent with the rest of this file), so this step is a regression check, not new coverage.

- [ ] **Step 5: Manually verify the borders render**

Run: `cargo run --release`

Expected: after the first scan loads, thin brown lines appear on the scope roughly where Georgia's borders with Alabama (west), South Carolina (northeast), and Florida (south) would be, positioned plausibly relative to the existing Macon/Warner Robins city markers. On first run this requires network access to fetch and cache `~/.rustywx/state_borders.geojson`; on later runs it should appear immediately from the cache with no fetch delay.

- [ ] **Step 6: Commit**

```bash
git add src/scope.rs src/app.rs
git commit -m "feat: draw state border overlay on the radarscope"
```

---

### Task 6: Update documentation

**Files:**
- Modify: `README.md`
- Modify: `CONTRIBUTING.md`
- Modify: `docs/USER_GUIDE.md`

- [ ] **Step 1: Add the new module to `README.md`'s architecture list**

In `README.md`, change:

```
- `src/app.rs` — egui application state and layout
```

to:

```
- `src/app.rs` — egui application state and layout
- `src/borders.rs` — fetches/caches US state boundary lines for the scope overlay
```

Also update the Features list to mention it:

```
- Range rings, cardinal spokes, city markers, and state border lines
```

(replacing the existing `- Range rings, cardinal spokes, and city markers` line).

- [ ] **Step 2: Add the module to `CONTRIBUTING.md`'s module table and testing section**

In `CONTRIBUTING.md`'s module table, add a row after the `src/geo.rs` row:

```
| [`src/borders.rs`](src/borders.rs) | Loads US state boundary lines for the scope overlay: checks `~/.rustywx/state_borders.geojson`, fetching it from the Census TIGERweb REST API on first run if missing, then reports parsed rings to the UI over its own one-shot channel. |
```

In the Testing section, update the live-test command list from:

```
cargo test --test network -- --ignored    # live end-to-end fetch/decode
```

to:

```
cargo test --test network -- --ignored          # live NEXRAD fetch/decode
cargo test --test borders_network -- --ignored  # live TIGERweb state-border fetch
```

- [ ] **Step 3: Add a row to `docs/USER_GUIDE.md`'s display table**

In the "Reading the display" table, add a row after the city-markers row:

```
| **Thin brown lines** | State borders (Georgia/Alabama/South Carolina/Florida), for orientation. Only drawn once the app has downloaded and cached them — see below. |
```

Add a short note after the table:

```
State border lines require a one-time download the first time you run
rustywx (cached afterward at `~/.rustywx/state_borders.geojson`, so later
launches show them immediately with no delay). If they don't appear, the
radar display and all controls still work normally — see
[Troubleshooting](#troubleshooting).
```

- [ ] **Step 4: Commit**

```bash
git add README.md CONTRIBUTING.md docs/USER_GUIDE.md
git commit -m "docs: document the state boundary overlay"
```
