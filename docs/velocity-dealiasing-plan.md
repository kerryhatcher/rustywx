# Velocity dealiasing — implementation plan

Unfold radial velocities beyond the Nyquist interval. Today the velocity product is
non-dealiased "purple haze": strong inbound/outbound couplets wrap past ±Nyquist and
render as the opposite sign, so mesocyclones and TVS signatures read as garbage. This is
the single biggest visible fix for the velocity product (`docs/radar-image-improvements-backlog.md`).

## Backlog correction — the data is NOT actually blocked

The backlog lists this as "hard — blocked on data source", claiming
`nexrad_model 1.0.0-rc.2` exposes no Nyquist/PRT field and that Nyquist must be computed
from VCP/PRT metadata (`va = λ/(4·PRT)`).

That is true only of the **model** layer. The **decode** layer we already depend on
transitively (`nexrad-decode 1.0.0-rc.3`) reads Nyquist straight out of the Message 31
Radial Data (Constant) Block:

- `nexrad-decode-1.0.0-rc.3/.../digital_radar_data/radial_data_block.rs:154`
  `pub fn nyquist_velocity(&self) -> Velocity` (raw `u16` × 0.01 m/s).
- Also exposes `unambiguous_range()` at the same block if ever needed.

The value is dropped when `nexrad-data` converts Message 31 → `nexrad_model::Radial`
(`into_radial()`), which is why the model `Radial` has no Nyquist accessor
(`nexrad-model-1.0.0-rc.2/src/data/radial.rs` — none of the accessors return it).

**So there is no PRT math to do.** We recover Nyquist with a second, cheap decode pass over
the same volume bytes the worker already downloads, keyed by elevation number. No upstream
PR required (a `nexrad-model` PR to carry Nyquist on `Radial` is the clean long-term fix,
but out of scope here).

---

## Data flow today

`data.rs::fetch_latest_scan` downloads a `nexrad_data::volume::File`, calls `file.scan()`
→ `nexrad_model::Scan`, then `ScanData::from_nexrad(&scan, timestamp)`
(`app/src/data.rs:82-84`). `ScanData::from_sweeps` splits per product into
`Vec<SweepData>` (`app/src/model.rs:174`). `SweepData` already carries per-tilt geometry
(`first_gate_km`, `gate_spacing_km`) added by the previous round.

`File` exposes the raw path we need:
- `file.records()` → `Vec<Record>` (`nexrad-data-1.0.0-rc.7/src/volume/file.rs:74`)
- `record.messages()` → `Vec<nexrad_decode::messages::Message>` (`.../volume/record.rs:71`)
- match `MessageContents::DigitalRadarData(m)`; then
  `m.header().elevation_number()` (`.../digital_radar_data/header.rs:83`) and
  `m.radial_data_block()?.nyquist_velocity()` (`DataBlock` derefs to inner).

Nyquist is constant within a Doppler cut, so one value per elevation number is exact.

---

## Stage A — Nyquist plumbing + readout (small, ship first)

Independently shippable, immediately visible: the scope readout currently hardcodes
`"Nyquist —"` (`app/src/model.rs:70-71`, called from `app/src/main.rs:3456`).

1. **`data.rs`**: after `file.scan()`, build `nyquist_by_elev: HashMap<u8, f32>` by walking
   `file.records()` → `record.messages()`, reading elevation number + `nyquist_velocity()`
   from each `DigitalRadarData` message. Decode errors on individual records are skipped
   (map entry simply absent → treated as unknown downstream). One extra decode pass over an
   already-in-memory buffer; negligible next to the network fetch.
2. **`model.rs`**: add `nyquist_ms: f32` to `SweepData` (0.0 = unknown), `#[serde(default)]`
   for cache back-compat exactly like the geometry fields. Change
   `from_nexrad`/`from_sweeps` to accept `&HashMap<u8, f32>` and look up
   `sweep.elevation_number()` per sweep (model `Sweep::elevation_number()` exists,
   `nexrad-model-1.0.0-rc.2/src/data/sweep.rs`). Populate only where the map has a value.
   Only velocity/spectrum-width sweeps care, but storing it on every `SweepData` is simplest
   and harmless.
3. **`cache.rs`**: persist `nyquist_ms` in `encode_sweeps`/`decode_sweeps`
   (`app/src/cache.rs:336-341,354`); JSON self-heals via serde default.
4. **`main.rs` synthetic_sweep** + **scope.rs test `radial()`/fixtures**: add the field
   (compile-forced by the struct change).
5. **Readout**: replace `format_nyquist_velocity()` stub with a function taking the current
   velocity sweep's `nyquist_ms`, formatting `"Nyquist ±26.4 m/s"` (or `"—"` when 0.0).
   Wire the active sweep's value in at `main.rs:3456`.

**Tests:** decode-map extraction from a fixture volume (or a hand-built message set);
round-trip `nyquist_ms` through cache; `format_nyquist_velocity` formats value and `—`
fallback.

**Ship A alone.** Real Nyquist on screen, no algorithm risk. Semver: patch/minor.

---

## Stage B — 1D gate-to-gate unfold (the core fix)

New module `app/src/dealias.rs`. Pure function over one radial:

```
fn dealias_radial(gates: &[Option<f32>], nyquist: f32) -> Vec<Option<f32>>
```

Algorithm (FMH-11B §4.3.3):
- Nyquist interval `nq = 2·nyquist`.
- Walk gates outward. Track last dealiased value `prev`. Seed `prev` from the first valid
  gate (assumed within the primary interval).
- For each valid gate `v`, choose integer `k` minimizing `|(v + k·nq) − prev|`; output
  `v + k·nq`; update `prev`. `None`/range-folded gates pass through and do not update
  `prev` (bridge small gaps; reset `prev` after a run of N≈5 missing gates so a distant
  unrelated echo doesn't inherit a stale reference).
- No-op when `nyquist <= 0.0` (unknown) — return input unchanged.

**Where it runs:** in `scope.rs::rasterize`, on `Product::Velocity`, before the existing
velocity spatial-SD censoring (`app/src/scope.rs:93-97`), gated by a new
`vel_dealias_enabled` setting mirroring `vel_sd_censor_enabled`. Dealias first (needs raw
gate-to-gate continuity), then SD-censor the residue.

- **ponytail:** recomputed per render, per sweep. Velocity is one sweep at a time — cheap.
  Ceiling: if profiling shows cost, memoize on the sweep at ingestion instead. Comment says so.

**Tests:**
- Synthetic radial with one fold (values jump `+28 → −25` across `nyquist=26.4`) unfolds to a
  monotone ramp.
- `nyquist=0.0` → identity.
- Missing-gate run resets the reference (no runaway unfold across a data gap).
- Rasterize integration: a folded couplet renders as same-sign gradient, not sign-flip.

Semver: minor. This is the payload the whole plan exists for.

---

## Stage C — 2D azimuthal continuity (refinement, later)

Upgrade to operational-grade (FMH-11D §3.3.3). Only after B is validated on real volumes.

- Also check azimuthal continuity against the previous dealiased radial.
- **Preserve couplets:** allow large gate-to-gate jumps over short distance so
  mesocyclone/TVS shear is NOT flattened — this is the failure mode of naive 2D smoothing.
- Isolated suspect gates fall back to a VAD environmental-wind estimate.
- Reject-and-reinsert after 5 consecutive failures; stop azimuthal propagation after 5 bad
  radials.

Needs a VAD wind estimate (new, self-contained sub-task) and processing the sweep as a 2D
array rather than independent radials — so C likely moves dealiasing to a
compute-once-at-ingestion pass and drops the per-render approach from B.

Semver: minor. Defer until B ships and is eyeballed against known couplet cases.

---

## Ordering & risk

Work order (each step compiles, tests pass):
1. Stage A: `data.rs` map → `model.rs` field + signature → `cache.rs` → fixtures/readout.
2. Stage B: `dealias.rs` + unit tests → wire into `rasterize` behind toggle → integration test.
3. Stage C: separate branch/PR after B is field-validated.

Risks / notes:
- **Split cuts:** one elevation number can carry separate surveillance (CS) and Doppler (CD)
  sub-cuts at different PRFs. Velocity radials carry the Doppler Nyquist; keying the map by
  elevation number from the same Message 31 radials that hold velocity keeps them
  consistent. Verify on a VCP 212 volume that velocity sweeps get a non-zero Nyquist.
- **Legacy vs modern radial block:** `nexrad-decode` handles both variants behind
  `nyquist_velocity()`; no branching needed in the app.
- **Unknown Nyquist:** any gap (decode failure, missing block) → `nyquist_ms = 0.0` → dealias
  is a no-op and the readout shows `—`. Never guess.
- Recommend one implementer per stage, one PR each, to avoid `SweepData` literal conflicts
  across the four files that construct it (same lesson as the geometry round).

## Source map

| Piece | Source |
|---|---|
| Nyquist from Message 31 | `nexrad-decode` radial_data_block `nyquist_velocity()` |
| Gate-to-gate unfold | FMH-11B §4.3.3 |
| 2D continuity, couplet preservation, VAD fallback | FMH-11D §3.3.3 |
