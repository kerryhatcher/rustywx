# Stage 1: "Hello Radar" — Synthetic Scope

**Status:** ✅ Complete
**Tag:** `v0.2.0-stage1` (pending — CI verification on `port/ply-engine`)
**Branch:** `port/ply-engine`

## Goal

App boots, renders a radar scope with overlays, pan/zoom works.

## What Was Done

### Crate swap (workspace approach)

The root `Cargo.toml` was converted to a workspace root with `ply-spike`
as the sole member. The old egui `src/` directory is orphaned — not built
by any crate, retained as reference until Stage 7 cleanup.

- Root `Cargo.toml`: `[workspace] members = ["ply-spike"] resolver = "2"`
- `ply-spike/Cargo.toml`: renamed crate to `rustywx`, version `0.2.0`,
  ported `license = "AGPL-3.0-only"` and `publish = false`, removed `log`
  dependency
- Cargo.lock consolidated at workspace root

### Module structure

| File | Status | Notes |
|---|---|---|
| `lib.rs` | **Created** | `pub mod` tree: colors, data, geo, model, scope, state |
| `state.rs` | **Created** | `AppState` struct extracted from spike's inline `main.rs` |
| `main.rs` | **Rewritten** | Spike test artifacts removed, library imports, clean window title |
| `model.rs` | **Ported** | Identical to spike; 5 unit tests ported from old `src/` |
| `colors.rs` | **Ported** | Adapted from spike; 4 unit tests adapted from `Color32` to `[u8; 4]` |
| `geo.rs` | **Ported** | Spike version (expanded CITIES); 3 unit tests ported (range_bearing, polar_to_offset) |
| `scope.rs` | **Ported** | Spike version; clippy fixes (needless_range_loop, needless_return, needless_borrows) |
| `data.rs` | **Ported** | Spike version (Stage 2 will enable the worker) |
| `logger.rs` | **Dropped** | Not ported; Ply provides its own logging |

### Spike artifact removal

All spike test code was stripped from `main.rs`:
- BLUR_SHADER / glass panel toggle (G key) — Stage 6
- Stress test (F5) — Spike S4 validation artifact
- Storage test (F6) — Spike S8 validation artifact
- Net test (F7) — Spike S6 validation artifact

### Data worker

The background NEXRAD worker (`data::spawn_worker`) is **not spawned** in
Stage 1. The channels remain in `AppState` for a trivial Stage 2 transition
(just uncomment one line). Running on synthetic data only ensures the scope
always has a visible pattern — real NEXRAD data on a clear day produces an
empty sweep, which was causing the synthetic pattern to disappear.

### Key bindings

Per the plan: **R → Velocity**, **V → Reflectivity**. (The spike had these
swapped; fixed during validation.)

### CI and tooling changes

- Added `port/ply-engine` to CI branch triggers
- Removed egui-era RUSTSEC advisory ignores (RUSTSEC-2026-0194/0195 for
  quick-xml, gone from tree); kept RUSTSEC-2026-0192 (ttf-parser unmaintained,
  transitive via ply-engine → macroquad-ply → fontdue)
- Fixed kingfisher pre-push hook: was using `kf` (template engine, wrong
  package); now uses `kingfisher scan` (secrets scanner, correct package
  installed via `uv tool install kingfisher-bin`)
- All GitHub Actions pinned to 40-char SHAs with version comments
- Added `just run` recipe (`cd ply-spike && cargo run --release`)
- Created `AGENTS.md` with mandatory smoke-test instructions

### Ply feedback issues

12 draft issues filed in `docs/ply-issues/` covering supply chain
(ttf-parser unmaintained), feature requests (blur shader, text input,
dropdown, scrollable list, headless testing), documentation gaps (GLSL
version, auto-uniforms, net API behaviors, Storage::new signature), and
ergonomics (color i32 overflow).

## Lessons Learned

### render_to_texture coordinate flip

**Problem:** `render_to_texture()` renders to a framebuffer with OpenGL
bottom-left origin. When Ply displays the resulting texture via `.image()`,
both axes flip (180° rotation): N↔S swap, E↔W swap, text appears mirrored.

**Fix:** Draw the scope directly to the screen with macroquad instead of
rendering to an intermediate texture. Ply's UI elements (top/bottom bars)
draw on top with their opaque backgrounds; the scope area is transparent,
letting the macroquad-drawn scope show through.

**Implication for future stages:** Any macroquad rendering that needs to
compose with Ply UI should be drawn directly to the screen, not through
`render_to_texture` + `.image()`. The `render_to_texture` approach only
works if Ply adds a flip option to `.image()` (see Ply issue #12 — though
that's about Storage::new, the flip issue isn't yet filed as a separate
issue; it should be).

### Font paths relative to CWD

`FontAsset::Path("assets/fonts/DejaVuSansMono.ttf")` resolves relative to
the process's current working directory, not the crate root. When running
`cargo run` from the workspace root, the path doesn't resolve. The `just
run` recipe uses `cd ply-spike && cargo run --release` to fix this.

### City marker sampling

The spike's `i % 50` sampling skipped all cities near the default site
(KABR/Aberdeen, SD). Removed the sampling — ~1000 haversine calculations
per frame is trivial. All cities within `MAX_RANGE_KM` (230 km) now render.

### Text sizes at high DPI

Initial text sizes (10–12px) were unreadable at high DPI. Bumped to 16–18px
for all scope overlay text (range ring labels, cardinal spokes, station
marker, city names).

## Validation Results

- [x] `cargo check`, `cargo clippy`, `cargo test` (12 tests), `cargo build --release` all pass
- [x] `just run` smoke test — app launches and stays running
- [x] Window opens with dark background
- [x] Radar scope visible with synthetic reflectivity pattern (persists — no worker to replace it)
- [x] Range rings at 50/100/150/200 km
- [x] Cardinal spokes labeled N/E/S/W (correct orientation after render_to_texture fix)
- [x] Station marker at center with site ID
- [x] City markers visible (all cities within 230 km, not sampled)
- [x] R key switches to Velocity (different colors)
- [x] V key switches back to Reflectivity
- [x] Arrow keys cycle through sites
- [x] Drag pans the view (range rings/spokes shift — subtle without borders; Stage 4 adds geographic reference)
- [x] Scroll wheel zooms in/out (rings expand/contract)
- [x] 0 key resets pan/zoom
- [ ] `git push` → CI passes → `git tag v0.2.0-stage1` → `git push --tags`

## Commits

| Commit | Description |
|---|---|
| `4a9035a` | Crate swap to workspace, clean spike artifacts, port unit tests |
| `11f4d92` | Fix synthetic scope — disable worker, swap R/V keys, show all cities |
| `a2f787e` | Increase scope text sizes for readability |
| `af3801f` | Fix scope coordinate flip — draw directly to screen |
| `6a2b0c6` | Fix `just run` recipe — cd ply-spike for font paths |
| `1d61bfd` | Fix kingfisher hook — use secrets scanner not template engine |
| `1ca7f4c` | Pin astral-sh/setup-uv to SHA |
