# Stage 7: "Settings & Polish" — Configuration + Fit & Finish

**Status:** 🔲 Not started
**Tag:** `v0.5.0-stage7`

## Goal

Settings panel, keyboard shortcuts, edge cases, cleanup.

## Scope

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
- **RLE compression for radar cache** — add a compressed cache format in
  `cache.rs` using the modified Run Length Encoding algorithm from Yi Ru (2007).
  The thesis achieves 99%+ compression (8 MB → 54 KB for 256×256×128 volumes)
  using a simple scheme: 8-bit byte with the first bit as a repetition flag,
  followed by a 32-bit unsigned integer run count. This enables storing
  hundreds of historical frames locally for animation playback without disk
  bloat. See `docs/post-v1-multi-site-animation.md` for the full algorithm.
- **Spectrum Width product** — add Spectrum Width as a third selectable
  product (alongside Reflectivity and Velocity). Spectrum Width is a base
  data moment already decoded by `nexrad-model` (it measures velocity
  dispersion in the sample volume — high values indicate turbulence, shear,
  or mixed targets). Only needs a new color table in `colors.rs` and a
  `Product::SpectrumWidth` variant. FMH-11 Part A §4.2.1 defines it as
  "standard deviation of the mean radial velocity spectrum."
  See `docs/research/2021_fmh11_parta.md`.
- **VCP / scan mode display** — show the active Volume Coverage Pattern
  (e.g., "VCP 12 — Precipitation") and operational mode in the status bar.
  VCP info is available from the decoded scan metadata. This gives users
  context about data quality and scan strategy (VCP 12 = 14 elevations
  in ~4.5 min for severe weather; VCP 31 = 5 elevations in ~10 min for
  clear air). FMH-11 Part A §4.4–4.5 defines all operational modes and VCPs.
  See `docs/research/2021_fmh11_parta.md`.
- **Nyquist velocity display** — show the Nyquist velocity (unambiguous
  velocity range) for the current tilt in the status bar. This helps users
  interpret velocity data: values near ±Nyquist are at the edge of the
  unambiguous range and may be aliased. The Nyquist velocity is derivable
  from the PRT in the scan metadata: `Va = λ / (4 × PRT)`. Hubbert et al.
  explain the range-velocity dilemma and why purple haze occurs.
  See `docs/research/atot-JTECH-D-25-0059.1.md`.
- **Tunable TDBZ kernel size** — expose the TDBZ clutter filter kernel size
  as a setting (currently hardcoded to 9×9 in `scope.rs`). Keem et al.
  tested window sizes from 3×3 to 41×41 and found accuracy improved
  monotonically from 98.99% to 99.97%. Larger windows are better at removing
  wind turbine clutter but may suppress weak precipitation at storm edges.
  Offer presets: "Sensitive" (5×5), "Default" (9×9), "Aggressive" (13×13).
  See `docs/research/remotesensing-18-00827-with-cover.md`.

## Notes from Stage 1

- The old `src/` directory (egui-based) is orphaned but still present. It
  is not built by any crate (workspace root has no `[package]`). This
  stage removes it entirely.
- `CONTRIBUTING.md` references the old build structure and may need updates.

## Deliverable

Polished, configurable app.

## Validation

- [ ] Settings panel opens via gear icon
- [ ] Default site setting works (app starts on chosen site)
- [ ] Animation level can be set to Full/Subtle/None
- [ ] Overlay defaults respected on startup
- [ ] ? key shows keyboard shortcuts overlay
- [ ] Network errors show user-friendly message, not crash
- [ ] Corrupt cache is handled gracefully
- [ ] No egui imports remain in codebase
- [ ] RLE compression achieves ≥90% space savings on cached radar volumes
- [ ] Spectrum Width product displays with appropriate color table
- [ ] VCP and scan mode shown in status bar
- [~] Nyquist velocity in status bar — shows "Nyquist —". Deferred: neither
  `nexrad-model` (1.0.0-rc.2) nor `nexrad-data` (1.0.0-rc.7) exposes Nyquist
  velocity or PRT in their public API; deriving it needs hand-parsing the
  raw message-31 radial header. Moved to post-v1 rather than fabricate a
  physical value. Status-bar slot + label plumbing already in place.
- [ ] TDBZ kernel size selectable in settings (Sensitive / Default / Aggressive)
- [ ] `git push` → CI passes → `git tag v0.5.0-stage7` → `git push --tags`
