# NEXRAD Radarscope (rustywx) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A Rust desktop GUI app that fetches live NEXRAD Level II data for KJGX (the radar covering Macon, GA) from the public AWS archive bucket and displays it as a classic PPI radarscope with auto-refresh, product toggle, tilt selector, and geographic overlays.

**Architecture:** Two threads — an eframe/egui UI thread and a worker thread owning a current-thread tokio runtime that polls S3, decodes volumes via the `nexrad-*` crates, and sends decoded scans over an `mpsc` channel. The selected sweep is CPU-rasterized to an `egui::ColorImage` only when scan/product/tilt changes, uploaded as a texture, and overlays (rings, spokes, city markers, legend) are painted on top each frame.

**Tech Stack:** Rust edition 2024, eframe/egui 0.35, nexrad-data 0.2 (default features: aws + decode + nexrad-model), nexrad-model 0.1, tokio 1 (rt + time), chrono 0.4, anyhow 1.

**Spec:** `docs/superpowers/specs/2026-07-13-rustywx-radarscope-design.md`

## Global Constraints

- Radar site: **KJGX**, antenna at lat `32.6755`, lon `-83.3511`.
- Data source: AWS public archive bucket via `nexrad_data::aws::archive::{list_files, download_file}` (bucket name is internal to the crate).
- Cities overlay: Macon (`32.8407, -83.6324`), Warner Robins (`32.6130, -83.6242`).
- Display: scope centered on the radar antenna, north up, range rings every 50 km out to 230 km max range.
- Gate geometry constants (Level II super-res): first gate at `2.125` km, gate spacing `0.25` km.
- Poll interval: 120 s; error retry backoff: 30 s doubling per consecutive error, capped at 600 s.
- The UI thread never performs I/O or decoding. Worker failures must never panic the app; they surface as status-bar text.
- Crate layout: library (`src/lib.rs`) + thin binary (`src/main.rs`) so tests can import modules.
- All code must pass `cargo test` and `cargo clippy -- -D warnings` at each commit.
- Commits use Conventional Commits style and end with `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`.

**API facts verified against crate sources (do not re-derive):**
- `nexrad_data::aws::archive::list_files(site: &str, date: &chrono::NaiveDate) -> Result<Vec<Identifier>>` (async). `Identifier` is `Ord` by file name (names sort chronologically), has `.name() -> &str` and `.date_time() -> Option<DateTime<Utc>>`. Filter out names ending in `"_MDM"` (metadata files).
- `nexrad_data::aws::archive::download_file(id: Identifier) -> Result<nexrad_data::volume::File>` (async).
- `File::scan() -> Result<nexrad_model::data::Scan>` does full decompress + decode.
- `Scan::sweeps() -> &Vec<Sweep>`; `Sweep::radials() -> &Vec<Radial>`.
- `Radial` getters: `azimuth_angle_degrees() -> f32`, `azimuth_spacing_degrees() -> f32`, `elevation_angle_degrees() -> f32`, `reflectivity() / velocity() -> Option<&MomentData>`.
- `MomentData::values() -> Vec<MomentValue>` where `MomentValue` is `Value(f32) | BelowThreshold | RangeFolded`. Reflectivity values are dBZ; velocity values are m/s.
- Test constructors are public: `Radial::new(collection_timestamp: i64, azimuth_number: u16, azimuth_angle_degrees: f32, azimuth_spacing_degrees: f32, radial_status: RadialStatus, elevation_number: u8, elevation_angle_degrees: f32, reflectivity: Option<MomentData>, velocity: Option<MomentData>, spectrum_width: Option<MomentData>, differential_reflectivity: Option<MomentData>, differential_phase: Option<MomentData>, correlation_coefficient: Option<MomentData>, specific_differential_phase: Option<MomentData>)` and `MomentData::from_fixed_point(scale: f32, offset: f32, values: Vec<u8>)` (decoded value = `(raw - offset) / scale`; raw `0` = BelowThreshold, raw `1` = RangeFolded when scale ≠ 0).
- `egui::ColorImage::new(size: [usize; 2], pixels: Vec<Color32>)`; `ctx.load_texture(name, image, TextureOptions) -> TextureHandle`.
- `eframe::run_native(app_name: &str, NativeOptions, Box::new(|cc| Ok(Box::new(app))))`; worker gets a repaint handle via `cc.egui_ctx.clone()`.
- Known upstream quirk: `Sweep::from_radials` (used by `File::scan()`) drops the final elevation group of the volume, so the topmost tilt is absent. Accepted for v1 — do not work around it.

---

### Task 1: Dependencies, crate skeleton, and geo module

**Files:**
- Modify: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/geo.rs`
- Modify: `src/main.rs` (temporary placeholder retained)

**Interfaces:**
- Consumes: nothing (first task).
- Produces: `geo::KJGX_LAT: f64`, `geo::KJGX_LON: f64`, `geo::CITIES: &[(&str, f64, f64)]`,
  `geo::range_bearing(from_lat: f64, from_lon: f64, to_lat: f64, to_lon: f64) -> (f64, f64)` (km, degrees clockwise from true north),
  `geo::polar_to_offset(azimuth_deg: f32, range_km: f32, px_per_km: f32) -> (f32, f32)` (screen-space x/y offsets from scope center, +y down).

- [ ] **Step 1: Add dependencies to `Cargo.toml`**

Replace the `[dependencies]` section (and add dev-dependencies):

```toml
[dependencies]
anyhow = "1"
chrono = "0.4"
eframe = "0.35"
egui = "0.35"
nexrad-data = "0.2"
nexrad-model = "0.1"
tokio = { version = "1", features = ["rt", "time"] }

[dev-dependencies]
tokio = { version = "1", features = ["rt", "macros"] }
```

- [ ] **Step 2: Create the library skeleton**

`src/lib.rs`:

```rust
pub mod geo;
```

`src/main.rs` (placeholder until Task 6):

```rust
fn main() {
    println!("rustywx: UI arrives in Task 6");
}
```

Run: `cargo check`
Expected: compiles (first build takes a few minutes; `bzip2-sys` needs a C compiler).

- [ ] **Step 3: Write failing tests for the geo math**

Create `src/geo.rs` with constants and test module first (functions not yet written):

```rust
//! Geographic math for the radarscope: great-circle range/bearing and
//! polar-to-screen projection. Azimuthal equidistant approximation is
//! acceptable at <= 230 km display range.

/// KJGX (Robins AFB, GA) antenna location.
pub const KJGX_LAT: f64 = 32.6755;
pub const KJGX_LON: f64 = -83.3511;

/// Cities drawn on the scope: (name, lat, lon).
pub const CITIES: &[(&str, f64, f64)] = &[
    ("Macon", 32.8407, -83.6324),
    ("Warner Robins", 32.6130, -83.6242),
];

const EARTH_RADIUS_KM: f64 = 6371.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macon_range_bearing_from_kjgx() {
        let (range_km, bearing_deg) = range_bearing(KJGX_LAT, KJGX_LON, 32.8407, -83.6324);
        assert!((31.0..34.0).contains(&range_km), "range {range_km}");
        assert!((302.0..308.0).contains(&bearing_deg), "bearing {bearing_deg}");
    }

    #[test]
    fn range_bearing_due_north() {
        let (range_km, bearing_deg) = range_bearing(32.0, -83.0, 33.0, -83.0);
        assert!((range_km - 111.2).abs() < 1.0, "range {range_km}");
        assert!(bearing_deg.abs() < 0.01 || (bearing_deg - 360.0).abs() < 0.01);
    }

    #[test]
    fn polar_offsets_cardinal_directions() {
        // North: straight up the screen (negative y).
        let (x, y) = polar_to_offset(0.0, 10.0, 2.0);
        assert!(x.abs() < 1e-4 && (y + 20.0).abs() < 1e-4, "north ({x},{y})");
        // East: +x.
        let (x, y) = polar_to_offset(90.0, 10.0, 2.0);
        assert!((x - 20.0).abs() < 1e-3 && y.abs() < 1e-3, "east ({x},{y})");
        // South: +y (screen y grows downward).
        let (x, y) = polar_to_offset(180.0, 10.0, 2.0);
        assert!(x.abs() < 1e-3 && (y - 20.0).abs() < 1e-3, "south ({x},{y})");
    }
}
```

- [ ] **Step 4: Run tests to verify they fail**

Run: `cargo test geo -- --nocapture`
Expected: FAIL to compile with "cannot find function `range_bearing`" and "cannot find function `polar_to_offset`".

- [ ] **Step 5: Implement the geo functions**

Add above the test module in `src/geo.rs`:

```rust
/// Great-circle distance (km) and initial bearing (degrees clockwise from
/// true north, in [0, 360)) from one lat/lon to another. Haversine formula.
pub fn range_bearing(from_lat: f64, from_lon: f64, to_lat: f64, to_lon: f64) -> (f64, f64) {
    let lat1 = from_lat.to_radians();
    let lat2 = to_lat.to_radians();
    let dlat = (to_lat - from_lat).to_radians();
    let dlon = (to_lon - from_lon).to_radians();

    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let range_km = 2.0 * EARTH_RADIUS_KM * a.sqrt().asin();

    let y = dlon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();
    let bearing_deg = (y.atan2(x).to_degrees() + 360.0) % 360.0;

    (range_km, bearing_deg)
}

/// Convert a polar position (azimuth degrees clockwise from north, range in
/// km) to screen-space pixel offsets from the scope center. Screen +y is
/// down, so north maps to a negative y offset.
pub fn polar_to_offset(azimuth_deg: f32, range_km: f32, px_per_km: f32) -> (f32, f32) {
    let theta = azimuth_deg.to_radians();
    let r = range_km * px_per_km;
    (r * theta.sin(), -r * theta.cos())
}
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test geo`
Expected: 3 tests PASS.

- [ ] **Step 7: Lint and commit**

Run: `cargo clippy -- -D warnings` — expected: no warnings.

```bash
git add Cargo.toml Cargo.lock src/lib.rs src/main.rs src/geo.rs
git commit -m "feat: add dependencies and geo math (range/bearing, polar projection)

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 2: Color tables

**Files:**
- Create: `src/colors.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: `colors::dbz_color(dbz: f32) -> egui::Color32`, `colors::velocity_color(ms: f32) -> egui::Color32`, and legend tables `colors::DBZ_LEGEND: &[(f32, egui::Color32)]`, `colors::VELOCITY_LEGEND: &[(f32, egui::Color32)]` (threshold, color) for drawing the color scale.

- [ ] **Step 1: Write failing tests**

Create `src/colors.rs` containing only the test module:

```rust
//! NWS-style stepped color tables for base reflectivity (dBZ) and base
//! velocity (m/s). Values below the product minimum are transparent.

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Color32;

    #[test]
    fn dbz_below_minimum_is_transparent() {
        assert_eq!(dbz_color(-10.0), Color32::TRANSPARENT);
        assert_eq!(dbz_color(4.9), Color32::TRANSPARENT);
    }

    #[test]
    fn dbz_bands() {
        assert_eq!(dbz_color(5.0), Color32::from_rgb(0x04, 0xe9, 0xe7)); // light cyan
        assert_eq!(dbz_color(20.0), Color32::from_rgb(0x02, 0xfd, 0x02)); // green
        assert_eq!(dbz_color(52.0), Color32::from_rgb(0xfd, 0x00, 0x00)); // red
        assert_eq!(dbz_color(80.0), Color32::from_rgb(0xfd, 0xfd, 0xfd)); // white cap
    }

    #[test]
    fn velocity_sign_convention() {
        // Inbound (negative) is green; outbound (positive) is red.
        let inbound = velocity_color(-25.0);
        let outbound = velocity_color(25.0);
        assert!(inbound.g() > inbound.r(), "inbound should be green: {inbound:?}");
        assert!(outbound.r() > outbound.g(), "outbound should be red: {outbound:?}");
    }

    #[test]
    fn legends_are_ascending() {
        assert!(DBZ_LEGEND.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(VELOCITY_LEGEND.windows(2).all(|w| w[0].0 < w[1].0));
    }
}
```

Add `pub mod colors;` to `src/lib.rs`.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test colors`
Expected: FAIL to compile — `dbz_color`, `velocity_color`, `DBZ_LEGEND`, `VELOCITY_LEGEND` not found.

- [ ] **Step 3: Implement the color tables**

Add above the test module in `src/colors.rs`:

```rust
use egui::Color32;

/// Standard NWS base reflectivity bands: (threshold dBZ, color). A value
/// maps to the color of the highest threshold it meets or exceeds.
pub const DBZ_LEGEND: &[(f32, Color32)] = &[
    (5.0, Color32::from_rgb(0x04, 0xe9, 0xe7)),
    (10.0, Color32::from_rgb(0x01, 0x9f, 0xf4)),
    (15.0, Color32::from_rgb(0x03, 0x00, 0xf4)),
    (20.0, Color32::from_rgb(0x02, 0xfd, 0x02)),
    (25.0, Color32::from_rgb(0x01, 0xc5, 0x01)),
    (30.0, Color32::from_rgb(0x00, 0x8e, 0x00)),
    (35.0, Color32::from_rgb(0xfd, 0xf8, 0x02)),
    (40.0, Color32::from_rgb(0xe5, 0xbc, 0x00)),
    (45.0, Color32::from_rgb(0xfd, 0x95, 0x00)),
    (50.0, Color32::from_rgb(0xfd, 0x00, 0x00)),
    (55.0, Color32::from_rgb(0xd4, 0x00, 0x00)),
    (60.0, Color32::from_rgb(0xbc, 0x00, 0x00)),
    (65.0, Color32::from_rgb(0xf8, 0x00, 0xfd)),
    (70.0, Color32::from_rgb(0x98, 0x54, 0xc6)),
    (75.0, Color32::from_rgb(0xfd, 0xfd, 0xfd)),
];

/// Base velocity bands in m/s: inbound (negative) greens, outbound
/// (positive) reds, weak echoes desaturated near zero.
pub const VELOCITY_LEGEND: &[(f32, Color32)] = &[
    (-64.0, Color32::from_rgb(0x00, 0xff, 0x90)),
    (-40.0, Color32::from_rgb(0x00, 0xe0, 0x00)),
    (-30.0, Color32::from_rgb(0x00, 0xb0, 0x00)),
    (-20.0, Color32::from_rgb(0x00, 0x80, 0x00)),
    (-10.0, Color32::from_rgb(0x4d, 0x66, 0x4d)),
    (0.0, Color32::from_rgb(0x66, 0x4d, 0x4d)),
    (10.0, Color32::from_rgb(0x80, 0x00, 0x00)),
    (20.0, Color32::from_rgb(0xb0, 0x00, 0x00)),
    (30.0, Color32::from_rgb(0xe0, 0x00, 0x00)),
    (40.0, Color32::from_rgb(0xff, 0x50, 0x50)),
];

fn banded(legend: &[(f32, Color32)], value: f32) -> Color32 {
    let mut color = Color32::TRANSPARENT;
    for &(threshold, band_color) in legend {
        if value >= threshold {
            color = band_color;
        } else {
            break;
        }
    }
    color
}

/// Color for a base reflectivity value in dBZ. Below 5 dBZ is transparent.
pub fn dbz_color(dbz: f32) -> Color32 {
    banded(DBZ_LEGEND, dbz)
}

/// Color for a base velocity value in m/s (negative = toward the radar).
/// Velocities below the lowest band (< -64 m/s) are clamped to it.
pub fn velocity_color(ms: f32) -> Color32 {
    banded(VELOCITY_LEGEND, ms.max(-64.0))
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test colors`
Expected: 4 tests PASS.

- [ ] **Step 5: Lint and commit**

Run: `cargo clippy -- -D warnings` — expected: clean.

```bash
git add src/colors.rs src/lib.rs
git commit -m "feat: add NWS-style reflectivity and velocity color tables

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 3: Scan model and nexrad conversion

**Files:**
- Create: `src/model.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: `nexrad_model::data::{Scan, MomentValue}`.
- Produces:
  - `model::Product` — `enum Product { Reflectivity, Velocity }` (derives `Clone, Copy, PartialEq, Eq, Debug, Hash`), with `pub fn label(self) -> &'static str` returning `"Reflectivity"` / `"Velocity"`.
  - `model::RadialData { pub azimuth_deg: f32, pub gates: Vec<Option<f32>> }`
  - `model::SweepData { pub elevation_deg: f32, pub radials: Vec<RadialData> }`
  - `model::ScanData { pub timestamp: chrono::DateTime<chrono::Utc>, pub reflectivity: Vec<SweepData>, pub velocity: Vec<SweepData> }` with `pub fn sweeps(&self, product: Product) -> &[SweepData]` and `pub fn from_nexrad(scan: &nexrad_model::data::Scan, timestamp: DateTime<Utc>) -> Self`.

- [ ] **Step 1: Write failing tests**

Create `src/model.rs` with the test module (a synthetic-scan helper included):

```rust
//! rustywx's own thin scan model, decoupling rendering from nexrad-model.
//! `Option<f32>` gates: `None` = below threshold / range folded (drawn
//! transparent). Sweeps are sorted by elevation and deduplicated so split
//! cuts at the same elevation appear once in the tilt selector.

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use nexrad_model::data::{MomentData, Radial, RadialStatus, Scan, Sweep};

    /// A REF-encoded moment: value = (raw - 66.0) / 2.0 dBZ.
    fn ref_moment(raws: Vec<u8>) -> MomentData {
        MomentData::from_fixed_point(2.0, 66.0, raws)
    }

    fn radial(az: f32, elev_num: u8, elev_deg: f32, refl: Option<MomentData>, vel: Option<MomentData>) -> Radial {
        Radial::new(
            0, 1, az, 0.5, RadialStatus::IntermediateRadialData, elev_num, elev_deg,
            refl, vel, None, None, None, None, None,
        )
    }

    fn synthetic_scan() -> Scan {
        // Sweep 1 (0.5 deg): reflectivity only (split-cut CS).
        let s1 = Sweep::new(1, vec![
            radial(0.0, 1, 0.5, Some(ref_moment(vec![0, 130, 190])), None),
            radial(0.5, 1, 0.5, Some(ref_moment(vec![0, 130, 190])), None),
        ]);
        // Sweep 2 (0.5 deg): velocity only (split-cut CD).
        let s2 = Sweep::new(2, vec![
            radial(0.0, 2, 0.5, None, Some(MomentData::from_fixed_point(2.0, 129.0, vec![0, 1, 65]))),
        ]);
        // Sweep 3 (1.5 deg): both moments.
        let s3 = Sweep::new(3, vec![
            radial(0.0, 3, 1.45, Some(ref_moment(vec![130])), Some(MomentData::from_fixed_point(2.0, 129.0, vec![193]))),
        ]);
        Scan::new(215, vec![s1, s2, s3])
    }

    #[test]
    fn converts_moment_values_to_gates() {
        let scan_data = ScanData::from_nexrad(&synthetic_scan(), Utc::now());
        let sweep = &scan_data.reflectivity[0];
        // raw 0 -> BelowThreshold -> None; raw 130 -> 32 dBZ; raw 190 -> 62 dBZ.
        assert_eq!(sweep.radials[0].gates, vec![None, Some(32.0), Some(62.0)]);
        assert_eq!(sweep.radials[0].azimuth_deg, 0.0);
    }

    #[test]
    fn range_folded_becomes_none() {
        let scan_data = ScanData::from_nexrad(&synthetic_scan(), Utc::now());
        // Velocity sweep at 0.5 deg: raws [0, 1, 65] -> [None, None(RF), Some(-32.0)].
        let sweep = &scan_data.velocity[0];
        assert_eq!(sweep.radials[0].gates, vec![None, None, Some(-32.0)]);
    }

    #[test]
    fn products_split_and_dedup_by_elevation() {
        let scan_data = ScanData::from_nexrad(&synthetic_scan(), Utc::now());
        // Reflectivity: 0.5 deg (from CS cut) and 1.45 deg. The CD cut has no
        // reflectivity so nothing to dedup here, but elevations are ascending.
        let elevations: Vec<f32> = scan_data.reflectivity.iter().map(|s| s.elevation_deg).collect();
        assert_eq!(elevations, vec![0.5, 1.45]);
        // Velocity: 0.5 and 1.45.
        let elevations: Vec<f32> = scan_data.velocity.iter().map(|s| s.elevation_deg).collect();
        assert_eq!(elevations, vec![0.5, 1.45]);
    }

    #[test]
    fn dedups_near_identical_elevations() {
        // Two reflectivity sweeps both at ~0.5 deg -> keep only the first.
        let s1 = Sweep::new(1, vec![radial(0.0, 1, 0.48, Some(ref_moment(vec![130])), None)]);
        let s2 = Sweep::new(2, vec![radial(0.0, 2, 0.52, Some(ref_moment(vec![190])), None)]);
        let scan = Scan::new(215, vec![s1, s2]);
        let scan_data = ScanData::from_nexrad(&scan, Utc::now());
        assert_eq!(scan_data.reflectivity.len(), 1);
        assert_eq!(scan_data.reflectivity[0].radials[0].gates, vec![Some(32.0)]);
    }

    #[test]
    fn sweeps_accessor_selects_product() {
        let scan_data = ScanData::from_nexrad(&synthetic_scan(), Utc::now());
        assert_eq!(scan_data.sweeps(Product::Reflectivity).len(), 2);
        assert_eq!(scan_data.sweeps(Product::Velocity).len(), 2);
    }
}
```

Add `pub mod model;` to `src/lib.rs`.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test model`
Expected: FAIL to compile — `ScanData`, `Product` not found.

- [ ] **Step 3: Implement the model**

Add above the test module in `src/model.rs`:

```rust
use chrono::{DateTime, Utc};
use nexrad_model::data::{MomentValue, Scan};

/// Radar products the app can display.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Product {
    Reflectivity,
    Velocity,
}

impl Product {
    pub fn label(self) -> &'static str {
        match self {
            Product::Reflectivity => "Reflectivity",
            Product::Velocity => "Velocity",
        }
    }
}

/// One ray of gate values. `None` gates are below threshold or range folded.
pub struct RadialData {
    pub azimuth_deg: f32,
    pub gates: Vec<Option<f32>>,
}

/// One full rotation at a single elevation angle, for a single product.
pub struct SweepData {
    pub elevation_deg: f32,
    pub radials: Vec<RadialData>,
}

/// A decoded volume scan, split per product, sweeps sorted by elevation.
pub struct ScanData {
    pub timestamp: DateTime<Utc>,
    pub reflectivity: Vec<SweepData>,
    pub velocity: Vec<SweepData>,
}

impl ScanData {
    pub fn sweeps(&self, product: Product) -> &[SweepData] {
        match product {
            Product::Reflectivity => &self.reflectivity,
            Product::Velocity => &self.velocity,
        }
    }

    pub fn from_nexrad(scan: &Scan, timestamp: DateTime<Utc>) -> Self {
        let mut reflectivity = Vec::new();
        let mut velocity = Vec::new();

        for sweep in scan.sweeps() {
            for (product, out) in [
                (Product::Reflectivity, &mut reflectivity),
                (Product::Velocity, &mut velocity),
            ] {
                let radials: Vec<RadialData> = sweep
                    .radials()
                    .iter()
                    .filter_map(|radial| {
                        let moment = match product {
                            Product::Reflectivity => radial.reflectivity(),
                            Product::Velocity => radial.velocity(),
                        }?;
                        Some(RadialData {
                            azimuth_deg: radial.azimuth_angle_degrees(),
                            gates: moment
                                .values()
                                .into_iter()
                                .map(|value| match value {
                                    MomentValue::Value(v) => Some(v),
                                    MomentValue::BelowThreshold | MomentValue::RangeFolded => None,
                                })
                                .collect(),
                        })
                    })
                    .collect();

                if !radials.is_empty() {
                    let elevation_deg = sweep
                        .radials()
                        .first()
                        .map(|r| r.elevation_angle_degrees())
                        .unwrap_or(0.0);
                    out.push(SweepData { elevation_deg, radials });
                }
            }
        }

        sort_and_dedup(&mut reflectivity);
        sort_and_dedup(&mut velocity);

        ScanData { timestamp, reflectivity, velocity }
    }
}

/// Sort sweeps by elevation; split cuts produce near-duplicate elevations
/// (e.g. 0.48 and 0.52 deg) — keep only the first of each cluster.
fn sort_and_dedup(sweeps: &mut Vec<SweepData>) {
    sweeps.sort_by(|a, b| a.elevation_deg.total_cmp(&b.elevation_deg));
    sweeps.dedup_by(|current, previous| (current.elevation_deg - previous.elevation_deg).abs() < 0.2);
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test model`
Expected: 5 tests PASS.

- [ ] **Step 5: Lint and commit**

Run: `cargo clippy -- -D warnings` — expected: clean.

```bash
git add src/model.rs src/lib.rs
git commit -m "feat: add scan model with nexrad conversion and split-cut dedup

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 4: Sweep rasterizer

**Files:**
- Create: `src/scope.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: `model::{Product, SweepData}`, `colors::{dbz_color, velocity_color}`.
- Produces:
  - Constants: `scope::FIRST_GATE_KM: f32 = 2.125`, `scope::GATE_SPACING_KM: f32 = 0.25`, `scope::MAX_RANGE_KM: f32 = 230.0`, `scope::RASTER_SIZE_PX: usize = 1024`.
  - `scope::rasterize(sweep: &SweepData, product: Product, size_px: usize, max_range_km: f32) -> egui::ColorImage` — inverse-maps each pixel to (azimuth, range), picks the nearest radial by azimuth (with 0/360 wraparound) and the gate by range.
  - (internal, `pub(crate)` for tests) `nearest_radial_index(sorted_azimuths: &[f32], az: f32) -> usize`.
  - Task 7 will add overlay drawing to this file.

- [ ] **Step 1: Write failing tests**

Create `src/scope.rs` with the test module:

```rust
//! PPI scope rendering: rasterize a sweep into an RGBA image (radar at
//! center, north up) via inverse polar mapping, and draw overlays.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Product, RadialData, SweepData};
    use egui::Color32;

    /// Four cardinal radials with distinct dBZ so quadrants are testable.
    /// 200 gates x 0.25 km reach 2.125 + 50 = 52.125 km.
    fn synthetic_sweep() -> SweepData {
        let radial = |az: f32, dbz: f32| RadialData {
            azimuth_deg: az,
            gates: vec![Some(dbz); 200],
        };
        SweepData {
            elevation_deg: 0.5,
            radials: vec![
                radial(0.0, 7.0),    // north: light cyan 04e9e7
                radial(90.0, 22.0),  // east: green 02fd02
                radial(180.0, 52.0), // south: red fd0000
                radial(270.0, 2.0),  // west: below threshold -> transparent
            ],
        }
    }

    #[test]
    fn rasterizes_quadrants_to_expected_colors() {
        let img = rasterize(&synthetic_sweep(), Product::Reflectivity, 128, 40.0);
        assert_eq!(img.size, [128, 128]);
        // 40 km max range on 128 px -> center at (64, 64), 0.625 km/px.
        // 32 px from center = 20 km: inside gate coverage, outside first-gate hole.
        let at = |x: usize, y: usize| img.pixels[y * 128 + x];
        assert_eq!(at(64, 32), Color32::from_rgb(0x04, 0xe9, 0xe7), "north");
        assert_eq!(at(96, 64), Color32::from_rgb(0x02, 0xfd, 0x02), "east");
        assert_eq!(at(64, 96), Color32::from_rgb(0xfd, 0x00, 0x00), "south");
        assert_eq!(at(32, 64), Color32::TRANSPARENT, "west below threshold");
    }

    #[test]
    fn center_and_beyond_range_are_transparent() {
        let img = rasterize(&synthetic_sweep(), Product::Reflectivity, 128, 40.0);
        let at = |x: usize, y: usize| img.pixels[y * 128 + x];
        // Center: inside the first-gate hole.
        assert_eq!(at(64, 64), Color32::TRANSPARENT);
        // Corner: range > max_range_km.
        assert_eq!(at(0, 0), Color32::TRANSPARENT);
    }

    #[test]
    fn gates_beyond_radial_data_are_transparent() {
        // Sweep reaches 52 km; ask for a pixel at ~60 km with 80 km range.
        let img = rasterize(&synthetic_sweep(), Product::Reflectivity, 128, 80.0);
        // 48 px north of center = 48 * 1.25 = 60 km.
        assert_eq!(img.pixels[16 * 128 + 64], Color32::TRANSPARENT);
    }

    #[test]
    fn nearest_radial_wraps_around_north() {
        let azimuths = [10.0, 180.0, 350.0];
        assert_eq!(nearest_radial_index(&azimuths, 355.0), 2);
        assert_eq!(nearest_radial_index(&azimuths, 3.0), 0);
        assert_eq!(nearest_radial_index(&azimuths, 175.0), 1);
        assert_eq!(nearest_radial_index(&azimuths, 359.9), 2);
    }
}
```

Add `pub mod scope;` to `src/lib.rs`.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test scope`
Expected: FAIL to compile — `rasterize`, `nearest_radial_index` not found.

- [ ] **Step 3: Implement the rasterizer**

Add above the test module in `src/scope.rs`:

```rust
use crate::colors;
use crate::model::{Product, SweepData};
use egui::{Color32, ColorImage};

/// Level II super-res gate geometry.
pub const FIRST_GATE_KM: f32 = 2.125;
pub const GATE_SPACING_KM: f32 = 0.25;
/// Display radius of the scope.
pub const MAX_RANGE_KM: f32 = 230.0;
/// Side length of the rasterized radar texture.
pub const RASTER_SIZE_PX: usize = 1024;

/// Rasterize one sweep to a square RGBA image, radar at center, north up.
/// Each pixel is inverse-mapped to (azimuth, range); the nearest radial by
/// azimuth and nearest gate by range supply its value.
pub fn rasterize(sweep: &SweepData, product: Product, size_px: usize, max_range_km: f32) -> ColorImage {
    let mut pixels = vec![Color32::TRANSPARENT; size_px * size_px];
    if sweep.radials.is_empty() {
        return ColorImage::new([size_px, size_px], pixels);
    }

    // Radial order sorted by azimuth for nearest-neighbor lookup.
    let mut order: Vec<usize> = (0..sweep.radials.len()).collect();
    order.sort_by(|&a, &b| sweep.radials[a].azimuth_deg.total_cmp(&sweep.radials[b].azimuth_deg));
    let sorted_azimuths: Vec<f32> = order.iter().map(|&i| sweep.radials[i].azimuth_deg).collect();

    let color_of = match product {
        Product::Reflectivity => colors::dbz_color as fn(f32) -> Color32,
        Product::Velocity => colors::velocity_color as fn(f32) -> Color32,
    };

    let center = size_px as f32 / 2.0;
    let km_per_px = 2.0 * max_range_km / size_px as f32;

    for py in 0..size_px {
        let dy = (py as f32 + 0.5 - center) * km_per_px;
        for px in 0..size_px {
            let dx = (px as f32 + 0.5 - center) * km_per_px;
            let range_km = (dx * dx + dy * dy).sqrt();
            if !(FIRST_GATE_KM..=max_range_km).contains(&range_km) {
                continue;
            }

            // Screen +y is down; north (0 deg) points up, east 90 deg right.
            let azimuth = dx.atan2(-dy).to_degrees().rem_euclid(360.0);
            let radial = &sweep.radials[order[nearest_radial_index(&sorted_azimuths, azimuth)]];

            let gate = ((range_km - FIRST_GATE_KM) / GATE_SPACING_KM) as usize;
            if let Some(Some(value)) = radial.gates.get(gate) {
                pixels[py * size_px + px] = color_of(*value);
            }
        }
    }

    ColorImage::new([size_px, size_px], pixels)
}

/// Index into `sorted_azimuths` of the entry angularly nearest to `az`,
/// accounting for wraparound at 0/360.
pub(crate) fn nearest_radial_index(sorted_azimuths: &[f32], az: f32) -> usize {
    let n = sorted_azimuths.len();
    match sorted_azimuths.binary_search_by(|a| a.total_cmp(&az)) {
        Ok(i) => i,
        Err(i) => {
            let before = (i + n - 1) % n;
            let after = i % n;
            if angular_distance(sorted_azimuths[before], az) <= angular_distance(sorted_azimuths[after], az) {
                before
            } else {
                after
            }
        }
    }
}

fn angular_distance(a: f32, b: f32) -> f32 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test scope`
Expected: 4 tests PASS.

- [ ] **Step 5: Lint and commit**

Run: `cargo clippy -- -D warnings` — expected: clean.

```bash
git add src/scope.rs src/lib.rs
git commit -m "feat: add PPI sweep rasterizer with inverse polar mapping

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 5: Data worker (fetch, decode, poll loop)

**Files:**
- Create: `src/data.rs`
- Create: `tests/network.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: `model::ScanData::from_nexrad`, `nexrad_data::aws::archive::{list_files, download_file}`.
- Produces:
  - `data::SITE: &str = "KJGX"`, `data::POLL_INTERVAL: Duration` (120 s).
  - `data::WorkerMessage` — `enum WorkerMessage { NewScan(Box<ScanData>), Status(String), Error(String) }`.
  - `data::fetch_latest_scan(site: &str) -> anyhow::Result<ScanData>` (async; falls back to the previous UTC date near midnight; skips `_MDM` metadata files).
  - `data::spawn_worker(tx: std::sync::mpsc::Sender<WorkerMessage>, egui_ctx: egui::Context)` — spawns the polling thread; calls `egui_ctx.request_repaint()` after each send.

- [ ] **Step 1: Write the module with its unit test (backoff schedule) — test first**

Create `src/data.rs` containing only:

```rust
//! Background worker: polls the AWS archive bucket for the latest KJGX
//! volume, decodes it off the UI thread, and reports over an mpsc channel.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_doubles_and_caps() {
        assert_eq!(retry_delay(0), POLL_INTERVAL); // no errors -> normal poll
        assert_eq!(retry_delay(1), std::time::Duration::from_secs(30));
        assert_eq!(retry_delay(2), std::time::Duration::from_secs(60));
        assert_eq!(retry_delay(3), std::time::Duration::from_secs(120));
        assert_eq!(retry_delay(6), std::time::Duration::from_secs(600)); // capped
        assert_eq!(retry_delay(20), std::time::Duration::from_secs(600));
    }
}
```

Add `pub mod data;` to `src/lib.rs`.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test data`
Expected: FAIL to compile — `retry_delay`, `POLL_INTERVAL` not found.

- [ ] **Step 3: Implement the worker**

Add above the test module in `src/data.rs`:

```rust
use crate::model::ScanData;
use anyhow::{anyhow, Result};
use chrono::{Duration as ChronoDuration, Utc};
use nexrad_data::aws::archive::{download_file, list_files};
use std::sync::mpsc::Sender;
use std::time::Duration;

/// The NEXRAD site covering Macon, GA (Robins AFB).
pub const SITE: &str = "KJGX";
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

/// Fetch and decode the most recent volume scan for `site`. Checks today's
/// (UTC) prefix and falls back to yesterday's shortly after midnight UTC.
pub async fn fetch_latest_scan(site: &str) -> Result<ScanData> {
    let today = Utc::now().date_naive();
    let mut files = list_files(site, &today)
        .await
        .map_err(|e| anyhow!("listing volumes for {site} {today}: {e}"))?;

    if files.is_empty() {
        let yesterday = today - ChronoDuration::days(1);
        files = list_files(site, &yesterday)
            .await
            .map_err(|e| anyhow!("listing volumes for {site} {yesterday}: {e}"))?;
    }

    // `_MDM` objects are metadata, not volume data. Identifier is Ord by
    // name, and names embed the timestamp, so max() is the newest volume.
    let identifier = files
        .into_iter()
        .filter(|id| !id.name().ends_with("_MDM"))
        .max()
        .ok_or_else(|| anyhow!("no volume files found for {site}"))?;

    let timestamp = identifier
        .date_time()
        .ok_or_else(|| anyhow!("unparseable volume name: {}", identifier.name()))?;

    let file = download_file(identifier)
        .await
        .map_err(|e| anyhow!("downloading volume: {e}"))?;

    let scan = file.scan().map_err(|e| anyhow!("decoding volume: {e}"))?;

    Ok(ScanData::from_nexrad(&scan, timestamp))
}

/// Spawn the background polling thread. It owns a current-thread tokio
/// runtime; all communication with the UI is via `tx` + `request_repaint`.
pub fn spawn_worker(tx: Sender<WorkerMessage>, egui_ctx: egui::Context) {
    std::thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(rt) => rt,
            Err(e) => {
                let _ = tx.send(WorkerMessage::Error(format!("tokio runtime: {e}")));
                egui_ctx.request_repaint();
                return;
            }
        };

        let mut last_timestamp = None;
        let mut consecutive_errors: u32 = 0;

        loop {
            let _ = tx.send(WorkerMessage::Status(format!("Checking {SITE} for new data…")));
            egui_ctx.request_repaint();

            match runtime.block_on(fetch_latest_scan(SITE)) {
                Ok(scan) => {
                    consecutive_errors = 0;
                    if last_timestamp != Some(scan.timestamp) {
                        last_timestamp = Some(scan.timestamp);
                        let _ = tx.send(WorkerMessage::NewScan(Box::new(scan)));
                    } else {
                        let _ = tx.send(WorkerMessage::Status("Up to date".to_string()));
                    }
                }
                Err(e) => {
                    consecutive_errors += 1;
                    let _ = tx.send(WorkerMessage::Error(format!("{e:#}")));
                }
            }
            egui_ctx.request_repaint();

            std::thread::sleep(retry_delay(consecutive_errors));
        }
    });
}
```

- [ ] **Step 4: Run unit test to verify it passes**

Run: `cargo test data`
Expected: 1 test PASS (plus previous tests still green).

- [ ] **Step 5: Add the opt-in network integration test**

Create `tests/network.rs`:

```rust
//! Live-network test, excluded from normal runs. Execute explicitly with:
//! `cargo test --test network -- --ignored`

#[tokio::test]
#[ignore = "requires network access to the NEXRAD AWS archive"]
async fn fetches_and_decodes_latest_kjgx_volume() {
    let scan = rustywx::data::fetch_latest_scan(rustywx::data::SITE)
        .await
        .expect("fetch+decode should succeed");

    assert!(!scan.reflectivity.is_empty(), "volume should contain reflectivity sweeps");
    let sweep = &scan.reflectivity[0];
    assert!(sweep.radials.len() > 300, "a sweep should have hundreds of radials");

    // Sanity: decoded dBZ values fall in a plausible range.
    let values: Vec<f32> = sweep.radials.iter()
        .flat_map(|r| r.gates.iter().flatten().copied())
        .collect();
    assert!(values.iter().all(|v| (-35.0..=95.0).contains(v)),
        "dBZ range check failed");
}
```

- [ ] **Step 6: Run the network test once to prove the pipeline works end-to-end**

Run: `cargo test --test network -- --ignored --nocapture`
Expected: PASS in under ~60 s on a normal connection (downloads one ~5–15 MB volume). If the machine is offline, note it and move on — this test is opt-in.

- [ ] **Step 7: Lint and commit**

Run: `cargo clippy --all-targets -- -D warnings` — expected: clean.

```bash
git add src/data.rs src/lib.rs tests/network.rs
git commit -m "feat: add S3 polling worker with decode and error backoff

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 6: App shell — window, controls, radar texture

**Files:**
- Create: `src/app.rs`
- Modify: `src/lib.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: `data::{WorkerMessage, spawn_worker, SITE}`, `model::{Product, ScanData}`, `scope::{rasterize, RASTER_SIZE_PX, MAX_RANGE_KM}`.
- Produces: `app::RadarApp` with `pub fn new(rx: std::sync::mpsc::Receiver<WorkerMessage>) -> Self`, implementing `eframe::App`. Task 7 consumes `RadarApp`'s central-panel drawing spot by calling `scope::draw_scope(...)` there.

- [ ] **Step 1: Implement the app struct**

Create `src/app.rs`:

```rust
//! The eframe application: owns UI state, drains worker messages, and
//! re-rasterizes the radar texture only when scan/product/tilt changes.

use crate::data::{WorkerMessage, SITE};
use crate::model::{Product, ScanData};
use crate::scope;
use chrono::{DateTime, Utc};
use egui::{Color32, TextureHandle, TextureOptions};
use std::sync::mpsc::Receiver;

pub struct RadarApp {
    rx: Receiver<WorkerMessage>,
    scan: Option<ScanData>,
    product: Product,
    tilt_index: usize,
    status: String,
    texture: Option<TextureHandle>,
    texture_key: Option<(DateTime<Utc>, Product, usize)>,
}

impl RadarApp {
    pub fn new(rx: Receiver<WorkerMessage>) -> Self {
        Self {
            rx,
            scan: None,
            product: Product::Reflectivity,
            tilt_index: 0,
            status: format!("Starting up — fetching latest {SITE} volume…"),
            texture: None,
            texture_key: None,
        }
    }

    fn drain_messages(&mut self) {
        while let Ok(message) = self.rx.try_recv() {
            match message {
                WorkerMessage::NewScan(scan) => {
                    self.status = format!(
                        "Scan {} UTC ({} local)",
                        scan.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        scan.timestamp.with_timezone(&chrono::Local).format("%H:%M:%S")
                    );
                    self.scan = Some(*scan);
                }
                WorkerMessage::Status(text) => self.status = text,
                WorkerMessage::Error(text) => self.status = format!("Error: {text} — retrying"),
            }
        }
    }

    /// Clamp the tilt index to the sweeps available for the current product
    /// (velocity may have fewer tilts than reflectivity).
    fn clamped_tilt(&self) -> usize {
        let count = self.scan.as_ref().map_or(0, |s| s.sweeps(self.product).len());
        self.tilt_index.min(count.saturating_sub(1))
    }

    fn ensure_texture(&mut self, ctx: &egui::Context) {
        let Some(scan) = &self.scan else { return };
        let sweeps = scan.sweeps(self.product);
        if sweeps.is_empty() {
            self.texture = None;
            self.texture_key = None;
            return;
        }

        let tilt = self.clamped_tilt();
        let key = (scan.timestamp, self.product, tilt);
        if self.texture_key == Some(key) && self.texture.is_some() {
            return;
        }

        let image = scope::rasterize(&sweeps[tilt], self.product, scope::RASTER_SIZE_PX, scope::MAX_RANGE_KM);
        self.texture = Some(ctx.load_texture("radar-sweep", image, TextureOptions::LINEAR));
        self.texture_key = Some(key);
    }
}

impl eframe::App for RadarApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_messages();
        self.ensure_texture(ctx);

        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{SITE} — Macon, GA")).strong());
                ui.separator();
                ui.selectable_value(&mut self.product, Product::Reflectivity, Product::Reflectivity.label());
                ui.selectable_value(&mut self.product, Product::Velocity, Product::Velocity.label());
                ui.separator();

                if let Some(scan) = &self.scan {
                    let sweeps = scan.sweeps(self.product);
                    if !sweeps.is_empty() {
                        let current = self.clamped_tilt();
                        egui::ComboBox::from_label("Tilt")
                            .selected_text(format!("{:.1}°", sweeps[current].elevation_deg))
                            .show_ui(ui, |ui| {
                                for (i, sweep) in sweeps.iter().enumerate() {
                                    ui.selectable_value(&mut self.tilt_index, i, format!("{:.1}°", sweep.elevation_deg));
                                }
                            });
                    }
                }
            });
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.label(&self.status);
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::from_rgb(6, 9, 14)))
            .show(ctx, |ui| {
                scope::draw_scope(ui, self.texture.as_ref(), self.scan.as_ref(), self.product);
            });
    }
}
```

- [ ] **Step 2: Add a minimal `draw_scope` to `src/scope.rs`** (Task 7 replaces the body with full overlays)

Append to `src/scope.rs` (above the test module):

```rust
/// Draw the scope into the available space: the radar texture in a centered
/// square. Overlays are added in a later task.
pub fn draw_scope(
    ui: &mut egui::Ui,
    texture: Option<&egui::TextureHandle>,
    _scan: Option<&crate::model::ScanData>,
    _product: Product,
) {
    let available = ui.available_rect_before_wrap();
    let side = available.width().min(available.height());
    let rect = egui::Rect::from_center_size(available.center(), egui::vec2(side, side));
    let painter = ui.painter_at(available);

    if let Some(texture) = texture {
        painter.image(
            texture.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }
}
```

- [ ] **Step 3: Wire up `main.rs` and `lib.rs`**

Add `pub mod app;` to `src/lib.rs`.

Replace `src/main.rs`:

```rust
fn main() -> eframe::Result {
    let (tx, rx) = std::sync::mpsc::channel();

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
            Ok(Box::new(rustywx::app::RadarApp::new(rx)))
        }),
    )
}
```

- [ ] **Step 4: Verify it compiles and tests still pass**

Run: `cargo test && cargo clippy --all-targets -- -D warnings`
Expected: all tests PASS, clippy clean. (`egui::Frame::new()` is verified present in egui 0.35.)

- [ ] **Step 5: Run the app manually**

Run: `cargo run --release` (needs network)
Expected: window opens; status bar shows fetch progress, then a radar image appears within ~60 s. Product toggle and tilt selector work. Close the window to exit.

- [ ] **Step 6: Commit**

```bash
git add src/app.rs src/scope.rs src/main.rs src/lib.rs
git commit -m "feat: add eframe app shell with product/tilt controls and radar texture

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 7: Scope overlays, legend, and README

**Files:**
- Modify: `src/scope.rs` (replace `draw_scope` body)
- Create: `README.md`

**Interfaces:**
- Consumes: `geo::{range_bearing, polar_to_offset, CITIES, KJGX_LAT, KJGX_LON}`, `colors::{DBZ_LEGEND, VELOCITY_LEGEND}`, `model::ScanData`.
- Produces: final `draw_scope` (same signature as Task 6, `_scan`/`_product` params become used: `scan: Option<&ScanData>`, `product: Product`).

- [ ] **Step 1: Replace `draw_scope` with the full overlay version**

Replace the entire `draw_scope` function in `src/scope.rs` with:

```rust
/// Draw the scope: radar texture, range rings every 50 km, cardinal spokes,
/// city markers, station marker, scan time, and the product color legend.
pub fn draw_scope(
    ui: &mut egui::Ui,
    texture: Option<&egui::TextureHandle>,
    scan: Option<&crate::model::ScanData>,
    product: Product,
) {
    use crate::geo;
    use egui::{pos2, vec2, Align2, FontId, Rect, Stroke};

    let available = ui.available_rect_before_wrap();
    let side = available.width().min(available.height());
    let rect = Rect::from_center_size(available.center(), vec2(side, side));
    let center = rect.center();
    let px_per_km = (side / 2.0) / MAX_RANGE_KM;
    let painter = ui.painter_at(available);

    let grid = Color32::from_rgb(0x2a, 0x3a, 0x2f);
    let grid_text = Color32::from_rgb(0x5f, 0x8a, 0x6a);
    let text_font = FontId::monospace(12.0);

    if let Some(texture) = texture {
        painter.image(
            texture.id(),
            rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }

    // Range rings every 50 km, labeled along the north spoke.
    let mut ring_km = 50.0;
    while ring_km <= MAX_RANGE_KM {
        painter.circle_stroke(center, ring_km * px_per_km, Stroke::new(1.0, grid));
        painter.text(
            center + vec2(4.0, -ring_km * px_per_km),
            Align2::LEFT_BOTTOM,
            format!("{ring_km:.0} km"),
            text_font.clone(),
            grid_text,
        );
        ring_km += 50.0;
    }

    // Cardinal spokes and labels.
    for (azimuth, label) in [(0.0, "N"), (90.0, "E"), (180.0, "S"), (270.0, "W")] {
        let (dx, dy) = geo::polar_to_offset(azimuth, MAX_RANGE_KM, px_per_km);
        painter.line_segment([center, center + vec2(dx, dy)], Stroke::new(1.0, grid));
        let (lx, ly) = geo::polar_to_offset(azimuth, MAX_RANGE_KM * 0.96, px_per_km);
        painter.text(center + vec2(lx, ly), Align2::CENTER_CENTER, label, text_font.clone(), grid_text);
    }

    // Station marker at scope center.
    painter.circle_filled(center, 3.0, Color32::WHITE);
    painter.text(
        center + vec2(6.0, 6.0),
        Align2::LEFT_TOP,
        crate::data::SITE,
        text_font.clone(),
        Color32::WHITE,
    );

    // City markers.
    for &(name, lat, lon) in geo::CITIES {
        let (range_km, bearing_deg) = geo::range_bearing(geo::KJGX_LAT, geo::KJGX_LON, lat, lon);
        if range_km as f32 > MAX_RANGE_KM {
            continue;
        }
        let (dx, dy) = geo::polar_to_offset(bearing_deg as f32, range_km as f32, px_per_km);
        let position = center + vec2(dx, dy);
        painter.circle_stroke(position, 3.5, Stroke::new(1.5, Color32::from_rgb(0xdd, 0xdd, 0xaa)));
        painter.text(
            position + vec2(6.0, -6.0),
            Align2::LEFT_BOTTOM,
            name,
            text_font.clone(),
            Color32::from_rgb(0xdd, 0xdd, 0xaa),
        );
    }

    // Scan time, top-left of the panel.
    if let Some(scan) = scan {
        let utc = scan.timestamp.format("%Y-%m-%d %H:%M:%S UTC");
        let local = scan.timestamp.with_timezone(&chrono::Local).format("%H:%M:%S %Z");
        painter.text(
            available.left_top() + vec2(8.0, 8.0),
            Align2::LEFT_TOP,
            format!("{utc}\n{local}"),
            text_font.clone(),
            Color32::WHITE,
        );
    }

    // Color legend, bottom-left of the panel.
    let legend: &[(f32, Color32)] = match product {
        Product::Reflectivity => crate::colors::DBZ_LEGEND,
        Product::Velocity => crate::colors::VELOCITY_LEGEND,
    };
    let unit = match product {
        Product::Reflectivity => "dBZ",
        Product::Velocity => "m/s",
    };
    let swatch = vec2(18.0, 12.0);
    let legend_origin = available.left_bottom() + vec2(8.0, -(swatch.y + 22.0));
    for (i, &(threshold, color)) in legend.iter().enumerate() {
        let min = legend_origin + vec2(i as f32 * swatch.x, 0.0);
        painter.rect_filled(Rect::from_min_size(min, swatch), 0.0, color);
        if i % 2 == 0 {
            painter.text(
                min + vec2(0.0, swatch.y + 2.0),
                Align2::LEFT_TOP,
                format!("{threshold:.0}"),
                FontId::monospace(10.0),
                grid_text,
            );
        }
    }
    painter.text(
        legend_origin + vec2(legend.len() as f32 * swatch.x + 6.0, 0.0),
        Align2::LEFT_TOP,
        unit,
        text_font,
        grid_text,
    );
}
```

- [ ] **Step 2: Verify tests and lints**

Run: `cargo test && cargo clippy --all-targets -- -D warnings`
Expected: all tests PASS, clippy clean.

- [ ] **Step 3: Write `README.md`**

```markdown
# rustywx

A Rust desktop radarscope for the Macon, GA area. Fetches live NEXRAD
Level II data for KJGX (Robins AFB) from the public AWS archive bucket
(`unidata-nexrad-level2`), decodes it, and renders a classic PPI scope
with egui.

## Features

- Base reflectivity and base velocity products
- Elevation tilt selector
- Auto-refresh (polls for new volume scans every 2 minutes)
- Range rings, cardinal spokes, and city markers (Macon, Warner Robins)

## Run

    cargo run --release

Requires network access. No AWS credentials needed — the bucket is public.

## Test

    cargo test                                  # unit tests, no network
    cargo test --test network -- --ignored      # live end-to-end fetch/decode

## Architecture

- `src/data.rs` — background worker: poll S3 → download → decode → channel
- `src/model.rs` — thin scan model (product → sweeps → radials → gates)
- `src/scope.rs` — PPI rasterizer and overlay painting
- `src/colors.rs` — NWS-style color tables
- `src/geo.rs` — range/bearing and polar→screen projection
- `src/app.rs` — egui application state and layout

Design docs live in `docs/superpowers/`.
```

- [ ] **Step 4: Run the app and verify visually**

Run: `cargo run --release`
Expected: scope shows radar echoes (if any weather) with rings every 50 km, N/E/S/W labels, Macon and Warner Robins markers NW/W of center, scan time top-left, legend bottom-left. Toggle products and tilts; status bar updates.

Take a screenshot of the window (e.g. `gnome-screenshot -w -f screenshots/scope.png` or the desktop's screenshot tool), save under `screenshots/` and add `screenshots/` to `.gitignore`.

- [ ] **Step 5: Commit**

```bash
git add src/scope.rs README.md .gitignore
git commit -m "feat: add scope overlays, legend, and README

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Final acceptance checklist

- [ ] `cargo test` green, `cargo clippy --all-targets -- -D warnings` clean, `cargo fmt --check` clean.
- [ ] `cargo test --test network -- --ignored` passes on a networked machine.
- [ ] `cargo run --release` shows live KJGX data; auto-refresh updates the scan time within ~5 minutes; killing the network mid-run degrades to an error status without crashing.
