# Spaghetti-models overlay, re-review 2

**Date:** 2026-10-19
**Range:** `5ff944f..719483a` (14 commits, not yet pushed)
**Worktree:** `rustywx-spaghetti`
**Earlier reviews:** `docs/reviews/spaghetti-models-review.md`, `docs/reviews/spaghetti-models-rereview-1.md`

This review checks:

- the fixes for the re-review 1 findings that dev addressed: N1, N2/N3/N7, N4, N5/N6, S11/L1, N11 (plus N17, which was fixed incidentally);
- L5–L8 and L19;
- the leftover-diff commit `9a35264`;
- the CI-fix batch: `d9d8fb6`, `611c580`, `4d5c491`, `b060520`;
- whether dev's deferrals are reasonable.

## Method

- **Team:** 4 read-only reviewers (2 Opus, 2 Sonnet), each owning one group:
  - GIS bin-number join and freshness gate: N2, N3, N7
  - draw clipping and legend: N1, N17, S10, S11, N5/N6, N12, N14, L1
  - scheduler and parser: N4, N11, L5–L8, L19, plus the deferrals
  - CI and dependency batch, with independent re-runs of every check
- **Version:** all code was read at `719483a` with `git show`.
- **Verification:** the lead re-read the source for every item marked ✔.
- **Live data:** NHC's live data was fetched read-only, twice. One fetch was by a reviewer. The other was by the lead, to confirm the `advdate` format behind R2-1. These public endpoints were the only network access.
- **Text-fit numbers:** one reviewer measured text widths from the bundled fonts' metrics. Every text-fit number below is an estimate from those measurements, not an observed render.

## Build and check results

These ran on an isolated `git archive` copy of `719483a`, with its own `CARGO_TARGET_DIR`. Toolchain: rustc 1.99.1 (2026-10-13).

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | pass |
| `cargo test --all-features` | pass: 416 unit + 2 bin, 0 failed |
| `typos --config typos.toml` (1.48.0) | pass |
| `cargo deny check` | pass (advisories, bans, licenses, sources) |
| `cargo audit` with CI's 4 ignores | clean |
| `cargo audit --ignore RUSTSEC-2026-0192` (Justfile and pre-commit) | exit 0, but prints 3 unmaintained warnings (R2-12) |

These results match the team lead's local runs. The `just run` smoke test was not repeated; the team lead reports it passed.

CI comparison:

- CI runs the same commands on unpinned `stable`. That explains why the clippy and typos failures appeared there.
- Every CI job is Linux. There's still no Windows or macOS job (Windows review H7).
- The `build` job has no `--locked`.

## Merge recommendation

**Not yet. It's ready to merge after 4 small fixes (R2-1 to R2-4).** Each is a few lines, and none needs redesign.

1. **R2-1 (N7):** the advisory freshness gate never fires on real NHC data. When its parse fails, it hides OFCL. Re-review 1 marked N7 as fixed, but it isn't.
2. **R2-2 (N2 remainder):** the name-substring layer merge still runs whenever a feature has a bin that matches no cone entry. NHC updates its layers independently, so this happens in practice.
3. **R2-3:** the new 200k record cap silently drops the *newest* rows of a large a-deck, which freezes guidance on an old cycle.
4. **R2-4 (N13):** the Tropical card says "No NHC official forecast this cycle" while the NHC advisory track is on the map.

Everything else is Low or a nit and can follow the merge. All of dev's deferrals are reasonable, except N13, which is R2-4 here.

## Verdict summary

| ID | Verdict | Notes |
|---|---|---|
| N1 | **Fixed** ✔ | Each segment is clipped (Liang–Barsky) to the screen plus 50 px *before* dashing, and the dash phase advances in O(1) across the clipped parts. Worst case is about 23k–38k `draw_line` calls per frame for 3–5 storms, down from 100k–800k, and no longer depends on zoom (estimate). No test covers the `scope.rs` wiring (R2-9) |
| N17 | **Fixed** | `dash_steps` returns early on non-finite input and clamps length to 1e6. The caller skips non-finite segments |
| N2 | **Partially fixed** ✔ | `binnumber` is parsed and merged on in all 4 layers, and suppression is matched by bin. But `find_storm_mut` still falls back to `name.contains` whenever the bin is non-empty and unmatched (R2-2) |
| N3 | **Fixed** | Matching is bin-only when both sides have a bin, which is the live case, so a rename or naming mismatch no longer breaks it. Only an empty `binNumber` falls back to the exact-name rule |
| N7 | **Not fixed** ✔ | The gate parses `advdate` as `%Y-%m-%d`. Live NHC sends `"200 PM MST Mon Oct 19 2026"`, so the parse always fails, the gate always returns "not stale", and OFCL is suppressed (R2-1) |
| N4 | **Fixed** | `take_parse_job` returns `None` while `has_parsing()`. Test: `take_parse_job_refuses_a_second_job_while_one_is_already_parsing` |
| N11 | **Partially fixed** | A 404 no longer adds to `failures`, so the next real failure backs off 30 s (tested). But a 404 doesn't clear *earlier* failure or toast-streak state (R2-7), and the one-toast-per-outage half wasn't done |
| N5 | **Fixed** | Compact is chosen on 400–513 px desktop windows, and its bottom edge clears the bar by about 20 px (estimate). The test fails if `9a35264` is reverted |
| N6 | **Partially fixed** | Single-row bar heights are right (40 desktop / 60 mobile). Wrapping is still ignored, and the bar wraps on desktop at 900–940 px widths (R2-6). The padding literal isn't tied to the constant, and nothing is tested |
| S10 | **Fixed** (small leftover via N6) | The trigger uses the same inputs as `legend_y`. The real Full height, about 401–416 px, fits within the 420 estimate |
| S11 | **Fixed** | The N1 regression is resolved, and widths now vary by group: 2.0 / 1.75 / 1.5 / 1.25 / 1.0, all under OFCL's 2.5 |
| N14 | **Partially fixed** | Width varies now. The legend still previews every group at 2 px, and the GIS violet track has no underlay |
| L1 | **Mostly fixed** | The caveat and "Source: NOAA/NHC ATCF" now show in both legend modes. The caveat line overflows the panel width (R2-10), and the fit tests still count characters |
| N12 | **Partially fixed** | The source line is kept in Compact, and the 160 px height estimate holds (about 139–144 px, estimate). The GIS swatch and a dash note are still Full-only |
| L5 | **Partially fixed** ✔ | The extra UTF-8 copy is gone, the cap comment is accurate, and a record cap was added. The record cap introduces R2-3. `tech` is still a heap `String` per row |
| L6 | **Fixed** | Tau is range-checked to −240..=384 at parse time, and `chart_max_tau` uses `saturating_add`. Both are tested |
| L7 | **Partially fixed** | East >1800 is rejected and digits-only is enforced. An empty or non-numeric VMAX still drops the row (real files write 0, so about nil impact) |
| L8 | **Fixed** | `MultiGzDecoder`, still bounded by `take(max+1)` across all members. Test: `gunzip_decodes_every_member_of_a_multi_member_stream` |
| L19 | **Partially fixed** | Added: S hemisphere, wide row with STORMNAME, CRLF, TECHNUM. Still missing: dateline, ragged rows, OFCL-missing-from-newest, and wiring tests. The CRLF test can't fail; the TECHNUM test pins L9's first-row-wins behaviour |
| `9a35264` | OK | Unifies the Compact/Full trigger with `legend_y`'s clamp; it's the N5 fix above |
| `d9d8fb6` | OK | The clippy `question_mark` rewrite of `parse_lat`/`parse_lon` doesn't change behaviour |
| `611c580` | OK (nit R2-13) | Only comments, error strings and test names change; no serde, settings or JSON key is touched. The `docs/reviews/` and `docs/superpowers/plans/` excludes cover internal, unshipped docs, but they don't seem needed |
| `4d5c491` | OK (notes R2-11, R2-12) ✔ | zip 2 → 9: one call site; the now-fallible `name()` is handled; no new C dependency, and fewer C crates overall. quinn-proto, rustls, rustls-webpki and the other bumps are lockfile-only and semver-compatible. The 3 advisory ignores are scoped by ID, unmaintained-only, Linux-only, and both necessary and sufficient |
| `b060520` | OK | The Support links point to `/issues`, which matches the existing bug-report link |

---

## Findings (must fix before merge)

### R2-1. Medium (safety): the N7 advisory-freshness gate is dead on real data and fails toward hiding OFCL ✔

**Where:**

- `app/src/adeck_style.rs:684-695` (`gis_advisory_not_known_stale`)
- input: `app/src/nhc.rs:811` (`advisory_date: prop_str(feature, "advdate")`)

**Evidence:**

```rust
let Ok(adv_date) = chrono::NaiveDate::parse_from_str(advisory_date.trim(), "%Y-%m-%d") else {
    return true;   // "not known stale" → suppression proceeds
};
```

Live NHC MapServer layer 6 (the track), fetched 2026-10-19 by the lead:

```
"advdate":"200 PM MST Mon Oct 19 2026", "advisnum":"26", "idp_filedate":1792443770000
```

Every test fixture uses a made-up `"2026-08-01"`-style date (`nhc.rs:1641`, `:1767`, `:1798`, `:1802`, `:1840`; `adeck_style.rs:~2054-2097`).

**Failure scenario:**

1. The parse fails on every real advisory, so the gate always passes, and OFCL suppression behaves exactly as it did before the "fix".
2. The scenario N7 describes really happens. During the reviewer's earlier fetch, the GIS cone, track and points layers were on advisory 25 (15Z) while CurrentStorms was already on advisory 26 (21Z). The watch/warning layer was 4 advisories behind.
3. A stale bundle that's kept after a failed fetch (`main.rs:1520-1523`) leaves the same gap.
4. In both cases a newer a-deck OFCL is hidden behind the older violet advisory track.
5. The test `..._fails_open_on_an_unparsable_advisory_date` locks in the unsafe direction.

**Even if the date parsed, the design would be wrong:**

- Comparing calendar days can't see a lag of 1–3 advisories within the same day, which is the common case.
- `advdate` is local time ("MST", "EDT", "HST"). The advisory for the 00Z cycle goes out around 03Z, which is the previous local evening. A date-only comparison would therefore mark the *correct* advisory stale for every 00Z cycle and bring back two official tracks.

**Fix:**

- Parse `idp_filedate` into `StormGis`. It's an `esriFieldTypeDate`, epoch ms, so read it with `as_i64` or `as_f64`. `prop_str` returns `""` for numbers.
- Suppress only when `adv_issue_utc >= ofcl_init_utc`. The advisory for cycle T is issued about T+3 h and the one before it about T−3 h, so this comparison separates them cleanly.
- If the time is missing or can't be parsed, **show both tracks**: don't suppress, and keep the legend label honest.
- Optionally, also stop suppressing once the bundle's fetch age passes a threshold.
- Replace the fixtures with real `advdate` and `idp_filedate` values.

### R2-2. Low–Medium (safety): the substring layer merge still runs for a non-empty, unmatched bin ✔

**Where:** `app/src/nhc.rs:772-785` (`find_storm_mut`). The doc at `nhc.rs:104-113` claims the fallback runs only when the bin is empty.

**Evidence:**

```rust
if !bin_number.is_empty() && let Some(pos) = storms.iter().position(|s| ... eq_ignore_ascii_case(bin_number)) {
    return storms.get_mut(pos);
}
storms.iter_mut().find(|s| name.contains(&s.name))   // reached for ANY unmatched bin
```

**Failure scenario:**

1. NHC's four layers update independently; the live data showed a 4-advisory gap.
2. So a storm can show up in the track layer while it's absent from the cone layer.
3. If its bare name contains a cone storm's name (Sixteen-E/Six, Fourteen-E/Four, Twenty-One/One), its track overwrites that storm's track (`nhc.rs:827`, last write wins).
4. The other storm's real GIS track disappears, and its OFCL is suppressed in favour of a line somewhere else. That's N2's original failure.

**Fix:**

- When the feature's bin is non-empty, match by bin only. Fall back to substring only when the feature has no bin, or only against cone entries with an empty bin.
- Regression test: cone `[Six/AT2]`, track `[Sixteen-E/EP3]`; assert that Six's track stays empty.
- Add bin-merge tests for the points and watches layers. Neither has one, so reverting either change fails nothing.

### R2-3. Medium: `MAX_ADECK_RECORDS` keeps the oldest rows and silently drops the newest cycles ✔

**Where:** `app/src/adeck.rs:69-73` (`parse_adeck` … `.take(MAX_ADECK_RECORDS)`), `:189-192` (`MAX_ADECK_RECORDS = 200_000`).

**Evidence:**

- a-decks are append-only and chronological, so `take(200_000)` keeps the start of the file.
- The cap's comment assumes "about 30 bytes per source line". Real rows are about 134 B (fixture average) to about 200 B (wide rows).
- At the largest live file seen (about 22 MB inflated), that's about 110k–164k rows, only 1.2–1.8× headroom (estimate).
- This cap is reached *before* the 64 MiB byte cap.

**Failure scenario:**

1. A long-lived or heavily modelled storm passes 200k rows.
2. `parse_adeck` returns only the old rows, with no error or log.
3. `latest_cycle` picks an old cycle, and guidance freezes.
4. Cycle-age dimming (S1) eventually marks it stale, but the cause is invisible.

This is the L5 failure mode ("outgrows the cap, guidance freezes") in a new place.

**Fix:**

- Make hitting the cap an `Err`, so the existing quiet parse-failure path and its status apply. Or keep a rolling window of the *newest* N rows, or drop rows older than the newest few cycles while streaming.
- Correct the 30-byte figure in the comment.
- Consider raising the cap (about 1M rows is about 90 MB worst case).
- Log when the cap is hit.

### R2-4. Medium (safety, was N13): the Tropical card says there's no official forecast while the NHC advisory track is drawn ✔

**Where:** `app/src/main.rs:2987-2991`.

**Evidence:** the card shows `if !g.has_official { ui.text("No NHC official forecast this cycle", ...) }`. That happens whenever the a-deck has no OFCL in the selected cycle, including the S8 fallback case, even while the violet advisory track is on the map.

**Failure scenario:** a user who reads the card could discount the real official track on the map. The legend was already reworded to avoid exactly this claim, so the card and legend now disagree.

**Fix:** compute `OfficialLegendState` per storm and reuse `official_legend_label`, or say "No OFCL in the model file this cycle; see the NHC advisory track" when the GIS track is shown.

---

## Findings (fine to defer past merge)

### R2-5. Low (safety): bin reuse across CurrentStorms and MapServer snapshots

**Where:** `adeck_style.rs:749-750`.

**Problem:**

- `gis_track_shown_for` compares CurrentStorms metadata against MapServer layers. Those are two services that lag each other, not one snapshot.
- Bin numbers are recycled within a season. If storm A (AT2) ends and storm B is assigned AT2 while MapServer still shows A, B's OFCL is suppressed by A's dead track.

**Fix:** also compare `stormnum` and basin (present on every layer; `21` live) against digits 2..4 of `meta.id`, read with `str::get`.

**Confidence:** medium; the timing of bin reassignment wasn't verified.

### R2-6. Low–Medium (layout): the top-bar estimate ignores wrapping, and the bar wraps on desktop

**Where:** `app/src/main.rs:120-143`, `:3263`. The literal `.padding(8)` is at `main.rs:1966`.

**Evidence (estimate):** with every layer on plus count badges, the bar measures about 936 px against the 900 px default window width (`main.rs:625`).

**Failure scenarios:**

- At 900–940 px wide, the desktop bar wraps to 2 rows, about 70 px tall against the 40 px estimate. Windows that are also 514–544 px tall get Full mode, and the legend covers the second row of buttons.
- On mobile the bar runs 2–4 rows (110–210 px) against the 60 px estimate.

**Fix:**

- Use the bar's measured height from last frame (ply bounding box), or lay the legend out inside the map area.
- Wire `CONTROLS_BAR_PADDING_PX` into `.padding()`.
- Test `controls_bar_height_estimate`.

### R2-7. Low: a 404 doesn't clear earlier failure or toast state

**Where:** `app/src/adeck_fetch.rs:258-270`, `:444`.

**Scenario:** 3 real failures, then hours of 404, then a transport error. That gives a 240 s backoff instead of 30 s, and no toast, because the earlier streak flag is still set.

**Fix:** on `not_found`, reset `failures = 0` and `toast_shown_in_streak = false`, since a 404 proves the server is reachable.

### R2-8. Low: a slot stuck in `Parsing` has no timeout

**Where:** `adeck_fetch.rs:226`, `:403`.

**Problem:** this predates the branch. If a result were ever lost, `has_parsing()` would stay true forever and block every storm. Today the worker always sends a result, and a `JoinError` is mapped to `Err`, so nothing reaches this state.

**Fix:** add a watchdog that fails a `Parsing` slot after about 60 s.

### R2-9. Low (tests): wiring is untested for N1, N6 and L1

- **N1:** the clip and phase composition lives inside `draw_model_tracks` (`scope.rs:2149-2190`). Reverting to unclipped dashing, or dropping the `clip_start` phase advance (which would make dashes shimmer while panning), passes every test.
  - **Fix:** extract `clipped_dash_spans(p0, p1, rect, pattern, phase)` and test it against unclipped `dash_steps` restricted to the rect, plus a fully off-screen segment that emits 0 spans and the same phase.
- **N6:** `controls_bar_height_estimate` has no test, and the legend test hard-codes `40.0`.
- **L1:** the caveat and the Compact attribution are inline strings with no test.
- **N2:** `gis_track_shown_for_bin_number_rejects_a_substring_false_positive` passes on revert, because the old exact-name rule already rejected that case.
- **Fixtures:** they use unrealistic bins (`AT14`, `AT15`, `EP14`). Real bins run 1–5 per basin.

### R2-10. Low: the caveat line is wider than the legend panel

**Where:** `app/src/widgets/model_legend.rs:238`.

**Problem:** "May differ from official NHC products." measures about 181 px in Inter and about 204 px in OpenDyslexic at 10 px (estimate), against 180 px of content width. It always wraps, leaving "products." alone on a second line. At 38 characters it's also over the code's own 32-character legend budget.

**Fix:** shorten it to something like "May differ from NHC products.", move it to a const, and make the fit tests measure width.

### R2-11. Low: zip is pulled in with all default features, and KML reading is still unbounded

**Where:** `app/Cargo.toml` (`zip = "9"`, no `features`), `app/src/nhc.rs:509-512`.

**Notes:**

- The defaults bring in aes-crypto, bzip2, deflate64, lzma, ppmd, xz and zstd (C, via cc). Only `deflate` is needed for KMZ. That isn't a regression: zip 2's defaults included zstd-sys too, and zip 9 removed bzip2-sys, lzma-sys and xz2.
- `read_to_string` on the KML entry still has no size cap, and the `.kml` match is still case-sensitive. Both predate the branch (Windows review M20), but they sit on the code this commit touched.

**Fix:**

- Set `default-features = false, features = ["deflate"]`, then re-run `cargo deny`.
- Read with `file.take(MAX_KML_BYTES)`.
- Match `.kml` case-insensitively.

### R2-12. Low: the advisory-ignore justification overstates the scope, and local audit drifts from CI

**Where:** `deny.toml:17-24`, commit message `4d5c491`, `Justfile:38-41`, `.pre-commit-config.yaml:85`.

**Problems:**

- The commit message's "not reachable on the platforms this app ships for" is wrong for Linux. The AppImage release and the Wayland smoke job ship and run `accesskit_unix` → `zbus`. The `deny.toml` comment, "not reachable on the macOS/Windows builds", is accurate.
- `just audit` and the pre-commit hook still ignore only 0192, so they print 3 warnings that CI suppresses.

**Fix:**

- Reword the comment to "Linux-only (AT-SPI accessibility); unmaintained-status advisories, no known vulnerability".
- Add 0370, 0371 and 0372 to the Justfile and pre-commit ignores.

### R2-13. Nits

- **Typos config:**
  - `typos.toml`: the `docs/reviews/` and `docs/superpowers/plans/` excludes look unnecessary; typos 1.48.0 passes without them. Drop them unless CI's typos version flags something.
  - The `unparseable` identifier allowance is now obsolete.
- **Width tiers:** the 0.25 px steps are subtle.
  - Global is now 1.0 px dots of 1.5 px length, the faintest marks on the map, and it covers 4 default-visible tracks.
  - Ensemble 5/5 and AI 3/3 still share a 1:1 rhythm.
  - Consider bigger steps, 2 px Global dots, and a non-1:1 AI pattern.
- **Allocations:** `dash_steps` still allocates a `Vec` for each visible segment, and `cap_dash_spans` runs after allocation, so it bounds draw calls but not CPU or memory. Use a reused buffer or an iterator.
- **Clip NaN check:** `clip_segment_to_rect` returns `Some((0,1))` for NaN input. That's safe only because the caller filters non-finite input first. Check for non-finite input up front and add tests for vertical, parallel-outside and degenerate segments.
- **Stale or misplaced docs:**
  - `MAX_DASH_STEPS_LENGTH_PX` sits between `dash_steps`'s doc block and the function, so rustdoc attaches the doc to the const (`adeck_style.rs:152-177`).
  - `legend_y`'s doc still says "36px" (`model_legend.rs:92`).
  - `LegendMode::Compact`'s doc still says the source line is dropped.
  - The `fetched_at_unix` doc still says "updated N min ago".
- **Bin trimming:** bins are compared without `trim()` (`nhc.rs:780`, `adeck_style.rs:750`). Names are trimmed.
- **MultiGzDecoder and trailing padding:** not verified. Add a test with trailing zero bytes.
- **Fixture realism:** SUBREGION "AL1" (real values are one letter), and MILTON in a 2026 cycle.
- **CI:** the `build` job has no `--locked`.
- **Pre-existing glyph:** the controls bar's "✓" renders in the body font, and OpenDyslexic has no U+2713 glyph.

---

## Deferral assessment

| ID | Defer? | Reason |
|---|---|---|
| N8 | Yes | Backoff is at most 300 s and 404 retries at most 600 s, so recovery after an outage is bounded |
| N9 | Yes | Poll-first already stops finished downloads from being thrown away. What's left is a possible false toast after sleep |
| N10 (jitter) | Yes | `next_due: now` for new slots is harmless with about 5 storms; refresh jitter exists, and the parse cap serializes the work |
| N12 | Yes | Mostly fixed (source line kept, estimate holds). The Compact GIS swatch and dash note are cosmetic |
| **N13** | **No: fix before merge** | This is R2-4. The card contradicts the map, and the fix is small. The colour-only "Checked" leftover can be deferred, because the cycle line carries "(stale)" text |
| N14 | Yes | The width half is done. The violet underlay is cosmetic |
| N15 | Yes | Understated age needs a clock about 12 h slow. A fast clock only over-dims. The misleading error text is minor |
| N16 | Yes (ticket) | A rejected older cycle doesn't bump `fetched_at_unix`, so "Checked … (stale)" and the cycle age both show it. The fallback-to-CARQ-only-cycle drop-out is worth a ticket |
| N17 | Already fixed | Fixed in `15065ae` |
| N18 | Yes | Nits |

## Checked and found fine

- **N1 clipping:**
  - The Liang–Barsky math is correct for: fully inside, fully outside, crossing one edge, crossing two edges, vertical and horizontal lines, and a corner touch. Zero-length segments are filtered by the caller.
  - The clip rect is in logical px, the same units as `draw_line`.
  - The dash phase stays continuous across clipped and culled segments, with no shimmer while panning (f32 error is under 0.01 px at realistic lengths).
  - Solid lines, the OFCL underlay and the GIS track each draw one line per segment, so they're bounded.
- **Bin parsing:** `binnumber` is a string in MapServer layers 5–8 and matches CurrentStorms `binNumber` exactly (live: `EP1`). A missing or null value becomes `""` without panicking.
- **Wind radii:** the layer isn't fetched. Wind-probability and arrival KMZs come from per-meta URLs, so there's no name join there.
- **Existing NHC overlay:** cone, track and points drawing is unchanged. No layer is dropped for storms that lack a bin, because they fall back as before.
- **Legend and suppression gate:** they use the same cycle time (`cycle_time_unix(g.init)`), so they agree. They inherit R2-1.
- **N4:** the cap can't get stuck. The generation counter only advances when a job is taken, and a stale or removed slot is disowned.
- **Parser:**
  - 404 detection still uses the exact ureq format.
  - Tau boundaries are correct: −240 and 384 are accepted, −241 and 385 are rejected.
  - `1800E`, `1800W` and `0N 0W` are handled correctly.
  - The compressed-size check, `take(max+1)` and the post-read length check are all intact.
- **Legend layout:**
  - Compact on 400–513 px desktop windows clears the bottom bar.
  - High DPI is handled consistently (logical px throughout).
  - OpenDyslexic: Full is about 416 px against the 420 estimate (tight), and Compact about 144 px against 160 (estimates).
- **Panics, slicing and glyphs:** no new panics, unwraps or byte slicing. No new emoji. The only icon is `nf::HURRICANE` via `SYMBOL_FONT`.
- **CI batch:**
  - The clippy rewrite doesn't change behaviour.
  - The typo fixes don't touch any serde, settings or JSON key.
  - The zip 9 `name()` handling is correct, because the CP437 fallback makes it effectively infallible. There's no new C dependency and no license problem.
  - The other dependency bumps are semver-compatible and lockfile-only.
  - The advisory IDs map to the right crates, are scoped, unmaintained-only and Linux-only, and are confirmed both necessary and sufficient.
  - The README link is correct.
- **Serde:** the new `StormGis.bin_number` has no `#[serde(default)]`, which is fine, because `NhcBundle` is never persisted.

## Suggested next steps

1. **Before merge:** fix R2-1 (`idp_filedate` gate that shows both tracks when the time is unknown), R2-2 (bin-only merge when a bin exists), R2-3 (record cap errors or keeps the newest rows), and R2-4 (card reuses `OfficialLegendState`). Add a regression test for each, using real NHC field formats.
2. **Right after merge:** R2-6, R2-7, R2-9 and R2-11, plus the Justfile and pre-commit ignore sync from R2-12.
3. **Backlog:** R2-5, R2-8, R2-10, R2-13, and the remaining deferrals.

---

## Follow-up issues to file after merge

These are drafts, ready to paste as GitHub issues. R2-1 to R2-4, R2-11 and R2-12 are left out because they're being fixed before merge. Suggested labels are in brackets.

**1. Tropical: cross-check `stormnum` when matching NHC GIS storms to CurrentStorms** [bug, tropical] (R2-5)
OFCL suppression matches CurrentStorms metadata to MapServer GIS storms by bin number only. Bins are recycled within a season, and the two services lag each other. A new storm that gets a bin while MapServer still shows the old storm in it could have its a-deck OFCL hidden behind the dead storm's track. Also compare `stormnum` and basin (present on every GIS layer) against digits 2..4 of `meta.id`.

**2. Tropical legend: base the top-bar offset on the controls bar's real height, including wrapping** [bug, ui] (R2-6)
`controls_bar_height_estimate` assumes a single-row bar (40 px desktop, 60 px mobile). The bar uses `.wrap()` and wraps on desktop at 900–940 px widths with every layer on, and to 2–4 rows on mobile, so the legend can cover the second row of buttons. Use the bar's measured height from last frame, or place the legend inside the map area. Wire `CONTROLS_BAR_PADDING_PX` into the bar's `.padding()` and add a test.

**3. A-deck fetch: clear earlier failure and toast state when a 404 arrives** [bug, tropical] (R2-7)
A 404 proves the server is reachable, but `record_failure(.., not_found = true)` keeps the earlier `failures` count and `toast_shown_in_streak`. After real failures followed by hours of 404s, the next transport error backs off up to 240 s instead of 30 s, and it doesn't toast. Reset both on `not_found`, and add a test that starts from a dirty history.

**4. A-deck fetch: add a watchdog for slots stuck in `Parsing`** [hardening, tropical] (R2-8)
`take_parse_job` now refuses to hand out work while any slot is in `Parsing`. If a parse result were ever lost, every storm's model guidance would stop updating with no error. The worker always replies today, so this is defensive. Fail a slot that has been in `Parsing` for more than about 60 s back to the normal retry path.

**5. Tests: cover the wiring for dash clipping, legend layout, and GIS bin merge** [tests] (R2-9)
- Extract the clip and dash-phase composition from `scope::draw_model_tracks` into a pure function. Test it against unclipped `dash_steps` restricted to the rect, including a fully off-screen segment.
- Test `controls_bar_height_estimate`, and move the legend caveat and attribution strings into consts with tests.
- Add bin-merge tests for the points and watches layers.
- Fix `gis_track_shown_for_bin_number_rejects_a_substring_false_positive`, which passes on revert.
- Use realistic bin fixtures (1–5 per basin).

**6. Legend: measure text fit by width and shorten the caveat line** [ui] (R2-10)
"May differ from official NHC products." is about 181–204 px against 180 px of content width (estimate), so it always wraps with "products." on its own line. The fit tests only count characters. Shorten it to something like "May differ from NHC products.", and switch the legend fit tests to measured width (Inter and OpenDyslexic).

**7. Tropical: polish from the re-review nits** [cleanup] (R2-13)
- Stale or misplaced docs: `MAX_DASH_STEPS_LENGTH_PX` captures the `dash_steps` doc block, and the docs for `legend_y` ("36px"), `LegendMode::Compact` and `fetched_at_unix` are out of date.
- `trim()` bin numbers before comparing them.
- Make `clip_segment_to_rect` reject non-finite input itself.
- Reuse a span buffer in `dash_steps` (it allocates once per segment per frame).
- Add a `MultiGzDecoder` test with trailing zero padding.
- Fixture realism: SUBREGION is one letter, and use a 2026 storm name.
- Remove the unneeded `docs/reviews/` and `docs/superpowers/plans/` typos excludes and the obsolete `unparseable` allowance.
- Add `--locked` to CI's build job.
- Accessibility: widen the line-width steps between model groups, give Global dots more length, and use a rhythm other than 1:1 for AI.

**8. Tropical: handle rejected or disappearing a-deck cycles** [bug, tropical] (N16, deferred)
A server that returns an older cycle is silently rejected and counted as a successful fetch, so the only visible sign is "Checked … (stale)". Also, the S8 fallback can switch to a newest cycle that so far holds only CARQ or intensity rows, which drops the storm's tracks from the map until that cycle fills in. Show an "older cycle returned" status, and require at least one default-visible track before switching cycles.
