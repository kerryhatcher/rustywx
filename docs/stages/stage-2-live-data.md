# Stage 2: "Live Data" — Real NEXRAD

**Status:** 🔲 Not started
**Tag:** `v0.2.0-stage2`

## Goal

Real radar data from AWS, site/product/tilt controls.

## Scope

- `data.rs` — the background-worker pattern is already ported from the spike
  (Stage 1 kept it in the codebase but did not spawn the worker). Stage 2
  enables it: uncomment `data::spawn_worker(...)` in `main.rs` and wire up
  the full data flow. The worker spawns a `std::thread` with a tokio
  runtime, runs `nexrad_data::aws::archive::list_files` +
  `download_file`, decodes via `nexrad_model`, sends `WorkerMessage`s over
  `mpsc::channel`. The tokio runtime lives on its own thread — Ply's game
  loop polls `rx.try_recv()` each frame.
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

## Notes from Stage 1

- The synthetic sweep (`synthetic_sweep()`) should remain as a fallback
  while real data loads, then be replaced when the first scan arrives.
- The worker channels (`worker_rx`, `site_tx`) are already in `AppState`.
  The polling loop (`while let Ok(msg) = state.worker_rx.try_recv()`) is
  already in `main.rs` — it just never receives because the worker isn't
  spawned. Enabling the worker is a one-line change.
- The `data` module import was changed to `use rustywx::data::WorkerMessage;`
  in Stage 1. When enabling the worker, change back to
  `use rustywx::data::{self, WorkerMessage};` for `data::spawn_worker()`.
- The `let _ = worker_tx;` suppression and `_site_rx` underscore prefix
  need to be removed, and `data::spawn_worker(worker_tx, ...)` uncommented.
- nexrad-data and nexrad-model are already pinned with `=` in
  `ply-spike/Cargo.toml`.

## Deliverable

`cargo run` → real radar data from KJGX (or any site), auto-refreshes every
2 minutes, cached on disk.

## Validation

- [ ] App boots and shows "Loading…" then real radar data
- [ ] Scan timestamp visible in status bar
- [ ] Site switching fetches new data (Left/Right arrow keys)
- [ ] Product toggle changes display (R/V keys)
- [ ] Tilt cycling works (T key)
- [ ] Data cached — restart app, data loads instantly
- [ ] Error shown if network unavailable (graceful)
- [ ] Auto-refresh picks up new scans
- [ ] `git push` → CI passes → `git tag v0.2.0-stage2` → `git push --tags`
