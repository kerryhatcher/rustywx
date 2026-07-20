//! Geographic math for the radarscope: great-circle range/bearing and
//! polar-to-screen projection, plus planar clipping helpers for drawing
//! overlays that stop at the scope edge. Azimuthal equidistant approximation
//! is acceptable at <= 230 km display range.

use egui::Vec2;

/// KJGX (Robins AFB, GA) antenna location — kept for backward compat.
pub const KJGX_LAT: f64 = 32.6755;
pub const KJGX_LON: f64 = -83.3511;

/// A NEXRAD site with its ICAO id, human-readable label, and antenna
/// coordinates.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct RadarSite {
    pub id: &'static str,
    pub name: &'static str,
    pub lat: f64,
    pub lon: f64,
}

/// All NEXRAD sites whose 230 km coverage radius overlaps Georgia.
pub const RADAR_SITES: &[RadarSite] = &[
    RadarSite {
        id: "KJGX",
        name: "Robins AFB, GA",
        lat: 32.6755,
        lon: -83.3511,
    },
    RadarSite {
        id: "KFFC",
        name: "Peachtree City, GA",
        lat: 33.3636,
        lon: -84.5658,
    },
    RadarSite {
        id: "KVAX",
        name: "Moody AFB, GA",
        lat: 30.8904,
        lon: -83.0019,
    },
    RadarSite {
        id: "KBMX",
        name: "Birmingham, AL",
        lat: 33.1722,
        lon: -86.7700,
    },
    RadarSite {
        id: "KHTX",
        name: "Huntsville, AL",
        lat: 34.9304,
        lon: -86.0836,
    },
    RadarSite {
        id: "KMRX",
        name: "Morristown, TN",
        lat: 36.1686,
        lon: -83.4019,
    },
    RadarSite {
        id: "KGSP",
        name: "Greer, SC",
        lat: 34.8833,
        lon: -82.2200,
    },
    RadarSite {
        id: "KCAE",
        name: "Columbia, SC",
        lat: 33.9486,
        lon: -81.1183,
    },
    RadarSite {
        id: "KCLX",
        name: "Charleston, SC",
        lat: 32.6550,
        lon: -81.0422,
    },
    RadarSite {
        id: "KJAX",
        name: "Jacksonville, FL",
        lat: 30.4847,
        lon: -81.7019,
    },
    RadarSite {
        id: "KTLH",
        name: "Tallahassee, FL",
        lat: 30.3976,
        lon: -84.3289,
    },
    RadarSite {
        id: "KEOX",
        name: "Fort Rucker, AL",
        lat: 31.4606,
        lon: -85.4594,
    },
];

/// Cities drawn on the scope: (name, lat, lon).
pub const CITIES: &[(&str, f64, f64)] = &[
    ("Macon", 32.8407, -83.6324),
    ("Warner Robins", 32.6130, -83.6242),
];

const EARTH_RADIUS_KM: f64 = 6371.0;

/// Great-circle distance (km) and initial bearing (degrees clockwise from
/// true north, in [0, 360)) from one lat/lon to another. Haversine formula.
pub fn range_bearing(from_lat: f64, from_lon: f64, to_lat: f64, to_lon: f64) -> (f64, f64) {
    let lat1 = from_lat.to_radians();
    let lat2 = to_lat.to_radians();
    let dlat = (to_lat - from_lat).to_radians();
    let dlon = (to_lon - from_lon).to_radians();

    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let range_km = 2.0 * EARTH_RADIUS_KM * a.sqrt().asin();

    let y = dlon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();
    let bearing_deg = (y.atan2(x).to_degrees() + 360.0) % 360.0;

    (range_km, bearing_deg)
}

/// Convert a polar position (azimuth degrees clockwise from north, range in
/// km) to screen-space pixel offsets from the scope center. Screen +y is
/// down, so north maps to a negative y offset.
pub fn polar_to_offset(azimuth_deg: f32, range_km: f32, px_per_km: f32) -> (f32, f32) {
    let theta = azimuth_deg.to_radians();
    let r = range_km * px_per_km;
    (r * theta.sin(), -r * theta.cos())
}

/// Convert a lat/lon point to a km-scale planar offset from an origin
/// (typically a radar site) using the same azimuthal-equidistant
/// convention as `polar_to_offset` with `px_per_km = 1.0`.
pub fn point_to_km_offset(origin_lat: f64, origin_lon: f64, (lat, lon): (f64, f64)) -> Vec2 {
    let (range_km, bearing_deg) = range_bearing(origin_lat, origin_lon, lat, lon);
    let theta = bearing_deg.to_radians();
    Vec2::new(
        (range_km * theta.sin()) as f32,
        (-range_km * theta.cos()) as f32,
    )
}

/// Return the portions of the segment `a` -> `b` that lie inside or on a
/// circle centered at the origin with radius `r`. This lets large alert or
/// border polygons that only clip through the scope still draw their visible
/// chord.
pub fn circle_subsegments(a: Vec2, b: Vec2, r: f32) -> Vec<(Vec2, Vec2)> {
    let d = b - a;
    let aa = d.length_sq();
    const EPS: f32 = 1e-5;

    // Degenerate segment; treat as inside if the point is within the circle.
    if aa < EPS {
        if a.length_sq() <= r * r + EPS {
            return vec![(a, b)];
        }
        return Vec::new();
    }

    let mut ts = Vec::new();
    if a.length_sq() <= r * r + EPS {
        ts.push(0.0);
    }
    if b.length_sq() <= r * r + EPS {
        ts.push(1.0);
    }

    // Solve |a + t*d|^2 = r^2 for t.
    let ad = a.dot(d);
    let c = a.length_sq() - r * r;
    let disc_sq = ad * ad - aa * c;
    if disc_sq >= 0.0 {
        let disc = disc_sq.sqrt();
        let t1 = (-ad - disc) / aa;
        let t2 = (-ad + disc) / aa;
        if (0.0..=1.0).contains(&t1) {
            ts.push(t1);
        }
        if (0.0..=1.0).contains(&t2) {
            ts.push(t2);
        }
    }

    ts.sort_by(|a, b| a.total_cmp(b));

    let mut out = Vec::new();
    for window in ts.windows(2) {
        let t_mid = (window[0] + window[1]) * 0.5;
        let mid = a + d * t_mid;
        if mid.length_sq() <= r * r + EPS {
            out.push((a + d * window[0], a + d * window[1]));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macon_range_bearing_from_kjgx() {
        let (range_km, bearing_deg) = range_bearing(KJGX_LAT, KJGX_LON, 32.8407, -83.6324);
        assert!((31.0..34.0).contains(&range_km), "range {range_km}");
        assert!(
            (302.0..308.0).contains(&bearing_deg),
            "bearing {bearing_deg}"
        );
    }

    #[test]
    fn range_bearing_due_north() {
        let (range_km, bearing_deg) = range_bearing(32.0, -83.0, 33.0, -83.0);
        assert!((range_km - 111.2).abs() < 1.0, "range {range_km}");
        assert!(bearing_deg.abs() < 0.01 || (bearing_deg - 360.0).abs() < 0.01);
    }

    #[test]
    fn polar_offsets_cardinal_directions() {
        // North: straight up the screen (negative y).
        let (x, y) = polar_to_offset(0.0, 10.0, 2.0);
        assert!(x.abs() < 1e-4 && (y + 20.0).abs() < 1e-4, "north ({x},{y})");
        // East: +x.
        let (x, y) = polar_to_offset(90.0, 10.0, 2.0);
        assert!((x - 20.0).abs() < 1e-3 && y.abs() < 1e-3, "east ({x},{y})");
        // South: +y (screen y grows downward).
        let (x, y) = polar_to_offset(180.0, 10.0, 2.0);
        assert!(x.abs() < 1e-3 && (y - 20.0).abs() < 1e-3, "south ({x},{y})");
    }

    #[test]
    fn keeps_both_endpoints_inside_circle() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 0.0);
        let segs = circle_subsegments(a, b, 20.0);
        assert_eq!(segs.len(), 1);
        assert!((segs[0].0 - a).length() < 1e-4);
        assert!((segs[0].1 - b).length() < 1e-4);
    }

    #[test]
    fn clips_outside_endpoint_to_circle() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(30.0, 0.0);
        let segs = circle_subsegments(a, b, 20.0);
        assert_eq!(segs.len(), 1);
        assert!((segs[0].0 - a).length() < 1e-4);
        assert!((segs[0].1 - Vec2::new(20.0, 0.0)).length() < 1e-4);
    }

    #[test]
    fn both_outside_but_crossing_produces_chord() {
        let a = Vec2::new(-30.0, 0.0);
        let b = Vec2::new(30.0, 0.0);
        let segs = circle_subsegments(a, b, 20.0);
        assert_eq!(segs.len(), 1);
        assert!((segs[0].0 - Vec2::new(-20.0, 0.0)).length() < 1e-4);
        assert!((segs[0].1 - Vec2::new(20.0, 0.0)).length() < 1e-4);
    }

    #[test]
    fn fully_outside_segment_yields_empty() {
        let a = Vec2::new(30.0, 0.0);
        let b = Vec2::new(40.0, 0.0);
        assert!(circle_subsegments(a, b, 20.0).is_empty());
    }
}
