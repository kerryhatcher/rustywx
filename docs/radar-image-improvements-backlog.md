# Radar image improvements — backlog

Derived from a review of `docs/research/` (federal handbooks FMH-11 B/C/D, dual-pol QC
papers, velocity-dehazing papers, volumetric-visualization theses). Techniques already
in the pipeline (bilinear azimuth+range interpolation, CC-gating, TDBZ texture filter,
`remove_small_regions`, `morphological_close`, Catmull-Rom color splines, super-res gate
geometry) are excluded.

The **easy tier** — range-folded rendering, reflectivity noise-floor cut, velocity spatial
SD censoring — is tracked separately in `docs/radar-easy-tier-plan.md` and is being
implemented. This file documents everything deferred beyond that.

Difficulty is relative effort, not value. Ordering within each tier is roughly by
bang-for-buck.

---

## Confirmed correctness issue (do before new features)

### Per-tilt / per-product gate geometry
- **Problem:** `app/src/scope.rs:16-17` hardcode `FIRST_GATE_KM = 2.125` and
  `GATE_SPACING_KM = 0.25` and apply them to the whole volume and both products.
- **Reality (FMH-11C §4.2.6):** super-res 0.25 km / 0.5° azimuth applies **only** to
  split-cut low tilts. Upper tilts are legacy 1 km reflectivity / 1° azimuth. Reflectivity
  and velocity moment blocks carry **different** first-gate range and gate spacing.
- **Fix:** read geometry per Message 31 moment block. `nexrad_model 1.0.0-rc.2` already
  exposes `first_gate_range_km()` and `gate_interval_km` per field — the app currently
  ignores them. Thread real per-sweep/per-product geometry through `model.rs` into
  `scope.rs::rasterize` instead of the module constants.
- **Symptom today:** upper tilts placed at the wrong range; possible reflectivity/velocity
  misregistration.
- **Difficulty:** medium. **Latent bug, not cosmetic.**

---

## Reflectivity QC (medium)

### Multi-scale texture windows
- Run the texture statistic at 2-3 window sizes instead of one. Small windows preserve
  weak storm-edge echo; large windows kill spatially-coherent clutter (wind turbines) that
  single-scale misses. RF paper shows out-of-bag accuracy climbing monotonically as scales
  are added.
- Reuses existing TDBZ windowing machinery.
- **Difficulty:** medium.

### Persistent / statistical clutter map
- Accumulate a static ground-clutter map over many volumes, then subtract per-bin:
  `Nr = Ns + 10·log10(1 − 10^((Ng − Ns)/10))` — strong precip passes, weak stationary
  clutter is cut. A statistical variant flags bins that are "strong but rare" or "weak but
  persistent" as clutter. Kills fixed clutter (towers, terrain) that CC/texture can't
  because it is temporally stationary, not spatially noisy.
- Needs a multi-scan accumulation store.
- **Difficulty:** hard.

### Sun-spike / RFI radial removal
- Detect and null narrow high-value spikes confined to one or few azimuths spanning many
  range gates (solar noise, co-channel interference). Removes thin radial "spoke" artifacts.
- **Difficulty:** medium.

---

## Dual-pol non-meteorological rejection (medium → hard)

Cheap hand-tuned gates first; the full classifier is the ceiling.

### Fuzzy / multi-variable non-met classifier
- Combine CC + ZDR + REF-texture (SD of ZDR/ΦDP) into a membership score instead of the
  current single-threshold CC gate. FMH-11A notes ZDR and ΦDP each independently flag
  non-met targets. Fewer false nulls on real precip, better biology/chaff/AP rejection.
- **Difficulty:** medium.

### Random Forest P/NP classifier
- All four moments + their multi-scale variances → >99.9% precip/non-precip accuracy in the
  source paper. This is the paper's headline but a big lift (training data, model, inference
  in Rust). The hand-tuned threshold gates in the easy tier + fuzzy classifier above capture
  most of the benefit cheaply.
- **Difficulty:** hard. Likely not worth it over the fuzzy classifier.

---

## Velocity product (hard — blocked on data source)

### Velocity dealiasing
- Unfold radial velocities beyond the Nyquist interval — the biggest single visible fix for
  the velocity product (currently non-dealiased "purple haze"; strong inbound/outbound
  couplets read as garbage).
- **Blocker:** `nexrad_model 1.0.0-rc.2` exposes **no** Nyquist-velocity or PRT field
  (verified). Nyquist must be computed from VCP/PRT metadata (`va = λ/(4·PRT)`,
  `ra·va = c·λ/8`, WSR-88D λ ≈ 0.1071 m) or by parsing raw Message 31 — a data-source task
  that must land first.
- **Algorithm once Nyquist is available:**
  1. Gate-to-gate radial unfold (FMH-11B §4.3.3): walk each radial outward; where the
     gate-to-gate difference approaches ±2·va, add/subtract 2·va to minimize the jump.
  2. Upgrade to 2D continuity (FMH-11D §3.3.3): also check azimuthal continuity against the
     previous dealiased radial; **allow large jumps over short distances** so mesocyclone /
     TVS couplets are NOT flattened; isolated suspects fall back to a VAD environmental-wind
     check; reject-and-reinsert after 5 consecutive failures; stop azimuthal propagation
     after 5 bad radials.
- **Difficulty:** hard (plus prerequisite data work).

---

## New derivable products

### KDP (specific differential phase)
- Derive from the ΦDP the app already parses (FMH-11C §2.5.3): unwrap total differential
  phase → median filter → average filter → interpolate across meteorological gates along the
  radial → range-derivative over a limited increment. Units °/km. Attenuation- and
  calibration-immune rain-rate signal.
- **Difficulty:** medium.

### Melting-layer hint
- Circular band of depressed CC at mid/high elevation angles. Cheap annotation on its own,
  and a prerequisite for HCA.
- **Difficulty:** medium.

### Hydrometeor Classification (HCA)
- Fuzzy-logic membership over Z, ZDR, CC, KDP + texture (SD of Z/ΦDP), gated by
  melting-layer proximity (FMH-11C §2.8.1). Produces 10 classes: Biological, Ground Clutter,
  Ice Crystals, Dry Snow, Wet Snow, Rain, Heavy Rain, Big Drops, Graupel, Rain+Hail
  (+Unknown). Even a partial version gives a far better non-met mask (GC/BI) than CC-gating
  alone, plus a hail-region layer.
- Needs membership beam tables + a melting-layer estimate (so KDP + melting-layer hint
  should land first).
- **Difficulty:** hard.

---

## Considered and rejected (out of scope for a single-site Level II 2D viewer)

- **GMAP clutter filter** (FMH-11C §4.2.4) — Gaussian-model adaptive filter applied at the
  RDA on time-series before moments are written. Cannot reproduce or improve from Level II
  moment data; already baked in.
- **Dual-PRF / MPDA** (FMH-11D §3.3.3) — requires multi-PRF scans (VCP 31/121) absent from
  standard VCPs 12/212.
- **SZ(8/64) regression filtering, ADVANCE, VRAD long-PRT infill, REG-VRAD** — need Level-1
  time series or the long-PRT surveillance scan; neither exists in Level II.
- **3D volume rendering, multi-site mosaic, spherical→geographic reprojection, temporal
  interpolation between volumes, vertical Z-gap infill** — serve a volumetric / multi-radar
  renderer, not the azimuthal-equidistant 2D raster.
- **Higher-order interpolation** — Kvasov et al.: bilinear is visually indistinguishable
  from higher-order once ≥16 sub-samples/gate are used; only a far-range sampling-density
  check remains (minor).
- **GOES-16 IR brightness-temperature fusion** — actionable but needs external satellite
  ingest/collocation; heavy for marginal gain over radar-only NP removal.

---

## Source map

| Technique | Source |
|---|---|
| Per-tilt geometry, TOVER=5 dB, range-averaging | FMH-11C §4.2.6, §4.2.8 |
| Velocity dealiasing (gate-to-gate) | FMH-11B §4.3.3 |
| Velocity dealiasing (2D continuity, operational) | FMH-11D §3.3.3 |
| KDP derivation | FMH-11C §2.5.3 |
| HCA fuzzy classification | FMH-11C §2.8.1 |
| Dual-pol texture / ZDR / ΦDP discriminators | `remotesensing-18-00827`, FMH-11A |
| Velocity SD censoring / median despeckle | `atot-JTECH-D-25-0059.1` |
| Noise floor, clutter-map subtraction, sun-spike, hybrid-scan | `2-5_Hands-on_training_on_weather_radar_QC` |
| Sample-density floor, dBZ clamp | Volumetric-viz thesis, first-order-interp paper |
