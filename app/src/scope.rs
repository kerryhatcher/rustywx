//! Radar scope rendering adapted for Ply/macroquad.
//! Uses render_to_texture for the radar sweep and macroquad draw calls
//! for overlays (range rings, borders, markers, etc.).

use crate::alerts::Alert;
use crate::borders::Ring;
use crate::colors;
use crate::dealias::dealias_sweep;
use crate::geo::{self, RadarSite};
use crate::location::Coords;
use crate::model::{Product, RadialData, SweepData};
use crate::nhc::{ArrivalTimeContour, NhcBundle, WindProbContour};
use crate::nonmet;
use macroquad::math::Vec2;
use ply_engine::prelude::*;

/// Level II super-res gate geometry. Legacy fallback/default only — real
/// per-tilt/per-product geometry rides on `SweepData::first_gate_km` /
/// `gate_spacing_km` (populated by `model::from_sweeps` from the moment's
/// own `first_gate_range_km()`/`gate_interval_km()`). Kept here because
/// `model::default_first_gate_km`/`default_gate_spacing_km` and the
/// synthetic sweep in `main.rs` still reference these values.
pub const FIRST_GATE_KM: f32 = 2.125;
pub const GATE_SPACING_KM: f32 = 0.25;
/// Display radius of the scope.
pub const MAX_RANGE_KM: f32 = 230.0;
/// Side length of the rasterized radar texture.
pub const RASTER_SIZE_PX: usize = 1024;

// ---------------------------------------------------------------------------
// Rasterization
// ---------------------------------------------------------------------------

/// QC pipeline inputs for `clean_sweep`/`rasterize`. Bundles what was
/// previously 9-11 positional args; mechanical no-op refactor (same fields,
/// same behavior) — see `docs/fuzzy-nonmet-classifier-plan.md` step 2.
/// `zdr_sweep`/`phidp_sweep`/`nonmet_fuzzy_enabled`/`nonmet_threshold` are
/// step 3's fuzzy non-met classifier (`nonmet.rs`); when disabled, `cc_sweep`
/// alone still drives the fallback CC-only gate below.
///
/// Derives `Default` (all fields have a sane zero/`None`/`false` default) so
/// tests can spread `..Default::default()` for the fields they don't
/// exercise instead of restating all of them.
#[derive(Default)]
pub struct QcConfig<'a> {
    pub tdbz_kernel_size: usize,
    pub cc_sweep: Option<&'a SweepData>,
    pub cc_gate_enabled: bool,
    pub cc_gate_threshold: f32,
    pub refl_floor_enabled: bool,
    pub refl_floor_dbz: f32,
    pub vel_dealias_enabled: bool,
    pub vel_sd_censor_enabled: bool,
    pub vel_sd_threshold: f32,
    pub zdr_sweep: Option<&'a SweepData>,
    pub phidp_sweep: Option<&'a SweepData>,
    pub nonmet_fuzzy_enabled: bool,
    pub nonmet_threshold: f32,
}

fn clean_sweep(sweep: &SweepData, product: Product, qc: &QcConfig) -> SweepData {
    let mut cleaned = SweepData {
        elevation_deg: sweep.elevation_deg,
        radials: sweep.radials.clone(),
        first_gate_km: sweep.first_gate_km,
        gate_spacing_km: sweep.gate_spacing_km,
        nyquist_ms: sweep.nyquist_ms,
    };

    // Non-met rejection: fuzzy multi-variable classifier (CC + SD(ΦDP) +
    // SD(ZDR) + |ZDR|, see `nonmet.rs`) when enabled — it supersedes, not
    // stacks with, the single-threshold CC gate below (CC membership already
    // subsumes the CC-only gate, and running both would double-null the same
    // gates). Falls back to the CC-only gate when fuzzy is off, unchanged.
    // Runs before the dBZ floor / TDBZ block so a suppressed gate is not also
    // texture-processed.
    if product == Product::Reflectivity && qc.nonmet_fuzzy_enabled {
        fuzzy_nonmet_gate(&mut cleaned, qc);
    } else if product == Product::Reflectivity
        && qc.cc_gate_enabled
        && let Some(cc) = qc.cc_sweep
        && !cc.radials.is_empty()
    {
        // CC-gating: null Reflectivity gates whose co-located correlation
        // coefficient is below threshold (non-meteorological echo — birds,
        // chaff, AP, ground clutter). Dual-pol REF/CC come from the same
        // surveillance cut, so they share gate geometry; align by nearest
        // azimuth and equal index. Fails open per-gate: a missing/absent CC
        // sweep or an out-of-range CC gate leaves the REF gate untouched.
        // Pre-sort CC azimuths once for nearest-azimuth lookup.
        let mut cc_order: Vec<usize> = (0..cc.radials.len()).collect();
        cc_order.sort_by(|&a, &b| {
            cc.radials[a]
                .azimuth_deg
                .total_cmp(&cc.radials[b].azimuth_deg)
        });
        let cc_azimuths: Vec<f32> = cc_order
            .iter()
            .map(|&i| cc.radials[i].azimuth_deg)
            .collect();

        for radial in &mut cleaned.radials {
            // Nearest CC radial by azimuth (reuse the existing helper; take its
            // primary index, ignore the interpolation weights — CC is a QC mask,
            // interpolating it would blur the birds/precip boundary).
            let (i1, _i2, _w1, _w2) = nearest_two_radial_indices(&cc_azimuths, radial.azimuth_deg);
            let cc_radial = &cc.radials[cc_order[i1]];
            for (i, gate) in radial.gates.iter_mut().enumerate() {
                if gate.is_some()
                    && let Some(Some(cc_val)) = cc_radial.gates.get(i)
                    && *cc_val < qc.cc_gate_threshold
                {
                    *gate = None;
                }
            }
        }
    }

    // 2D Nyquist unfold — gate-to-gate then azimuth-to-azimuth (FMH-11B
    // §4.3.3 / FMH-11D §3.3.3): runs before SD-censoring so the censor's
    // scatter test sees dealiased (continuous) values rather than the raw
    // aliased sign-flips. No-op when Nyquist is unknown (`nyquist_ms <=
    // 0.0`) — see `dealias::dealias_sweep`.
    //
    // ponytail: recomputed per render, per sweep. Velocity is one sweep at a
    // time, so this is cheap; if profiling ever says otherwise, memoize on
    // the sweep at ingestion instead.
    if product == Product::Velocity && qc.vel_dealias_enabled && cleaned.nyquist_ms > 0.0 {
        let mut az_order: Vec<usize> = (0..cleaned.radials.len()).collect();
        az_order.sort_by(|&a, &b| {
            cleaned.radials[a]
                .azimuth_deg
                .total_cmp(&cleaned.radials[b].azimuth_deg)
        });
        let azimuths: Vec<f32> = az_order
            .iter()
            .map(|&i| cleaned.radials[i].azimuth_deg)
            .collect();
        let gates: Vec<Vec<Option<f32>>> = az_order
            .iter()
            .map(|&i| cleaned.radials[i].gates.clone())
            .collect();
        let mut dealiased = dealias_sweep(&azimuths, &gates, cleaned.nyquist_ms);
        for (sorted_idx, &radial_idx) in az_order.iter().enumerate() {
            cleaned.radials[radial_idx].gates = std::mem::take(&mut dealiased[sorted_idx]);
        }
    }

    // Velocity spatial-SD censoring: null a Velocity gate whose local ~9-azimuth
    // x 5-gate window has too much scatter to be a coherent Doppler return
    // (dealias residue, non-meteorological velocity noise). Two-pass
    // read-then-write so the window always reads pre-censoring values.
    if product == Product::Velocity && qc.vel_sd_censor_enabled {
        const AZ_HALF: i64 = 4;
        const GATE_HALF: usize = 2;
        const MIN_SAMPLES: u32 = 5;

        let mut az_order: Vec<usize> = (0..cleaned.radials.len()).collect();
        az_order.sort_by(|&a, &b| {
            cleaned.radials[a]
                .azimuth_deg
                .total_cmp(&cleaned.radials[b].azimuth_deg)
        });
        let n_az = az_order.len();

        if n_az > 0 {
            let mut nulls: Vec<(usize, usize)> = Vec::new();
            for (sorted_idx, &radial_idx) in az_order.iter().enumerate() {
                let n_gates = cleaned.radials[radial_idx].gates.len();
                for gate in 0..n_gates {
                    if cleaned.radials[radial_idx].gates[gate].is_none() {
                        continue;
                    }
                    let g_start = gate.saturating_sub(GATE_HALF);
                    let g_end = (gate + GATE_HALF + 1).min(n_gates);

                    let mut sum = 0.0f64;
                    let mut sum_sq = 0.0f64;
                    let mut count = 0u32;
                    for d in -AZ_HALF..=AZ_HALF {
                        let az_idx = (sorted_idx as i64 + d).rem_euclid(n_az as i64) as usize;
                        let r = &cleaned.radials[az_order[az_idx]];
                        for g in g_start..g_end.min(r.gates.len()) {
                            if let Some(v) = r.gates[g] {
                                sum += v as f64;
                                sum_sq += (v as f64) * (v as f64);
                                count += 1;
                            }
                        }
                    }
                    if count < MIN_SAMPLES {
                        continue;
                    }
                    let mean = sum / count as f64;
                    let variance = (sum_sq / count as f64) - mean * mean;
                    let sd = variance.max(0.0).sqrt();
                    if sd as f32 > qc.vel_sd_threshold {
                        nulls.push((radial_idx, gate));
                    }
                }
            }
            for (radial_idx, gate) in nulls {
                cleaned.radials[radial_idx].gates[gate] = None;
            }
        }
    }

    if product == Product::Reflectivity {
        for radial in &mut cleaned.radials {
            for (i, gate) in radial.gates.iter_mut().enumerate() {
                let range_km = sweep.first_gate_km + i as f32 * sweep.gate_spacing_km;
                let mut floor: f32 = if range_km < 20.0 {
                    20.0
                } else if range_km < 80.0 {
                    10.0
                } else {
                    5.0
                };
                if qc.refl_floor_enabled {
                    floor = floor.max(qc.refl_floor_dbz);
                }
                if let Some(v) = *gate
                    && v < floor
                {
                    *gate = None;
                }
            }
        }
    }

    // TDBZ texture filter is a Reflectivity-only QC pass (D5): ZDR/CC/PhiDP are
    // not intensity fields, so running an intensity-oriented texture filter on
    // them would punch holes in valid dual-pol data.
    if product == Product::Reflectivity {
        const TDBZ_THRESHOLD: f32 = 25.0;
        // Only null a high-texture gate when the surrounding reflectivity is also
        // LOW. Clutter/AP is high-texture-at-low-dBZ; a real convective core has a
        // sharp gradient (high texture) at high dBZ and must be preserved. This
        // mirrors the WSR-88D CMD/REC "SPIN reflectivity threshold" — texture tests
        // are gated on local mean signal so they don't punch holes in storm cores.
        // (FMH-11 Part C §3.2.10 / §4.2.4.3; see docs/research.)
        const LOW_DBZ_GATE: f32 = 35.0;
        for radial in &mut cleaned.radials {
            let n = radial.gates.len();
            let half = qc.tdbz_kernel_size / 2;
            let mut tdbz = vec![0.0f32; n];
            let mut mean = vec![f32::MAX; n]; // MAX = unknown → never counts as "low"
            for i in 0..n {
                let start = i.saturating_sub(half);
                let end = (i + half + 1).min(n);
                if end - start < 2 {
                    continue;
                }
                let mut sum_sq = 0.0f32;
                let mut diff_count = 0u32;
                let mut sum = 0.0f32;
                let mut val_count = 0u32;
                for j in start..end {
                    if let Some(v) = radial.gates[j] {
                        sum += v;
                        val_count += 1;
                        if j + 1 < end
                            && let Some(b) = radial.gates[j + 1]
                        {
                            sum_sq += (v - b).powi(2);
                            diff_count += 1;
                        }
                    }
                }
                if diff_count > 0 {
                    tdbz[i] = sum_sq / diff_count as f32;
                }
                if val_count > 0 {
                    mean[i] = sum / val_count as f32;
                }
            }
            for i in 0..n {
                if tdbz[i] > TDBZ_THRESHOLD && mean[i] < LOW_DBZ_GATE {
                    radial.gates[i] = None;
                }
            }
        }
    }

    cleaned
}

/// Sort a sweep's radials by azimuth once, returning the index permutation
/// alongside the sorted azimuth values — the shared prep behind every
/// nearest-azimuth radial lookup (`nearest_two_radial_indices`) in this file.
fn sorted_azimuths(radials: &[RadialData]) -> (Vec<usize>, Vec<f32>) {
    let mut order: Vec<usize> = (0..radials.len()).collect();
    order.sort_by(|&a, &b| radials[a].azimuth_deg.total_cmp(&radials[b].azimuth_deg));
    let azimuths = order.iter().map(|&i| radials[i].azimuth_deg).collect();
    (order, azimuths)
}

/// Fuzzy multi-variable non-met gate (`nonmet::nonmet_score`): nulls a
/// Reflectivity gate whose weighted CC / SD(ΦDP) / SD(ZDR) / |ZDR| membership
/// score clears `qc.nonmet_threshold`. Dual-pol REF/CC/ZDR/ΦDP come from the
/// same surveillance cut and share gate geometry; each aux sweep is aligned
/// to the REF radial by nearest azimuth (primary index only — a QC mask must
/// not be blurred by interpolation, same rule the CC-only gate follows).
///
/// Fails open per `nonmet_score`'s contract: any aux sweep that's absent
/// just drops out of the weighted mean. When *both* ΦDP and ZDR texture are
/// absent for a gate (the pure CC-only degrade case), the decision defers
/// to `nonmet::should_null_reflectivity_gate`'s legacy fallback — the exact
/// hard `cc < qc.cc_gate_threshold` comparison the old CC-only gate used —
/// rather than the fuzzy score, so CC-only volumes truly reduce to today's
/// gate instead of a stricter one. Leaves REF untouched if CC is also
/// absent.
fn fuzzy_nonmet_gate(cleaned: &mut SweepData, qc: &QcConfig) {
    if qc.cc_sweep.is_none() && qc.zdr_sweep.is_none() && qc.phidp_sweep.is_none() {
        return;
    }
    // Mirrors the TDBZ kernel's window so ΦDP/ZDR texture is computed over
    // the same range-neighborhood the reflectivity texture pass uses.
    let half = qc.tdbz_kernel_size / 2;

    // Nearest-azimuth lookup table per aux sweep, built once per sweep (not
    // per REF radial). ZDR/ΦDP additionally carry their gate-to-gate range
    // texture, also computed once per aux radial and reused across every REF
    // radial that nearest-azimuth-matches it (plan: "once per aux sweep").
    let cc = qc.cc_sweep.filter(|s| !s.radials.is_empty()).map(|s| {
        let (order, azimuths) = sorted_azimuths(&s.radials);
        (s, order, azimuths)
    });
    let zdr = qc.zdr_sweep.filter(|s| !s.radials.is_empty()).map(|s| {
        let (order, azimuths) = sorted_azimuths(&s.radials);
        let textures: Vec<Vec<f32>> = s
            .radials
            .iter()
            .map(|r| nonmet::range_texture(&r.gates, half, None))
            .collect();
        (s, order, azimuths, textures)
    });
    let phidp = qc.phidp_sweep.filter(|s| !s.radials.is_empty()).map(|s| {
        let (order, azimuths) = sorted_azimuths(&s.radials);
        // ΦDP wraps 0-360°: fold the gate-to-gate difference so a 358°->2°
        // step reads as ~4° of texture, not a ~356° spike.
        let textures: Vec<Vec<f32>> = s
            .radials
            .iter()
            .map(|r| nonmet::range_texture(&r.gates, half, Some(360.0)))
            .collect();
        (s, order, azimuths, textures)
    });

    for radial in &mut cleaned.radials {
        let cc_radial = cc.as_ref().map(|(s, order, azimuths)| {
            let (i1, _i2, _w1, _w2) = nearest_two_radial_indices(azimuths, radial.azimuth_deg);
            &s.radials[order[i1]]
        });
        let zdr_hit = zdr.as_ref().map(|(s, order, azimuths, textures)| {
            let (i1, _i2, _w1, _w2) = nearest_two_radial_indices(azimuths, radial.azimuth_deg);
            (&s.radials[order[i1]], &textures[order[i1]])
        });
        let phidp_hit = phidp.as_ref().map(|(s, order, azimuths, textures)| {
            let (i1, _i2, _w1, _w2) = nearest_two_radial_indices(azimuths, radial.azimuth_deg);
            (&s.radials[order[i1]], &textures[order[i1]])
        });

        for (i, gate) in radial.gates.iter_mut().enumerate() {
            if gate.is_none() {
                continue;
            }
            let cc_val = cc_radial.and_then(|r| r.gates.get(i).copied().flatten());
            let (zdr_val, sd_zdr) = match zdr_hit {
                Some((r, tex)) => (r.gates.get(i).copied().flatten(), tex.get(i).copied()),
                None => (None, None),
            };
            let sd_phidp = phidp_hit.and_then(|(_, tex)| tex.get(i).copied());
            if nonmet::should_null_reflectivity_gate(
                cc_val,
                sd_phidp,
                sd_zdr,
                zdr_val,
                qc.cc_gate_threshold,
                qc.nonmet_threshold,
            ) {
                *gate = None;
            }
        }
    }
}

/// Rasterize one sweep to raw RGBA bytes.
pub fn rasterize(
    sweep: &SweepData,
    product: Product,
    size_px: usize,
    max_range_km: f32,
    qc: &QcConfig,
) -> Vec<u8> {
    let sweep = clean_sweep(sweep, product, qc);
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
        Product::SpectrumWidth => colors::spectrum_width_color as fn(f32) -> [u8; 4],
        Product::DifferentialReflectivity => colors::zdr_color as fn(f32) -> [u8; 4],
        Product::CorrelationCoefficient => colors::cc_color as fn(f32) -> [u8; 4],
        Product::DifferentialPhase => colors::phidp_color as fn(f32) -> [u8; 4],
    };

    let center = size_px as f32 / 2.0;
    let km_per_px = 2.0 * max_range_km / size_px as f32;

    for py in 0..size_px {
        let dy = (py as f32 + 0.5 - center) * km_per_px;
        for px in 0..size_px {
            let dx = (px as f32 + 0.5 - center) * km_per_px;
            let range_km = (dx * dx + dy * dy).sqrt();
            if !(sweep.first_gate_km..=max_range_km).contains(&range_km) {
                continue;
            }

            let azimuth = dx.atan2(-dy).to_degrees().rem_euclid(360.0);
            let (i1, i2, w1, w2) = nearest_two_radial_indices(&sorted_azimuths, azimuth);
            let radial1 = &sweep.radials[order[i1]];
            let radial2 = &sweep.radials[order[i2]];

            // Bilinear interpolation across azimuth (ξ) and range (η).
            // Replaces the old hard gate index that produced blocky
            // gate-aligned artifacts. Kvasov et al. show ~90% improvement.
            let gate_frac = (range_km - sweep.first_gate_km) / sweep.gate_spacing_km;
            let gate = gate_frac.floor() as usize;
            let eta = gate_frac.fract().clamp(0.0, 1.0);

            let value = bilinear_sample(radial1, radial2, w1, w2, gate, eta);
            if let Some(value) = value {
                let c = color_of(value);
                let idx = (py * size_px + px) * 4;
                pixels[idx] = c[0];
                pixels[idx + 1] = c[1];
                pixels[idx + 2] = c[2];
                pixels[idx + 3] = c[3];
            } else if matches!(product, Product::Velocity | Product::SpectrumWidth)
                && radial1.range_folded.get(gate).copied().unwrap_or(false)
            {
                let c = colors::RANGE_FOLDED_COLOR;
                let idx = (py * size_px + px) * 4;
                pixels[idx..idx + 4].copy_from_slice(&c);
            }
        }
    }

    // Speckle/close QC passes are Reflectivity-only (D5): ZDR/CC/PhiDP are not
    // intensity fields, so morphological close/despeckle/region-area filtering
    // (tuned for dBZ clutter) would punch holes in valid dual-pol data.
    if product == Product::Reflectivity {
        morphological_close(&mut pixels, size_px, 2);
        despeckle(&mut pixels, size_px, 2);
        // Region-area speckle filter: a fixed 3×3 density check passes small
        // *clumps* (a 2×2 clutter blob has 3 live neighbours per pixel). Flood-fill
        // 8-connected blobs and drop any below a minimum pixel area — the standard
        // shape-independent "isolated echo removal" (JMA QC; see docs/research).
        // ponytail: constant area threshold at the current 1024px/230km raster;
        // make it scale with size_px/max_range_km if the raster geometry changes.
        remove_small_regions(&mut pixels, size_px, 8);
    }

    pixels
}

/// Zero out 8-connected blobs of live (alpha≠0) pixels smaller than `min_area`.
fn remove_small_regions(pixels: &mut [u8], size_px: usize, min_area: usize) {
    let n = size_px * size_px;
    let mut visited = vec![false; n];
    let mut stack: Vec<usize> = Vec::new();
    let mut blob: Vec<usize> = Vec::new();
    for start in 0..n {
        if visited[start] || pixels[start * 4 + 3] == 0 {
            continue;
        }
        // Flood-fill this blob.
        stack.clear();
        blob.clear();
        stack.push(start);
        visited[start] = true;
        while let Some(idx) = stack.pop() {
            blob.push(idx);
            let x = (idx % size_px) as i32;
            let y = (idx / size_px) as i32;
            for dy in -1..=1i32 {
                for dx in -1..=1i32 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x + dx;
                    let ny = y + dy;
                    if nx < 0 || ny < 0 || nx >= size_px as i32 || ny >= size_px as i32 {
                        continue;
                    }
                    let nidx = (ny as usize) * size_px + nx as usize;
                    if !visited[nidx] && pixels[nidx * 4 + 3] != 0 {
                        visited[nidx] = true;
                        stack.push(nidx);
                    }
                }
            }
        }
        if blob.len() < min_area {
            for &idx in &blob {
                let p = idx * 4;
                pixels[p] = 0;
                pixels[p + 1] = 0;
                pixels[p + 2] = 0;
                pixels[p + 3] = 0;
            }
        }
    }
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

/// Bilinear interpolation across azimuth (ξ) and range (η).
///
/// Blends the four surrounding gates (2 radials × 2 range gates) using
/// the azimuth weights `w1`/`w2` and the range fraction `eta`. Missing
/// gates are excluded and the result renormalised by available weight,
/// so partial data does not bias the sample.
///
/// Formula: `Z = (1-ξ)(1-η)·Z_ij + (1-ξ)η·Z_i(j+1) + ξ(1-η)·Z_(i+1)j + ξη·Z_(i+1)(j+1)`
pub(crate) fn bilinear_sample(
    radial1: &RadialData,
    radial2: &RadialData,
    w1: f32,
    w2: f32,
    gate: usize,
    eta: f32,
) -> Option<f32> {
    let v1g = radial1.gates.get(gate).and_then(|v| *v);
    let v1g1 = radial1.gates.get(gate + 1).and_then(|v| *v);
    let v2g = radial2.gates.get(gate).and_then(|v| *v);
    let v2g1 = radial2.gates.get(gate + 1).and_then(|v| *v);

    let eta_c = 1.0 - eta;
    // (weight, value) for each of the 4 corners
    let corners = [
        (w1 * eta_c, v1g),
        (w1 * eta, v1g1),
        (w2 * eta_c, v2g),
        (w2 * eta, v2g1),
    ];

    let mut sum = 0.0;
    let mut total_w = 0.0;
    for (w, v) in corners {
        if let Some(v) = v {
            sum += w * v;
            total_w += w;
        }
    }
    if total_w > 0.0 {
        Some(sum / total_w)
    } else {
        None
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

/// Project a radar site's lat/lon to screen pixel coordinates, given the
/// center site, pan offset, and zoom. Uses the same azimuthal-equidistant
/// projection as the scope overlays. Used both for drawing site markers and
/// for double-click hit-testing in the input handler.
pub fn project_site(
    target_lat: f64,
    target_lon: f64,
    center: &RadarSite,
    pan_km: (f32, f32),
    zoom: f32,
) -> (f32, f32) {
    let side = screen_width().min(screen_height());
    let px_per_km = (side / 2.0) / MAX_RANGE_KM * zoom;
    let center_x = screen_width() / 2.0 + pan_km.0 * px_per_km;
    let center_y = screen_height() / 2.0 + pan_km.1 * px_per_km;
    let km = geo::point_to_km_offset(center.lat, center.lon, (target_lat, target_lon));
    (center_x + km.x * px_per_km, center_y + km.y * px_per_km)
}

/// Draw the full radar scope. Called directly to screen (avoids
/// render_to_texture coordinate flip — see Stage 1 lesson).
///
/// Stage 4 adds optional border and alert overlay drawing.
/// Stage 5 adds optional NHC tropical cyclone overlay drawing.
#[allow(clippy::too_many_arguments)]
pub fn draw_scope_to_texture(
    radar_texture: Option<&Texture2D>,
    site: &RadarSite,
    pan_km: (f32, f32),
    zoom: f32,
    borders: Option<(&[Ring], bool)>,
    alerts: Option<(&[Alert], bool, bool)>,
    nhc: Option<(&NhcBundle, &NhcOverlayState)>,
    user: Option<Coords>,
    show_sites: bool,
    show_rings: bool,
) {
    let side = screen_width().min(screen_height());
    let px_per_km = (side / 2.0) / MAX_RANGE_KM * zoom;
    let center_x = screen_width() / 2.0 + pan_km.0 * px_per_km;
    let center_y = screen_height() / 2.0 + pan_km.1 * px_per_km;

    // Background is cleared by the main loop (dark observatory gradient);
    // the scope draws the radar texture and overlays on top of it.

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

    // Range rings + cardinal crosshairs (optional).
    if show_rings {
        // Range rings every 50 km
        let mut ring_km = 50.0;
        while ring_km <= MAX_RANGE_KM {
            draw_circle_lines(center_x, center_y, ring_km * px_per_km, 1.0, grid_color);
            draw_text(
                format!("{ring_km:.0} km"),
                center_x + 4.0,
                center_y - ring_km * px_per_km,
                18.0,
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
            let text_dims = measure_text(label, None, 18, 1.0);
            draw_text(
                label,
                center_x + lx - text_dims.width / 2.0,
                center_y + ly - text_dims.height / 2.0,
                18.0,
                grid_text_color,
            );
        }
    }

    // Station marker at center
    draw_circle(center_x, center_y, 4.0, WHITE);
    draw_text(site.id, center_x + 8.0, center_y + 8.0, 18.0, WHITE);

    // ── City markers (progressive disclosure + collision avoidance) ──
    //
    // Cities are filtered by a population threshold derived from zoom
    // (biggest cities when zoomed out, progressively smaller as you zoom
    // in), then greedily placed sorted by population descending with a
    // label safe-zone so names never overlap.
    use crate::cities;
    let city_color = MacroquadColor::from_rgba(0xdd, 0xdd, 0xaa, 230);
    let min_pop = cities::min_population_for_zoom(zoom);
    let sw = screen_width();
    let sh = screen_height();
    let label_margin = 6.0; // safe-zone padding around each label
    let cull_margin = 80.0;

    // Collect on-screen candidates meeting the population threshold.
    let mut candidates: Vec<(&str, f32, f32, i64)> = Vec::new();
    for city in cities::cities().iter() {
        if city.pop < min_pop {
            continue;
        }
        let km = geo::point_to_km_offset(site.lat, site.lon, (city.lat, city.lon));
        let cx = center_x + km.x * px_per_km;
        let cy = center_y + km.y * px_per_km;
        if cx < -cull_margin || cx > sw + cull_margin || cy < -cull_margin || cy > sh + cull_margin
        {
            continue;
        }
        candidates.push((city.name.as_str(), cx, cy, city.pop));
    }
    // Biggest cities first — they get priority for label placement.
    candidates.sort_unstable_by_key(|&(_, _, _, pop)| std::cmp::Reverse(pop));

    // Greedy label placement with rectangle overlap avoidance.
    // Each entry: (min_x, min_y, max_x, max_y) of the placed label safe-zone.
    let mut placed: Vec<(f32, f32, f32, f32)> = Vec::new();
    let label_font = 15.0;
    for (name, cx, cy, _pop) in &candidates {
        let text_dims = measure_text(name, None, label_font as u16, 1.0);
        let lx = cx + 8.0;
        let ly = cy - 8.0 - text_dims.height;
        let bw = text_dims.width + label_margin * 2.0;
        let bh = text_dims.height + label_margin * 2.0;
        let box_x = lx - label_margin;
        let box_y = ly - label_margin;
        // Check collision against already-placed labels.
        let collides = placed.iter().any(|&(px0, py0, px1, py1)| {
            box_x < px1 && box_x + bw > px0 && box_y < py1 && box_y + bh > py0
        });
        if collides {
            continue; // skip — would overlap an existing label
        }
        draw_circle(*cx, *cy, 3.0, city_color);
        draw_text(name, lx, ly, label_font, city_color);
        placed.push((box_x, box_y, box_x + bw, box_y + bh));
    }

    // ── Radar site markers (double-click to select) ─────────────
    // Shown only while the Radar side panel is open (mirrors `show_sites`).
    if show_sites {
        let sw = screen_width();
        let sh = screen_height();
        let site_marker_color = MacroquadColor::from_rgba(0x0d, 0xc5, 0xb8, 220);
        let site_label_color = MacroquadColor::from_rgba(0x0d, 0xc5, 0xb8, 255);
        let margin = 60.0;
        for other in geo::RADAR_SITES.iter() {
            if other.id == site.id {
                continue; // the active site is already marked at center
            }
            let (sx, sy) = project_site(other.lat, other.lon, site, pan_km, zoom);
            // Cull off-screen markers.
            if sx < -margin || sx > sw + margin || sy < -margin || sy > sh + margin {
                continue;
            }
            draw_circle(sx, sy, 5.0, site_marker_color);
            draw_circle_lines(
                sx,
                sy,
                5.0,
                1.5,
                MacroquadColor::from_rgba(0xff, 0xff, 0xff, 180),
            );
            draw_text(other.id, sx + 8.0, sy - 6.0, 14.0, site_label_color);
        }
    }

    // ── Border overlays (Stage 4) ────────────────────────────────
    if let Some((rings, show)) = borders
        && show
    {
        draw_borders(rings, site, center_x, center_y, px_per_km);
    }

    // ── Alert overlays (Stage 4) ─────────────────────────────────
    if let Some((alerts, show_watches, show_warnings)) = alerts
        && (show_watches || show_warnings)
    {
        draw_alerts(
            alerts,
            show_watches,
            show_warnings,
            site,
            center_x,
            center_y,
            px_per_km,
        );
    }

    // ── NHC overlays (Stage 5) ──────────────────────────────────
    if let Some((bundle, overlays)) = nhc {
        draw_nhc_overlays(bundle, overlays, site, center_x, center_y, px_per_km);
    }

    // ── User location marker ─────────────────────────────────────
    if let Some(user) = user {
        draw_user_location(user, site, center_x, center_y, px_per_km);
    }
}

/// Draw state-border and coastline line segments on the scope, extending
/// to the full window rather than clipping to the radar circle.
fn draw_borders(rings: &[Ring], site: &RadarSite, center_x: f32, center_y: f32, px_per_km: f32) {
    let border_color = MacroquadColor::from_rgba(0x8b, 0x73, 0x55, 180);
    let sw = screen_width();
    let sh = screen_height();
    // Margin for on-screen culling (pixels)
    let margin = 50.0;

    for ring in rings {
        if ring.len() < 2 {
            continue;
        }
        for pair in ring.windows(2) {
            let a_km = geo::point_to_km_offset(site.lat, site.lon, pair[0]);
            let b_km = geo::point_to_km_offset(site.lat, site.lon, pair[1]);

            let ax = center_x + a_km.x * px_per_km;
            let ay = center_y + a_km.y * px_per_km;
            let bx = center_x + b_km.x * px_per_km;
            let by = center_y + b_km.y * px_per_km;

            // Cull segments entirely off-screen
            if (ax < -margin && bx < -margin)
                || (ax > sw + margin && bx > sw + margin)
                || (ay < -margin && by < -margin)
                || (ay > sh + margin && by > sh + margin)
            {
                continue;
            }

            draw_line(ax, ay, bx, by, 1.0, border_color);
        }
    }
}

/// Draw active NWS warning/watch polygons across the full screen.
/// Each alert gets its NWS color and a label near the polygon centroid.
fn draw_alerts(
    alerts: &[Alert],
    show_watches: bool,
    show_warnings: bool,
    site: &RadarSite,
    center_x: f32,
    center_y: f32,
    px_per_km: f32,
) {
    let sw = screen_width();
    let sh = screen_height();
    let margin = 50.0;

    // ponytail: tuned by eye — below this px/km, zoomed-out labels are just
    // clutter; outlines still draw, labels reappear once you zoom back in.
    const MIN_PX_PER_KM_FOR_LABELS: f32 = 1.5;

    // ── Pass 1: draw every outline, no labels ────────────────────
    for alert in alerts {
        // Skip whichever category the user has toggled off.
        if crate::alerts::is_watch(&alert.event) {
            if !show_watches {
                continue;
            }
        } else if !show_warnings {
            continue;
        }

        let line_color =
            MacroquadColor::from_rgba(alert.color[0], alert.color[1], alert.color[2], 255);

        for ring in &alert.rings {
            if ring.len() < 3 {
                continue;
            }

            // Convert all points to screen-space pixel coordinates
            let pts_px: Vec<(f32, f32)> = ring
                .iter()
                .map(|&(lat, lon)| {
                    let km = geo::point_to_km_offset(site.lat, site.lon, (lat, lon));
                    (center_x + km.x * px_per_km, center_y + km.y * px_per_km)
                })
                .collect();

            // Draw the polygon outline, culling fully off-screen segments
            for i in 0..pts_px.len() {
                let (ax, ay) = pts_px[i];
                let (bx, by) = pts_px[(i + 1) % pts_px.len()];

                if (ax < -margin && bx < -margin)
                    || (ax > sw + margin && bx > sw + margin)
                    || (ay < -margin && by < -margin)
                    || (ay > sh + margin && by > sh + margin)
                {
                    continue;
                }
                draw_line(ax, ay, bx, by, 2.0, line_color);
            }
        }
    }

    // ── Pass 2: one label candidate per alert, collision-placed ──
    if px_per_km < MIN_PX_PER_KM_FOR_LABELS {
        return; // zoomed too far out — outlines only.
    }

    let mut candidates: Vec<(&str, f32, f32, [u8; 4])> = Vec::new();
    for alert in alerts {
        if crate::alerts::is_watch(&alert.event) {
            if !show_watches {
                continue;
            }
        } else if !show_warnings {
            continue;
        }

        let mut sum_x = 0.0f32;
        let mut sum_y = 0.0f32;
        let mut point_count = 0u32;

        for ring in &alert.rings {
            if ring.len() < 3 {
                continue;
            }
            for &(lat, lon) in ring {
                let km = geo::point_to_km_offset(site.lat, site.lon, (lat, lon));
                let px = center_x + km.x * px_per_km;
                let py = center_y + km.y * px_per_km;
                if px >= 0.0 && px <= sw && py >= 0.0 && py <= sh {
                    sum_x += px;
                    sum_y += py;
                    point_count += 1;
                }
            }
        }

        if point_count > 0 {
            let cx = sum_x / point_count as f32;
            let cy = sum_y / point_count as f32;
            let label = if alert.event.len() > 30 {
                &alert.event[..30]
            } else {
                &alert.event[..]
            };
            candidates.push((label, cx, cy, alert.color));
        }
    }

    // Deterministic order for the collision pass.
    candidates.sort_unstable_by(|a, b| a.1.total_cmp(&b.1).then(a.2.total_cmp(&b.2)));

    let label_margin = 6.0; // safe-zone padding around each label
    let label_font = 14.0;
    let mut placed: Vec<(f32, f32, f32, f32)> = Vec::new();
    for (label, cx, cy, color) in &candidates {
        let text_dims = measure_text(label, None, label_font as u16, 1.0);
        let bw = text_dims.width + label_margin * 2.0;
        let bh = text_dims.height + label_margin * 2.0;
        let box_x = cx - label_margin;
        let box_y = cy - label_margin;
        let collides = placed.iter().any(|&(px0, py0, px1, py1)| {
            box_x < px1 && box_x + bw > px0 && box_y < py1 && box_y + bh > py0
        });
        if collides {
            continue;
        }
        let label_color = MacroquadColor::from_rgba(color[0], color[1], color[2], 255);
        draw_text(label, *cx, *cy, label_font, label_color);
        placed.push((box_x, box_y, box_x + bw, box_y + bh));
    }
}

/// Draw the user's location as a cyan crosshair-pin at its projected position.
fn draw_user_location(
    user: Coords,
    site: &RadarSite,
    center_x: f32,
    center_y: f32,
    px_per_km: f32,
) {
    let off = geo::point_to_km_offset(site.lat, site.lon, (user.lat, user.lon));
    let x = center_x + off.x * px_per_km;
    let y = center_y + off.y * px_per_km;
    let cyan = MacroquadColor::from_rgba(0, 220, 220, 255);
    // Crosshair + center dot — distinct from alert polygons and echoes.
    draw_line(x - 10.0, y, x + 10.0, y, 2.0, cyan);
    draw_line(x, y - 10.0, x, y + 10.0, 2.0, cyan);
    draw_circle(x, y, 4.0, cyan);
    draw_circle_lines(x, y, 9.0, 1.5, cyan);
}

// ---------------------------------------------------------------------------
// NHC overlay drawing (Stage 5)
// ---------------------------------------------------------------------------

/// Toggle state for individual NHC overlay layers.
pub struct NhcOverlayState {
    pub show_cone: bool,
    pub show_track: bool,
    pub show_points: bool,
    pub show_watches_warnings: bool,
    pub show_wind_probs: bool,
    pub show_earliest_arrival: bool,
    pub show_most_likely_arrival: bool,
}

impl Default for NhcOverlayState {
    fn default() -> Self {
        Self {
            show_cone: true,
            show_track: true,
            show_points: true,
            show_watches_warnings: true,
            show_wind_probs: false,
            show_earliest_arrival: false,
            show_most_likely_arrival: false,
        }
    }
}

/// Draw NHC tropical cyclone overlays on the scope.
fn draw_nhc_overlays(
    bundle: &NhcBundle,
    overlays: &NhcOverlayState,
    site: &RadarSite,
    center_x: f32,
    center_y: f32,
    px_per_km: f32,
) {
    let sw = screen_width();
    let sh = screen_height();
    let margin = 50.0;

    // ── GIS overlays: cone, track, points, watches/warnings ──────
    for storm in &bundle.gis_storms {
        // Forecast cone (semi-transparent white outline)
        if overlays.show_cone {
            let cone_color = MacroquadColor::from_rgba(0xff, 0xff, 0xff, 100);
            for ring in &storm.cone {
                if ring.len() < 3 {
                    continue;
                }
                let pts: Vec<(f32, f32)> = ring
                    .iter()
                    .map(|&(lat, lon)| {
                        let km = geo::point_to_km_offset(site.lat, site.lon, (lat, lon));
                        (center_x + km.x * px_per_km, center_y + km.y * px_per_km)
                    })
                    .collect();
                for i in 0..pts.len() {
                    let (ax, ay) = pts[i];
                    let (bx, by) = pts[(i + 1) % pts.len()];
                    if (ax < -margin && bx < -margin)
                        || (ax > sw + margin && bx > sw + margin)
                        || (ay < -margin && by < -margin)
                        || (ay > sh + margin && by > sh + margin)
                    {
                        continue;
                    }
                    draw_line(ax, ay, bx, by, 1.5, cone_color);
                }
            }
        }

        // Forecast track (dotted line in storm color)
        if overlays.show_track {
            let track_color = MacroquadColor::from_rgba(0xff, 0xcc, 0x66, 200);
            for ring in &storm.track {
                for pair in ring.windows(2) {
                    let a_km = geo::point_to_km_offset(site.lat, site.lon, pair[0]);
                    let b_km = geo::point_to_km_offset(site.lat, site.lon, pair[1]);
                    let ax = center_x + a_km.x * px_per_km;
                    let ay = center_y + a_km.y * px_per_km;
                    let bx = center_x + b_km.x * px_per_km;
                    let by = center_y + b_km.y * px_per_km;
                    if (ax < -margin && bx < -margin)
                        || (ax > sw + margin && bx > sw + margin)
                        || (ay < -margin && by < -margin)
                        || (ay > sh + margin && by > sh + margin)
                    {
                        continue;
                    }
                    draw_line(ax, ay, bx, by, 2.0, track_color);
                }
            }
        }

        // Forecast points (markers with labels)
        if overlays.show_points {
            let pt_color = MacroquadColor::from_rgba(0xff, 0xcc, 0x66, 255);
            for &(lat, lon, ref label) in &storm.points {
                let km = geo::point_to_km_offset(site.lat, site.lon, (lat, lon));
                let px = center_x + km.x * px_per_km;
                let py = center_y + km.y * px_per_km;
                if px < -margin || px > sw + margin || py < -margin || py > sh + margin {
                    continue;
                }
                draw_circle(px, py, 3.0, pt_color);
                if !label.is_empty() {
                    draw_text(label, px + 5.0, py - 8.0, 12.0, pt_color);
                }
            }
        }

        // Watches/warnings (colored line segments)
        if overlays.show_watches_warnings {
            for (ring, ww_type) in &storm.watches_warnings {
                let color = crate::nhc::watch_warning_color(ww_type);
                let line_color = MacroquadColor::from_rgba(color[0], color[1], color[2], color[3]);
                if ring.len() < 2 {
                    continue;
                }
                for pair in ring.windows(2) {
                    let a_km = geo::point_to_km_offset(site.lat, site.lon, pair[0]);
                    let b_km = geo::point_to_km_offset(site.lat, site.lon, pair[1]);
                    let ax = center_x + a_km.x * px_per_km;
                    let ay = center_y + a_km.y * px_per_km;
                    let bx = center_x + b_km.x * px_per_km;
                    let by = center_y + b_km.y * px_per_km;
                    if (ax < -margin && bx < -margin)
                        || (ax > sw + margin && bx > sw + margin)
                        || (ay < -margin && by < -margin)
                        || (ay > sh + margin && by > sh + margin)
                    {
                        continue;
                    }
                    draw_line(ax, ay, bx, by, 3.0, line_color);
                }
            }
        }
    }

    // ── Wind probability contours ──────────────────────────────────
    if overlays.show_wind_probs {
        draw_contours(
            &bundle.wind_probs_34kt,
            site,
            center_x,
            center_y,
            px_per_km,
            sw,
            sh,
            margin,
        );
        draw_contours(
            &bundle.wind_probs_50kt,
            site,
            center_x,
            center_y,
            px_per_km,
            sw,
            sh,
            margin,
        );
        draw_contours(
            &bundle.wind_probs_64kt,
            site,
            center_x,
            center_y,
            px_per_km,
            sw,
            sh,
            margin,
        );
    }

    // ── Arrival time contours ──────────────────────────────────────
    if overlays.show_earliest_arrival {
        draw_arrival_contours(
            &bundle.earliest_arrival,
            site,
            center_x,
            center_y,
            px_per_km,
            sw,
            sh,
            margin,
            MacroquadColor::from_rgba(0x66, 0xaa, 0xff, 180),
        );
    }
    if overlays.show_most_likely_arrival {
        draw_arrival_contours(
            &bundle.most_likely_arrival,
            site,
            center_x,
            center_y,
            px_per_km,
            sw,
            sh,
            margin,
            MacroquadColor::from_rgba(0x66, 0xdd, 0x88, 180),
        );
    }
}

/// Draw wind probability contours as colored polygon outlines.
#[allow(clippy::too_many_arguments)]
fn draw_contours(
    contours: &[WindProbContour],
    site: &RadarSite,
    center_x: f32,
    center_y: f32,
    px_per_km: f32,
    sw: f32,
    sh: f32,
    margin: f32,
) {
    for contour in contours {
        let color = crate::nhc::wind_prob_color(contour.prob_high);
        let line_color = MacroquadColor::from_rgba(color[0], color[1], color[2], color[3]);
        for ring in &contour.rings {
            if ring.len() < 3 {
                continue;
            }
            let pts: Vec<(f32, f32)> = ring
                .iter()
                .map(|&(lat, lon)| {
                    let km = geo::point_to_km_offset(site.lat, site.lon, (lat, lon));
                    (center_x + km.x * px_per_km, center_y + km.y * px_per_km)
                })
                .collect();
            for i in 0..pts.len() {
                let (ax, ay) = pts[i];
                let (bx, by) = pts[(i + 1) % pts.len()];
                if (ax < -margin && bx < -margin)
                    || (ax > sw + margin && bx > sw + margin)
                    || (ay < -margin && by < -margin)
                    || (ay > sh + margin && by > sh + margin)
                {
                    continue;
                }
                draw_line(ax, ay, bx, by, 1.5, line_color);
            }
        }
    }
}

/// Draw arrival time contours as colored polygon outlines.
#[allow(clippy::too_many_arguments)]
fn draw_arrival_contours(
    contours: &[ArrivalTimeContour],
    site: &RadarSite,
    center_x: f32,
    center_y: f32,
    px_per_km: f32,
    sw: f32,
    sh: f32,
    margin: f32,
    color: MacroquadColor,
) {
    let arrival_color = color;
    for contour in contours {
        for ring in &contour.rings {
            if ring.len() < 3 {
                continue;
            }
            let pts: Vec<(f32, f32)> = ring
                .iter()
                .map(|&(lat, lon)| {
                    let km = geo::point_to_km_offset(site.lat, site.lon, (lat, lon));
                    (center_x + km.x * px_per_km, center_y + km.y * px_per_km)
                })
                .collect();
            for i in 0..pts.len() {
                let (ax, ay) = pts[i];
                let (bx, by) = pts[(i + 1) % pts.len()];
                if (ax < -margin && bx < -margin)
                    || (ax > sw + margin && bx > sw + margin)
                    || (ay < -margin && by < -margin)
                    || (ay > sh + margin && by > sh + margin)
                {
                    continue;
                }
                draw_line(ax, ay, bx, by, 1.5, arrival_color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn radial(az: f32, gates: Vec<Option<f32>>) -> RadialData {
        RadialData {
            azimuth_deg: az,
            gates,
            range_folded: vec![],
        }
    }

    #[test]
    fn range_folded_gates_render_purple() {
        // One radial, ~100 gates, all None; only gate 0 is flagged range-folded.
        let n_gates = 100;
        let mut range_folded = vec![false; n_gates];
        range_folded[0] = true;
        let sweep = SweepData {
            elevation_deg: 0.5,
            radials: vec![RadialData {
                azimuth_deg: 0.0,
                gates: vec![None; n_gates],
                range_folded,
            }],
            first_gate_km: 2.125,
            gate_spacing_km: 0.25,
            nyquist_ms: 0.0,
        };

        let velocity_pixels = rasterize(
            &sweep,
            Product::Velocity,
            200,
            20.0,
            &QcConfig {
                tdbz_kernel_size: 3,
                cc_sweep: None,
                cc_gate_enabled: false,
                cc_gate_threshold: 0.0,
                refl_floor_enabled: false,
                refl_floor_dbz: 7.0,
                vel_dealias_enabled: true,
                vel_sd_censor_enabled: false,
                vel_sd_threshold: 7.0,
                ..Default::default()
            },
        );
        let has_purple = velocity_pixels
            .chunks_exact(4)
            .any(|px| px == colors::RANGE_FOLDED_COLOR);
        assert!(has_purple, "expected at least one RF-purple pixel");

        // Product gating: Reflectivity never range-folds, so no purple pixels
        // even though the RF flag is present in the sweep data.
        let refl_pixels = rasterize(
            &sweep,
            Product::Reflectivity,
            200,
            20.0,
            &QcConfig {
                tdbz_kernel_size: 3,
                cc_sweep: None,
                cc_gate_enabled: false,
                cc_gate_threshold: 0.0,
                refl_floor_enabled: false,
                refl_floor_dbz: 7.0,
                vel_dealias_enabled: true,
                vel_sd_censor_enabled: false,
                vel_sd_threshold: 7.0,
                ..Default::default()
            },
        );
        let has_purple_refl = refl_pixels
            .chunks_exact(4)
            .any(|px| px == colors::RANGE_FOLDED_COLOR);
        assert!(!has_purple_refl, "Reflectivity must not render RF purple");
    }

    #[test]
    fn rasterize_uses_per_sweep_gate_geometry_not_module_consts() {
        // Gate 5 carries the only real value. Geometry is 1.0/1.0 km
        // (legacy upper-tilt), not the super-res module consts (2.125/0.25)
        // — the pixel bearing gate 5's value must land at
        // range = 1.0 + 5*1.0 = 6.0 km, not the module-const range
        // 2.125 + 5*0.25 = 3.375 km. This is the test that would have
        // caught the original per-tilt/per-product geometry bug.
        let mut gates = vec![None; 20];
        gates[5] = Some(15.0);
        let sweep = SweepData {
            elevation_deg: 0.5,
            radials: vec![radial(0.0, gates)],
            first_gate_km: 1.0,
            gate_spacing_km: 1.0,
            nyquist_ms: 0.0,
        };

        let size_px = 400;
        let max_range_km = 20.0;
        let pixels = rasterize(
            &sweep,
            Product::Velocity,
            size_px,
            max_range_km,
            &QcConfig {
                tdbz_kernel_size: 3,
                cc_sweep: None,
                cc_gate_enabled: false,
                cc_gate_threshold: 0.0,
                refl_floor_enabled: false,
                refl_floor_dbz: 7.0,
                vel_dealias_enabled: true,
                vel_sd_censor_enabled: false,
                vel_sd_threshold: 7.0,
                ..Default::default()
            },
        );
        let target = colors::velocity_color(15.0);
        let km_per_px = 2.0 * max_range_km / size_px as f32;
        let center = size_px as f32 / 2.0;

        let mut ranges: Vec<f32> = Vec::new();
        for py in 0..size_px {
            let dy = (py as f32 + 0.5 - center) * km_per_px;
            for px in 0..size_px {
                let dx = (px as f32 + 0.5 - center) * km_per_px;
                let idx = (py * size_px + px) * 4;
                if pixels[idx..idx + 4] == target {
                    ranges.push((dx * dx + dy * dy).sqrt());
                }
            }
        }
        assert!(
            !ranges.is_empty(),
            "expected some pixels colored for gate 5's value"
        );
        let min_r = ranges.iter().cloned().fold(f32::MAX, f32::min);
        let max_r = ranges.iter().cloned().fold(f32::MIN, f32::max);
        assert!(
            min_r > 5.0 && max_r < 7.0,
            "expected gate-5 pixels clustered around 6.0km (1.0/1.0 geometry), got range [{min_r}, {max_r}]"
        );
    }

    #[test]
    fn rasterize_dealias_turns_sign_flip_into_same_sign_gradient() {
        // One radial, gate-to-gate ramp that folds across Nyquist: raw gate 3
        // reads -25.0 (aliased) even though it's really a continuation of the
        // +20/+24/+28 outbound ramp (nyquist=26.4 -> interval 52.8; matches
        // dealias::tests::unfolds_single_fold_to_monotone_ramp).
        let gates = vec![Some(20.0), Some(24.0), Some(28.0), Some(-25.0)];
        let sweep = SweepData {
            elevation_deg: 0.5,
            radials: vec![radial(0.0, gates)],
            first_gate_km: 2.125,
            gate_spacing_km: 0.25,
            nyquist_ms: 26.4,
        };

        let size_px = 400;
        let max_range_km = 20.0;
        // Gate 3 -> range = 2.125 + 3*0.25 = 2.875 km.
        let gate3_range_km = 2.875;
        let color_at_gate3_range = |pixels: &[u8], target: [u8; 4]| -> bool {
            let km_per_px = 2.0 * max_range_km / size_px as f32;
            let center = size_px as f32 / 2.0;
            for py in 0..size_px {
                let dy = (py as f32 + 0.5 - center) * km_per_px;
                for px in 0..size_px {
                    let dx = (px as f32 + 0.5 - center) * km_per_px;
                    let idx = (py * size_px + px) * 4;
                    if pixels[idx..idx + 4] == target {
                        let r = (dx * dx + dy * dy).sqrt();
                        if (r - gate3_range_km).abs() < 1.0 {
                            return true;
                        }
                    }
                }
            }
            false
        };

        // Dealiasing on: gate 3 should render as the unfolded, same-sign
        // value (~27.8, continuing the outbound ramp), not raw -25.0.
        let dealiased_pixels = rasterize(
            &sweep,
            Product::Velocity,
            size_px,
            max_range_km,
            &QcConfig {
                tdbz_kernel_size: 3,
                cc_sweep: None,
                cc_gate_enabled: false,
                cc_gate_threshold: 0.0,
                refl_floor_enabled: false,
                refl_floor_dbz: 0.0,
                vel_dealias_enabled: true,
                vel_sd_censor_enabled: false,
                vel_sd_threshold: 0.0,
                ..Default::default()
            },
        );
        let unfolded_target = colors::velocity_color(27.8);
        let raw_target = colors::velocity_color(-25.0);
        assert!(
            color_at_gate3_range(&dealiased_pixels, unfolded_target),
            "dealiased gate 3 should render as the unfolded ~27.8 m/s color"
        );
        assert!(
            !color_at_gate3_range(&dealiased_pixels, raw_target),
            "dealiased gate 3 should not still render as the raw sign-flipped color"
        );

        // Dealiasing off: gate 3 renders as the raw, sign-flipped value.
        let raw_pixels = rasterize(
            &sweep,
            Product::Velocity,
            size_px,
            max_range_km,
            &QcConfig {
                tdbz_kernel_size: 3,
                cc_sweep: None,
                cc_gate_enabled: false,
                cc_gate_threshold: 0.0,
                refl_floor_enabled: false,
                refl_floor_dbz: 0.0,
                vel_dealias_enabled: false,
                vel_sd_censor_enabled: false,
                vel_sd_threshold: 0.0,
                ..Default::default()
            },
        );
        assert!(
            color_at_gate3_range(&raw_pixels, raw_target),
            "with dealiasing off, gate 3 should render as the raw -25.0 color"
        );
    }

    #[test]
    fn bilinear_exact_gate_returns_gate_value() {
        // eta = 0 → result is the azimuth blend of gate `gate` only.
        let r1 = radial(0.0, vec![Some(10.0), Some(20.0), Some(30.0)]);
        let r2 = radial(1.0, vec![Some(20.0), Some(30.0), Some(40.0)]);
        // gate 1, eta 0, equal azimuth weight
        let v = bilinear_sample(&r1, &r2, 0.5, 0.5, 1, 0.0);
        assert!((v.unwrap() - 25.0).abs() < 1e-4); // (20+30)/2
    }

    #[test]
    fn bilinear_range_interpolates_between_gates() {
        // eta = 0.5 at gate 0 → average of gate 0 and gate 1 (single radial).
        let r1 = radial(0.0, vec![Some(10.0), Some(20.0)]);
        let r2 = radial(0.0, vec![Some(10.0), Some(20.0)]); // identical
        let v = bilinear_sample(&r1, &r2, 1.0, 0.0, 0, 0.5);
        assert!((v.unwrap() - 15.0).abs() < 1e-4); // (10+20)/2
    }

    #[test]
    fn bilinear_all_missing_returns_none() {
        let r1 = radial(0.0, vec![None, None]);
        let r2 = radial(1.0, vec![None, None]);
        assert!(bilinear_sample(&r1, &r2, 0.5, 0.5, 0, 0.5).is_none());
    }

    #[test]
    fn bilinear_partial_missing_renormalises() {
        // Only the lower-right corner (radial2 gate+1) is present → returns it.
        let r1 = radial(0.0, vec![None, None]);
        let r2 = radial(1.0, vec![None, Some(42.0)]);
        let v = bilinear_sample(&r1, &r2, 0.5, 0.5, 0, 0.5);
        assert!((v.unwrap() - 42.0).abs() < 1e-4);
    }

    #[test]
    fn bilinear_full_four_corner_blend() {
        // Four distinct corners, equal weights and eta=0.5 → mean.
        let r1 = radial(0.0, vec![Some(0.0), Some(10.0)]);
        let r2 = radial(1.0, vec![Some(20.0), Some(30.0)]);
        let v = bilinear_sample(&r1, &r2, 0.5, 0.5, 0, 0.5);
        // (0 + 10 + 20 + 30) / 4 = 15
        assert!((v.unwrap() - 15.0).abs() < 1e-4);
    }

    #[test]
    fn bilinear_out_of_range_gate_returns_none() {
        let r1 = radial(0.0, vec![Some(10.0)]);
        let r2 = radial(1.0, vec![Some(20.0)]);
        // gate 5 is out of range for both → None
        assert!(bilinear_sample(&r1, &r2, 0.5, 0.5, 5, 0.5).is_none());
    }

    #[test]
    fn remove_small_regions_drops_speckle_keeps_blob() {
        // 8×8 RGBA image: a lone pixel (area 1) and a 3×3 block (area 9).
        let size = 8;
        let mut px = vec![0u8; size * size * 4];
        let set = |px: &mut [u8], x: usize, y: usize| {
            px[(y * size + x) * 4 + 3] = 255;
        };
        set(&mut px, 0, 0); // isolated speckle
        for y in 4..7 {
            for x in 4..7 {
                set(&mut px, x, y); // 3×3 blob
            }
        }
        remove_small_regions(&mut px, size, 8);
        assert_eq!(px[3], 0, "lone pixel at (0,0) removed");
        assert_eq!(px[(5 * size + 5) * 4 + 3], 255, "9-px blob kept");
    }

    #[test]
    fn tdbz_kernel_size_widens_clutter_removal_footprint() {
        // A single spike gate surrounded by uniform reflectivity: a wider
        // TDBZ kernel averages over more gate-pairs per position, so the
        // spike's influence (and thus clutter removal) spreads further.
        let mut gates = vec![Some(20.0); 21];
        gates[10] = Some(80.0);
        let sweep = SweepData {
            elevation_deg: 0.0,
            radials: vec![radial(0.0, gates)],
            first_gate_km: 2.125,
            gate_spacing_km: 0.25,
            nyquist_ms: 0.0,
        };

        let removed_count = |kernel_size: usize| {
            clean_sweep(
                &sweep,
                Product::Reflectivity,
                &QcConfig {
                    tdbz_kernel_size: kernel_size,
                    cc_sweep: None,
                    cc_gate_enabled: false,
                    cc_gate_threshold: 0.80,
                    refl_floor_enabled: false,
                    refl_floor_dbz: 7.0,
                    vel_dealias_enabled: true,
                    vel_sd_censor_enabled: false,
                    vel_sd_threshold: 7.0,
                    ..Default::default()
                },
            )
            .radials[0]
                .gates
                .iter()
                .filter(|g| g.is_none())
                .count()
        };

        assert_eq!(removed_count(5), 5);
        assert_eq!(removed_count(13), 13);
    }

    #[test]
    fn cc_gating_nulls_low_cc_ref_gate_and_keeps_high() {
        // REF radial: two live gates. CC radial (same azimuth): gate0 low
        // (0.55 — birds), gate1 high (0.98 — precip). Both dBZ values (30/40)
        // are above the dBZ floor (20.0 within 20km) and far enough apart in
        // range that the 2-gate TDBZ pass's `end - start < 2` guard keeps out
        // of the way, isolating the CC-gating effect.
        let ref_sweep = SweepData {
            elevation_deg: 0.5,
            radials: vec![radial(0.0, vec![Some(30.0), Some(40.0)])],
            first_gate_km: 2.125,
            gate_spacing_km: 0.25,
            nyquist_ms: 0.0,
        };
        let cc_sweep = SweepData {
            elevation_deg: 0.5,
            radials: vec![radial(0.0, vec![Some(0.55), Some(0.98)])],
            first_gate_km: 2.125,
            gate_spacing_km: 0.25,
            nyquist_ms: 0.0,
        };
        let cleaned = clean_sweep(
            &ref_sweep,
            Product::Reflectivity,
            &QcConfig {
                tdbz_kernel_size: 9,
                cc_sweep: Some(&cc_sweep),
                cc_gate_enabled: true,
                cc_gate_threshold: 0.80,
                refl_floor_enabled: false,
                refl_floor_dbz: 7.0,
                vel_dealias_enabled: true,
                vel_sd_censor_enabled: false,
                vel_sd_threshold: 7.0,
                ..Default::default()
            },
        );
        assert_eq!(cleaned.radials[0].gates[0], None, "low-CC gate suppressed");
        assert_eq!(
            cleaned.radials[0].gates[1],
            Some(40.0),
            "high-CC gate preserved"
        );

        // Disabled → both preserved.
        let ungated = clean_sweep(
            &ref_sweep,
            Product::Reflectivity,
            &QcConfig {
                tdbz_kernel_size: 9,
                cc_sweep: Some(&cc_sweep),
                cc_gate_enabled: false,
                cc_gate_threshold: 0.80,
                refl_floor_enabled: false,
                refl_floor_dbz: 7.0,
                vel_dealias_enabled: true,
                vel_sd_censor_enabled: false,
                vel_sd_threshold: 7.0,
                ..Default::default()
            },
        );
        assert_eq!(ungated.radials[0].gates[0], Some(30.0));

        // Fail-open: no CC sweep → both preserved.
        let no_cc = clean_sweep(
            &ref_sweep,
            Product::Reflectivity,
            &QcConfig {
                tdbz_kernel_size: 9,
                cc_sweep: None,
                cc_gate_enabled: true,
                cc_gate_threshold: 0.80,
                refl_floor_enabled: false,
                refl_floor_dbz: 7.0,
                vel_dealias_enabled: true,
                vel_sd_censor_enabled: false,
                vel_sd_threshold: 7.0,
                ..Default::default()
            },
        );
        assert_eq!(no_cc.radials[0].gates[0], Some(30.0));
    }

    #[test]
    fn refl_noise_floor_nulls_weak_long_range_gate() {
        // Build a Reflectivity radial with enough gates to reach beyond 80 km.
        // Gate index 312 → range = 2.125 + 312*0.25 = 80.125 km,
        // so the range floor is 5 dBZ (not 10 dBZ). A user floor of 7 dBZ
        // should cull this gate.
        let n_gates = 320;
        let mut gates = vec![None; n_gates];
        gates[312] = Some(6.0); // Above 5 dBZ range floor, below 7 dBZ user floor
        let sweep = SweepData {
            elevation_deg: 0.5,
            radials: vec![radial(0.0, gates.clone())],
            first_gate_km: 2.125,
            gate_spacing_km: 0.25,
            nyquist_ms: 0.0,
        };

        // With floor enabled (default), gate 312 should be nulled.
        let cleaned_with_floor = clean_sweep(
            &sweep,
            Product::Reflectivity,
            &QcConfig {
                tdbz_kernel_size: 3,
                cc_sweep: None,
                cc_gate_enabled: false,
                cc_gate_threshold: 0.80,
                refl_floor_enabled: true, // refl_floor_enabled
                refl_floor_dbz: 7.0,      // refl_floor_dbz
                vel_dealias_enabled: true,
                vel_sd_censor_enabled: false,
                vel_sd_threshold: 7.0,
                ..Default::default()
            },
        );
        assert_eq!(
            cleaned_with_floor.radials[0].gates[312], None,
            "6.0 dBZ gate below 7.0 dBZ floor should be nulled"
        );

        // With floor disabled, gate 312 should be preserved.
        let cleaned_without_floor = clean_sweep(
            &sweep,
            Product::Reflectivity,
            &QcConfig {
                tdbz_kernel_size: 3,
                cc_sweep: None,
                cc_gate_enabled: false,
                cc_gate_threshold: 0.80,
                refl_floor_enabled: false, // refl_floor_enabled
                refl_floor_dbz: 7.0,
                vel_dealias_enabled: true,
                vel_sd_censor_enabled: false,
                vel_sd_threshold: 7.0,
                ..Default::default()
            },
        );
        assert_eq!(
            cleaned_without_floor.radials[0].gates[312],
            Some(6.0),
            "6.0 dBZ gate preserved when floor disabled"
        );
    }

    #[test]
    fn velocity_sd_censor_nulls_noisy_gate_keeps_smooth() {
        // 9 radials, 1 degree apart (all fall within the wrapping ±4-azimuth
        // window). Each has 9 gates, all constant Some(10.0) except gate
        // column 4 (the center gate under test), which alternates +/-
        // strongly across radials to blow up the local SD. Gate 0 sits far
        // enough away (window half is 2 gates) that its window never
        // touches the noisy column, so it stays smooth.
        let n_radials = 9;
        let n_gates = 9;
        let mut radials: Vec<RadialData> = (0..n_radials)
            .map(|r| radial(r as f32, vec![Some(10.0); n_gates]))
            .collect();

        let center_radial = n_radials / 2;
        let center_gate = n_gates / 2;
        for (r, radial) in radials.iter_mut().enumerate() {
            if r == center_radial {
                continue;
            }
            // Alternate high/low so mean ~10, variance large, around the
            // center gate's window.
            let v = if r % 2 == 0 { 40.0 } else { -20.0 };
            radial.gates[center_gate] = Some(v);
        }

        let sweep = SweepData {
            elevation_deg: 0.5,
            radials,
            first_gate_km: 2.125,
            gate_spacing_km: 0.25,
            nyquist_ms: 0.0,
        };

        let center_gate_after = |vel_sd_censor_enabled: bool| {
            clean_sweep(
                &sweep,
                Product::Velocity,
                &QcConfig {
                    tdbz_kernel_size: 9,
                    cc_sweep: None,
                    cc_gate_enabled: false,
                    cc_gate_threshold: 0.80,
                    refl_floor_enabled: false,
                    refl_floor_dbz: 7.0,
                    vel_dealias_enabled: true,
                    vel_sd_censor_enabled,
                    vel_sd_threshold: 7.0,
                    ..Default::default()
                },
            )
            .radials[center_radial]
                .gates[center_gate]
        };

        assert_eq!(
            center_gate_after(true),
            None,
            "noisy center gate should be censored"
        );
        assert_eq!(
            center_gate_after(false),
            Some(10.0),
            "censoring disabled preserves the gate"
        );

        // A gate far from the noisy column (window never overlaps it)
        // stays untouched.
        let cleaned = clean_sweep(
            &sweep,
            Product::Velocity,
            &QcConfig {
                tdbz_kernel_size: 9,
                cc_sweep: None,
                cc_gate_enabled: false,
                cc_gate_threshold: 0.80,
                refl_floor_enabled: false,
                refl_floor_dbz: 7.0,
                vel_dealias_enabled: true,
                vel_sd_censor_enabled: true,
                vel_sd_threshold: 7.0,
                ..Default::default()
            },
        );
        assert_eq!(
            cleaned.radials[center_radial].gates[0],
            Some(10.0),
            "smooth-region gate unaffected"
        );

        // Product gating: Reflectivity input is untouched by this pass.
        let mut refl_radials = sweep.radials.clone();
        for r in &mut refl_radials {
            r.gates = vec![Some(30.0); n_gates];
        }
        let refl_sweep = SweepData {
            elevation_deg: 0.5,
            radials: refl_radials,
            first_gate_km: 2.125,
            gate_spacing_km: 0.25,
            nyquist_ms: 0.0,
        };
        let refl_cleaned = clean_sweep(
            &refl_sweep,
            Product::Reflectivity,
            &QcConfig {
                tdbz_kernel_size: 9,
                cc_sweep: None,
                cc_gate_enabled: false,
                cc_gate_threshold: 0.80,
                refl_floor_enabled: false,
                refl_floor_dbz: 7.0,
                vel_dealias_enabled: true,
                vel_sd_censor_enabled: true,
                vel_sd_threshold: 7.0,
                ..Default::default()
            },
        );
        assert_eq!(
            refl_cleaned.radials[center_radial].gates[center_gate],
            Some(30.0),
            "Reflectivity must not be affected by velocity SD censor"
        );
    }

    #[test]
    fn fuzzy_nonmet_gate_nulls_bird_radial_keeps_precip_radial() {
        // Two REF radials at well-separated azimuths so nearest-azimuth
        // matching against the aux sweeps is unambiguous. tdbz_kernel_size=3
        // -> half_window=1, so with only 2 gates per radial the texture
        // window covers the whole radial: a single gate-to-gate step sets
        // the SD for both gate indices.
        let ref_sweep = SweepData {
            elevation_deg: 0.5,
            radials: vec![
                radial(0.0, vec![Some(30.0), Some(30.0)]),  // "bird" radial
                radial(90.0, vec![Some(40.0), Some(40.0)]), // "precip" radial
            ],
            first_gate_km: 2.125,
            gate_spacing_km: 0.25,
            nyquist_ms: 0.0,
        };
        // CC: low (0.5) on the bird radial, high (0.98) on the precip radial.
        let cc_sweep = SweepData {
            elevation_deg: 0.5,
            radials: vec![
                radial(0.0, vec![Some(0.5), Some(0.5)]),
                radial(90.0, vec![Some(0.98), Some(0.98)]),
            ],
            first_gate_km: 2.125,
            gate_spacing_km: 0.25,
            nyquist_ms: 0.0,
        };
        // ZDR: an 8 dB gate-to-gate step on the bird radial (SD(ZDR)=8dB,
        // well past the 3dB non-met breakpoint); flat (SD=0) on precip.
        let zdr_sweep = SweepData {
            elevation_deg: 0.5,
            radials: vec![
                radial(0.0, vec![Some(0.0), Some(8.0)]),
                radial(90.0, vec![Some(0.0), Some(0.0)]),
            ],
            first_gate_km: 2.125,
            gate_spacing_km: 0.25,
            nyquist_ms: 0.0,
        };
        // PhiDP: a 40 deg gate-to-gate step on the bird radial (SD(PhiDP)=40,
        // past the 30deg non-met breakpoint); flat (SD=0) on precip.
        let phidp_sweep = SweepData {
            elevation_deg: 0.5,
            radials: vec![
                radial(0.0, vec![Some(0.0), Some(40.0)]),
                radial(90.0, vec![Some(0.0), Some(0.0)]),
            ],
            first_gate_km: 2.125,
            gate_spacing_km: 0.25,
            nyquist_ms: 0.0,
        };

        let cleaned = clean_sweep(
            &ref_sweep,
            Product::Reflectivity,
            &QcConfig {
                tdbz_kernel_size: 3,
                cc_sweep: Some(&cc_sweep),
                zdr_sweep: Some(&zdr_sweep),
                phidp_sweep: Some(&phidp_sweep),
                nonmet_fuzzy_enabled: true,
                nonmet_threshold: nonmet::NONMET_THRESHOLD_DEFAULT,
                ..Default::default()
            },
        );
        assert_eq!(
            cleaned.radials[0].gates,
            vec![None, None],
            "bird radial (low CC, noisy ZDR/PhiDP) should be nulled"
        );
        assert_eq!(
            cleaned.radials[1].gates,
            vec![Some(40.0), Some(40.0)],
            "precip radial (high CC, smooth ZDR/PhiDP) should be preserved"
        );
    }

    #[test]
    fn fuzzy_nonmet_gate_fails_open_with_no_aux_sweeps() {
        // Fuzzy enabled but every aux sweep absent -> nonmet_score is always
        // 0.0 (see nonmet::nonmet_score), so REF is left untouched, same as
        // if the classifier weren't wired in at all.
        let ref_sweep = SweepData {
            elevation_deg: 0.5,
            radials: vec![radial(0.0, vec![Some(30.0), Some(30.0)])],
            first_gate_km: 2.125,
            gate_spacing_km: 0.25,
            nyquist_ms: 0.0,
        };
        let cleaned = clean_sweep(
            &ref_sweep,
            Product::Reflectivity,
            &QcConfig {
                tdbz_kernel_size: 3,
                nonmet_fuzzy_enabled: true,
                nonmet_threshold: nonmet::NONMET_THRESHOLD_DEFAULT,
                ..Default::default()
            },
        );
        assert_eq!(cleaned.radials[0].gates, vec![Some(30.0), Some(30.0)]);
    }
}
