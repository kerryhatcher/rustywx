# Macroquad Async HTTP Patterns — Research Findings

**Date:** 2025-07-19
**Source:** Rust forum, macroquad GitHub, community discussions

## Key Finding: Ply's `net` module solves this

Ply's `net` feature provides a **polling-based HTTP API** that handles all the async complexity internally. You do NOT need to manage tokio runtimes or threads yourself. See `ply-api-surface.md` for the API details.

However, if you ever need to do custom async work (e.g., using `nexrad-data` directly), here are the patterns:

---

## Macroquad's Async Model

Macroquad uses `async`/`.await` as a **stack-preserving mechanism** for WASM compatibility — NOT as a general-purpose async runtime. There is no executor. The `#[macroquad::main]` attribute transforms the main function to work cross-platform.

Key implications:
- `.await` in macroquad is for macroquad's own async functions (`next_frame().await`, `load_texture().await`)
- There is NO built-in async executor for general futures
- The main loop is single-threaded

## Pattern 1: Tokio Runtime (Desktop Only)

Create a tokio `Runtime` before the loop, spawn tasks on it, and poll channels each frame:

```rust
#[macroquad::main("My App")]
async fn main() {
    let (tx, rx) = std::sync::mpsc::channel();

    // Create tokio runtime OUTSIDE the loop
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Spawn background work
    rt.spawn(async move {
        loop {
            let data = fetch_data().await.unwrap();
            tx.send(data).unwrap();
            tokio::time::sleep(Duration::from_secs(120)).await;
        }
    });

    loop {
        // Poll for results each frame (non-blocking)
        if let Ok(data) = rx.try_recv() {
            // Process data
        }

        // Render...
        next_frame().await;
    }
}
```

**Caveats from community reports:**
- Some users report runtime crashes with this pattern
- `rt.block_on()` must NOT be called on the UI thread
- This pattern does NOT work on WASM (no threads)

## Pattern 2: Ply's `net` Module (Recommended)

This is the approach the port plan should use. No tokio management needed:

```rust
// Fire request (non-blocking)
net::get("radar", "https://example.com/radar.bin", |r| r);

// Poll each frame
if let Some(req) = net::request("radar") {
    if let Some(Ok(resp)) = req.response() {
        let data = resp.bytes();
        // Process...
    }
}
```

Ply handles:
- Background threads on native
- Browser XHR on WASM
- Auto-cleanup of completed requests
- Idempotent request deduplication

## Pattern 3: Channel-Based Worker (for nexrad-data)

If you need to use `nexrad-data` directly (which uses `reqwest` + `tokio` internally), you can spawn a tokio runtime on a separate thread:

```rust
use std::sync::mpsc;

enum UiMessage {
    NewScan(ScanData),
    Error(String),
}

#[macroquad::main("rustywx")]
async fn main() {
    let (tx, rx) = mpsc::channel::<UiMessage>();

    // Spawn OS thread for tokio work
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            loop {
                match fetch_latest_scan("KJGX", None).await {
                    Ok(Some(scan)) => tx.send(UiMessage::NewScan(scan)).unwrap(),
                    Err(e) => tx.send(UiMessage::Error(e.to_string())).unwrap(),
                    _ => {}
                }
                tokio::time::sleep(Duration::from_secs(120)).await;
            }
        });
    });

    loop {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                UiMessage::NewScan(scan) => { /* update state */ }
                UiMessage::Error(e) => { /* show error */ }
            }
        }
        next_frame().await;
    }
}
```

**This is essentially what the existing egui app does** — it just uses `data.rs` with `std::thread::spawn` and `mpsc` channels. The same pattern works with macroquad.

## Recommendation for the Port Plan

1. **Use Ply's `net` module for simple HTTP fetches** (borders GeoJSON, NWS alerts, NHC JSON). This is the cleanest approach and works on all platforms including WASM.

2. **Keep the thread + channel pattern for `nexrad-data`** — since it does S3 API calls that Ply's `net` doesn't handle natively. This works on desktop. For WASM, use `nexrad`'s `wasm` feature (see nexrad-data-analysis.md).

3. **Don't try to mix tokio into the macroquad main loop** — use threads + channels for anything complex.
