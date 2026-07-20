# Post-v1: Multi-Site Integration & Temporal Animation

**Status:** 📝 Planned (post-v1.0.0)
**Source:** Yi Ru (2007), *Volumetric Visualization of NEXRAD Level II Doppler Weather Data from Multiple Sites* — see `docs/Volumetric_Visualization_Of_NEXRAD_Level_II_Doppler_WeatherVolumetric_Visualization_Of_NEXRAD_Level_II_Doppler_Weather.md` for the full thesis summary.

---

## Overview

Two related features deferred from v1:

1. **Multi-site integration** — combining data from multiple NEXRAD radar sites into a single unified view, giving forecasters a wide-area picture without switching between sites.
2. **Temporal animation** — playing back a sequence of historical volume scans as a smooth animation, showing storm evolution over time.

Both are listed as "out of scope for v1" in the radarscope design spec (`docs/superpowers/specs/2026-07-13-rustywx-radarscope-design.md`). This document captures what we know from the thesis and from our own codebase to inform future implementation.

---

## 1. Multi-Site Integration

### The Problem

- NEXRAD radars at different locations operate at different tempos — data from one site is collected at a time and rate different from another site.
- Each site stores data in its own local spherical coordinate system (azimuth, elevation, range) with the origin at the radar antenna.
- Overlapping coverage between adjacent radars produces duplicate data in overlapping regions.
- The thesis integrates three sites (KEAX, KILX, KSLX) into a single volume covering the Midwest US.

### Thesis Approach

**Step 1 — Coordinate conversion (spherical → geographic):**

Given a radar site at `(lon, lat, alt)` and a sample point at local spherical coordinates `(r, θ, φ)`:

```
lon' = lon + O'A'·cos(φ) / lonScale
lat' = lat + O'A'·sin(φ) / latScale
alt' = OA - Re
```

Where `Re ≈ 6,367,450 m`, `latScale = Re·π/180`, `lonScale = Re·cos(lat)·π/180`.

`OA` and `O'A'` account for the Earth's curvature and radar beam height:

```
OA = sqrt((Re + alt)² + r² + 2r·cosθ·(Re + alt))
O'A' = (Re + alt) · arcsin(r·sinθ / OA)
```

Cell indices in the target 3D grid are then:

```
index_x = (lon' - bbox_min_x) / bbox_x
index_y = (lat' - bbox_min_y) / bbox_y
index_z = (alt' - bbox_min_z) / bbox_z
```

**Step 2 — Temporal synchronization (see §1.2 below):**

Linear interpolation between timestamps to align asynchronous sites to a common time grid.

**Step 3 — Overlap handling:**

- In cells where multiple sites contribute samples, values are averaged.
- Vertical interpolation fills gaps where no sample landed in a cell.
- Empty cells below a known value are filled with that value (clamp downward).

**Step 4 — Grid structure:**

The thesis uses a 256×256×128 semi-regular grid (128 layers of 256×256 XY planes). More data points are distributed in the XY plane than in the Z direction, so the grid is non-uniform.

### What We Already Have

Our `geo.rs` already provides the building blocks:

- `range_bearing(lat1, lon1, lat2, lon2) → (range_km, bearing_deg)` — great-circle distance and bearing between two lat/lon points.
- `polar_to_offset(bearing_deg, range_km, px_per_km) → (dx, dy)` — converts polar coordinates to pixel offsets for scope rendering.
- `RADAR_SITES` — a const table of 143 NEXRAD sites with lat/lon coordinates.

### What We'd Need to Build

| Component | Description | Dependencies |
|---|---|---|
| `geo::spherical_to_geographic()` | Convert local spherical coords to global geographic coords | `geo.rs` constants, Earth radius |
| `geo::geographic_to_grid_index()` | Map geographic coords into a unified grid cell | Bounding box definition |
| `model::UnifiedVolume` | A 3D grid structure holding merged reflectivity/velocity data | New type in `model.rs` |
| `data::multi_site_worker()` | Background thread that fetches from N S3 sites, synchronizes timestamps, merges | `data.rs` pattern, `nexrad-data` |
| `scope::rasterize_volume()` | Rasterize a 3D volume slice (CAPPI at a given altitude) rather than a single sweep | `scope.rs` rasterization pipeline |
| Altitude selector UI | Replace tilt selector with altitude selector for the merged view | Widgets from Stage 3 |

### Design Questions for Later

- **Grid extent:** How large a bounding box? The thesis covers ~600 km between KEAX and KILX. For a national view, we'd need a much larger grid or a tiled approach.
- **Grid resolution:** The thesis uses 256×256×128. Higher resolution captures more detail but costs more memory and processing time. The thesis's performance analysis (Table 4.3) shows the trade-offs.
- **Real-time vs. pre-processed:** The thesis does data processing offline, then renders in real-time. For a live app, we'd need to decide whether to process on-the-fly or cache pre-merged volumes.
- **CAPPI vs. sweep display:** A multi-site merged view naturally lends itself to Constant Altitude PPI (CAPPI) display — showing a horizontal slice at a fixed altitude rather than a single radar sweep. This is a different visualization paradigm from the current PPI scope.

---

## 2. Temporal Animation

### The Problem

- NEXRAD produces a new volume scan every ~5–6 minutes.
- Displaying a sequence of these scans as an animation reveals storm motion, development, and decay.
- The thesis stores 3D textures on disk at 5-minute intervals and plays them back sequentially.
- For a 24-hour period, this means ~288 frames.

### Thesis Approach

**Temporal interpolation (Eq. 3.4):**

For each time step `t`, find the two radar product files at times `ta` and `tb` immediately before and after `t`. The sample value `vt` at time `t` is:

```
vt = vta · (1 - (t - ta)/(tb - ta)) + vtb · ((t - ta)/(tb - ta))
```

This is done per-site to synchronize data from multiple radars to a common time grid.

**Compression for storage:**

The thesis's modified RLE algorithm compresses each volume to ~0.66% of its original size (8 MB → 54 KB for 256×256×128). This makes storing hundreds of frames feasible — 288 frames at 54 KB each is only ~15 MB.

**Playback:**

The program reads sequential texture files on a continuous basis. An adjustable parameter controls the interval between frames. Rendering time depends on both CPU and GPU speed.

### What We Already Have

- `cache.rs` — already caches radar data locally using Ply storage.
- `data.rs` — background worker that fetches and decodes NEXRAD volumes.
- `model.rs` — `ScanData` with timestamp, sweeps per product.
- The thesis's RLE compression algorithm is simple enough to implement in a day.

### What We'd Need to Build

| Component | Description | Dependencies |
|---|---|---|
| `cache::compressed_store()` | RLE-compressed cache format for historical volumes | `cache.rs`, RLE encode/decode |
| `data::historical_fetcher()` | Fetch not just the latest volume but a range of historical volumes | `data.rs`, S3 listing by date |
| `state::animation_state` | Play/pause, speed control, frame scrubber, loop toggle | `state.rs` |
| `scope::animation_overlay()` | Timeline indicator, current frame counter, playback controls | `scope.rs` overlays |
| Animation controls UI | Play/pause button, speed slider, scrub bar | Widgets from Stage 3 |

### Design Questions for Later

- **Source of historical data:** The AWS S3 archive bucket (`unidata-nexrad-level2`) retains data for ~30 days. For longer historical archives, we'd need the NOAA Big Data Project or another source.
- **Pre-fetch vs. on-demand:** Should we download all frames for the animation window upfront, or stream them on-demand? The thesis pre-processes everything. For a live app, streaming would be more responsive but adds complexity.
- **Interpolation quality:** Linear interpolation (the thesis's approach) is fast but can miss rapidly changing features. Higher-order interpolation or feature-tracking could improve quality at the cost of complexity.
- **Memory budget:** 288 frames of uncompressed 256×256×128 volume data = 288 × 8 MB = 2.3 GB. Compression brings this to ~15 MB. Even with compression, decompression speed matters for smooth playback.

---

## 3. Synergies Between the Two Features

Multi-site integration and temporal animation are naturally complementary:

- **Multi-site + animation** = a wide-area time-lapse of storm systems moving across regions.
- The thesis already combines them — it integrates 3 sites into a volume at each timestamp, then plays back the sequence as an animation.
- The temporal interpolation step (§1.2 in the thesis) is required for multi-site synchronization anyway — it's the same interpolation used for animation frames.
- The RLE compression that makes animation storage feasible also helps with multi-site data: each merged volume is compressed individually.

---

## 4. References

- Ru, Y. (2007). *Volumetric Visualization of NEXRAD Level II Doppler Weather Data from Multiple Sites.* Purdue University. — Full summary in `docs/Volumetric_Visualization_Of_NEXRAD_Level_II_Doppler_WeatherVolumetric_Visualization_Of_NEXRAD_Level_II_Doppler_Weather.md`
- Radarscope design spec: `docs/superpowers/specs/2026-07-13-rustywx-radarscope-design.md`
- NEXRAD data analysis: `research/nexrad-data-analysis.md`
- Ply port plan: `docs/ply-port-plan.md`
