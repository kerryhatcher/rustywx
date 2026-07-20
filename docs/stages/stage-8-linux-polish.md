# Stage 8: "Linux Polish" — Performance & Accessibility

**Status:** 🔲 Not started
**Tag:** `v1.0.0-stage8`

## Goal

Optimize for Linux desktop, ensure accessibility compliance.

## Scope

- Performance profiling (frame time, texture cache hit rate)
- **Rasterization benchmarks** — add `#[bench]` tests for sweep rasterization
  at various resolutions (128×128, 256×256, 512×512, 1024×1024) and sweep
  sizes (few radials vs. full 360-radial sweep). Modeled on the thesis's
  systematic benchmarking methodology (resolution × sampling rate tables,
  hardware comparison across 5 machines). These benchmarks catch performance
  regressions as features grow. See `docs/post-v1-multi-site-animation.md`.
- HiDPI testing (verify scaling on high-DPI displays)
- Wayland testing (verify native Wayland support, no X11 fallback issues)
- Accessibility audit (labels, tab order, screen reader via Ply's `a11y`
  feature — AccessKit on desktop)
- System tray icon (optional — runs in background, quick access to current alerts)
- Final cleanup and release build

## Notes from Stage 1

- The app uses `high_dpi: true` in the window config. Text sizes were
  bumped during Stage 1 validation (10–12px → 16–18px) for readability.
  HiDPI testing should verify these sizes look correct across different
  display scales.
- The scope is drawn directly to the screen (not through
  `render_to_texture`). Performance profiling should verify this doesn't
  cause unnecessary redraws when the scope is static.

## Deliverable

Optimized Linux desktop build with full accessibility support.

## Validation

- [ ] Frame time <16ms (60fps) on target hardware
- [ ] Texture cache hit rate >90% (scope not re-rendered when static)
- [ ] HiDPI display renders correctly (no blurry UI)
- [ ] Wayland native — no X11 warnings or fallback
- [ ] Screen reader announces controls correctly
- [ ] Tab navigation works through all interactive elements
- [ ] Release build runs without debug overhead
- [ ] (Optional) System tray icon shows current alert status
- [ ] Rasterization benchmarks exist and run in CI (or documented as manual)
- [ ] `git push` → CI passes → `git tag v1.0.0-stage8` → `git push --tags`
