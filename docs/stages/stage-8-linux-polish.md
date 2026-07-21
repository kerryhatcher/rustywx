# Stage 8: "Desktop Polish" — Performance & Accessibility

**Status:** 🔲 Not started
**Tag:** `v1.0.0-stage8`

## Goal

Optimize for **Linux and macOS desktop** (both v1 targets), ensure
accessibility compliance. The ply-engine backend is cross-platform;
v1 ships and is validated on both. macOS was confirmed working during
Stage 7 (fetch, cache, and scope/border rendering all verified).

## Scope

- Performance profiling (frame time, texture cache hit rate)
- **Rasterization benchmarks** — add `#[bench]` tests for sweep rasterization
  at various resolutions (128×128, 256×256, 512×512, 1024×1024) and sweep
  sizes (few radials vs. full 360-radial sweep). Modeled on the thesis's
  systematic benchmarking methodology (resolution × sampling rate tables,
  hardware comparison across 5 machines). These benchmarks catch performance
  regressions as features grow. See `docs/post-v1-multi-site-animation.md`.
- HiDPI testing (verify scaling on high-DPI displays — Linux fractional
  scaling AND macOS Retina)
- Linux windowing: Wayland testing (verify native Wayland support, no X11
  fallback issues)
- macOS windowing: verify on native (Cocoa/Metal via ply-engine); confirm
  window chrome, menu bar, and asset/CWD handling work when launched from
  Finder/dock (not just `cargo run`)
- Accessibility audit (labels, tab order, screen reader via Ply's `a11y`
  feature — AccessKit: Orca/AT-SPI on Linux, VoiceOver on macOS)
- Background/quick-access alert indicator (optional — system tray on Linux,
  menu bar item on macOS)
- Final cleanup and release build for **both** Linux and macOS

## Notes from Stage 1

- The app uses `high_dpi: true` in the window config. Text sizes were
  bumped during Stage 1 validation (10–12px → 16–18px) for readability.
  HiDPI testing should verify these sizes look correct across different
  display scales.
- The scope is drawn directly to the screen (not through
  `render_to_texture`). Performance profiling should verify this doesn't
  cause unnecessary redraws when the scope is static.

## Deliverable

Optimized Linux and macOS desktop builds with full accessibility support.

## Validation

- [ ] Frame time <16ms (60fps) on target hardware (Linux and macOS)
- [ ] Texture cache hit rate >90% (scope not re-rendered when static)
- [ ] HiDPI renders correctly, no blurry UI (Linux fractional scale + macOS Retina)
- [ ] Wayland native — no X11 warnings or fallback (Linux)
- [ ] macOS native window works, incl. launch from Finder/dock (assets resolve)
- [ ] Screen reader announces controls correctly (Orca on Linux, VoiceOver on macOS)
- [ ] Tab navigation works through all interactive elements
- [ ] Release build runs without debug overhead (both platforms)
- [ ] (Optional) Alert-status indicator: system tray (Linux) / menu bar (macOS)
- [ ] Rasterization benchmarks exist and run in CI (or documented as manual)
- [ ] `git push` → CI passes → `git tag v1.0.0-stage8` → `git push --tags`
