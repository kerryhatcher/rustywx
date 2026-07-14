//! Geographic math for the radarscope: great-circle range/bearing and
//! polar-to-screen projection. Azimuthal equidistant approximation is
//! acceptable at <= 230 km display range.

/// KJGX (Robins AFB, GA) antenna location.
pub const KJGX_LAT: f64 = 32.6755;
pub const KJGX_LON: f64 = -83.3511;

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
}
