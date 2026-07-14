//! PPI scope rendering: rasterize a sweep into an RGBA image (radar at
//! center, north up) via inverse polar mapping, and draw overlays.

use crate::colors;
use crate::model::{Product, SweepData};
use egui::{Color32, ColorImage};

/// Level II super-res gate geometry.
pub const FIRST_GATE_KM: f32 = 2.125;
pub const GATE_SPACING_KM: f32 = 0.25;
/// Display radius of the scope.
pub const MAX_RANGE_KM: f32 = 230.0;
/// Side length of the rasterized radar texture.
pub const RASTER_SIZE_PX: usize = 1024;

/// Rasterize one sweep to a square RGBA image, radar at center, north up.
/// Each pixel is inverse-mapped to (azimuth, range); the nearest radial by
/// azimuth and nearest gate by range supply its value.
pub fn rasterize(sweep: &SweepData, product: Product, size_px: usize, max_range_km: f32) -> ColorImage {
    let mut pixels = vec![Color32::TRANSPARENT; size_px * size_px];
    if sweep.radials.is_empty() {
        return ColorImage::new([size_px, size_px], pixels);
    }

    // Radial order sorted by azimuth for nearest-neighbor lookup.
    let mut order: Vec<usize> = (0..sweep.radials.len()).collect();
    order.sort_by(|&a, &b| sweep.radials[a].azimuth_deg.total_cmp(&sweep.radials[b].azimuth_deg));
    let sorted_azimuths: Vec<f32> = order.iter().map(|&i| sweep.radials[i].azimuth_deg).collect();

    let color_of = match product {
        Product::Reflectivity => colors::dbz_color as fn(f32) -> Color32,
        Product::Velocity => colors::velocity_color as fn(f32) -> Color32,
    };

    let center = size_px as f32 / 2.0;
    let km_per_px = 2.0 * max_range_km / size_px as f32;

    for py in 0..size_px {
        let dy = (py as f32 + 0.5 - center) * km_per_px;
        for px in 0..size_px {
            let dx = (px as f32 + 0.5 - center) * km_per_px;
            let range_km = (dx * dx + dy * dy).sqrt();
            if !(FIRST_GATE_KM..=max_range_km).contains(&range_km) {
                continue;
            }

            // Screen +y is down; north (0 deg) points up, east 90 deg right.
            let azimuth = dx.atan2(-dy).to_degrees().rem_euclid(360.0);
            let radial = &sweep.radials[order[nearest_radial_index(&sorted_azimuths, azimuth)]];

            let gate = ((range_km - FIRST_GATE_KM) / GATE_SPACING_KM) as usize;
            if let Some(Some(value)) = radial.gates.get(gate) {
                pixels[py * size_px + px] = color_of(*value);
            }
        }
    }

    ColorImage::new([size_px, size_px], pixels)
}

/// Index into `sorted_azimuths` of the entry angularly nearest to `az`,
/// accounting for wraparound at 0/360.
pub(crate) fn nearest_radial_index(sorted_azimuths: &[f32], az: f32) -> usize {
    let n = sorted_azimuths.len();
    match sorted_azimuths.binary_search_by(|a| a.total_cmp(&az)) {
        Ok(i) => i,
        Err(i) => {
            let before = (i + n - 1) % n;
            let after = i % n;
            if angular_distance(sorted_azimuths[before], az) <= angular_distance(sorted_azimuths[after], az) {
                before
            } else {
                after
            }
        }
    }
}

fn angular_distance(a: f32, b: f32) -> f32 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

#[cfg(test)]
mod tests {
    use crate::model::{Product, RadialData, SweepData};
    use egui::Color32;
    use super::{rasterize, nearest_radial_index};

    /// Four cardinal radials with distinct dBZ so quadrants are testable.
    /// 200 gates x 0.25 km reach 2.125 + 50 = 52.125 km.
    fn synthetic_sweep() -> SweepData {
        let radial = |az: f32, dbz: f32| RadialData {
            azimuth_deg: az,
            gates: vec![Some(dbz); 200],
        };
        SweepData {
            elevation_deg: 0.5,
            radials: vec![
                radial(0.0, 7.0),    // north: light cyan 04e9e7
                radial(90.0, 22.0),  // east: green 02fd02
                radial(180.0, 52.0), // south: red fd0000
                radial(270.0, 2.0),  // west: below threshold -> transparent
            ],
        }
    }

    #[test]
    fn rasterizes_quadrants_to_expected_colors() {
        let img = rasterize(&synthetic_sweep(), Product::Reflectivity, 128, 40.0);
        assert_eq!(img.size, [128, 128]);
        // 40 km max range on 128 px -> center at (64, 64), 0.625 km/px.
        // 32 px from center = 20 km: inside gate coverage, outside first-gate hole.
        let at = |x: usize, y: usize| img.pixels[y * 128 + x];
        assert_eq!(at(64, 32), Color32::from_rgb(0x04, 0xe9, 0xe7), "north");
        assert_eq!(at(96, 64), Color32::from_rgb(0x02, 0xfd, 0x02), "east");
        assert_eq!(at(64, 96), Color32::from_rgb(0xfd, 0x00, 0x00), "south");
        assert_eq!(at(32, 64), Color32::TRANSPARENT, "west below threshold");
    }

    #[test]
    fn center_and_beyond_range_are_transparent() {
        let img = rasterize(&synthetic_sweep(), Product::Reflectivity, 128, 40.0);
        let at = |x: usize, y: usize| img.pixels[y * 128 + x];
        // Center: inside the first-gate hole.
        assert_eq!(at(64, 64), Color32::TRANSPARENT);
        // Corner: range > max_range_km.
        assert_eq!(at(0, 0), Color32::TRANSPARENT);
    }

    #[test]
    fn gates_beyond_radial_data_are_transparent() {
        // Sweep reaches 52 km; ask for a pixel at ~60 km with 80 km range.
        let img = rasterize(&synthetic_sweep(), Product::Reflectivity, 128, 80.0);
        // 48 px north of center = 48 * 1.25 = 60 km.
        assert_eq!(img.pixels[16 * 128 + 64], Color32::TRANSPARENT);
    }

    #[test]
    fn nearest_radial_wraps_around_north() {
        let azimuths = [10.0, 180.0, 350.0];
        assert_eq!(nearest_radial_index(&azimuths, 355.0), 2);
        assert_eq!(nearest_radial_index(&azimuths, 3.0), 0);
        assert_eq!(nearest_radial_index(&azimuths, 175.0), 1);
        assert_eq!(nearest_radial_index(&azimuths, 359.9), 2);
    }
}
