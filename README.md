# rustywx

## License

- **Code:** [AGPL-3.0-only](LICENSE)
- **Custom graphics, artwork, and assets:** [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/legalcode.en)

---

A Rust desktop radarscope for the Macon, GA area. Fetches live NEXRAD
Level II data for KJGX (Robins AFB) from the public AWS archive bucket
(`unidata-nexrad-level2`), decodes it, and renders a classic PPI scope
with the ply-engine graphics framework.

## Features

- Reflectivity, Velocity, and Spectrum Width products
- Elevation tilt selector with current VCP and Nyquist velocity display
- Configurable auto-refresh (default 2 minutes)
- Range rings, cardinal spokes, city markers, and state border lines
- NWS warning/watch polygon overlay and state/county borders
- National Hurricane Center (NHC) tropical storm overlay
- Configurable settings panel: default site, poll interval, overlay defaults, animation level, TDBZ clutter filter sensitivity
- Keyboard shortcuts overlay (press ?)

## Run

    just run

Or manually:

    cd ply-spike && cargo run --release

**Note:** The current working directory matters — assets are loaded from the `ply-spike/` 
directory, so the above paths are required.

Requires network access. No AWS credentials needed — the bucket is public.

See [`USER_GUIDE.md`](USER_GUIDE.md) for a full walkthrough of the
controls, how to read the display, and configuring settings.

## Test

    cargo test                                  # unit tests, no network
    cargo test --test network -- --ignored      # live end-to-end fetch/decode

## Architecture

The app is the `rustywx` crate under `ply-spike/` (ply-engine backend):

- `ply-spike/src/main.rs` — app entry, async game loop, and frame drawing
- `ply-spike/src/state.rs` — app state (selected site, overlays, animation)
- `ply-spike/src/data.rs` — background worker: poll S3 → download → decode → channel
- `ply-spike/src/model.rs` — thin scan model (product → sweeps → radials → gates)
- `ply-spike/src/scope.rs` — PPI rasterizer and overlay painting
- `ply-spike/src/colors.rs` — NWS-style color tables
- `ply-spike/src/geo.rs` — range/bearing and polar→screen projection
- `ply-spike/src/cities.rs` — city markers for the scope overlay
- `ply-spike/src/borders.rs` — fetches/caches US state boundary lines for the overlay
- `ply-spike/src/alerts.rs`, `nhc.rs` — NWS alerts and NHC tropical overlays
- `ply-spike/src/cache.rs`, `rle.rs` — Ply-storage scan cache + RLE compression
- `ply-spike/src/widgets/` — reusable glass-panel UI widgets

Design docs live in `docs/superpowers/`. For build/test/lint commands, the
module map, and how to extend the app, see
[`CONTRIBUTING.md`](CONTRIBUTING.md).
