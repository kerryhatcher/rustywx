# rustywx

A Rust desktop radarscope for the Macon, GA area. Fetches live NEXRAD
Level II data for KJGX (Robins AFB) from the public AWS archive bucket
(`unidata-nexrad-level2`), decodes it, and renders a classic PPI scope
with egui.

## Features

- Base reflectivity and base velocity products
- Elevation tilt selector
- Auto-refresh (polls for new volume scans every 2 minutes)
- Range rings, cardinal spokes, city markers, and state border lines
- NWS warning/watch polygon overlay (refreshed every 2 minutes)

## Run

    cargo run --release

Requires network access. No AWS credentials needed — the bucket is public.

See [`docs/USER_GUIDE.md`](docs/USER_GUIDE.md) for a full walkthrough of the
controls and how to read the display.

## Test

    cargo test                                  # unit tests, no network
    cargo test --test network -- --ignored      # live end-to-end fetch/decode

## Architecture

- `src/data.rs` — background worker: poll S3 → download → decode → channel
- `src/model.rs` — thin scan model (product → sweeps → radials → gates)
- `src/scope.rs` — PPI rasterizer and overlay painting
- `src/colors.rs` — NWS-style color tables
- `src/geo.rs` — range/bearing and polar→screen projection
- `src/app.rs` — egui application state and layout
- `src/borders.rs` — fetches/caches US state boundary lines for the scope overlay

Design docs live in `docs/superpowers/`. For build/test/lint commands, the
module map, and how to extend the app, see
[`CONTRIBUTING.md`](CONTRIBUTING.md).
