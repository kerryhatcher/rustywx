//! Radar scope rendering adapted for Ply/macroquad.
//! Uses render_to_texture for the radar sweep and macroquad draw calls
//! for overlays (range rings, borders, markers, etc.).

use crate::colors;
use crate::geo::{self, RadarSite};
use crate::model::{Product, SweepData};
use ply_engine::prelude::*;

/// Level II super-res gate geometry.
pub const FIRST_GATE_KM: f32 = 2.125;
pub const GATE_SPACING_KM: f32 = 0.25;
/// Display radius of the scope.
pub const MAX_RANGE_KM: f32 = 230.0;
/// Side length of the rasterized radar texture.
pub const RASTER_SIZE_PX: usize = 1024;

// ---------------------------------------------------------------------------
// Rasterization
// ---------------------------------------------------------------------------

fn clean_sweep(sweep: &SweepData, product: Product) -> SweepData {
    let mut cleaned = SweepData {
        elevation_deg: sweep.elevation_deg,
        radials: sweep.radials.clone(),
    };

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

    const TDBZ_KERNEL: usize = 9;
    const TDBZ_THRESHOLD: f32 = 25.0;
    for radial in &mut cleaned.radials {
        let n = radial.gates.len();
        let half = TDBZ_KERNEL / 2;
        let mut tdbz = vec![0.0f32; n];
        for (i, tdbz_slot) in tdbz.iter_mut().enumerate() {
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
                *tdbz_slot = sum_sq / count as f32;
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

/// Rasterize one sweep to raw RGBA bytes.
pub fn rasterize(
    sweep: &SweepData,
    product: Product,
    size_px: usize,
    max_range_km: f32,
) -> Vec<u8> {
    let sweep = clean_sweep(sweep, product);
    let mut pixels = vec![0u8; size_px * size_px * 4];

    if sweep.radials.is_empty() {
        return pixels;
    }

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
        Product::Reflectivity => colors::dbz_color as fn(f32) -> [u8; 4],
        Product::Velocity => colors::velocity_color as fn(f32) -> [u8; 4],
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

            let azimuth = dx.atan2(-dy).to_degrees().rem_euclid(360.0);
            let (i1, i2, w1, w2) = nearest_two_radial_indices(&sorted_azimuths, azimuth);
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
                let c = color_of(value);
                let idx = (py * size_px + px) * 4;
                pixels[idx] = c[0];
                pixels[idx + 1] = c[1];
                pixels[idx + 2] = c[2];
                pixels[idx + 3] = c[3];
            }
        }
    }

    morphological_close(&mut pixels, size_px, 2);
    despeckle(&mut pixels, size_px, 2);

    pixels
}

fn morphological_close(pixels: &mut [u8], size_px: usize, radius: usize) {
    let s = size_px as i32;
    let r = radius as i32;
    let original = pixels.to_vec();

    let mut mask = vec![false; size_px * size_px];
    for i in 0..size_px * size_px {
        mask[i] = original[i * 4 + 3] != 0;
    }

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
                    if nx >= 0 && nx < s && ny >= 0 && ny < s && mask[(ny * s + nx) as usize] {
                        found = true;
                        break 'outer;
                    }
                }
            }
            dilated[idx] = found;
        }
    }

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
                    if nx >= 0 && nx < s && ny >= 0 && ny < s && !dilated[(ny * s + nx) as usize] {
                        invalid = true;
                        break 'outer;
                    }
                }
            }
            closed[idx] = !invalid;
        }
    }

    for y in 0..s {
        for x in 0..s {
            let idx = (y * s + x) as usize;
            if closed[idx] && !mask[idx] {
                let mut best_dist = i32::MAX;
                let mut best_idx = idx;
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
                                    best_idx = nidx;
                                }
                            }
                        }
                    }
                }
                let src = best_idx * 4;
                let dst = idx * 4;
                pixels[dst] = original[src];
                pixels[dst + 1] = original[src + 1];
                pixels[dst + 2] = original[src + 2];
                pixels[dst + 3] = original[src + 3];
            }
        }
    }
}

fn despeckle(pixels: &mut [u8], size_px: usize, min_neighbors: usize) {
    let original = pixels.to_vec();
    let s = size_px as i32;
    for y in 0..s {
        for x in 0..s {
            let idx = (y * s + x) as usize;
            if original[idx * 4 + 3] == 0 {
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
                    if nx >= 0
                        && nx < s
                        && ny >= 0
                        && ny < s
                        && original[((ny * s + nx) as usize) * 4 + 3] != 0
                    {
                        neighbors += 1;
                    }
                }
            }
            if (neighbors as usize) < min_neighbors {
                let dst = idx * 4;
                pixels[dst] = 0;
                pixels[dst + 1] = 0;
                pixels[dst + 2] = 0;
                pixels[dst + 3] = 0;
            }
        }
    }
}

pub(crate) fn nearest_two_radial_indices(
    sorted_azimuths: &[f32],
    az: f32,
) -> (usize, usize, f32, f32) {
    const MAX_GAP_DEG: f32 = 3.0;
    let n = sorted_azimuths.len();
    if n == 1 {
        return (0, 0, 1.0, 0.0);
    }
    match sorted_azimuths.binary_search_by(|a| a.total_cmp(&az)) {
        Ok(i) => (i, (i + 1) % n, 1.0, 0.0),
        Err(i) => {
            let before = (i + n - 1) % n;
            let after = i % n;
            let d1 = angular_distance(sorted_azimuths[before], az);
            let d2 = angular_distance(sorted_azimuths[after], az);
            let gap = angular_distance(sorted_azimuths[before], sorted_azimuths[after]);
            if gap > MAX_GAP_DEG {
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
    }
}

fn angular_distance(a: f32, b: f32) -> f32 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

// ---------------------------------------------------------------------------
// Overlay drawing (uses macroquad directly — called inside render_to_texture)
// ---------------------------------------------------------------------------

/// Draw the full radar scope. Called inside a `render_to_texture` closure.
pub fn draw_scope_to_texture(
    radar_texture: Option<&Texture2D>,
    site: &RadarSite,
    pan_km: (f32, f32),
    zoom: f32,
) {
    let side = screen_width().min(screen_height());
    let px_per_km = (side / 2.0) / MAX_RANGE_KM * zoom;
    let center_x = screen_width() / 2.0 + pan_km.0 * px_per_km;
    let center_y = screen_height() / 2.0 + pan_km.1 * px_per_km;

    clear_background(BLACK);

    // Radar texture
    if let Some(tex) = radar_texture {
        let tex_size = side * zoom;
        draw_texture_ex(
            tex,
            center_x - tex_size / 2.0,
            center_y - tex_size / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(tex_size, tex_size)),
                ..Default::default()
            },
        );
    }

    let grid_color = MacroquadColor::from_rgba(0x2a, 0x3a, 0x2f, 255);
    let grid_text_color = MacroquadColor::from_rgba(0x5f, 0x8a, 0x6a, 255);

    // Range rings every 50 km
    let mut ring_km = 50.0;
    while ring_km <= MAX_RANGE_KM {
        draw_circle_lines(center_x, center_y, ring_km * px_per_km, 1.0, grid_color);
        draw_text(
            format!("{ring_km:.0} km"),
            center_x + 4.0,
            center_y - ring_km * px_per_km,
            12.0,
            grid_text_color,
        );
        ring_km += 50.0;
    }

    // Cardinal spokes
    for (azimuth, label) in [(0.0, "N"), (90.0, "E"), (180.0, "S"), (270.0, "W")] {
        let (dx, dy) = geo::polar_to_offset(azimuth, MAX_RANGE_KM, px_per_km);
        draw_line(
            center_x,
            center_y,
            center_x + dx,
            center_y + dy,
            1.0,
            grid_color,
        );
        let (lx, ly) = geo::polar_to_offset(azimuth, MAX_RANGE_KM * 0.96, px_per_km);
        let text_dims = measure_text(label, None, 12, 1.0);
        draw_text(
            label,
            center_x + lx - text_dims.width / 2.0,
            center_y + ly - text_dims.height / 2.0,
            12.0,
            grid_text_color,
        );
    }

    // Station marker at center
    draw_circle(center_x, center_y, 3.0, WHITE);
    draw_text(site.id, center_x + 6.0, center_y + 6.0, 12.0, WHITE);

    // City markers — check all cities within range (~1000 haversine
    // calculations per frame is trivial).
    let city_color = MacroquadColor::from_rgba(0xdd, 0xdd, 0xaa, 255);
    for &(name, lat, lon) in geo::CITIES.iter() {
        let (range_km, bearing_deg) = geo::range_bearing(site.lat, site.lon, lat, lon);
        if range_km as f32 > MAX_RANGE_KM {
            continue;
        }
        let (dx, dy) = geo::polar_to_offset(bearing_deg as f32, range_km as f32, px_per_km);
        draw_circle_lines(center_x + dx, center_y + dy, 3.5, 1.5, city_color);
        draw_text(
            name,
            center_x + dx + 6.0,
            center_y + dy - 6.0,
            10.0,
            city_color,
        );
    }
}
