//! Loading and caching US state boundary data for the radarscope overlay.
//!
//! Boundaries come from the Census Bureau's TIGERweb REST API as GeoJSON.
//! GeoJSON coordinates are `[lon, lat]`; everything in this module flips
//! that to `(lat, lon)` to match the rest of the codebase's convention
//! (see `geo::CITIES`, `geo::range_bearing`).

use anyhow::{Result, anyhow};
use serde_json::Value;
use std::path::{Path, PathBuf};

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

/// Where the borders cache lives, under a given home directory. Split out
/// from `cache_path` so it's testable without touching the real `$HOME` env
/// var (mutating env vars in tests is racy across parallel test threads).
fn cache_path_under(home: &Path) -> PathBuf {
    home.join(".rustywx").join("state_borders.geojson")
}

/// Where the borders cache lives on this machine.
fn cache_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").map_err(|_| anyhow!("HOME environment variable is not set"))?;
    Ok(cache_path_under(Path::new(&home)))
}

/// Load cached state-boundary rings from `path`, fetching and caching them
/// first if the file doesn't exist yet. Malformed or unreadable existing
/// files are returned as errors rather than silently overwritten — a
/// corrupt cache is left for a human to investigate.
pub fn load_or_fetch(path: &Path) -> Result<Vec<Ring>> {
    match std::fs::read_to_string(path) {
        Ok(json) => parse_geojson_rings(&json),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let json = fetch_geojson()?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| anyhow!("creating {}: {e}", parent.display()))?;
            }
            std::fs::write(path, &json).map_err(|e| anyhow!("writing {}: {e}", path.display()))?;
            parse_geojson_rings(&json)
        }
        Err(e) => Err(anyhow!("reading {}: {e}", path.display())),
    }
}

/// Georgia plus its neighbors whose border could plausibly fall within the
/// scope's 230 km display radius from KJGX: Alabama, South Carolina, Florida.
/// Verified live against the TIGERweb service: returns 4 features (SC, GA,
/// AL as `Polygon`, FL as `MultiPolygon`).
const TIGERWEB_URL: &str = "https://tigerweb.geo.census.gov/arcgis/rest/services/TIGERweb/State_County/MapServer/0/query?where=STUSAB+IN+(%27GA%27%2C%27AL%27%2C%27SC%27%2C%27FL%27)&outFields=STUSAB,NAME&f=geojson";

/// Fetch the GA/AL/SC/FL state boundaries from the Census Bureau's TIGERweb
/// REST service as GeoJSON. A single blocking request — unlike
/// `data::fetch_latest_scan`, this has no polling loop, since boundary data
/// doesn't change on a timescale this app cares about.
fn fetch_geojson() -> Result<String> {
    ureq::get(TIGERWEB_URL)
        .call()
        .map_err(|e| anyhow!("fetching state borders: {e}"))?
        .body_mut()
        .read_to_string()
        .map_err(|e| anyhow!("reading state borders response: {e}"))
}

/// Messages the border-loader thread sends to the UI thread.
pub enum BorderMessage {
    Loaded(Vec<Ring>),
    Error(String),
}

/// Spawn a one-shot thread that loads (fetching and caching first, if
/// needed) the state-boundary rings and reports them once. Unlike
/// `data::spawn_worker`, this thread does its work once and exits — there's
/// no polling loop, since state borders aren't expected to change.
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

    #[test]
    fn cache_path_is_under_dot_rustywx() {
        let path = cache_path_under(std::path::Path::new("/home/example"));
        assert_eq!(
            path,
            std::path::Path::new("/home/example/.rustywx/state_borders.geojson")
        );
    }

    fn unique_temp_path(name: &str) -> PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("rustywx-test-{}-{n}-{name}", std::process::id()))
    }

    #[test]
    fn load_or_fetch_reads_existing_cache_without_network() {
        let path = unique_temp_path("cache-hit.geojson");
        std::fs::write(&path, POLYGON_FIXTURE).unwrap();

        let rings = load_or_fetch(&path).unwrap();

        assert_eq!(rings.len(), 1);
        std::fs::remove_file(&path).unwrap();
    }
}
