//! Loading and caching US state boundary and coastline data for the
//! radarscope overlay. Data comes from Natural Earth's 1:110m vectors.

use anyhow::{Result, anyhow};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// One boundary line: a sequence of (lat, lon) vertices in degrees.
pub type Ring = Vec<(f64, f64)>;

/// Natural Earth 1:50m Admin 1 States/Provinces boundary lines.
const STATES_URL: &str = "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/ne_50m_admin_1_states_provinces_lines.geojson";

/// Natural Earth 1:50m coastline — much more detailed than 1:110m.
const COAST_URL: &str = "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/ne_50m_coastline.geojson";

/// Natural Earth 1:50m country boundary lines (international land borders).
const COUNTRY_LINES_URL: &str = "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/ne_50m_admin_0_boundary_lines_land.geojson";

/// Fetch a GeoJSON file from a URL.
fn fetch_url(url: &str) -> Result<String> {
    ureq::get(url)
        .call()
        .map_err(|e| anyhow!("fetching {url}: {e}"))?
        .body_mut()
        .read_to_string()
        .map_err(|e| anyhow!("reading {url}: {e}"))
}

/// Fetch and parse state lines, coastlines, and country borders, merging
/// them into one list of rings.
fn fetch_all() -> Result<Vec<Ring>> {
    let mut rings = parse_state_lines(&fetch_url(STATES_URL)?)?;
    rings.extend(parse_coastlines(&fetch_url(COAST_URL)?)?);
    rings.extend(parse_us_country_lines(&fetch_url(COUNTRY_LINES_URL)?)?);
    Ok(rings)
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
/// land borders (the dataset is small enough that filtering isn't needed —
/// the painter's clip rect handles off-screen lines).
fn parse_us_country_lines(json: &str) -> Result<Vec<Ring>> {
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
        let a = crate::geo::point_to_km_offset(origin_lat, origin_lon, pair[0]);
        let b = crate::geo::point_to_km_offset(origin_lat, origin_lon, pair[1]);
        if !crate::geo::circle_subsegments(a, b, crate::scope::MAX_RANGE_KM).is_empty() {
            return true;
        }
    }
    false
}

/// Where the borders cache lives, under a given home directory.
fn cache_path_under(home: &Path) -> PathBuf {
    home.join(".rustywx").join("state_borders_v8.geojson")
}

fn cache_path() -> Result<PathBuf> {
    let home =
        std::env::var("HOME").map_err(|_| anyhow!("HOME environment variable is not set"))?;
    Ok(cache_path_under(Path::new(&home)))
}

/// Load cached state-boundary rings from `path`, fetching and caching them
/// first if the file doesn't exist yet. The cache stores the merged result
/// of state lines + coastlines as JSON.
pub fn load_or_fetch(path: &Path) -> Result<Vec<Ring>> {
    match std::fs::read_to_string(path) {
        Ok(json) => serde_json::from_str(&json).map_err(|e| anyhow!("parsing cached borders: {e}")),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let rings = fetch_all()?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| anyhow!("creating {}: {e}", parent.display()))?;
            }
            let json =
                serde_json::to_string(&rings).map_err(|e| anyhow!("serializing borders: {e}"))?;
            std::fs::write(path, &json).map_err(|e| anyhow!("writing {}: {e}", path.display()))?;
            Ok(rings)
        }
        Err(e) => Err(anyhow!("reading {}: {e}", path.display())),
    }
}

/// Messages the border-loader thread sends to the UI thread.
pub enum BorderMessage {
    Loaded(Vec<Ring>),
    Error(String),
}

/// Spawn a one-shot thread that loads (fetching and caching first, if
/// needed) the state boundary rings and sends them to the UI.
pub fn spawn_border_loader(tx: std::sync::mpsc::Sender<BorderMessage>, egui_ctx: egui::Context) {
    std::thread::spawn(move || {
        let message = match cache_path().and_then(|path| load_or_fetch(&path)) {
            Ok(rings) => BorderMessage::Loaded(rings),
            Err(e) => BorderMessage::Error(format!("{e:#}")),
        };
        let _ = tx.send(message);
        egui_ctx.request_repaint();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_path_is_under_dot_rustywx() {
        let path = cache_path_under(std::path::Path::new("/home/example"));
        assert_eq!(
            path,
            std::path::Path::new("/home/example/.rustywx/state_borders_v8.geojson")
        );
    }

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

    fn unique_temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("rustywx-test-{}-{}", std::process::id(), name))
    }

    #[test]
    fn load_or_fetch_reads_existing_cache_without_network() {
        let path = unique_temp_path("cache-hit.geojson");
        // Cache stores serialized Vec<Ring> as JSON.
        let rings: Vec<Ring> = vec![vec![(32.0, -83.0), (33.0, -84.0)]];
        let json = serde_json::to_string(&rings).unwrap();
        std::fs::write(&path, json).unwrap();
        let loaded = load_or_fetch(&path).unwrap();
        assert_eq!(loaded.len(), 1);
        let _ = std::fs::remove_file(&path);
    }
}
