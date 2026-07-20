//! Loading and caching US state boundary and coastline data for the
//! radarscope overlay. Data comes from Natural Earth's 1:50m vectors.
//!
//! Stage 4: Replaces `ureq` with Ply `net` for fetching and Ply `storage`
//! for caching. The fetch is fire-and-poll: `net::get("borders", URL, |c| c)`
//! is called once, then `net::request("borders")` is polled each frame until
//! a response arrives.

use crate::geo;
use anyhow::{Result, anyhow};
use ply_engine::prelude::Storage;
use serde_json::Value;

/// One boundary line: a sequence of (lat, lon) vertices in degrees.
pub type Ring = Vec<(f64, f64)>;

/// Natural Earth 1:50m Admin 1 States/Provinces boundary lines.
const STATES_URL: &str = "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/ne_50m_admin_1_states_provinces_lines.geojson";

/// Natural Earth 1:50m coastline — much more detailed than 1:110m.
const COAST_URL: &str = "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/ne_50m_coastline.geojson";

/// Natural Earth 1:50m country boundary lines (international land borders).
const COUNTRY_LINES_URL: &str = "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/ne_50m_admin_0_boundary_lines_land.geojson";

/// Net request IDs for each border source.
pub const NET_ID_STATES: &str = "borders-states";
pub const NET_ID_COAST: &str = "borders-coast";
pub const NET_ID_COUNTRY: &str = "borders-country";

/// Storage key for the merged cached borders.
const STORAGE_KEY: &str = "state_borders_v8";

/// Fire all three border-fetch requests via Ply `net`. Idempotent — won't
/// re-fire if a request with the same ID already exists.
pub fn fire_fetch_all() {
    use ply_engine::prelude::net;
    net::get(NET_ID_STATES, STATES_URL, |c| c);
    net::get(NET_ID_COAST, COAST_URL, |c| c);
    net::get(NET_ID_COUNTRY, COUNTRY_LINES_URL, |c| c);
}

/// Check whether all three border requests have completed and, if so,
/// parse and merge the results. Returns `Some(rings)` when all three are
/// done, or `None` if any are still pending.
pub fn poll_and_merge() -> Option<Result<Vec<Ring>>> {
    use ply_engine::prelude::net;

    let states_resp = net::request(NET_ID_STATES)?.response()?;
    let coast_resp = net::request(NET_ID_COAST)?.response()?;
    let country_resp = net::request(NET_ID_COUNTRY)?.response()?;

    // All three have completed (response() returned Some).
    let states_resp = match states_resp {
        Ok(r) => r,
        Err(e) => return Some(Err(anyhow!("fetching states: {e}"))),
    };
    let coast_resp = match coast_resp {
        Ok(r) => r,
        Err(e) => return Some(Err(anyhow!("fetching coastlines: {e}"))),
    };
    let country_resp = match country_resp {
        Ok(r) => r,
        Err(e) => return Some(Err(anyhow!("fetching country lines: {e}"))),
    };

    let result = (|| {
        let mut rings = parse_state_lines(states_resp.text())?;
        rings.extend(parse_coastlines(coast_resp.text())?);
        rings.extend(parse_country_lines(country_resp.text())?);
        Ok(rings)
    })();

    Some(result)
}

/// Load cached border rings from Ply storage (async).
pub async fn load_cached(storage: &Storage) -> Result<Option<Vec<Ring>>> {
    let bytes = storage
        .load_bytes(STORAGE_KEY)
        .await
        .map_err(|e| anyhow!("loading borders cache: {e}"))?;

    match bytes {
        Some(data) => {
            let rings: Vec<Ring> = serde_json::from_slice(&data)
                .map_err(|e| anyhow!("parsing cached borders: {e}"))?;
            Ok(Some(rings))
        }
        None => Ok(None),
    }
}

/// Save merged border rings to Ply storage (fire-and-forget).
pub fn save_cached(storage: &Storage, rings: &[Ring]) {
    let storage = storage.clone();
    let rings = rings.to_vec();
    tokio::spawn(async move {
        if let Ok(json) = serde_json::to_vec(&rings) {
            let _ = storage.save_bytes(STORAGE_KEY, &json).await;
        }
    });
}

/// Parse Natural Earth state-lines GeoJSON. Filters to US states only
/// (`ADM0_A3 == "USA"`). Coordinates are `[lon, lat]` → `(lat, lon)`.
fn parse_state_lines(json: &str) -> Result<Vec<Ring>> {
    let root: Value = serde_json::from_str(json).map_err(|e| anyhow!("parsing states: {e}"))?;
    let features = root
        .get("features")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("states GeoJSON has no \"features\" array"))?;

    let mut rings = Vec::new();
    for feature in features {
        let adm0 = feature
            .get("properties")
            .and_then(|p| p.get("ADM0_A3"))
            .and_then(Value::as_str)
            .unwrap_or("");
        if adm0 != "USA" {
            continue;
        }
        if let Some(ring) = extract_linestring(feature) {
            rings.push(ring);
        }
    }
    Ok(rings)
}

/// Parse Natural Earth coastline GeoJSON. Takes all coastlines (global).
fn parse_coastlines(json: &str) -> Result<Vec<Ring>> {
    let root: Value = serde_json::from_str(json).map_err(|e| anyhow!("parsing coastlines: {e}"))?;
    let features = root
        .get("features")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("coastline GeoJSON has no \"features\" array"))?;

    let mut rings = Vec::new();
    for feature in features {
        if let Some(ring) = extract_linestring(feature) {
            rings.push(ring);
        }
    }
    Ok(rings)
}

/// Parse Natural Earth country boundary lines. Includes all international
/// land borders (the dataset is small enough that filtering isn't needed).
fn parse_country_lines(json: &str) -> Result<Vec<Ring>> {
    let root: Value =
        serde_json::from_str(json).map_err(|e| anyhow!("parsing country lines: {e}"))?;
    let features = root
        .get("features")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("country lines GeoJSON has no \"features\" array"))?;

    let mut rings = Vec::new();
    for feature in features {
        if let Some(ring) = extract_linestring(feature) {
            rings.push(ring);
        }
    }
    Ok(rings)
}

/// Extract a LineString geometry from a GeoJSON feature as a Ring.
fn extract_linestring(feature: &Value) -> Option<Ring> {
    let coords = feature
        .get("geometry")
        .and_then(|g| g.get("coordinates"))
        .and_then(Value::as_array)?;
    let ring: Vec<(f64, f64)> = coords
        .iter()
        .filter_map(|pt| {
            let arr = pt.as_array()?;
            Some((arr.get(1)?.as_f64()?, arr.first()?.as_f64()?))
        })
        .collect();
    if ring.len() >= 2 { Some(ring) } else { None }
}

/// Does a ring have any portion inside the 230 km radarscope?
pub fn ring_affects_scope(ring: &Ring, origin_lat: f64, origin_lon: f64) -> bool {
    if ring.len() < 2 {
        return false;
    }
    for pair in ring.windows(2) {
        let a = geo::point_to_km_offset(origin_lat, origin_lon, pair[0]);
        let b = geo::point_to_km_offset(origin_lat, origin_lon, pair[1]);
        if !geo::circle_subsegments(a, b, crate::scope::MAX_RANGE_KM).is_empty() {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_state_lines_as_lat_lon_pairs() {
        let json = r#"{
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "properties": {"ADM0_A3": "USA"},
                "geometry": {
                    "type": "LineString",
                    "coordinates": [[-83.0, 32.0], [-84.0, 33.0], [-85.0, 34.0]]
                }
            }]
        }"#;
        let rings = parse_state_lines(json).unwrap();
        assert_eq!(rings.len(), 1);
        assert_eq!(rings[0].len(), 3);
        assert!((rings[0][0].0 - 32.0).abs() < 0.001);
        assert!((rings[0][0].1 - (-83.0)).abs() < 0.001);
    }

    #[test]
    fn parses_coastlines() {
        let json = r#"{
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "properties": {},
                "geometry": {
                    "type": "LineString",
                    "coordinates": [[-80.0, 28.0], [-81.0, 29.0]]
                }
            }]
        }"#;
        let rings = parse_coastlines(json).unwrap();
        assert_eq!(rings.len(), 1);
    }

    #[test]
    fn rejects_malformed_json() {
        assert!(parse_state_lines("not json").is_err());
    }

    #[test]
    fn rejects_missing_features_array() {
        assert!(parse_state_lines(r#"{"type": "FeatureCollection"}"#).is_err());
    }

    #[test]
    fn filters_non_us_features() {
        let json = r#"{
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "properties": {"ADM0_A3": "CAN"},
                "geometry": {
                    "type": "LineString",
                    "coordinates": [[-80.0, 44.0], [-81.0, 45.0]]
                }
            }]
        }"#;
        let rings = parse_state_lines(json).unwrap();
        assert!(rings.is_empty());
    }
}
