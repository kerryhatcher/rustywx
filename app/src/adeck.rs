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
/// staying pinned to the last OFCL forever (review S8).
pub const OFCL_LAG_FALLBACK_SECS: i64 = 18 * 60 * 60;

/// The cycle to show: the newest init with an OFCL forecast (the newest cycle
/// fills in over several hours, so it may have models but no OFCL yet), else
/// the newest init of any row (invests). When OFCL is stuck on an older
/// cycle than the newest-any-tech cycle by more than `OFCL_LAG_FALLBACK_SECS`
/// of real elapsed time, the newest-any cycle wins instead — either OFCL is
/// unusually late, or the storm has gone post-tropical and OFCL has stopped
/// altogether (review S8). The gap is measured in actual elapsed time via
/// `cycle_time_unix`, not by counting synoptic cycles, so it still behaves
/// sensibly if a cycle is ever skipped.
pub fn latest_cycle(records: &[ADeckRecord]) -> Option<u32> {
    let newest_any = records.iter().map(|r| r.init).max()?;
    let newest_ofcl = records
        .iter()
        .filter(|r| r.tech == "OFCL")
        .map(|r| r.init)
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
pub fn guidance_from_gzip(storm_id: &str, gz: &[u8]) -> Result<StormGuidance> {
    let text = gunzip_limited(gz, MAX_UNCOMPRESSED_BYTES)?;
    let records = parse_adeck(&text);
    let init = latest_cycle(&records)
        .ok_or_else(|| anyhow!("a-deck for {storm_id} has no parseable rows"))?;
    Ok(build_guidance(storm_id, &records, init))
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
    /// Replace the stored guidance for `guidance.storm_id`, unless a lagging
    /// mirror or cache serves a cycle older than the one already stored —
    /// that insert is a no-op (no replace, no revision bump), so an older
    /// cycle can never roll the shown guidance backward and reset the
    /// freshness label to "just now" (review S3). A same-or-newer `init`
    /// still replaces normally, including a same-cycle re-fetch that only
    /// updates `fetched_at_unix` or fills in more model data.
    pub fn insert(&mut self, guidance: StormGuidance) {
        if let Some(existing) = self.by_storm.get(&guidance.storm_id)
            && guidance.init < existing.init
        {
            return;
        }
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
        let records: Vec<_> = parse_adeck(FIXTURE)
            .into_iter()
            .filter(|r| r.tech != "OFCL")
            .collect();
        assert_eq!(latest_cycle(&records), Some(2026100912));
        assert_eq!(latest_cycle(&[]), None);
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
        assert_eq!(latest_cycle(&records), Some(2026100912));
    }

    #[test]
    fn latest_cycle_falls_back_to_newest_any_once_ofcl_lag_exceeds_threshold() {
        // 2026101100 is 36 h after 2026100912: OFCL has either stalled badly
        // or (review S8's other scenario) the storm went post-tropical and
        // OFCL has stopped entirely while model-only cycles keep advancing.
        let records = [ofcl_row(2026100912), avni_row(2026101100)];
        assert_eq!(latest_cycle(&records), Some(2026101100));
    }

    #[test]
    fn latest_cycle_ofcl_lag_threshold_boundary() {
        // Exactly at the threshold: OFCL still wins ("exceeds", not "at").
        let at_threshold = [
            ofcl_row(2026100912),
            avni_row(2026101006), // +18h0m exactly
        ];
        assert_eq!(latest_cycle(&at_threshold), Some(2026100912));
        let just_over = [
            ofcl_row(2026100912),
            avni_row(2026101007), // +19h, just past the boundary
        ];
        assert_eq!(latest_cycle(&just_over), Some(2026101007));
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
        assert_eq!(latest_cycle(&records), Some(2026101112));
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

    #[test]
    fn insert_rejects_a_cycle_older_than_the_one_stored() {
        let mut store = GuidanceStore::default();
        store.insert(guidance_with_init("al092026", 2026100912, 10));
        let r1 = store.revision();
        // A lagging mirror serves the previous cycle: must not replace, and
        // must not reset the freshness label by bumping fetched_at_unix.
        store.insert(guidance_with_init("al092026", 2026100906, 20));
        assert_eq!(store.revision(), r1, "older init is a no-op");
        let kept = store.get("al092026").unwrap();
        assert_eq!(kept.init, 2026100912, "newer cycle is kept");
        assert_eq!(kept.fetched_at_unix, 10, "fetch time is not bumped either");
    }

    #[test]
    fn insert_accepts_the_same_cycle_refetched() {
        let mut store = GuidanceStore::default();
        store.insert(guidance_with_init("al092026", 2026100912, 10));
        let r1 = store.revision();
        store.insert(guidance_with_init("al092026", 2026100912, 20));
        assert!(
            store.revision() > r1,
            "same cycle still replaces (refresh path)"
        );
        assert_eq!(store.get("al092026").unwrap().fetched_at_unix, 20);
    }

    #[test]
    fn insert_accepts_a_newer_cycle() {
        let mut store = GuidanceStore::default();
        store.insert(guidance_with_init("al092026", 2026100912, 10));
        let r1 = store.revision();
        store.insert(guidance_with_init("al092026", 2026100918, 20));
        assert!(store.revision() > r1);
        assert_eq!(store.get("al092026").unwrap().init, 2026100918);
    }

    #[test]
    fn insert_of_a_brand_new_storm_is_unaffected_by_the_guard() {
        let mut store = GuidanceStore::default();
        store.insert(guidance_with_init("al092026", 2026100912, 10));
        let r1 = store.revision();
        store.insert(guidance_with_init("ep182026", 2026090100, 5));
        assert!(store.revision() > r1);
        assert!(store.get("ep182026").is_some());
    }

    #[test]
    fn earliest_cycle_picks_by_init_not_fetch_order() {
        let mut store = GuidanceStore::default();
        // al is fetched most recently but is on the older cycle; ep is
        // stale by fetch time but on the newer cycle. oldest() (fetch
        // order) and earliest_cycle() (cycle order) must disagree here.
        store.insert(shown_guidance("al092026", 2026100918));
        store.insert({
            let mut g = shown_guidance("ep182026", 2026100906);
            g.fetched_at_unix = 5;
            g
        });
        assert_eq!(store.earliest_cycle().unwrap().storm_id, "ep182026");
    }

    #[test]
    fn earliest_cycle_ignores_storms_with_nothing_shown() {
        let mut store = GuidanceStore::default();
        // Older cycle, but nothing default-visible: must not win.
        store.insert(guidance_with_init("al092026", 2026100900, 0));
        store.insert(shown_guidance("ep182026", 2026100912));
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
        store.insert(shown_guidance("al092026", 2026100912));
        store.insert(shown_guidance("ep182026", 2026100912));
        assert!(store.has_single_cycle());
    }

    #[test]
    fn has_single_cycle_false_when_shown_storms_differ() {
        let mut store = GuidanceStore::default();
        store.insert(shown_guidance("al092026", 2026100912));
        store.insert(shown_guidance("ep182026", 2026100906));
        assert!(!store.has_single_cycle());
    }
}
