//! Active NWS warning/watch polygon overlay for the radarscope.
//!
//! Fetches warnings and watches from the National Weather Service API
//! (`api.weather.gov/alerts/active`) for Georgia, parses their GeoJSON
//! geometries, and reports the ones whose polygons intersect the scope back to
//! the UI thread. Polygons are drawn in bright NWS-style colors and clipped
//! to the radar display circle.

use crate::borders::Ring;
use crate::geo;
use anyhow::{Result, anyhow};
use egui::{Color32, Vec2};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::mpsc::Sender;
use std::time::Duration;

/// How often to refresh the active alert list. NWS warning/watch updates are
/// not instantaneous, so two minutes matches the radar volume poll interval.
pub const POLL_INTERVAL: Duration = Duration::from_secs(120);

/// NWS API endpoint: actual (non-test) alerts for Georgia.
const NWS_ALERTS_URL: &str = "https://api.weather.gov/alerts/active?area=GA&status=actual";
const USER_AGENT: &str = "rustywx/dev (https://github.com/rustywx/rustywx)";

/// A single active warning or watch, ready to draw on the scope.
#[derive(Serialize, Deserialize)]
pub struct Alert {
    pub event: String,
    pub headline: String,
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub color: Color32,
    pub rings: Vec<Ring>,
}

fn serialize_color<S: serde::Serializer>(c: &Color32, s: S) -> Result<S::Ok, S::Error> {
    let hex = format!("{:02x}{:02x}{:02x}{:02x}", c.r(), c.g(), c.b(), c.a());
    s.serialize_str(&hex)
}

fn deserialize_color<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Color32, D::Error> {
    let hex = String::deserialize(d)?;
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);
    Ok(Color32::from_rgba_premultiplied(r, g, b, a))
}

/// Messages the alerts worker sends to the UI thread.
pub enum AlertMessage {
    Loaded(Vec<Alert>),
    Error(String),
}

/// Spawn a background thread that refreshes active NWS warnings/watches
/// every [`POLL_INTERVAL`].
pub fn spawn_alerts_worker(tx: Sender<AlertMessage>, egui_ctx: egui::Context) {
    std::thread::spawn(move || {
        loop {
            let message = match fetch_alerts() {
                Ok(alerts) => {
                    crate::cache::save_alerts(&alerts);
                    AlertMessage::Loaded(alerts)
                }
                Err(e) => AlertMessage::Error(format!("{e:#}")),
            };
            let _ = tx.send(message);
            egui_ctx.request_repaint();
            std::thread::sleep(POLL_INTERVAL);
        }
    });
}

/// Fetch and parse active NWS alerts for Georgia, keeping only warnings and
/// watches whose polygons intersect the radar display radius.
fn fetch_alerts() -> Result<Vec<Alert>> {
    let json = ureq::get(NWS_ALERTS_URL)
        .header("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| anyhow!("fetching NWS alerts: {e}"))?
        .body_mut()
        .read_to_string()
        .map_err(|e| anyhow!("reading NWS alerts response: {e}"))?;

    parse_alerts(&json)
}

/// Parse a GeoJSON FeatureCollection of NWS alerts, filtering to warnings and
/// watches that overlap the 230 km scope around KJGX.
fn parse_alerts(json: &str) -> Result<Vec<Alert>> {
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

        if !rings.iter().any(|r| {
            crate::borders::ring_affects_scope(r, crate::geo::KJGX_LAT, crate::geo::KJGX_LON)
        }) {
            continue;
        }

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
/// warnings default to red, unknown watches to yellow.
pub fn alert_color(event: &str) -> Color32 {
    let e = event.to_ascii_lowercase();
    if e.contains("tornado warning") {
        Color32::from_rgb(0xff, 0x00, 0x00) // bright red
    } else if e.contains("severe thunderstorm warning") {
        Color32::from_rgb(0xff, 0xa5, 0x00) // orange
    } else if e.contains("flash flood emergency") || e.contains("flash flood warning") {
        Color32::from_rgb(0x00, 0xff, 0x00) // bright green
    } else if e.contains("flood warning") || e.contains("areal flood warning") {
        Color32::from_rgb(0x00, 0xcc, 0xcc) // cyan
    } else if e.contains("tornado watch") {
        Color32::from_rgb(0xff, 0x14, 0x93) // hot pink
    } else if e.contains("severe thunderstorm watch") {
        Color32::from_rgb(0xff, 0xd7, 0x00) // gold
    } else if e.contains("flash flood watch") {
        Color32::from_rgb(0x2e, 0x8b, 0x57) // sea green
    } else if e.contains("warning") {
        Color32::from_rgb(0xff, 0x33, 0x33) // generic warning red
    } else if e.contains("watch") {
        Color32::from_rgb(0xff, 0xee, 0x00) // generic watch yellow
    } else {
        Color32::WHITE
    }
}

/// Return the portions of the segment `a` -> `b` that lie inside or on a
/// circle centered at the origin with radius `r`. This lets large alert
/// polygons that only clip through the scope still draw their visible
/// chord.
pub fn circle_subsegments(a: Vec2, b: Vec2, r: f32) -> Vec<(Vec2, Vec2)> {
    geo::circle_subsegments(a, b, r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Vec2;

    #[test]
    fn color_matches_common_events() {
        assert_eq!(
            alert_color("Tornado Warning"),
            Color32::from_rgb(0xff, 0x00, 0x00)
        );
        assert_eq!(
            alert_color("Severe Thunderstorm Warning"),
            Color32::from_rgb(0xff, 0xa5, 0x00)
        );
        assert_eq!(
            alert_color("Flash Flood Warning"),
            Color32::from_rgb(0x00, 0xff, 0x00)
        );
        assert_eq!(
            alert_color("Tornado Watch"),
            Color32::from_rgb(0xff, 0x14, 0x93)
        );
        assert_eq!(
            alert_color("Severe Thunderstorm Watch"),
            Color32::from_rgb(0xff, 0xd7, 0x00)
        );
    }

    #[test]
    #[ignore = "requires network access to the NWS API"]
    fn fetches_active_alerts_from_nws() {
        let alerts = fetch_alerts().expect("fetch should succeed");
        for alert in &alerts {
            assert!(
                is_warning_or_watch(&alert.event),
                "unexpected event type: {}",
                alert.event
            );
        }
    }

    #[test]
    fn generic_warning_is_red_watch_is_yellow() {
        assert_eq!(
            alert_color("Special Weather Warning"),
            Color32::from_rgb(0xff, 0x33, 0x33)
        );
        assert_eq!(
            alert_color("Heat Watch"),
            Color32::from_rgb(0xff, 0xee, 0x00)
        );
    }

    #[test]
    fn keeps_both_endpoints_inside_circle() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 0.0);
        let segs = circle_subsegments(a, b, 20.0);
        assert_eq!(segs.len(), 1);
        assert!((segs[0].0 - a).length() < 1e-4);
        assert!((segs[0].1 - b).length() < 1e-4);
    }

    #[test]
    fn clips_outside_endpoint_to_circle() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(30.0, 0.0);
        let segs = circle_subsegments(a, b, 20.0);
        assert_eq!(segs.len(), 1);
        assert!((segs[0].0 - a).length() < 1e-4);
        assert!((segs[0].1 - Vec2::new(20.0, 0.0)).length() < 1e-4);
    }

    #[test]
    fn both_outside_but_crossing_produces_chord() {
        let a = Vec2::new(-30.0, 0.0);
        let b = Vec2::new(30.0, 0.0);
        let segs = circle_subsegments(a, b, 20.0);
        assert_eq!(segs.len(), 1);
        assert!((segs[0].0 - Vec2::new(-20.0, 0.0)).length() < 1e-4);
        assert!((segs[0].1 - Vec2::new(20.0, 0.0)).length() < 1e-4);
    }

    #[test]
    fn fully_outside_segment_yields_empty() {
        let a = Vec2::new(30.0, 0.0);
        let b = Vec2::new(40.0, 0.0);
        assert!(circle_subsegments(a, b, 20.0).is_empty());
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
