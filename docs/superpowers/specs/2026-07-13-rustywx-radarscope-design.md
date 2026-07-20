# rustywx — NEXRAD Radarscope Design

**Date:** 2026-07-13
**Status:** Approved design, pending implementation plan

## Purpose

A cross-platform Rust desktop GUI app that fetches live NEXRAD Level II radar
data for KJGX (Robins AFB, the radar covering Macon, GA) and displays it as a
classic PPI radarscope centered on the radar antenna, with Macon shown as a
geographic marker.

## Decisions

| Question | Decision |
|---|---|
| Data source | NEXRAD Level II raw volume scans from the public AWS S3 archive bucket (`unidata-nexrad-level2`) |
| Radar site | KJGX (lat 32.6755, lon -83.3511) |
| GUI toolkit | egui / eframe |
| Scope center | The radar antenna (classic PPI); Macon rendered as a marker ~20 mi NW of center |
| Rendering | CPU-rasterize the selected sweep to an RGBA image, upload as an egui texture once per new scan or product/tilt change; overlays painted with the egui painter each frame |
| Decode | `nexrad-data` / `nexrad-decode` / `nexrad-model` crates (1.0.0-rc.x) |

## v1 Features

1. Fetch and display the latest base reflectivity sweep on a dark PPI scope.
2. **Auto-refresh:** poll S3 every ~2 minutes (configurable constant) and update
   the scope when a new volume scan arrives.
3. **Product toggle:** base reflectivity ⇄ base velocity, each with an
   NWS-style color table.
4. **Elevation tilt selector:** choose which elevation sweep of the volume to
   display (default: lowest tilt).
5. **Geographic overlays:** range rings (every 50 km out to ~230 km), cardinal
   direction spokes/labels, markers for Macon and Warner Robins, station ID,
   scan timestamp (UTC and local), and a color scale legend.

Out of scope for v1: pan/zoom, animation loops of historical frames, multiple
radar sites, real-time chunk feed (`unidata-nexrad-level2-chunks`), storm
tracking products.

## Post-v1

Two features deferred past v1.0.0 are tracked in a dedicated planning document:
[`docs/post-v1-multi-site-animation.md`](../../post-v1-multi-site-animation.md).

- **Multi-site integration** — combining data from multiple NEXRAD sites into a
  single unified view (CAPPI at a fixed altitude rather than a single sweep).
  Informed by Yi Ru (2007), *Volumetric Visualization of NEXRAD Level II Doppler
  Weather Data from Multiple Sites*.
- **Temporal animation** — playing back a sequence of historical volume scans
  as a smooth animation with temporal interpolation between frames.

## Architecture

Two threads:

- **UI thread** — eframe/egui event loop. Never performs I/O or decoding.
- **Worker thread** — owns a small tokio runtime. Loop: list the newest KJGX
  volume file on S3 → download → decode → send the decoded scan over a channel
  → sleep until next poll. On failure, send an error status and retry next
  cycle with backoff.

Communication is a `std::sync::mpsc` channel carrying a `WorkerMessage` enum
(`NewScan(Scan)`, `Status(String)`, `Error(String)`). The UI calls
`ctx.request_repaint()` when messages arrive (worker holds an
`egui::Context` clone to wake the UI).

## Modules

- `main.rs` — wiring: spawn worker, start eframe.
- `app.rs` — eframe `App` impl. Holds latest `Scan`, UI state (selected
  product, selected tilt, status line, last-update time), drains the channel,
  triggers re-rasterization when scan/product/tilt changes.
- `data.rs` — worker loop and fetch/decode. Converts `nexrad-model` types into
  our own thin `Scan` model (`Scan → Sweep(elevation) → Radial(azimuth) →
  gates: Vec<Option<f32>>` per product, plus gate spacing/range metadata and
  scan timestamp). Fetch is behind a small trait so decode/rasterize logic can
  be tested from a local fixture file without network access.
- `scope.rs` — rendering. `rasterize(sweep, product, size) → egui::ColorImage`
  (polar→cartesian mapping, radar at center, north up), and overlay painting
  (rings, spokes, markers, labels, legend) via `egui::Painter`.
- `colors.rs` — color tables: `dbz_color(f32) → Color32` and
  `velocity_color(f32) → Color32`, NWS-style ramps; `None`/below-threshold
  gates are transparent.

## Data Flow

S3 archive bucket → bytes → `nexrad` decode → our `Scan` → channel → app
state → rasterize (on change only, not per frame) → egui texture → painter
draws texture + overlays.

## Geographic Math

- Scope projection: azimuthal equidistant approximation centered on KJGX —
  adequate at ≤230 km range for display purposes.
- City markers: compute great-circle range/bearing from KJGX to the city
  lat/lon, then place using the same polar→pixel mapping as radar gates.
- Macon (32.8407, -83.6324) and Warner Robins (32.6130, -83.6242) included as
  a small const table so more cities are trivial to add.

## Error Handling

- Network or decode failure: UI keeps showing the last good scan; status bar
  shows the error and next retry time. The app never panics on bad data —
  decode errors are contained in the worker.
- No scan yet (startup): scope renders rings/overlays with a "fetching…"
  status.
- Missing product/tilt in a volume (velocity absent on some tilts due to
  split cuts): tilt selector lists only tilts present for the selected
  product; fall back to nearest available tilt.

## Testing

- Unit tests: color table mapping at boundary values; polar→cartesian gate
  mapping (known azimuth/range → pixel); lat/lon → range/bearing (Macon ≈
  33 km at ≈ 305° from KJGX); tilt fallback logic.
- Integration test: an opt-in (`#[ignore]`-tagged) live network test that
  fetches and decodes the latest KJGX volume, asserting sweep/radial counts
  and a plausible dBZ range. (Chosen over a committed fixture file: Level II
  volumes are 5–15 MB binaries, and `nexrad-model`'s public constructors let
  unit tests build synthetic scans without any fixture.)
- Manual/visual verification of the running app for layout and colors.

## Dependencies (anticipated)

`eframe`, `egui`, `nexrad-data` (aws feature), `nexrad-decode`,
`nexrad-model`, `tokio` (rt, worker only), `chrono` (timestamps),
`anyhow`/`thiserror` (errors). Exact versions and feature flags verified
against docs during implementation planning.
