# Research Summary — Gaps Resolved

**Date:** 2025-07-19

Four research documents were produced. Here's how they change the original review.

---

## Issues RESOLVED (Ply APIs confirmed real)

| Original Concern | Finding | Status |
|---|---|---|
| Ply `net` module may not exist | **Exists.** Polling-based HTTP + WebSocket. Uses `ureq` + `tokio` internally, WASM uses browser XHR. | ✅ Resolved |
| Ply `storage` module may not exist | **Exists.** Cross-platform persistent file storage. `save_string`, `load_string`, `save_bytes`, `load_bytes`. | ✅ Resolved |
| `built-in-shaders` may not exist | **Exists.** FOIL, HOLOGRAPHIC, DISSOLVE, GLOW, CRT, GRADIENT_*. No built-in blur, but custom GLSL shaders supported. | ✅ Resolved (need custom blur shader) |
| `a11y` feature may not exist | **Exists.** AccessKit on desktop, JS bridge on web. On by default. | ✅ Resolved |
| `plyx` CLI may not exist | **Exists.** `init`, `web`, `apk`, `ios`, `add` commands confirmed. | ✅ Resolved |
| Async runtime mismatch | **Ply's `net` solves this.** Polling-based — fire requests, poll each frame. No tokio management needed in app code. | ✅ Resolved |

## Issues CONFIRMED (real problems)

| Original Concern | Finding | Severity |
|---|---|---|
| WASM CORS for NEXRAD S3 | **Confirmed blocker.** S3 buckets don't serve CORS headers for anonymous browser access. A relay proxy is required. | 🔴 Critical for WASM |
| `nexrad-data` replacement non-trivial | **Confirmed.** The crate does S3 API + binary decoding. Replacing just the HTTP layer is feasible; replacing the decoder is not. | 🟡 Medium (keep the crate) |
| NWS API CORS | **Works!** `Access-Control-Allow-Origin: *` confirmed. Just don't set custom User-Agent from browser. | ✅ Not an issue |
| NHC CORS | **Unknown.** No evidence found either way. Needs testing. | 🟡 Medium |

## Issues PARTIALLY RESOLVED

| Original Concern | Updated Finding |
|---|---|
| Font loading | Ply uses `FontAsset::Path("...")` — fonts load from disk. WASM bundling still needs verification. |
| `webbrowser` crate | No Ply-native equivalent found. For WASM, external links need JS interop. For desktop, `webbrowser` crate may still work. |
| `image` crate compatibility | Ply supports PNG via its texture system. The `image` crate should still work for decoding NHC thumbnails into raw RGBA bytes that Ply can consume. |

## Updated Recommendations

### 1. Keep `nexrad-data` — don't replace it
The plan should keep `nexrad-data` for S3 access + NEXRAD decoding. Use Ply's `net` for simpler HTTP (borders, alerts, NHC JSON). Run `nexrad-data` on a background thread with `mpsc` channels — same pattern as the existing `data.rs`.

### 2. WASM needs a relay proxy
Stage 8's WASM target cannot fetch radar data directly from S3. Options:
- Build a simple Cloudflare Worker relay
- Accept WASM as "UI + overlays only, no live radar"
- Use synthetic/demo data for WASM

### 3. Custom blur shader needed for Stage 6
Ply's `built-in-shaders` has GLOW but no Gaussian blur. A custom GLSL ES 1.00 fragment shader will be needed for the frosted glass effect. This is doable but requires GLSL expertise.

### 4. Test NHC CORS early
Before Stage 5, verify that `www.nhc.noaa.gov/CurrentStorms.json` and `mapservices.weather.noaa.gov` serve CORS headers. If not, they'll need the same relay proxy as NEXRAD.

### 5. The plan's structure is sound
The staged approach, validation checklists, and git workflow are all good. The timeline (8-10 days) is still optimistic but more plausible now that the Ply API assumptions are validated. The main schedule risk is the custom blur shader (Stage 6) and the WASM relay proxy (Stage 8).

## Updated Cargo.toml Target

```toml
[package]
name = "rustywx"
version = "0.2.0"
edition = "2024"

[dependencies]
ply-engine = { version = "1.1", features = [
    "net",
    "net-json",
    "storage",
    "built-in-shaders",
    "text-styling",
] }
nexrad-data = "1.0.0-rc.7"
nexrad-model = "1.0.0-rc.2"
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
image = { version = "0.25", default-features = false, features = ["png", "jpeg"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["rt", "time"] }
webbrowser = "0.8"
zip = "2"

# Removed from original:
# eframe, egui — replaced by ply-engine
# ureq — replaced by ply-engine net
# rusqlite — replaced by ply-engine storage
```
