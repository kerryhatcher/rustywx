//! Doppler velocity unfold, gate-to-gate then azimuth-to-azimuth
//! (FMH-11B §4.3.3 / FMH-11D §3.3.3).
//!
//! Radial velocities beyond the Nyquist interval alias (wrap) to the wrong
//! sign — a strong inbound/outbound couplet reads as a sign-flip instead of
//! a coherent shear signature. [`dealias_radial`] walks a single radial
//! outward from the radar and shifts each gate by an integer multiple of
//! the Nyquist interval so it stays continuous with its inbound neighbor.
//! [`dealias_sweep`] upgrades that to the full sweep: it also checks each
//! gate against the same range gate on the previous (already-corrected)
//! radial, so two radials don't settle on a different "which interval"
//! answer for what is physically the same feature. Gates that never find a
//! consistent unfold (isolated suspects) fall back to a single-ring VAD
//! environmental-wind estimate rather than a guess.

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

/// Consecutive inconclusive-verdict radials after which the azimuthal
/// reference is dropped instead of kept alive. Mirrors [`MAX_GAP_GATES`]'s
/// reasoning but for radials whose shift can't be confidently decided:
/// without a cutoff, a systematically bad patch would keep being compared
/// against a stale reference for the rest of the sweep.
const MAX_BAD_RADIALS: usize = 5;

/// Minimum valid samples on a range ring before we trust a VAD fit off it.
/// Three unknowns (mean, sin, cos coefficients) need at least that many
/// points to not be a coin flip; a real ring has dozens.
const MIN_VAD_SAMPLES: usize = 8;

/// Unfold a full sweep's Doppler velocities: gate-to-gate ([`dealias_radial`]
/// per radial) then azimuth-to-azimuth continuity across radials at each
/// range gate (FMH-11D §3.3.3).
///
/// `azimuths_deg` and `gates` are parallel and must already be sorted by
/// azimuth ascending — the caller (`scope::clean_sweep`) already builds that
/// ordering for the CC-gating and SD-censor passes, so this reuses it rather
/// than re-deriving it.
///
/// Azimuthal correction is decided per *radial*, not per gate: a candidate
/// whole-interval shift is only applied when a clear majority of the
/// radial's gates agree it's needed against the previous radial (see
/// [`radial_shift_verdict`]). That's what preserves couplets — a real
/// mesocyclone/TVS shear shows up as a handful of gates disagreeing with
/// the previous radial while the rest of that same radial still lines up
/// fine (no shift needed, verdict `Some(0)`), so it's left completely
/// untouched. A radial that's genuinely seeded in the wrong interval instead
/// shows *most* of its gates disagreeing by the same amount, which is what
/// earns it a shift. Once a radial's shift is trusted, any gate that still
/// doesn't line up after applying it is the isolated-suspect case — real
/// aliasing was confirmed for that radial, so a leftover outlier is residue,
/// not signal — and falls back to a single-ring VAD environmental-wind
/// estimate (or `None` if no ring supports a fit — never guess).
///
/// A radial whose verdict is inconclusive (too little gate overlap with the
/// reference, or votes split with no majority) doesn't get shifted either;
/// it's queued and retried against the next radial that *does* get a clear
/// verdict ("reinserted"). After [`MAX_BAD_RADIALS`] consecutive
/// inconclusive radials the reference is dropped (a systematically bad
/// patch shouldn't keep propagating) and the whole queued run is rejected —
/// every gate in it becomes an isolated suspect.
///
/// `nyquist <= 0.0` (unknown) is a no-op, same as [`dealias_radial`].
///
/// ponytail: single forward pass around the circle, not a wraparound
/// double-pass — the first radial in azimuth order has no reference at all
/// (seeded as-is). Good enough for a refinement pass; revisit with a
/// circular double-pass if boundary-azimuth artifacts show up on real
/// volumes.
pub fn dealias_sweep(
    azimuths_deg: &[f32],
    gates: &[Vec<Option<f32>>],
    nyquist: f32,
) -> Vec<Vec<Option<f32>>> {
    if nyquist <= 0.0 || gates.is_empty() {
        return gates.to_vec();
    }
    let interval = 2.0 * nyquist;
    let n_radials = gates.len();

    // Pass 1: gate-to-gate continuity within each radial (Stage B).
    let mut out: Vec<Vec<Option<f32>>> = gates.iter().map(|g| dealias_radial(g, nyquist)).collect();

    // Pass 2: azimuth-to-azimuth continuity, radial by radial.
    let mut suspects: Vec<(usize, usize)> = Vec::new();
    let mut prev: Option<Vec<Option<f32>>> = None;
    let mut pending: Vec<usize> = Vec::new();
    let mut bad_run = 0usize;

    for i in 0..n_radials {
        let Some(reference) = &prev else {
            prev = Some(out[i].clone());
            continue;
        };
        match radial_shift_verdict(&out[i], reference, interval, nyquist) {
            Some(k) => {
                apply_shift_and_flag_suspects(
                    &mut out,
                    i,
                    reference,
                    k,
                    interval,
                    nyquist,
                    &mut suspects,
                );
                let new_reference = out[i].clone();
                prev = Some(new_reference.clone());
                bad_run = 0;
                reinsert_pending(
                    &mut pending,
                    &mut out,
                    new_reference,
                    interval,
                    nyquist,
                    &mut suspects,
                );
            }
            None => {
                pending.push(i);
                bad_run += 1;
                if bad_run >= MAX_BAD_RADIALS {
                    for pi in pending.drain(..) {
                        for (g, v) in out[pi].iter().enumerate() {
                            if v.is_some() {
                                suspects.push((pi, g));
                            }
                        }
                    }
                    prev = None;
                    bad_run = 0;
                }
            }
        }
    }
    // Anything still pending at the end never found a trustworthy anchor.
    for pi in pending {
        for (g, v) in out[pi].iter().enumerate() {
            if v.is_some() {
                suspects.push((pi, g));
            }
        }
    }

    if !suspects.is_empty() {
        let n_gates = out.iter().map(|r| r.len()).max().unwrap_or(0);
        let fit = best_vad_ring(&out, n_gates).and_then(|ring| fit_vad(azimuths_deg, &ring));
        for (i, g) in suspects {
            out[i][g] = fit.as_ref().map(|f| f.estimate(azimuths_deg[i]));
        }
    }

    out
}

/// Minimum gates in common with the reference radial before a verdict is
/// trusted at all — too little overlap and a "majority" is just noise.
const MIN_OVERLAP_GATES: usize = 5;
/// Fraction of overlapping gates that must agree on a shift (including the
/// "no shift needed" case) before that shift is applied to the whole radial.
const MAJORITY_FRACTION: f32 = 0.6;
/// Search range for the whole-radial interval shift. Real volumes fold at
/// most a couple of intervals; wider search just invites false matches.
const SHIFT_SEARCH: std::ops::RangeInclusive<i32> = -2..=2;

/// Decide whether radial `cur` needs a whole-interval shift to line up with
/// `reference` (the previous, already-corrected radial), by majority vote
/// across their overlapping gates. Returns `None` when there isn't enough
/// overlap or the vote doesn't clear [`MAJORITY_FRACTION`] for any shift —
/// an inconclusive radial, not a confirmed one.
fn radial_shift_verdict(
    cur: &[Option<f32>],
    reference: &[Option<f32>],
    interval: f32,
    nyquist: f32,
) -> Option<i32> {
    let overlap: Vec<(f32, f32)> = cur
        .iter()
        .zip(reference)
        .filter_map(|(c, r)| match (c, r) {
            (Some(c), Some(r)) => Some((*c, *r)),
            _ => None,
        })
        .collect();
    if overlap.len() < MIN_OVERLAP_GATES {
        return None;
    }
    let (mut best_k, mut best_votes) = (0i32, -1i32);
    for k in SHIFT_SEARCH {
        let votes = overlap
            .iter()
            .filter(|(c, r)| (*c + k as f32 * interval - *r).abs() <= nyquist)
            .count() as i32;
        // Ties prefer the smaller |k| — don't shift unless the vote clearly
        // wants to.
        if votes > best_votes || (votes == best_votes && k.abs() < best_k.abs()) {
            best_k = k;
            best_votes = votes;
        }
    }
    if best_votes as f32 / overlap.len() as f32 >= MAJORITY_FRACTION {
        Some(best_k)
    } else {
        None
    }
}

/// Apply a confirmed whole-radial shift and flag any gate that still
/// doesn't line up with the reference afterward as an isolated suspect —
/// this radial's aliasing is confirmed, so a leftover outlier is residue.
///
/// A uniform additive shift doesn't change which gates are genuine local
/// shear (couplets) versus noise — the same couplet-preservation reasoning
/// that skips suspect-flagging entirely on the `k == 0` path still applies
/// here. So a gate disagreeing with the reference is only flagged when it
/// ALSO breaks gate-to-gate (Stage B) continuity against its own immediate
/// neighbors *within this radial*: a coherent multi-gate shear feature stays
/// smooth locally even where it splits from the reference, while a genuine
/// residual outlier is discontinuous on both sides.
fn apply_shift_and_flag_suspects(
    out: &mut [Vec<Option<f32>>],
    i: usize,
    reference: &[Option<f32>],
    k: i32,
    interval: f32,
    nyquist: f32,
    suspects: &mut Vec<(usize, usize)>,
) {
    if k == 0 {
        // No shift needed: this is the couplet-preserving branch. A gate
        // that still disagrees with the reference here is real signal (a
        // couplet spans a handful of gates, not the whole radial), not
        // aliasing — leave it alone, no suspect flagging.
        return;
    }
    for x in out[i].iter_mut().flatten() {
        *x += k as f32 * interval;
    }
    // This radial's aliasing is now confirmed by majority vote; a gate that
    // *still* doesn't line up with the reference AND breaks continuity with
    // its own neighbors is residue, not signal.
    for g in 0..out[i].len() {
        let (Some(c), Some(r)) = (out[i][g], reference[g]) else {
            continue;
        };
        if (c - r).abs() > nyquist && local_discontinuity(&out[i], g, c, nyquist) {
            suspects.push((i, g));
        }
    }
}

/// True when gate `g` (value `val`) is discontinuous with every immediate,
/// present neighbor within the same radial — i.e. an isolated jump rather
/// than part of a locally-smooth (couplet) feature. With no neighbor to
/// check against at all, falls back to flagging (no local evidence either
/// way, so preserve the prior conservative behavior).
fn local_discontinuity(radial: &[Option<f32>], g: usize, val: f32, nyquist: f32) -> bool {
    let mut present_neighbors = [g.checked_sub(1), Some(g + 1)]
        .into_iter()
        .flatten()
        .filter_map(|n| radial.get(n).copied().flatten());
    let mut any = false;
    let all_disagree = present_neighbors.all(|n| {
        any = true;
        (val - n).abs() > nyquist
    });
    !any || all_disagree
}

/// Retry every still-pending radial against a newly confirmed reference;
/// any that now get a clear verdict are resolved (shifted, suspects
/// flagged) and dropped from the pending queue.
fn reinsert_pending(
    pending: &mut Vec<usize>,
    out: &mut [Vec<Option<f32>>],
    reference: Vec<Option<f32>>,
    interval: f32,
    nyquist: f32,
    suspects: &mut Vec<(usize, usize)>,
) {
    pending.retain(
        |&pi| match radial_shift_verdict(&out[pi], &reference, interval, nyquist) {
            Some(k) => {
                apply_shift_and_flag_suspects(out, pi, &reference, k, interval, nyquist, suspects);
                false
            }
            None => true,
        },
    );
}

/// Single-ring VAD (velocity-azimuth display) environmental-wind fit:
/// `v(az) ≈ mean + a·sin(az) + b·cos(az)` (FMH-11D §3.3.3). This is a
/// fallback estimate for isolated dealias failures, not a full VAD wind
/// profile retrieval.
struct VadFit {
    mean: f32,
    a: f32,
    b: f32,
}

impl VadFit {
    fn estimate(&self, az_deg: f32) -> f32 {
        let rad = (az_deg as f64).to_radians();
        self.mean + self.a * rad.sin() as f32 + self.b * rad.cos() as f32
    }
}

/// Range-gate column with the most valid samples — used as the VAD ring.
///
/// ponytail: one fixed ring rather than a range-weighted composite across
/// several rings. This is a fallback of last resort for isolated gates, not
/// the primary product; revisit if isolated-gate replacement quality
/// becomes a field complaint.
fn best_vad_ring(gates: &[Vec<Option<f32>>], n_gates: usize) -> Option<Vec<Option<f32>>> {
    let valid_count = |g: usize| {
        gates
            .iter()
            .filter(|r| matches!(r.get(g), Some(Some(_))))
            .count()
    };
    let best = (0..n_gates).max_by_key(|&g| valid_count(g))?;
    if valid_count(best) < MIN_VAD_SAMPLES {
        return None;
    }
    Some(
        gates
            .iter()
            .map(|r| r.get(best).copied().flatten())
            .collect(),
    )
}

/// Least-squares fit of `v(az) = mean + a*sin(az) + b*cos(az)` via the 3x3
/// normal equations, solved directly with Cramer's rule (fixed 3-unknown
/// system — a linear-algebra dependency would be overkill for one solve).
fn fit_vad(azimuths_deg: &[f32], ring: &[Option<f32>]) -> Option<VadFit> {
    let (mut n, mut sum_v, mut sum_s, mut sum_c) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    let (mut sum_vs, mut sum_vc, mut sum_ss, mut sum_cc, mut sum_sc) =
        (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for (&az, &val) in azimuths_deg.iter().zip(ring) {
        let Some(v) = val else { continue };
        let (s, c) = (az as f64).to_radians().sin_cos();
        n += 1.0;
        sum_v += v as f64;
        sum_s += s;
        sum_c += c;
        sum_vs += v as f64 * s;
        sum_vc += v as f64 * c;
        sum_ss += s * s;
        sum_cc += c * c;
        sum_sc += s * c;
    }
    if n < MIN_VAD_SAMPLES as f64 {
        return None;
    }

    // | n      sum_s  sum_c  |   |mean|   |sum_v |
    // | sum_s  sum_ss sum_sc | * |a   | = |sum_vs|
    // | sum_c  sum_sc sum_cc |   |b   |   |sum_vc|
    let det = n * (sum_ss * sum_cc - sum_sc * sum_sc) - sum_s * (sum_s * sum_cc - sum_sc * sum_c)
        + sum_c * (sum_s * sum_sc - sum_ss * sum_c);
    if det.abs() < 1e-6 {
        return None; // degenerate ring (e.g. azimuths don't span the circle)
    }
    let det_mean = sum_v * (sum_ss * sum_cc - sum_sc * sum_sc)
        - sum_s * (sum_vs * sum_cc - sum_sc * sum_vc)
        + sum_c * (sum_vs * sum_sc - sum_ss * sum_vc);
    let det_a = n * (sum_vs * sum_cc - sum_sc * sum_vc) - sum_v * (sum_s * sum_cc - sum_sc * sum_c)
        + sum_c * (sum_s * sum_vc - sum_vs * sum_c);
    let det_b = n * (sum_ss * sum_vc - sum_vs * sum_sc) - sum_s * (sum_s * sum_vc - sum_vs * sum_c)
        + sum_v * (sum_s * sum_sc - sum_ss * sum_c);

    Some(VadFit {
        mean: (det_mean / det) as f32,
        a: (det_a / det) as f32,
        b: (det_b / det) as f32,
    })
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

    // ---- Stage C: azimuthal continuity ------------------------------------

    #[test]
    fn dealias_sweep_identity_when_nyquist_unknown() {
        let gates = vec![vec![Some(1.0), None], vec![Some(2.0), Some(3.0)]];
        let azimuths = vec![0.0, 10.0];
        assert_eq!(dealias_sweep(&azimuths, &gates, 0.0), gates);
    }

    #[test]
    fn dealias_sweep_corrects_whole_radial_seeded_in_wrong_interval() {
        // Radial 1 is internally consistent (every gate-to-gate step is
        // well under Nyquist, so Stage B leaves it alone) but its whole
        // seed sits one full interval below radial 0's — the same feature
        // seen from the next azimuth. The azimuthal pass should recognize
        // the whole-radial offset and correct it.
        let nyquist = 26.4;
        let interval = 2.0 * nyquist;
        let reference: Vec<Option<f32>> = [10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0]
            .into_iter()
            .map(Some)
            .collect();
        let offset: Vec<Option<f32>> = reference
            .iter()
            .map(|v| Some(v.unwrap() - interval))
            .collect();
        let azimuths = vec![0.0, 1.0];
        let out = dealias_sweep(&azimuths, &[reference.clone(), offset], nyquist);
        for (a, b) in out[1].iter().zip(&reference) {
            assert!((a.unwrap() - b.unwrap()).abs() < 1e-3, "got {:?}", out[1]);
        }
    }

    #[test]
    fn dealias_sweep_preserves_couplet_gate_against_azimuthal_majority() {
        // Radial 1 matches radial 0 (environmental 15 m/s) at every gate
        // except gate 4, where it reads -5.0 against radial 0's +25.0 — a
        // localized couplet pair, not a mis-seeded radial. The 30 m/s
        // azimuthal jump at that one gate is well over Nyquist (a naive
        // per-gate check would "fix" it), but the other 7 gates vote for no
        // shift, and majority wins: gate 4 must survive untouched.
        let nyquist = 26.4;
        let mut reference = vec![Some(15.0); 8];
        reference[4] = Some(25.0);
        let mut cur = vec![Some(15.0); 8];
        cur[4] = Some(-5.0);
        let azimuths = vec![0.0, 1.0];
        let out = dealias_sweep(&azimuths, &[reference, cur], nyquist);
        assert_eq!(out[1][4], Some(-5.0), "couplet gate must not be shifted");
    }

    #[test]
    fn dealias_sweep_preserves_couplet_in_mis_seeded_radial() {
        // Radial 1's true field matches radial 0 everywhere except a
        // couplet at gate 4 (25.0 -> -5.0) — but the whole radial was
        // seeded one interval high, so every gate (including the couplet)
        // arrives shifted by +interval. The majority vote correctly detects
        // k=-1 and recovers gate 4 to -5.0 via the uniform shift. Before the
        // fix, the post-shift suspect check then re-flagged gate 4 (it
        // still disagrees with the reference by 30 m/s > Nyquist) and
        // nulled/VAD-replaced it, flattening the couplet. The fix requires
        // the suspect to ALSO break local (Stage B) continuity with its own
        // neighbors — gate 4's shifted value (-5.0) is only 20 m/s from its
        // neighbors (15.0), well under Nyquist, so it must survive.
        let nyquist = 26.4;
        let interval = 2.0 * nyquist;
        let mut reference = vec![Some(15.0); 8];
        reference[4] = Some(25.0);
        let mut mis_seeded = vec![Some(15.0 + interval); 8];
        mis_seeded[4] = Some(-5.0 + interval);
        let azimuths = vec![0.0, 1.0];
        let out = dealias_sweep(&azimuths, &[reference, mis_seeded], nyquist);
        assert!(
            (out[1][4].unwrap() - (-5.0)).abs() < 1e-3,
            "couplet gate must survive the whole-radial shift, not be nulled: got {:?}",
            out[1]
        );
        for (g, v) in out[1].iter().enumerate().filter(|&(g, _)| g != 4) {
            assert!(
                (v.unwrap() - 15.0).abs() < 1e-3,
                "gate {g} should recover cleanly: {:?}",
                out[1]
            );
        }
    }

    #[test]
    fn dealias_sweep_rejects_persistent_ambiguous_run_and_stops_propagation() {
        // Five consecutive radials with too little azimuthal overlap to
        // reach a verdict (sparse returns, e.g. a low-SNR patch) exceed
        // MAX_BAD_RADIALS: they're rejected — no reliable ring to fall back
        // on here, so nulled rather than guessed — and the reference is
        // dropped so the next healthy radial isn't compared against a
        // stale one.
        let nyquist = 26.4;
        let good = vec![Some(15.0); 8];
        let sparse = {
            let mut g = vec![None; 8];
            g[0] = Some(15.0);
            g[1] = Some(15.0);
            g
        };
        let mut radials = vec![good.clone()];
        for _ in 0..MAX_BAD_RADIALS {
            radials.push(sparse.clone());
        }
        radials.push(good.clone());
        let azimuths: Vec<f32> = (0..radials.len() as i32).map(|i| i as f32).collect();
        let out = dealias_sweep(&azimuths, &radials, nyquist);

        for (r, radial) in out.iter().enumerate().take(MAX_BAD_RADIALS + 1).skip(1) {
            assert_eq!(
                radial[0], None,
                "ambiguous radial {r} gate 0 should be rejected"
            );
            assert_eq!(
                radial[1], None,
                "ambiguous radial {r} gate 1 should be rejected"
            );
        }
        // The healthy radial after the bad run is untouched by the stale
        // reference — propagation stopped.
        assert_eq!(out[MAX_BAD_RADIALS + 1], good);
    }

    #[test]
    fn radial_shift_verdict_needs_overlap() {
        let cur = vec![Some(15.0), Some(15.0), None, None, None, None, None, None];
        let reference = vec![Some(15.0); 8];
        assert_eq!(radial_shift_verdict(&cur, &reference, 52.8, 26.4), None);
    }

    #[test]
    fn radial_shift_verdict_needs_a_clear_majority() {
        // Exactly half the gates support "no shift", half support a
        // one-interval shift — neither clears MAJORITY_FRACTION.
        let reference = vec![Some(15.0); 8];
        let mut cur = vec![Some(15.0); 8];
        for v in cur.iter_mut().skip(4) {
            *v = Some(-25.0); // matches reference only after a +1 interval shift
        }
        assert_eq!(radial_shift_verdict(&cur, &reference, 52.8, 26.4), None);
    }

    #[test]
    fn fit_vad_recovers_harmonic_wind() {
        // v(az) = 2.0 + 5.0*sin(az) + 3.0*cos(az), sampled on a full ring —
        // a noiseless case the least-squares fit should recover exactly.
        let azimuths: Vec<f32> = (0..36).map(|i| i as f32 * 10.0).collect();
        let ring: Vec<Option<f32>> = azimuths
            .iter()
            .map(|&az| {
                let rad = (az as f64).to_radians();
                Some((2.0 + 5.0 * rad.sin() + 3.0 * rad.cos()) as f32)
            })
            .collect();
        let fit = fit_vad(&azimuths, &ring).expect("full ring should fit");
        assert!((fit.mean - 2.0).abs() < 1e-2, "mean={}", fit.mean);
        assert!((fit.a - 5.0).abs() < 1e-2, "a={}", fit.a);
        assert!((fit.b - 3.0).abs() < 1e-2, "b={}", fit.b);
        let est = fit.estimate(90.0);
        assert!((est - 7.0).abs() < 1e-2, "estimate(90)={est}");
    }
}
