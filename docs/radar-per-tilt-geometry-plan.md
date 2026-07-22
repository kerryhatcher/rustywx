# Plan: per-tilt / per-product gate geometry fix

Bug: `app/src/scope.rs:16-17` hardcode `FIRST_GATE_KM = 2.125` and
`GATE_SPACING_KM = 0.25` (super-res split-cut values) and apply them to every
sweep/product. Legacy upper tilts (1 km REF spacing) and REF-vs-VEL moment
blocks with different first-gate range are placed at the wrong physical range.

Ground truth confirmed by reading the vendored crate source
(`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/nexrad-model-1.0.0-rc.2`):

- `nexrad_model::data::DataMoment` trait (`src/data/moment.rs:14-30`) declares
  `fn first_gate_range_km(&self) -> f64` and `fn gate_interval_km(&self) -> f64`.
  `MomentData` (the public per-radial-per-moment type returned by
  `Radial::reflectivity()` etc., `src/data/radial.rs:136-161`, all
  `Option<&MomentData>`) implements this trait — values are decoded from raw
  `first_gate_range`/`gate_interval` `u16` fields stored in units of 0.001 km
  (`src/data/moment.rs:158-170`), i.e. already in km as `f64`.
- The trait is re-exported at `nexrad_model::data::DataMoment` (`src/data.rs:20`
  `pub use moment::*;`), so it just needs `use nexrad_model::data::DataMoment;`
  in `model.rs` to call `.first_gate_range_km()` / `.gate_interval_km()` on the
  `&MomentData` already in hand.

## 1. `app/src/model.rs` — carry geometry per `SweepData`

- `SweepData` (`model.rs:87-91`) gains two fields:
  ```rust
  pub struct SweepData {
      pub elevation_deg: f32,
      pub radials: Vec<RadialData>,
      #[serde(default = "default_first_gate_km")]
      pub first_gate_km: f32,
      #[serde(default = "default_gate_spacing_km")]
      pub gate_spacing_km: f32,
  }
  ```
  `#[serde(default = ...)]` (not bare `Default::default()` / 0.0) so that a
  stale plain-JSON cache entry (`Cache::save_scan`/`load_scan`, JSON path) that
  predates this change deserializes to the *current* legacy constants instead
  of `0.0`, which would divide-by-zero in `scope::rasterize`'s `gate_frac`
  computation. Add two free functions `default_first_gate_km() -> f32 { 2.125 }`
  and `default_gate_spacing_km() -> f32 { 0.25 }` next to the struct, matching
  the values previously hardcoded in `scope.rs`.
- Keep `scope::FIRST_GATE_KM`/`scope::GATE_SPACING_KM` as documented
  fallback/default constants (used by the two functions above, or referenced
  directly) — do not delete them per the task's explicit instruction.
- `from_sweeps` (`model.rs:151-244`): today it does one `radials()` pass per
  product per sweep, discovering the moment via `radial.reflectivity()` /
  `.velocity()` / etc. (`model.rs:178-185`). Per FMH-11C, gate geometry is
  constant across all radials of one moment block within one sweep, so pull it
  from the *first* radial that has that moment, once, before/alongside the
  `radials` collection:
  ```rust
  use nexrad_model::data::DataMoment;
  ...
  let geometry = sweep.radials().iter().find_map(|r| {
      let m = match product { Product::Reflectivity => r.reflectivity(), ... }?;
      Some((m.first_gate_range_km() as f32, m.gate_interval_km() as f32))
  });
  let (first_gate_km, gate_spacing_km) = geometry.unwrap_or((2.125, 0.25));
  ```
  Fold this into the existing `for (product, out) in [...]` loop
  (`model.rs:160-173`) — the `radials` filter_map already visits every radial
  per product; simplest is to capture geometry from the *first* radial that
  yields `Some(moment)` inside that same filter_map closure (e.g. write it into
  an `Option<(f32,f32)>` captured by a `let mut geometry = None;` before the
  closure, set on first hit) rather than a second pass over `sweep.radials()`.
  Push it onto `SweepData { elevation_deg, radials, first_gate_km, gate_spacing_km }`
  at `model.rs:219-222`.
  Fallback: if the moment is present but a lib call returns `0.0` (defensive,
  shouldn't happen) or no radial carries this moment (`radials.is_empty()` path
  already skips pushing — no change needed there), use the 2.125/0.25 defaults.
- `synthetic_sweep` (`model.rs:96-116`, test helper) and the `SweepData { .. }`
  literal at `model.rs:112-115`: add `first_gate_km: 2.125, gate_spacing_km: 0.25`
  (it already hardcodes 0.25-implicit spacing via `g as f32 * 0.5` — unrelated,
  leave that azimuth-step logic alone, just add the two new struct fields).
- Test moment builders `ref_moment`/`vel_moment`/`cc_moment`
  (`model.rs:262-276`) already pass literal `2125, 250` (= 2.125 km, 0.25 km in
  the raw 0.001-km encoding) into `MomentData::from_fixed_point`. No change
  needed there — they exercise the super-res path. Existing sweep-construction
  tests (`model.rs:365-461`, e.g. `converts_moment_values_to_gates`,
  `dedups_near_identical_elevations`) don't inspect `first_gate_km`/
  `gate_spacing_km` on the output `SweepData`, so they compile unchanged; **add
  one new assertion** in a new test (see §5) that reads
  `scan_data.reflectivity[0].first_gate_km` / `.gate_spacing_km` off a sweep
  built with `2125, 250` and checks it equals `2.125`/`0.25` — proving
  `from_sweeps` actually threads the library geometry through instead of
  silently defaulting.

## 2. `app/src/scope.rs` — use per-sweep geometry, not the consts

- Keep the module consts (`scope.rs:16-17`) as documented legacy defaults
  (used only by `model.rs`'s default functions now) — do not delete.
- Keep `MAX_RANGE_KM` (`scope.rs:19`) as-is: it's the *display* radius, not
  gate geometry, and stays a single global const per the task's instructions.
- `clean_sweep` (`scope.rs:28-223`): the per-range dBZ floor loop at
  `scope.rs:145-166` computes `range_km = FIRST_GATE_KM + i as f32 * GATE_SPACING_KM`
  (`scope.rs:148`). Change to `sweep.first_gate_km + i as f32 * sweep.gate_spacing_km`
  — `sweep: &SweepData` is already a parameter, so no signature change here,
  just read the new fields off it instead of the consts.
- `rasterize` (`scope.rs:227-...`): two more call sites:
  - `scope.rs:287`: range-bounds check
    `if !(FIRST_GATE_KM..=max_range_km).contains(&range_km)` →
    `sweep.first_gate_km` (the `sweep` binding here is the *cleaned* sweep
    returned from `clean_sweep`, `scope.rs:241`, which is a `SweepData` clone
    of the input — the new fields must be carried through `clean_sweep`'s
    construction at `scope.rs:40-43`: add
    `first_gate_km: sweep.first_gate_km, gate_spacing_km: sweep.gate_spacing_km`
    to that literal, otherwise this step silently reads defaulted/garbage
    fields).
  - `scope.rs:299`: `let gate_frac = (range_km - FIRST_GATE_KM) / GATE_SPACING_KM;`
    → `(range_km - sweep.first_gate_km) / sweep.gate_spacing_km`.
  - No signature change needed to `rasterize`/`clean_sweep` — geometry now
    rides on the `SweepData` parameter that was already there. This keeps the
    diff to the two call sites plus the `clean_sweep` struct-literal carry-over.
- Test literals in `scope.rs` (`#[cfg(test)] mod tests`, `scope.rs:1303-...`):
  every `SweepData { elevation_deg, radials }` literal
  (`scope.rs:1321, 1450, 1486, 1490, 1553, 1622, 1680`, plus the `radial()`
  helper at `scope.rs:1307-1313` which is unaffected — it builds `RadialData`,
  not `SweepData`) must gain `first_gate_km: 2.125, gate_spacing_km: 0.25` (the
  values the existing assertions already assume, e.g.
  `refl_noise_floor_nulls_weak_long_range_gate`'s comment
  "Gate index 312 → range = 2.125 + 312*0.25 = 80.125 km" at `scope.rs:~1587`)
  so none of the existing range-derived assertions change behavior.

## 3. `app/src/cache.rs` — persist the two new f32s

- No explicit cache-format version byte exists today (`scan_to_bytes`/
  `bytes_to_scan`, `cache.rs:289-350`) — the format is implicitly versioned by
  strict positional decoding; any field-count mismatch already surfaces as a
  `Reader` "truncated"/tag-byte error, which both `load_scan_compressed` and
  `load_scan` already treat as a corrupt-cache-miss (log + remove + re-fetch,
  `cache.rs:136-158` / `cache.rs:77-97`). Adding two per-sweep f32s is exactly
  such a change: **no explicit version bump needed**, just document in the
  diff/commit message that this is a breaking cache-format change and stale
  entries self-heal via the existing corrupt-cache path. Do not add a new
  version scheme — that would be unrequested scope for what already
  self-heals.
- `encode_sweeps` (`cache.rs:302-322`): after
  `buf.extend_from_slice(&sweep.elevation_deg.to_le_bytes());` (`cache.rs:305`)
  add
  ```rust
  buf.extend_from_slice(&sweep.first_gate_km.to_le_bytes());
  buf.extend_from_slice(&sweep.gate_spacing_km.to_le_bytes());
  ```
  before the existing radial-count line (`cache.rs:306`).
- `decode_sweeps` (`cache.rs:352-394`): after
  `let elevation_deg = r.read_f32()?;` (`cache.rs:356`) add
  ```rust
  let first_gate_km = r.read_f32()?;
  let gate_spacing_km = r.read_f32()?;
  ```
  and add both fields to the `SweepData { .. }` literal at `cache.rs:388-391`.
- JSON path (`Cache::save_scan`/`load_scan`, `cache.rs:48-99`) needs **no
  code change** — `serde_json` picks up the new `SweepData` fields
  automatically, and old cached JSON without them deserializes via the
  `#[serde(default = ...)]` fallbacks from §1.
- Test literals: `sample_scan()` (`cache.rs:447-465`) and the sweep-building
  closure in `realistic_volume_compresses_by_at_least_90_percent`
  (`cache.rs:513-531`) both construct `SweepData { .. }` and must add
  `first_gate_km: 2.125, gate_spacing_km: 0.25`. **Add one new test** (see §5)
  round-tripping a sweep with non-default geometry (e.g. `1.0`/`1.0` km, the
  legacy upper-tilt values) through `scan_to_bytes`/`bytes_to_scan` to prove
  the new fields actually survive encode/decode instead of being silently
  dropped or misaligned.

## 4. `app/src/main.rs` — synthetic sweep + call site

- `synthetic_sweep()` (`main.rs:184-218`) is the offline/no-data fallback; it
  currently derives its own gate count and per-gate range straight from the
  consts (`main.rs:190, 192`). Since this is synthetic (not real per-tilt
  data), the correct fix is: keep using the super-res consts for the synthetic
  pattern's shape (`scope::FIRST_GATE_KM`, `scope::GATE_SPACING_KM` — leave
  `main.rs:190,192` untouched) but populate the new `SweepData` literal at
  `main.rs:214-217` with the matching
  `first_gate_km: scope::FIRST_GATE_KM, gate_spacing_km: scope::GATE_SPACING_KM`
  so the synthetic sweep is internally consistent with the geometry it
  actually used to place its gates. (Do not invent legacy-tilt synthetic data —
  out of scope; the bug is about real per-tilt data.)
- Rasterize call site (`main.rs:969-982`): **no change** — `rasterize` still
  takes `&sweep` (a `SweepData`) plus `max_range_km` (still `scope::MAX_RANGE_KM`,
  the display radius, unrelated to gate geometry); per-sweep geometry now
  rides inside `sweep`/`cc_sweep` automatically once §1–§2 land. Only verify
  after the change that `sweep` (`main.rs:933-943`, cloned straight from
  `scan.sweeps(state.product)[idx]`) and `cc_sweep` (`main.rs:950-964`, cloned
  from `scan.correlation_coefficient`) carry real geometry from `from_sweeps`
  rather than the synthetic fallback's consts — true automatically since both
  come from real `ScanData` when `state.scan` is `Some`.

## 5. New tests to add (proving the fix, not just preserving old behavior)

1. `model.rs`: a test building a sweep via `ScanData::from_sweeps` where the
   moment's raw geometry is `1000, 1000` (1.0 km first gate, 1.0 km spacing —
   the legacy non-super-res values) via `MomentData::from_fixed_point(n, 1000,
   1000, 8, 2.0, 66.0, raws)`, asserting
   `scan_data.reflectivity[0].first_gate_km == 1.0` and
   `.gate_spacing_km == 1.0` — distinct from the existing `2125/250` fixtures.
2. `scope.rs`: a `rasterize`/`clean_sweep`-level test with a sweep whose
   `first_gate_km`/`gate_spacing_km` are `1.0`/`1.0` (not the super-res
   defaults), asserting a gate at a specific range (e.g. gate index 5 → 6.0 km)
   maps to the correct pixel — i.e. that `rasterize` reads `sweep.first_gate_km`
   /`sweep.gate_spacing_km` rather than the module consts. This is the test
   that would have caught the original bug.
3. `cache.rs`: round-trip a `SweepData` with `first_gate_km: 1.0,
   gate_spacing_km: 1.0` through `scan_to_bytes`/`bytes_to_scan` and assert
   both fields survive.

## 6. Work breakdown, ordering, and single-vs-split recommendation

This is **one cohesive change** — `model.rs` (produces the two new fields),
`cache.rs` (persists them), `scope.rs` (consumes them), and `main.rs`
(populates them for the synthetic path) all touch the same struct
(`SweepData`) in the same PR; splitting across people/PRs would just create
merge conflicts on every `SweepData { .. }` literal. **Recommend one
implementer, one PR**, in this order (each step keeps the crate compiling):

1. `model.rs` — add the two fields + default fns to `SweepData`, update
   `from_sweeps`, update `synthetic_sweep()` test helper and all test literals
   in `model.rs`. Add the new `model.rs` test (§5.1). Run `cargo test -p
   <app-crate> model::` to confirm.
2. `cache.rs` — update `encode_sweeps`/`decode_sweeps` and all `SweepData`
   test literals. Add the new cache round-trip test (§5.3). Run cache tests.
   (Depends on step 1 only for the struct shape; independent of scope.rs.)
3. `scope.rs` — switch `clean_sweep`/`rasterize`'s three const-uses
   (`scope.rs:148, 287, 299`) to `sweep.first_gate_km`/`sweep.gate_spacing_km`,
   carry the fields through the `clean_sweep` struct literal (`scope.rs:40-43`),
   update all seven test `SweepData` literals, add the new geometry test
   (§5.2). Run `cargo test -p <app-crate> scope::`.
4. `main.rs` — add the two fields to the `synthetic_sweep()` literal
   (`main.rs:214-217`). No other main.rs change needed. Build the whole crate.
5. Full `cargo test` + `cargo build` (and `cargo build --target
   wasm32-unknown-unknown` if that's part of this repo's CI, since Ply targets
   WASM) to catch any remaining `SweepData { .. }` literal missed by grep.

Before starting, re-run
`grep -rn "SweepData {" app/src/` and
`grep -rn "FIRST_GATE_KM\|GATE_SPACING_KM" app/src/`
to catch any literal or const-use introduced by commits after this plan was
written (the repo is mid-refactor per `git status`/recent Ply-port commits).
