# Implementation Plan: Dual-Pol Products (ZDR / CC / PhiDP) + CC-Gating

Status: PLAN ONLY — do not implement from this document without picking up an
individual task. Every line/anchor below was verified against the current HEAD
of `port/ply-engine` (line numbers drift as tasks land — always re-anchor on the
quoted code, never the bare line number).

## Goal (two features)

1. **Three new selectable display products**, decoded from moments the pinned
   crates already expose on `nexrad_model::data::Radial`:
   - Differential Reflectivity — `radial.differential_reflectivity()` → ZDR (dB)
   - Correlation Coefficient — `radial.correlation_coefficient()` → RhoHV/CC (unitless 0–1)
   - Differential Phase — `radial.differential_phase()` → PhiDP (degrees 0–360)
2. **CC-gating**: a render-time toggle that nulls Reflectivity gates whose
   co-located CC value is below a threshold (default 0.80), suppressing
   birds/chaff/AP (CC < ~0.80) while keeping precip (CC > ~0.95).

## Confirmed facts (already researched — trust, do not re-verify)

- `nexrad-data = "=1.0.0-rc.7"`, `nexrad-model = "=1.0.0-rc.2"` already decode
  and expose all three moments. No crate/version change, no decoder work.
- Accessors return `Option<&MomentData>`, identical shape to the existing
  `reflectivity()`/`velocity()`/`spectrum_width()`. `MomentData::values()`
  yields `MomentValue::Value(f32) | BelowThreshold | RangeFolded`, already
  mapped in `model.rs::from_sweeps`.
- **`Radial::new` signature confirmed** (from
  `~/.cargo/.../nexrad-model-1.0.0-rc.2/src/data/radial.rs:41-56`). The 14 args,
  in order, are: `collection_timestamp, azimuth_number, azimuth_angle_degrees,
  azimuth_spacing_degrees, radial_status, elevation_number,
  elevation_angle_degrees,` then the **7 moment slots**:
  `reflectivity, velocity, spectrum_width, differential_reflectivity,
  differential_phase, correlation_coefficient, clutter_filter_power`.
  The last is `Option<CFPMomentData>` (not `MomentData`).
  → The existing `radial(...)` test helper in `model.rs` passes
  `refl, vel, None, None, None, None, None` — i.e. `sw=None, zdr=None,
  phi=None, cc=None, cfp=None`. Test authors must supply moments in **exactly
  this positional order**.
- KDP is out of scope (not transmitted; derived from PhiDP).

## Decisions made in this plan (flagged with rationale)

- **D1 — CC-gating is a render-time filter, not baked at decode.** CC is stored
  as its own product, so gating stays toggleable with no re-fetch/re-decode.
  The matching CC sweep is passed into `clean_sweep`/`rasterize` as
  `Option<&SweepData>`. Fails open (no CC volume → no gating).
- **D2 — Keyboard keys: `Z`=ZDR, `C`=CC, `P`=PhiDP.** Verified non-colliding
  against the active bindings (`R` refl, `V` vel, `W` spectrum-width AND watches
  [pre-existing double-bind, do NOT touch], `B` borders, `A` warnings, `N`
  tropical, `T` tilt, `F` fullscreen, `0` reset, arrows nav, `/`+shift shortcuts,
  `Esc` close). `Z`/`C`/`P` are free.
- **D3 — Toggle labels stay short** ("ZDR", "CC", "PhiDP") because the product
  toggle row now holds 6 buttons; `Product::label()` keeps the descriptive names
  for status/legend use.
- **D4 — Cache back-compat: append-at-end + fail-to-miss.** The three new sweep
  groups are appended after `spectrum_width` in the binary layout; short old
  blobs hit the `Reader` "truncated" error and are treated as a cache miss
  (verified fail-safe: `cache.rs:137-150` and `78-88` catch decode errors, log,
  remove the key, and yield `None` → refetch). For the JSON path the three new
  `Vec` fields get `#[serde(default)]` so old JSON deserializes with empty vecs.
- **D5 — QC passes (dBZ floor, TDBZ texture filter, morphological close,
  despeckle, remove_small_regions) run for Reflectivity ONLY.** ZDR/CC/PhiDP have
  their own valid ranges and are not intensity fields; running intensity-oriented
  speckle/texture filters on them would punch holes in valid data. The dBZ floor
  is already gated on `Product::Reflectivity` (`scope.rs:33`); the TDBZ block
  (`scope.rs:53-100`) and the three post-passes (`scope.rs:173-181`) currently
  run for **all** products and MUST be gated.
- **D6 — CC-gate settings live in `Settings`:** `cc_gate_enabled: bool`
  (default `true`) and `cc_gate_threshold: f32` (default `0.80`), persisted like
  the other toggles. UI: a settings-panel toggle + keyboard shortcut is out
  (no free single key that reads well); a settings-panel toggle is in. Threshold
  is not surfaced in the UI (fixed default; edit-in-settings can come later —
  YAGNI). Flagged: if a threshold control is wanted, add a cycle button later.

---

## Task graph / sequencing

```
T1 colors.rs (legends+fns) ─┐
                            ├─► T3 FOUNDATION (model+cache+scope-arms+legend-match) ─► T4 UI wiring ─► T5 shortcuts
T2 settings.rs (cc fields) ─┘                                                     └─► T6 CC-gating ─┘
                                                                                                    └─► T7 integration+verify
```

- **T1, T2** are independent and parallel (touch disjoint files, compile alone).
- **T3** depends on **T1** (its `scope.rs` `color_of` arm calls the new color fns).
- **T4** depends on **T3**. **T5** depends on **T4** (needs final key labels).
- **T6** depends on **T2 + T3**.
- **T7** depends on **all**.
- Recommended linear order for a single implementer: **T1 → T2 → T3 → T6 → T4 → T5 → T7**
  (T6 before T4 so the new products render correctly the moment they become
  selectable; between T3 and T6 they are unreachable from the UI anyway).

The workspace compiles after every task. The reason T3 is one large task and not
several: adding variants to `Product` breaks every exhaustive `match` on it
simultaneously (Rust has no partial-enum compile), and adding fields to
`ScanData` breaks every struct literal simultaneously. Those edits MUST land
together.

---

## T1 — colors.rs: three new legends + color functions

**Tier: Sonnet** (needs meteorological judgment on the ramps; mechanical otherwise)
**Depends on: nothing. Parallel with T2.**
**File: `app/src/colors.rs`**

### 1a. Add three legend constants after `SPECTRUM_WIDTH_LEGEND` (ends `colors.rs:56`)

`spline_color` returns fully transparent below the first anchor's threshold
(`colors.rs:85-87`) and clamps to the last anchor at/above the top — so the
first anchor sets the "start drawing here" floor. Ranges per the goal:

```rust
/// Differential reflectivity (ZDR) in dB. Diverging ramp: negative (oblate-
/// down / small drops, hail) cool blues → near-zero neutral gray → positive
/// (large oblate drops, melting) greens→yellows→reds. Range ~ -4..+6 dB.
pub const ZDR_LEGEND: &[(f32, [u8; 4])] = &[
    (-4.0, [0x40, 0x00, 0x80, 0xff]), // deep purple
    (-2.0, [0x00, 0x40, 0xc0, 0xff]), // blue
    (-0.5, [0x40, 0x80, 0xc0, 0xff]), // pale blue
    (0.0,  [0x80, 0x80, 0x80, 0xff]), // neutral gray (ZDR ~ 0)
    (0.5,  [0x00, 0x80, 0x00, 0xff]), // green
    (1.5,  [0x00, 0xe0, 0x00, 0xff]), // bright green
    (3.0,  [0xfd, 0xf8, 0x02, 0xff]), // yellow
    (4.5,  [0xfd, 0x95, 0x00, 0xff]), // orange
    (6.0,  [0xfd, 0x00, 0x00, 0xff]), // red
];

/// Correlation coefficient (RhoHV / CC), unitless. The meteorologically useful
/// band is 0.80–1.00; below ~0.80 is non-meteorological (birds, chaff, AP,
/// ground clutter). First anchor 0.2 so anything lower draws transparent; the
/// 0.80–1.00 range gets most of the color resolution.
pub const CC_LEGEND: &[(f32, [u8; 4])] = &[
    (0.20, [0x30, 0x30, 0x30, 0xff]), // very low — biology/clutter
    (0.45, [0x60, 0x00, 0x80, 0xff]), // purple
    (0.65, [0x00, 0x40, 0xc0, 0xff]), // blue
    (0.80, [0x00, 0xc0, 0xc0, 0xff]), // cyan — gating threshold
    (0.90, [0x00, 0xe0, 0x00, 0xff]), // green
    (0.95, [0xfd, 0xf8, 0x02, 0xff]), // yellow
    (0.98, [0xfd, 0x95, 0x00, 0xff]), // orange
    (1.00, [0xfd, 0x00, 0x00, 0xff]), // red — highest correlation (uniform precip)
    (1.05, [0xfd, 0xfd, 0xfd, 0xff]), // white cap (clamp guard above 1.0)
];

/// Differential phase (PhiDP) in degrees, 0..360. Cyclic-ish quantity; use a
/// perceptually even hue sweep so the wrap at 360→0 is not jarring.
pub const PHIDP_LEGEND: &[(f32, [u8; 4])] = &[
    (0.0,   [0x00, 0x00, 0x80, 0xff]), // navy
    (60.0,  [0x00, 0x80, 0xc0, 0xff]), // blue-cyan
    (120.0, [0x00, 0xc0, 0x40, 0xff]), // green
    (180.0, [0xe0, 0xe0, 0x00, 0xff]), // yellow
    (240.0, [0xf0, 0x80, 0x00, 0xff]), // orange
    (300.0, [0xd0, 0x00, 0x00, 0xff]), // red
    (360.0, [0x80, 0x00, 0x80, 0xff]), // purple (wraps toward navy)
];
```

### 1b. Add three public color functions after `spectrum_width_color` (`colors.rs:131-133`)

```rust
pub fn zdr_color(db: f32) -> [u8; 4] {
    spline_color(ZDR_LEGEND, db)
}

pub fn cc_color(cc: f32) -> [u8; 4] {
    spline_color(CC_LEGEND, cc)
}

pub fn phidp_color(deg: f32) -> [u8; 4] {
    spline_color(PHIDP_LEGEND, deg)
}
```

### 1c. Extend the test module

- Add the three legends to the `legends_are_ascending` assertion
  (`colors.rs:239-243`) — all three anchor tables are strictly ascending; verify.
- Add a `dualpol_legends_pass_through_anchors` test mirroring
  `spectrum_width_passes_through_anchors` (`colors.rs:246-251`) for each new fn.
- Add `cc_below_minimum_is_transparent` (`cc_color(0.1) == [0,0,0,0]`) and
  `zdr_below_minimum_is_transparent` (`zdr_color(-5.0) == [0,0,0,0]`).
- Update the `use super::{...}` import list (`colors.rs:137-140`) to include the
  new consts/fns referenced by tests.

**Verify:** `cargo test -p rustywx colors::` ; `cargo fmt --check`.

---

## T2 — settings.rs: CC-gate fields

**Tier: Haiku** (mechanical, fully specified)
**Depends on: nothing. Parallel with T1.**
**File: `app/src/settings.rs`**

### 2a. Add a serde-default helper near `default_true` (`settings.rs:87-89`)

```rust
/// Serde default for [`Settings::cc_gate_threshold`] (missing in older configs).
fn default_cc_threshold() -> f32 {
    0.80
}
```

### 2b. Add two fields to `struct Settings` (after `center_on_location`, `settings.rs:157`)

```rust
    /// Whether correlation-coefficient gating suppresses non-meteorological
    /// echo (CC < threshold) from the Reflectivity display. Default on.
    #[serde(default = "default_true")]
    pub cc_gate_enabled: bool,
    /// CC value below which a Reflectivity gate is suppressed when gating is on.
    #[serde(default = "default_cc_threshold")]
    pub cc_gate_threshold: f32,
```

### 2c. Add to `Default for Settings` (after `center_on_location: false,`, `settings.rs:182`)

```rust
            cc_gate_enabled: true,
            cc_gate_threshold: 0.80,
```

### 2d. Tests

- Extend `default_matches_existing_startup_behaviour` (`settings.rs:192-207`):
  `assert!(settings.cc_gate_enabled);` and
  `assert_eq!(settings.cc_gate_threshold, 0.80);`.
- Extend `serde_round_trip` (`settings.rs:210-236`) struct literal with the two
  new fields (e.g. `cc_gate_enabled: false, cc_gate_threshold: 0.85`).
- Extend `deserializes_settings_without_location_fields` (`settings.rs:259-278`):
  the old blob lacks the new keys — assert `s.cc_gate_enabled` is `true` and
  `s.cc_gate_threshold == 0.80` (defaults applied).

**Verify:** `cargo test -p rustywx settings::` ; `cargo fmt --check`.

---

## T3 — FOUNDATION: Product variants + ScanData fields + cache + broken matches

**Tier: Sonnet** (large cross-file; the atomic "make it compile with 6 products" change)
**Depends on: T1** (scope `color_of` arm calls `colors::zdr_color` etc.)
**Files: `app/src/model.rs`, `app/src/cache.rs`, `app/src/scope.rs`, `app/src/main.rs`**

> This is deliberately one task. Adding `Product` variants breaks every
> exhaustive match at once; adding `ScanData` fields breaks every struct literal
> at once. All must land together or the crate will not build.

### 3a. `model.rs` — enum + impls

`Product` enum (`model.rs:18-22`) → add three variants:

```rust
pub enum Product {
    Reflectivity,
    Velocity,
    SpectrumWidth,
    DifferentialReflectivity,
    CorrelationCoefficient,
    DifferentialPhase,
}
```

`label()` (`model.rs:25-31`) → add arms:

```rust
            Product::DifferentialReflectivity => "Differential Reflectivity",
            Product::CorrelationCoefficient => "Correlation Coefficient",
            Product::DifferentialPhase => "Differential Phase",
```

`units()` (`model.rs:34-39`) — current match is
`Reflectivity => "dBZ", Velocity | SpectrumWidth => "m/s"`. Add:

```rust
            Product::DifferentialReflectivity => "dB",
            Product::CorrelationCoefficient => "",       // unitless
            Product::DifferentialPhase => "°",
```

### 3b. `model.rs` — ScanData fields

`struct ScanData` (`model.rs:103-109`) — add three `Vec<SweepData>` fields with
`#[serde(default)]` (D4 back-compat for the JSON cache path):

```rust
pub struct ScanData {
    pub timestamp: DateTime<Utc>,
    pub reflectivity: Vec<SweepData>,
    pub velocity: Vec<SweepData>,
    pub spectrum_width: Vec<SweepData>,
    #[serde(default)]
    pub differential_reflectivity: Vec<SweepData>,
    #[serde(default)]
    pub correlation_coefficient: Vec<SweepData>,
    #[serde(default)]
    pub differential_phase: Vec<SweepData>,
    pub vcp_number: u16,
}
```

### 3c. `model.rs` — `sweeps()` accessor (`model.rs:112-118`)

```rust
            Product::DifferentialReflectivity => &self.differential_reflectivity,
            Product::CorrelationCoefficient => &self.correlation_coefficient,
            Product::DifferentialPhase => &self.differential_phase,
```

### 3d. `model.rs` — `from_sweeps` (`model.rs:125-184`)

1. Declare three more accumulators alongside `reflectivity`/`velocity`/`spectrum_width`
   (`model.rs:126-128`):
   ```rust
   let mut differential_reflectivity = Vec::new();
   let mut correlation_coefficient = Vec::new();
   let mut differential_phase = Vec::new();
   ```
2. Extend the `for (product, out) in [ ... ]` array (`model.rs:131-135`):
   ```rust
       (Product::DifferentialReflectivity, &mut differential_reflectivity),
       (Product::CorrelationCoefficient, &mut correlation_coefficient),
       (Product::DifferentialPhase, &mut differential_phase),
   ```
3. Extend the moment match (`model.rs:140-144`):
   ```rust
       Product::DifferentialReflectivity => radial.differential_reflectivity(),
       Product::CorrelationCoefficient => radial.correlation_coefficient(),
       Product::DifferentialPhase => radial.differential_phase(),
   ```
4. Add three `sort_and_dedup(...)` calls (`model.rs:173-175`).
5. Extend the `ScanData { ... }` literal (`model.rs:177-183`) with the three new
   fields.

### 3e. `model.rs` — tests

- Extend the `radial(...)` helper (`model.rs:212-235`) to accept the dual-pol
  moment slots so tests can exercise the new accessors. Recommended signature:
  ```rust
  fn radial(
      az: f32, elev_num: u8, elev_deg: f32,
      refl: Option<MomentData>, vel: Option<MomentData>,
      zdr: Option<MomentData>, cc: Option<MomentData>, phi: Option<MomentData>,
  ) -> Radial {
      Radial::new(
          0, 1, az, 0.5, RadialStatus::IntermediateRadialData, elev_num, elev_deg,
          refl, vel,
          None,       // spectrum_width
          zdr,        // differential_reflectivity
          phi,        // differential_phase  ← NOTE order: phi BEFORE cc
          cc,         // correlation_coefficient
          None,       // clutter_filter_power
      )
  }
  ```
  **Critical:** `Radial::new` moment order is `... spectrum_width,
  differential_reflectivity, differential_phase, correlation_coefficient,
  clutter_filter_power`. PhiDP comes **before** CC positionally. Update every
  existing `radial(...)` call site in the test module (`model.rs:242-262`,
  `303-308`) to the new arity (pass `None, None, None` for the new params where
  they were previously implicit).
- Add a `cc_moment(raws)` helper mirroring `ref_moment`/`vel_moment`
  (`model.rs:202-210`). CC is encoded 0..1; pick a scale/offset that yields
  values in range (e.g. `MomentData::from_fixed_point(gate_count, 2125, 250, 8,
  scale, offset, raws)` — confirm CC encoding constants against
  `nexrad-model` docs; a simple `scale=0.00333, offset=-0.0` style is fine for a
  synthetic test as long as decoded values land in 0.2..1.05).
- Add a test `dualpol_moments_convert_to_gates` that builds a sweep with a CC
  moment and asserts `scan_data.correlation_coefficient[0].radials[0].gates`
  maps values correctly and BelowThreshold→None.
- Update `sweeps_accessor_selects_product` (`model.rs:315-320`) if desired to
  cover a dual-pol product.

### 3f. `cache.rs` — binary serializer (append at end, D4)

- `scan_to_bytes` (`cache.rs:289-297`) — after the existing three `encode_sweeps`
  calls (`cache.rs:293-295`) add:
  ```rust
  encode_sweeps(&mut buf, &scan.differential_reflectivity);
  encode_sweeps(&mut buf, &scan.correlation_coefficient);
  encode_sweeps(&mut buf, &scan.differential_phase);
  ```
- `bytes_to_scan` (`cache.rs:324-340`) — after the existing three
  `decode_sweeps` calls (`cache.rs:330-332`) add three more reads, and extend the
  returned `ScanData { ... }` literal (`cache.rs:333-339`):
  ```rust
  let differential_reflectivity = decode_sweeps(&mut r)?;
  let correlation_coefficient = decode_sweeps(&mut r)?;
  let differential_phase = decode_sweeps(&mut r)?;
  ```
  Old/short blobs error in `decode_sweeps`→`Reader::take` ("truncated") and are
  treated as a cache miss by both load paths — no code change needed there, but
  cite `cache.rs:137-150` (compressed) and `cache.rs:78-88` (JSON) in the PR to
  document the fail-open path.

### 3g. `cache.rs` — test fixtures

- `sample_scan` (`cache.rs:421-435`) — add the three new fields (`: vec![]`) to
  the `ScanData { ... }` literal.
- `realistic_volume_compresses_by_at_least_90_percent` (`cache.rs:502-508`) — add
  the three new fields (`: vec![]`) to that `ScanData { ... }` literal.
- Optional: add a `scan_bytes_round_trip_preserves_dualpol` test that populates
  `correlation_coefficient` with a sweep and round-trips it.

### 3h. `scope.rs` — `color_of` match (`scope.rs:131-135`)

Add three arms (functions exist from T1):

```rust
        Product::DifferentialReflectivity => colors::zdr_color as fn(f32) -> [u8; 4],
        Product::CorrelationCoefficient => colors::cc_color as fn(f32) -> [u8; 4],
        Product::DifferentialPhase => colors::phidp_color as fn(f32) -> [u8; 4],
```

### 3i. `scope.rs` — gate the QC passes on Reflectivity (D5)

- The dBZ floor block is already `if product == Product::Reflectivity`
  (`scope.rs:33`) — leave as is.
- Wrap the **TDBZ block** (`scope.rs:53-100`, from `const TDBZ_THRESHOLD` through
  the `for radial in &mut cleaned.radials { ... }` texture loop) in
  `if product == Product::Reflectivity { ... }`. (Cleanest: fold both the floor
  and TDBZ into a single `if product == Product::Reflectivity` block.)
- In `rasterize`, gate the three post-passes
  (`morphological_close`/`despeckle`/`remove_small_regions`, `scope.rs:173-181`)
  on `if product == Product::Reflectivity { ... }`. ZDR/CC/PhiDP skip them.
- Update `tdbz_kernel_size_widens_clutter_removal_footprint`
  (`scope.rs:1233-1255`) — it calls `clean_sweep(&sweep, Product::Reflectivity,
  kernel_size)`; its signature changes in T6, but under T3 it still passes
  `Product::Reflectivity` so behavior is unchanged. No edit needed in T3 (T6
  updates the call). Note the dependency so T6 does not double-edit.

### 3j. `main.rs` — legend match (`main.rs:2014-2018`)

Add three arms:

```rust
                            Product::DifferentialReflectivity => colors::ZDR_LEGEND,
                            Product::CorrelationCoefficient => colors::CC_LEGEND,
                            Product::DifferentialPhase => colors::PHIDP_LEGEND,
```

> Note: `PRODUCT_OPTIONS` (`main.rs:51-67`) is a fixed-size array `[…; 3]`;
> leaving it untouched here does NOT break compilation (arrays don't force
> enum-exhaustiveness). New products are simply not yet selectable via the toggle
> or keyboard — that is T4. Status-bar rendering (`main.rs:2852-2874`) and
> `select_product`/`select_tilt` (`main.rs:2793-2813`) are product-agnostic
> (they call `scan.sweeps(product)` and `product.units()`), so no change here.

**Verify (end of T3):** `cargo check -p rustywx` (MUST pass — this is the
compile-gate task) ; `cargo test -p rustywx model:: cache::` ; `cargo fmt --check`.

---

## T4 — main.rs: make the new products selectable (toggle + keyboard)

**Tier: Sonnet** (toggle layout must absorb 6 buttons without wrapping badly)
**Depends on: T3.**
**File: `app/src/main.rs`**

### 4a. `PRODUCT_OPTIONS` (`main.rs:51-67`) — 3 → 6 entries, short labels (D3)

Change the array type to `[ToggleOption<Product>; 6]` and append:

```rust
    ToggleOption { id: "btn-zdr",   label: "ZDR",   value: Product::DifferentialReflectivity },
    ToggleOption { id: "btn-cc",    label: "CC",    value: Product::CorrelationCoefficient },
    ToggleOption { id: "btn-phidp", label: "PhiDP", value: Product::DifferentialPhase },
```

(Keep the existing three; consider shortening "Spectrum Width" → "SW" if the row
overflows — see 4c.)

### 4b. Keyboard selection (`main.rs:2345-2354`) — add Z/C/P (D2)

After the `KeyCode::W → SpectrumWidth` block (`main.rs:2352-2354`), inside the
same `if !dropdown_open && !modal_open {` guard:

```rust
        if is_key_pressed(KeyCode::Z) {
            select_product(state, Product::DifferentialReflectivity);
        }
        if is_key_pressed(KeyCode::C) {
            select_product(state, Product::CorrelationCoefficient);
        }
        if is_key_pressed(KeyCode::P) {
            select_product(state, Product::DifferentialPhase);
        }
```

`toggle::pressed(ply, &PRODUCT_OPTIONS)` (`main.rs:2389-2391`) already iterates
the array — no change; the three new buttons dispatch automatically.

### 4c. Toggle layout / width (`main.rs:1348`, `widgets/toggle.rs`)

`toggle::draw` (`widgets/toggle.rs:17-35`) lays each option out at `width(fit!())`
`height(fixed!(24.0))` in whatever parent row `main.rs:1348` sits in. Six
`fit!()` buttons will widen the controls bar. Implementer MUST:
- Read the parent container around `main.rs:1348` (the controls row) to confirm
  it either grows or scrolls; if it is a fixed-width row, six buttons may clip on
  narrow/mobile windows.
- If clipping: either shorten labels ("Refl", "Vel", "SW", "ZDR", "CC", "PhiDP")
  or allow the row to wrap. Do NOT add a new abstraction — reuse the existing
  layout primitives. Flag in the PR whichever choice was made.

**Verify:** `cargo check -p rustywx` ; run the app (see T7 smoke list), confirm 6
buttons render and each selects; `cargo fmt --check`.

---

## T5 — widgets/shortcuts.rs: document the new keys

**Tier: Haiku** (static text)
**Depends on: T4** (must match final key choices).
**File: `app/src/widgets/shortcuts.rs`**

In the PRODUCTS section (`shortcuts.rs:131-134`), after the "W → Spectrum Width"
row, add:

```rust
                            shortcut_row(ui, "Z", "Differential Reflectivity");
                            shortcut_row(ui, "C", "Correlation Coefficient");
                            shortcut_row(ui, "P", "Differential Phase");
```

(Leave the pre-existing `W` double-listing — Spectrum Width in PRODUCTS and
watches in OVERLAYS, `shortcuts.rs:134` and `145` — untouched, per D2.)

**Verify:** `cargo check -p rustywx` ; visually confirm the shortcuts modal (`?`).

---

## T6 — CC-gating (render-time filter)

**Tier: Sonnet** (cross-file: signature change, elevation/azimuth alignment, settings UI, tests)
**Depends on: T2 (settings fields) + T3 (ScanData.correlation_coefficient, gated QC).**
**Files: `app/src/scope.rs`, `app/src/main.rs`, `app/src/widgets/settings.rs`.**

### 6a. `scope.rs` — thread CC into `clean_sweep` and `rasterize`

Change signatures to accept the matching CC sweep + gate config:

```rust
fn clean_sweep(
    sweep: &SweepData,
    product: Product,
    tdbz_kernel_size: usize,
    cc_sweep: Option<&SweepData>,
    cc_gate_enabled: bool,
    cc_gate_threshold: f32,
) -> SweepData { ... }

pub fn rasterize(
    sweep: &SweepData,
    product: Product,
    size_px: usize,
    max_range_km: f32,
    tdbz_kernel_size: usize,
    cc_sweep: Option<&SweepData>,
    cc_gate_enabled: bool,
    cc_gate_threshold: f32,
) -> Vec<u8> {
    let sweep = clean_sweep(sweep, product, tdbz_kernel_size,
                            cc_sweep, cc_gate_enabled, cc_gate_threshold);
    ...
}
```

### 6b. `scope.rs` — the CC-gating pass inside `clean_sweep`

Add, at the **start** of `clean_sweep` (before the dBZ floor / TDBZ block so a
suppressed gate is not also texture-processed), a block that only fires for
Reflectivity with gating enabled and a CC sweep present (fail-open otherwise):

```rust
    // CC-gating: null Reflectivity gates whose co-located correlation
    // coefficient is below threshold (non-meteorological echo — birds, chaff,
    // AP, ground clutter). Dual-pol REF/CC come from the same surveillance cut,
    // so they share gate geometry; align by nearest azimuth and equal index.
    if product == Product::Reflectivity
        && cc_gate_enabled
        && let Some(cc) = cc_sweep
        && !cc.radials.is_empty()
    {
        // Pre-sort CC azimuths once for nearest-azimuth lookup.
        let mut cc_order: Vec<usize> = (0..cc.radials.len()).collect();
        cc_order.sort_by(|&a, &b| {
            cc.radials[a].azimuth_deg.total_cmp(&cc.radials[b].azimuth_deg)
        });
        let cc_azimuths: Vec<f32> =
            cc_order.iter().map(|&i| cc.radials[i].azimuth_deg).collect();

        for radial in &mut cleaned.radials {
            // Nearest CC radial by azimuth (reuse the existing helper; take its
            // primary index, ignore the interpolation weights — CC is a QC mask,
            // interpolating it would blur the birds/precip boundary).
            let (i1, _i2, _w1, _w2) =
                nearest_two_radial_indices(&cc_azimuths, radial.azimuth_deg);
            let cc_radial = &cc.radials[cc_order[i1]];
            for (i, gate) in radial.gates.iter_mut().enumerate() {
                if gate.is_some()
                    && let Some(Some(cc_val)) = cc_radial.gates.get(i)
                    && *cc_val < cc_gate_threshold
                {
                    *gate = None;
                }
            }
        }
    }
```

Notes for the implementer:
- `nearest_two_radial_indices` (`scope.rs:405-438`) is already `pub(crate)` and
  handles the wrap and the `MAX_GAP_DEG` fallback. Reuse it; do not write a new
  nearest-azimuth search.
- **Missing CC gate** (index out of range, or CC gate is `None`/below-threshold-
  in-CC-sense meaning unknown): leave the REF gate as-is (fail open per gate).
  Only an explicit CC value `< threshold` suppresses. This is intentional — a
  gap in CC must not blank valid reflectivity.
- Gate-index alignment assumes REF and CC share the first-gate range and spacing
  within the CS cut (true for dual-pol super-res). If CC has fewer gates,
  `.get(i)` returns `None` → fail open. Document this assumption in a comment.

### 6c. `scope.rs` — update the TDBZ test call (`scope.rs:1246`)

`tdbz_kernel_size_widens_clutter_removal_footprint` calls
`clean_sweep(&sweep, Product::Reflectivity, kernel_size)` — update to the new
arity: `clean_sweep(&sweep, Product::Reflectivity, kernel_size, None, false, 0.80)`
(gating off, no CC → behavior identical to before, test still valid).

### 6d. `scope.rs` — self-check test for CC-gating

Add to the test module (`scope.rs:1150+`):

```rust
    #[test]
    fn cc_gating_nulls_low_cc_ref_gate_and_keeps_high() {
        // REF radial: two live gates. CC radial (same azimuth): gate0 low
        // (0.55 — birds), gate1 high (0.98 — precip).
        let ref_sweep = SweepData {
            elevation_deg: 0.5,
            radials: vec![radial(0.0, vec![Some(30.0), Some(40.0)])],
        };
        let cc_sweep = SweepData {
            elevation_deg: 0.5,
            radials: vec![radial(0.0, vec![Some(0.55), Some(0.98)])],
        };
        let cleaned = clean_sweep(
            &ref_sweep, Product::Reflectivity, 9,
            Some(&cc_sweep), true, 0.80,
        );
        assert_eq!(cleaned.radials[0].gates[0], None,       "low-CC gate suppressed");
        assert_eq!(cleaned.radials[0].gates[1], Some(40.0), "high-CC gate preserved");

        // Disabled → both preserved.
        let ungated = clean_sweep(
            &ref_sweep, Product::Reflectivity, 9,
            Some(&cc_sweep), false, 0.80,
        );
        assert_eq!(ungated.radials[0].gates[0], Some(30.0));

        // Fail-open: no CC sweep → both preserved.
        let no_cc = clean_sweep(&ref_sweep, Product::Reflectivity, 9, None, true, 0.80);
        assert_eq!(no_cc.radials[0].gates[0], Some(30.0));
    }
```

> Caution: the dBZ floor (`scope.rs:37-43`) nulls REF gates below 20 dBZ within
> 20 km. Use gate values ≥ 20 (as above: 30/40 dBZ) and note that gate index i=0
> → range 2.125 km, floor 20.0 → 30/40 survive the floor, isolating the CC effect.
> The TDBZ pass with a 2-gate radial has `end - start < 2` guard so it won't
> interfere; confirm by keeping the two values close (30/40, |diff|²=100 >
> TDBZ_THRESHOLD 25 but `mean` 35 is NOT `< LOW_DBZ_GATE` 35 → not nulled). If
> the TDBZ pass interferes, widen to 3+ uniform gates. Verify empirically.

### 6e. `main.rs` — obtain the matching CC sweep and pass it to `rasterize`

At the raster prep (`main.rs:780-810`): the REF `sweep` is chosen by
`state.tilt_index` into `scan.sweeps(state.product)`. Build the matching CC sweep
by **nearest elevation** to the REF sweep being rasterized. Add, after `sweep`
is resolved (`main.rs:790`) and before the `rasterize` call (`main.rs:797`):

```rust
        // For CC-gating: find the CC sweep at the nearest elevation to the REF
        // sweep we are about to rasterize. Only needed for Reflectivity; None
        // for every other product (and when there is no dual-pol CC volume).
        let cc_sweep: Option<SweepData> = if state.product == Product::Reflectivity
            && state.settings.cc_gate_enabled
        {
            state.scan.as_ref().and_then(|scan| {
                let cc = &scan.correlation_coefficient;
                cc.iter()
                    .min_by(|a, b| {
                        (a.elevation_deg - sweep.elevation_deg)
                            .abs()
                            .total_cmp(&(b.elevation_deg - sweep.elevation_deg).abs())
                    })
                    .cloned()
            })
        } else {
            None
        };
```

Then extend the `scope::rasterize(...)` call (`main.rs:797-803`):

```rust
            let rgba = scope::rasterize(
                &sweep,
                state.product,
                scope::RASTER_SIZE_PX,
                scope::MAX_RANGE_KM,
                state.settings.tdbz_kernel.size() as usize,
                cc_sweep.as_ref(),
                state.settings.cc_gate_enabled,
                state.settings.cc_gate_threshold,
            );
```

> Elevation match uses nearest-elevation rather than exact because split cuts
> and the model's 0.2° dedup (`model.rs:189-193`) mean REF and CC elevations may
> differ slightly. `.cloned()` is acceptable — one sweep clone per re-raster,
> and re-raster only fires on `needs_reraster`. ponytail: clone is the cheap
> lazy choice; switch to borrowing if a profiler ever flags it.

### 6f. `widgets/settings.rs` — add the CC-gate toggle

- Add an ID constant near the other toggle IDs (`widgets/settings.rs:14-26`):
  ```rust
  pub const CC_GATE_TOGGLE_ID: &str = "settings-toggle-cc-gate";
  ```
- Add a `row(...)` with a `bool_toggle` in `draw(...)` — put it next to the
  scope-decoration toggles (after the SCOPE_RINGS row, `widgets/settings.rs:224-231`):
  ```rust
                      row(ui, "CC-gate reflectivity", |ui| {
                          bool_toggle(
                              ui,
                              CC_GATE_TOGGLE_ID,
                              "CC-gate reflectivity",
                              settings.cc_gate_enabled,
                          );
                      });
  ```
- The modal is fixed height `modal_h = 470.0` (`widgets/settings.rs:100`). Adding
  a 32px row may overflow; the content area is not scrollable (unlike the
  shortcuts modal). Bump `modal_h` to ~510.0, OR verify the new row fits. Flag
  the choice.

### 6g. `main.rs` — handle the toggle press

Alongside the other settings-panel handlers (after the SCOPE_RINGS handler,
`main.rs:2514-2517`):

```rust
        if ply.is_just_pressed(settings_widget::CC_GATE_TOGGLE_ID) {
            state.settings.cc_gate_enabled = !state.settings.cc_gate_enabled;
            state.cache.save_settings(&state.settings);
            state.needs_reraster = true;
        }
```

`needs_reraster = true` is essential — CC-gating is applied at raster time, so
the texture must rebuild when the toggle flips (mirrors the TDBZ_CYCLE handler,
`main.rs:2522-2526`).

**Verify:** `cargo test -p rustywx scope::` (includes the new CC-gating test) ;
`cargo check -p rustywx` ; `cargo fmt --check` ; run app, toggle CC-gate on a
clear-air/AP case (see T7).

---

## T7 — Integration + verification (final)

**Tier: Sonnet.**
**Depends on: all.**

### 7a. Grep gates (all must pass)

```bash
# All six products present in each exhaustive match:
grep -c "DifferentialReflectivity\|CorrelationCoefficient\|DifferentialPhase" \
  app/src/model.rs   # expect >= 12 (enum 3 + label 3 + units 3 + sweeps 3 + from_sweeps arms)
grep -n "zdr_color\|cc_color\|phidp_color" app/src/scope.rs   # 3 hits (color_of arms)
grep -n "ZDR_LEGEND\|CC_LEGEND\|PHIDP_LEGEND" app/src/main.rs  # 3 hits (legend match)
grep -n "btn-zdr\|btn-cc\|btn-phidp" app/src/main.rs           # 3 hits (PRODUCT_OPTIONS)
grep -n "KeyCode::Z\|KeyCode::C\|KeyCode::P" app/src/main.rs    # 3 hits (keyboard)
grep -n "cc_gate_enabled\|cc_gate_threshold" app/src/settings.rs # field + default + tests
grep -n "encode_sweeps\|decode_sweeps" app/src/cache.rs        # 6 encode + 6 decode calls total
grep -n "CC_GATE_TOGGLE_ID" app/src/widgets/settings.rs app/src/main.rs # decl + use + handler
```

### 7b. Build / test / lint

```bash
cargo fmt --check
cargo clippy -p rustywx --all-targets -- -D warnings
cargo test -p rustywx
cargo check -p rustywx --target wasm32-unknown-unknown   # if WASM is a supported target
```

### 7c. Manual smoke checklist (run the app)

1. Launch; select a site with active precip.
2. Click each of the 6 product buttons AND press R/V/W/Z/C/P — each switches the
   display and the legend/units update (ZDR "dB", CC blank, PhiDP "°").
3. Confirm legends render sensible colors: ZDR diverging (blue↔red through gray),
   CC banded with the 0.80–1.0 range emphasized, PhiDP a hue sweep.
4. Confirm ZDR/CC/PhiDP do NOT show the speckle/close artifacts — no dBZ floor
   holes, no despeckling of valid low values (D5).
5. On a clear-air or AP/biology case (early morning, or a known bird-roost
   bloom): Reflectivity ON, open Settings, toggle "CC-gate reflectivity" OFF →
   the low-CC bloom reappears; ON → it is suppressed. Texture rebuilds on toggle.
6. Confirm CC-gating never blanks precip cores (high CC preserved).
7. Restart the app → CC-gate setting persisted; old cache blob (if present) does
   not crash (logs a "corrupt/treating as cache miss" line at worst, then
   refetches).

### 7d. Report

Summarize: files touched, the D1–D6 decisions as-shipped (esp. any layout/modal
sizing choices from T4c/T6f), and any deviation. Confirm the pre-existing `W`
double-bind was left intact.

---

## Appendix — files and why each is touched

| File | Tasks | What changes |
|------|-------|--------------|
| `app/src/colors.rs` | T1 | 3 legends + 3 color fns + tests |
| `app/src/settings.rs` | T2 | `cc_gate_enabled` + `cc_gate_threshold` + tests |
| `app/src/model.rs` | T3 | Product 3 variants; ScanData 3 fields; from_sweeps; sweeps(); label/units; tests |
| `app/src/cache.rs` | T3 | encode/decode 3 groups (append); fixtures |
| `app/src/scope.rs` | T3, T6 | color_of arms + QC gating (T3); clean_sweep/rasterize CC params + gating pass + test (T6) |
| `app/src/main.rs` | T3, T4, T6 | legend match (T3); PRODUCT_OPTIONS + keyboard (T4); CC sweep resolve + rasterize args + settings handler (T6) |
| `app/src/widgets/toggle.rs` | T4 | (read only; possible label/layout tweak) |
| `app/src/widgets/shortcuts.rs` | T5 | 3 shortcut rows |
| `app/src/widgets/settings.rs` | T6 | CC-gate toggle ID + row (+ modal height) |
| `app/src/data.rs` | — | no change (delegates to from_sweeps) |
| `app/src/state.rs` | — | no change (`product`/`scan`/`settings` already generic) |
</content>
</invoke>
