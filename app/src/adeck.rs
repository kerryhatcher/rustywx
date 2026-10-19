//! NHC ATCF a-deck ("aid") parsing for the optional spaghetti-models overlay:
//! model-guidance tracks and intensities. Pure functions only (no network, no
//! drawing). Record format: `docs/research/spaghetti-models.md` section 2.

use anyhow::{Result, anyhow, bail};
use std::collections::BTreeMap;

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

/// Sane bound on a row's forecast hour. ATCF forecasts run out to 168 h
/// (7 days) and CARQ past positions go back about 24-36 h; -240..=384
/// covers that with generous slack either way while rejecting the kind of
/// garbage tau (e.g. `i32::MAX`) that would otherwise overflow
/// `adeck_style::chart_max_tau`'s `max + 23` in debug and test builds
/// (review L6).
const TAU_RANGE_H: std::ops::RangeInclusive<i32> = -240..=384;

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
    if !TAU_RANGE_H.contains(&tau_h) {
        return None;
    }
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

/// Parse every well-formed line; malformed lines are skipped. Stops at
/// `MAX_ADECK_RECORDS` rows regardless of how much more input remains
/// (review L5).
pub fn parse_adeck(text: &str) -> Vec<ADeckRecord> {
    text.lines()
        .filter_map(parse_adeck_line)
        .take(MAX_ADECK_RECORDS)
        .collect()
}

fn parse_init(s: &str) -> Option<u32> {
    if s.len() != 10 || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

/// All-digit tenths, e.g. `"270"` -> 27.0. Rejects anything `u32::from_str`
/// would accept but a real ATCF field never contains, such as a leading
/// `+` (review L7: `parse_init` already requires all-digit input; this
/// brings `parse_lat`/`parse_lon` in line with it).
fn parse_tenths(digits: &str) -> Option<f64> {
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
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

/// `"876W"` -> -87.6, `"1795E"` -> 179.5. ATCF stores longitude as 0-1800
/// tenths (0-180.0°) plus E/W; a value past that is malformed, not an
/// alternate 0-360° east convention. A previous version folded anything
/// over 180° by subtracting 360 once, so a corrupt row like `"3700E"`
/// (370.0° raw) silently became a plausible-looking 10.0°E and could jump
/// a track across the ocean instead of being rejected (review L7).
pub fn parse_lon(s: &str) -> Option<f64> {
    let lon = if let Some(d) = s.strip_suffix('W') {
        -parse_tenths(d)?
    } else if let Some(d) = s.strip_suffix('E') {
        parse_tenths(d)?
    } else {
        return None;
    };
    (-180.0..=180.0).contains(&lon).then_some(lon)
}

/// Split `init` (`YYYYMMDDHH`) into its (year, month, day, hour) digit
/// groups by arithmetic, never slicing. Out-of-range groups (bad input) are
/// returned unvalidated; `cycle_time_unix` checks them via `chrono`, and
/// `adeck_style::format_cycle` falls back to "???" for a bad month.
pub(crate) fn cycle_digits(init: u32) -> (i32, u32, u32, u32) {
    let hour = init % 100;
    let day = (init / 100) % 100;
    let month = (init / 10_000) % 100;
    let year = (init / 1_000_000) as i32;
    (year, month, day, hour)
}

/// Parse `init` (`YYYYMMDDHH`, UTC) into a unix timestamp. `None` for an
/// impossible month, day, or hour — `chrono`'s `from_ymd_opt`/`from_hms_opt`
/// validate and return `None` rather than panicking or wrapping. Lets the
/// overlay show forecast-cycle age instead of fetch age (review S1) and
/// measure OFCL lag by elapsed time rather than synoptic-cycle counting
/// (review S8).
pub fn cycle_time_unix(init: u32) -> Option<i64> {
    let (year, month, day, hour) = cycle_digits(init);
    let date = chrono::NaiveDate::from_ymd_opt(year, month, day)?;
    let time = chrono::NaiveTime::from_hms_opt(hour, 0, 0)?;
    Some(date.and_time(time).and_utc().timestamp())
}

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

/// Largest compressed body accepted. Ply's ureq backend already refuses
/// bodies over 10 MiB (`ureq` `MAX_BODY_SIZE`). The biggest live a-deck seen
/// was 2.7 MB gzipped, so real files never hit this.
pub const MAX_GZIP_BYTES: usize = 10 * 1024 * 1024;

/// Uncompressed cap. The largest live a-deck seen is about 2.7 MB gzipped,
/// about 22 MB inflated at the observed ~8.3x ratio, so 64 MiB leaves about
/// 3x headroom over that real worst case — not the ">20x" an earlier
/// version of this comment claimed off a since-superseded 350 KB sample
/// (review L5). `MAX_ADECK_RECORDS` below additionally bounds the parsed
/// `Vec`'s memory, since a crafted gzip that actually inflates to this cap
/// would otherwise still be able to push it far past the text buffer's own
/// size.
pub const MAX_UNCOMPRESSED_BYTES: u64 = 64 * 1024 * 1024;

/// Hard cap on parsed rows, independent of `MAX_UNCOMPRESSED_BYTES`. Each
/// `ADeckRecord` costs roughly 90 bytes against about 30 bytes per source
/// line, so a crafted gzip that inflates all the way to the 64 MiB text cap
/// could still push `parse_adeck`'s `Vec` to a couple hundred MB. No real
/// a-deck comes close to this many rows (review L5).
pub const MAX_ADECK_RECORDS: usize = 200_000;

/// Inflate a gzip body, refusing non-gzip input and anything that inflates
/// past `max_out` bytes. Invalid UTF-8 is replaced, not rejected (a-decks
/// are ASCII). Decodes every member of a multi-member stream, not just the
/// first — NHC has never been observed to concatenate members, but nothing
/// stops it, and a silently-dropped later member would mean a silently
/// stale cycle (review L8). Run on the background worker only, never the UI
/// thread.
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
    flate2::read::MultiGzDecoder::new(gz)
        .take(max_out + 1)
        .read_to_end(&mut out)
        .map_err(|e| anyhow!("inflating a-deck: {e}"))?;
    if out.len() as u64 > max_out {
        bail!("a-deck inflates past {max_out} bytes");
    }
    // Valid UTF-8 (every real a-deck) takes this branch, which moves `out`
    // into the String without copying it; only the invalid-UTF-8 fallback
    // (never expected in practice) pays for a copy (review L5).
    Ok(String::from_utf8(out)
        .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned()))
}

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
        && tech
            .get(2..4)
            .is_some_and(|d| d.bytes().all(|b| b.is_ascii_digit()));
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

/// Gap between the newest-any-tech cycle and the newest-OFCL cycle beyond
/// which `latest_cycle` prefers the newest-any cycle instead of waiting on
/// OFCL. NHC issues a new synoptic cycle every 6 h and typically finishes
/// that cycle's OFCL about 3 h after the nominal time, so up to about 9 h
/// of lag is routine. 18 h covers that plus one entire missed cycle without
/// discarding OFCL over an ordinary few-hour delay, and it also comfortably
/// exceeds the cycle-age thresholds in `adeck_style` (12 h amber, 24 h dim):
/// by the time this fallback fires, showing the old OFCL cycle instead would
/// already read as stale under those thresholds anyway. This same path
/// handles a storm gone post-tropical while still listed in
/// `CurrentStorms.json`: OFCL stops appearing, so the gap only grows, and
/// once it passes 18 h the newest model-only cycle takes over instead of
/// staying pinned to the last OFCL forever (review S8). Cycles land on 6 h
/// boundaries, so in practice this fires once the gap reaches a full 24 h
/// (4 cycles): a gap of exactly 18 h (3 cycles) does not exceed the
/// threshold and still prefers OFCL.
pub const OFCL_LAG_FALLBACK_SECS: i64 = 18 * 60 * 60;

/// How far into the future a cycle's own timestamp may sit relative to
/// `now` and still be treated as a real candidate. A few hours covers
/// ordinary processing delay and modest clock skew between this machine
/// and NHC's publishing pipeline; `parse_init` only checks "10 ASCII
/// digits," so a corrupt or typo'd `init` (e.g. a stray digit landing it a
/// year or a month ahead) could otherwise "win" as the newest cycle. Left
/// unchecked, that bogus cycle would then latch permanently: it would look
/// newest forever, and `GuidanceStore::insert`'s regression guard would
/// reject every later, legitimate cycle for looking "older" (review S8).
pub const FUTURE_CYCLE_TOLERANCE_SECS: i64 = 3 * 60 * 60;

/// Whether `init` is a real calendar timestamp (not malformed) that isn't
/// further in the future than `FUTURE_CYCLE_TOLERANCE_SECS` allows. Shared
/// by `latest_cycle` (excludes implausible rows from cycle selection) and
/// `GuidanceStore::insert` (lets a valid cycle recover from a previously
/// stored implausible one).
fn is_plausible_cycle(init: u32, now_unix: i64) -> bool {
    cycle_time_unix(init).is_some_and(|t| t <= now_unix + FUTURE_CYCLE_TOLERANCE_SECS)
}

/// The cycle to show: the newest init with an OFCL forecast (the newest cycle
/// fills in over several hours, so it may have models but no OFCL yet), else
/// the newest init of any row (invests). When OFCL is stuck on an older
/// cycle than the newest-any-tech cycle by more than `OFCL_LAG_FALLBACK_SECS`
/// of real elapsed time, the newest-any cycle wins instead — either OFCL is
/// unusually late, or the storm has gone post-tropical and OFCL has stopped
/// altogether (review S8). The gap is measured in actual elapsed time via
/// `cycle_time_unix`, not by counting synoptic cycles, so it still behaves
/// sensibly if a cycle is ever skipped. Rows with a calendar-invalid or
/// implausibly future-dated `init` (relative to `now_unix`) are excluded
/// from "newest" entirely, so a single corrupt row can't hijack cycle
/// selection (review S8/S3 interaction).
pub fn latest_cycle(records: &[ADeckRecord], now_unix: i64) -> Option<u32> {
    let newest_any = records
        .iter()
        .map(|r| r.init)
        .filter(|&init| is_plausible_cycle(init, now_unix))
        .max()?;
    let newest_ofcl = records
        .iter()
        .filter(|r| r.tech == "OFCL")
        .map(|r| r.init)
        .filter(|&init| is_plausible_cycle(init, now_unix))
        .max();
    let Some(ofcl) = newest_ofcl else {
        return Some(newest_any);
    };
    if ofcl == newest_any {
        return Some(ofcl);
    }
    let gap = cycle_time_unix(newest_any).zip(cycle_time_unix(ofcl));
    if gap.is_some_and(|(any_t, ofcl_t)| any_t - ofcl_t > OFCL_LAG_FALLBACK_SECS) {
        Some(newest_any)
    } else {
        Some(ofcl)
    }
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
/// `now_unix` bounds `latest_cycle`'s future-dated-row check.
pub fn guidance_from_gzip(storm_id: &str, gz: &[u8], now_unix: i64) -> Result<StormGuidance> {
    let text = gunzip_limited(gz, MAX_UNCOMPRESSED_BYTES)?;
    let records = parse_adeck(&text);
    let init = latest_cycle(&records, now_unix)
        .ok_or_else(|| anyhow!("a-deck for {storm_id} has no parseable rows"))?;
    Ok(build_guidance(storm_id, &records, init))
}

/// Whether `a` and `b` describe the same guidance content, ignoring
/// `fetched_at_unix` (which always differs between one fetch and the next).
/// Used by `GuidanceStore::insert` to skip the revision bump — and the
/// projection-cache rebuild it drives — when a refetch brings back exactly
/// what's already stored (review S9).
fn guidance_content_eq(a: &StormGuidance, b: &StormGuidance) -> bool {
    let a = StormGuidance {
        fetched_at_unix: 0,
        ..a.clone()
    };
    let b = StormGuidance {
        fetched_at_unix: 0,
        ..b.clone()
    };
    a == b
}

/// What `GuidanceStore::insert` actually did, so a caller can log a real
/// cycle regression (S3) distinctly from the common, harmless case of an
/// unchanged refetch (S9) — both leave `revision` unchanged, so a caller
/// that only compared `revision()` before and after (as the original S3
/// fix did) can no longer tell them apart (fix-round-1 follow-up).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsertOutcome {
    /// The stored cycle was newer (and plausible); the incoming guidance
    /// was discarded entirely. Worth logging distinctly — a real cycle
    /// regression (review S3).
    RejectedOlder,
    /// Same init, same content as what's stored (ignoring
    /// `fetched_at_unix`). Only `fetched_at_unix` was updated; `revision`
    /// did not bump. Routine — most refetches within a ~6 h cycle land
    /// here (review S9).
    Unchanged,
    /// The entry was replaced and `revision` bumped: a brand-new storm, a
    /// newer cycle, or same-or-newer content that genuinely differs.
    Replaced,
}

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
    /// Replace the stored guidance for `guidance.storm_id`, with two guards
    /// ahead of the normal replace-and-bump, applied in this order. Returns
    /// which of the three happened (`InsertOutcome`), so a caller can log a
    /// real cycle regression separately from a routine unchanged refetch —
    /// both leave `revision` unchanged, so "did `revision` change" alone no
    /// longer distinguishes them (fix-round-1 follow-up).
    ///
    /// 1. **Cycle regression (review S3/S8) → `RejectedOlder`.** If a
    ///    lagging mirror or cache serves a cycle older than the one already
    ///    stored, the insert is a no-op: no replace, no revision bump, no
    ///    `fetched_at_unix` update. This guard only fires when the
    ///    *stored* cycle is itself plausible (not implausibly future-dated
    ///    or calendar-invalid per `is_plausible_cycle`), so a bogus stored
    ///    `init` can't entrench itself forever by rejecting every later
    ///    legitimate cycle for looking "older."
    /// 2. **Unchanged content (review S9) → `Unchanged`.** Otherwise, if
    ///    the incoming guidance is byte-for-byte identical to what's
    ///    stored — same `init`, same tracks, same intensity, same
    ///    `has_official` — except for `fetched_at_unix` (which always
    ///    differs between fetches), only `fetched_at_unix` is updated on
    ///    the stored entry. The revision does NOT bump, so the
    ///    km-projection cache (keyed on revision) isn't wastefully rebuilt
    ///    for a 15-min refetch that brought back exactly what was already
    ///    on screen; a-decks only change meaningfully every ~6 h. The
    ///    "checked N min ago" line still advances because `fetched_at_unix`
    ///    does.
    /// 3. **Normal replace → `Replaced`.** Anything else — a brand-new
    ///    storm, a newer cycle, or a same-or-newer cycle whose content
    ///    genuinely differs (e.g. a cycle still filling in as more models
    ///    arrive, or recovery from a previously stored implausible cycle)
    ///    — replaces the entry and bumps the revision as before.
    pub fn insert(&mut self, guidance: StormGuidance, now_unix: i64) -> InsertOutcome {
        if let Some(existing) = self.by_storm.get(&guidance.storm_id) {
            if is_plausible_cycle(existing.init, now_unix) && guidance.init < existing.init {
                return InsertOutcome::RejectedOlder;
            }
            if guidance_content_eq(&guidance, existing) {
                let fetched_at_unix = guidance.fetched_at_unix;
                if let Some(stored) = self.by_storm.get_mut(&guidance.storm_id) {
                    stored.fetched_at_unix = fetched_at_unix;
                }
                return InsertOutcome::Unchanged;
            }
        }
        self.by_storm.insert(guidance.storm_id.clone(), guidance);
        self.revision += 1;
        InsertOutcome::Replaced
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

    /// Storms with at least one default-visible track, i.e. storms whose
    /// guidance is actually drawn. Both legend helpers below only consider
    /// these, since a storm with nothing on screen shouldn't drive the
    /// legend's single cycle line.
    fn shown(&self) -> impl Iterator<Item = &StormGuidance> {
        self.by_storm
            .values()
            .filter(|g| g.tracks.iter().any(|t| t.default_visible))
    }

    /// The shown storm with the earliest forecast cycle (`init`), picked by
    /// cycle order rather than fetch order so the legend's cycle line names
    /// a real cycle instead of whichever storm happened to be fetched least
    /// recently (review S5). Callers should show "mixed cycles" instead of
    /// this storm's cycle when `has_single_cycle()` is false.
    pub fn earliest_cycle(&self) -> Option<&StormGuidance> {
        self.shown().min_by_key(|g| g.init)
    }

    /// True when every shown storm is on the same forecast cycle. The
    /// legend's one cycle line is misleading otherwise (review S5); an empty
    /// or single-storm store trivially counts as one cycle.
    pub fn has_single_cycle(&self) -> bool {
        let mut inits = self.shown().map(|g| g.init);
        let Some(first) = inits.next() else {
            return true;
        };
        inits.all(|i| i == first)
    }
}

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
    fn parse_line_reads_a_southern_hemisphere_row() {
        // All other fixtures in this module are northern hemisphere; a
        // storm south of the equator is a real, common case ATCF covers
        // via the "S" suffix (review L19 fixture gap).
        let line = "SH, 09, 2026100912, 03, AVNI,  12, 234S,  1500E,  87,    0,   ,  34, NEQ,";
        let r = parse_adeck_line(line).expect("valid row");
        assert_eq!(r.position, Some((-23.4, 150.0)));
    }

    #[test]
    fn parse_line_reads_a_wide_row_with_stormname() {
        // Real OFCL/CARQ rows run up to 46 comma-separated fields,
        // including STORMNAME at field 28 and several userdefined fields
        // after it; every fixture elsewhere in this module is cut down to
        // about 17-36 fields (review L19 fixture gap).
        let line = "AL, 14, 2026100912, 03, OFCL,  24, 275N,  800W,  95, 970, HU,  34, NEQ,  120,   90,   70,  100,  200,  300,   20,  115,   25, AL1, 10, WW, 300,   12,     MILTON,     D,  10, WW,   8,   9,  10,  11,   S,   M,   W, HRD,   0,   0,   0,   0,   0,   0,";
        let r = parse_adeck_line(line).expect("wide row with trailing fields parses");
        assert_eq!(r.tech, "OFCL");
        assert_eq!(r.tau_h, 24);
        assert_eq!(r.position, Some((27.5, -80.0)));
        assert_eq!(r.vmax_kt, Some(95));
    }

    #[test]
    fn parse_adeck_handles_crlf_line_endings() {
        // str::lines() already treats "\r\n" as one line ending, so this
        // pins that behaviour against a regression rather than fixing a
        // bug (review L19 fixture gap).
        let lf = "AL, 09, 2026100912, 03, AVNI,  12, 301N,  870W,  87,    0,   ,  34, NEQ,\nAL, 09, 2026100912, 03, OFCL,  24, 280N,  875W,  90,    0,   ,  34, NEQ,";
        let crlf = lf.replace('\n', "\r\n");
        assert_eq!(parse_adeck(crlf.as_str()), parse_adeck(lf));
        assert_eq!(parse_adeck(crlf.as_str()).len(), 2);
    }

    #[test]
    fn build_guidance_keeps_the_first_row_when_technum_differs() {
        // ATCF files are append-only; a later revision of the same
        // (tech, tau) can carry a different TECHNUM. build_guidance's
        // dedupe keys on (tech, tau) only and keeps the first row seen
        // (`.or_insert`), so a later revision at the same tau is currently
        // discarded. This pins that documented behaviour (review L9 is the
        // fix; this is just the L19 coverage gap for it).
        let lines = [
            "AL, 09, 2026100912, 01, CARQ,  -6, 260N,  800W,  75,    0,   ,  34, NEQ,",
            "AL, 09, 2026100912, 01, CARQ,   0, 270N,  800W,  80,    0,   ,  34, NEQ,",
            // Same tech+tau as the row above, a later TECHNUM revision.
            "AL, 09, 2026100912, 02, CARQ,   0, 271N,  801W,  85,    0,   ,  34, NEQ,",
        ];
        let records = parse_adeck(&lines.join("\n"));
        assert_eq!(records.len(), 3, "all three rows parse individually");
        let g = build_guidance("al092026", &records, 2026100912);
        let carq = g.tracks.iter().find(|t| t.tech == "CARQ").unwrap();
        let at_tau_0 = carq.points.iter().find(|p| p.tau_h == 0).unwrap();
        assert_eq!(
            (at_tau_0.lat, at_tau_0.lon),
            (27.0, -80.0),
            "the first TECHNUM's row wins over the later revision"
        );
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
            "AL, 09, 2026100912, 03, AVNI, 2147483647, 301N, 870W, 87",
        ] {
            assert_eq!(parse_adeck_line(bad), None, "should reject {bad:?}");
        }
    }

    #[test]
    fn parse_line_rejects_tau_outside_the_sane_range() {
        // A row with tau i32::MAX passes i32's own parse; without a range
        // check at parse time, `adeck_style::chart_max_tau`'s `max + 23`
        // would panic on the UI thread in debug and test builds (review
        // L6). Also covered via parse_adeck_line above; this pins the
        // boundary directly.
        let row = |tau: i32| {
            format!("AL, 09, 2026100912, 03, AVNI, {tau}, 301N, 870W, 87,    0,   ,  34, NEQ,")
        };
        assert!(
            parse_adeck_line(&row(-240)).is_some(),
            "lower bound is fine"
        );
        assert!(parse_adeck_line(&row(-241)).is_none());
        assert!(parse_adeck_line(&row(384)).is_some(), "upper bound is fine");
        assert!(parse_adeck_line(&row(385)).is_none());
        assert!(parse_adeck_line(&row(i32::MAX)).is_none());
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
    fn parse_lon_rejects_east_past_180_degrees() {
        // ATCF longitude is 0-1800 tenths (0-180.0°) plus E/W; a value past
        // that is malformed, not an alternate 0-360° convention. A previous
        // version folded it by subtracting 360 once, so "3700E" (370.0°
        // raw) silently became a plausible 10.0°E instead of being
        // rejected (review L7).
        assert_eq!(parse_lon("3700E"), None);
        assert_eq!(parse_lon("1900E"), None);
        assert_eq!(parse_lon("1800E"), Some(180.0), "exactly at the cap");
        assert_eq!(parse_lon("3700W"), None);
    }

    #[test]
    fn parse_tenths_rejects_a_leading_plus_sign() {
        // u32::from_str accepts a leading '+' that no real ATCF field ever
        // contains; parse_init already requires all-digit input, and
        // parse_lat/parse_lon (via parse_tenths) now match it (review L7).
        assert_eq!(parse_lat("+270N"), None);
        assert_eq!(parse_lon("+870W"), None);
    }

    #[test]
    fn cycle_time_unix_parses_a_valid_cycle() {
        let got = cycle_time_unix(2026100912).expect("valid cycle");
        let want = chrono::NaiveDate::from_ymd_opt(2026, 10, 9)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        assert_eq!(got, want);
    }

    #[test]
    fn cycle_time_unix_boundary_hours_and_midnight() {
        assert!(cycle_time_unix(2026100900).is_some(), "00Z is valid");
        assert!(cycle_time_unix(2026100923).is_some(), "23Z is valid");
    }

    #[test]
    fn cycle_time_unix_rejects_impossible_calendar_values() {
        for bad in [
            2026130100u32, // month 13
            2026023000,    // Feb 30 (2026 isn't even a leap year)
            2028023000,    // Feb 30, even on a leap year
            2026100924,    // hour 24
            2026100199,    // hour 99
            0,
        ] {
            assert_eq!(cycle_time_unix(bad), None, "should reject {bad}");
        }
    }

    #[test]
    fn cycle_time_unix_accepts_leap_day() {
        assert!(cycle_time_unix(2028022912).is_some(), "2028 is a leap year");
        assert_eq!(cycle_time_unix(2026022912), None, "2026 is not");
    }

    #[test]
    fn parse_adeck_reads_every_fixture_line() {
        assert_eq!(FIXTURE.lines().count(), 48);
        assert_eq!(parse_adeck(FIXTURE).len(), 48);
    }

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

    #[test]
    fn gunzip_decodes_every_member_of_a_multi_member_stream() {
        // Nothing stops NHC from building the .gz by appending members; a
        // decoder that only reads the first would silently drop a later
        // cycle and show an older one as fresh (review L8, S1).
        let mut multi = gzip(b"first member\n");
        multi.extend(gzip(b"second member\n"));
        let text = gunzip_limited(&multi, MAX_UNCOMPRESSED_BYTES).unwrap();
        assert_eq!(text, "first member\nsecond member\n");
    }

    #[test]
    fn gunzip_round_trips_non_ascii_bytes_without_copying_valid_utf8() {
        // Exercises the String::from_utf8 fast path directly (review L5);
        // the happy path for every real a-deck, which is pure ASCII.
        let text = "café — ASCII plus a few UTF-8 bytes\n";
        let got = gunzip_limited(&gzip(text.as_bytes()), MAX_UNCOMPRESSED_BYTES).unwrap();
        assert_eq!(got, text);
    }

    #[test]
    fn parse_adeck_caps_the_record_count() {
        // A crafted gzip that inflates to the full text cap must not be
        // able to push the parsed Vec past a bounded size (review L5).
        let line = "AL, 09, 2026100912, 03, AVNI,  12, 301N,  870W,  87,    0,   ,  34, NEQ,\n";
        let text = line.repeat(MAX_ADECK_RECORDS + 10);
        assert_eq!(parse_adeck(&text).len(), MAX_ADECK_RECORDS);
    }

    /// The research "on by default" set plus OFCL and CARQ (always on).
    const DEFAULT_TRACKS: [&str; 16] = [
        "OFCL", "CARQ", "AVNI", "HWFI", "HFAI", "HFBI", "HMNI", "CTCI", "NVG2", "CMC2", "UKX2",
        "AEMI", "CEM2", "GDMI", "TVCN", "HCCA",
    ];

    #[test]
    fn default_track_set_matches_research() {
        for tech in DEFAULT_TRACKS {
            let info = classify_tech(tech).unwrap_or_else(|| panic!("{tech} must be known"));
            assert!(
                info.track && info.default_visible,
                "{tech} should be a default track"
            );
        }
    }

    #[test]
    fn extra_models_are_tracks_but_hidden_by_default() {
        for tech in [
            "TABS", "TABM", "TABD", "XTRP", "CLP5", "TCLP", "AP01", "AP30", "OFCI", "AVNO", "HWRF",
            "ZZZZ",
        ] {
            let info = classify_tech(tech).unwrap();
            assert!(info.track, "{tech} is a track");
            assert!(!info.default_visible, "{tech} hidden by default");
        }
        assert_eq!(classify_tech("ZZZZ").unwrap().group, ModelGroup::Other);
        assert_eq!(
            classify_tech("AP17").unwrap().group,
            ModelGroup::EnsembleMember
        );
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
        for tech in DEFAULT_TRACKS
            .iter()
            .chain(["IVCN", "SHIP", "TABS", "AP01", "ZZZZ"].iter())
        {
            if let Some(info) = classify_tech(tech) {
                assert!(!info.default_visible || info.track, "{tech}");
            }
        }
    }

    fn fixture_guidance() -> StormGuidance {
        build_guidance("AL092026", &parse_adeck(FIXTURE), 2026100912)
    }

    /// `now` exactly at `init`'s cycle time, so nothing in a record set
    /// whose newest row is `init` reads as future-dated.
    fn now_after(init: u32) -> i64 {
        cycle_time_unix(init).unwrap()
    }

    #[test]
    fn latest_cycle_is_newest_with_official_forecast() {
        assert_eq!(
            latest_cycle(&parse_adeck(FIXTURE), now_after(2026100912)),
            Some(2026100912)
        );
        // A newer, still-filling cycle without OFCL must not win.
        let mut records = parse_adeck(FIXTURE);
        records.extend(parse_adeck_line(
            "AL, 09, 2026100918, 03, AVNI,  12, 301N,  870W,  87,    0,   ,  34, NEQ,",
        ));
        assert_eq!(
            latest_cycle(&records, now_after(2026100918)),
            Some(2026100912)
        );
    }

    #[test]
    fn latest_cycle_falls_back_when_no_official_forecast() {
        // Invests often have models but no OFCL.
        let records: Vec<_> = parse_adeck(FIXTURE)
            .into_iter()
            .filter(|r| r.tech != "OFCL")
            .collect();
        assert_eq!(
            latest_cycle(&records, now_after(2026100912)),
            Some(2026100912)
        );
        assert_eq!(latest_cycle(&[], now_after(2026100912)), None);
    }

    fn avni_row(init: u32) -> ADeckRecord {
        ADeckRecord {
            init,
            tech: "AVNI".to_string(),
            tau_h: 12,
            position: Some((30.1, -87.0)),
            vmax_kt: Some(87),
        }
    }

    fn ofcl_row(init: u32) -> ADeckRecord {
        ADeckRecord {
            tech: "OFCL".to_string(),
            ..avni_row(init)
        }
    }

    #[test]
    fn latest_cycle_prefers_ofcl_when_lag_is_within_the_threshold() {
        // OFCL still fills in for a few hours after the nominal cycle time;
        // that's routine lag, not a stalled feed, so OFCL should still win.
        let records = [ofcl_row(2026100912), avni_row(2026100918)]; // 6 h gap
        assert_eq!(
            latest_cycle(&records, now_after(2026100918)),
            Some(2026100912)
        );
    }

    #[test]
    fn latest_cycle_falls_back_to_newest_any_once_ofcl_lag_exceeds_threshold() {
        // 2026101100 is 36 h after 2026100912: OFCL has either stalled badly
        // or (review S8's other scenario) the storm went post-tropical and
        // OFCL has stopped entirely while model-only cycles keep advancing.
        let records = [ofcl_row(2026100912), avni_row(2026101100)];
        assert_eq!(
            latest_cycle(&records, now_after(2026101100)),
            Some(2026101100)
        );
    }

    #[test]
    fn latest_cycle_ofcl_lag_threshold_boundary() {
        // Exactly at the threshold: OFCL still wins ("exceeds", not "at").
        let at_threshold = [
            ofcl_row(2026100912),
            avni_row(2026101006), // +18h0m exactly
        ];
        assert_eq!(
            latest_cycle(&at_threshold, now_after(2026101006)),
            Some(2026100912)
        );
        let just_over = [
            ofcl_row(2026100912),
            avni_row(2026101007), // +19h, just past the boundary
        ];
        assert_eq!(
            latest_cycle(&just_over, now_after(2026101007)),
            Some(2026101007)
        );
    }

    #[test]
    fn latest_cycle_pins_to_last_ofcl_cycle_is_no_longer_permanent() {
        // A storm gone post-tropical: OFCL rows stop appearing for this and
        // later cycles entirely, but models keep advancing far past it.
        let records = [
            ofcl_row(2026100912),
            avni_row(2026101012),
            avni_row(2026101112), // another day of model-only cycles
        ];
        assert_eq!(
            latest_cycle(&records, now_after(2026101112)),
            Some(2026101112)
        );
    }

    #[test]
    fn latest_cycle_excludes_calendar_invalid_rows_from_newest() {
        // Month 99: passes parse_init (10 ASCII digits) but cycle_time_unix
        // rejects it. Must not win over a valid, recent OFCL cycle.
        let bogus = ADeckRecord {
            tech: "AVNI".to_string(),
            ..avni_row(2026109912)
        };
        let records = [ofcl_row(2026100912), bogus];
        assert_eq!(
            latest_cycle(&records, now_after(2026100912)),
            Some(2026100912)
        );
    }

    #[test]
    fn latest_cycle_excludes_future_dated_rows_from_newest() {
        // A row dated far beyond FUTURE_CYCLE_TOLERANCE_SECS ahead of `now`
        // must not win, or hijack cycle selection, over a valid recent
        // OFCL cycle (review S8/S3 interaction: an unchecked bogus "newest"
        // cycle would otherwise latch forever via the insert regression
        // guard).
        let now = now_after(2026100912);
        let far_future = avni_row(2026200100); // ~10 months ahead
        let records = [ofcl_row(2026100912), far_future];
        assert_eq!(latest_cycle(&records, now), Some(2026100912));
    }

    #[test]
    fn latest_cycle_future_tolerance_boundary() {
        let base = now_after(2026100912);
        // No OFCL in either case, so newest-any directly decides the
        // result and the plausibility filter's boundary is what's on test.
        let base_row = avni_row(2026100912);

        // Exactly at the tolerance (+3h): still a plausible candidate, and
        // wins as the newest.
        let at_tolerance = avni_row(2026100915);
        assert_eq!(
            latest_cycle(&[base_row.clone(), at_tolerance], base),
            Some(2026100915),
            "a row right at the tolerance boundary is still a candidate"
        );

        // One hour further out (+4h): excluded, so the only remaining
        // plausible row wins instead.
        let just_past = avni_row(2026100916);
        assert_eq!(
            latest_cycle(&[base_row, just_past], base),
            Some(2026100912),
            "a row past the tolerance is excluded from newest"
        );
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
        assert!(
            g.intensity
                .iter()
                .find(|s| s.tech == "OFCL")
                .unwrap()
                .official
        );
    }

    #[test]
    fn invest_without_official_forecast_is_flagged() {
        let records: Vec<_> = parse_adeck(FIXTURE)
            .into_iter()
            .filter(|r| r.tech != "OFCL")
            .collect();
        let init = latest_cycle(&records, now_after(2026100912)).unwrap();
        let g = build_guidance("al942026", &records, init);
        assert!(!g.has_official);
        assert!(g.tracks.iter().all(|t| t.tech != "OFCL"));
    }

    #[test]
    fn guidance_from_gzip_round_trip() {
        let g = guidance_from_gzip("al092026", &gzip(FIXTURE.as_bytes()), now_after(2026100912))
            .unwrap();
        assert_eq!(g.init, 2026100912);
        assert_eq!(g.tracks.len(), 5);
    }

    #[test]
    fn guidance_from_gzip_rejects_empty_file() {
        let err = guidance_from_gzip("al092026", &gzip(b""), now_after(2026100912)).unwrap_err();
        assert!(format!("{err:#}").contains("no parseable rows"), "{err:#}");
    }

    /// `now` for tests that don't care about the future-cycle guard: far
    /// enough past every hand-built fixture `init` in this module that
    /// nothing is ever excluded as "future-dated", so `GuidanceStore`
    /// behaves exactly as before that guard existed.
    const TEST_NOW: i64 = 4_102_444_800; // 2100-01-01T00:00:00Z

    fn guidance(id: &str, fetched_at_unix: i64) -> StormGuidance {
        guidance_with_init(id, 2026100912, fetched_at_unix)
    }

    fn guidance_with_init(id: &str, init: u32, fetched_at_unix: i64) -> StormGuidance {
        StormGuidance {
            storm_id: id.to_string(),
            init,
            has_official: true,
            tracks: Vec::new(),
            intensity: Vec::new(),
            fetched_at_unix,
        }
    }

    /// A single default-visible, two-point track, for tests that need a
    /// storm to count as "shown" (`GuidanceStore::earliest_cycle`,
    /// `has_single_cycle`).
    fn visible_track() -> AidTrack {
        AidTrack {
            tech: "AVNI".to_string(),
            group: ModelGroup::Global,
            default_visible: true,
            points: vec![
                TrackPoint {
                    tau_h: 0,
                    lat: 20.0,
                    lon: -80.0,
                    vmax_kt: Some(50),
                },
                TrackPoint {
                    tau_h: 12,
                    lat: 21.0,
                    lon: -81.0,
                    vmax_kt: Some(55),
                },
            ],
        }
    }

    fn shown_guidance(id: &str, init: u32) -> StormGuidance {
        StormGuidance {
            tracks: vec![visible_track()],
            ..guidance_with_init(id, init, 0)
        }
    }

    #[test]
    fn guidance_content_eq_ignores_fetched_at_unix_only() {
        let a = shown_guidance("al092026", 2026100912);
        let mut b = a.clone();
        b.fetched_at_unix = a.fetched_at_unix + 999;
        assert!(guidance_content_eq(&a, &b), "fetched_at_unix is excluded");
    }

    #[test]
    fn guidance_content_eq_detects_differing_tracks() {
        let a = shown_guidance("al092026", 2026100912);
        let mut b = a.clone();
        b.tracks.push(visible_track());
        assert!(!guidance_content_eq(&a, &b));
    }

    #[test]
    fn guidance_content_eq_detects_differing_intensity() {
        let a = shown_guidance("al092026", 2026100912);
        let mut b = a.clone();
        b.intensity.push(IntensitySeries {
            tech: "OFCL".to_string(),
            official: true,
            points: vec![(0, 50), (12, 55)],
        });
        assert!(!guidance_content_eq(&a, &b));
    }

    #[test]
    fn guidance_content_eq_detects_differing_has_official() {
        let a = shown_guidance("al092026", 2026100912);
        let mut b = a.clone();
        b.has_official = !a.has_official;
        assert!(!guidance_content_eq(&a, &b));
    }

    #[test]
    fn store_insert_same_content_refetch_keeps_one_entry_and_updates_fetched_at() {
        // `guidance(id, fetched_at_unix)` builds empty-tracks/empty-intensity
        // guidance at the same init both times, so the only difference
        // between the two inserts is `fetched_at_unix` — genuinely unchanged
        // content (review S9). The revision must NOT bump, but the stored
        // entry's `fetched_at_unix` must still advance so the "checked N min
        // ago" line stays accurate.
        let mut store = GuidanceStore::default();
        assert!(store.is_empty());
        store.insert(guidance("al092026", 10), TEST_NOW);
        let r1 = store.revision();
        store.insert(guidance("al092026", 20), TEST_NOW);
        assert_eq!(
            store.revision(),
            r1,
            "unchanged content does not bump the revision (S9)"
        );
        assert_eq!(store.iter().count(), 1);
        assert_eq!(store.get("AL092026").unwrap().fetched_at_unix, 20);
    }

    #[test]
    fn retain_active_drops_missing_storms() {
        let mut store = GuidanceStore::default();
        store.insert(guidance("al092026", 10), TEST_NOW);
        store.insert(guidance("ep182026", 10), TEST_NOW);
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
        store.insert(guidance("al092026", 50), TEST_NOW);
        store.insert(guidance("ep182026", 30), TEST_NOW);
        assert_eq!(store.oldest().unwrap().storm_id, "ep182026");
    }

    #[test]
    fn insert_rejects_a_cycle_older_than_the_one_stored() {
        let mut store = GuidanceStore::default();
        store.insert(guidance_with_init("al092026", 2026100912, 10), TEST_NOW);
        let r1 = store.revision();
        // A lagging mirror serves the previous cycle: must not replace, and
        // must not reset the freshness label by bumping fetched_at_unix.
        let outcome = store.insert(guidance_with_init("al092026", 2026100906, 20), TEST_NOW);
        assert_eq!(outcome, InsertOutcome::RejectedOlder);
        assert_eq!(store.revision(), r1, "older init is a no-op");
        let kept = store.get("al092026").unwrap();
        assert_eq!(kept.init, 2026100912, "newer cycle is kept");
        assert_eq!(kept.fetched_at_unix, 10, "fetch time is not bumped either");
    }

    #[test]
    fn insert_same_cycle_unchanged_content_does_not_bump_revision() {
        let mut store = GuidanceStore::default();
        store.insert(guidance_with_init("al092026", 2026100912, 10), TEST_NOW);
        let r1 = store.revision();
        let outcome = store.insert(guidance_with_init("al092026", 2026100912, 20), TEST_NOW);
        assert_eq!(outcome, InsertOutcome::Unchanged);
        assert_eq!(
            store.revision(),
            r1,
            "same cycle, unchanged content: no revision bump (S9)"
        );
        assert_eq!(store.get("al092026").unwrap().fetched_at_unix, 20);
    }

    #[test]
    fn insert_same_cycle_with_differing_content_still_bumps_revision() {
        // A cycle that's still filling in as more models arrive: same
        // `init`, but the content genuinely differs, so this must NOT be
        // treated as "unchanged" — it's a normal replace-and-bump, composing
        // correctly with the S9 unchanged-content guard above.
        let mut store = GuidanceStore::default();
        store.insert(guidance_with_init("al092026", 2026100912, 10), TEST_NOW);
        let r1 = store.revision();
        let mut filled_in = guidance_with_init("al092026", 2026100912, 20);
        filled_in.tracks.push(visible_track());
        let outcome = store.insert(filled_in, TEST_NOW);
        assert_eq!(outcome, InsertOutcome::Replaced);
        assert!(
            store.revision() > r1,
            "same cycle but different content still replaces and bumps"
        );
        let stored = store.get("al092026").unwrap();
        assert_eq!(stored.fetched_at_unix, 20);
        assert_eq!(stored.tracks.len(), 1);
    }

    #[test]
    fn insert_outcome_distinguishes_rejected_unchanged_and_replaced() {
        // Fix-round-1 follow-up: a caller that only compared `revision()`
        // before/after could no longer tell "real cycle regression" (S3)
        // apart from "routine unchanged refetch" (S9), since both leave
        // revision unchanged. `InsertOutcome` must let a caller distinguish
        // all three cases directly from the return value, not by inference.
        let mut store = GuidanceStore::default();
        // A brand-new storm always replaces.
        assert_eq!(
            store.insert(guidance_with_init("al092026", 2026100912, 10), TEST_NOW),
            InsertOutcome::Replaced
        );
        // Same init, identical content (only fetched_at_unix differs).
        assert_eq!(
            store.insert(guidance_with_init("al092026", 2026100912, 20), TEST_NOW),
            InsertOutcome::Unchanged
        );
        // An older cycle than what's stored.
        assert_eq!(
            store.insert(guidance_with_init("al092026", 2026100906, 30), TEST_NOW),
            InsertOutcome::RejectedOlder
        );
        // A newer cycle replaces again.
        assert_eq!(
            store.insert(guidance_with_init("al092026", 2026100918, 40), TEST_NOW),
            InsertOutcome::Replaced
        );
    }

    #[test]
    fn insert_accepts_a_newer_cycle() {
        let mut store = GuidanceStore::default();
        store.insert(guidance_with_init("al092026", 2026100912, 10), TEST_NOW);
        let r1 = store.revision();
        store.insert(guidance_with_init("al092026", 2026100918, 20), TEST_NOW);
        assert!(store.revision() > r1);
        assert_eq!(store.get("al092026").unwrap().init, 2026100918);
    }

    #[test]
    fn insert_of_a_brand_new_storm_is_unaffected_by_the_guard() {
        let mut store = GuidanceStore::default();
        store.insert(guidance_with_init("al092026", 2026100912, 10), TEST_NOW);
        let r1 = store.revision();
        store.insert(guidance_with_init("ep182026", 2026090100, 5), TEST_NOW);
        assert!(store.revision() > r1);
        assert!(store.get("ep182026").is_some());
    }

    #[test]
    fn insert_recovers_from_a_stored_future_dated_cycle() {
        let mut store = GuidanceStore::default();
        let now = now_after(2026100912);
        // Simulates a corrupt/typo'd init that reached `insert` directly
        // (the regression guard only applies from the SECOND insert for a
        // storm onward, so a bogus value can land on the very first one).
        // Numerically "newer" than any real forthcoming cycle, so a plain
        // older-init guard would reject every later legitimate cycle
        // forever without the plausibility check.
        store.insert(guidance_with_init("al092026", 2027100912, 10), now);
        let r1 = store.revision();
        store.insert(guidance_with_init("al092026", 2026101000, 20), now);
        assert!(
            store.revision() > r1,
            "a valid cycle recovers from a stored future-dated one"
        );
        assert_eq!(store.get("al092026").unwrap().init, 2026101000);
    }

    #[test]
    fn insert_recovers_from_a_stored_calendar_invalid_cycle() {
        let mut store = GuidanceStore::default();
        let now = now_after(2026100912);
        // Month 99: passes `parse_init`'s "10 ASCII digits" check, so it
        // could reach `insert` directly, but `cycle_time_unix` rejects it.
        store.insert(guidance_with_init("al092026", 2026109912, 10), now);
        let r1 = store.revision();
        store.insert(guidance_with_init("al092026", 2026100912, 20), now);
        assert!(
            store.revision() > r1,
            "a valid cycle recovers from a stored calendar-invalid one"
        );
        assert_eq!(store.get("al092026").unwrap().init, 2026100912);
    }

    #[test]
    fn earliest_cycle_picks_by_init_not_fetch_order() {
        let mut store = GuidanceStore::default();
        // ep is fetched most recently (fetched_at_unix=5) but is on the
        // older cycle; al is stale by fetch time (fetched_at_unix=0) but is
        // on the newer cycle. oldest() (fetch order) would pick al;
        // earliest_cycle() (cycle order) must pick ep instead.
        store.insert(shown_guidance("al092026", 2026100918), TEST_NOW);
        store.insert(
            {
                let mut g = shown_guidance("ep182026", 2026100906);
                g.fetched_at_unix = 5;
                g
            },
            TEST_NOW,
        );
        assert_eq!(store.earliest_cycle().unwrap().storm_id, "ep182026");
    }

    #[test]
    fn earliest_cycle_ignores_storms_with_nothing_shown() {
        let mut store = GuidanceStore::default();
        // Older cycle, but nothing default-visible: must not win.
        store.insert(guidance_with_init("al092026", 2026100900, 0), TEST_NOW);
        store.insert(shown_guidance("ep182026", 2026100912), TEST_NOW);
        assert_eq!(store.earliest_cycle().unwrap().storm_id, "ep182026");
    }

    #[test]
    fn has_single_cycle_true_for_empty_or_matching_storms() {
        let store = GuidanceStore::default();
        assert!(
            store.has_single_cycle(),
            "empty store is trivially one cycle"
        );

        let mut store = GuidanceStore::default();
        store.insert(shown_guidance("al092026", 2026100912), TEST_NOW);
        store.insert(shown_guidance("ep182026", 2026100912), TEST_NOW);
        assert!(store.has_single_cycle());
    }

    #[test]
    fn has_single_cycle_false_when_shown_storms_differ() {
        let mut store = GuidanceStore::default();
        store.insert(shown_guidance("al092026", 2026100912), TEST_NOW);
        store.insert(shown_guidance("ep182026", 2026100906), TEST_NOW);
        assert!(!store.has_single_cycle());
    }
}
