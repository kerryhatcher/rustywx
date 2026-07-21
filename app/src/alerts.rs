//! Active NWS warning/watch polygon overlay for the radarscope.
//!
//! Stage 4: Replaces `ureq` with Ply `net` for fetching. Colors use
//! `[u8; 4]` arrays instead of `egui::Color32`. The fetch is fire-and-poll:
//! `net::get("alerts", URL, |c| c.header("User-Agent", ...))` is called
//! once, then `net::request("alerts")` is polled each frame.

use crate::borders::Ring;
use crate::geo::{self, RadarSite};
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
    /// Full NWS alert body (`properties.description`).
    pub description: String,
    /// Precautionary/preparedness actions (`properties.instruction`); empty if absent.
    pub instruction: String,
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

        let description = properties
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        let instruction = properties
            .get("instruction")
            .and_then(Value::as_str)
            .unwrap_or("")
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
            description,
            instruction,
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

/// Whether an event is a watch. Anything that passed `is_warning_or_watch`
/// and isn't a watch is treated as a warning. "Warning" wins if a name
/// somehow contains both.
pub fn is_watch(event: &str) -> bool {
    let e = event.to_ascii_lowercase();
    e.contains("watch") && !e.contains("warning")
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

/// Rank an event by hazard severity for hit-test priority. Higher wins when
/// polygons overlap, so a click inside both a Tornado Warning and a Severe
/// Thunderstorm Warning opens the tornado. Warnings always outrank watches.
pub fn severity_rank(event: &str) -> u8 {
    let e = event.to_ascii_lowercase();
    if e.contains("tornado emergency") {
        100
    } else if e.contains("tornado warning") {
        90
    } else if e.contains("flash flood emergency") {
        80
    } else if e.contains("severe thunderstorm warning") {
        70
    } else if e.contains("flash flood warning") {
        60
    } else if e.contains("warning") {
        50
    } else {
        // Watches — all below any warning.
        10
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

/// Ray-casting point-in-polygon test against a single ring (screen-space).
///
// ponytail: NWS alert polygons don't carry holes in practice, so a ring hit
// is treated as an unconditional "inside" — no even-odd hole subtraction.
fn point_in_ring(point: (f32, f32), ring: &[(f32, f32)]) -> bool {
    let (px, py) = point;
    let mut inside = false;
    let n = ring.len();
    for i in 0..n {
        let (ax, ay) = ring[i];
        let (bx, by) = ring[(i + 1) % n];
        if (ay > py) != (by > py) {
            let x_at_y = ax + (py - ay) / (by - ay) * (bx - ax);
            if px < x_at_y {
                inside = !inside;
            }
        }
    }
    inside
}

/// Hit-test a click against the currently visible alerts, projecting each
/// alert's rings to screen space the same way `scope::draw_alerts` does.
/// Among all alerts whose polygon contains `click_screen`, returns the most
/// severe (see `severity_rank`) — so a tornado warning wins over a severe
/// thunderstorm warning it overlaps — breaking ties by draw order (later =
/// topmost). Returns `None` if nothing was hit.
pub fn hit_test<'a>(
    alerts: &'a [Alert],
    show_watches: bool,
    show_warnings: bool,
    site: &RadarSite,
    click_screen: (f32, f32),
    center: (f32, f32),
    px_per_km: f32,
) -> Option<&'a Alert> {
    let mut best: Option<&'a Alert> = None;

    for alert in alerts {
        if is_watch(&alert.event) {
            if !show_watches {
                continue;
            }
        } else if !show_warnings {
            continue;
        }

        let hit = alert.rings.iter().any(|ring| {
            if ring.len() < 3 {
                return false;
            }
            let pts_px: Vec<(f32, f32)> = ring
                .iter()
                .map(|&(lat, lon)| {
                    let km = geo::point_to_km_offset(site.lat, site.lon, (lat, lon));
                    (center.0 + km.x * px_per_km, center.1 + km.y * px_per_km)
                })
                .collect();
            point_in_ring(click_screen, &pts_px)
        });

        // Prefer higher severity; `>=` keeps the later (topmost) alert on ties.
        if hit && best.is_none_or(|b| severity_rank(&alert.event) >= severity_rank(&b.event)) {
            best = Some(alert);
        }
    }

    best
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
    fn classifies_watch_vs_warning() {
        assert!(is_watch("Tornado Watch"));
        assert!(is_watch("Severe Thunderstorm Watch"));
        assert!(!is_watch("Tornado Warning"));
        assert!(!is_watch("Flash Flood Warning"));
        // "Warning" wins when a name mixes both.
        assert!(!is_watch("Watch upgraded to Warning"));
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

    #[test]
    fn hit_test_finds_click_inside_ring_and_misses_outside() {
        let site = RadarSite {
            id: "TEST",
            name: "Test",
            lat: 0.0,
            lon: 0.0,
        };
        let alert = Alert {
            event: "Tornado Warning".to_string(),
            headline: "Tornado Warning for...".to_string(),
            description: String::new(),
            instruction: String::new(),
            color: alert_color("Tornado Warning"),
            // A ~111km square (1 degree lat/lon) around the origin site.
            rings: vec![vec![(0.0, 0.0), (0.0, 1.0), (1.0, 1.0), (1.0, 0.0)]],
        };
        let alerts = vec![alert];

        // Site-relative screen projection: north is -y, east is +x, so this
        // ~111km square (1 degree lat/lon) around the origin site lands at
        // roughly x in [0, 111], y in [-111, 0] in screen space.
        let inside = hit_test(&alerts, true, true, &site, (50.0, -50.0), (0.0, 0.0), 1.0);
        assert!(inside.is_some());

        let outside = hit_test(&alerts, true, true, &site, (500.0, 500.0), (0.0, 0.0), 1.0);
        assert!(outside.is_none());

        // Toggled-off category never hits, even inside the polygon.
        let toggled_off = hit_test(&alerts, true, false, &site, (50.0, -50.0), (0.0, 0.0), 1.0);
        assert!(toggled_off.is_none());
    }

    #[test]
    fn hit_test_prefers_more_severe_alert_in_overlap() {
        let site = RadarSite {
            id: "TEST",
            name: "Test",
            lat: 0.0,
            lon: 0.0,
        };
        let square = || vec![vec![(0.0, 0.0), (0.0, 1.0), (1.0, 1.0), (1.0, 0.0)]];
        // Tornado listed FIRST — old "last match" logic would return the
        // severe thunderstorm; severity ranking must still pick the tornado.
        let alerts = vec![
            Alert {
                event: "Tornado Warning".to_string(),
                headline: String::new(),
                description: String::new(),
                instruction: String::new(),
                color: alert_color("Tornado Warning"),
                rings: square(),
            },
            Alert {
                event: "Severe Thunderstorm Warning".to_string(),
                headline: String::new(),
                description: String::new(),
                instruction: String::new(),
                color: alert_color("Severe Thunderstorm Warning"),
                rings: square(),
            },
        ];
        let hit = hit_test(&alerts, true, true, &site, (50.0, -50.0), (0.0, 0.0), 1.0);
        assert_eq!(hit.map(|a| a.event.as_str()), Some("Tornado Warning"));
    }
}
