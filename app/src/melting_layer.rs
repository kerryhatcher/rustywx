//! Melting-layer hint: an annotation overlay, not a QC null and not a full
//! raster product (FMH-11C §2.8.1 HCA melting-layer gate).
//!
//! Rain below the melting layer and dry snow/ice above it both correlate
//! well (high CC); the mixed-phase hydrometeors in between (rain-coated
//! snow/ice) scatter less coherently, depressing CC into ~0.85-0.95. At a
//! mid-elevation tilt, averaged across azimuth, that shows up as a genuine
//! local *dip* in the CC-vs-range profile flanked on both sides by higher
//! CC "shoulders" — as opposed to a uniformly moderate CC field, which has
//! no shoulders and is therefore not a melting-layer signature (just
//! ordinary noisy/mixed returns).
//!
//! Detection scans the CC-vs-range profile (mean CC over all azimuths, per
//! range gate) of each mid-elevation CC sweep for a local minimum that
//! sits inside the depressed-CC band, has enough azimuthal coverage to
//! call it an annular ring rather than an isolated clutter/biology patch,
//! and is far enough below its shoulders to be a real dip rather than
//! noise. The height is then estimated from the range/elevation of that
//! minimum via the standard 4/3-effective-earth-radius beam-height
//! equation. Rendered on the scope by `scope::draw_melting_layer_ring`,
//! toggled by `Settings::melting_layer_hint_enabled`.

use crate::model::SweepData;

/// Mid-elevation window searched for the melting-layer ring. Low tilts
/// only sample the melting layer at long range (thin coverage per volume,
/// more ground-clutter contamination); high tilts overshoot it at useful
/// ranges. calibration knob — FMH-11C §2.8.1 gives no single fixed band;
/// 4-10 deg is the commonly used operational window for bright-band ID.
/// Upgrade path: pick per-VCP from the volume's actual elevation list
/// instead of a fixed degree band.
const MID_ELEV_MIN_DEG: f32 = 4.0;
const MID_ELEV_MAX_DEG: f32 = 10.0;

/// CC band characteristic of the melting layer (see module docs / plan).
const CC_RING_MIN: f32 = 0.85;
const CC_RING_MAX: f32 = 0.95;

/// Minimum fraction of radials that must have a valid CC value at a range
/// gate for that gate's dip to count as annular rather than a small
/// azimuthally-confined patch (clutter, biology, a single noisy radial).
/// calibration knob.
const MIN_RING_COVERAGE: f32 = 0.5;

/// Gates on either side of a candidate dip averaged into its "shoulder"
/// reference. calibration knob — wide enough to ride out per-gate noise,
/// narrow enough to stay local to the dip.
const SHOULDER_GATES: usize = 4;

/// Minimum CC the dip must sit below *both* shoulders by to count as a real
/// depression rather than noise around an otherwise flat/uniform CC field.
/// calibration knob.
const MIN_PROMINENCE: f32 = 0.03;

/// Effective earth radius under standard (4/3) atmospheric refraction, km
/// — the standard NWS/FMH-11C radar beam-height approximation. Height is
/// above radar level; antenna height AGL isn't modeled anywhere else in
/// the app either (`geo::RadarSite` carries no elevation field), so it's
/// omitted here too.
const EFFECTIVE_EARTH_RADIUS_KM: f32 = 8_494.67; // 4/3 * 6371.0 km

/// A detected melting-layer estimate: the elevation/range of the CC-
/// minimum ring, and the height derived from them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeltingLayerHint {
    pub elevation_deg: f32,
    pub range_km: f32,
    pub height_km: f32,
}

/// Scan every mid-elevation CC sweep for the strongest depressed-CC ring
/// and return the best one found (by prominence below its shoulders), or
/// `None` if no sweep has a qualifying ring.
pub fn detect(cc_sweeps: &[SweepData]) -> Option<MeltingLayerHint> {
    cc_sweeps
        .iter()
        .filter(|s| s.elevation_deg >= MID_ELEV_MIN_DEG && s.elevation_deg <= MID_ELEV_MAX_DEG)
        .filter_map(detect_ring_in_sweep)
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(hint, _prominence)| hint)
}

/// Find the strongest qualifying CC-minimum ring in one sweep, if any.
/// Returns the hint plus its prominence (the margin below its shoulders)
/// so `detect` can pick the clearest ring across candidate sweeps.
fn detect_ring_in_sweep(sweep: &SweepData) -> Option<(MeltingLayerHint, f32)> {
    let n_radials = sweep.radials.len();
    let n_gates = sweep
        .radials
        .iter()
        .map(|r| r.gates.len())
        .max()
        .unwrap_or(0);
    if n_radials == 0 || n_gates < 2 * SHOULDER_GATES + 1 {
        return None;
    }

    // Mean CC per range-gate index (averaged over azimuth), plus a count of
    // radials whose *own* value at that gate falls in the depressed-CC
    // band — used to measure how much of the azimuthal circle actually
    // carries the dip, as opposed to merely having a valid CC reading.
    let mut sum = vec![0.0f32; n_gates];
    let mut count = vec![0u32; n_gates];
    let mut band_count = vec![0u32; n_gates];
    for radial in &sweep.radials {
        for (i, gate) in radial.gates.iter().enumerate() {
            if let Some(cc) = gate {
                sum[i] += cc;
                count[i] += 1;
                if (CC_RING_MIN..=CC_RING_MAX).contains(cc) {
                    band_count[i] += 1;
                }
            }
        }
    }
    let mean_cc: Vec<Option<f32>> = sum
        .iter()
        .zip(&count)
        .map(|(&s, &c)| if c > 0 { Some(s / c as f32) } else { None })
        .collect();

    let mut best: Option<(usize, f32)> = None; // (gate index, prominence)
    for i in SHOULDER_GATES..n_gates - SHOULDER_GATES {
        let Some(mean) = mean_cc[i] else { continue };
        if !(CC_RING_MIN..=CC_RING_MAX).contains(&mean) {
            continue;
        }
        let coverage = band_count[i] as f32 / n_radials as f32;
        if coverage < MIN_RING_COVERAGE {
            continue;
        }
        let Some(before) = shoulder_mean(&mean_cc, i - SHOULDER_GATES, i) else {
            continue;
        };
        let Some(after) = shoulder_mean(&mean_cc, i + 1, i + 1 + SHOULDER_GATES) else {
            continue;
        };
        let prominence = (before - mean).min(after - mean);
        if prominence < MIN_PROMINENCE {
            continue;
        }
        if best.is_none_or(|(_, best_prom)| prominence > best_prom) {
            best = Some((i, prominence));
        }
    }

    let (idx, prominence) = best?;
    let range_km = sweep.first_gate_km + idx as f32 * sweep.gate_spacing_km;
    let hint = MeltingLayerHint {
        elevation_deg: sweep.elevation_deg,
        range_km,
        height_km: beam_height_km(range_km, sweep.elevation_deg),
    };
    Some((hint, prominence))
}

/// Average of the valid (`Some`) entries in `mean_cc[start..end)`, or
/// `None` if the window has no valid gates.
fn shoulder_mean(mean_cc: &[Option<f32>], start: usize, end: usize) -> Option<f32> {
    let vals: Vec<f32> = mean_cc[start..end].iter().filter_map(|v| *v).collect();
    if vals.is_empty() {
        None
    } else {
        Some(vals.iter().sum::<f32>() / vals.len() as f32)
    }
}

/// Standard NWS/FMH-11C beam-height equation under 4/3-effective-earth-
/// radius refraction: height above radar level, km, for a given slant
/// range (km) and elevation angle (deg).
fn beam_height_km(range_km: f32, elevation_deg: f32) -> f32 {
    let el = elevation_deg.to_radians();
    let re = EFFECTIVE_EARTH_RADIUS_KM;
    (range_km * range_km + re * re + 2.0 * range_km * re * el.sin()).sqrt() - re
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::RadialData;

    /// Build a synthetic CC sweep: `n_radials` evenly spaced azimuths, each
    /// with `n_gates` gates. `ring_gate` gates in `ring_radials` radials get
    /// `dip_cc`; the `SHOULDER_GATES`-wide bands on either side get
    /// `shoulder_cc`; everything else gets `background_cc` (kept outside
    /// the depressed-CC band so it can't be mistaken for a ring).
    #[allow(clippy::too_many_arguments)]
    fn ring_sweep(
        elevation_deg: f32,
        n_radials: usize,
        n_gates: usize,
        ring_gate: usize,
        ring_radials: usize,
        dip_cc: f32,
        shoulder_cc: f32,
        background_cc: f32,
    ) -> SweepData {
        let radials = (0..n_radials)
            .map(|r| {
                let gates = (0..n_gates)
                    .map(|g| {
                        if g == ring_gate && r < ring_radials {
                            Some(dip_cc)
                        } else if ring_gate.abs_diff(g) <= SHOULDER_GATES && g != ring_gate {
                            Some(shoulder_cc)
                        } else {
                            Some(background_cc)
                        }
                    })
                    .collect();
                RadialData {
                    azimuth_deg: r as f32 * 360.0 / n_radials as f32,
                    gates,
                    range_folded: vec![],
                }
            })
            .collect();
        SweepData {
            elevation_deg,
            radials,
            first_gate_km: 1.0,
            gate_spacing_km: 1.0,
            nyquist_ms: 0.0,
        }
    }

    #[test]
    fn detects_ring_at_known_range_and_elevation() {
        let sweep = ring_sweep(6.0, 36, 60, 30, 36, 0.90, 0.97, 0.5);
        let hint = detect(std::slice::from_ref(&sweep)).expect("ring should be detected");

        assert_eq!(hint.elevation_deg, 6.0);
        // first_gate_km(1.0) + 30 * gate_spacing_km(1.0) = 31.0 km.
        assert!((hint.range_km - 31.0).abs() < 1e-4);

        // Independent cross-check of the height against the standard
        // leading-order beam-height approximation
        // h ~= r*sin(el) + r^2 / (2 * k*Re) — the error term the full
        // closed-form formula corrects for is a few millimetres at this
        // range, so a tight tolerance still catches a wrong formula.
        let r = hint.range_km as f64;
        let el = (6.0f64).to_radians();
        let ke_re = 8_494.67f64;
        let approx = r * el.sin() + (r * r) / (2.0 * ke_re);
        assert!(
            (hint.height_km as f64 - approx).abs() < 0.01,
            "height {} not close to approx {}",
            hint.height_km,
            approx
        );
    }

    #[test]
    fn flat_cc_yields_no_detection() {
        // Uniform CC inside the depressed band everywhere — no shoulders,
        // so no genuine dip, so no false ring.
        let sweep = ring_sweep(6.0, 36, 60, 30, 36, 0.90, 0.90, 0.90);
        assert!(detect(std::slice::from_ref(&sweep)).is_none());
    }

    #[test]
    fn ring_outside_mid_elevation_window_is_ignored() {
        // Same qualifying ring, but at a low tilt outside the searched band.
        let sweep = ring_sweep(0.5, 36, 60, 30, 36, 0.90, 0.97, 0.5);
        assert!(detect(std::slice::from_ref(&sweep)).is_none());
    }

    #[test]
    fn isolated_patch_below_coverage_threshold_is_not_a_ring() {
        // Realistic near-1.0 background (typical non-melting-layer CC),
        // with the dip present on only 10 of 36 radials (~28% coverage,
        // below the 50% MIN_RING_COVERAGE threshold). The dip still pulls
        // the all-radial mean at that gate into the 0.85-0.95 band
        // (0.98 - 10*(0.98-0.85)/36 ~= 0.944) with prominence above
        // MIN_PROMINENCE, so only a correctly-implemented coverage check
        // (measuring per-radial band membership, not `Option::is_some()`)
        // rejects this as a clutter/biology patch rather than a ring.
        let sweep = ring_sweep(6.0, 36, 60, 30, 10, 0.85, 0.98, 0.98);
        assert!(detect(std::slice::from_ref(&sweep)).is_none());
    }

    #[test]
    fn empty_input_yields_no_detection() {
        assert!(detect(&[]).is_none());
    }
}
