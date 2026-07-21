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

/// One hour of the rain-chance outlook.
#[derive(Clone, Debug, PartialEq)]
pub struct Hour {
    /// Short 12-hour label, e.g. "4p".
    pub label: String,
    /// Calendar date "YYYY-MM-DD" (for day grouping / delimiters).
    pub date: String,
    /// Precipitation probability (%).
    pub precip_pct: i64,
}

/// A full forecast for one place.
#[derive(Clone, Debug, PartialEq)]
pub struct Forecast {
    pub place: String,
    pub current: Current,
    pub days: Vec<Day>,
    /// Next 24 hours of rain chance, starting at the current hour.
    pub hours: Vec<Hour>,
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
&hourly=precipitation_probability\
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

/// "YYYY-MM-DDTHH:MM" → short 12-hour label ("4p", "12a"). Reads the hour
/// field positionally; falls back to "12a" if it can't be parsed.
pub fn hour_label(iso: &str) -> String {
    let h: i64 = iso.get(11..13).and_then(|s| s.parse().ok()).unwrap_or(0);
    let period = if h < 12 { "a" } else { "p" };
    let h12 = match h % 12 {
        0 => 12,
        n => n,
    };
    format!("{h12}{period}")
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

    // Hourly rain chance: next 24 hours starting at the hour containing "now"
    // (matched against `current.time`; falls back to the start of the array).
    let mut hours = Vec::new();
    if let Some(hourly) = root.get("hourly") {
        let times = hourly
            .get("time")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let probs = hourly
            .get("precipitation_probability")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let now_hour = cur.get("time").and_then(Value::as_str).unwrap_or("");
        // ISO-8601 hour prefixes ("YYYY-MM-DDTHH") sort lexically, so the first
        // hourly slot whose prefix is >= now's prefix is the current hour.
        let start = now_hour
            .get(..13)
            .and_then(|nh| {
                times
                    .iter()
                    .position(|t| t.as_str().and_then(|s| s.get(..13)).is_some_and(|h| h >= nh))
            })
            .unwrap_or(0);
        for idx in start..(start + 24).min(times.len()) {
            let ts = times[idx].as_str().unwrap_or("");
            hours.push(Hour {
                label: hour_label(ts),
                date: ts.get(..10).unwrap_or("").to_string(),
                precip_pct: probs.get(idx).and_then(Value::as_i64).unwrap_or(0),
            });
        }
    }

    Ok(Forecast { place: String::new(), current, days, hours })
}

/// Rasterize the hourly rain-chance line graph into a straight-RGBA8 buffer
/// (`w*h*4` bytes, top-left origin, transparent background). Each hour is a
/// dot at x = evenly spaced, y = height by precip % (0% bottom, 100% top);
/// consecutive dots are joined by a line. Faint gridlines mark 0/50/100%.
///
/// Pure (no GPU/ply) so it is unit-testable; the caller wraps the bytes in a
/// `Texture2D` for display. Colors match the forecast view (line `0x6F9FE0`).
pub fn render_hourly_chart(hours: &[Hour], w: usize, h: usize) -> Vec<u8> {
    // Opaque panel-colored background — ply's `.image` does not alpha-composite
    // a transparent texture, so bake the background in (matches the opaque-PNG
    // path NHC thumbnails use).
    const BG: [u8; 4] = [0x1E, 0x1B, 0x1B, 0xFF];
    let mut buf = vec![0u8; w * h * 4];
    for px in buf.chunks_exact_mut(4) {
        px.copy_from_slice(&BG);
    }
    if hours.is_empty() || w < 8 || h < 8 {
        return buf;
    }

    const LINE: [u8; 4] = [0x8F, 0xC0, 0xFF, 0xFF]; // bright blue line/dots
    const FILL: [u8; 4] = [0x6F, 0x9F, 0xE0, 0x55]; // translucent area under line
    const GRID: [u8; 4] = [0xFF, 0xFF, 0xFF, 0x22]; // faint horizontal quartiles
    const DAY: [u8; 4] = [0x0d, 0xc5, 0xb8, 0x66]; // teal day-boundary verticals
    let margin = 8usize;
    let plot_w = w.saturating_sub(2 * margin).max(1);
    let plot_h = h.saturating_sub(2 * margin).max(1);

    // Alpha-blend a pixel (src over dst) so translucent fill/grid layer visibly.
    let blend = |buf: &mut [u8], x: i64, y: i64, c: [u8; 4]| {
        if x < 0 || y < 0 || x as usize >= w || y as usize >= h {
            return;
        }
        let idx = ((y as usize) * w + x as usize) * 4;
        let a = c[3] as u32;
        if a == 0 {
            return;
        }
        if a == 255 {
            buf[idx..idx + 4].copy_from_slice(&c);
            return;
        }
        for k in 0..3 {
            let src = c[k] as u32;
            let dst = buf[idx + k] as u32;
            buf[idx + k] = ((src * a + dst * (255 - a)) / 255) as u8;
        }
        buf[idx + 3] = (a + buf[idx + 3] as u32 * (255 - a) / 255) as u8;
    };

    let n = hours.len();
    let px = |i: usize| -> i64 {
        (margin + if n > 1 { i * plot_w / (n - 1) } else { plot_w / 2 }) as i64
    };
    let py = |pct: i64| -> i64 {
        let p = pct.clamp(0, 100) as usize;
        (margin + plot_h - p * plot_h / 100) as i64
    };
    let baseline = py(0);

    // Horizontal gridlines at 0/25/50/75/100% for vertical scale context.
    for pct in [0i64, 25, 50, 75, 100] {
        let y = py(pct);
        for x in margin..(w - margin) {
            blend(&mut buf, x as i64, y, GRID);
        }
    }

    // Vertical day-boundary delimiters (where the date changes).
    for i in 1..n {
        if hours[i].date != hours[i - 1].date {
            let x = px(i);
            for y in margin..(h - margin) {
                blend(&mut buf, x, y as i64, DAY);
            }
        }
    }

    // Area fill + line, segment by segment (integer DDA).
    for i in 0..n.saturating_sub(1) {
        let (x0, y0) = (px(i), py(hours[i].precip_pct));
        let (x1, y1) = (px(i + 1), py(hours[i + 1].precip_pct));
        let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
        for s in 0..=steps {
            let x = x0 + (x1 - x0) * s / steps;
            let y = y0 + (y1 - y0) * s / steps;
            // Fill column from the curve down to the baseline.
            let (top, bot) = if y <= baseline { (y, baseline) } else { (baseline, y) };
            for fy in top..=bot {
                blend(&mut buf, x, fy, FILL);
            }
            blend(&mut buf, x, y, LINE);
            blend(&mut buf, x, y + 1, LINE); // 2px thick
        }
    }

    // Dots per hour.
    for i in 0..n {
        let (cx, cy) = (px(i), py(hours[i].precip_pct));
        for dy in -2..=2i64 {
            for dx in -2..=2i64 {
                if dx * dx + dy * dy <= 4 {
                    blend(&mut buf, cx + dx, cy + dy, LINE);
                }
            }
        }
    }

    buf
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
        "time": "2026-07-21T16:45",
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
      },
      "hourly": {
        "time": ["2026-07-21T15:00", "2026-07-21T16:00", "2026-07-21T17:00", "2026-07-21T18:00"],
        "precipitation_probability": [5, 20, 45, 70]
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
    fn parse_forecast_reads_hourly_from_current_hour() {
        let f = parse_forecast(FORECAST_FIXTURE).unwrap();
        // current.time is 16:45, so the window starts at the 16:00 slot,
        // skipping the earlier 15:00 slot.
        assert_eq!(f.hours.len(), 3);
        assert_eq!(f.hours[0].label, "4p");
        assert_eq!(f.hours[0].precip_pct, 20);
        assert_eq!(f.hours[2].label, "6p");
        assert_eq!(f.hours[2].precip_pct, 70);
    }

    #[test]
    fn render_hourly_chart_dims_and_plots() {
        let hours = vec![
            Hour { label: "4p".into(), date: "2026-07-21".into(), precip_pct: 0 },
            Hour { label: "5p".into(), date: "2026-07-21".into(), precip_pct: 100 },
        ];
        let (w, h) = (120, 60);
        let buf = render_hourly_chart(&hours, w, h);
        assert_eq!(buf.len(), w * h * 4);
        // The 100% hour must paint the bright-blue line near the top; scan the
        // top rows for a strongly-blue pixel (distinct from the dark bg).
        let is_line = |i: usize| buf[i * 4] > 0x60 && buf[i * 4 + 1] > 0x90 && buf[i * 4 + 2] > 0xD0;
        let top_has_line = (0..h / 3).any(|y| (0..w).any(|x| is_line(y * w + x)));
        assert!(top_has_line, "100% hour should ink the line near the top");
    }

    #[test]
    fn render_hourly_chart_empty_is_opaque_background() {
        // Empty input → an opaque panel-colored buffer (ply won't composite a
        // transparent texture), no line pixels.
        let buf = render_hourly_chart(&[], 120, 60);
        assert_eq!(buf.len(), 120 * 60 * 4);
        assert!(buf.chunks_exact(4).all(|p| p == [0x1E, 0x1B, 0x1B, 0xFF]));
    }

    #[test]
    fn hour_label_formats_12h() {
        assert_eq!(hour_label("2026-07-21T16:00"), "4p");
        assert_eq!(hour_label("2026-07-21T00:00"), "12a");
        assert_eq!(hour_label("2026-07-21T12:00"), "12p");
        assert_eq!(hour_label("2026-07-21T09:00"), "9a");
        assert_eq!(hour_label("garbage"), "12a");
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
