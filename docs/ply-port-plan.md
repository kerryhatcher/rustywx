# rustywx → Ply Port — Staged Implementation Plan

Each stage ships a **runnable, validatable** increment. No stage depends on
future-stage polish — every one stands on its own as a working app.

## Git Workflow

- **Commit often** — at minimum after each logical change (a new module, a
  working widget, a data source wired up). Small, focused commits make
  bisecting and reviewing straightforward.
- **Semver tag after each stage** — the app version tracks port progress:

| Stage | Tag |
|---|---|
| 1 — Hello Radar | `v0.2.0-stage1` |
| 2 — Live Data | `v0.2.0-stage2` |
| 3 — Custom Widgets | `v0.2.0-stage3` |
| 4 — Borders & Alerts | `v0.2.0-stage4` |
| 5 — Tropical | `v0.3.0-stage5` |
| 6 — Observatory Look | `v0.4.0-stage6` |
| 7 — Settings & Polish | `v0.5.0-stage7` |
| 8 — Cross-Platform | `v1.0.0-stage8` |

- Tags let you jump back to any stage's working state for comparison or
  rollback. The minor version bumps at Stage 5 (major feature: NHC) and
  Stage 6 (visual identity), with 1.0.0 at Stage 8 when the port is complete.
- Branch: `spike/ply-radar-scope` (already exists). Merge to `main` after
  Stage 8 is validated.

---

## Stage 1: "Hello Radar" — Synthetic Scope

**Goal:** App boots, renders a radar scope with overlays, pan/zoom works.

**Scope:**
- Ply project scaffold (`plyx init` equivalent)
- `model.rs`, `colors.rs`, `geo.rs` ported (no egui deps — copy + minor tweaks)
- `scope.rs` rasterization + overlay drawing (from spike, synthetic data)
- `state.rs` — `AppState` struct
- `main.rs` — window config, game loop, Ply UI shell
- Basic controls: keyboard R/V for product, arrow keys for site, scroll/drag for pan/zoom
- Top bar: site name, product indicator, zoom/pan readout
- Bottom strip: status text

**Deliverable:** `cargo run` → window with synthetic radar sweep, range rings,
cardinal spokes, station marker, city markers. Drag to pan, scroll to zoom.

**Validation:**
- [ ] Window opens with dark background
- [ ] Radar scope visible with synthetic reflectivity pattern
- [ ] Range rings at 50/100/150/200 km
- [ ] Cardinal spokes labeled N/E/S/W
- [ ] Station marker at center with site ID
- [ ] City markers visible
- [ ] Drag pans the view
- [ ] Scroll wheel zooms in/out
- [ ] R key switches to Velocity (different colors)
- [ ] V key switches back to Reflectivity
- [ ] Arrow keys cycle through sites
- [ ] 0 key resets pan/zoom
- [ ] `git commit` + `git tag v0.2.0-stage1`

---

## Stage 2: "Live Data" — Real NEXRAD

**Goal:** Real radar data from AWS, site/product/tilt controls.

**Scope:**
- `net.rs` — radar poller using Ply `net` module (replaces `data.rs` thread)
- `cache.rs` — disk cache using Ply `storage` or JSON files
- Wire up real NEXRAD data flow: fetch → decode → rasterize → display
- Site selector (keyboard for now, dropdown in Stage 3)
- Tilt selector (keyboard for now)
- Status bar shows scan timestamp, site, elevation
- Loading state while fetching first scan
- Error state if fetch fails

**Deliverable:** `cargo run` → real radar data from KJGX (or any site),
auto-refreshes every 2 minutes, cached on disk.

**Validation:**
- [ ] App boots and shows "Loading…" then real radar data
- [ ] Scan timestamp visible in status bar
- [ ] Site switching fetches new data (arrow keys)
- [ ] Product toggle changes display (R/V keys)
- [ ] Tilt cycling works (T key or similar)
- [ ] Data cached — restart app, data loads instantly
- [ ] Error shown if network unavailable (graceful)
- [ ] Auto-refresh picks up new scans
- [ ] `git commit` + `git tag v0.2.0-stage2`

---

## Stage 3: "Custom Widgets" — Dropdown, Toggle, Collapsing

**Goal:** Proper clickable controls replacing keyboard-only hacks.

**Scope:**
- `widgets/dropdown.rs` — searchable dropdown (site selector, tilt selector)
- `widgets/toggle.rs` — product toggle (Reflectivity ⇄ Velocity)
- `widgets/collapsing.rs` — collapsible section (for NHC text later)
- `widgets/glass_panel.rs` — reusable frosted glass panel wrapper
- Wire widgets into control bar
- Site dropdown with search/filter for 160+ sites
- Tilt dropdown populated from actual sweep data

**Deliverable:** Clickable site selector with type-to-filter, product toggle
buttons, working tilt dropdown.

**Validation:**
- [ ] Site dropdown opens on click, shows list of sites
- [ ] Typing in site dropdown filters the list
- [ ] Selecting a site triggers data fetch
- [ ] Product toggle shows active state with accent color
- [ ] Tilt dropdown shows available elevations from current scan
- [ ] All controls work with mouse and keyboard
- [ ] Dropdowns close on outside click
- [ ] `git commit` + `git tag v0.2.0-stage3`

---

## Stage 4: "Borders & Alerts" — Map Overlays

**Goal:** State borders and NWS warning/watch polygons on the scope.

**Scope:**
- `borders.rs` — port fetch + parse (replace `ureq` with Ply `net`)
- `alerts.rs` — port fetch + parse (replace `ureq`, remove `egui::Color32`)
- Draw border line segments on scope
- Draw alert polygons on scope (clipped to radar circle)
- Alert labels near polygon centers
- Toggle overlays on/off

**Deliverable:** State boundary lines and active NWS warnings/watches visible
on the radar scope.

**Validation:**
- [ ] State borders visible as subtle brown lines
- [ ] Coastlines visible
- [ ] Active warnings/watches appear as colored polygons
- [ ] Alert labels visible (e.g. "Severe Thunderstorm Warning")
- [ ] Alerts refresh every 2 minutes
- [ ] Overlays cached to disk
- [ ] No overlays when none are active (graceful empty state)
- [ ] `git commit` + `git tag v0.2.0-stage4`

---

## Stage 5: "Tropical" — NHC Hurricane Data

**Goal:** NHC tropical cyclone data: GIS overlays on scope + detail panel.

**Scope:**
- `nhc.rs` — port fetch + parse (replace `ureq` with Ply `net`)
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

**Deliverable:** Full NHC tropical cyclone data — overlays on scope, detail
panel with all products.

**Validation:**
- [ ] NHC button shows storm count badge when storms are active
- [ ] NHC panel slides in from right on click
- [ ] Storm selector switches between active storms
- [ ] Storm stats display correctly
- [ ] Graphics products show thumbnails (or placeholders while loading)
- [ ] Map overlay toggles work (wind probs, arrival times)
- [ ] Text products expand/collapse
- [ ] GIS overlays visible on scope (cone, track, points)
- [ ] Wind probability contours render as colored lines
- [ ] "No active storms" state when season is quiet
- [ ] Data refreshes every 5 minutes
- [ ] `git commit` + `git tag v0.3.0-stage5`

---

## Stage 6: "Observatory Look" — Visual Design

**Goal:** The full "Observatory" visual identity — frosted glass, animations,
typography, responsive layout.

**Scope:**
- Frosted glass styling on all panels (shader-based blur via `built-in-shaders`)
- Dark gradient background with noise texture
- Teal/cyan accent color on all interactive elements
- Inter font for UI chrome, JetBrains Mono for scope/data labels
- Rich animations:
  - Panel slide-in/out with spring physics
  - Hover glow on buttons and interactive elements
  - Pulse animation on new data arrival
  - Staggered entrance on app launch
  - Radar sweep line (optional visual flourish)
- Auto-hiding control bar (fades after 3s inactivity)
- Responsive layout:
  - Desktop: top control bar, side NHC panel, bottom status
  - Mobile (<900px): bottom control bar, full-screen NHC panel, larger touch targets
- Loading skeleton while data loads
- Empty states ("No active storms", "No alerts", etc.)

**Deliverable:** App matches the `observatory-mockup.html` look and feel.

**Validation:**
- [ ] Frosted glass effect visible on panels
- [ ] Control bar auto-hides after 3s, reappears on mouse move
- [ ] Buttons glow on hover with accent color
- [ ] NHC panel slides with spring animation
- [ ] New data triggers subtle pulse
- [ ] Inter font used in UI, JetBrains Mono on scope
- [ ] Resize to <900px — controls move to bottom, NHC goes full-screen
- [ ] Touch targets ≥44px on mobile
- [ ] Loading state shows skeleton/spinner
- [ ] Empty states show helpful messages
- [ ] `git commit` + `git tag v0.4.0-stage6`

---

## Stage 7: "Settings & Polish" — Configuration + Fit & Finish

**Goal:** Settings panel, keyboard shortcuts, edge cases, cleanup.

**Scope:**
- `widgets/settings.rs` — settings panel UI (glass modal)
- Settings persistence via Ply `storage`
- Settings: default site, poll interval, NHC refresh, overlay defaults, animation level
- Keyboard shortcuts overlay (? key)
- Error recovery (network failures, corrupt cache)
- Remove all egui-era dead code
- Update documentation

**Deliverable:** Polished, configurable app.

**Validation:**
- [ ] Settings panel opens via gear icon
- [ ] Default site setting works (app starts on chosen site)
- [ ] Animation level can be set to Full/Subtle/None
- [ ] Overlay defaults respected on startup
- [ ] ? key shows keyboard shortcuts overlay
- [ ] Network errors show user-friendly message, not crash
- [ ] Corrupt cache is handled gracefully
- [ ] No egui imports remain in codebase
- [ ] `git commit` + `git tag v0.5.0-stage7`

---

## Stage 8: "Cross-Platform" — WASM, Android, Perf

**Goal:** Build and test on all target platforms.

**Scope:**
- WASM build via `plyx web`
- Android build via `plyx apk`
- Performance profiling (frame time, texture cache hit rate)
- Accessibility audit (labels, tab order, screen reader via `a11y` feature)
- Final cleanup and release build

**Deliverable:** App runs on desktop, web, and Android.

**Validation:**
- [ ] `plyx web` produces working WASM build
- [ ] WASM build loads real radar data (CORS permitting)
- [ ] `plyx apk` produces working Android APK
- [ ] Frame time <16ms (60fps) on desktop
- [ ] Frame time <33ms (30fps) on mobile
- [ ] Texture cache hit rate >90% (scope not re-rendered when static)
- [ ] Screen reader announces controls correctly
- [ ] Tab navigation works through all interactive elements
- [ ] Release build runs without debug overhead
- [ ] `git commit` + `git tag v1.0.0-stage8`

---

## Summary

| Stage | Name | Cumulative Days | Ships |
|---|---|---|---|
| 1 | Hello Radar | 1 | Synthetic scope, pan/zoom |
| 2 | Live Data | +1 | Real NEXRAD, site/product/tilt |
| 3 | Custom Widgets | +1 | Dropdown, toggle, collapsing |
| 4 | Borders & Alerts | +1 | State lines, NWS warnings |
| 5 | Tropical | +1–2 | NHC data, GIS overlays, panel |
| 6 | Observatory Look | +1–2 | Visual design, animations, responsive |
| 7 | Settings & Polish | +1 | Settings, shortcuts, error handling |
| 8 | Cross-Platform | +1 | WASM, Android, perf, a11y |

**Total: ~8–10 days** to production-ready on all platforms.
