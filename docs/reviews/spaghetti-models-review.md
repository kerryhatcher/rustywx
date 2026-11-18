# Spaghetti-models overlay review

- **Date:** 2026-10-16
- **Branch:** `feat/spaghetti-models`, `main...HEAD`, 19 commits, head `8d27f8f`
- **Worktree:** `rustywx-spaghetti`
- **Design references:**
  - `docs/plans/spaghetti-models-overlay.md`
  - `docs/research/spaghetti-models.md`
  - `docs/reviews/windows-stability-review.md`

## Method

- **Team:** 7 reviewers, 2 Opus and 5 Sonnet. Each owned one slice of the branch:
  - a-deck parse correctness
  - fetch scheduler and network resilience
  - UI-thread work and performance
  - rendering, legend, card, chart, and the icon rule
  - settings and tests
  - architecture and integration
  - safety and panics
- **Read-only:** nothing was edited or committed. Only the settings and tests reviewer ran `cargo test`.
- **Verification:** reviewers had to ground every finding in the source. The lead reviewer then re-read the source for each finding marked ✔.
- **Disagreements:** where reviewers disagreed, the lead read the code and settled it. See [Refuted claims](#refuted-claims).
- **Estimates:** where a cost or size is given, it is an estimate from reading the code, not a measurement.

## Test results

`cargo test -p rustywx` passes:

- library: 280 passed, 0 failed
- binary: 2 passed
- doc tests: 0

## Verdict

- **No critical findings.**
- **One high finding:** the "updated N min ago" label measures fetch time, not cycle age (S1).
- **The integration is sound:**
  - Gunzip and parsing really run on the background worker.
  - The projection cache is keyed on `(site_index, revision)` and doesn't depend on zoom or pan.
  - Every network exit path calls `cancel()`, so the C2 cached-`Err` trap is avoided.
  - Stale results are dropped by generation.
  - A parse panic turns into a `JoinError`, which becomes an `Err`, so the shared worker survives.
  - There's no reachable panic in a release build.
- **Most of the medium findings fall into two groups:**
  1. The shared parse queue can be filled by a-deck jobs (S2).
  2. Several places can show data as fresher or more official than it is (S3–S7).

## Severity legend

- **High:** can mislead a user about forecast data in normal operation.
- **Medium:** a real defect with bounded impact, or a narrower trigger.
- **Low:** latent problems, polish, or hygiene.

Confidence is **confirmed** when the mechanism was read in the source and the trigger is reachable. It is **plausible** when the mechanism was read but the trigger or size wasn't shown.

## Summary table

| ID | Sev | Area | Finding |
|---|---|---|---|
| S1 | High | safety | Freshness label and "(stale)" count from fetch time; cycle age is never shown or checked |
| S2 | Medium | integration | A-deck submit loop can fill the shared 4-slot parse queue and starve NHC metadata, alerts, and borders |
| S3 | Medium | safety | No cycle-age threshold hides or dims old tracks, and `insert` accepts an older cycle over a newer one |
| S4 | Medium | safety | Official forecast is easy to confuse with the models: two "official" tracks, a GIS track colour close to Cat 3, and OFCL Cat 1–2 colours lost in the white outline |
| S5 | Medium | safety | Legend shows one cycle for every storm, chosen by fetch order, not cycle order |
| S6 | Medium | scheduler | Deadline and backoff use wall-clock `get_time()`; the deadline is checked before polling |
| S7 | Medium | scheduler | One fetcher-wide toast flag, so a single bad storm re-toasts about every 15 min and one storm's failure hides another's |
| S8 | Medium | correctness | Global newest-OFCL cycle selection holds every model on the old cycle when OFCL lags, and stays pinned when OFCL stops |
| S9 | Medium | network/perf | Full re-download every 15 min with no conditional GET; unchanged data still bumps the revision; all storms refresh in lockstep |
| S10 | Medium | UI | Legend has a fixed 262 px height and a hard-coded y position, so it goes off-screen or overlaps on short or mobile windows |
| S11 | Medium | UI | No per-group line styles; wind colour alone tells models apart (the plan said dashed or dotted) |
| L1–L19 | Low | mixed | See the Low section |

---

## High

### S1. "Updated N min ago" and "(stale)" measure fetch time, not cycle age ✔

- **Where:**
  - `app/src/main.rs:1222` sets `guidance.fetched_at_unix = chrono::Utc::now().timestamp()` on every successful parse.
  - `app/src/adeck_style.rs:162-196`: `STALE_AFTER_SECS = 30 * 60`, `format_age`, and `is_stale`, which all take `fetched_at_unix`.
  - `app/src/adeck.rs:295-302` (`latest_cycle`).
- **Category:** safety (misleading). **Confidence:** confirmed.
- **Evidence:**
  ```rust
  pub fn is_stale(fetched_at_unix: i64, now_unix: i64) -> bool {
      now_unix - fetched_at_unix > STALE_AFTER_SECS
  }
  ```
  `init` (the cycle `YYYYMMDDHH`) is only formatted for display by `format_cycle`. It's never compared with the current time.
- **Failure scenarios:**
  - **Stalled feed:** the a-deck stops getting new cycles (an ATCF outage, a file that isn't rewritten, a mirror lag). Every 15-min refetch resets the label to "updated just now" and clears "(stale)", while the tracks are 12–48 h old.
  - **Normal operation:** `latest_cycle` requires OFCL, and OFCL is written about 3 h after the nominal time. So the cycle shown is usually 3–9 h old under a "just now" label.
  - This tool may be used during a hurricane threat, so presenting old guidance as current is the most serious finding on the branch.
- **Proposed fix:**
  1. Parse `init` into a UTC `chrono::DateTime`, rejecting an invalid month, day, or hour.
  2. Show the cycle age ("12 UTC cycle, 7 h old") as the main freshness signal.
  3. Turn amber past about 12 h, and dim or hide tracks past about 24 h (see S3).
  4. Keep the fetch time as a secondary "checked N min ago" line.
  5. Base `is_stale` and the legend's age line on cycle age.

---

## Medium

### S2. A-deck jobs can fill the shared parse queue and starve NHC, alerts, and borders ✔

- **Where:**
  - `app/src/main.rs:1542-1566` (submit loop).
  - `app/src/background_work.rs:20` (`QUEUE_CAPACITY = 4`) and `:233-258`. One serial worker runs `spawn_blocking(...).await` before the next `recv`.
  - The starved paths are borders at `main.rs:1317`, alerts at `main.rs:1366`, and NHC metadata at `main.rs:1426`.
- **Category:** integration/regression. **Confidence:** the mechanism is confirmed (4 reviewers agree). How often it happens in the field is plausible. This is the parse-queue risk the dev reported.
- **Evidence:**
  ```rust
  while background.parse_queue_has_capacity() {
      let generation = next_generation;
      let Some((storm_id, gz)) = state.adeck_fetch.take_parse_job(generation) else { break; };
      ...
      if background.submit_parse(request).is_ok() { ... }
  ```
- **How it happens:**
  1. All storms fire in the same frame. Their refreshes stay clustered at success time + 900 s.
  2. When 5 or more a-decks are downloaded at once (4 queued plus 1 parsing), or 4 plus any other job, the loop uses every free slot in one frame.
  3. tokio frees a slot only when the worker calls `recv`. So on the next frame, borders, alerts, and NHC (which submit before the a-deck block) see `Full`.
  4. While the backlog lasts, the a-deck loop takes any slot that frees up.
  5. Before this branch, at most 3 parse jobs could exist at once (borders, alerts, NHC), so `Full` could not happen.
- **What each starved job does on `Full`:**
  - **NHC metadata:** `reject_metadata("metadata parse queue is full")` sets the Error state. That shows a false "Network error" toast, and because `nhc_bundle` is already `Some`, the next try waits `nhc_refresh_secs` (300 s by default). The official cone and advisory, which also supply the a-deck storm list, are up to 5 min staler.
  - **Alerts:** the body is dropped and `alerts_fetch_fired = false`. One refresh is lost (120 s), and the log line misleadingly says `retry=true`. Before alerts first load, the C2 pattern re-copies the cached multi-MB body every frame until a slot frees.
  - **Borders (first run only):** the body is dropped and the whole border set is fetched again.
- **Head-of-line delay:** even when the queue isn't full, NHC metadata or alerts can wait behind up to 4 a-deck parses. Each one inflates up to about 20+ MB, builds a `Vec<ADeckRecord>` for every cycle, and allocates a `tech` `String` per row (`adeck.rs:56`, `:370`). This is much worse in debug builds.
- **What does not happen:** no a-deck body is lost. The capacity check plus the single UI-thread sender mean bodies wait in `Phase::Downloaded`. The `Err` arm only covers `Closed`, which means shutdown.
- **Plan gap:** the plan's appendix §4 treats `Full` as normal back-pressure and tells Task 12 to drain the queue to capacity. It never considered a-decks starving the 3 existing jobs.
- **Proposed fix:**
  - Allow only one a-deck in the parse pipeline at a time. Add `AdeckFetcher::has_parsing()` and replace the `while` with `if background.parse_queue_has_capacity() && !state.adeck_fetch.has_parsing()`. That keeps 3 slots free for the 3 existing jobs.
  - Or give a-decks their own channel and worker.
  - Optionally:
    - Stagger the initial fires (one new fire per tick).
    - Filter lines to the chosen cycle before building records, to cut parse time.
  - Separately, the pre-existing "keep the body on `Full`" debt in alerts and borders is Windows review C2, fix 3.

### S3. Nothing hides old tracks, and an older cycle can replace a newer one ✔

- **Where:** `app/src/adeck.rs:389-392` (`GuidanceStore::insert`), and the draw gate at `main.rs:1784-1815`.
- **Category:** safety (misleading). **Confidence:** the code is confirmed. The cycle-regression trigger (a lagging mirror or cache) is plausible.
- **Evidence:**
  ```rust
  pub fn insert(&mut self, guidance: StormGuidance) {
      self.by_storm.insert(guidance.storm_id.clone(), guidance);
      self.revision += 1;
  }
  ```
- **Failure scenarios:**
  - **Old tracks:** the store keeps the last good guidance for as long as the storm is listed. That includes days of failed fetches and time with the overlay off (see L2). The only cue is the amber 10 pt age line, which is fetch-based (S1).
  - **Cycle regression:** nothing checks `init`. If a lagging mirror or cache serves an older file, the older cycle replaces the newer one, and the label resets to "updated just now".
- **Proposed fix:**
  - Refuse an insert whose `init` is older than the stored `init` for the same storm id.
  - Add a cycle-age hide or grey-out threshold (S1).
  - Skip the insert entirely, without bumping the revision, when `init` and the content are unchanged (see S9).

### S4. The official forecast is easy to confuse with the models, and the map can show two "official" tracks ✔

- **Where:**
  - `app/src/scope.rs:1900`: the advisory GIS forecast track uses `from_rgba(0xff, 0xcc, 0x66, 200)` at 2 px. This code predates the branch, but now sits next to the model palette.
  - `app/src/adeck_style.rs:48-60` (wind colours, OFCL style with `underlay_px: Some(5.0)`).
  - `app/src/scope.rs:2043-2053` (OFCL white underlay `from_rgba(0xff,0xff,0xff,230)`).
  - `app/src/widgets/model_legend.rs:44-48` (OFCL swatch, plain white).
- **Category:** safety (misleading) and accessibility. **Confidence:** the colours are confirmed. How much they blend against radar echo is plausible.
- **Failure scenarios:**
  - **Two "official" tracks:** the advisory GIS track (official) and the a-deck OFCL track (labelled "OFCL  NHC official") are both drawn. When the a-deck OFCL comes from an older cycle than the cone, the map shows two diverging tracks that both read as official. That happens when OFCL lags (S8) or the a-deck fetch fails while CurrentStorms keeps updating. The ATCF README warns that these files can differ from official NHC products.
  - **GIS track looks like a model:** its colour `#FFCC66` is close to the Cat 3 model segment `#FFC140` (alpha 210, 1.5 px), and it isn't in the legend.
  - **OFCL colours vanish:** OFCL segments for Cat 1 (`#FFFFCC`) and Cat 2 (`#FFE775`) almost disappear into OFCL's white 5 px outline. The official track loses its intensity colouring right where hurricane strength starts.
  - **Swatch mismatch:** the legend's OFCL swatch is solid white, but on the map OFCL is wind-coloured with a white outline.
- **Proposed fix:**
  - Hide the a-deck OFCL while the advisory GIS track is shown, or draw it only when its cycle matches the current advisory.
  - Add the GIS track to the legend, and move its colour off the Saffir-Simpson palette.
  - Give OFCL a dark outline (about `#101010`), or use saturated Cat 1 and Cat 2 colours.
  - Make the OFCL swatch match how OFCL is actually drawn.

### S5. The legend shows one cycle for every storm, picked by fetch order ✔

- **Where:** `app/src/adeck.rs:420-424` (`oldest()` uses `min_by_key(|g| g.fetched_at_unix)`) and `app/src/main.rs:3089-3098` (legend uses `format_cycle(oldest.init)`).
- **Category:** safety (misleading). **Confidence:** confirmed.
- **Failure scenario:** storm A is on the 06Z cycle and storm B, whose OFCL lags, is on 00Z. The legend reads "06Z Oct 9 cycle" over both sets of tracks. Track labels carry only the model name, with no storm or cycle.
- **Proposed fix:**
  - Show the cycle per storm, or "mixed cycles", or the oldest `init`. Pick the storm for the cycle line by `init`.
  - Keep the least-recently-fetched storm only for the "checked" line.

### S6. Deadline and backoff use the wall clock, and the deadline is checked before polling ✔

- **Where:** `app/src/main.rs:946` (`let now = get_time();`), `app/src/adeck_fetch.rs:211` (`if now - started_at >= ATTEMPT_TIMEOUT_SECS`), and `adeck_fetch.rs:145` and `:272` (`next_due = now + ...`).
- **Category:** scheduler/stability. **Confidence:** confirmed. macroquad's `get_time()` is `miniquad::date::now() - start_time`, which is `SystemTime`, and Windows review L2 asked for `Instant`.
- **Failure scenarios:**
  - **Sleep or resume:** a request in flight when the lid closes shows `now - started_at` well over 60 s on wake. The deadline branch runs before `net.poll`, so even a download that finished is cancelled and thrown away. The result is a false "Network error" toast plus a backoff.
  - **Throttled or minimized window:** the same thing happens after any frame gap of 60 s or more.
  - **Clock steps back (NTP or manual):** `now - started_at` goes negative, so the deadline never fires and a hung request never times out. `next_due` set from the earlier clock also delays refresh by the size of the step.
  - **Clock steps forward:** every in-flight slot times out at once.
- **Proposed fix:**
  - Drive the scheduler from a monotonic clock: `Instant`, or `Duration` since a fixed base `Instant`. Tests keep the injected clock.
  - Poll first, then apply the deadline.
  - Treat a frame-to-frame gap over about 30 s as a suspend: re-arm the deadlines without counting a failure.

### S7. One fetcher-wide toast flag ✔

- **Where:** `app/src/adeck_fetch.rs:157` (`toast_shown_in_streak: bool`), `:143-152` (`record_failure`), `:274` (cleared on any storm's success), and `:294` (cleared on `disable`).
- **Category:** scheduler/UX. **Confidence:** confirmed.
- **Failure scenarios:**
  - **Recurring toast:** storm A fails persistently (backoff capped at 900 s) while storm B succeeds every 900 s. Each success from B clears the flag, so A's next failure toasts again. The user sees a "Network error" toast about every 15 min indefinitely, while everything works.
  - **Hidden failure:** while A's failure holds the flag, storm B's first failure doesn't toast.
  - **No spam in a full outage:** during a total outage the one flag correctly holds N storms to a single toast.
- **Proposed fix:**
  - Keep the streak state per slot, and clear a slot's flag only when that slot succeeds.
  - Or toast only on a fetcher-wide transition from "at least one slot healthy" to "no slot healthy", or on the initial all-fail case.

### S8. A global newest-OFCL cycle holds every model back when OFCL lags, and stays pinned when OFCL stops ✔

- **Where:** `app/src/adeck.rs:295-302` (`latest_cycle`) and `:307-315` (`build_guidance` filters `r.init == init`).
- **Category:** correctness/staleness. **Confidence:** the behaviour is confirmed and matches research doc §5 ("prefer the single newest cycle that contains OFCL"). The impact is plausible.
- **Evidence:**
  ```rust
  records.iter().filter(|r| r.tech == "OFCL").map(|r| r.init).max()
      .or_else(|| records.iter().map(|r| r.init).max())
  ```
- **Failure scenarios:**
  - **OFCL lag:** OFCL for the newest cycle is often written about 3 h after the nominal time. Until then, every model shows the previous 6 h cycle even though fresher guidance exists.
  - **Missing from the chosen cycle:** models that aren't in the chosen cycle are dropped, with no fallback. That covers late runs, AP01–AP30, and TABS/TABM/TABD while a cycle fills in. The default set is mostly early or interpolated models stamped with the current cycle, so the impact falls mainly on the hidden set.
  - **OFCL stops:** after a storm goes post-tropical or becomes a remnant low while it's still listed in CurrentStorms.json, `latest_cycle` stays pinned to the last OFCL cycle forever and newer model-only cycles are ignored. With S1 this then shows as "updated just now".
- **Proposed fix:**
  - When `max_init_any - max_init_ofcl` exceeds about 12–18 h, use the newest cycle and flag it "no recent official forecast".
  - For non-default techs, consider each tech's newest cycle within about 12 h of the selected cycle.

### S9. Full re-download every 15 min, an unconditional revision bump, and lockstep refresh ✔

- **Where:** `app/src/adeck_fetch.rs:20` (`REFRESH_SECS = 900.0`), `:272`, `:62-70` (`PlyNet::get` sends only a User-Agent), `:76` (`resp.bytes().to_vec()` drops the status), and `app/src/adeck.rs:389-392`.
- **Category:** network/perf. **Confidence:** confirmed.
- **Failure scenarios:**
  - **Wasted downloads:** a-decks change about every 6 h, with edits in between. With 5 storms the app pulls about 13 MB of gzip every 15 min (about 50 MB/h), plus a gunzip and parse per storm.
  - **Wasted rebuilds:** each successful parse calls `insert`, which bumps `revision`, so unchanged data still rebuilds the projection cache.
  - **Burst refresh:** every slot refreshes in lockstep with no jitter, which feeds the S2 burst.
- **Constraint:** ply's `Response` has `status()`, but `PlyNet::poll` discards it. ureq treats only 4xx and 5xx as errors, so a 304 would arrive as `Ok` with an empty body, fail to parse, and count as a failure that toasts. Conditional GET therefore needs the status checked in `poll`, or the planned app-owned HTTP module.
- **Proposed fix:**
  - Skip the `insert` and the revision bump when `init` and a body hash or length are unchanged.
  - Add about ±10% jitter to `next_due`.
  - Consider aligning to the cycle cadence (for example 30–60 min outside an expected update window).
  - Add `If-Modified-Since` / `ETag` once the status is available.

### S10. Legend has a fixed height and a hard-coded y position

- **Where:** `app/src/main.rs:3090-3092` (`let y = screen_height() - 54.0 - rustywx::widgets::model_legend::HEIGHT;`, where 54 is the hard-coded "46 px bottom status bar") and `app/src/widgets/model_legend.rs:13` (`HEIGHT`, 262 px, `.height(fixed!(HEIGHT))`).
- **Category:** UI/layout. **Confidence:** the geometry is confirmed; the visible overflow is plausible.
- **Failure scenarios:**
  - In a window shorter than about 320 px, `y` goes negative or collides with the top and bottom bars.
  - On mobile (`is_mobile`, where controls are 44 px), the legend has no special handling.
  - Dyslexia-font mode (`main.rs:701`) and high-DPI text change text height, and the content is already close to 262 px.
  - Windows review M5 means most 125–150% Windows laptops start in the mobile layout.
- **Proposed fix:**
  - Clamp `y` to below the top bar.
  - Use `fit!()` height.
  - Collapse to a compact legend (drop the swatch rows) when `screen_height()` is under about 400 px or on mobile.

### S11. No per-group line styles; colour alone tells models apart

- **Where:** `app/src/adeck_style.rs` (`track_style`). Every non-OFCL group is 1.5 px with no dash. The draw is at `scope.rs:2034-2098`.
- **Category:** UI/accessibility and a divergence from the plan. **Confidence:** confirmed.
- **Failure scenario:**
  - The plan and research doc call for dashed or dotted styles per model family, and none exist.
  - Model families are indistinguishable except by wind colour, which is the same for every model at a given intensity.
  - Thin pale-yellow or cyan lines over green, yellow, and red radar echo are hard to read, and colour-blind users lose most of the information.
- **Proposed fix:**
  - Add a dash or dot pattern or a width per `ModelGroup`, and match it in the legend.
  - Dashes should use a fixed screen-space pattern, so they don't explode into tiny segments at high zoom.

---

## Low

### L1. Legend disclaimer and attribution gaps ✔

- **Where:** `app/src/widgets/model_legend.rs:46-55`, and the card's `DISCLAIMER_LINES` (`adeck_style.rs:~150-157`, card at `main.rs:2843-2850`).
- **What's there:** the legend always shows "Guidance only. Not an official forecast." whenever tracks are drawn, including mobile, where full-screen panels cover the map anyway.
- **What's missing from the legend:**
  - It has no source credit, no "consult NHC", and no "may differ from official products". The full 6-line disclaimer and the "Source: NOAA/NHC" credit are only in the Tropical card, which can be closed and is off-screen while the map is in view on mobile.
  - It always shows the OFCL swatch, even when `has_official == false` (an invest).
  - It signals stale state by colour only (amber). The card appends " (stale)", but the legend doesn't.
- **Fit check:** the test `disclaimer_lines_fit_the_panel` only checks `chars().count() <= 48`. A 48-character line is about 264 px in a 272 px column, and the panel doesn't wrap.
- **Fix:**
  - Add "Source: NOAA/NHC ATCF; consult nhc.noaa.gov" to the legend.
  - Replace the OFCL swatch with "No official forecast" when there isn't one.
  - Add " (stale)" text in the legend.
  - Measure line width in the test, or allow wrapping.

### L2. Toggle-off keeps guidance, and toggle-on refetches everything

- **Where:** `app/src/main.rs:1524-1571` (the fetch block runs only when `show_model_tracks && show_nhc_data`, with `else if is_active() { disable() }`) and `app/src/adeck_fetch.rs:287-295` (`disable` clears all slots).
- **Confidence:** confirmed.
- **Problems:**
  - Turning the overlay off (or turning NHC off with N) cancels requests but never clears `model_guidance` or `model_projection`, and `retain_active` stops running.
  - Turning it back on shows the old tracks at once, honestly labelled by fetch age, until the refetch lands. The plan says tracks clear when the overlay is toggled off.
  - `disable()` also drops each slot's freshness and backoff, so re-enabling refetches every storm right away, even guidance a minute old and storms in 404 backoff.
  - `cancel()` doesn't stop the ureq thread (L11), so rapid toggling stacks parallel downloads.
- **Fix:**
  - On disable, clear `model_guidance` and `model_projection`, or keep them but prune them and refetch based on `fetched_at`.
  - Keep the slots `Idle` with `next_due` and `failures` intact, or seed `next_due = fetched_at + REFRESH_SECS` on re-enable.

### L3. Model-track draw gating is implicit ✔

- **Where:** `app/src/scope.rs:1603-1610`, where the draw is nested in `if let Some((bundle, overlays)) = nhc { if let Some(tracks) = model_tracks { draw_model_tracks(..) } .. }`. `main.rs:646-648` (`model_tracks_enabled(show_model_tracks, guidance_is_empty)`) has no `show_nhc_data` term. `main.rs:1827-1831` passes `nhc = None` when `show_nhc_data` is off.
- **Behaviour today:** correct. Turning off N hides the tracks and the legend together.
- **Problem:** the rule that model tracks need NHC on is enforced only by nesting inside `draw_scope_to_texture`. A future caller that passes tracks without a bundle silently draws nothing. The projection cache is also still rebuilt while NHC is off.
- **Fix:** add `show_nhc_data` to `model_tracks_enabled` explicitly, draw independently of `nhc`, and extend `requires_both_the_setting_on_and_nonempty_guidance`.

### L4. Error classification

- **Where:** `app/src/adeck_fetch.rs:229` (`let quiet = e.contains("404");`), `adeck.rs:370-376` ("no parseable rows"), and `adeck_fetch.rs:28-34` and `:301-312` (backoff and status text).
- **Confidence:** confirmed. The empty-file trigger is plausible.
- **Problems:**
  - **Parse failures toast as "Network error":** a valid but empty a-deck (a brand-new invest), a captive-portal HTML page, or an inflate error goes to `parse_finished(false)` with `quiet = false`.
  - **Fragile 404 detection:** it's a substring match on the error text.
  - **Misleading status:** a 404 still increments `failures` and shows "Model guidance unavailable, retrying in N s" for every new storm that has no a-deck yet.
  - **Slow recovery:** after an outage, a storm at the 900 s backoff cap can wait up to 15 min.
- **Fix:**
  - Treat "no rows yet" and gunzip errors as quiet, like 404, and keep toasts for transport failures.
  - Detect 404 from the status code.
  - Give 404 its own flat 5–10 min retry and a "Waiting for model guidance" status.
  - Cap post-failure backoff lower (about 300 s), and reset all `next_due` when the NHC fetch succeeds.

### L5. Gunzip memory and cap headroom ✔

- **Where:** `app/src/adeck.rs:150-158` (`Ok(String::from_utf8_lossy(&out).into_owned())`) and `adeck.rs:56-58` (a heap `String` for `tech` per row).
- **What's fine:** the decompressed cap really is enforced while streaming (`.take(max_out + 1)` plus a length check), and the compressed size is checked first.
- **Problems:**
  - **Extra copy:** `from_utf8_lossy(..).into_owned()` copies the whole buffer even when it's valid UTF-8 (up to 64 MiB).
  - **Record overhead:** about 90 bytes per `ADeckRecord` against about 30 bytes per line. A crafted 10 MiB gzip that inflates to 64 MiB can push the peak to roughly 300 MB on the worker.
  - **Headroom:** the largest real file is about 2.7 MB of gzip, about 22 MB inflated at the observed 8.3× ratio. So the 64 MiB cap is about 3× headroom, not the "over 20×" the comment says. A long-lived storm that outgrows the cap fails to parse forever and its guidance freezes, which compounds S1 and S3.
- **Fix:**
  - Use `String::from_utf8(out).unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())`.
  - Parse while streaming and drop rows for other cycles early.
  - Store `tech` as `[u8; 4]` or a `SmolStr`-style small string.
  - Cap the record count, and revisit the cap and its comment.

### L6. `chart_max_tau` overflows in debug builds ✔

- **Where:** `app/src/adeck_style.rs:252-259` (`(((max + 23) / 24) * 24).clamp(24, 168)`).
- **Problem:**
  - A row with tau `2147483647` passes the `i32` parse.
  - `max + 23` then panics on the UI thread in debug and test builds.
  - Release builds wrap around and clamp, so they don't crash.
- **Fix:** reject tau outside a sane range at parse time (for example -240..=384), and use `saturating_add`.

### L7. Lenient coordinate and field parsing ✔

- **Where:** `app/src/adeck.rs:88-97` (`parse_lon`), `:67-70` (`parse_tenths`), and `:37-40` (VMAX).
- **Problems:**
  - **Single 360 subtraction:** `parse_lon` subtracts 360 only once, so `"3700E"` becomes 10.0°E and is accepted. One corrupt row makes a track jump across the ocean. ATCF stores 0–1800 tenths plus E/W, so the "East > 180" branch only fires for malformed input, and its "Some feeds write 0..360 east" comment doesn't describe ATCF.
  - **Lenient digits:** `u32::from_str` accepts a leading `+` (`"+270N"`).
  - **Strict letters:** lowercase hemisphere letters and an inner space are rejected (harmless).
  - **VMAX drops the row:** an empty or non-numeric VMAX discards the whole row, including a valid position. Real a-decks write 0, so the impact is probably nil, and a test asserts this behaviour on purpose.
- **Fix:**
  - Reject east values over 1800 tenths, and require all-digit input as `parse_init` does.
  - Optionally flag jumps between consecutive points.
  - Default VMAX to 0 and require only the position fields.

### L8. Only the first gzip member is decoded ✔

- **Where:** `app/src/adeck.rs:151` (`flate2::read::GzDecoder`).
- **Problem:** if NHC ever builds the `.gz` by appending members, later cycles would silently disappear, and an older cycle would be shown as fresh (S1).
- **Fix:** use `flate2::read::MultiGzDecoder`, which also handles single-member files.

### L9. Dedupe ignores TECHNUM and keeps the first row

- **Where:** `app/src/adeck.rs:312-314` (`.entry(r.tau_h).or_insert(r)`).
- **Confidence:** plausible.
- **Problem:** ATCF files are append-only. A later revised fix, such as a second CARQ tau-0 row with TECHNUM 02, loses to the earlier row. Collapsing the 34/50/64 kt wind-radii duplicates is correct, because those rows share position and VMAX.
- **Fix:** keep the last row per (tech, tau), or the first row of the last TECHNUM batch.

### L10. Time and chart labelling

- **Where:** `app/src/adeck_style.rs:167-180` (`format_cycle`) and `app/src/main.rs:377-445` (`draw_intensity_chart`, with `x_at` at about :395).
- **Problems:**
  - **UTC is implicit:** the cycle shows as "12Z Oct 9", marked only by the "Z".
  - **Axis origin:** the intensity chart's 0h–168h axis counts from the cycle start, not from now.
  - **False spikes:** `tau.clamp(0, max_tau)` squeezes points past 168 h onto the right edge.
  - **Label overlap:** Y-axis labels for 83 and 96 kt (and 96 and 113) sit about 8–11 px apart in a 106 px plot at 10 pt, so they touch.
  - **Tick spill:** the "168h" tick label can spill about 3 px past the chart box.
  - **Clipping (plausible):** the chart's custom `draw_*` calls may not be clipped to the panel's `scroll_y` area. This wasn't verified against ply's scissor handling.
- **Fix:**
  - Show "12 UTC Oct 9" with local time beside it.
  - Label the axis "h after 12 UTC".
  - Drop points past `max_tau` instead of clamping them.
  - Thin the y labels.
  - Clip the chart, or check ply's scissor behaviour.

### L11. Leaked ureq threads after timeout

- **Where:** `app/src/adeck_fetch.rs:84-91` (`cancel`) and `:211-213` (deadline). Ply's `Request::cancel` is only `mgr.http_requests.remove(&self.id)`.
- **Confidence:** confirmed.
- **What works:** `cancel()` exists in ply 1.1.1 and does remove the entry, so a same-id refire really refires.
- **Problem:** it can't stop the ureq thread, which has no timeout (Windows review C3). A blackholed server leaks one thread and one socket per attempt, at most one per storm per (60 s + backoff). At the 900 s cap that's about one per storm per 15 min, until the OS TCP timeout ends them. The leak is bounded by rate but not by total count.
- **Fix:** add a global in-flight cap (about 2 concurrent a-deck GETs) or an orphan count that refuses new fires above a limit. The real fix is ply timeouts, or the app-owned HTTP module (Windows review C3/H1).

### L12. Per-frame allocations

- **Where:**
  - `main.rs:2924-2945`: `ChartWidget::IntensityChart(g.intensity.clone())` deep-clones every series every frame while the card is open.
  - `main.rs:1528-1537` and `adeck_fetch.rs:167-178`: per frame, a `Vec<String>` of storm ids, plus `to_ascii_lowercase` and a `format!` URL per storm in `tick`, plus a `gone` vec, plus `retain_active` (O(storms²), at most 5).
  - `scope.rs:2034-2098`: a `pts` vec per visible track, a `labels` vec, and `measure_text` per label, with an O(L²) collision loop.
  - Card and legend: `Utc::now()` and `format!` every frame.
  - `adeck_fetch.rs:76`: `resp.bytes().to_vec()` copies up to about 2.7 MB per download on the UI thread (estimated under 1 ms).
- **Confidence:** confirmed. Each item is small.
- **Fix:**
  - Store `Arc<[IntensitySeries]>` and cache the static band and tick labels.
  - Run the scheduler block only when the metas change or `now >= next_due`.
  - Reuse scratch buffers in the draw, and cache label sizes by tech.

### L13. `ParseResult` has no job identity

- **Where:** `app/src/main.rs:1213-1250` and `app/src/background_work.rs:118`.
- **Confidence:** confirmed.
- **Problems:**
  - Results are routed by the order of generation-guard arms in one large `match`. The new a-deck `Err` arm only works because it sits before the generic `Err`.
  - The a-deck parse-failure log line has no storm id.
  - `ParseOutcome.storm_id` (`adeck_fetch.rs:122`) is returned but only tests read it.
- **Fix:** log `outcome.storm_id`. Longer term, tag `ParseResult` with its job kind.

### L14. Guidance card is inlined in `main.rs`

- **Where:** `app/src/main.rs:2843-2967`.
- **Problem:** this roughly 130-line card lives in the already very large `main.rs`, while `model_legend` lives in `widgets/`.
- **Fix:** move it to `widgets/model_card.rs`.

### L15. Legend swatches don't match the map

- **Where:** `app/src/widgets/model_legend.rs:44-46` and `app/src/adeck_style.rs:48`, `:126`.
- **Confidence:** confirmed.
- **Problems:**
  - Swatches are 4 px thick, but model lines draw at 1.5 px and CARQ at 1.5 px (its swatch is 2 px), so the legend makes lines look more visible than they are.
  - "No intensity" `#9E9590` is nearly the same as CARQ `#B0B0B0`.
  - The OFCL swatch problem is covered in S4.
- **Fix:** draw the swatches at the real widths, and separate the two greys.

### L16. Track-end labels

- **Where:** `app/src/scope.rs:2077-2098`.
- **Confidence:** plausible.
- **Problems:**
  - Labels avoid each other but not the OFCL line or the legend.
  - A label is dropped only when its anchor is off-screen, so a label whose anchor is just inside the edge can be partly clipped.
  - Dense clusters drop most labels, which is intended.
- **Fix:** also avoid the legend rect, and clamp label boxes to the screen.

### L17. Comment inaccuracies

- **Where:**
  - `adeck_style.rs:49`: "Standard Saffir-Simpson scale colours", but the palette is custom. The thresholds are correct.
  - `adeck_fetch.rs:22` against `adeck.rs:129-132`: "350 KB" and "2.7 MB gzipped" disagree.
  - `adeck.rs:94-96`: see L7.
  - `adeck.rs` cap comment: "over 20×" headroom; see L5.
- **Fix:** correct the comments, using one measured figure for the file size.

### L18. Unlisted model codes fall to a hidden "Other" group

- **Where:** `app/src/adeck.rs:207-244` (`classify_tech`).
- **Confidence:** plausible.
- **Detail:**
  - Variants such as `HFA2`, `HFB2`, `HMN2`, `CTC2`, `EMXI`/`EGRI`, and `TCOA` become `ModelGroup::Other` and are hidden.
  - That's safe, but any new model worth showing by default must be added by hand.
  - CARQ and WRNG correctly map to Observed (WRNG hidden), so neither is drawn as a model.
- **Fix:** none required. Note the maintenance cost in a comment.

### L19. Test and fixture gaps

- **Where:** `app/src/testdata/aal092026_sample.dat` and the test modules.
- **Confidence:** confirmed.
- **Gaps:**
  - No tests for `widgets/model_legend.rs`, `scope::draw_model_tracks`, or the `main.rs` wiring. The toggle-off cases (stops fetching, clears the draw) are untested. Only the pure gate function is tested, and it lacks the NHC term (L3).
  - **Fixture rows** are cut to about 17 columns, so no wide OFCL or CARQ rows (up to 46 fields, with STORMNAME) go through `parse_adeck`.
  - **Missing fixture cases:**
    - a southern-hemisphere row;
    - a dateline track (only the projection is tested);
    - CRLF line endings;
    - duplicate rows with different TECHNUM;
    - ragged short rows;
    - OFCL missing from the newest cycle while models are present (beyond one injected AVNI line);
    - tau beyond 168.
  - **No tests for the new findings:** there's nothing for S1/S3 (cycle age, cycle regression), S2 (queue headroom), S6 (clock jump), or S7 (per-storm toast).
- **Fix:**
  - Add the fixture cases above.
  - Add wiring tests for the toggle and NHC gates.
  - Add regression tests alongside each fix in this review.

---

## Refuted claims

- **"Model tracks keep drawing when the Tropical (NHC) overlay is off, with no legend or disclaimer."** This was reported by the settings and tests reviewer and is **refuted**.
  - `main.rs:1827-1831` passes `nhc = None` when `show_nhc_data` is false.
  - `scope.rs:1603-1610` draws model tracks only inside `if let Some((bundle, overlays)) = nhc`.
  - So tracks and legend hide together. The real, smaller issue is that this gate is implicit, which is L3.

## Checked and found fine

- **Settings:**
  - `show_model_tracks` is off by default.
  - It has `#[serde(default)]`, so old `settings.json` files load and nothing is wiped. The test `deserializes_settings_without_location_fields` covers this.
  - Both toggles persist.
  - No new keyboard shortcut was added, so there's no collision with the double-bound W.
- **Dependencies:** `flate2 = "1"` uses the default pure-Rust miniz_oxide backend, already in the lock via `zip`, and is safe on Windows. The `typos.toml` testdata exclude is appropriate.
- **Parser safety:**
  - No byte slicing on untrusted text. `str::get` and `split` are used throughout.
  - `parse_init` requires exactly 10 ASCII digits.
  - `adeck_url` checks ASCII before checking length.
  - Rows with fewer than 9 fields are skipped.
  - `lines()` plus `trim` tolerate CRLF and padding.
  - `unwrap` and `expect` appear only in tests.
- **Format handling:**
  - Hemisphere signs and the lat ±90 / lon ±180 ranges are correct.
  - `0N 0W` becomes no position, kept for the intensity chart.
  - VMAX 0 becomes `None`.
  - Negative tau is kept only for Observed (CARQ/WRNG).
  - Wind-radii duplicates collapse.
  - Points are tau-sorted through the `BTreeMap`.
  - Tracks with fewer than 2 points are skipped.
  - RVCN, DRCL, and RI25 are skipped, and intensity-only techs never get tracks.
- **Default-visible set:** matches research §3 exactly.
- **URL mapping:** lowercased `a{id}.dat.gz`. Basins `al`/`ep`/`cp`; numbers 01–79 plus invests 90–99; 00 and training storms 80–89 are rejected. The year comes from the storm id, so there's no year-boundary bug.
- **Bounded gunzip:**
  - The 10 MiB compressed cap, the gzip magic check, and the streaming `take(max+1)` cap all hold.
  - A truncated stream returns an error.
  - Invalid UTF-8 is replaced instead of causing an error.
- **Scheduler:**
  - `cancel()` really frees the id.
  - `PlyNet::get` cancels before firing, and every exit path cancels.
  - Backoff runs 30 → 900 s, can't overflow (`1u32 << min(5)`, `saturating_add`), and resets on success.
  - A 404 isn't retried at full rate.
  - A hung storm doesn't block the others.
  - Net ids `nhc-adeck-{id}` don't collide with the existing NHC ids.
  - Storm-id changes (invest → numbered) and removal mid-parse are handled by generation.
  - Tests use an injected clock, a FakeNet that copies ply's ignore-repeat-id behaviour, and cover backoff, deadline, eviction, one toast per streak, and disable/enable.
- **Integration:**
  - Gunzip, parse, `latest_cycle`, `build_guidance`, and `classify_tech` all run on the worker.
  - The body is moved, not cloned, into the job.
  - `nhc.rs` is untouched, and the storm list comes only from the finished bundle.
  - Frame order is apply → tick → submit → draw.
  - A parse panic becomes `JoinError` → `Err` → backoff, so the shared worker survives (Windows review M17 doesn't apply here).
  - Startup with the overlay off costs nothing.
  - Demo mode shows live a-decks, as documented.
- **Projection and draw:**
  - The cache key `(site_index, revision)` doesn't depend on zoom or pan.
  - Each track is culled by bounding box before per-vertex mapping.
  - Only `default_visible` tracks are projected (about 8–15 per storm).
  - There are no dashed tiny segments.
  - The store keeps only the newest cycle per storm.
  - Indexing is safe (`windows(2)`, `get(i)`).
  - The dateline is handled through `range_bearing`, and there's no NaN path.
  - The model draw order puts models after radar and alerts and before the official cone and track.
- **Icon rule (AGENTS.md):**
  - The only rendered glyphs are `nf::HURRICANE` and `nf::EXTERNAL_LINK`, both existing codepoints drawn with `SYMBOL_FONT`.
  - The "…" (U+2026) is in Inter.
  - There's no emoji in the new code. The "✓" in `nhc_toggle_button` predates the branch.
- **Chart maths:**
  - `chart_max_tau` clamps to 24..=168, so there's no divide by zero.
  - An empty series returns early.
  - `plot_h` is at least 1.
  - With all VMAX missing, the chart is omitted.
- **Age formatting:** `format_age` and `is_stale` handle a negative age (clock skew) without panicking.
- **Plan vs. implementation:**
  - Tasks 1–17 are all present.
  - Fix commit `1013cba` diverged from the plan on purpose, and correctly: one shared gate for the cache rebuild and the draw.
  - Storing the generation in `Phase::Parsing { generation }` is equivalent to the plan's appendix suggestion.
  - The divergences are S11 (no line styles) and the plan's `Full` assumption (S2).
- **Plan's Windows-review constraints:** the cancel()-based retry, background gunzip and parse, the projection cache, and no byte slicing are all implemented as claimed. The deadline clock is still `get_time()` (S6).

## Known limitations (dev-reported)

- **Parse-queue saturation with 5+ storms:** confirmed and assessed as S2.
- **Live network behaviour not verified:** the dev machine's TLS inspection breaks rustls (Windows review H1). Every network finding here (S6, S7, S9, L4, L11) comes from reading the code and the ply and ureq source. None was observed against live NHC servers.

## Suggested remediation order

1. **S1 + S3 + S8 + S5: freshness honesty.**
   - Show cycle age, and make "(stale)" and the hide threshold depend on it.
   - Refuse cycle regressions.
   - Handle OFCL lag and post-tropical pinning.
   - Show a per-storm cycle in the legend.
   - These are the user-safety items and share one data change: a parsed UTC `init`.
2. **S2: parse-queue headroom.** A one-line change (`!has_parsing()`) that stops a-decks from starving the official NHC, alerts, and borders feeds.
3. **S4 + L1: official vs. model clarity.**
   - Show one official track.
   - Add the GIS track to the legend with a non-category colour.
   - Give OFCL a dark outline.
   - Put attribution, "No official forecast", and " (stale)" text in the legend.
4. **S6 + S7 + L4: scheduler robustness.**
   - Use a monotonic clock and poll before the deadline.
   - Keep the toast streak per slot.
   - Make "no rows" and gunzip errors quiet, detect 404 by status, and use a separate 404 cadence.
5. **S9 + L2: network economy.** Skip unchanged inserts, add jitter, keep slot state across toggles, and clear or prune guidance on disable.
6. **S10 + S11 + L15: legend layout and line styles.**
7. **L5–L8: input hardening.** Avoid the extra UTF-8 copy, cap tau, tighten lon validation, and use `MultiGzDecoder`.
8. **L19: tests.** Add fixture cases and wiring tests, plus regression tests for steps 1–7.
9. **Remaining Low items** as capacity allows: L3, L9–L14, L16–L18.
