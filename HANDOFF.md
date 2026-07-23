# Handoff — Radar reflectivity rendering: smooth + sharp

**Branch:** `feat/qc-derived-products` (work is **uncommitted** — `git status` shows modified files)
**Date:** 2026-07-23

## Goal

Make the reflectivity radar display look like the reference viewer (GRLevelX/RadarScope
"smoothing"): **smooth, flowing band shapes** with **razor-sharp, crisp color-band edges**
(blue→green→yellow→red), holding up **at any zoom** — instead of the blocky, gate-cell look
the app started with.

## Why / how we got here (each fix built on the last)

The user iterated on screenshots; each step fixed one visible problem and exposed the next:

1. **Blocky gate cells** → the raster already did bilinear value interpolation, but sparse
   regions still had hard cell edges. Added **value-field smoothing** (`smooth_values`, a
   NaN-aware box blur applied to the dBZ value grid *before* colorizing). `scope.rs`.
2. **Rectangular black holes mid-precip** → QC passes null gate cells; rasterized they became
   hard cutouts. Added **enclosed-hole fill** (`fill_enclosed_holes`). `scope.rs`.
3. **Holes that leaked to the background stayed open** → added **channel sealing**: dilate the
   coverage mask by `SEAL_RADIUS_PX` to pinch thin leak channels shut before the open-air
   flood-fill, so any hole with a narrow opening fills at any size. `scope.rs`.
4. **Fuzzy band edges** → the texture used `Linear` magnification, which blends palette colors
   across dBZ boundaries. Switched to `Nearest`. Fixed fuzz, but…
5. **Blocky/undersampled + stair-stepped edges** → raster was 1024px over 460km (0.45 km/px)
   but data is 0.25 km gates. Bumped `RASTER_SIZE_PX` to **2048** (~0.22 km/px, matches data);
   scaled the smoothing/seal/post-pass constants. Still fundamentally limited: CPU-colorized
   texture can only be crisp-but-stair-stepped (Nearest) or smooth-but-fuzzy (Linear).
6. **The real fix — GPU palette shader (JUST COMPLETED, NEEDS VISUAL VERIFICATION):**
   Stop colorizing on the CPU. Upload the **dBZ value field** as a texture (R = normalized
   dBZ, A = coverage), let the GPU interpolate the *value* at screen resolution with `Linear`,
   and apply the discrete palette in a **fragment shader** via a Nearest-filtered 256×1 LUT.
   Result: color steps stay razor-sharp (palette applied per-screen-pixel after interpolation)
   while boundaries are smooth at *any* zoom. This is how pro viewers do it.

## What is DONE (implemented, compiles, 200 lib tests pass, clippy + fmt clean)

GPU palette shader pipeline for **Reflectivity only** (other products unchanged, still
CPU-colorized + `Nearest`):

- `app/src/colors.rs`
  - `DBZ_LUT_MIN` / `DBZ_LUT_MAX` (0.0 / 80.0 dBZ) — palette range baked into the LUT.
  - `dbz_lut() -> [u8; 256*4]` — builds the 256×1 RGBA palette lookup texture.
- `app/src/scope.rs`
  - Reflectivity colorize step now **encodes a value texture** (R = normalized dBZ,
    A = coverage) instead of RGBA colors. `DBZ_MIN_VISIBLE_DBZ = 5.0` mirrors `dbz_color`'s
    transparency cutoff so the QC speckle passes behave exactly as before.
  - `PALETTE_VERTEX` / `PALETTE_FRAGMENT` GLSL + `load_palette_material()`.
  - `draw_scope_to_texture(...)` takes a new final arg `palette: Option<&Material>`; when set,
    it binds the material around the `draw_texture_ex` call.
  - Tuning constants: `RASTER_SIZE_PX = 2048`, `SMOOTH_RADIUS_PX = 3` (light — GPU does most
    edge smoothing now), `SEAL_RADIUS_PX = 8`.
  - Value-field helpers with tests: `smooth_values`, `fill_enclosed_holes`.
- `app/src/state.rs` — new fields: `radar_texture_is_value: bool`, `palette_material:
  Option<Material>`, `dbz_lut_tex: Option<Texture2D>`.
- `app/src/main.rs`
  - Reflectivity texture uploaded with `Linear`; other products `Nearest`.
  - Palette material + LUT built lazily on first raster (GL context must exist).
  - Draw call binds the LUT (`mat.set_texture("Palette", lut)`) and passes the material.

Graphics backend confirmed **OpenGL** (miniquad `AppleGfxApi` defaults to GL on macOS), so the
GLSL shader compiles. If the app is ever forced to Metal, an MSL variant would be needed.

## WHAT'S LEFT

1. **Visual verification — DONE 2026-07-23, found and fixed a shader bug.** First run
   rendered every echo as saturated pure-primary blobs (green cells showed yellow/white).
   Root cause: macroquad feeds the `color0` vertex attribute as *unnormalized bytes*
   (`VertexFormat::Byte4`, 0–255); its default shader divides by 255 but `PALETTE_VERTEX`
   didn't, so the fragment multiply scaled every LUT color ×255 and clamped each nonzero
   channel to full. Fix: `color = color0 / 255.0;` in `scope.rs`. Regression harness:
   `cargo run -p rustywx --example gpu_palette_check --release` renders a 0..255 value
   gradient through the real material and asserts all 256 entries match the CPU LUT.
   **Note for the next look:** with the user's saved settings (`refl_floor_enabled` at
   20 dBZ + CC gate on), only ~7% of the scope has echo — the broad blue/green light-rain
   shield is *QC'd away by settings, not a rendering bug*. Toggle the reflectivity floor
   off in the settings panel to judge the smooth+sharp look against a reference viewer.
2. **Tune to taste** once seen:
   - `SMOOTH_RADIUS_PX` (`scope.rs`): 0 = maximum crispness, higher = softer. Currently 3.
   - `SEAL_RADIUS_PX`: how aggressively interior QC holes fill.
   - LUT range `DBZ_LUT_MIN/MAX` (`colors.rs`) if colors look off at the extremes.
3. **Possible issues to watch for:**
   - Coordinate/flip: radar drawn via raw `draw_texture_ex` (there's a documented history of
     render_to_texture flips). Verify the shader path didn't introduce any orientation change.
   - GPU value interpolation runs across the whole texture incl. coverage edges; confirm the
     `A < 0.5` discard gives clean storm outlines, not a faint halo.
4. **Commit.** Everything is uncommitted. Suggested split: (a) value-field smoothing +
   hole-fill, (b) resolution bump, (c) GPU palette shader. Branch already has prior QC commits.
5. **Deferred (only if needed):**
   - **Zoom-adaptive rasterization** — 2048 is still a *fixed* texture; extreme zoom eventually
     shows texels. True any-zoom fix = re-rasterize only the visible window at zoom. Big change.
   - **Separable seal / BFS hole-fill** — `fill_enclosed_holes` seal is O(n·r²); if re-raster
     (fires on new scan / QC toggle, not per frame) feels laggy at 2048, make the mask dilation
     separable O(n·r) and the fill BFS-based. `ponytail:` comments mark the spots.
   - **Extend the shader to other products** (velocity, ZDR, …) — currently only reflectivity
     gets the smooth+sharp treatment; others are CPU-colorized. Each needs its palette LUT.

## Verify commands

```
cargo test -p rustywx --lib      # 200 pass
cargo clippy -p rustywx          # clean
cargo run -p rustywx --release   # <-- visual check (the missing step)
```
