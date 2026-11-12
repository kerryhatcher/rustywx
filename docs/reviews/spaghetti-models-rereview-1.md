# Spaghetti-models overlay, re-review 1

- **Date:** 2026-10-19
- **Scope:** the 11 fix commits `eb0fee5..5ff944f` on `feat/spaghetti-models`, for findings S1–S11, L1 and L4 from `docs/reviews/spaghetti-models-review.md`.
- **Code reviewed:** everything was read at commit `5ff944f` with `git show 5ff944f:<path>`. The worktree's uncommitted `model_legend.rs` edits were excluded. Legend code is as of `5ff944f`, and the later delta (groups 7–8: L5–L8, L19) is pending.

## Method

- Four read-only reviewers (2 Opus, 2 Sonnet), each owning one finding group:
  - freshness: S1, S3, S5, S8, S9
  - official forecast vs. models: S4, L1
  - scheduler: S2, S6, S7, L4
  - legend and line styles: S10, S11, plus test, clippy and fmt
- For each finding, they checked three things: is the fix correct, does a regression test fail if the fix is reverted, and does the fix introduce new problems.
- The lead reviewer re-read the source for every verdict and new finding marked ✔.
- One reviewer also fetched NHC's live `CurrentStorms.json` and MapServer layers, read-only, to check how GIS storms are identified.

## Build and test results

These ran on an isolated `git archive` copy of `5ff944f` with its own `CARGO_TARGET_DIR`:

- `cargo test -p rustywx`: **pass**. Lib 381 passed, 0 failed; bin 2 passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: **pass**.
- `cargo fmt --all -- --check`: **pass**.
- `just run` smoke test: **not run**. It would have used the shared worktree.

## Verdict summary

| ID | Verdict | Notes |
|---|---|---|
| S1 | **Fixed** ✔ | Freshness comes from cycle age (`init` parsed as UTC, cannot panic); amber after 12 h, dimmed after 24 h; fetch time is now only the secondary "Checked" line |
| S2 | **Fixed** ✔ | Gate is now `if parse_queue_has_capacity() && !has_parsing()`. **No test covers the `main.rs` gate** (new N4) |
| S3 | **Fixed** ✔ | Older cycles are rejected unless the stored cycle is implausible; tracks grey out to a 35% alpha floor at 48 h. No hard hide, which is acceptable |
| S4 | **Fixed, with edge cases** ✔ | One official track per storm; GIS track is violet `#B266FF`; OFCL has a dark outline; swatch matches. Storm matching is name + basin, not bin number (N2, N3) |
| S5 | **Fixed** ✔ | Legend picks the cycle by `init` across shown storms and labels mixed cycles |
| S6 | **Fixed** ✔ | Monotonic `Instant` via a `Clock` trait; poll happens before the deadline; all differences use `saturating_duration_since`. No explicit suspend/gap re-arm (N9) |
| S7 | **Fixed** ✔ | Each storm has its own toast streak, cleared only by that storm's success |
| S8 | **Fixed (core)** | Falls back to the newest cycle of any type once OFCL is more than 18 h behind; future-dated rows are excluded; cycles never mix within a storm. The optional per-tech fallback was not done |
| S9 | **Partially fixed** | Unchanged refetches skip the revision bump (full structural compare) and refreshes get ±10% jitter. Still a full 15-min re-download with no conditional GET, and lockstep fires at startup and re-enable (N10) |
| S10 | **Partially fixed** | `fit!()` height, a Compact mode and a top clamp were added, but the top-bar constant is wrong and Full mode overlaps the bottom bar at 400–490 px window heights (N5, N6) |
| S11 | **Partially fixed, with a regression** ✔ | Per-group screen-space dash patterns with continuous phase. **Dashes are not clipped per segment, so draw calls explode at high zoom** (N1). Width is still 1.5 px for every group |
| L1 | **Partially fixed** | "No official forecast" state and "(stale)" text are fixed. Attribution appears only in Full mode, and "may differ from official products" was never added. The fit test still counts characters (mitigated by ply word-wrap) |
| L4 | **Mostly fixed** ✔ | 404 is detected from the exact ureq status format; parse and gunzip errors are quiet; 404 has its own flat 600 s cadence and a "Waiting for model guidance" status; backoff is capped at 300 s. The `next_due` reset on NHC success was not done (N8) |
| L15 | Not fixed (out of scope) | Noted only. The commit declares it out of scope; swatch widths and the two greys are unchanged |

---

## Per-ID detail

### S1. Fixed ✔

- **Cycle parsing:** `adeck.rs:118-123` (`cycle_time_unix`) uses `NaiveDate::from_ymd_opt` and `NaiveTime::from_hms_opt` followed by `.and_utc()`. It has no unwrap, and since it never touches a local timezone, DST can't affect it.
- **Thresholds:** `adeck_style.rs:713` sets `CYCLE_STALE_AFTER_SECS` to 12 h, and `adeck_style.rs:721` sets `CYCLE_DIM_AFTER_SECS` to 24 h.
- **Per-frame dimming:** `track_alpha_mult` is computed every frame (`adeck_style.rs:818-823`) and applies to the outline, segments and labels (`scope.rs:2109-2117`, `:2128`, `:2180`).
- **Shared label:** the card (`main.rs:2899-2916`) and the legend (`main.rs:3162-3171`) both use `cycle_freshness_line`.
- **Clock:** cycle age uses `chrono::Utc::now()`, which is correct against a UTC cycle.
- **Tests:**
  - `cycle_freshness_line_single_stale_cycle`
  - `is_stale_cycle_boundary`
  - `track_alpha_mult_changes_with_now_independent_of_any_store_mutation`
  - `cycle_time_unix_rejects_impossible_calendar_values`
- **Gap:** no test covers the card or legend wiring in `main.rs`.

### S2. Fixed ✔ (no test on the gate)

- **The gate:** `main.rs:1575` is `if background.parse_queue_has_capacity() && !state.adeck_fetch.has_parsing()`; the old `while` is now an `if`.
- **Headroom:** the worst case is 3 other jobs plus 1 a-deck job against capacity 4, and the worker has already taken one job off the queue, so there is room.
- **No stuck "parsing" state:** a slot leaves `Parsing` only through `parse_finished`, `disable`, or removal of the slot.
  - The worker always sends a result.
  - A `JoinError` or panic becomes `Err`, which goes to `parse_finished(false)`.
  - A submit `Err` releases the slot (`main.rs:1588-1594`).
- **Throughput:** serial parsing adds latency for the last of 5+ storms, a few seconds in release builds. That is acceptable.
- **Test:** only `has_parsing_true_only_while_a_slot_is_in_the_parsing_phase` exists. Deleting `&& !has_parsing()` fails nothing (N4).

### S3. Fixed ✔

- **Insert guard:** `adeck.rs:550-552`: `if is_plausible_cycle(existing.init, now_unix) && guidance.init < existing.init { return RejectedOlder }`. Because the guard only applies when the stored cycle is plausible, a bogus stored cycle can't block recovery.
- **Dimming:** grey-out down to a 0.35 alpha floor at 48 h (`adeck_style.rs:770-778`). It is recomputed each frame, so dimming keeps going during an outage.
- **Tests:**
  - `insert_rejects_a_cycle_older_than_the_one_stored`
  - `insert_recovers_from_a_stored_future_dated_cycle`
  - `insert_recovers_from_a_stored_calendar_invalid_cycle`
  - `track_alpha_mult_dims_past_the_cycle_age_threshold`

### S4. Fixed, with edge cases ✔

- **One official track:**
  - `draw_model_tracks` skips the Official group when `gis_track_shown_for` is true (`scope.rs:2092-2102`).
  - The check respects `overlays.show_track`, and it requires a non-empty GIS track (`adeck_style.rs:600-622`).
  - When the track layer is switched off, OFCL comes back.
- **GIS colour:** the GIS advisory track is violet `#B266FF` (`adeck_style.rs:457`; `scope.rs:1920-1922`, `:1945`). The legend has "NHC advisory track" and "NHC advisory cone" rows (`model_legend.rs:181-187`).
- **OFCL outline:** `#101010` at alpha 230 (`adeck_style.rs:405-410`).
- **Legend swatch:** three wind-colour segments inside a light outline, chosen because the legend panel is dark (`model_legend.rs:146-160`, `:249-271`).
- **Matching:** despite commit `257159b`'s title ("match storms by bin number"), `gis_track_shown_for` matches on an exact case-insensitive **name** plus the bin's **two-letter basin prefix**, aliasing AT to AL. The bin digit is never compared.
  - Bin reuse can't cause a mismatch.
  - Every weakness of name matching remains, and it compounds with a pre-existing substring merge in `nhc.rs` (N2, N3).
- **Tests:** 12 `gis_track_shown_for_*` and `basin_matches_*` tests, plus:
  - `official_outline_is_dark_not_white`
  - `official_swatch_segments_are_distinct_wind_categories`
  - `gis_advisory_color_is_off_the_saffir_simpson_palette`, which is weak because it only checks inequality.

### S5. Fixed

- **Cycle selection:** `adeck.rs:613-626` (`earliest_cycle` by `init` over shown storms, plus `has_single_cycle`), used at `main.rs:3151` and `:3162-3166`.
- **Mixed cycles:** shown as "Mixed cycles, oldest X cycle, age (stale)". The text wraps, because ply defaults to `WrapMode::Words`.
- **Tests:**
  - `earliest_cycle_picks_by_init_not_fetch_order`
  - `has_single_cycle_false_when_shown_storms_differ`
  - `cycle_freshness_line_mixed_still_surfaces_the_oldest_shown_cycles_age`

### S6. Fixed ✔

- **Clock:** the `Clock` trait (`adeck_fetch.rs:167-177`) makes `Instant` used end to end.
  - `next_due: Instant` (`:235`) and `InFlight { started_at: Instant }` (`:224`).
  - `tick` reads the clock once (`:304`).
- **Ordering:** poll happens before the deadline (`:353-362`), and the deadline uses `saturating_duration_since`.
- **Remaining `get_time()` calls** (`main.rs:946`, `3872`, `4025`, `4247`, `4277`, `4427`) are UI and toast timing only.
- **Tests:**
  - `response_arriving_same_tick_as_the_deadline_is_processed_not_cancelled`
  - `deadline_cancels_hung_request_and_retries`
  - `deadline_requires_real_elapsed_time_not_just_more_ticks`
  - All scheduler tests use `with_clock(FakeClock)`, so reverting to `f64` wall time breaks compilation.

### S7. Fixed ✔

- **Per-storm streak:** `toast_shown_in_streak` is now a field on `Slot` (`adeck_fetch.rs:240`). It is set in `record_failure` (`:263-267`) and cleared only when that slot succeeds (`:436`).
- **Cleanup:** state is dropped automatically when the slot is removed (`:321`) or on `disable` (`:460`).
- **Tests:**
  - `one_storms_success_does_not_reset_another_storms_toast_streak`
  - `another_storms_first_failure_still_toasts_during_an_unrelated_ongoing_failure`
  - `parse_failure_is_quiet_and_does_not_spend_the_toast_streak`
- **Open point:** see N11, the outage toast count.

### S8. Fixed (core)

- **Fallback:** once the OFCL gap exceeds 18 h, `adeck.rs:367-391` falls back to the newest cycle of any type (`:333`). Implausible and future-dated rows are excluded from "newest" (`:351-353`, `:371`, `:377`).
- **No mixing:** `build_guidance` filters `r.init == init`, so cycles never mix within one storm (`adeck.rs:398`).
- **Labelling:** the fallback sets `has_official = false`, which shows "No NHC official forecast this cycle" on the card and "No OFCL this cycle" in the legend. The wording isn't specific to the fallback (N13).
- **Not done:** the optional per-tech fallback for non-default models.
- **Tests:**
  - `latest_cycle_falls_back_to_newest_any_once_ofcl_lag_exceeds_threshold`
  - `latest_cycle_pins_to_last_ofcl_cycle_is_no_longer_permanent`
  - `latest_cycle_ofcl_lag_threshold_boundary`
  - `latest_cycle_excludes_future_dated_rows_from_newest`

### S9. Partially fixed

- **Fixed: unchanged refetch.**
  - It updates only `fetched_at_unix` and skips the revision bump (`adeck.rs:553-559`).
  - The comparison is a full structural `PartialEq`, so it can't falsely report "unchanged".
  - "Checked" still advances.
- **Fixed: jitter.**
  - ±10% FNV-1a jitter keyed by storm id on success (`adeck_fetch.rs:66-75`, `:434`).
  - Bounded to [-90, +90) s.
- **Not fixed:**
  - A full download, gunzip and parse still happen every 15 min, with no conditional GET and no alignment to the cycle schedule.
  - Initial fires are still in lockstep (N10).
- **Tests:**
  - `insert_same_cycle_unchanged_content_does_not_bump_revision`
  - `insert_outcome_distinguishes_rejected_unchanged_and_replaced`
  - The jitter is covered only incidentally by `success_cancels_handle_then_refreshes_same_id_later`, which fails on revert only because `al092026`'s jitter happens to be negative.

### S10. Partially fixed

- **What changed:** `model_legend.rs` now uses `.height(fit!())`. `legend_mode()` selects Compact on `is_mobile` or when height is under 400 px, and `legend_y()` clamps the legend below the top bar.
- **Problems:** the top-bar constant is wrong (N6), and Full mode overflows or overlaps on 400–490 px windows (N5).
- **Tests:** 5 `legend_mode_*` and 4 `legend_y_*` tests. They test only the pure helpers; nothing checks that the real panel fits.

### S11. Partially fixed, with a new regression ✔

- **Patterns:** per-group `DashPattern` in screen pixels (`adeck_style.rs:~255-320`):
  - Consensus 12/6, Hurricane 8/4, Ensemble 5/5, AI 3/3, Global 1.5/4.
  - Official and Observed are solid.
- **Phase and edge cases:**
  - Dash phase carries across a track's whole polyline (`scope.rs:2123-2136`).
  - Zero-length and NaN segments draw nothing.
  - The legend previews each group's pattern.
- **Regression:** dashes are not clipped per segment (N1).
- **Line width:** every group is still 1.5 px. Ensemble 5/5 and AI 3/3 look alike at that width (N14).
- **Tests:** the `dash_steps_*` and `dash_preview_segments_*` tests, plus:
  - `default_visible_model_groups_have_distinct_dash_patterns`
  - `official_and_observed_are_solid`

### L1. Partially fixed

- **Fixed:**
  - Three-state official entry: `OfficialLegendState` Drawn / SuppressedForGis / NotIssued (`adeck_style.rs:469-508`, wired at `main.rs:3195-3214`), with tests.
  - "(stale)" text in the legend via `stale_suffix` (`adeck_style.rs:514`; `main.rs:3171`, `:3178-3184`).
- **Partial:** "Source: NOAA/NHC ATCF" and "Consult nhc.noaa.gov" appear only in Full mode (`model_legend.rs:216-224`). Compact mode, which is used on mobile and windows under 400 px tall, drops both. "May differ from official products" was never added.
- **Not fixed:**
  - `disclaimer_lines_fit_the_panel` still checks `chars().count() <= 48` (`adeck_style.rs:1771-1782`).
  - The new `official_legend_labels_fit_the_legend_panel` also counts characters.
  - The longer freshness lines have no fit test.
  - Mitigation: ply wraps on word boundaries, so text wraps rather than clips (see N12 for the height effect).

### L4. Mostly fixed ✔

- **404 detection:** `is_not_found` (`adeck_fetch.rs:102-107`) requires the exact `"http status: "` prefix and parses the remainder as `u16 == 404`. That matches ureq-3.3.0's `error.rs:232` format, passed through by ply's `net/mod.rs:158`.
- **Quiet errors:** parse and gunzip errors don't toast (`record_failure(slot, now, true, false)`, `:444`).
- **404 cadence:** a flat `NOT_FOUND_RETRY_SECS = 600` (`:49`), with a `WaitingForGuidance` status reading "Waiting for model guidance, checking again in N s".
- **Backoff cap:** 300 s (`:44`).
- **Not done:** resetting all `next_due` when the NHC fetch succeeds (N8).
- **Tests:**
  - `not_found_retries_quietly_on_its_own_flat_cadence`
  - `is_not_found_matches_ureqs_exact_status_format_only`
  - `backoff_caps_lower_than_a_healthy_refresh`
  - `backoff_doubles_then_caps_at_300s`

---

## New findings

These were introduced or exposed by the fixes. They're ordered by severity.

### N1. High (perf): dash draw calls explode at high zoom because segments aren't clipped ✔

- **Where:** `scope.rs:2102-2156` (`draw_model_tracks`), `adeck_style.rs:169-206` (`dash_steps`).
- **Evidence:**
  - The only cull is the whole-track bounding box (`scope.rs:2102-2106`).
  - For each segment, `dash_steps` loops `while pos < length_px` over the full on-screen length, pushes one `DashSpan` per dash into a new `Vec`, and each span becomes one `draw_line`.
  - No segment is clipped to the viewport.
- **Failure scenario:**
  - Max zoom is 8 (`main.rs:3866`), and `px_per_km` is about 19 at 1080p and about 75 on a 2160 px window.
  - A 2000 km model track is then about 37k–150k px long.
  - With Global's 5.5 px cycle, that's about 7k–27k `draw_line` calls per track; Consensus (18 px cycle) gives about 2k–8k.
  - Across 10–30 visible tracks that's roughly 100k–800k quads per frame, plus a `Vec` allocated per segment. That is a severe frame-time collapse (estimated, not measured).
  - The doc comments claim the pattern "can't explode", which holds for pattern scale but not for call count.
- **Fix:**
  - Clip each segment to the screen rectangle plus a margin (Liang–Barsky) before dashing, and advance the dash phase by the clipped-off length.
  - Also cap the dash count per segment, falling back to a solid line.
  - Reuse one span buffer, or iterate lazily.
  - Add a test that a long off-screen segment emits zero spans.

### N2. Medium (safety): OFCL can be suppressed for the wrong storm ✔ (mechanism) / plausible (trigger)

- **Where:** `nhc.rs:783-787` (`storms.iter_mut().find(|s| name.contains(&s.name))` then `storm.track = ...`, last write wins), read by `adeck_style.rs:600-622`.
- **Failure scenario:**
  - One storm's name contains another's, for example "Fourteen-E" contains "Four".
  - That lets the track-layer merge attach storm B's GIS track to storm A's `StormGis`.
  - A's track is then non-empty, so its a-deck OFCL is suppressed, and the only "advisory track" left for A is B's violet line somewhere else.
  - Other colliding pairs include Six/Sixteen-E, Seven/Seventeen-E and One/Twenty-One.
  - The substring merge predates this branch, but before the S4 fix it couldn't hide OFCL.
- **Live evidence:** NHC's MapServer track and cone layers carry `binnumber` and `stormnum`, and they match CurrentStorms exactly (checked live: `ep212026` / `EP1` / `stormnum 21`). `nhc.rs` doesn't parse either field.
- **Fix:**
  - Parse `binnumber` (and `stormnum`) into `StormGis`, and merge layers on `binnumber` instead of name substring.
  - Match in `gis_track_shown_for` on `meta.bin_number == g.bin_number`, optionally cross-checking `stormnum` and basin against `meta.id`.

### N3. Low–Medium (safety): name mismatches bring back two official tracks

- **Where:** `adeck_style.rs:555-566`, `:612-621`.
- **Failure scenarios:** any naming difference between CurrentStorms and MapServer breaks the exact name match, so both the GIS track and the a-deck OFCL are drawn, which is the S4 failure again:
  - a TD being renamed (TD Fifteen becoming TS Nadine while MapServer lags);
  - PTC or TD naming formats;
  - an empty `binNumber`;
  - possibly an EP→CP crossover (unverified).
- **Fix:** the bin-number join from N2. At minimum, derive the basin from `meta.id[..2]` with `str::get`.

### N4. Low–Medium (test): no regression test for the S2 gate ✔

- **Where:** `main.rs:1575`.
- **Problem:** the cap exists only inline in the frame loop. Changing it back to the old `while` leaves the whole suite green.
- **Fix:** enforce the cap inside the fetcher, so `take_parse_job` returns `None` while `has_parsing()` is true. That's easy to test directly.

### N5. Medium (layout): Full legend overflows on 400–490 px windows

- **Where:** `model_legend.rs:~56-96` (`COMPACT_HEIGHT_TRIGGER_PX = 400`, `FULL_HEIGHT_ESTIMATE = 420`), `main.rs:~3236`.
- **Failure scenario:**
  - Full mode has a header, 8 wind rows, 1 OFCL row, 3 key rows, 5 dash rows and 6 text lines. That's roughly 380–400 px (an estimate).
  - At window heights of 400–486 px, `legend_y` clamps to 36, so the panel overlaps the 46 px bottom bar or runs off-screen.
  - Larger text (dyslexia font, high DPI) makes it worse.
- **Fix:** raise the trigger to about 500 px or derive it from measured content, or anchor the panel to the bottom with a ply floating or attach layout so no height estimate is needed.

### N6. Medium (layout): wrong top-bar constant

- **Where:** `main.rs:~3237` passes `36.0`.
- **Evidence:** the controls bar is about 40 px on desktop (24 px controls plus 8 px padding) and at least 60 px on mobile (44 px controls). It also uses `.wrap()`, so it can be several rows tall (`main.rs:1927-1936`, `:1954`).
- **Fix:** compute the real bar height, or lay the legend out in the flow below the bar.

### N7. Low (safety): stale GIS advisory can hide a newer a-deck OFCL

- **Where:** `main.rs:1486-1491` keeps the old `nhc_bundle` when an NHC fetch fails. `gis_track_shown_for` never compares the advisory's time with the OFCL `init`.
- **Failure scenario:** NHC's GIS feed fails or lags while the a-deck host keeps updating. The new OFCL is suppressed in favour of an old violet advisory line that carries no age cue.
- **Fix:** suppress only when the bundle is fresh, or when the advisory's synoptic time is at least the OFCL `init`.

### N8. Low: no `next_due` reset when NHC succeeds (L4 remainder)

- **Problem:** after an outage, a storm in backoff waits up to 300 s, and a 404 storm up to 600 s, even after NHC has just refreshed.
- **Fix:** add `AdeckFetcher::nudge_due()` and call it from the NHC success path. It should set `next_due = now` for idle slots with failures or a 404.

### N9. Low (plausible): long suspend still counts a failure

- **Problem:** there's no explicit suspend or frame-gap detection. Poll-first fixes the "finished download discarded" case.
- **Remaining case:** a request that is truly pending across a long suspend times out on the first tick after wake, and toasts. On Windows, `Instant` (QPC) may include suspended time; that was not verified.
- **Fix:** when `now - last_tick` exceeds about 30 s, re-arm `started_at` for every in-flight slot.

### N10. Low: lockstep fires at startup and re-enable; no backoff jitter

- **Where:** `adeck_fetch.rs:333` (`next_due: now` for every new slot), `:253-261` (`record_failure` adds no jitter), `:454-461` (`disable` clears the slots).
- **Problem:** every storm fires in the same frame at startup and on each overlay toggle. The one-in-flight parse cap serializes the parsing, but the downloads still burst.
- **Fix:** seed the first `next_due` with a jitter offset, and jitter the backoff.

### N11. Low: one toast per storm in a staggered outage; 404 streak inflates backoff ✔

- **Toast count:**
  - Streaks are per storm, and the jitter spreads failures across ticks. So a full outage toasts once per storm over a few minutes, not once per outage.
  - `show_toast` replaces rather than stacks, so this is mild.
  - No test covers the all-storms-down case.
- **Backoff inflation:**
  - `record_failure` increments `slot.failures` on 404s too (`adeck_fetch.rs:254`).
  - After hours of 404s, the first real transport error jumps straight to the 300 s cap.
- **Fix:**
  - Toast only on a fetcher-wide transition, or skip the toast when another slot is already in a streak.
  - Don't count 404s in `failures`.

### N12. Low: compact legend loses the attribution and GIS key; wrapped lines overflow the height estimate

- **Where:** `model_legend.rs:177-201`, `:216-224` (Full only), `:29` (`COMPACT_HEIGHT_ESTIMATE = 130`).
- **Mobile key:** on mobile, "OFCL shown as advisory track" points at a violet line that has no swatch, and the source and consult lines are gone.
- **Height:** a 62-character "Mixed cycles, … (stale)" line wraps to 2–3 lines, which can push Compact to about 150 px, over the 130 px estimate.
- **Dash key:** Compact mode also drops the dash key.
- **Fix:**
  - Keep one source line in Compact.
  - Show the violet swatch whenever the state is SuppressedForGis.
  - Keep a one-line dash note.
  - Base the estimate on wrapped line count.

### N13. Low: card and legend disagree; fallback label is generic

- **Problems:**
  - The card says "No NHC official forecast this cycle" whenever `!has_official` (`main.rs:2951-2955`), even when the advisory track is drawn. The legend was reworded to avoid exactly that claim.
  - The card's "Checked" line is still colour-only (`main.rs:2919-2937`).
  - The S8 fallback reads the same as an invest ("No OFCL this cycle"), even though an older OFCL exists and was skipped.
- **Fix:**
  - Reuse `OfficialLegendState` in the card, and add "(stale)" text there.
  - Name the last OFCL cycle in the fallback label, for example with a `fallback_from_ofcl: Option<u32>` field.

### N14. Low (accessibility): GIS violet over intense echo; dash patterns too similar

- **Violet:** `#B266FF` against the 65–70 dBZ band is about ΔE 11–12 under protan or deutan vision, and the 2 px track has no outline.
- **Dashes:** Ensemble 5/5 and AI 3/3 are hard to tell apart at 1.5 px, and the legend previews draw at 2 px.
- **Fix:** give the GIS track the same dark underlay as OFCL, and vary width as well as dash pattern per group.

### N15. Low: clock skew

- **Where:** `adeck.rs:351-353`, `:367-372`, `:463-464`.
- **Slow clock:** a system clock that runs behind understates cycle age, so tracks show no amber and no dim (fails open).
- **Far behind:** if the clock is more than the newest cycle's age plus 3 h behind, every row looks future-dated, so `latest_cycle` returns `None`. The parse then fails with the misleading "has no parseable rows" and the card shows "retrying" indefinitely.
- **Fast clock:** a clock that runs ahead only over-dims, which is safe.
- **Fix:**
  - Give a distinct error and status when every row is future-dated ("check system clock").
  - Optionally compare against the HTTP `Date` header once headers are available.

### N16. Low: rejected or vanished cycles

- **Where:** `adeck.rs:550-552`; `main.rs:1218-1243`, where `parse_finished(true)` runs before `insert`.
- **Rejected older cycle:** it counts as a successful fetch, so the slot looks healthy while "Checked … (stale)" grows with no explanation.
- **Vanished cycle:** a stored cycle that later disappears from the file (withdrawn, or a stray row up to 3 h in the future on the fallback path) blocks the real older cycles until a newer one arrives, about 6 h or less.
- **Drop-out:** the fallback can switch a drawable cycle for one that so far holds only CARQ or intensity rows, so the storm drops off the map until that cycle fills in.
- **Fix:**
  - Accept an older cycle once the stored cycle is no longer in the file, or after N agreeing fetches.
  - Surface "server returned an older cycle".
  - Require at least one default-visible track before the fallback switches cycles.

### N17. Low: `dash_steps` can loop forever on infinite or huge lengths

- **Where:** `adeck_style.rs:175-205`.
- **Problem:** an f32 `pos` stops advancing around 2²⁵ px when the step is under 2. An infinite or enormous `length_px`, which a non-finite pan could produce, would then never end the loop.
- **Reachability:** not reachable with finite geometry today.
- **Fix:** bail out to a solid line when `!length_px.is_finite()` or the length exceeds about 1e6, or when `pos + step == pos`. The N1 clip also removes this.

### N18. Nits

- **Stale comments:**
  - `adeck_style.rs:212` still says "White outline drawn under the line (OFCL only)".
  - The doc comment at `adeck.rs:312-313` still says `fetched_at_unix` drives the "updated N min ago" label.
  - The comment at `main.rs:1236-1238` says "older/implausible cycle", but an implausible *incoming* cycle is never rejected.
- **Inefficiency:** `guidance_content_eq` (`adeck.rs:473-483`) deep-clones both guidances on every insert.
- **Weak test:** `jitter_secs_differs_across_storm_ids` uses `dedup_by` on an unsorted `Vec`.
- **Fail-open path:** an unparseable `init` is never dimmed or marked stale (`adeck_style.rs:822` `unwrap_or(1.0)`, `:882`). It's unreachable today; the safe default is to treat it as stale.
- **Unrealistic fixtures:**
  - Bins "AT14" and "AT9", where real Atlantic bins are AT1–AT5.
  - The `nhc.rs` fixture still uses GIS basin "AT" (`nhc.rs:1569`).
  - There's no end-to-end realistic Atlantic GIS fixture.
  - The commit claims the GIS basin reads "AL" for Atlantic storms. That was not verifiable live, because no Atlantic storm was active.
- **Wiring tests missing** for `scope.rs:1614-1622` and the legend inputs at `main.rs:3195-3214`.

---

## Checked and found fine

- **S1 parsing:** Feb 30, hour 24 and above, and month 0 or 13 all return `None`; leap days are handled; there's no unwrap outside tests; no local timezone or DST is involved.
- **Bogus cycles:** cycles more than 3 h in the future are excluded from selection, and a stored implausible cycle can't latch.
- **Clocks:** cycle age uses `Utc::now()` and the scheduler uses `Instant`; no units are mixed.
- **Thresholds and disclaimer:**
  - Amber and dim thresholds come from shared constants, and the card and legend share one freshness line.
  - Dimmed tracks still show the disclaimer.
  - Dimming isn't baked into the projection cache.
- **No stuck parse state:** `has_parsing()` can't stick. The flag is derived from the slot's phase, and every exit path releases it.
- **Arithmetic and ids:**
  - Every `Instant` difference saturates, so subtraction can't panic.
  - Backoff can't overflow.
  - Generations are unique.
- **S4 matching basics:**
  - The AT→AL / EP / CP prefix mapping is correct, and EP is confirmed live.
  - Comparisons are case-insensitive and trimmed.
  - A failed lookup returns "not suppressed".
  - Invests with no GIS data, and empty GIS tracks, aren't suppressed.
  - `metas` and `gis_storms` come from the same bundle, which is replaced atomically.
- **OFCL styling:**
  - OFCI and OFC2 aren't default-visible.
  - The dark OFCL outline has good contrast across the wind palette.
  - OFCL stays distinct because it's solid while the models are dashed.
- **Dash maths:**
  - Phase continuity holds.
  - Zero-length, negative and NaN segments draw nothing.
  - `rem_euclid` is reached only when `cycle > 0`.
  - The pattern is in screen space and doesn't change with zoom.
  - Dashes inherit the per-segment wind colour.
- **Compact legend:** it keeps the S1 and S4 safety content (cycle line, checked line, official state, "Not an official forecast").
- **Code hygiene:** no new byte slicing (`chars().take(2)`), no new unwraps, no emoji or non-`SYMBOL_FONT` glyphs. The header still uses `nf::HURRICANE`.
- **CI checks:** tests, clippy `-D warnings` and fmt all pass.

## Suggested next steps

1. **N1:** clip segments before dashing. This blocks merging, because high zoom becomes unusable.
2. **N2 + N3 + N7:** parse `binnumber` into `StormGis`, merge layers on it, match OFCL suppression on the bin number, and suppress only when the advisory is at least as new as the OFCL.
3. **N4:** move the S2 cap into the fetcher and test it.
4. **N5 + N6 + N12:** bottom-anchor the legend, use the real top-bar height, and keep attribution and the GIS swatch in Compact mode.
5. **N8, N10, N11, N13, N15, N16:** scheduler and labelling polish.
6. **N9, N14, N17, N18:** as capacity allows.
