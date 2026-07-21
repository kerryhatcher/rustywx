# Contributing to rustywx

This is a reference for working on rustywx's code: building, testing, the
module layout, the invariants worth knowing before you touch something, and
how to extend it. For what the app *does* from a user's perspective, see
[`docs/USER_GUIDE.md`](docs/USER_GUIDE.md).

## Prerequisites

- Rust with **edition 2024** support (rustc 1.85+; this project was built
  and tested against 1.94).
- Network access for the live-network test and for actually running the app
  — no credentials needed, the NEXRAD archive bucket is public.

## Building and running

```
cd ply-spike
cargo build                    # debug build
cargo run --release            # run the app; release mode matters, see below
just run                       # alias for `cargo run --release`
```

Debug builds are fine for `cargo check`/`cargo test`, but running the actual
app in debug mode makes rasterizing a sweep (see [`src/scope.rs`](ply-spike/src/scope.rs))
noticeably slow. Always use `--release` when running the app interactively.

## Testing

```
cd ply-spike
cargo test                                         # unit tests — no network required
cargo test --test network -- --ignored             # live NEXRAD fetch/decode
cargo test --test borders_network -- --ignored     # live TIGERweb state-border fetch
```

The unit test suite (`src/*.rs`, inline `#[cfg(test)]` modules) covers
geometry math, color banding, scan-model conversion, and rasterization, all
using synthetic data built from `nexrad-model`'s public constructors — no
network access needed, and they run in milliseconds.

[`tests/network.rs`](ply-spike/tests/network.rs) is `#[ignore]`d by default because it
downloads and decodes a real volume from S3. Run it explicitly when you
change anything in [`src/data.rs`](ply-spike/src/data.rs)'s fetch path, or after
bumping the `nexrad-data`/`nexrad-model` dependency versions.

## Linting and formatting

```
cd ply-spike
cargo fmt
cargo clippy
```

Run both before committing. There's no CI enforcing this yet, so it's on the
honor system — `cargo fmt` in particular should be run right before a commit
(see the `style: cargo fmt` commit in the project history for the pattern).

## Architecture

Data flows in one direction, from network to pixels:

```
S3 archive bucket
      │  poll every 2 min (src/data.rs, background thread + tokio runtime)
      ▼
nexrad-data / nexrad-model (raw decode)
      │  ScanData::from_nexrad (src/model.rs)
      ▼
ScanData (rustywx's own thin model)
      │  mpsc channel → UI thread
      ▼
RadarApp (src/app.rs) — picks product + tilt, tracks what needs redrawing
      │  scope::rasterize (src/scope.rs)
      ▼
pixel buffer → scope::draw_scope (overlays: rings, spokes, cities, borders, legend)
```

| Module | Responsibility |
|---|---|
| [`src/data.rs`](ply-spike/src/data.rs) | Background worker thread: polls S3 for the latest volume, downloads, decodes, reports status/errors/new scans over an `mpsc` channel. Owns retry/backoff. |
| [`src/model.rs`](ply-spike/src/model.rs) | `ScanData` — a thin, rendering-oriented model built from `nexrad-model`'s `Scan`. Converts raw moment values into `Option<f32>` gates and splits/sorts/dedups sweeps per product. |
| [`src/scope.rs`](ply-spike/src/scope.rs) | Rasterizes one sweep into an RGBA `ColorImage` (inverse polar mapping — for each pixel, find its azimuth/range and look up the nearest radial/gate), and paints all overlays (rings, spokes, city markers, legend, timestamp). |
| [`src/colors.rs`](ply-spike/src/colors.rs) | NWS-style stepped color tables for dBZ and velocity, plus the lookup functions the rasterizer calls per-gate. |
| [`src/geo.rs`](ply-spike/src/geo.rs) | Great-circle range/bearing (haversine) and polar-to-screen-pixel projection. Also holds the KJGX coordinates and the city list. |
| [`src/borders.rs`](ply-spike/src/borders.rs) | Loads US state boundary lines for the scope overlay: checks `~/.rustywx/state_borders.geojson`, fetching it from the Census TIGERweb REST API on first run if missing, then reports parsed rings to the UI over its own one-shot channel. |
| [`src/app.rs`](ply-spike/src/app.rs) | Ply-engine app: owns UI state (selected product/tilt, current scan, cached texture), drains worker messages, and manages the display. |

### Why a separate `ScanData` model instead of using `nexrad-model` directly

`nexrad-model`'s `Scan`/`Sweep`/`Radial` types are decode-oriented and carry
every moment (reflectivity, velocity, spectrum width, etc.) together, with
raw threshold/range-folding semantics baked into `MomentValue`. Rendering
code wants something flatter: "give me this product's sweeps, and each gate
is either a plain `f32` or nothing to draw." `ScanData` does that conversion
once, in `ScanData::from_nexrad`, so `scope.rs` never has to know about
`MomentValue` variants or which accessor method corresponds to which
product.

## Invariants worth knowing before you change things

These aren't enforced by types, so it's easy to break them silently:

- **`None` gates are drawn transparent, not zero.** A gate is `None` for two
  different reasons — below the minimum detectable threshold, or range-folded
  (ambiguous return) — and both are treated identically at render time
  (transparent). If you ever need to distinguish them visually, that
  distinction needs to be threaded through from `model.rs`.
- **Split cuts produce near-duplicate elevations.** VCP scans often sample
  the same elevation angle twice in a row (e.g. reflectivity-only, then
  velocity-only) as separate NEXRAD "sweeps." `ScanData::from_sweeps`
  dedups any two sweeps within 0.2° of each other for a given product,
  keeping the first. If a new VCP pattern has legitimately close (but
  distinct) elevations, this threshold may need revisiting.
- **The topmost tilt is silently absent.** `Sweep::from_radials` in the
  underlying `nexrad-model` decode step drops the final elevation group.
  This is an accepted limitation, not a bug to "fix" locally — it would need
  to be addressed upstream in `nexrad-model` or by decoding radials directly.
- **`Identifier` sorts by name, and name embeds the timestamp** — this is
  why `fetch_latest_scan` can just call `.max()` on the file listing to get
  the newest volume, with no separate timestamp parsing/sorting step. Don't
  reintroduce manual date comparison here; it's redundant and a source of
  subtle bugs if the two ever disagree.
- **The projection is azimuthal equidistant, not a true map projection.**
  Acceptable and visually indistinguishable from correct at the app's
  230 km max display range; do not extend `MAX_RANGE_KM` significantly
  without reassessing whether a proper projection is needed.
- **Retry backoff**: 0 errors → normal 2-minute poll; each consecutive
  failure doubles the delay starting at 30s, capped at 600s. Resets to the
  normal interval on the next success. See `retry_delay` in `ply-spike/src/data.rs`.

## How to extend

**Add a new radar product (e.g. Spectrum Width):**
1. Add a variant to `Product` in `ply-spike/src/model.rs` and give it a `label()`.
2. Add a field to `ScanData` and populate it in `from_sweeps`, following the
   `reflectivity`/`velocity` pattern (pull the right accessor off `Radial`).
3. Update `ScanData::sweeps()` to match the new variant.
4. Add a color table to `ply-spike/src/colors.rs` and wire it into the `color_of`
   match in `scope::rasterize` and the `legend`/`unit` match in
   `scope::draw_scope`.
5. Add a button for it in `ply-spike/src/app.rs`'s control bar.
6. Add unit tests mirroring the existing ones in `model.rs`/`colors.rs`.

**Add a new city marker:**
Add an entry to `CITIES` in `ply-spike/src/geo.rs` — `(name, lat, lon)`. That's it;
`scope::draw_scope` iterates the list and computes range/bearing
automatically, skipping anything beyond `MAX_RANGE_KM`.

**Change the color legend or add a band:**
Edit `DBZ_LEGEND` / `VELOCITY_LEGEND` in `ply-spike/src/colors.rs`. Keep entries sorted
ascending by threshold — `banded()` and the `legends_are_ascending` test both
assume this.

**Change the poll interval:**
Adjust `POLL_INTERVAL` in `ply-spike/src/data.rs`. Consider whether `retry_delay`'s cap
(600s) still makes sense relative to a much shorter or longer interval.

**Point at a different radar site:**
Change `SITE` in `ply-spike/src/data.rs` and `KJGX_LAT`/`KJGX_LON` in `ply-spike/src/geo.rs`
(and probably the window title in `ply-spike/src/main.rs`, and the `CITIES` list).

## Commit and PR conventions

- Use [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`,
  `fix:`, `docs:`, `style:`, `refactor:`, `chore:`) — see `git log` for the
  established pattern.
- Run `cargo fmt`, `cargo clippy`, and `cargo test` before opening a PR.
  If your change touches `src/data.rs`'s fetch/decode path, also run the
  ignored network test.
- Design history (the original architecture spec and the TDD implementation
  plan this project was built from) lives under
  [`docs/superpowers/`](docs/superpowers/) if you want background on why
  certain choices were made.
