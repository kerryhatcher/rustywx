//! Active NWS warning/watch polygon overlay for the radarscope.
//!
//! Stage 4: Replaces `ureq` with Ply `net` for fetching. Colors use
//! `[u8; 4]` arrays instead of `egui::Color32`. The fetch is fire-and-poll:
//! `net::get("alerts", URL, |c| c.header("User-Agent", ...))` is called
//! once, then `net::request("alerts")` is polled each frame.

use crate::borders::Ring;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// How often to refresh the active alert list. NWS warning/watch updates are
/// not instantaneous, so two minutes matches the radar volume poll interval.
pub const POLL_INTERVAL: Duration = Duration::from_secs(120);

/// NWS API endpoint: actual (non-test) alerts for the entire US.
/// Using `area=GA` was too restrictive for a national radar app; we now
/// fetch all active alerts and filter by scope intersection.
const NWS_ALERTS_URL: &str = "https://api.weather.gov/alerts/active?status=actual";
const USER_AGENT: &str = "rustywx/dev (https://github.com/rustywx/rustywx)";

/// Net request ID for the alerts fetch.
pub const NET_ID: &str = "alerts";

/// A single active warning or watch, ready to draw on the scope.
#[derive(Serialize, Deserialize)]
pub struct Alert {
    pub event: String,
    pub headline: String,
    pub color: [u8; 4],
    pub rings: Vec<Ring>,
}

/// Fire the NWS alerts fetch via Ply `net`. Idempotent.
pub fn fire_fetch() {
    use ply_engine::prelude::net;
    net::get(NET_ID, NWS_ALERTS_URL, |c| {
        c.header("User-Agent", USER_AGENT)
            .header("Accept", "application/geo+json")
    });
}

/// Poll for the alerts response. Returns `Some(Ok(alerts))` when done,
/// `Some(Err(e))` on error, or `None` if still pending.
pub fn poll_response() -> Option<Result<Vec<Alert>>> {
    use ply_engine::prelude::net;

    let resp = net::request(NET_ID)?.response()?;
    let resp = match resp {
        Ok(r) => r,
        Err(e) => return Some(Err(anyhow!("fetching NWS alerts: {e}"))),
    };

    Some(parse_alerts(resp.text()))
}

/// Parse a GeoJSON FeatureCollection of NWS alerts, filtering to warnings and
/// watches that overlap the 230 km scope around the given radar site.
pub fn parse_alerts(json: &str) -> Result<Vec<Alert>> {
    let root: Value =
        serde_json::from_str(json).map_err(|e| anyhow!("parsing alerts JSON: {e}"))?;
    let features = root
        .get("features")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("alerts GeoJSON has no \"features\" array"))?;

    let mut alerts = Vec::new();
    for feature in features {
        let properties = feature
            .get("properties")
            .ok_or_else(|| anyhow!("alert feature has no \"properties\""))?;
        let event = properties
            .get("event")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        if !is_warning_or_watch(&event) {
            continue;
        }

        let headline = properties
            .get("headline")
            .and_then(Value::as_str)
            .unwrap_or(&event)
            .to_string();

        let geometry = match feature.get("geometry") {
            Some(g) if !g.is_null() => g,
            _ => continue,
        };
        let geom_type = geometry
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("alert geometry has no \"type\""))?;
        let coordinates = geometry
            .get("coordinates")
            .ok_or_else(|| anyhow!("alert geometry has no \"coordinates\""))?;

        let mut rings = Vec::new();
        match geom_type {
            "Polygon" => rings.extend(parse_polygon_rings(coordinates)?),
            "MultiPolygon" => {
                let polygons = coordinates
                    .as_array()
                    .ok_or_else(|| anyhow!("MultiPolygon coordinates are not an array"))?;
                for polygon in polygons {
                    rings.extend(parse_polygon_rings(polygon)?);
                }
            }
            other => return Err(anyhow!("unsupported alert geometry type: {other}")),
        }

        // Note: scope filtering is done in the app layer using the current
        // radar site, not here, since the site can change between fetches.
        alerts.push(Alert {
            event: event.clone(),
            headline,
            color: alert_color(&event),
            rings,
        });
    }

    Ok(alerts)
}

/// Convert a Polygon `coordinates` value into `(lat, lon)` rings. GeoJSON
/// gives coordinates as `[lon, lat]`; we flip them to match the rest of the
/// app.
fn parse_polygon_rings(coordinates: &Value) -> Result<Vec<Ring>> {
    let rings_json = coordinates
        .as_array()
        .ok_or_else(|| anyhow!("Polygon coordinates are not an array"))?;

    rings_json
        .iter()
        .map(|ring| {
            let points = ring
                .as_array()
                .ok_or_else(|| anyhow!("alert ring is not an array"))?;
            points
                .iter()
                .map(|point| {
                    let pair = point
                        .as_array()
                        .ok_or_else(|| anyhow!("alert coordinate pair is not an array"))?;
                    let lon = pair
                        .first()
                        .and_then(Value::as_f64)
                        .ok_or_else(|| anyhow!("missing longitude in alert"))?;
                    let lat = pair
                        .get(1)
                        .and_then(Value::as_f64)
                        .ok_or_else(|| anyhow!("missing latitude in alert"))?;
                    Ok((lat, lon))
                })
                .collect()
        })
        .collect()
}

fn is_warning_or_watch(event: &str) -> bool {
    let e = event.to_ascii_lowercase();
    e.contains("warning") || e.contains("watch")
}

/// Map a common NWS event name to a bright warning/watch color. Unknown
/// warnings default to red, unknown watches to yellow. Returns `[r, g, b, a]`.
pub fn alert_color(event: &str) -> [u8; 4] {
    let e = event.to_ascii_lowercase();
    if e.contains("tornado warning") {
        [0xff, 0x00, 0x00, 0xff] // bright red
    } else if e.contains("severe thunderstorm warning") {
        [0xff, 0xa5, 0x00, 0xff] // orange
    } else if e.contains("flash flood emergency") || e.contains("flash flood warning") {
        [0x00, 0xff, 0x00, 0xff] // bright green
    } else if e.contains("flood warning") || e.contains("areal flood warning") {
        [0x00, 0xcc, 0xcc, 0xff] // cyan
    } else if e.contains("tornado watch") {
        [0xff, 0x14, 0x93, 0xff] // hot pink
    } else if e.contains("severe thunderstorm watch") {
        [0xff, 0xd7, 0x00, 0xff] // gold
    } else if e.contains("flash flood watch") {
        [0x2e, 0x8b, 0x57, 0xff] // sea green
    } else if e.contains("warning") {
        [0xff, 0x33, 0x33, 0xff] // generic warning red
    } else if e.contains("watch") {
        [0xff, 0xee, 0x00, 0xff] // generic watch yellow
    } else {
        [0xff, 0xff, 0xff, 0xff] // white
    }
}

/// Check whether any ring of an alert intersects the scope around the
/// given radar site.
pub fn alert_affects_scope(alert: &Alert, origin_lat: f64, origin_lon: f64) -> bool {
    alert
        .rings
        .iter()
        .any(|r| crate::borders::ring_affects_scope(r, origin_lat, origin_lon))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_matches_common_events() {
        assert_eq!(alert_color("Tornado Warning"), [0xff, 0x00, 0x00, 0xff]);
        assert_eq!(
            alert_color("Severe Thunderstorm Warning"),
            [0xff, 0xa5, 0x00, 0xff]
        );
        assert_eq!(alert_color("Flash Flood Warning"), [0x00, 0xff, 0x00, 0xff]);
        assert_eq!(alert_color("Tornado Watch"), [0xff, 0x14, 0x93, 0xff]);
        assert_eq!(
            alert_color("Severe Thunderstorm Watch"),
            [0xff, 0xd7, 0x00, 0xff]
        );
    }

    #[test]
    fn generic_warning_is_red_watch_is_yellow() {
        assert_eq!(
            alert_color("Special Weather Warning"),
            [0xff, 0x33, 0x33, 0xff]
        );
        assert_eq!(alert_color("Heat Watch"), [0xff, 0xee, 0x00, 0xff]);
    }

    #[test]
    fn parses_simple_alert_geojson() {
        let fixture = r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "properties": {
                        "event": "Tornado Warning",
                        "headline": "Tornado Warning for..."
                    },
                    "geometry": {
                        "type": "Polygon",
                        "coordinates": [[[-83.4, 32.6], [-83.4, 32.7], [-83.3, 32.7], [-83.3, 32.6], [-83.4, 32.6]]]
                    }
                }
            ]
        }"#;
        let alerts = parse_alerts(fixture).unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].event, "Tornado Warning");
        assert_eq!(alerts[0].rings.len(), 1);
        assert_eq!(alerts[0].rings[0][0], (32.6, -83.4));
    }

    #[test]
    fn ignores_advisories_and_null_geometry() {
        let fixture = r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "properties": {"event": "Flood Advisory"},
                    "geometry": {
                        "type": "Polygon",
                        "coordinates": [[[-83.4, 32.6], [-83.4, 32.7], [-83.3, 32.6], [-83.4, 32.6]]]
                    }
                },
                {
                    "type": "Feature",
                    "properties": {"event": "Tornado Watch"},
                    "geometry": null
                }
            ]
        }"#;
        let alerts = parse_alerts(fixture).unwrap();
        assert!(alerts.is_empty());
    }
}
