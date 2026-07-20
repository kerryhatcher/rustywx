# rustywx → Ply Port — Staged Implementation Plan

**Target Platform:** Linux Desktop (v1.0)

Each stage ships a **runnable, validatable** increment. No stage depends on
future-stage polish — every one stands on its own as a working app.

## Pre-Flight: Research Findings

Before starting Stage 1, these decisions were validated against the
Ply engine v1.1 API surface, crates.io, and NOAA data source documentation.
Full research in `research/`.

### Dependencies: What stays, what goes

| Crate | Decision | Reason |
|---|---|---|
| `ply-engine` | **Add** — `net`, `net-json`, `storage` up front; `built-in-shaders` + `text-styling` added at Stage 6 (blur shader, fonts) | Replaces eframe/egui/ureq/rusqlite |
| `nexrad-data` | **Keep** | Handles S3 sigv4 signing, bucket listing, NEXRAD binary decoding — thousands of lines not worth rewriting |
| `nexrad-model` | **Keep** | Pure data types, no egui dependency |
| `tokio` | **Keep** | Needed for `nexrad-data` background thread; Ply's `net` handles its own async internally |
| `eframe` / `egui` | **Remove** | Replaced by `ply-engine` |
| `ureq` | **Remove** | Replaced by Ply `net` for simple HTTP; `nexrad-data` uses `reqwest` internally |
| `rusqlite` | **Remove** | Replaced by Ply `storage` |
| `anyhow` | **Keep** | Error handling throughout |
| `chrono` | **Keep** | Timestamp parsing, display formatting |
| `image` | **Keep** | Decode NHC graphics product thumbnails into RGBA for Ply textures (Stage 5) |
| `serde`, `serde_json` | **Keep** | JSON parsing for borders, alerts, NHC data |
| `zip` | **Keep** | Decompress NEXRAD volume files from S3; used directly by the app, not just a transitive dependency |
| `webbrowser` | **Keep** (0.8 sufficient; optionally bump to 1.2) | Open external links from NHC panel (Stage 5); confirmed WASM-compatible (R4). R4 found 0.8.15 works on WASM; 1.2 is an optional bump, not required |

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
  ✅ Spike S1 validated this — 5×5 Gaussian blur compiles and runs.
- **Linux desktop:** Native HTTP clients (Ply `net`, `reqwest`) don't require CORS headers.
  All data sources (NEXRAD S3, NHC, NWS alerts, Natural Earth) work directly without proxies.

## Git Workflow

- **Commit often** — at minimum after each logical change (a new module, a
  working widget, a data source wired up). Small, focused commits make
  bisecting and reviewing straightforward.
- **Push and verify before tagging** — each stage follows this sequence:

  1. Commit all changes for the stage
  2. `git push` to GitHub
  3. Wait for GitHub Actions CI to pass (all 13 jobs green)
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
- Branch: `port/ply-engine` (renamed from `spike/ply-radar-scope`). All spike research
  code and the implementation live here. Merge to `main` after Stage 8
  is validated.

---

## Stage 1: "Hello Radar" — Synthetic Scope

**Goal:** App boots, renders a radar scope with overlays, pan/zoom works.

**Scope:**
- Manual Ply project scaffold (matching `ply-spike/` structure; no `plyx init`)
- `model.rs`, `colors.rs`, `geo.rs` ported (no egui deps — copy + minor tweaks)
- `scope.rs` rasterization + overlay drawing — **full rewrite** of the drawing
  layer from egui (`Painter`, `Pos2`, `ColorImage`, `Stroke`) to Ply's
  rendering API. The spike has a working Ply version; use that as the
  starting point. Note: egui uses top-left origin with Y-down; Ply uses a
  different coordinate system — all drawing math must be translated.
- `state.rs` — `AppState` struct (replaces the egui `app.rs` app struct;
  `app.rs` is removed in this stage)
- `lib.rs` — updated module tree for Ply (old egui modules removed)
- `logger.rs` — **drop** this module. Ply's engine provides its own logging;
  the egui-era logger is not ported. (If a thin shim is later needed for
  module-level `tracing`/`log` bridging, add it then — don't speculatively
  port.)
- `main.rs` — window config, game loop, Ply UI shell
- Basic controls: keyboard R/V for product, arrow keys for site, scroll/drag for pan/zoom, 0 to reset view
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
  `mpsc::channel`. Same architecture as the egui app, adapted for Ply.
  The `tokio` runtime lives on its own thread — Ply's game loop just polls
  `rx.try_recv()` each frame. Validated by Spike S2 before this stage.
- `cache.rs` — **full rewrite** from SQLite/rusqlite to Ply `storage`
  (`save_bytes().await` / `load_bytes().await` for serialized scan data,
  or JSON metadata files). Ply's `Storage` API is **async** (Spike S5);
  calls must be `.await`ed inside the async game loop or an async helper.
  The old `cache.rs` uses SQL schema and prepared statements;
  Ply `storage` is a key-value API — the data model changes accordingly.
  The old `store.rs` (rusqlite-based settings) is also removed in this
  stage since Ply `storage` handles all persistence.

  **Integration pattern (Spike S8 validated):**
  - **Settings/metadata** (<2ms): Direct `.await` in game loop
  - **Raw scan data** (10–15ms): Spawn task + `oneshot::channel` to avoid frame stalls
  - **Save**: Fire-and-forget `tokio::spawn` after receiving new scan
  - **Load**: Show "Loading…" UI while async load completes, poll channel with `try_recv()`
- Wire up real NEXRAD data flow: fetch → decode → rasterize → display
- Site selector (keyboard: Left/Right arrow keys)
- Tilt selector (keyboard: T key cycles tilts)
- Status bar shows scan timestamp, site, elevation
- Loading state while fetching first scan
- Error state if fetch fails (Recoverable: status bar alert; Fatal: full-screen error modal)

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
  (Note: `glass_panel.rs` lives in Stage 6 — it's not needed until visual theming.)
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
- `widgets/glass_panel.rs` — reusable frosted glass panel wrapper
- Frosted glass styling on all panels. Ply's `built-in-shaders` has GLOW but
  no Gaussian blur — **author a custom GLSL ES 1.00 fragment shader** for the
  blur effect. Apply via `.shader(&BLUR_SHADER, |s| s.uniform("u_radius", 8.0))`.
  Validated by Spike S1.
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
  `storage.save_string("settings.json", &serde_json::to_string(&settings)?).await`
  and `storage.load_string("settings.json").await` on startup (both inside
  the async game loop — Spike S5 confirmed `Storage` is async).
  (Ply `storage` replaced SQLite/rusqlite in Stage 2; this stage adds the
  settings schema and UI on top of it.)
  **Direct `.await` is acceptable** — settings JSON is small (<2ms load/save).
- Settings: default site, poll interval, NHC refresh, overlay defaults,
  animation level (Full/Subtle/None)
- Keyboard shortcuts overlay (? key)
- Error recovery (network failures, corrupt cache)
- Remove all egui-era dead code from `src/` — old `app.rs`, any remaining
  `egui::` imports. The old `src/` files should have been incrementally
  replaced as each module was ported; this stage is the final sweep.
  (`store.rs` was already removed in Stage 2 when Ply `storage` took over.)
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

## Stage 8: "Linux Polish" — Performance & Accessibility

**Goal:** Optimize for Linux desktop, ensure accessibility compliance.

**Scope:**
- Performance profiling (frame time, texture cache hit rate)
- HiDPI testing (verify scaling on high-DPI displays)
- Wayland testing (verify native Wayland support, no X11 fallback issues)
- Accessibility audit (labels, tab order, screen reader via Ply's `a11y`
  feature — AccessKit on desktop)
- System tray icon (optional — runs in background, quick access to current alerts)
- Final cleanup and release build

**Deliverable:** Optimized Linux desktop build with full accessibility support.

**Validation:**
- [ ] Frame time <16ms (60fps) on target hardware
- [ ] Texture cache hit rate >90% (scope not re-rendered when static)
- [ ] HiDPI display renders correctly (no blurry UI)
- [ ] Wayland native — no X11 warnings or fallback
- [ ] Screen reader announces controls correctly
- [ ] Tab navigation works through all interactive elements
- [ ] Release build runs without debug overhead
- [ ] (Optional) System tray icon shows current alert status
- [ ] `git push` → CI passes → `git tag v1.0.0-stage8` → `git push --tags`

---

## Directory Strategy

**`ply-spike/` is the new `src/`.** The existing `ply-spike/` directory
(created during spike research) contains the Ply project scaffold. During
Stage 1, `ply-spike/src/` becomes the canonical source tree. The old
`src/` directory (egui-based) is **not** modified — it remains as a
reference until the final cleanup sweep in Stage 7, at which point it is
removed entirely.

Each stage adds or rewrites files under `ply-spike/src/`. The old `src/`
is never the build target after Stage 1 begins.

### Asset Structure
Assets are stored relative to the crate root in `assets/`:
- `assets/fonts/`: TTF files (Inter, JetBrains Mono)
- `assets/shaders/`: Custom GLSL ES 1.00 fragment shaders
- `assets/textures/`: Static textures, noise maps, and backgrounds

### Stage 1 crate swap (explicit steps)

The root `Cargo.toml` and `ply-spike/Cargo.toml` are currently two
independent crates — there is **no `[workspace]`**, and CI's
`cargo build --release` / `cargo test` target the root crate (which still
builds the egui `src/`). For CI to build the Ply app from Stage 1 onward,
Stage 1 must perform the swap explicitly:

1. Convert the root `Cargo.toml` into a workspace root and make `ply-spike`
   the sole member: add `[workspace] members = ["ply-spike"]` (or move
   `ply-spike` to the repo root and drop the inner crate). The root crate's
   old egui `src/` is no longer built.
2. Update `.github/workflows/ci.yml`'s build/test jobs to build the
   workspace (`cargo build --release` / `cargo test` resolve to the
   `ply-spike` member automatically once it's the only member).
3. Port the root crate's metadata (license, publish=false, edition 2024)
   onto `ply-spike/Cargo.toml` so the published/shipped crate identity is
   preserved.
4. Verify `just ci-full` and CI both build the Ply app, not the egui app,
   before tagging `v0.2.0-stage1`.

If instead the choice is to move `ply-spike/src/*` into the root `src/`
and rewrite the root `Cargo.toml` in place, that is also acceptable — the
key requirement is that **exactly one** crate is the build target and CI
builds it. Pick one approach in Stage 1 and document it; don't leave both
the egui root crate and `ply-spike` buildable.

## Testing Strategy

- **Unit tests** for pure-data modules (`model.rs`, `colors.rs`, `geo.rs`)
  survive the port with minimal changes. These modules have no egui
  dependencies and their existing tests carry over.
- **No new integration tests** are planned for the Ply UI layer — Ply
  does not have a headless testing mode. Manual validation checklists
  (in each stage) are the primary verification method.
- **CI** (`just ci-full`) continues to run `cargo test` for all unit tests.
  The CI-parity recipes live in the capital `Justfile` (5358 B, with `ci`,
  `ci-full`, `fmt`, `lint`, `kingfisher`, etc.). The lowercase `justfile`
  that previously held only `cargo run --release` is removed in the Stage 1
  cleanup — having both present makes `just` abort with "Multiple candidate
  justfiles found." After that removal, `just ci-full` works unchanged for
  Ply-based builds since `cargo build` / `cargo test` resolve the same way.
- **Data Pipeline Integration**: While UI testing is manual, the data pipeline (Fetch $\rightarrow$ Decode $\rightarrow$ Cache) will be verified via integration tests in CI using mock responses to ensure data integrity without needing a GPU.

## Toolchain Requirements

- **Rust edition 2024** — requires Rust ≥1.85 (stabilized February 2025).
  CI runners and dev machines must be on a recent stable toolchain.
- **`nexrad-data` and `nexrad-model` are RCs** (1.0.0-rc.7 and 1.0.0-rc.2).
  Pin **both** with `=` in Cargo.toml to avoid surprise breakage. The main
  `Cargo.toml` currently uses `^` (caret) for both — update to
  `=1.0.0-rc.7` and `=1.0.0-rc.2` before Stage 2. (`ply-spike/Cargo.toml`
  already pins both with `=`; carry that over.) Monitor upstream for the
  1.0 stable releases.
- **`image` crate** (Stage 5): Current features `["png", "jpeg"]` are
  sufficient for NHC graphics products. No additional features needed —
  `image` decodes to RGBA bytes natively with these codecs.
- **Fonts** (Stage 6): Inter and JetBrains Mono are both SIL OFL licensed.
  Download from Google Fonts or GitHub releases, place in `assets/fonts/`,
  and commit to the repo. On Android, Ply's asset bundling handles paths;
  on WASM, fonts are bundled into the build by `plyx web`.
- **Manual scaffold:** Stage 1 uses a manual scaffold
  (matching the existing `ply-spike/` structure). This is sufficient for Linux
  desktop builds — no `plyx` tooling required for Stage 8.

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
| 8 | Linux Polish | 2 | HiDPI, Wayland, perf, a11y |

**Total: ~11–13 days** of stage work, plus **~5–8 hours** of pre-stage spikes
(S1–S6 in `de-risking-report.md`) to de-risk the blur shader, nexrad-data
integration, and custom widget approach before their respective stages.

**Linux-desktop scope saves ~7–10 hours** by eliminating WASM/Android spikes
(S7, S9, S10) and simplifying Stage 8.

### Key risk items

| Risk | Stage | Mitigation |
|---|---|---|
| Custom GLSL blur shader | 6 | ✅ Spike S1 validated — 5×5 Gaussian blur compiles and runs; GLOW shader is fallback |
| Font loading on Linux | 6 | Standard file paths; test early with Inter/JetBrains Mono TTF files |
| HiDPI/Wayland compatibility | 8 | Test on target hardware; Ply uses AccessKit for native Linux support |
| nexrad-data + Ply integration unvalidated | 2 | ✅ Spike S2 validated — background-thread + mpsc pattern works |
| Custom dropdown widget unvalidated | 3 | ✅ Spike S3 validated — Ply-native approach works; no raw macroquad needed |
