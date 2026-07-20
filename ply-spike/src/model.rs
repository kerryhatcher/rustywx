//! Thin scan model — no egui dependency. Copied from rustywx.
//! `Option<f32>` gates: `None` = below threshold / range folded (drawn
//! transparent). Sweeps are sorted by elevation and deduplicated so split
//! cuts at the same elevation appear once in the tilt selector.

use chrono::{DateTime, Utc};
use nexrad_model::data::{MomentValue, Scan, Sweep};
use serde::{Deserialize, Serialize};

/// Radar products the app can display.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
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
#[derive(Clone, Serialize, Deserialize)]
pub struct RadialData {
    pub azimuth_deg: f32,
    pub gates: Vec<Option<f32>>,
}

/// One full rotation at a single elevation angle, for a single product.
#[derive(Clone, Serialize, Deserialize)]
pub struct SweepData {
    pub elevation_deg: f32,
    pub radials: Vec<RadialData>,
}

/// A decoded volume scan, split per product, sweeps sorted by elevation.
#[derive(Serialize, Deserialize)]
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
                    out.push(SweepData {
                        elevation_deg,
                        radials,
                    });
                }
            }
        }

        sort_and_dedup(&mut reflectivity);
        sort_and_dedup(&mut velocity);

        ScanData {
            timestamp,
            reflectivity,
            velocity,
        }
    }
}

/// Sort sweeps by elevation; split cuts produce near-duplicate elevations
/// (e.g. 0.48 and 0.52 deg) — keep only the first of each cluster.
fn sort_and_dedup(sweeps: &mut Vec<SweepData>) {
    sweeps.sort_by(|a, b| a.elevation_deg.total_cmp(&b.elevation_deg));
    sweeps.dedup_by(|current, previous| {
        (current.elevation_deg - previous.elevation_deg).abs() < 0.2
    });
}
