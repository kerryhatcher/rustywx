# Windows performance and stability review

- **Date:** 2026-10-09
- **Commit reviewed:** `081df73` (`feat: make first-load processing non-blocking`)
- **Scope:** all of `app/src`, the Cargo manifests, `.github/workflows`, `packaging/`, and the `Justfile`
- **Dependency sources read:** ply-engine 1.1.1, miniquad-ply 0.4.8, macroquad-ply 0.4.14, ureq 3.3.0, nexrad-data 1.0.0-rc.7
- **Method:**
  - 7 read-only reviewers (2 Opus, 5 Sonnet), each owning one slice of the codebase. Nothing was edited, built, or run.
  - Each reviewer had to ground every finding in the code before reporting it.
  - The lead reviewer then re-read the source for the findings marked ✔ below.

## How to read this

**Severity:**
- **Critical:** data loss, a permanent feature outage, or a multi-second freeze in normal use.
- **High:** a likely crash, a feature that silently stops working, or a large performance cost on common hardware.
- **Medium:** real but bounded impact, or a narrower trigger.
- **Low:** latent problems, minor waste, or polish.

**Platform:**
- **Win** means Windows only.
- **Win+** means cross-platform, but Windows makes it worse.
- **All** means it applies equally on every platform.

**Confidence:**
- **Confirmed:** the mechanism was read in the source, and the trigger is reachable.
- **Plausible:** the mechanism was read, but the trigger or size was not shown.
- **✔** means the lead reviewer re-checked the cited lines against the source.

**Estimates:** where a time in ms or seconds is given, it's a reviewer's estimate from counting operations, not a measurement.

Line numbers refer to commit `081df73`. Dependency paths are relative to `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`.

---

## Summary table

| ID | Severity | Platform | Area | Finding |
|---|---|---|---|---|
| C1 | Critical | Win+ | cache | A ~400 MB scan is serialized to JSON on the UI thread every launch, site switch, and new volume, and the scan cache grows without limit |
| C2 | Critical | All | network | One failed fetch permanently disables alerts, borders, and NHC until restart, and shows a toast every frame |
| C3 | Critical | Win+ | network | No HTTP timeouts anywhere, and a stalled request is never cleared, so feeds freeze silently after sleep/resume or a VPN change |
| H1 | High | Win | network | TLS uses only built-in roots and only env-var proxies; Windows cert store and system proxy are ignored |
| H2 | High | Win | input | One mouse-wheel notch on Windows (±120) jumps zoom to its limit and scrolls modals to the end |
| H3 | High | All | render | `fill_enclosed_holes` copies 16 MB per pass, one pass per pixel of hole radius, and is triggered by the range floor |
| H4 | High | All | render | Brute-force neighbourhood filters (dilate/erode/close/smooth) scan the full 2048² grid on every reflectivity raster |
| H5 | High | Win+ | render | Every border, alert, and NHC vertex is re-projected with haversine math twice per frame, before culling |
| H6 | High | All | network | `borders::poll_raw` stops at the first `?`, so the other responses are cleared before they're read and borders never load |
| H7 | High | Win | build | Windows is never built or tested in CI; the first Windows compile is in the release job, after the tag exists |
| H8 | High | Win | runtime | Console-subsystem exe, no panic hook, no log file; closing the console kills the app mid-write; QuickEdit stalls the render thread |
| H9 | High | Win+ | crash | UI-thread panics: byte slicing NHC text (`[..300]`) and alert names (`[..30]`), and miniquad unwraps on a dropped file |
| M1 | Medium | Win+ | data | A stale cached scan can overwrite a newer live scan, and stays until the next volume arrives |
| M2 | Medium | All | render | A failed raster retries every frame forever; the radar texture is blanked on every submit, and results are dropped while dirty |
| M3 | Medium | Win+ | storage | Settings and cache writes race, aren't atomic, and fail silently; a torn `settings.json` is deleted, losing all settings |
| M4 | Medium | Win+ | startup | `expect`s on storage init, the tokio runtime, and WGL abort startup with no message |
| M5 | Medium | Win | DPI | Default window plus `is_mobile < 900` puts most 125–150% Windows laptops in the mobile layout; system-DPI-aware only; no manifest |
| M6 | Medium | Win+ | render | Background "gradient" draws 24 near-full-screen blended circles per frame, and only one is visible |
| M7 | Medium | Win | render | No throttle while minimized or with vsync off; animation speed depends on frame rate |
| M8 | Medium | All | input | Typed characters build up in macroquad's LIFO queue and are later inserted into fields in reverse |
| M9 | Medium | Win+ | input | Global shortcuts fire while typing in the Forecast search box; Windows fullscreen exit restores to (0,0) |
| M10 | Medium | All | perf | Re-raster prep on the UI thread: deep sweep clones and per-re-raster melting-layer detection |
| M11 | Medium | Win+ | perf | KDP derivation does about 17M small allocations per volume, unconditionally |
| M12 | Medium | All | perf | NHC phase-2 work (KMZ inflate, KML parse, GIS JSON, image copies) runs on the UI thread |
| M13 | Medium | All | network | NHC refresh starts about 9 threads per storm, with no conditional GET, and re-uploads unchanged textures |
| M14 | Medium | All | network | One unsupported alert geometry drops the whole alert list; the national feed is re-downloaded in full every 120 s against a 10 MiB cap |
| M15 | Medium | Win+ | render | A new 16 MB texture is allocated and uploaded per raster, and macroquad's `unbatched` list grows without bound |
| M16 | Medium | Win+ | memory | Site switches start uncancellable ~400 MB cache loads; rapid switching stacks them |
| M17 | Medium | Win+ | threads | A panic in the data worker or parse worker dies silently, and live updates stop for the session |
| M18 | Medium | Win | render | Palette shader compile failure panics, with no CPU-colour fallback |
| M19 | Medium | Win | release | Dynamic MSVC CRT, no exe icon or version resource, unsigned zip with no checksum, a dead `assets/` copy |
| M20 | Medium | All | network | Unbounded KMZ decompression; case-sensitive `.kml` match |
| M21 | Medium | All | crash | Byte-slice panics on short NHC IDs and timestamps, in the parse worker (kills it) and on the UI thread |
| M22 | Medium | All | network | Fixed per-storm KMZ net IDs mean only the first storm's arrival contours ever load |
| M23 | Medium | All | network | Forecast never refreshes, a failed fetch is never retried, and the geocoder can get stuck on edited query text |
| M24 | Medium | All | network | Geocoder query is not percent-encoded |
| M25 | Medium | All | perf | Demo mode still downloads and decodes live volumes every 2 minutes |
| M26 | Medium | Win+ | threads | CPU-heavy work on tokio async workers; `Runtime` drop may wait on blocking jobs at exit |
| M27 | Medium | All | algo | Dealias unfold offset can random-walk in noisy regions with no clamp |
| L1–L22 | Low | mixed | mixed | See the Low section |

---

## Critical

### C1. A ~400 MB scan is serialized to JSON on the UI thread, and the scan cache grows without limit ✔

- **Where:** `app/src/main.rs:976` calls `app/src/cache.rs:48-58` (`save_scan`).
  - The unused alternatives are `app/src/cache.rs:121` (`save_scan_compressed`) and `cache.rs:137` (`load_scan_compressed`).
  - The load path is `cache.rs:72-99`.
  - Storage root is ply `ply-engine-1.1.1/src/storage.rs:52-60` and `:431-440`.
- **Severity:** Critical. **Category:** perf and stability. **Platform:** Win+. **Confidence:** confirmed. The size is measured from real cache files.
- **Evidence:**
  ```rust
  pub fn save_scan(&self, site: &str, scan: &ScanData) {
      let storage = self.storage.clone();
      let key = Self::scan_key(site);
      match serde_json::to_vec(scan) {          // runs on the caller (render) thread
          Ok(json) => { tokio::spawn(async move { let _ = storage.save_bytes(&key, &json).await; }); }
          ...
  ```
  - `main.rs:976` calls it inside the `WorkerMessage::NewScan` arm of the frame loop.
  - A grep shows no call sites for `save_scan_compressed` or `load_scan_compressed` in `main.rs`.
  - Real cache files on the developer machine: `scan_KRAX` 465 MB, `scan_KJGX` 403 MB, `scan_KVAX` 402 MB, `scan_KHGX` 398 MB. That's 28 scan files, 4.5 GB in total.
  - `ScanData` holds every sweep, radial, and gate as `Option<f32>` across about 7 moment arrays, and JSON writes below-threshold gates as `null`.
- **When it happens:**
  - On every launch. The data worker starts with `last_timestamp = None`, so the first poll always sends a `NewScan`.
  - On every site switch.
  - On every new volume, about every 4–10 minutes (polls are every 120 s, `data.rs:20`).
- **Failure scenarios:**
  - **UI freeze:** the frame stalls for hundreds of ms to several seconds (estimated). The time goes into the serialize, the growing `Vec` reallocations, and dropping the old `ScanData` at `main.rs:978` (about 100k `Vec` frees).
  - **Memory:** peak use is the in-RAM scan (about 640 MB as `Vec<Option<f32>>` at 80M gates) plus 400–512 MB of JSON.
  - **Windows:** if the message pump stops for more than 5 s, the window is ghosted as "Not Responding". Defender then scans each 400 MB write, and the system allocator (HeapAlloc) makes the reallocation churn worse.
  - **Slow startup:** `load_scan` runs `serde_json::from_slice` over the same 400 MB file inside a plain `tokio::spawn`, which blocks a runtime worker.
  - **Disk growth:** one `scan_{site}` file per site ever viewed, never evicted. Worst case at ~160 sites is tens of GB.
  - **Roaming profile:** on Windows this lives in `%APPDATA%\rustywx` (Roaming). Domain machines sync it at logon and logoff, and it can hit profile quotas.
- **Proposed fix:**
  1. Switch to the existing `save_scan_compressed` / `load_scan_compressed`: a flat binary layout plus RLE, tested at 90% or more savings. Or quantize gates to `u8`/`i16`.
  2. Do all serialization and deserialization inside `tokio::task::spawn_blocking`. Pass the `Box<ScanData>` by move or as `Arc<ScanData>`, so the UI thread never clones or serializes.
  3. Skip the save when the volume timestamp matches what is already cached.
  4. Add a cap: LRU over the N most recent sites, or a total byte budget.
  5. Store large blobs under `%LOCALAPPDATA%`, not Roaming. Ply picks the storage location, so this needs a separate path for scan blobs or a ply patch.

### C2. One failed fetch permanently disables alerts, borders, and NHC, and the toast never clears ✔

- **Where:**
  - Borders: `main.rs:1128-1136` (fire), `main.rs:1164-1175` (error and queue-full arms).
  - Alerts: `main.rs:1180-1188` (fire), `main.rs:1213-1224` (error and queue-full arms).
  - NHC: `main.rs:1229-1236` (fire), `main.rs:1290-1309` (error arm).
  - Root cause in ply: `ply-engine-1.1.1/src/net/mod.rs:86-91` (`fire_http` ignores a request whose ID already exists) and `net/mod.rs:41-63` (`clean()`).
- **Severity:** Critical. **Category:** stability. **Platform:** All; on Windows it's most likely for apps launched at logon before Wi-Fi is up, or behind captive portals. **Confidence:** confirmed by tracing the ply source.
- **Evidence (ply):**
  ```rust
  // Idempotent: don't re-fire if a request with this ID already exists
  if mgr.http_requests.contains_key(&key) { return; }
  ...
  match &entry.state {
      HttpRequestState::Pending(_) => true, // never evict
      _ => { entry.frames_not_accessed += 1; entry.frames_not_accessed <= self.max_frames_not_used } // 60
  }
  ```
  - Calling `net::request(id)` resets `frames_not_accessed`.
- **Trace (alerts; borders and NHC are the same shape):**
  1. The fetch errors. The `Err` arm sets `alerts_fetch_fired = false` and calls `show_toast(..Network)`.
  2. The next frame's gate (`!alerts_loaded || elapsed > 120s`) passes again, because `!alerts_loaded` skips the throttle.
  3. `fire_fetch()` does nothing, because an entry with that ID still exists.
  4. `poll` calls `net::request(id)`, which resets the eviction counter, and gets the same cached `Err`.
  5. That repeats every frame. The entry is touched every frame, so it is never evicted and never re-fetched.
- **Failure scenarios:**
  - Launch while offline or behind a TLS-intercepting proxy, and alerts, borders, and NHC stay empty until restart even after the network comes back.
  - `show_toast` replaces the toast every frame, so it never fades and dismissing it does nothing.
  - **Parse failures:** the cached `Done` body is re-copied, re-sent to the parse worker, and re-parsed every frame, which burns multi-MB of CPU.
  - **Queue full:** the `Full` arm (`main.rs:1164-1169`, `main.rs:1213-1218`) drops a body that was already downloaded, and it gets re-fetched once evicted.
  - **NHC:** `bundle.is_none()` skips `nhc_refresh_secs`, phase 1 errors, `start()`'s `net::get` does nothing, and `poll_phase1` finds the cached error again.
- **Proposed fix:**
  1. Use a unique net ID per attempt (`format!("alerts-{attempt}")`). Ply 1.1 has no remove API, so this is the reliable route; `net::request(id).cancel()` works too if it's available.
  2. Store `next_retry_at` per feed, with exponential backoff (for example 5 s, 10 s, 30 s, 60 s, capped at the refresh interval), and gate on it even when `!loaded`.
  3. On a parse or queue-full error, keep the body and retry the submit, not the download.
  4. Show the toast only when the feed first moves into failure, and clear it on success.

### C3. No HTTP timeouts, and stalled requests are never cleared ✔

- **Where:**
  - ply `net/mod.rs:107-111` creates a fresh `ureq::Agent::new_with_defaults()` per request on a new `std::thread`.
  - ureq `ureq-3.3.0/src/config.rs:894-907`: `Timeouts::default()` is all `None` except `await_100`.
  - ply `net/mod.rs:59`: `Pending` is never evicted.
  - App sites: `nhc.rs:936`, `nhc.rs:1182-1296` (`still_pending` at `nhc.rs:1277`), `alerts.rs:40`, `forecast.rs:84`, `forecast.rs:96`, the `*_fetch_fired` guards at `main.rs:1181`, `main.rs:1229`, `main.rs:3787`.
  - Radar data worker: `data.rs:233-253`, `data.rs:276` (`runtime.block_on(fetch_latest_scan(...))`), and nexrad-data `aws/client.rs:17-25` (`Client::builder()` with no timeout).
- **Severity:** Critical. **Category:** stability. **Platform:** Win+. Sleep/resume, Wi-Fi roaming, and VPN changes leave dead TCP connections, and Windows keepalive defaults to about 2 hours. **Confidence:** confirmed.
- **Failure scenarios:**
  - **Alerts (safety relevant):** the request stays `Pending`, `alerts_fetch_fired` stays `true`, and the 120 s refresh never runs again. Old tornado and severe-thunderstorm warnings stay on screen with no sign that they're stale.
  - **NHC:** one hung image or text request keeps `still_pending` non-empty forever.
  - **Forecast:** a hung request leaves `forecast_fetch_fired` true until the location changes.
  - **Radar:** `block_on` can't be interrupted, so `site_rx` is only read between fetches. A stalled `list_files` or `download_file` hangs the worker, so live radar and site switching both stop, and the status stays at "Checking…".
  - **Site switching in normal use:** a switch still waits for the old site's full download and decode.
- **Proposed fix:**
  1. Patch or fork ply so the Agent sets `timeout_global(30s)`, `timeout_connect(10s)`, and `timeout_recv_body(30s)`, ideally through `HttpConfig`.
  2. Until then, keep an app-side deadline: record an `Instant` when the request fires, and on expiry drop the request (unique ID per attempt, see C2), reset `fired`, and back off.
  3. Radar worker: wrap the fetch in `tokio::time::timeout` (about 30–60 s for listing, 120 s for download). Change `site_rx` to `tokio::sync::watch` or `mpsc` and `select!` it against the fetch, so a site change cancels the old fetch.
  4. Show a freshness label ("alerts as of HH:MM") and flag it when stale.

---

## High

### H1. TLS ignores the Windows certificate store, and the system proxy is ignored ✔

- **Where:** `ply-engine-1.1.1/Cargo.toml:164-167` sets `ureq = { version = "3", features = ["rustls"], default-features = false }`. The proxy comes only from `Proxy::try_from_env()` (`ureq config.rs:872`).
- **Severity:** High. **Category:** stability. **Platform:** Win. **Confidence:** confirmed from the dependency config.
- **Failure scenarios:**
  - Corporate or antivirus TLS inspection (Zscaler, Netskope, Bitdefender/Kaspersky HTTPS scanning) installs a CA into the Windows store. The bundled webpki-roots don't trust it, so every HTTPS request fails with an `UnknownIssuer`-style error.
  - There's no WinINet, registry, or PAC proxy support (ureq's `win-system-proxy` feature is off). Only `HTTP(S)_PROXY` and `ALL_PROXY` are honoured, and a GUI app launched from Explorer rarely has those set.
  - **Result:** NHC, alerts, borders, and forecast all fail, the user sees only a generic network toast, and C2 then makes the failure permanent.
- **Proposed fix:** patch or fork ply to turn on ureq's `platform-verifier` (or set `RootCerts::PlatformVerifier`) and `win-system-proxy`. At minimum, show the real error text in the toast or a diagnostics panel, and document the env-var proxy workaround.

### H2. One mouse-wheel notch on Windows jumps zoom to its limit and scrolls modals to the end ✔

- **Where:** `main.rs:3334-3338` (zoom), `main.rs:3950-3953` and `main.rs:3999-4002` (modal scroll).
  - The cause is `miniquad-ply-0.4.8/src/native/windows.rs:566-567`.
  - macroquad passes the value through unchanged (`macroquad-ply-0.4.14/src/lib.rs:564-568`).
- **Severity:** High. **Category:** stability/UX. **Platform:** Win. **Confidence:** confirmed.
- **Evidence:**
  ```rust
  // miniquad windows.rs
  WM_MOUSEWHEEL => { event_handler.mouse_wheel_event(0.0, (HIWORD(wparam as _) as i16) as f32); }   // ±120 per notch
  // main.rs:3337
  state.zoom = (state.zoom * (1.0 + scroll * 0.05)).clamp(0.05, 8.0);
  ```
  - macOS passes `dy` from `NSEvent`, about 1–10 per notch.
- **Failure scenario:**
  - Scroll up: `zoom * (1 + 120*0.05)` is ×7, clamped to 8.0.
  - Scroll down: `zoom * (1 - 6)` is negative, clamped to 0.05.
  - So one notch goes straight to max or min zoom.
  - Modals: `wheel*14*3` is about 5040 px per notch, which jumps to the end. Ply's own scroll containers use `SCROLL_SPEED = 20`, so 2400 px per notch.
  - Precision touchpads send smaller, variable deltas, so behaviour also differs between devices.
- **Proposed fix:** normalize to notches. On Windows, `notches = scroll / 120.0`; or use `scroll.signum()` everywhere with a fixed step. Zoom with `zoom *= 1.1f32.powf(notches)`, and scroll modals by `notches * line_height * 3`. Raise this upstream with ply for its scroll containers.

### H3. `fill_enclosed_holes` copies the whole grid once per pixel of hole radius ✔

- **Where:** `app/src/scope.rs:1202-1235`. The trigger is the range floor at `scope.rs:341-347`.
- **Severity:** High. **Category:** perf, with a correctness side effect. **Platform:** All; possibly worse on Windows. **Confidence:** confirmed from the algorithm; the timing is an estimate.
- **Evidence:**
  ```rust
  loop {
      let snapshot = values.to_vec();      // 2048² f32 = 16.8 MB allocation + copy, every pass
      let mut changed = false;
      for y in 0..s { for x in 0..s { ... 8-neighbour average from snapshot ... } }
      if !changed { break; }
  }
  ```
  - Each pass fills only one pixel ring of each hole, so the number of passes equals the hole radius in pixels.
- **Failure scenario:**
  - The range floor nulls gates under 20 dBZ inside 20 km, and under 10 dBZ inside 80 km.
  - In widespread light rain or snow (10–20 dBZ, very common), that leaves an empty disk around the radar enclosed by echo.
  - At 20 km that's about 89 px, so about 89 passes. At 80 km in light snow it's about 356 px, so about 356 passes.
  - Each pass is a 16.8 MB allocation plus memcpy plus a 4.2M-cell scan, which adds roughly 0.5–4 s per re-raster (estimated).
  - **Windows (plausible):** each 16 MB `to_vec` goes through `VirtualAlloc`/`VirtualFree`, so the pages are faulted in fresh every pass.
  - **Correctness:** the disk the floor deliberately cleared is repainted with interpolated values of 5 dBZ or more, which defeats the floor.
- **Proposed fix:**
  1. At minimum, allocate the snapshot once outside the loop and `copy_from_slice` each pass.
  2. Better, a single BFS/frontier fill over hole cells only, which is O(hole cells).
  3. Skip hole components above a size cap, and never fill pixels the range floor nulled. That needs a "floor-nulled" mask, separate from "no data".

### H4. Brute-force neighbourhood filters over the full 2048² grid on every reflectivity raster

- **Where:** `scope.rs:1118-1162` (dilate/erode inside `fill_enclosed_holes`, r=8), `scope.rs:970-1055` (`morphological_close`, r=4, 3 passes), `scope.rs:1246-1276` (`smooth_values`, r=3).
- **Severity:** High. **Category:** perf. **Platform:** All. **Confidence:** confirmed from the algorithm.
- **Evidence:**
  - Dilate and erode scan a 289-cell window per pixel. Clear-air pixels never exit early, and the ~0.9M corner pixels outside 230 km are always NaN, so they always do the full scan.
  - Close scans 81 cells × 3 passes, and smooth scans 49 cells.
  - Total: about 1.5–2×10⁹ bounds-checked inner iterations per raster, single-threaded.
- **Failure scenarios:**
  - Multi-second latency on every product, tilt, or QC toggle, on top of H3.
  - `state.radar_texture = None` at submit (`main.rs:1555-1557`) leaves the radar blank the whole time (see M2).
  - The benchmarks (`app/benches/rasterize.rs`) only go up to 1024 px, a quarter of production's pixel count, on a synthetic sweep. They never exercise 2048 or the enclosed-hole case, so this cost never showed up.
- **Proposed fix:**
  1. Use a summed-area table of the finite mask. "Any finite in the Chebyshev window" becomes `count > 0`, and "all sealed" becomes `count == area`, both O(1) per pixel.
  2. Do the NaN-aware box blur with a sum SAT plus a count SAT.
  3. Skip pixels outside the 230 km disk.
  4. Parallelize by rows (`std::thread::scope`, or add `rayon`).
  5. Add a 2048 benchmark with an enclosed hole.

### H5. Border, alert, and NHC geometry is re-projected every frame, twice per vertex, before culling ✔

- **Where:** `scope.rs:1612-1644` (`draw_borders`), `scope.rs:1684-1761` (alerts, passes 1 and 2), `scope.rs:1861-2103` (NHC cones and contours). Alerts aren't filtered to the scope: `state.alerts = alerts_list` at `main.rs:1076`.
- **Severity:** High. **Category:** perf. **Platform:** Win+ (weak Windows laptops and iGPUs). **Confidence:** the code path is confirmed; the per-frame cost is an estimate.
- **Evidence:**
  ```rust
  for ring in rings {
      for pair in ring.windows(2) {
          let a_km = geo::point_to_km_offset(site.lat, site.lon, pair[0]);
          let b_km = geo::point_to_km_offset(site.lat, site.lon, pair[1]);
          // on-screen cull happens after projection
  ```
  - The local cache `state_borders_v8` holds 1,940 rings and 72,229 vertices: the global NE 1:50m coastline, world land borders, and US states.
  - That's about 140k `range_bearing` calls per frame, each about 12–16 f64 trig/sqrt/atan2 operations, so about 2.4M transcendental ops per frame.
- **Failure scenarios:**
  - An estimated 10–25 ms per frame on borders alone, which is most of a 16.6 ms budget. Borders are on by default.
  - At zoom 0.05 almost nothing is culled, so tens of thousands of `draw_line` quads also go through macroquad's CPU batcher.
  - Alerts allocate a fresh `Vec` per ring per frame and are projected in both passes.
  - The km offsets depend only on the site, not on pan or zoom, so all of this work is redundant.
- **Proposed fix:**
  1. When the site changes or borders, alerts, or NHC data load, project each ring once into `Vec<Vec<Vec2>>` km offsets, and keep a km bounding box per ring.
  2. Drop rings nowhere near the site at load time; `borders::ring_affects_scope` already exists.
  3. Per frame, cull by bounding box, then do one multiply-add per vertex.
  4. Filter alerts to the scope area when they arrive.
  5. Consider an equirectangular local approximation for display.

### H6. `borders::poll_raw` stops at the first pending request, so the other responses are cleared before they're read ✔

- **Where:** `app/src/borders.rs:59-61` (`poll_and_merge`), `borders.rs:79-81` (`poll_raw`).
- **Severity:** High. **Category:** stability. **Platform:** All; Windows makes it more likely, with antivirus TLS scanning and cold connections. **Confidence:** the code is confirmed; the trigger is plausible.
- **Evidence:**
  ```rust
  let states  = net::request(NET_ID_STATES)?.response()?;
  let coast   = net::request(NET_ID_COAST)?.response()?;
  let country = net::request(NET_ID_COUNTRY)?.response()?;
  ```
- **Failure scenario:**
  1. States is still pending, so the function returns at the first `?` and never touches coast or country.
  2. Their `Done` entries go unaccessed and are cleared after 60 frames (about 1 s at 60 fps; see C2 for the ply mechanism).
  3. After that, `net::request(NET_ID_COAST)?` returns `None` forever.
  4. `borders_fetch_fired` stays `true` and nothing re-fires, so state lines and coastlines never appear.
  - It triggers whenever the three downloads finish more than about 1 s apart. That's likely with a TLS handshake per request on a cold connection, antivirus scanning, or a slow GitHub raw response.
- **Proposed fix:** call `net::request(id)` on all three IDs on every poll before inspecting any of them, then return `None` until all three are done. Unique IDs per attempt (C2) also make the fallback re-fire safe.

### H7. Windows is never built or tested in CI ✔

- **Where:** `.github/workflows/ci.yml` runs every job on `ubuntu-latest` or `ubuntu-24.04` (lines 18–178). `.github/workflows/release-please.yml:88` holds the only `windows-latest` / `x86_64-pc-windows-msvc` matrix entry.
- **Severity:** High. **Category:** build/release. **Platform:** Win. **Confidence:** confirmed.
- **Failure scenarios:**
  - Any Windows-only compile or link break (miniquad Win32 code, `webbrowser`, a native build dependency through rustls/ureq, a `windows-sys` version clash) shows up first in the release workflow, after release-please has already cut the tag and the GitHub release.
  - The matrix has `fail-fast: false`, so the result is a half-populated release, and `publish-crate` is skipped.
  - `cargo test` never runs on Windows, so the `PathBuf`/`APPDATA` storage handling and any Windows-specific behaviour is untested.
  - The release build step is only `cargo build --release --locked -p rustywx --target ...`, with no launch check. The Linux AppImage step has a `timeout` launch check; Windows has none.
- **Proposed fix:**
  1. Add a `windows-latest` job (and ideally `macos-latest`) to `ci.yml` running `cargo check`, `cargo test --all-features`, and `cargo build --release --locked`.
  2. In the release job, launch the `.exe` for a few seconds, mirroring the AppImage check. A GUI app on a headless runner may need `--version` or a smoke flag that exits after init.

### H8. Console-subsystem exe, no panic hook, no log file ✔

- **Where:** `app/src/main.rs:1-12` has no `#![windows_subsystem]`, and `main.rs:555-556` has no `std::panic::set_hook`.
  - The UI-thread `eprintln!` calls this matters for include `main.rs:1045`, `1307`, `1359`, and `1445`.
  - The Windows package step is `release-please.yml:260-272`.
- **Severity:** High. **Category:** stability. **Platform:** Win. **Confidence:** confirmed. A grep for `windows_subsystem` and `set_hook` finds nothing.
- **Failure scenarios today (console subsystem):**
  - A black console window opens behind the GUI on every double-click or Start-menu launch.
  - Closing the console kills the app, possibly during a non-atomic 400 MB scan write (C1, M3). The next launch then reads and parses the truncated file in full before discarding it.
  - **QuickEdit:** if the user clicks into the console, selection mode blocks every stderr write. The render thread freezes at the next `eprintln!` (a raster upload, toast, or parse result) until the selection is cleared.
  - Windows console writes are synchronous and slow, which amplifies any `eprintln!` in a hot loop (M2, C2).
- **Failure scenarios after adding `windows_subsystem = "windows"` alone:**
  - All 35 `eprintln!` diagnostics go nowhere.
  - Any startup panic (`main.rs:569` tokio, `main.rs:629` storage) makes the window vanish with no trace.
  - `RUST_BACKTRACE` is set only in CI.
- **Proposed fix:**
  1. Add `#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]`.
  2. Install a panic hook as the first statement of `main()`, before the storage `expect`. It should append the message, location, and backtrace to `%LOCALAPPDATA%\rustywx\crash.log`, and optionally show a `MessageBoxW`.
  3. Send diagnostics to a rolling log file through a background logger (for example `tracing` plus a non-blocking file appender), so no UI-thread code writes to stderr directly.

### H9. Panics on the UI thread ✔

- **(a) NHC text preview, byte slice:** `main.rs:2707-2712`. **Severity:** High. **Platform:** Win+. **Confidence:** the mechanism is confirmed; the trigger depends on content.
  ```rust
  let truncated = if product.content.len() > 300 { &product.content[..300] } else { &product.content };
  ```
  - Content comes from `extract_pre_content`, or falls back to `strip_html` of the whole page (`nhc.rs:349-350`). That fallback keeps `<script>`/`<style>` text and page chrome.
  - Any non-ASCII character that crosses byte 300 panics while the Tropical panel draws: an em dash, a smart quote, `°`, or accented Spanish advisory text.
  - Under H8 the message is lost.
  - **Fix:** `content.char_indices().nth(300).map_or(content.as_str(), |(i, _)| &content[..i])`, or compute the preview when parsing.
- **(b) Alert event name, byte slice:** `scope.rs:1766`, `&alert.event[..30]`. **Severity:** Low–Medium (latent). The event comes from api.weather.gov and is filtered to names containing "warning" or "watch", so it's ASCII in practice. **Fix:** truncate on a char boundary.
- **(c) Dropped file aborts the process:** this is in miniquad, not the app. `miniquad-ply-0.4.8/src/native/windows.rs:973` calls `DragAcceptFiles(hwnd, TRUE)` unconditionally, and `windows.rs:758` does `d.dropped_files.bytes.push(std::fs::read(&path).unwrap());` inside the window procedure.
  - **Severity:** Medium. **Platform:** Win. **Confidence:** confirmed.
  - Dropping a folder or a locked file onto the window panics inside an `extern "system"` callback, which aborts the process.
  - Dropping a huge file reads it fully on the UI thread.
  - The app never uses dropped files.
  - **Fix:** call `DragAcceptFiles(hwnd, FALSE)` after window creation (miniquad exposes the HWND on Windows), or patch miniquad to handle the error.

---

## Medium

### M1. A stale cached scan can overwrite a newer live scan ✔

- **Where:** `main.rs:842-857` (cache consumer) against `main.rs:954-983` (`NewScan`). Also `cache.rs:72-99` and `data.rs:276-283`.
- **Category:** stability/correctness. **Platform:** Win+ (slower disk I/O and antivirus make the cache load lose the race more often). **Confidence:** confirmed; it depends on timing.
- **Evidence:**
  ```rust
  if let Some(ref mut rx) = state.pending_load && let Ok(cached) = rx.try_recv() {
      state.pending_load = None;
      if let Some(scan) = cached { state.scan = Some(scan); state.needs_reraster = true; update_scan_status(&mut state, " [cached]"); }
  ```
  - There's no `state.scan.is_none()` or timestamp check. `NewScan` doesn't clear `pending_load`.
- **Failure scenario:**
  1. At startup, or after `select_site` (`main.rs:4130`), the 400 MB cache load is still parsing when the worker's first fetch arrives and sets `state.scan`.
  2. The cache result then lands and replaces the newer data, and the status shows `[cached]`.
  3. The worker already holds the newer `last_timestamp`, so the next poll returns "up to date" and doesn't resend.
  4. The user sees old data until the next volume, about 5–10 minutes later.
- **Proposed fix:** set `state.pending_load = None` when a `NewScan` for the current site is applied, and only apply a cached scan if `state.scan.is_none()` or the cached timestamp is newer.

### M2. Raster failure retries every frame, the texture is blanked on every submit, and results are dropped while dirty ✔

- **Where:** `main.rs:1451-1460` (error arm), `main.rs:1466` and `main.rs:1553-1557` (submit, `radar_texture = None`), `background_work.rs:180-188` (`raster_result_is_current`), `background_work.rs:261-272`.
- **Category:** stability and perf. **Platform:** All; Windows console writes make it worse (H8). **Confidence:** the retry loop is confirmed; the flicker and starvation depend on interaction timing.
- **Evidence:**
  ```rust
  Err(error) => {
      state.needs_reraster = true;
      eprintln!("Warning: radar raster failed generation={}: {error}; retrying", result.generation);
  }
  ```
- **Failure scenarios:**
  - **Hot retry:** a panic in `scope::rasterize_with_report` becomes a `JoinError`, which becomes `Err("raster worker join")`, and the next frame resubmits. A deterministic panic, such as a QC or dealias bug on one unusual sweep, then:
    - pegs a core with multi-second rasters forever;
    - clones the sweep (about 12 MB) each time;
    - writes a backtrace plus a warning to the slow console each time.
    - The radar stays blank.
  - **Blank on submit:** setting `radar_texture = None` blanks the scope for the whole raster time, which is seconds given H3 and H4.
  - **Dropped results while dirty:** any `needs_reraster = true` while a job runs makes `raster_result_is_current(dirty=true)` return false, and the finished result is thrown away. Sources include the slider and toggle handlers at `main.rs:3606-3700` and `select_site`. A job can't be cancelled, so rapid setting changes keep the radar blank and waste CPU.
- **Proposed fix:**
  1. Count retries per generation: retry once, then mark the sweep failed and show a toast.
  2. Keep the previous texture until a newer one is uploaded.
  3. Accept a result whose site, product, and generation are still current even when `dirty` is set, and coalesce setting changes (debounce about 100–200 ms).

### M3. Settings and cache writes race, aren't atomic, and fail silently

- **Where:**
  - `cache.rs:224-237` (`save_settings`), called from 30+ sites in `main.rs` (for example `1379`, `3477-3761`).
  - Other non-atomic writes: `cache.rs:126`, `cache.rs:180`, `borders.rs:135`, `data.rs:140`.
  - The ply implementation is `storage.rs:59` (`std::fs::write`).
  - The runtime is multi-threaded (`Runtime::new()`, `main.rs:569`).
- **Category:** stability. **Platform:** Win+. **Confidence:** the race and the non-atomic write are confirmed; the Windows lock behaviour is plausible.
- **Failure scenarios:**
  - **Race:** each settings change spawns an independent task that truncates and rewrites `settings.json`, with no ordering. Rapid stepper clicks plus a location poll plus a toggle can run at once. A stale snapshot can finish last and win, and interleaved truncate-and-write can tear the file.
  - **Crash or power loss mid-write:** this leaves a zero-length or partial file. `load_settings` logs "corrupt" and deletes it, so all settings are silently lost.
  - **Windows locks:** if antivirus, OneDrive or another sync client, an indexer, or a second app instance locks the file exclusively, `fs::write` fails and the error is swallowed with `let _ =`, so the setting is silently not saved.
- **Proposed fix:**
  1. Run a single settings-writer task fed by a `tokio::sync::watch` of the latest snapshot. That coalesces bursts and serializes writes.
  2. Write atomically: `settings.json.tmp`, then `fsync`, then `std::fs::rename` (which replaces the target on Windows via `MoveFileExW(MOVEFILE_REPLACE_EXISTING)`). Retry a few times on `PermissionDenied` or sharing violations.
  3. Keep `settings.json.bak` from the last successful write, and restore from it instead of deleting on parse failure.
  4. Apply the same atomic write to the scan, border, and demo blobs.

### M4. Startup `expect`s abort with no message

- **Where:** `main.rs:629` (`Cache::new().await.expect("Ply storage initialisation")`), `main.rs:569` (`.expect("tokio runtime")`), and miniquad's WGL setup (`miniquad-ply-0.4.8/src/native/windows/wgl.rs:279`, `:385`, `:452`).
- **Category:** stability. **Platform:** Win+. **Confidence:** confirmed.
- **Failure scenarios:**
  - **Storage:** `Storage::new` fails when both `%APPDATA%` and `%USERPROFILE%` are unset (service or restricted environments), or when `create_dir_all` fails. That happens with a redirected or offline roaming profile, a read-only or ACL-denied profile, or Controlled Folder Access blocking the write.
  - **WGL:** it requires `WGL_ARB_pixel_format` and GL 2.1 or newer, so RDP sessions and VMs on the Microsoft Basic Display Adapter panic before the window appears.
  - With H8 fixed but no hook, both are silent.
- **Proposed fix:**
  - Make `Cache.storage` an `Option`, or fall back to `std::env::temp_dir()`, run with no persistence, and show a warning toast.
  - For GL failures, use the panic hook from H8 to show a `MessageBoxW` that explains what's needed (an OpenGL 3.x driver, and RDP limitations).

### M5. DPI: most 125–150% Windows laptops get the mobile layout, and only system DPI is supported

- **Where:** `main.rs:505-507` (`window_conf`: 900×960, `high_dpi: true`), `main.rs:1653` (`is_mobile = screen_width() < 900.0`). Also `miniquad-ply-0.4.8/src/native/windows.rs:941-947` (window created at physical size) and `windows.rs:1159` (`SetProcessDPIAware()`). The repo has no `build.rs` or `.manifest`.
- **Category:** stability/UX. **Platform:** Win. **Confidence:** confirmed from source; the visual symptoms are plausible.
- **Failure scenarios:**
  - **Mobile layout:** miniquad creates the window at 900×960 physical pixels, and `screen_width()` divides by `dpi_scale`. That gives a logical width of 720 at 125% and 600 at 150%, so most Windows laptops start in the mobile layout, with 44 px buttons and full-width panels.
  - **Off-screen status bar:** on a 1366×768 display at 100%, a 960 px window puts the bottom status bar off-screen.
  - **Blur across monitors:** `SetProcessDPIAware()` is the legacy "system DPI aware" mode, not Per-Monitor V2. `init_dpi` runs once, and `WM_DPICHANGED` isn't handled. Moving the window between a 200% and a 100% monitor bitmap-scales it: the scope blurs and the hit-testing may not line up.
  - **No manifest:** nothing declares `longPathAware` or `activeCodePage=UTF-8`.
- **Proposed fix:**
  1. Embed a Windows manifest through a `build.rs` (`embed-resource` or `winres`, Windows-only) declaring `<dpiAwareness>PerMonitorV2</dpiAwareness>`, `longPathAware`, and `activeCodePage UTF-8`. A manifest takes effect before miniquad's call, which then does nothing.
  2. Base `is_mobile` on logical size with a lower breakpoint, or on physical size, so a default desktop window is never "mobile".
  3. On Windows, scale the requested window size by the monitor DPI and clamp it to the work area.

### M6. Background "gradient" draws 24 near-full-screen blended circles per frame ✔

- **Where:** `main.rs:364-382` (`draw_observatory_background`).
- **Category:** perf. **Platform:** Win+ (integrated GPUs at high DPI with `sample_count: 4`). **Confidence:** the wasted draws are confirmed; the GPU cost is plausible.
- **Evidence:** `r = max_r * (1.0 + t * 0.04)` where `max_r` is the half-diagonal, so every circle covers essentially the whole screen. Only the last one drawn (i=0) is visible, apart from corner slivers.
- **Failure scenario:** 24 blended full-screen fills per frame on top of `clear_background`. At 4K with 4× MSAA that's about 800M samples per frame, a large share of an Intel UHD-class fill budget. Each circle is also tessellated by the CPU batcher.
- **Proposed fix:** a single `clear_background` in the final colour, or one textured quad or small shader if a real radial gradient is wanted.

### M7. No throttle while minimized or with vsync off, and animation depends on frame rate

- **Where:** main loop `main.rs:812-3228`, `main.rs:837` (`state.sweep_angle = (state.sweep_angle + 0.6) % 360.0`), and `miniquad-ply-0.4.8/src/native/windows.rs:1255-1290` (update, draw, and `SwapBuffers` every iteration; `iconified` is only used to fire events).
- **Category:** perf. **Platform:** Win. **Confidence:** the missing throttle is confirmed; the minimized busy loop depends on the driver.
- **Failure scenarios:**
  - While minimized, WGL `SwapBuffers` with swap interval 1 often returns immediately, depending on the driver. The full frame then runs unthrottled at 100% of a core: layout, about 284 `format!` calls, the 24 circles from M6, and border projection from H5.
  - The same happens when vsync is forced off in the driver control panel.
  - Sweep animation speed scales with fps.
- **Proposed fix:**
  - When `screen_width() <= 1.0` (miniquad clamps a minimized framebuffer to 1×1), or when the window isn't focused and nothing is animating, sleep about 50–100 ms per frame.
  - Add a soft frame cap, or switch to `blocking_event_loop` with `schedule_update` for idle periods.
  - Scale animations by `get_frame_time()`.

### M8. Typed characters build up and are inserted into fields in reverse

- **Where:** `main.rs:3715`, `main.rs:3817`, and `app/src/widgets/dropdown.rs:194`. macroquad's `chars_pressed_queue` is only cleared by `clear_input_queue()`, and `get_char_pressed()` uses `pop()`, which is LIFO.
- **Category:** stability/correctness. **Platform:** All; Windows IME commits make it worse. **Confidence:** confirmed.
- **Failure scenarios:**
  - Shortcut letters pressed earlier ("r v t b") flow into the site dropdown filter the first time it opens, so it shows no matches. They also flow into the location and forecast fields.
  - Any burst of characters in one frame is inserted reversed. That includes a Windows IME commit, which sends several `WM_CHAR` messages at once, and keys typed during a C1 stall.
  - Ply drains the queue only while a ply text input is focused.
- **Proposed fix:** call `clear_input_queue()` when a field gains focus or a dropdown opens, and once per frame when no text field is focused. When draining, collect into a `Vec` and reverse it before inserting.

### M9. Shortcuts fire while typing in the Forecast search box, and Windows fullscreen restore is broken

- **Where:** `main.rs:3507-3512` (F, fullscreen), `main.rs:3558` (N, NHC), `main.rs:3873` (`?`), `main.rs:3571` (Escape closing Settings). Also `miniquad-ply-0.4.8/src/native/windows.rs:268-309` (`set_fullscreen`).
- **Category:** stability/UX. **Platform:** Win+. **Confidence:** confirmed.
- **Evidence:** `(!dropdown_open && !state.location_input_focused && is_key_pressed(KeyCode::F))` checks `location_input_focused` but not `fc_search_focused`.
- **Failure scenarios:**
  - Typing "Fort Worth" toggles fullscreen. "N" toggles the NHC layer and writes settings (M3).
  - Escape closing Settings leaves `location_input_focused == true`, so F and N stay disabled until the field is used again.
  - On Windows, miniquad's `set_fullscreen` always uses the primary monitor at (0,0). Leaving fullscreen restores a window at (0,0) sized to the whole monitor plus its frame, which overflows the screen, and the previous position and size are lost.
- **Proposed fix:**
  - Use one `text_input_focused` flag (any field) that suppresses all single-key shortcuts, and clear it whenever its panel or field closes.
  - For fullscreen, save and restore the window rect yourself, or patch miniquad to use `MonitorFromWindow` and restore the previous placement.

### M10. Re-raster preparation runs on the UI thread: deep sweep clones and melting-layer detection

- **Where:**
  - `main.rs:1466-1558`: submit block, including `main.rs:1478-1483` (sweep clone) and `nearest_aux` clones.
  - `main.rs:1515-1520`: calls `melting_layer::detect` (`melting_layer.rs:~105-160`).
  - Clones on the worker: `scope.rs:191-197` (`clean_sweep` always clones), `scope.rs:268-271` (dealias), `scope.rs:516-519` (gap-fill), and `dealias.rs:129-190`.
- **Category:** perf. **Platform:** All. **Confidence:** the clones and the UI-thread location are confirmed; the size is an estimate.
- **Evidence:** `sweeps[..].clone()` of the full `SweepData` (`Vec<Vec<Option<f32>>>`, 8 bytes per gate, about 7–15 MB, one allocation per radial), plus up to 3 aux sweeps (CC, ZDR, ΦDP) when fuzzy non-met is on.
  ```rust
  state.melting_layer_hint = if state.settings.melting_layer_hint_enabled {
      state.scan.as_ref().and_then(|scan| melting_layer::detect(&scan.correlation_coefficient)) } ...
  ```
- **Failure scenarios:**
  - Every tilt, product, or setting change, and every new scan, does 10–40 MB of allocation and copying on the UI thread.
  - `melting_layer::detect` scans every CC sweep at 4–10° elevation (2–4M `Option<f32>` reads). It allocates a `Vec` per candidate gate in `shoulder_mean`, and it depends only on the scan, not on product or tilt.
  - On the worker, a velocity sweep is copied about 5 times per raster: `gates` cloned at `scope.rs:265`, a new `Vec` per radial in `dealias_radial`, `prev = Some(out[i].clone())`, and `new_reference.clone()` twice.
- **Proposed fix:**
  1. Hold `ScanData`, or each sweep, in an `Arc`, and send the `Arc` plus indices to the worker.
  2. Compute the melting-layer hint once per scan, in the parse/background step, and cache it on `ScanData`. Make `shoulder_mean` allocation-free with a running sum and count.
  3. In `clean_sweep`, clone only when a pass will mutate (`Cow`).
  4. In dealias, keep `prev` as an index into `out`, use `split_at_mut`, and pass slices.

### M11. KDP derivation allocates per gate, on every volume load

- **Where:** `app/src/kdp.rs:112-125` (`windowed_filter`, `median`), called unconditionally from `data.rs:101` (`scan_from_volume`).
- **Category:** perf. **Platform:** Win+ (the Windows system allocator, debug builds). **Confidence:** the allocation pattern is confirmed; the timing is an estimate.
- **Evidence:** for every valid gate, in two passes (median and mean), `gates[start..end].iter().filter_map(..).collect()` allocates a fresh `Vec<f32>`, and the median pass also sorts it.
- **Failure scenario:** for about 10 sweeps at 720×1200 gates, that's roughly 17M small heap allocations on one thread, on every volume load, even if the user never views KDP. It sits on the first-load path that commit `081df73` tried to make non-blocking, and it adds latency to every new scan.
- **Proposed fix:**
  - Use a fixed stack window (`[f32; 2*HALF+1]` plus a count), with an insertion-sort or sorting-network median and a running sum for the mean.
  - Process radials in parallel.
  - Compute KDP lazily the first time the KDP product is viewed.

### M12. NHC phase-2 work runs on the UI thread

- **Where:** `nhc.rs:1182-1285` (`poll_phase2`), called from `main.rs:1240`. `nhc.rs:1021` (`accept_metadata`, which runs `construct_image_products` and fires requests).
- **Category:** perf. **Platform:** All. **Confidence:** confirmed.
- **Evidence:** per frame in phase 2, on the render thread:
  - `extract_kml_from_kmz` (inflate, then `read_to_string`);
  - `parse_wind_prob_kml` and `parse_arrival_kml`, which do an O(n) substring scan and allocate a `Vec<&str>` per coordinate tuple before building `Vec<(f64,f64)>`;
  - `parse_gis_layer` (a full `serde_json` parse) when layers arrive late, and `parse_gis_storms`;
  - `r.bytes().to_vec()` for every image.
  - Commit `081df73` moved only CurrentStorms and the phase-1 GIS parse off-thread.
- **Failure scenario:** the 34/50/64 kt wind-probability KMZs have large polygon sets, so the frame hitches for tens of ms or more every refresh cycle (5 min by default).
- **Proposed fix:** send the raw phase-2 bodies (KMZ, GIS JSON, image bytes) to `background_work`, as phase 1 does. The UI thread should only receive finished `Vec<Ring>` and decoded images.

### M13. NHC refresh floods requests with no conditional GET, and re-uploads unchanged textures

- **Where:** `nhc.rs:1058-1133` (request fan-out), `main.rs:1279-1285` (texture reset and re-upload).
- **Category:** perf. **Platform:** All. **Confidence:** confirmed. The `nhc_refresh_secs` lower bound was not verified.
- **Evidence:**
  - Per storm: 3 text, 6 image, up to 2 arrival KMZ, and 3 shared wind-probability KMZ requests, on top of phase 1's 5. That's about 9 or more per storm, each on its own `std::thread` with its own TLS handshake (ply net).
  - Several are expected 404s (Key Messages, Peak Surge, Rainfall).
  - On completion, all NHC textures are cleared and every image is re-decoded and re-uploaded, even when the advisory hasn't changed.
- **Failure scenarios:**
  - 5 active storms means about 45 concurrent threads and TLS connections to the same host every refresh, plus redundant GPU uploads.
  - `nhc_refresh_secs` is user-settable. If there's no lower clamp, a value near 0 hammers NOAA.
- **Proposed fix:**
  1. Skip phase 2 when `advisory_num` and `graphics_issuance` / `last_update` haven't changed, and reuse the previous bundle and textures.
  2. Send `If-Modified-Since` / `If-None-Match`, and handle a 304 as unchanged (ureq 3 returns it as an error by default).
  3. Cap concurrency (about 4 in flight).
  4. Clamp `nhc_refresh_secs` to at least 60 s.

### M14. One bad alert feature drops the whole list, and the national feed is re-downloaded in full

- **Where:** `alerts.rs:117-137` and `alerts.rs:171-195` (geometry parsing), `alerts.rs:23` and `alerts.rs:40` (URL and fetch), `main.rs:1181` (gate). The ureq body cap is `ureq-3.3.0/src/body/mod.rs:30` (10 MiB for `read_to_vec`).
- **Category:** stability and perf. **Platform:** All. **Confidence:** the drop-all behaviour is confirmed; the cap risk is plausible.
- **Evidence:**
  ```rust
  other => return Err(anyhow!("unsupported alert geometry type: {other}")),
  ...
  rings.extend(parse_polygon_rings(coordinates)?);
  ```
- **Failure scenarios:**
  - **Drop-all:** one feature with a `GeometryCollection`, a `Point`, or a ring point missing a coordinate makes `parse_alerts` return `Err`. Every warning is then discarded for that poll, and at startup it triggers the C2 loop.
  - **Re-download and cap:** `alerts/active?status=actual` covers the whole US, with no `ETag` or `If-Modified-Since`, every 120 s. That's a repeated multi-MB download, a `to_owned()` copy, and a full `serde_json::Value` parse even when nothing changed. During a major outbreak the payload can approach the 10 MiB `read_to_vec` limit, and beyond it the fetch errors.
- **Proposed fix:**
  - Skip and log per feature (`continue`) instead of `?` or `return Err`.
  - Use `If-None-Match`.
  - Narrow the query with `area=`, `point=`, or `region=` where possible, or confirm the size margin.

### M15. A new 16 MB texture is allocated and uploaded per raster, and macroquad's `unbatched` list grows

- **Where:** `main.rs:1425-1436` (`Texture2D::from_rgba8(RASTER_SIZE_PX, ...)`) and `macroquad-ply-0.4.14/src/texture.rs:921` (`from_rgba8` pushes into `texture_batcher.unbatched`).
- **Category:** perf, plus a small leak. **Platform:** Win+ (Intel iGPU drivers). **Confidence:** the code is confirmed; the hitch size is plausible.
- **Failure scenarios:**
  - `from_rgba8` does a synchronous `glTexImage2D` allocation and a 16 MB upload on the render thread. Intel iGPU drivers commonly stall for 10–30 ms here, a visible hitch, and the code already logs it against `FRAME_BUDGET`.
  - The old texture is freed at end of frame by `garbage_collect`, so there's allocation churn on every raster.
  - The `unbatched` list is only drained by `build_textures_atlas`, which the app never calls, so it grows by a few bytes per raster with no limit.
- **Proposed fix:** keep one persistent radar texture. The size never changes, so use `update_from_bytes` / `glTexSubImage2D`, optionally split across frames with `texture_update_part`. That also fixes the `unbatched` growth.

### M16. Site switches start uncancellable ~400 MB cache loads

- **Where:** `main.rs:4126-4130` (`select_site` calls `load_scan`), `main.rs:674`, `cache.rs:72-99`.
- **Category:** perf and stability (memory). **Platform:** Win+. **Confidence:** the code is confirmed; the out-of-memory size is plausible.
- **Failure scenario:**
  - Each `select_site` spawns a full 400 MB `fs::read` plus `serde_json::from_slice` inside a plain `tokio::spawn`, which blocks a runtime worker.
  - Dropping the oneshot receiver doesn't cancel the task.
  - Pressing Right-arrow 5 times starts 5 concurrent loads, each peaking at about 1 GB. That's a real risk of paging or running out of memory on an 8 GB Windows laptop.
- **Proposed fix:** fixing C1 (compact format) mostly removes this. Also use `spawn_blocking`, keep an `AbortHandle` or generation check so superseded loads stop early, and debounce site switches (about 250 ms).

### M17. Panics in worker threads die silently

- **Where:** `data.rs:233` (`std::thread::spawn` with no `catch_unwind`, covering `model.rs from_sweeps`, `kdp::derive_kdp_sweeps`, and decode), `main.rs:4124` (`let _ = state.site_tx.send(...)`), and the `background_work.rs` parse worker (no `catch_unwind`; see M21).
- **Category:** stability. **Platform:** Win+ (no console). **Confidence:** the structure is confirmed; no specific panicking input was found.
- **Failure scenarios:**
  - A malformed volume that panics inside `fetch_latest_scan` kills the data thread with no `WorkerMessage`. The UI gets no error, and live updates stop for the session.
  - A failed runtime build in the thread sends `Error` and returns, after which every `site_tx.send` fails silently.
  - A panic in the shared parse worker stops borders, alerts, and NHC parsing at once.
- **Proposed fix:**
  - Wrap each job (data loop iteration, parse job) in `std::panic::catch_unwind(AssertUnwindSafe(..))` and report `WorkerMessage::Error`.
  - Or run decode in `spawn_blocking` and treat `JoinError` as an error with backoff.
  - Detect a dead worker (`site_tx.send` returns `Err`) and either restart it or show a persistent banner.

### M18. Palette shader compile failure panics, with no fallback

- **Where:** `scope.rs:59-71` (`.expect("reflectivity palette shader failed to compile")`), compiled lazily from the render loop at `main.rs:1566`. The context is `miniquad-ply-0.4.8/src/native/windows/wgl.rs:411-420`.
- **Category:** stability. **Platform:** Win. **Confidence:** plausible (depends on the driver).
- **Evidence:** on Windows, miniquad creates a GL 3.1 core forward-compatible WGL context and passes `#version 100` GLSL ES source straight to the driver. Whether it's accepted depends on `ARB_ES2_compatibility`. macroquad's own default shaders share that dependency, so the extra risk is small, but there's no fallback.
- **Proposed fix:** on `Err`, log it and switch to CPU-coloured reflectivity (`encode_value = false` in `rasterize_cleaned`; `dbz_color` already exists).

### M19. Windows release packaging gaps

- **Where:** `Cargo.toml` `[profile.release]`; there's no `.cargo/config.toml`, no `build.rs`, and no `[build-dependencies]`. The runtime icon only is set at `main.rs:441-452`. Packaging is in `release-please.yml`'s "Package (windows)" step (`Compress-Archive` then `softprops/action-gh-release`). Also `README.md:142-143`.
- **Category:** build/release. **Platform:** Win. **Confidence:** confirmed, except the CRT point, which is plausible.
- **Findings:**
  - **(a) Dynamic MSVC CRT:** the default `x86_64-pc-windows-msvc` build links `vcruntime140.dll` dynamically. On a clean Windows install without the VC++ Redistributable, the exe fails to start, and once H8 is fixed it fails silently. **Fix:** set `rustflags = ["-C", "target-feature=+crt-static"]` for the msvc target in `.cargo/config.toml`, or document the redistributable.
  - **(b) No exe icon or version resource:** Explorer, a pinned taskbar icon, Start-menu shortcuts, and Alt-Tab show the generic icon. File Properties has no version or publisher, which also hurts SmartScreen and antivirus reputation. **Fix:** a `build.rs` with `winres` or `embed-resource`, an `.ico` built from `app/assets/icon/`, and `FileVersion` set from `CARGO_PKG_VERSION`. Combine with the M5 manifest.
  - **(c) Unsigned zip, no checksum, no installer:** SmartScreen shows "Windows protected your PC". Linux gets GPG-signed AppImages plus `.sha256` files; Windows gets neither. **Fix:** Authenticode signing (Azure Trusted Signing or a cert secret), a `.sha256`, and optionally an MSI or Inno Setup installer with a Start-menu shortcut.
  - **(d) Dead `assets/` copy:** the zip ships `assets/` copied from `app/assets`, but all fonts, icons, and shaders are `include_bytes!`. The README's "launches from `app/` so runtime assets resolve correctly" is out of date. **Fix:** drop the copy and update the README.

### M20. Unbounded KMZ decompression

- **Where:** `nhc.rs:489-503` (`extract_kml_from_kmz`).
- **Category:** stability. **Platform:** All. **Confidence:** plausible.
- **Evidence:** `read_to_string(&mut file, &mut kml)` with no `.take(N)`. The compressed input is capped at 10 MiB by ureq, but a broken or hostile KMZ can expand to GBs, and this runs on the UI thread (M12).
- **Other problems:**
  - The `.kml` match is case-sensitive, so `.KML` is missed.
  - Non-UTF-8 KML makes `read_to_string` error, and the contour is silently dropped.
  - There's no zip-slip risk: entries are matched by name and read in memory, never extracted to disk.
- **Proposed fix:** check `file.size()` before reading, and use `file.take(32 << 20).read_to_end`. Match names case-insensitively, and decode non-UTF-8 lossily.

### M21. Byte-slice panics on short NHC IDs and timestamps

- **Where:**
  - `nhc.rs:309-311` (`iso_to_ddhhmm`: `&time_part[..2]`, `[3..5]`).
  - `nhc.rs:403`, `405`, `455`, `457` (`construct_image_products`: `&storm_id[..2]`, `[2..4]`, `[6..8.min(len)]`). `[6..8.min(len)]` still panics if `len < 6`.
  - `nhc.rs:370` (`&meta.bin_number[2..]`) is safe, because a prefix check guards it.
- **Category:** stability. **Platform:** All. **Confidence:** plausible. The data is NOAA's, so a bad value is unlikely.
- **Failure scenario:** a malformed `CurrentStorms.json` entry. The `iso_to_ddhhmm` case runs in the parse worker, which has no `catch_unwind`, so it kills the worker (M17). The `construct_image_products` case panics the UI thread via `accept_metadata`.
- **Proposed fix:** use `.get(a..b)` everywhere and skip the entry on `None`, plus wrap worker jobs in `catch_unwind` (M17).

### M22. Fixed KMZ net IDs inside the per-storm loop

- **Where:** `nhc.rs:1095` (`"nhc-kmz-earliest"`) and `nhc.rs:1103` (`"nhc-kmz-mostlikely"`).
- **Category:** correctness. **Platform:** All. **Confidence:** confirmed.
- **Failure scenario:** the second storm's `net::get` does nothing, because the ID already exists (the C2 mechanism). Only the first storm's arrival-time contours are ever shown, and `results.earliest_arrival` is overwritten with the same data. The wind-probability KMZs are deliberately taken from the first storm only, which is fine because they're basin-wide.
- **Proposed fix:** use storm-specific IDs (`nhc-kmz-earliest-{storm_id}`) and keep results per storm, or document that only the first storm is intended.

### M23. Forecast never refreshes, and the geocoder can get stuck

- **Where:** `main.rs:3779-3790` (forecast fetch gate), `main.rs:3844-3847` (`poll_geo` result handling).
- **Category:** stability. **Platform:** All. **Confidence:** confirmed.
- **Failure scenarios:**
  - The fetch condition is `stale = forecast_coords != Some(target)`, which only changes when the coordinates change. Leaving the app open shows hours-old forecasts indefinitely, and a failed fetch is never retried.
  - `poll_geo` builds its net ID from the live search-box text. If the user edits the box after pressing Enter, the poll looks for an ID that was never fired, gets `None` forever, and `fc_geo_fired` stays stuck.
- **Proposed fix:** store `forecast_fetched_at` and refresh every 15–30 min, with retry and backoff on failure. Store the submitted query string in state and poll by that.

### M24. Geocoder query is not percent-encoded

- **Where:** `forecast.rs:62-64` (`query.trim().replace(' ', "%20")`).
- **Category:** stability. **Platform:** All; Windows IME input makes non-ASCII common. **Confidence:** confirmed.
- **Failure scenario:** `&`, `#`, `%`, `+`, `?`, or non-ASCII input ("São Paulo", or CJK text from an IME) either injects extra query parameters or makes the `http` crate reject the URI. `poll_geo` turns the error into empty hits, so the search silently returns nothing.
- **Proposed fix:** percent-encode with `url::form_urlencoded` or the `urlencoding` crate.

### M25. Demo mode still downloads and decodes live volumes

- **Where:** `data.rs:276` and `main.rs:947-952` (`if state.demo.is_some() { continue; }`).
- **Category:** perf. **Platform:** All. **Confidence:** confirmed.
- **Failure scenario:** in `--demo` mode the worker keeps polling the live site every 2 min. It downloads and fully decodes a volume (plus KDP, M11), and then the UI throws it away, wasting bandwidth, CPU, and memory.
- **Proposed fix:** don't start the data worker in demo mode, or add a paused state.

### M26. CPU-heavy work on tokio async workers, and possible slow exit

- **Where:** `data.rs:150-152` (`fetch_demo_scan` decodes inside an async task), `cache.rs:79` (`from_slice` in `tokio::spawn`), `data.rs:123` and `demo.rs:105` (`std::fs::read` inside async functions), and `main.rs:569` (`Runtime::new()`, a local in async `main`).
- **Category:** perf and stability. **Platform:** Win+. **Confidence:** plausible. Whether `main` returns normally on window close under macroquad was not verified.
- **Failure scenarios:**
  - Decode and JSON parse run on tokio async worker threads instead of `spawn_blocking`. That starves other cache I/O on low-core machines.
  - If `main` returns on window close, `Runtime::drop` waits for running `spawn_blocking` jobs (raster, borders parse, image decode), so exit can hang for seconds.
- **Proposed fix:** use `spawn_blocking` for CPU and blocking file I/O, and call `rt.shutdown_timeout(Duration::from_millis(500))` (or `shutdown_background()`) on exit.

### M27. Dealias unfold offset can random-walk in noisy regions

- **Where:** `dealias.rs:30-65` (`dealias_radial`).
- **Category:** stability (numerical drift). **Platform:** All. **Confidence:** plausible; not demonstrated on real data.
- **Evidence:** `prev` is set to the unfolded value with no clamp.
- **Failure scenario:**
  - In noisy or non-meteorological regions, consecutive gates that differ by more than the Nyquist velocity let the offset `k * interval` walk over about 1800 gates, reaching hundreds of m/s.
  - `MAX_GAP_GATES = 5` only resets on missing data.
  - Garbage values then reach the colormap and the VAD fit. The `SHIFT_SEARCH` of ±2 intervals can't undo drift larger than 2 intervals relative to the previous radial.
  - There are no NaNs, because the SD censor accumulates in f64.
- **Proposed fix:** reset `prev = None` when `|unfolded| > 2 * nyquist` or `|k| > 1`.

---

## Low

### L1. Gap-fill walk can loop forever on a degenerate sweep

- **Where:** `scope.rs:528-574`.
- **Confidence:** the logic is confirmed; the trigger is unlikely, and gap-fill is off by default.
- **Evidence:** `gaps = (az[k+1]-az[k]).rem_euclid(360.0)` is all 0 when every radial shares one azimuth (mod 360). Then `while span <= MAX_FILL_GAP_DEG { span += gaps[cur]; ... }` never makes progress.
- **Failure scenario:** the blocking worker hangs, `raster_in_flight` never clears, and the radar never updates again for the session.
- **Fix:** bound each walk to `n_az` steps.

### L2. Refresh and animation timers use wall-clock time

- **Where:** `main.rs:1183`, `main.rs:1231`, plus toast, pulse, and double-click timing. macroquad `get_time()` is built on `SystemTime::now()` (`miniquad-ply-0.4.8/src/lib.rs:72-79`).
- **Confidence:** the mechanism is confirmed; the trigger is rare.
- **Failure scenario:** a backward clock step (w32time correction after resume, a manual change, VM drift) makes `now - last_poll` negative, which stalls alert and NHC refresh for the size of the jump.
- **Fix:** use `std::time::Instant` for intervals.

### L3. Avoidable allocations every frame

- **Confidence:** confirmed; the per-item impact is small. The Windows HeapAlloc allocator makes many small allocations slower.
- **Where, in `main.rs`:**
  - `1623-1631`: `site_options` rebuilt with 142 sites × 2 `format!` every frame.
  - `2458` and `3904`: `storm_options` built twice.
  - `2791`/`2909`, and the metrics at `166`/`180`: `wrap_modal_text` runs twice per frame while a modal is open.
  - `2231`: `ChartWidget::RainChart(fc.hours.clone())` clones a `Vec<Hour>` every frame the forecast panel is open.
  - `839`: `pointer_over_ids()` returns a new `Vec`.
- **Where, in the widgets:**
  - `widgets/dropdown.rs:255-264`: `filtered_indices` lowercases the filter and every option's `search_text` every frame while open.
  - `dropdown.rs:121-123`, `widgets/settings.rs:251,266,350,367,416,441,503`, and `widgets/collapsing.rs:46`: a `format!` label every frame.
- **Fix:**
  - Build the site options once.
  - Cache storm options and wrapped modal lines when their source changes.
  - Use `Arc<[Hour]>` for the chart.
  - Pre-lowercase `search_text` and cache the filtered indices keyed on the filter.

### L4. Input binding bugs

- **Confidence:** confirmed.
- **Duplicate `W`:** `main.rs:3410` (Spectrum Width) and `main.rs:3451` (watches toggle), so one press does both.
- **Wheel over the radar panel:** `main.rs:3333` excludes `over_nhc_panel` but not `over_radar_panel`, so scrolling the radar panel also zooms the map.
- **Physical scancodes:** shortcuts use physical scancodes on Windows (miniquad `keycodes.rs`), so on AZERTY and QWERTZ layouts W, Z, A, and `?` land on other keys.
- **Fix:** rebind one of the `W` actions, add `over_radar_panel` to the zoom guard, and consider character-based shortcuts.

### L5. Emoji in UI text

- **Where:** `main.rs:2762` (`"🌐 {title}"`).
- **Problem:** no emoji font is bundled, so this never renders (see AGENTS.md).
- **Fix:** use an `nf` codepoint from `SYMBOL_FONT`.

### L6. NHC text-product IDs keyed by `title.len()`

- **Where:** `main.rs:2676` and `main.rs:4058`.
- **Problem:** the 3 current titles happen to have different lengths. A new or renamed product would collide.
- **Fix:** key by an explicit enum or the product code.

### L7. Justfile doesn't work on Windows

- **Where:** `Justfile` has no `set windows-shell`. The `setup` recipe uses `pip`/`cargo install`, and `wayland-smoke` runs `scripts/wayland-smoke.sh`. README and CONTRIBUTING have no Windows dev notes, beyond README:150 ("x86-64 build published").
- **Problem:** `just` on Windows defaults to `sh`, which a plain Windows dev box doesn't have, so `just run` (required by AGENTS.md) and `just ci` fail.
- **Fix:** `set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]`, mark Linux-only recipes, and document Windows setup.

### L8. `cargo deny` doesn't check Windows targets

- **Where:** `deny.toml` `[graph].targets` lists only linux-gnu and wasm32.
- **Problem:** Windows-only crates (`windows-sys`, `winapi`) skip the license, advisory, and duplicate checks.
- **Fix:** add `x86_64-pc-windows-msvc` (and the macOS triples).

### L9. Tokio runtime relies on feature unification

- **Where:** `app/Cargo.toml` (`tokio` features `rt`, `time`, `sync`) and `main.rs:569` (`Runtime::new()`, which needs `rt-multi-thread`).
- **Problem:** it compiles only because another dependency turns on `rt-multi-thread`.
- **Fix:** add `rt-multi-thread` explicitly.

### L10. Self-heal deletes fail on locked files

- **Where:** `cache.rs:275-278`, `cache.rs:85`, `cache.rs:153`, `cache.rs:201`, `cache.rs:257`, `borders.rs:123`. Ply's `remove` is a bare `std::fs::remove_file`, and its result is discarded.
- **Platform:** Win+. **Confidence:** plausible.
- **Problem:** if antivirus or a concurrent reader holds the file open, the delete fails. The corrupt entry survives, and it's re-read and re-logged on every start.
- **Fix:** retry, or overwrite with an empty or valid value instead of deleting. This is covered by the M3 atomic-write work.

### L11. No real system location on Windows

- **Where:** `system_location.rs:62-76`.
- **Confidence:** confirmed. There's no crash or hang risk.
- **Detail:** on non-macOS, `SystemLocator::start()` returns a unit struct, and `poll()` returns `SystemFix::Unavailable` immediately. Windows users therefore always fall back to IP geolocation (ipapi.co) or manual entry.
- **Fix (optional):** implement it with `Windows.Devices.Geolocation`.

### L12. Raster range-texture recomputation

- **Where:** `scope.rs:662-681` calls `nonmet::range_texture` (`nonmet.rs:~175-210`).
- **Confidence:** confirmed. It runs on the worker, so it's latency only.
- **Problem:** ZDR and ΦDP range textures are recomputed for the whole aux sweeps on every reflectivity raster with fuzzy non-met on. That's O(radials × gates × kernel), about 1M gates × 2 sweeps × a 5–9 window.
- **Fix:** cache per scan (compute alongside KDP), or use a sliding-window sum-of-squares.

### L13. `circle_subsegments` allocates when only a boolean is needed

- **Where:** `geo.rs:918-965`, called at `borders.rs:228`.
- **Problem:** it makes two `Vec` allocations per segment, and the call site only checks `.is_empty()`. This runs at border load time.
- **Fix:** add a non-allocating `segment_touches_circle(a, b, r) -> bool` (closest-point distance), and use `[f32; 4]` for `ts`.

### L14. Dealias API guards

- **Where:** `dealias.rs:117-129` and `dealias.rs:194-196`.
- **Problems:**
  - `azimuths_deg[i]` is indexed with no length check against `gates.len()`. The only caller builds both from the same `az_order`, so it's safe today.
  - A NaN `nyquist` gets past `nyquist <= 0.0` and turns every gate into NaN. The caller checks `> 0.0`, so that can't happen today.
- **Fix:** add `debug_assert_eq!` or an early return on a length mismatch, and use `if !(nyquist > 0.0)`.

### L15. Background workers process one job at a time per kind, and image decode has no size limit

- **Where:** `background_work.rs:226`, `:252`, `:282`, and `:383` (`decode_image`). Also `main.rs:1396-1403`.
- **Problems:**
  - A slow NHC GIS parse delays the alerts and borders parses queued behind it.
  - `decode_image` doesn't limit image dimensions.
  - `image.width as u16` truncates in `main.rs` for widths over 65535.
  - `let _ = results.send(...).await` drops results if the receiver is gone, which is harmless at shutdown.
- **Fix:** give the NHC GIS parse its own worker or a priority, cap decoded dimensions (for example 8192²), and use `u16::try_from`.

### L16. Border `MultiLineString` features dropped

- **Where:** `borders.rs:204-215` (`extract_linestring`).
- **Confidence:** plausible.
- **Problem:** it assumes `LineString`. A `MultiLineString` feature, which some Natural Earth boundary data contains, fails `arr.get(1)?.as_f64()` and is silently dropped, so some border segments go missing.
- **Fix:** handle `MultiLineString`.

### L17. NaN or inf coordinates from KML

- **Where:** `nhc.rs:632-644`.
- **Problem:** Rust's `f64::parse` accepts `"NaN"` and `"inf"`, so malformed KML can put non-finite coordinates into rings.
- **Fix:** filter with `is_finite()` and clamp to valid lat/lon ranges.

### L18. CRLF and HTML entities in NHC text

- **Where:** `nhc.rs:319-330` (`extract_pre_content`).
- **Confidence:** plausible.
- **Problem:** `\r\n` is kept in `TextProduct.content`, and only five HTML entities are decoded.
- **Fix:** normalize `\r\n` to `\n` and decode numeric entities.

### L19. Cities data fails as a whole

- **Where:** `cities.rs:38` and `cities.rs:49` (`unwrap_or_default()`).
- **Problem:** one feature with a null `pop` or a bad shape silently empties every city. The data is bundled and tested, so the risk is low.
- **Fix:** skip bad features one at a time.

### L20. Alert hit-test allocates per click

- **Where:** `alerts.rs:228-243` (`hit_test`). `ring_affects_scope` is at `alerts.rs:258` and `borders.rs:221`.
- **Problem:** it allocates and re-projects every ring on each click. That's fine for a click, but it would be O(alerts × vertices) if anything called it per frame. No per-frame caller was found; `alert_affects_scope` has no caller in `main.rs`.
- **Fix:** reuse the H5 cached km-offset geometry.

### L21. RLE decompress can request a huge allocation

- **Where:** `rle.rs` (`decompress`).
- **Problem:** it's bounds-checked, but a corrupt repeat count can request up to 4 GB through `resize`.
- **Fix:** cap the output length at the expected gate count before resizing. This matters once C1 switches to the RLE path.

### L22. Binary size: embedded fonts

- **Where:** `main.rs:576-594` and `widgets/mod.rs:10`.
- **Detail:** about 9.5 MB of fonts are `include_bytes!`. The two OpenDyslexic fonts are 4.3 MB of that, loaded even when unused.
- **Fix (optional):** load the OpenDyslexic fonts lazily, or ship them as optional assets.

---

## Correctness notes outside the perf/stability brief

These came up during review and are recorded so they aren't lost.

- **False colour rim on echo edges** (`scope.rs`, value-texture path):
  - Linear filtering of the value texture blends R toward 0 at coverage edges, because no-data texels encode R=0.
  - Before the 0.5-alpha discard applies, every storm edge and QC-hole rim shows a thin band of false low-dBZ colour.
  - **Fix:** on the CPU, extend each edge texel's R value into neighbouring empty texels before upload, or use nearest sampling for R plus a separate alpha.
- **Polygonal range rings at high zoom:**
  - `draw_circle_lines` uses 30 segments (`macroquad-ply-0.4.14/src/shapes.rs:264`).
  - At zoom 8 the 230 km ring has a radius of about 4000 px, which puts the segment midpoints about 22 px off the true ring.
  - **Fix:** scale the segment count with the on-screen radius.
- **Dealias seeding:** the first radial in azimuth order is the unreferenced seed, so a wrongly seeded radial 0 carries through the whole sweep. The `ponytail` comment in `dealias.rs` already acknowledges this.

---

## Checked and found fine

- **Paths and filenames:**
  - Ply storage uses `PathBuf` built from `%APPDATA%` (falling back to `%USERPROFILE%\AppData\Roaming`), and keys are split on `/` into path components.
  - Usernames with non-ASCII characters or spaces are fine.
  - No MAX_PATH risk (`%APPDATA%\rustywx\<key>` is well under 260).
  - Cache keys are `scan_{site}`, `scan_rle_v2_{site}`, `selected_site`, `settings.json`, `state_borders_v8`, and `demo_volume_{name}`. Site IDs are static 4-letter IDs and demo names are static, so there are no reserved device names (CON, NUL, COM1…), no illegal characters (`:` `*` `?`), and no case collisions.
- **CRLF in local files:** only JSON via serde, and `location.rs` input is `trim()`med and split on `,`, space, and tab.
- **Stack overflow:** there's no recursion and there are no large stack arrays anywhere in `dealias.rs`, `kdp.rs`, `melting_layer.rs`, `nonmet.rs`, `geo.rs`, `model.rs`, `state.rs`, `demo.rs`, `data.rs`, or `background_work.rs`. Rasterizing runs on `spawn_blocking` (2 MB stack) and KDP on a `std::thread` (2 MB), so the 1 MB Windows main-thread stack doesn't come into play.
- **Indexing:** `.get(g)` is used throughout the radar algorithms, and radials with different gate counts are handled and tested. All `usize` subtractions are guarded (`checked_sub`, `saturating_sub`, or explicit checks), so there are no overflow differences between debug and release.
- **Numerical edge cases:**
  - KDP median over an empty window can't happen.
  - `range_derivative` guards `gate_spacing_km <= 0` and `half_window == 0`.
  - `fit_vad`'s `det.abs() < 1e-6` is fine in f64.
  - `gate_spacing == 0` is rejected at `model.rs:291`.
  - NaN azimuths or geometry degrade to `None`.
  - The `nonmet.rs` NaN and zero-division handling is fine.
- **Timer granularity:** there's no `thread::sleep` or spin loop that depends on Windows' ~15.6 ms resolution. The only `recv_timeout` is 120 s or longer.
- **Background workers:**
  - The generation and stale-result checks for parse, raster, and image work are correct.
  - The queues are bounded (capacity 4, `try_send`), so submits never block.
  - At most one raster job is in flight.
  - The data worker drains queued site changes correctly.
- **Renderer:**
  - No division by screen dimensions, so a 0×0 or minimized window is safe in `scope.rs`.
  - A 2048² texture is within every GL 3.1-class GPU's limit (there's no `GL_MAX_TEXTURE_SIZE` query, so raising `RASTER_SIZE_PX` above 4096 would need one).
  - The LUT uses clamp wrap.
  - `lowp`/`mediump` are ignored on desktop GL.
  - `nearest_two_radial_indices` callers guard `n == 0`.
  - The hole-fill loop always terminates; it's slow (H3), not infinite.
- **Assets:** all fonts, icons, and shaders are `include_bytes!`, so the working directory and launching from a shortcut don't matter. The widget `unwrap`s are test-only, and there are no glyph-lookup unwraps.
- **Misc:**
  - `webbrowser::open` is fire-and-forget and safe on Windows.
  - Focus loss on Windows clears held keys and mouse state, so drags don't stick.
  - The release profile is sane (`lto = "thin"`, `codegen-units = 1`, `strip = true`, and unwinding panics, which is what a panic hook needs).
  - The runtime window icon is set through miniquad `set_icon`.
  - The NHC image decode path uploads one image per frame, as intended.

---

## Suggested remediation order

The order puts the biggest stability gain per line of code first, then Windows-specific breakage, then performance, then release hygiene.

1. **Stop the UI-thread scan serialize and bound the cache:** C1, M16, M1. Switch to the existing compressed path, use `spawn_blocking`, add LRU eviction, and fix the cache-vs-live race. This removes the biggest freeze and memory spike.
2. **Make network feeds recover:**
   - C2 (unique net IDs per attempt plus backoff, toast on transition only)
   - C3 (timeouts and deadlines, cancellable radar fetch)
   - H6 (poll all border IDs)
   - M14 (skip bad alert features)
   - M22, M23
3. **Windows crash visibility and console:** H8 (subsystem, panic hook, file log), M4 (graceful storage and GL failure), M17 (`catch_unwind` in workers), H9 (char-boundary slicing, turn off file drops), M21.
4. **Windows input and DPI:** H2 (wheel normalization), M9 (unified text-focus flag, fullscreen restore), M8 (char queue), M5 (manifest with PerMonitorV2, `is_mobile` breakpoint, window sizing), L4.
5. **Windows TLS and proxy:** H1. This needs a ply patch or fork, so start the upstream conversation early.
6. **CI coverage:** H7 (Windows job in `ci.yml`, a launch check in release), L7, L8, L9. Ideally this goes in alongside step 3 so the fixes there are tested on Windows.
7. **Render-thread performance:** H5 (cached projected geometry), M6 (single clear), M7 (minimized throttle, `get_frame_time` animation), M15 (persistent texture), M12 (NHC phase 2 off-thread), L3.
8. **Raster latency:**
   - H3 (BFS hole fill, respect the range floor)
   - H4 (summed-area filters, 2048 benchmark)
   - M2 (retry cap, keep the old texture, accept current results)
   - M10 (`Arc` sweeps, melting layer once per scan)
   - M11 (allocation-free KDP, lazy)
   - L12
9. **Settings durability:** M3 (single writer, atomic rename, `.bak`), L10.
10. **Release packaging:** M19 (static CRT, icon and version resource, signing, checksum, drop `assets/`), and M18 (shader fallback).
11. **Remaining Medium and Low items** as capacity allows: M13, M20, M24–M27, L1, L2, L5, L6, L11, L13–L22, and the correctness notes.
