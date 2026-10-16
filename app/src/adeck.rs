//! NHC ATCF a-deck ("aid") parsing for the optional spaghetti-models overlay:
//! model-guidance tracks and intensities. Pure functions only (no network, no
//! drawing). Record format: `docs/research/spaghetti-models.md` section 2.

use anyhow::{Result, anyhow, bail};

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
}
