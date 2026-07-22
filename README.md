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

    cd app && cargo run --release

**Note:** The current working directory matters — assets are loaded from the `app/` 
directory, so the above paths are required.

Requires network access. No AWS credentials needed — the bucket is public.

See [`USER_GUIDE.md`](USER_GUIDE.md) for a full walkthrough of the
controls, how to read the display, and configuring settings.

## Test

    cargo test                                  # unit tests, no network
    cargo test --test network -- --ignored      # live end-to-end fetch/decode

## Architecture

The app is the `rustywx` crate under `app/` (ply-engine backend):

- `app/src/main.rs` — app entry, async game loop, and frame drawing
- `app/src/state.rs` — app state (selected site, overlays, animation)
- `app/src/data.rs` — background worker: poll S3 → download → decode → channel
- `app/src/model.rs` — thin scan model (product → sweeps → radials → gates)
- `app/src/scope.rs` — PPI rasterizer and overlay painting
- `app/src/colors.rs` — NWS-style color tables
- `app/src/geo.rs` — range/bearing and polar→screen projection
- `app/src/cities.rs` — city markers for the scope overlay
- `app/src/borders.rs` — fetches/caches US state boundary lines for the overlay
- `app/src/alerts.rs`, `nhc.rs` — NWS alerts and NHC tropical overlays
- `app/src/cache.rs`, `rle.rs` — Ply-storage scan cache + RLE compression
- `app/src/widgets/` — reusable glass-panel UI widgets

Design docs live in `docs/superpowers/`. For build/test/lint commands, the
module map, and how to extend the app, see
[`CONTRIBUTING.md`](CONTRIBUTING.md).
