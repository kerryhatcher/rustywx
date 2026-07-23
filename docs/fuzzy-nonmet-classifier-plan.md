# Fuzzy / multi-variable non-meteorological classifier — implementation plan

Replace the current single-threshold CC gate with a fuzzy membership score over
**CC + SD(ΦDP) + SD(ZDR)** (with an optional reflectivity-texture term). Non-met echo
(birds, insects, chaff, AP, ground clutter) is flagged when the aggregate non-met score
exceeds a threshold, and the co-located Reflectivity gate is nulled. Per FMH-11A, ZDR and
ΦDP each independently flag non-met targets that a CC-only gate misses; combining them
gives fewer false nulls on real precip *and* better biology/chaff/AP rejection.

This is the next deferred item after per-tilt geometry and velocity dealiasing
(`docs/radar-image-improvements-backlog.md`, "Dual-pol non-meteorological rejection").

## What exists today

`scope::clean_sweep` (`app/src/scope.rs:34-94`) does **CC-gating**: for the Reflectivity
product, null a REF gate whose co-located CC (nearest-azimuth, equal gate index) is below
`cc_gate_threshold` (default 0.80). The CC sweep is selected in `main.rs:966-983` by nearest
`elevation_deg` and passed into `rasterize`. All four dual-pol moments are already decoded
into `ScanData` (`reflectivity`, `differential_reflectivity` = ZDR,
`correlation_coefficient` = CC, `differential_phase` = ΦDP) — so **no new data plumbing**;
the ZDR and ΦDP sweeps just need to be selected and threaded the same way CC already is.

The TDBZ texture pass (`scope.rs:208-262`) already computes a gate-to-gate range texture on
reflectivity — the same machinery generalizes to ΦDP/ZDR texture.

## Design — fuzzy membership

New module `app/src/nonmet.rs`. Pure, testable. Each variable maps to a "non-met membership"
in `[0,1]` via a clamped linear ramp (trapezoidal fuzzy set); the aggregate is a weighted
mean. All breakpoints are named consts = **calibration knobs** (site/hardware variation is
real; FMH-11A/HCA give typical ranges, not universal constants).

```rust
/// Non-met membership for each discriminator, 0.0 = definitely precip, 1.0 = definitely non-met.
fn mu_cc(cc: f32) -> f32;        // 1.0 at cc<=0.80, ramp to 0.0 at cc>=0.95 (precip has high CC)
fn mu_sd_phidp(sd: f32) -> f32;  // 0.0 at sd<=10°, ramp to 1.0 at sd>=30° (precip ΦDP smooth)
fn mu_sd_zdr(sd: f32) -> f32;    // 0.0 at sd<=1dB, ramp to 1.0 at sd>=3dB
fn mu_zdr_mag(zdr: f32) -> f32;  // 0.0 for |zdr|<=4dB, ramp to 1.0 by |zdr|>=6dB (optional term)

/// Weighted non-met score in [0,1]. SD(ΦDP) and CC dominate (operational discriminators).
fn nonmet_score(cc: Option<f32>, sd_phidp: Option<f32>, sd_zdr: Option<f32>, zdr: Option<f32>) -> f32;
```

- **Fail open, per variable:** any absent input (no ZDR volume, out-of-range gate) drops out
  of the weighted mean. Note that `nonmet_score` itself, with only `cc` present, reduces
  *mathematically* to `mu_cc(cc)` — but comparing that to `nonmet_threshold` is **not** the
  same decision as the legacy `cc < cc_gate_threshold` hard gate (the ramp crosses
  `nonmet_threshold` at a different CC value), so it would be a strictly *more aggressive*
  mask on real precip, not an equivalent one.
- **Decision:** the CC-only degrade is a true behavioral reduction, handled by a dedicated
  `should_null_reflectivity_gate(cc, sd_phidp, sd_zdr, zdr, cc_gate_threshold, nonmet_threshold)`
  wrapper: when *both* dual-pol texture terms (`sd_phidp`, `sd_zdr`) are unavailable at a gate,
  it falls back to the exact legacy hard comparison `cc < cc_gate_threshold` — so a legacy
  single-pol / CC-only volume degrades gracefully to *today's actual gate*, never a worse mask.
  Only when at least one dual-pol texture term is present does the fuzzy
  `nonmet_score >= nonmet_threshold` (default `0.5`) decision govern.
- **Weights** (consts): `W_PHIDP = 0.4`, `W_CC = 0.35`, `W_SDZDR = 0.2`, `W_ZDRMAG = 0.05`;
  normalized over whichever inputs are present.

### ΦDP / ZDR range texture

Add a wrap-robust gate-to-gate texture helper (generalize the TDBZ inner loop):
`range_texture(gates: &[Option<f32>], half_window: usize, wrap_deg: Option<f32>) -> Vec<f32>`
= RMS of consecutive valid-gate differences within the window.
- **ΦDP wraps 0–360°:** pass `wrap_deg = Some(360.0)` so each gate-to-gate difference is folded
  into `[-180, 180]` before squaring. Skipping this makes a single 359°→1° step read as a
  huge fake texture spike. **This is the one real correctness trap in the whole feature.**
- ZDR does not wrap (`wrap_deg = None`).

Textures are computed **once per aux sweep**, not per REF radial (the aux radial is reused
across the nearest-azimuth match), then indexed alongside the CC value.

## Integration

In `clean_sweep`, replace the CC-gating block (`scope.rs:62-94`) with:
- if `nonmet_fuzzy_enabled` → fuzzy pass (needs CC + ZDR + ΦDP sweeps and their range
  textures; Reflectivity only);
- else if `cc_gate_enabled` → the existing CC gate (unchanged fallback).

Runs before the dBZ floor / TDBZ block, same as CC-gating does now, so a suppressed gate
isn't also texture-processed. Reuse `nearest_two_radial_indices` for azimuth alignment (take
the primary index, no interpolation — a QC mask must not be blurred), exactly as CC-gating
does today.

### Threading the aux sweeps (do the param refactor first)

`rasterize`/`clean_sweep` already take 11–12 positional args; adding ZDR + ΦDP sweeps + a
toggle + threshold pushes it past readable. **First mechanical step:** bundle the QC inputs
into a struct — a no-op refactor with the current fields, then add the new ones:

```rust
pub struct QcConfig<'a> {
    pub tdbz_kernel_size: usize,
    pub cc_sweep: Option<&'a SweepData>,
    pub cc_gate_enabled: bool,
    pub cc_gate_threshold: f32,
    pub refl_floor_enabled: bool,
    pub refl_floor_dbz: f32,
    pub vel_dealias_enabled: bool,
    pub vel_sd_censor_enabled: bool,
    pub vel_sd_threshold: f32,
    // new:
    pub zdr_sweep: Option<&'a SweepData>,
    pub phidp_sweep: Option<&'a SweepData>,
    pub nonmet_fuzzy_enabled: bool,
    pub nonmet_threshold: f32,
}
```

Touches every `rasterize` call site (main.rs, `app/benches/rasterize.rs`) — but those are
already being touched to pass the new sweeps, so the struct is a net simplification.

- **`main.rs:966-983`**: select `zdr_sweep` and `phidp_sweep` by nearest `elevation_deg`,
  mirroring the existing `cc_sweep` block; gate on `nonmet_fuzzy_enabled`.
- **`settings.rs`**: add `nonmet_fuzzy_enabled: bool` and `nonmet_threshold: f32`, both
  `#[serde(default = ...)]` for config back-compat (mirror `vel_sd_threshold` / geometry
  fields). Default `nonmet_fuzzy_enabled = false` initially — ship it opt-in, flip the default
  to `true` in a follow-up once eyeballed on real dual-pol volumes (the mask changes the
  default Reflectivity view everyone sees, so don't flip it sight-unseen).
- **`widgets/settings.rs`**: add a "Fuzzy non-met filter" toggle next to the CC-gate toggle.

## Tests (`nonmet.rs` unit + `scope.rs` integration)

- Each membership fn: clamped to `[0,1]`, monotonic, correct at the breakpoints.
- `nonmet_score`: a bird gate (CC 0.5, SD(ΦDP) 40°, SD(ZDR) 4 dB) → high score → nulled;
  a precip gate (CC 0.98, SD(ΦDP) 5°, SD(ZDR) 0.5 dB) → low score → preserved.
- Fail-open: score with only CC present equals `mu_cc`-driven behavior (matches old gate);
  all-None inputs → score 0 (never null).
- **ΦDP wrap:** `range_texture` on a radial stepping 358°→2° yields near-zero texture, not a
  spike (the decisive test — asserts the wrap fold works).
- Integration: `rasterize` with fuzzy enabled nulls a synthetic non-met gate and preserves a
  synthetic core; with all aux sweeps absent it leaves REF untouched (fails open).

## Ordering & risk

1. `nonmet.rs`: membership fns + `range_texture` (wrap-aware) + `nonmet_score` + unit tests.
2. `QcConfig` refactor (mechanical no-op) across `clean_sweep`/`rasterize` + call sites.
3. Wire fuzzy pass into `clean_sweep`; add ZDR/ΦDP selection in main.rs; settings + UI toggle;
   bench call site.
4. Integration test.

Risks / notes:
- **ΦDP wrapping** — the one real trap; covered by the wrap test above.
- **Absent dual-pol** — legacy/single-pol or partial volumes: fail open, degrade to CC-only.
- **Hand-tuned breakpoints** — named consts, expected to need field tuning; not universal.
  Cite FMH-11A ranges in comments; leave the knobs visible (`// calibration knob`).
- **Performance** — three aux textures per sweep, computed once per aux sweep (not per REF
  radial). Reflectivity is one sweep per render; cheap. ponytail-comment the recompute-per-render
  ceiling like the dealias pass does, memoize at ingestion only if profiling says so.
- **Supersedes, not stacks:** fuzzy replaces the CC-only gate when on (CC membership subsumes
  it); don't run both.

## Source map

| Piece | Source |
|---|---|
| Multi-variable fuzzy non-met membership (CC/ZDR/ΦDP) | FMH-11A; `remotesensing-18-00827` |
| SD(ΦDP) / ZDR texture as non-met discriminator | dual-pol QC papers in `docs/research/` |
| Reuse of gate-to-gate range texture | existing TDBZ pass, `scope.rs:208-262` |
