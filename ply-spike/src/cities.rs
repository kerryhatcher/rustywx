//! City data parsed from a bundled Natural Earth GeoJSON, with population.
//!
//! Stage 6 replaces the old hand-curated `geo::CITIES` constant (1,194
//! entries, no population) with Natural Earth 10m populated places trimmed
//! to North America (1,369 entries, each with `pop_max`). The data is
//! embedded via `include_str!` (no filesystem/CWD dependency — same lesson
//! as the blur shader) and parsed once into a `OnceLock<Vec<City>>`.
//!
//! The population field enables **progressive disclosure**: the scope
//! draw code filters cities by a population threshold derived from the
//! zoom level (biggest cities when zoomed out, progressively smaller as
//! you zoom in) and then greedily places labels with collision avoidance
//! so names never overlap.

use std::sync::OnceLock;

/// A city with its geographic coordinates and population.
#[derive(Debug, Clone)]
pub struct City {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub pop: i64,
}

/// The bundled GeoJSON source (trimmed Natural Earth 10m populated places,
/// North America only, fields: name, pop, geometry).
const CITIES_GEOJSON: &str = include_str!("../assets/cities.geojson");

static CITIES: OnceLock<Vec<City>> = OnceLock::new();

/// Return the parsed city list, parsing on first access.
pub fn cities() -> &'static [City] {
    CITIES.get_or_init(|| parse_geojson(CITIES_GEOJSON).unwrap_or_default())
}

/// Parse a GeoJSON FeatureCollection of point features with `name` and
/// `pop` properties into a `Vec<City>`.
fn parse_geojson(source: &str) -> Result<Vec<City>, serde_json::Error> {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct FeatureCollection {
        features: Vec<Feature>,
    }
    #[derive(Deserialize)]
    struct Feature {
        properties: Props,
        geometry: Geometry,
    }
    #[derive(Deserialize)]
    struct Props {
        name: String,
        pop: i64,
    }
    #[derive(Deserialize)]
    struct Geometry {
        coordinates: (f64, f64), // (lon, lat)
    }

    let fc: FeatureCollection = serde_json::from_str(source)?;
    let out = fc
        .features
        .into_iter()
        .map(|f| City {
            name: f.properties.name,
            lon: f.geometry.coordinates.0,
            lat: f.geometry.coordinates.1,
            pop: f.properties.pop,
        })
        .collect();
    Ok(out)
}

/// Compute the population threshold for a given zoom level.
///
/// Progressive disclosure: as zoom increases (zoom in), the threshold
/// decreases, revealing progressively smaller cities. The curve is
/// `threshold = BASE / zoom`, clamped so that at extreme zoom-out only
/// megacities show and at extreme zoom-in even small towns appear.
///
/// | zoom | radius (km) | min pop |
/// |------|-------------|---------|
/// | 0.25 | ~920        | 1.2M    |
/// | 0.5  | ~460        | 600K    |
/// | 1.0  | ~230        | 300K    |
/// | 2.0  | ~115        | 150K    |
/// | 4.0  | ~57         | 75K     |
/// | 8.0  | ~29         | ~1K (floor) |
pub fn min_population_for_zoom(zoom: f32) -> i64 {
    const BASE: f32 = 300_000.0;
    const FLOOR: f32 = 1_000.0;
    const CEILING: f32 = 5_000_000.0;
    let threshold = (BASE / zoom).clamp(FLOOR, CEILING);
    threshold as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cities_load_nonempty() {
        let c = cities();
        assert!(!c.is_empty(), "city list should not be empty");
        // Every city has a name and a population.
        assert!(c.iter().all(|city| !city.name.is_empty()));
    }

    #[test]
    fn cities_have_population() {
        let c = cities();
        // Natural Earth includes major cities; at least some have pop > 1M.
        assert!(c.iter().any(|city| city.pop > 1_000_000));
    }

    #[test]
    fn cities_are_in_north_america() {
        let c = cities();
        // All within the trim bounding box (lat 14-72, lon -180 to -50).
        assert!(c.iter().all(|city| (14.0..=72.0).contains(&city.lat)));
        assert!(c.iter().all(|city| (-180.0..=-50.0).contains(&city.lon)));
    }

    #[test]
    fn min_population_decreases_with_zoom() {
        // Zooming in lowers the threshold (more cities show).
        let low = min_population_for_zoom(0.5);
        let mid = min_population_for_zoom(1.0);
        let high = min_population_for_zoom(4.0);
        assert!(low > mid, "zoomed-out threshold should be higher");
        assert!(mid > high, "threshold should drop as you zoom in");
    }

    #[test]
    fn min_population_is_clamped() {
        // Extreme zoom-out clamps to the ceiling.
        assert_eq!(min_population_for_zoom(0.01), 5_000_000);
        // Extreme zoom-in clamps to the floor (300K / 1000 = 300 < 1000).
        assert_eq!(min_population_for_zoom(1000.0), 1_000);
    }
}
