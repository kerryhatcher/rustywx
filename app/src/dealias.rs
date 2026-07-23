//! 1D gate-to-gate Doppler velocity unfold (FMH-11B §4.3.3).
//!
//! Radial velocities beyond the Nyquist interval alias (wrap) to the wrong
//! sign — a strong inbound/outbound couplet reads as a sign-flip instead of
//! a coherent shear signature. This module walks a single radial outward
//! from the radar and shifts each gate by an integer multiple of the
//! Nyquist interval so it stays continuous with its inbound neighbor.
//!
//! Azimuthal (2D) continuity, couplet preservation, and a VAD-wind fallback
//! are Stage C — see `docs/velocity-dealiasing-plan.md`. This stage is
//! deliberately just the gate-to-gate walk.

/// Consecutive missing-gate run length after which the dealias reference
/// resets. Long enough to bridge an ordinary beam gap, short enough that a
/// distant, unrelated echo doesn't inherit a stale reference across it.
const MAX_GAP_GATES: usize = 5;

/// Unfold one radial's gate-to-gate Doppler velocities beyond ±`nyquist`.
///
/// `nyquist` is the Nyquist velocity in m/s (not the interval — the
/// interval is `2 * nyquist`). `nyquist <= 0.0` means "unknown" (decode
/// failure or missing Radial Data Block) — dealiasing is a no-op rather
/// than a guess, matching [`crate::model::format_nyquist_velocity`].
///
/// `None` gates (below-threshold or range-folded) pass straight through
/// and never update the running reference.
pub fn dealias_radial(gates: &[Option<f32>], nyquist: f32) -> Vec<Option<f32>> {
    if nyquist <= 0.0 {
        return gates.to_vec();
    }
    let interval = 2.0 * nyquist;
    let mut out = Vec::with_capacity(gates.len());
    let mut prev: Option<f32> = None;
    let mut gap_run = 0usize;

    for &gate in gates {
        match gate {
            None => {
                out.push(None);
                gap_run += 1;
                if gap_run >= MAX_GAP_GATES {
                    prev = None;
                }
            }
            Some(v) => {
                gap_run = 0;
                let unfolded = match prev {
                    // Seed from the first valid gate: assumed to already be
                    // within the primary interval.
                    None => v,
                    Some(p) => {
                        let k = ((p - v) / interval).round();
                        v + k * interval
                    }
                };
                out.push(Some(unfolded));
                prev = Some(unfolded);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_when_nyquist_unknown() {
        let gates = vec![Some(28.0), Some(-25.0), None, Some(30.0)];
        assert_eq!(dealias_radial(&gates, 0.0), gates);
    }

    #[test]
    fn unfolds_single_fold_to_monotone_ramp() {
        // Nyquist 26.4 m/s -> interval 52.8. True wind ramps up past
        // +Nyquist and aliases to a negative value; dealiasing should
        // restore gate-to-gate continuity.
        let gates = vec![Some(20.0), Some(24.0), Some(28.0), Some(-25.0), Some(-20.0)];
        let out = dealias_radial(&gates, 26.4);
        let vals: Vec<f32> = out.iter().map(|g| g.unwrap()).collect();
        assert_eq!(vals[0], 20.0);
        assert_eq!(vals[1], 24.0);
        assert_eq!(vals[2], 28.0);
        assert!((vals[3] - 27.8).abs() < 1e-3, "got {:?}", vals);
        assert!((vals[4] - 32.8).abs() < 1e-3, "got {:?}", vals);
        // The fold is closed: no gate-to-gate jump exceeds the Nyquist
        // velocity itself (the raw data had a ~53 m/s jump at the fold).
        for i in 1..vals.len() {
            assert!(
                (vals[i] - vals[i - 1]).abs() <= 26.4,
                "gate-to-gate jump too large at index {i}: {vals:?}"
            );
        }
    }

    #[test]
    fn missing_gate_run_resets_reference() {
        // A run of MAX_GAP_GATES missing gates severs continuity: the gate
        // beyond the gap seeds fresh instead of unfolding toward the stale
        // reference before it.
        let mut gates = vec![Some(28.0)];
        gates.extend(std::iter::repeat_n(None, MAX_GAP_GATES));
        gates.push(Some(-25.0));
        let out = dealias_radial(&gates, 26.4);
        assert_eq!(*out.last().unwrap(), Some(-25.0), "reference should reset");
    }

    #[test]
    fn short_gap_bridges_reference() {
        // Fewer than MAX_GAP_GATES missing gates must NOT reset the
        // reference — the fold on the far side still gets corrected.
        let gates = vec![Some(28.0), None, None, Some(-25.0)];
        let out = dealias_radial(&gates, 26.4);
        assert!((out[3].unwrap() - 27.8).abs() < 1e-3);
    }
}
