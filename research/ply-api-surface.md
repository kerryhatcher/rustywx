# Ply Engine API Surface — Research Findings

**Date:** 2025-07-19
**Source:** crates.io, docs.rs, plyx.iz.rs/docs

## Verdict: All assumed APIs EXIST

Every feature the port plan assumes about Ply is confirmed real and documented.

---

## Feature Flags (from Cargo.toml)

| Feature | What it provides | Plan relevance |
|---|---|---|
| `net` | HTTP + WebSocket (uses `ureq` + `tokio` internally) | Stages 2, 4, 5 |
| `net-json` | JSON deserialization for network responses (adds `serde`/`serde_json`) | Stages 2, 4, 5 |
| `storage` | Cross-platform persistent file storage (`Storage::new`, `save_string`, `load_string`, `save_bytes`, `load_bytes`) | Stages 2, 7 |
| `built-in-shaders` | Pre-made GLSL shaders: FOIL, HOLOGRAPHIC, DISSOLVE, GLOW, CRT, GRADIENT_LINEAR, GRADIENT_RADIAL, GRADIENT_CONIC | Stage 6 |
| `a11y` | AccessKit on desktop, JS bridge on web. Screen readers, keyboard nav, focus rings, tab order, live regions | Stage 8 |
| `text-styling` | Rich text: inline colors, wave, pulse, gradient, typewriter, fade | Stage 6 |
| `audio` | WAV/OGG playback | Not needed |
| `tinyvg` | TinyVG vector graphics | Not needed |
| `shader-build` | SPIR-V shader compilation pipeline | Optional |

## CLI Tool (`plyx`)

Confirmed commands:
- `cargo install plyx` — install the CLI
- `plyx init` — scaffold a new Ply project
- `plyx web` — build for WASM
- `plyx apk` — build for Android
- `plyx ios` — build for iOS
- `plyx add <feature>` — add a feature flag

## Networking API (`net` feature)

Uses a **polling-based** model — no async/await needed in the game loop:

```rust
// Fire a request (non-blocking, idempotent by ID)
net::get("users", "https://api.example.com/users", |r| r
    .header("Authorization", "Bearer token123")
);

// Poll for response each frame
if let Some(req) = net::request("users") {
    match req.response() {
        None => { /* loading */ }
        Some(Ok(resp)) => {
            let status = resp.status();   // u16
            let body = resp.text();       // &str
            let raw = resp.bytes();       // &[u8]
        }
        Some(Err(e)) => { /* error */ }
    }
}
```

Key details:
- Native builds use background threads (HTTP) and tokio runtime (WebSocket)
- WASM builds use browser's XMLHttpRequest/WebSocket APIs
- Requests auto-cleanup after 60 frames of not being accessed
- `net::post`, `net::put`, `net::delete` also available
- WebSocket support via `net::ws_connect` / `net::ws`

**This is the key insight:** Ply's `net` module handles the async complexity internally. You just fire requests and poll. No tokio management needed in application code. This directly addresses the async runtime concern from the review.

## Storage API (`storage` feature)

```rust
let storage = Storage::new("my_app/data").await?;
storage.save_string("settings.json", "{\"theme\":\"dark\"}").await?;
let settings = storage.load_string("settings.json").await?; // Ok(None) if missing
storage.save_bytes("saves/slot1.bin", &bytes).await?;
storage.remove("saves/old_slot.bin").await?;
storage.export("settings.json").await?; // native save dialog
```

Platform paths:
- Linux: `~/.local/share`
- macOS: `~/Library/Application Support`
- Windows: `%APPDATA%`
- Web: OPFS root
- Android: App-specific storage

## Shaders (`built-in-shaders` feature)

Available built-in effects: FOIL, HOLOGRAPHIC, DISSOLVE, GLOW, CRT, GRADIENT_LINEAR, GRADIENT_RADIAL, GRADIENT_CONIC

Custom shaders use GLSL ES 1.00 (compatible with WebGL 1 & 2). Applied per-element via `.effect()` or to element groups via `.shader()`.

**For the frosted glass effect (Stage 6):** There's no built-in "blur" shader listed, but custom GLSL shaders can be written. A Gaussian blur shader would need to be authored. The `GLOW` built-in might serve as a starting point.

## Accessibility (`a11y` feature, default on)

Uses AccessKit on desktop platforms and a JS bridge on web. Supports:
- Screen readers
- Keyboard navigation
- Focus rings
- Tab order
- Live regions

## Recommended Cargo.toml for rustywx

```toml
[dependencies]
ply-engine = { version = "1.1", features = [
    "net",
    "net-json",
    "storage",
    "built-in-shaders",
    "text-styling",
] }
# a11y is on by default
```

## Gaps / Concerns

1. **No built-in blur shader** — Stage 6's frosted glass effect will need a custom GLSL blur shader. This is doable but requires GLSL expertise.

2. **`plyx init` vs manual scaffold** — The spike was scaffolded manually. `plyx init` may produce a different project structure. Worth testing before Stage 1.

3. **`net` uses `ureq` internally** — This is good for native but means WASM uses browser XHR. The CORS issues for WASM are separate (see cors-wasm-feasibility.md).

4. **`storage` is async** — Uses `.await`, which means it needs to be called within the macroquad async context. This should work fine since the main loop is already async.
