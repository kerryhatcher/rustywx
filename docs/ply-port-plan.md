# rustywx → Ply Port — Staged Implementation Plan

**Target Platform:** Linux Desktop (v1.0)

Each stage ships a **runnable, validatable** increment. No stage depends on
future-stage polish — every one stands on its own as a working app.

## Progress

| Stage | Name | Status | Tag | File |
|---|---|---|---|---|
| 1 | Hello Radar | ✅ Complete | `v0.2.0-stage1` | [stage-1-hello-radar.md](stages/stage-1-hello-radar.md) |
| 2 | Live Data | ✅ Complete | `v0.2.0-stage2` | [stage-2-live-data.md](stages/stage-2-live-data.md) |
| 3 | Custom Widgets | ✅ Complete | `v0.2.0-stage3` | [stage-3-custom-widgets.md](stages/stage-3-custom-widgets.md) |
| 4 | Borders & Alerts | ✅ Complete | `v0.2.0-stage4` | [stage-4-borders-alerts.md](stages/stage-4-borders-alerts.md) |
| 5 | Tropical | ✅ Complete | `v0.3.0-stage5` | [stage-5-tropical.md](stages/stage-5-tropical.md) |
| 6 | Observatory Look | ✅ Complete | `v0.4.0-stage6` | [stage-6-observatory-look.md](stages/stage-6-observatory-look.md) |
| 7 | Settings & Polish | 🔲 Not started | `v0.5.0-stage7` | [stage-7-settings-polish.md](stages/stage-7-settings-polish.md) |
| 8 | Linux Polish | 🔲 Not started | `v1.0.0-stage8` | [stage-8-linux-polish.md](stages/stage-8-linux-polish.md) |
| — | **Post-v1** | 📝 Planned | — | [post-v1-multi-site-animation.md](post-v1-multi-site-animation.md) |

## Pre-Flight: Research Findings

Before starting Stage 1, these decisions were validated against the
Ply engine v1.1 API surface, crates.io, and NOAA data source documentation.
Full research in `docs/research/` (9 PDFs with self-contained markdown
summaries). De-risking report in `docs/de-risking-report.md`.

### Research Document Insights

The 9 research documents in `docs/research/` (each with a self-contained
markdown replacement) provide actionable insights that inform the stage
plans and post-v1 roadmap:

| Document | Key Insight | Where Applied |
|---|---|---|
| Kvasov et al. — Bilinear interpolation | Bilinear range+azimuth interpolation gives ~90% improvement over nearest-neighbor; max error 4.3–6.7% | Stage 6 |
| Hubbert et al. — REG-VRAD | 25–30% more velocity data recoverable; purple haze = unrecoverable SZ(8/64) velocities | Stage 7 (Nyquist display), Post-v1 (velocity recovery) |
| Keem et al. — Random Forest QC | ρHV > 0.9 for precipitation is best single clutter discriminator; multi-scale windows improve accuracy | Stage 7 (TDBZ tuning), Post-v1 (dual-pol QC) |
| FMH-11 Part A — System Concepts | VCP/mode info in scan metadata; Spectrum Width is a base moment | Stage 7 (VCP display, Spectrum Width) |
| FMH-11 Part B — Doppler Theory | Range-velocity dilemma physics; attenuation/AP effects | Reference for Stage 7 QC tuning |
| FMH-11 Part C — Products & Algorithms | Full dual-pol product suite (ZDR, ρHV, KDP); clutter filtering algorithms | Post-v1 (dual-pol products) |
| FMH-11 Part D — Unit Description | RDA→RPG→PUP data flow; product interpretation | Reference |
| JMA Training — Radar QC | Elevation angle composite table for CAPPI; clutter map; statistical QC | Post-v1 (CAPPI, clutter map) |
| Yi Ru thesis — Volumetric Visualization | Spline transfer functions, RLE compression, multi-site integration, benchmarks | Stages 6, 7, 8 (already planned), Post-v1 |

### Dependencies: What stays, what goes

| Crate | Decision | Reason |
|---|---|---|
| `ply-engine` | **Add** — `net`, `net-json`, `storage` up front; `built-in-shaders` + `text-styling` added at Stage 6 | Replaces eframe/egui/ureq/rusqlite |
| `nexrad-data` | **Keep** (pinned `=1.0.0-rc.7`) | Handles S3 sigv4 signing, bucket listing, NEXRAD binary decoding |
| `nexrad-model` | **Keep** (pinned `=1.0.0-rc.2`) | Pure data types, no egui dependency |
| `tokio` | **Keep** | Needed for `nexrad-data` background thread; Ply's `net` handles its own async internally |
| `eframe` / `egui` | **Remove** | Replaced by `ply-engine` |
| `ureq` | **Remove** | Replaced by Ply `net` for simple HTTP; `nexrad-data` uses `reqwest` internally |
| `rusqlite` | **Remove** | Replaced by Ply `storage` |
| `anyhow` | **Keep** | Error handling throughout |
| `chrono` | **Keep** | Timestamp parsing, display formatting |
| `image` | **Keep** (Stage 5) | Decode NHC graphics product thumbnails into RGBA for Ply textures |
| `serde`, `serde_json` | **Keep** | JSON parsing for borders, alerts, NHC data |
| `zip` | **Keep** | Decompress NEXRAD volume files from S3 |
| `webbrowser` | **Keep** (Stage 5) | Open external links from NHC panel; confirmed WASM-compatible (R4) |

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
- **Scope rendering:** Draw directly to the screen with macroquad — do NOT
  use `render_to_texture` + Ply `.image()`. The framebuffer's bottom-left
  origin causes a 180° rotation when displayed through Ply. (Learned during
  Stage 1 validation.)
- **Linux desktop:** Native HTTP clients (Ply `net`, `reqwest`) don't require CORS headers.
  All data sources (NEXRAD S3, NHC, NWS alerts, Natural Earth) work directly without proxies.
- **Composite controls (Stage 3):** Keep widget rendering and event handling in
  separate phases: declare elements through `Ui`, call `show`, then query
  presses through `Ply`. Widgets return semantic values/indices; application
  state transitions remain centralized in `main.rs`.
- **Dynamic option IDs (Stage 3):** Use Ply's indexed `(&str, u32)` IDs with
  stable source indices. Filtering and scrolling must not change an option's ID.
- **Dropdown input isolation (Stage 3):** While a dropdown is open, consume its
  wheel/keyboard input and suppress radar pan/zoom to avoid double actions.
- **Text input API update (Stage 3):** Ply 1.1.1 includes text-input and focus
  APIs. The site filter intentionally retains raw `get_char_pressed()` because
  it is a compact type-to-filter control without cursor/selection/IME needs.

## Git Workflow

- **Commit often** — at minimum after each logical change (a new module, a
  working widget, a data source wired up). Small, focused commits make
  bisecting and reviewing straightforward.
- **Push and verify before tagging** — each stage follows this sequence:

  1. Commit all changes for the stage
  2. `git push` to GitHub
  3. Wait for GitHub Actions CI to pass (all jobs green)
  4. Only then: `git tag` + `git push --tags`

  This ensures no broken code ever gets a version tag. The CI runs fmt,
  clippy, check, test, doc-test, audit, deny, gitleaks, trivy, typos,
  lychee, kingfisher, and build — the same checks as `just ci-full` locally.
- **Semver tag after each stage** — tags let you jump back to any stage's
  working state for comparison or rollback. The minor version bumps at
  Stage 5 (major feature: NHC) and Stage 6 (visual identity), with 1.0.0
  at Stage 8 when the port is complete.
- **Branch:** `port/ply-engine`. All implementation lives here. Merge to
  `main` after Stage 8 is validated.

## Crate Structure

**Workspace approach** (chosen in Stage 1):

```
rustywx/
├── Cargo.toml          # [workspace] members = ["ply-spike"] resolver = "2"
├── Cargo.lock          # workspace-level lock file
├── src/                # old egui source (orphaned, not built — removed in Stage 7)
├── ply-spike/
│   ├── Cargo.toml      # [package] name = "rustywx" version = "0.2.0"
│   ├── assets/
│   │   ├── fonts/      # DejaVuSansMono.ttf (Stage 6 adds Inter, JetBrains Mono)
│   │   └── shaders/    # blur.frag (Stage 6 re-enables)
│   └── src/
│       ├── lib.rs      # pub mod tree
│       ├── main.rs     # window config, game loop, Ply UI shell
│       ├── state.rs    # AppState struct
│       ├── model.rs    # scan data types (5 unit tests)
│       ├── nhc.rs      # NHC tropical cyclone data (26 unit tests)
│       ├── colors.rs   # NWS color tables (4 unit tests)
│       ├── geo.rs      # geographic utilities, RADAR_SITES, CITIES (3 unit tests)
│       ├── scope.rs    # rasterization + overlay drawing
│       ├── data.rs     # background NEXRAD worker (enabled in Stage 2)
│       └── widgets/    # reusable dropdown, toggle, and collapsing controls
└── docs/
    ├── ply-port-plan.md        # this file (overview/index)
    ├── stages/                 # per-stage detail files
    ├── ply-issues/             # 12 draft Ply engine feedback issues
    ├── de-risking-report.md    # spike results
    └── observatory-mockup.html # Stage 6 visual target
```

## Testing Strategy

- **Unit tests** cover pure-data modules (`model.rs`, `colors.rs`, `geo.rs`)
  plus pure widget state/filter logic. After Stage 5 the workspace has 54 tests.
- **No full headless integration tests** for the Ply UI layer — Ply does not have
  a convenient end-to-end headless interaction mode (see Ply issue #8).
  Stage checklists remain primary, supplemented in Stage 3 by driving the real
  X11 window with `xdotool` and capturing screenshots.
- **Mandatory smoke test** — `just run` must launch and stay alive for 3+
  seconds before any task is claimed complete (see `AGENTS.md`).
- **CI** (`just ci-full`) runs `cargo test` for all unit tests plus fmt,
  clippy, check, audit, deny, gitleaks, trivy, typos, lychee, kingfisher.

## Toolchain Requirements

- **Rust edition 2024** — requires Rust ≥1.85 (stabilized February 2025).
- **`nexrad-data` and `nexrad-model` are RCs** — pinned with `=` in
  `ply-spike/Cargo.toml`. Monitor upstream for 1.0 stable releases.
- **`image` crate** (Stage 5): features `["png", "jpeg"]` sufficient for NHC products.
- **Fonts** (Stage 6): Inter and JetBrains Mono are SIL OFL licensed.
  Download from Google Fonts, place in `assets/fonts/`, commit to repo.
- **Manual scaffold** — no `plyx` tooling required for Linux desktop builds.

## Summary

| Stage | Name | Days | Ships |
|---|---|---|---|
| 1 | Hello Radar | 1 ✅ | Synthetic scope, pan/zoom |
| 2 | Live Data | 1–2 ✅ | Real NEXRAD via nexrad-data + thread |
| 3 | Custom Widgets | 1 ✅ local | Searchable site/tilt dropdowns, product toggle, collapsing |
| 4 | Borders & Alerts | 1 ✅ | State lines, NWS warnings via Ply net |
| 5 | Tropical | 2 ✅ | NHC data, GIS overlays, panel via Ply net |
| 6 | Observatory Look | 2–3 ✅ | Visual design, custom blur shader, animations, responsive |
| 7 | Settings & Polish | 1 | Settings via Ply storage, shortcuts, error handling |
| 8 | Linux Polish | 2 | HiDPI, Wayland, perf, a11y |

**Total: ~11–13 days** of stage work. Pre-flight spikes (S1–S8) are complete.

## Post-v1

Features deferred past v1.0.0 are tracked in [post-v1-multi-site-animation.md](post-v1-multi-site-animation.md):

- **Multi-site integration** — combining data from multiple NEXRAD sites into a single unified view. The thesis by Yi Ru (2007) provides a complete blueprint: spherical→geographic coordinate conversion, temporal synchronization, overlap averaging, and a semi-regular 3D grid structure.
- **Temporal animation** — playing back a sequence of historical volume scans as a smooth animation. The thesis's modified RLE compression (99%+) makes storing hundreds of frames feasible (~15 MB for 288 frames at 256×256×128).

Both features are informed by the thesis analysis in `docs/post-v1-multi-site-animation.md`.

### Additional post-v1 features from research

These items are informed by the other 8 research documents (see
`docs/research/` for full markdown transcriptions):

- **Dual-polarization product suite** — add ZDR, ρHV, KDP, ΦDP as displayable
  products. ρHV is the single best clutter discriminator (Keem et al.:
  >99.98% classification accuracy with ρHV alone). FMH-11 Part C defines all
  product numbers, resolutions, and characteristics.
- **ρHV-based clutter filtering** — replace the current range-adaptive
  threshold + TDBZ approach with ρHV-gated QC. Precipitation consistently
  shows ρHV > 0.9; non-meteorological targets show ρHV < 0.85 (Keem et al.).
- **CAPPI composite display** — composite multiple elevation tilts into a
  single constant-altitude view (~2 km), eliminating the cone of silence and
  ground clutter near the radar. The JMA training document describes the
  elevation angle composite table approach; Keem et al. confirm CAPPI
  improves near-radar precipitation detection.
- **Velocity recovery (purple haze mitigation)** — implement a simplified
  VRAD approach using long-PRT surveillance scan velocities to fill gaps
  in the short-PRT Doppler scan. Hubbert et al. show 25–30% more velocity
  data recoverable with REG-VRAD; even a Level-2-only approach would
  partially fill the transparent range-folded regions.
- **Persistent clutter map** — pre-compute persistent clutter locations
  (buildings, towers, terrain) from accumulated statistics, as described
  in the JMA training document. Reduces per-scan false positives and allows
  lowering the real-time threshold to preserve weak precipitation.
- **Interactive transfer functions** — extend Stage 6's spline-based color
  tables to user-adjustable color/opacity mapping via cubic Hermite spline
  controls (Yi Ru thesis, §3.2.4). Lets users highlight specific dBZ ranges
  or create custom color schemes.

### Key risk items

| Risk | Stage | Mitigation | Status |
|---|---|---|---|
| Custom GLSL blur shader | 6 | ✅ Spike S1 validated — 5×5 Gaussian blur compiles and runs; GLOW shader is fallback | Resolved |
| Font loading on Linux | 6 | Standard file paths; test early with Inter/JetBrains Mono TTF files | Pending |
| HiDPI/Wayland compatibility | 8 | Test on target hardware; Ply uses AccessKit for native Linux support | Pending |
| nexrad-data + Ply integration | 2 | ✅ Spike S2 validated — background-thread + mpsc pattern works | Resolved |
| Custom dropdown widget | 3 | ✅ Reusable implementation complete; 143 sites filtered with only 12 visible rows hit-tested | Resolved |
| render_to_texture coordinate flip | 1 | Draw directly to screen instead of render_to_texture + .image() | ✅ Resolved |

## Ply Engine Feedback

12 draft issues in `docs/ply-issues/` cover findings from the spikes and
early stage work. Revalidate drafts against the installed Ply version before
filing; Stage 3 confirmed that issue #3's original “no text input” premise is
stale in Ply 1.1.1. See `docs/ply-issues/README.md` for the filing guide.
Key items:

1. **ttf-parser unmaintained** (RUSTSEC-2026-0192) — transitive via ply-engine
2. **Built-in blur shader** — most common UI effect, currently requires custom GLSL
3. **Text input ergonomics/docs** — Ply 1.1.1 now has a primitive; reassess the stale draft before filing
4. **GLSL version docs mismatch** — docs say 3.00, engine uses 1.00
5. **Headless testing guide/harness** — `new_headless` exists; downstream interaction testing needs a documented path
