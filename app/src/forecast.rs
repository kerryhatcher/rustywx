//! Open-Meteo current-conditions + 7-day forecast for the Forecast view.
//!
//! Fire-and-poll over Ply `net`, mirroring `alerts.rs`. Net request IDs embed
//! the coordinates / query string so relocating the forecast fires a fresh
//! request (`net::get` is idempotent per ID). Parsing is pure and unit-tested.

use crate::location::Coords;
use crate::widgets::nf;
use anyhow::{Result, anyhow};
use serde_json::Value;

const FORECAST_BASE: &str = "https://api.open-meteo.com/v1/forecast";
const GEO_BASE: &str = "https://geocoding-api.open-meteo.com/v1/search";
const USER_AGENT: &str = "rustywx/dev (https://github.com/rustywx/rustywx)";

/// Current conditions (°F, mph, %).
#[derive(Clone, Debug, PartialEq)]
pub struct Current {
    pub temp: f64,
    pub feels_like: f64,
    pub humidity: i64,
    pub wind: f64,
    pub code: i64,
    pub is_day: bool,
}

/// One day of the daily outlook.
#[derive(Clone, Debug, PartialEq)]
pub struct Day {
    pub weekday: String,
    pub code: i64,
    pub hi: f64,
    pub lo: f64,
    pub precip_pct: i64,
}

/// A full forecast for one place.
#[derive(Clone, Debug, PartialEq)]
pub struct Forecast {
    pub place: String,
    pub current: Current,
    pub days: Vec<Day>,
}

/// A geocoding search result.
#[derive(Clone, Debug, PartialEq)]
pub struct GeoHit {
    pub label: String,
    pub coords: Coords,
}

/// Net request ID for a forecast at `c` (embeds coords so relocating refetches).
fn forecast_net_id(c: Coords) -> String {
    format!("fc:{:.4},{:.4}", c.lat, c.lon)
}

/// Net request ID for a geocode query (embeds the query).
fn geo_net_id(query: &str) -> String {
    format!("geo:{query}")
}

/// Open-Meteo forecast URL for `c`.
fn forecast_url(c: Coords) -> String {
    format!(
        "{FORECAST_BASE}?latitude={:.4}&longitude={:.4}\
&current=temperature_2m,apparent_temperature,relative_humidity_2m,weather_code,wind_speed_10m,is_day\
&daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max\
&temperature_unit=fahrenheit&wind_speed_unit=mph&timezone=auto&forecast_days=7",
        c.lat, c.lon
    )
}

/// Open-Meteo geocoding URL for `query`.
fn geo_url(query: &str) -> String {
    // Minimal query encoding: spaces → %20 (city names rarely need more).
    let q = query.trim().replace(' ', "%20");
    format!("{GEO_BASE}?name={q}&count=5&language=en&format=json")
}

/// Fire the forecast fetch for `c`. Idempotent per coords.
pub fn fire_forecast(c: Coords) {
    use ply_engine::prelude::net;
    net::get(&forecast_net_id(c), &forecast_url(c), |cfg| {
        cfg.header("User-Agent", USER_AGENT)
    });
}

/// Poll the forecast fetch for `c`. `Some(Ok)` when done, `Some(Err)` on error,
/// `None` while pending. Note: `place` is left empty — the caller fills it.
pub fn poll_forecast(c: Coords) -> Option<Result<Forecast>> {
    use ply_engine::prelude::net;
    let resp = net::request(&forecast_net_id(c))?.response()?;
    match resp {
        Ok(r) => Some(parse_forecast(r.text())),
        Err(e) => Some(Err(anyhow!("fetching forecast: {e}"))),
    }
}

/// Fire the geocode search for `query`. Idempotent per query.
pub fn fire_geo(query: &str) {
    use ply_engine::prelude::net;
    net::get(&geo_net_id(query), &geo_url(query), |cfg| {
        cfg.header("User-Agent", USER_AGENT)
    });
}

/// Poll the geocode search for `query`.
pub fn poll_geo(query: &str) -> Option<Result<Vec<GeoHit>>> {
    use ply_engine::prelude::net;
    let resp = net::request(&geo_net_id(query))?.response()?;
    match resp {
        Ok(r) => Some(parse_geo(r.text())),
        Err(e) => Some(Err(anyhow!("geocoding: {e}"))),
    }
}

/// "YYYY-MM-DD" → short weekday ("Mon"). Empty string if unparseable.
pub fn weekday_from_iso(date: &str) -> String {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| d.format("%a").to_string())
        .unwrap_or_default()
}

/// Map a WMO weather code to a (glyph, label) pair. `is_day` only affects the
/// clear-sky code (sun vs. moon). Codes per Open-Meteo's WMO table.
pub fn wmo_icon(code: i64, is_day: bool) -> (&'static str, &'static str) {
    match code {
        0 => {
            if is_day {
                (nf::WX_SUNNY, "Clear")
            } else {
                (nf::WX_NIGHT, "Clear")
            }
        }
        1 | 2 => (nf::WX_PARTLY, "Partly cloudy"),
        3 => (nf::WX_CLOUDY, "Overcast"),
        45 | 48 => (nf::WX_FOG, "Fog"),
        51 | 53 | 55 | 56 | 57 => (nf::WX_RAINY, "Drizzle"),
        61 | 63 | 66 => (nf::WX_RAINY, "Rain"),
        65 | 67 => (nf::WX_POURING, "Heavy rain"),
        71 | 73 | 75 | 77 => (nf::WX_SNOWY, "Snow"),
        80 | 81 => (nf::WX_RAINY, "Showers"),
        82 => (nf::WX_POURING, "Heavy showers"),
        85 | 86 => (nf::WX_SNOWY, "Snow showers"),
        95 | 96 | 99 => (nf::WX_LIGHTNING, "Thunderstorm"),
        _ => (nf::WX_CLOUDY, "—"),
    }
}

/// Parse an Open-Meteo forecast response. `place` is set to "" (caller fills).
pub fn parse_forecast(json: &str) -> Result<Forecast> {
    let root: Value = serde_json::from_str(json).map_err(|e| anyhow!("parsing forecast: {e}"))?;

    let cur = root
        .get("current")
        .ok_or_else(|| anyhow!("forecast JSON has no \"current\""))?;
    let f = |v: &Value, k: &str| v.get(k).and_then(Value::as_f64).unwrap_or(0.0);
    let i = |v: &Value, k: &str| v.get(k).and_then(Value::as_i64).unwrap_or(0);
    let current = Current {
        temp: f(cur, "temperature_2m"),
        feels_like: f(cur, "apparent_temperature"),
        humidity: i(cur, "relative_humidity_2m"),
        wind: f(cur, "wind_speed_10m"),
        code: i(cur, "weather_code"),
        is_day: i(cur, "is_day") == 1,
    };

    let daily = root
        .get("daily")
        .ok_or_else(|| anyhow!("forecast JSON has no \"daily\""))?;
    let arr = |k: &str| daily.get(k).and_then(Value::as_array).cloned().unwrap_or_default();
    let dates = arr("time");
    let codes = arr("weather_code");
    let his = arr("temperature_2m_max");
    let los = arr("temperature_2m_min");
    let precs = arr("precipitation_probability_max");

    let mut days = Vec::new();
    for (idx, date) in dates.iter().enumerate() {
        let date_str = date.as_str().unwrap_or("");
        days.push(Day {
            weekday: weekday_from_iso(date_str),
            code: codes.get(idx).and_then(Value::as_i64).unwrap_or(0),
            hi: his.get(idx).and_then(Value::as_f64).unwrap_or(0.0),
            lo: los.get(idx).and_then(Value::as_f64).unwrap_or(0.0),
            precip_pct: precs.get(idx).and_then(Value::as_i64).unwrap_or(0),
        });
    }

    Ok(Forecast { place: String::new(), current, days })
}

/// Parse an Open-Meteo geocoding response into up to 5 hits.
pub fn parse_geo(json: &str) -> Result<Vec<GeoHit>> {
    let root: Value = serde_json::from_str(json).map_err(|e| anyhow!("parsing geocode: {e}"))?;
    // Open-Meteo returns `{}` (no "results") when there are no matches.
    let results = match root.get("results").and_then(Value::as_array) {
        Some(r) => r,
        None => return Ok(Vec::new()),
    };
    let mut hits = Vec::new();
    for r in results {
        let name = r.get("name").and_then(Value::as_str).unwrap_or("");
        let admin = r.get("admin1").and_then(Value::as_str).unwrap_or("");
        let country = r.get("country").and_then(Value::as_str).unwrap_or("");
        let lat = r.get("latitude").and_then(Value::as_f64);
        let lon = r.get("longitude").and_then(Value::as_f64);
        if let (Some(lat), Some(lon)) = (lat, lon) {
            let label = [name, admin, country]
                .iter()
                .filter(|s| !s.is_empty())
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            hits.push(GeoHit { label, coords: Coords { lat, lon } });
        }
    }
    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FORECAST_FIXTURE: &str = r#"{
      "current": {
        "temperature_2m": 72.4,
        "apparent_temperature": 74.1,
        "relative_humidity_2m": 55,
        "wind_speed_10m": 8.3,
        "weather_code": 2,
        "is_day": 1
      },
      "daily": {
        "time": ["2026-07-21", "2026-07-22", "2026-07-23"],
        "weather_code": [2, 61, 95],
        "temperature_2m_max": [75.0, 73.2, 68.9],
        "temperature_2m_min": [55.1, 54.0, 60.3],
        "precipitation_probability_max": [10, 60, 80]
      }
    }"#;

    const GEO_FIXTURE: &str = r#"{
      "results": [
        {"name": "Atlanta", "admin1": "Georgia", "country": "United States",
         "latitude": 33.749, "longitude": -84.388},
        {"name": "Atlanta", "admin1": "Texas", "country": "United States",
         "latitude": 33.113, "longitude": -94.164}
      ]
    }"#;

    #[test]
    fn parse_forecast_reads_current_and_days() {
        let f = parse_forecast(FORECAST_FIXTURE).unwrap();
        assert_eq!(f.current.temp, 72.4);
        assert_eq!(f.current.feels_like, 74.1);
        assert_eq!(f.current.humidity, 55);
        assert_eq!(f.current.code, 2);
        assert!(f.current.is_day);
        assert_eq!(f.days.len(), 3);
        assert_eq!(f.days[1].code, 61);
        assert_eq!(f.days[1].hi, 73.2);
        assert_eq!(f.days[1].lo, 54.0);
        assert_eq!(f.days[1].precip_pct, 60);
        assert_eq!(f.place, "");
    }

    #[test]
    fn parse_geo_composes_labels_and_coords() {
        let hits = parse_geo(GEO_FIXTURE).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].label, "Atlanta, Georgia, United States");
        assert_eq!(hits[0].coords.lat, 33.749);
    }

    #[test]
    fn parse_geo_no_results_is_empty() {
        assert!(parse_geo("{}").unwrap().is_empty());
    }

    #[test]
    fn weekday_from_iso_maps_known_date() {
        assert_eq!(weekday_from_iso("2026-07-21"), "Tue"); // 2026-07-21 is a Tuesday
        assert_eq!(weekday_from_iso("garbage"), "");
    }

    #[test]
    fn wmo_icon_covers_buckets() {
        assert_eq!(wmo_icon(0, true).1, "Clear");
        assert_eq!(wmo_icon(0, false).0, nf::WX_NIGHT);
        assert_eq!(wmo_icon(3, true).1, "Overcast");
        assert_eq!(wmo_icon(65, true).1, "Heavy rain");
        assert_eq!(wmo_icon(71, true).1, "Snow");
        assert_eq!(wmo_icon(95, true).1, "Thunderstorm");
    }
}
