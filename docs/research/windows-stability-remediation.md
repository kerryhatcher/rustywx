# Windows stability remediation: research and recommendations

Research date: 2026-10-09. Companion to `docs/reviews/windows-stability-review.md` (finding ids C1, H1, M3 and so on refer to that file). No code was changed.

I have no Windows machine here. Everything below comes from reading source in `~/.cargo/registry`, the rustc 1.96.1 std source, vendor docs, and crates.io metadata. Anything that needs a Windows run to confirm is marked **[verify on Windows]**.

## Summary of recommendations

| Item | Recommendation | Patch/fork needed? |
|---|---|---|
| A. Networking (C2, C3, H1, M13) | Short term: use ply's existing `Request::cancel()` plus app-side deadlines and backoff, and enable ureq's `win-system-proxy` from our own `Cargo.toml`. End state: an app-owned HTTP module on **reqwest 0.13** (already in the build graph, with platform verifier and system proxy). | No |
| B. miniquad (H2, H9c, M5) | One `[patch.crates-io]` of `miniquad-ply` 0.4.8 for the wheel, drop-file and DPI fixes, plus an upstream PR. Manifest and window sizing are app-side. | Yes (small) |
| C. Packaging (H8, M19) | `windows_subsystem`, a std-only panic hook, stderr redirected to a log file, `embed-manifest` 1.5.1 plus `winresource` 0.1.31, static CRT, SignPath Foundation for signing. | No |
| D. CI (H7, L7, L8) | Add a `windows-latest` job to `ci.yml`. Smoke-test with `--version`, not a GL launch. | No |
| E. Cache and settings (C1, M3, M16, L10, L21) | Keep the compressed format as the first fix, but it only saves about 80%. Move scans to `%LOCALAPPDATA%`, bound them with LRU, and use std-only atomic writes with a retry. | No |

## A. Ply networking

### What ply 1.1.1 actually does (read from `ply-engine-1.1.1/src/net/`)

| Behaviour | Source | Consequence |
|---|---|---|
| Every request spawns a new `std::thread` with a fresh `ureq::Agent::new_with_defaults()` | `mod.rs:107-111` | No pooling, no thread cap. A hung request pins its thread. |
| `HttpConfig` exposes only `header`, `body`, `body_bytes` | `http.rs:5-38` | There is **no** way to set timeouts, TLS roots, or a proxy per request. |
| Same-id re-fire is a silent no-op while an entry exists | `mod.rs:91` ("Idempotent") | A stored `Err` or `Done` blocks every retry with that id (the C2 root cause). |
| `Pending` entries are never evicted | `mod.rs:59` (`HttpRequestState::Pending(_) => true`) | A hung request stays forever. |
| `Done` and `Error` entries are evicted after 60 frames **without access**; `net::request(id)` and `Request::response()` reset the counter | `mod.rs:60-63`, `http.rs:67` | Polling every frame keeps an entry alive indefinitely. Not polling for 60 frames loses the response silently. |
| **`Request::cancel(self)` removes the entry** | `http.rs:115-118` | This is the missing piece for C2. It makes the id reusable. It does **not** stop the thread; the result goes to a dropped channel. |
| `net::request(id)` returns `None` when no entry exists | `mod.rs:238-244` | It doubles as an "is anything tracked for this id" check. |

`docs/ply-issues/06` and `07` ask for these behaviours to be documented. Both are accurate against 1.1.1. The review's remark that unique ids are the "reliable route" because there may be no remove API is outdated: `cancel()` exists.

### ureq 3.3.0 facts (`ureq-3.3.0/src/config.rs`, `proxy.rs`, `tls/mod.rs`)

- `Timeouts::default()` is all `None` except `await_100` (1 s). `Config::default()` takes the proxy from `Proxy::try_from_env()`.
- `TlsConfig::default()` sets `root_certs: RootCerts::WebPki`. Enabling the `platform-verifier` feature only makes `RootCerts::PlatformVerifier` *available*; it does **not** change the default. Enabling the feature alone fixes nothing.
- Feature `win-system-proxy = ["dep:winreg"]` is wired into `Proxy::try_from_env()` (`proxy.rs:241-265`). Enabling it from outside *does* change behaviour, because it is a compile-time `cfg` inside the default path.
  - Limit: it reads only `HKCU\...\Internet Settings\ProxyServer` and wraps the value as `http://{value}`. The per-protocol form (`http=a:80;https=b:443`) and PAC/`AutoConfigURL` are not handled. **[verify on Windows]**
- ureq changelog (docs of the `algesten/ureq` repo): 3.4.0 bumps rustls-platform-verifier to 0.7.0 and does TLS to the target through HTTPS CONNECT proxies; 3.4.1 fixes timeout budgets that restarted for later phases and makes `timeout_connect` cover the TLS handshake; 3.4.2 is current (crates.io, 2026-09-13). The lock has 3.3.0. ply's requirement is `ureq = "3"`, so `cargo update -p ureq` is allowed. Only `ply-engine` depends on ureq in our `Cargo.lock`.

### reqwest is already in the build (this changes the options)

`nexrad-data` pulls in `reqwest 0.13.4` with `default-features = false, features = ["rustls"]` (`nexrad-data-1.0.0-rc.7/Cargo.toml:157-160`).

- reqwest 0.13's `rustls` feature depends on `rustls-platform-verifier` (`reqwest-0.13.4/Cargo.toml:169-171`), which is why `rustls-platform-verifier 0.7.0` is in our lock. **The radar downloads already use the Windows certificate store.** Only ply's net module (alerts, borders, NHC, forecast, geocoder) does not.
- reqwest's `system-proxy` feature maps to `hyper-util/client-proxy-system`. hyper-util 0.1.20 (`client/proxy/matcher.rs:109`, `:238`, `:674`) reads the system configuration including the Windows registry `ProxyEnable`. PAC is not supported.
- Timeouts: `ClientBuilder::timeout`, `connect_timeout`, `read_timeout`. All default to none, so we must set them.

### Options compared

| Option | Timeouts | Win cert store | Win proxy | Effort / risk |
|---|---|---|---|---|
| 0. Do nothing in ply, app-side deadline and `cancel()` | App-side only; the thread leaks until the OS gives up | No | Partial, via `ureq/win-system-proxy` | Smallest. Good stopgap. |
| 1. **App-owned module on reqwest** | Real (per-request) | Yes (already default) | Yes | Medium. No fork, no patch. |
| 2. `[patch.crates-io] ureq` (vendored copy, two default changes) | Yes (global default) | Yes | Yes | Small diff. See caveats below. |
| 3. Fork ply-engine | Yes | Yes | Yes | Large. ply also drags `miniquad-ply`, which is already a fork of a fork. |
| 4. Upstream to ply | Eventually | Eventually | Eventually | Do alongside, but don't wait on it. |

Caveats for option 2 (and for every `[patch]`): cargo ignores `[patch]` when publishing, and the release workflow runs `cargo publish`. Anyone running `cargo install rustywx` would get the unpatched behaviour. GitHub release binaries are patched. The vendored crate also needs re-syncing on each ureq bump.

**Recommendation**
1. **Now (no new dependencies, no patch):**
   - Use `cancel()` for C2 (pattern below).
   - Add `ureq = { version = "3", default-features = false, features = ["win-system-proxy"] }` to `app/Cargo.toml` so the registry proxy is honoured. Cargo feature unification merges it with ply's `rustls` feature.
   - Run `cargo update -p ureq` to reach 3.4.2.
   - App-side deadlines: record `Instant` at fire, `cancel()` after about 30 s, back off.
2. **End state:** replace `net::get` with a small `app/src/http.rs` on reqwest, because it fixes C2, C3, H1 and M13 in one place. Dependency line:
   ```toml
   reqwest = { version = "0.13", default-features = false, features = ["rustls", "system-proxy", "gzip"] }
   ```
   (lock: 0.13.4; latest 0.13.5.) The crate is already compiled, so the cost is two small crates (`hyper-util` proxy code and gzip decoding).
   - Each feed owns a state machine `Idle | InFlight{started, JoinHandle} | Backoff{until, attempt}`.
   - Results arrive over a `tokio::sync::oneshot` that we hold ourselves, so there is no 60-frame eviction and no id dedupe.
   - `JoinHandle::abort()` actually cancels the request and frees the socket.
   - ETag and `If-None-Match` for M13 are plain headers.
   - Client settings: `connect_timeout(10 s)`, `timeout(30 s)` (60 s for large bodies), a `User-Agent`, `redirect` default.
   - The existing multi-thread tokio runtime in `main.rs:569` hosts it.
   - If a wasm build is still a goal, keep ply `net` behind `cfg(target_arch = "wasm32")`. I did not find evidence of an active web build.
3. File an upstream issue (`docs/ply-issues/14-net-timeouts-tls-proxy.md`): `HttpConfig::timeout`, a root-cert choice, and a documented `net::cancel`. Do not depend on a reply: ply 1.1.1 has about 1.3k downloads.

### Safe retry pattern with ply `net` (stopgap)

```rust
// per feed: id: &'static str, state: FeedState { fired_at: Option<Instant>, next_retry: Instant, attempt: u32 }
match net::request(id) {
    None if now >= st.next_retry => { net::get(id, url, |c| c); st.fired_at = Some(now); }
    None => {}                                   // backing off
    Some(req) => match req.response() {
        None if st.fired_at.is_some_and(|t| t.elapsed() > TIMEOUT) => {
            req.cancel();                        // frees the id; thread is abandoned
            st.fail();                           // attempt += 1; next_retry = now + backoff(attempt)
        }
        None => {}                               // still pending
        Some(Ok(resp)) => { handle(resp); req.cancel(); st.ok(); }   // cancel() = consume; stops re-parse every frame
        Some(Err(e)) => { log(e); req.cancel(); st.fail(); }        // toast only on first failure
    },
}
```

Backoff: 5 s, 15 s, 30 s, 60 s, capped at the feed's refresh interval, with ±20% jitter. Rules that follow from the source:
- Always `cancel()` after consuming a `Done` or `Error`. Otherwise polling keeps it alive and the parse repeats (the C2 trace).
- Never rely on same-id re-fire. Use unique ids only for genuinely concurrent requests (the per-storm KMZ ids in M22); stable ids built from the storm id are fine.
- Without ureq timeouts an abandoned thread stays blocked until the OS gives up. With the backoff cap that is bounded (at most about 12 stuck threads per hour per feed). Acceptable for a stopgap, not an end state.

## B. miniquad on Windows

Version facts: `miniquad-ply 0.4.8` (crates.io, 2026-03-01; repo `TheRedDeveloper/miniquad-fix`) is what `macroquad-ply 0.4.14` uses (`package = "miniquad-ply"`). Upstream `not-fl3/miniquad` master (fetched 2026-10-09) has **identical** code for all four issues below, and crates.io `miniquad` is at 0.4.11. So there is no fixed release to upgrade to.

| Issue | Where (0.4.8) | Fix |
|---|---|---|
| Wheel delta is raw `WHEEL_DELTA` (±120 per notch). X11 sends ±1.0 per notch; macOS about 1 to 10. | `native/windows.rs:566-572` | Divide by `120.0` in both `WM_MOUSEWHEEL` and `WM_MOUSEHWHEEL`. |
| `std::fs::read(&path).unwrap()` in the window proc aborts the process on a dropped folder or locked file; `DragFinish` is never called. | `windows.rs:758`; `DragAcceptFiles(hwnd, TRUE)` at `:973` | Don't read file contents at all (the app never uses drops), or `if let Ok(..)`. Add `DragFinish(hdrop)`. |
| `assert_eq!(hr, 0)` on `GetDpiForMonitor` | `windows.rs:1107` | Fall back to 96 dpi. |
| `SetProcessDPIAware()` (system-aware only); no `WM_DPICHANGED` | `windows.rs:1159` | Manifest (below), plus a `WM_DPICHANGED` handler. |

### Wheel normalisation

- **Recommendation: patch the one line in miniquad** (divide by 120). Ply's own scroll containers consume `mouse_wheel()` directly (`ply-engine-1.1.1/src/lib.rs:613-630`, `SCROLL_SPEED = 20.0`), which on Windows means 2,400 px per notch. An app-side shim in `main.rs` (4 call sites) cannot fix that.
- After the patch the app's constants (zoom `* 0.05`, modal scroll) keep their Linux-tuned meaning. Do not also add an app-side `/120`; it would double-divide.
- Hi-res wheels and precision touchpads then deliver fractional notches, which the `1.1f32.powf(notches)` zoom in H2 handles smoothly.

### Dropped-file panic without a patch

If the patch is delayed: the render closure runs on the window's thread, so call `GetActiveWindow()` once and `DragAcceptFiles(hwnd, FALSE)` (`windows-sys 0.61.2`, features `Win32_UI_Shell` and `Win32_UI_WindowsAndMessaging`). This turns off `WM_DROPFILES` entirely. I did not verify that `GetActiveWindow()` returns miniquad's HWND at that moment. **[verify on Windows]**

### Patch mechanism

```toml
# Cargo.toml (workspace root)
[patch.crates-io]
miniquad-ply = { path = "vendor/miniquad-ply" }   # copy of 0.4.8, same version number
```
About 15 changed lines. Same `[patch]` caveat as in A: it is ignored by `cargo publish`. Send the same diff upstream to `TheRedDeveloper/miniquad-fix`, and also to `not-fl3/miniquad` since master has the same bugs, then drop the vendor copy when a release lands.

### DPI manifest (Per-Monitor V2)

- Microsoft recommends the manifest over API calls; an API call after the first window exists is unsupported (MS Learn, *Setting the default DPI awareness for a process*). Once a manifest sets the mode, miniquad's later `SetProcessDPIAware()` is a harmless no-op.
- **Crate: `embed-manifest 1.5.1`** (2026-09-14). It embeds by passing `/MANIFESTINPUT` to `link.exe`, so it needs no `rc.exe`. It has `dpi_awareness(DpiAwareness::{System, PerMonitor, PerMonitorV2, PerMonitorV2Only, ...})`, `long_path_aware(Setting::Enabled)`, `active_code_page(ActiveCodePage::Utf8)`, `requested_execution_level(..)`. Verified by reading the 1.5.1 source.
  ```rust
  // app/build.rs
  use embed_manifest::{embed_manifest, new_manifest, manifest::{DpiAwareness, Setting, ActiveCodePage}};
  fn main() {
      if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
          embed_manifest(
              new_manifest("hatek.rustywx")
                  .dpi_awareness(DpiAwareness::PerMonitorV2)   // see staging note
                  .long_path_aware(Setting::Enabled)
                  .active_code_page(ActiveCodePage::Utf8),
          ).expect("unable to embed manifest");
      }
      println!("cargo:rerun-if-changed=build.rs");
  }
  ```
  Put `embed-manifest = "1.5"` in plain `[build-dependencies]` and gate on `CARGO_CFG_WINDOWS` inside `build.rs` (that also works when cross-compiling).
- `winres 0.1.12` is abandoned (last release 2021). The maintained fork is **`winresource 0.1.31`** (2026-03-16); `embed-resource 3.0.12` is the generic alternative.
- **Staging (important):** miniquad computes its scale once in `init_dpi` and never handles `WM_DPICHANGED`.
  - With Per-Monitor V2 declared and no handler, moving the window to a monitor with a different DPI would *not* blur (that is today's system-aware behaviour) but would keep the old scale factor. I could not test this. **[verify on Windows]**
  - Plan: step 1 ships the manifest with `DpiAwareness::System` plus long paths and UTF-8 (behaviour equals today's, but explicit). Step 2 adds a `WM_DPICHANGED` handler to the miniquad patch (recompute `window_scale`, `content_scale`, `mouse_scale`, `SetWindowPos` to the suggested rect, fire a resize event), then switches the manifest to `PerMonitorV2`.

### Window size versus DPI scale

`window_conf()` (`main.rs:493-520`) requests 900×960 and miniquad creates the window at those *physical* pixels (`windows.rs:941-947`), so `screen_width()` is 900/scale logical.

- `GetDpiForSystem()` returns the real value only when the process is already DPI-aware, so **the manifest must come first**. Without it the call returns 96.
- On `cfg(windows)`, compute `scale = GetDpiForSystem() as f32 / 96.0`, set `window_width/height = (900.0 * scale).round()`, then clamp to the work area from `SystemParametersInfoW(SPI_GETWORKAREA)`, minus an allowance for the title bar and borders.
- `windows-sys = { version = "0.61", features = ["Win32_UI_HiDpi", "Win32_UI_WindowsAndMessaging"] }` under `[target.'cfg(windows)'.dependencies]`. 0.61.2 is already in the lock (alongside 0.45 and 0.52).
- Combine with the M5 `is_mobile` change (a lower, logical-size breakpoint).

## C. Windows packaging

### Console subsystem, panic hook, log file (H8, M4, M17)

1. `#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]` as the first line of `app/src/main.rs`. Debug builds keep the console. Note that `just run` uses `--release`, so on Windows its output goes to the log file, not the terminal. Update AGENTS.md's smoke test accordingly.
2. **Redirect stderr to a file instead of rewriting 35 `eprintln!` calls.** std's Windows stderr calls `get_handle(STD_ERROR_HANDLE)` on every write (rustc 1.96.1 `sys/stdio/windows.rs:68`, `:457`), so a `SetStdHandle(STD_ERROR_HANDLE, file)` at startup captures all of them. Feature `Win32_System_Console` in windows-sys; pass `file.into_raw_handle()`. Keep the `File` alive for the process lifetime. **[verify on Windows]**
3. Panic hook, std only, installed as the first statement in `main()`:
   - Append `{timestamp, thread name, message, location, Backtrace::force_capture()}` to `%LOCALAPPDATA%\rustywx\crash.log`, then call the previous hook.
   - Optional `MessageBoxW` (`Win32_UI_WindowsAndMessaging`) so a startup failure (storage, WGL) is visible: "rustywx crashed; details in <path>".
4. Log rotation: at startup, if `app.log` exceeds 2 MiB, rename it over `app.log.1`. No crate needed.
5. Symbols: `Cargo.toml` has `strip = true` in `[profile.release]`, so backtraces will contain only addresses. Either set `strip = "none"` with `debug = "line-tables-only"` and ship the `.pdb` in the zip, or log module base plus offsets and symbolicate against the release's `.pdb`. The PDB is the practical route; MSVC symbolication needs it next to the exe.

Logging crates, if we later migrate `eprintln!` to `log`:

| Crate | Version | Notes |
|---|---|---|
| `log` | 0.4.34 | Facade. Already in the lock. |
| `fern` | 0.7.1 (2024-12) | Light. Sync file output; no rotation. Good fit with step 4 above. |
| `flexi_logger` | 0.31.10 (2026-08) | Rotation, optional async writer. Heavier. |
| `tracing` + `tracing-appender` | appender 0.2.5 | Non-blocking worker and rotation; also needs `tracing-subscriber`. Overkill here. |

**Recommendation:** steps 1 to 4 now (zero new crates apart from the windows-sys features), then `log` + `fern` when the app actually migrates its diagnostics.

### Icon, version info, static CRT

- Icon and VERSIONINFO: **`winresource 0.1.31`** with a committed multi-size `.ico` (16, 24, 32, 48, 64, 128, 256) built from `app/assets/icon/`. Set `FileVersion`/`ProductVersion` from `CARGO_PKG_VERSION`. It needs `rc.exe` (present on `windows-latest`) or `windres`/`llvm-rc` when cross-compiling.
- Do **not** let `winresource` also embed a manifest: `embed-manifest` already does, and a second manifest risks a duplicate-resource link error (CVT1100). **[verify on first Windows CI build]** A single `build.rs` can call both.
- Static CRT: `.cargo/config.toml`
  ```toml
  [target.x86_64-pc-windows-msvc]
  rustflags = ["-C", "target-feature=+crt-static"]
  ```
  This removes the `vcruntime140.dll` dependency on clean machines (M19a). C code built through `cc` (ring, aws-lc-sys) honours `crt-static`. Cost is roughly 100 to 200 KB. **[verify on Windows CI: link errors LNK2038 would show a mixed /MD vs /MT dependency]**. Alternative: set `RUSTFLAGS` only in the release job.

### Code signing for an open-source project

| Option | Eligibility | Cost | Fit |
|---|---|---|---|
| **SignPath Foundation** (`signpath.org`) | OSI-approved licence, no proprietary components, actively maintained, automated CI builds, documented release process. The certificate is issued to SignPath Foundation, so it appears as the publisher. | Free for qualifying projects | **Best fit.** rustywx is `AGPL-3.0-only` and builds in GitHub Actions. |
| **Azure Artifact Signing** (formerly Trusted Signing) | Public Trust: organisations in US, Canada, EU, UK, AU, NZ, JP, KR, SG, CH, NO, IL; **individuals only in US or Canada** (MS Learn quickstart, updated 2026-10-08). Individual identity needs an Azure billing account of type Individual; validation takes 1 to 20 business days. | Paid (a third-party post cites $9.99/month for the basic tier; I did not confirm it on Azure pricing) | Good fallback if SignPath is declined and the signer is eligible. |
| Commercial OV/EV cert | Anyone | Paid, plus HSM/cloud handling for CI | Only if the above fail. |
| Unsigned | | | SmartScreen warning stays until reputation builds. |

I could not load SignPath's apply or terms page through the fetch tool (it returned only navigation), so the SignPath conditions above come from secondary sources and the SignPath open-source page summary. **Read https://signpath.org/terms and https://signpath.org/apply before applying.** The SignPath process requires crediting them (README and a code-signing policy page) and keeping builds fully automated. Also publish a `.sha256` for the Windows zip (Linux already has one), and drop the `assets/` copy in the zip (M19d).

## D. CI: a `windows-latest` job

The release workflow already builds on `windows-latest` (`release-please.yml:104`), so the toolchain and native dependencies work there. `ci.yml` uses SHA-pinned actions; reuse the same pins:

```yaml
  windows:
    name: windows (check, test, release build)
    runs-on: windows-latest
    env:
      CARGO_INCREMENTAL: "0"
    steps:
      - uses: actions/checkout@34e114876b0b11c390a56381ad16ebd13914f8d5  # v4
      - uses: dtolnay/rust-toolchain@e97e2d8cc328f1b50210efc529dca0028893a2d9  # v1
        with:
          toolchain: stable
      - uses: Swatinem/rust-cache@42dc69e1aa15d09112580998cf2ef0119e2e91ae  # v2
        with:
          key: windows-msvc
      - run: cargo check --all-targets --all-features --locked
      - run: cargo test --all-features --locked
      - run: cargo build --release --locked -p rustywx
      # Verifies the exe starts and links (static CRT, manifest). Needs the app to handle --version before creating a window.
      - run: ./target/release/rustywx.exe --version
```

Notes:
- **Do not smoke-launch the GUI on the hosted runner.** miniquad creates a GL 3.1 core WGL context (`windows/wgl.rs`), and a runner without a GPU or software OpenGL fails exactly like the RDP/VM case in M4. A real launch test would need a software GL (for example a Mesa llvmpipe `opengl32.dll`); I did not evaluate that. `--version` (handled before window creation) still catches missing-DLL and link failures. This needs a small app change.
- `just` is not preinstalled on the runner. If the Windows job should run recipes, install it with a SHA-pinned installer action; otherwise call cargo directly as above.
- `Swatinem/rust-cache` works on Windows. Keep a separate `key` so it does not share a cache with Linux.
- `deny.toml` `[graph].targets`: add `x86_64-pc-windows-msvc` and the macOS triples (L8).

### Justfile on Windows (L7)

- Local `just` is 1.58.0. On Windows it defaults to `sh`, which a plain machine lacks. Add:
  ```just
  set windows-shell := ["pwsh.exe", "-NoLogo", "-NoProfile", "-Command"]
  ```
  The `just` manual lists `windows-shell` and marks it deprecated in favour of the `[windows]` attribute on `set shell`; both work in 1.x. **[verify against `just --help` for the pinned version]**
- Use `pwsh.exe`, not `powershell.exe`: the `run` recipe's `cd app && cargo run --release` needs `&&`, which Windows PowerShell 5.1 does not support. Simpler still, change it to `cargo run --release -p rustywx` and remove the `cd`.
- The smoke test in `AGENTS.md` (`just run &`, `sleep`, `kill %1`) is POSIX-only. Add a Windows recipe with `Start-Process -PassThru`, `Start-Sleep 5`, a `HasExited` check, and `Stop-Process`. Linux-only recipes (`wayland-smoke`, the `setup` pip steps) get `[linux]`/`[unix]` attributes.

## E. Cache and settings

### Is the compressed format adequate? (C1, M16, L21)

`save_scan_compressed` / `load_scan_compressed` (`cache.rs:121-190`, `scan_to_bytes` at `:300`) store 1 tag byte per gate and 4 more bytes for each valid gate, then RLE it (`rle.rs`: runs of 6 or more identical bytes).

Measured on the live cache (`scan_KJGX`, 403,404,127 bytes of JSON): **22.0 M `null` gates and 13.9 M float values**, so about 39% of gates are valid. That is far less sparse than the RLE doc's "99%+" assumption.

| Layout | Estimate for that scan |
|---|---|
| JSON today | 403 MB |
| Current binary + RLE | null runs collapse, but valid gates cost 5 bytes and RLE cannot shrink floats: about 13.9 M × 5 B ≈ **70 to 80 MB**, about 80% smaller. The review's "90% or more" does not hold for real volumes. |
| Quantised to the Level II integer counts (1 byte per gate with sentinel values for none and folded, with per-moment scale/offset) then `zstd` or `lz4` | About 36 MB before compression; plausibly 10 to 25 MB after. **Estimate, not measured.** |
| Cache the raw Archive II volume the app already downloads | Few MB to about 20 MB (depends on the volume); re-decode on load. Decode time was not measured. |

Candidates: `zstd 0.13.3` is already in `Cargo.lock` (via `zip`); `lz4_flex 0.14.0` is pure Rust and new. Level II moments are stored as 8- or 16-bit integers with scale and offset, so quantising back to counts should be lossless. **[verify per moment]**

**Recommendation:**
1. First fix (matches C1): switch to the compressed path, run it inside `spawn_blocking`, and add LRU. At about 75 MB per site, 28 sites is still about 2 GB unbounded, so the LRU is not optional.
2. Bump `CACHE_FORMAT_VERSION` (already supported by `scan_key_rle`) when the layout changes.
3. Before investing in a third format, measure how much the cache actually buys. The app downloads the latest volume every poll anyway, so caching only speeds first paint and site switches. A raw-volume cache would be the smallest and the least format-sensitive.
4. L21: cap the decompressed length (a declared-size header, or a hard limit such as 512 MiB) before any `resize`.

### Storage location

`Storage::new()` takes an **absolute path** and uses it as the root (`ply-engine-1.1.1/src/storage.rs:21-24`), but its relative form resolves to `%APPDATA%` (Roaming) (`:431-440`). Scan blobs do not belong in Roaming profiles.
- Put scan blobs under `dirs::cache_dir()` (`dirs 7.0.0`; Windows `%LOCALAPPDATA%`, macOS `~/Library/Caches`, Linux `$XDG_CACHE_HOME`) plus `rustywx`.
- Keep `settings.json` small in the normal app-data directory.
- Ply's `Storage::save_bytes` is a bare `std::fs::write` (`storage.rs:59`) and `remove` a bare `remove_file`, so atomic writes need our own helper whichever root we use.

### LRU eviction

- Policy: keep the 6 most recently used sites and at most 512 MB total (tunable), never evict the selected site.
- Recency: set `File::set_modified(SystemTime::now())` (std, stable since 1.75) on every successful load, and rely on mtime from saves. **Do not use atime**: NTFS last-access updating is off by default on modern Windows.
- Run one `read_dir` plus sort plus delete pass on a blocking thread after startup, then after each save. Deleting a locked file fails; log it and retry next time (L10) instead of treating it as an error.
- The `lru` crate (0.18.5) is an in-memory cache and does not apply here.

### Atomic write on Windows

How the pieces behave (all read from source):

| API | Windows behaviour |
|---|---|
| `std::fs::rename` (rustc 1.96.1 `sys/fs/windows.rs:1311-1372`) | `MoveFileExW(MOVEFILE_REPLACE_EXISTING)`. On `ERROR_ACCESS_DENIED` it retries with `SetFileInformationByHandle(FileRenameInfoEx, REPLACE_IF_EXISTS \| POSIX_SEMANTICS)`, which can also replace a destination that another handle has open **with `FILE_SHARE_DELETE`**, and ignores read-only. |
| `tempfile 3.27.0` `NamedTempFile::persist` | `MoveFileExW` with `REPLACE_EXISTING` only (`src/file/imp/windows.rs:92-109`). Returns the file inside `PersistError`, so it can be retried. Per its docs it does not sync file contents or the directory. |
| `MoveFileExW` (MS Learn) | `MOVEFILE_WRITE_THROUGH` asks for the move to be flushed to disk. Fails if the destination is open without share-delete (antivirus scan, indexer, sync client). Same volume only for an atomic replace. |
| `atomic-write-file 0.3.1` | Temp file in the destination directory then replace; fsyncs on Unix. Docs do not name the Windows calls. Less retry control. |

**Recommendation: write the helper with std only** (`app/src/fsutil.rs`, about 40 lines), because `std::fs::rename` has the better fallback:
1. Create `{dir}/.{name}.{pid}.{n}.tmp` in the **same directory** (same volume, and a temp location AV is more likely to exclude than `%TEMP%`), `write_all`, `sync_all()`, drop the handle.
2. `std::fs::rename(tmp, dest)`. On `PermissionDenied`, error code 32 (`ERROR_SHARING_VIOLATION`) or 33 (`ERROR_LOCK_VIOLATION`), retry 5 times with 20, 40, 80, 160, 320 ms sleeps (about 0.6 s, on a blocking thread, never the UI thread).
3. If it still fails, delete the temp file, log once, keep the in-memory value dirty, and retry on the next change or a 30 s tick. Toast once, not per failure.
4. On startup sweep stale `.*.tmp` files older than an hour.

If we prefer a crate, `tempfile = "3.27"` is already in the lock and its `PersistError` returns the temp file for retries; it just lacks the POSIX-semantics fallback. Windows needs no directory fsync (there is no API for it); for caches that is fine.

OneDrive and other sync clients: Known Folder Move redirects Documents, Desktop and Pictures, not `AppData`. Roaming-profile and folder-redirection setups can include `AppData\Roaming`, which is the other reason to keep large blobs in Local.

### Settings serialization (M3)

- One writer task fed by a `tokio::sync::watch<Settings>`: coalesce bursts with a 300 to 500 ms debounce, serialise on the blocking pool, and flush synchronously on exit. This removes the lost-update race (a stale snapshot finishing last).
- Sequence: `to_vec_pretty` → temp → `sync_all` → copy the current file to `settings.json.bak` (ignore failure) → rename over `settings.json`.
- Load: parse `settings.json`, fall back to `.bak`, and **rename** a corrupt file to `settings.json.corrupt-<ts>` (keep the last 3) instead of deleting it. Add a `schema_version` with `#[serde(default)]` for forward compatibility.
- Two instances of the app: `std::fs::File::try_lock()` is stable since **Rust 1.89** (verified in std `fs.rs`), so an exclusive lock on `instance.lock` in the cache directory needs no crate (Windows uses `LockFileEx`). The second instance can run read-only for settings and cache writes. Optional.

## Exact versions

| Crate | Use | Why |
|---|---|---|
| `reqwest` | `0.13` (lock 0.13.4; latest 0.13.5), `default-features = false`, features `rustls`, `system-proxy`, `gzip` | App-owned HTTP: platform verifier, system proxy, real timeouts |
| `ureq` | `3` (lock 3.3.0 → update to 3.4.2); direct dep with `win-system-proxy` (stopgap only) | Timeout and TLS-handshake fixes; registry proxy |
| `embed-manifest` | `1.5` (1.5.1) build-dep | PerMonitorV2/System DPI, long paths, UTF-8 |
| `winresource` | `0.1.31` build-dep | Exe icon and VERSIONINFO (not `winres` 0.1.12, abandoned) |
| `windows-sys` | `0.61` (0.61.2 already in lock); features `Win32_UI_HiDpi`, `Win32_UI_WindowsAndMessaging`, `Win32_System_Console`, `Win32_UI_Shell` | DPI sizing, MessageBox, SetStdHandle, drop-files |
| `dirs` | `7.0.0` | `cache_dir()` → `%LOCALAPPDATA%` |
| `miniquad-ply` | `0.4.8` patched via `[patch.crates-io]` | Wheel, drop-files, DPI |
| `tempfile` | `3.27.0` (optional; already in lock) | Alternative atomic persist |
| `zstd` | `0.13.3` (already in lock via `zip`) or `lz4_flex 0.14.0` | If a quantised cache format is adopted |
| `log` / `fern` | `0.4.34` / `0.7.1` | Only if diagnostics move off `eprintln!` |

## What I could not verify

- Any Windows runtime behaviour: `SetStdHandle` capture, per-monitor behaviour with and without a `WM_DPICHANGED` handler, `GetActiveWindow()` at drop-file time, and the proxy registry formats.
- SignPath's current terms (the page did not render in the fetch tool) and Azure's pricing.
- The cache savings from quantisation or raw-volume caching are estimates; only the null/valid gate counts of one JSON file were measured.
- Whether a `winresource` plus `embed-manifest` build trips a duplicate-manifest link error on MSVC.
- Whether the pinned `just` version accepts the exact `windows-shell` form shown.

## Sources

- Ply net and storage: `~/.cargo/registry/src/*/ply-engine-1.1.1/src/{net/mod.rs,net/http.rs,storage.rs,lib.rs}`; `docs/ply-issues/06-…`, `07-…`.
- ureq 3.3.0 source (`config.rs`, `proxy.rs`, `tls/mod.rs`); changelog https://github.com/algesten/ureq/blob/main/CHANGELOG.md
- reqwest 0.13.4 and hyper-util 0.1.20 source; nexrad-data 1.0.0-rc.7 `Cargo.toml`.
- miniquad-ply 0.4.8 `src/native/windows.rs`; upstream https://github.com/not-fl3/miniquad/blob/master/src/native/windows.rs
- DPI: https://learn.microsoft.com/en-us/windows/win32/hidpi/setting-the-default-dpi-awareness-for-a-process
- embed-manifest 1.5.1 source and README (static.crates.io tarball); https://docs.rs/embed-manifest
- MoveFileExW: https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw
- Rust std (1.96.1) `sys/fs/windows.rs`, `sys/stdio/windows.rs`, `fs.rs` (file locks); https://docs.rs/tempfile/3.27.0/tempfile/struct.NamedTempFile.html ; https://docs.rs/atomic-write-file/0.3.1/atomic_write_file/
- Signing: https://learn.microsoft.com/azure/artifact-signing/quickstart ; https://signpath.org/ ; https://about.signpath.io/product/open-source
- just settings: https://just.systems/man/en/settings.html
- crates.io API (versions and dates as of 2026-10-09).
- Local measurements: `~/Library/Application Support/rustywx/scan_KJGX`; `app/src/cache.rs`; `app/src/rle.rs`; `.github/workflows/{ci,release-please}.yml`.
