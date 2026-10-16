//! NHC ATCF a-deck ("aid") parsing for the optional spaghetti-models overlay:
//! model-guidance tracks and intensities. Pure functions only (no network, no
//! drawing). Record format: `docs/research/spaghetti-models.md` section 2.

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
