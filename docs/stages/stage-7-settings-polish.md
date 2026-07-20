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
- [ ] `git push` → CI passes → `git tag v0.5.0-stage7` → `git push --tags`
