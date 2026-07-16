//! Loading and caching US state boundary data for the radarscope overlay.
//!
//! Boundaries come from the Census Bureau's TIGERweb REST API as GeoJSON.
//! GeoJSON coordinates are `[lon, lat]`; everything in this module flips
//! that to `(lat, lon)` to match the rest of the codebase's convention
//! (see `geo::CITIES`, `geo::range_bearing`).

use anyhow::{Result, anyhow};
use serde_json::Value;

/// One polygon ring: a closed sequence of (lat, lon) vertices in degrees.
pub type Ring = Vec<(f64, f64)>;

/// Parse a GeoJSON `FeatureCollection` of Polygon/MultiPolygon features into
/// a flat list of rings. Ring winding order (exterior vs. hole) and feature
/// properties are ignored — every ring is drawn as a plain border line.
fn parse_geojson_rings(json: &str) -> Result<Vec<Ring>> {
    let root: Value = serde_json::from_str(json).map_err(|e| anyhow!("parsing GeoJSON: {e}"))?;
    let features = root
        .get("features")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("GeoJSON has no \"features\" array"))?;

    let mut rings = Vec::new();
    for feature in features {
        let geometry = feature
            .get("geometry")
            .ok_or_else(|| anyhow!("feature has no \"geometry\""))?;
        let geom_type = geometry
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("geometry has no \"type\""))?;
        let coordinates = geometry
            .get("coordinates")
            .ok_or_else(|| anyhow!("geometry has no \"coordinates\""))?;

        match geom_type {
            "Polygon" => rings.extend(polygon_rings(coordinates)?),
            "MultiPolygon" => {
                let polygons = coordinates
                    .as_array()
                    .ok_or_else(|| anyhow!("MultiPolygon coordinates are not an array"))?;
                for polygon in polygons {
                    rings.extend(polygon_rings(polygon)?);
                }
            }
            other => return Err(anyhow!("unsupported geometry type: {other}")),
        }
    }
    Ok(rings)
}

/// Convert a Polygon's `coordinates` value (an array of rings, each an
/// array of `[lon, lat]` pairs) into `Ring`s.
fn polygon_rings(coordinates: &Value) -> Result<Vec<Ring>> {
    let rings_json = coordinates
        .as_array()
        .ok_or_else(|| anyhow!("Polygon coordinates are not an array"))?;

    rings_json
        .iter()
        .map(|ring| {
            let points = ring
                .as_array()
                .ok_or_else(|| anyhow!("ring is not an array"))?;
            points
                .iter()
                .map(|point| {
                    let pair = point
                        .as_array()
                        .ok_or_else(|| anyhow!("coordinate pair is not an array"))?;
                    let lon = pair
                        .first()
                        .and_then(Value::as_f64)
                        .ok_or_else(|| anyhow!("missing longitude"))?;
                    let lat = pair
                        .get(1)
                        .and_then(Value::as_f64)
                        .ok_or_else(|| anyhow!("missing latitude"))?;
                    Ok((lat, lon))
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLYGON_FIXTURE: &str = r#"{
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "properties": {"STUSAB": "GA"},
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[[-83.0, 32.0], [-83.1, 32.1], [-83.2, 32.0], [-83.0, 32.0]]]
                }
            }
        ]
    }"#;

    const MULTIPOLYGON_FIXTURE: &str = r#"{
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "properties": {"STUSAB": "FL"},
                "geometry": {
                    "type": "MultiPolygon",
                    "coordinates": [
                        [[[-82.0, 27.0], [-82.1, 27.1], [-82.2, 27.0], [-82.0, 27.0]]],
                        [[[-80.0, 25.0], [-80.1, 25.1], [-80.2, 25.0], [-80.0, 25.0]]]
                    ]
                }
            }
        ]
    }"#;

    #[test]
    fn parses_polygon_ring_as_lat_lon_pairs() {
        let rings = parse_geojson_rings(POLYGON_FIXTURE).unwrap();
        assert_eq!(rings.len(), 1);
        assert_eq!(
            rings[0],
            vec![(32.0, -83.0), (32.1, -83.1), (32.0, -83.2), (32.0, -83.0)]
        );
    }

    #[test]
    fn parses_multipolygon_as_one_ring_per_part() {
        let rings = parse_geojson_rings(MULTIPOLYGON_FIXTURE).unwrap();
        assert_eq!(rings.len(), 2);
        assert_eq!(rings[0][0], (27.0, -82.0));
        assert_eq!(rings[1][0], (25.0, -80.0));
    }

    #[test]
    fn rejects_malformed_json() {
        assert!(parse_geojson_rings("not json").is_err());
    }

    #[test]
    fn rejects_missing_features_array() {
        assert!(parse_geojson_rings(r#"{"type": "FeatureCollection"}"#).is_err());
    }
}
