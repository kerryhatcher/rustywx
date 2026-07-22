//! Thin scan model — no egui dependency. Copied from rustywx.
//! `Option<f32>` gates: `None` = below threshold or range folded; the
//! parallel `RadialData::range_folded` flag distinguishes the two (range
//! folded is drawn distinctly, below-threshold stays transparent). Sweeps
//! are sorted by elevation and deduplicated so split cuts at the same
//! elevation appear once in the tilt selector.

use chrono::{DateTime, Utc};
use nexrad_model::data::{MomentValue, Scan, Sweep, VCPNumber};
use serde::{Deserialize, Serialize};

/// NEXRAD WSR-88D wavelength in meters. Used for Nyquist velocity calculation.
/// Source: FMH-11 Part A, NEXRAD Technical Specifications.
/// ponytail: Nyquist computation currently shows "—" since PRT unavailable; this const for future use.
#[allow(dead_code)]
const NEXRAD_WAVELENGTH_M: f32 = 0.1071;

/// Radar products the app can display.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum Product {
    Reflectivity,
    Velocity,
    SpectrumWidth,
    DifferentialReflectivity,
    CorrelationCoefficient,
    DifferentialPhase,
}

impl Product {
    pub fn label(self) -> &'static str {
        match self {
            Product::Reflectivity => "Reflectivity",
            Product::Velocity => "Velocity",
            Product::SpectrumWidth => "Spectrum Width",
            Product::DifferentialReflectivity => "Differential Reflectivity",
            Product::CorrelationCoefficient => "Correlation Coefficient",
            Product::DifferentialPhase => "Differential Phase",
        }
    }

    /// Unit label for the status bar / legend.
    pub fn units(self) -> &'static str {
        match self {
            Product::Reflectivity => "dBZ",
            Product::Velocity | Product::SpectrumWidth => "m/s",
            Product::DifferentialReflectivity => "dB",
            Product::CorrelationCoefficient => "",
            Product::DifferentialPhase => "°",
        }
    }
}

/// Map a VCP number to a short mode label.
pub fn vcp_mode_label(vcp: VCPNumber) -> &'static str {
    match vcp {
        VCPNumber::Precipitation12 => "Precip",
        VCPNumber::PrecipitationSz2_212 => "Precip",
        VCPNumber::GeneralSurveillance215 => "Precip",
        VCPNumber::PrecipitationMpda112 => "Precip",
        VCPNumber::ClearAirLongPulse31 => "Clear Air",
        VCPNumber::ClearAirShortPulse32 => "Clear Air",
        VCPNumber::ClearAir35 => "Clear Air",
        VCPNumber::Unknown(_) => "Unknown",
    }
}

/// Format the Nyquist velocity for display.
/// Returns a string like "Nyquist ±26.4 m/s" or "Nyquist —" if unavailable.
/// ponytail: PRT not available in decoded sweep data, so showing "—" as fallback.
pub fn format_nyquist_velocity() -> &'static str {
    "Nyquist —"
}

/// One ray of gate values. `None` gates are below threshold or range folded;
/// `range_folded[i]` distinguishes range-folded (`true`) from below-threshold (`false`).
#[derive(Clone, Serialize, Deserialize)]
pub struct RadialData {
    pub azimuth_deg: f32,
    pub gates: Vec<Option<f32>>,
    /// Parallel to `gates`; `true` = range-folded (drawn distinctly, value is None).
    /// May be shorter/empty than `gates` when unknown — treat missing as false.
    #[serde(default)]
    pub range_folded: Vec<bool>,
}

/// One full rotation at a single elevation angle, for a single product.
#[derive(Clone, Serialize, Deserialize)]
pub struct SweepData {
    pub elevation_deg: f32,
    pub radials: Vec<RadialData>,
}

/// Build a synthetic sweep for tests/benchmarks: `n_radials` evenly spaced
/// azimuths, each with `gates_per_radial` gates of varying dBZ-like values
/// (some `None` to exercise the missing-gate paths).
pub fn synthetic_sweep(n_radials: usize, gates_per_radial: usize) -> SweepData {
    let mut radials = Vec::with_capacity(n_radials);
    for i in 0..n_radials {
        let azimuth = i as f32 * 360.0 / n_radials as f32;
        let mut gates = Vec::with_capacity(gates_per_radial);
        for g in 0..gates_per_radial {
            let angle = (azimuth + g as f32 * 0.5).to_radians();
            let value = 30.0 + 20.0 * angle.sin();
            gates.push(if value > 5.0 { Some(value) } else { None });
        }
        radials.push(RadialData {
            azimuth_deg: azimuth,
            gates,
            range_folded: vec![],
        });
    }
    SweepData {
        elevation_deg: 0.5,
        radials,
    }
}

/// A decoded volume scan, split per product, sweeps sorted by elevation.
#[derive(Serialize, Deserialize)]
pub struct ScanData {
    pub timestamp: DateTime<Utc>,
    pub reflectivity: Vec<SweepData>,
    pub velocity: Vec<SweepData>,
    pub spectrum_width: Vec<SweepData>,
    #[serde(default)]
    pub differential_reflectivity: Vec<SweepData>,
    #[serde(default)]
    pub correlation_coefficient: Vec<SweepData>,
    #[serde(default)]
    pub differential_phase: Vec<SweepData>,
    pub vcp_number: u16,
}

impl ScanData {
    pub fn sweeps(&self, product: Product) -> &[SweepData] {
        match product {
            Product::Reflectivity => &self.reflectivity,
            Product::Velocity => &self.velocity,
            Product::SpectrumWidth => &self.spectrum_width,
            Product::DifferentialReflectivity => &self.differential_reflectivity,
            Product::CorrelationCoefficient => &self.correlation_coefficient,
            Product::DifferentialPhase => &self.differential_phase,
        }
    }

    pub fn from_nexrad(scan: &Scan, timestamp: DateTime<Utc>) -> Self {
        let vcp_number = scan.coverage_pattern_number().number();
        Self::from_sweeps(scan.sweeps(), timestamp, vcp_number)
    }

    pub fn from_sweeps(sweeps: &[Sweep], timestamp: DateTime<Utc>, vcp_number: u16) -> Self {
        let mut reflectivity = Vec::new();
        let mut velocity = Vec::new();
        let mut spectrum_width = Vec::new();
        let mut differential_reflectivity = Vec::new();
        let mut correlation_coefficient = Vec::new();
        let mut differential_phase = Vec::new();

        for sweep in sweeps {
            for (product, out) in [
                (Product::Reflectivity, &mut reflectivity),
                (Product::Velocity, &mut velocity),
                (Product::SpectrumWidth, &mut spectrum_width),
                (
                    Product::DifferentialReflectivity,
                    &mut differential_reflectivity,
                ),
                (
                    Product::CorrelationCoefficient,
                    &mut correlation_coefficient,
                ),
                (Product::DifferentialPhase, &mut differential_phase),
            ] {
                let radials: Vec<RadialData> = sweep
                    .radials()
                    .iter()
                    .filter_map(|radial| {
                        let moment = match product {
                            Product::Reflectivity => radial.reflectivity(),
                            Product::Velocity => radial.velocity(),
                            Product::SpectrumWidth => radial.spectrum_width(),
                            Product::DifferentialReflectivity => radial.differential_reflectivity(),
                            Product::CorrelationCoefficient => radial.correlation_coefficient(),
                            Product::DifferentialPhase => radial.differential_phase(),
                        }?;
                        let values = moment.values();
                        let mut gates = Vec::with_capacity(values.len());
                        let mut range_folded = Vec::with_capacity(values.len());
                        for value in values {
                            match value {
                                MomentValue::Value(v) => {
                                    gates.push(Some(v));
                                    range_folded.push(false);
                                }
                                MomentValue::BelowThreshold => {
                                    gates.push(None);
                                    range_folded.push(false);
                                }
                                MomentValue::RangeFolded => {
                                    gates.push(None);
                                    range_folded.push(true);
                                }
                            }
                        }
                        Some(RadialData {
                            azimuth_deg: radial.azimuth_angle_degrees(),
                            gates,
                            range_folded,
                        })
                    })
                    .collect();

                if !radials.is_empty() {
                    let elevation_deg = sweep
                        .radials()
                        .first()
                        .map(|r| r.elevation_angle_degrees())
                        .unwrap_or(0.0);
                    out.push(SweepData {
                        elevation_deg,
                        radials,
                    });
                }
            }
        }

        sort_and_dedup(&mut reflectivity);
        sort_and_dedup(&mut velocity);
        sort_and_dedup(&mut spectrum_width);
        sort_and_dedup(&mut differential_reflectivity);
        sort_and_dedup(&mut correlation_coefficient);
        sort_and_dedup(&mut differential_phase);

        ScanData {
            timestamp,
            reflectivity,
            velocity,
            spectrum_width,
            differential_reflectivity,
            correlation_coefficient,
            differential_phase,
            vcp_number,
        }
    }
}

/// Sort sweeps by elevation; split cuts produce near-duplicate elevations
/// (e.g. 0.48 and 0.52 deg) — keep only the first of each cluster.
fn sort_and_dedup(sweeps: &mut Vec<SweepData>) {
    sweeps.sort_by(|a, b| a.elevation_deg.total_cmp(&b.elevation_deg));
    sweeps
        .dedup_by(|current, previous| (current.elevation_deg - previous.elevation_deg).abs() < 0.2);
}

#[cfg(test)]
mod tests {
    use super::{Product, ScanData, format_nyquist_velocity, vcp_mode_label};
    use chrono::Utc;
    use nexrad_model::data::{MomentData, Radial, RadialStatus, Sweep, VCPNumber};

    /// A REF-encoded moment: value = (raw - 66.0) / 2.0 dBZ.
    fn ref_moment(raws: Vec<u8>) -> MomentData {
        let gate_count = raws.len() as u16;
        MomentData::from_fixed_point(gate_count, 2125, 250, 8, 2.0, 66.0, raws)
    }

    fn vel_moment(raws: Vec<u8>) -> MomentData {
        let gate_count = raws.len() as u16;
        MomentData::from_fixed_point(gate_count, 2125, 250, 8, 2.0, 129.0, raws)
    }

    /// A CC-encoded moment (RhoHV, unitless 0..1): value = (raw - 2) / 300.
    fn cc_moment(raws: Vec<u8>) -> MomentData {
        let gate_count = raws.len() as u16;
        MomentData::from_fixed_point(gate_count, 2125, 250, 8, 300.0, 2.0, raws)
    }

    #[allow(clippy::too_many_arguments)]
    fn radial(
        az: f32,
        elev_num: u8,
        elev_deg: f32,
        refl: Option<MomentData>,
        vel: Option<MomentData>,
        zdr: Option<MomentData>,
        cc: Option<MomentData>,
        phi: Option<MomentData>,
    ) -> Radial {
        Radial::new(
            0,
            1,
            az,
            0.5,
            RadialStatus::IntermediateRadialData,
            elev_num,
            elev_deg,
            refl,
            vel,
            None, // spectrum_width
            zdr,  // differential_reflectivity
            phi,  // differential_phase  ← NOTE order: phi BEFORE cc
            cc,   // correlation_coefficient
            None, // clutter_filter_power
        )
    }

    fn synthetic_sweeps() -> Vec<Sweep> {
        // Sweep 1 (0.5 deg): reflectivity only (split-cut CS).
        let s1 = Sweep::new(
            1,
            vec![
                radial(
                    0.0,
                    1,
                    0.5,
                    Some(ref_moment(vec![0, 130, 190])),
                    None,
                    None,
                    None,
                    None,
                ),
                radial(
                    0.5,
                    1,
                    0.5,
                    Some(ref_moment(vec![0, 130, 190])),
                    None,
                    None,
                    None,
                    None,
                ),
            ],
        );
        // Sweep 2 (0.5 deg): velocity only (split-cut CD).
        let s2 = Sweep::new(
            2,
            vec![radial(
                0.0,
                2,
                0.5,
                None,
                Some(vel_moment(vec![0, 1, 65])),
                None,
                None,
                None,
            )],
        );
        // Sweep 3 (1.45 deg): both moments.
        let s3 = Sweep::new(
            3,
            vec![radial(
                0.0,
                3,
                1.45,
                Some(ref_moment(vec![130])),
                Some(vel_moment(vec![193])),
                None,
                None,
                None,
            )],
        );
        vec![s1, s2, s3]
    }

    #[test]
    fn converts_moment_values_to_gates() {
        let scan_data = ScanData::from_sweeps(&synthetic_sweeps(), Utc::now(), 12);
        let sweep = &scan_data.reflectivity[0];
        // raw 0 -> BelowThreshold -> None; raw 130 -> 32 dBZ; raw 190 -> 62 dBZ.
        assert_eq!(sweep.radials[0].gates, vec![None, Some(32.0), Some(62.0)]);
        assert_eq!(sweep.radials[0].azimuth_deg, 0.0);
    }

    #[test]
    fn range_folded_flagged() {
        let scan_data = ScanData::from_sweeps(&synthetic_sweeps(), Utc::now(), 12);
        // Velocity sweep at 0.5 deg: raws [0, 1, 65] -> [None, None(RF), Some(-32.0)].
        let sweep = &scan_data.velocity[0];
        assert_eq!(sweep.radials[0].gates, vec![None, None, Some(-32.0)]);
        assert_eq!(sweep.radials[0].range_folded, vec![false, true, false]);
    }

    #[test]
    fn products_split_and_dedup_by_elevation() {
        let scan_data = ScanData::from_sweeps(&synthetic_sweeps(), Utc::now(), 12);
        // Reflectivity: 0.5 deg (from CS cut) and 1.45 deg. The CD cut has no
        // reflectivity so nothing to dedup here, but elevations are ascending.
        let elevations: Vec<f32> = scan_data
            .reflectivity
            .iter()
            .map(|s| s.elevation_deg)
            .collect();
        assert_eq!(elevations, vec![0.5, 1.45]);
        // Velocity: 0.5 and 1.45.
        let elevations: Vec<f32> = scan_data.velocity.iter().map(|s| s.elevation_deg).collect();
        assert_eq!(elevations, vec![0.5, 1.45]);
    }

    #[test]
    fn dedups_near_identical_elevations() {
        // Two reflectivity sweeps both at ~0.5 deg -> keep only the first.
        let s1 = Sweep::new(
            1,
            vec![radial(
                0.0,
                1,
                0.48,
                Some(ref_moment(vec![130])),
                None,
                None,
                None,
                None,
            )],
        );
        let s2 = Sweep::new(
            2,
            vec![radial(
                0.0,
                2,
                0.52,
                Some(ref_moment(vec![190])),
                None,
                None,
                None,
                None,
            )],
        );
        let scan_data =
            ScanData::from_sweeps(&[s1, s2], Utc::now(), VCPNumber::Precipitation12.into());
        assert_eq!(scan_data.reflectivity.len(), 1);
        assert_eq!(scan_data.reflectivity[0].radials[0].gates, vec![Some(32.0)]);
    }

    #[test]
    fn sweeps_accessor_selects_product() {
        let scan_data = ScanData::from_sweeps(&synthetic_sweeps(), Utc::now(), 12);
        assert_eq!(scan_data.sweeps(Product::Reflectivity).len(), 2);
        assert_eq!(scan_data.sweeps(Product::Velocity).len(), 2);
        assert_eq!(scan_data.sweeps(Product::CorrelationCoefficient).len(), 0);
    }

    #[test]
    fn dualpol_moments_convert_to_gates() {
        // CC raws: 0 -> BelowThreshold -> None; 242 -> (242-2)/300 = 0.8.
        let s1 = Sweep::new(
            1,
            vec![radial(
                0.0,
                1,
                0.5,
                None,
                None,
                None,
                Some(cc_moment(vec![0, 242])),
                None,
            )],
        );
        let scan_data = ScanData::from_sweeps(&[s1], Utc::now(), 12);
        let sweep = &scan_data.correlation_coefficient[0];
        assert_eq!(sweep.radials[0].gates, vec![None, Some(0.8)]);
    }

    #[test]
    fn vcp_mode_label_maps_known_vcps() {
        assert_eq!(vcp_mode_label(VCPNumber::Precipitation12), "Precip");
        assert_eq!(vcp_mode_label(VCPNumber::PrecipitationSz2_212), "Precip");
        assert_eq!(vcp_mode_label(VCPNumber::GeneralSurveillance215), "Precip");
        assert_eq!(vcp_mode_label(VCPNumber::PrecipitationMpda112), "Precip");
        assert_eq!(vcp_mode_label(VCPNumber::ClearAirLongPulse31), "Clear Air");
        assert_eq!(vcp_mode_label(VCPNumber::ClearAirShortPulse32), "Clear Air");
        assert_eq!(vcp_mode_label(VCPNumber::ClearAir35), "Clear Air");
    }

    #[test]
    fn vcp_mode_label_handles_unknown() {
        assert_eq!(vcp_mode_label(VCPNumber::Unknown(99)), "Unknown");
    }

    #[test]
    fn nyquist_velocity_returns_unavailable() {
        assert_eq!(format_nyquist_velocity(), "Nyquist —");
    }
}
