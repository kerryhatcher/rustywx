# Stage 8: "Desktop Polish" — Performance & Accessibility

**Status:** 🚧 In progress — automated work complete, hardware validation pending
**Tag:** `v1.0.0-stage8` (not yet tagged)

## What shipped

- **Rasterization benchmarks** — `app/benches/rasterize.rs` (criterion): `scope::rasterize`
  over resolutions 128/256/512/1024 × sparse (30) and full (360) radial sweeps.
  Run with `cargo bench`. Fixtures via new `model::synthetic_sweep(...)`.
- **Release profile** — workspace `[profile.release]` (opt-level 3, thin LTO,
  codegen-units 1, strip). `panic=abort` deliberately omitted (no `catch_unwind`).
- **Accessibility** — ply-engine 1.1.1 exposes a real per-widget a11y builder
  (`.accessibility(|a| a.button(...).checked(...))`, `AccessibilityRole`,
  live regions). Wired roles/labels into dropdowns, product toggle, collapsing
  headers, settings toggles/buttons, overlay toggles, NHC link, and gave the
  alert toast an assertive live region so fetch failures are announced.
- **Texture cache** — verified already handled by the `needs_reraster` flag
  (scope rasterizes only on change, not per frame); no new caching needed.
- **Rename** — the crate directory `ply-spike/` (a leftover from the ply-engine
  porting spike) was renamed to `app/`.

Hardware-gated validation (HiDPI, Wayland, Finder launch, screen-reader
announcement, live frame-time) is documented in
[`docs/stage-8-manual-validation.md`](../stage-8-manual-validation.md).

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

- [~] Texture cache hit rate >90% — satisfied by `needs_reraster` design (rasterize only on change)
- [x] Release build runs without debug overhead — `[profile.release]` added
- [x] Rasterization benchmarks exist — `app/benches/rasterize.rs`, run via `cargo bench` (manual, not in CI)
- [x] Accessibility roles/labels wired into interactive controls (screen-reader announcement to verify on hardware)
- [ ] Frame time <16ms (60fps) on target hardware — manual, see manual-validation doc
- [ ] HiDPI renders correctly, no blurry UI (Linux fractional scale + macOS Retina) — manual
- [ ] Wayland native — no X11 warnings or fallback (Linux) — manual
- [ ] macOS native window works, incl. launch from Finder/dock (assets resolve) — manual
- [ ] Screen reader announces controls correctly (Orca on Linux, VoiceOver on macOS) — manual
- [ ] Tab navigation works through all interactive elements — manual
- [ ] (Optional) Alert-status indicator: system tray (Linux) / menu bar (macOS) — deferred
- [ ] `git push` → CI passes → `git tag v1.0.0-stage8` → `git push --tags` (awaiting go-ahead)
