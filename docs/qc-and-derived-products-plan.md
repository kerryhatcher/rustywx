# QC refinements + derived products — plan

Covers the next four backlog items from `docs/radar-image-improvements-backlog.md`,
ordered by bang-for-buck and dependency:

1. **Multi-scale texture windows** (Reflectivity QC) — cheapest, extends existing TDBZ pass.
2. **Sun-spike / RFI radial removal** (Reflectivity QC) — self-contained new pass.
3. **KDP** (new derived product) — new product surface; unlocks HCA later.
4. **Melting-layer hint** (annotation overlay) — uses CC; prereq for HCA with KDP.

Stages 1–2 are Reflectivity QC passes inside `clean_sweep` (`scope.rs:63`), driven by
new `QcConfig`/`Settings` flags following the established `cc_gate`/`refl_gap_fill`
pattern. Stages 3–4 add product/overlay surface. Ship as **four separate PRs** — each
is independently useful and independently revertable. All four default **off** except
where noted (honest rendering by default; opt-in QC is the house style).

Shared conventions already in the tree (reuse, do not reinvent):
- Membership ramp + `range_texture` gate-to-gate/azimuth machinery: `nonmet.rs`.
- Settings knob: `bool` with `#[serde(default = "default_true")]` (or bare `#[serde(default)]`
  for opt-in false) + a flat threshold with a `default_*` fn. `settings.rs:87-108, 180-223`.
- QcConfig field + main.rs wiring at `scope.rs:46` / `main.rs:1043`.
- Every non-trivial pass leaves at least one `assert`-based unit test in the module
  (see the `#[test]` blocks in `scope.rs`/`nonmet.rs`).

---

## Stage 1 — Multi-scale texture windows

**Backlog:** Reflectivity QC (medium). Run the texture statistic at 2–3 window sizes
instead of one; small windows preserve weak storm-edge echo, large windows kill
spatially-coherent clutter (wind turbines) single-scale misses.

**Where:** the TDBZ block in `clean_sweep`, `scope.rs:236-285`. Today it computes one
`tdbz[]`/`mean[]` at `half = qc.tdbz_kernel_size / 2` and nulls where
`tdbz[i] > 25.0 && mean[i] < 35.0`.

**Approach:**
- Compute TDBZ at N kernel sizes (default two: the configured `tdbz_kernel_size` and a
  larger one, e.g. `size` and `size + 6`). Null a gate if **any** scale exceeds
  `TDBZ_THRESHOLD` while its `mean` is still `< LOW_DBZ_GATE`. Union of masks — a clutter
  bin flagged at any scale is clutter.
- Factor the existing inner loop (lines 250-278) into a small
  `fn tdbz_and_mean(gates: &[Option<f32>], half: usize) -> (Vec<f32>, Vec<f32>)` so the
  multi-scale loop just calls it per half-window. Mechanical extract, same math.
- **Reuse note:** this is the *intensity* TDBZ (variance of raw dBZ), distinct from
  `nonmet::range_texture` (RMS, dual-pol). Keep them separate; do not merge.

**Config:** add `multi_scale_texture_enabled: bool` (default `false` — it's more
aggressive). When off, behavior is byte-identical to today (single scale). No new
threshold needed initially; the extra scale reuses `TDBZ_THRESHOLD`/`LOW_DBZ_GATE`.
Optional later knob: explicit `Vec` of kernel sizes.

**Touch:** `scope.rs` (extract + loop), `QcConfig` (+1 bool), `settings.rs` (+1 bool,
default fn not needed), `main.rs` (wire the flag), settings UI toggle in `widgets/`.

**Test:** a synthetic radial with coherent low-dBZ clutter that a size-3 window *keeps*
but a size-9 window *nulls* — assert the union pass nulls it, single-scale doesn't.
Extends the existing `tdbz_kernel_size_widens_clutter_removal_footprint` test
(`scope.rs:1887`).

**Difficulty:** medium (mostly the extract; low risk, off by default).

---

## Stage 2 — Sun-spike / RFI radial removal

**Backlog:** Reflectivity QC (medium). Detect and null narrow high-value spikes confined
to one or a few azimuths but spanning many range gates (solar noise at dawn/dusk,
co-channel interference) — the thin radial "spoke" artifacts.

**Where:** new pass in `clean_sweep`, Reflectivity-only, after the dBZ floor but before
gap-fill (so a spoke never seeds a fill). Azimuth-oriented — mirror the sort-by-azimuth
setup already used by the velocity-SD (`scope.rs:155-208`) and gap-fill
(`scope.rs:294-...`) passes.

**Detection (per radial, two-pass read-then-write):**
- Candidate = a radial whose *fraction of valid gates beyond mid-range* and *mean dBZ*
  are both high, **and** whose along-azimuth neighbors (±1–2 radials at the same gate)
  are mostly empty. A spike is bright-and-long-in-range but isolated-in-azimuth; real
  precip is azimuthally continuous.
- Concretely: for each radial, over gates `>= MID_GATE`, count valid gates and average
  dBZ; compare to the mean of the same statistic in the K nearest azimuth neighbors. If
  this radial's coverage/intensity exceeds neighbors' by a ratio `RFI_RATIO` and neighbor
  coverage is below `RFI_NEIGHBOR_MAX`, null the whole radial's REF gates (or only the
  far-range run).
- Constants are calibration knobs (`// ponytail:` comment naming the ceiling): fixed
  thresholds tuned on a real sun-spike case in `docs/research/`; upgrade path is a
  per-volume adaptive threshold if fixed values misfire across VCPs.

**Config:** `sun_spike_removal_enabled: bool` (default `false`).

**Touch:** `scope.rs` (new pass + `QcConfig` bool), `settings.rs` (+1 bool), `main.rs`
(wire), settings UI toggle.

**Test:** synthetic sweep with one bright long radial flanked by empty radials + a block
of continuous precip; assert the spoke radial is nulled and the precip block is
untouched.

**Difficulty:** medium.

---

## Stage 3 — KDP (specific differential phase)

**Backlog:** new derivable product (medium). Derive from ΦDP the app already parses
(`ScanData::differential_phase`, `model.rs:160`). Attenuation- and calibration-immune
rain-rate signal. **Biggest lift of the four** — it adds a new product surface, not just a
QC flag.

**Derivation (FMH-11C §2.5.3), per radial, new module `kdp.rs`:**
1. Unwrap total differential phase along the radial (ΦDP is the 0–360° wrapping field;
   reuse the `wrap_deg` folding idea from `nonmet::range_texture`).
2. Median filter (despeckle), then average filter (smooth).
3. Interpolate across meteorological gates along the radial.
4. Range-derivative over a limited increment → °/km. `KDP = 0.5 · dΦDP/dr`.

Derive a `Vec<SweepData>` at ingestion — same shape as the parsed moments — so it flows
through cache and rasterize unchanged. Cleanest home: compute in `data.rs` after ΦDP is
decoded and store on `ScanData` as `pub specific_differential_phase: Vec<SweepData>`.

**New product surface (grep the six existing sites, add the seventh):**
- `Product::SpecificDifferentialPhase` variant + `name()`/`unit()` arms (`model.rs:15-43`).
- `ScanData` field + `sweeps()` arm (`model.rs:150, 165`).
- `colors::kdp_color` + `KDP_LEGEND` (`colors.rs` — mirror `zdr_color`, `colors.rs:181`).
- Product-picker entry (`main.rs:~61-86`), legend map (`main.rs:~2491`), keybind
  (`main.rs:~2862-2877`).
- Cache encode/decode of the new sweep vec (`cache.rs:296, 340` — mirror
  `differential_phase`; bump the cache format version so old caches invalidate cleanly).

**Config:** none required — it's a product, always available when ΦDP exists (empty vec
when absent, like the other dual-pol products on legacy scans).

**Touch:** new `kdp.rs`; edits to `model.rs`, `data.rs`, `colors.rs`, `cache.rs`,
`main.rs`.

**Test:** `kdp.rs` unit test — a synthetic radial with a known linear ΦDP ramp of
`+2°/gate` at `0.25 km/gate` must yield a constant KDP of `0.5 · 2 / 0.25 = 4 °/km`
across the smoothed interior; a flat ΦDP radial yields ~0. Add a `cache.rs`
round-trip test for the new vec.

**Difficulty:** medium (derivation is simple; the cost is the product-surface breadth
+ cache versioning).

---

## Stage 4 — Melting-layer hint

**Backlog:** new derivable product (medium). Circular band of depressed CC at mid/high
elevation angles. Cheap annotation on its own; prereq for HCA (with KDP).

**Approach:** an **overlay/annotation**, not a QC null and not a full raster product.
Detect the melting-layer height, draw a hint on the scope like the existing
borders/cities/alerts overlays.

- Detection: over the CC sweeps (`ScanData::correlation_coefficient`) at mid elevations,
  find the range/height band where CC drops into a characteristic depressed range
  (~0.85–0.95) forming an annular ring. Estimate a single melting-layer height (or
  top/bottom pair) from the elevation angle + range of the CC minimum ring.
- Render: a faint ring/band annotation on the scope, toggled by a settings flag —
  follow the border overlay draw path (`borders.rs`) and the alert-status overlay
  pattern. Keep it a thin annotation; no new full-screen raster.
- Output the estimated height as a small readout value too (reuse the readout panel that
  already shows per-gate values).

**Config:** `melting_layer_hint_enabled: bool` (default `false`).

**Touch:** new `melting_layer.rs` (detection); `settings.rs` (+1 bool); draw hook in the
scope overlay path (`main.rs` / `scope.rs` overlay draw); settings UI toggle.

**Test:** `melting_layer.rs` unit test — synthetic CC sweeps with an annular depressed-CC
ring at a known range/elevation; assert the estimated height is within tolerance, and
that flat-CC input yields no detection (no false ring).

**Difficulty:** medium.

---

## Source map

| Stage | Source |
|---|---|
| Multi-scale texture | RF QC paper (`docs/research/`), FMH-11C §4.2.4.3 |
| Sun-spike / RFI removal | `2-5_Hands-on_training_on_weather_radar_QC` |
| KDP derivation | FMH-11C §2.5.3 |
| Melting-layer hint | FMH-11C §2.8.1 (HCA melting-layer gate) |

## Not in this plan

- **Random Forest P/NP classifier** (backlog, hard) — the fuzzy classifier already
  shipped captures most of the benefit; skip per backlog note.
- **Full HCA** (backlog, hard) — needs KDP (Stage 3) + melting-layer (Stage 4) to land
  first; scope separately once both exist.
- **Persistent clutter map** (backlog, hard) — needs a multi-scan accumulation store; no
  dependency on these four, defer.
