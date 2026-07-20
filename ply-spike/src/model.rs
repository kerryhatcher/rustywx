//! Thin scan model — no egui dependency. Copied from rustywx.

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
