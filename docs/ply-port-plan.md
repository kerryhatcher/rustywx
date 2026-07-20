# rustywx → Ply Port — Staged Implementation Plan

Each stage ships a **runnable, validatable** increment. No stage depends on
future-stage polish — every one stands on its own as a working app.

## Pre-Flight: Research Findings

Before starting Stage 1, these decisions were validated against the
Ply engine v1.1 API surface, crates.io, and NOAA data source documentation.
Full research in `research/`.

### Dependencies: What stays, what goes

| Crate | Decision | Reason |
|---|---|---|
| `ply-engine` | **Add** — `net`, `net-json`, `storage`, `built-in-shaders`, `text-styling` features | Replaces eframe/egui/ureq/rusqlite |
| `nexrad-data` | **Keep** | Handles S3 sigv4 signing, bucket listing, NEXRAD binary decoding — thousands of lines not worth rewriting |
| `nexrad-model` | **Keep** | Pure data types, no egui dependency |
| `tokio` | **Keep** | Needed for `nexrad-data` background thread; Ply's `net` handles its own async internally |
| `eframe` / `egui` | **Remove** | Replaced by `ply-engine` |
| `ureq` | **Remove** | Replaced by Ply `net` for simple HTTP; `nexrad-data` uses `reqwest` internally |
| `rusqlite` | **Remove** | Replaced by Ply `storage` |
| `anyhow`, `chrono`, `image`, `serde`, `serde_json`, `zip`, `webbrowser` | **Keep** | No egui dependency, still needed |

### Architecture decisions

- **Radar data:** Keep `nexrad-data` on a background thread with `mpsc` channels
  (same pattern as existing `data.rs`). Do NOT try to replace it with Ply `net`.
- **Simple HTTP:** Use Ply's `net` module for borders GeoJSON, NWS alerts,
  NHC JSON. It's polling-based — fire requests, check `net::request()` each
  frame. No tokio management needed in app code.
- **Persistence:** Use Ply's `storage` module (backed by platform-appropriate
  paths: `~/.local/share` on Linux, OPFS on WASM). Replaces SQLite.
- **Frosted glass:** Ply's `built-in-shaders` has GLOW but no Gaussian blur.
  A custom GLSL ES 1.00 fragment shader will be needed for the blur effect.
  **Spike S1 (pending)** must validate this before Stage 6.
- **WASM radar data:** The NEXRAD S3 bucket does NOT serve CORS headers.
  WASM builds will need a relay proxy for live radar data (see Stage 8).
- **NWS alerts from WASM:** Works — `api.weather.gov` returns
  `Access-Control-Allow-Origin: *`. Don't set a custom `User-Agent` header.
- **NHC data from WASM:** Researched (R1 in `de-risking-report.md`).
  `CurrentStorms.json` has **NO CORS** (needs relay proxy).
  GIS MapServer (`mapservices.weather.noaa.gov`) **has CORS** — overlays
  work directly from WASM.
- **Natural Earth GeoJSON from WASM:** Works — GitHub serves
  `Access-Control-Allow-Origin: *`. Borders load directly in WASM builds.

## Git Workflow

- **Commit often** — at minimum after each logical change (a new module, a
  working widget, a data source wired up). Small, focused commits make
  bisecting and reviewing straightforward.
- **Push and verify before tagging** — each stage follows this sequence:

  1. Commit all changes for the stage
  2. `git push` to GitHub
  3. Wait for GitHub Actions CI to pass (all 14 jobs green)
  4. Only then: `git tag` + `git push --tags`

  This ensures no broken code ever gets a version tag. The CI runs fmt,
  clippy, check, test, doc-test, audit, deny, gitleaks, trivy, typos,
  lychee, kingfisher, and build — the same checks as `just ci-full` locally.
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
- [ ] `git push` → CI passes → `git tag v0.2.0-stage1` → `git push --tags`

---

## Stage 2: "Live Data" — Real NEXRAD

**Goal:** Real radar data from AWS, site/product/tilt controls.

**Scope:**
- `data.rs` — port the existing background-worker pattern: spawn a thread with
  a tokio runtime, run `nexrad_data::aws::archive::list_files` +
  `download_file`, decode via `nexrad_model`, send `WorkerMessage`s over
  `mpsc::channel`. Same architecture as the egui app, adapted for macroquad.
- `cache.rs` — disk cache using Ply `storage` (`save_bytes` / `load_bytes`
  for serialized scan data, or JSON metadata files)
- Wire up real NEXRAD data flow: fetch → decode → rasterize → display
- Site selector (keyboard: Left/Right arrow keys)
- Tilt selector (keyboard: T key cycles tilts)
- Status bar shows scan timestamp, site, elevation
- Loading state while fetching first scan
- Error state if fetch fails

**Deliverable:** `cargo run` → real radar data from KJGX (or any site),
auto-refreshes every 2 minutes, cached on disk.

**Validation:**
- [ ] App boots and shows "Loading…" then real radar data
- [ ] Scan timestamp visible in status bar
- [ ] Site switching fetches new data (Left/Right arrow keys)
- [ ] Product toggle changes display (R/V keys)
- [ ] Tilt cycling works (T key)
- [ ] Data cached — restart app, data loads instantly
- [ ] Error shown if network unavailable (graceful)
- [ ] Auto-refresh picks up new scans
- [ ] `git push` → CI passes → `git tag v0.2.0-stage2` → `git push --tags`

---

## Stage 3: "Custom Widgets" — Dropdown, Toggle, Collapsing

**Goal:** Proper clickable controls replacing keyboard-only hacks.

**Scope:**
- `widgets/dropdown.rs` — searchable dropdown (site selector, tilt selector)
- `widgets/toggle.rs` — product toggle (Reflectivity ⇄ Velocity)
- `widgets/collapsing.rs` — collapsible section (for NHC text later)
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
- [ ] `git push` → CI passes → `git tag v0.2.0-stage3` → `git push --tags`

---

## Stage 4: "Borders & Alerts" — Map Overlays

**Goal:** State borders and NWS warning/watch polygons on the scope.

**Scope:**
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

**Deliverable:** State boundary lines and active NWS warnings/watches visible
on the radar scope.

**Validation:**
- [ ] State borders visible as subtle brown lines
- [ ] Coastlines visible
- [ ] Active warnings/watches appear as colored polygons
- [ ] Alert labels visible (e.g. "Severe Thunderstorm Warning")
- [ ] Alerts refresh every 2 minutes
- [ ] Overlays cached to disk via Ply `storage`
- [ ] No overlays when none are active (graceful empty state)
- [ ] `git push` → CI passes → `git tag v0.2.0-stage4` → `git push --tags`

---

## Stage 5: "Tropical" — NHC Hurricane Data

**Goal:** NHC tropical cyclone data: GIS overlays on scope + detail panel.

**Scope:**
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
- [ ] NHC CORS status confirmed: CurrentStorms.json proxied, GIS MapServer direct
- [ ] `git push` → CI passes → `git tag v0.3.0-stage5` → `git push --tags`

---

## Stage 6: "Observatory Look" — Visual Design

**Goal:** The full "Observatory" visual identity — frosted glass, animations,
typography, responsive layout.

**Scope:**
- `widgets/glass_panel.rs` — reusable frosted glass panel wrapper (moved here
  from Stage 3 since it's not needed until visual theming)
- Frosted glass styling on all panels. Ply's `built-in-shaders` has GLOW but
  no Gaussian blur — **author a custom GLSL ES 1.00 fragment shader** for the
  blur effect. Apply via `.shader(&BLUR_SHADER, |s| s.uniform("u_radius", 8.0))`.
- Dark gradient background with noise texture (use `GRADIENT_RADIAL` built-in
  or a custom shader)
- Teal/cyan accent color on all interactive elements
- Inter font for UI chrome, JetBrains Mono for scope/data labels.
  Use `FontAsset::Path("assets/fonts/Inter.ttf")` — verify font loading
  works on all platforms. Bundle fonts in `assets/fonts/`.
- Rich animations:
  - Panel slide-in/out with spring physics
  - Hover glow on buttons and interactive elements (use `GLOW` built-in shader)
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
- [ ] Frosted glass effect visible on panels (custom blur shader working)
- [ ] Control bar auto-hides after 3s, reappears on mouse move
- [ ] Buttons glow on hover with accent color
- [ ] NHC panel slides with spring animation
- [ ] New data triggers subtle pulse
- [ ] Inter font used in UI, JetBrains Mono on scope
- [ ] Resize to <900px — controls move to bottom, NHC goes full-screen
- [ ] Touch targets ≥44px on mobile
- [ ] Loading state shows skeleton/spinner
- [ ] Empty states show helpful messages
- [ ] `git push` → CI passes → `git tag v0.4.0-stage6` → `git push --tags`

---

## Stage 7: "Settings & Polish" — Configuration + Fit & Finish

**Goal:** Settings panel, keyboard shortcuts, edge cases, cleanup.

**Scope:**
- `widgets/settings.rs` — settings panel UI (glass modal using Stage 6
  `glass_panel` widget)
- Settings persistence via Ply `storage`:
  `storage.save_string("settings.json", &serde_json::to_string(&settings)?)`
  and `storage.load_string("settings.json")` on startup.
  Replaces the old `store.rs` + SQLite approach.
- Settings: default site, poll interval, NHC refresh, overlay defaults,
  animation level (Full/Subtle/None)
- Keyboard shortcuts overlay (? key)
- Error recovery (network failures, corrupt cache)
- Remove all egui-era dead code from `src/` — old `app.rs`, old `store.rs`,
  any remaining `egui::` imports. The old `src/` files should have been
  incrementally replaced as each module was ported; this stage is the final
  sweep.
- Update documentation (`README.md`, `USER_GUIDE.md`)

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
- [ ] `git push` → CI passes → `git tag v0.5.0-stage7` → `git push --tags`

---

## Stage 8: "Cross-Platform" — WASM, Android, Perf

**Goal:** Build and test on all target platforms.

**Scope:**
- WASM build via `plyx web` (requires `rustup target add wasm32-unknown-unknown`;
  see R3 in `de-risking-report.md`)
- **WASM relay proxy (Cloudflare Worker).** Two routes are needed:
  | Route | Proxied URL | Reason |
  |---|---|---|
  | `/api/nexrad/*` | NEXRAD S3 (`unidata-nexrad-level2`) | S3 has no CORS |
  | `/api/nhc/storms` | `www.nhc.noaa.gov/CurrentStorms.json` | CloudFront has no CORS |
  **Not needed** (direct fetch from WASM): NWS Alerts, NHC GIS overlays,
  Natural Earth GeoJSON — all serve CORS headers.
- Android build via `plyx apk`
- Performance profiling (frame time, texture cache hit rate)
- Accessibility audit (labels, tab order, screen reader via Ply's `a11y`
  feature — AccessKit on desktop, JS bridge on web)
- Final cleanup and release build

**Deliverable:** App runs on desktop, web, and Android.

**Validation:**
- [ ] `plyx web` produces working WASM build
- [ ] WASM build loads real radar data via relay proxy (or documented limitation)
- [ ] `plyx apk` produces working Android APK
- [ ] Frame time <16ms (60fps) on desktop
- [ ] Frame time <33ms (30fps) on mobile
- [ ] Texture cache hit rate >90% (scope not re-rendered when static)
- [ ] Screen reader announces controls correctly
- [ ] Tab navigation works through all interactive elements
- [ ] Release build runs without debug overhead
- [ ] `git push` → CI passes → `git tag v1.0.0-stage8` → `git push --tags`

---

## Summary

| Stage | Name | Days | Ships |
|---|---|---|---|
| 1 | Hello Radar | 1 | Synthetic scope, pan/zoom |
| 2 | Live Data | 1–2 | Real NEXRAD via nexrad-data + thread |
| 3 | Custom Widgets | 1 | Dropdown, toggle, collapsing |
| 4 | Borders & Alerts | 1 | State lines, NWS warnings via Ply net |
| 5 | Tropical | 2 | NHC data, GIS overlays, panel via Ply net |
| 6 | Observatory Look | 2–3 | Visual design, custom blur shader, animations, responsive |
| 7 | Settings & Polish | 1 | Settings via Ply storage, shortcuts, error handling |
| 8 | Cross-Platform | 2 | WASM relay proxy, Android, perf, a11y |

**Total: ~11–13 days** of stage work, plus **~5–8 hours** of pre-stage spikes
(S1–S6 in `de-risking-report.md`) to de-risk the blur shader, nexrad-data
integration, and custom widget approach before their respective stages.

### Key risk items

| Risk | Stage | Mitigation |
|---|---|---|
| Custom GLSL blur shader | 6 | Spike S1 (pending) must validate before Stage 6; GLOW shader is fallback |
| WASM CORS relay proxy | 8 | Cloudflare Worker with 2 routes (NEXRAD S3 + NHC CurrentStorms.json) |
| NHC CORS: CurrentStorms.json blocked on WASM | 5, 8 | Confirmed no CORS (R1); relay proxy route added to Stage 8 Worker |
| Font loading on WASM/Android | 6 | Test font bundling early; DejaVuSansMono from spike is known-good fallback |
| nexrad-data + Ply integration unvalidated | 2 | Spike S2 (pending) must validate background-thread pattern before Stage 2 |
| Custom dropdown widget unvalidated | 3 | Spike S3 (pending) must validate composite widget approach before Stage 3 |
