//! NWS-style stepped color tables. Returns [r, g, b, a] arrays
//! for easy conversion to both Ply Color and macroquad Color.

/// Standard NWS base reflectivity bands: (threshold dBZ, [r, g, b, a]).
pub const DBZ_LEGEND: &[(f32, [u8; 4])] = &[
    (5.0, [0x04, 0xe9, 0xe7, 0xff]),
    (10.0, [0x01, 0x9f, 0xf4, 0xff]),
    (15.0, [0x03, 0x00, 0xf4, 0xff]),
    (20.0, [0x02, 0xfd, 0x02, 0xff]),
    (25.0, [0x01, 0xc5, 0x01, 0xff]),
    (30.0, [0x00, 0x8e, 0x00, 0xff]),
    (35.0, [0xfd, 0xf8, 0x02, 0xff]),
    (40.0, [0xe5, 0xbc, 0x00, 0xff]),
    (45.0, [0xfd, 0x95, 0x00, 0xff]),
    (50.0, [0xfd, 0x00, 0x00, 0xff]),
    (55.0, [0xd4, 0x00, 0x00, 0xff]),
    (60.0, [0xbc, 0x00, 0x00, 0xff]),
    (65.0, [0xf8, 0x00, 0xfd, 0xff]),
    (70.0, [0x98, 0x54, 0xc6, 0xff]),
    (75.0, [0xfd, 0xfd, 0xfd, 0xff]),
];

/// Base velocity bands in m/s.
pub const VELOCITY_LEGEND: &[(f32, [u8; 4])] = &[
    (-64.0, [0x00, 0xff, 0x90, 0xff]),
    (-40.0, [0x00, 0xe0, 0x00, 0xff]),
    (-30.0, [0x00, 0xb0, 0x00, 0xff]),
    (-20.0, [0x00, 0x80, 0x00, 0xff]),
    (-10.0, [0x4d, 0x66, 0x4d, 0xff]),
    (0.0, [0x66, 0x4d, 0x4d, 0xff]),
    (10.0, [0x80, 0x00, 0x00, 0xff]),
    (20.0, [0xb0, 0x00, 0x00, 0xff]),
    (30.0, [0xe0, 0x00, 0x00, 0xff]),
    (40.0, [0xff, 0x50, 0x50, 0xff]),
];

fn banded(legend: &[(f32, [u8; 4])], value: f32) -> [u8; 4] {
    let mut color = [0u8; 4]; // transparent
    for &(threshold, band_color) in legend {
        if value >= threshold {
            color = band_color;
        } else {
            break;
        }
    }
    color
}

pub fn dbz_color(dbz: f32) -> [u8; 4] {
    banded(DBZ_LEGEND, dbz)
}

pub fn velocity_color(ms: f32) -> [u8; 4] {
    banded(VELOCITY_LEGEND, ms.max(-64.0))
}

#[cfg(test)]
mod tests {
    use super::{DBZ_LEGEND, VELOCITY_LEGEND, dbz_color, velocity_color};

    #[test]
    fn dbz_below_minimum_is_transparent() {
        assert_eq!(dbz_color(-10.0), [0, 0, 0, 0]);
        assert_eq!(dbz_color(4.9), [0, 0, 0, 0]);
    }

    #[test]
    fn dbz_bands() {
        assert_eq!(dbz_color(5.0), [0x04, 0xe9, 0xe7, 0xff]); // light cyan
        assert_eq!(dbz_color(20.0), [0x02, 0xfd, 0x02, 0xff]); // green
        assert_eq!(dbz_color(52.0), [0xfd, 0x00, 0x00, 0xff]); // red
        assert_eq!(dbz_color(80.0), [0xfd, 0xfd, 0xfd, 0xff]); // white cap
    }

    #[test]
    fn velocity_sign_convention() {
        // Inbound (negative) is green; outbound (positive) is red.
        let inbound = velocity_color(-25.0);
        let outbound = velocity_color(25.0);
        assert!(
            inbound[1] > inbound[0],
            "inbound should be green: {inbound:?}"
        );
        assert!(
            outbound[0] > outbound[1],
            "outbound should be red: {outbound:?}"
        );
    }

    #[test]
    fn legends_are_ascending() {
        assert!(DBZ_LEGEND.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(VELOCITY_LEGEND.windows(2).all(|w| w[0].0 < w[1].0));
    }
}
