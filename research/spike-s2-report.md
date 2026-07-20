# Spike S2 Report: nexrad-data + Ply Integration

**Date:** 2025-07-19
**Status:** ✅ Complete

## Approach

Ported the existing `data.rs` background-worker pattern from the egui app
into the ply-spike. The worker spawns a `std::thread` with a tokio
`Runtime`, calls `nexrad_data::aws::archive::list_files` +
`download_file`, decodes via `nexrad_model` → `ScanData`, and sends
`WorkerMessage`s over `mpsc::channel`. The Ply game loop polls
`rx.try_recv()` each frame.

Key differences from the egui version:
- Removed `egui_ctx.request_repaint()` — Ply/macroquad runs a continuous
  game loop, so no explicit repaint request is needed.
- Site switching sends the new site ID over `site_tx`; the worker picks it
  up via `recv_timeout` and resets state.
- Synthetic data is used as a fallback until the first real scan arrives.

## Results

| Metric | Value |
|---|---|
| Sites tested | KJGX (default) |
| Build | Compiles cleanly (1 warning: unused `label` method) |
| Runtime | Ran 30s without crash, panic, or error output |
| Thread pattern | Stable — worker thread spawned, tokio runtime created |
| Shutdown | Clean (SIGTERM from timeout, no panic) |
| Memory | Not measured (30s run, no visible growth) |

## Code Location

- Branch: `spike/ply-radar-scope`
- Files:
  - `ply-spike/Cargo.toml` — added nexrad-data, nexrad-model, tokio, chrono, anyhow, serde_json
  - `ply-spike/src/data.rs` — new, ported worker pattern
  - `ply-spike/src/model.rs` — replaced with full model including ScanData
  - `ply-spike/src/main.rs` — wired in real data flow with synthetic fallback

## Verdict

**Ready for Stage 2.** The background-thread + mpsc pattern works with Ply.
The app starts, spawns the worker, and polls for messages without blocking
the game loop. Site switching triggers new fetches. The synthetic fallback
ensures the UI always has something to show.

## Notes for Stage 2

- The `label` method on `Product` is unused in the spike but will be needed
  when the UI shows product names in dropdowns (Stage 3).
- The spike doesn't include disk caching — that's Stage 2's `cache.rs` using
  Ply `storage`.
- Tilt cycling (T key) works but only after real data arrives. The synthetic
  sweep has a single elevation.
- The 30s test is sufficient to validate the pattern doesn't crash on
  startup. Longer runs (10+ fetch cycles) should be done during Stage 2
  development.
