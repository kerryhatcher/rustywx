# Sparse, legible alert labels + legend + click-for-detail

## Context

When many watch/warning polygons cluster in one area (e.g. a tropical system near the coast), `draw_alerts()` in `app/src/scope.rs` draws one label per alert unconditionally at its on-screen centroid, with no collision avoidance and no dedup. Overlapping polygons stack their text on top of each other and become unreadable (see reported screenshot: "Tropical Storm Warning" repeated ~5x, illegible). The city-label code directly above `draw_alerts` in the same function already solves this exact problem for city names (zoom-based filtering + greedy rectangle-collision placement) — we reuse that pattern for alerts instead of inventing a new one.

User wants: sparse (not hidden) labels, per-type color/legend, and click-to-detail — not label removal.

## Approach

### 1 & 2. Dedup + collision-sparse labels (scope.rs `draw_alerts`)

Rework `draw_alerts` to mirror the city-label block (`scope.rs:493-533`):
- Draw all polygon outlines first (for every alert passing the existing watch/warning toggle), in a pass that does *not* touch labels — so the map stays fully colored regardless of label thinning.
- **One label candidate per alert (per-polygon centroid) — collision pass does the deduping, not a global event-string dedup.** The reported bug ("Tropical Storm Warning" ×5 illegible) is same-event polygons stacked *near-coincident on screen* — their centroids are close, so the bbox-collision loop already drops the overlapping ones. Global event-dedup is the wrong tool: if two same-event centroids are far enough apart that their bboxes don't collide, their labels don't visually overlap either, so both *should* draw — collapsing them to one label would silently lose a legitimate label for a geographically separate alert (e.g. two Tornado Warnings 100 km apart). So keep one candidate per alert and let collision thin them. Needs no new fields on `Alert`.
- Build `candidates: Vec<(&str, f32, f32, [u8;4])>` of (event label, screen cx, cy, color), one entry per on-screen alert.
- Sort candidates for a deterministic collision pass (e.g. by centroid x then y — no population-style priority exists for alerts, so any stable order works).
- Run the same greedy `placed: Vec<(f32,f32,f32,f32)>` rectangle-collision loop, **local to `draw_alerts`** (alert-vs-alert only — the city block's `placed` lives in a separate function and is not threaded in, so no signature change): if a label's bbox collides with an already-placed *alert* label, skip drawing that text. The outline was already drawn in the first pass.
- Net: collision removes the near-coincident stacking; spatially separate labels (even same event type) all draw; outlines always draw.
- `// ponytail:` if many tiny adjacent counties of one event still feel noisy after collision, add a proximity-cell dedup (round centroid to an N-px grid, one label per cell) — do *not* dedup globally by event.

### 4. Zoom-density gating (part of the same rework)

Reuse the `px_per_km` value already passed into `draw_alerts` (no need to thread `zoom` through) as a density signal: skip adding a candidate label at all if `px_per_km` is below a small fixed threshold (zoomed way out — outlines stay visible, labels disappear until you zoom in). Mirrors the spirit of `cities::min_population_for_zoom` but keeps the diff smaller since `px_per_km` is already in scope. Pick the threshold empirically while running the app; leave a `// ponytail: tuned by eye` note on the constant — it's a visual knob, not a derivable value.

### 5. Legend (main.rs status bar)

Mirror the existing dBZ/velocity legend swatch-row pattern (`main.rs:1799-1834`, arrays of `(label, color)` rendered as small colored boxes + text). Build the legend list dynamically each frame from `state.alerts`: dedup by `event` string, keep `(event, color)`, cap to a handful of entries to avoid the legend itself becoming clutter (e.g. take first N distinct types). If more than N distinct types are active, append a `+K more` marker so the cap doesn't silently read as a complete key. Render only when `state.show_watches || state.show_warnings` and `!state.alerts.is_empty()`, in the same status-bar area as the other legends. Static key only — no filter-toggle interactivity (per your answer).

### 6. Click-for-detail modal

- **Hit testing**: add `alerts::hit_test(alerts: &[Alert], show_watches: bool, show_warnings: bool, site: &RadarSite, click_screen: (f32,f32), center: (f32,f32), px_per_km: f32) -> Option<&Alert>` in `app/src/alerts.rs`. (7 params, mirrors `draw_alerts`'s projection args — fine as-is; no context struct needed for one call site.) Skip alerts whose category is toggled off (same `is_watch` filter as `draw_alerts` at scope.rs:648-656) so clicks never open a modal for a hidden alert. For each remaining alert's rings, project points to screen space the same way `draw_alerts` does, then run a standard ray-casting point-in-polygon test against the click point. Test each ring independently and treat a hit on any ring as a hit (handles MultiPolygon). `// ponytail:` note that inner rings (holes) are counted as inside, not subtracted — NWS alert polygons effectively never carry holes, so full even-odd hole handling is not worth the code. **Return the last match, not the first** — draw order is array order, so the last-drawn (topmost) polygon is what the user sees on top; iterate and keep the last hit.
- **State**: add a small `AlertModal { title: String, content: String }` (or `Option<AlertModal>`) field on `AppState` (`state.rs`), separate from `NhcModal` — alerts have no image/url variant, so don't force-fit them into `NhcModal`. Use `alert.event.clone()` as title (full string — the modal title bar has room; do *not* reuse the on-scope `&alert.event[..30]` byte-slice truncation, which the map label uses only to keep labels short), `alert.headline` as content (already parsed but currently unused per exploration).
- **Click wiring** (`main.rs`): add a single-click handler alongside the existing radar-site double-click block (`main.rs:2023-2058`), guarded the same way (`!dropdown_open && !modal_open && !over_nhc_panel`). On click, call `alerts::hit_test(..., state.show_watches, state.show_warnings, ...)`; if hit, populate `state.alert_modal`.
- **Rendering**: copy the NHC text-modal render block (`main.rs:1590-1681`) structurally for the alert modal — title bar + close button + scrollable content using the existing `wrap_modal_text` helper and `NHC_MODAL_LINE_HEIGHT` constant — but drop the "Open in browser" bottom bar entirely (alerts have no URL). Close on the same close-button/Escape handling pattern used for `NhcModal` (`main.rs:2407-2408` equivalent, targeting `state.alert_modal` instead).
- Gate `modal_open` (`main.rs:1996`) to also be `true` when `state.alert_modal.is_some()`, so it correctly suppresses other click-handling while open.

## Files touched

- `app/src/scope.rs` — rework `draw_alerts` (collision placement + zoom gating), no signature change needed beyond what it already receives.
- `app/src/alerts.rs` — add `hit_test` point-in-polygon helper.
- `app/src/state.rs` — add `AlertModal` enum/struct + `state.alert_modal` field.
- `app/src/main.rs` — click handler, modal render block, legend swatch row, `modal_open` gating update.

## Verification

- `cargo build` / `cargo check` in `app/` to confirm it compiles.
- Run the app (`cargo run`) against a period with multiple overlapping alerts (or temporarily inject test `Alert`s with overlapping centroids) and visually confirm: outlines all draw, labels thin out with no overlapping text, labels disappear when zoomed far out and reappear when zoomed in, legend row appears in the status bar with correct colors, clicking a polygon opens the modal with correct event/headline text and closes cleanly.
- **Regression check for the §1 fix:** inject two alerts of the *same* event type with centroids far apart on screen; confirm **both** keep labels (global event-dedup would have dropped one). Then inject two same-event polygons near-coincident on screen; confirm collision drops the overlap down to one legible label.
- `cargo test` for any existing alert/scope unit tests to ensure no regressions.
