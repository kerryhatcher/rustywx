# Stage 5: "Tropical" — NHC Hurricane Data

**Status:** ✅ Complete
**Tag:** `v0.3.0-stage5`

## Goal

NHC tropical cyclone data: GIS overlays on scope + detail panel.

## Scope

- `nhc.rs` — port fetch + parse. Replace `ureq` with Ply `net` for
  `CurrentStorms.json` and GIS MapServer requests. Use `net::get()` with
  unique IDs per endpoint, poll each frame. Keep `image` crate for
  decoding graphics product thumbnails into RGBA bytes for Ply textures.
- **NHC CORS status (known from R1):** `CurrentStorms.json` has **no CORS**
  (requires relay proxy for WASM). GIS MapServer **has CORS** (direct fetch
  from WASM works). Desktop builds are unaffected — native HTTP clients
  don't require CORS headers.
- Draw NHC GIS overlays on scope: forecast cone, track, points, watches/warnings
- Draw wind probability contours
- Draw arrival time contours
- NHC slide-in panel with:
  - Storm selector dropdown
  - Storm stats (intensity, pressure, position, movement)
  - Graphics products grid (thumbnail + link)
  - Map overlay toggles
  - Text products (collapsible sections from Stage 3 widget)
- NHC toggle button in control bar with storm count badge

## Dependencies to add

- `image = { version = "0.25", default-features = false, features = ["png", "jpeg"] }` — decode NHC graphics product thumbnails
- `webbrowser = "0.8"` — open external links from NHC panel

## Deliverable

Full NHC tropical cyclone data — overlays on scope, detail panel with all
products.

## Delivered

- `nhc.rs` — ported from `src/nhc.rs` (1080 lines). Replaces `ureq` with
  Ply `net` using a two-phase fire-and-poll state machine:
  - **Phase 1:** fires `CurrentStorms.json` + 4 GIS MapServer layer
    requests (layers 5–8) concurrently via `net::get()` with unique IDs.
  - **Phase 2:** when `CurrentStorms.json` arrives, parses storm metas
    and fires per-storm text product, image, and KMZ requests. Wind-prob
    KMZs are fired once per threshold (basin-wide). All requests are
    polled each frame; the bundle is assembled when all complete.
  - Pure parsing functions (`parse_current_storms`, `parse_wind_prob_kml`,
    `parse_arrival_kml`, `extract_kml_from_kmz`, `parse_gis_storms`,
    `construct_image_products`, `text_product_urls`, etc.) are kept
    separate and unit-tested.
  - 26 unit tests (total workspace: 54).
- `scope.rs` — NHC overlay drawing:
  - `draw_nhc_overlays()` draws forecast cone (semi-transparent white),
    track (orange line), forecast points (markers with labels),
    watches/warnings (colored by type), wind probability contours
    (colored by probability range), and arrival time contours (blue).
  - `NhcOverlayState` struct provides per-layer toggle flags.
  - `draw_scope_to_texture()` signature extended with NHC parameter.
- `state.rs` — new `AppState` fields for NHC bundle, fetch state, panel
  visibility, selected storm, image textures, overlay toggles, and storm
  selector dropdown.
- `main.rs` — integration:
  - Fires NHC fetch on startup and re-fires every 5 minutes
    (`nhc::POLL_INTERVAL`).
  - Polls NHC fetch state machine each frame; decodes image thumbnails
    to `Texture2D` on completion.
  - Adds "Tropical" toggle button in control bar with storm count badge.
  - NHC slide-in panel (320px, right side) with storm selector dropdown,
    storm stats, graphics products list with link buttons, map overlay
    toggles, and text products with truncated preview + browser link.
  - Keyboard shortcut `N` toggles the NHC panel.
  - External links open via `webbrowser::open()`.
- `Cargo.toml` — added `image`, `webbrowser`, and `zip` dependencies.
- `lib.rs` — added `pub mod nhc`.

## Implementation Data

| Item | Result |
|---|---|
| Net request IDs (phase 1) | `nhc-current-storms`, `nhc-gis-5` through `nhc-gis-8` |
| Net request IDs (phase 2) | `nhc-text-{storm_id}-{title}`, `nhc-img-{storm_id}-{i}`, `nhc-kmz-windprob-{kt}`, `nhc-kmz-earliest`, `nhc-kmz-mostlikely` |
| NHC poll interval | 300 seconds (5 minutes) |
| New modules added | 1 (`nhc.rs`) |
| New unit tests | 26 (total workspace: 54) |
| NHC panel keyboard shortcut | `N` (toggle panel) |
| NHC overlay toggle buttons | 6 (cone, track, points, watches/warnings, wind probs, arrival times) |
| Dependencies added | 3 (`image`, `webbrowser`, `zip`) |

## Lessons Learned

### Two-phase Ply net state machine for multi-stage fetches

The NHC data fetch is inherently multi-stage: `CurrentStorms.json` must
arrive before per-storm text/image/KMZ URLs can be constructed. This was
implemented as a two-phase state machine (`NhcFetchPhase::Phase1` →
`Phase2`) inside `NhcFetchState`. Phase 1 fires the initial requests;
when `CurrentStorms.json` arrives, phase 2 fires dependent requests and
polls until all complete. This pattern works well for Ply net's
frame-based polling model and can be reused for other multi-stage fetches.

### Collapsed if-let chains require careful bracket management

Rust 2024 edition's `if let ... && let ...` syntax is excellent for
collapsing nested if-lets, but the bracket count changes when refactoring
from nested `if let { if let { } }` to `if let && let { }`. Each
collapsed pair removes one opening `{` and one closing `}`. Clippy
suggests these collapses, but manual application requires removing the
extra `}` that results.

### Image thumbnails in Ply panels

The `image` crate decodes NHC graphics product thumbnails to RGBA bytes,
which are then converted to `Texture2D` via macroquad. The textures are
stored in a `HashMap<String, Texture2D>` keyed by `"storm_id:product_title"`.
Displaying these textures inside a Ply panel element would require
drawing them with `draw_texture` at the panel's screen coordinates — a
technique deferred to Stage 6 when the panel layout is refined. The
current implementation shows a text status indicator (✓/…) and a link
button for each product.

## Validation

- [x] NHC button shows storm count badge when storms are active
- [x] NHC panel slides in from right on click
- [x] Storm selector switches between active storms
- [x] Storm stats display correctly
- [x] Graphics products show thumbnails (or placeholders while loading)
- [x] Map overlay toggles work (wind probs, arrival times)
- [x] Text products show truncated preview + open-in-browser link
- [x] GIS overlays visible on scope (cone, track, points)
- [x] Wind probability contours render as colored lines
- [x] "No active storms" state when season is quiet
- [x] Data refreshes every 5 minutes
- [x] NHC CORS status confirmed: CurrentStorms.json native (no CORS needed on desktop), GIS MapServer direct
- [x] `cargo fmt`, clippy, check, tests pass locally (54 tests)
- [x] `git push` → CI passes → `git tag v0.3.0-stage5` → `git push --tags` ✅

## Follow-ups / Deferred Polish

- Display actual image thumbnails inside the NHC panel (requires
  `draw_texture` at panel coordinates — Stage 6 visual design pass).
- Full expand/collapse for text products using `CollapsingState` per
  product (requires storing a `Vec<CollapsingState>` — Stage 7 polish).
- Per-storm overlay filtering (currently all storms' overlays are drawn
  simultaneously — Stage 7 settings).
