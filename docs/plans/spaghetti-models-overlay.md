# Spaghetti Models Overlay Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an opt-in (default OFF) tropical overlay. It draws NHC ATCF model-guidance tracks ("spaghetti") on the map, colors each segment by forecast wind, and shows a legend, the data age, a "not an official forecast" disclaimer, and a per-storm intensity chart in the NHC panel.

**Architecture:** There are three new pure-logic modules: `adeck.rs` (parse, select cycle, classify models, build guidance), `adeck_style.rs` (colors, styles, projection cache, text), and `adeck_fetch.rs` (a per-storm download scheduler behind a `NetPort` trait, so it can be unit-tested with a fake). Downloads go through Ply `net`, with one stable request id per storm that's `cancel()`ed after every result, an app-side 60 s deadline, and backoff. Downloads sit behind a small `NetPort` transport trait, so the planned app-owned reqwest HTTP module can replace Ply `net` later without touching the parser or renderer. Gunzip and parse run in the existing `background_work` parse worker. The UI thread only polls, schedules, and draws. The existing NHC state machine (`nhc.rs`) is not touched, so a hung a-deck request can't stall any other tropical data.

**Tech Stack:** Rust 2024, ply-engine 1.1 (`net`, Ply layout, macroquad drawing), tokio (background worker), serde, `flate2 = "1"` (the only new dependency, already in `Cargo.lock` transitively), chrono (wall-clock data age).

**Spec:** `docs/research/spaghetti-models.md` (data source, record format, model catalogue, licensing), the wx-team-lead brief (MVP scope, constraints), and `docs/reviews/windows-stability-review.md` findings C2, C3, H5, H9, M12, M20, and M21 (net retry, timeouts, per-frame projection, byte-slice panics, UI-thread work, unbounded inflate). Codebase facts are in **Appendix A: Codebase context** at the end of this file.

## Global Constraints

- New dependency: `flate2 = "1"` in `app/Cargo.toml` only. No other new crates.
- A-deck URL: `https://ftp.nhc.noaa.gov/atcf/aid_public/a{storm_id}.dat.gz`, where `storm_id` is the lower-case `CurrentStorms.json` id (`al092026` gives `aal092026.dat.gz`). Basins `al`, `ep`, `cp` only. Storm numbers 80–89 are training storms and are skipped.
- User-Agent on every a-deck request: `rustywx/0.3`.
- Ply net request id is **stable per storm**: `nhc-adeck-{storm_id}`. Call `Request::cancel()` after every finished response (Ok or Err) and on the app-side deadline, so the id is freed before the retry and a cached `Err` is never re-read (review C2; `docs/research/windows-stability-remediation.md`, networking). Never copy the alerts/borders/NHC retry pattern, which re-reads the cached `Err`.
- Transport boundary: only `adeck_fetch::PlyNet` touches Ply `net`. It implements the `NetPort` trait that the planned app-owned reqwest module (`app/src/http.rs`) will implement later. Don't build that module in this feature.
- Timing constants: refresh `REFRESH_SECS = 900.0` while the overlay is on, attempt timeout `ATTEMPT_TIMEOUT_SECS = 60.0` (review C3), backoff 30 → 60 → 120 → 240 → 480 → 900 s (capped), stale after `STALE_AFTER_SECS = 1800` (30 min).
- Toast (`ErrorKind::Network`) only on the **first** failure of a streak. Never toast on HTTP 404 (a brand-new storm whose a-deck isn't published yet).
- Gunzip and parse never run on the UI thread; they go through `BackgroundWorkers::submit_parse` (review M12). Compressed cap is 10 MiB (Ply/ureq's body cap), uncompressed cap is 64 MiB (review M20).
- No byte-index slicing (`&s[..n]`, `&s[a..b]`) on any string from the network or a-deck. Use `str::get`, `strip_suffix`, `split`, and `chars()` (reviews H9 and M21).
- Default OFF: `Settings::show_model_tracks` is `#[serde(default)]` false. While it's off, nothing is fetched. Tracks draw only when both `show_model_tracks` and the master tropical gate `show_nhc_data` are on.
- Icons come only from `widgets::nf` with `SYMBOL_FONT`. No emoji, and don't add `"✓"` or `"🌐"`. Map labels drawn with macroquad `draw_text` are plain ASCII.
- Disclaimer text is the exact `adeck_style::DISCLAIMER_LINES` (Task 7). Show it whenever the overlay is on: the short form in the legend, the full form in the NHC panel.
- Don't modify `nhc.rs` or the existing NHC fetch flow. Fix nothing outside this feature (see Follow-ups for known violations).
- Commits use Conventional Commits (`feat:`, `test:`, `docs:`, `chore:`). Run `cargo fmt --all` before every commit, because the plan's snippets aren't guaranteed to be rustfmt-exact.
- The pure-module code in Tasks 1–10, 13, and 16 (`adeck.rs`, `adeck_style.rs`, and `adeck_fetch.rs` minus `PlyNet`) was compiled and tested by the planner in a scratch crate: 60 tests pass and clippy `-D warnings` is clean. If a snippet fails to compile in the real tree, the cause is integration (imports, paths), not the logic.
- Before claiming done: `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features`, `typos --config typos.toml`, then the AGENTS.md smoke test (`just run & sleep 3; kill %1`).

## Review Focus

Each of these failure modes has a pinning test in the task listed after it.

1. **Garbage, truncated, or non-ASCII a-deck lines** (server hiccup, partial write). Expected: the line is skipped, nothing panics, and the rest of the file still parses. Pinned in Task 1 (`parse_line_rejects_garbage_without_panicking`) and Task 3 (`gunzip_rejects_truncated_input`).
2. **A storm whose a-deck isn't published yet (HTTP 404)**. Expected: a quiet retry with backoff, no toast, and a status of "retrying" in the panel. Pinned in Task 10 (`not_found_retries_quietly`).
3. **A storm that leaves `CurrentStorms.json` while its download or parse is in flight**. Expected: the download is cancelled, a late parse result is dropped, and the store forgets the storm. Pinned in Task 6 (`retain_active_drops_missing_storms`) and Task 10 (`removed_storm_cancels_download_and_disowns_parse`).
4. **Hours offline (laptop sleep, VPN drop)**. Expected: one toast, backoff capped at 900 s, the last good tracks stay on screen with "(stale)" after 30 min, and a hung request never blocks another storm. Pinned in Task 8 (`is_stale_boundary`), Task 10 (`backoff_doubles_then_caps`, `hung_storm_does_not_block_other_storms`, `deadline_cancels_hung_request_and_retries`).
5. **A Central Pacific track crossing the antimeridian, or an `E` longitude over 180**. Expected: there's no 360° wrap-around streak across the map. Pinned in Task 1 (`parse_lon_normalizes_east_past_180`) and Task 13 (`projection_is_continuous_across_antimeridian`).

---

## File Structure

| File | Status | Responsibility |
|---|---|---|
| `app/src/testdata/aal092026_sample.dat` | create | 48 real a-deck rows (fixture) |
| `app/src/adeck.rs` | create | Record parser, URL mapping, gunzip, model classification, cycle selection, `StormGuidance`, `GuidanceStore` |
| `app/src/adeck_style.rs` | create | Wind categories and colors, segment color, track style, projection cache, cycle/age text, disclaimer, chart scale |
| `app/src/adeck_fetch.rs` | create | `NetPort` trait, `PlyNet`, `AdeckFetcher` scheduler (stable id plus `cancel()`, deadline, backoff, toast streak), status text |
| `app/src/widgets/model_legend.rs` | create | Floating map legend (Ply) |
| `app/src/lib.rs` | modify | `pub mod adeck; pub mod adeck_fetch; pub mod adeck_style;` |
| `app/src/widgets/mod.rs` | modify | `pub mod model_legend;` and the `ChartWidget::IntensityChart` variant |
| `app/Cargo.toml` | modify | `flate2 = "1"` |
| `app/src/background_work.rs` | modify | `ParsePayload::ADeck`, `ParsedData::ADeck`, `parse_queue_has_capacity()` |
| `app/src/settings.rs` | modify | `show_model_tracks` field plus 3 tests |
| `app/src/widgets/settings.rs` | modify | Settings-modal row and toggle id |
| `app/src/state.rs` | modify | `adeck_fetch`, `model_guidance`, `model_projection` fields |
| `app/src/scope.rs` | modify | `draw_model_tracks`, plus a new parameter on `draw_scope_to_texture` |
| `app/src/main.rs` | modify | Fetch/parse wiring, draw call, legend placement, NHC panel card and toggle, chart draw, input |
| `typos.toml`, `USER_GUIDE.md` | modify | Spell-check exclusions and user docs |

Task order is a dependency chain: 1–8 are pure library code, 9 is the worker, 10 the scheduler, 11 the settings, 12 the wiring, and 13–16 the UI. Task 17 covers docs and acceptance.

---

### Task 1: A-deck record parser and real-data fixture

**Files:**
- Create: `app/src/testdata/aal092026_sample.dat`
- Create: `app/src/adeck.rs`
- Modify: `app/src/lib.rs` (add `pub mod adeck;` as the first `pub mod` line; it sorts before `alerts`)

**Interfaces:**
- Produces: `pub struct ADeckRecord { pub init: u32, pub tech: String, pub tau_h: i32, pub position: Option<(f64, f64)>, pub vmax_kt: Option<u32> }`, `pub fn parse_adeck_line(line: &str) -> Option<ADeckRecord>`, `pub fn parse_adeck(text: &str) -> Vec<ADeckRecord>`, `pub fn parse_lat(s: &str) -> Option<f64>`, `pub fn parse_lon(s: &str) -> Option<f64>`.

- [ ] **Step 1: Create the fixture.** Write exactly these 48 lines (real rows from `aal092026.dat.gz`, fetched 2026-10-09 15:36 UTC, with trailing whitespace removed) to `app/src/testdata/aal092026_sample.dat`:

```text
AL, 09, 2026100512, 01, CARQ, -24, 220N,  965W,  15,    0, DB,  34, AAA,    0,    0,    0,    0,    0,    0,   0,   0,   0,   L,   0,   X,   0,   0,     INVEST,  ,
AL, 09, 2026100906, 03, OFCL,   0, 258N,  884W,  90,    0, HU,  34, NEQ,  150,  110,   70,  160,    0,    0,   0, 110,   0,    ,   0, ESB,  30,  11,           ,  , 12, NEQ,   90,   90,   90,   90,
AL, 09, 2026100906, 03, OFCL,   0, 258N,  884W,  90,    0, HU,  50, NEQ,   50,   60,   30,   60,    0,    0,   0, 110,   0,    ,   0, ESB,  30,  11,           ,  , 12, NEQ,   90,   90,   90,   90,
AL, 09, 2026100906, 03, AVNO,   0, 258N,  885W,  64,  974, XX,  34, NEQ,  136,  100,   82,  118, 1010,  317,  24,   0,   0,
AL, 09, 2026100912, 01, CARQ, -24, 234N,  906W,  75,    0, HU,  34, AAA,    0,    0,    0,    0,    0,    0,   0,   0,   0,   L,   0,   X,   0,   0,     ISAIAS,  ,
AL, 09, 2026100912, 01, CARQ, -18, 239N,  898W,  80,    0, HU,  34, AAA,    0,    0,    0,    0,    0,    0,   0,   0,   0,   L,   0,   X,   0,   0,     ISAIAS,  ,
AL, 09, 2026100912, 01, CARQ, -12, 248N,  890W,  85,    0, HU,  34, AAA,    0,    0,    0,    0,    0,    0,   0,   0,   0,   L,   0,   X,  33,  11,     ISAIAS,  ,
AL, 09, 2026100912, 01, CARQ,  -6, 258N,  884W,  90,    0, HU,  34, AAA,    0,    0,    0,    0,    0,    0,   0,   0,   0,   L,   0,   X,   0,   0,     ISAIAS,  ,
AL, 09, 2026100912, 01, CARQ,   0, 270N,  876W, 105,  959, HU,  34, NEQ,  180,  110,   80,  160, 1008,  250,  20,   0,   0,   L,   0,   X,  30,  13,     ISAIAS, D,
AL, 09, 2026100912, 01, CARQ,   0, 270N,  876W, 105,  959, HU,  50, NEQ,   60,   70,   40,   60, 1008,  250,  20,   0,   0,   L,   0,   X,  30,  13,     ISAIAS, D,
AL, 09, 2026100912, 01, CARQ,   0, 270N,  876W, 105,  959, HU,  64, NEQ,   30,   30,   20,   20, 1008,  250,  20,   0,   0,   L,   0,   X,  30,  13,     ISAIAS, D,
AL, 09, 2026100912, 03, OFCL,   0, 270N,  876W, 105,    0, HU,  34, NEQ,  180,  110,   80,  160,    0,    0,   0, 130,   0,    ,   0, RJB,  30,  14,           ,  , 12, NEQ,  150,  120,   90,  150,
AL, 09, 2026100912, 03, OFCL,   0, 270N,  876W, 105,    0, HU,  50, NEQ,   60,   70,   40,   60,    0,    0,   0, 130,   0,    ,   0, RJB,  30,  14,           ,  , 12, NEQ,  150,  120,   90,  150,
AL, 09, 2026100912, 03, OFCL,   0, 270N,  876W, 105,    0, HU,  64, NEQ,   30,   30,   20,   20,    0,    0,   0, 130,   0,    ,   0, RJB,  30,  14,           ,  , 12, NEQ,  150,  120,   90,  150,
AL, 09, 2026100912, 03, OFCL,   3, 277N,  873W, 105,  959, HU,  34, NEQ,  180,  110,   80,  160,    0,    0,   0, 130,   0,    ,   0, RJB,  25,  14,           ,  , 12, NEQ,  180,  180,  120,  150,
AL, 09, 2026100912, 03, OFCL,   3, 277N,  873W, 105,  959, HU,  50, NEQ,   60,   70,   40,   60,    0,    0,   0, 130,   0,    ,   0, RJB,  25,  14,           ,  , 12, NEQ,  180,  180,  120,  150,
AL, 09, 2026100912, 03, OFCL,   3, 277N,  873W, 105,  959, HU,  64, NEQ,   30,   30,   20,   20,    0,    0,   0, 130,   0,    ,   0, RJB,  25,  14,           ,  , 12, NEQ,  180,  180,  120,  150,
AL, 09, 2026100912, 03, OFCL,  12, 297N,  868W, 100,    0, HU,  34, NEQ,  140,  100,   60,  110,    0,    0,   0, 120,   0,    ,   0, RJB,  10,  14,
AL, 09, 2026100912, 03, OFCL,  12, 297N,  868W, 100,    0, HU,  50, NEQ,   50,   50,   30,   50,    0,    0,   0, 120,   0,    ,   0, RJB,  10,  14,
AL, 09, 2026100912, 03, OFCL,  12, 297N,  868W, 100,    0, HU,  64, NEQ,   30,   30,   15,   25,    0,    0,   0, 120,   0,    ,   0, RJB,  10,  14,
AL, 09, 2026100912, 03, OFCL,  24, 321N,  871W,  55,    0, TS,  34, NEQ,   70,   50,   30,   60,    0,    0,   0,  65,   0,    ,   0, RJB, 355,  12,
AL, 09, 2026100912, 03, OFCL,  24, 321N,  871W,  55,    0, TS,  50, NEQ,   30,   30,   20,   30,    0,    0,   0,  65,   0,    ,   0, RJB, 355,  12,
AL, 09, 2026100912, 03, OFCL,  36, 342N,  872W,  35,    0, LO,  34, NEQ,    0,    0,    0,    0,    0,    0,   0,  45,   0,    ,   0, RJB, 360,  11,
AL, 09, 2026100912, 03, OFCL,  48, 365N,  850W,  25,    0, LO,  34, NEQ,    0,    0,    0,    0,    0,    0,   0,  35,   0,    ,   0, RJB,  35,  15,
AL, 09, 2026100912, 03, OFCL,  60, 391N,  810W,  20,    0, LO,  34, NEQ,    0,    0,    0,    0,    0,    0,   0,  30,   0,    ,   0, RJB,  50,  20,
AL, 09, 2026100912, 03, AVNI,   0, 270N,  876W, 105,    0,   ,  34, NEQ,  180,  110,   80,  160,
AL, 09, 2026100912, 03, AVNI,   0, 270N,  876W, 105,    0,   ,  50, NEQ,   60,   70,   40,   60,
AL, 09, 2026100912, 03, AVNI,   0, 270N,  876W, 105,    0,   ,  64, NEQ,   30,   30,   20,   20,
AL, 09, 2026100912, 03, AVNI,  12, 301N,  870W,  87,    0,   ,  34, NEQ,  110,   71,   59,   91,
AL, 09, 2026100912, 03, AVNI,  12, 301N,  870W,  87,    0,   ,  50, NEQ,   37,   51,   12,   36,
AL, 09, 2026100912, 03, AVNI,  12, 301N,  870W,  87,    0,   ,  64, NEQ,   25,   24,    6,   20,
AL, 09, 2026100912, 03, AVNI,  24, 324N,  877W,  58,    0,   ,  34, NEQ,   46,   12,    2,   34,
AL, 09, 2026100912, 03, AVNI,  24, 324N,  877W,  58,    0,   ,  50, NEQ,   17,    7,    0,   13,
AL, 09, 2026100912, 03, AVNI,  36, 337N,  877W,  43,    0,   ,  34, NEQ,   33,    3,    0,   25,
AL, 09, 2026100912, 03, AVNI,  48, 358N,  855W,  33,    0,   ,  34, NEQ,    0,    0,    0,    0,
AL, 09, 2026100912, 03, HWFI,   0, 270N,  876W, 105,    0,   ,  34, NEQ,  180,  110,   80,  160,
AL, 09, 2026100912, 03, HWFI,  12, 300N,  872W,  81,    0,   ,  34, NEQ,  150,   46,   55,   75,
AL, 09, 2026100912, 03, HWFI,  24, 325N,  880W,  42,    0,   ,  34, NEQ,  128,    0,    3,   10,
AL, 09, 2026100912, 03, IVCN,   0,   0N,    0W, 105,    0,   ,   0,    ,    0,    0,    0,    0,
AL, 09, 2026100912, 03, IVCN,  12,   0N,    0W,  89,    0,   ,   0,    ,    0,    0,    0,    0,
AL, 09, 2026100912, 03, IVCN,  24,   0N,    0W,  47,    0,   ,   0,    ,    0,    0,    0,    0,
AL, 09, 2026100912, 03, TVCN,   0, 270N,  876W,   0,    0,   ,   0,    ,    0,    0,    0,    0,
AL, 09, 2026100912, 03, TVCN,  12, 298N,  870W,   0,    0,   ,   0,    ,    0,    0,    0,    0,
AL, 09, 2026100912, 03, TVCN,  24, 323N,  877W,   0,    0,   ,   0,    ,    0,    0,    0,    0,
AL, 09, 2026100912, 03, RVCN,   0, 270N,  876W,  65,    0,   ,  34, NEQ,  180,  110,   80,  160,
AL, 09, 2026100912, 03, SHIP,   0, 270N,  876W, 105,    0, HU,   0,    ,    0,    0,    0,    0,
AL, 09, 2026100912, 03, XTRP,   0, 270N,  876W,   0,    0,   ,   0,    ,    0,    0,    0,    0,
AL, 09, 2026100912, 03, HFAI,   0, 270N,  876W, 105,    0,   ,  34, NEQ,  180,  110,   80,  160,
```

Verify: `wc -l app/src/testdata/aal092026_sample.dat` prints `48`.

- [ ] **Step 2: Write the failing tests.** Create `app/src/adeck.rs` containing only the module doc and the tests:

```rust
//! NHC ATCF a-deck ("aid") parsing for the optional spaghetti-models overlay:
//! model-guidance tracks and intensities. Pure functions only (no network, no
//! drawing). Record format: `docs/research/spaghetti-models.md` section 2.

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("testdata/aal092026_sample.dat");

    #[test]
    fn parse_line_reads_model_row() {
        let line = "AL, 09, 2026100912, 03, AVNI,  12, 301N,  870W,  87,    0,   ,  34, NEQ,  110,   71,   59,   91,";
        let r = parse_adeck_line(line).expect("valid row");
        assert_eq!(r.init, 2026100912);
        assert_eq!(r.tech, "AVNI");
        assert_eq!(r.tau_h, 12);
        assert_eq!(r.position, Some((30.1, -87.0)));
        assert_eq!(r.vmax_kt, Some(87));
    }

    #[test]
    fn parse_line_handles_negative_tau_carq() {
        let line = "AL, 09, 2026100912, 01, CARQ, -24, 234N,  906W,  75,    0, HU,  34, AAA,    0,    0,    0,    0,    0,    0,   0,   0,   0,   L,   0,   X,   0,   0,     ISAIAS,  ,";
        let r = parse_adeck_line(line).expect("valid row");
        assert_eq!(r.tau_h, -24);
        assert_eq!(r.position, Some((23.4, -90.6)));
        assert_eq!(r.vmax_kt, Some(75));
    }

    #[test]
    fn parse_line_maps_zero_position_to_none() {
        let line = "AL, 09, 2026100912, 03, IVCN,  12,   0N,    0W,  89,    0,   ,   0,    ,    0,    0,    0,    0,";
        let r = parse_adeck_line(line).expect("intensity-only rows still parse");
        assert_eq!(r.position, None);
        assert_eq!(r.vmax_kt, Some(89));
    }

    #[test]
    fn parse_line_maps_zero_vmax_to_none() {
        let line = "AL, 09, 2026100912, 03, TVCN,  12, 298N,  870W,   0,    0,   ,   0,    ,    0,    0,    0,    0,";
        let r = parse_adeck_line(line).expect("valid row");
        assert_eq!(r.vmax_kt, None);
        assert_eq!(r.position, Some((29.8, -87.0)));
    }

    #[test]
    fn parse_line_rejects_garbage_without_panicking() {
        for bad in [
            "",
            "AL, 09",
            "AL, 09, 20261009, 03, AVNI, 12, 301N, 870W, 87",
            "AL, 09, 2026100912, 03, \u{c5}VNI, 12, 301N, 870W, 87",
            "AL, 09, 2026100912, 03, AVNI, x, 301N, 870W, 87",
            "AL, 09, 2026100912, 03, AVNI, 12, 30.1N, 870W, 87",
            "AL, 09, 2026100912, 03, AVNI, 12, 301Q, 870W, 87",
            "AL, 09, 2026100912, 03, AVNI, 12, 301N, 870W,",
            "AL, 09, 2026100912, 03, , 12, 301N, 870W, 87",
            "\u{65e5}\u{672c},,,,,,,,",
            "AL, 09, 2026100912, 03, AVNI, 12, 950N, 870W, 87",
        ] {
            assert_eq!(parse_adeck_line(bad), None, "should reject {bad:?}");
        }
    }

    #[test]
    fn parse_lat_lon_hemispheres() {
        assert_eq!(parse_lat("234S"), Some(-23.4));
        assert_eq!(parse_lat("0N"), Some(0.0));
        assert_eq!(parse_lon("1795E"), Some(179.5));
        assert_eq!(parse_lon("876W"), Some(-87.6));
        assert_eq!(parse_lat(""), None);
        assert_eq!(parse_lon("W"), None);
    }

    #[test]
    fn parse_lon_normalizes_east_past_180() {
        // Some feeds write 0..360 east; 190E is 170W. Must not produce a
        // +190 vertex that wraps a line across the whole map.
        assert_eq!(parse_lon("1900E"), Some(-170.0));
        assert_eq!(parse_lon("3700W"), None);
    }

    #[test]
    fn parse_adeck_reads_every_fixture_line() {
        assert_eq!(FIXTURE.lines().count(), 48);
        assert_eq!(parse_adeck(FIXTURE).len(), 48);
    }
}
```

Add `pub mod adeck;` to `app/src/lib.rs` as the first `pub mod` line (alphabetical).

- [ ] **Step 3: Run the tests to verify they fail.**

Run: `cargo test -p rustywx --lib adeck::`
Expected: compile error `cannot find function 'parse_adeck_line' in this scope`.

- [ ] **Step 4: Write the minimal implementation.** Insert above `#[cfg(test)]` in `app/src/adeck.rs`:

```rust
/// One a-deck row, reduced to the columns the overlay needs.
#[derive(Clone, Debug, PartialEq)]
pub struct ADeckRecord {
    /// Run init time as YYYYMMDDHH (UTC), e.g. `2026100912`.
    pub init: u32,
    /// Model id, upper-case (e.g. `"AVNI"`, `"OFCL"`).
    pub tech: String,
    /// Forecast hour. Negative for CARQ past positions.
    pub tau_h: i32,
    /// (lat, lon) in degrees. `None` for the `0N, 0W` "no position" rows that
    /// intensity-only techs (IVCN, NNIC, ...) emit. Never plot those.
    pub position: Option<(f64, f64)>,
    /// Max sustained wind in kt. `None` when the column is 0 ("no value").
    pub vmax_kt: Option<u32>,
}

/// Parse one comma-separated a-deck row. Returns `None` for anything
/// malformed: the caller skips the line, never fails the file. Uses no byte
/// slicing, so hostile or non-ASCII input can't panic.
pub fn parse_adeck_line(line: &str) -> Option<ADeckRecord> {
    let mut fields = line.split(',').map(str::trim);
    let _basin = fields.next()?;
    let _cy = fields.next()?;
    let init = parse_init(fields.next()?)?;
    let _technum = fields.next()?;
    let tech = fields.next()?;
    if tech.is_empty() || !tech.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    let tau_h: i32 = fields.next()?.parse().ok()?;
    let lat = parse_lat(fields.next()?)?;
    let lon = parse_lon(fields.next()?)?;
    let vmax: u32 = fields.next()?.parse().ok()?;
    let position = if lat == 0.0 && lon == 0.0 {
        None
    } else {
        Some((lat, lon))
    };
    Some(ADeckRecord {
        init,
        tech: tech.to_ascii_uppercase(),
        tau_h,
        position,
        vmax_kt: (vmax > 0).then_some(vmax),
    })
}

/// Parse every well-formed line; malformed lines are skipped.
pub fn parse_adeck(text: &str) -> Vec<ADeckRecord> {
    text.lines().filter_map(parse_adeck_line).collect()
}

fn parse_init(s: &str) -> Option<u32> {
    if s.len() != 10 || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

fn parse_tenths(digits: &str) -> Option<f64> {
    let v: u32 = digits.parse().ok()?;
    Some(f64::from(v) / 10.0)
}

/// `"270N"` -> 27.0, `"234S"` -> -23.4.
pub fn parse_lat(s: &str) -> Option<f64> {
    let lat = if let Some(d) = s.strip_suffix('N') {
        parse_tenths(d)?
    } else if let Some(d) = s.strip_suffix('S') {
        -parse_tenths(d)?
    } else {
        return None;
    };
    (-90.0..=90.0).contains(&lat).then_some(lat)
}

/// `"876W"` -> -87.6, `"1795E"` -> 179.5. East values past 180 are folded
/// into -180..180 so a track can't wrap across the whole map.
pub fn parse_lon(s: &str) -> Option<f64> {
    let mut lon = if let Some(d) = s.strip_suffix('W') {
        -parse_tenths(d)?
    } else if let Some(d) = s.strip_suffix('E') {
        parse_tenths(d)?
    } else {
        return None;
    };
    if lon > 180.0 {
        lon -= 360.0;
    }
    (-180.0..=180.0).contains(&lon).then_some(lon)
}
```

- [ ] **Step 5: Run the tests to verify they pass.**

Run: `cargo test -p rustywx --lib adeck::`
Expected: 8 passed.

- [ ] **Step 6: Commit.**

```bash
git add app/src/adeck.rs app/src/lib.rs app/src/testdata/aal092026_sample.dat
git commit -m "feat(tropical): parse ATCF a-deck records"
```

---

### Task 2: Storm id to a-deck URL mapping

**Files:**
- Modify: `app/src/adeck.rs`

**Interfaces:**
- Produces: `pub const ADECK_BASE_URL: &str`, `pub fn adeck_url(storm_id: &str) -> Option<String>`.

- [ ] **Step 1: Write the failing test.** Add to `mod tests` in `app/src/adeck.rs`:

```rust
    #[test]
    fn adeck_url_maps_current_storms_ids() {
        assert_eq!(
            adeck_url("al092026").as_deref(),
            Some("https://ftp.nhc.noaa.gov/atcf/aid_public/aal092026.dat.gz")
        );
        assert_eq!(
            adeck_url("EP182026").as_deref(),
            Some("https://ftp.nhc.noaa.gov/atcf/aid_public/aep182026.dat.gz")
        );
        assert!(adeck_url("cp012026").is_some());
        assert!(adeck_url("al942026").is_some(), "invests are supported");
    }

    #[test]
    fn adeck_url_rejects_unsupported_ids_without_panicking() {
        for bad in [
            "al852026", // training storm (80-89)
            "al002026",
            "wp012026", // West Pacific: not in NHC aid_public
            "al9",
            "",
            "alxx2026",
            "al09202",
            "al0920\u{e9}6", // non-ASCII, 9 bytes
            "al092026x",
        ] {
            assert_eq!(adeck_url(bad), None, "should reject {bad:?}");
        }
    }
```

- [ ] **Step 2: Run the tests to verify they fail.**

Run: `cargo test -p rustywx --lib adeck::tests::adeck_url`
Expected: compile error `cannot find function 'adeck_url'`.

- [ ] **Step 3: Implement.** Add above `#[cfg(test)]`:

```rust
/// Public ATCF guidance directory (HTTPS, public domain).
pub const ADECK_BASE_URL: &str = "https://ftp.nhc.noaa.gov/atcf/aid_public/";

/// Map a `CurrentStorms.json` storm id (`"al092026"`) to its a-deck URL.
/// `None` for basins outside NHC (al/ep/cp), training storms 80-89, and
/// anything malformed. Uses `str::get`, never panicking slices.
pub fn adeck_url(storm_id: &str) -> Option<String> {
    let id = storm_id.trim().to_ascii_lowercase();
    if !id.is_ascii() || id.len() != 8 {
        return None;
    }
    let basin = id.get(0..2)?;
    let number = id.get(2..4)?;
    let year = id.get(4..8)?;
    if !matches!(basin, "al" | "ep" | "cp") {
        return None;
    }
    if !number.bytes().all(|b| b.is_ascii_digit()) || !year.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n: u32 = number.parse().ok()?;
    if n == 0 || (80..=89).contains(&n) {
        return None;
    }
    Some(format!("{ADECK_BASE_URL}a{id}.dat.gz"))
}
```

- [ ] **Step 4: Run the tests to verify they pass.**

Run: `cargo test -p rustywx --lib adeck::`
Expected: 10 passed.

- [ ] **Step 5: Commit.**

```bash
git add app/src/adeck.rs
git commit -m "feat(tropical): map storm ids to ATCF a-deck URLs"
```

---

### Task 3: Bounded gunzip

**Files:**
- Modify: `app/Cargo.toml` (in `[dependencies]`, after `zip = "2"`, add `flate2 = "1"`)
- Modify: `app/src/adeck.rs`

**Interfaces:**
- Produces: `pub const MAX_GZIP_BYTES: usize`, `pub const MAX_UNCOMPRESSED_BYTES: u64`, `pub fn gunzip_limited(gz: &[u8], max_out: u64) -> anyhow::Result<String>`.

- [ ] **Step 1: Add the dependency.** In `app/Cargo.toml`, after the line `zip = "2"`, add:

```toml
# A-deck guidance files are gzip only (spaghetti-models overlay). Already in
# Cargo.lock via `zip`; this pins it as a direct dependency.
flate2 = "1"
```

Run: `cargo check -p rustywx`. Expected: success, and `Cargo.lock` still shows `flate2 1.1.9` (`git diff Cargo.lock` shows at most the new dependency edge).

- [ ] **Step 2: Write the failing tests.** Add to `mod tests`:

```rust
    fn gzip(bytes: &[u8]) -> Vec<u8> {
        use std::io::Write;
        let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        enc.write_all(bytes).unwrap();
        enc.finish().unwrap()
    }

    #[test]
    fn gunzip_round_trips_fixture() {
        let text = gunzip_limited(&gzip(FIXTURE.as_bytes()), MAX_UNCOMPRESSED_BYTES).unwrap();
        assert_eq!(text, FIXTURE);
    }

    #[test]
    fn gunzip_rejects_non_gzip_body() {
        let err = gunzip_limited(b"<html>404</html>", MAX_UNCOMPRESSED_BYTES).unwrap_err();
        assert!(format!("{err:#}").contains("not gzip"), "{err:#}");
    }

    #[test]
    fn gunzip_stops_a_decompression_bomb() {
        let bomb = gzip(&vec![0u8; 1_000_000]);
        let err = gunzip_limited(&bomb, 1024).unwrap_err();
        assert!(format!("{err:#}").contains("inflates past"), "{err:#}");
    }

    #[test]
    fn gunzip_rejects_truncated_input() {
        let gz = gzip(FIXTURE.as_bytes());
        let half = gz.get(..gz.len() / 2).unwrap();
        assert!(gunzip_limited(half, MAX_UNCOMPRESSED_BYTES).is_err());
    }

    #[test]
    fn gunzip_rejects_oversized_body() {
        let mut big = vec![0x1f, 0x8b];
        big.resize(MAX_GZIP_BYTES + 1, 0);
        let err = gunzip_limited(&big, MAX_UNCOMPRESSED_BYTES).unwrap_err();
        assert!(format!("{err:#}").contains("cap"), "{err:#}");
    }
```

- [ ] **Step 3: Run the tests to verify they fail.**

Run: `cargo test -p rustywx --lib adeck::tests::gunzip`
Expected: compile error `cannot find function 'gunzip_limited'`.

- [ ] **Step 4: Implement.** Add at the top of `app/src/adeck.rs`, below the module doc:

```rust
use anyhow::{Result, anyhow, bail};
```

Add above `#[cfg(test)]`:

```rust
/// Largest compressed body accepted. Ply's ureq backend already refuses
/// bodies over 10 MiB (`ureq` `MAX_BODY_SIZE`). The biggest live a-deck seen
/// was 2.7 MB gzipped, so real files never hit this.
pub const MAX_GZIP_BYTES: usize = 10 * 1024 * 1024;

/// Uncompressed cap. A 350 KB live file inflates to 2.9 MB, so 64 MiB leaves
/// over 20x headroom while stopping a decompression bomb (review M20).
pub const MAX_UNCOMPRESSED_BYTES: u64 = 64 * 1024 * 1024;

/// Inflate a gzip body, refusing non-gzip input and anything that inflates
/// past `max_out` bytes. Invalid UTF-8 is replaced, not rejected (a-decks
/// are ASCII). Run on the background worker only, never the UI thread.
pub fn gunzip_limited(gz: &[u8], max_out: u64) -> Result<String> {
    use std::io::Read;
    if gz.len() > MAX_GZIP_BYTES {
        bail!(
            "a-deck body is {} bytes, over the {MAX_GZIP_BYTES} byte cap",
            gz.len()
        );
    }
    if gz.get(0..2) != Some(&[0x1f, 0x8b][..]) {
        bail!("a-deck body is not gzip ({} bytes)", gz.len());
    }
    let mut out = Vec::new();
    flate2::read::GzDecoder::new(gz)
        .take(max_out + 1)
        .read_to_end(&mut out)
        .map_err(|e| anyhow!("inflating a-deck: {e}"))?;
    if out.len() as u64 > max_out {
        bail!("a-deck inflates past {max_out} bytes");
    }
    Ok(String::from_utf8_lossy(&out).into_owned())
}
```

- [ ] **Step 5: Run the tests to verify they pass.**

Run: `cargo test -p rustywx --lib adeck::`
Expected: 15 passed.

- [ ] **Step 6: Commit.**

```bash
git add app/Cargo.toml Cargo.lock app/src/adeck.rs
git commit -m "feat(tropical): bounded gunzip for a-deck bodies"
```

---

### Task 4: Model classification (grouping and default model set)

**Files:**
- Modify: `app/src/adeck.rs`

**Interfaces:**
- Produces: `pub enum ModelGroup { Official, Observed, Consensus, Hurricane, Global, Ensemble, Ai, Statistical, Baseline, Trajectory, EnsembleMember, Late, Other }` (derives `Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash`), `pub struct TechInfo { pub group: ModelGroup, pub track: bool, pub default_visible: bool, pub intensity: bool }`, `pub fn classify_tech(tech: &str) -> Option<TechInfo>` (`None` means skip entirely).

- [ ] **Step 1: Write the failing tests.** Add to `mod tests`:

```rust
    /// The research "on by default" set plus OFCL and CARQ (always on).
    const DEFAULT_TRACKS: [&str; 16] = [
        "OFCL", "CARQ", "AVNI", "HWFI", "HFAI", "HFBI", "HMNI", "CTCI", "NVG2", "CMC2", "UKX2",
        "AEMI", "CEM2", "GDMI", "TVCN", "HCCA",
    ];

    #[test]
    fn default_track_set_matches_research() {
        for tech in DEFAULT_TRACKS {
            let info = classify_tech(tech).unwrap_or_else(|| panic!("{tech} must be known"));
            assert!(info.track && info.default_visible, "{tech} should be a default track");
        }
    }

    #[test]
    fn extra_models_are_tracks_but_hidden_by_default() {
        for tech in ["TABS", "TABM", "TABD", "XTRP", "CLP5", "TCLP", "AP01", "AP30", "OFCI", "AVNO", "HWRF", "ZZZZ"] {
            let info = classify_tech(tech).unwrap();
            assert!(info.track, "{tech} is a track");
            assert!(!info.default_visible, "{tech} hidden by default");
        }
        assert_eq!(classify_tech("ZZZZ").unwrap().group, ModelGroup::Other);
        assert_eq!(classify_tech("AP17").unwrap().group, ModelGroup::EnsembleMember);
    }

    #[test]
    fn intensity_only_techs_never_draw_tracks() {
        for tech in ["IVCN", "NNIC", "SHIP", "DSHP", "LGEM"] {
            let info = classify_tech(tech).unwrap();
            assert!(!info.track, "{tech} has no usable track");
            assert!(info.intensity, "{tech} feeds the intensity chart");
        }
    }

    #[test]
    fn radii_and_probability_products_are_skipped() {
        for tech in ["RVCN", "DRCL", "RI25"] {
            assert_eq!(classify_tech(tech), None);
        }
    }

    #[test]
    fn default_visible_implies_track() {
        for tech in DEFAULT_TRACKS.iter().chain(["IVCN", "SHIP", "TABS", "AP01", "ZZZZ"].iter()) {
            if let Some(info) = classify_tech(tech) {
                assert!(!info.default_visible || info.track, "{tech}");
            }
        }
    }
```

- [ ] **Step 2: Run the tests to verify they fail.**

Run: `cargo test -p rustywx --lib adeck::`
Expected: compile error `cannot find function 'classify_tech'`.

- [ ] **Step 3: Implement.** Add above `#[cfg(test)]`:

```rust
/// Model family, used for draw order and line style (track colour is by wind).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModelGroup {
    /// NHC official forecast (OFCL and its interpolated twins).
    Official,
    /// CARQ/WRNG past positions and the current fix.
    Observed,
    Consensus,
    Hurricane,
    Global,
    Ensemble,
    Ai,
    Statistical,
    Baseline,
    Trajectory,
    /// GEFS members AP01..AP30.
    EnsembleMember,
    /// Late (non-interpolated) runs that duplicate their `I` twins.
    Late,
    Other,
}

/// How one tech id is used by the overlay.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TechInfo {
    pub group: ModelGroup,
    /// Has a plottable track.
    pub track: bool,
    /// Drawn without the (follow-up) "more models" toggle.
    pub default_visible: bool,
    /// Feeds the intensity chart.
    pub intensity: bool,
}

const fn info(group: ModelGroup, track: bool, default_visible: bool, intensity: bool) -> TechInfo {
    TechInfo {
        group,
        track,
        default_visible,
        intensity,
    }
}

/// Classify an ATCF tech id (upper-case). `None` means skip entirely
/// (wind-radii and probability products). Catalogue and defaults:
/// `docs/research/spaghetti-models.md` section 3.
pub fn classify_tech(tech: &str) -> Option<TechInfo> {
    use ModelGroup::*;
    let is_gefs_member = tech.len() == 4
        && tech.starts_with("AP")
        && tech.get(2..4).is_some_and(|d| d.bytes().all(|b| b.is_ascii_digit()));
    if is_gefs_member {
        return Some(info(EnsembleMember, true, false, false));
    }
    Some(match tech {
        "OFCL" => info(Official, true, true, true),
        "OFCI" | "OFC2" => info(Official, true, false, false),
        "CARQ" => info(Observed, true, true, false),
        "WRNG" => info(Observed, true, false, false),
        "TVCN" => info(Consensus, true, true, false),
        "HCCA" => info(Consensus, true, true, true),
        "RYOC" => info(Consensus, true, false, true),
        "IVCN" | "NNIC" => info(Consensus, false, false, true),
        "NNIB" => info(Consensus, false, false, false),
        "HWFI" | "HFAI" | "HFBI" | "HMNI" | "CTCI" => info(Hurricane, true, true, true),
        "HWF2" => info(Hurricane, true, false, false),
        "AVNI" => info(Global, true, true, true),
        "NVG2" | "CMC2" | "UKX2" => info(Global, true, true, false),
        "NVGI" | "CMCI" | "UKXI" => info(Global, true, false, false),
        "AEMI" | "CEM2" => info(Ensemble, true, true, false),
        "CEMI" => info(Ensemble, true, false, false),
        "GDMI" => info(Ai, true, true, false),
        "SHIP" | "DSHP" | "LGEM" => info(Statistical, false, false, true),
        "OCD5" | "BCD5" | "SHF5" => info(Baseline, false, false, false),
        "TCLP" | "CLP5" | "XTRP" => info(Baseline, true, false, false),
        "TABS" | "TABM" | "TABD" => info(Trajectory, true, false, false),
        "AVNO" | "HWRF" | "CTCX" | "HMON" | "HFSA" | "HFSB" | "NVGM" | "AEMN" | "CEMN" | "GDMN" => {
            info(Late, true, false, false)
        }
        "RVCN" | "DRCL" | "RI25" => return None,
        _ => info(Other, true, false, false),
    })
}
```

- [ ] **Step 4: Run the tests to verify they pass.**

Run: `cargo test -p rustywx --lib adeck::`
Expected: 20 passed.

- [ ] **Step 5: Commit.**

```bash
git add app/src/adeck.rs
git commit -m "feat(tropical): classify ATCF models into groups and defaults"
```

---

### Task 5: Latest-cycle selection and guidance building

**Files:**
- Modify: `app/src/adeck.rs`

**Interfaces:**
- Consumes: `ADeckRecord`, `parse_adeck`, `gunzip_limited`, `MAX_UNCOMPRESSED_BYTES`, `classify_tech`, `ModelGroup` (Tasks 1–4).
- Produces:
  - `pub struct TrackPoint { pub tau_h: i32, pub lat: f64, pub lon: f64, pub vmax_kt: Option<u32> }`
  - `pub struct AidTrack { pub tech: String, pub group: ModelGroup, pub default_visible: bool, pub points: Vec<TrackPoint> }`
  - `pub struct IntensitySeries { pub tech: String, pub official: bool, pub points: Vec<(i32, u32)> }`
  - `pub struct StormGuidance { pub storm_id: String, pub init: u32, pub has_official: bool, pub tracks: Vec<AidTrack>, pub intensity: Vec<IntensitySeries>, pub fetched_at_unix: i64 }`
  - All of these derive `Clone, Debug, PartialEq`.
  - `pub fn latest_cycle(records: &[ADeckRecord]) -> Option<u32>`
  - `pub fn build_guidance(storm_id: &str, records: &[ADeckRecord], init: u32) -> StormGuidance`
  - `pub fn guidance_from_gzip(storm_id: &str, gz: &[u8]) -> anyhow::Result<StormGuidance>`

- [ ] **Step 1: Write the failing tests.** Add to `mod tests`:

```rust
    fn fixture_guidance() -> StormGuidance {
        build_guidance("AL092026", &parse_adeck(FIXTURE), 2026100912)
    }

    #[test]
    fn latest_cycle_is_newest_with_official_forecast() {
        assert_eq!(latest_cycle(&parse_adeck(FIXTURE)), Some(2026100912));
        // A newer, still-filling cycle without OFCL must not win.
        let mut records = parse_adeck(FIXTURE);
        records.extend(parse_adeck_line(
            "AL, 09, 2026100918, 03, AVNI,  12, 301N,  870W,  87,    0,   ,  34, NEQ,",
        ));
        assert_eq!(latest_cycle(&records), Some(2026100912));
    }

    #[test]
    fn latest_cycle_falls_back_when_no_official_forecast() {
        // Invests often have models but no OFCL.
        let records: Vec<_> = parse_adeck(FIXTURE).into_iter().filter(|r| r.tech != "OFCL").collect();
        assert_eq!(latest_cycle(&records), Some(2026100912));
        assert_eq!(latest_cycle(&[]), None);
    }

    #[test]
    fn build_guidance_keeps_plottable_tracks_only() {
        let g = fixture_guidance();
        assert_eq!(g.storm_id, "al092026");
        assert_eq!(g.init, 2026100912);
        assert!(g.has_official);
        assert_eq!(g.fetched_at_unix, 0);
        let techs: Vec<&str> = g.tracks.iter().map(|t| t.tech.as_str()).collect();
        // HFAI/XTRP have one point; IVCN/SHIP are intensity-only; RVCN is
        // skipped; AVNO is from an older cycle.
        assert_eq!(techs, ["AVNI", "CARQ", "HWFI", "OFCL", "TVCN"]);
        assert!(g.tracks.iter().all(|t| t.default_visible));
    }

    #[test]
    fn build_guidance_dedupes_wind_radii_rows() {
        let g = fixture_guidance();
        let ofcl = g.tracks.iter().find(|t| t.tech == "OFCL").unwrap();
        let taus: Vec<i32> = ofcl.points.iter().map(|p| p.tau_h).collect();
        assert_eq!(taus, [0, 3, 12, 24, 36, 48, 60]);
        assert_eq!(ofcl.group, ModelGroup::Official);
        assert_eq!((ofcl.points[0].lat, ofcl.points[0].lon), (27.0, -87.6));
        assert_eq!(ofcl.points[0].vmax_kt, Some(105));
    }

    #[test]
    fn build_guidance_keeps_carq_past_positions() {
        let g = fixture_guidance();
        let carq = g.tracks.iter().find(|t| t.tech == "CARQ").unwrap();
        let taus: Vec<i32> = carq.points.iter().map(|p| p.tau_h).collect();
        assert_eq!(taus, [-24, -18, -12, -6, 0]);
        assert_eq!(carq.group, ModelGroup::Observed);
    }

    #[test]
    fn build_guidance_keeps_missing_intensity_as_none() {
        let g = fixture_guidance();
        let tvcn = g.tracks.iter().find(|t| t.tech == "TVCN").unwrap();
        assert!(tvcn.points.iter().all(|p| p.vmax_kt.is_none()));
    }

    #[test]
    fn build_guidance_collects_intensity_series() {
        let g = fixture_guidance();
        let techs: Vec<&str> = g.intensity.iter().map(|s| s.tech.as_str()).collect();
        assert_eq!(techs, ["AVNI", "HWFI", "IVCN", "OFCL"]);
        let ivcn = g.intensity.iter().find(|s| s.tech == "IVCN").unwrap();
        assert_eq!(ivcn.points, [(0, 105), (12, 89), (24, 47)]);
        assert!(!ivcn.official);
        assert!(g.intensity.iter().find(|s| s.tech == "OFCL").unwrap().official);
    }

    #[test]
    fn invest_without_official_forecast_is_flagged() {
        let records: Vec<_> = parse_adeck(FIXTURE).into_iter().filter(|r| r.tech != "OFCL").collect();
        let g = build_guidance("al942026", &records, latest_cycle(&records).unwrap());
        assert!(!g.has_official);
        assert!(g.tracks.iter().all(|t| t.tech != "OFCL"));
    }

    #[test]
    fn guidance_from_gzip_round_trip() {
        let g = guidance_from_gzip("al092026", &gzip(FIXTURE.as_bytes())).unwrap();
        assert_eq!(g.init, 2026100912);
        assert_eq!(g.tracks.len(), 5);
    }

    #[test]
    fn guidance_from_gzip_rejects_empty_file() {
        let err = guidance_from_gzip("al092026", &gzip(b"")).unwrap_err();
        assert!(format!("{err:#}").contains("no parseable rows"), "{err:#}");
    }
```

- [ ] **Step 2: Run the tests to verify they fail.**

Run: `cargo test -p rustywx --lib adeck::`
Expected: compile error `cannot find function 'build_guidance'`.

- [ ] **Step 3: Implement.** Change the `use` line at the top to:

```rust
use anyhow::{Result, anyhow, bail};
use std::collections::BTreeMap;
```

Add above `#[cfg(test)]`:

```rust
/// One plotted forecast position.
#[derive(Clone, Debug, PartialEq)]
pub struct TrackPoint {
    pub tau_h: i32,
    pub lat: f64,
    pub lon: f64,
    pub vmax_kt: Option<u32>,
}

/// One model's track for the selected cycle, sorted by tau, deduplicated.
#[derive(Clone, Debug, PartialEq)]
pub struct AidTrack {
    pub tech: String,
    pub group: ModelGroup,
    pub default_visible: bool,
    pub points: Vec<TrackPoint>,
}

/// One model's wind-vs-forecast-hour series for the intensity chart.
#[derive(Clone, Debug, PartialEq)]
pub struct IntensitySeries {
    pub tech: String,
    pub official: bool,
    /// (tau_h, vmax_kt), tau >= 0, sorted by tau.
    pub points: Vec<(i32, u32)>,
}

/// Everything the overlay shows for one storm.
#[derive(Clone, Debug, PartialEq)]
pub struct StormGuidance {
    /// Lower-case CurrentStorms id, e.g. `"al092026"`.
    pub storm_id: String,
    /// Selected cycle, YYYYMMDDHH.
    pub init: u32,
    /// Whether the selected cycle has an OFCL forecast (invests often don't).
    pub has_official: bool,
    /// Tracks with at least two positioned points.
    pub tracks: Vec<AidTrack>,
    /// Intensity series with at least two points.
    pub intensity: Vec<IntensitySeries>,
    /// Wall-clock time (unix seconds) the UI accepted this parse. Set by
    /// `main.rs`, 0 until then. Drives the "updated N min ago" label.
    pub fetched_at_unix: i64,
}

/// The cycle to show: the newest init with an OFCL forecast (the newest cycle
/// fills in over several hours, so it may have models but no OFCL yet), else
/// the newest init of any row (invests).
pub fn latest_cycle(records: &[ADeckRecord]) -> Option<u32> {
    records
        .iter()
        .filter(|r| r.tech == "OFCL")
        .map(|r| r.init)
        .max()
        .or_else(|| records.iter().map(|r| r.init).max())
}

/// Build the per-storm guidance for one cycle: dedupe the repeated
/// wind-radii rows on (tech, tau), keep the first, split positioned tracks
/// from intensity-only series, and drop anything with fewer than two points.
pub fn build_guidance(storm_id: &str, records: &[ADeckRecord], init: u32) -> StormGuidance {
    let mut by_tech: BTreeMap<&str, BTreeMap<i32, &ADeckRecord>> = BTreeMap::new();
    for r in records.iter().filter(|r| r.init == init) {
        by_tech
            .entry(r.tech.as_str())
            .or_default()
            .entry(r.tau_h)
            .or_insert(r);
    }
    let mut tracks = Vec::new();
    let mut intensity = Vec::new();
    for (tech, rows) in &by_tech {
        let Some(info) = classify_tech(tech) else {
            continue;
        };
        if info.track {
            let points: Vec<TrackPoint> = rows
                .values()
                .filter(|r| info.group == ModelGroup::Observed || r.tau_h >= 0)
                .filter_map(|r| {
                    r.position.map(|(lat, lon)| TrackPoint {
                        tau_h: r.tau_h,
                        lat,
                        lon,
                        vmax_kt: r.vmax_kt,
                    })
                })
                .collect();
            if points.len() >= 2 {
                tracks.push(AidTrack {
                    tech: (*tech).to_string(),
                    group: info.group,
                    default_visible: info.default_visible,
                    points,
                });
            }
        }
        if info.intensity {
            let points: Vec<(i32, u32)> = rows
                .values()
                .filter(|r| r.tau_h >= 0)
                .filter_map(|r| r.vmax_kt.map(|v| (r.tau_h, v)))
                .collect();
            if points.len() >= 2 {
                intensity.push(IntensitySeries {
                    tech: (*tech).to_string(),
                    official: info.group == ModelGroup::Official,
                    points,
                });
            }
        }
    }
    StormGuidance {
        storm_id: storm_id.to_ascii_lowercase(),
        init,
        has_official: by_tech.contains_key("OFCL"),
        tracks,
        intensity,
        fetched_at_unix: 0,
    }
}

/// Full background-worker pipeline: inflate, parse, pick a cycle, build.
pub fn guidance_from_gzip(storm_id: &str, gz: &[u8]) -> Result<StormGuidance> {
    let text = gunzip_limited(gz, MAX_UNCOMPRESSED_BYTES)?;
    let records = parse_adeck(&text);
    let init = latest_cycle(&records)
        .ok_or_else(|| anyhow!("a-deck for {storm_id} has no parseable rows"))?;
    Ok(build_guidance(storm_id, &records, init))
}
```

- [ ] **Step 4: Run the tests to verify they pass.**

Run: `cargo test -p rustywx --lib adeck::`
Expected: 30 passed.

- [ ] **Step 5: Commit.**

```bash
git add app/src/adeck.rs
git commit -m "feat(tropical): select latest cycle and build model guidance"
```

---

### Task 6: Guidance store

**Files:**
- Modify: `app/src/adeck.rs`

**Interfaces:**
- Consumes: `StormGuidance` (Task 5).
- Produces: `#[derive(Default, Debug)] pub struct GuidanceStore` with `insert(&mut self, StormGuidance)`, `get(&self, &str) -> Option<&StormGuidance>`, `retain_active(&mut self, &[String])`, `iter(&self) -> impl Iterator<Item = &StormGuidance>`, `is_empty(&self) -> bool`, `revision(&self) -> u64`, and `oldest(&self) -> Option<&StormGuidance>` (minimum `fetched_at_unix`).

- [ ] **Step 1: Write the failing tests.** Add to `mod tests`:

```rust
    fn guidance(id: &str, fetched_at_unix: i64) -> StormGuidance {
        StormGuidance {
            storm_id: id.to_string(),
            init: 2026100912,
            has_official: true,
            tracks: Vec::new(),
            intensity: Vec::new(),
            fetched_at_unix,
        }
    }

    #[test]
    fn store_insert_replaces_and_bumps_revision() {
        let mut store = GuidanceStore::default();
        assert!(store.is_empty());
        store.insert(guidance("al092026", 10));
        let r1 = store.revision();
        store.insert(guidance("al092026", 20));
        assert!(store.revision() > r1);
        assert_eq!(store.iter().count(), 1);
        assert_eq!(store.get("AL092026").unwrap().fetched_at_unix, 20);
    }

    #[test]
    fn retain_active_drops_missing_storms() {
        let mut store = GuidanceStore::default();
        store.insert(guidance("al092026", 10));
        store.insert(guidance("ep182026", 10));
        let r = store.revision();
        store.retain_active(&["AL092026".to_string()]);
        assert!(store.get("ep182026").is_none());
        assert!(store.get("al092026").is_some());
        assert!(store.revision() > r);
        let r = store.revision();
        store.retain_active(&["al092026".to_string()]);
        assert_eq!(store.revision(), r, "no change, no revision bump");
    }

    #[test]
    fn oldest_is_least_recently_fetched() {
        let mut store = GuidanceStore::default();
        assert!(store.oldest().is_none());
        store.insert(guidance("al092026", 50));
        store.insert(guidance("ep182026", 30));
        assert_eq!(store.oldest().unwrap().storm_id, "ep182026");
    }
```

- [ ] **Step 2: Run the tests to verify they fail.**

Run: `cargo test -p rustywx --lib adeck::`
Expected: compile error `cannot find type 'GuidanceStore'`.

- [ ] **Step 3: Implement.** Add above `#[cfg(test)]`:

```rust
/// Last good guidance per active storm. Survives fetch failures (offline
/// keeps the previous tracks on screen). `revision` changes on every
/// mutation so the projection cache (`adeck_style::project_tracks`) knows
/// when to rebuild.
#[derive(Default, Debug)]
pub struct GuidanceStore {
    by_storm: BTreeMap<String, StormGuidance>,
    revision: u64,
}

impl GuidanceStore {
    pub fn insert(&mut self, guidance: StormGuidance) {
        self.by_storm.insert(guidance.storm_id.clone(), guidance);
        self.revision += 1;
    }

    pub fn get(&self, storm_id: &str) -> Option<&StormGuidance> {
        self.by_storm.get(&storm_id.to_ascii_lowercase())
    }

    /// Forget storms that are no longer in `CurrentStorms.json`.
    pub fn retain_active(&mut self, active: &[String]) {
        let before = self.by_storm.len();
        self.by_storm
            .retain(|id, _| active.iter().any(|a| a.eq_ignore_ascii_case(id)));
        if self.by_storm.len() != before {
            self.revision += 1;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &StormGuidance> {
        self.by_storm.values()
    }

    pub fn is_empty(&self) -> bool {
        self.by_storm.is_empty()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// The least recently fetched storm. The legend shows its age, so the
    /// legend never claims fresher data than the stalest track on screen.
    pub fn oldest(&self) -> Option<&StormGuidance> {
        self.by_storm.values().min_by_key(|g| g.fetched_at_unix)
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass.**

Run: `cargo test -p rustywx --lib adeck::`
Expected: 33 passed.

- [ ] **Step 5: Commit.**

```bash
git add app/src/adeck.rs
git commit -m "feat(tropical): guidance store for active storms"
```

---

### Task 7: Wind colors, segment color, track style, and disclaimer text

**Files:**
- Create: `app/src/adeck_style.rs`
- Modify: `app/src/lib.rs` (add `pub mod adeck_style;` right after `pub mod adeck;`)

**Interfaces:**
- Consumes: `ModelGroup` (Task 4).
- Produces:
  - `pub enum WindCategory { Unknown, Depression, Storm, Cat1, Cat2, Cat3, Cat4, Cat5 }`, with `LEGEND: [WindCategory; 8]`, `from_kt(Option<u32>) -> Self`, `rgb(self) -> [u8; 3]`, `hex(self) -> u32`, `label(self) -> &'static str`, and `min_kt(self) -> Option<u32>`
  - `pub fn segment_rgba(group: ModelGroup, start_kt: Option<u32>, end_kt: Option<u32>) -> [u8; 4]`
  - `pub struct TrackStyle { pub width_px: f32, pub underlay_px: Option<f32>, pub label: bool, pub draw_rank: u8 }` and `pub fn track_style(group: ModelGroup) -> TrackStyle`
  - `pub const DISCLAIMER_LINES: [&str; 6]` and `pub const MODEL_SUMMARY_URL: &str`

- [ ] **Step 1: Write the failing tests.** Create `app/src/adeck_style.rs`:

```rust
//! Presentation rules for the spaghetti-models overlay: wind-category colours,
//! per-segment colour, line styles, the km-projection cache, cycle/age text,
//! the disclaimer, and intensity-chart scaling. Pure: no drawing calls.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adeck::ModelGroup;

    #[test]
    fn wind_category_thresholds_follow_saffir_simpson() {
        let cases = [
            (33, WindCategory::Depression),
            (34, WindCategory::Storm),
            (63, WindCategory::Storm),
            (64, WindCategory::Cat1),
            (82, WindCategory::Cat1),
            (83, WindCategory::Cat2),
            (95, WindCategory::Cat2),
            (96, WindCategory::Cat3),
            (112, WindCategory::Cat3),
            (113, WindCategory::Cat4),
            (136, WindCategory::Cat4),
            (137, WindCategory::Cat5),
        ];
        for (kt, cat) in cases {
            assert_eq!(WindCategory::from_kt(Some(kt)), cat, "{kt} kt");
        }
        assert_eq!(WindCategory::from_kt(None), WindCategory::Unknown);
    }

    #[test]
    fn legend_colors_are_distinct() {
        let mut colors: Vec<u32> = WindCategory::LEGEND.iter().map(|c| c.hex()).collect();
        colors.sort_unstable();
        colors.dedup();
        assert_eq!(colors.len(), 8);
        assert_eq!(WindCategory::Cat5.hex(), 0xFF6060);
    }

    #[test]
    fn segment_color_uses_start_wind_then_end() {
        let start = segment_rgba(ModelGroup::Global, Some(100), Some(20));
        assert_eq!(&start[..3], &WindCategory::Cat3.rgb());
        let fallback = segment_rgba(ModelGroup::Global, None, Some(40));
        assert_eq!(&fallback[..3], &WindCategory::Storm.rgb());
        let unknown = segment_rgba(ModelGroup::Consensus, None, None);
        assert_eq!(&unknown[..3], &WindCategory::Unknown.rgb());
    }

    #[test]
    fn observed_track_is_grey_and_official_is_opaque() {
        assert_eq!(segment_rgba(ModelGroup::Observed, Some(140), Some(140)), [0xB0, 0xB0, 0xB0, 200]);
        assert_eq!(segment_rgba(ModelGroup::Official, Some(50), None)[3], 255);
        assert!(segment_rgba(ModelGroup::Hurricane, Some(50), None)[3] < 255);
    }

    #[test]
    fn official_track_draws_last_and_widest() {
        let ofcl = track_style(ModelGroup::Official);
        let carq = track_style(ModelGroup::Observed);
        let model = track_style(ModelGroup::Hurricane);
        assert!(ofcl.draw_rank > carq.draw_rank && carq.draw_rank > model.draw_rank);
        assert!(ofcl.width_px > model.width_px);
        assert!(ofcl.underlay_px.is_some() && model.underlay_px.is_none());
        assert!(!carq.label && model.label && ofcl.label);
    }

    #[test]
    fn disclaimer_lines_fit_the_panel() {
        // NHC panel content is ~288 px; 10 px Inter is ~5.5 px/char.
        for line in DISCLAIMER_LINES {
            assert!(line.chars().count() <= 48, "{line:?} too long");
        }
        assert!(DISCLAIMER_LINES.iter().any(|l| l.contains("not official forecasts")));
    }
}
```

Add `pub mod adeck_style;` to `app/src/lib.rs` after `pub mod adeck;`.

- [ ] **Step 2: Run the tests to verify they fail.**

Run: `cargo test -p rustywx --lib adeck_style::`
Expected: compile error `cannot find type 'WindCategory'`.

- [ ] **Step 3: Implement.** Insert above `#[cfg(test)]` in `app/src/adeck_style.rs`:

```rust
use crate::adeck::ModelGroup;

/// Saffir-Simpson wind category of a forecast point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindCategory {
    /// VMAX column was 0 (no value), e.g. TVCN.
    Unknown,
    Depression,
    Storm,
    Cat1,
    Cat2,
    Cat3,
    Cat4,
    Cat5,
}

impl WindCategory {
    /// Legend order: weakest to strongest, then "no intensity".
    pub const LEGEND: [WindCategory; 8] = [
        WindCategory::Depression,
        WindCategory::Storm,
        WindCategory::Cat1,
        WindCategory::Cat2,
        WindCategory::Cat3,
        WindCategory::Cat4,
        WindCategory::Cat5,
        WindCategory::Unknown,
    ];

    pub fn from_kt(kt: Option<u32>) -> Self {
        match kt {
            None => WindCategory::Unknown,
            Some(k) if k < 34 => WindCategory::Depression,
            Some(k) if k < 64 => WindCategory::Storm,
            Some(k) if k < 83 => WindCategory::Cat1,
            Some(k) if k < 96 => WindCategory::Cat2,
            Some(k) if k < 113 => WindCategory::Cat3,
            Some(k) if k < 137 => WindCategory::Cat4,
            Some(_) => WindCategory::Cat5,
        }
    }

    /// Standard Saffir-Simpson scale colours; grey for unknown.
    pub fn rgb(self) -> [u8; 3] {
        match self {
            WindCategory::Unknown => [0x9E, 0x95, 0x90],
            WindCategory::Depression => [0x5E, 0xBA, 0xFF],
            WindCategory::Storm => [0x00, 0xFA, 0xF4],
            WindCategory::Cat1 => [0xFF, 0xFF, 0xCC],
            WindCategory::Cat2 => [0xFF, 0xE7, 0x75],
            WindCategory::Cat3 => [0xFF, 0xC1, 0x40],
            WindCategory::Cat4 => [0xFF, 0x8F, 0x20],
            WindCategory::Cat5 => [0xFF, 0x60, 0x60],
        }
    }

    /// `0xRRGGBB` for Ply `background_color`.
    pub fn hex(self) -> u32 {
        let [r, g, b] = self.rgb();
        (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
    }

    pub fn label(self) -> &'static str {
        match self {
            WindCategory::Unknown => "No intensity",
            WindCategory::Depression => "TD  <34 kt",
            WindCategory::Storm => "TS  34-63 kt",
            WindCategory::Cat1 => "Cat 1  64-82 kt",
            WindCategory::Cat2 => "Cat 2  83-95 kt",
            WindCategory::Cat3 => "Cat 3  96-112 kt",
            WindCategory::Cat4 => "Cat 4  113-136 kt",
            WindCategory::Cat5 => "Cat 5  137+ kt",
        }
    }

    /// Lower bound in kt (intensity-chart bands); `None` for Unknown.
    pub fn min_kt(self) -> Option<u32> {
        match self {
            WindCategory::Unknown => None,
            WindCategory::Depression => Some(0),
            WindCategory::Storm => Some(34),
            WindCategory::Cat1 => Some(64),
            WindCategory::Cat2 => Some(83),
            WindCategory::Cat3 => Some(96),
            WindCategory::Cat4 => Some(113),
            WindCategory::Cat5 => Some(137),
        }
    }
}

/// RGBA for the segment between two consecutive points: coloured by the
/// wind at its start (falling back to its end). CARQ (observed) is plain
/// grey; OFCL is opaque, models slightly translucent.
pub fn segment_rgba(group: ModelGroup, start_kt: Option<u32>, end_kt: Option<u32>) -> [u8; 4] {
    if group == ModelGroup::Observed {
        return [0xB0, 0xB0, 0xB0, 200];
    }
    let [r, g, b] = WindCategory::from_kt(start_kt.or(end_kt)).rgb();
    let a = if group == ModelGroup::Official { 255 } else { 210 };
    [r, g, b, a]
}

/// Line style per model group.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackStyle {
    pub width_px: f32,
    /// White outline drawn under the line (OFCL only), in px.
    pub underlay_px: Option<f32>,
    /// Draw the tech id at the track's last point.
    pub label: bool,
    /// Higher draws later (on top).
    pub draw_rank: u8,
}

pub fn track_style(group: ModelGroup) -> TrackStyle {
    match group {
        ModelGroup::Official => TrackStyle {
            width_px: 2.5,
            underlay_px: Some(5.0),
            label: true,
            draw_rank: 2,
        },
        ModelGroup::Observed => TrackStyle {
            width_px: 1.5,
            underlay_px: None,
            label: false,
            draw_rank: 1,
        },
        _ => TrackStyle {
            width_px: 1.5,
            underlay_px: None,
            label: true,
            draw_rank: 0,
        },
    }
}

/// Full disclaimer and attribution, one panel line each (no auto-wrap).
/// Wording from `docs/research/spaghetti-models.md` section 4.
pub const DISCLAIMER_LINES: [&str; 6] = [
    "Model guidance from NHC ATCF (aid_public).",
    "Individual models are not official forecasts.",
    "The NHC forecast (OFCL) is the official track.",
    "Data are automated and may contain errors.",
    "Consult NHC and your local NWS office.",
    "Source: NOAA/NHC, public domain.",
];

/// NHC's model catalogue and guidance disclaimer.
pub const MODEL_SUMMARY_URL: &str = "https://www.nhc.noaa.gov/modelsummary.shtml";
```

- [ ] **Step 4: Run the tests to verify they pass.**

Run: `cargo test -p rustywx --lib adeck_style::`
Expected: 6 passed.

- [ ] **Step 5: Commit.**

```bash
git add app/src/adeck_style.rs app/src/lib.rs
git commit -m "feat(tropical): wind colours and line styles for model tracks"
```

---

### Task 8: Cycle and data-age text, staleness

**Files:**
- Modify: `app/src/adeck_style.rs`

**Interfaces:**
- Produces: `pub const STALE_AFTER_SECS: i64 = 1800`, `pub fn format_cycle(init: u32) -> String`, `pub fn format_age(fetched_at_unix: i64, now_unix: i64) -> String`, `pub fn is_stale(fetched_at_unix: i64, now_unix: i64) -> bool`.

- [ ] **Step 1: Write the failing tests.** Add to `mod tests` in `app/src/adeck_style.rs`:

```rust
    #[test]
    fn format_cycle_reads_init_digits() {
        assert_eq!(format_cycle(2026100912), "12Z Oct 9");
        assert_eq!(format_cycle(2026010100), "00Z Jan 1");
        // Garbage never panics.
        assert_eq!(format_cycle(0), "00Z ??? 0");
        assert_eq!(format_cycle(2026130100), "00Z ??? 1");
    }

    #[test]
    fn format_age_buckets() {
        assert_eq!(format_age(1000, 1000), "just now");
        assert_eq!(format_age(1000, 1059), "just now");
        assert_eq!(format_age(1000, 1060), "1 min ago");
        assert_eq!(format_age(1000, 4599), "59 min ago");
        assert_eq!(format_age(1000, 4600), "1 h 0 min ago");
        assert_eq!(format_age(1000, 900), "just now", "clock skew clamps to 0");
    }

    #[test]
    fn is_stale_boundary() {
        assert!(!is_stale(0, STALE_AFTER_SECS));
        assert!(is_stale(0, STALE_AFTER_SECS + 1));
    }
```

- [ ] **Step 2: Run the tests to verify they fail.**

Run: `cargo test -p rustywx --lib adeck_style::`
Expected: compile error `cannot find function 'format_cycle'`.

- [ ] **Step 3: Implement.** Add above `#[cfg(test)]`:

```rust
/// Data older than this is labelled "(stale)": two missed 15-min refreshes.
pub const STALE_AFTER_SECS: i64 = 30 * 60;

/// `2026100912` -> `"12Z Oct 9"`. Arithmetic on the integer, no string
/// slicing, so malformed values just print `???`.
pub fn format_cycle(init: u32) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let hour = init % 100;
    let day = (init / 100) % 100;
    let month = (init / 10_000) % 100;
    let name = (month as usize)
        .checked_sub(1)
        .and_then(|i| MONTHS.get(i))
        .copied()
        .unwrap_or("???");
    format!("{hour:02}Z {name} {day}")
}

/// Human age of a fetch: "just now", "7 min ago", "2 h 5 min ago".
pub fn format_age(fetched_at_unix: i64, now_unix: i64) -> String {
    let secs = (now_unix - fetched_at_unix).max(0);
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{} min ago", secs / 60)
    } else {
        format!("{} h {} min ago", secs / 3600, (secs % 3600) / 60)
    }
}

pub fn is_stale(fetched_at_unix: i64, now_unix: i64) -> bool {
    now_unix - fetched_at_unix > STALE_AFTER_SECS
}
```

- [ ] **Step 4: Run the tests to verify they pass.**

Run: `cargo test -p rustywx --lib adeck_style::`
Expected: 9 passed.

- [ ] **Step 5: Commit.**

```bash
git add app/src/adeck_style.rs
git commit -m "feat(tropical): cycle and data-age labels for model guidance"
```

---

### Task 9: Background parse payload (gunzip and parse off the UI thread)

**Files:**
- Modify: `app/src/background_work.rs` (`ParsePayload` around l.42, `phase()`/`payload_size()` around l.57–88, `ParsedData` around l.104, `parse()` around l.330, `impl BackgroundWorkers` around l.300)

**Interfaces:**
- Consumes: `crate::adeck::{StormGuidance, guidance_from_gzip}` (Task 5).
- Produces: `ParsePayload::ADeck { storm_id: String, gz: Vec<u8> }`, `ParsedData::ADeck(crate::adeck::StormGuidance)`, `BackgroundWorkers::parse_queue_has_capacity(&self) -> bool`.

- [ ] **Step 1: Write the failing tests.** Add to `mod tests` in `app/src/background_work.rs`:

```rust
    fn gzip_fixture() -> Vec<u8> {
        use std::io::Write;
        let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        enc.write_all(include_str!("testdata/aal092026_sample.dat").as_bytes())
            .unwrap();
        enc.finish().unwrap()
    }

    fn run_adeck_parse(gz: Vec<u8>) -> ParseResult {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let _guard = runtime.enter();
        let mut workers = BackgroundWorkers::new(runtime.handle());
        assert!(workers.parse_queue_has_capacity());
        workers
            .submit_parse(ParseRequest {
                generation: 21,
                payload: ParsePayload::ADeck {
                    storm_id: "al092026".to_owned(),
                    gz,
                },
                submitted_at: Instant::now(),
            })
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(result) = workers.poll_parse() {
                return result;
            }
            assert!(Instant::now() < deadline, "parse worker timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn parse_worker_turns_adeck_gzip_into_guidance() {
        let result = run_adeck_parse(gzip_fixture());
        assert_eq!(result.generation, 21);
        match result.result {
            Ok(ParsedData::ADeck(g)) => {
                assert_eq!(g.storm_id, "al092026");
                assert_eq!(g.init, 2026100912);
            }
            Ok(_) => panic!("wrong ParsedData variant"),
            Err(e) => panic!("parse failed: {e}"),
        }
    }

    #[test]
    fn parse_worker_reports_bad_adeck_body_as_error() {
        let result = run_adeck_parse(b"<html>oops</html>".to_vec());
        match result.result {
            Err(e) => assert!(e.contains("not gzip"), "{e}"),
            Ok(_) => panic!("non-gzip body must fail"),
        }
    }
```

- [ ] **Step 2: Run the tests to verify they fail.**

Run: `cargo test -p rustywx --lib background_work::`
Expected: compile error `no variant named 'ADeck'`.

- [ ] **Step 3: Implement.** Make these edits in `app/src/background_work.rs`:

1. Add a variant to `pub enum ParsePayload` after `NhcMetadata { .. },`:

```rust
    /// Gzipped ATCF a-deck body for one storm (spaghetti-models overlay).
    ADeck { storm_id: String, gz: Vec<u8> },
```

2. In `fn phase`, add the arm `Self::ADeck { .. } => "adeck_parse",`.
3. In `fn payload_size`, add the arm `Self::ADeck { gz, .. } => gz.len(),`.
4. Add a variant to `pub enum ParsedData` after `NhcMetadata(NhcMetadata),`:

```rust
    ADeck(crate::adeck::StormGuidance),
```

5. In `fn parse`, add before the closing brace of the `match`:

```rust
        ParsePayload::ADeck { storm_id, gz } => crate::adeck::guidance_from_gzip(&storm_id, &gz)
            .map(ParsedData::ADeck)
            .map_err(|e| format!("{e:#}")),
```

6. In `impl BackgroundWorkers`, after `poll_parse`, add:

```rust
    /// True when `submit_parse` will not fail with `Full`. The UI thread is
    /// the only sender, so capacity can't vanish between this check and the
    /// send. Mirrors `image_queue_has_capacity`.
    pub fn parse_queue_has_capacity(&self) -> bool {
        self.parse_tx.capacity() > 0
    }
```

`main.rs` already has an `Ok(_) =>` catch-all on `ParsedData`, so it still compiles.

- [ ] **Step 4: Run the tests to verify they pass.**

Run: `cargo test -p rustywx --lib background_work::`
Expected: all background_work tests pass, including the 2 new ones.

- [ ] **Step 5: Commit.**

```bash
git add app/src/background_work.rs
git commit -m "feat(tropical): parse a-deck gzip on the background worker"
```

---

### Task 10: A-deck fetch scheduler (stable id plus `cancel()`, deadline, backoff, one toast per streak)

**Files:**
- Create: `app/src/adeck_fetch.rs`
- Modify: `app/src/lib.rs` (add `pub mod adeck_fetch;` after `pub mod adeck;`)

**Design note:** Ply 1.1.1's `Request::cancel()` removes the cached entry and frees the id (`ply-engine-1.1.1/src/net/http.rs`, `docs/research/windows-stability-remediation.md` networking section). So each storm uses one stable id, `nhc-adeck-{storm_id}`. The scheduler calls `cancel()` after **every** finished response (Ok or Err) and on the app-side deadline. That way the id is always free before the next attempt, and the failed result can't be re-read for 60 frames (review C2). `cancel()` does not stop the ureq thread; the result is dropped.

The `NetPort` trait is the **transport boundary**. `PlyNet` is the only code in this feature that touches Ply `net`. The planned app-owned reqwest module (`app/src/http.rs`, remediation plan) will replace it by implementing `NetPort`, with no change to `adeck.rs`, `adeck_style.rs`, `scope.rs`, or the scheduler. **Don't build that module here.**

**Interfaces:**
- Consumes: `crate::adeck::adeck_url` (Task 2).
- Produces:
  - Constants: `pub const REFRESH_SECS: f64 = 900.0`, `pub const ATTEMPT_TIMEOUT_SECS: f64 = 60.0`, `pub const BACKOFF_BASE_SECS: f64 = 30.0`
  - `pub fn backoff_secs(failures: u32) -> f64`
  - `pub fn adeck_net_id(storm_id: &str) -> String` (returns `"nhc-adeck-{storm_id}"`)
  - `pub enum NetPoll { Pending, Done(Result<Vec<u8>, String>), Missing }`
  - `pub trait NetPort { fn get(&mut self, id: &str, url: &str); fn poll(&mut self, id: &str) -> NetPoll; fn cancel(&mut self, id: &str); }`
  - `pub struct PlyNet;` implementing `NetPort`
  - `pub enum SlotStatus { Idle, Downloading, Parsing, Retrying { in_secs: u32 } }`
  - `pub fn status_message(status: &SlotStatus) -> String`
  - `#[derive(Default)] pub struct TickReport { pub show_toast: bool }` and `#[derive(Debug, PartialEq)] pub struct ParseOutcome { pub storm_id: String, pub show_toast: bool }`
  - `#[derive(Default)] pub struct AdeckFetcher`, with:
    - `new() -> Self`
    - `tick(&mut self, now: f64, active: &[String], net: &mut impl NetPort) -> TickReport`
    - `take_parse_job(&mut self, generation: u64) -> Option<(String, Vec<u8>)>`
    - `owns_generation(&self, generation: u64) -> bool`
    - `parse_finished(&mut self, generation: u64, ok: bool, now: f64) -> Option<ParseOutcome>`
    - `disable(&mut self, net: &mut impl NetPort)`
    - `is_active(&self) -> bool`
    - `status(&self, storm_id: &str, now: f64) -> SlotStatus`

`now` is `macroquad::get_time()` seconds (monotonic, app-relative), as used everywhere in `main.rs`.

- [ ] **Step 1: Write the failing tests.** Create `app/src/adeck_fetch.rs`. `FakeNet` models Ply's real semantics: `get` on an id that still has an entry is **ignored**, and only `cancel` (or eviction) frees it. So a test fails if the scheduler ever re-fires without cancelling first.

```rust
//! Per-storm a-deck download scheduler for the spaghetti-models overlay.
//!
//! Ply `net` ignores `get` for an id that still has an entry and keeps a
//! finished result (including an `Err`) for 60 frames; it also has no request
//! timeout (`docs/reviews/windows-stability-review.md` C2/C3). Each storm
//! uses one stable id, and the scheduler calls `cancel()` after every result
//! and on an app-side deadline, which frees the id for the retry. Failures
//! back off exponentially, and only the first failure of a streak asks for a
//! toast. Each storm has its own slot, so a hung download never blocks
//! another storm or any other NHC data.
//!
//! `NetPort` is the transport boundary: the planned app-owned HTTP module
//! can replace `PlyNet` without touching the parser, renderer, or scheduler.

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    /// Scripted stand-in for Ply net with its real id semantics.
    #[derive(Default)]
    struct FakeNet {
        /// Ids with an entry (pending or finished, not yet cancelled).
        live: HashSet<String>,
        fired: Vec<(String, String)>,
        /// `get` calls Ply would have silently ignored. Must stay empty.
        ignored: Vec<String>,
        responses: HashMap<String, NetPoll>,
        cancelled: Vec<String>,
    }

    impl FakeNet {
        fn evict(&mut self, id: &str) {
            self.live.remove(id);
        }
        fn respond(&mut self, id: &str, poll: NetPoll) {
            self.responses.insert(id.to_owned(), poll);
        }
    }

    impl NetPort for FakeNet {
        fn get(&mut self, id: &str, url: &str) {
            if !self.live.insert(id.to_owned()) {
                self.ignored.push(id.to_owned());
                return;
            }
            self.fired.push((id.to_owned(), url.to_owned()));
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

    const AL: &str = "nhc-adeck-al092026";
    const EP: &str = "nhc-adeck-ep182026";

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn fires_once_per_active_storm_with_stable_ids() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        f.tick(0.0, &ids(&["al092026", "EP182026"]), &mut net);
        assert_eq!(
            net.fired,
            [
                (
                    AL.to_owned(),
                    "https://ftp.nhc.noaa.gov/atcf/aid_public/aal092026.dat.gz".to_owned()
                ),
                (
                    EP.to_owned(),
                    "https://ftp.nhc.noaa.gov/atcf/aid_public/aep182026.dat.gz".to_owned()
                ),
            ]
        );
        f.tick(1.0, &ids(&["al092026", "ep182026"]), &mut net);
        assert_eq!(net.fired.len(), 2, "no re-fire while in flight");
        assert!(net.ignored.is_empty());
        assert_eq!(f.status("al092026", 1.0), SlotStatus::Downloading);
    }

    #[test]
    fn ignores_unsupported_storm_ids() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        f.tick(0.0, &ids(&["wp012026", "al852026", ""]), &mut net);
        assert!(net.fired.is_empty());
        assert!(!f.is_active());
    }

    #[test]
    fn success_cancels_handle_then_refreshes_same_id_later() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let active = ids(&["al092026"]);
        f.tick(0.0, &active, &mut net);
        net.respond(AL, NetPoll::Done(Ok(vec![1, 2, 3])));
        assert!(!f.tick(1.0, &active, &mut net).show_toast);
        assert_eq!(net.cancelled, [AL], "finished entry is released");
        assert_eq!(f.take_parse_job(7), Some(("al092026".to_owned(), vec![1, 2, 3])));
        assert_eq!(f.take_parse_job(8), None);
        assert!(f.owns_generation(7));
        assert_eq!(f.status("al092026", 1.0), SlotStatus::Parsing);
        let outcome = f.parse_finished(7, true, 2.0).unwrap();
        assert_eq!(outcome.storm_id, "al092026");
        assert!(!outcome.show_toast);
        assert!(!f.owns_generation(7));
        f.tick(901.0, &active, &mut net);
        assert_eq!(net.fired.len(), 1, "not due before REFRESH_SECS");
        f.tick(902.0, &active, &mut net);
        assert_eq!(net.fired.len(), 2);
        assert_eq!(net.fired[1].0, AL, "same stable id, re-fired after cancel");
        assert!(net.ignored.is_empty());
    }

    #[test]
    fn network_error_cancels_then_retries_with_backoff() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let active = ids(&["al092026"]);
        f.tick(0.0, &active, &mut net);
        net.respond(AL, NetPoll::Done(Err("http status: 503".into())));
        assert!(f.tick(1.0, &active, &mut net).show_toast, "first failure toasts");
        assert_eq!(net.cancelled, [AL], "failed entry is released, not re-read");
        assert_eq!(f.status("al092026", 1.0), SlotStatus::Retrying { in_secs: 30 });
        f.tick(30.0, &active, &mut net);
        assert_eq!(net.fired.len(), 1);
        f.tick(31.0, &active, &mut net);
        assert_eq!(net.fired.len(), 2, "retry fires after backoff");
        assert!(net.ignored.is_empty(), "retry was not swallowed by a stale entry");
        net.respond(AL, NetPoll::Done(Err("http status: 503".into())));
        assert!(!f.tick(32.0, &active, &mut net).show_toast, "one toast per streak");
        assert_eq!(f.status("al092026", 32.0), SlotStatus::Retrying { in_secs: 60 });
    }

    #[test]
    fn not_found_retries_quietly() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let active = ids(&["al942026"]);
        f.tick(0.0, &active, &mut net);
        net.respond("nhc-adeck-al942026", NetPoll::Done(Err("http status: 404".into())));
        assert!(!f.tick(1.0, &active, &mut net).show_toast);
        assert!(matches!(f.status("al942026", 1.0), SlotStatus::Retrying { .. }));
    }

    #[test]
    fn deadline_cancels_hung_request_and_retries() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let active = ids(&["al092026"]);
        f.tick(0.0, &active, &mut net);
        f.tick(59.9, &active, &mut net);
        assert!(net.cancelled.is_empty());
        assert!(f.tick(60.0, &active, &mut net).show_toast);
        assert_eq!(net.cancelled, [AL]);
        f.tick(90.0, &active, &mut net);
        assert_eq!(net.fired.len(), 2);
        assert!(net.ignored.is_empty());
    }

    #[test]
    fn evicted_handle_counts_as_failure() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let active = ids(&["al092026"]);
        f.tick(0.0, &active, &mut net);
        net.evict(AL);
        f.tick(1.0, &active, &mut net);
        assert!(matches!(f.status("al092026", 1.0), SlotStatus::Retrying { .. }));
    }

    #[test]
    fn parse_failure_backs_off_and_toast_resets_after_success() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let active = ids(&["al092026"]);
        f.tick(0.0, &active, &mut net);
        net.respond(AL, NetPoll::Done(Ok(vec![0])));
        f.tick(1.0, &active, &mut net);
        f.take_parse_job(1).unwrap();
        assert!(f.parse_finished(1, false, 1.0).unwrap().show_toast);
        assert!(matches!(f.status("al092026", 1.0), SlotStatus::Retrying { .. }));
        // Retry succeeds, which ends the streak...
        f.tick(31.0, &active, &mut net);
        net.respond(AL, NetPoll::Done(Ok(vec![0])));
        f.tick(32.0, &active, &mut net);
        f.take_parse_job(2).unwrap();
        assert!(!f.parse_finished(2, true, 32.0).unwrap().show_toast);
        // ...so the next failure toasts again.
        f.tick(932.0, &active, &mut net);
        net.respond(AL, NetPoll::Done(Err("http status: 500".into())));
        assert!(f.tick(933.0, &active, &mut net).show_toast);
        assert!(net.ignored.is_empty());
    }

    #[test]
    fn removed_storm_cancels_download_and_disowns_parse() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let both = ids(&["al092026", "ep182026"]);
        f.tick(0.0, &both, &mut net);
        net.respond(EP, NetPoll::Done(Ok(vec![9])));
        f.tick(1.0, &both, &mut net);
        let (storm, _) = f.take_parse_job(5).unwrap();
        assert_eq!(storm, "ep182026");
        f.tick(2.0, &[], &mut net);
        assert!(net.cancelled.contains(&AL.to_owned()));
        assert!(!f.owns_generation(5), "late parse result must be dropped");
        assert_eq!(f.parse_finished(5, true, 3.0), None);
        assert_eq!(f.status("al092026", 2.0), SlotStatus::Idle);
    }

    #[test]
    fn hung_storm_does_not_block_other_storms() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let both = ids(&["al092026", "ep182026"]);
        f.tick(0.0, &both, &mut net);
        net.respond(EP, NetPoll::Done(Ok(vec![4])));
        f.tick(1.0, &both, &mut net);
        assert_eq!(f.take_parse_job(1), Some(("ep182026".to_owned(), vec![4])));
        assert_eq!(f.status("al092026", 1.0), SlotStatus::Downloading);
    }

    #[test]
    fn disable_cancels_and_enable_refires() {
        let mut f = AdeckFetcher::new();
        let mut net = FakeNet::default();
        let active = ids(&["al092026"]);
        f.tick(0.0, &active, &mut net);
        f.disable(&mut net);
        assert_eq!(net.cancelled, [AL]);
        assert!(!f.is_active());
        f.tick(5.0, &active, &mut net);
        assert_eq!(net.fired.len(), 2);
        assert!(net.ignored.is_empty());
    }

    #[test]
    fn backoff_doubles_then_caps() {
        let table: Vec<f64> = (0..=7).map(backoff_secs).collect();
        assert_eq!(table, [0.0, 30.0, 60.0, 120.0, 240.0, 480.0, 900.0, 900.0]);
        assert_eq!(backoff_secs(u32::MAX), 900.0);
    }

    #[test]
    fn status_messages() {
        assert_eq!(status_message(&SlotStatus::Downloading), "Downloading model guidance\u{2026}");
        assert_eq!(
            status_message(&SlotStatus::Retrying { in_secs: 30 }),
            "Model guidance unavailable, retrying in 30 s"
        );
        assert_eq!(status_message(&SlotStatus::Idle), "No model guidance yet");
    }
}
```

Add `pub mod adeck_fetch;` to `app/src/lib.rs` after `pub mod adeck;`.

- [ ] **Step 2: Run the tests to verify they fail.**

Run: `cargo test -p rustywx --lib adeck_fetch::`
Expected: compile error `cannot find trait 'NetPort'`.

- [ ] **Step 3: Implement.** Insert above `#[cfg(test)]` in `app/src/adeck_fetch.rs`:

```rust
use crate::adeck::adeck_url;
use std::collections::BTreeMap;

/// Refresh period while the overlay is on. A-decks fill in over hours; the
/// research asks for no faster than every 10-15 min.
pub const REFRESH_SECS: f64 = 900.0;
/// App-side deadline per attempt. Ply net sets no HTTP timeout (review C3);
/// a 350 KB file normally lands in a few seconds.
pub const ATTEMPT_TIMEOUT_SECS: f64 = 60.0;
/// First retry delay; doubles per consecutive failure, capped at REFRESH_SECS.
pub const BACKOFF_BASE_SECS: f64 = 30.0;

/// Delay before the next attempt after `failures` consecutive failures.
pub fn backoff_secs(failures: u32) -> f64 {
    if failures == 0 {
        return 0.0;
    }
    let exp = (failures - 1).min(5);
    (BACKOFF_BASE_SECS * f64::from(1u32 << exp)).min(REFRESH_SECS)
}

/// Stable per-storm request id. Reusable because every result is cancelled.
pub fn adeck_net_id(storm_id: &str) -> String {
    format!("nhc-adeck-{storm_id}")
}

/// Result of polling one request id.
pub enum NetPoll {
    Pending,
    Done(Result<Vec<u8>, String>),
    /// No such id: Ply evicted it or it was never fired.
    Missing,
}

/// Transport boundary for a-deck downloads. `PlyNet` implements it today; the
/// planned app-owned HTTP module replaces it later. Contract: after `cancel`,
/// the id is free and a new `get` with it starts a fresh request.
pub trait NetPort {
    fn get(&mut self, id: &str, url: &str);
    fn poll(&mut self, id: &str) -> NetPoll;
    fn cancel(&mut self, id: &str);
}

/// `NetPort` over Ply 1.1 `net`. The only Ply-specific code in the feature.
pub struct PlyNet;

impl NetPort for PlyNet {
    fn get(&mut self, id: &str, url: &str) {
        use ply_engine::prelude::net;
        // The scheduler only fires from Idle, so any entry still under this id
        // is stale; free it, or Ply would silently ignore this `get`.
        if let Some(req) = net::request(id) {
            req.cancel();
        }
        net::get(id, url, |c| c.header("User-Agent", "rustywx/0.3"));
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

    /// Removes Ply's entry and frees the id. A hung ureq thread keeps running
    /// until the socket dies, but its result goes to a dropped channel.
    fn cancel(&mut self, id: &str) {
        use ply_engine::prelude::net;
        if let Some(req) = net::request(id) {
            req.cancel();
        }
    }
}

/// What the NHC panel shows for a storm with no (or refreshing) guidance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SlotStatus {
    Idle,
    Downloading,
    Parsing,
    Retrying { in_secs: u32 },
}

pub fn status_message(status: &SlotStatus) -> String {
    match status {
        SlotStatus::Idle => "No model guidance yet".to_string(),
        SlotStatus::Downloading => "Downloading model guidance\u{2026}".to_string(),
        SlotStatus::Parsing => "Processing model guidance\u{2026}".to_string(),
        SlotStatus::Retrying { in_secs } => {
            format!("Model guidance unavailable, retrying in {in_secs} s")
        }
    }
}

#[derive(Default)]
pub struct TickReport {
    /// Caller should `show_toast(.., ErrorKind::Network)`.
    pub show_toast: bool,
}

#[derive(Debug, PartialEq)]
pub struct ParseOutcome {
    pub storm_id: String,
    pub show_toast: bool,
}

enum Phase {
    Idle,
    InFlight { started_at: f64 },
    Downloaded(Vec<u8>),
    Parsing { generation: u64 },
}

struct Slot {
    url: String,
    net_id: String,
    phase: Phase,
    next_due: f64,
    failures: u32,
}

/// Count a failure: back off, go idle, and say whether to toast (first
/// failure of a streak only; 404 never toasts).
fn record_failure(slot: &mut Slot, toast_shown: &mut bool, now: f64, quiet: bool) -> bool {
    slot.failures = slot.failures.saturating_add(1);
    slot.next_due = now + backoff_secs(slot.failures);
    slot.phase = Phase::Idle;
    if quiet || *toast_shown {
        return false;
    }
    *toast_shown = true;
    true
}

#[derive(Default)]
pub struct AdeckFetcher {
    slots: BTreeMap<String, Slot>,
    toast_shown_in_streak: bool,
}

impl AdeckFetcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Per-frame driver. `active` are `CurrentStorms.json` ids; unsupported
    /// ids are ignored. Never blocks: it only fires, polls, and cancels.
    pub fn tick(&mut self, now: f64, active: &[String], net: &mut impl NetPort) -> TickReport {
        let mut report = TickReport::default();
        let wanted: Vec<(String, String)> = active
            .iter()
            .filter_map(|id| {
                let id = id.trim().to_ascii_lowercase();
                adeck_url(&id).map(|url| (id, url))
            })
            .collect();

        // Storms that left CurrentStorms.json: cancel and forget.
        let gone: Vec<String> = self
            .slots
            .keys()
            .filter(|k| !wanted.iter().any(|(id, _)| id == *k))
            .cloned()
            .collect();
        for id in gone {
            if let Some(slot) = self.slots.remove(&id)
                && matches!(slot.phase, Phase::InFlight { .. })
            {
                net.cancel(&slot.net_id);
            }
        }
        for (id, url) in wanted {
            let net_id = adeck_net_id(&id);
            self.slots.entry(id).or_insert(Slot {
                url,
                net_id,
                phase: Phase::Idle,
                next_due: now,
                failures: 0,
            });
        }

        for (storm_id, slot) in self.slots.iter_mut() {
            match slot.phase {
                Phase::Idle if now >= slot.next_due => {
                    net.get(&slot.net_id, &slot.url);
                    slot.phase = Phase::InFlight { started_at: now };
                }
                Phase::InFlight { started_at } => {
                    // Every exit path cancels, so the id is free for the retry
                    // and a cached Err is never re-read (review C2).
                    let outcome = if now - started_at >= ATTEMPT_TIMEOUT_SECS {
                        net.cancel(&slot.net_id);
                        Some(Err(format!("timed out after {ATTEMPT_TIMEOUT_SECS} s")))
                    } else {
                        match net.poll(&slot.net_id) {
                            NetPoll::Pending => None,
                            NetPoll::Done(result) => {
                                net.cancel(&slot.net_id);
                                Some(result)
                            }
                            NetPoll::Missing => Some(Err("request handle evicted".to_owned())),
                        }
                    };
                    match outcome {
                        None => {}
                        Some(Ok(bytes)) => slot.phase = Phase::Downloaded(bytes),
                        Some(Err(e)) => {
                            eprintln!("Warning: a-deck download failed storm={storm_id}: {e}");
                            let quiet = e.contains("404");
                            if record_failure(slot, &mut self.toast_shown_in_streak, now, quiet) {
                                report.show_toast = true;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        report
    }

    /// Hand the next downloaded body to the parse worker, tagging it with
    /// `generation`. Call only when the parse queue has capacity.
    pub fn take_parse_job(&mut self, generation: u64) -> Option<(String, Vec<u8>)> {
        let (storm_id, slot) = self
            .slots
            .iter_mut()
            .find(|(_, s)| matches!(s.phase, Phase::Downloaded(_)))?;
        match std::mem::replace(&mut slot.phase, Phase::Parsing { generation }) {
            Phase::Downloaded(bytes) => Some((storm_id.clone(), bytes)),
            other => {
                slot.phase = other;
                None
            }
        }
    }

    pub fn owns_generation(&self, generation: u64) -> bool {
        self.slots
            .values()
            .any(|s| matches!(s.phase, Phase::Parsing { generation: g } if g == generation))
    }

    /// Record a parse result. `None` when the generation isn't ours (storm
    /// removed or overlay disabled meanwhile): the caller drops the result.
    pub fn parse_finished(&mut self, generation: u64, ok: bool, now: f64) -> Option<ParseOutcome> {
        let (storm_id, slot) = self.slots.iter_mut().find(
            |(_, s)| matches!(s.phase, Phase::Parsing { generation: g } if g == generation),
        )?;
        let show_toast = if ok {
            slot.failures = 0;
            slot.next_due = now + REFRESH_SECS;
            slot.phase = Phase::Idle;
            self.toast_shown_in_streak = false;
            false
        } else {
            record_failure(slot, &mut self.toast_shown_in_streak, now, false)
        };
        Some(ParseOutcome {
            storm_id: storm_id.clone(),
            show_toast,
        })
    }

    /// Overlay switched off: cancel downloads and forget all slots, so
    /// switching it back on fetches immediately.
    pub fn disable(&mut self, net: &mut impl NetPort) {
        for slot in self.slots.values() {
            if matches!(slot.phase, Phase::InFlight { .. }) {
                net.cancel(&slot.net_id);
            }
        }
        self.slots.clear();
        self.toast_shown_in_streak = false;
    }

    pub fn is_active(&self) -> bool {
        !self.slots.is_empty()
    }

    pub fn status(&self, storm_id: &str, now: f64) -> SlotStatus {
        let Some(slot) = self.slots.get(&storm_id.to_ascii_lowercase()) else {
            return SlotStatus::Idle;
        };
        match slot.phase {
            Phase::InFlight { .. } => SlotStatus::Downloading,
            Phase::Downloaded(_) | Phase::Parsing { .. } => SlotStatus::Parsing,
            Phase::Idle if slot.failures > 0 => SlotStatus::Retrying {
                in_secs: (slot.next_due - now).max(0.0).ceil() as u32,
            },
            Phase::Idle => SlotStatus::Idle,
        }
    }
}
```

`match slot.phase` on a non-`Copy` enum moves only `Copy` fields (`started_at: f64`); the `_` arm binds nothing. If the compiler complains about moving out of `slot.phase`, match on `&slot.phase` and copy `*started_at` into a local. If the borrow checker rejects `self.toast_shown_in_streak` inside the `self.slots.iter_mut()` loop, destructure first: `let Self { slots, toast_shown_in_streak } = self;` and use the locals. They are disjoint fields.

- [ ] **Step 4: Run the tests to verify they pass.**

Run: `cargo test -p rustywx --lib adeck_fetch::`
Expected: 13 passed.

- [ ] **Step 5: Commit.**

```bash
git add app/src/adeck_fetch.rs app/src/lib.rs
git commit -m "feat(tropical): resilient per-storm a-deck fetch scheduler"
```

---

### Task 11: Settings toggle (default OFF)

**Files:**
- Modify: `app/src/settings.rs` (struct after `melting_layer_hint_enabled`, `Default`, 3 tests)
- Modify: `app/src/widgets/settings.rs` (const id, plus a row after "Show tropical panel on startup", around l.305–311)
- Modify: `app/src/main.rs` (settings press handler, next to `MELTING_LAYER_TOGGLE_ID` around l.3644)

**Interfaces:**
- Produces: `Settings::show_model_tracks: bool` and `settings_widget::MODEL_TRACKS_TOGGLE_ID`.

- [ ] **Step 1: Write the failing tests.** In `app/src/settings.rs` tests:
  - In `default_matches_existing_startup_behaviour`, add `assert!(!settings.show_model_tracks);` after the melting-layer assert.
  - In `serde_round_trip`, add `show_model_tracks: true,` after `melting_layer_hint_enabled: true,`.
  - In `deserializes_settings_without_location_fields`, add:

```rust
        // Missing spaghetti-models field defaults off (opt-in overlay).
        assert!(!s.show_model_tracks);
```

- [ ] **Step 2: Run the tests to verify they fail.**

Run: `cargo test -p rustywx --lib settings::`
Expected: compile error `struct 'Settings' has no field named 'show_model_tracks'`.

- [ ] **Step 3: Implement.**
  1. In `pub struct Settings`, after `melting_layer_hint_enabled`, add:

```rust
    /// Whether the spaghetti-models overlay (ATCF model-guidance tracks,
    /// legend, intensity chart) is on. Default OFF: it is unofficial
    /// guidance and costs one extra download per active storm. Nothing is
    /// fetched while off (see `docs/plans/spaghetti-models-overlay.md`).
    #[serde(default)]
    pub show_model_tracks: bool,
```

  2. In `impl Default for Settings`, add `show_model_tracks: false,` after `melting_layer_hint_enabled: false,`.
  3. In `app/src/widgets/settings.rs`, after `pub const NHC_TOGGLE_ID ...`, add:

```rust
pub const MODEL_TRACKS_TOGGLE_ID: &str = "settings-toggle-model-tracks";
```

     After the `row(ui, "Show tropical panel on startup", ...)` block, add:

```rust
                    row(ui, "Spaghetti models (tropical)", |ui| {
                        bool_toggle(
                            ui,
                            MODEL_TRACKS_TOGGLE_ID,
                            "Spaghetti models (tropical)",
                            settings.show_model_tracks,
                        );
                    });
```

  4. In `app/src/main.rs`, inside the `if state.show_settings_panel {` input block, next to the `MELTING_LAYER_TOGGLE_ID` handler, add:

```rust
        if ply.is_just_pressed(settings_widget::MODEL_TRACKS_TOGGLE_ID) {
            state.settings.show_model_tracks = !state.settings.show_model_tracks;
            state.cache.save_settings(&state.settings);
        }
```

- [ ] **Step 4: Run the tests to verify they pass.**

Run: `cargo test -p rustywx --lib settings:: && cargo clippy -p rustywx --all-targets -- -D warnings`
Expected: settings tests pass; clippy is clean.

- [ ] **Step 5: Commit.**

```bash
git add app/src/settings.rs app/src/widgets/settings.rs app/src/main.rs
git commit -m "feat(tropical): spaghetti-models setting, default off"
```

---

### Task 12: Wire fetch and parse into the frame loop (no drawing yet)

**Files:**
- Modify: `app/src/state.rs` (fields after `alert_modal_scroll`, in the NHC block)
- Modify: `app/src/main.rs`:
  - `AppState` initializer around l.736–757
  - `use` block around l.9–37
  - parse-result match around l.1044–1129
  - new block after the NHC image upload block (after about l.1372)

**Interfaces:**
- Consumes: `AdeckFetcher`, `PlyNet`, `GuidanceStore`, `ParsePayload::ADeck`, `ParsedData::ADeck`, `parse_queue_has_capacity` (Tasks 6, 9, 10).
- Produces: `AppState.adeck_fetch: rustywx::adeck_fetch::AdeckFetcher`, `AppState.model_guidance: rustywx::adeck::GuidanceStore`, and `AppState.model_projection: Option<(usize, u64, Vec<rustywx::adeck_style::ProjectedTrack>)>`. Add `model_projection` in Task 13, together with the `ProjectedTrack` type.

This task has no unit test of its own: the scheduler and parser logic are covered in Tasks 9–10, and this is glue. Its gate is clippy plus the smoke test with logs.

- [ ] **Step 1: Add the state fields.** In `app/src/state.rs`, after `pub alert_modal_scroll: f32,`, add:

```rust
    // ── Spaghetti models (ATCF a-deck guidance, opt-in) ───────────
    /// Per-storm a-deck download scheduler (stable id + cancel, deadline, backoff).
    pub adeck_fetch: crate::adeck_fetch::AdeckFetcher,
    /// Last good model guidance per active storm.
    pub model_guidance: crate::adeck::GuidanceStore,
```

In the `AppState { .. }` initializer in `main.rs`, after `nhc_modal_scroll: 0.0,`, add:

```rust
        adeck_fetch: rustywx::adeck_fetch::AdeckFetcher::new(),
        model_guidance: rustywx::adeck::GuidanceStore::default(),
```

Next to `let mut nhc_image_results = ...` (about l.798), add:

```rust
    let mut ply_net = rustywx::adeck_fetch::PlyNet;
```

- [ ] **Step 2: Apply parse results.** In the `while let Some(parsed) = background.poll_parse()` match, add this arm **before** `Ok(_) => eprintln!(..stale..)`:

```rust
                Ok(ParsedData::ADeck(mut guidance))
                    if state.adeck_fetch.owns_generation(parsed.generation) =>
                {
                    let _ = state.adeck_fetch.parse_finished(parsed.generation, true, now);
                    guidance.fetched_at_unix = chrono::Utc::now().timestamp();
                    eprintln!(
                        "startup phase=adeck_parse_applied generation={} storm={} cycle={} tracks={} intensity={} elapsed_ms={}",
                        parsed.generation,
                        guidance.storm_id,
                        guidance.init,
                        guidance.tracks.len(),
                        guidance.intensity.len(),
                        parsed.elapsed.as_millis()
                    );
                    state.model_guidance.insert(guidance);
                }
```

Then add this arm **before** the existing `Err(error) => {` arm:

```rust
                Err(error) if state.adeck_fetch.owns_generation(parsed.generation) => {
                    eprintln!(
                        "Warning: a-deck parse failed generation={}: {error}",
                        parsed.generation
                    );
                    if state
                        .adeck_fetch
                        .parse_finished(parsed.generation, false, now)
                        .is_some_and(|o| o.show_toast)
                    {
                        show_toast(&mut state, now, toast_widget::ErrorKind::Network);
                    }
                }
```

- [ ] **Step 3: Add the per-frame fetch block.** Immediately after the NHC texture-upload block (the `if let Some(decoded) = nhc_image_results.pop_next() { .. }` that ends around l.1372), add:

```rust
        // ── Spaghetti models: per-storm a-deck fetch (opt-in) ─────────
        // Separate from the NHC state machine, so a hung a-deck can't stall
        // other tropical data. Cancel-and-retry, deadline, and backoff live in
        // AdeckFetcher; gunzip+parse run on the background worker.
        if state.settings.show_model_tracks && state.show_nhc_data {
            let active: Vec<String> = state
                .nhc_bundle
                .as_ref()
                .map(|b| b.metas.iter().map(|m| m.id.clone()).collect())
                .unwrap_or_default();
            state.model_guidance.retain_active(&active);
            if state.adeck_fetch.tick(now, &active, &mut ply_net).show_toast {
                show_toast(&mut state, now, toast_widget::ErrorKind::Network);
            }
            while background.parse_queue_has_capacity() {
                let generation = next_generation;
                let Some((storm_id, gz)) = state.adeck_fetch.take_parse_job(generation) else {
                    break;
                };
                next_generation += 1;
                let payload_bytes = gz.len();
                let request = ParseRequest {
                    generation,
                    payload: ParsePayload::ADeck {
                        storm_id: storm_id.clone(),
                        gz,
                    },
                    submitted_at: std::time::Instant::now(),
                };
                if background.submit_parse(request).is_ok() {
                    eprintln!(
                        "startup phase=adeck_response generation={generation} storm={storm_id} payload_bytes={payload_bytes}"
                    );
                } else if state
                    .adeck_fetch
                    .parse_finished(generation, false, now)
                    .is_some_and(|o| o.show_toast)
                {
                    show_toast(&mut state, now, toast_widget::ErrorKind::Network);
                }
            }
        } else if state.adeck_fetch.is_active() {
            state.adeck_fetch.disable(&mut ply_net);
        }
```

- [ ] **Step 4: Build, lint, and smoke.**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: clean.

Run (AGENTS.md smoke test, with logs captured):

```bash
just run 2> /tmp/rustywx-smoke.log &
sleep 3
kill %1 2>/dev/null || true
grep -c "adeck" /tmp/rustywx-smoke.log || true
```

Expected: the app launches without a panic, and with the default setting (OFF) the log has **0** `adeck` lines. To check the ON path, set `"show_model_tracks": true` in the app's `settings.json`, or toggle it in the Settings panel during a longer manual run (`just run`, open Settings, turn "Spaghetti models (tropical)" On, wait about 15 s, quit). The log should then show `phase=adeck_response` and `phase=adeck_parse_applied` lines for each active storm. If `CurrentStorms.json` lists no storms, there will be no `adeck` lines. That's expected; note it in the task report.

- [ ] **Step 5: Commit.**

```bash
git add app/src/state.rs app/src/main.rs
git commit -m "feat(tropical): fetch and parse a-decks while the overlay is on"
```

---

### Task 13: Draw tracks on the map (projection cache and per-segment wind color)

**Files:**
- Modify: `app/src/adeck_style.rs` (`ProjectedTrack`, `project_tracks`, tests)
- Modify: `app/src/scope.rs` (`draw_scope_to_texture` signature around l.1395, the NHC block around l.1600, and a new `fn draw_model_tracks` after `draw_nhc_overlays`)
- Modify: `app/src/state.rs` (the `model_projection` field)
- Modify: `app/src/main.rs` (state initializer, and the draw call around l.1588)

**Interfaces:**
- Consumes: `GuidanceStore`, `ModelGroup`, `track_style`, `segment_rgba` (Tasks 6, 7), `geo::point_to_km_offset`.
- Produces: `#[derive(Clone, Debug)] pub struct ProjectedTrack { pub tech: String, pub group: ModelGroup, pub km: Vec<Vec2>, pub vmax_kt: Vec<Option<u32>>, pub min_km: Vec2, pub max_km: Vec2 }` and `pub fn project_tracks(store: &GuidanceStore, origin_lat: f64, origin_lon: f64) -> Vec<ProjectedTrack>`. `draw_scope_to_texture` gains the parameter `model_tracks: Option<&[ProjectedTrack]>` right after `nhc`.

- [ ] **Step 1: Write the failing tests.** Add to `mod tests` in `app/src/adeck_style.rs`:

```rust
    use crate::adeck::{AidTrack, GuidanceStore, StormGuidance, TrackPoint};

    const FIXTURE: &str = include_str!("testdata/aal092026_sample.dat");

    fn fixture_store() -> GuidanceStore {
        let records = crate::adeck::parse_adeck(FIXTURE);
        let mut store = GuidanceStore::default();
        store.insert(crate::adeck::build_guidance("al092026", &records, 2026100912));
        store
    }

    #[test]
    fn project_tracks_orders_official_last() {
        let tracks = project_tracks(&fixture_store(), 27.0, -87.6);
        let techs: Vec<&str> = tracks.iter().map(|t| t.tech.as_str()).collect();
        assert_eq!(techs, ["AVNI", "HWFI", "TVCN", "CARQ", "OFCL"]);
        let ofcl = tracks.last().unwrap();
        assert!(ofcl.km[0].length() < 0.01, "OFCL tau 0 is at the origin");
        for t in &tracks {
            assert_eq!(t.km.len(), t.vmax_kt.len());
            for p in &t.km {
                assert!(p.x >= t.min_km.x && p.x <= t.max_km.x);
                assert!(p.y >= t.min_km.y && p.y <= t.max_km.y);
            }
        }
    }

    #[test]
    fn project_tracks_skips_hidden_models() {
        let mut store = fixture_store();
        let mut g = store.get("al092026").unwrap().clone();
        g.tracks.push(AidTrack {
            tech: "TABS".into(),
            group: ModelGroup::Trajectory,
            default_visible: false,
            points: g.tracks[0].points.clone(),
        });
        store.insert(g);
        assert!(project_tracks(&store, 27.0, -87.6).iter().all(|t| t.tech != "TABS"));
    }

    #[test]
    fn projection_is_continuous_across_antimeridian() {
        let point = |lon: f64| TrackPoint { tau_h: 0, lat: 20.0, lon, vmax_kt: Some(50) };
        let mut store = GuidanceStore::default();
        store.insert(StormGuidance {
            storm_id: "cp012026".into(),
            init: 2026100912,
            has_official: true,
            tracks: vec![AidTrack {
                tech: "AVNI".into(),
                group: ModelGroup::Global,
                default_visible: true,
                points: vec![point(179.5), point(-179.5)],
            }],
            intensity: Vec::new(),
            fetched_at_unix: 0,
        });
        let tracks = project_tracks(&store, 20.0, 179.0);
        let gap = (tracks[0].km[1] - tracks[0].km[0]).length();
        assert!(gap < 200.0, "1 degree of longitude, not a 360-degree wrap: {gap} km");
    }
```

- [ ] **Step 2: Run the tests to verify they fail.**

Run: `cargo test -p rustywx --lib adeck_style::`
Expected: compile error `cannot find function 'project_tracks'`.

- [ ] **Step 3: Implement the projection.** In `app/src/adeck_style.rs`, change the top `use` to:

```rust
use crate::adeck::{GuidanceStore, ModelGroup};
use crate::geo;
use ply_engine::prelude::Vec2;
```

Add above `#[cfg(test)]`:

```rust
/// A visible track pre-projected to km offsets from the radar site.
/// Rebuilt only when the site or the store changes, so per frame the draw
/// is a bbox cull plus one multiply-add per vertex (review H5).
#[derive(Clone, Debug)]
pub struct ProjectedTrack {
    pub tech: String,
    pub group: ModelGroup,
    pub km: Vec<Vec2>,
    pub vmax_kt: Vec<Option<u32>>,
    pub min_km: Vec2,
    pub max_km: Vec2,
}

/// Project every default-visible track, ordered by draw rank (models, then
/// CARQ, then OFCL on top).
pub fn project_tracks(store: &GuidanceStore, origin_lat: f64, origin_lon: f64) -> Vec<ProjectedTrack> {
    let mut out: Vec<ProjectedTrack> = store
        .iter()
        .flat_map(|g| g.tracks.iter())
        .filter(|t| t.default_visible && t.points.len() >= 2)
        .map(|t| {
            let km: Vec<Vec2> = t
                .points
                .iter()
                .map(|p| geo::point_to_km_offset(origin_lat, origin_lon, (p.lat, p.lon)))
                .collect();
            let min_km = km.iter().copied().reduce(Vec2::min).unwrap_or(Vec2::ZERO);
            let max_km = km.iter().copied().reduce(Vec2::max).unwrap_or(Vec2::ZERO);
            ProjectedTrack {
                tech: t.tech.clone(),
                group: t.group,
                vmax_kt: t.points.iter().map(|p| p.vmax_kt).collect(),
                km,
                min_km,
                max_km,
            }
        })
        .collect();
    out.sort_by_key(|t| track_style(t.group).draw_rank);
    out
}
```

Run: `cargo test -p rustywx --lib adeck_style::`. Expected: 12 passed.

- [ ] **Step 4: Draw in `scope.rs`.**
  1. Add `use crate::adeck_style::{self, ProjectedTrack};` to the `use` block.
  2. In `draw_scope_to_texture`, add the parameter `model_tracks: Option<&[ProjectedTrack]>,` directly after `nhc: Option<(&NhcBundle, &NhcOverlayState)>,`. Update the doc comment: "`model_tracks` adds the optional spaghetti-models overlay, drawn under the official NHC layers."
  3. Replace the NHC block (`if let Some((bundle, overlays)) = nhc { draw_nhc_overlays(...) }`) with:

```rust
    // ── NHC overlays (Stage 5) + model tracks (spaghetti) ───────
    if let Some((bundle, overlays)) = nhc {
        // Models first so the official cone/track sit on top.
        if let Some(tracks) = model_tracks {
            draw_model_tracks(tracks, center_x, center_y, px_per_km);
        }
        draw_nhc_overlays(bundle, overlays, site, center_x, center_y, px_per_km);
    }
```

  4. After `fn draw_nhc_overlays` (before `fn draw_contours`), add:

```rust
/// Draw ATCF model-guidance tracks ("spaghetti"). Each segment is coloured
/// by the forecast wind at its start (`adeck_style::segment_rgba`); OFCL gets
/// a white outline. Tech ids are drawn at track ends with greedy collision
/// skipping (same idiom as the city labels).
fn draw_model_tracks(tracks: &[ProjectedTrack], center_x: f32, center_y: f32, px_per_km: f32) {
    let sw = screen_width();
    let sh = screen_height();
    let margin = 50.0;
    let to_px = |km: Vec2| (center_x + km.x * px_per_km, center_y + km.y * px_per_km);
    let mut labels: Vec<(&str, f32, f32)> = Vec::new();

    for track in tracks {
        let (x0, y0) = to_px(track.min_km);
        let (x1, y1) = to_px(track.max_km);
        if x1 < -margin || x0 > sw + margin || y1 < -margin || y0 > sh + margin {
            continue; // whole track off-screen
        }
        let style = adeck_style::track_style(track.group);
        let pts: Vec<(f32, f32)> = track.km.iter().map(|&k| to_px(k)).collect();
        if let Some(under) = style.underlay_px {
            let outline = MacroquadColor::from_rgba(0xff, 0xff, 0xff, 230);
            for w in pts.windows(2) {
                draw_line(w[0].0, w[0].1, w[1].0, w[1].1, under, outline);
            }
        }
        for (i, w) in pts.windows(2).enumerate() {
            let start = track.vmax_kt.get(i).copied().flatten();
            let end = track.vmax_kt.get(i + 1).copied().flatten();
            let [r, g, b, a] = adeck_style::segment_rgba(track.group, start, end);
            draw_line(
                w[0].0,
                w[0].1,
                w[1].0,
                w[1].1,
                style.width_px,
                MacroquadColor::from_rgba(r, g, b, a),
            );
        }
        if style.label
            && let Some(&(lx, ly)) = pts.last()
        {
            labels.push((track.tech.as_str(), lx, ly));
        }
    }

    // OFCL is last in draw order; reversing gives it first pick of space.
    let label_font = 12.0;
    let pad = 3.0;
    let label_color = MacroquadColor::from_rgba(0xe8, 0xe0, 0xdc, 230);
    let mut placed: Vec<(f32, f32, f32, f32)> = Vec::new();
    for &(text, x, y) in labels.iter().rev() {
        if x < 0.0 || x > sw || y < 0.0 || y > sh {
            continue;
        }
        let dims = measure_text(text, None, label_font as u16, 1.0);
        let bx = x + 4.0 - pad;
        let by = y - 4.0 - dims.height - pad;
        let bw = dims.width + 2.0 * pad;
        let bh = dims.height + 2.0 * pad;
        let collides = placed
            .iter()
            .any(|&(px0, py0, px1, py1)| bx < px1 && bx + bw > px0 && by < py1 && by + bh > py0);
        if collides {
            continue;
        }
        draw_text(text, x + 4.0, y - 4.0, label_font, label_color);
        placed.push((bx, by, bx + bw, by + bh));
    }
}
```

- [ ] **Step 5: Cache and pass from `main.rs`.**
  1. In `app/src/state.rs`, after `model_guidance`, add:

```rust
    /// `model_guidance` projected to km for the current site, keyed by
    /// (site index, store revision). Rebuilt only when either changes.
    pub model_projection: Option<(usize, u64, Vec<crate::adeck_style::ProjectedTrack>)>,
```

     In the `main.rs` initializer, add `model_projection: None,`.
  2. In `main.rs`, just before `scope::draw_scope_to_texture(` (around l.1588), add:

```rust
            if state.settings.show_model_tracks && !state.model_guidance.is_empty() {
                let key = (state.site_index, state.model_guidance.revision());
                let fresh = state
                    .model_projection
                    .as_ref()
                    .is_some_and(|(s, r, _)| (*s, *r) == key);
                if !fresh {
                    let tracks = rustywx::adeck_style::project_tracks(
                        &state.model_guidance,
                        site.lat,
                        site.lon,
                    );
                    state.model_projection = Some((key.0, key.1, tracks));
                }
            }
            let model_tracks = if state.settings.show_model_tracks {
                state.model_projection.as_ref().map(|(_, _, t)| t.as_slice())
            } else {
                None
            };
```

  3. In the `draw_scope_to_texture(...)` call, insert `model_tracks,` as the argument right after the `nhc` argument (the `if state.show_nhc_data { ... } else { None },` expression).

- [ ] **Step 6: Verify.**

Run: `cargo test -p rustywx --lib && cargo clippy --all-targets --all-features -- -D warnings`
Expected: all pass; clippy is clean.

Smoke: `just run & sleep 3; kill %1 2>/dev/null || true`. It must launch without a panic. For a manual visual check with the setting on and an active storm, zoom out (scroll) until the storm is on screen. Colored model lines should end in tech-id labels, and a white-outlined OFCL line should sit on top of them, under the official cone.

- [ ] **Step 7: Commit.**

```bash
git add app/src/adeck_style.rs app/src/scope.rs app/src/state.rs app/src/main.rs
git commit -m "feat(tropical): draw wind-coloured model tracks on the map"
```

---

### Task 14: Map legend, with data age and the short disclaimer

**Files:**
- Create: `app/src/widgets/model_legend.rs`
- Modify: `app/src/widgets/mod.rs` (`pub mod model_legend;` in the `pub mod` list)
- Modify: `app/src/main.rs` (Ply layout, right after the NHC slide-in panel block ends around l.2723, before `// ── NHC product modal`)

**Interfaces:**
- Consumes: `WindCategory`, `format_cycle`, `format_age`, `is_stale` (Tasks 7–8), `GuidanceStore::oldest` (Task 6).
- Produces: `pub const WIDTH: f32`, `pub const HEIGHT: f32`, and `pub fn draw(ui: &mut Ui<'_, ChartWidget>, x: f32, y: f32, cycle_line: &str, age_line: &str, stale: bool)`.

Ply widgets can't be unit-tested headless (`docs/ply-issues/08-headless-testing-mode.md`). Everything the legend says comes from functions tested in Tasks 7–8. The gate is clippy plus the visual smoke check.

- [ ] **Step 1: Create the widget.** Write `app/src/widgets/model_legend.rs`:

```rust
//! Floating map legend for the spaghetti-models overlay: wind-category
//! swatches, line keys, data age, and the short "not an official forecast"
//! notice. Shown whenever model tracks are drawn.

use crate::adeck_style::WindCategory;
use crate::widgets::{ChartWidget, SYMBOL_FONT, glass_panel, nf};
use ply_engine::prelude::*;

pub const WIDTH: f32 = 196.0;
/// Fixed so the caller can anchor the legend above the bottom bar. Tune by
/// eye if rows are added.
pub const HEIGHT: f32 = 262.0;

const TEXT: u32 = 0xE8E0DC;
const MUTED: u32 = 0x9E9590;
const WARN: u32 = 0xFFC140;

pub fn draw(
    ui: &mut Ui<'_, ChartWidget>,
    x: f32,
    y: f32,
    cycle_line: &str,
    age_line: &str,
    stale: bool,
) {
    glass_panel::glass(
        ui.element()
            .id("model-legend")
            .width(fixed!(WIDTH))
            .height(fixed!(HEIGHT)),
    )
    .floating(|f| f.offset((x, y)).z_index(40).attach_root())
    .layout(|l| l.direction(TopToBottom).padding(8).gap(3))
    .children(|ui| {
        ui.element()
            .width(grow!())
            .height(fit!())
            .layout(|l| l.direction(LeftToRight).gap(6).align(Left, CenterY))
            .children(|ui| {
                ui.text(nf::HURRICANE, |t| {
                    t.font_size(12).font(&SYMBOL_FONT).color(0x0dc5b8)
                });
                ui.text("Model tracks", |t| t.font_size(12).color(TEXT));
            });
        for cat in WindCategory::LEGEND {
            swatch_row(ui, cat.hex(), 4.0, cat.label());
        }
        swatch_row(ui, 0xFFFFFF, 4.0, "OFCL  NHC official");
        swatch_row(ui, 0xB0B0B0, 2.0, "CARQ  past track");
        ui.text(cycle_line, |t| t.font_size(10).color(MUTED));
        ui.text(age_line, |t| {
            t.font_size(10).color(if stale { WARN } else { MUTED })
        });
        ui.text("Guidance only.", |t| t.font_size(10).color(WARN));
        ui.text("Not an official forecast.", |t| t.font_size(10).color(WARN));
    });
}

fn swatch_row(ui: &mut Ui<'_, ChartWidget>, color: u32, thickness: f32, label: &str) {
    ui.element()
        .width(grow!())
        .height(fixed!(13.0))
        .layout(|l| l.direction(LeftToRight).gap(6).align(Left, CenterY))
        .children(|ui| {
            ui.element()
                .width(fixed!(18.0))
                .height(fixed!(thickness))
                .background_color(color)
                .empty();
            ui.text(label, |t| t.font_size(10).color(TEXT));
        });
}
```

Add `pub mod model_legend;` to the `pub mod` list at the bottom of `app/src/widgets/mod.rs` (alphabetical: after `glass_panel`).

- [ ] **Step 2: Place it.** In `main.rs`, after the closing `}` of `if state.nhc_show_panel { ... }` and before `// ── NHC product modal (Stage 5)`, add:

```rust
                // ── Spaghetti-models legend (bottom-left, above the bar) ──
                if state.view_mode == ViewMode::Radar
                    && state.show_nhc_data
                    && state.settings.show_model_tracks
                    && let Some(oldest) = state.model_guidance.oldest()
                {
                    let now_unix = chrono::Utc::now().timestamp();
                    // 46 px bottom status bar + 8 px gap.
                    let y = screen_height() - 54.0 - rustywx::widgets::model_legend::HEIGHT;
                    rustywx::widgets::model_legend::draw(
                        ui,
                        8.0,
                        y,
                        &format!("{} cycle", rustywx::adeck_style::format_cycle(oldest.init)),
                        &format!(
                            "updated {}",
                            rustywx::adeck_style::format_age(oldest.fetched_at_unix, now_unix)
                        ),
                        rustywx::adeck_style::is_stale(oldest.fetched_at_unix, now_unix),
                    );
                }
```

- [ ] **Step 3: Verify.**

Run: `cargo clippy --all-targets --all-features -- -D warnings`. Expected: clean.

Smoke: `just run & sleep 3; kill %1 2>/dev/null || true`. It must not panic. For the manual check with the setting on and guidance loaded, the legend should sit bottom-left, above the status bar, with nothing clipped. If rows clip or there's a big empty gap, adjust `HEIGHT` and note the final value. The legend should disappear when you turn the setting off or press `N`.

- [ ] **Step 4: Commit.**

```bash
git add app/src/widgets/model_legend.rs app/src/widgets/mod.rs app/src/main.rs
git commit -m "feat(tropical): map legend for model tracks with data age"
```

---

### Task 15: NHC panel toggle, guidance card, disclaimer, and attribution

**Files:**
- Modify: `app/src/main.rs`:
  - the "Map Overlays" card in the NHC panel (around l.2553–2607)
  - `handle_input` near the NHC overlay presses (around l.3924–3945)
  - the external-link handlers (around l.4035)

**Interfaces:**
- Consumes: `Settings::show_model_tracks`, `GuidanceStore::get`, `AdeckFetcher::status`, `status_message`, `SlotStatus`, `format_cycle`, `format_age`, `is_stale`, `DISCLAIMER_LINES`, `MODEL_SUMMARY_URL`.
- Produces: the element ids `"btn-nhc-models"` and `"btn-nhc-model-summary"`, and the anchor where Task 16 inserts the chart (marked `// Intensity chart (Task 16)`).

- [ ] **Step 1: Add the toggle.** In the Map Overlays card, after the `nhc_toggle_button(ui, "btn-nhc-likely", ...)` call, add:

```rust
                                        nhc_toggle_button(
                                            ui,
                                            "btn-nhc-models",
                                            "Model Tracks (spaghetti)",
                                            state.settings.show_model_tracks,
                                        );
```

In `handle_input`, after the `btn-nhc-likely` handler, add:

```rust
    if ply.is_just_pressed("btn-nhc-models") {
        // Same persisted field as the Settings-panel row.
        state.settings.show_model_tracks = !state.settings.show_model_tracks;
        state.cache.save_settings(&state.settings);
    }
    if ply.is_just_pressed("btn-nhc-model-summary") {
        let _ = webbrowser::open(rustywx::adeck_style::MODEL_SUMMARY_URL);
    }
```

- [ ] **Step 2: Add the guidance card.** Directly after the Map Overlays card (after its `.children(|ui| { ... });`, before `// Graphics products (thumbnails)`), add:

```rust
                                // Model guidance (spaghetti) — only while on.
                                if state.settings.show_model_tracks {
                                    let now_unix = chrono::Utc::now().timestamp();
                                    let guidance = state.model_guidance.get(&meta.id);
                                    let status = state.adeck_fetch.status(&meta.id, now);
                                    ui.element()
                                        .width(grow!())
                                        .height(fit!())
                                        .background_color(0x171A1F)
                                        .corner_radius(4.0)
                                        .layout(|layout| {
                                            layout.direction(TopToBottom).padding(8).gap(4)
                                        })
                                        .children(|ui| {
                                            ui.text("Model Guidance", |t| {
                                                t.font_size(12).color(0xE8E0DC)
                                            });
                                            if let Some(g) = guidance {
                                                let stale = rustywx::adeck_style::is_stale(
                                                    g.fetched_at_unix,
                                                    now_unix,
                                                );
                                                ui.text(
                                                    &format!(
                                                        "{} cycle",
                                                        rustywx::adeck_style::format_cycle(g.init)
                                                    ),
                                                    |t| t.font_size(11).color(0x9E9590),
                                                );
                                                ui.text(
                                                    &format!(
                                                        "Updated {}{}",
                                                        rustywx::adeck_style::format_age(
                                                            g.fetched_at_unix,
                                                            now_unix
                                                        ),
                                                        if stale { " (stale)" } else { "" }
                                                    ),
                                                    |t| {
                                                        t.font_size(11).color(if stale {
                                                            0xFFC140
                                                        } else {
                                                            0x9E9590
                                                        })
                                                    },
                                                );
                                                let shown = g
                                                    .tracks
                                                    .iter()
                                                    .filter(|t| t.default_visible)
                                                    .count();
                                                ui.text(
                                                    &format!(
                                                        "{shown} tracks, {} intensity models",
                                                        g.intensity.len()
                                                    ),
                                                    |t| t.font_size(11).color(0x9E9590),
                                                );
                                                if !g.has_official {
                                                    ui.text(
                                                        "No NHC official forecast this cycle",
                                                        |t| t.font_size(11).color(0xFFC140),
                                                    );
                                                }
                                                if let rustywx::adeck_fetch::SlotStatus::Retrying {
                                                    in_secs,
                                                } = &status
                                                {
                                                    ui.text(
                                                        &format!(
                                                            "Refresh failed, retrying in {in_secs} s"
                                                        ),
                                                        |t| t.font_size(10).color(0xFFC140),
                                                    );
                                                }
                                                // Intensity chart (Task 16)
                                            } else {
                                                ui.text(
                                                    &rustywx::adeck_fetch::status_message(&status),
                                                    |t| t.font_size(11).color(0x9E9590),
                                                );
                                            }
                                            for line in rustywx::adeck_style::DISCLAIMER_LINES {
                                                ui.text(line, |t| t.font_size(10).color(0x9E9590));
                                            }
                                            ui.element()
                                                .id("btn-nhc-model-summary")
                                                .width(fit!())
                                                .height(fixed!(22.0))
                                                .background_color(0x1E1B1B)
                                                .corner_radius(4.0)
                                                .layout(|layout| {
                                                    layout
                                                        .padding((0, 8, 0, 8))
                                                        .gap(5)
                                                        .align(CenterX, CenterY)
                                                })
                                                .accessibility(|a| a.link("NHC model summary"))
                                                .children(|ui| {
                                                    ui.text("NHC model summary", |t| {
                                                        t.font_size(11).color(0x4a90d9)
                                                    });
                                                    ui.text(nf::EXTERNAL_LINK, |t| {
                                                        t.font_size(11)
                                                            .font(&SYMBOL_FONT)
                                                            .color(0x4a90d9)
                                                    });
                                                });
                                        });
                                }
```

If `meta.id` and the store key differ in case, it doesn't matter: `GuidanceStore::get` and `AdeckFetcher::status` both lowercase.

- [ ] **Step 3: Verify.**

Run: `cargo clippy --all-targets --all-features -- -D warnings`. Expected: clean.

Run: `grep -n '✓\|🌐' app/src/main.rs`. Expected: only the three pre-existing lines (about 1833, 2762, 3243). No new ones.

Smoke: `just run & sleep 3; kill %1 2>/dev/null || true`. It must not panic. For the manual check, open Tropical. "Model Tracks (spaghetti)" should be in Map Overlays. Turning it on shows the Model Guidance card ("Downloading model guidance…", then the cycle, age, and counts), all 6 disclaimer lines unwrapped and unclipped, and a link that opens NHC's model summary in a browser. The same switch in Settings stays in sync.

- [ ] **Step 4: Commit.**

```bash
git add app/src/main.rs
git commit -m "feat(tropical): model guidance card with disclaimer and attribution"
```

---

### Task 16: Per-storm intensity chart (MVP, using the rain-chart custom-draw pattern)

**Files:**
- Modify: `app/src/adeck_style.rs` (`CHART_MAX_KT`, `chart_max_tau`, `chart_bands`, tests)
- Modify: `app/src/widgets/mod.rs` (the `ChartWidget::IntensityChart` variant)
- Modify: `app/src/main.rs`:
  - a new `draw_intensity_chart` and `draw_custom_widget` next to `draw_rain_chart` (about l.241)
  - the `ui.show(draw_rain_chart)` call (about l.3185)
  - the Task 15 anchor `// Intensity chart (Task 16)`

**Interfaces:**
- Consumes: `IntensitySeries` (Task 5), `WindCategory` (Task 7).
- Produces: `pub const CHART_MAX_KT: u32 = 160`, `pub fn chart_max_tau(series: &[IntensitySeries]) -> i32`, `pub fn chart_bands() -> Vec<(u32, u32, [u8; 3])>`, and `ChartWidget::IntensityChart(Vec<crate::adeck::IntensitySeries>)`.

- [ ] **Step 1: Write the failing tests.** Add to `mod tests` in `app/src/adeck_style.rs`:

```rust
    use crate::adeck::IntensitySeries;

    fn series(max_tau: i32) -> IntensitySeries {
        IntensitySeries { tech: "AVNI".into(), official: false, points: vec![(0, 50), (max_tau, 60)] }
    }

    #[test]
    fn chart_max_tau_rounds_up_to_a_day_and_clamps() {
        assert_eq!(chart_max_tau(&[]), 24);
        assert_eq!(chart_max_tau(&[series(0)]), 24);
        assert_eq!(chart_max_tau(&[series(60)]), 72);
        assert_eq!(chart_max_tau(&[series(72)]), 72);
        assert_eq!(chart_max_tau(&[series(200)]), 168);
    }

    #[test]
    fn chart_bands_are_contiguous_and_capped() {
        let bands = chart_bands();
        assert_eq!(bands.len(), 7);
        assert_eq!(bands[0].0, 0);
        assert_eq!(bands.last().unwrap().1, CHART_MAX_KT);
        for pair in bands.windows(2) {
            assert_eq!(pair[0].1, pair[1].0, "no gaps between categories");
        }
        assert_eq!(bands[2], (64, 83, WindCategory::Cat1.rgb()));
    }
```

- [ ] **Step 2: Run the tests to verify they fail.**

Run: `cargo test -p rustywx --lib adeck_style::`
Expected: compile error `cannot find function 'chart_max_tau'`.

- [ ] **Step 3: Implement the helpers.** Change the top `use` in `adeck_style.rs` to `use crate::adeck::{GuidanceStore, IntensitySeries, ModelGroup};`, and add above `#[cfg(test)]`:

```rust
/// Top of the intensity chart's kt axis.
pub const CHART_MAX_KT: u32 = 160;

/// X-axis extent: the longest series' last tau, rounded up to a whole day,
/// clamped to 24..=168 h.
pub fn chart_max_tau(series: &[IntensitySeries]) -> i32 {
    let max = series
        .iter()
        .flat_map(|s| s.points.iter().map(|p| p.0))
        .max()
        .unwrap_or(0);
    (((max + 23) / 24) * 24).clamp(24, 168)
}

/// Category bands (lo_kt, hi_kt, rgb), TD through Cat 5, capped at CHART_MAX_KT.
pub fn chart_bands() -> Vec<(u32, u32, [u8; 3])> {
    let cats = [
        WindCategory::Depression,
        WindCategory::Storm,
        WindCategory::Cat1,
        WindCategory::Cat2,
        WindCategory::Cat3,
        WindCategory::Cat4,
        WindCategory::Cat5,
    ];
    cats.iter()
        .enumerate()
        .map(|(i, c)| {
            let lo = c.min_kt().unwrap_or(0);
            let hi = cats
                .get(i + 1)
                .and_then(|n| n.min_kt())
                .unwrap_or(CHART_MAX_KT)
                .min(CHART_MAX_KT);
            (lo, hi, c.rgb())
        })
        .collect()
}
```

Run: `cargo test -p rustywx --lib adeck_style::`. Expected: 14 passed.

- [ ] **Step 4: Add the widget variant.** In `app/src/widgets/mod.rs`, in `pub enum ChartWidget`, after `RainChart(Vec<Hour>),`, add:

```rust
    /// Spaghetti-models intensity graph: wind (kt) vs forecast hour per model.
    IntensityChart(Vec<crate::adeck::IntensitySeries>),
```

- [ ] **Step 5: Add the draw functions.** In `main.rs`, directly after the closing `}` of `fn draw_rain_chart`, add:

```rust
/// Ply custom-draw dispatcher: each chart fn returns early on other variants.
fn draw_custom_widget(cmd: &RenderCommand<ChartWidget>) {
    draw_rain_chart(cmd);
    draw_intensity_chart(cmd);
}

/// Spaghetti-models intensity chart: faint Saffir-Simpson bands, kt labels at
/// band floors, 24 h ticks, model lines thin blue, NHC official thick white.
fn draw_intensity_chart(cmd: &RenderCommand<ChartWidget>) {
    use rustywx::adeck_style::{CHART_MAX_KT, chart_bands, chart_max_tau};
    let RenderCommandConfig::Custom(c) = &cmd.config else {
        return;
    };
    let ChartWidget::IntensityChart(series) = &c.data else {
        return;
    };
    if series.is_empty() {
        return;
    }
    let bb = cmd.bounding_box;
    let gray = MacroquadColor::new(0.6, 0.58, 0.56, 1.0);
    draw_rectangle(
        bb.x,
        bb.y,
        bb.width,
        bb.height,
        MacroquadColor::new(0.118, 0.106, 0.106, 1.0),
    );
    let left = bb.x + 28.0;
    let right = bb.x + bb.width - 8.0;
    let top = bb.y + 8.0;
    let bottom = bb.y + bb.height - 16.0;
    let plot_w = (right - left).max(1.0);
    let plot_h = (bottom - top).max(1.0);
    let max_tau = chart_max_tau(series);
    let x_at = |tau: i32| left + (tau.clamp(0, max_tau) as f32 / max_tau as f32) * plot_w;
    let y_at = |kt: u32| bottom - (kt.min(CHART_MAX_KT) as f32 / CHART_MAX_KT as f32) * plot_h;

    for (lo, hi, [r, g, b]) in chart_bands() {
        draw_rectangle(
            left,
            y_at(hi),
            plot_w,
            y_at(lo) - y_at(hi),
            MacroquadColor::from_rgba(r, g, b, 38),
        );
        if lo > 0 {
            let label = lo.to_string();
            let d = measure_text(&label, None, 10, 1.0);
            draw_text(&label, left - 4.0 - d.width, y_at(lo) + d.height / 2.0, 10.0, gray);
        }
    }
    let mut tau = 0;
    while tau <= max_tau {
        let x = x_at(tau);
        draw_line(x, bottom, x, bottom + 3.0, 1.0, gray);
        let label = format!("{tau}h");
        let d = measure_text(&label, None, 10, 1.0);
        draw_text(&label, x - d.width / 2.0, bottom + 13.0, 10.0, gray);
        tau += 24;
    }

    let polyline = |points: &[(i32, u32)], width: f32, color: MacroquadColor| {
        for w in points.windows(2) {
            draw_line(x_at(w[0].0), y_at(w[0].1), x_at(w[1].0), y_at(w[1].1), width, color);
        }
    };
    let model_color = MacroquadColor::new(0.62, 0.78, 0.95, 0.75);
    for s in series.iter().filter(|s| !s.official) {
        polyline(&s.points, 1.2, model_color);
    }
    for s in series.iter().filter(|s| s.official) {
        polyline(&s.points, 2.5, WHITE);
    }
}
```

Replace `ui.show(draw_rain_chart).await;` with `ui.show(draw_custom_widget).await;`.

- [ ] **Step 6: Put the chart in the panel.** At the `// Intensity chart (Task 16)` anchor from Task 15, replace the comment with:

```rust
                                                if !g.intensity.is_empty() {
                                                    ui.element()
                                                        .width(fixed!(272.0))
                                                        .height(fixed!(130.0))
                                                        .custom_element(
                                                            ChartWidget::IntensityChart(
                                                                g.intensity.clone(),
                                                            ),
                                                        )
                                                        .empty();
                                                    ui.text(
                                                        "Wind (kt) by forecast hour. White = NHC official.",
                                                        |t| t.font_size(10).color(0x9E9590),
                                                    );
                                                }
```

- [ ] **Step 7: Verify.**

Run: `cargo test -p rustywx --lib && cargo clippy --all-targets --all-features -- -D warnings`
Expected: all pass; clippy is clean.

Smoke: `just run & sleep 3; kill %1 2>/dev/null || true`. It must not panic. For the manual check, the Forecast view's rain chart should still draw (the dispatcher regression check). With guidance loaded, the Model Guidance card should show the chart with category bands and a white OFCL line. Scroll the NHC panel. If the chart draws outside the panel's clip area while scrolled, write it down as a known issue in the task report and don't hack a fix (Ply scissor behavior for custom elements is unverified).

- [ ] **Step 8: Commit.**

```bash
git add app/src/adeck_style.rs app/src/widgets/mod.rs app/src/main.rs
git commit -m "feat(tropical): per-storm model intensity chart"
```

---

### Task 17: Docs, spell-check, and final acceptance

**Files:**
- Modify: `typos.toml`
- Modify: `USER_GUIDE.md` (`## Overlays` around l.73, and `### Overlay Defaults` around l.94)

- [ ] **Step 1: Spell-check.** Add `"app/src/testdata/",` to `[files] extend-exclude` in `typos.toml`. Then run:

Run: `typos --config typos.toml`
Expected: no findings. If it flags an ATCF tech id (for example in `adeck.rs`), add it under `[default.extend-words]` as `XXXX = "XXXX"` with a `# ATCF model ids` comment, and re-run until it's clean.

- [ ] **Step 2: User guide.** In `USER_GUIDE.md`, under `## Overlays`, after the `Tropical (NHC)` bullet, add:

```markdown
- **Spaghetti models (optional, off by default):** Computer-model forecast tracks for each active storm, from the National Hurricane Center's public model-guidance files (ATCF). Each line is colored by the model's forecast wind speed. The thick white-outlined line is NHC's official forecast, and the thin grey line is the storm's past track. A legend in the lower left shows the color key and how old the data is. Turn it on from the Tropical panel ("Model Tracks (spaghetti)") or in Settings ("Spaghetti models (tropical)"). The panel also shows a wind-speed chart for each model. **These are guidance, not official forecasts.** Always follow NHC and your local National Weather Service office. The overlay downloads one file per active storm every 15 minutes while it's on, and nothing while it's off. In `--demo` mode it shows today's storms, not the historical event.
```

Under `### Overlay Defaults`, add:

```markdown
- **Spaghetti models (tropical):** Show model-guidance tracks (default off).
```

- [ ] **Step 3: Run full acceptance** (see the next section). Every command must pass.

- [ ] **Step 4: Commit.**

```bash
git add typos.toml USER_GUIDE.md
git commit -m "docs: document the spaghetti-models overlay"
```

---

## Acceptance / smoke test

Run these from the repo root after Task 17. Each must succeed before the feature is reported done.

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features            # ~200 existing + ~60 new tests, 0 failed
typos --config typos.toml
# AGENTS.md mandatory smoke test:
just run &
sleep 3
kill %1 2>/dev/null || true
```

Manual checklist (one longer `just run` session; skip the visual items if no storm is active and say so):

1. **Fresh settings, overlay OFF:** no tracks, no legend, no Model Guidance card, and stderr has no `adeck` lines (no network cost).
2. **Turn it ON** (Tropical panel, "Model Tracks (spaghetti)"): within about 15 s, stderr shows `phase=adeck_response` and `phase=adeck_parse_applied` per storm. Tracks are colored by wind and labelled at their ends. OFCL is white-outlined on top. CARQ is grey. The official cone/track still draws above the models.
3. **Legend** sits bottom-left above the status bar and shows the cycle, "updated N min ago", and "Guidance only. / Not an official forecast."
4. **Panel card** shows the cycle, age, counts, intensity chart, all 6 disclaimer lines, and a working "NHC model summary" link.
5. **Settings modal** "Spaghetti models (tropical)" mirrors the panel toggle, and the setting persists across a restart.
6. **`N` key** (tropical master gate) hides tracks and legend together with the other NHC layers.
7. **Offline** (disable Wi-Fi, or block `ftp.nhc.noaa.gov`, after data loaded): one "Network error" toast total, the card shows "Refresh failed, retrying in N s", the tracks stay, and "(stale)" appears after 30 min if you wait that long. Re-enabling the network recovers without a restart.
8. **Turn it OFF:** tracks and legend vanish immediately, and no further `adeck` lines are logged.
9. **No new `✓`/`🌐`:** `grep -n '✓\|🌐' app/src/main.rs` shows only the three pre-existing lines.

---

## MVP vs follow-ups

**MVP (Tasks 1–17):**
- a-deck fetch, gunzip, and parse with real-data tests
- storm-to-file mapping
- latest-cycle selection with the OFCL rule
- model grouping and the research default set
- wind-colored tracks with an OFCL outline, a CARQ past track, and end labels
- km projection cache
- legend with data age
- settings toggle (default OFF) in both Settings and the NHC panel
- disclaimer and attribution
- resilient fetch: stable id plus `cancel()`, 60 s deadline, backoff, one toast per streak, 404 quiet, behind the `NetPort` transport boundary
- in-memory last-good data when offline
- intensity chart
- docs

The intensity chart is in the MVP because the rain-chart custom-draw hook made it about one task.

**Follow-ups** (not planned task-by-task; each needs its own plan):

1. **Persistent guidance cache.** Save `StormGuidance` (add `Serialize`/`Deserialize`) under a Ply storage key like `adeck_v1_{storm_id}`, using the `borders.rs` `load_cached`/`save_cached` oneshot pattern. That gives instant tracks on relaunch and offline-at-launch. Prune keys whose storm isn't in `CurrentStorms.json`. Keep the cache small: it's KB, not the scan-cache problem in review C1.
2. **"More models" toggle.** Show the `default_visible == false` tracks (TABS/M/D, XTRP, CLP5, TCLP, OFCI, late runs) behind one panel toggle. The GEFS members AP01–AP30 should get their own toggle, because that's 30 more lines. `project_tracks` would take a filter.
3. **Selected-storm-only option.** With several active storms the map gets busy. Add an option to draw only `nhc_selected_storm`.
4. **Track hover/tooltip.** Show the model description (from `nhc_techlist.dat`) and per-tau wind and position. This needs hit-testing against `ProjectedTrack.km`.
5. **Demo-mode guidance.** Embed a trimmed Harvey 2017 a-deck (`aal092017`) plus a synthetic `StormMeta` for `--demo harvey2017`, so the overlay can be checked against a fixed scene. Today demo shows live storms only, the same as the rest of NHC.
6. **b-deck best track** (`https://ftp.nhc.noaa.gov/atcf/btk/b{id}.dat`, plain text) for the full observed history instead of CARQ's 24 h.
7. **Move the a-deck transport to the app-owned HTTP module.** When the remediation plan's `app/src/http.rs` (reqwest 0.13, real timeouts, system proxy, platform cert verifier) lands, implement `adeck_fetch::NetPort` for it and swap it in for `PlyNet` at the one `main.rs` construction site. `adeck.rs`, `adeck_style.rs`, `scope.rs`, and the scheduler don't change. Keep the app-side deadline until per-request timeouts are confirmed. A-decks are served as `application/x-gzip` files, not with `Content-Encoding: gzip`, so reqwest's `gzip` feature won't inflate them, and `gunzip_limited` stays.
8. **Conditional GET (ETag/If-None-Match).** This is blocked on Ply, because `Response` exposes no headers. It comes free with follow-up 7, since reqwest exposes headers. Fetch only when the file changed.
9. **Pre-existing icon-rule violations (out of scope here, separate `fix:` commit):** `"✓"` in `nhc_toggle_button` (`main.rs:3243`), `"Tropical ✓"` (`main.rs:1833`), and `"🌐 "` in the NHC modal title (`main.rs:2762`). Replace them with `nf::` glyphs per AGENTS.md. The new "Model Tracks (spaghetti)" toggle inherits the `"✓"` from `nhc_toggle_button` until this is fixed.
10. **Projection cache for existing overlays.** The borders, alerts, and NHC cone still re-project every frame (review H5). `ProjectedTrack` is a template for fixing them.

---

# Appendix A: Codebase context

Line numbers are as of commit `081df73`. These notes were written in Phase 1, before the plan above.

Line numbers are as of commit `081df73` (main). Treat them as hints and
grep before editing.

### 1. Crate layout and build/test entry points

- Workspace root `Cargo.toml` is a **virtual manifest** (`members = ["app"]`).
  The only package is `app/` (`rustywx`, edition 2024). The binary is
  `app/src/main.rs`. The library is `app/src/lib.rs`, which declares every
  module `pub mod …`. A new `nhc_models.rs` (or similar) has to be added
  there.
- `Justfile` targets: `fmt` (`cargo fmt --all -- --check`), `lint`
  (`cargo clippy --all-targets --all-features -- -D warnings`), `check`,
  `test` (`cargo test --all-features`), `run` (`cd app && cargo run --release`),
  `ci` (fmt, lint, check, test, audit, deny, gitleaks, typos). CI
  (`.github/workflows`) runs fmt, clippy `-D warnings`, check, test, and test --doc.
- **AGENTS.md smoke test is mandatory** before claiming done:
  `just run & sleep 3; kill %1`. A release build is required, and it must not
  panic at startup.
- `typos` runs in CI (`typos.toml`). ATCF tech names like `OFCL`, `AVNO`, `HWRF`,
  `CARQ`, `XTRP`, `TVCN` and similar may trip it, so add them to `typos.toml`
  `extend-words` if needed.
- Commits use Conventional Commits (`feat:`, `fix:`, `test:`, `docs:`…).

### 2. How tests are structured

- **Unit tests live inline** in each module as `#[cfg(test)] mod tests`. Good
  templates are `nhc.rs` (~1344–1713: parsers tested on literal JSON/KML
  strings), `settings.rs` (default/serde round-trip/back-compat), and
  `background_work.rs` (spins a real `tokio::runtime::Runtime`, submits to the
  worker, polls with a 2 s deadline).
- **Fixtures** are embedded with `include_bytes!`/`include_str!` relative to the
  source file. For example, `app/src/data.rs:365` does
  `include_bytes!("testdata/digital_radar_data_full.bin")`, so fixtures go in
  `app/src/testdata/`. A real a-deck sample should go there too (for example
  `app/src/testdata/aal092020_sample.dat`, a trimmed set of real lines). Keep
  it small, because it is compiled into the test binary.
- **Root-level `tests/` (`tests/network.rs`, `tests/borders_network.rs`) is NOT
  compiled.** The virtual workspace has no package at the root, and
  `cargo test --test network` reports "no test target named `network`". Don't put
  new integration tests there. If a live-network test is wanted, put it in
  `app/tests/` with `#[ignore = "requires network"]`. Better still, skip it and
  rely on fixture-driven unit tests.
- Pure functions (parse, filter, group, color mapping, projection math) are
  the TDD surface. Anything that calls macroquad draw functions or
  `screen_width()` can't be unit-tested headless (see
  `docs/ply-issues/08-headless-testing-mode.md`). Keep the drawing code thin and
  push the logic into pure helpers. `app/examples/gpu_palette_check.rs` is the
  one GPU harness example. It's not needed here.
- Current baseline is about 200 lib tests. Clippy is `-D warnings`, so new
  `pub` items that main.rs doesn't use yet will trip `dead_code` lints in the
  bin unless they're used. Lib `pub` items are fine.

### 3. Tropical section end to end (`app/src/nhc.rs`, 1713 lines)

**Data types** (all `Clone + Serialize + Deserialize`):
- `StormMeta` (l.43) comes from `CurrentStorms.json`. Its key fields for
  spaghetti mapping are `id` (lower-case ATCF id like `"al022026"`, which is
  basin + 2-digit number + 4-digit year), `bin_number` (`"AT2"`), `name`,
  `classification`, `intensity_kt`, `lat`, `lon`, `last_update`, and
  `advisory_num`. **`id` maps directly to the ATCF a-deck filename stem**
  (`a{id}.dat`, for example `aal022026.dat`). Note the basin prefixes: storm id
  `al` → NHC graphics bin `AT` (see `construct_image_products`, l.399–405). A-deck
  files use `al`/`ep`/`cp`.
- `StormGis` (l.100) holds the official cone/track/points/watches-warnings from
  the ArcGIS MapServer. It is matched to storms **by name substring**, not id
  (l.785, l.801).
- `NhcBundle` (l.118) is the one bundle the app holds in `AppState.nhc_bundle`.
  It holds metas, gis_storms, text/image products, and wind-prob and arrival
  contours. It is `Default`.
- `Ring = Vec<(f64, f64)>` from `borders.rs`, with points as **(lat, lon)**. GeoJSON
  and KML coords are `[lon, lat]` and get swapped at parse time.

**Fetch state machine** (`NhcFetchState`, l.865–1298), polled every frame:
1. `start()` (l.931) fires `CurrentStorms.json` (net id `nhc-current-storms`,
   `User-Agent: rustywx/0.3`) and GIS layers 5–8 (`nhc-gis-{layer}`).
   `Idle → Phase1`.
2. `poll()` → `poll_phase1()` (l.980). It waits for `CurrentStorms.json`, then
   returns `NhcPoll::Metadata(NhcMetadataBodies)` (raw strings) and moves to
   `AwaitingMetadata`. JSON parsing is **not** done on the UI thread.
3. main.rs submits the bodies to the background parse worker
   (`ParsePayload::NhcMetadata`). When the result returns, main calls
   `accept_metadata(metas, gis…)` (l.1021). That fires per-storm text, image,
   and KMZ requests with deterministic net ids, re-fires any GIS layer that
   hasn't arrived (Ply may have evicted it), and moves to `Phase2`.
4. `poll_phase2()` (l.1168) drains each pending request. Errors that contain
   `"404"` are silently tolerated. Other errors are `eprintln!`'d and dropped,
   so a single product failing never fails the bundle. When none remain it
   assembles `gis_storms` and moves to `Complete(bundle)`.
5. `poll()` returns `NhcPoll::Complete(bundle)`, which is terminal. main.rs
   (l.1279) queues image bytes for background decode, clamps
   `nhc_selected_storm`, and stores `state.nhc_bundle = Some(bundle)`. Errors
   produce `eprintln!` plus `show_toast(.., ErrorKind::Network)`.
- **Note:** KMZ decompression (`extract_kml_from_kmz`) and KML/text parsing in
  `poll_phase2` currently run **on the UI thread**. That's tolerated because
  the payloads are small. A-deck files are bigger (hundreds of KB to several MB
  late in a long storm), so they **must** be parsed on the background worker.
  Don't copy the phase-2 inline-parse pattern.

**Refresh cadence:** `nhc::POLL_INTERVAL = 300 s` (l.20) is now just a
default. The live value is `settings.nhc_refresh_secs` (default 300), checked
every frame at main.rs:1228–1236:
`!nhc_fetch_fired && (bundle.is_none() || now - nhc_last_poll > nhc_refresh_secs)`.
`nhc_fetch_fired` is cleared on a terminal result or error (l.1242). A-decks
update roughly every 6 h per cycle, with early-cycle aids arriving within an
hour or so. Re-fetching them every 5 min is wasteful, so they need their own
slower cadence. Conditional requests aren't possible because `HttpConfig` only
sets headers, though `If-Modified-Since` could be sent as a header. Ply's
`Response` exposes only `status()`, `text()`, `bytes()`, and no response
headers.

**Ply `net` facts** (ply-engine 1.1.1, `src/net/mod.rs`):
- `net::get(id, url, |c| c.header(..))` / `net::request(id)?.response()`
  → `Option<Result<Arc<Response>, String>>`. `None` means still pending.
- Native backend is **ureq 3, `default-features = false, features = ["rustls"]`**.
  - **No transparent gzip.** A `.dat.gz` body arrives as raw gzip bytes.
    `flate2` is already in `Cargo.lock` transitively (via `zip`), but using it
    needs a direct `[dependencies]` entry. Prefer an uncompressed `.dat`
    endpoint if the research finds one.
  - **10 MB body cap** (`ureq` `MAX_BODY_SIZE`, `read_to_vec`). A body over that
    becomes an `Err(String)`. A-decks are normally well under it, but the
    parser and error path should treat it as a soft failure.
  - Non-2xx status comes back as `Err(String)` containing the status, which is
    why `nhc.rs` checks `e.contains("404")`.
  - No timeout config is exposed (ureq defaults).
- **Completed responses are evicted after 60 un-polled frames**
  (`max_frames_not_used = 60`; see `docs/ply-issues/07-…`). Pending requests are
  never evicted. Poll every frame, or re-fire before polling if a phase may
  have stalled (the pattern at `accept_metadata` l.1046–1049).
- Same-id re-fire behavior is undocumented (`docs/ply-issues/06-…`). Use stable
  per-storm ids (`nhc-adeck-{storm_id}`) and never re-fire while one is pending.

### 4. Background (off-UI-thread) work — `app/src/background_work.rs`

- `BackgroundWorkers::new(&tokio Handle)` starts three tokio tasks (parse,
  raster, image decode). Each uses `tokio::sync::mpsc::channel(QUEUE_CAPACITY = 4)`
  and does CPU work in `spawn_blocking`. Requests and results are owned and `Send`.
- **Parse worker:** `ParsePayload` enum (`Borders`, `Alerts(String)`,
  `NhcMetadata{..}`) → `fn parse(payload) -> Result<ParsedData, String>`
  (l.330) → `ParsedData` enum. To add a-deck parsing, add a variant to
  each, such as `ParsePayload::ADeck { storm_id, body: Vec<u8> or String }` →
  `ParsedData::ADeck { storm_id, guidance }`, plus `phase()` and
  `payload_size()` arms. The logging (`log_background`, warn over 250 ms) is
  automatic.
- `submit_parse` is `try_send`, so `Err(SubmitError::Full)` is normal back-pressure.
  Existing callers handle it by clearing the fired flag and retrying next
  frame (borders, alerts) or by rejecting (NHC metadata, main.rs:1274).
- **Generations:** main.rs keeps `next_generation: u64` and per-consumer
  `Option<u64>` slots (`borders_parse_generation`, `alerts_parse_generation`,
  `nhc_parse_generation`, around main.rs:790). A result is applied only if its
  generation matches the slot; anything else is logged as `parse_drop stale=true`.
  The `Err` arm (main.rs:1101–1127) must clear and handle the new slot as well.
  A-decks need one more slot. With multiple storms, use a
  `HashMap<String /*storm_id*/, u64>` of in-flight generations, or one
  combined payload for all storms.
- Test pattern for a new payload: copy
  `parse_worker_is_polled_without_blocking_the_render_caller` (l.~520).

### 5. How overlays are drawn on the map

- Per frame (main.rs:1572–1613), in `ViewMode::Radar`,
  `scope::draw_scope_to_texture(...)` draws **directly to screen** with
  macroquad (despite the name; see the Stage 1 note about render_to_texture
  flips). It takes `nhc: Option<(&NhcBundle, &NhcOverlayState)>`, which is
  `Some` only when `state.show_nhc_data` is on (the master tropical-layer gate,
  key `N`, button `btn-tropical-data`).
- Draw order inside `draw_scope_to_texture` (scope.rs:1395–1608) is radar
  texture, then rings, site marker, cities, site markers, borders, melting
  layer, alerts, **NHC overlays** (l.1600), and finally the user location.
  Spaghetti tracks should draw inside the NHC block, **before** the official
  track/cone so the official forecast sits on top. Better: call a new
  `draw_model_tracks(..)` first inside `draw_nhc_overlays`.
- Projection: `px_per_km = (min(sw,sh)/2) / MAX_RANGE_KM(230) * zoom`,
  `center = screen_center + pan_km * px_per_km`. Each point maps through
  `geo::point_to_km_offset(site.lat, site.lon, (lat, lon)) -> Vec2` (geo.rs:905),
  an azimuthal-equidistant projection around the **current radar site**. Then
  `x = center_x + km.x*px_per_km`, `y = center_y + km.y*px_per_km`. `scope::project_site`
  (l.1371) wraps the same math. Tracks thousands of km from the site still
  project fine. Distortion grows with range, but it matches how the official
  cone and track are already drawn.
- Line idiom (repeated at scope.rs:1891–1912, `draw_borders`, `draw_contours`):
  project both endpoints, cull if both are beyond the same screen edge plus a
  50 px margin, then `draw_line(ax, ay, bx, by, thickness, MacroquadColor)`.
  Colors come from `[u8;4]` helpers in `nhc.rs` (`wind_prob_color`,
  `watch_warning_color`), which are unit-tested pure functions. A
  `model_wind_color(kt) -> [u8;4]` (Saffir-Simpson bands) belongs there and is
  test-first.
- Per-segment coloring (cyclocane style): color segment i→i+1 by the wind at
  its start point, or by the max of its two endpoints. Doing it per segment
  falls out naturally from the `windows(2)` loop.
- There is no per-frame caching of projected points. Every overlay re-projects
  every frame (cheap trig). About 40 aids × 20 points × N storms is fine.
- **Legend:** nothing draws a map legend today. Two options: (a) draw in
  screen space after `draw_scope_to_texture` with macroquad
  `draw_rectangle`/`draw_text` (like `draw_radar_sweep`), or (b) a Ply floating
  element. Ply elements draw in `ui.show(...)` *after* the scope, so (b)
  layers correctly over the map and respects panel z-order. Mind the left/right
  panels (NHC panel is 320 px on the right, top bar is 36 px).
- **Charts:** there's an existing custom-draw hook for an intensity graph.
  `widgets::ChartWidget` (widgets/mod.rs) is the Ply custom-element payload enum
  (`None`, `RainChart(Vec<Hour>)`). A Ply element declares
  `.custom_element(ChartWidget::RainChart(..))` (main.rs:2231), and
  `ui.show(draw_rain_chart)` (main.rs:3185) dispatches to `fn draw_rain_chart`
  (main.rs:241), which draws with macroquad inside `cmd.bounding_box` (grid,
  labels via `draw_text`/`measure_text`, lines, dots). An intensity chart
  would add `ChartWidget::IntensityChart(..)` and extend the `ui.show` handler
  to match both variants. The handler is a plain `fn`, so either rename it to
  `draw_custom_widget` with a match, or keep one fn and add an arm. The payload
  is cloned every frame, so keep it small (`Vec<(model, Vec<(tau_h, vmax_kt)>)>`).

### 6. NHC panel and in-panel toggles (main.rs)

- Panel is at main.rs:2365–2723. It's a `glass_panel` floating element with id
  `"nhc-panel"`, 320 px wide on the right (full-screen on mobile), with a
  spring slide-in and `scroll_y` overflow. Sections, in order: header
  (`nf::HURRICANE` + "NHC Tropical Cyclones"), storm dropdown (`STORM_DROPDOWN`,
  `state.nhc_storm_dropdown`, selection index into `bundle.metas`), stats
  card, graphics link, **"Map Overlays" card of `nhc_toggle_button`s**
  (l.2553–2607), graphics thumbnails, text products.
- `nhc_toggle_button(ui, id, label, active)` (main.rs:3236). Presses are handled
  in `handle_input` (main.rs:3924–3945) via `ply.is_just_pressed("btn-nhc-…")`,
  which flips `state.nhc_overlays.*`. `NhcOverlayState` (scope.rs:1823) is
  **not persisted**. It resets to `Default` every launch.
- The spaghetti toggle (`"btn-nhc-models"`, "Model Tracks (spaghetti)") goes
  in the Map Overlays card. The disclaimer/attribution text and an optional
  intensity chart can sit in a new card below it, visible only when the toggle
  is on.
- `handle_input` gates scope pan/zoom on `!over_nhc_panel` (main.rs:3305). Clicks
  in the panel don't pan the map.

### 7. Settings and toggles

- `settings::Settings` (settings.rs) is serde-persisted as `"settings.json"` via
  `Cache::save_settings`/`load_settings`. **Every new field needs a
  `#[serde(default)]`** (or `default = "fn"`) so old config files still
  deserialize. A test pins this: `deserializes_settings_without_location_fields`
  feeds an old JSON blob. Also update `Default`,
  `default_matches_existing_startup_behaviour`, and the `serde_round_trip`
  literal, which lists every field and stops compiling when one is added.
- Precedent for an opt-in, default-OFF feature: `melting_layer_hint_enabled`
  (`#[serde(default)]` = false, doc comment saying "Default OFF").
- Settings are applied once at boot (main.rs:902–944, `settings_applied`), copying
  `settings.show_*` into `state.show_*`. Top-bar toggles write back and save
  immediately (for example main.rs:3479–3483 for `btn-tropical-data`).
- Settings modal (`widgets/settings.rs`): `pub const …_TOGGLE_ID` ids, `row(ui,
  label, |ui| bool_toggle(ui, ID, label, value))`, fixed 420×542 modal with a
  scrolling content area. main.rs handles presses (for example
  main.rs:3644–3650: flip, `save_settings`). The draw signature takes
  `&Settings`, so no signature change is needed for a new bool.
- For "settings toggle defaulting OFF", persist it as
  `Settings::show_model_tracks: bool` (`#[serde(default)]`). Two ways to
  surface it: (a) a settings-modal row "Spaghetti models (tropical)", plus
  (b) the NHC-panel Map Overlays toggle mirroring the same field, saving on
  each flip. The draw path should read one source of truth: either mirror into
  `NhcOverlayState.show_model_tracks` at boot (as `show_nhc_data` does), or
  read `state.settings` directly.
- **Gating the fetch on the toggle:** when the toggle is OFF, a-decks should not
  be fetched at all (no network cost for an opt-in feature). Fetch on the
  OFF→ON transition, and on refresh while ON.

### 8. Caching — `app/src/cache.rs`, `borders.rs`

- `Cache` wraps Ply `Storage::new("rustywx")`, a key-value byte store
  (`save_bytes`/`load_bytes`/`remove`, all async).
- Patterns: **save** is fire-and-forget `tokio::spawn`. **Load** is
  `tokio::spawn` plus a `oneshot::Receiver` polled per frame with `try_recv()`.
  Corrupt entries are logged, `remove`d, and treated as a miss (never panic).
  Keys can carry a format version (`scan_rle_v{CACHE_FORMAT_VERSION}_{site}`).
- `borders.rs:112–140` shows the simplest JSON-blob cache (`load_cached` async,
  `save_cached` fire-and-forget). `state.cache.storage()` hands out a
  `Storage` clone.
- **NHC data is not cached today.** Each launch re-fetches. An a-deck cache
  keyed `adeck_v1_{storm_id}`, holding parsed guidance plus `fetched_at`, lets
  the overlay appear instantly after relaunch and work offline. Storage is
  async, so the load must use the oneshot pattern and must not block startup.
  Bound or prune it: keep only storms in the current `metas`, or skip pruning
  and rely on staleness checks.

### 9. Offline and demo behavior — `app/src/demo.rs`

- `--demo <event|path>` loads a curated historical NEXRAD volume
  (`EVENTS`: `moore2013`, `harvey2017` on KCRP 2017-08-26). Demo pins the
  site and drops live worker messages. **NHC fetching is not gated by demo.**
  `nhc_fetch.start()` still fires live `CurrentStorms.json`, so demo shows
  today's storms (often none) over a historical radar scene.
- For spaghetti in demo mode, the MVP choice is to do what NHC already does
  (live data, or nothing if no storms are active) and say so. Harvey-era
  aids from an embedded fixture for `harvey2017` would be a follow-up. That
  needs a matching `StormMeta`, since `CurrentStorms.json` won't list al092017.
- Offline: net errors surface as `Err(String)` from `response()`. NHC shows a
  `Network` toast and keeps the previous bundle. Spaghetti should keep the last
  good (or cached) guidance, show a muted "model data unavailable" line in
  the panel, and **not** toast on every retry (one toast per failure streak,
  like `location_error_shown`).

### 10. Icons and text rules (AGENTS.md)

- Icons come only from `SYMBOL_FONT` (Nerd Fonts Symbols Mono), with
  codepoints named in `widgets/mod.rs` `pub mod nf`. Render with
  `ui.text(nf::X, |t| t.font(&SYMBOL_FONT)…)`. No emoji, because there's no
  color-emoji font and it renders as tofu.
- Existing code has a few violations. **Don't copy them:** `nhc_toggle_button`
  uses `"✓"` (main.rs:3243), the top bar uses `"Tropical ✓"` (main.rs:1833),
  and the NHC modal title prefixes `"🌐 "` (main.rs:2762). New UI should use
  `nf::` glyphs. Any new glyph (for example a chart-line or info icon, such as
  `md-chart-line` or `md-information-outline`, with codepoints unverified, so
  look them up in the Nerd Fonts cheat sheet) must be added
  to `nf` with a name comment and checked against the font. Fall back to
  iconoir only if no Nerd glyph fits.
- Macroquad `draw_text` on the map uses the default macroquad font, not Inter
  or Nerd. Map legend labels must be plain ASCII (as in alert labels and
  "50 km"). Use "kt", not glyphs.
- Body font can be OpenDyslexic (`active_dyslexic`). Ply text picks that up
  automatically. Bold uses `if active_dyslexic { &DYSLEXIC_BOLD } else { &INTER_BOLD }`.

### 11. Error handling and UI-thread rules (summary for the plan)

- Never block the frame. No `.await` on network or storage inside the loop
  (except Ply's own `ui.show(..).await`/`next_frame().await`). Network goes through
  `net::get` plus per-frame poll. Parsing goes through
  `BackgroundWorkers::submit_parse`. Storage goes through tokio spawn plus oneshot.
- Errors are `eprintln!("Warning: …")` plus an optional
  `show_toast(&mut state, now, ErrorKind::Network)`. The parse-worker `Err` path
  returns `String` (`format!("{e:#}")` from anyhow).
- Parsers return `anyhow::Result` and skip malformed lines rather than failing
  the file (`filter_map`/`continue`, as in the KML and GeoJSON helpers).

### 12. Integration-point checklist (where Phase 2 tasks will touch)

| Concern | File / symbol |
|---|---|
| Parser, types, color fn, URL builder | new `app/src/nhc_models.rs` (+ `lib.rs` `pub mod`) or a section in `nhc.rs` |
| Fixture | `app/src/testdata/<real a-deck sample>.dat` |
| Off-thread parse | `background_work.rs` `ParsePayload`/`ParsedData`/`parse()` |
| Fetch + cadence + generation slot | `main.rs` near l.1228–1311 (beside NHC block), new `AppState` fields in `state.rs` |
| Setting (default OFF) | `settings.rs` field + 3 tests; `widgets/settings.rs` row + ID; `main.rs` press handler |
| Panel toggle, disclaimer, chart | `main.rs` NHC panel Map Overlays card (l.2553), `handle_input` (l.3924) |
| Map drawing | `scope.rs` `NhcOverlayState` + `draw_nhc_overlays` (l.1848) + new `draw_model_tracks` |
| Legend | Ply floating element in main.rs layout or macroquad after `draw_scope_to_texture` |
| Intensity graph | `widgets/mod.rs` `ChartWidget` variant + `draw_rain_chart` dispatcher (main.rs:241, 3185) |
| Cache | `cache.rs` (new key + load/save) or a `borders.rs`-style pair in the new module |
| Icons | `widgets/mod.rs` `nf` |
| Spell-check | `typos.toml` |

---

