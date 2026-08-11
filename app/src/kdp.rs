//! Specific differential phase (KDP), derived from ΦDP (FMH-11C §2.5.3).
//!
//! KDP is the range-derivative of differential phase — an attenuation- and
//! calibration-immune rain-rate proxy. Unlike the other five products, it
//! isn't decoded from a radar moment; it's computed here from the already-
//! decoded ΦDP sweeps and stored on `ScanData` as a normal `Vec<SweepData>`
//! so it flows through cache/rasterize unchanged (see `data.rs`).
//!
//! Per-radial pipeline:
//! 1. Unwrap the wrapping 0-360° ΦDP field into a continuous phase (same
//!    fold idea as `nonmet::range_texture`'s `wrap_deg` handling).
//! 2. Median filter (despeckle).
//! 3. Average filter (smooth).
//! 4. Linearly interpolate across interior meteorological gaps.
//! 5. Range-derivative over a limited increment: `KDP = 0.5 * dΦDP/dr`
//!    (deg/km) — gate-to-gate would be far too noisy.

use crate::model::{RadialData, SweepData};

/// Median-filter half-window (gates each side) — despeckle pass.
const MEDIAN_HALF_WINDOW: usize = 2;
/// Average-filter half-window (gates each side) — smoothing pass.
const AVERAGE_HALF_WINDOW: usize = 2;
/// Range-derivative half-window (gates each side). KDP is the derivative
/// over a limited range increment, not gate-to-gate — gate-to-gate noise
/// would dominate. calibration knob: fixed gate count, not a physical
/// range increment, so it narrows/widens with the sweep's gate spacing;
/// upgrade path is deriving this from `gate_spacing_km` if that misfires
/// across legacy (1 km) vs. super-res (0.25 km) geometries.
const DERIVATIVE_HALF_WINDOW: usize = 6;

/// ΦDP wraps at this many degrees (0-360°).
const PHIDP_WRAP_DEG: f32 = 360.0;

/// Derive KDP sweeps from ΦDP sweeps. Same shape as the input (elevation,
/// gate geometry, radial/gate counts) — empty in, empty out, matching how
/// `ScanData` treats absent dual-pol products on legacy scans.
pub fn derive_kdp_sweeps(phidp_sweeps: &[SweepData]) -> Vec<SweepData> {
    phidp_sweeps
        .iter()
        .map(|sweep| SweepData {
            elevation_deg: sweep.elevation_deg,
            first_gate_km: sweep.first_gate_km,
            gate_spacing_km: sweep.gate_spacing_km,
            nyquist_ms: sweep.nyquist_ms,
            radials: sweep
                .radials
                .iter()
                .map(|radial| RadialData {
                    azimuth_deg: radial.azimuth_deg,
                    gates: kdp_radial(&radial.gates, sweep.gate_spacing_km),
                    // Derived product, not a decoded moment — no range-fold
                    // concept of its own. Empty = "unknown/false", per the
                    // `RadialData::range_folded` doc comment.
                    range_folded: Vec::new(),
                })
                .collect(),
        })
        .collect()
}

/// Full per-radial derivation: unwrap -> despeckle -> smooth -> interpolate
/// -> range-derivative.
fn kdp_radial(phidp_gates: &[Option<f32>], gate_spacing_km: f32) -> Vec<Option<f32>> {
    let unwrapped = unwrap_phidp(phidp_gates);
    let despeckled = median_filter(&unwrapped, MEDIAN_HALF_WINDOW);
    let smoothed = average_filter(&despeckled, AVERAGE_HALF_WINDOW);
    let filled = interpolate_gaps(&smoothed);
    range_derivative(&filled, gate_spacing_km, DERIVATIVE_HALF_WINDOW)
}

/// Unwrap the wrapping 0-360° ΦDP field into a continuous phase.
///
/// Walks only between consecutive `Some` gates (skipping over `None`s), so a
/// missing/below-threshold gate is never mistaken for a real wrap. Each
/// step's raw difference is folded into `[-180, 180]` — the same idea as
/// `nonmet::range_texture`'s `wrap_deg` — before being accumulated onto the
/// running unwrapped total, so a 358°->2° step reads as the true ~+4° step
/// rather than a ~-356° spike.
fn unwrap_phidp(gates: &[Option<f32>]) -> Vec<Option<f32>> {
    let mut out = vec![None; gates.len()];
    let mut last_raw: Option<f32> = None;
    let mut last_unwrapped = 0.0f32;
    for (i, gate) in gates.iter().enumerate() {
        let Some(raw) = gate else { continue };
        let unwrapped = match last_raw {
            None => *raw,
            Some(prev_raw) => {
                let mut diff = raw - prev_raw;
                diff -= PHIDP_WRAP_DEG * (diff / PHIDP_WRAP_DEG).round();
                last_unwrapped + diff
            }
        };
        out[i] = Some(unwrapped);
        last_raw = Some(*raw);
        last_unwrapped = unwrapped;
    }
    out
}

/// Median filter (despeckle) over a `[-half, +half]` window of valid gates.
fn median_filter(gates: &[Option<f32>], half_window: usize) -> Vec<Option<f32>> {
    windowed_filter(gates, half_window, median)
}

/// Average filter (smooth) over a `[-half, +half]` window of valid gates.
fn average_filter(gates: &[Option<f32>], half_window: usize) -> Vec<Option<f32>> {
    windowed_filter(gates, half_window, mean)
}

/// Shared skeleton for the median/average passes: for each `Some` gate,
/// apply `stat` to the valid gates in `[i-half, i+half]`. `None` gates pass
/// through unchanged — a gap isn't data to despeckle/smooth, it's filled
/// later by `interpolate_gaps`.
fn windowed_filter(
    gates: &[Option<f32>],
    half_window: usize,
    stat: fn(&mut [f32]) -> f32,
) -> Vec<Option<f32>> {
    let n = gates.len();
    let mut out = vec![None; n];
    for i in 0..n {
        if gates[i].is_none() {
            continue;
        }
        let start = i.saturating_sub(half_window);
        let end = (i + half_window + 1).min(n);
        let mut window: Vec<f32> = gates[start..end].iter().filter_map(|g| *g).collect();
        out[i] = Some(stat(&mut window));
    }
    out
}

fn median(values: &mut [f32]) -> f32 {
    values.sort_by(f32::total_cmp);
    let n = values.len();
    if n % 2 == 1 {
        values[n / 2]
    } else {
        (values[n / 2 - 1] + values[n / 2]) / 2.0
    }
}

fn mean(values: &mut [f32]) -> f32 {
    values.iter().sum::<f32>() / values.len() as f32
}

/// Linearly interpolate across interior `None` gaps (bounded by valid gates
/// on both sides) — "interpolate across met gates" so a single missing/
/// below-threshold gate doesn't break the range-derivative below. Leading
/// and trailing gaps (no data on one side) are left `None`; there's nothing
/// to interpolate from.
fn interpolate_gaps(gates: &[Option<f32>]) -> Vec<Option<f32>> {
    let n = gates.len();
    let mut out = gates.to_vec();
    let mut i = 0;
    while i < n {
        if out[i].is_some() {
            i += 1;
            continue;
        }
        let gap_start = i;
        while i < n && out[i].is_none() {
            i += 1;
        }
        let gap_end = i; // exclusive; index of the first valid gate after the gap, or n.
        if gap_start == 0 || gap_end == n {
            continue; // leading/trailing gap — nothing to interpolate from.
        }
        let before = out[gap_start - 1].expect("gap_start-1 is the valid gate before the gap");
        let after = out[gap_end].expect("gap_end is the valid gate after the gap");
        let span = (gap_end - gap_start + 1) as f32;
        for (k, idx) in (gap_start..gap_end).enumerate() {
            let t = (k + 1) as f32 / span;
            out[idx] = Some(before + (after - before) * t);
        }
    }
    out
}

/// Range-derivative over a limited increment (not gate-to-gate — too
/// noisy): central difference across `half_window` gates on each side,
/// `KDP = 0.5 * dΦDP/dr` in deg/km. A gate needs both increment endpoints
/// present (post-interpolation) to get a value; otherwise it's `None`.
/// Guards `gate_spacing_km <= 0.0` (bad/absent geometry) and `half_window ==
/// 0` by returning all-`None` rather than dividing by zero.
fn range_derivative(
    gates: &[Option<f32>],
    gate_spacing_km: f32,
    half_window: usize,
) -> Vec<Option<f32>> {
    let n = gates.len();
    let mut out = vec![None; n];
    if gate_spacing_km <= 0.0 || half_window == 0 {
        return out;
    }
    let dr_km = 2.0 * half_window as f32 * gate_spacing_km;
    for i in 0..n {
        if i < half_window || i + half_window >= n {
            continue;
        }
        if let (Some(before), Some(after)) = (gates[i - half_window], gates[i + half_window]) {
            out[i] = Some(0.5 * (after - before) / dr_km);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The interior of `derive_kdp_sweeps` output beyond every filter/
    /// derivative window's reach on both sides — the "smoothed interior"
    /// the plan's test targets, unaffected by edge truncation.
    const SAFE_MARGIN: usize = MEDIAN_HALF_WINDOW + AVERAGE_HALF_WINDOW + DERIVATIVE_HALF_WINDOW;

    /// A radial with a linear, wrapping ΦDP ramp of `deg_per_gate` per gate.
    fn ramp_radial(n: usize, deg_per_gate: f32) -> Vec<Option<f32>> {
        (0..n)
            .map(|g| Some((g as f32 * deg_per_gate).rem_euclid(PHIDP_WRAP_DEG)))
            .collect()
    }

    #[test]
    fn linear_ramp_yields_constant_kdp_in_smoothed_interior() {
        // +2 deg/gate at 0.25 km/gate spacing -> KDP = 0.5 * 2 / 0.25 = 4 deg/km,
        // per the plan. Enough gates to wrap the raw ΦDP at least once,
        // exercising the unwrap step, and to clear every window on both
        // sides of the sampled interior point.
        let gate_spacing_km = 0.25;
        let n = 200;
        let gates = ramp_radial(n, 2.0);
        let kdp = kdp_radial(&gates, gate_spacing_km);

        for (i, v) in kdp
            .iter()
            .enumerate()
            .take(n - SAFE_MARGIN)
            .skip(SAFE_MARGIN)
        {
            let v = v.unwrap_or_else(|| panic!("gate {i} should have a KDP value"));
            assert!(
                (v - 4.0).abs() < 1e-3,
                "gate {i}: expected ~4.0 deg/km, got {v}"
            );
        }
    }

    #[test]
    fn flat_phidp_yields_near_zero_kdp() {
        let gates: Vec<Option<f32>> = vec![Some(90.0); 60];
        let kdp = kdp_radial(&gates, 0.25);
        let n = gates.len();
        for (i, v) in kdp
            .iter()
            .enumerate()
            .take(n - SAFE_MARGIN)
            .skip(SAFE_MARGIN)
        {
            let v = v.unwrap_or_else(|| panic!("gate {i} should have a KDP value"));
            assert!(v.abs() < 1e-3, "gate {i}: expected ~0, got {v}");
        }
    }

    #[test]
    fn zero_gate_spacing_never_divides_by_zero() {
        let gates = ramp_radial(40, 2.0);
        let kdp = kdp_radial(&gates, 0.0);
        assert!(kdp.iter().all(|g| g.is_none()));
    }

    #[test]
    fn unwrap_reconstructs_continuous_phase_across_a_wrap() {
        // Raw ΦDP wraps 358 -> 0 -> 2 (a true +2 deg/gate step each time);
        // the unwrapped phase must keep climbing, not jump down ~356 deg.
        let gates = vec![Some(358.0), Some(0.0), Some(2.0)];
        let unwrapped = unwrap_phidp(&gates);
        let vals: Vec<f32> = unwrapped.into_iter().map(|g| g.unwrap()).collect();
        assert!((vals[1] - vals[0] - 2.0).abs() < 1e-4);
        assert!((vals[2] - vals[1] - 2.0).abs() < 1e-4);
    }

    #[test]
    fn unwrap_skips_over_gaps_without_treating_them_as_wraps() {
        let gates = vec![Some(10.0), None, None, Some(16.0)];
        let unwrapped = unwrap_phidp(&gates);
        assert_eq!(unwrapped[0], Some(10.0));
        assert_eq!(unwrapped[1], None);
        assert_eq!(unwrapped[2], None);
        assert!((unwrapped[3].unwrap() - 16.0).abs() < 1e-4);
    }

    #[test]
    fn interpolate_gaps_fills_interior_but_not_edges() {
        let gates = vec![None, None, Some(0.0), None, None, Some(9.0), None];
        let filled = interpolate_gaps(&gates);
        assert_eq!(filled[0], None, "leading gap stays None");
        assert_eq!(filled[1], None, "leading gap stays None");
        assert_eq!(filled[2], Some(0.0));
        assert!((filled[3].unwrap() - 3.0).abs() < 1e-4);
        assert!((filled[4].unwrap() - 6.0).abs() < 1e-4);
        assert_eq!(filled[5], Some(9.0));
        assert_eq!(filled[6], None, "trailing gap stays None");
    }

    #[test]
    fn derive_kdp_sweeps_preserves_shape_and_handles_empty_input() {
        assert!(derive_kdp_sweeps(&[]).is_empty());

        let sweep = SweepData {
            elevation_deg: 0.5,
            first_gate_km: 2.125,
            gate_spacing_km: 0.25,
            nyquist_ms: 0.0,
            radials: vec![
                RadialData {
                    azimuth_deg: 0.0,
                    gates: ramp_radial(60, 2.0),
                    range_folded: vec![],
                },
                RadialData {
                    azimuth_deg: 1.0,
                    gates: ramp_radial(60, 2.0),
                    range_folded: vec![],
                },
            ],
        };
        let kdp = derive_kdp_sweeps(&[sweep]);
        assert_eq!(kdp.len(), 1);
        assert_eq!(kdp[0].elevation_deg, 0.5);
        assert_eq!(kdp[0].gate_spacing_km, 0.25);
        assert_eq!(kdp[0].radials.len(), 2);
        assert_eq!(kdp[0].radials[0].gates.len(), 60);
        assert_eq!(kdp[0].radials[0].azimuth_deg, 0.0);
        assert_eq!(kdp[0].radials[1].azimuth_deg, 1.0);
    }
}
