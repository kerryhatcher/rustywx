//! PPI scope rendering: rasterize a sweep into an RGBA image (radar at
//! center, north up) via inverse polar mapping, and draw overlays.

use crate::colors;
use crate::model::{Product, SweepData};
use egui::{Color32, ColorImage, Stroke};

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

/// Toggle options for optional NHC overlay layers.
#[derive(Clone)]
pub struct OverlayOptions {
    pub show_wind_probs_34kt: bool,
    pub show_wind_probs_50kt: bool,
    pub show_wind_probs_64kt: bool,
    pub show_earliest_arrival: bool,
    pub show_most_likely_arrival: bool,
    /// Reference to wind prob contours (34 kt).
    pub wind_probs_34kt: Vec<crate::nhc::WindProbContour>,
    /// Reference to wind prob contours (50 kt).
    pub wind_probs_50kt: Vec<crate::nhc::WindProbContour>,
    /// Reference to wind prob contours (64 kt).
    pub wind_probs_64kt: Vec<crate::nhc::WindProbContour>,
    /// Reference to earliest arrival contours.
    pub earliest_arrival: Vec<crate::nhc::ArrivalTimeContour>,
    /// Reference to most likely arrival contours.
    pub most_likely_arrival: Vec<crate::nhc::ArrivalTimeContour>,
}

impl Default for OverlayOptions {
    fn default() -> Self {
        Self {
            show_wind_probs_34kt: false,
            show_wind_probs_50kt: false,
            show_wind_probs_64kt: false,
            show_earliest_arrival: false,
            show_most_likely_arrival: false,
            wind_probs_34kt: Vec::new(),
            wind_probs_50kt: Vec::new(),
            wind_probs_64kt: Vec::new(),
            earliest_arrival: Vec::new(),
            most_likely_arrival: Vec::new(),
        }
    }
}

/// Pre-process a sweep: apply range-adaptive thresholding (reflectivity
/// only) and spatial texture filtering (TDBZ) to remove ground clutter
/// and anomalous propagation. Returns a cleaned copy of the sweep.
fn clean_sweep(sweep: &SweepData, product: Product) -> SweepData {
    let mut cleaned = SweepData {
        elevation_deg: sweep.elevation_deg,
        radials: sweep.radials.clone(),
    };

    // --- Range-adaptive dBZ floor (reflectivity only) ---
    // Mirrors the NEXRAD Clutter Editor's range-zone approach:
    //   zone 1 (0–20 km):  reject below 20 dBZ
    //   zone 2 (20–80 km): reject below 10 dBZ
    //   zone 3 (>80 km):   reject below  5 dBZ
    if product == Product::Reflectivity {
        for radial in &mut cleaned.radials {
            for (i, gate) in radial.gates.iter_mut().enumerate() {
                let range_km = FIRST_GATE_KM + i as f32 * GATE_SPACING_KM;
                let floor = if range_km < 20.0 {
                    20.0
                } else if range_km < 80.0 {
                    10.0
                } else {
                    5.0
                };
                if let Some(v) = *gate
                    && v < floor
                {
                    *gate = None;
                }
            }
        }
    }

    // --- Spatial texture filter (TDBZ) ---
    // The CMD algorithm's primary clutter discriminator: compute the
    // mean squared difference between adjacent gates in a 9-gate kernel.
    // Clutter has jagged, high-variance returns; weather is smooth.
    const TDBZ_KERNEL: usize = 9;
    const TDBZ_THRESHOLD: f32 = 25.0; // dBZ² for refl, (m/s)² for vel
    for radial in &mut cleaned.radials {
        let n = radial.gates.len();
        let half = TDBZ_KERNEL / 2;
        let mut tdbz = vec![0.0f32; n];
        for (i, tdbz_val) in tdbz.iter_mut().enumerate().take(n) {
            let start = i.saturating_sub(half);
            let end = (i + half + 1).min(n);
            if end - start < 2 {
                continue;
            }
            let mut sum_sq = 0.0f32;
            let mut count = 0u32;
            for j in start..end - 1 {
                if let (Some(a), Some(b)) = (radial.gates[j], radial.gates[j + 1]) {
                    sum_sq += (a - b).powi(2);
                    count += 1;
                }
            }
            if count > 0 {
                *tdbz_val = sum_sq / count as f32;
            }
        }
        for (i, &tdbz_val) in tdbz.iter().enumerate() {
            if tdbz_val > TDBZ_THRESHOLD {
                radial.gates[i] = None;
            }
        }
    }

    cleaned
}

/// Morphological closing on the rasterized image: dilate then erode to
/// fill small gaps (e.g. from missing split-cut radials) while preserving
/// the overall shape of storm features.
fn morphological_close(pixels: &mut [Color32], size_px: usize, radius: usize) {
    let s = size_px as i32;
    let r = radius as i32;
    let original = pixels.to_vec();

    // Build binary mask of valid (non-transparent) pixels.
    let mut mask = vec![false; pixels.len()];
    for (i, p) in original.iter().enumerate() {
        mask[i] = *p != Color32::TRANSPARENT;
    }

    // Step 1 — dilate: any pixel within `radius` of a valid pixel becomes
    // valid, filling small holes and bridging narrow gaps.
    let mut dilated = mask.clone();
    for y in 0..s {
        for x in 0..s {
            let idx = (y * s + x) as usize;
            if mask[idx] {
                continue;
            }
            let mut found = false;
            'outer: for dy in -r..=r {
                for dx in -r..=r {
                    let nx = x + dx;
                    let ny = y + dy;
                    if nx >= 0 && nx < s && ny >= 0 && ny < s
                        && mask[(ny * s + nx) as usize]
                    {
                        found = true;
                        break 'outer;
                    }
                }
            }
            dilated[idx] = found;
        }
    }

    // Step 2 — erode: any pixel within `radius` of an invalid pixel
    // becomes invalid, shrinking back to approximately the original
    // shapes but with small gaps now filled.
    let mut closed = dilated.clone();
    for y in 0..s {
        for x in 0..s {
            let idx = (y * s + x) as usize;
            if !dilated[idx] {
                continue;
            }
            let mut invalid = false;
            'outer: for dy in -r..=r {
                for dx in -r..=r {
                    let nx = x + dx;
                    let ny = y + dy;
                    if nx >= 0 && nx < s && ny >= 0 && ny < s
                        && !dilated[(ny * s + nx) as usize]
                    {
                        invalid = true;
                        break 'outer;
                    }
                }
            }
            closed[idx] = !invalid;
        }
    }

    // Step 3 — fill newly-closed pixels with the colour of the nearest
    // valid pixel from the original image.
    for y in 0..s {
        for x in 0..s {
            let idx = (y * s + x) as usize;
            if closed[idx] && !mask[idx] {
                let mut best = Color32::TRANSPARENT;
                let mut best_dist = i32::MAX;
                for dy in -r..=r {
                    for dx in -r..=r {
                        let nx = x + dx;
                        let ny = y + dy;
                        if nx >= 0 && nx < s && ny >= 0 && ny < s {
                            let nidx = (ny * s + nx) as usize;
                            if mask[nidx] {
                                let dist = dx * dx + dy * dy;
                                if dist < best_dist {
                                    best_dist = dist;
                                    best = original[nidx];
                                }
                            }
                        }
                    }
                }
                pixels[idx] = best;
            }
        }
    }
}

/// Rasterize one sweep to a square RGBA image, radar at center, north up.
/// Applies range-adaptive thresholding and spatial texture filtering to
/// suppress ground clutter, bilinear radial interpolation to fill wedge
/// gaps, then morphological close + despeckle as final cleanup.
pub fn rasterize(
    sweep: &SweepData,
    product: Product,
    size_px: usize,
    max_range_km: f32,
) -> ColorImage {
    // Pre-process: remove clutter at the gate level before rasterization.
    let sweep = clean_sweep(sweep, product);

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
            let (i1, i2, w1, w2) =
                nearest_two_radial_indices(&sorted_azimuths, azimuth);
            let radial1 = &sweep.radials[order[i1]];
            let radial2 = &sweep.radials[order[i2]];

            let gate = ((range_km - FIRST_GATE_KM) / GATE_SPACING_KM) as usize;
            let v1 = radial1.gates.get(gate).and_then(|v| *v);
            let v2 = radial2.gates.get(gate).and_then(|v| *v);

            let value = match (v1, v2) {
                (Some(a), Some(b)) => Some(a * w1 + b * w2),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            };
            if let Some(value) = value {
                pixels[py * size_px + px] = color_of(value);
            }
        }
    }

    // Fill small gaps from missing split-cut radials.
    morphological_close(&mut pixels, size_px, 2);
    // Remove any remaining isolated noise speckle.
    despeckle(&mut pixels, size_px, 2);

    ColorImage::new([size_px, size_px], pixels)
}
/// blend weights (summing to 1.0). The closer radial gets more weight.
/// Wraps around 0/360. If only one radial exists, both indices are 0 with
/// weights 1.0 and 0.0.
pub(crate) fn nearest_two_radial_indices(
    sorted_azimuths: &[f32],
    az: f32,
) -> (usize, usize, f32, f32) {
    const MAX_GAP_DEG: f32 = 3.0; // don't interpolate across gaps > 3°
    let n = sorted_azimuths.len();
    if n == 1 {
        return (0, 0, 1.0, 0.0);
    }
    let (before, after) = match sorted_azimuths.binary_search_by(|a| a.total_cmp(&az)) {
        Ok(i) => {
            // Exact hit: the matched radial gets full weight.
            return (i, (i + 1) % n, 1.0, 0.0);
        }
        Err(i) => {
            let before = (i + n - 1) % n;
            let after = i % n;
            (before, after)
        }
    };
    let d1 = angular_distance(sorted_azimuths[before], az);
    let d2 = angular_distance(sorted_azimuths[after], az);
    let gap = angular_distance(sorted_azimuths[before], sorted_azimuths[after]);
    if gap > MAX_GAP_DEG {
        // Radials too far apart — use nearest-neighbour.
        if d1 <= d2 {
            (before, before, 1.0, 0.0)
        } else {
            (after, after, 1.0, 0.0)
        }
    } else {
        let total = d1 + d2;
        if total < 1e-6 {
            (before, after, 1.0, 0.0)
        } else {
            (before, after, d2 / total, d1 / total)
        }
    }
}

fn angular_distance(a: f32, b: f32) -> f32 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

/// Remove isolated noise pixels: any non-transparent pixel with fewer than
/// `min_neighbors` non-transparent neighbours (out of 8) is set to
/// transparent. Operates on a copy so existing pixels don't influence the
/// decision for later pixels.
fn despeckle(pixels: &mut [Color32], size_px: usize, min_neighbors: usize) {
    let original = pixels.to_vec();
    let s = size_px as i32;
    for y in 0..s {
        for x in 0..s {
            let idx = (y * s + x) as usize;
            if original[idx] == Color32::TRANSPARENT {
                continue;
            }
            let mut neighbors = 0u8;
            for dy in -1..=1i32 {
                for dx in -1..=1i32 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x + dx;
                    let ny = y + dy;
                    if nx >= 0 && nx < s && ny >= 0 && ny < s
                        && original[(ny * s + nx) as usize] != Color32::TRANSPARENT
                    {
                        neighbors += 1;
                    }
                }
            }
            if (neighbors as usize) < min_neighbors {
                pixels[idx] = Color32::TRANSPARENT;
            }
        }
    }
}

/// Draw wind probability contours as colored outlines.
fn draw_wind_prob_overlay(
    painter: &egui::Painter,
    center: egui::Pos2,
    px_per_km: f32,
    site: &crate::geo::RadarSite,
    contours: &[crate::nhc::WindProbContour],
    visible: bool,
    _label: &str,
) {
    if !visible || contours.is_empty() {
        return;
    }

    for contour in contours {
        // Color based on probability: low = cool blue, high = hot red.
        let mid_prob = (contour.prob_low + contour.prob_high) as f32 / 2.0;
        let t = (mid_prob / 100.0).clamp(0.0, 1.0);
        let color = if t < 0.5 {
            // Blue to yellow
            let r = (t * 2.0 * 255.0) as u8;
            let g = (t * 2.0 * 200.0) as u8;
            let b = (255.0 - t * 2.0 * 200.0) as u8;
            Color32::from_rgba_premultiplied(r, g, b, 160)
        } else {
            // Yellow to red
            let r = 255u8;
            let g = ((1.0 - (t - 0.5) * 2.0) * 200.0) as u8;
            let b = 0u8;
            Color32::from_rgba_premultiplied(r, g, b, 160)
        };

        for ring in &contour.rings {
            if ring.len() < 2 {
                continue;
            }
            // Draw as line segments — wind prob contours are concave
            // polygons, so convex_polygon would create meaningless blobs.
            for pair in ring.windows(2) {
                let (lat1, lon1) = pair[0];
                let (lat2, lon2) = pair[1];
                let (r1, b1) =
                    crate::geo::range_bearing(site.lat, site.lon, lat1, lon1);
                let (r2, b2) =
                    crate::geo::range_bearing(site.lat, site.lon, lat2, lon2);
                // Use a generous filter — these are basin-wide contours
                // meant to be viewed when zoomed out.
                if r1 as f32 > MAX_RANGE_KM * 5.0
                    && r2 as f32 > MAX_RANGE_KM * 5.0
                {
                    continue;
                }
                let (dx1, dy1) = crate::geo::polar_to_offset(
                    b1 as f32,
                    r1 as f32,
                    px_per_km,
                );
                let (dx2, dy2) = crate::geo::polar_to_offset(
                    b2 as f32,
                    r2 as f32,
                    px_per_km,
                );
                let a = center + egui::vec2(dx1, dy1);
                let b = center + egui::vec2(dx2, dy2);
                painter.line_segment([a, b], Stroke::new(2.0, color));
            }
        }
    }
}

/// Draw arrival time contours as colored lines.
fn draw_arrival_overlay(
    painter: &egui::Painter,
    center: egui::Pos2,
    px_per_km: f32,
    site: &crate::geo::RadarSite,
    contours: &[crate::nhc::ArrivalTimeContour],
    visible: bool,
    color: Color32,
) {
    if !visible || contours.is_empty() {
        return;
    }

    for contour in contours {
        for ring in &contour.rings {
            for pair in ring.windows(2) {
                let (lat1, lon1) = pair[0];
                let (lat2, lon2) = pair[1];
                let (r1, b1) =
                    crate::geo::range_bearing(site.lat, site.lon, lat1, lon1);
                let (r2, b2) =
                    crate::geo::range_bearing(site.lat, site.lon, lat2, lon2);
                if r1 as f32 > MAX_RANGE_KM * 5.0
                    && r2 as f32 > MAX_RANGE_KM * 5.0
                {
                    continue;
                }
                let (dx1, dy1) = crate::geo::polar_to_offset(
                    b1 as f32,
                    r1 as f32,
                    px_per_km,
                );
                let (dx2, dy2) = crate::geo::polar_to_offset(
                    b2 as f32,
                    r2 as f32,
                    px_per_km,
                );
                let a = center + egui::vec2(dx1, dy1);
                let b = center + egui::vec2(dx2, dy2);
                painter.line_segment([a, b], Stroke::new(2.5, color));
            }
        }
    }
}

/// Draw the scope: radar texture, range rings every 50 km, cardinal spokes,
/// state border overlay, city markers, NWS warning/watch boxes, NHC
/// tropical cyclone overlays, station marker, scan time, and the product
/// color legend.
#[allow(clippy::too_many_arguments)]
pub fn draw_scope(
    ui: &mut egui::Ui,
    texture: Option<&egui::TextureHandle>,
    scan: Option<&crate::model::ScanData>,
    product: Product,
    borders: &[crate::borders::Ring],
    alerts: &[crate::alerts::Alert],
    gis_storms: &[&crate::nhc::StormGis],
    storm_metas: &[&crate::nhc::StormMeta],
    site: &crate::geo::RadarSite,
    pan_km: egui::Vec2,
    zoom: f32,
    overlay_opts: &OverlayOptions,
) {
    use crate::geo;
    use egui::{Align2, FontId, Rect, Stroke, pos2, vec2};

    let available = ui.available_rect_before_wrap();
    let side = available.width().min(available.height());
    let px_per_km = (side / 2.0) / MAX_RANGE_KM * zoom;
    let pan_px = pan_km * px_per_km;
    let rect = Rect::from_center_size(available.center() + pan_px, vec2(side * zoom, side * zoom));
    let center = rect.center();
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

    // State border outlines — drawn in full; the painter's clip rect
    // handles any edges that fall outside the window.
    for ring in borders {
        for pair in ring.windows(2) {
            let (lat1, lon1) = pair[0];
            let (lat2, lon2) = pair[1];
            let (range1, bearing1) =
                geo::range_bearing(site.lat, site.lon, lat1, lon1);
            let (range2, bearing2) =
                geo::range_bearing(site.lat, site.lon, lat2, lon2);
            let (dx1, dy1) = geo::polar_to_offset(bearing1 as f32, range1 as f32, px_per_km);
            let (dx2, dy2) = geo::polar_to_offset(bearing2 as f32, range2 as f32, px_per_km);
            let a = center + vec2(dx1, dy1);
            let b = center + vec2(dx2, dy2);
            painter.line_segment([a, b], Stroke::new(1.2, BORDER_COLOR));
        }
    }

    // Station marker at scope center.
    painter.circle_filled(center, 3.0, Color32::WHITE);
    painter.text(
        center + vec2(6.0, 6.0),
        Align2::LEFT_TOP,
        site.id,
        text_font.clone(),
        Color32::WHITE,
    );

    // City markers.
    for &(name, lat, lon) in geo::CITIES {
        let (range_km, bearing_deg) =
            geo::range_bearing(site.lat, site.lon, lat, lon);
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

    // NWS warning/watch polygons. Draw only the parts that fall inside the
    // radar circle, with a thick outline and a small label near each alert's
    // center.
    for alert in alerts {
        let color = alert.color;
        for ring in &alert.rings {
            for pair in ring.windows(2) {
                let (lat1, lon1) = pair[0];
                let (lat2, lon2) = pair[1];
                let (range1, bearing1) =
                    geo::range_bearing(site.lat, site.lon, lat1, lon1);
                let (range2, bearing2) =
                    geo::range_bearing(site.lat, site.lon, lat2, lon2);
                let (dx1, dy1) = geo::polar_to_offset(bearing1 as f32, range1 as f32, px_per_km);
                let (dx2, dy2) = geo::polar_to_offset(bearing2 as f32, range2 as f32, px_per_km);
                let a = center + vec2(dx1, dy1);
                let b = center + vec2(dx2, dy2);
                let radius_px = MAX_RANGE_KM * px_per_km;
                for (p1, p2) in crate::alerts::circle_subsegments(a - center, b - center, radius_px)
                {
                    painter.line_segment([center + p1, center + p2], Stroke::new(2.5, color));
                }
            }
        }

        // Label each visible alert near the average of its ring vertices that
        // are inside the display radius.
        let mut label_sum = vec2(0.0, 0.0);
        let mut label_count = 0;
        for ring in &alert.rings {
            for &(lat, lon) in ring {
                let (range_km, bearing_deg) =
                    geo::range_bearing(site.lat, site.lon, lat, lon);
                if range_km as f32 > MAX_RANGE_KM {
                    continue;
                }
                let (dx, dy) = geo::polar_to_offset(bearing_deg as f32, range_km as f32, px_per_km);
                label_sum += vec2(dx, dy);
                label_count += 1;
            }
        }
        if label_count > 0 {
            let label_pos = center + label_sum / (label_count as f32);
            painter.text(
                label_pos,
                Align2::CENTER_CENTER,
                &alert.event,
                FontId::monospace(11.0),
                Color32::WHITE,
            );
        }
    }

    // ── NHC tropical cyclone overlays ─────────────────────────────────
    // Draw GIS overlays (cone, track, watches/warnings) for each storm.
    for storm in gis_storms {
        // Forecast cone — red outline with slight transparency.
        let cone_color = Color32::from_rgba_premultiplied(230, 40, 40, 180);
        for ring in &storm.cone {
            for pair in ring.windows(2) {
                let (lat1, lon1) = pair[0];
                let (lat2, lon2) = pair[1];
                let (r1, b1) = geo::range_bearing(site.lat, site.lon, lat1, lon1);
                let (r2, b2) = geo::range_bearing(site.lat, site.lon, lat2, lon2);
                let (dx1, dy1) = geo::polar_to_offset(b1 as f32, r1 as f32, px_per_km);
                let (dx2, dy2) = geo::polar_to_offset(b2 as f32, r2 as f32, px_per_km);
                let a = center + vec2(dx1, dy1);
                let b = center + vec2(dx2, dy2);
                painter.line_segment([a, b], Stroke::new(1.8, cone_color));
            }
        }

        // Forecast track — dashed white line.
        for ring in &storm.track {
            for pair in ring.windows(2) {
                let (lat1, lon1) = pair[0];
                let (lat2, lon2) = pair[1];
                let (r1, b1) = geo::range_bearing(site.lat, site.lon, lat1, lon1);
                let (r2, b2) = geo::range_bearing(site.lat, site.lon, lat2, lon2);
                let (dx1, dy1) = geo::polar_to_offset(b1 as f32, r1 as f32, px_per_km);
                let (dx2, dy2) = geo::polar_to_offset(b2 as f32, r2 as f32, px_per_km);
                let a = center + vec2(dx1, dy1);
                let b = center + vec2(dx2, dy2);
                // Dashed effect: draw short segments.
                let dir = b - a;
                let len = dir.length();
                if len > 0.0 {
                    let step = 8.0;
                    let mut t = 0.0;
                    while t < len {
                        let end = (t + step * 0.5).min(len);
                        let p1 = a + dir * (t / len);
                        let p2 = a + dir * (end / len);
                        painter.line_segment(
                            [p1, p2],
                            Stroke::new(2.0, Color32::WHITE),
                        );
                        t += step;
                    }
                }
            }
        }

        // Forecast points — hurricane symbols based on intensity.
        // Find matching meta for intensity info.
        let intensity = storm_metas
            .iter()
            .find(|m| m.name == storm.name || m.id.to_uppercase().contains(&storm.name.to_uppercase()))
            .map(|m| m.intensity_kt)
            .unwrap_or(0);

        for &(lat, lon, ref label) in &storm.points {
            let (r, bearing) = geo::range_bearing(site.lat, site.lon, lat, lon);
            let (dx, dy) = geo::polar_to_offset(bearing as f32, r as f32, px_per_km);
            let pos = center + vec2(dx, dy);

            // Draw hurricane symbol: filled circle with size based on intensity.
            let radius = if intensity >= 64 {
                5.0 // Hurricane
            } else if intensity >= 34 {
                4.0 // Tropical storm
            } else {
                3.0 // Tropical depression
            };
            let point_color = if intensity >= 64 {
                Color32::from_rgb(255, 200, 0) // Gold for hurricane
            } else if intensity >= 34 {
                Color32::from_rgb(255, 80, 80) // Red for TS
            } else {
                Color32::from_rgb(100, 200, 255) // Blue for TD
            };
            painter.circle_filled(pos, radius, point_color);
            painter.circle_stroke(pos, radius + 1.0, Stroke::new(1.0, Color32::WHITE));

            if !label.is_empty() {
                painter.text(
                    pos + vec2(6.0, -6.0),
                    Align2::LEFT_BOTTOM,
                    label,
                    FontId::monospace(10.0),
                    Color32::WHITE,
                );
            }
        }

        // Watches/warnings — coloured coastal segments.
        for (ring, ww_type) in &storm.watches_warnings {
            let color = match ww_type.as_str() {
                "HWR" => Color32::from_rgb(255, 0, 0),     // Hurricane Warning
                "HWA" => Color32::from_rgb(255, 127, 127), // Hurricane Watch
                "TWR" => Color32::from_rgb(0, 77, 168),    // TS Warning
                "TWA" => Color32::from_rgb(255, 255, 0),   // TS Watch
                _ => Color32::from_rgb(200, 200, 200),
            };
            for pair in ring.windows(2) {
                let (lat1, lon1) = pair[0];
                let (lat2, lon2) = pair[1];
                let (r1, b1) = geo::range_bearing(site.lat, site.lon, lat1, lon1);
                let (r2, b2) = geo::range_bearing(site.lat, site.lon, lat2, lon2);
                let (dx1, dy1) = geo::polar_to_offset(b1 as f32, r1 as f32, px_per_km);
                let (dx2, dy2) = geo::polar_to_offset(b2 as f32, r2 as f32, px_per_km);
                let a = center + vec2(dx1, dy1);
                let b = center + vec2(dx2, dy2);
                painter.line_segment([a, b], Stroke::new(3.0, color));
            }
        }

        // Storm name label near the first forecast point.
        if let Some(&(lat, lon, _)) = storm.points.first() {
            let (r, bearing) = geo::range_bearing(site.lat, site.lon, lat, lon);
            let (dx, dy) = geo::polar_to_offset(bearing as f32, r as f32, px_per_km);
            let pos = center + vec2(dx, dy);
            painter.text(
                pos + vec2(8.0, -18.0),
                Align2::LEFT_BOTTOM,
                format!("{} {}", storm.name, storm.storm_type),
                FontId::monospace(12.0),
                Color32::WHITE,
            );
        }
    }

    // Draw current storm position markers from metadata.
    for meta in storm_metas {
        let (r, bearing) = geo::range_bearing(site.lat, site.lon, meta.lat, meta.lon);
        let (dx, dy) = geo::polar_to_offset(bearing as f32, r as f32, px_per_km);
        let pos = center + vec2(dx, dy);

        // Only draw if within range.
        if r as f32 > MAX_RANGE_KM * 1.2 {
            continue;
        }

        // Hurricane symbol based on classification.
        let is_hurricane = meta.classification == "HU";
        let is_ts = meta.classification == "TS";

        let symbol_radius = if is_hurricane {
            7.0
        } else if is_ts {
            5.0
        } else {
            4.0
        };

        let symbol_color = if is_hurricane {
            Color32::from_rgb(255, 200, 0) // Gold
        } else if is_ts {
            Color32::from_rgb(255, 60, 60) // Red
        } else {
            Color32::from_rgb(80, 180, 255) // Blue
        };

        // Draw hurricane symbol (double circle for hurricane).
        if is_hurricane {
            painter.circle_filled(pos, symbol_radius + 2.0, Color32::from_rgba_premultiplied(255, 200, 0, 80));
        }
        painter.circle_filled(pos, symbol_radius, symbol_color);
        painter.circle_stroke(pos, symbol_radius, Stroke::new(1.5, Color32::WHITE));

        // Label.
        let label = format!(
            "{} {} {}kt",
            meta.name, meta.classification, meta.intensity_kt
        );
        painter.text(
            pos + vec2(symbol_radius + 4.0, -symbol_radius),
            Align2::LEFT_BOTTOM,
            label,
            FontId::monospace(11.0),
            Color32::WHITE,
        );
    }

    // ── NHC wind probability overlays ──────────────────────────────
    // Draw wind probability contours as semi-transparent filled polygons.
    draw_wind_prob_overlay(
        &painter,
        center,
        px_per_km,
        site,
        &overlay_opts.wind_probs_34kt,
        overlay_opts.show_wind_probs_34kt,
        "34kt Wind Prob",
    );
    draw_wind_prob_overlay(
        &painter,
        center,
        px_per_km,
        site,
        &overlay_opts.wind_probs_50kt,
        overlay_opts.show_wind_probs_50kt,
        "50kt Wind Prob",
    );
    draw_wind_prob_overlay(
        &painter,
        center,
        px_per_km,
        site,
        &overlay_opts.wind_probs_64kt,
        overlay_opts.show_wind_probs_64kt,
        "64kt Wind Prob",
    );

    // ── NHC arrival time overlays ──────────────────────────────────
    draw_arrival_overlay(
        &painter,
        center,
        px_per_km,
        site,
        &overlay_opts.earliest_arrival,
        overlay_opts.show_earliest_arrival,
        Color32::from_rgb(255, 165, 0),
    );
    draw_arrival_overlay(
        &painter,
        center,
        px_per_km,
        site,
        &overlay_opts.most_likely_arrival,
        overlay_opts.show_most_likely_arrival,
        Color32::from_rgb(255, 255, 0),
    );

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
    use super::{despeckle, nearest_two_radial_indices, rasterize};
    use crate::model::{Product, RadialData, SweepData};
    use egui::Color32;

    /// Four cardinal radials with distinct dBZ so quadrants are testable.
    /// Values are chosen to survive the range-adaptive threshold:
    ///   < 20 km → ≥ 20 dBZ, 20–80 km → ≥ 10 dBZ, > 80 km → ≥ 5 dBZ.
    /// 200 gates × 0.25 km reach 2.125 + 50 = 52.125 km.
    fn synthetic_sweep() -> SweepData {
        let radial = |az: f32, dbz: f32| RadialData {
            azimuth_deg: az,
            gates: vec![Some(dbz); 200],
        };
        SweepData {
            elevation_deg: 0.5,
            radials: vec![
                radial(0.0, 25.0),   // north: green  0x01c501
                radial(90.0, 30.0),  // east:  dark green 0x008e00
                radial(180.0, 55.0), // south: red    0xfd0000
                radial(270.0, 4.0),  // west:  below threshold → transparent
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
        assert_eq!(at(64, 32), Color32::from_rgb(0x01, 0xc5, 0x01), "north 25 dBZ");
        assert_eq!(at(96, 64), Color32::from_rgb(0x00, 0x8e, 0x00), "east 30 dBZ");
        assert_eq!(at(64, 96), Color32::from_rgb(0xd4, 0x00, 0x00), "south 55 dBZ");
        assert_eq!(at(32, 64), Color32::TRANSPARENT, "west 4 dBZ below threshold");
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
    fn nearest_two_radials_wraps_around_north() {
        let azimuths = [10.0, 180.0, 350.0];
        // 355° is between 350° and 10° but gap (20°) > 3° max →
        // falls back to nearest-neighbour (350°).
        let (i1, i2, w1, _w2) = nearest_two_radial_indices(&azimuths, 355.0);
        assert_eq!(i1, 2); // 350°
        assert_eq!(i2, 2); // same — nearest-neighbour
        assert!((w1 - 1.0).abs() < 1e-6);
        // 3° is between 350° and 10° but gap > 3° → nearest-neighbour (10°).
        let (i1, i2, w1, _w2) = nearest_two_radial_indices(&azimuths, 3.0);
        assert_eq!(i1, 0); // 10°
        assert_eq!(i2, 0); // same — nearest-neighbour
        assert!((w1 - 1.0).abs() < 1e-6);
        // Exact hit at 180° returns that index with full weight.
        let (i1, _i2, w1, _w2) = nearest_two_radial_indices(&azimuths, 180.0);
        assert_eq!(i1, 1);
        assert!((w1 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn nearest_two_radials_single_radial() {
        let (i1, i2, w1, w2) = nearest_two_radial_indices(&[42.0], 100.0);
        assert_eq!(i1, 0);
        assert_eq!(i2, 0);
        assert!((w1 - 1.0).abs() < 1e-6);
        assert!((w2 - 0.0).abs() < 1e-6);
    }

    #[test]
    fn despeckle_removes_isolated_pixels() {
        let size = 8;
        let mut pixels = vec![Color32::TRANSPARENT; size * size];
        // Single isolated pixel at (4,4).
        pixels[4 * size + 4] = Color32::RED;
        // 2-pixel cluster at (1,1) and (1,2) — each has only 1 neighbour.
        pixels[size + 1] = Color32::GREEN;
        pixels[2 * size + 1] = Color32::GREEN;
        // 3-pixel line at (6,0)-(6,2) — middle has 2 neighbours, ends have 1.
        pixels[6] = Color32::BLUE;
        pixels[size + 6] = Color32::BLUE;
        pixels[2 * size + 6] = Color32::BLUE;

        despeckle(&mut pixels, size, 2);

        // Single pixel removed.
        assert_eq!(pixels[4 * size + 4], Color32::TRANSPARENT);
        // 2-pixel cluster removed (each has only 1 neighbour).
        assert_eq!(pixels[size + 1], Color32::TRANSPARENT);
        assert_eq!(pixels[2 * size + 1], Color32::TRANSPARENT);
        // 3-pixel line: middle survives, ends removed.
        assert_eq!(pixels[6], Color32::TRANSPARENT);
        assert_eq!(pixels[size + 6], Color32::BLUE);
        assert_eq!(pixels[2 * size + 6], Color32::TRANSPARENT);
    }
}
