//! rustywx's own thin scan model, decoupling rendering from nexrad-model.
//! `Option<f32>` gates: `None` = below threshold / range folded (drawn
//! transparent). Sweeps are sorted by elevation and deduplicated so split
//! cuts at the same elevation appear once in the tilt selector.

use chrono::{DateTime, Utc};
use nexrad_model::data::{MomentValue, Scan, Sweep};

/// Radar products the app can display.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Product {
    Reflectivity,
    Velocity,
}

impl Product {
    pub fn label(self) -> &'static str {
        match self {
            Product::Reflectivity => "Reflectivity",
            Product::Velocity => "Velocity",
        }
    }
}

/// One ray of gate values. `None` gates are below threshold or range folded.
pub struct RadialData {
    pub azimuth_deg: f32,
    pub gates: Vec<Option<f32>>,
}

/// One full rotation at a single elevation angle, for a single product.
pub struct SweepData {
    pub elevation_deg: f32,
    pub radials: Vec<RadialData>,
}

/// A decoded volume scan, split per product, sweeps sorted by elevation.
pub struct ScanData {
    pub timestamp: DateTime<Utc>,
    pub reflectivity: Vec<SweepData>,
    pub velocity: Vec<SweepData>,
}

impl ScanData {
    pub fn sweeps(&self, product: Product) -> &[SweepData] {
        match product {
            Product::Reflectivity => &self.reflectivity,
            Product::Velocity => &self.velocity,
        }
    }

    pub fn from_nexrad(scan: &Scan, timestamp: DateTime<Utc>) -> Self {
        Self::from_sweeps(scan.sweeps(), timestamp)
    }

    pub fn from_sweeps(sweeps: &[Sweep], timestamp: DateTime<Utc>) -> Self {
        let mut reflectivity = Vec::new();
        let mut velocity = Vec::new();

        for sweep in sweeps {
            for (product, out) in [
                (Product::Reflectivity, &mut reflectivity),
                (Product::Velocity, &mut velocity),
            ] {
                let radials: Vec<RadialData> = sweep
                    .radials()
                    .iter()
                    .filter_map(|radial| {
                        let moment = match product {
                            Product::Reflectivity => radial.reflectivity(),
                            Product::Velocity => radial.velocity(),
                        }?;
                        Some(RadialData {
                            azimuth_deg: radial.azimuth_angle_degrees(),
                            gates: moment
                                .values()
                                .into_iter()
                                .map(|value| match value {
                                    MomentValue::Value(v) => Some(v),
                                    MomentValue::BelowThreshold | MomentValue::RangeFolded => None,
                                })
                                .collect(),
                        })
                    })
                    .collect();

                if !radials.is_empty() {
                    let elevation_deg = sweep
                        .radials()
                        .first()
                        .map(|r| r.elevation_angle_degrees())
                        .unwrap_or(0.0);
                    out.push(SweepData { elevation_deg, radials });
                }
            }
        }

        sort_and_dedup(&mut reflectivity);
        sort_and_dedup(&mut velocity);

        ScanData { timestamp, reflectivity, velocity }
    }
}

/// Sort sweeps by elevation; split cuts produce near-duplicate elevations
/// (e.g. 0.48 and 0.52 deg) — keep only the first of each cluster.
fn sort_and_dedup(sweeps: &mut Vec<SweepData>) {
    sweeps.sort_by(|a, b| a.elevation_deg.total_cmp(&b.elevation_deg));
    sweeps.dedup_by(|current, previous| (current.elevation_deg - previous.elevation_deg).abs() < 0.2);
}

#[cfg(test)]
mod tests {
    use super::{Product, ScanData};
    use chrono::Utc;
    use nexrad_model::data::{MomentData, Radial, RadialStatus, Sweep};

    /// A REF-encoded moment: value = (raw - 66.0) / 2.0 dBZ.
    fn ref_moment(raws: Vec<u8>) -> MomentData {
        let gate_count = raws.len() as u16;
        MomentData::from_fixed_point(gate_count, 2125, 250, 8, 2.0, 66.0, raws)
    }

    fn vel_moment(raws: Vec<u8>) -> MomentData {
        let gate_count = raws.len() as u16;
        MomentData::from_fixed_point(gate_count, 2125, 250, 8, 2.0, 129.0, raws)
    }

    fn radial(az: f32, elev_num: u8, elev_deg: f32, refl: Option<MomentData>, vel: Option<MomentData>) -> Radial {
        Radial::new(
            0, 1, az, 0.5, RadialStatus::IntermediateRadialData, elev_num, elev_deg,
            refl, vel, None, None, None, None, None,
        )
    }

    fn synthetic_sweeps() -> Vec<Sweep> {
        // Sweep 1 (0.5 deg): reflectivity only (split-cut CS).
        let s1 = Sweep::new(1, vec![
            radial(0.0, 1, 0.5, Some(ref_moment(vec![0, 130, 190])), None),
            radial(0.5, 1, 0.5, Some(ref_moment(vec![0, 130, 190])), None),
        ]);
        // Sweep 2 (0.5 deg): velocity only (split-cut CD).
        let s2 = Sweep::new(2, vec![
            radial(0.0, 2, 0.5, None, Some(vel_moment(vec![0, 1, 65]))),
        ]);
        // Sweep 3 (1.45 deg): both moments.
        let s3 = Sweep::new(3, vec![
            radial(0.0, 3, 1.45, Some(ref_moment(vec![130])), Some(vel_moment(vec![193]))),
        ]);
        vec![s1, s2, s3]
    }

    #[test]
    fn converts_moment_values_to_gates() {
        let scan_data = ScanData::from_sweeps(&synthetic_sweeps(), Utc::now());
        let sweep = &scan_data.reflectivity[0];
        // raw 0 -> BelowThreshold -> None; raw 130 -> 32 dBZ; raw 190 -> 62 dBZ.
        assert_eq!(sweep.radials[0].gates, vec![None, Some(32.0), Some(62.0)]);
        assert_eq!(sweep.radials[0].azimuth_deg, 0.0);
    }

    #[test]
    fn range_folded_becomes_none() {
        let scan_data = ScanData::from_sweeps(&synthetic_sweeps(), Utc::now());
        // Velocity sweep at 0.5 deg: raws [0, 1, 65] -> [None, None(RF), Some(-32.0)].
        let sweep = &scan_data.velocity[0];
        assert_eq!(sweep.radials[0].gates, vec![None, None, Some(-32.0)]);
    }

    #[test]
    fn products_split_and_dedup_by_elevation() {
        let scan_data = ScanData::from_sweeps(&synthetic_sweeps(), Utc::now());
        // Reflectivity: 0.5 deg (from CS cut) and 1.45 deg. The CD cut has no
        // reflectivity so nothing to dedup here, but elevations are ascending.
        let elevations: Vec<f32> = scan_data.reflectivity.iter().map(|s| s.elevation_deg).collect();
        assert_eq!(elevations, vec![0.5, 1.45]);
        // Velocity: 0.5 and 1.45.
        let elevations: Vec<f32> = scan_data.velocity.iter().map(|s| s.elevation_deg).collect();
        assert_eq!(elevations, vec![0.5, 1.45]);
    }

    #[test]
    fn dedups_near_identical_elevations() {
        // Two reflectivity sweeps both at ~0.5 deg -> keep only the first.
        let s1 = Sweep::new(1, vec![radial(0.0, 1, 0.48, Some(ref_moment(vec![130])), None)]);
        let s2 = Sweep::new(2, vec![radial(0.0, 2, 0.52, Some(ref_moment(vec![190])), None)]);
        let scan_data = ScanData::from_sweeps(&[s1, s2], Utc::now());
        assert_eq!(scan_data.reflectivity.len(), 1);
        assert_eq!(scan_data.reflectivity[0].radials[0].gates, vec![Some(32.0)]);
    }

    #[test]
    fn sweeps_accessor_selects_product() {
        let scan_data = ScanData::from_sweeps(&synthetic_sweeps(), Utc::now());
        assert_eq!(scan_data.sweeps(Product::Reflectivity).len(), 2);
        assert_eq!(scan_data.sweeps(Product::Velocity).len(), 2);
    }
}
