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
/// decreases in discrete steps, revealing progressively smaller cities.
/// The default zoom (1.0) shows cities with population ≥ 75K.
///
/// | zoom range      | radius (km) | min pop |
/// |-----------------|-------------|---------|
/// | < 0.3           | ~770+       | 1M      |
/// | 0.3 – 0.5       | ~460–770    | 400K    |
/// | 0.5 – 0.8       | ~290–460    | 100K    |
/// | 0.8 – 1.5       | ~155–290    | 75K     |
/// | 1.5 – 3.0       | ~77–155     | 25K     |
/// | ≥ 3.0           | < 77        | 1K      |
pub fn min_population_for_zoom(zoom: f32) -> i64 {
    if zoom < 0.3 {
        1_000_000
    } else if zoom < 0.5 {
        400_000
    } else if zoom < 0.8 {
        100_000
    } else if zoom < 1.5 {
        75_000
    } else if zoom < 3.0 {
        25_000
    } else {
        1_000
    }
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
        let low = min_population_for_zoom(0.2);
        let mid = min_population_for_zoom(1.0);
        let high = min_population_for_zoom(4.0);
        assert!(low > mid, "zoomed-out threshold should be higher");
        assert!(mid > high, "threshold should drop as you zoom in");
    }

    #[test]
    fn min_population_steps() {
        // Discrete steps matching the design.
        assert_eq!(min_population_for_zoom(0.1), 1_000_000); // < 0.3
        assert_eq!(min_population_for_zoom(0.4), 400_000); // 0.3–0.5
        assert_eq!(min_population_for_zoom(0.6), 100_000); // 0.5–0.8
        assert_eq!(min_population_for_zoom(1.0), 75_000); // 0.8–1.5 (default)
        assert_eq!(min_population_for_zoom(2.0), 25_000); // 1.5–3.0
        assert_eq!(min_population_for_zoom(4.0), 1_000); // ≥ 3.0 (floor)
    }
}
