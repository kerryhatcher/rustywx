# Stage 4: "Borders & Alerts" — Map Overlays

**Status:** 🔲 Not started
**Tag:** `v0.2.0-stage4`

## Goal

State borders and NWS warning/watch polygons on the scope.

## Scope

- `borders.rs` — port fetch + parse. Replace `ureq` with Ply `net`:
  `net::get("borders", URL, |r| r)` then poll `net::request("borders")`
  each frame. Parse GeoJSON with `serde_json`. Cache parsed rings via
  Ply `storage`.
- `alerts.rs` — port fetch + parse. Replace `ureq` with Ply `net`.
  Replace `egui::Color32` with `[u8; 4]` arrays (matching `colors.rs`).
  Replace custom serde for Color32 with plain `[u8; 4]` serialization.
- Draw border line segments on scope
- Draw alert polygons on scope (clipped to radar circle)
- Alert labels near polygon centers
- Toggle overlays on/off

## Notes from Stage 1

- The scope is drawn directly to the screen (not through
  `render_to_texture`). Border and alert overlays should also be drawn
  directly to the screen in the `draw_scope_to_texture` function (or a
  renamed `draw_scope` function). See Stage 1 lesson: render_to_texture
  causes a 180° coordinate flip.
- The old `src/geo.rs` has `circle_subsegments()` and `point_to_km_offset()`
  functions for clipping polygons to the radar circle. These use
  `egui::Vec2` and need to be ported to plain `(f32, f32)` tuples or
  macroquad's `Vec2`. The 4 `circle_subsegments` unit tests from the old
  `src/geo.rs` were deferred from Stage 1 to this stage.
- Pan/zoom validation becomes obvious once borders are visible — the
  geographic reference makes movement clearly visible.

## Deliverable

State boundary lines and active NWS warnings/watches visible on the radar
scope.

## Validation

- [ ] State borders visible as subtle brown lines
- [ ] Coastlines visible
- [ ] Active warnings/watches appear as colored polygons
- [ ] Alert labels visible (e.g. "Severe Thunderstorm Warning")
- [ ] Alerts refresh every 2 minutes
- [ ] Overlays cached to disk via Ply `storage`
- [ ] No overlays when none are active (graceful empty state)
- [ ] `git push` → CI passes → `git tag v0.2.0-stage4` → `git push --tags`
