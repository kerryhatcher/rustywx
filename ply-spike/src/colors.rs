//! NWS-style color tables with smooth spline interpolation.
//!
//! Stage 6 replaces the stepped `banded()` lookup with cubic Hermite
//! spline (Catmull-Rom / Cardinal Spline with tension 0) interpolation
//! across the NWS anchor colours. This produces smooth, professional
//! colour ramps without visible banding while still passing exactly
//! through every NWS anchor point at its threshold value.
//!
//! Returns `[r, g, b, a]` arrays for easy conversion to both Ply `Color`
//! and macroquad `Color`.

/// Standard NWS base reflectivity anchors: (threshold dBZ, [r, g, b, a]).
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

/// Base velocity anchors in m/s.
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

/// Base spectrum width anchors in m/s. Low values (smooth, laminar flow)
/// are dark; high values (turbulence, wind shear, mixed targets) rise
/// through green → yellow → red.
pub const SPECTRUM_WIDTH_LEGEND: &[(f32, [u8; 4])] = &[
    (0.0, [0x20, 0x20, 0x20, 0xff]),
    (2.0, [0x00, 0x80, 0x00, 0xff]),
    (4.0, [0x00, 0xe0, 0x00, 0xff]),
    (6.0, [0xfd, 0xf8, 0x02, 0xff]),
    (8.0, [0xfd, 0x95, 0x00, 0xff]),
    (10.0, [0xfd, 0x00, 0x00, 0xff]),
    (15.0, [0xbc, 0x00, 0x00, 0xff]),
];

/// Catmull-Rom spline (Cardinal spline with tension 0) interpolation of
/// a single channel. Interpolates between `p1` and `p2` using the
/// neighbouring control points `p0` and `p3` for tangent estimation.
fn catmull_rom(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    // Catmull-Rom tangents (tension 0 → tangent scale = 1.0).
    let m1 = p2 - p0;
    let m2 = p3 - p1;
    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;
    h00 * p1 + h10 * m1 + h01 * p2 + h11 * m2
}

/// Smooth colour lookup via Catmull-Rom spline interpolation across
/// the anchor points of a legend.
///
/// - Below the first anchor's threshold: fully transparent `[0,0,0,0]`.
/// - At or above the last anchor's threshold: the last anchor's colour.
/// - Between anchors: cubic spline interpolation (passes exactly through
///   each anchor at its threshold, smooth in between).
fn spline_color(legend: &[(f32, [u8; 4])], value: f32) -> [u8; 4] {
    if legend.is_empty() {
        return [0, 0, 0, 0];
    }
    if value < legend[0].0 {
        return [0, 0, 0, 0]; // below minimum: transparent
    }
    let n = legend.len();
    if value >= legend[n - 1].0 {
        return legend[n - 1].1; // at/above maximum: clamp to last anchor
    }

    // Find segment i: legend[i].0 <= value < legend[i+1].0
    let i = match legend.binary_search_by(|(threshold, _)| threshold.total_cmp(&value)) {
        Ok(idx) => idx, // exact anchor hit — t will be 0
        Err(idx) => idx.saturating_sub(1),
    };
    let i = i.min(n - 2); // clamp to last valid segment

    let (_, c0) = legend[i.saturating_sub(1)]; // P0 (clamp at start)
    let (t1, c1) = legend[i]; // P1
    let (t2, c2) = legend[i + 1]; // P2
    let (_, c3) = legend[(i + 2).min(n - 1)]; // P3 (clamp at end)

    let span = t2 - t1;
    let t = if span > 0.0 { (value - t1) / span } else { 0.0 };
    let t = t.clamp(0.0, 1.0);

    let mut out = [0u8; 4];
    for ch in 0..4 {
        let v = catmull_rom(
            c0[ch] as f32,
            c1[ch] as f32,
            c2[ch] as f32,
            c3[ch] as f32,
            t,
        );
        out[ch] = v.round().clamp(0.0, 255.0) as u8;
    }
    out
}

pub fn dbz_color(dbz: f32) -> [u8; 4] {
    spline_color(DBZ_LEGEND, dbz)
}

pub fn velocity_color(ms: f32) -> [u8; 4] {
    spline_color(VELOCITY_LEGEND, ms.max(-64.0))
}

pub fn spectrum_width_color(ms: f32) -> [u8; 4] {
    spline_color(SPECTRUM_WIDTH_LEGEND, ms)
}

#[cfg(test)]
mod tests {
    use super::{
        DBZ_LEGEND, SPECTRUM_WIDTH_LEGEND, VELOCITY_LEGEND, catmull_rom, dbz_color,
        spectrum_width_color, velocity_color,
    };

    #[test]
    fn dbz_below_minimum_is_transparent() {
        assert_eq!(dbz_color(-10.0), [0, 0, 0, 0]);
        assert_eq!(dbz_color(4.9), [0, 0, 0, 0]);
    }

    #[test]
    fn dbz_at_anchor_hits_exact_colour() {
        // The spline passes exactly through every anchor at its threshold.
        assert_eq!(dbz_color(5.0), [0x04, 0xe9, 0xe7, 0xff]); // light cyan
        assert_eq!(dbz_color(20.0), [0x02, 0xfd, 0x02, 0xff]); // green
        assert_eq!(dbz_color(50.0), [0xfd, 0x00, 0x00, 0xff]); // red
        assert_eq!(dbz_color(75.0), [0xfd, 0xfd, 0xfd, 0xff]); // white cap
    }

    #[test]
    fn dbz_above_maximum_clamps_to_last() {
        assert_eq!(dbz_color(80.0), [0xfd, 0xfd, 0xfd, 0xff]);
        assert_eq!(dbz_color(999.0), [0xfd, 0xfd, 0xfd, 0xff]);
    }

    #[test]
    fn dbz_between_anchors_is_smooth_blend() {
        // 52 dBZ is between red (50) and dark red (55): a smooth blend,
        // not equal to either anchor.
        let mid = dbz_color(52.0);
        let red = dbz_color(50.0);
        let dark_red = dbz_color(55.0);
        assert_ne!(mid, red, "should not equal lower anchor");
        assert_ne!(mid, dark_red, "should not equal upper anchor");
        // Red channel should be between the two anchors (0xfd → 0xd4).
        assert!(
            mid[0] > dark_red[0] && mid[0] < red[0],
            "R channel should be between anchors: {mid:?}"
        );
        // Alpha stays opaque in range.
        assert_eq!(mid[3], 0xff);
    }

    #[test]
    fn dbz_is_continuous_near_anchors() {
        // Just above an anchor, the colour should be very close to that
        // anchor (continuity — the spline passes through every anchor).
        let near = dbz_color(20.1);
        let anchor = dbz_color(20.0);
        for ch in 0..3 {
            assert!(
                (near[ch] as i32 - anchor[ch] as i32).abs() <= 10,
                "channel {ch} should be near anchor near 20 dBZ: {near:?} vs {anchor:?}"
            );
        }
    }

    #[test]
    fn dbz_no_huge_jumps_between_neighbours() {
        // Smoothness: consecutive 0.5 dBZ steps should never jump by more
        // than a small amount (no banding discontinuities).
        let mut prev = dbz_color(30.0);
        let mut max_jump = 0i32;
        let mut v = 30.0f32;
        while v < 50.0 {
            v += 0.5;
            let cur = dbz_color(v);
            for ch in 0..3 {
                max_jump = max_jump.max((cur[ch] as i32 - prev[ch] as i32).abs());
            }
            prev = cur;
        }
        // A 0.5 dBZ step should never jump more than ~33 per channel.
        // (The NWS table itself has sharp transitions like green→yellow
        // that span ~250 units over 5 dBZ, so ~25/0.5 dBZ is expected.)
        assert!(max_jump <= 33, "max channel jump too large: {max_jump}");
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
    fn velocity_below_minimum_clamps() {
        let clamped = velocity_color(-100.0);
        let at_min = velocity_color(-64.0);
        assert_eq!(clamped, at_min);
    }

    #[test]
    fn legends_are_ascending() {
        assert!(DBZ_LEGEND.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(VELOCITY_LEGEND.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(SPECTRUM_WIDTH_LEGEND.windows(2).all(|w| w[0].0 < w[1].0));
    }

    #[test]
    fn spectrum_width_passes_through_anchors() {
        // The spline hits every anchor exactly at its threshold value.
        for &(threshold, color) in SPECTRUM_WIDTH_LEGEND {
            assert_eq!(spectrum_width_color(threshold), color);
        }
    }

    #[test]
    fn spectrum_width_below_minimum_is_transparent() {
        assert_eq!(spectrum_width_color(-1.0), [0, 0, 0, 0]);
    }

    #[test]
    fn spectrum_width_above_maximum_clamps_to_last() {
        let last = SPECTRUM_WIDTH_LEGEND.last().unwrap().1;
        assert_eq!(spectrum_width_color(30.0), last);
    }

    #[test]
    fn spectrum_width_increases_toward_warm_colors() {
        // Low width (smooth flow) should be darker/greener than high width
        // (turbulence), which should read hotter (more red than green).
        let low = spectrum_width_color(2.0);
        let high = spectrum_width_color(10.0);
        assert!(high[0] > low[0], "red channel should rise with width");
    }

    #[test]
    fn catmull_rom_endpoints_exact() {
        // At t=0 returns p1, at t=1 returns p2.
        assert!((catmull_rom(0.0, 10.0, 20.0, 30.0, 0.0) - 10.0).abs() < 1e-4);
        assert!((catmull_rom(0.0, 10.0, 20.0, 30.0, 1.0) - 20.0).abs() < 1e-4);
    }
}
