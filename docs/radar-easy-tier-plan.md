# Radar "Easy Tier" Image Improvements — Implementation Plan

Three self-contained radar-image QC/render improvements for the rustywx viewer.
This is a **plan**, not the implementation. Each item is broken into a work-unit
that could be handed to a separate agent. Shared files (`model.rs`, `settings.rs`,
`widgets/settings.rs`, `main.rs`, `scope.rs`) create ordering dependencies that are
called out explicitly at the end.

Code read before writing this plan: `app/src/model.rs`, `app/src/scope.rs`,
`app/src/colors.rs`, `app/src/settings.rs`, `app/src/widgets/settings.rs`,
`app/src/cache.rs`, `app/src/rle.rs`, and the `main.rs` raster call site
(~L946–984) + settings-toggle handlers (~L2965–3002).

---

## Key facts that shape the design

- **Gate type is `Vec<Option<f32>>`** on `model::RadialData`. `None` currently means
  *both* below-threshold and range-folded (`model.rs:185` collapses
  `MomentValue::BelowThreshold | MomentValue::RangeFolded => None`).
- **`nexrad_model` (1.0.0-rc.2) `MomentValue`** has exactly three variants:
  `Value(f32)`, `BelowThreshold`, `RangeFolded`. RF and a numeric value are mutually
  exclusive — an RF gate never carries a dBZ/velocity value.
- **Persistence is NOT serde** for scans — `cache.rs::scan_to_bytes` /
  `bytes_to_scan` hand-encode gates as a tag byte (`0` = None, `1` = `Some(f32)`),
  then `rle::compress` runs over the bytes. `rle.rs` is byte-level and does **not**
  need to change for any item here.
- **QC lives in `scope::clean_sweep`** (called only from `scope::rasterize`).
  Reflectivity already has: a per-range dBZ floor (`scope.rs:81–99`, 20/10/5 dBZ by
  range), a TDBZ texture filter, and post-raster morphological/despeckle/region
  passes. Velocity has no QC pass yet.
- **Settings pattern (cc_gate)**: a `bool` field with `#[serde(default = "default_true")]`
  + an `f32` threshold field with a named serde-default fn (`default_cc_threshold`);
  a `bool_toggle` row in `widgets/settings.rs`; a `*_TOGGLE_ID` const; a handler in
  `main.rs` that flips the bool, calls `state.cache.save_settings`, and sets
  `state.needs_reraster = true`. The threshold is **not** exposed in the UI (only the
  toggle) — match that: expose a toggle only, keep the numeric threshold as a
  serde-defaulted stored value.
- **`clean_sweep` / `rasterize` take flat scalar params** (currently 6). Adding
  Items 2 and 3 grows this to ~10 args → both functions will need
  `#[allow(clippy::too_many_arguments)]` (already used elsewhere in the file). We keep
  flat params (reuse the established cc_gate pattern) rather than introduce a params
  struct — see "Ordering" for why Items 2 & 3 must not be edited concurrently.
- **All `RadialData { .. }` construction sites** (must be updated for Item 1's new
  field): `model.rs:99` (synthetic_sweep), `model.rs:178` (from_sweeps),
  `main.rs:208` (a second synthetic sweep builder), `cache.rs:368` (decode),
  `cache.rs:435` + `cache.rs:499` (tests), `scope.rs:1226` (test helper `radial`).

---

## Item 1 — Render range-folded gates distinctly

**Goal:** RF gates draw in a distinct NWS-style purple instead of transparent.
RF is meaningful only for Doppler products (Velocity, Spectrum Width); dBZ never
range-folds.

**Design decision — parallel flag, not a new enum.** Add a parallel
`range_folded: Vec<bool>` to `RadialData` and keep `gates: Vec<Option<f32>>`
unchanged. Rationale (churn minimization, the task's stated constraint):

- Keeping `gates: Vec<Option<f32>>` means **every existing test literal and every
  QC access** (`gate.is_some()`, `*gate = None`, `if let Some(v)`, bilinear's
  `.and_then(|v| *v)`) compiles unchanged. A `GateValue { Value/Below/RangeFolded }`
  enum would rewrite all of those plus every `vec![None, Some(x)]` literal across
  `model.rs`, `cache.rs`, and `scope.rs` tests — and `scope.rs` is shared with Items
  2 & 3, so a large Item-1 footprint there maximizes collisions.
- RF value is always `None`, so the numeric channel is untouched; bilinear naturally
  excludes RF gates from the blend (no contamination of real velocities), which is
  exactly what we want for a categorical mask.
- Cache stays compact: because `Some`/RF/below are mutually exclusive, the tag byte
  extends cleanly to 3 states (`2 = RangeFolded`) — no second byte stream, still
  RLE-friendly (RF is rare → long runs), and round-trips.

### Files + functions to touch

**`app/src/model.rs`**
- `struct RadialData`: add field
  ```rust
  /// Parallel to `gates`; `true` = range-folded (drawn distinctly, value is None).
  /// May be shorter/empty than `gates` when unknown — treat missing as false.
  #[serde(default)]
  pub range_folded: Vec<bool>,
  ```
- `from_sweeps` (the `.map(|value| ...).collect()` at L180–187): replace the
  single-vec map with a loop building both vecs:
  ```rust
  let mut gates = Vec::with_capacity(...);
  let mut range_folded = Vec::with_capacity(...);
  for value in moment.values() {
      match value {
          MomentValue::Value(v)      => { gates.push(Some(v)); range_folded.push(false); }
          MomentValue::BelowThreshold => { gates.push(None);    range_folded.push(false); }
          MomentValue::RangeFolded    => { gates.push(None);    range_folded.push(true);  }
      }
  }
  ```
  and set `range_folded` in the returned `RadialData`.
- `synthetic_sweep` (L99): add `range_folded: vec![]` (or `vec![false; gates.len()]`).
- Update the doc comment at the top of the file (the "`None` = below threshold /
  range folded" line) to reflect the split.

**`app/src/main.rs`**
- L208 synthetic-sweep builder: add `range_folded: vec![]`.

**`app/src/cache.rs`**
- `encode_sweeps` gate loop (L310–318): derive the 3-state tag:
  ```rust
  match gate {
      Some(v) => { buf.push(1); buf.extend_from_slice(&v.to_le_bytes()); }
      None if radial.range_folded.get(i).copied().unwrap_or(false) => buf.push(2),
      None => buf.push(0),
  }
  ```
  (index the gate loop with `.enumerate()`).
- `decode_sweeps` gate loop (L361–367): produce both vecs — tag `2` → `(None, true)`,
  `1` → `(Some, false)`, `0` → `(None, false)`; update the bad-tag error to allow 2.
  Build `range_folded` alongside `gates` and set it on the returned `RadialData`
  (L368).
- Test builders `cache.rs:435` and `cache.rs:499`: add `range_folded: vec![]`.

**`app/src/colors.rs`**
- Add a public const (NWS AWIPS velocity RF is a dark purple):
  ```rust
  /// Range-folded ("RF") gates — NWS convention draws these a distinct purple.
  pub const RANGE_FOLDED_COLOR: [u8; 4] = [0x77, 0x00, 0x77, 0xff];
  ```

**`app/src/scope.rs`**
- `rasterize` inner loop (L227–235): after the `if let Some(value) = value { .. }`
  block, add an `else` branch for RF, product-gated:
  ```rust
  } else if matches!(product, Product::Velocity | Product::SpectrumWidth) {
      let radial1 = &sweep.radials[order[i1]];
      if radial1.range_folded.get(gate).copied().unwrap_or(false) {
          let c = colors::RANGE_FOLDED_COLOR;
          let idx = (py * size_px + px) * 4;
          pixels[idx..idx + 4].copy_from_slice(&c);
      }
  }
  ```
  (`order`, `i1`, `gate` are already in scope.) `clean_sweep` needs **no** change —
  it clones radials (RF flag rides along) and only mutates `gates`.
- Test helper `radial` (L1226): add `range_folded: vec![]`.

### Unit test to add (match scope.rs style)

Add to `scope.rs` `#[cfg(test)]`, using the existing `radial` helper plus an RF-aware
variant, and to `model.rs` update the existing `range_folded_becomes_none` test:

- **`model.rs`** — rename/extend `range_folded_becomes_none` → `range_folded_flagged`:
  velocity raws `[0, 1, 65]` should give `gates == [None, None, Some(-32.0)]` **and**
  `range_folded == [false, true, false]` (raw 1 decodes to `RangeFolded`). *Verify raw
  1 actually maps to RF for the `vel_moment` scale before asserting; if the fixture's
  scale/offset does not produce RF at raw 1, construct a moment that does (or assert on
  whichever raw yields RF).*
- **`scope.rs`** — `range_folded_gates_render_purple`: build a Velocity `SweepData`
  with one radial, one gate `None` but `range_folded[0] = true`, at a range inside the
  scope; call `rasterize(.., Product::Velocity, ..)`; assert at least one pixel equals
  `colors::RANGE_FOLDED_COLOR`. Companion assert: same sweep as `Product::Reflectivity`
  (RF flag present) produces **no** purple pixels (product gating).

---

## Item 2 — Reflectivity noise-floor cut (configurable dBZ floor)

**Goal:** Null Reflectivity gates below a fixed dBZ floor (~7 dBZ default) as a QC
pass, user-toggleable. This is a **config/threshold addition**, not new machinery —
it folds into the existing per-range floor block at `scope.rs:81–99`.

Note on effect: within 80 km the existing range floor (10/20 dBZ) already exceeds 7,
so the user floor only bites beyond 80 km (where the range floor is 5 dBZ) — a
long-range speckle/noise cut.

### Files + functions to touch

**`app/src/settings.rs`**
- Add fields (match cc_gate exactly):
  ```rust
  #[serde(default = "default_true")]
  pub refl_floor_enabled: bool,
  #[serde(default = "default_refl_floor")]
  pub refl_floor_dbz: f32,
  ```
- Add `fn default_refl_floor() -> f32 { 7.0 }`.
- `Default::default()`: `refl_floor_enabled: true, refl_floor_dbz: 7.0`.
- Extend `default_matches_existing_startup_behaviour`, `serde_round_trip`, and
  `deserializes_settings_without_location_fields` to cover the two new fields.

**`app/src/scope.rs`**
- `clean_sweep` signature: add `refl_floor_enabled: bool, refl_floor_dbz: f32`
  (add/keep `#[allow(clippy::too_many_arguments)]`).
- In the existing reflectivity floor loop (L81–99), raise the computed `floor`:
  ```rust
  let mut floor = if range_km < 20.0 { 20.0 } else if range_km < 80.0 { 10.0 } else { 5.0 };
  if refl_floor_enabled { floor = floor.max(refl_floor_dbz); }
  ```
- `rasterize` signature: add the same two params and forward them into `clean_sweep`.

**`app/src/main.rs`**
- Raster call site (L968–977): pass `state.settings.refl_floor_enabled,
  state.settings.refl_floor_dbz`.
- Add a toggle handler (mirror CC_GATE, L2985–2989) that flips
  `refl_floor_enabled`, saves settings, sets `needs_reraster = true`.

**`app/src/widgets/settings.rs`**
- Add `pub const REFL_FLOOR_TOGGLE_ID: &str = "settings-toggle-refl-floor";`
- Add a `row(ui, "Noise-floor cut (dBZ)", |ui| bool_toggle(ui, REFL_FLOOR_TOGGLE_ID,
  "Noise-floor cut", settings.refl_floor_enabled))` next to the CC-gate row.
  (Bump `modal_h` if the panel overflows — it is fixed at 510.0.)

### Unit test to add (scope.rs style)

`refl_noise_floor_nulls_weak_long_range_gate`: build a Reflectivity radial with
enough gates to reach beyond 80 km (gate index ≥ 312, since
`range = 2.125 + i*0.25`), set that gate to `Some(6.0)` (above the 5 dBZ range floor,
below the 7 dBZ user floor), rest `None`. `clean_sweep(.., refl_floor_enabled=true,
7.0)` → that gate `None`; with `refl_floor_enabled=false` → still `Some(6.0)`.
Keep TDBZ/CC disabled/neutral (kernel small, `cc_sweep=None`) to isolate the floor.

---

## Item 3 — Velocity spatial standard-deviation censoring

**Goal:** New QC pass in `clean_sweep`, gated to `Product::Velocity`. Over a
~9-azimuth × 5-gate window centered on each gate, compute the local standard
deviation of velocity; null the center gate if SD exceeds a threshold (~7–8 m/s).
No Nyquist needed.

### Design

- **Neighbor access by azimuth.** `clean_sweep`'s `cleaned.radials` are not azimuth-
  sorted. Reuse the CC block's pattern (L52–62): build `az_order` (indices sorted by
  azimuth). The 9-azimuth window is `±4` in sorted order, **wrapping** with
  `rem_euclid`/modulo (radials cover 360°). Gate window is `±2`, **clamped** at array
  ends (`saturating_sub` / `.min(n)`, mirroring the TDBZ block, L118–120).
- **Two-pass (read-then-write).** Compute a null mask for every gate first (reading
  neighbors from the *pre-censoring* values), then apply. Mutating in place while
  reading neighbors would bias the window.
- **SD over present gates only.** `SD = sqrt(mean(v²) − mean(v)²)` over `Some` gates in
  the window. **Fail open**: require a minimum sample count (e.g. `≥ 5`) before
  computing/censoring; below that, leave the gate untouched. Only consider censoring a
  gate that itself has a value (`Some`) — RF/`None` centers are skipped.
- **Threshold as setting** (default 7.0 m/s).

### Files + functions to touch

**`app/src/settings.rs`**
- Add fields:
  ```rust
  #[serde(default = "default_true")]
  pub vel_sd_censor_enabled: bool,
  #[serde(default = "default_vel_sd")]
  pub vel_sd_threshold: f32,
  ```
- Add `fn default_vel_sd() -> f32 { 7.0 }`.
- `Default`: `vel_sd_censor_enabled: true, vel_sd_threshold: 7.0`.
- Extend the same three settings tests as Item 2.

**`app/src/scope.rs`**
- `clean_sweep` signature: add `vel_sd_censor_enabled: bool, vel_sd_threshold: f32`.
- Add a new block, product-gated, after the CC block and before/after the
  Reflectivity blocks (it is Velocity-only so order vs the REF blocks is irrelevant):
  ```rust
  if product == Product::Velocity && vel_sd_censor_enabled {
      // az_order by azimuth (as CC block); collect (radial_idx, gate) nulls; apply.
      // window: ±4 azimuth (wrap), ±2 gate (clamp); need >= 5 samples; SD > threshold.
  }
  ```
- `rasterize` signature: add the same two params, forward to `clean_sweep`.

**`app/src/main.rs`**
- Raster call site: pass `state.settings.vel_sd_censor_enabled,
  state.settings.vel_sd_threshold`.
- Toggle handler mirroring CC_GATE (flip, save, `needs_reraster = true`).

**`app/src/widgets/settings.rs`**
- `pub const VEL_SD_TOGGLE_ID: &str = "settings-toggle-vel-sd";`
- A `row(ui, "Velocity SD censor", |ui| bool_toggle(ui, VEL_SD_TOGGLE_ID,
  "Velocity SD censor", settings.vel_sd_censor_enabled))` row.

### Unit test to add (scope.rs style)

`velocity_sd_censor_nulls_noisy_gate_keeps_smooth`: build a Velocity `SweepData` of
~9 radials each with a small number of gates. Populate a smooth region (all gates ≈
constant, e.g. `Some(10.0)`) and one center gate embedded in an alternating
high-variance neighborhood (e.g. neighbors ±30 m/s) so its window SD exceeds 7.
`clean_sweep(.., Product::Velocity, .., vel_sd_censor_enabled=true, 7.0)`:
- the noisy center gate → `None`;
- a gate in the smooth region → unchanged `Some`;
- with `vel_sd_censor_enabled=false` → noisy gate preserved.
Add a second assertion that `Product::Reflectivity` input is untouched by this pass
(product gating). Match the closure-based `removed_count` style used in
`tdbz_kernel_size_widens_clutter_removal_footprint`.

---

## Shared files & ordering dependencies

| File | Item 1 | Item 2 | Item 3 | Conflict surface |
|------|:---:|:---:|:---:|---|
| `model.rs` | struct field + `from_sweeps` + `synthetic_sweep` + test | — | — | Item 1 only |
| `cache.rs` | tag byte 2 + encode/decode + 2 test literals | — | — | Item 1 only |
| `colors.rs` | RF const | — | — | Item 1 only |
| `settings.rs` | — | 2 fields + default fn + 3 tests | 2 fields + default fn + 3 tests | **2 ↔ 3 collide** |
| `widgets/settings.rs` | — | ID + row | ID + row | **2 ↔ 3 collide** (adjacent) |
| `main.rs` | field on L208 literal | call-site args + handler | call-site args + handler | **1,2,3 all touch; 2 ↔ 3 collide hardest** |
| `scope.rs` | `rasterize` render branch + test helper field | `clean_sweep`/`rasterize` sig + floor line + test | `clean_sweep`/`rasterize` sig + new block + test | **2 ↔ 3 collide on the two signatures + main call site** |

**Hard dependencies:**

1. **Item 1 is independent** of 2 & 3 except for the shared `RadialData` field, which
   only affects `scope.rs`'s test helper (`radial`) and the render branch — it does
   **not** touch `clean_sweep`'s signature. Item 1 can run fully in parallel.
2. **Items 2 and 3 both mutate the exact same lines**: the `clean_sweep` and
   `rasterize` parameter lists, the `main.rs` raster call site, `settings.rs` (fields +
   `Default` + the same three tests), and adjacent rows/consts in `widgets/settings.rs`.
   Running them concurrently guarantees merge conflicts on those signatures and the
   call site. **Do Item 2 and Item 3 sequentially** (same agent, or strict order),
   **not** in parallel.

**Optional de-collision (skipped for now — YAGNI):** bundling all `rasterize`/
`clean_sweep` QC knobs into a `QcParams` struct with `Default` would let Items 2 & 3
each add a field without editing shared signatures, and would spare the
`#[allow(clippy::too_many_arguments)]`. It is a new abstraction over an existing,
working flat-param pattern (cc_gate), so it is **not** recommended for three knobs.
Revisit if a fourth+ QC knob lands. If chosen, it becomes a mandatory "Unit 0" prep
that all three items depend on.

---

## Recommended implementation order

1. **Item 1 (range-folded rendering)** — run **in parallel** with the Item 2→3 track.
   Its only shared-file overlap is `scope.rs` (render branch + one test-helper field)
   and one `main.rs` struct literal, none of which touch `clean_sweep`'s signature. Low
   collision risk; a trivial rebase if `scope.rs` tests are edited by the other track.
2. **Item 2 (reflectivity noise floor)** then **Item 3 (velocity SD censor)** — run
   **sequentially, in that order, ideally by one agent.** They share `clean_sweep`/
   `rasterize` signatures, the `main.rs` call site, `settings.rs`, and
   `widgets/settings.rs`. Item 2 first establishes the "add two QC params" edit; Item 3
   extends the same lines. Order between 2 and 3 is otherwise arbitrary (both
   product-gated to different products, no runtime interaction).

**Parallelizable:** Item 1 ∥ {Item 2 → Item 3}.
**Strictly serial:** Item 2 → Item 3.

After all three: run `cargo test -p <app crate>` and `cargo clippy`. Verify the new
`#[allow(clippy::too_many_arguments)]` on `clean_sweep`/`rasterize`, and that the
settings modal height (`widgets/settings.rs` `modal_h = 510.0`) still fits the two new
rows.
