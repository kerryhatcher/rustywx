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

- Ru, Y. (2007). *Volumetric Visualization of NEXRAD Level II Doppler Weather Data from Multiple Sites.* Purdue University. — Full markdown in `docs/research/Volumetric_Visualization_Of_NEXRAD_Level_II_Doppler_WeatherVolumetric_Visualization_Of_NEXRAD_Level_II_Doppler_Weather.md`
- Kvasov, R. et al. *Weather Radar Data Visualization Using First-Order Interpolation.* — Bilinear interpolation validation. Full markdown in `docs/research/Weatherradardatavisualizationusingfirst-orderinterpolation.md`
- Hubbert, J.C. et al. (2026). *Improving NEXRAD Velocity Retrievals Using Multi-PRT Scans with Regression Processing.* J. Atmos. Ocean. Technol. — REG-VRAD velocity recovery. Full markdown in `docs/research/atot-JTECH-D-25-0059.1.md`
- Keem, M. et al. (2026). *Data-Driven Non-Precipitation Echo Removal of NEXRAD Radars Based on a Random Forest Classifier.* Remote Sensing, 18, 827. — ρHV-based QC, multi-scale TDBZ. Full markdown in `docs/research/remotesensing-18-00827-with-cover.md`
- FMH-11 Part A (2021). *System Concepts, Responsibilities, and Procedures.* FCM-H11A-2021. — VCPs, operational modes, data types. Full markdown in `docs/research/2021_fmh11_parta.md`
- FMH-11 Part B (2005). *Doppler Radar Theory and Meteorology.* FCM-H11B-2005. — Radar equation, Doppler effect, signal processing, velocity patterns. Full markdown in `docs/research/fmh-11B-2005.md`
- FMH-11 Part C (2017). *WSR-88D Products and Algorithms.* FCM-H11C-2017. — Full product suite, algorithm descriptions, VCP details. Full markdown in `docs/research/fmh11partC.md`
- FMH-11 Part D (2006). *WSR-88D Unit Description and Operational Applications.* FCM-H11D-2006. — System components, data flow, operational applications. Full markdown in `docs/research/FMH11D-2006.md`
- Hotta, J. (2018). *Hands-on Training on Weather Radar QC.* JMA / WMO-ASEAN Workshop. — PCAPPI process, clutter map, statistical QC. Full markdown in `docs/research/2-5_Hands-on_training_on_weather_radar_QC.md`
- Radarscope design spec: `docs/superpowers/specs/2026-07-13-rustywx-radarscope-design.md`
- NEXRAD data analysis: `research/nexrad-data-analysis.md`
- Ply port plan: `docs/ply-port-plan.md`

---

## 5. Dual-Polarization Product Suite

**Status:** 📝 Planned (post-v1.0.0)
**Source:** FMH-11 Part C (product definitions), Keem et al. (QC applications)

### The Opportunity

Currently rustywx displays only Reflectivity and Velocity. NEXRAD Level II
data includes three dual-polarization variables that are already decoded by
`nexrad-model` but not exposed in the `Product` enum:

| Product | Symbol | What it shows | Why it matters |
|---|---|---|---|
| Differential Reflectivity | ZDR | Drop shape/size | Hail detection, rain vs snow |
| Correlation Coefficient | ρHV | Target homogeneity | **Best clutter discriminator** (ρHV > 0.9 = precipitation) |
| Specific Differential Phase | KDP | Rainwater content | Rain rate, immune to attenuation |

### Implementation

Each product needs:
- A `Product::ZDR`, `Product::CorrelationCoefficient`, `Product::KDP` enum variant
- A color table in `colors.rs` (NWS standard tables exist for each)
- Wire into `model.rs` `from_sweeps()` to decode the moment
- Add to the product toggle in the UI

### Impact

Adding ρHV alone would transform rustywx's clutter filtering. Instead of the
current heuristic (range-adaptive dBZ floor + TDBZ texture filter), users could
visually identify clutter: precipitation shows ρHV > 0.9 (warm colors),
non-meteorological targets show ρHV < 0.85 (cool colors). Keem et al.
demonstrated >99.98% classification accuracy using ρHV features.

---

## 6. ρHV-Based Clutter Filtering

**Status:** 📝 Planned (post-v1.0.0, depends on §5)
**Source:** Keem et al. (2026)

### The Opportunity

Once ρHV is available as a decoded product (§5), replace the current
`clean_sweep()` heuristic in `scope.rs` with ρHV-gated quality control:

- **Gate out** any pixel where ρHV < 0.85 (eliminates ground clutter, AP,
  biological targets, wind turbine interference)
- **Keep** any pixel where ρHV > 0.90 (reliable precipitation)
- **Marginal zone** (0.85–0.90): apply the existing TDBZ texture filter as
  a secondary check

This is far more robust than the current range-zone approach, which can
remove weak precipitation or fail to catch strong AP. The Random Forest
paper achieved >99.98% accuracy with ρHV features alone — a simple threshold
approach should capture most of that benefit.

See `docs/research/remotesensing-18-00827-with-cover.md` for the full paper,
including Table 2 with descriptive statistics of ρHV for P vs NP echoes.

---

## 7. CAPPI Composite Display

**Status:** 📝 Planned (post-v1.0.0)
**Source:** JMA Training (Hotta 2018), Keem et al. (2026)

### The Opportunity

Currently rustywx shows a single elevation tilt at a time. A CAPPI
(Constant Altitude Plan Position Indicator) composites multiple elevations
into a single view at a fixed altitude (typically ~2 km):

- Eliminates the "cone of silence" near the radar where the lowest elevation
  overshoots
- Removes ground clutter by using higher elevations near the radar
- Provides a consistent altitude reference for comparing storms at different
  ranges

### JMA Approach

The JMA training document describes the elevation angle composite table:
per-radar, per-direction parameters that select the optimal elevation angle
to stay near 2 km altitude while avoiding terrain blockage. See
`docs/research/2-5_Hands-on_training_on_weather_radar_QC.md`.

### Research Validation

Keem et al. confirm that a CAPPI scan strategy "improves near-radar
precipitation detection by recovering valid echoes misclassified at the
lowest elevation due to side-lobe interference and limited sampling volume."
Their RF-GOES CAPPI model showed the greatest reduction in variability and
the most consistent radial profile. See
`docs/research/remotesensing-18-00827-with-cover.md` §3.5.

---

## 8. Velocity Recovery (Purple Haze Mitigation)

**Status:** 📝 Planned (post-v1.0.0)
**Source:** Hubbert et al. (2026)

### The Problem

Currently rustywx renders range-folded velocities as transparent (None
gates). These are the "purple haze" regions — velocities that cannot be
resolved by SZ(8/64) phase coding processing. Hubbert et al. explain that
this occurs where overlaid echoes from different range trips cannot be
sufficiently separated.

### REG-VRAD Approach

The paper combines two techniques:
1. **VRAD** — uses long-PRT surveillance scan velocities (aliased but
   present) to fill gaps in the short-PRT Doppler scan, with dealiasing
2. **Regression processing (RFSF)** — replaces legacy SZ(8/64) filtering
   with a regression filter that recovers more WT velocities

Results: 25–30% more velocity data points recovered, with lower spatial
standard deviation. The KLWX case revealed a tornado signature obscured by
purple haze.

### Simplified Level-2 Approach

Full REG-VRAD requires Level-1 (I/Q) data, but a simplified approach using
only Level-2 data could:
- Use the long-PRT surveillance scan's velocity (available in the split-cut
  structure rustywx already decodes) to fill range-folded regions
- Apply UNRAVEL-style spatial continuity dealiasing (Louf et al. 2020)
- Show recovered velocities in a distinct color so users know they are
  dealiased estimates, not direct measurements

See `docs/research/atot-JTECH-D-25-0059.1.md` for the full paper including
the 6-step REG-VRAD algorithm and censoring strategy.

---

## 9. Persistent Clutter Map

**Status:** 📝 Planned (post-v1.0.0)
**Source:** JMA Training (Hotta 2018)

### The Opportunity

The JMA training document describes a clutter map approach where persistent
clutter locations (buildings, towers, terrain) are pre-computed from
accumulated statistics and subtracted from real-time data. This is more
accurate than per-scan thresholding because:
- Clutter locations are persistent (same buildings/towers every scan)
- Statistics accumulated over time distinguish persistent clutter from
  transient weather echoes
- Allows lowering the real-time threshold, preserving more weak precipitation

### JMA Statistical QC

JMA uses two statistical measures:
- **Appearance count** — how often a gate shows echo > 1 mm/h over a month.
  Weak but continuous clutter has high appearance count but low average.
- **Summation/Average** — total precipitation estimate. Strong clutter has
  high average but low appearance count.

Combining these two identifies suspicious gates that are likely clutter.

See `docs/research/2-5_Hands-on_training_on_weather_radar_QC.md` for
the full methodology including the clutter map subtraction formula.

---

## 10. Interactive Transfer Functions

**Status:** 📝 Planned (post-v1.0.0)
**Source:** Yi Ru (2007) thesis, §3.2.4

### The Opportunity

Stage 6 introduces spline-based color tables for smoother gradients. The
thesis goes further with interactive transfer functions — user-adjustable
color/opacity mapping via cubic Hermite spline controls. This lets users:

- Highlight specific dBZ ranges (e.g., focus on 50+ dBZ for hail)
- Adjust opacity for overlay compositing
- Create custom color schemes for different analysis tasks

The thesis uses a transfer function window with RGB and alpha channel spline
controls. The cubic Hermite spline (Cardinal Spline) is defined by:

```
P(t) = (t³, t², t, 1) · MH · (P1, P2, P1', P2')ᵀ
```

where MH is the Hermite basis matrix. See
`docs/research/Volumetric_Visualization_Of_NEXRAD_Level_II_Doppler_WeatherVolumetric_Visualization_Of_NEXRAD_Level_II_Doppler_Weather.md`
§3.2.4 for the full implementation.
