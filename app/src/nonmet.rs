//! Fuzzy multi-variable non-meteorological (non-met) classifier.
//!
//! Replaces a single-threshold CC gate with a weighted membership score over
//! CC + SD(ΦDP) + SD(ZDR) (+ optional |ZDR| magnitude). Per FMH-11A, ZDR and
//! ΦDP each independently flag non-met targets (birds, insects, chaff, AP,
//! ground clutter) that a CC-only gate misses; combining them gives fewer
//! false nulls on real precip *and* better biology/chaff/AP rejection.
//!
//! Pure module — no wiring into `scope.rs`/`main.rs` yet. See
//! `docs/fuzzy-nonmet-classifier-plan.md`.

/// CC membership: 1.0 (non-met) at cc<=0.80, ramp to 0.0 (precip) by cc>=0.95.
/// Precip has high correlation coefficient; birds/chaff/AP/clutter are lower.
/// calibration knob — FMH-11A typical range, not a universal constant.
const CC_NONMET_MAX: f32 = 0.80;
const CC_PRECIP_MIN: f32 = 0.95;

/// SD(ΦDP) membership: 0.0 (precip) at sd<=10°, ramp to 1.0 (non-met) by sd>=30°.
/// calibration knob.
const SD_PHIDP_PRECIP_MAX: f32 = 10.0;
const SD_PHIDP_NONMET_MIN: f32 = 30.0;

/// SD(ZDR) membership: 0.0 (precip) at sd<=1dB, ramp to 1.0 (non-met) by sd>=3dB.
/// calibration knob.
const SD_ZDR_PRECIP_MAX: f32 = 1.0;
const SD_ZDR_NONMET_MIN: f32 = 3.0;

/// |ZDR| magnitude membership: 0.0 (precip) at |zdr|<=4dB, ramp to 1.0
/// (non-met) by |zdr|>=6dB. Optional term — large |ZDR| is typical of biology
/// (birds/insects) rather than precip. calibration knob.
const ZDR_MAG_PRECIP_MAX: f32 = 4.0;
const ZDR_MAG_NONMET_MIN: f32 = 6.0;

/// Weighted-mean weights over `[mu_cc, mu_sd_phidp, mu_sd_zdr, mu_zdr_mag]`.
/// SD(ΦDP) and CC dominate — they're the operational discriminators; the
/// other two are secondary evidence. calibration knobs, normalized over
/// whichever inputs are present so a subset of sweeps still sums to weight 1.
const W_CC: f32 = 0.35;
const W_SD_PHIDP: f32 = 0.4;
const W_SD_ZDR: f32 = 0.2;
const W_ZDR_MAG: f32 = 0.05;

/// Non-met threshold: score at/above this nulls the co-located REF gate.
pub const NONMET_THRESHOLD_DEFAULT: f32 = 0.5;

/// Clamped linear ramp from 0.0 at `lo` to 1.0 at `hi` (or the mirror image
/// if `lo > hi`). Shared shape behind every membership function below.
fn ramp(x: f32, lo: f32, hi: f32) -> f32 {
    if lo == hi {
        return if x >= hi { 1.0 } else { 0.0 };
    }
    ((x - lo) / (hi - lo)).clamp(0.0, 1.0)
}

/// Non-met membership from correlation coefficient. 1.0 = definitely
/// non-met, 0.0 = definitely precip. Falling ramp: high CC -> low membership.
fn mu_cc(cc: f32) -> f32 {
    1.0 - ramp(cc, CC_NONMET_MAX, CC_PRECIP_MIN)
}

/// Non-met membership from ΦDP range-texture standard deviation (degrees).
/// Rising ramp: precip's ΦDP is range-smooth (low SD); non-met is noisy.
fn mu_sd_phidp(sd: f32) -> f32 {
    ramp(sd, SD_PHIDP_PRECIP_MAX, SD_PHIDP_NONMET_MIN)
}

/// Non-met membership from ZDR range-texture standard deviation (dB).
fn mu_sd_zdr(sd: f32) -> f32 {
    ramp(sd, SD_ZDR_PRECIP_MAX, SD_ZDR_NONMET_MIN)
}

/// Non-met membership from |ZDR| magnitude (dB). Optional term.
fn mu_zdr_mag(zdr: f32) -> f32 {
    ramp(zdr.abs(), ZDR_MAG_PRECIP_MAX, ZDR_MAG_NONMET_MIN)
}

/// Weighted non-met score in `[0,1]` over whichever inputs are present.
///
/// Fail-open per variable: any `None` input drops out of the weighted mean
/// (and its weight is excluded from the normalizer) rather than being
/// treated as evidence either way. If only `cc` is available the score
/// reduces to a pure `mu_cc` gate — a legacy single-pol / CC-only volume
/// degrades gracefully to the old CC-gate behavior, never a worse mask.
/// All-`None` inputs score 0.0 (never null).
pub fn nonmet_score(
    cc: Option<f32>,
    sd_phidp: Option<f32>,
    sd_zdr: Option<f32>,
    zdr: Option<f32>,
) -> f32 {
    let terms: [(Option<f32>, f32, fn(f32) -> f32); 4] = [
        (cc, W_CC, mu_cc),
        (sd_phidp, W_SD_PHIDP, mu_sd_phidp),
        (sd_zdr, W_SD_ZDR, mu_sd_zdr),
        (zdr, W_ZDR_MAG, mu_zdr_mag),
    ];
    let mut weighted_sum = 0.0f32;
    let mut weight_total = 0.0f32;
    for (value, weight, membership) in terms {
        if let Some(v) = value {
            weighted_sum += weight * membership(v);
            weight_total += weight;
        }
    }
    if weight_total == 0.0 {
        return 0.0;
    }
    weighted_sum / weight_total
}

/// Gate-to-gate RMS texture along a radial, generalizing the TDBZ inner loop
/// (`scope.rs`) to any moment. `half_window` mirrors `tdbz_kernel_size / 2`.
///
/// `wrap_deg`: pass `Some(360.0)` for a variable that wraps (ΦDP, 0-360°) so
/// each gate-to-gate difference is folded into `[-wrap/2, wrap/2]` before
/// squaring — otherwise a 358°->2° step reads as a ~356° spike instead of the
/// true ~4° step. Pass `None` for a variable that doesn't wrap (ZDR).
pub fn range_texture(gates: &[Option<f32>], half_window: usize, wrap_deg: Option<f32>) -> Vec<f32> {
    let n = gates.len();
    let mut out = vec![0.0f32; n];
    for i in 0..n {
        let start = i.saturating_sub(half_window);
        let end = (i + half_window + 1).min(n);
        if end - start < 2 {
            continue;
        }
        let mut sum_sq = 0.0f32;
        let mut diff_count = 0u32;
        for j in start..end {
            if j + 1 >= end {
                continue;
            }
            if let (Some(a), Some(b)) = (gates[j], gates[j + 1]) {
                let mut diff = b - a;
                if let Some(wrap) = wrap_deg {
                    diff -= wrap * (diff / wrap).round();
                }
                sum_sq += diff * diff;
                diff_count += 1;
            }
        }
        if diff_count > 0 {
            out[i] = (sum_sq / diff_count as f32).sqrt();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- mu_cc ---

    #[test]
    fn mu_cc_breakpoints_and_range() {
        assert_eq!(mu_cc(0.80), 1.0);
        assert_eq!(mu_cc(0.5), 1.0); // clamped below the breakpoint
        assert_eq!(mu_cc(0.95), 0.0);
        assert_eq!(mu_cc(1.0), 0.0); // clamped above
        let mid = mu_cc(0.875);
        assert!((mid - 0.5).abs() < 1e-6);
    }

    #[test]
    fn mu_cc_monotonic_decreasing() {
        let a = mu_cc(0.82);
        let b = mu_cc(0.90);
        assert!(a >= b);
    }

    // --- mu_sd_phidp ---

    #[test]
    fn mu_sd_phidp_breakpoints_and_range() {
        assert_eq!(mu_sd_phidp(10.0), 0.0);
        assert_eq!(mu_sd_phidp(0.0), 0.0);
        assert_eq!(mu_sd_phidp(30.0), 1.0);
        assert_eq!(mu_sd_phidp(90.0), 1.0);
        let mid = mu_sd_phidp(20.0);
        assert!((mid - 0.5).abs() < 1e-6);
    }

    #[test]
    fn mu_sd_phidp_monotonic_increasing() {
        let a = mu_sd_phidp(12.0);
        let b = mu_sd_phidp(25.0);
        assert!(a <= b);
    }

    // --- mu_sd_zdr ---

    #[test]
    fn mu_sd_zdr_breakpoints_and_range() {
        assert_eq!(mu_sd_zdr(1.0), 0.0);
        assert_eq!(mu_sd_zdr(3.0), 1.0);
        let mid = mu_sd_zdr(2.0);
        assert!((mid - 0.5).abs() < 1e-6);
    }

    // --- mu_zdr_mag ---

    #[test]
    fn mu_zdr_mag_breakpoints_and_symmetric_in_sign() {
        assert_eq!(mu_zdr_mag(4.0), 0.0);
        assert_eq!(mu_zdr_mag(-4.0), 0.0); // |zdr|, sign doesn't matter
        assert_eq!(mu_zdr_mag(6.0), 1.0);
        assert_eq!(mu_zdr_mag(-6.0), 1.0);
        let mid = mu_zdr_mag(5.0);
        assert!((mid - 0.5).abs() < 1e-6);
    }

    // --- nonmet_score ---

    #[test]
    fn nonmet_score_bird_gate_scores_high() {
        // CC 0.5, SD(PhiDP) 40 deg, SD(ZDR) 4 dB -> all pegged to full non-met
        // membership; score should clear the default threshold comfortably.
        let score = nonmet_score(Some(0.5), Some(40.0), Some(4.0), None);
        assert!(score >= NONMET_THRESHOLD_DEFAULT, "score={score}");
    }

    #[test]
    fn nonmet_score_precip_gate_scores_low() {
        // CC 0.98, SD(PhiDP) 5 deg, SD(ZDR) 0.5 dB -> all pegged to full precip
        // membership; score should stay well under the default threshold.
        let score = nonmet_score(Some(0.98), Some(5.0), Some(0.5), None);
        assert!(score < NONMET_THRESHOLD_DEFAULT, "score={score}");
    }

    #[test]
    fn nonmet_score_cc_only_matches_mu_cc() {
        // Fail-open: with only CC present, the weighted mean over one term
        // reduces to that term's membership exactly (old CC-gate behavior).
        for cc in [0.5f32, 0.80, 0.875, 0.95, 0.99] {
            let score = nonmet_score(Some(cc), None, None, None);
            assert!((score - mu_cc(cc)).abs() < 1e-6, "cc={cc} score={score}");
        }
    }

    #[test]
    fn nonmet_score_all_none_is_zero() {
        assert_eq!(nonmet_score(None, None, None, None), 0.0);
    }

    // --- range_texture ---

    #[test]
    fn range_texture_flat_signal_is_zero() {
        let gates: Vec<Option<f32>> = vec![Some(0.98); 10];
        let tex = range_texture(&gates, 2, None);
        assert!(tex.iter().all(|&t| t.abs() < 1e-6));
    }

    #[test]
    fn range_texture_phidp_wrap_yields_near_zero_not_a_spike() {
        // A radial stepping 358 -> 2 -> 6 -> 10 degrees is a physically smooth
        // +4 deg/gate ramp that wrapped past 360; without wrap-folding this
        // reads as a ~356 deg spike. This is the decisive wrap test.
        let gates: Vec<Option<f32>> = vec![Some(358.0), Some(2.0), Some(6.0), Some(10.0)];
        let tex = range_texture(&gates, 1, Some(360.0));
        for (i, &t) in tex.iter().enumerate() {
            assert!(t < 10.0, "gate {i} texture too high: {t}");
        }

        // Without the wrap fold, the same data spikes hugely at the 358->2 step.
        let tex_unwrapped = range_texture(&gates, 1, None);
        assert!(tex_unwrapped.iter().any(|&t| t > 100.0));
    }

    #[test]
    fn range_texture_zdr_no_wrap_short_window_returns_zero() {
        let gates: Vec<Option<f32>> = vec![Some(0.0)];
        let tex = range_texture(&gates, 1, None);
        assert_eq!(tex, vec![0.0]);
    }

    #[test]
    fn range_texture_none_gates_are_skipped() {
        let gates: Vec<Option<f32>> = vec![Some(1.0), None, Some(1.0), Some(1.0)];
        let tex = range_texture(&gates, 2, None);
        assert!(tex.iter().all(|&t| t.abs() < 1e-6));
    }
}
