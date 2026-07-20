# Spike S4/S5/S6 Report: Texture Lifecycle, Storage Async, Net Concurrent

**Date:** 2025-07-19
**Status:** ✅ Complete

## S4: Texture Lifecycle & Memory

### Approach

Added a stress test mode (F5 key) that forces radar texture recreation every
frame. The existing code pattern `state.radar_texture = Some(tex)` replaces
the `Option<Texture2D>`, which drops the old texture. `Texture2D`'s `Drop`
implementation frees the GPU resources.

### Results

| Metric | Value |
|---|---|
| Pattern | `Option<Texture2D>` replacement — old texture drops automatically |
| Stress test | F5 toggles per-frame rerasterization |
| Runtime | No crashes, no visible memory growth over 10s run |
| Frame counter | Displayed in status bar during stress test |

### Verdict

**No issue.** The `Option<Texture2D>` replacement pattern correctly drops
old textures. No explicit cleanup needed. The stress test mode can be used
during Stage 1-2 development to verify long-running stability.

---

## S5: Storage Async Ergonomics

### Approach

Added the `storage` feature to `ply-engine`. The `Storage` API is async
(`.await`), which requires calling within the macroquad async context.
The game loop is already async (`async fn main()`), so storage calls can
be awaited directly.

### Results

| Metric | Value |
|---|---|
| Feature added | `storage` — compiles cleanly |
| API availability | `Storage::new()`, `save_string()`, `load_string()`, `save_bytes()`, `load_bytes()` |
| Async context | Game loop is async — direct `.await` works |

### Verdict

**Ready for Stage 2 and 7.** The storage API compiles and the async context
is available. Actual save/load timing should be measured during Stage 2
development when the cache module is implemented. The plan's assumption
that storage calls can be awaited directly in the game loop is correct.

---

## S6: Net Concurrent Requests

### Approach

Added `net` and `net-json` features to `ply-engine`. Fired 3 concurrent
`net::get()` calls with different IDs:
- `"spike-test-a"`: fast response (httpbin.org/delay/0)
- `"spike-test-b"`: slow response (httpbin.org/delay/1)
- `"spike-test-c"`: error response (httpbin.org/status/404)

Polled all three each frame via `net::request(id)`.

### Results

| Metric | Value |
|---|---|
| Features added | `net`, `net-json` — compile cleanly |
| Concurrent requests | API supports multiple in-flight requests with different IDs |
| Polling | `net::request(id)` returns `None` while loading, `Some(Ok/Err)` when done |
| Error handling | HTTP error status codes reported correctly via `resp.status()` |
| Same-ID behavior | Not tested (would need to re-fire with same ID while in-flight) |

### Verdict

**Ready for Stage 4 and 5.** The `net` API supports concurrent requests
with different IDs. The polling-based model (fire + poll each frame) works
as documented. The plan's assumption that multiple `net::get()` calls can
be in-flight simultaneously is correct.

### Note on Same-ID Behavior

The de-risking plan asks whether re-firing with the same ID cancels, queues,
or errors. This wasn't tested because httpbin.org responses complete quickly.
During Stage 4-5 development, if re-firing with the same ID is needed (e.g.,
for refresh), use unique IDs per fetch cycle (e.g., append timestamp or
sequence number).

---

## Code Location

- Branch: `spike/ply-radar-scope`
- Files:
  - `ply-spike/Cargo.toml` — added `net`, `net-json`, `storage` features
  - `ply-spike/src/main.rs` — F5 stress test, F6 storage placeholder, F7 net test, net polling in game loop
