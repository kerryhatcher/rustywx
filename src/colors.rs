//! NWS-style stepped color tables for base reflectivity (dBZ) and base
//! velocity (m/s). Values below the product minimum are transparent.

use egui::Color32;

/// Standard NWS base reflectivity bands: (threshold dBZ, color). A value
/// maps to the color of the highest threshold it meets or exceeds.
pub const DBZ_LEGEND: &[(f32, Color32)] = &[
    (5.0, Color32::from_rgb(0x04, 0xe9, 0xe7)),
    (10.0, Color32::from_rgb(0x01, 0x9f, 0xf4)),
    (15.0, Color32::from_rgb(0x03, 0x00, 0xf4)),
    (20.0, Color32::from_rgb(0x02, 0xfd, 0x02)),
    (25.0, Color32::from_rgb(0x01, 0xc5, 0x01)),
    (30.0, Color32::from_rgb(0x00, 0x8e, 0x00)),
    (35.0, Color32::from_rgb(0xfd, 0xf8, 0x02)),
    (40.0, Color32::from_rgb(0xe5, 0xbc, 0x00)),
    (45.0, Color32::from_rgb(0xfd, 0x95, 0x00)),
    (50.0, Color32::from_rgb(0xfd, 0x00, 0x00)),
    (55.0, Color32::from_rgb(0xd4, 0x00, 0x00)),
    (60.0, Color32::from_rgb(0xbc, 0x00, 0x00)),
    (65.0, Color32::from_rgb(0xf8, 0x00, 0xfd)),
    (70.0, Color32::from_rgb(0x98, 0x54, 0xc6)),
    (75.0, Color32::from_rgb(0xfd, 0xfd, 0xfd)),
];

/// Base velocity bands in m/s: inbound (negative) greens, outbound
/// (positive) reds, weak echoes desaturated near zero.
pub const VELOCITY_LEGEND: &[(f32, Color32)] = &[
    (-64.0, Color32::from_rgb(0x00, 0xff, 0x90)),
    (-40.0, Color32::from_rgb(0x00, 0xe0, 0x00)),
    (-30.0, Color32::from_rgb(0x00, 0xb0, 0x00)),
    (-20.0, Color32::from_rgb(0x00, 0x80, 0x00)),
    (-10.0, Color32::from_rgb(0x4d, 0x66, 0x4d)),
    (0.0, Color32::from_rgb(0x66, 0x4d, 0x4d)),
    (10.0, Color32::from_rgb(0x80, 0x00, 0x00)),
    (20.0, Color32::from_rgb(0xb0, 0x00, 0x00)),
    (30.0, Color32::from_rgb(0xe0, 0x00, 0x00)),
    (40.0, Color32::from_rgb(0xff, 0x50, 0x50)),
];

fn banded(legend: &[(f32, Color32)], value: f32) -> Color32 {
    let mut color = Color32::TRANSPARENT;
    for &(threshold, band_color) in legend {
        if value >= threshold {
            color = band_color;
        } else {
            break;
        }
    }
    color
}

/// Color for a base reflectivity value in dBZ. Below 5 dBZ is transparent.
pub fn dbz_color(dbz: f32) -> Color32 {
    banded(DBZ_LEGEND, dbz)
}

/// Color for a base velocity value in m/s (negative = toward the radar).
/// Velocities below the lowest band (< -64 m/s) are clamped to it.
pub fn velocity_color(ms: f32) -> Color32 {
    banded(VELOCITY_LEGEND, ms.max(-64.0))
}

#[cfg(test)]
mod tests {
    use egui::Color32;
    use super::{dbz_color, velocity_color, DBZ_LEGEND, VELOCITY_LEGEND};

    #[test]
    fn dbz_below_minimum_is_transparent() {
        assert_eq!(dbz_color(-10.0), Color32::TRANSPARENT);
        assert_eq!(dbz_color(4.9), Color32::TRANSPARENT);
    }

    #[test]
    fn dbz_bands() {
        assert_eq!(dbz_color(5.0), Color32::from_rgb(0x04, 0xe9, 0xe7)); // light cyan
        assert_eq!(dbz_color(20.0), Color32::from_rgb(0x02, 0xfd, 0x02)); // green
        assert_eq!(dbz_color(52.0), Color32::from_rgb(0xfd, 0x00, 0x00)); // red
        assert_eq!(dbz_color(80.0), Color32::from_rgb(0xfd, 0xfd, 0xfd)); // white cap
    }

    #[test]
    fn velocity_sign_convention() {
        // Inbound (negative) is green; outbound (positive) is red.
        let inbound = velocity_color(-25.0);
        let outbound = velocity_color(25.0);
        assert!(inbound.g() > inbound.r(), "inbound should be green: {inbound:?}");
        assert!(outbound.r() > outbound.g(), "outbound should be red: {outbound:?}");
    }

    #[test]
    fn legends_are_ascending() {
        assert!(DBZ_LEGEND.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(VELOCITY_LEGEND.windows(2).all(|w| w[0].0 < w[1].0));
    }
}
