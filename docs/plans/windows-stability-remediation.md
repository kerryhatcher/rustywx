# Windows Stability Remediation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the Critical and High findings from the Windows stability review, in five independently shippable PRs: cache durability, network recovery, Windows crash-proofing, render/raster performance, Windows CI and packaging, plus an outlined follow-up HTTP module.

**Architecture:** Each phase is its own branch and PR off `main`, in the reviewer's order. New behavior goes into small, pure, unit-tested modules:

| Module | Owns |
|---|---|
| `fsutil.rs` | atomic writes, platform directories |
| `scan_cache.rs` | bounded scan cache |
| `net_port.rs` | transport trait, shared with the spaghetti feature |
| `net_feed.rs` | retry, deadline, and toast-once logic per feed |
| `crash_log.rs` | panic hook and log file |
| `text.rs` | char-safe truncation |
| `sat.rs` | summed-area tables |

`main.rs`, `scope.rs`, and `nhc.rs` only get thin wiring. Windows-only fixes inside miniquad go into a vendored `miniquad-ply` behind `[patch.crates-io]`.

**Tech Stack:** Rust 2024, ply-engine 1.1.1 (macroquad-ply 0.4.14, miniquad-ply 0.4.8, ureq 3), tokio, serde.

New dependencies:
- `dirs`
- `windows-sys 0.61` (`cfg(windows)` only)
- build-only: `embed-manifest 1.5`, `winresource 0.1.31`
- a direct `ureq` feature entry (`win-system-proxy`)

**Spec:** `docs/reviews/windows-stability-review.md` (finding ids C1, H5, M21 and so on) and `docs/research/windows-stability-remediation.md` (fix designs, crate versions, `[verify on Windows]` items). Read the finding a task cites before starting that task.

## Global Constraints

- Package is `rustywx` in `app/`. Run cargo as `cargo <cmd> -p rustywx` unless a step says otherwise.
- Every PR must pass `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-features`. Run `cargo fmt --all` before each commit.
- AGENTS.md smoke test before claiming any task done: `just run & sleep 3; kill %1 2>/dev/null || true`. After Phase 2, a Windows release build writes stderr to the log file (see Task 17), not the terminal.
- Conventional Commits (`fix:`, `feat:`, `perf:`, `build:`, `ci:`, `test:`, `docs:`, `refactor:`).
- No byte-index slicing (`&s[..n]`) of any string that didn't come from a literal. Use `str::get`, `strip_prefix`/`strip_suffix`, or `text::truncate_chars` (Task 18).
- Nothing new may block the UI thread. That covers disk I/O, (de)serialization of anything over a few KB, gzip, and JSON parse. Use the data worker, `tokio::task::spawn_blocking`, or `BackgroundWorkers`.
- Ply `net` rules (research section A):
  - Use stable ids.
  - Call `Request::cancel()` after consuming **every** `Done` or `Err`, and on an app-side deadline.
  - Never rely on a re-`get` of an id that still has an entry.
  - Back off on failure, and toast only on the first failure of a streak.
- Icons only from `widgets::nf` with `SYMBOL_FONT`. No emoji.
- Don't add a dependency that isn't named in a task.
- Items marked **[verify on Windows]** can't be proven on macOS or Linux. They are done only after the check in **Appendix B** passes on a real Windows 10/11 machine (not the hosted runner, unless the item says so).
- Don't edit the `.claude/worktrees/radar-improvments` worktree or its 2026-08-27 plan. See "Overlap with the 2026-08-27 audit plan".
- **Verified by the planner (2026-10-16):** the pure-module code was compiled and its tests run in a scratch crate on macOS. That covers Tasks 1–4 (`fsutil`, the bounded RLE and codec, `scan_cache`), 7–8 (`net_port`, `net_feed`), 17 (`crash_log`, non-Windows parts), 18 (`text`), 20 (`sat`), 21–22 (the SAT and BFS rewrites, bit-identical to the current `scope.rs` code on the equivalence tests), and 23 (`KmPolyline`). 53 tests pass and clippy is clean. Not verified: the `main.rs`/`nhc.rs`/`data.rs` glue (Tasks 5, 6, 9–13, 16), the miniquad patch (15), and all Windows-only code. A compile error in the glue means an integration mismatch, not a logic error; fix it and say so in the PR.

## Review Focus

Each of these is pinned by a test in the task named.

1. **A corrupt or hostile cache file.** A truncated file, a repeat count of `u32::MAX`, or a gate count of 4 billion. Expected: a cache miss, no multi-GB allocation, no panic. Pinned by `decompress_bounded_rejects_huge_repeat` and `bytes_to_scan_rejects_huge_gate_count_without_allocating` (Task 2) and `load_blocking_treats_corrupt_file_as_miss_and_removes_it` (Task 4).
2. **The destination file is held open by antivirus or a sync client during save.** Expected: retry with backoff, then a clean error; never a torn file or a lost previous copy. Pinned by `write_atomic_retries_transient_rename_errors` and `write_atomic_keeps_old_file_when_rename_fails` (Task 1). **[verify on Windows]** with a real held handle (Appendix B).
3. **The app launches offline, then the network comes back.** Expected: every feed (alerts, borders, NHC, forecast) recovers on its own with one toast, and the C2 trace can't recur. Pinned by `failed_request_is_cancelled_and_retried_after_backoff` and `toast_only_on_first_failure_of_streak` (Task 8), plus the per-feed tests in Tasks 9–12.
4. **A request hangs forever after sleep/resume** (C3). Expected: it's cancelled at the deadline and retried, and other feeds are unaffected. Pinned by `deadline_cancels_hung_request` (Task 8) and `nhc_phase2_deadline_completes_with_partial_bundle` (Task 12).
5. **Non-ASCII text crosses a truncation boundary**, e.g. Spanish NHC advisories or an em dash at byte 300. Expected: truncated on a char boundary, no panic. Pinned by `truncate_chars_never_splits_a_char` (Task 18) and `iso_to_ddhhmm_rejects_short_time` / `construct_image_products_short_id_is_empty` (Task 18).

---

## Phase map (one PR each)

| PR | Branch | Contents | Review ids | Prerequisite |
|---|---|---|---|---|
| **1a** | `fix/cache-durability` | Tasks 1–6: atomic writes, bounded decode, compressed scan cache with LRU in the OS cache dir, saves off the UI thread, cache/live race, single settings writer | C1, M16, M1, M3, L10, L21 | none |
| **1b** | `fix/net-recovery` | Tasks 7–14: shared `NetPort`, `Feed` retry/deadline/toast-once, alerts, borders (H6), forecast/geocoder, NHC (C2, C3, M22), ureq `win-system-proxy` | C2, C3 (ply feeds), H6, M22, M23 (part), H1 (stopgap) | **`feat/spaghetti-models` merged** (reuses its `NetPort`, `PlyNet`, `parse_queue_has_capacity`) |
| **2** | `fix/windows-crash-proofing` | Tasks 15–18: vendored miniquad patch (wheel, drop-file, DPI assert), `--version` and explicit `main`, `windows_subsystem` + panic hook + log file, byte-slice panics | H2, H9, H8, M21, M4 (visibility) | none (merges cleanly after 1a/1b) |
| **3** | `perf/raster-and-borders` | Tasks 19–23: SAT morphology/smoothing, BFS hole fill, 2048 benchmark, border projection cache | H3, H4, H5 (borders) | none; coordinate with the radar-improvments worktree (it edits `scope.rs`) |
| **4** | `ci/windows` | Tasks 24–26: `windows-latest` CI job, `deny.toml` targets, Justfile on Windows, manifest + icon/version resource, static CRT, release zip fixes | H7, L7, L8, L9, M5 (manifest only), M19 (a, b, d, checksum) | Phase 2 Task 16 (`--version`) |
| **5** | `feat/app-http` | Outline only: app-owned reqwest HTTP module behind `NetPort` | H1, C3 (full), M13 | Phase 1b |

Phases 1a, 2, and 3 can proceed in parallel on separate worktrees. Phase 1b waits for the spaghetti merge. Phase 4 waits for Task 16.

---

## Overlap with the spaghetti-models feature (`feat/spaghetti-models`)

That plan (`docs/plans/spaghetti-models-overlay.md`) introduces helpers this plan should reuse rather than rebuild:

| Spaghetti introduces | This plan | Decision |
|---|---|---|
| `adeck_fetch::NetPort` / `NetPoll` / `PlyNet` (transport trait, Ply impl) | Needed by every feed (Phase 1b) and by the Phase 5 HTTP module | **Task 7 moves them to `app/src/net_port.rs`** and leaves `pub use` re-exports in `adeck_fetch`, so spaghetti code and tests compile unchanged. Task 7 adds `get_with_headers` with a default impl. Phase 5 implements `NetPort` once and every feed switches. |
| `AdeckFetcher` per-storm scheduler (stable id plus cancel, 60 s deadline, backoff, toast-once) | `net_feed::Feed` is the same state machine for one request | Feed's API mirrors AdeckFetcher's (`tick`, `take_body`, `parse_finished`, `status`). Porting `AdeckFetcher` onto `Feed` is **Follow-up F1**, gated on its 13 tests passing unchanged; it's not in Phase 1b, to keep that PR's blast radius down. |
| `BackgroundWorkers::parse_queue_has_capacity()` | Feeds hand bodies to the parse worker only when the queue has room (C2 fix 3) | Reused as-is. This is why 1b requires the spaghetti merge. |
| `adeck_style::ProjectedTrack` / `project_tracks` (km-projection cache keyed by site and revision, review H5) | Border projection cache (Task 23) | Task 23 adds a shared `geo::KmPolyline { km, min_km, max_km }` with `from_latlon` and `visible`. Borders use it. **Task 23 Step 6** refactors `ProjectedTrack` to embed a `KmPolyline` (its 3 projection tests are the gate). |
| `adeck::gunzip_limited` (bounded inflate with `Read::take`) | `rle::decompress_bounded` (Task 2) uses the same cap-then-error shape | Same pattern, separate code (different codecs). The 2026-08-27 plan's KMZ cap (its Task 14) should copy `gunzip_limited`'s shape. |
| No-byte-slicing rule for a-deck parsing | `text::truncate_chars` (Task 18) | Spaghetti needs no change. |
| Edits `main.rs` near the NHC fetch block, `background_work.rs` enums, and the `settings.rs` test literals | 1b edits the same NHC block and the `ParsePayload` match arms; 1a edits the settings tests | Merge spaghetti first. 1a's settings change doesn't touch the `Settings` struct, so the conflict is at most in test literals. |

---

## Overlap with the 2026-08-27 audit plan (worktree `radar-improvments`)

An earlier plan sits uncommitted in `.claude/worktrees/radar-improvments`. Its files are `docs/audit-2026-08-27.md` and `docs/superpowers/plans/2026-08-27-audit-remediation.md`, 28 tasks in total. When I checked, only its Task 1 (shader alpha edge, F1) had uncommitted code. Several of its tasks cover the same ground as this plan. **wx-team-lead should pick one owner per row before either plan's overlapping task starts.** My recommendation is in the last column.

| Audit task (finding) | This plan | Recommendation |
|---|---|---|
| T6 save off render thread (F4), T8 eviction (F5), T9 RLE bound (F7), T21 length bounds (F7), T23 adopt compressed cache (F7) | Tasks 2–5 | **This plan.** It also moves scans out of Roaming, adds atomic writes, and saves from the data worker with no clone, which avoids audit T6's clone-versus-serialize dilemma. Its measured 52% saving (audit F7) is why the LRU is mandatory here. |
| T5/T11 overlay cache: pre-filter rings/alerts to scope (F3) | Task 23: per-site km projection with bbox cull (H5) | **Complementary.** Filtering drops rings; projection removes per-frame trig. If audit T5/T11 lands first, Task 23 projects `overlay_cache.rings` instead of `state.borders`; the code is otherwise identical. |
| T10 2048 benchmark (F6), T25 separable morphology (F6) | Tasks 20–22 (SAT and BFS) | **This plan** (SAT is exact for all three filters, including the NaN-aware blur, which separable morphology doesn't cover). Keep audit T10's benchmark if it lands first and skip Task 22. |
| T12–T14 NHC parser bounds (F8a/b/c = M21, M20) | Task 18 covers M21 only | **Split:** this plan takes M21 (Task 18); the audit plan keeps T14 (KMZ cap, M20). |
| T16/T19 worker panic isolation (F11 = M17) | not in scope | Audit plan. |
| T20 shader fallback (F12 = M18) | not in scope | Audit plan. |
| T22 bound raster retries (F7 part 4 ≈ M2) | not in scope | Audit plan. |
| T15 declare `rt-multi-thread` (F14 = L9) | Task 24 Step 1 | **This plan** (one line, needed for the Windows CI job). Drop it from audit T15. |
| T26 request timeouts (F10 = C3) | Tasks 8–12 (app-side deadlines), Phase 5 (real timeouts) | **This plan**: audit T26 left the approach as a decision point; the research has since settled it (`cancel()` plus deadlines now, reqwest later). |
| T1–T4 (F1, F2a–c), T7, T17, T18, T27, T28 | not in scope | Audit plan. |

---

## File structure

| File | Phase | Responsibility |
|---|---|---|
| `app/src/fsutil.rs` (new) | 1a | `app_data_dir`, `cache_root`, `is_transient_fs_error`, `write_atomic`, `sweep_stale_temps` |
| `app/src/scan_cache.rs` (new) | 1a | Compressed scan files in the OS cache dir, LRU policy, legacy cleanup, blocking load/save |
| `app/src/rle.rs`, `app/src/cache.rs` | 1a | Bounded decompress; `pub(crate)` codec; settings writer; drop the JSON scan path |
| `app/src/data.rs`, `app/src/main.rs`, `app/src/state.rs`, `app/src/borders.rs` | 1a | Save scans from the data worker; load via `ScanCache`; M1 race fix; atomic borders save |
| `app/src/net_port.rs` (new) | 1b | `NetPort`, `NetPoll`, `PlyNet` (moved from `adeck_fetch`), `testing::FakeNet` |
| `app/src/net_feed.rs` (new) | 1b | `RetryPolicy`, `RetryClock`, `Feed` |
| `app/src/alerts.rs`, `borders.rs`, `forecast.rs`, `nhc.rs`, `main.rs`, `state.rs` | 1b | Feeds on `Feed`; NHC on `NetPort` with cancel, deadlines, per-storm KMZ ids |
| `vendor/miniquad-ply/` (new), root `Cargo.toml` | 2 | Patched miniquad (wheel, drop-file, DPI) |
| `app/src/crash_log.rs` (new), `app/src/text.rs` (new) | 2 | Panic hook, log file, stderr redirect; char-safe truncation |
| `app/src/sat.rs` (new), `app/src/scope.rs`, `app/benches/rasterize.rs` | 3 | SAT filters, BFS fill, benchmark, border projection |
| `app/src/geo.rs` | 3 | `KmPolyline` |
| `.github/workflows/ci.yml`, `release-please.yml`, `deny.toml`, `Justfile`, `app/build.rs` (new), `.cargo/config.toml` (new), `app/assets/icon/rustywx.ico` (new) | 4 | Windows CI and packaging |

---

# Phase 1a — Cache and settings durability (PR `fix/cache-durability`)

Closes C1, M16, M1, M3, L10, and L21. After this PR:
- No scan is serialized or written on the UI thread.
- Scans live as compressed binary in the OS cache directory (`%LOCALAPPDATA%\rustywx\scans`, never Roaming), capped at 6 sites and 512 MiB.
- Every write is atomic.
- Settings come from one debounced writer with a `.bak`.
- The 4.5 GB of legacy JSON scans are deleted.

Measured savings are 52% (audit F7, dense volume) to 80% (research section E). The LRU, not the compression, is what bounds disk use.

### Task 1: `fsutil` — directories, atomic write, temp sweep

**Files:**
- Create: `app/src/fsutil.rs`
- Modify: `app/src/lib.rs` (add `pub mod fsutil;` in alphabetical order)
- Modify: `app/Cargo.toml` (add `dirs`)

**Interfaces:**
- Produces:
  - `pub fn app_data_dir() -> Option<PathBuf>` (the same directory Ply `Storage::new("rustywx")` uses)
  - `pub fn cache_root() -> Option<PathBuf>` (`dirs::cache_dir()/rustywx`)
  - `pub fn is_transient_fs_error(&io::Error) -> bool`
  - `pub fn write_atomic(dest: &Path, bytes: &[u8]) -> io::Result<()>`
  - `pub fn sweep_stale_temps(dir: &Path, max_age: Duration) -> usize`
  - `#[cfg(test)] pub(crate) fn test_dir(tag: &str) -> PathBuf`

- [ ] **Step 1: Add the dependency.** Run `cargo add dirs --package rustywx`. The research verified `dirs 7.0.0`; record the version cargo picked in the commit message. `cargo deny check` must still pass (dirs is MIT/Apache-2.0).

- [ ] **Step 2: Write the failing tests.** Create `app/src/fsutil.rs` with the module doc and tests only:

```rust
//! Std-only filesystem helpers: the app's data and cache directories, an
//! atomic replace-on-write that retries the sharing violations antivirus,
//! indexers, and sync clients cause on Windows, and a sweep for temp files a
//! crash left behind. Design: docs/research/windows-stability-remediation.md
//! section E ("Atomic write on Windows").

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    fn env_of<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<OsString> + 'a {
        move |k| pairs.iter().find(|(key, _)| *key == k).map(|(_, v)| OsString::from(v))
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn app_data_dir_matches_ply_layout() {
        let dir = app_data_dir_from(env_of(&[("HOME", "/Users/u")])).unwrap();
        assert_eq!(dir, PathBuf::from("/Users/u/Library/Application Support/rustywx"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn app_data_dir_matches_ply_layout() {
        let xdg = app_data_dir_from(env_of(&[("XDG_DATA_HOME", "/x"), ("HOME", "/h")])).unwrap();
        assert_eq!(xdg, PathBuf::from("/x/rustywx"));
        let home = app_data_dir_from(env_of(&[("HOME", "/h")])).unwrap();
        assert_eq!(home, PathBuf::from("/h/.local/share/rustywx"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn app_data_dir_matches_ply_layout() {
        let appdata = app_data_dir_from(env_of(&[("APPDATA", r"C:\A"), ("USERPROFILE", r"C:\U")])).unwrap();
        assert_eq!(appdata, PathBuf::from(r"C:\A\rustywx"));
        let profile = app_data_dir_from(env_of(&[("USERPROFILE", r"C:\U")])).unwrap();
        assert_eq!(profile, PathBuf::from(r"C:\U\AppData\Roaming\rustywx"));
    }

    #[test]
    fn app_data_dir_is_none_without_env() {
        assert_eq!(app_data_dir_from(env_of(&[])), None);
    }

    #[test]
    fn transient_errors_are_permission_and_windows_sharing_violations() {
        assert!(is_transient_fs_error(&io::Error::from(io::ErrorKind::PermissionDenied)));
        assert!(!is_transient_fs_error(&io::Error::from(io::ErrorKind::NotFound)));
        // 32 = ERROR_SHARING_VIOLATION, 33 = ERROR_LOCK_VIOLATION on Windows;
        // on Unix 32 is EPIPE, which must not be retried.
        assert_eq!(is_transient_fs_error(&io::Error::from_raw_os_error(32)), cfg!(windows));
        assert_eq!(is_transient_fs_error(&io::Error::from_raw_os_error(33)), cfg!(windows));
    }

    fn temps_in(dir: &Path) -> usize {
        std::fs::read_dir(dir)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
            .count()
    }

    #[test]
    fn write_atomic_creates_parent_and_replaces() {
        let dir = test_dir("atomic-replace");
        let dest = dir.join("nested").join("blob.bin");
        write_atomic(&dest, b"first").unwrap();
        write_atomic(&dest, b"second").unwrap();
        assert_eq!(std::fs::read(&dest).unwrap(), b"second");
        assert_eq!(temps_in(&dir.join("nested")), 0);
    }

    #[test]
    fn write_atomic_retries_transient_rename_errors() {
        let dir = test_dir("atomic-retry");
        let dest = dir.join("blob.bin");
        let mut failures_left = 2;
        let mut sleeps = Vec::new();
        write_atomic_with(
            &dest,
            b"data",
            |from: &Path, to: &Path| {
                if failures_left > 0 {
                    failures_left -= 1;
                    Err(io::Error::from(io::ErrorKind::PermissionDenied))
                } else {
                    std::fs::rename(from, to)
                }
            },
            |d| sleeps.push(d),
        )
        .unwrap();
        assert_eq!(std::fs::read(&dest).unwrap(), b"data");
        assert_eq!(sleeps, [Duration::from_millis(20), Duration::from_millis(40)]);
    }

    #[test]
    fn write_atomic_keeps_old_file_when_rename_fails() {
        let dir = test_dir("atomic-keep");
        let dest = dir.join("blob.bin");
        std::fs::write(&dest, b"old").unwrap();
        let mut sleeps = 0;
        let err = write_atomic_with(
            &dest,
            b"new",
            |_: &Path, _: &Path| Err(io::Error::from(io::ErrorKind::PermissionDenied)),
            |_| sleeps += 1,
        )
        .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(sleeps, 5, "five retries, then give up");
        assert_eq!(std::fs::read(&dest).unwrap(), b"old", "previous copy survives");
        assert_eq!(temps_in(&dir), 0, "temp file cleaned up");
    }

    #[test]
    fn write_atomic_does_not_retry_permanent_errors() {
        let dir = test_dir("atomic-permanent");
        let mut sleeps = 0;
        let err = write_atomic_with(
            &dir.join("blob.bin"),
            b"x",
            |_: &Path, _: &Path| Err(io::Error::from(io::ErrorKind::NotFound)),
            |_| sleeps += 1,
        )
        .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert_eq!(sleeps, 0);
    }

    #[test]
    fn sweep_stale_temps_removes_only_old_temp_files() {
        let dir = test_dir("sweep");
        let two_hours_ago = SystemTime::now() - Duration::from_secs(2 * 3600);
        for name in [".old.tmp", "keep.bin"] {
            std::fs::write(dir.join(name), b"x").unwrap();
            std::fs::File::options()
                .write(true)
                .open(dir.join(name))
                .unwrap()
                .set_modified(two_hours_ago)
                .unwrap();
        }
        std::fs::write(dir.join(".fresh.tmp"), b"x").unwrap();
        assert_eq!(sweep_stale_temps(&dir, Duration::from_secs(3600)), 1);
        assert!(!dir.join(".old.tmp").exists());
        assert!(dir.join(".fresh.tmp").exists());
        assert!(dir.join("keep.bin").exists());
    }
}
```

Add `pub mod fsutil;` to `app/src/lib.rs`.

- [ ] **Step 3: Run the tests to verify they fail.** Run `cargo test -p rustywx --lib fsutil::`. Expected: compile error `cannot find function 'write_atomic'`.

- [ ] **Step 4: Implement.** Insert above `#[cfg(test)]`:

```rust
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

/// Retries after the first failed rename: 20, 40, 80, 160, 320 ms (~0.6 s).
const RENAME_RETRIES: u32 = 5;
const FIRST_RETRY_DELAY: Duration = Duration::from_millis(20);
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// The directory `ply_engine::Storage::new("rustywx")` resolves to. Mirrors
/// ply-engine 1.1.1 `storage.rs` `platform_app_data_dir` exactly, so files
/// written here are the ones Ply reads (settings.json, borders, site).
pub fn app_data_dir() -> Option<PathBuf> {
    app_data_dir_from(|k| std::env::var_os(k))
}

fn app_data_dir_from(env: impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    let base = if cfg!(target_os = "windows") {
        env("APPDATA").map(PathBuf::from).or_else(|| {
            env("USERPROFILE").map(|h| PathBuf::from(h).join("AppData").join("Roaming"))
        })
    } else if cfg!(target_os = "macos") {
        env("HOME").map(|h| PathBuf::from(h).join("Library").join("Application Support"))
    } else {
        env("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| env("HOME").map(|h| PathBuf::from(h).join(".local").join("share")))
    };
    base.map(|b| b.join("rustywx"))
}

/// Per-user cache directory for large, re-downloadable blobs:
/// `%LOCALAPPDATA%\rustywx` (never Roaming), `~/Library/Caches/rustywx`,
/// `$XDG_CACHE_HOME/rustywx`.
pub fn cache_root() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("rustywx"))
}

/// Errors worth retrying a rename for: access denied, and on Windows the
/// sharing (32) and lock (33) violations that antivirus, the search indexer,
/// and sync clients cause while they hold the destination open.
pub fn is_transient_fs_error(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::PermissionDenied
        || (cfg!(windows) && matches!(err.raw_os_error(), Some(32 | 33)))
}

/// Write `bytes` to `dest` so a crash or power loss leaves either the old
/// file or the new one, never a torn mix. Writes a temp file in the same
/// directory (same volume, so the rename is atomic), fsyncs it, then
/// renames over `dest`, retrying transient errors. Blocking: call from a
/// worker thread or `spawn_blocking`, never the UI thread.
pub fn write_atomic(dest: &Path, bytes: &[u8]) -> io::Result<()> {
    write_atomic_with(
        dest,
        bytes,
        |from: &Path, to: &Path| std::fs::rename(from, to),
        std::thread::sleep,
    )
}

fn write_atomic_with<R, S>(dest: &Path, bytes: &[u8], mut rename: R, mut sleep: S) -> io::Result<()>
where
    R: FnMut(&Path, &Path) -> io::Result<()>,
    S: FnMut(Duration),
{
    let name = dest.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "write_atomic: destination has no file name")
    })?;
    let dir = match dest.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    std::fs::create_dir_all(dir)?;
    let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp = dir.join(format!(
        ".{}.{}.{n}.tmp",
        name.to_string_lossy(),
        std::process::id()
    ));
    let result = write_then_rename(&tmp, dest, bytes, &mut rename, &mut sleep);
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

fn write_then_rename<R, S>(
    tmp: &Path,
    dest: &Path,
    bytes: &[u8],
    rename: &mut R,
    sleep: &mut S,
) -> io::Result<()>
where
    R: FnMut(&Path, &Path) -> io::Result<()>,
    S: FnMut(Duration),
{
    let mut file = std::fs::File::create(tmp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    let mut delay = FIRST_RETRY_DELAY;
    let mut retries = 0;
    loop {
        match rename(tmp, dest) {
            Ok(()) => return Ok(()),
            Err(e) if retries < RENAME_RETRIES && is_transient_fs_error(&e) => {
                retries += 1;
                sleep(delay);
                delay *= 2;
            }
            Err(e) => return Err(e),
        }
    }
}

/// Delete `.*.tmp` files in `dir` older than `max_age` (left by a crash
/// mid-`write_atomic`). Returns how many were removed. Never fails.
pub fn sweep_stale_temps(dir: &Path, max_age: Duration) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let now = SystemTime::now();
    let mut removed = 0;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !(name.starts_with('.') && name.ends_with(".tmp")) {
            continue;
        }
        let old = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| now.duration_since(t).ok())
            .is_some_and(|age| age > max_age);
        if old && std::fs::remove_file(entry.path()).is_ok() {
            removed += 1;
        }
    }
    removed
}

/// Fresh, empty directory for one test.
#[cfg(test)]
pub(crate) fn test_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rustywx-{tag}-{}-{}",
        std::process::id(),
        TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
```

The tests module's `use std::ffi::OsString;` now duplicates the glob import. Remove it if clippy flags it.

- [ ] **Step 5: Run the tests to verify they pass.** Run `cargo test -p rustywx --lib fsutil::`. Expected: 8 passed (the per-OS test runs once on each OS).

- [ ] **Step 6: Commit.** `git add app/Cargo.toml Cargo.lock app/src/fsutil.rs app/src/lib.rs && git commit -m "feat(storage): atomic file writes with Windows sharing-violation retries"`

---

### Task 2: Bounded decode of cache bytes (L21, audit F7)

**Files:**
- Modify: `app/src/rle.rs` (`decompress` around l.73)
- Modify: `app/src/cache.rs` (`CACHE_FORMAT_VERSION`, `scan_to_bytes`, `bytes_to_scan`, `decode_sweeps`, `Reader`, around l.296–465)

**Interfaces:**
- Produces:
  - `pub fn rle::decompress_bounded(data: &[u8], max_out: usize) -> Result<Vec<u8>, String>`
  - `pub const rle::DEFAULT_MAX_OUTPUT: usize = 1 << 30`
  - `pub(crate) const cache::CACHE_FORMAT_VERSION`
  - `pub(crate) fn cache::scan_to_bytes(&ScanData) -> Vec<u8>`
  - `pub(crate) fn cache::bytes_to_scan(&[u8]) -> Result<ScanData, String>`
  - `fn cache::bounded_capacity(declared: u32, remaining: usize) -> usize`

- [ ] **Step 1: Write the failing tests.** Add to `mod tests` in `app/src/rle.rs`:

```rust
    #[test]
    fn decompress_bounded_rejects_huge_repeat() {
        // 6 bytes asking for u32::MAX copies of 0x00: must error, not allocate 4 GB.
        let mut data = vec![0x80, 0x00];
        data.extend_from_slice(&u32::MAX.to_le_bytes());
        let err = super::decompress_bounded(&data, 1024).unwrap_err();
        assert!(err.contains("exceeds"), "{err}");
    }

    #[test]
    fn decompress_bounded_rejects_literal_past_cap() {
        let data = super::compress(&[1, 2, 3, 4, 5]);
        assert!(super::decompress_bounded(&data, 4).is_err());
        assert_eq!(super::decompress_bounded(&data, 5).unwrap(), [1, 2, 3, 4, 5]);
    }
```

Add to `mod tests` in `app/src/cache.rs` (extend its `use super::{...}` with `bounded_capacity`):

```rust
    #[test]
    fn bounded_capacity_never_exceeds_remaining_bytes() {
        // Every gate costs at least 1 byte, so a declared count larger than
        // the bytes left is corrupt; never pre-allocate for it.
        assert_eq!(bounded_capacity(u32::MAX, 3), 3);
        assert_eq!(bounded_capacity(10, 1_000), 10);
    }

    #[test]
    fn bytes_to_scan_rejects_huge_gate_count_without_allocating() {
        let mut b = Vec::new();
        b.extend_from_slice(&0i64.to_le_bytes()); // timestamp
        b.extend_from_slice(&12u16.to_le_bytes()); // vcp
        b.extend_from_slice(&1u32.to_le_bytes()); // 1 reflectivity sweep
        for _ in 0..4 {
            b.extend_from_slice(&0f32.to_le_bytes()); // elevation, first gate, spacing, nyquist
        }
        b.extend_from_slice(&1u32.to_le_bytes()); // 1 radial
        b.extend_from_slice(&0f32.to_le_bytes()); // azimuth
        b.extend_from_slice(&u32::MAX.to_le_bytes()); // hostile gate count
        // ScanData isn't Debug, so no unwrap_err().
        let Err(err) = bytes_to_scan(&b) else {
            panic!("a hostile gate count must fail");
        };
        assert!(err.contains("truncated"), "{err}");
    }
```

- [ ] **Step 2: Run the tests to verify they fail.** Run `cargo test -p rustywx --lib rle:: cache::`. Expected: compile errors for `decompress_bounded` and `bounded_capacity`.

- [ ] **Step 3: Implement.**

In `app/src/rle.rs`, replace `pub fn decompress` with:

```rust
/// Upper bound for [`decompress`]: no caller decodes more than this.
pub const DEFAULT_MAX_OUTPUT: usize = 1 << 30;

/// Decompress a stream produced by [`compress`], capped at
/// [`DEFAULT_MAX_OUTPUT`] bytes.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    decompress_bounded(data, DEFAULT_MAX_OUTPUT)
}

/// Decompress, refusing to produce more than `max_out` bytes. A corrupt
/// repeat count can otherwise request ~4 GB from 6 input bytes (review L21).
pub fn decompress_bounded(data: &[u8], max_out: usize) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut i = 0;
    let too_big = || format!("rle: output exceeds {max_out} bytes");

    while i < data.len() {
        let control = data[i];
        i += 1;

        if control & 0x80 != 0 {
            let value = *data.get(i).ok_or("rle: truncated repeat run (value)")?;
            i += 1;
            let count_bytes: [u8; 4] = data
                .get(i..i + 4)
                .ok_or("rle: truncated repeat run (count)")?
                .try_into()
                .map_err(|_| "rle: truncated repeat run (count)")?;
            i += 4;
            let count = u32::from_le_bytes(count_bytes) as usize;
            let new_len = out
                .len()
                .checked_add(count)
                .filter(|&n| n <= max_out)
                .ok_or_else(too_big)?;
            out.resize(new_len, value);
        } else {
            let len = control as usize; // lower 7 bits == control, since bit 7 is clear
            let literal = data.get(i..i + len).ok_or("rle: truncated literal run")?;
            if out.len() + len > max_out {
                return Err(too_big());
            }
            out.extend_from_slice(literal);
            i += len;
        }
    }

    Ok(out)
}
```

In `app/src/cache.rs`:
1. Make `const CACHE_FORMAT_VERSION`, `fn scan_to_bytes`, and `fn bytes_to_scan` `pub(crate)`.
2. Add to `impl Reader`:

```rust
    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }
```

3. Add a free function next to `decode_sweeps`:

```rust
/// Capacity to pre-allocate for `declared` gates when `remaining` bytes are
/// left. Each gate costs at least one tag byte, so a larger declared count
/// is corrupt; clamp so a hostile `u32` can't trigger a multi-GB allocation.
fn bounded_capacity(declared: u32, remaining: usize) -> usize {
    (declared as usize).min(remaining)
}
```

4. In `decode_sweeps`, replace the two `Vec::with_capacity(gate_count as usize)` with `Vec::with_capacity(bounded_capacity(gate_count, r.remaining()))`.

- [ ] **Step 4: Run the tests to verify they pass.** Run `cargo test -p rustywx --lib rle:: cache::`. Expected: all pass, including the 4 new tests.

- [ ] **Step 5: Commit.** `git commit -am "fix(cache): bound RLE output and pre-allocation on corrupt input"`

---

### Task 3: Scan-cache naming, LRU policy, and legacy-file rules (pure)

**Files:**
- Create: `app/src/scan_cache.rs`
- Modify: `app/src/lib.rs` (`pub mod scan_cache;`)

**Interfaces:**
- Consumes: `cache::CACHE_FORMAT_VERSION` (Task 2).
- Produces:
  - Constants: `pub const KEEP_SITES: usize = 6`, `pub const MAX_TOTAL_BYTES: u64 = 512 MiB`, `pub const MAX_DECODED_BYTES: usize = 768 MiB`
  - `pub fn scan_file_name(site: &str) -> String`
  - `pub fn site_from_file_name(name: &str) -> Option<&str>`
  - `pub fn is_legacy_scan_file(name: &str) -> bool`
  - `#[derive(Clone, Debug)] pub struct CacheEntry { pub file_name: String, pub bytes: u64, pub modified: SystemTime }`
  - `pub fn entries_to_evict(entries: &[CacheEntry], protect_file: &str, keep: usize, max_total: u64) -> Vec<String>`

- [ ] **Step 1: Write the failing tests.** Create `app/src/scan_cache.rs`:

```rust
//! Bounded, crash-safe on-disk cache of decoded radar volumes (review C1,
//! M16, L21). Replaces the JSON `Cache::save_scan` path. Files live in the OS
//! cache directory (`%LOCALAPPDATA%\rustywx\scans` on Windows, never Roaming)
//! as `scan_v{CACHE_FORMAT_VERSION}_{SITE}.bin`: the flat binary layout from
//! `cache.rs`, RLE-compressed, written with `fsutil::write_atomic`. An LRU
//! pass keeps at most KEEP_SITES files and MAX_TOTAL_BYTES. All I/O here is
//! blocking: call it from the data worker or `spawn_blocking`.

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    fn entry(name: &str, bytes: u64, secs: u64) -> CacheEntry {
        CacheEntry {
            file_name: name.to_string(),
            bytes,
            modified: UNIX_EPOCH + Duration::from_secs(secs),
        }
    }

    #[test]
    fn file_names_round_trip() {
        let name = scan_file_name("KFFC");
        assert_eq!(name, format!("scan_v{}_KFFC.bin", crate::cache::CACHE_FORMAT_VERSION));
        assert_eq!(site_from_file_name(&name), Some("KFFC"));
        assert_eq!(site_from_file_name("scan_v1_KFFC.bin"), None, "other format version");
        assert_eq!(site_from_file_name("scan_KFFC"), None);
        assert_eq!(site_from_file_name(&scan_file_name("")), None);
        assert_eq!(site_from_file_name(".scan_v2_KFFC.bin.123.0.tmp"), None);
    }

    #[test]
    fn legacy_files_are_the_old_json_and_rle_keys() {
        assert!(is_legacy_scan_file("scan_KFFC"));
        assert!(is_legacy_scan_file("scan_rle_v2_KFFC"));
        assert!(!is_legacy_scan_file("settings.json"));
        assert!(!is_legacy_scan_file("state_borders_v8"));
        assert!(!is_legacy_scan_file("selected_site"));
    }

    #[test]
    fn evicts_least_recent_beyond_keep() {
        let entries = [entry("a", 1, 30), entry("b", 1, 20), entry("c", 1, 10)];
        assert_eq!(entries_to_evict(&entries, "none", 2, u64::MAX), ["c"]);
    }

    #[test]
    fn evicts_everything_older_once_budget_is_hit() {
        let entries = [entry("new", 300, 30), entry("mid", 300, 20), entry("old", 1, 10)];
        // "mid" doesn't fit in 500 bytes; "old" is older, so it goes too (strict LRU).
        assert_eq!(entries_to_evict(&entries, "none", 10, 500), ["mid", "old"]);
    }

    #[test]
    fn protected_entry_survives_even_when_oldest() {
        let entries = [entry("a", 1, 30), entry("b", 1, 20), entry("cur", 1, 10)];
        assert_eq!(entries_to_evict(&entries, "cur", 2, u64::MAX), ["b"]);
    }

    #[test]
    fn ties_break_by_name_and_empty_is_fine() {
        let entries = [entry("b", 1, 10), entry("a", 1, 10)];
        assert_eq!(entries_to_evict(&entries, "none", 1, u64::MAX), ["b"]);
        assert!(entries_to_evict(&[], "none", 1, 0).is_empty());
    }
}
```

Add `pub mod scan_cache;` to `lib.rs`.

- [ ] **Step 2: Run the tests to verify they fail.** Run `cargo test -p rustywx --lib scan_cache::`. Expected: compile error `cannot find function 'scan_file_name'`.

- [ ] **Step 3: Implement.** Insert above `#[cfg(test)]`:

```rust
use crate::cache::CACHE_FORMAT_VERSION;
use std::time::SystemTime;

/// Most recently used sites kept on disk.
pub const KEEP_SITES: usize = 6;
/// Total on-disk budget for scan files.
pub const MAX_TOTAL_BYTES: u64 = 512 * 1024 * 1024;
/// Largest decompressed scan accepted (a dense volume is ~180 MB raw).
pub const MAX_DECODED_BYTES: usize = 768 * 1024 * 1024;

pub fn scan_file_name(site: &str) -> String {
    format!("scan_v{CACHE_FORMAT_VERSION}_{site}.bin")
}

/// Inverse of [`scan_file_name`]; `None` for other versions and temp files.
pub fn site_from_file_name(name: &str) -> Option<&str> {
    let prefix = format!("scan_v{CACHE_FORMAT_VERSION}_");
    name.strip_prefix(prefix.as_str())?
        .strip_suffix(".bin")
        .filter(|site| !site.is_empty())
}

/// Old-format scan blobs in the Ply app-data directory (`scan_{SITE}` JSON,
/// `scan_rle_v2_{SITE}`): hundreds of MB each, never read again.
pub fn is_legacy_scan_file(name: &str) -> bool {
    name.starts_with("scan_")
}

#[derive(Clone, Debug)]
pub struct CacheEntry {
    pub file_name: String,
    pub bytes: u64,
    pub modified: SystemTime,
}

/// Strict LRU: newest first (ties by name), keep up to `keep` entries within
/// `max_total` bytes; once one doesn't fit, every older entry is evicted too.
/// `protect_file` (the current site) is always kept and counted first.
pub fn entries_to_evict(
    entries: &[CacheEntry],
    protect_file: &str,
    keep: usize,
    max_total: u64,
) -> Vec<String> {
    let mut sorted: Vec<&CacheEntry> = entries.iter().collect();
    sorted.sort_by(|a, b| {
        b.modified
            .cmp(&a.modified)
            .then_with(|| a.file_name.cmp(&b.file_name))
    });
    let (mut kept, mut total) = match sorted.iter().find(|e| e.file_name == protect_file) {
        Some(p) => (1usize, p.bytes),
        None => (0, 0),
    };
    let mut evicting = false;
    let mut evict = Vec::new();
    for e in sorted {
        if e.file_name == protect_file {
            continue;
        }
        if !evicting && kept < keep && total.saturating_add(e.bytes) <= max_total {
            kept += 1;
            total += e.bytes;
        } else {
            evicting = true;
            evict.push(e.file_name.clone());
        }
    }
    evict
}
```

- [ ] **Step 4: Run the tests to verify they pass.** Run `cargo test -p rustywx --lib scan_cache::`. Expected: 6 passed.

- [ ] **Step 5: Commit.** `git add app/src/scan_cache.rs app/src/lib.rs && git commit -m "feat(cache): LRU policy and file naming for the scan cache"`

---

### Task 4: `ScanCache` blocking I/O (save, load, evict, legacy cleanup)

**Files:**
- Modify: `app/src/scan_cache.rs`

**Interfaces:**
- Consumes:
  - `fsutil::{cache_root, write_atomic, sweep_stale_temps, test_dir}` (Task 1)
  - `rle::{compress, decompress_bounded}` and `cache::{scan_to_bytes, bytes_to_scan}` (Task 2)
- Produces: `#[derive(Clone, Debug)] pub struct ScanCache`, with:
  - `open_default() -> Self`
  - `at(dir: PathBuf) -> Self`
  - `disabled() -> Self`
  - `is_enabled(&self) -> bool`
  - `encode(scan: &ScanData) -> Vec<u8>`
  - `save_blocking(&self, site: &str, raw: &[u8]) -> io::Result<()>`
  - `load_blocking(&self, site: &str) -> Option<ScanData>`
  - `evict_blocking(&self, protect_site: &str) -> usize`
  - `load(&self, site: &str) -> tokio::sync::oneshot::Receiver<Option<ScanData>>`

  It also produces the free function `pub fn remove_legacy_scans(app_dir: &Path) -> usize`.

- [ ] **Step 1: Write the failing tests.** Add to `mod tests`:

```rust
    use crate::model::{RadialData, ScanData, SweepData};

    fn scan(vcp: u16) -> ScanData {
        ScanData {
            timestamp: chrono::Utc::now(),
            reflectivity: vec![SweepData {
                elevation_deg: 0.5,
                radials: vec![RadialData {
                    azimuth_deg: 12.0,
                    gates: vec![None, Some(32.0), None, Some(-5.5)],
                    range_folded: vec![],
                }],
                first_gate_km: 2.125,
                gate_spacing_km: 0.25,
                nyquist_ms: 0.0,
            }],
            velocity: vec![],
            spectrum_width: vec![],
            differential_reflectivity: vec![],
            correlation_coefficient: vec![],
            differential_phase: vec![],
            specific_differential_phase: vec![],
            vcp_number: vcp,
        }
    }

    #[test]
    fn save_then_load_round_trips() {
        let cache = ScanCache::at(crate::fsutil::test_dir("scan-rt"));
        cache.save_blocking("KFFC", &ScanCache::encode(&scan(212))).unwrap();
        let loaded = cache.load_blocking("KFFC").unwrap();
        assert_eq!(loaded.vcp_number, 212);
        assert_eq!(loaded.reflectivity[0].radials[0].gates, [None, Some(32.0), None, Some(-5.5)]);
        assert!(cache.load_blocking("KTLX").is_none());
    }

    #[test]
    fn load_blocking_treats_corrupt_file_as_miss_and_removes_it() {
        let dir = crate::fsutil::test_dir("scan-corrupt");
        let cache = ScanCache::at(dir.clone());
        let mut hostile = vec![0x80, 0x00];
        hostile.extend_from_slice(&u32::MAX.to_le_bytes());
        std::fs::write(dir.join(scan_file_name("KFFC")), hostile).unwrap();
        assert!(cache.load_blocking("KFFC").is_none());
        assert!(!dir.join(scan_file_name("KFFC")).exists(), "self-heals");
    }

    #[test]
    fn save_evicts_beyond_keep_sites_but_never_the_saved_site() {
        let dir = crate::fsutil::test_dir("scan-lru");
        let cache = ScanCache::at(dir.clone());
        let raw = ScanCache::encode(&scan(12));
        for (i, site) in ["K001", "K002", "K003", "K004", "K005", "K006", "K007"].iter().enumerate() {
            cache.save_blocking(site, &raw).unwrap();
            // Distinct mtimes so the LRU order is deterministic.
            std::fs::File::options()
                .write(true)
                .open(dir.join(scan_file_name(site)))
                .unwrap()
                .set_modified(std::time::UNIX_EPOCH + Duration::from_secs(1_000 + i as u64))
                .unwrap();
        }
        cache.save_blocking("K001", &raw).unwrap(); // K001 is now newest and protected
        let names: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names.len(), KEEP_SITES);
        assert!(names.contains(&scan_file_name("K001")));
        assert!(!names.contains(&scan_file_name("K002")), "oldest evicted");
    }

    #[test]
    fn disabled_cache_is_a_no_op() {
        let cache = ScanCache::disabled();
        assert!(!cache.is_enabled());
        assert!(cache.save_blocking("KFFC", b"x").is_ok());
        assert!(cache.load_blocking("KFFC").is_none());
    }

    #[test]
    fn remove_legacy_scans_deletes_only_old_scan_blobs() {
        let dir = crate::fsutil::test_dir("legacy");
        for name in ["scan_KFFC", "scan_rle_v2_KTLX", "settings.json", "state_borders_v8"] {
            std::fs::write(dir.join(name), b"x").unwrap();
        }
        assert_eq!(remove_legacy_scans(&dir), 2);
        assert!(dir.join("settings.json").exists());
        assert!(dir.join("state_borders_v8").exists());
    }
```

- [ ] **Step 2: Run the tests to verify they fail.** Run `cargo test -p rustywx --lib scan_cache::`. Expected: compile error `cannot find type 'ScanCache'`.

- [ ] **Step 3: Implement.** Extend the `use` block and add above `#[cfg(test)]`:

```rust
use crate::cache::{bytes_to_scan, scan_to_bytes};
use crate::fsutil;
use crate::model::ScanData;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct ScanCache {
    /// `None` when no cache directory could be created: persistence is off
    /// and every call is a no-op, never a panic (review M4).
    dir: Option<PathBuf>,
}

impl ScanCache {
    /// `cache_root()/scans`, created if needed; sweeps stale temp files.
    pub fn open_default() -> Self {
        match fsutil::cache_root() {
            Some(root) => Self::at(root.join("scans")),
            None => {
                eprintln!("Warning: no cache directory; scan cache disabled");
                Self::disabled()
            }
        }
    }

    pub fn at(dir: PathBuf) -> Self {
        match std::fs::create_dir_all(&dir) {
            Ok(()) => {
                fsutil::sweep_stale_temps(&dir, Duration::from_secs(3600));
                Self { dir: Some(dir) }
            }
            Err(e) => {
                eprintln!("Warning: scan cache disabled ({}): {e}", dir.display());
                Self::disabled()
            }
        }
    }

    pub fn disabled() -> Self {
        Self { dir: None }
    }

    pub fn is_enabled(&self) -> bool {
        self.dir.is_some()
    }

    /// Flatten a scan for [`Self::save_blocking`]. Separate so the data
    /// worker can encode before moving the scan to the UI thread.
    pub fn encode(scan: &ScanData) -> Vec<u8> {
        scan_to_bytes(scan)
    }

    /// Compress `raw` (from [`Self::encode`]), write it atomically, then run
    /// the LRU pass protecting `site`.
    pub fn save_blocking(&self, site: &str, raw: &[u8]) -> io::Result<()> {
        let Some(dir) = &self.dir else {
            return Ok(());
        };
        fsutil::write_atomic(&dir.join(scan_file_name(site)), &crate::rle::compress(raw))?;
        self.evict_blocking(site);
        Ok(())
    }

    /// Load and decode; a missing file is `None`, a corrupt one is logged,
    /// removed, and `None`. A hit refreshes the file's mtime (LRU recency;
    /// atime is off by default on NTFS).
    pub fn load_blocking(&self, site: &str) -> Option<ScanData> {
        let path = self.dir.as_ref()?.join(scan_file_name(site));
        let compressed = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return None,
            Err(e) => {
                eprintln!("Warning: reading {}: {e}", path.display());
                return None;
            }
        };
        match crate::rle::decompress_bounded(&compressed, MAX_DECODED_BYTES)
            .and_then(|raw| bytes_to_scan(&raw))
        {
            Ok(scan) => {
                let _ = std::fs::File::options()
                    .write(true)
                    .open(&path)
                    .and_then(|f| f.set_modified(std::time::SystemTime::now()));
                Some(scan)
            }
            Err(e) => {
                eprintln!("Warning: corrupt cached scan {} ({e}); removing", path.display());
                if let Err(e) = std::fs::remove_file(&path) {
                    eprintln!("Warning: could not remove {}: {e}; will retry next start", path.display());
                }
                None
            }
        }
    }

    /// Apply [`entries_to_evict`] to the directory. Returns files removed.
    /// A locked file (antivirus) is logged and retried next time (review L10).
    pub fn evict_blocking(&self, protect_site: &str) -> usize {
        let Some(dir) = &self.dir else {
            return 0;
        };
        let Ok(read) = std::fs::read_dir(dir) else {
            return 0;
        };
        let entries: Vec<CacheEntry> = read
            .flatten()
            .filter_map(|e| {
                let file_name = e.file_name().to_string_lossy().into_owned();
                site_from_file_name(&file_name)?;
                let meta = e.metadata().ok()?;
                Some(CacheEntry {
                    file_name,
                    bytes: meta.len(),
                    modified: meta.modified().ok()?,
                })
            })
            .collect();
        let mut removed = 0;
        for name in entries_to_evict(&entries, &scan_file_name(protect_site), KEEP_SITES, MAX_TOTAL_BYTES) {
            match std::fs::remove_file(dir.join(&name)) {
                Ok(()) => removed += 1,
                Err(e) => eprintln!("Warning: scan cache eviction of {name} failed: {e}"),
            }
        }
        removed
    }

    /// Non-blocking load for the UI: decode runs on the blocking pool; poll
    /// the receiver with `try_recv()` each frame. Must be called inside the
    /// tokio runtime context (`main` holds `rt.enter()`).
    pub fn load(&self, site: &str) -> tokio::sync::oneshot::Receiver<Option<ScanData>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let cache = self.clone();
        let site = site.to_string();
        tokio::task::spawn_blocking(move || {
            let _ = tx.send(cache.load_blocking(&site));
        });
        rx
    }
}

/// Delete old-format scan blobs from the Ply app-data directory. Run once,
/// off the UI thread, at startup. Returns how many files were removed.
pub fn remove_legacy_scans(app_dir: &Path) -> usize {
    let Ok(read) = std::fs::read_dir(app_dir) else {
        return 0;
    };
    read.flatten()
        .filter(|e| is_legacy_scan_file(&e.file_name().to_string_lossy()))
        .filter(|e| std::fs::remove_file(e.path()).is_ok())
        .count()
}
```

- [ ] **Step 4: Run the tests to verify they pass.** Run `cargo test -p rustywx --lib scan_cache::`. Expected: 11 passed.

- [ ] **Step 5: Commit.** `git commit -am "feat(cache): ScanCache with bounded load, atomic save, and LRU eviction"`

---

### Task 5: Save scans from the data worker, load via `ScanCache`, fix the cache-vs-live race (C1, M16, M1)

**Files:**
- Modify: `app/src/data.rs` (`spawn_worker` around l.227–300)
- Modify: `app/src/state.rs` (new `scan_cache` field)
- Modify: `app/src/main.rs`:
  - startup around l.627–676
  - cache-load poll around l.842–857
  - `NewScan` arm around l.954–983
  - `select_site` around l.4113–4131
- Modify: `app/src/cache.rs` (delete the JSON and compressed scan methods)

**Interfaces:**
- Consumes: `ScanCache` (Task 4), `scan_cache::remove_legacy_scans`, `fsutil::app_data_dir` (Task 1).
- Produces: `data::spawn_worker(tx, initial_site, site_rx, poll_interval, scan_cache: ScanCache)` and `AppState.scan_cache: ScanCache`.

This task is glue; Tasks 1–4 carry the unit tests. Its gate is the build, the smoke test, and the manual checks in Step 6.

- [ ] **Step 1: Save in the data worker.** In `data.rs`, add the parameter `scan_cache: crate::scan_cache::ScanCache` to `spawn_worker` (last position) and to the doc comment. Replace the `Ok(Some(scan)) => { ... }` arm with:

```rust
                Ok(Some(scan)) => {
                    consecutive_errors = 0;
                    last_timestamp = Some(scan.timestamp);
                    // Encode before the scan moves to the UI, then compress and
                    // write after sending, so the UI never waits on serialization
                    // or the disk (review C1). This thread is otherwise idle.
                    let encode_started = std::time::Instant::now();
                    let raw = scan_cache
                        .is_enabled()
                        .then(|| crate::scan_cache::ScanCache::encode(&scan));
                    let encode_ms = encode_started.elapsed().as_millis();
                    let _ = tx.send(WorkerMessage::NewScan {
                        site: site.clone(),
                        scan: Box::new(scan),
                    });
                    if let Some(raw) = raw {
                        let save_started = std::time::Instant::now();
                        match scan_cache.save_blocking(&site, &raw) {
                            Ok(()) => eprintln!(
                                "phase=scan_cache_save site={site} raw_bytes={} encode_ms={encode_ms} save_ms={}",
                                raw.len(),
                                save_started.elapsed().as_millis()
                            ),
                            Err(e) => eprintln!("Warning: scan cache save failed for {site}: {e}"),
                        }
                    }
                    retry_delay(consecutive_errors, healthy)
                }
```

- [ ] **Step 2: Open the cache and clean up legacy files at startup.** In `main.rs`, right after `let cache = Cache::new().await...;`, add:

```rust
    // Compressed, LRU-bounded scan cache in the OS cache dir (review C1).
    let scan_cache = rustywx::scan_cache::ScanCache::open_default();
    // One-time cleanup of the old JSON / RLE scan blobs in the Roaming app-data
    // dir (multi-GB on long-time installs). Off the UI thread.
    if let Some(app_dir) = rustywx::fsutil::app_data_dir() {
        tokio::task::spawn_blocking(move || {
            let removed = rustywx::scan_cache::remove_legacy_scans(&app_dir);
            if removed > 0 {
                eprintln!("phase=legacy_scan_cleanup removed={removed}");
            }
        });
    }
```

Pass `scan_cache.clone()` as the new last argument of `data::spawn_worker(...)`. Replace `Some(cache.load_scan(&initial_site))` with `Some(scan_cache.load(&initial_site))`. In `state.rs`, add after `pub cache: Cache,`:

```rust
    /// Compressed scan cache (OS cache dir). Loads go through
    /// `ScanCache::load`; saves happen in the data worker.
    pub scan_cache: crate::scan_cache::ScanCache,
```

and add `scan_cache,` to the `AppState { .. }` initializer in `main.rs`.

- [ ] **Step 3: Remove the UI-thread save and fix the race (M1).** In the `WorkerMessage::NewScan` arm, delete the line `state.cache.save_scan(&current_site, &scan);` and its comment. Then add, before `state.scan = Some(*scan);`:

```rust
                    // A live scan supersedes any cache load still in flight; a
                    // late cache result must never replace newer data (review M1).
                    state.pending_load = None;
```

In the `// ── Poll cache load` block, change the success branch so it applies only when nothing newer has arrived:

```rust
            if let Some(scan) = cached
                && state.scan.is_none()
            {
```

The `else` branch that sets "Waiting for data…" stays, but should run only when `state.scan.is_none()`. Wrap it as `} else if state.scan.is_none() {`.

- [ ] **Step 4: Site switches.** In `select_site`, replace `state.pending_load = Some(state.cache.load_scan(&site_id));` with `state.pending_load = Some(state.scan_cache.load(&site_id));`. Each load now runs on the blocking pool and replaces the previous receiver, and a superseded load's result is dropped. A 250 ms debounce for rapid site switching is Follow-up F3.

- [ ] **Step 5: Delete the dead scan methods.** In `cache.rs`, delete `scan_key`, `save_scan`, `load_scan`, `scan_key_rle`, `save_scan_compressed`, `load_scan_compressed`, and `remove_scan`, plus any test that only exercises them. Keep `scan_to_bytes`/`bytes_to_scan` and their tests. Run `cargo clippy --all-targets --all-features -- -D warnings`; it must be clean, with no unused imports.

- [ ] **Step 6: Verify.**

Run: `cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings`. Expected: pass.

Smoke: `just run & sleep 3; kill %1 2>/dev/null || true`. It must launch without a panic.

Manual check (one longer run): wait for the first volume, then quit. Expect:
- a `phase=scan_cache_save ... save_ms=` line on stderr;
- a `scan_v2_KFFC.bin` in `~/Library/Caches/rustywx/scans/` (macOS) or `~/.cache/rustywx/scans/` (Linux);
- the old `scan_*` files gone from the app-data dir: `ls ~/Library/Application\ Support/rustywx/`.

Relaunch: the status shows ` [cached]` before the live scan arrives. Record the logged `raw_bytes`, the file size, and `save_ms` in the PR description: that's the real compression ratio and the cost moved off the UI thread.

**[verify on Windows]:** the files land in `%LOCALAPPDATA%\rustywx\scans`, and `%APPDATA%\rustywx` loses its `scan_*` files (Appendix B, item 1).

- [ ] **Step 7: Commit.** `git commit -am "fix(cache): save scans off the UI thread into a bounded compressed cache"`

---

### Task 6: Single settings writer with atomic write and `.bak`; atomic borders save (M3, L10)

**Files:**
- Modify: `app/src/cache.rs` (`Cache` struct, `new`, `save_settings`, `load_settings`; new free functions)
- Modify: `app/src/borders.rs` (`save_cached`, around l.132)
- Modify: `app/src/main.rs` (the `borders::save_cached` call around l.1056)

**Interfaces:**
- Consumes: `fsutil::{app_data_dir, write_atomic, test_dir}`.
- Produces:
  - `Cache::save_settings(&self, &Settings)` (same signature; now debounced and atomic)
  - `Cache::save_blob_atomic(&self, key: &str, bytes: Vec<u8>)`
  - `pub(crate) fn write_settings_file(path: &Path, s: &Settings) -> io::Result<()>`
  - `pub(crate) fn read_settings_file(path: &Path) -> Option<Settings>`
  - `borders::save_cached(cache: &Cache, rings: &[Ring])`

- [ ] **Step 1: Write the failing tests.** Add to `mod tests` in `cache.rs`:

```rust
    use super::{read_settings_file, write_settings_file};
    use crate::settings::Settings;

    #[test]
    fn settings_write_keeps_a_backup_of_the_previous_copy() {
        let dir = crate::fsutil::test_dir("settings-bak");
        let path = dir.join("settings.json");
        let first = Settings::default();
        let second = Settings { default_site: "KTLX".into(), ..Settings::default() };
        write_settings_file(&path, &first).unwrap();
        write_settings_file(&path, &second).unwrap();
        assert_eq!(read_settings_file(&path).unwrap().default_site, "KTLX");
        let bak = std::fs::read(dir.join("settings.json.bak")).unwrap();
        assert_eq!(serde_json::from_slice::<Settings>(&bak).unwrap().default_site, "KFFC");
    }

    #[test]
    fn corrupt_settings_fall_back_to_backup_and_are_kept_aside() {
        let dir = crate::fsutil::test_dir("settings-corrupt");
        let path = dir.join("settings.json");
        let good = Settings { default_site: "KTLX".into(), ..Settings::default() };
        std::fs::write(dir.join("settings.json.bak"), serde_json::to_vec(&good).unwrap()).unwrap();
        std::fs::write(&path, b"{ torn").unwrap();
        assert_eq!(read_settings_file(&path).unwrap().default_site, "KTLX");
        assert!(!path.exists(), "corrupt file moved aside, not left in place");
        let kept_aside = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .any(|e| e.file_name().to_string_lossy().starts_with("settings.json.corrupt-"));
        assert!(kept_aside, "corrupt copy kept for diagnosis, never deleted");
    }

    #[test]
    fn missing_settings_and_backup_is_none() {
        let dir = crate::fsutil::test_dir("settings-missing");
        assert!(read_settings_file(&dir.join("settings.json")).is_none());
    }
```

- [ ] **Step 2: Run the tests to verify they fail.** Run `cargo test -p rustywx --lib cache::`. Expected: compile error `cannot find function 'write_settings_file'`.

- [ ] **Step 3: Implement the file functions.** Add to `cache.rs` (outside `impl Cache`):

```rust
use std::path::{Path, PathBuf};

/// Debounce for the settings writer: coalesces stepper-click bursts into
/// one write. A quit within this window loses only that last change.
const SETTINGS_DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(400);

/// Serialize and atomically replace `path`, first copying the current file
/// to `settings.json.bak`. Blocking.
pub(crate) fn write_settings_file(path: &Path, settings: &crate::settings::Settings) -> std::io::Result<()> {
    let json = serde_json::to_vec_pretty(settings).map_err(std::io::Error::other)?;
    if path.exists() {
        let _ = std::fs::copy(path, path.with_extension("json.bak"));
    }
    crate::fsutil::write_atomic(path, &json)
}

/// Read settings, falling back to `.bak`. A corrupt primary is renamed to
/// `settings.json.corrupt-<unix-ts>`, never deleted (review M3). Blocking.
pub(crate) fn read_settings_file(path: &Path) -> Option<crate::settings::Settings> {
    match std::fs::read(path) {
        Ok(bytes) => match serde_json::from_slice(&bytes) {
            Ok(settings) => return Some(settings),
            Err(e) => {
                let aside = path.with_extension(format!("json.corrupt-{}", chrono::Utc::now().timestamp()));
                eprintln!("Warning: corrupt settings.json ({e}); moved to {}", aside.display());
                let _ = std::fs::rename(path, &aside);
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => eprintln!("Warning: reading {}: {e}", path.display()),
    }
    let bak = std::fs::read(path.with_extension("json.bak")).ok()?;
    serde_json::from_slice(&bak).ok()
}

/// The single settings writer: waits for a change, debounces, writes the
/// latest snapshot on the blocking pool. Serializes all writes, so a stale
/// snapshot can never land last (review M3).
async fn settings_writer(
    path: PathBuf,
    mut rx: tokio::sync::watch::Receiver<Option<crate::settings::Settings>>,
) {
    while rx.changed().await.is_ok() {
        tokio::time::sleep(SETTINGS_DEBOUNCE).await;
        let Some(settings) = rx.borrow_and_update().clone() else {
            continue;
        };
        let path = path.clone();
        match tokio::task::spawn_blocking(move || write_settings_file(&path, &settings)).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => eprintln!("Warning: saving settings failed: {e}"),
            Err(e) => eprintln!("Warning: settings writer task failed: {e}"),
        }
    }
}
```

- [ ] **Step 4: Rewire `Cache`.** Change the struct and `new`:

```rust
#[derive(Clone)]
pub struct Cache {
    storage: Storage,
    /// Same directory as `storage`'s root (see `fsutil::app_data_dir`).
    app_dir: Option<PathBuf>,
    settings_tx: std::sync::Arc<tokio::sync::watch::Sender<Option<crate::settings::Settings>>>,
}

impl Cache {
    pub async fn new() -> Result<Self, String> {
        let storage = Storage::new("rustywx").await?;
        let app_dir = crate::fsutil::app_data_dir();
        let (settings_tx, settings_rx) = tokio::sync::watch::channel(None);
        if let Some(dir) = &app_dir {
            tokio::spawn(settings_writer(dir.join(Self::SETTINGS_KEY), settings_rx));
        }
        Ok(Cache {
            storage,
            app_dir,
            settings_tx: std::sync::Arc::new(settings_tx),
        })
    }
```

Replace the body of `save_settings` with:

```rust
        // Hand the latest snapshot to the single writer task; never blocks.
        self.settings_tx.send_replace(Some(settings.clone()));
```

Replace the body of `load_settings` (keep its signature) with:

```rust
        let (tx, rx) = oneshot::channel();
        let path = self.app_dir.as_ref().map(|d| d.join(Self::SETTINGS_KEY));
        tokio::task::spawn_blocking(move || {
            let _ = tx.send(path.as_deref().and_then(read_settings_file));
        });
        rx
```

Add:

```rust
    /// Atomically write a small blob into the app-data dir (key = file name),
    /// on the blocking pool. Replaces Ply `Storage::save_bytes`, which is a
    /// bare `fs::write` (review M3).
    pub fn save_blob_atomic(&self, key: &str, bytes: Vec<u8>) {
        let Some(path) = self.app_dir.as_ref().map(|d| d.join(key)) else {
            return;
        };
        tokio::task::spawn_blocking(move || {
            if let Err(e) = crate::fsutil::write_atomic(&path, &bytes) {
                eprintln!("Warning: saving {} failed: {e}", path.display());
            }
        });
    }
```

- [ ] **Step 5: Borders.** In `borders.rs`, replace `save_cached` with:

```rust
/// Save merged border rings atomically (serialize and write on the blocking pool).
pub fn save_cached(cache: &crate::cache::Cache, rings: &[Ring]) {
    let cache = cache.clone();
    let rings = rings.to_vec();
    tokio::task::spawn_blocking(move || {
        if let Ok(json) = serde_json::to_vec(&rings) {
            cache.save_blob_atomic(STORAGE_KEY, json);
        }
    });
}
```

In `main.rs`, change `borders::save_cached(&state.cache.storage(), &rings);` to `borders::save_cached(&state.cache, &rings);`. `save_blob_atomic` calls `spawn_blocking` from inside a blocking task, which is fine: it's inside the runtime context.

- [ ] **Step 6: Verify.**

Run: `cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings`. Expected: pass.

Smoke test. Manual check:
1. Open Settings, click a stepper 10 times quickly, and quit after one second.
2. `settings.json` should have the final value, a `settings.json.bak` should exist, and there should be no `.tmp` files left.
3. Type garbage into `settings.json` and relaunch. The settings load from `.bak` and the garbage is kept as `settings.json.corrupt-*`.

**[verify on Windows]** (Appendix B, item 2): hold `settings.json` open with a share mode that blocks delete, then change a setting. The log shows the retry and then a warning, with no crash and no torn file.

- [ ] **Step 7: Commit.** `git commit -am "fix(settings): serialize settings writes through one debounced atomic writer"`


---

# Phase 1b — Network feeds recover on their own (PR `fix/net-recovery`)

**Prerequisite:** `feat/spaghetti-models` is merged into `main`. This phase moves that branch's `NetPort` and `PlyNet` and reuses `BackgroundWorkers::parse_queue_has_capacity()`.

Closes C2, and C3 for every Ply-backed feed, plus H6, M22, the forecast half of M23, and the H1 stopgap. After this PR, every feed:
- polls its id every frame;
- `cancel()`s after each result;
- gives up at an app-side deadline;
- backs off with jitter;
- toasts once per failure streak.

This is the research's "Safe retry pattern" (section A), packaged as `net_feed::Feed`. The radar data worker (`data.rs`, nexrad-data's reqwest client) is out of scope here; see Follow-up F4.

### Task 7: Move `NetPort` / `PlyNet` to a shared `net_port` module

**Files:**
- Create: `app/src/net_port.rs`
- Modify: `app/src/adeck_fetch.rs` (delete the moved items, add re-exports)
- Modify: `app/src/lib.rs` (`pub mod net_port;`)

**Interfaces:**
- Produces:
  - `pub enum NetPoll { Pending, Done(Result<Vec<u8>, String>), Missing }`
  - `pub trait NetPort { fn get(&mut self, id: &str, url: &str); fn get_with_headers(&mut self, id: &str, url: &str, headers: &[(&str, &str)]) { .. default calls get .. } fn poll(&mut self, id: &str) -> NetPoll; fn cancel(&mut self, id: &str); }`
  - `pub struct PlyNet;` and `pub const DEFAULT_USER_AGENT: &str = "rustywx/0.3"`
  - `#[cfg(test)] pub(crate) mod testing { pub struct FakeNet { .. } }` with `respond`, `evict`, `respond_all_live`
  - `adeck_fetch` keeps `pub use crate::net_port::{NetPoll, NetPort, PlyNet};` so spaghetti code and `main.rs` compile unchanged.

- [ ] **Step 1: Write the failing tests.** Create `app/src/net_port.rs`:

```rust
//! Transport boundary for every network feed. `PlyNet` is the only code that
//! talks to ply-engine `net`; the planned app-owned HTTP module (Phase 5 of
//! docs/plans/windows-stability-remediation.md) replaces it by implementing
//! `NetPort`. Moved here from `adeck_fetch` (spaghetti-models feature).
//!
//! Contract (ply 1.1.1 semantics, research section A): `get` on an id that
//! still has an entry is ignored; `cancel` removes the entry and frees the id.

#[cfg(test)]
mod tests {
    use super::testing::FakeNet;
    use super::*;

    #[test]
    fn fake_net_models_ply_id_semantics() {
        let mut net = FakeNet::default();
        net.get("a", "https://x/1");
        net.get("a", "https://x/2"); // still live: ignored, like ply
        assert_eq!(net.fired.len(), 1);
        assert_eq!(net.ignored, ["a"]);
        assert!(matches!(net.poll("a"), NetPoll::Pending));
        net.respond("a", NetPoll::Done(Ok(vec![7])));
        assert!(matches!(net.poll("a"), NetPoll::Done(Ok(ref b)) if b == &[7]));
        net.cancel("a");
        assert!(matches!(net.poll("a"), NetPoll::Missing));
        net.get("a", "https://x/3"); // freed by cancel: fires
        assert_eq!(net.fired.len(), 2);
    }

    #[test]
    fn get_with_headers_records_headers_in_the_fake() {
        let mut net = FakeNet::default();
        net.get_with_headers("b", "https://x", &[("Accept", "application/geo+json")]);
        assert_eq!(net.headers_for("b"), [("Accept".to_owned(), "application/geo+json".to_owned())]);
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail.** Add `pub mod net_port;` to `lib.rs` and run `cargo test -p rustywx --lib net_port::`. Expected: compile errors (`testing`, `NetPoll` missing).

- [ ] **Step 3: Implement.** Cut `NetPoll`, `NetPort`, and `PlyNet` (with its `impl`) out of `app/src/adeck_fetch.rs` and paste them above `#[cfg(test)]` in `net_port.rs`. Then make these changes so the result reads:

```rust
/// Default User-Agent for NOAA endpoints (matches the existing NHC requests).
pub const DEFAULT_USER_AGENT: &str = "rustywx/0.3";

/// Result of polling one request id.
pub enum NetPoll {
    Pending,
    Done(Result<Vec<u8>, String>),
    /// No such id: Ply evicted it or it was never fired.
    Missing,
}

/// Transport boundary for network feeds. After `cancel`, the id is free and
/// a new `get` with it starts a fresh request.
pub trait NetPort {
    fn get(&mut self, id: &str, url: &str);
    /// `get` with extra request headers. The default ignores them (fine for
    /// test fakes); `PlyNet` sends them.
    fn get_with_headers(&mut self, id: &str, url: &str, headers: &[(&str, &str)]) {
        let _ = headers;
        self.get(id, url);
    }
    fn poll(&mut self, id: &str) -> NetPoll;
    fn cancel(&mut self, id: &str);
}

/// `NetPort` over ply-engine 1.1 `net`.
pub struct PlyNet;

impl NetPort for PlyNet {
    fn get(&mut self, id: &str, url: &str) {
        self.get_with_headers(id, url, &[("User-Agent", DEFAULT_USER_AGENT)]);
    }

    fn get_with_headers(&mut self, id: &str, url: &str, headers: &[(&str, &str)]) {
        use ply_engine::prelude::net;
        // Callers fire only from an idle state, so any entry still under this
        // id is stale; free it, or ply would silently ignore this `get`.
        if let Some(req) = net::request(id) {
            req.cancel();
        }
        net::get(id, url, |c| {
            for (k, v) in headers {
                c.header(k, v);
            }
            c
        });
    }

    fn poll(&mut self, id: &str) -> NetPoll {
        use ply_engine::prelude::net;
        match net::request(id) {
            None => NetPoll::Missing,
            Some(req) => match req.response() {
                None => NetPoll::Pending,
                Some(Ok(resp)) => NetPoll::Done(Ok(resp.bytes().to_vec())),
                Some(Err(e)) => NetPoll::Done(Err(e)),
            },
        }
    }

    /// Removes ply's entry and frees the id. A hung ureq thread keeps running
    /// until the socket dies, but its result goes to a dropped channel.
    fn cancel(&mut self, id: &str) {
        use ply_engine::prelude::net;
        if let Some(req) = net::request(id) {
            req.cancel();
        }
    }
}

/// Scripted stand-in for ply net with its real id semantics, shared by every
/// module's tests.
#[cfg(test)]
pub(crate) mod testing {
    use super::{NetPoll, NetPort};
    use std::collections::{HashMap, HashSet};

    #[derive(Default)]
    pub struct FakeNet {
        /// Ids with an entry (pending or finished, not yet cancelled).
        pub live: HashSet<String>,
        pub fired: Vec<(String, String)>,
        /// `get` calls ply would have ignored. Tests assert this stays empty.
        pub ignored: Vec<String>,
        pub responses: HashMap<String, NetPoll>,
        pub cancelled: Vec<String>,
        pub headers: Vec<(String, Vec<(String, String)>)>,
    }

    impl FakeNet {
        pub fn respond(&mut self, id: &str, poll: NetPoll) {
            self.responses.insert(id.to_owned(), poll);
        }
        pub fn evict(&mut self, id: &str) {
            self.live.remove(id);
        }
        /// Answer every live request without a scripted response.
        pub fn respond_all_live(&mut self, make: impl Fn(&str) -> NetPoll) {
            let ids: Vec<String> = self.live.iter().cloned().collect();
            for id in ids {
                self.responses.entry(id.clone()).or_insert_with(|| make(&id));
            }
        }
        pub fn headers_for(&self, id: &str) -> Vec<(String, String)> {
            self.headers
                .iter()
                .rev()
                .find(|(i, _)| i == id)
                .map(|(_, h)| h.clone())
                .unwrap_or_default()
        }
    }

    impl NetPort for FakeNet {
        fn get(&mut self, id: &str, url: &str) {
            self.get_with_headers(id, url, &[]);
        }
        fn get_with_headers(&mut self, id: &str, url: &str, headers: &[(&str, &str)]) {
            if !self.live.insert(id.to_owned()) {
                self.ignored.push(id.to_owned());
                return;
            }
            self.fired.push((id.to_owned(), url.to_owned()));
            self.headers.push((
                id.to_owned(),
                headers.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect(),
            ));
        }
        fn poll(&mut self, id: &str) -> NetPoll {
            if !self.live.contains(id) {
                return NetPoll::Missing;
            }
            self.responses.remove(id).unwrap_or(NetPoll::Pending)
        }
        fn cancel(&mut self, id: &str) {
            self.live.remove(id);
            self.cancelled.push(id.to_owned());
        }
    }
}
```

In `adeck_fetch.rs`, after its module doc, add `pub use crate::net_port::{NetPoll, NetPort, PlyNet};`, and delete its now-unused `use ply_engine...` lines if any. Leave its own test `FakeNet` alone.

- [ ] **Step 4: Run all tests.** Run `cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings`. Expected: everything passes, including all 13 `adeck_fetch::` tests unchanged.

- [ ] **Step 5: Commit.** `git commit -am "refactor(net): move NetPort and PlyNet to a shared net_port module"`

---

### Task 8: `net_feed` — retry policy, retry clock, single-request `Feed`

**Files:**
- Create: `app/src/net_feed.rs`
- Modify: `app/src/lib.rs` (`pub mod net_feed;`)

**Interfaces:**
- Consumes: `net_port::{NetPoll, NetPort, testing::FakeNet}` (Task 7).
- Produces:
  - `#[derive(Clone, Copy, Debug, PartialEq)] pub struct RetryPolicy { pub first_retry_secs: f64, pub max_retry_secs: f64, pub timeout_secs: f64 }`, with `backoff_secs(&self, failures: u32, seed: u64) -> f64`
  - `#[derive(Clone, Debug)] pub struct RetryClock`, with:
    - `new(policy, label: &str)`
    - `is_due(now) -> bool`
    - `failures() -> u32`
    - `succeeded(now, refresh_secs: Option<f64>)`
    - `failed(now) -> bool` (true means toast)
    - `failed_quietly(now)`
    - `retry_in_secs(now) -> Option<u32>`
    - `make_due_now(now)`
    - `policy() -> RetryPolicy`
  - `#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)] pub struct FeedTick { pub show_toast: bool }`
  - `#[derive(Debug)] pub struct Feed`, with:
    - `new(id: impl Into<String>, policy)`
    - `id() -> &str`
    - `clock() -> &RetryClock`
    - `tick(&mut self, now: f64, url: &str, headers: &[(&str, &str)], net: &mut impl NetPort) -> FeedTick`
    - `has_body() -> bool`
    - `take_body() -> Option<Vec<u8>>`
    - `parse_finished(&mut self, ok: bool, now: f64, refresh_secs: Option<f64>) -> bool` (true means toast)
    - `is_busy() -> bool`
    - `cancel(&mut self, net: &mut impl NetPort)`
    - `retry_in_secs(now) -> Option<u32>`

`now` is `macroquad::get_time()` seconds, as everywhere in `main.rs`.

- [ ] **Step 1: Write the failing tests.** Create `app/src/net_feed.rs`:

```rust
//! Retry, deadline, and toast-once logic for one network feed (review C2,
//! C3). Implements the research's "Safe retry pattern" (section A): one
//! stable id per feed; `cancel()` after every result and on the deadline, so
//! the id is free for the retry and a cached `Err` is never re-read;
//! exponential backoff with jitter; a toast only on the first failure of a
//! streak. Mirrors `adeck_fetch::AdeckFetcher`, which does the same per storm.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net_port::testing::FakeNet;

    const POLICY: RetryPolicy = RetryPolicy {
        first_retry_secs: 5.0,
        max_retry_secs: 120.0,
        timeout_secs: 30.0,
    };

    fn feed() -> Feed {
        Feed::new("alerts", POLICY)
    }

    #[test]
    fn backoff_doubles_caps_and_jitters_within_20_percent() {
        for f in 1..=10u32 {
            let base = (5.0 * 2f64.powi(f as i32 - 1)).min(120.0);
            let b = POLICY.backoff_secs(f, 42);
            assert!(b >= base * 0.8 && b < base * 1.2, "failures={f} backoff={b}");
        }
        assert_eq!(POLICY.backoff_secs(0, 42), 0.0);
        assert_eq!(POLICY.backoff_secs(3, 7), POLICY.backoff_secs(3, 7), "deterministic");
        let spread: std::collections::HashSet<u64> =
            (0..8u64).map(|s| POLICY.backoff_secs(2, s).to_bits()).collect();
        assert!(spread.len() > 1, "different feeds get different jitter");
    }

    #[test]
    fn fires_when_due_with_headers_and_polls_until_done() {
        let mut f = feed();
        let mut net = FakeNet::default();
        f.tick(0.0, "https://x/alerts", &[("Accept", "application/geo+json")], &mut net);
        assert_eq!(net.fired, [("alerts".to_owned(), "https://x/alerts".to_owned())]);
        assert_eq!(net.headers_for("alerts").len(), 1);
        f.tick(1.0, "https://x/alerts", &[], &mut net);
        assert_eq!(net.fired.len(), 1, "no re-fire while in flight");
        assert!(f.is_busy() && !f.has_body());
    }

    #[test]
    fn success_cancels_entry_and_hands_over_body() {
        let mut f = feed();
        let mut net = FakeNet::default();
        f.tick(0.0, "u", &[], &mut net);
        net.respond("alerts", NetPoll::Done(Ok(b"{}".to_vec())));
        f.tick(1.0, "u", &[], &mut net);
        assert_eq!(net.cancelled, ["alerts"]);
        assert_eq!(f.take_body().as_deref(), Some(&b"{}"[..]));
        assert!(f.take_body().is_none());
        assert!(!f.parse_finished(true, 2.0, Some(120.0)));
        f.tick(121.0, "u", &[], &mut net);
        assert_eq!(net.fired.len(), 1, "not due before refresh");
        f.tick(122.0, "u", &[], &mut net);
        assert_eq!(net.fired.len(), 2);
        assert!(net.ignored.is_empty());
    }

    #[test]
    fn failed_request_is_cancelled_and_retried_after_backoff() {
        // The C2 trace: launch offline, then the network comes back.
        let mut f = feed();
        let mut net = FakeNet::default();
        f.tick(0.0, "u", &[], &mut net);
        net.respond("alerts", NetPoll::Done(Err("dns error".into())));
        assert!(f.tick(1.0, "u", &[], &mut net).show_toast);
        assert_eq!(net.cancelled, ["alerts"], "cached Err released, not re-read");
        let wait = f.retry_in_secs(1.0).unwrap();
        assert!((4..=6).contains(&wait), "{wait}");
        f.tick(1.0 + f64::from(wait), "u", &[], &mut net);
        assert_eq!(net.fired.len(), 2, "retry actually fires");
        assert!(net.ignored.is_empty());
        net.respond("alerts", NetPoll::Done(Ok(vec![1])));
        f.tick(10.0, "u", &[], &mut net);
        assert!(f.has_body(), "recovered");
    }

    #[test]
    fn toast_only_on_first_failure_of_streak() {
        let mut f = feed();
        let mut net = FakeNet::default();
        let mut now = 0.0;
        let mut toasts = 0;
        for _ in 0..4 {
            f.tick(now, "u", &[], &mut net);
            net.respond("alerts", NetPoll::Done(Err("offline".into())));
            now += 0.1;
            if f.tick(now, "u", &[], &mut net).show_toast {
                toasts += 1;
            }
            now += 200.0;
        }
        assert_eq!(toasts, 1);
        // A success ends the streak, so the next failure toasts again.
        f.tick(now, "u", &[], &mut net);
        net.respond("alerts", NetPoll::Done(Ok(vec![1])));
        f.tick(now + 0.1, "u", &[], &mut net);
        f.take_body();
        f.parse_finished(true, now + 0.2, Some(1.0));
        f.tick(now + 2.0, "u", &[], &mut net);
        net.respond("alerts", NetPoll::Done(Err("offline".into())));
        assert!(f.tick(now + 2.1, "u", &[], &mut net).show_toast);
    }

    #[test]
    fn deadline_cancels_hung_request() {
        let mut f = feed();
        let mut net = FakeNet::default();
        f.tick(0.0, "u", &[], &mut net);
        f.tick(29.9, "u", &[], &mut net);
        assert!(net.cancelled.is_empty());
        assert!(f.tick(30.0, "u", &[], &mut net).show_toast);
        assert_eq!(net.cancelled, ["alerts"]);
        assert!(!f.is_busy());
    }

    #[test]
    fn not_found_backs_off_quietly() {
        let mut f = feed();
        let mut net = FakeNet::default();
        f.tick(0.0, "u", &[], &mut net);
        net.respond("alerts", NetPoll::Done(Err("http status: 404".into())));
        assert!(!f.tick(1.0, "u", &[], &mut net).show_toast);
        assert!(f.retry_in_secs(1.0).is_some());
    }

    #[test]
    fn evicted_entry_counts_as_failure() {
        let mut f = feed();
        let mut net = FakeNet::default();
        f.tick(0.0, "u", &[], &mut net);
        net.evict("alerts");
        f.tick(1.0, "u", &[], &mut net);
        assert!(f.retry_in_secs(1.0).is_some());
    }

    #[test]
    fn parse_failure_backs_off_then_redownloads() {
        let mut f = feed();
        let mut net = FakeNet::default();
        f.tick(0.0, "u", &[], &mut net);
        net.respond("alerts", NetPoll::Done(Ok(b"garbage".to_vec())));
        f.tick(1.0, "u", &[], &mut net);
        f.take_body().unwrap();
        assert!(f.parse_finished(false, 1.0, Some(120.0)), "first failure toasts");
        let wait = f.retry_in_secs(1.0).unwrap();
        f.tick(1.0 + f64::from(wait), "u", &[], &mut net);
        assert_eq!(net.fired.len(), 2);
    }

    #[test]
    fn one_shot_feed_never_refires_after_success() {
        let mut f = feed();
        let mut net = FakeNet::default();
        f.tick(0.0, "u", &[], &mut net);
        net.respond("alerts", NetPoll::Done(Ok(vec![1])));
        f.tick(1.0, "u", &[], &mut net);
        f.take_body();
        f.parse_finished(true, 1.0, None);
        f.tick(1.0e9, "u", &[], &mut net);
        assert_eq!(net.fired.len(), 1);
    }

    #[test]
    fn cancel_releases_in_flight_request() {
        let mut f = feed();
        let mut net = FakeNet::default();
        f.tick(0.0, "u", &[], &mut net);
        f.cancel(&mut net);
        assert_eq!(net.cancelled, ["alerts"]);
        assert!(!f.is_busy());
    }

    #[test]
    fn retry_clock_streak_semantics() {
        let mut c = RetryClock::new(POLICY, "nhc");
        assert!(c.is_due(0.0));
        assert!(c.failed(0.0));
        assert!(!c.failed(10.0));
        assert_eq!(c.failures(), 2);
        assert!(!c.is_due(10.0));
        c.succeeded(20.0, Some(300.0));
        assert_eq!(c.failures(), 0);
        assert!(!c.is_due(319.0) && c.is_due(320.0));
        assert!(c.failed(320.0), "new streak toasts again");
        c.make_due_now(321.0);
        assert!(c.is_due(321.0));
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail.** Add `pub mod net_feed;` to `lib.rs` and run `cargo test -p rustywx --lib net_feed::`. Expected: compile error `cannot find type 'RetryPolicy'`.

- [ ] **Step 3: Implement.** Insert above `#[cfg(test)]`:

```rust
use crate::net_port::{NetPoll, NetPort};

/// Backoff and deadline settings for one feed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RetryPolicy {
    /// Delay after the first failure; doubles per consecutive failure.
    pub first_retry_secs: f64,
    /// Cap on the delay (normally the feed's refresh interval).
    pub max_retry_secs: f64,
    /// App-side deadline for one request (ply has no HTTP timeout, C3).
    pub timeout_secs: f64,
}

impl RetryPolicy {
    /// Delay before the next attempt after `failures` consecutive failures,
    /// with a deterministic ±20% jitter so feeds that failed together don't
    /// retry in lockstep.
    pub fn backoff_secs(&self, failures: u32, seed: u64) -> f64 {
        if failures == 0 {
            return 0.0;
        }
        let exp = (failures - 1).min(16) as i32;
        let base = (self.first_retry_secs * 2f64.powi(exp)).min(self.max_retry_secs);
        base * jitter(seed, failures)
    }
}

/// Deterministic factor in [0.8, 1.2).
fn jitter(seed: u64, failures: u32) -> f64 {
    let mut h = seed ^ u64::from(failures).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h ^= h >> 33;
    0.8 + 0.4 * ((h % 10_000) as f64 / 10_000.0)
}

fn seed_of(label: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    label.hash(&mut h);
    h.finish()
}

/// When a feed may fire next, and whether a failure should toast.
#[derive(Clone, Debug)]
pub struct RetryClock {
    policy: RetryPolicy,
    seed: u64,
    failures: u32,
    next_due: f64,
    toast_shown: bool,
}

impl RetryClock {
    pub fn new(policy: RetryPolicy, label: &str) -> Self {
        Self {
            policy,
            seed: seed_of(label),
            failures: 0,
            next_due: 0.0,
            toast_shown: false,
        }
    }

    pub fn policy(&self) -> RetryPolicy {
        self.policy
    }

    pub fn is_due(&self, now: f64) -> bool {
        now >= self.next_due
    }

    pub fn failures(&self) -> u32 {
        self.failures
    }

    /// End a failure streak. `refresh_secs = None` for one-shot feeds, which
    /// are then never due again.
    pub fn succeeded(&mut self, now: f64, refresh_secs: Option<f64>) {
        self.failures = 0;
        self.toast_shown = false;
        self.next_due = refresh_secs.map_or(f64::INFINITY, |r| now + r);
    }

    /// Record a failure and back off. True only for the first failure of a
    /// streak: the caller shows the toast then, not every frame (C2).
    pub fn failed(&mut self, now: f64) -> bool {
        self.record_failure(now);
        !std::mem::replace(&mut self.toast_shown, true)
    }

    /// A failure that should never toast (HTTP 404: not published yet).
    pub fn failed_quietly(&mut self, now: f64) {
        self.record_failure(now);
    }

    fn record_failure(&mut self, now: f64) {
        self.failures = self.failures.saturating_add(1);
        self.next_due = now + self.policy.backoff_secs(self.failures, self.seed);
    }

    /// Seconds until the next retry while failing, else `None`.
    pub fn retry_in_secs(&self, now: f64) -> Option<u32> {
        (self.failures > 0).then(|| (self.next_due - now).max(0.0).ceil() as u32)
    }

    pub fn make_due_now(&mut self, now: f64) {
        self.next_due = now;
    }
}

#[derive(Debug)]
enum Phase {
    Idle,
    InFlight { started_at: f64 },
    Downloaded(Vec<u8>),
    Parsing,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FeedTick {
    /// Caller should `show_toast(.., ErrorKind::Network)`.
    pub show_toast: bool,
}

/// One request id with its retry clock. Call [`Feed::tick`] every frame; it
/// fires when due, polls while in flight, and cancels on every outcome.
#[derive(Debug)]
pub struct Feed {
    id: String,
    clock: RetryClock,
    phase: Phase,
}

impl Feed {
    pub fn new(id: impl Into<String>, policy: RetryPolicy) -> Self {
        let id = id.into();
        let clock = RetryClock::new(policy, &id);
        Self {
            id,
            clock,
            phase: Phase::Idle,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn clock(&self) -> &RetryClock {
        &self.clock
    }

    pub fn tick(
        &mut self,
        now: f64,
        url: &str,
        headers: &[(&str, &str)],
        net: &mut impl NetPort,
    ) -> FeedTick {
        match self.phase {
            Phase::Idle if self.clock.is_due(now) => {
                net.get_with_headers(&self.id, url, headers);
                self.phase = Phase::InFlight { started_at: now };
                FeedTick::default()
            }
            Phase::InFlight { started_at } => {
                let timeout = self.clock.policy().timeout_secs;
                let outcome = if now - started_at >= timeout {
                    net.cancel(&self.id);
                    Some(Err(format!("timed out after {timeout} s")))
                } else {
                    match net.poll(&self.id) {
                        NetPoll::Pending => None,
                        NetPoll::Done(result) => {
                            net.cancel(&self.id);
                            Some(result)
                        }
                        NetPoll::Missing => Some(Err("request handle evicted".to_owned())),
                    }
                };
                match outcome {
                    None => FeedTick::default(),
                    Some(Ok(body)) => {
                        self.phase = Phase::Downloaded(body);
                        FeedTick::default()
                    }
                    Some(Err(e)) => {
                        eprintln!("Warning: fetch {} failed: {e}", self.id);
                        self.phase = Phase::Idle;
                        if e.contains("404") {
                            self.clock.failed_quietly(now);
                            FeedTick::default()
                        } else {
                            FeedTick {
                                show_toast: self.clock.failed(now),
                            }
                        }
                    }
                }
            }
            _ => FeedTick::default(),
        }
    }

    pub fn has_body(&self) -> bool {
        matches!(self.phase, Phase::Downloaded(_))
    }

    /// Hand the downloaded body to a parser. Call only when it can be
    /// submitted this frame (check `parse_queue_has_capacity` first), so a
    /// downloaded body is never thrown away (C2 fix 3).
    pub fn take_body(&mut self) -> Option<Vec<u8>> {
        match std::mem::replace(&mut self.phase, Phase::Parsing) {
            Phase::Downloaded(body) => Some(body),
            other => {
                self.phase = other;
                None
            }
        }
    }

    /// Record the parse of the body from `take_body`. True means toast.
    pub fn parse_finished(&mut self, ok: bool, now: f64, refresh_secs: Option<f64>) -> bool {
        if !matches!(self.phase, Phase::Parsing) {
            return false;
        }
        self.phase = Phase::Idle;
        if ok {
            self.clock.succeeded(now, refresh_secs);
            false
        } else {
            self.clock.failed(now)
        }
    }

    pub fn is_busy(&self) -> bool {
        !matches!(self.phase, Phase::Idle)
    }

    /// Drop whatever is in flight (target changed, feed no longer wanted).
    pub fn cancel(&mut self, net: &mut impl NetPort) {
        if matches!(self.phase, Phase::InFlight { .. }) {
            net.cancel(&self.id);
        }
        self.phase = Phase::Idle;
    }

    pub fn retry_in_secs(&self, now: f64) -> Option<u32> {
        if self.is_busy() {
            None
        } else {
            self.clock.retry_in_secs(now)
        }
    }
}
```

If the compiler rejects `match self.phase` because the enum isn't `Copy`, match on `&self.phase` and copy `*started_at` into a local.

- [ ] **Step 4: Run the tests to verify they pass.** Run `cargo test -p rustywx --lib net_feed::`. Expected: 12 passed.

- [ ] **Step 5: Commit.** `git add app/src/net_feed.rs app/src/lib.rs && git commit -m "feat(net): Feed with cancel, deadline, backoff, and toast-once"`

---

### Task 9: Alerts on `Feed` (C2, C3)

**Files:**
- Modify: `app/src/alerts.rs` (constants, delete `fire_fetch`, `poll_response`, `poll_raw`)
- Modify: `app/src/state.rs` (replace `alerts_fetch_fired`, `last_alert_poll` with `alerts_feed`)
- Modify: `app/src/main.rs`:
  - initializer
  - parse-result arms around l.1066 and l.1101–1127
  - the two alerts blocks around l.1179–1226

**Interfaces:**
- Consumes: `net_feed::{Feed, RetryPolicy}`, `net_port::PlyNet` (`ply_net` is declared in `main` by the spaghetti feature).
- Produces:
  - `pub const alerts::NWS_ALERTS_URL`
  - `pub const alerts::REQUEST_HEADERS: [(&str, &str); 2]`
  - `pub const alerts::RETRY: RetryPolicy`
  - `AppState.alerts_feed: Feed`

Glue; the gate is clippy, the smoke test, and the offline check in Step 5.

- [ ] **Step 1: Constants.** In `alerts.rs`, make `NWS_ALERTS_URL` `pub`, and add:

```rust
/// Headers NWS requires (User-Agent with contact) plus GeoJSON.
pub const REQUEST_HEADERS: [(&str, &str); 2] =
    [("User-Agent", USER_AGENT), ("Accept", "application/geo+json")];

/// 5 s first retry, capped at the refresh interval; 30 s app-side deadline.
pub const RETRY: crate::net_feed::RetryPolicy = crate::net_feed::RetryPolicy {
    first_retry_secs: 5.0,
    max_retry_secs: 120.0,
    timeout_secs: 30.0,
};
```

Run `grep -rn "alerts::fire_fetch\|alerts::poll_raw\|alerts::poll_response" app/src`. After Step 3 there must be no hits; then delete those three functions.

- [ ] **Step 2: State.** In `state.rs`, replace the fields `alerts_fetch_fired` and `last_alert_poll` (and their docs) with:

```rust
    /// NWS alerts feed: stable id + cancel, deadline, backoff, toast-once.
    pub alerts_feed: crate::net_feed::Feed,
```

In the initializer: `alerts_feed: rustywx::net_feed::Feed::new(alerts::NET_ID, alerts::RETRY),`. Run `grep -n "alerts_fetch_fired\|last_alert_poll" app/src/*.rs` and resolve every hit.

- [ ] **Step 3: Frame loop.** Replace both blocks, `// ── Fire alerts fetch if not yet started` and `// ── Hand alerts response body to the parse worker`, with:

```rust
        // ── Alerts feed: stable id + cancel, deadline, backoff (C2/C3) ──
        if state
            .alerts_feed
            .tick(now, alerts::NWS_ALERTS_URL, &alerts::REQUEST_HEADERS, &mut ply_net)
            .show_toast
        {
            show_toast(&mut state, now, toast_widget::ErrorKind::Network);
        }
        if alerts_parse_generation.is_none()
            && background.parse_queue_has_capacity()
            && let Some(body) = state.alerts_feed.take_body()
        {
            let generation = next_generation;
            next_generation += 1;
            let body = String::from_utf8_lossy(&body).into_owned();
            let payload_bytes = body.len();
            let request = ParseRequest {
                generation,
                payload: ParsePayload::Alerts(body),
                submitted_at: std::time::Instant::now(),
            };
            if background.submit_parse(request).is_ok() {
                alerts_parse_generation = Some(generation);
                eprintln!(
                    "startup phase=alerts_response generation={generation} payload_bytes={payload_bytes} elapsed_ms={}",
                    startup_started.elapsed().as_millis()
                );
            } else if state.alerts_feed.parse_finished(false, now, Some(alerts::POLL_INTERVAL.as_secs_f64())) {
                show_toast(&mut state, now, toast_widget::ErrorKind::Network);
            }
        }
```

- [ ] **Step 4: Parse results.** In the `Ok(ParsedData::Alerts(alerts_list))` arm, add `state.alerts_feed.parse_finished(true, now, Some(alerts::POLL_INTERVAL.as_secs_f64()));`. Add this arm **before** the generic `Err(error) => {` arm:

```rust
                Err(error) if alerts_parse_generation == Some(parsed.generation) => {
                    alerts_parse_generation = None;
                    eprintln!(
                        "Warning: alerts parse failed generation={}: {error}",
                        parsed.generation
                    );
                    if state
                        .alerts_feed
                        .parse_finished(false, now, Some(alerts::POLL_INTERVAL.as_secs_f64()))
                    {
                        show_toast(&mut state, now, toast_widget::ErrorKind::Network);
                    }
                }
```

In the generic `Err` arm, delete the `|| alerts_parse_generation == Some(parsed.generation)` term and the `if alerts_parse_generation == ...` block.

- [ ] **Step 5: Verify.** Run `cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings`, then the smoke test.

Offline check:
1. Turn networking off (macOS: Wi-Fi off).
2. `just run`; wait 20 s. Expect exactly one toast and `Warning: fetch alerts failed` lines spaced 5 s, 10 s, 20 s… apart (not every frame).
3. Turn networking on. Within one backoff interval, expect `phase=alerts_response` and alert polygons with no restart.

- [ ] **Step 6: Commit.** `git commit -am "fix(alerts): recover from failed and hung fetches with cancel and backoff"`

---

### Task 10: Borders on three `Feed`s (C2, H6)

**Files:**
- Modify: `app/src/borders.rs` (constants; delete `fire_fetch_all`, `poll_and_merge`, `poll_raw`)
- Modify: `app/src/state.rs` (replace `borders_fetch_fired` with `border_feeds`)
- Modify: `app/src/main.rs`:
  - the two borders blocks around l.1131–1177
  - parse arms around l.1052 and the generic `Err` arm

**Interfaces:**
- Produces:
  - `pub const borders::URLS: [&str; 3]`
  - `pub const borders::NET_IDS: [&str; 3]`
  - `pub const borders::RETRY: RetryPolicy`
  - `AppState.border_feeds: [Feed; 3]`

Each feed polls its own id every frame and holds its body itself, so the H6 trace (two finished responses evicted while the third is pending) can't happen.

- [ ] **Step 1: Constants.** In `borders.rs`:

```rust
/// Fetch order matches `parse_and_merge(states, coast, country)`.
pub const URLS: [&str; 3] = [STATES_URL, COAST_URL, COUNTRY_LINES_URL];
pub const NET_IDS: [&str; 3] = [NET_ID_STATES, NET_ID_COAST, NET_ID_COUNTRY];

/// Multi-MB GitHub raw files: a 90 s deadline, retries capped at 5 min.
pub const RETRY: crate::net_feed::RetryPolicy = crate::net_feed::RetryPolicy {
    first_retry_secs: 5.0,
    max_retry_secs: 300.0,
    timeout_secs: 90.0,
};
```

Delete `fire_fetch_all`, `poll_and_merge`, and `poll_raw` once the grep for them in `app/src` is empty after Step 3.

- [ ] **Step 2: State.** Replace `borders_fetch_fired` with:

```rust
    /// One feed per border source (states, coast, country). Bodies are held
    /// until all three arrive, then parsed together.
    pub border_feeds: [crate::net_feed::Feed; 3],
```

In the initializer: `border_feeds: borders::NET_IDS.map(|id| rustywx::net_feed::Feed::new(id, borders::RETRY)),`.

- [ ] **Step 3: Frame loop.** Replace both blocks, `// ── Fire border fetch if not yet started and no cache` and `// ── Hand border response bodies to the parse worker`, with:

```rust
        // ── Border feeds: three independent requests (C2, H6) ──────────
        if !state.borders_loaded && borders_parse_generation.is_none() {
            let mut toast = false;
            for (feed, url) in state.border_feeds.iter_mut().zip(borders::URLS) {
                toast |= feed.tick(now, url, &[], &mut ply_net).show_toast;
            }
            if state.border_feeds.iter().all(|f| f.has_body()) && background.parse_queue_has_capacity() {
                let [states, coast, country] = state
                    .border_feeds
                    .each_mut()
                    .map(|f| String::from_utf8_lossy(&f.take_body().unwrap_or_default()).into_owned());
                let payload_bytes = states.len() + coast.len() + country.len();
                let generation = next_generation;
                next_generation += 1;
                let request = ParseRequest {
                    generation,
                    payload: ParsePayload::Borders { states, coast, country },
                    submitted_at: std::time::Instant::now(),
                };
                if background.submit_parse(request).is_ok() {
                    borders_parse_generation = Some(generation);
                    eprintln!(
                        "startup phase=borders_response generation={generation} payload_bytes={payload_bytes} elapsed_ms={}",
                        startup_started.elapsed().as_millis()
                    );
                } else {
                    for feed in &mut state.border_feeds {
                        toast |= feed.parse_finished(false, now, None);
                    }
                }
            }
            if toast {
                show_toast(&mut state, now, toast_widget::ErrorKind::Network);
            }
        }
```

- [ ] **Step 4: Parse results.** In the `Ok(ParsedData::Borders(rings))` arm, add `for feed in &mut state.border_feeds { feed.parse_finished(true, now, None); }` (one-shot: borders are cached after this). Add before the generic `Err` arm:

```rust
                Err(error) if borders_parse_generation == Some(parsed.generation) => {
                    borders_parse_generation = None;
                    eprintln!("Warning: borders parse failed generation={}: {error}", parsed.generation);
                    let mut toast = false;
                    for feed in &mut state.border_feeds {
                        toast |= feed.parse_finished(false, now, None);
                    }
                    if toast {
                        show_toast(&mut state, now, toast_widget::ErrorKind::Network);
                    }
                }
```

Remove the borders terms from the generic `Err` arm.

- [ ] **Step 5: Verify.** Run tests and clippy, then the smoke test.

Manual check:
1. Delete the cached borders file (`state_borders_v8` in the app-data dir) and run offline for 15 s, then go online.
2. Borders appear without a restart. There's one toast.
3. stderr shows each of the three `fetch borders-*` failures backing off independently.

- [ ] **Step 6: Commit.** `git commit -am "fix(borders): fetch each source independently and recover from failures"`

---

### Task 11: Forecast and geocoder recovery (C3 forecast, M23)

**Files:**
- Modify: `app/src/forecast.rs` (pub helpers and constants; `fire_geo` and `poll_geo` cancel; delete `fire_forecast` and `poll_forecast`)
- Modify: `app/src/state.rs` (`forecast_fetch_fired` becomes `forecast_feed: Option<Feed>`; `fc_geo_fired` becomes `fc_geo_query: Option<String>`)
- Modify: `app/src/main.rs` (forecast block in `handle_input` around l.3772–3851)

**Interfaces:**
- Produces:
  - `pub fn forecast::forecast_net_id(Coords) -> String`
  - `pub fn forecast::forecast_url(Coords) -> String`
  - `pub const forecast::HEADERS`
  - `pub const forecast::RETRY`
  - `pub const forecast::REFRESH_SECS: f64 = 1800.0`

- [ ] **Step 1: forecast.rs.** Make `forecast_net_id` and `forecast_url` `pub`, and add:

```rust
pub const HEADERS: [(&str, &str); 1] = [("User-Agent", USER_AGENT)];
pub const RETRY: crate::net_feed::RetryPolicy = crate::net_feed::RetryPolicy {
    first_retry_secs: 5.0,
    max_retry_secs: 300.0,
    timeout_secs: 30.0,
};
/// Refresh an open forecast every 30 min (review M23: it never refreshed).
pub const REFRESH_SECS: f64 = 1800.0;
```

Replace `fire_geo` and `poll_geo` with:

```rust
/// Fire the geocode search for `query`, freeing any stale entry first.
pub fn fire_geo(query: &str) {
    use ply_engine::prelude::net;
    let id = geo_net_id(query);
    if let Some(stale) = net::request(&id) {
        stale.cancel();
    }
    net::get(&id, &geo_url(query), |cfg| cfg.header("User-Agent", USER_AGENT));
}

/// Poll the geocode search for `query`. Cancels the entry once consumed so
/// the same query can be searched again (C2). A lost entry is an error,
/// never "pending forever" (M23).
pub fn poll_geo(query: &str) -> Option<Result<Vec<GeoHit>>> {
    use ply_engine::prelude::net;
    let Some(req) = net::request(&geo_net_id(query)) else {
        return Some(Err(anyhow!("geocode request lost")));
    };
    let resp = req.response()?;
    req.cancel();
    match resp {
        Ok(r) => Some(parse_geo(r.text())),
        Err(e) => Some(Err(anyhow!("geocoding: {e}"))),
    }
}
```

Delete `fire_forecast` and `poll_forecast` after Step 3.

- [ ] **Step 2: State.** Replace `forecast_fetch_fired: bool` with `pub forecast_feed: Option<crate::net_feed::Feed>,` (doc: "Forecast feed for `forecast_coords`; replaced when the coords change.") and `fc_geo_fired: bool` with `pub fc_geo_query: Option<String>,` (doc: "Query submitted to the geocoder, polled by this string, not the live box text."). Update the initializer (`None`, `None`) and resolve every `grep -n "forecast_fetch_fired\|fc_geo_fired" app/src/*.rs` hit. Use `state.fc_geo_query.is_some()` where a "searching" flag was read.

- [ ] **Step 3: Forecast fetch.** In `handle_input`'s Forecast block, replace everything from `if let Some(target) = state.user_location {` through the end of the `// Poll whichever coords we last fired for.` block with:

```rust
        let mut net = rustywx::net_port::PlyNet;
        let now = get_time();
        if let Some(target) = state.user_location
            && state.forecast_coords != Some(target)
        {
            if let Some(old) = state.forecast_feed.as_mut() {
                old.cancel(&mut net);
            }
            if state.forecast_place.is_empty() {
                state.forecast_place = if !state.settings.location_input.is_empty() {
                    state.settings.location_input.clone()
                } else {
                    format!("{:.2}, {:.2}", target.lat, target.lon)
                };
            }
            state.forecast_feed = Some(rustywx::net_feed::Feed::new(
                forecast::forecast_net_id(target),
                forecast::RETRY,
            ));
            state.forecast_coords = Some(target);
            state.forecast_error = None;
        }
        if let (Some(feed), Some(coords)) = (state.forecast_feed.as_mut(), state.forecast_coords) {
            feed.tick(now, &forecast::forecast_url(coords), &forecast::HEADERS, &mut net);
            if let Some(body) = feed.take_body() {
                match forecast::parse_forecast(&String::from_utf8_lossy(&body)) {
                    Ok(mut fc) => {
                        fc.place = state.forecast_place.clone();
                        state.forecast = Some(fc);
                        state.forecast_error = None;
                        feed.parse_finished(true, now, Some(forecast::REFRESH_SECS));
                    }
                    Err(e) => {
                        state.forecast_error = Some(e.to_string());
                        feed.parse_finished(false, now, Some(forecast::REFRESH_SECS));
                    }
                }
            } else if state.forecast.is_none()
                && let Some(secs) = feed.retry_in_secs(now)
            {
                state.forecast_error = Some(format!("Forecast unavailable, retrying in {secs} s"));
            }
        }
```

The forecast JSON is a few KB, so parsing it here on the UI thread stays as it was.

- [ ] **Step 4: Geocoder.** In the Enter handler, replace `forecast::fire_geo(&q); state.fc_geo_fired = true;` with `forecast::fire_geo(&q); state.fc_geo_query = Some(q);`. Replace the `// Poll geocode results.` block with:

```rust
        // Poll by the submitted query, not the live box text (M23).
        if let Some(q) = state.fc_geo_query.clone() {
            match forecast::poll_geo(&q) {
                Some(Ok(hits)) => {
                    state.fc_geo_hits = hits;
                    state.fc_geo_query = None;
                }
                Some(Err(_)) => {
                    state.fc_geo_hits = Vec::new();
                    state.fc_geo_query = None;
                }
                None => {}
            }
        }
```

- [ ] **Step 5: Verify.** Run tests and clippy, then the smoke test. Manual check:
1. Open Forecast offline. It shows "Forecast unavailable, retrying in N s" and recovers when the network returns.
2. Search "Atlanta", then edit the box before the results land. The results still arrive.
3. Searching "Atlanta" twice in a row works both times.

- [ ] **Step 6: Commit.** `git commit -am "fix(forecast): retry failed forecasts, refresh every 30 min, unstick geocoder"`

---

### Task 12: NHC fetch state machine on `NetPort` — cancel after consume, deadlines, per-storm KMZ ids (C2, C3, M22)

**Files:**
- Modify: `app/src/nhc.rs`:
  - `fire_gis_layer` l.32
  - `NhcFetchPhase` l.865
  - `start` l.931
  - `poll` l.948
  - `poll_phase1` l.980
  - `accept_metadata` l.1021
  - `poll_phase2` l.1168
  - tests

**Interfaces:**
- Consumes: `net_port::{NetPoll, NetPort, testing::FakeNet}`.
- Produces (signature changes; Task 13 wires them):
  - `NhcFetchState::start(&mut self, now: f64, net: &mut impl NetPort)`
  - `poll(&mut self, now: f64, net: &mut impl NetPort) -> Option<Result<NhcPoll>>`
  - `accept_metadata(&mut self, metas, gis_cone, gis_track, gis_points, gis_ww, now: f64, net: &mut impl NetPort)`
  - `pub const nhc::RETRY: RetryPolicy`
  - `pub const PHASE1_TIMEOUT_SECS: f64 = 45.0`
  - `pub const PHASE2_TIMEOUT_SECS: f64 = 90.0`

- [ ] **Step 1: Write the failing tests.** Add to `mod tests` in `nhc.rs`:

```rust
    use crate::net_port::testing::FakeNet;
    use crate::net_port::NetPoll;

    fn storms_json(ids: &[&str]) -> Vec<u8> {
        let storms: Vec<String> = ids
            .iter()
            .map(|id| {
                format!(
                    r#"{{"id":"{id}","binNumber":"AT2","name":"Test","classification":"HU",
                    "intensity":"100","pressure":"974","latitudeNumeric":25.3,"longitudeNumeric":-85.2,
                    "movementDir":315,"movementSpeed":12,"lastUpdate":"2026-07-19T23:44:32.968Z",
                    "publicAdvisory":{{"advNum":"12","url":"u","fileUpdateTime":"2026-07-19T23:44:32.968Z"}},
                    "forecastGraphics":{{"advNum":"12","url":"u","fileUpdateTime":"2026-07-19T23:44:32.968Z"}},
                    "earliestArrivalTimeTSWindsGIS":{{"kmzFile":"https://x/{id}-earliest.kmz"}}}}"#
                )
            })
            .collect();
        format!(r#"{{"activeStorms":[{}]}}"#, storms.join(",")).into_bytes()
    }

    /// Drive one cycle up to phase 2: start, answer CurrentStorms, accept.
    fn to_phase2(st: &mut NhcFetchState, net: &mut FakeNet, ids: &[&str], now: f64) {
        st.start(now, net);
        net.respond(NET_ID_CURRENT_STORMS, NetPoll::Done(Ok(storms_json(ids))));
        let Some(Ok(NhcPoll::Metadata(bodies))) = st.poll(now + 1.0, net) else {
            panic!("expected metadata handoff");
        };
        let metas = parse_current_storms(&bodies.storms).unwrap();
        st.accept_metadata(metas, None, None, None, None, now + 1.0, net);
    }

    #[test]
    fn nhc_cycle_cancels_every_request_and_refires_next_cycle() {
        let mut st = NhcFetchState::new();
        let mut net = FakeNet::default();
        to_phase2(&mut st, &mut net, &["al022026"], 0.0);
        net.respond_all_live(|_| NetPoll::Done(Err("http status: 404".into())));
        assert!(st.poll(2.0, &mut net).is_none());
        assert!(matches!(st.poll(2.1, &mut net), Some(Ok(NhcPoll::Complete(_)))));
        assert!(net.live.is_empty(), "every request released: {:?}", net.live);
        to_phase2(&mut st, &mut net, &["al022026"], 400.0);
        assert!(net.ignored.is_empty(), "second cycle really fired (C2): {:?}", net.ignored);
    }

    #[test]
    fn nhc_phase1_deadline_errors_and_cancels() {
        let mut st = NhcFetchState::new();
        let mut net = FakeNet::default();
        st.start(0.0, &mut net);
        assert!(st.poll(PHASE1_TIMEOUT_SECS - 0.1, &mut net).is_none());
        assert!(st.poll(PHASE1_TIMEOUT_SECS, &mut net).is_none());
        assert!(matches!(st.poll(PHASE1_TIMEOUT_SECS + 0.1, &mut net), Some(Err(_))));
        assert!(net.live.is_empty(), "{:?}", net.live);
        assert!(!st.is_fetching());
    }

    #[test]
    fn nhc_phase2_deadline_completes_with_partial_bundle() {
        let mut st = NhcFetchState::new();
        let mut net = FakeNet::default();
        to_phase2(&mut st, &mut net, &["al022026"], 0.0);
        assert!(st.poll(1.0 + PHASE2_TIMEOUT_SECS, &mut net).is_none());
        match st.poll(1.1 + PHASE2_TIMEOUT_SECS, &mut net) {
            Some(Ok(NhcPoll::Complete(bundle))) => assert_eq!(bundle.metas.len(), 1),
            _ => panic!("expected a partial bundle"),
        }
        assert!(net.live.is_empty(), "hung requests cancelled: {:?}", net.live);
    }

    #[test]
    fn arrival_kmz_ids_are_per_storm() {
        let mut st = NhcFetchState::new();
        let mut net = FakeNet::default();
        to_phase2(&mut st, &mut net, &["al012026", "al022026"], 0.0);
        let ids: Vec<&str> = net.fired.iter().map(|(id, _)| id.as_str()).collect();
        assert!(ids.contains(&"nhc-kmz-earliest-al012026"), "{ids:?}");
        assert!(ids.contains(&"nhc-kmz-earliest-al022026"), "{ids:?}");
        assert!(net.ignored.is_empty(), "second storm's KMZ not swallowed (M22)");
    }
```

- [ ] **Step 2: Run the tests to verify they fail.** Run `cargo test -p rustywx --lib nhc::`. Expected: compile errors (wrong arity for `start`/`poll`/`accept_metadata`).

- [ ] **Step 3: Implement.** In `nhc.rs`:

1. Imports and constants (top of file):

```rust
use crate::net_port::{NetPoll, NetPort};

/// User-Agent for NHC requests.
const USER_AGENT: &str = "rustywx/0.3";
/// GIS layers in `NhcMetadataBodies` order: cone, track, points, watches/warnings.
const GIS_LAYERS: [u32; 4] = [7, 6, 5, 8];
/// App-side deadlines (ply has no HTTP timeout, review C3).
pub const PHASE1_TIMEOUT_SECS: f64 = 45.0;
pub const PHASE2_TIMEOUT_SECS: f64 = 90.0;
/// Retry policy for starting a cycle after a failed one.
pub const RETRY: crate::net_feed::RetryPolicy = crate::net_feed::RetryPolicy {
    first_retry_secs: 5.0,
    max_retry_secs: 300.0,
    timeout_secs: PHASE1_TIMEOUT_SECS,
};

fn gis_net_id(layer: u32) -> String {
    format!("{NET_ID_GIS_PREFIX}{layer}")
}

fn fire_gis_layer(net: &mut impl NetPort, layer: u32) {
    let url = format!("{MAPSERVER}/{layer}/query?where=1%3D1&outFields=*&f=geojson");
    net.get_with_headers(&gis_net_id(layer), &url, &[]);
}

fn cancel_gis(net: &mut impl NetPort) {
    for layer in GIS_LAYERS {
        net.cancel(&gis_net_id(layer));
    }
}

/// Poll `id`; on a result, cancel the entry so the id is free (C2).
fn take_response(net: &mut impl NetPort, id: &str) -> Option<std::result::Result<Vec<u8>, String>> {
    match net.poll(id) {
        NetPoll::Pending => None,
        NetPoll::Done(result) => {
            net.cancel(id);
            Some(result)
        }
        NetPoll::Missing => Some(Err("request handle evicted".to_owned())),
    }
}
```

   Delete the old `fire_gis_layer`.

2. Phases. Change `Phase1` to `Phase1 { started_at: f64, gis: [Option<String>; 4] }`, and add `started_at: f64` to `Phase2 { .. }`. GIS bodies that finish before CurrentStorms are now taken and stashed, because cancel-after-consume means they can't be left in ply to re-read.

3. `start`:

```rust
    pub fn start(&mut self, now: f64, net: &mut impl NetPort) {
        if self.is_fetching() {
            return;
        }
        net.get_with_headers(NET_ID_CURRENT_STORMS, CURRENT_STORMS_URL, &[("User-Agent", USER_AGENT)]);
        for layer in GIS_LAYERS {
            fire_gis_layer(net, layer);
        }
        self.phase = NhcFetchPhase::Phase1 {
            started_at: now,
            gis: [None, None, None, None],
        };
    }
```

4. `poll(&mut self, now: f64, net: &mut impl NetPort)`. Same `match`, with:
   - `Phase1 { started_at, gis } => self.poll_phase1(now, started_at, gis, net)`
   - the `Phase2 { .., started_at }` arm passing `now`, `started_at`, and `net` through to `poll_phase2`

5. `poll_phase1`:

```rust
    fn poll_phase1(
        &mut self,
        now: f64,
        started_at: f64,
        mut gis: [Option<String>; 4],
        net: &mut impl NetPort,
    ) -> Option<Result<NhcPoll>> {
        for (slot, layer) in gis.iter_mut().zip(GIS_LAYERS) {
            if slot.is_none()
                && let Some(Ok(body)) = take_response(net, &gis_net_id(layer))
            {
                *slot = Some(String::from_utf8_lossy(&body).into_owned());
            }
        }
        match take_response(net, NET_ID_CURRENT_STORMS) {
            None if now - started_at >= PHASE1_TIMEOUT_SECS => {
                net.cancel(NET_ID_CURRENT_STORMS);
                cancel_gis(net);
                self.phase = NhcFetchPhase::Error(format!(
                    "CurrentStorms.json: timed out after {PHASE1_TIMEOUT_SECS} s"
                ));
                None
            }
            None => {
                self.phase = NhcFetchPhase::Phase1 { started_at, gis };
                None
            }
            Some(Err(e)) => {
                cancel_gis(net);
                self.phase = NhcFetchPhase::Error(format!("CurrentStorms.json: {e}"));
                None
            }
            Some(Ok(body)) => {
                let [gis_cone, gis_track, gis_points, gis_ww] = gis;
                self.phase = NhcFetchPhase::AwaitingMetadata;
                Some(Ok(NhcPoll::Metadata(NhcMetadataBodies {
                    storms: String::from_utf8_lossy(&body).into_owned(),
                    gis_cone,
                    gis_track,
                    gis_points,
                    gis_ww,
                })))
            }
        }
    }
```


6. `accept_metadata(..., now: f64, net: &mut impl NetPort)`:
   - For each GIS layer that hasn't arrived, call `net.cancel(&gis_net_id(layer)); fire_gis_layer(net, layer);`. That's an explicit cancel then re-fire, so it's correct with any transport.
   - Text requests: `net.get_with_headers(&net_id, &url, &[("User-Agent", USER_AGENT)])`.
   - Images and wind-prob KMZs: `net.get_with_headers(&net_id, url, &[])`.
   - **M22:** the arrival KMZ ids become `format!("nhc-kmz-earliest-{}", meta.id)` and `format!("nhc-kmz-mostlikely-{}", meta.id)`.
   - Store `started_at: now` in `Phase2`.

7. `poll_phase2(..., now: f64, started_at: f64, net: &mut impl NetPort)`:
   - Replace `net::request(&req.net_id).and_then(|r| r.response())` with `take_response(net, &req.net_id)`.
   - The `Some(Ok(r))` arm now gets a `Vec<u8>` named `bytes`. Replace `r.text()` with `&String::from_utf8_lossy(&bytes)`, and `r.bytes()` with `&bytes` (`.to_vec()` for images becomes a move of `bytes`).
   - **M22:** `results.earliest_arrival = parse_arrival_kml(&kml)` becomes `results.earliest_arrival.extend(parse_arrival_kml(&kml))`, and the same for `most_likely_arrival`, so every storm's contours are kept.
   - After the drain loop, add the deadline:

```rust
        if !still_pending.is_empty() && now - started_at >= PHASE2_TIMEOUT_SECS {
            eprintln!(
                "Warning: NHC phase 2 timed out with {} request(s) pending; completing with partial data",
                still_pending.len()
            );
            for req in still_pending.drain(..) {
                net.cancel(&req.net_id);
            }
        }
```

   The existing `if still_pending.is_empty()` assembly then runs. Make `still_pending` `let mut`.

8. Delete every remaining `use ply_engine::prelude::net;` in `nhc.rs`; the module must not touch ply `net` directly any more. Check with `grep -n "prelude::net" app/src/nhc.rs`, which should print nothing.

- [ ] **Step 4: Run the tests to verify they pass.** Run `cargo test -p rustywx --lib nhc::`. Expected: all pass, including the 4 new tests. `main.rs` won't compile until Task 13, so run only the lib tests here, then do Task 13 before committing.

---

### Task 13: Wire NHC into the frame loop with a `RetryClock` (C2) and clamp the refresh (M13)

**Files:**
- Modify: `app/src/state.rs` (replace `nhc_fetch_fired`, `nhc_last_poll` with `nhc_retry`)
- Modify: `app/src/main.rs`:
  - NHC fire/poll block around l.1228–1311
  - `accept_metadata` call around l.1089
  - every read of `nhc_fetch_fired`

**Interfaces:**
- Consumes: Task 12 signatures, `net_feed::RetryClock`, `nhc::RETRY`.
- Produces: `AppState.nhc_retry: RetryClock`.

- [ ] **Step 1: State.** Replace `nhc_fetch_fired` and `nhc_last_poll` with:

```rust
    /// When the next NHC cycle may start: refresh interval after success,
    /// backoff after failure, toast once per streak.
    pub nhc_retry: crate::net_feed::RetryClock,
```

In the initializer: `nhc_retry: rustywx::net_feed::RetryClock::new(nhc::RETRY, "nhc"),`. Replace every read of `state.nhc_fetch_fired` (the "Loading NHC data…" label, the storm badge, the spaghetti block if it reads it) with `state.nhc_fetch.is_fetching()`. Check with `grep -n "nhc_fetch_fired\|nhc_last_poll" app/src/*.rs`, which should print nothing.

- [ ] **Step 2: Frame loop.** Replace the `// ── Fire NHC fetch` and `// ── Poll NHC fetch state machine` blocks with:

```rust
        // ── NHC: start when due (refresh or backoff; review C2) ──────────
        // M13: never refresh faster than once a minute, whatever the setting.
        let nhc_refresh = state.settings.nhc_refresh_secs.max(60) as f64;
        if !state.nhc_fetch.is_fetching()
            && nhc_parse_generation.is_none()
            && state.nhc_retry.is_due(now)
        {
            state.nhc_fetch.start(now, &mut ply_net);
        }
        if let Some(result) = state.nhc_fetch.poll(now, &mut ply_net) {
            match result {
                Ok(nhc::NhcPoll::Metadata(bodies)) => {
                    // unchanged: the existing submit-to-parse-worker code,
                    // including `reject_metadata` when the queue is full
                }
                Ok(nhc::NhcPoll::Complete(mut bundle)) => {
                    state.nhc_retry.succeeded(now, Some(nhc_refresh));
                    // unchanged: the existing image queueing and bundle install
                }
                Err(e) => {
                    eprintln!("Warning: NHC fetch failed: {e:#}");
                    if state.nhc_retry.failed(now) {
                        show_toast(&mut state, now, toast_widget::ErrorKind::Network);
                    }
                }
            }
        }
```

Paste the bodies of the existing `Metadata` and `Complete` arms in place of the "unchanged" comments. Delete the old `state.nhc_fetch_fired = false` line and the `is_terminal` check.

- [ ] **Step 3: accept_metadata.** In the `Ok(ParsedData::NhcMetadata(metadata))` arm, add the arguments `now, &mut ply_net` to `state.nhc_fetch.accept_metadata(...)`.

- [ ] **Step 4: Verify.** Run `cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings`, then the smoke test.

Offline check:
1. Launch offline. There's one toast, and `NHC fetch failed` retries are spaced 5, 10, 20 s….
2. Go online. NHC loads without a restart.
3. Open the Tropical panel: "Loading NHC data…" shows only while a cycle runs.

With 2+ active storms that have arrival KMZs, turn on "Earliest Arrival" and check that contours draw for each storm (M22).

- [ ] **Step 5: Commit (Tasks 12 and 13 together).** `git commit -am "fix(nhc): cancel consumed requests, add phase deadlines and backoff, per-storm KMZ ids"`

---

### Task 14: ureq `win-system-proxy` stopgap and ureq 3.4 (H1 partial)

**Files:**
- Modify: `app/Cargo.toml`, `Cargo.lock`
- Modify: `USER_GUIDE.md` (Troubleshooting)

- [ ] **Step 1: Add the feature.** In `app/Cargo.toml` `[dependencies]`, add:

```toml
# Stopgap (review H1): ply-engine's ureq ignores the Windows proxy setting.
# Enabling this feature from here unifies with ply's `rustls` and makes
# ureq read HKCU Internet Settings\ProxyServer. No PAC support. Remove when
# the app-owned HTTP module (remediation Phase 5) lands.
ureq = { version = "3", default-features = false, features = ["win-system-proxy"] }
```

- [ ] **Step 2: Update ureq.** Run `cargo update -p ureq`. Expect 3.3.0 → 3.4.x, which brings the TLS-through-CONNECT and timeout fixes. Then `cargo tree -i ureq -e features --target x86_64-pc-windows-msvc | head -20` should show both `rustls` and `win-system-proxy` enabled. Run `cargo deny check`; `winreg` is MIT and must pass.

- [ ] **Step 3: Document.** Add to `USER_GUIDE.md` under Tips (or a new `## Troubleshooting`):

```markdown
### Network errors behind a proxy (Windows)

rustywx uses the proxy set in Windows Settings > Network > Proxy (manual proxy only; automatic setup scripts/PAC are not supported yet). If your network requires a PAC script or the manual proxy isn't picked up, set the `HTTPS_PROXY` environment variable (for example `http://proxy.example.com:8080`) for your user and restart rustywx. Networks that inspect TLS with their own certificate authority are not supported yet.
```

- [ ] **Step 4: Verify.** Run `cargo build --release -p rustywx` and the smoke test.

**[verify on Windows]** (Appendix B, item 3): with a manual proxy set in Windows Settings and no `HTTPS_PROXY` set, alerts and NHC load through the proxy. With the proxy set to a dead port, the failure is one toast plus a retry, not a hang.

- [ ] **Step 5: Commit.** `git commit -am "fix(net): honor the Windows proxy setting and update ureq to 3.4"`


---

# Phase 2 — Windows crash-proofing (PR `fix/windows-crash-proofing`)

Closes H2, H9 (a, b, c), H8, and M21. It also makes M4's startup failures visible: they now produce a crash log and a message box instead of a vanishing window. Independent of Phase 1.

### Task 15: Vendor `miniquad-ply` 0.4.8 with the wheel, drop-file, and DPI fixes (H2, H9c, M5 assert)

**Files:**
- Create: `vendor/miniquad-ply/` (a copy of the registry crate)
- Create: `vendor/miniquad-ply/src/win_fixes.rs`
- Create: `vendor/miniquad-ply/PATCHES.md`
- Modify: `vendor/miniquad-ply/src/lib.rs`
- Modify: `vendor/miniquad-ply/src/native/windows.rs`:
  - `WM_MOUSEWHEEL`/`WM_MOUSEHWHEEL` around l.566–572
  - `WM_DROPFILES` around l.741–762
  - `DragAcceptFiles` around l.973
  - `init_dpi` around l.1095–1110
  - the `shellapi` import around l.21
- Modify: root `Cargo.toml` (`exclude`, `[patch.crates-io]`)
- Modify: `app/src/main.rs` (one test)

**Interfaces:**
- Produces: `miniquad::win_fixes::{WHEEL_DELTA, wheel_notches(raw: i16) -> f32, dpi_scale(hr: i32, dpi: u32) -> f32}`, compiled on every OS so they're testable off Windows.

Why patch rather than fix in the app: ply's own scroll containers read `mouse_wheel()` directly with `SCROLL_SPEED = 20`, so no app-side shim can fix them (research section B). After the patch, Windows delivers ±1.0 per notch, matching X11. **Do not** also divide by 120 in the app.

- [ ] **Step 1: Vendor the crate.**

```bash
mkdir -p vendor
cp -R ~/.cargo/registry/src/index.crates.io-*/miniquad-ply-0.4.8 vendor/miniquad-ply
rm -f vendor/miniquad-ply/ime_output.txt
```

Edit the root `Cargo.toml`:

```toml
[workspace]
members = ["app"]
exclude = ["vendor"]
resolver = "2"

# Windows fixes for miniquad (review H2, H9c; research section B). Same
# version number as crates.io. NOTE: cargo ignores [patch] on `cargo publish`,
# so `cargo install rustywx` users get unpatched miniquad; GitHub release
# binaries are patched. Drop this when an upstream release has the fixes
# (see vendor/miniquad-ply/PATCHES.md).
[patch.crates-io]
miniquad-ply = { path = "vendor/miniquad-ply" }
```

Run `cargo build -p rustywx` and `grep -A3 'name = "miniquad-ply"' Cargo.lock`. The `source =` line must be gone, which shows the path patch is in use.

- [ ] **Step 2: Write the failing test.** In `app/src/main.rs`'s `#[cfg(test)] mod window_backend_tests` (or a new `#[cfg(test)] mod miniquad_patch_tests { use super::*; ... }`), add:

```rust
    #[test]
    fn vendored_miniquad_patch_is_active() {
        // Compiles only against vendor/miniquad-ply (the [patch] is in effect).
        assert_eq!(miniquad::win_fixes::wheel_notches(120), 1.0);
        assert_eq!(miniquad::win_fixes::wheel_notches(-240), -2.0);
        assert_eq!(miniquad::win_fixes::wheel_notches(30), 0.25, "precision touchpad");
        assert_eq!(miniquad::win_fixes::dpi_scale(0, 144), 1.5);
        assert_eq!(miniquad::win_fixes::dpi_scale(-2147024809, 0), 1.0, "E_INVALIDARG falls back to 96 dpi");
        assert_eq!(miniquad::win_fixes::dpi_scale(0, 0), 1.0);
    }
```

Run `cargo test -p rustywx --bin rustywx vendored_miniquad_patch_is_active`. Expected: compile error `could not find 'win_fixes' in 'miniquad'`.

- [ ] **Step 3: Add the helpers.** Create `vendor/miniquad-ply/src/win_fixes.rs`:

```rust
//! Pure helpers for the rustywx Windows patches (see PATCHES.md). Compiled
//! on every platform so they can be unit-tested off Windows.

/// One detent of a standard mouse wheel in WM_MOUSEWHEEL units.
pub const WHEEL_DELTA: f32 = 120.0;

/// Raw WM_MOUSEWHEEL / WM_MOUSEHWHEEL delta to notches, so Windows reports
/// ±1.0 per detent like X11. Precision touchpads give fractions.
pub fn wheel_notches(raw: i16) -> f32 {
    f32::from(raw) / WHEEL_DELTA
}

/// Window scale from `GetDpiForMonitor`, falling back to 1.0 (96 dpi) when
/// the call fails (seen on some RDP/VM setups) or reports 0, instead of the
/// upstream `assert_eq!(hr, 0)` that aborts startup.
pub fn dpi_scale(hr: i32, dpi: u32) -> f32 {
    if hr == 0 && dpi > 0 {
        dpi as f32 / 96.0
    } else {
        1.0
    }
}
```

In `vendor/miniquad-ply/src/lib.rs`, after `pub mod native;`, add:

```rust
#[doc(hidden)]
pub mod win_fixes;
```

- [ ] **Step 4: Patch `native/windows.rs`.**

1. Wheel:

```rust
        WM_MOUSEWHEEL => {
            // rustywx patch: notches, not raw WHEEL_DELTA units (±120).
            let notches = crate::win_fixes::wheel_notches(HIWORD(wparam as _) as i16);
            event_handler.mouse_wheel_event(0.0, notches);
        }

        WM_MOUSEHWHEEL => {
            let notches = crate::win_fixes::wheel_notches(HIWORD(wparam as _) as i16);
            event_handler.mouse_wheel_event(notches, 0.0);
        }
```

2. Drop files. Add `DragFinish` to the `shellapi::{...}` import. In `WM_DROPFILES`, replace `d.dropped_files.bytes.push(std::fs::read(&path).unwrap());` with:

```rust
                    // rustywx patch: never read dropped files on the window
                    // thread; a folder or locked file used to abort the process
                    // (unwrap inside an extern "system" callback).
                    d.dropped_files.bytes.push(Vec::new());
```

   After the `for` loop, still inside the `WM_DROPFILES` arm, add `DragFinish(hdrop);`.

3. Stop accepting drops at all (the app never uses them). Replace `DragAcceptFiles(hwnd, TRUE);` with:

```rust
    // rustywx patch: drag-and-drop disabled; the app has no use for dropped
    // files and the handler above is kept only as a safe fallback.
    DragAcceptFiles(hwnd, FALSE);
```

   If `FALSE` isn't imported, use `0`.

4. DPI: replace `assert_eq!(hr, 0);` and the next `self.window_scale = dpix as f32 / 96.0;` with:

```rust
            // rustywx patch: fall back to 96 dpi instead of aborting.
            self.window_scale = crate::win_fixes::dpi_scale(hr as i32, dpix as u32);
```

   Check the type of `hr` in that function. It's an `HRESULT` (`i32`), so `as i32` is a no-op cast; drop it if clippy flags it.

- [ ] **Step 5: Record the patch.** Create `vendor/miniquad-ply/PATCHES.md`:

```markdown
# rustywx patches to miniquad-ply 0.4.8

Vendored copy of crates.io `miniquad-ply` 0.4.8 (repo `TheRedDeveloper/miniquad-fix`), applied through `[patch.crates-io]` in the workspace `Cargo.toml`. Upstream `not-fl3/miniquad` master has the same code for all of these, so no fixed release exists yet (checked 2026-10-09).

| Change | File | Why |
|---|---|---|
| `src/win_fixes.rs` (new, pure helpers) | `src/lib.rs`, `src/win_fixes.rs` | Testable on every OS |
| Wheel delta divided by 120 (`WM_MOUSEWHEEL`, `WM_MOUSEHWHEEL`) | `src/native/windows.rs` | One notch jumped zoom to its limit and scrolled modals to the end (rustywx review H2) |
| `WM_DROPFILES` never reads file contents; calls `DragFinish` | same | `std::fs::read(..).unwrap()` in the window proc aborted on a dropped folder or locked file (H9c) |
| `DragAcceptFiles(hwnd, FALSE)` | same | The app never uses drops |
| `GetDpiForMonitor` failure falls back to 96 dpi | same | `assert_eq!(hr, 0)` aborted startup on some RDP/VM setups |

Upstream: send this diff to `TheRedDeveloper/miniquad-fix` and `not-fl3/miniquad`, then drop the vendor copy when a release carries it. To re-sync after a version bump, re-copy the crate and re-apply this table.
```

- [ ] **Step 6: Verify.** Run `cargo test --all-features` (the new test passes) and `cargo clippy --all-targets --all-features -- -D warnings`. Clippy doesn't lint the excluded vendor crate. Then run the smoke test on macOS or Linux, where behavior is unchanged.

Optional: `cargo test --manifest-path vendor/miniquad-ply/Cargo.toml --lib` (needs its dev-dependencies from the network).

**[verify on Windows]** (Appendix B, item 4):
- One wheel notch changes zoom by about 5%.
- One notch in the NHC panel scrolls about 20 px.
- A precision touchpad scrolls smoothly.
- Dragging a folder onto the window does nothing and doesn't crash.

- [ ] **Step 7: Commit.** `git add vendor Cargo.toml Cargo.lock app/src/main.rs && git commit -m "fix(windows): patch miniquad wheel delta, file-drop abort, and DPI assert"`

---

### Task 16: Explicit `main` and a `--version` flag that exits before the window exists

**Files:**
- Modify: `app/src/main.rs` (`#[macroquad::main(window_conf)] async fn main()` around l.555)

**Interfaces:**
- Produces:
  - `fn early_exit_text(args: impl Iterator<Item = String>) -> Option<String>`
  - `fn main()`, which calls `macroquad::Window::from_config(window_conf(), amain())`
  - `async fn amain()` (the former `main` body)

Phase 4's CI runs `rustywx.exe --version` on a GPU-less runner, so the flag must be handled before window or GL creation. Task 17 also needs a plain `fn main` to install the panic hook first.

- [ ] **Step 1: Write the failing test.** Add to a test module in `main.rs`:

```rust
    #[test]
    fn version_flag_exits_early_with_package_version() {
        let args = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>().into_iter();
        assert_eq!(
            early_exit_text(args(&["--version"])),
            Some(format!("rustywx {}", env!("CARGO_PKG_VERSION")))
        );
        assert!(early_exit_text(args(&["-V"])).is_some());
        assert_eq!(early_exit_text(args(&["--demo", "moore2013"])), None);
        assert_eq!(early_exit_text(args(&[])), None);
    }
```

- [ ] **Step 2: Run it to verify it fails.** Run `cargo test -p rustywx --bin rustywx version_flag`. Expected: `cannot find function 'early_exit_text'`.

- [ ] **Step 3: Implement.** Replace `#[macroquad::main(window_conf)]` and `async fn main() {` with `async fn amain() {`, leaving the body unchanged. Add above it:

```rust
/// Flags handled before any window or GL context exists (CI runs
/// `--version` on GPU-less runners; research section D).
fn early_exit_text(mut args: impl Iterator<Item = String>) -> Option<String> {
    args.find(|a| a == "--version" || a == "-V")
        .map(|_| format!("rustywx {}", env!("CARGO_PKG_VERSION")))
}

fn main() {
    if let Some(text) = early_exit_text(std::env::args().skip(1)) {
        println!("{text}");
        return;
    }
    macroquad::Window::from_config(window_conf(), amain());
}
```

- [ ] **Step 4: Verify.** Run `cargo test --all-features`, then `cargo run --release -p rustywx -- --version`, which must print `rustywx 0.x.y` and exit 0 with no window. Then the AGENTS.md smoke test: the app launches normally with no args.

- [ ] **Step 5: Commit.** `git commit -am "feat(cli): --version exits before the window is created"`

---

### Task 17: `windows_subsystem`, panic hook with crash log, stderr to a rotating log file (H8, M4 visibility)

**Files:**
- Create: `app/src/crash_log.rs`
- Modify: `app/src/lib.rs` (`pub mod crash_log;`)
- Modify: `app/src/main.rs` (first line; first statements of `fn main`)
- Modify: `app/Cargo.toml`:
  - `[target.'cfg(windows)'.dependencies]` windows-sys
  - `dirs`, if Phase 1a hasn't landed yet

**Interfaces:**
- Produces:
  - `pub fn log_dir() -> PathBuf` (`%LOCALAPPDATA%\rustywx`; `~/Library/Application Support`… resolved by `dirs::data_local_dir()`)
  - `pub const MAX_LOG_BYTES: u64`
  - `pub fn rotate_if_large(path: &Path, max: u64) -> bool`
  - `pub fn format_crash_report(when: &str, thread: &str, message: &str, location: &str, backtrace: &str) -> String`
  - `pub fn install_panic_hook(dir: PathBuf)`
  - `#[cfg(windows)] pub fn redirect_stderr_to_log(dir: &Path) -> Option<std::fs::File>`

- [ ] **Step 1: Dependencies.** If `dirs` isn't in `app/Cargo.toml` yet, run `cargo add dirs --package rustywx`. Add:

```toml
[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.61", features = ["Win32_Foundation", "Win32_System_Console", "Win32_UI_WindowsAndMessaging"] }
```

0.61.2 is already in `Cargo.lock`.

- [ ] **Step 2: Write the failing tests.** Create `app/src/crash_log.rs`:

```rust
//! Crash visibility, mainly for Windows (review H8, M4): a panic hook that
//! appends to `crash.log` (and shows a message box for main-thread panics on
//! Windows), plus `app.log`, which captures every `eprintln!` on Windows
//! release builds where there is no console. Std only, apart from
//! windows-sys for SetStdHandle and MessageBoxW.

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("rustywx-crashlog-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn rotate_if_large_moves_big_log_aside() {
        let d = dir("rotate");
        let log = d.join("app.log");
        std::fs::write(&log, vec![b'x'; 100]).unwrap();
        assert!(!rotate_if_large(&log, 1_000));
        assert!(rotate_if_large(&log, 50));
        assert!(!log.exists());
        assert_eq!(std::fs::read(d.join("app.log.1")).unwrap().len(), 100);
        assert!(!rotate_if_large(&d.join("missing.log"), 1));
    }

    #[test]
    fn crash_report_has_every_field() {
        let r = format_crash_report("2026-10-09T12:00:00Z", "main", "boom", "src/x.rs:1:2", "bt");
        for needle in ["2026-10-09T12:00:00Z", "thread: main", "message: boom", "location: src/x.rs:1:2", "bt", env!("CARGO_PKG_VERSION")] {
            assert!(r.contains(needle), "missing {needle}: {r}");
        }
    }

    #[test]
    fn panic_hook_appends_to_crash_log() {
        let d = dir("hook");
        install_panic_hook(d.clone());
        let _ = std::thread::Builder::new()
            .name("crash-test".into())
            .spawn(|| panic!("deliberate test panic"))
            .unwrap()
            .join();
        let _ = std::panic::take_hook(); // restore the default hook for other tests
        let log = std::fs::read_to_string(d.join("crash.log")).unwrap();
        assert!(log.contains("deliberate test panic"), "{log}");
        assert!(log.contains("thread: crash-test"), "{log}");
    }
}
```

Add `pub mod crash_log;` to `lib.rs`. Run `cargo test -p rustywx --lib crash_log::`. Expected: compile error `cannot find function 'rotate_if_large'`.

- [ ] **Step 3: Implement.** Insert above `#[cfg(test)]`:

```rust
use std::io::Write;
use std::path::{Path, PathBuf};

/// `app.log` is rotated to `app.log.1` at startup once it passes this size.
pub const MAX_LOG_BYTES: u64 = 2 * 1024 * 1024;

/// Where `crash.log` and `app.log` live: `%LOCALAPPDATA%\rustywx` on Windows
/// (never Roaming), the platform local-data dir elsewhere, temp as a last resort.
pub fn log_dir() -> PathBuf {
    dirs::data_local_dir()
        .map(|d| d.join("rustywx"))
        .unwrap_or_else(|| std::env::temp_dir().join("rustywx"))
}

/// Rename `path` to `<path>.1` (replacing an older one) if it's over `max` bytes.
pub fn rotate_if_large(path: &Path, max: u64) -> bool {
    let big = std::fs::metadata(path).map(|m| m.len() > max).unwrap_or(false);
    if !big {
        return false;
    }
    let old = path.with_extension("log.1");
    let _ = std::fs::remove_file(&old);
    std::fs::rename(path, &old).is_ok()
}

pub fn format_crash_report(when: &str, thread: &str, message: &str, location: &str, backtrace: &str) -> String {
    format!(
        "=== rustywx {} crash at {when}\nthread: {thread}\nmessage: {message}\nlocation: {location}\nbacktrace:\n{backtrace}\n",
        env!("CARGO_PKG_VERSION")
    )
}

fn panic_message(info: &std::panic::PanicHookInfo<'_>) -> String {
    let payload = info.payload();
    payload
        .downcast_ref::<&str>()
        .map(|s| (*s).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "<non-string panic payload>".to_owned())
}

/// Install as the first statement of `main`. Appends every panic to
/// `dir/crash.log`, shows a message box for main-thread panics on Windows
/// (the window would otherwise just vanish), then runs the previous hook.
pub fn install_panic_hook(dir: PathBuf) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("<unnamed>");
        let report = format_crash_report(
            &chrono::Utc::now().to_rfc3339(),
            thread_name,
            &panic_message(info),
            &info.location().map(|l| l.to_string()).unwrap_or_default(),
            &std::backtrace::Backtrace::force_capture().to_string(),
        );
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("crash.log"))
        {
            let _ = f.write_all(report.as_bytes());
        }
        // Only the main thread: worker panics are recoverable (JoinError) and
        // a retry loop must not spawn a message box per attempt.
        #[cfg(windows)]
        if thread_name == "main" {
            show_crash_box(&dir);
        }
        previous(info);
    }));
}

#[cfg(windows)]
fn show_crash_box(dir: &Path) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
    let wide = |s: &str| s.encode_utf16().chain([0]).collect::<Vec<u16>>();
    let text = wide(&format!(
        "rustywx stopped because of an error.\n\nDetails were saved to:\n{}",
        dir.join("crash.log").display()
    ));
    let title = wide("rustywx");
    unsafe {
        MessageBoxW(std::ptr::null_mut(), text.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR);
    }
}

/// Point the process stderr at `dir/app.log` (rotated first). std re-reads
/// the stderr handle on every write, so every existing `eprintln!` lands in
/// the file. Keep the returned `File` alive for the whole process.
/// **[verify on Windows]** (research section C).
#[cfg(windows)]
pub fn redirect_stderr_to_log(dir: &Path) -> Option<std::fs::File> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::System::Console::{STD_ERROR_HANDLE, SetStdHandle};
    std::fs::create_dir_all(dir).ok()?;
    let path = dir.join("app.log");
    rotate_if_large(&path, MAX_LOG_BYTES);
    let file = std::fs::OpenOptions::new().create(true).append(true).open(&path).ok()?;
    let ok = unsafe { SetStdHandle(STD_ERROR_HANDLE, file.as_raw_handle() as _) } != 0;
    ok.then_some(file)
}
```

- [ ] **Step 4: Wire it.** Add this as the very first line of `app/src/main.rs`, above all `use` lines:

```rust
// Windows release builds are GUI-subsystem: no console window behind the
// app, and closing a console can no longer kill it mid-write (review H8).
// Debug builds keep the console. stderr goes to app.log (crash_log).
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
```

Make the start of `fn main` (from Task 16):

```rust
fn main() {
    let log_dir = rustywx::crash_log::log_dir();
    rustywx::crash_log::install_panic_hook(log_dir.clone());
    #[cfg(all(windows, not(debug_assertions)))]
    let _stderr_log = rustywx::crash_log::redirect_stderr_to_log(&log_dir);
    if let Some(text) = early_exit_text(std::env::args().skip(1)) {
```

`_stderr_log` lives until `main` returns, which is after the window closes.

- [ ] **Step 5: Verify.** Run `cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings`, then the smoke test. On macOS and Linux nothing visible changes, except that a panic now also appends to `~/Library/Application Support/rustywx/crash.log` (macOS: `dirs::data_local_dir` is Application Support) or `~/.local/share/rustywx/crash.log`.

Windows CI (Phase 4) compiles the `cfg(windows)` code.

**[verify on Windows]** (Appendix B, item 5):
- Double-clicking `rustywx.exe` opens no console.
- `%LOCALAPPDATA%\rustywx\app.log` fills with the `startup phase=` lines.
- A forced panic (temporarily add `panic!()` after the settings load in a scratch build) shows the message box and writes `crash.log` with a backtrace.
- `rustywx.exe --version | Out-String` in pwsh still prints the version, because piped handles are inherited even by GUI-subsystem exes.

- [ ] **Step 6: Commit.** `git commit -am "fix(windows): GUI subsystem, panic hook with crash log, stderr to app.log"`

---

### Task 18: Byte-slice panics (H9a, H9b, M21)

**Files:**
- Create: `app/src/text.rs`
- Modify: `app/src/lib.rs` (`pub mod text;`)
- Modify:
  - `app/src/main.rs` (NHC text preview around l.2707)
  - `app/src/scope.rs` (alert label around l.1766)
  - `app/src/nhc.rs` (`iso_to_ddhhmm` l.304, `construct_image_products` l.391)

**Interfaces:**
- Produces: `pub fn text::truncate_chars(s: &str, max_chars: usize) -> &str`.

If the 2026-08-27 audit plan's Tasks 12–13 already landed, Step 4 is done: check `nhc.rs` for `.get(` in those two functions and skip.

- [ ] **Step 1: Write the failing tests.** Create `app/src/text.rs`:

```rust
//! Char-boundary-safe string helpers. Never slice untrusted text by byte
//! index: a multi-byte char across the cut panics (review H9).

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_chars_never_splits_a_char() {
        let spanish = "é".repeat(400); // 2 bytes per char
        let cut = truncate_chars(&spanish, 300);
        assert_eq!(cut.chars().count(), 300);
        let dash = format!("{}—tail", "a".repeat(299)); // em dash (3 bytes) straddles byte 300
        assert_eq!(truncate_chars(&dash, 300), format!("{}—", "a".repeat(299)));
        assert_eq!(truncate_chars("short", 300), "short");
        assert_eq!(truncate_chars("abc", 0), "");
    }
}
```

Add `pub mod text;` to `lib.rs`.

- [ ] **Step 2: Add NHC tests.** In `nhc.rs` `mod tests`:

```rust
    #[test]
    fn iso_to_ddhhmm_rejects_short_time() {
        assert!(iso_to_ddhhmm("2026-07-19T2").is_none());
        assert!(iso_to_ddhhmm("2026-07-19T23:4").is_none());
        assert!(iso_to_ddhhmm("2026-07-19T23:4\u{e9}Z").is_none(), "non-ASCII never panics");
        assert!(iso_to_ddhhmm("T23:44").is_none());
    }

    #[test]
    fn construct_image_products_short_id_is_empty() {
        for id in ["al", "al0920", "", "al\u{e9}2026"] {
            let meta = StormMeta {
                id: id.to_string(),
                name: "T".into(),
                classification: "HU".into(),
                intensity_kt: 0,
                pressure_mb: 0,
                lat: 0.0,
                lon: 0.0,
                movement_dir_deg: None,
                movement_speed_kt: None,
                last_update: String::new(),
                bin_number: "AT2".into(),
                advisory_num: String::new(),
                graphics_url: String::new(),
                graphics_issuance: "192344".into(),
                earliest_arrival_kmz: String::new(),
                most_likely_arrival_kmz: String::new(),
                wind_probs_34kt_kmz: String::new(),
                wind_probs_50kt_kmz: String::new(),
                wind_probs_64kt_kmz: String::new(),
            };
            let _ = construct_image_products(&meta); // must not panic
        }
    }
```

Run `cargo test -p rustywx --lib text:: nhc::`. Expected: `truncate_chars` doesn't compile; once it exists, the NHC tests **panic** with `byte index ... is out of range` or `is not a char boundary`.

- [ ] **Step 3: Implement `truncate_chars`.** Insert above `#[cfg(test)]` in `text.rs`:

```rust
/// The longest prefix of `s` with at most `max_chars` chars.
pub fn truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((i, _)) => s.get(..i).unwrap_or(s),
        None => s,
    }
}
```

- [ ] **Step 4: Fix the NHC slices.** Replace `iso_to_ddhhmm` with:

```rust
fn iso_to_ddhhmm(iso: &str) -> Option<String> {
    let (date_part, time_part) = iso.split_once('T')?;
    let day = date_part.rsplit('-').next()?;
    let hour = time_part.get(0..2)?;
    let minute = time_part.get(3..5)?;
    let two_digits = |s: &str| s.len() == 2 && s.bytes().all(|b| b.is_ascii_digit());
    (two_digits(day) && two_digits(hour) && two_digits(minute)).then(|| format!("{day}{hour}{minute}"))
}
```

In `construct_image_products`, replace the `graphics_basin` / `bin` computation and the later `short_id` with a guarded version at the top of the function, after `let storm_id = meta.id.to_uppercase();`:

```rust
    // ATCF ids are BBNNYYYY; anything else (or non-ASCII) yields no products
    // instead of a byte-slice panic on the UI thread (review M21).
    let (Some(basin), Some(number), Some(year2)) =
        (storm_id.get(0..2), storm_id.get(2..4), storm_id.get(6..8))
    else {
        return products;
    };
    let graphics_basin = if basin == "AL" { "AT" } else { basin };
    let bin = format!("{graphics_basin}{number}");
```

and later `let short_id = format!("{graphics_basin}{number}{year2}");`. Delete the old `match storm_id.get(..2)` block and every `&storm_id[..]` slice.

- [ ] **Step 5: Fix the UI slices.**
  - `main.rs` NHC preview: replace the `let truncated = if product.content.len() > 300 { &product.content[..300] } else { &product.content };` block with `let truncated = rustywx::text::truncate_chars(&product.content, 300);`.
  - `scope.rs` alert label: replace the `let label = if alert.event.len() > 30 { &alert.event[..30] } else { &alert.event[..] };` block with `let label = crate::text::truncate_chars(&alert.event, 30);`.

- [ ] **Step 6: Sweep for the rest.** Run:

```bash
grep -rnE '\[[0-9]*\.\.[0-9a-z_.()]*\]|\[\.\.[0-9a-z_]+\]' app/src --include=*.rs | grep -v '#\[cfg(test)\]' | grep -v '^app/src/.*tests'
```

For each hit on a `&str`/`String` (not a slice of numbers), confirm it's either a literal-only or guarded slice (`nhc.rs:370` `bin_number[2..]` is guarded by its prefix check), or convert it to `str::get`/`truncate_chars`. List what you checked in the PR description.

- [ ] **Step 7: Verify and commit.** Run `cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings`, then the smoke test.

`git add app/src/text.rs && git commit -am "fix: truncate and parse text on char boundaries, never by byte index"`


---

# Phase 3 — Raster latency and border draw cost (PR `perf/raster-and-borders`)

Closes H3 and H4 (latency only; the H3 floor-repaint correctness note is Follow-up F6) and H5 for borders.

Coordinate with the `radar-improvments` worktree. It edits `scope.rs` (the shader region, l.16–90), which doesn't overlap these functions, but rebase before you start. If the 2026-08-27 plan's T10 benchmark already landed, skip Task 19 and use its numbers as the baseline.

### Task 19: A 2048 px benchmark with an enclosed hole (baseline first)

**Files:**
- Modify: `app/benches/rasterize.rs`

The existing benchmarks stop at 1024 px on a synthetic sweep with no enclosed hole, so they never exercise H3/H4 (review H4). This task records the "before" number for Tasks 21–22.

- [ ] **Step 1: Add the case.** In `app/benches/rasterize.rs`, add a helper and a second benchmark function, and register both:

```rust
use rustywx::model::{RadialData, SweepData};

/// 720 radials of 25–34 dBZ echo between 40 and 150 km, empty inside 40 km:
/// the "light rain all around the radar" case that makes fill_enclosed_holes
/// fill a ~180 px radius disk (review H3).
fn ring_sweep() -> SweepData {
    let radials = (0..720)
        .map(|i| RadialData {
            azimuth_deg: i as f32 * 0.5,
            gates: (0..GATES_PER_RADIAL)
                .map(|g| {
                    let km = 2.125 + g as f32 * 0.25;
                    (40.0..150.0).contains(&km).then(|| 25.0 + ((g * 7 + i * 3) % 10) as f32)
                })
                .collect(),
            range_folded: vec![],
        })
        .collect();
    SweepData {
        elevation_deg: 0.5,
        radials,
        first_gate_km: 2.125,
        gate_spacing_km: 0.25,
        nyquist_ms: 0.0,
    }
}

fn bench_production_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("rasterize_2048");
    group.sample_size(10);
    let sweep = ring_sweep();
    group.bench_function("ring_with_enclosed_hole", |b| {
        b.iter(|| {
            black_box(rasterize(
                black_box(&sweep),
                Product::Reflectivity,
                rustywx::scope::RASTER_SIZE_PX,
                MAX_RANGE_KM,
                &QcConfig {
                    tdbz_kernel_size: TDBZ_KERNEL_SIZE,
                    cc_sweep: None,
                    cc_gate_enabled: false,
                    cc_gate_threshold: 0.80,
                    refl_floor_enabled: false,
                    refl_floor_dbz: 7.0,
                    vel_dealias_enabled: false,
                    vel_sd_censor_enabled: false,
                    vel_sd_threshold: 7.0,
                    zdr_sweep: None,
                    phidp_sweep: None,
                    nonmet_fuzzy_enabled: false,
                    nonmet_threshold: 0.5,
                    refl_gap_fill_enabled: false,
                    multi_scale_texture_enabled: false,
                    sun_spike_removal_enabled: false,
                },
            ))
        });
    });
    group.finish();
}

criterion_group!(benches, bench_rasterize, bench_production_size);
```

Remove the old `criterion_group!` line. If `QcConfig` gained fields since this plan was written, copy them from the existing benchmark.

- [ ] **Step 2: Record the baseline.** Run `cargo bench -p rustywx --bench rasterize -- rasterize_2048 2>&1 | tee bench-before.txt` and put the median in the PR description. The review estimates multiple seconds, and audit F6 measured ~2.1–2.4 s.

- [ ] **Step 3: Commit.** `git commit -am "test(bench): rasterize at production size with an enclosed hole"`. Don't commit `bench-before.txt`.

---

### Task 20: Summed-area tables

**Files:**
- Create: `app/src/sat.rs`
- Modify: `app/src/lib.rs` (`pub mod sat;`)

**Interfaces:**
- Produces:
  - `pub struct CountSat`, with `new(mask: impl Fn(usize) -> bool, side: usize) -> Self` and `window(&self, x: usize, y: usize, r: usize) -> (u32, usize)` (count, clipped area)
  - `pub struct SumSat`, with `new_finite(values: &[f32], side: usize) -> Self` and `window(&self, x, y, r) -> f64` (sum of finite values)

Each window query is O(1) for a square `side × side` grid in row-major order.

- [ ] **Step 1: Write the failing tests.** Create `app/src/sat.rs`:

```rust
//! Summed-area tables: O(1) count/sum over a Chebyshev (square) window,
//! clipped to the grid, on a `side × side` row-major grid. Replaces the
//! O(r²)-per-pixel neighbourhood scans in scope.rs (review H4).

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic pseudo-random grid (no rand dependency).
    fn lcg_grid(side: usize, seed: u32) -> Vec<f32> {
        let mut s = seed;
        (0..side * side)
            .map(|_| {
                s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                if s >> 29 == 0 { f32::NAN } else { (s >> 16) as f32 / 1000.0 }
            })
            .collect()
    }

    fn brute(values: &[f32], side: usize, x: usize, y: usize, r: usize) -> (u32, usize, f64) {
        let (mut c, mut area, mut sum) = (0u32, 0usize, 0f64);
        for yy in y.saturating_sub(r)..(y + r + 1).min(side) {
            for xx in x.saturating_sub(r)..(x + r + 1).min(side) {
                area += 1;
                let v = values[yy * side + xx];
                if v.is_finite() {
                    c += 1;
                    sum += f64::from(v);
                }
            }
        }
        (c, area, sum)
    }

    #[test]
    fn windows_match_brute_force_including_edges() {
        for (side, seed) in [(1usize, 1u32), (7, 2), (33, 3), (64, 4)] {
            let v = lcg_grid(side, seed);
            let counts = CountSat::new(|i| v[i].is_finite(), side);
            let sums = SumSat::new_finite(&v, side);
            for r in [0usize, 1, 3, 8, 100] {
                for y in 0..side {
                    for x in 0..side {
                        let (c, area, sum) = brute(&v, side, x, y, r);
                        assert_eq!(counts.window(x, y, r), (c, area), "side={side} r={r} ({x},{y})");
                        assert!((sums.window(x, y, r) - sum).abs() < 1e-6 * sum.abs().max(1.0));
                    }
                }
            }
        }
    }
}
```

Add `pub mod sat;` to `lib.rs`. Run `cargo test -p rustywx --lib sat::`. Expected: compile error `cannot find type 'CountSat'`.

- [ ] **Step 2: Implement.** Insert above `#[cfg(test)]`:

```rust
/// Prefix counts: `data[(y)*(side+1)+x]` = cells set in rows `< y`, cols `< x`.
pub struct CountSat {
    side: usize,
    data: Vec<u32>,
}

impl CountSat {
    pub fn new(mask: impl Fn(usize) -> bool, side: usize) -> Self {
        let w = side + 1;
        let mut data = vec![0u32; w * w];
        for y in 0..side {
            let mut row = 0u32;
            for x in 0..side {
                row += u32::from(mask(y * side + x));
                data[(y + 1) * w + x + 1] = data[y * w + x + 1] + row;
            }
        }
        Self { side, data }
    }

    /// (set cells, cells) in the window of radius `r` around (x, y), clipped.
    pub fn window(&self, x: usize, y: usize, r: usize) -> (u32, usize) {
        let w = self.side + 1;
        let (x0, y0) = (x.saturating_sub(r), y.saturating_sub(r));
        let (x1, y1) = ((x + r + 1).min(self.side), (y + r + 1).min(self.side));
        let d = &self.data;
        // (D + A) - B - C never underflows: D >= B and D + A - B >= C.
        let count = d[y1 * w + x1] + d[y0 * w + x0] - d[y0 * w + x1] - d[y1 * w + x0];
        (count, (x1 - x0) * (y1 - y0))
    }
}

/// Prefix sums of the finite cells, in f64 (2048² sums stay exact enough).
pub struct SumSat {
    side: usize,
    data: Vec<f64>,
}

impl SumSat {
    pub fn new_finite(values: &[f32], side: usize) -> Self {
        let w = side + 1;
        let mut data = vec![0f64; w * w];
        for y in 0..side {
            let mut row = 0f64;
            for x in 0..side {
                let v = values[y * side + x];
                if v.is_finite() {
                    row += f64::from(v);
                }
                data[(y + 1) * w + x + 1] = data[y * w + x + 1] + row;
            }
        }
        Self { side, data }
    }

    /// Sum of finite values in the window of radius `r` around (x, y), clipped.
    pub fn window(&self, x: usize, y: usize, r: usize) -> f64 {
        let w = self.side + 1;
        let (x0, y0) = (x.saturating_sub(r), y.saturating_sub(r));
        let (x1, y1) = ((x + r + 1).min(self.side), (y + r + 1).min(self.side));
        let d = &self.data;
        d[y1 * w + x1] - d[y0 * w + x1] - d[y1 * w + x0] + d[y0 * w + x0]
    }
}
```

- [ ] **Step 3: Run the tests to verify they pass.** Run `cargo test -p rustywx --lib sat::`. Expected: 1 passed.

- [ ] **Step 4: Commit.** `git add app/src/sat.rs app/src/lib.rs && git commit -m "feat(raster): summed-area tables for window counts and sums"`

---

### Task 21: Morphology and smoothing on SATs, proven equal to the old code (H4)

**Files:**
- Modify: `app/src/scope.rs`:
  - `morphological_close` (~l.970)
  - the seal (dilate and erode) section of `fill_enclosed_holes` (~l.1110–1162)
  - `smooth_values` (~l.1246)
  - tests

**Interfaces:**
- Consumes: `sat::{CountSat, SumSat}`.
- Produces: the same private function signatures. These are behavior-preserving: exact for both masks, within 1e-4 relative for the f64 blur.

- [ ] **Step 1: Freeze the reference implementations.** Copy the **current** bodies of `morphological_close`, `fill_enclosed_holes`, and `smooth_values` verbatim into `scope.rs`'s `#[cfg(test)] mod tests`, renamed `morphological_close_reference`, `fill_enclosed_holes_reference`, and `smooth_values_reference`. They stay there permanently as the oracle.

- [ ] **Step 2: Write the equivalence tests.** Add to `mod tests`:

```rust
    /// Deterministic grid with NaN gaps, solid blobs, and an enclosed hole.
    fn test_grid(side: usize, seed: u32) -> Vec<f32> {
        let mut s = seed;
        let mut v: Vec<f32> = (0..side * side)
            .map(|_| {
                s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                if s >> 30 == 0 { f32::NAN } else { 5.0 + (s >> 20) as f32 / 100.0 }
            })
            .collect();
        // Clear a border band (open air) and punch an enclosed hole.
        for y in 0..side {
            for x in 0..side {
                let edge = x < 3 || y < 3 || x + 3 >= side || y + 3 >= side;
                let (dx, dy) = (x as i32 - side as i32 / 2, y as i32 - side as i32 / 2);
                if edge || dx * dx + dy * dy < (side as i32 / 6).pow(2) {
                    v[y * side + x] = f32::NAN;
                }
            }
        }
        v
    }

    fn same_bits(a: &[f32], b: &[f32]) -> bool {
        a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits() || (x.is_nan() && y.is_nan()))
    }

    #[test]
    fn sat_fill_enclosed_holes_matches_reference_exactly() {
        for (side, seed, r) in [(31usize, 1u32, 0usize), (48, 2, 1), (64, 3, 2), (64, 4, 8)] {
            let mut fast = test_grid(side, seed);
            let mut slow = fast.clone();
            fill_enclosed_holes(&mut fast, side, r);
            fill_enclosed_holes_reference(&mut slow, side, r);
            assert!(same_bits(&fast, &slow), "side={side} seed={seed} r={r}");
        }
    }

    #[test]
    fn sat_smooth_values_matches_reference() {
        for (side, seed, r) in [(31usize, 5u32, 1usize), (64, 6, 3)] {
            let mut fast = test_grid(side, seed);
            let mut slow = fast.clone();
            smooth_values(&mut fast, side, r);
            smooth_values_reference(&mut slow, side, r);
            for (a, b) in fast.iter().zip(&slow) {
                assert_eq!(a.is_nan(), b.is_nan(), "coverage preserved");
                if a.is_finite() {
                    assert!((a - b).abs() <= 1e-4 * b.abs().max(1.0), "{a} vs {b}");
                }
            }
        }
    }

    #[test]
    fn sat_morphological_close_matches_reference_exactly() {
        for (side, seed, r) in [(31usize, 7u32, 1usize), (64, 8, 4)] {
            let v = test_grid(side, seed);
            let mut pixels: Vec<u8> = v
                .iter()
                .flat_map(|x| if x.is_finite() { [200, 100, 50, 255] } else { [0, 0, 0, 0] })
                .collect();
            let mut slow = pixels.clone();
            morphological_close(&mut pixels, side, r);
            morphological_close_reference(&mut slow, side, r);
            assert_eq!(pixels, slow, "side={side} r={r}");
        }
    }
```

Run `cargo test -p rustywx --lib scope::tests::sat_`. Expected: PASS. These are characterization tests: they pass against the untouched code, then guard the rewrite in Steps 3–5.

- [ ] **Step 3: Rewrite `smooth_values`.**

```rust
fn smooth_values(values: &mut [f32], size_px: usize, radius: usize) {
    // O(1) per pixel via summed-area tables of the finite cells (review H4).
    let counts = crate::sat::CountSat::new(|i| values[i].is_finite(), size_px);
    let sums = crate::sat::SumSat::new_finite(values, size_px);
    for (i, v) in values.iter_mut().enumerate() {
        if !v.is_finite() {
            continue;
        }
        let (x, y) = (i % size_px, i / size_px);
        let (count, _) = counts.window(x, y, radius);
        *v = (sums.window(x, y, radius) / f64::from(count)) as f32;
    }
}
```

Keep the doc comment above it.

- [ ] **Step 4: Rewrite the seal in `fill_enclosed_holes`.** Replace everything from `let finite: Vec<bool> = ...` through `let sealed = closed;` with:

```rust
    // Sealed coverage (dilate): any finite value within Chebyshev distance
    // seal_radius, window clipped to the grid. O(1)/px via a SAT (review H4).
    let finite_sat = crate::sat::CountSat::new(|i| values[i].is_finite(), size_px);
    let sealed: Vec<bool> = (0..n)
        .map(|i| finite_sat.window(i % size_px, i / size_px, seal_radius).0 > 0)
        .collect();

    // Erode back by the same radius: a cell stays only if its whole window is
    // inside the grid and sealed (out-of-bounds counts as unsealed, so nothing
    // phantom-seals at the edge). See the comment history for why erosion is
    // required.
    let sealed_sat = crate::sat::CountSat::new(|i| sealed[i], size_px);
    let full = (2 * seal_radius + 1) * (2 * seal_radius + 1);
    let sealed: Vec<bool> = (0..n)
        .map(|i| {
            let (x, y) = (i % size_px, i / size_px);
            sealed[i]
                && x >= seal_radius
                && y >= seal_radius
                && x + seal_radius < size_px
                && y + seal_radius < size_px
                && sealed_sat.window(x, y, seal_radius).0 as usize == full
        })
        .collect();
```

Keep the explanatory comment block about why the erode exists (move it above the second `let sealed`). Delete `let r = seal_radius as i32;` at the top of the function: nothing uses `r` after this change, and `-D warnings` fails on it.

- [ ] **Step 5: Rewrite `morphological_close`.** Replace the dilate and erode loops (everything before the final "copy nearest original pixel" loop) with:

```rust
    let s = size_px as i32;
    let r = radius as i32;
    let n = size_px * size_px;
    let original = pixels.to_vec();
    let mask: Vec<bool> = (0..n).map(|i| original[i * 4 + 3] != 0).collect();

    // Dilate: any covered cell in the clipped window (SAT, O(1)/px; review H4).
    let mask_sat = crate::sat::CountSat::new(|i| mask[i], size_px);
    let dilated: Vec<bool> = (0..n)
        .map(|i| mask[i] || mask_sat.window(i % size_px, i / size_px, radius).0 > 0)
        .collect();
    // Erode: every in-bounds cell of the clipped window dilated (out-of-bounds
    // ignored here, unlike fill_enclosed_holes).
    let dilated_sat = crate::sat::CountSat::new(|i| dilated[i], size_px);
    let closed: Vec<bool> = (0..n)
        .map(|i| {
            let (count, area) = dilated_sat.window(i % size_px, i / size_px, radius);
            dilated[i] && count as usize == area
        })
        .collect();
```

Keep the existing final loop (nearest covered pixel copy for `closed[idx] && !mask[idx]`) unchanged. It touches only the few closed-in cells.

- [ ] **Step 6: Run the tests.** Run `cargo test -p rustywx --lib scope::`. Expected: the three `sat_*` equivalence tests and every existing scope test pass. Re-run Task 19's benchmark and record the new median.

- [ ] **Step 7: Commit.** `git commit -am "perf(raster): O(1)-per-pixel morphology and smoothing via summed-area tables"`

---

### Task 22: Ring-by-ring BFS hole fill (H3)

**Files:**
- Modify: `app/src/scope.rs` (the final `loop { let snapshot = values.to_vec(); ... }` in `fill_enclosed_holes`, plus a new `fn neighbor_mean`)

- [ ] **Step 1: Confirm the guard test.** `sat_fill_enclosed_holes_matches_reference_exactly` (Task 21) compares against the frozen old implementation bit for bit. It's the test for this task, and it must keep passing.

- [ ] **Step 2: Implement.** Add next to `fill_enclosed_holes`:

```rust
/// Mean of the finite 8-neighbours (and self) of (x, y), in the same order
/// the old snapshot loop summed them, so results are bit-identical.
fn neighbor_mean(values: &[f32], s: i32, x: i32, y: i32) -> Option<f32> {
    let mut sum = 0.0f32;
    let mut count = 0u32;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || nx >= s || ny < 0 || ny >= s {
                continue;
            }
            let v = values[(ny * s + nx) as usize];
            if v.is_finite() {
                sum += v;
                count += 1;
            }
        }
    }
    (count > 0).then(|| sum / count as f32)
}
```

Replace the final `loop { ... }` with:

```rust
    // Hole = NaN and not open air. Fill ring by ring, outermost first. Each
    // ring is averaged from the grid as it stood before that ring (exactly
    // what the old snapshot-per-pass loop produced), but only hole cells are
    // visited: O(hole cells) total instead of a full-grid copy and scan per
    // ring (review H3: ~180 passes of 16 MB for a 40 km clear disk).
    let xy = |i: usize| ((i as i32) % s, (i as i32) / s);
    let is_hole = |values: &[f32], i: usize| values[i].is_nan() && !open[i];
    let mut queued = vec![false; n];
    let mut ring: Vec<usize> = (0..n)
        .filter(|&i| {
            let (x, y) = xy(i);
            is_hole(values, i) && neighbor_mean(values, s, x, y).is_some()
        })
        .collect();
    for &i in &ring {
        queued[i] = true;
    }
    while !ring.is_empty() {
        let fills: Vec<(usize, f32)> = ring
            .iter()
            .filter_map(|&i| {
                let (x, y) = xy(i);
                neighbor_mean(values, s, x, y).map(|v| (i, v))
            })
            .collect();
        for &(i, v) in &fills {
            values[i] = v;
        }
        let mut next = Vec::new();
        for &(i, _) in &fills {
            let (x, y) = xy(i);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || nx >= s || ny < 0 || ny >= s {
                        continue;
                    }
                    let j = (ny * s + nx) as usize;
                    if !queued[j] && is_hole(values, j) {
                        queued[j] = true;
                        next.push(j);
                    }
                }
            }
        }
        ring = next;
    }
```

Why this is identical to the old loop: old pass *k* fills every hole cell that has a finite neighbor in the snapshot. A cell gets its first finite neighbor either originally (so it's in the first ring) or when a neighbor is filled in ring *k* (so it's queued for ring *k+1*). The membership and the summation order match.

- [ ] **Step 3: Run the tests.** Run `cargo test -p rustywx --lib scope::`. Expected: all pass, including `sat_fill_enclosed_holes_matches_reference_exactly` and the existing `fill_enclosed_holes_*` tests. Re-run the benchmark and record the median. The PR description gets a three-row table: before, after Task 21, after Task 22.

- [ ] **Step 4: Smoke and visual check.** Run the smoke test, then `cargo run --release -p rustywx -- --demo harvey2017`. The reflectivity image must look the same as on `main`; take a screenshot of each for the PR. The status line updates after a tilt or product click noticeably faster.

- [ ] **Step 5: Commit.** `git commit -am "perf(raster): fill enclosed holes ring by ring instead of re-scanning the grid"`

---

### Task 23: Border projection cache via a shared `geo::KmPolyline` (H5)

**Files:**
- Modify: `app/src/geo.rs` (add `KmPolyline`)
- Modify: `app/src/scope.rs` (`draw_borders` ~l.1612; `draw_scope_to_texture` `borders` parameter)
- Modify: `app/src/state.rs` (`borders_revision`, `border_projection`)
- Modify: `app/src/main.rs` (bump the revision at both `state.borders = rings;` sites, l.1031 and l.1063; rebuild before draw ~l.1588)
- Modify (if `feat/spaghetti-models` is merged): `app/src/adeck_style.rs`, `scope.rs` `draw_model_tracks` (Step 6)

**Interfaces:**
- Produces:
  - `#[derive(Clone, Debug, PartialEq)] pub struct geo::KmPolyline { pub km: Vec<Vec2>, pub min_km: Vec2, pub max_km: Vec2 }`
  - `KmPolyline::from_latlon(points: &[(f64, f64)], origin_lat: f64, origin_lon: f64) -> Self`
  - `KmPolyline::visible(&self, center: Vec2, px_per_km: f32, screen: Vec2, margin: f32) -> bool`
  - `draw_scope_to_texture(.., borders: Option<(&[geo::KmPolyline], bool)>, ..)`

If the 2026-08-27 plan's `overlay_cache` (its T5/T11) landed first, project `overlay_cache.rings` instead of `state.borders` in Step 4; everything else is the same.

- [ ] **Step 1: Write the failing tests.** Add to `geo.rs` tests (create `#[cfg(test)] mod tests { use super::*; }` if absent):

```rust
    #[test]
    fn km_polyline_projects_and_bounds() {
        let line = KmPolyline::from_latlon(&[(33.0, -84.0), (34.0, -84.0), (33.5, -83.0)], 33.0, -84.0);
        assert!(line.km[0].length() < 0.01, "origin projects to (0,0)");
        assert!(line.km[1].y < -100.0, "north is up (negative y)");
        for p in &line.km {
            assert!(p.x >= line.min_km.x && p.x <= line.max_km.x);
            assert!(p.y >= line.min_km.y && p.y <= line.max_km.y);
        }
        assert_eq!(line.km[2], point_to_km_offset(33.0, -84.0, (33.5, -83.0)));
    }

    #[test]
    fn km_polyline_visibility_culls_by_bbox() {
        let line = KmPolyline::from_latlon(&[(33.0, -84.0), (33.1, -84.0)], 33.0, -84.0);
        let screen = Vec2::new(800.0, 600.0);
        assert!(line.visible(Vec2::new(400.0, 300.0), 2.0, screen, 50.0));
        assert!(!line.visible(Vec2::new(-5000.0, 300.0), 2.0, screen, 50.0));
        assert!(!line.visible(Vec2::new(400.0, 9000.0), 2.0, screen, 50.0));
        let empty = KmPolyline::from_latlon(&[], 33.0, -84.0);
        assert!(empty.km.is_empty());
    }
```

Run `cargo test -p rustywx --lib geo::`. Expected: compile error `cannot find type 'KmPolyline'`.

- [ ] **Step 2: Implement.** Add to `geo.rs` after `point_to_km_offset`:

```rust
/// A lat/lon polyline pre-projected to km offsets from a site, with its km
/// bounding box. Rebuilt only when the site or data changes; per frame the
/// draw is a bbox cull plus one multiply-add per vertex (review H5). Shared
/// with the spaghetti-models track cache.
#[derive(Clone, Debug, PartialEq)]
pub struct KmPolyline {
    pub km: Vec<Vec2>,
    pub min_km: Vec2,
    pub max_km: Vec2,
}

impl KmPolyline {
    pub fn from_latlon(points: &[(f64, f64)], origin_lat: f64, origin_lon: f64) -> Self {
        let km: Vec<Vec2> = points
            .iter()
            .map(|&p| point_to_km_offset(origin_lat, origin_lon, p))
            .collect();
        let min_km = km.iter().copied().reduce(Vec2::min).unwrap_or(Vec2::ZERO);
        let max_km = km.iter().copied().reduce(Vec2::max).unwrap_or(Vec2::ZERO);
        Self { km, min_km, max_km }
    }

    /// Whether the bbox, in screen pixels, comes within `margin` of the screen.
    pub fn visible(&self, center: Vec2, px_per_km: f32, screen: Vec2, margin: f32) -> bool {
        let lo = center + self.min_km * px_per_km;
        let hi = center + self.max_km * px_per_km;
        hi.x >= -margin && lo.x <= screen.x + margin && hi.y >= -margin && lo.y <= screen.y + margin
    }
}
```

Run `cargo test -p rustywx --lib geo::`. Expected: pass.

- [ ] **Step 3: Draw from the cache.** In `scope.rs`, change the `draw_scope_to_texture` parameter to `borders: Option<(&[geo::KmPolyline], bool)>`, the call to `draw_borders(lines, center_x, center_y, px_per_km)`, and replace `draw_borders` with:

```rust
/// Draw pre-projected border/coastline polylines across the full window.
fn draw_borders(lines: &[geo::KmPolyline], center_x: f32, center_y: f32, px_per_km: f32) {
    let border_color = MacroquadColor::from_rgba(0x8b, 0x73, 0x55, 180);
    let sw = screen_width();
    let sh = screen_height();
    let margin = 50.0;
    let center = Vec2::new(center_x, center_y);
    let screen = Vec2::new(sw, sh);
    for line in lines {
        if !line.visible(center, px_per_km, screen, margin) {
            continue;
        }
        for pair in line.km.windows(2) {
            let (ax, ay) = (center_x + pair[0].x * px_per_km, center_y + pair[0].y * px_per_km);
            let (bx, by) = (center_x + pair[1].x * px_per_km, center_y + pair[1].y * px_per_km);
            if (ax < -margin && bx < -margin)
                || (ax > sw + margin && bx > sw + margin)
                || (ay < -margin && by < -margin)
                || (ay > sh + margin && by > sh + margin)
            {
                continue;
            }
            draw_line(ax, ay, bx, by, 1.0, border_color);
        }
    }
}
```

- [ ] **Step 4: Cache in state.** In `state.rs`, after `pub borders: Vec<Ring>,`:

```rust
    /// Bumped whenever `borders` is replaced (cache load or fresh parse).
    pub borders_revision: u64,
    /// `borders` projected for the current site, keyed by (site index,
    /// borders_revision). Rebuilt only when either changes (review H5).
    pub border_projection: Option<(usize, u64, Vec<crate::geo::KmPolyline>)>,
```

Initialize as `0` and `None`. At both `state.borders = rings;` sites in `main.rs`, add `state.borders_revision += 1;`. Before the `scope::draw_scope_to_texture(` call:

```rust
            let border_key = (state.site_index, state.borders_revision);
            if state.border_projection.as_ref().map(|(s, r, _)| (*s, *r)) != Some(border_key) {
                let lines = state
                    .borders
                    .iter()
                    .filter(|ring| ring.len() >= 2)
                    .map(|ring| geo::KmPolyline::from_latlon(ring, site.lat, site.lon))
                    .collect();
                state.border_projection = Some((border_key.0, border_key.1, lines));
            }
```

Change the argument `Some((&state.borders, state.show_borders))` to `state.border_projection.as_ref().map(|(_, _, lines)| (lines.as_slice(), state.show_borders))`.

- [ ] **Step 5: Verify.** Run `cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings`, then the smoke test.

Borders must look the same at every zoom: pan to a coastline, zoom out to 0.05, then switch sites.

Measure: temporarily wrap the `draw_scope_to_texture` call in `let t = std::time::Instant::now(); ... eprintln!("draw_ms={:.2}", t.elapsed().as_secs_f64()*1e3)` on a release build, on `main` and on this branch, at default zoom with borders on. Report both medians (audit F3 measured 5.9–8.7 ms before), then remove the timing.

- [ ] **Step 6: Reuse in the spaghetti track cache (only if `feat/spaghetti-models` is merged).** In `adeck_style.rs`:
  1. Change `ProjectedTrack` to `{ pub tech: String, pub group: ModelGroup, pub line: crate::geo::KmPolyline, pub vmax_kt: Vec<Option<u32>> }`.
  2. Build it in `project_tracks` with `line: KmPolyline::from_latlon(&points_latlon, origin_lat, origin_lon)`, where `points_latlon: Vec<(f64, f64)>` is `t.points.iter().map(|p| (p.lat, p.lon)).collect()`.
  3. In `scope.rs` `draw_model_tracks`, replace the bbox computation with `if !track.line.visible(Vec2::new(center_x, center_y), px_per_km, Vec2::new(sw, sh), margin) { continue; }` and `track.km` with `track.line.km`.
  4. Update the three `adeck_style` projection tests' field access the same way (`t.km` → `t.line.km`, `t.min_km` → `t.line.min_km`).

  All `adeck_style::` tests must pass unchanged in substance.

- [ ] **Step 7: Commit.** `git commit -am "perf(render): project borders once per site and cull by bounding box"`


---

# Phase 4 — Windows CI and packaging (PR `ci/windows`)

**Prerequisite:** Task 16 (`--version`) merged. Closes H7, L7, L8, and L9; covers M5's manifest step and M19 a, b, d plus the checksum.

Tasks 25 and 26 change the Windows link and are only provable on Windows. The CI job from Task 24 is their gate.

### Task 24: `windows-latest` CI job, `deny.toml` targets, explicit tokio runtime, Justfile on Windows

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `deny.toml` (`[graph].targets`)
- Modify: `app/Cargo.toml` (tokio features)
- Modify: `Justfile`
- Modify: `AGENTS.md` (smoke test)

- [ ] **Step 1: L9.** In `app/Cargo.toml`, change tokio to `tokio = { version = "1", features = ["rt", "rt-multi-thread", "time", "sync"] }`. `Runtime::new()` needs `rt-multi-thread`, which today arrives only through feature unification. Run `cargo check -p rustywx`.

- [ ] **Step 2: L8.** In `deny.toml`, make `[graph].targets`:

```toml
targets = [
    { triple = "x86_64-unknown-linux-gnu" },
    { triple = "x86_64-pc-windows-msvc" },
    { triple = "aarch64-apple-darwin" },
    { triple = "wasm32-unknown-unknown" },
]
```

Run `cargo deny check`. Fix any new license or advisory findings for Windows-only crates by adding them to the existing `[licenses]`/`[advisories]` sections with a comment; don't loosen the global policy.

- [ ] **Step 3: H7, the CI job.** Append to `.github/workflows/ci.yml` under `jobs:`, reusing the SHA pins the file already uses:

```yaml
  # ── Windows: build, test, and a launch check (review H7) ──────
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
      # The hosted runner has no GPU/OpenGL, so never launch the GUI here.
      # `--version` exits before window creation and still proves the exe
      # loads (static CRT, manifest, resources). Piping makes pwsh wait for
      # the GUI-subsystem exe and captures its stdout.
      - name: Launch check (--version)
        shell: pwsh
        run: |
          $out = & ./target/release/rustywx.exe --version | Out-String
          if ($LASTEXITCODE -ne 0 -or $out -notmatch 'rustywx \d') {
            Write-Error "rustywx.exe --version failed (exit $LASTEXITCODE): $out"
            exit 1
          }
          Write-Output $out
```

- [ ] **Step 4: L7, the Justfile.** Add near the top of `Justfile`:

```just
# Windows: use PowerShell 7 (`&&` works; Windows PowerShell 5.1 lacks it).
set windows-shell := ["pwsh.exe", "-NoLogo", "-NoProfile", "-Command"]
```

Change `run:` to `cargo run --release -p rustywx` (no `cd`). Mark `wayland-smoke` with `[linux]`, and `setup` and `setup-tools` with `[unix]`. Add a cross-platform smoke recipe:

```just
# AGENTS.md smoke test: launch, wait, confirm still running, kill.
[unix]
smoke:
    cargo build --release -p rustywx
    ./target/release/rustywx & pid=$!; sleep 3; kill -0 $pid && kill $pid

[windows]
smoke:
    cargo build --release -p rustywx
    $p = Start-Process -FilePath target/release/rustywx.exe -PassThru; Start-Sleep 5; if ($p.HasExited) { Write-Error "rustywx exited early (code $($p.ExitCode)); see $env:LOCALAPPDATA\rustywx\app.log and crash.log"; exit 1 }; Stop-Process -Id $p.Id
```

Run `just --list` on macOS. It should show `smoke` once and no `wayland-smoke` warnings.

**[verify on Windows]** (Appendix B, item 6): `just smoke` passes with pwsh 7 and `just --version` ≥ 1.20; the `windows-shell` setting is accepted.

- [ ] **Step 5: AGENTS.md.** Under the mandatory smoke test, add:

```markdown
`just smoke` runs the same check on every platform, including Windows
(PowerShell 7). On a Windows **release** build there is no console:
stdout/stderr go to `%LOCALAPPDATA%\rustywx\app.log`, and a panic writes
`%LOCALAPPDATA%\rustywx\crash.log` and shows a message box.
```

- [ ] **Step 6: Verify and commit.** Push the branch. The `windows` job must be green before Tasks 25–26 land.

`git commit -am "ci: build, test, and launch-check on windows-latest"`

---

### Task 25: Windows manifest, exe icon, and version resource (M5 manifest, M19b)

**Files:**
- Create: `app/build.rs`
- Create: `app/assets/icon/rustywx.ico`
- Modify: `app/Cargo.toml` (`build = "build.rs"` if needed; `[build-dependencies]`)

- [ ] **Step 1: Make the `.ico`.** From the repo root (ImageMagick 7):

```bash
magick app/assets/icon/icon_16.png app/assets/icon/icon_32.png app/assets/icon/icon_64.png \
       app/assets/icon/icon_128.png app/assets/icon/icon_256.png app/assets/icon/rustywx.ico
file app/assets/icon/rustywx.ico   # expect: MS Windows icon resource - 5 icons
```

If ImageMagick isn't available, any tool that packs those PNGs into one `.ico` is fine. Commit the `.ico`.

- [ ] **Step 2: Build dependencies.** In `app/Cargo.toml`:

```toml
[build-dependencies]
embed-manifest = "1.5"
winresource = "0.1.31"
```

- [ ] **Step 3: `app/build.rs`.**

```rust
//! Windows-only resources (no-op elsewhere): an application manifest
//! (DPI awareness, long paths, UTF-8 code page; review M5) and the exe
//! icon plus VERSIONINFO (review M19b). See
//! docs/research/windows-stability-remediation.md section C.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/icon/rustywx.ico");
    // CARGO_CFG_WINDOWS is the *target* OS, so this also works when cross-compiling.
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        windows_resources();
    }
}

fn windows_resources() {
    use embed_manifest::manifest::{ActiveCodePage, DpiAwareness, Setting};
    use embed_manifest::{embed_manifest, new_manifest};

    // Step 1 of the DPI staging (research section B): `System` matches
    // today's behaviour explicitly. PerMonitorV2 waits for a WM_DPICHANGED
    // handler in the miniquad patch (Follow-up F7).
    embed_manifest(
        new_manifest("hatek.rustywx")
            .dpi_awareness(DpiAwareness::System)
            .long_path_aware(Setting::Enabled)
            .active_code_page(ActiveCodePage::Utf8),
    )
    .expect("embedding the Windows application manifest");

    // Icon and version info only. Never let winresource embed a manifest too:
    // two manifests fail the link with CVT1100 (embed-manifest owns it).
    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/icon/rustywx.ico")
        .set("ProductName", "rustywx")
        .set("FileDescription", "rustywx NEXRAD radar viewer")
        .set("LegalCopyright", "AGPL-3.0-only");
    res.compile().expect("compiling Windows resources (needs rc.exe)");
}
```

winresource fills `FileVersion`/`ProductVersion` from `CARGO_PKG_VERSION` by default.

- [ ] **Step 4: Verify.** On macOS or Linux, `cargo build -p rustywx` and the smoke test behave as before (`build.rs` is a no-op). Push; the Windows CI job must stay green, which proves the link (no CVT1100 and no LNK errors).

**[verify on Windows]** (Appendix B, item 7):
- Explorer shows the rustywx icon on `rustywx.exe`.
- Properties > Details shows the product name and version.
- `mt.exe -inputresource:rustywx.exe;#1 -out:m.xml`, or Resource Hacker, shows `dpiAware`, `longPathAware`, and `activeCodePage UTF-8`.

- [ ] **Step 5: Commit.** `git add app/build.rs app/assets/icon/rustywx.ico && git commit -am "build(windows): embed manifest, icon, and version resource"`

---

### Task 26: Static CRT and release packaging fixes (M19a, M19d, checksum)

**Files:**
- Create: `.cargo/config.toml`
- Modify: `.github/workflows/release-please.yml`:
  - build step
  - "Package (unix)" ~l.243–257
  - "Package (windows)" ~l.260–273
  - upload
- Modify: `README.md` (~l.142–143)

- [ ] **Step 1: Static CRT.** Create `.cargo/config.toml`:

```toml
# Windows: link the MSVC C runtime statically so rustywx.exe starts on a
# clean machine without the VC++ Redistributable (review M19a). C code built
# through `cc` honours crt-static.
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]
```

Push. The Windows CI job must stay green; an LNK2038 error means a mixed `/MD`/`/MT` dependency. **[verify on Windows]** (Appendix B, item 8): on a clean Windows VM without the VC++ redistributable, the release exe starts.

- [ ] **Step 2: Release launch check.** In `release-please.yml`, after the release `cargo build` step, add:

```yaml
      - name: Launch check (windows)
        if: runner.os == 'Windows'
        shell: pwsh
        run: |
          $out = & "target/${{ matrix.target }}/release/rustywx.exe" --version | Out-String
          if ($LASTEXITCODE -ne 0 -or $out -notmatch 'rustywx \d') { Write-Error "launch check failed: $out"; exit 1 }
```

- [ ] **Step 3: M19d, drop the dead `assets/` copy.** Delete `cp -R app/assets "$STAGE/assets"` from "Package (unix)" and `Copy-Item -Recurse app/assets "$STAGE/assets"` from "Package (windows)". Every font, icon, and shader is `include_bytes!`. Update `README.md` around l.142–143, removing the note that the app "launches from `app/` so runtime assets resolve correctly".

- [ ] **Step 4: Windows checksum.** At the end of "Package (windows)", add:

```powershell
          $hash = (Get-FileHash -Algorithm SHA256 "$STAGE.zip").Hash.ToLower()
          Set-Content -Path "$STAGE.zip.sha256" -Value "$hash  $STAGE.zip" -NoNewline
          Add-Content -Path $env:GITHUB_ENV -Value "CHECKSUM_PATH=$STAGE.zip.sha256"
```

After "Upload release asset", add:

```yaml
      - name: Upload checksum (windows)
        if: runner.os == 'Windows'
        uses: softprops/action-gh-release@3bb12739c298aeb8a4eeaf626c5b8d85266b0e65  # v2
        with:
          tag_name: ${{ github.event_name == 'workflow_dispatch' && inputs.tag_name || needs.release-please.outputs.tag_name }}
          files: ${{ env.CHECKSUM_PATH }}
```

The format matches `sha256sum`, so `sha256sum -c` works on it.

- [ ] **Step 5: Verify.** Run `actionlint .github/workflows/*.yml` if installed. Push; CI must be green. To exercise the release path before a real release, run the workflow with `workflow_dispatch` against a throwaway tag on a fork, or review the diff line by line with wx-team-lead.

- [ ] **Step 6: Commit.** `git add .cargo/config.toml && git commit -am "build(windows): static CRT, release launch check, checksum, drop dead assets copy"`

---

# Phase 5 — App-owned HTTP module (follow-up; outline, plan separately)

**Goal:** Replace ply `net` for every feed with `app/src/http.rs` on reqwest 0.13. reqwest is already in the build via nexrad-data. This gives real per-request timeouts (C3), the Windows certificate store (H1; reqwest's `rustls` feature uses `rustls-platform-verifier`), the system proxy (H1; the `system-proxy` feature reads the registry), true cancellation (`JoinHandle::abort()` frees the socket), and conditional GETs through response headers (M13, and spaghetti's ETag follow-up).

**Why it's cheap after Phase 1b:** every feed already talks to `net_port::NetPort`. Phase 5 adds one implementation, `HttpNet`, and swaps `PlyNet` for it where `ply_net` is constructed in `main.rs` and in `handle_input`. `net_feed`, `nhc`, `adeck_fetch`, and the parsers don't change.

**Design sketch** (research section A, "End state"):
- Dependency: `reqwest = { version = "0.13", default-features = false, features = ["rustls", "system-proxy", "gzip"] }`.
- One `reqwest::Client`, built once with `connect_timeout(10 s)`, `timeout(30 s)` (60 s for the borders and a-deck bodies, via per-request `.timeout()`), and the User-Agent.
- `HttpNet` holds `HashMap<String, Slot>`, where `Slot = InFlight(tokio::task::JoinHandle<Result<Vec<u8>, String>>) | Done(Result<..>)`, spawned on the existing multi-thread runtime.
  - `get_with_headers` spawns the request.
  - `poll` checks `JoinHandle::is_finished()` and takes the result without blocking.
  - `cancel` calls `abort()` and removes the slot.
  - There's no 60-frame eviction and no id de-dupe, so `Missing` only means "never fired".
- A body cap (`Response::chunk` loop up to 16 MiB), so the 10 MiB ureq limit (review M14) is a deliberate choice, not an accident.
- `NetPoll::Done(Err(..))` strings keep `"404"` for not-found so `Feed`'s quiet-404 rule still works. Alternatively, extend `NetPoll` with a status code then.
- Conditional GET: an `etag`/`last_modified` per id inside `HttpNet`; a 304 becomes `Done(Ok(previous_body))` or a new `NetPoll::NotModified`. Decide in the detailed plan.
- Keep `PlyNet` behind `cfg(target_arch = "wasm32")` only if a web build is still a goal (the research found no active one).
- Remove the Task 14 `ureq` feature line and the `USER_GUIDE.md` proxy caveat about PAC once verified.

**Tasks to detail when scheduled:** (1) `HttpNet` with tests against a local `tokio` TCP listener that never responds, proving abort and timeout; (2) swap the transport behind a setting or env var for a soak period; (3) conditional GET; (4) remove ureq's direct dependency; (5) **[verify on Windows]** behind TLS inspection (corporate CA in the Windows store) and behind a registry proxy.

---

## Follow-ups (not in these PRs)

| Id | Item | Why deferred |
|---|---|---|
| F1 | Port `adeck_fetch::AdeckFetcher` onto `net_feed::Feed` (a `BTreeMap<String, Feed>`), gated on its 13 tests passing unchanged | Same logic, two copies; deferred to keep PR 1b small |
| F2 | ProjectedTrack refactor | Done in Task 23 Step 6 if spaghetti is merged; otherwise do it when spaghetti lands |
| F3 | Debounce rapid site switches (~250 ms) and cancel superseded cache loads early (M16 remainder) | Loads are now on the blocking pool and results are dropped when superseded; the debounce is UX polish |
| F4 | Radar data worker: `tokio::time::timeout` around list and download, a `select!` on site changes (C3 radar half) | Different client (nexrad-data/reqwest); its own PR |
| F5 | Move the demo-volume blob (`demo_volume_*`, ~20 MB) from Roaming to `fsutil::cache_root()` with `write_atomic` | Rarely written; small |
| F6 | H3 correctness: never fill pixels the range floor nulled (needs a "floor-nulled" mask) | Behavior change; needs a product decision and before/after images |
| F7 | `WM_DPICHANGED` handler in the miniquad patch, then switch the manifest to `PerMonitorV2`; `is_mobile` breakpoint on logical size; DPI-scaled initial window clamped to the work area (M5 remainder) | **[verify on Windows]** with mixed-DPI monitors; staged per research section B |
| F8 | Ship `rustywx.pdb` in the Windows zip (`debug = "line-tables-only"`), so `crash.log` backtraces symbolicate | Profile change affects every platform's binary size; decide separately |
| F9 | Code signing via SignPath Foundation (research section C) | Application and approval process |
| F10 | Existing AGENTS.md icon violations: `"✓"` in `nhc_toggle_button` (main.rs:3243), `"Tropical ✓"` (main.rs:1833), `"🌐 "` NHC modal title (main.rs:2762; review L5) | Out of scope here, also noted by the spaghetti plan; one small `fix:` PR |
| F11 | M2 (raster retry cap, keep the old texture), M17 (worker `catch_unwind`), M18 (shader fallback), M20 (KMZ cap) | Owned by the 2026-08-27 audit plan (see the overlap table) |
| F12 | M6, M7, M8, M9, M10, M11, M12, M13 (NHC skip-if-unchanged, concurrency cap), M14, M24, M25, M26, M27, the remaining Lows | Reviewer order steps 4, 7, 8, 11; schedule after these PRs |

---

## Acceptance per PR

Every PR, before review:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo deny check
typos --config typos.toml
just smoke            # Phase 4 adds it; before then: just run & sleep 3; kill %1 2>/dev/null || true
```

Plus the PR's manual checks:

| PR | Manual checks |
|---|---|
| 1a | Task 5 Step 6, Task 6 Step 6 |
| 1b | the offline checks in Tasks 9–13 |
| 2 | Task 16 Step 4 |
| 3 | benchmark table and the Harvey screenshots (Task 22), `draw_ms` numbers (Task 23) |
| 4 | green `windows` CI job |

The PR description lists each **[verify on Windows]** item it touches, with its Appendix B result, or says "pending a Windows machine" so the item stays open.

---

## Appendix B: Windows verification checklist

Run on a real Windows 10 22H2 or Windows 11 machine with the release zip or a `cargo build --release` exe. These can't be proven on macOS/Linux or on the GPU-less hosted runner (except item 7's link, which CI proves). Record the date, the Windows build, and pass/fail in the PR.

1. **Scan cache location (Task 5):** after one live volume, `dir %LOCALAPPDATA%\rustywx\scans` shows `scan_v2_KFFC.bin`, and `%APPDATA%\rustywx` has no `scan_*` files. Restart; the status shows `[cached]`.
2. **Atomic write under a held handle (Tasks 1, 6):**
   1. In PowerShell: `$f = [IO.File]::Open("$env:APPDATA\rustywx\settings.json", 'Open', 'Read', 'Read')`. This share mode excludes Delete.
   2. Change a setting.
   3. `app.log` shows retries and then `Warning: saving settings failed`. No crash.
   4. `$f.Close()`, then change the setting again. It saves, and `settings.json.bak` exists.
3. **Proxy (Task 14):** set a manual proxy in Settings > Network & Internet > Proxy (a local Fiddler or mitmproxy works), with `HTTPS_PROXY` unset. Alerts and NHC traffic go through the proxy. Then point the proxy at a dead port: one toast, retries in `app.log`, no hang.
4. **miniquad patch (Task 15):**
   - One wheel notch zooms by about 5% (not to the limit).
   - One notch over the NHC panel or a modal scrolls a few lines.
   - A precision touchpad two-finger scroll is smooth.
   - Dragging a folder and a locked file onto the window does nothing.
   - Startup over RDP reaches either the window or a crash box with `crash.log`, never a silent exit.
5. **No console and logs (Task 17):**
   - Double-click `rustywx.exe` from Explorer: no console window.
   - `%LOCALAPPDATA%\rustywx\app.log` contains `startup phase=` lines.
   - With a scratch build containing a forced panic, a message box appears and `crash.log` has the message and a backtrace.
   - `rustywx.exe --version | Out-String` prints the version.
6. **Justfile (Task 24):** `just smoke` passes in pwsh 7.
7. **Resources (Task 25):** the Explorer icon, the Properties > Details version, and the manifest contents (see Task 25 Step 4).
8. **Static CRT (Task 26):** on a clean VM (Windows Sandbox works) without the VC++ redistributable, `rustywx.exe` starts. `dumpbin /dependents rustywx.exe` from a VS Developer prompt lists no `VCRUNTIME140.dll`.
9. **Sleep/resume (Phase 1b):**
   1. Launch.
   2. Sleep the laptop for 10+ minutes on Wi-Fi.
   3. Resume, and switch Wi-Fi networks or toggle a VPN.

   Within about 2 minutes, alerts and NHC refresh on their own. `app.log` shows `timed out after` / `failed` lines followed by success, with at most one toast.
