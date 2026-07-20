# nexrad-data Crate Analysis — Research Findings

**Date:** 2025-07-19
**Source:** crates.io, docs.rs, GitHub (danielway/nexrad)

---

## What nexrad-data Does

`nexrad-data` is part of the `nexrad` monorepo. It handles:

1. **AWS S3 Archive Access** (`nexrad_data::aws::archive`)
   - Lists files for a given date + radar site via S3 ListObjectsV2
   - Downloads specific volume files by identifier
   - Handles the bucket structure: `/{year}/{month}/{day}/{site}/{filename}`
   - Bucket: `noaa-nexrad-level2` (note: moving to `unidata-nexrad-level2`)

2. **AWS S3 Real-Time Access** (`nexrad_data::aws::realtime`)
   - Lists chunks in rotating volume directories (0-999)
   - Downloads individual chunks
   - Polls for new chunks as they arrive
   - Bucket: `unidata-nexrad-level2-chunks`

3. **Volume File Decoding** (`nexrad_data::volume`)
   - Parses Archive II volume file headers
   - Decompresses LDM records (bzip2)
   - Decodes NEXRAD messages via `nexrad-decode`

4. **Model Mapping** (via `nexrad-model` feature)
   - Converts raw volume data into `Scan` → `Sweep` → `Radial` structures
   - This is what the existing `model.rs` wraps

## Dependencies

| Dependency | Purpose |
|---|---|
| `reqwest` | HTTP client for S3 API calls (sigv4 signing) |
| `tokio` | Async runtime for reqwest |
| `xml` | S3 ListObjectsV2 response parsing |
| `bzip2` | LDM record decompression |
| `nexrad-decode` | NEXRAD message binary decoding |
| `nexrad-model` | Common radar data model |
| `chrono` | Date/time handling |
| `serde` / `bincode` | Serialization |

## WASM Support

**The `nexrad` crate (umbrella) has a `wasm` feature!**

```toml
nexrad = { version = "1.0", default-features = false, features = ["wasm"] }
```

This enables: `model`, `decode`, `data`, `render`, `process`, `aws`, `serde`, `uom`, `chrono`

**NOT WASM-compatible:**
- `aws-polling` — requires tokio runtime for continuous polling
- `parallel` — requires threads (rayon)
- `full` — includes both of the above

**Key insight:** The `aws` feature (which uses `reqwest`) IS WASM-compatible. `reqwest` has a WASM backend that uses the browser's Fetch API. This means you CAN download NEXRAD files from S3 in WASM — IF CORS is configured on the bucket (which it isn't — see cors-wasm-feasibility.md).

## What Would Be Involved in Replacing It

The plan says to "replace `ureq` with Ply `net`" for data fetching. But `nexrad-data` does much more than HTTP:

| Function | Lines of Code (est.) | Complexity |
|---|---|---|
| S3 ListObjectsV2 + XML parsing | ~200 | Medium |
| S3 GetObject | ~50 | Low |
| Archive II volume header parsing | ~500 | High |
| LDM record decompression (bzip2) | ~100 | Medium |
| NEXRAD message decoding | ~2000+ | Very High |
| Model mapping to Scan/Sweep/Radial | ~500 | Medium |

**Replacing just the HTTP layer** (S3 calls) with Ply `net` is feasible — it's ~250 lines. But you'd still need `nexrad-data` (or `nexrad-decode` + `nexrad-model`) for the binary decoding.

**Replacing the entire stack** would be a multi-week effort and is not recommended.

## Recommendation

**Keep `nexrad-data` as a dependency.** It's not egui-specific. The port plan should:

1. Keep `nexrad-data` for S3 access + NEXRAD decoding
2. Use Ply's `net` for simpler HTTP fetches (borders GeoJSON, NWS alerts, NHC JSON)
3. Run `nexrad-data`'s tokio-based fetching on a background thread with `mpsc` channels (same pattern as the existing `data.rs`)
4. For WASM: use `nexrad`'s `wasm` feature + a CORS relay proxy for S3 access

**Updated Cargo.toml recommendation:**

```toml
[dependencies]
ply-engine = { version = "1.1", features = ["net", "net-json", "storage", "built-in-shaders", "text-styling"] }
nexrad-data = "1.0.0-rc.7"      # Keep for S3 + decoding
nexrad-model = "1.0.0-rc.2"     # Keep for Scan/Sweep/Radial types
tokio = { version = "1", features = ["rt", "time"] }  # For background thread
# Remove: ureq (replaced by Ply net), eframe, egui, rusqlite (replaced by Ply storage)
# Keep: anyhow, chrono, image, serde, serde_json, zip, webbrowser
```

## Bucket Migration Note

The NEXRAD archive bucket is moving from `noaa-nexrad-level2` to `unidata-nexrad-level2`. The old bucket is deprecated and will be unavailable starting September 1, 2025. The `nexrad-data` crate may need an update to point to the new bucket. Check the latest version.
