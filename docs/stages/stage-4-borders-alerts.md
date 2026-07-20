# Stage 4: "Borders & Alerts" — Map Overlays

**Status:** ✅ Complete

**Target tag:** `v0.2.0-stage4`

## Goal

State borders and NWS warning/watch polygons on the scope.

## Delivered

- `geo.rs` — ported `point_to_km_offset()` and `circle_subsegments()`
  from the old egui-era `src/geo.rs`. Uses `ply_engine::prelude::Vec2`
  (macroquad/glam `Vec2`) instead of `egui::Vec2`. The 4 deferred
  `circle_subsegments` unit tests are now in `ply-spike/src/geo.rs`.
- `borders.rs` — ported fetch + parse from `src/borders.rs`:
  - Replaces `ureq` with Ply `net`: fires three concurrent `net::get()`
    calls with distinct IDs (`borders-states`, `borders-coast`,
    `borders-country`), then polls `net::request()` each frame until
    all three complete.
  - Replaces filesystem JSON caching with Ply `storage` (key:
    `state_borders_v8`). Cached rings are loaded on startup; if no
    cache exists, the network fetch fires and the result is saved.
  - Keeps the pure GeoJSON parsing functions (`parse_state_lines`,
    `parse_coastlines`, `parse_country_lines`) with 5 unit tests.
  - `ring_affects_scope()` uses the ported `geo::point_to_km_offset`
    and `geo::circle_subsegments` for scope-intersection testing.
- `alerts.rs` — ported fetch + parse from `src/alerts.rs`:
  - Replaces `ureq` with Ply `net`: fires `net::get("alerts", URL, |c|`
    with a `User-Agent` header, then polls `net::request("alerts")`.
  - Replaces `egui::Color32` with `[u8; 4]` arrays throughout. The
    `Alert` struct serializes colors as plain arrays — no custom serde.
  - `alert_color()` returns `[u8; 4]` instead of `Color32`.
  - `alert_affects_scope()` filters alerts to those overlapping the
    current radar site's 230 km scope.
  - 4 unit tests cover color mapping and GeoJSON parsing.
- `scope.rs` — overlay drawing:
  - `draw_borders()` draws each ring's line segments, clipped to the
    radar circle via `circle_subsegments`. Subtle brown color
    (0x8b7355, alpha 180). Scope circle outline added for visual
    delimitation.
  - `draw_alerts()` draws each alert polygon's edges (clipped to the
    scope circle) with the NWS event color, plus a truncated event
    label near the polygon centroid.
  - `draw_scope_to_texture()` signature extended with optional
    `borders` and `alerts` parameters (each a tuple of data + show
    flag).
- `main.rs` — integration:
  - Loads cached borders on startup via a background task.
  - Fires border net requests when no cache is available.
  - Polls borders and alerts each frame; parses and caches on
    completion.
  - Fires alerts fetch on startup and re-fires every 120 seconds
    (POLL_INTERVAL).
  - Adds "Borders" and "Alerts" toggle buttons in the control bar
    with active/inactive styling and alert count.
  - Keyboard shortcuts: `B` toggles borders, `A` toggles alerts.
- `state.rs` — new `AppState` fields for borders, alerts, toggles,
  fetch state, and poll timing.
- `cache.rs` — added `storage()` accessor for use by borders caching.

## Implementation Data

| Item | Result |
|---|---|
| Border GeoJSON sources | 3 (states, coastlines, country lines) |
| Net request IDs | `borders-states`, `borders-coast`, `borders-country`, `alerts` |
| Storage key for borders cache | `state_borders_v8` |
| Alert poll interval | 120 seconds |
| New modules added | 2 (`borders.rs`, `alerts.rs`) |
| New unit tests (borders + alerts + geo clipping) | 13 |
| Workspace unit tests after Stage 4 | 28 |
| Overlay toggle keyboard shortcuts | `B` (borders), `A` (alerts) |

## Notes from Stage 1

- The scope is drawn directly to the screen (not through
  `render_to_texture`). Border and alert overlays are also drawn
  directly to the screen in the `draw_scope_to_texture` function.
  See Stage 1 lesson: render_to_texture causes a 180° coordinate flip.
- The old `src/geo.rs` `circle_subsegments()` and `point_to_km_offset()`
  functions were ported to `ply-spike/src/geo.rs` using
  `ply_engine::prelude::Vec2` (glam `Vec2`). The 4 `circle_subsegments`
  unit tests from the old `src/geo.rs` are now in the new `geo.rs`.
- Pan/zoom validation becomes obvious once borders are visible — the
  geographic reference makes movement clearly visible.

## Lessons Learned

### Ply net is idempotent — plan for re-fetch

`net::get(id, ...)` won't re-fire if a request with the same ID already
exists. For the alerts refresh cycle, after a response is received and
the entry is no longer polled, Ply's frame-based eviction (60 frames)
removes it, allowing the next `net::get()` to fire. This works for a
120-second poll interval but would need unique IDs per cycle for
faster refreshes.

### glam Vec2 method names differ from egui

`egui::Vec2` uses `length_sq()`; glam/macroquad `Vec2` uses
`length_squared()`. The `dot()` method name is the same. This is a
minor but easy-to-miss difference when porting math code.

### Alert scope filtering is per-site

The old code filtered alerts to the KJGX scope at parse time. The new
code parses all US alerts and filters at draw time using the current
radar site. This is correct because the user can switch sites without
re-fetching alerts.

## Validation

- [x] State borders visible as subtle brown lines
- [x] Coastlines visible
- [x] Active warnings/watches appear as colored polygons
- [x] Alert labels visible (e.g. "Severe Thunderstorm Warning")
- [x] Alerts refresh every 2 minutes (POLL_INTERVAL)
- [x] Overlays cached to disk via Ply `storage`
- [x] No overlays when none are active (graceful empty state)
- [x] `cargo fmt`, clippy, check, tests pass locally
- [x] Workspace unit tests: 28 passed (13 new)
- [x] `git push` → CI passes → `git tag v0.2.0-stage4` → `git push --tags`

## Follow-ups / Deferred Polish

- Consider coast vs. border color differentiation in Stage 6 visual
  design pass.
- Add a scrollbar or alert list panel in Stage 5 (NHC) or Stage 7
  (Settings & Polish).
- Replace fixed border color with per-source coloring (states vs.
  coastlines vs. international) during Stage 6.
- Revisit alert polygon fill (currently outline-only) for Stage 6
  visual design.
