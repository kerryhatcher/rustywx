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

/// Muted brown for state border overlays, distinct from the grid green
/// (`0x2a3a2f`) and the pale-yellow city markers (`0xddddaa`).
const BORDER_COLOR: Color32 = Color32::from_rgb(0x8a, 0x6d, 0x4a);

/// Rasterize one sweep to a square RGBA image, radar at center, north up.
/// Each pixel is inverse-mapped to (azimuth, range); the nearest radial by
/// azimuth and nearest gate by range supply its value.
pub fn rasterize(
    sweep: &SweepData,
    product: Product,
    size_px: usize,
    max_range_km: f32,
) -> ColorImage {
    let mut pixels = vec![Color32::TRANSPARENT; size_px * size_px];
    if sweep.radials.is_empty() {
        return ColorImage::new([size_px, size_px], pixels);
    }

    // Radial order sorted by azimuth for nearest-neighbor lookup.
    let mut order: Vec<usize> = (0..sweep.radials.len()).collect();
    order.sort_by(|&a, &b| {
        sweep.radials[a]
            .azimuth_deg
            .total_cmp(&sweep.radials[b].azimuth_deg)
    });
    let sorted_azimuths: Vec<f32> = order
        .iter()
        .map(|&i| sweep.radials[i].azimuth_deg)
        .collect();

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
            if angular_distance(sorted_azimuths[before], az)
                <= angular_distance(sorted_azimuths[after], az)
            {
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

/// Draw the scope: radar texture, range rings every 50 km, cardinal spokes,
/// state border overlay, city markers, station marker, scan time, and the product color legend.
pub fn draw_scope(
    ui: &mut egui::Ui,
    texture: Option<&egui::TextureHandle>,
    scan: Option<&crate::model::ScanData>,
    product: Product,
    borders: &[crate::borders::Ring],
) {
    use crate::geo;
    use egui::{Align2, FontId, Rect, Stroke, pos2, vec2};

    let available = ui.available_rect_before_wrap();
    let side = available.width().min(available.height());
    let rect = Rect::from_center_size(available.center(), vec2(side, side));
    let center = rect.center();
    let px_per_km = (side / 2.0) / MAX_RANGE_KM;
    let painter = ui.painter_at(available);

    let grid = Color32::from_rgb(0x2a, 0x3a, 0x2f);
    let grid_text = Color32::from_rgb(0x5f, 0x8a, 0x6a);
    let text_font = FontId::monospace(12.0);

    if let Some(texture) = texture {
        painter.image(
            texture.id(),
            rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }

    // Range rings every 50 km, labeled along the north spoke.
    let mut ring_km = 50.0;
    while ring_km <= MAX_RANGE_KM {
        painter.circle_stroke(center, ring_km * px_per_km, Stroke::new(1.0, grid));
        painter.text(
            center + vec2(4.0, -ring_km * px_per_km),
            Align2::LEFT_BOTTOM,
            format!("{ring_km:.0} km"),
            text_font.clone(),
            grid_text,
        );
        ring_km += 50.0;
    }

    // Cardinal spokes and labels.
    for (azimuth, label) in [(0.0, "N"), (90.0, "E"), (180.0, "S"), (270.0, "W")] {
        let (dx, dy) = geo::polar_to_offset(azimuth, MAX_RANGE_KM, px_per_km);
        painter.line_segment([center, center + vec2(dx, dy)], Stroke::new(1.0, grid));
        let (lx, ly) = geo::polar_to_offset(azimuth, MAX_RANGE_KM * 0.96, px_per_km);
        painter.text(
            center + vec2(lx, ly),
            Align2::CENTER_CENTER,
            label,
            text_font.clone(),
            grid_text,
        );
    }

    // State border outlines, drawn only where both endpoints of a segment
    // are within the display radius (same rule city markers use).
    for ring in borders {
        for pair in ring.windows(2) {
            let (lat1, lon1) = pair[0];
            let (lat2, lon2) = pair[1];
            let (range1, bearing1) = geo::range_bearing(geo::KJGX_LAT, geo::KJGX_LON, lat1, lon1);
            let (range2, bearing2) = geo::range_bearing(geo::KJGX_LAT, geo::KJGX_LON, lat2, lon2);
            if range1 as f32 > MAX_RANGE_KM || range2 as f32 > MAX_RANGE_KM {
                continue;
            }
            let (dx1, dy1) = geo::polar_to_offset(bearing1 as f32, range1 as f32, px_per_km);
            let (dx2, dy2) = geo::polar_to_offset(bearing2 as f32, range2 as f32, px_per_km);
            painter.line_segment(
                [center + vec2(dx1, dy1), center + vec2(dx2, dy2)],
                Stroke::new(1.2, BORDER_COLOR),
            );
        }
    }

    // Station marker at scope center.
    painter.circle_filled(center, 3.0, Color32::WHITE);
    painter.text(
        center + vec2(6.0, 6.0),
        Align2::LEFT_TOP,
        crate::data::SITE,
        text_font.clone(),
        Color32::WHITE,
    );

    // City markers.
    for &(name, lat, lon) in geo::CITIES {
        let (range_km, bearing_deg) = geo::range_bearing(geo::KJGX_LAT, geo::KJGX_LON, lat, lon);
        if range_km as f32 > MAX_RANGE_KM {
            continue;
        }
        let (dx, dy) = geo::polar_to_offset(bearing_deg as f32, range_km as f32, px_per_km);
        let position = center + vec2(dx, dy);
        painter.circle_stroke(
            position,
            3.5,
            Stroke::new(1.5, Color32::from_rgb(0xdd, 0xdd, 0xaa)),
        );
        painter.text(
            position + vec2(6.0, -6.0),
            Align2::LEFT_BOTTOM,
            name,
            text_font.clone(),
            Color32::from_rgb(0xdd, 0xdd, 0xaa),
        );
    }

    // Scan time, top-left of the panel.
    if let Some(scan) = scan {
        let utc = scan.timestamp.format("%Y-%m-%d %H:%M:%S UTC");
        let local = scan
            .timestamp
            .with_timezone(&chrono::Local)
            .format("%H:%M:%S %Z");
        painter.text(
            available.left_top() + vec2(8.0, 8.0),
            Align2::LEFT_TOP,
            format!("{utc}\n{local}"),
            text_font.clone(),
            Color32::WHITE,
        );
    }

    // Color legend, bottom-left of the panel.
    let legend: &[(f32, Color32)] = match product {
        Product::Reflectivity => crate::colors::DBZ_LEGEND,
        Product::Velocity => crate::colors::VELOCITY_LEGEND,
    };
    let unit = match product {
        Product::Reflectivity => "dBZ",
        Product::Velocity => "m/s",
    };
    let swatch = vec2(18.0, 12.0);
    let legend_origin = available.left_bottom() + vec2(8.0, -(swatch.y + 22.0));
    for (i, &(threshold, color)) in legend.iter().enumerate() {
        let min = legend_origin + vec2(i as f32 * swatch.x, 0.0);
        painter.rect_filled(Rect::from_min_size(min, swatch), 0.0, color);
        if i % 2 == 0 {
            painter.text(
                min + vec2(0.0, swatch.y + 2.0),
                Align2::LEFT_TOP,
                format!("{threshold:.0}"),
                FontId::monospace(10.0),
                grid_text,
            );
        }
    }
    painter.text(
        legend_origin + vec2(legend.len() as f32 * swatch.x + 6.0, 0.0),
        Align2::LEFT_TOP,
        unit,
        text_font,
        grid_text,
    );
}

#[cfg(test)]
mod tests {
    use super::{nearest_radial_index, rasterize};
    use crate::model::{Product, RadialData, SweepData};
    use egui::Color32;

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
