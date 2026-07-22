# Forecast View Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a full-screen "Forecast" view mode showing current conditions and a 7-day outlook for the app's location or an in-view searched city, using Open-Meteo.

**Architecture:** A new `forecast.rs` module fetches Open-Meteo over `ply_engine::prelude::net` (fire-and-poll, same pattern as `alerts.rs`) with pure, unit-tested parse functions. A `ViewMode` enum in `AppState` gates rendering: in `Forecast` mode the radar scope/sweep draw is skipped and a full-area forecast UI element renders under the existing top bar. Net request IDs embed the coordinates/query so a new location mints a fresh request (ply's `net::get` is idempotent per ID).

**Tech Stack:** Rust, macroquad/miniquad, `ply-engine` 1.1.1 (UI + `net` HTTP), `serde_json`, `chrono`, `anyhow`. No new dependencies.

## Global Constraints

- No new crate dependencies — reuse `ply net`, `serde_json`, `chrono`, `anyhow`, `crate::location::Coords`.
- Units hardcoded: temperature °F, wind mph. No unit toggle in v1.
- US-agnostic: Open-Meteo is global; do not assume US.
- View mode is not persisted; app launches in `Radar`.
- Follow existing patterns: module registered in `app/src/lib.rs`; imported in `main.rs` as `use rustywx::forecast;`. Fetch code mirrors `app/src/alerts.rs`. UI code mirrors the top-bar / glass-panel idioms already in `main.rs`.
- Net request IDs must embed coordinates/query (never a fixed string) so relocating refetches. Format: `fc:{lat:.4},{lon:.4}` and `geo:{query}`.
- All new public parse/helper functions carry unit tests in a `#[cfg(test)] mod tests` block, matching `alerts.rs` test style (no framework, plain `#[test]` + `assert!`).
- Run the full test suite with `cargo test -p rustywx` and build with `cargo build -p rustywx` from the repo root (`/Users/kerry.hatcher/projects/rustywx`).

---

## File Structure

- **Create** `app/src/forecast.rs` — Open-Meteo data layer: types, URL builders, net IDs, fire/poll functions, parse functions, WMO code→glyph/label mapping, weekday helper, unit tests.
- **Modify** `app/src/lib.rs` — register `pub mod forecast;`.
- **Modify** `app/src/widgets/mod.rs` — add `md-weather-*` glyph constants to the `nf` module.
- **Modify** `app/src/state.rs` — add `ViewMode` enum + forecast/search `AppState` fields.
- **Modify** `app/src/main.rs` — import `forecast`; init new state fields; gate scope/sweep draw on view mode; add the top-bar Forecast button + mode-toggle click wiring; render the forecast view element; per-frame fetch/poll + search input handling.

---

## Task 1: Forecast data layer (`forecast.rs`)

**Files:**
- Create: `app/src/forecast.rs`
- Modify: `app/src/lib.rs` (add `pub mod forecast;` in alphabetical position — between `pub mod data;` and `pub mod geo;`)
- Modify: `app/src/widgets/mod.rs` (add weather glyph constants to `nf`)

**Interfaces:**
- Consumes: `crate::location::Coords { pub lat: f64, pub lon: f64 }` (existing, `Clone + Copy`).
- Produces (later tasks rely on these exact signatures):
  ```rust
  pub struct Forecast { pub place: String, pub current: Current, pub days: Vec<Day> }
  pub struct Current { pub temp: f64, pub feels_like: f64, pub humidity: i64, pub wind: f64, pub code: i64, pub is_day: bool }
  pub struct Day { pub weekday: String, pub code: i64, pub hi: f64, pub lo: f64, pub precip_pct: i64 }
  pub struct GeoHit { pub label: String, pub coords: crate::location::Coords }

  pub fn fire_forecast(c: crate::location::Coords);
  pub fn poll_forecast(c: crate::location::Coords) -> Option<anyhow::Result<Forecast>>;
  pub fn fire_geo(query: &str);
  pub fn poll_geo(query: &str) -> Option<anyhow::Result<Vec<GeoHit>>>;
  pub fn wmo_icon(code: i64, is_day: bool) -> (&'static str, &'static str); // (glyph, label)
  pub fn parse_forecast(json: &str) -> anyhow::Result<Forecast>;
  pub fn parse_geo(json: &str) -> anyhow::Result<Vec<GeoHit>>;
  pub fn weekday_from_iso(date: &str) -> String;
  ```

- [ ] **Step 1: Register the module and add weather glyph constants**

In `app/src/lib.rs`, add between `pub mod data;` and `pub mod geo;`:
```rust
pub mod forecast;
```

In `app/src/widgets/mod.rs`, inside the `nf` module (after the `HURRICANE` constant), add:
```rust
    // Weather glyphs (Material Design Icons — same set as HURRICANE).
    // Verify rendering against the SYMBOL_FONT; the paired text label in
    // `forecast::wmo_icon` carries meaning even if a glyph shows as tofu.
    pub const WX_SUNNY: &str = "\u{f0599}"; // md-weather-sunny
    pub const WX_NIGHT: &str = "\u{f0594}"; // md-weather-night
    pub const WX_PARTLY: &str = "\u{f0595}"; // md-weather-partly-cloudy
    pub const WX_CLOUDY: &str = "\u{f0590}"; // md-weather-cloudy
    pub const WX_FOG: &str = "\u{f0591}"; // md-weather-fog
    pub const WX_RAINY: &str = "\u{f0597}"; // md-weather-rainy
    pub const WX_POURING: &str = "\u{f0596}"; // md-weather-pouring
    pub const WX_SNOWY: &str = "\u{f0598}"; // md-weather-snowy
    pub const WX_LIGHTNING: &str = "\u{f0593}"; // md-weather-lightning
```

- [ ] **Step 2: Write `forecast.rs` with types, helpers, and failing tests**

Create `app/src/forecast.rs`:
```rust
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
```

- [ ] **Step 3: Run the tests to verify they pass**

Run: `cargo test -p rustywx forecast`
Expected: PASS — `parse_forecast_reads_current_and_days`, `parse_geo_composes_labels_and_coords`, `parse_geo_no_results_is_empty`, `weekday_from_iso_maps_known_date`, `wmo_icon_covers_buckets` all green.

If `weekday_from_iso_maps_known_date` fails, correct the expected weekday to match what chrono returns for `2026-07-21` and re-run (this is a fixed calendar fact; adjust the assertion, not the code).

- [ ] **Step 4: Build to confirm no warnings break the crate**

Run: `cargo build -p rustywx`
Expected: compiles. `fire_forecast` / `poll_forecast` / `fire_geo` / `poll_geo` / `wmo_icon` will be flagged dead-code (unused) until Task 4 — acceptable for now; do NOT add `#[allow(dead_code)]` (they get used in Task 4).

- [ ] **Step 5: Commit**

```bash
git add app/src/forecast.rs app/src/lib.rs app/src/widgets/mod.rs
git commit -m "feat: forecast data layer (Open-Meteo fetch + parse)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: View mode + state fields

**Files:**
- Modify: `app/src/state.rs` (add `ViewMode` enum near the top, after imports; add fields to `AppState`)
- Modify: `app/src/main.rs` (add `use rustywx::forecast;` and `use rustywx::state::ViewMode;`; init new fields in the `AppState { … }` constructor at ~line 422)

**Interfaces:**
- Consumes: `forecast::{Forecast, GeoHit}` from Task 1.
- Produces: `state.view_mode`, `state.forecast`, `state.forecast_coords`, `state.forecast_fetch_fired`, `state.forecast_error`, `state.forecast_place`, `state.fc_search_text`, `state.fc_search_focused`, `state.fc_geo_hits`, `state.fc_geo_fired` — read/written by Tasks 3 and 4.

- [ ] **Step 1: Add the `ViewMode` enum to `state.rs`**

In `app/src/state.rs`, after the `use` block (before `pub enum NhcModal`), add:
```rust
/// Which top-level screen is showing. Radar is the map scope; Forecast is the
/// full-screen non-map current-conditions + 7-day view.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum ViewMode {
    #[default]
    Radar,
    Forecast,
}
```

- [ ] **Step 2: Add the import for the forecast types in `state.rs`**

At the top of `app/src/state.rs`, with the other `use crate::...` lines, add:
```rust
use crate::forecast::{Forecast, GeoHit};
```

- [ ] **Step 3: Add the new fields to `AppState`**

In `app/src/state.rs`, inside `pub struct AppState`, after the `pub toast: Option<Toast>,` field (the last field), add:
```rust
    // ── Forecast view ─────────────────────────────────────────────
    /// Which top-level screen is showing (Radar scope or Forecast).
    pub view_mode: ViewMode,
    /// Latest fetched forecast, if any.
    pub forecast: Option<Forecast>,
    /// Coords the current `forecast` was fetched for (None until first fetch).
    pub forecast_coords: Option<crate::location::Coords>,
    /// Whether a forecast fetch is in flight for the current target coords.
    pub forecast_fetch_fired: bool,
    /// Last forecast fetch/parse error, shown inline in the view.
    pub forecast_error: Option<String>,
    /// Display label for the forecast location (from search pick or settings).
    pub forecast_place: String,
    /// Forecast-view search box text.
    pub fc_search_text: String,
    /// Whether the forecast search box is capturing keystrokes.
    pub fc_search_focused: bool,
    /// Geocode results for the current search.
    pub fc_geo_hits: Vec<GeoHit>,
    /// Whether a geocode search is in flight.
    pub fc_geo_fired: bool,
```

- [ ] **Step 4: Import the new types in `main.rs`**

In `app/src/main.rs`, with the other `use rustywx::...` lines (~line 8-25), add:
```rust
use rustywx::forecast;
use rustywx::state::ViewMode;
```
(Adjust the existing `use rustywx::state::{AlertModal, AppState, NhcModal};` line — leave it as is; add the `ViewMode` import as a separate line to keep the diff small.)

- [ ] **Step 5: Initialize the new fields in the `AppState { … }` constructor**

In `app/src/main.rs`, find the `let mut state = AppState { … };` block (~line 422). After the `toast:` initializer (the last field, near line 500), add:
```rust
        view_mode: ViewMode::Radar,
        forecast: None,
        forecast_coords: None,
        forecast_fetch_fired: false,
        forecast_error: None,
        forecast_place: String::new(),
        fc_search_text: String::new(),
        fc_search_focused: false,
        fc_geo_hits: Vec::new(),
        fc_geo_fired: false,
```

- [ ] **Step 6: Build to verify state compiles**

Run: `cargo build -p rustywx`
Expected: compiles. Existing forecast functions still dead-code-warn (used in Task 4).

- [ ] **Step 7: Commit**

```bash
git add app/src/state.rs app/src/main.rs
git commit -m "feat: ViewMode enum and forecast view state fields

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Top-bar Forecast button + scope-draw gating

**Files:**
- Modify: `app/src/main.rs` — add the Forecast top-bar button (Panels group, after the Tropical panel button block ~line 1009); gate the scope + sweep draw (~line 855-885) on `ViewMode::Radar`; wire the click handlers (radar/tropical/forecast buttons) to set `view_mode`.

**Interfaces:**
- Consumes: `state.view_mode` (Task 2), `state.hovered_ids`, `hover_tint(...)`, `SYMBOL_FONT`, the top-bar element idioms already in `main.rs`.
- Produces: a clickable `"btn-forecast"` element and `view_mode` transitions relied on by Task 4's render gate.

- [ ] **Step 1: Gate the radar scope + sweep draw on Radar mode**

In `app/src/main.rs`, wrap the `scope::draw_scope_to_texture(...)` call and the `if state.settings.show_sweep { draw_radar_sweep(...) }` block (~line 856-885) so they only run in Radar mode. `draw_observatory_background()` stays unconditional (it's the backdrop for both views). Change:
```rust
        draw_observatory_background();
        scope::draw_scope_to_texture(
            // … existing args …
        );

        // Radar sweep line …
        if state.settings.show_sweep {
            draw_radar_sweep(state.pan_km, state.zoom, state.sweep_angle, entrance);
        }
```
to:
```rust
        draw_observatory_background();
        if state.view_mode == ViewMode::Radar {
            scope::draw_scope_to_texture(
                // … existing args unchanged …
            );

            // Radar sweep line …
            if state.settings.show_sweep {
                draw_radar_sweep(state.pan_km, state.zoom, state.sweep_angle, entrance);
            }
        }
```
(Keep the existing argument list to `draw_scope_to_texture` verbatim; only add the `if` wrapper.)

- [ ] **Step 2: Add the Forecast button to the Panels group**

In `app/src/main.rs`, immediately after the Tropical panel button element block ends (the `ui.element().id("btn-tropical")…children(|ui| { … });` block, ~line 1009) and before the "Divider between Panels and Layers groups" element (~line 1011), add:
```rust
                        let forecast_active = state.view_mode == ViewMode::Forecast;
                        let forecast_bg = hover_tint(
                            &state.hovered_ids,
                            "btn-forecast",
                            if forecast_active { 0x0dc5b8 } else { 0x1E1B1B },
                            0x1E1B1B,
                        );
                        let forecast_label = if forecast_active {
                            "Forecast ✓"
                        } else {
                            "Forecast"
                        };
                        ui.element()
                            .id("btn-forecast")
                            .width(fit!())
                            .height(fixed!(if is_mobile { 44.0 } else { 24.0 }))
                            .background_color(forecast_bg)
                            .corner_radius(4.0)
                            .layout(|layout| {
                                layout
                                    .direction(LeftToRight)
                                    .gap(6)
                                    .padding((0, 8, 0, 8))
                                    .align(CenterX, CenterY)
                            })
                            .accessibility(|a| {
                                a.button(forecast_label).checked(forecast_active)
                            })
                            .children(|ui| {
                                ui.text(forecast_label, |text| {
                                    text.font_size(12).color(0xE8E0DC)
                                });
                            });
```

- [ ] **Step 3: Wire the Forecast button click and mode-exclusivity**

In `app/src/main.rs`, find the radar toggle button click handler (`if ply.is_just_pressed("btn-radar") { … }` — near the "Radar toggle button" comment ~line 2498-2512) and the tropical panel toggle handler (~line 2514+). Add a Forecast click handler and make Radar/Tropical clicks leave Forecast mode.

Add, near the other top-bar button handlers (after the tropical panel toggle handler block):
```rust
        // ── Forecast view toggle ──────────────────────────────────
        if ply.is_just_pressed("btn-forecast") {
            state.view_mode = if state.view_mode == ViewMode::Forecast {
                ViewMode::Radar
            } else {
                // Entering forecast: close the scope side panels (exclusive).
                state.radar_panel_open = false;
                state.nhc_show_panel = false;
                ViewMode::Forecast
            };
        }
```

In the existing `btn-radar` handler (the block that sets `state.radar_panel_open = !state.radar_panel_open;`), add as its first line inside the `if`:
```rust
            state.view_mode = ViewMode::Radar;
```
In the existing `btn-tropical` handler (the block that toggles `nhc_show_panel` / sets `radar_panel_open = false`), add as its first line inside the `if`:
```rust
            state.view_mode = ViewMode::Radar;
```

- [ ] **Step 4: Build and manually verify the toggle**

Run: `cargo build -p rustywx`
Expected: compiles.

Run: `cargo run -p rustywx` (or the project `run` skill). Verify:
- A "Forecast" button appears in the top bar after "Tropical" (Panels group).
- Clicking it hides the radar scope (blank observatory background — no forecast UI yet, that's Task 4) and shows the ✓/teal active state.
- Clicking "Radar" or "Tropical" returns to the scope.

- [ ] **Step 5: Commit**

```bash
git add app/src/main.rs
git commit -m "feat: Forecast top-bar button and view-mode scope gating

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Forecast view render + data flow + search

**Files:**
- Modify: `app/src/main.rs` — render the forecast UI element (in the content column, after the top-bar glass panel block, before the `if state.radar_panel_open` block ~line 1281), gated on `ViewMode::Forecast`; add per-frame fetch/poll + search input handling (near the other `ply.is_just_pressed` / keyboard blocks, ~line 2585+).

**Interfaces:**
- Consumes: `forecast::{fire_forecast, poll_forecast, fire_geo, poll_geo, wmo_icon, Forecast, GeoHit}` (Task 1); `state` forecast fields (Task 2); `state.user_location` and `state.settings.location_input` (existing); `get_char_pressed()`, `is_key_pressed(KeyCode::…)`, `ply.is_just_pressed(id)` (existing input helpers); glass-panel / `ui.text` idioms.
- Produces: the complete Forecast view (terminal deliverable).

- [ ] **Step 1: Render the forecast view element**

In `app/src/main.rs`, inside the content column's `.children(|ui| { … })`, after the top-bar glass panel block closes (~line 1279, the `});` ending the controls-bar `.children`) and before the `// ── Radar slide-in panel ──` / `if state.radar_panel_open {` block (~line 1281), insert:
```rust
                // ── Forecast full-screen view ───────────────────────────
                if state.view_mode == ViewMode::Forecast {
                    glass_panel::glass(ui.element().width(grow!()).height(grow!()))
                        .layout(|layout| {
                            layout
                                .direction(TopToBottom)
                                .padding(24)
                                .gap(20)
                                .align(CenterX, Top)
                        })
                        .children(|ui| {
                            // Search row.
                            let search_bg = hover_tint(
                                &state.hovered_ids,
                                "fc-search",
                                if state.fc_search_focused { 0x2A2727 } else { 0x1E1B1B },
                                0x2A2727,
                            );
                            let search_display = if state.fc_search_text.is_empty() {
                                "Search a city…".to_string()
                            } else {
                                state.fc_search_text.clone()
                            };
                            ui.element()
                                .id("fc-search")
                                .width(fixed!(360.0))
                                .height(fixed!(32.0))
                                .background_color(search_bg)
                                .corner_radius(6.0)
                                .layout(|layout| {
                                    layout.padding((0, 10, 0, 10)).align(Left, CenterY)
                                })
                                .accessibility(|a| a.text_input(&search_display))
                                .children(|ui| {
                                    ui.text(&search_display, |t| t.font_size(14).color(0xE8E0DC));
                                });

                            // Geocode results dropdown (if any).
                            for (idx, hit) in state.fc_geo_hits.iter().enumerate() {
                                let hit_id = format!("fc-hit-{idx}");
                                let hit_bg = hover_tint(
                                    &state.hovered_ids,
                                    &hit_id,
                                    0x1E1B1B,
                                    0x2A2727,
                                );
                                ui.element()
                                    .id(&hit_id)
                                    .width(fixed!(360.0))
                                    .height(fixed!(26.0))
                                    .background_color(hit_bg)
                                    .corner_radius(4.0)
                                    .layout(|layout| layout.padding((0, 10, 0, 10)).align(Left, CenterY))
                                    .accessibility(|a| a.button(&hit.label))
                                    .children(|ui| {
                                        ui.text(&hit.label, |t| t.font_size(13).color(0xC8C0BC));
                                    });
                            }

                            // Body: current conditions + 7-day strip, or a status line.
                            if let Some(err) = &state.forecast_error {
                                ui.text(&format!("Couldn't load forecast: {err}"), |t| {
                                    t.font_size(14).color(0xE08080)
                                });
                            } else if let Some(fc) = &state.forecast {
                                // Current conditions.
                                let (glyph, label) = forecast::wmo_icon(fc.current.code, fc.current.is_day);
                                ui.text(&fc.place, |t| t.font_size(20).color(0xE8E0DC));
                                ui.element()
                                    .width(fit!())
                                    .height(fit!())
                                    .layout(|layout| layout.direction(LeftToRight).gap(16).align(CenterX, CenterY))
                                    .children(|ui| {
                                        ui.text(glyph, |t| t.font_size(48).font(&SYMBOL_FONT).color(0xE8E0DC));
                                        ui.text(&format!("{:.0}°F", fc.current.temp), |t| {
                                            t.font_size(48).color(0xE8E0DC)
                                        });
                                    });
                                ui.text(
                                    &format!(
                                        "{label}   Feels {:.0}°   Wind {:.0} mph   Humidity {}%",
                                        fc.current.feels_like, fc.current.wind, fc.current.humidity
                                    ),
                                    |t| t.font_size(14).color(0xC8C0BC),
                                );

                                // 7-day strip.
                                ui.element()
                                    .width(fit!())
                                    .height(fit!())
                                    .layout(|layout| layout.direction(LeftToRight).gap(12).align(CenterX, Top))
                                    .children(|ui| {
                                        for day in &fc.days {
                                            let (dglyph, _dlabel) = forecast::wmo_icon(day.code, true);
                                            ui.element()
                                                .width(fixed!(72.0))
                                                .height(fit!())
                                                .background_color(0x1E1B1B)
                                                .corner_radius(6.0)
                                                .layout(|layout| {
                                                    layout.direction(TopToBottom).padding(8).gap(6).align(CenterX, Top)
                                                })
                                                .children(|ui| {
                                                    ui.text(&day.weekday, |t| t.font_size(13).color(0xE8E0DC));
                                                    ui.text(dglyph, |t| t.font_size(22).font(&SYMBOL_FONT).color(0xE8E0DC));
                                                    ui.text(&format!("{:.0}°", day.hi), |t| t.font_size(14).color(0xE8E0DC));
                                                    ui.text(&format!("{:.0}°", day.lo), |t| t.font_size(13).color(0x9A9490));
                                                    ui.text(&format!("{}%", day.precip_pct), |t| t.font_size(12).color(0x6F9FE0));
                                                });
                                        }
                                    });
                            } else if state.user_location.is_none() {
                                ui.text("Detecting location…", |t| t.font_size(14).color(0xC8C0BC));
                            } else {
                                ui.text("Loading forecast…", |t| t.font_size(14).color(0xC8C0BC));
                            }
                        });
                }
```
Note: match `Top` / `CenterX` / `TopToBottom` alignment tokens to what the ply prelude actually exports (grep an existing `.align(` call in `main.rs` for the exact identifiers, e.g. `CenterY`, `Left`; use the analogous vertical-top token). If `a.text_input(...)` is not a method on the accessibility builder, use `a.button(&search_display)` instead — the search field is click-to-focus.

- [ ] **Step 2: Add the per-frame fetch/poll + search input handling**

In `app/src/main.rs`, in the input-handling section (near the settings location-input block ~line 2585, but this must run every frame regardless of whether the settings panel is open — place it after the settings-panel `if` block, alongside the other unconditional top-bar handlers). Add:
```rust
        // ── Forecast view: fetch, poll, and search input ────────────
        if state.view_mode == ViewMode::Forecast {
            // Target coords: a picked search hit sets forecast_coords directly
            // (handled below); otherwise follow the app's user_location.
            if let Some(target) = state.user_location {
                let stale = state.forecast_coords != Some(target);
                if stale && !state.forecast_fetch_fired {
                    // Derive a place label if we don't already have one.
                    if state.forecast_place.is_empty() {
                        state.forecast_place = if !state.settings.location_input.is_empty() {
                            state.settings.location_input.clone()
                        } else {
                            format!("{:.2}, {:.2}", target.lat, target.lon)
                        };
                    }
                    forecast::fire_forecast(target);
                    state.forecast_fetch_fired = true;
                    state.forecast_error = None;
                    // Remember which coords we're fetching for.
                    state.forecast_coords = Some(target);
                }
            }
            // Poll whichever coords we last fired for.
            if state.forecast_fetch_fired {
                if let Some(coords) = state.forecast_coords {
                    match forecast::poll_forecast(coords) {
                        Some(Ok(mut fc)) => {
                            fc.place = state.forecast_place.clone();
                            state.forecast = Some(fc);
                            state.forecast_fetch_fired = false;
                        }
                        Some(Err(e)) => {
                            state.forecast_error = Some(e.to_string());
                            state.forecast_fetch_fired = false;
                        }
                        None => {}
                    }
                }
            }

            // Search box focus (click) + typing.
            if ply.is_just_pressed("fc-search") {
                state.fc_search_focused = true;
            }
            if state.fc_search_focused {
                while let Some(ch) = get_char_pressed() {
                    if !ch.is_control() && state.fc_search_text.len() < 48 {
                        state.fc_search_text.push(ch);
                    }
                }
                if is_key_pressed(KeyCode::Backspace) {
                    state.fc_search_text.pop();
                }
                if is_key_pressed(KeyCode::Enter) {
                    state.fc_search_focused = false;
                    let q = state.fc_search_text.trim().to_string();
                    if !q.is_empty() {
                        forecast::fire_geo(&q);
                        state.fc_geo_fired = true;
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    state.fc_search_focused = false;
                }
            }
            // Poll geocode results.
            if state.fc_geo_fired {
                let q = state.fc_search_text.trim().to_string();
                if !q.is_empty() {
                    match forecast::poll_geo(&q) {
                        Some(Ok(hits)) => {
                            state.fc_geo_hits = hits;
                            state.fc_geo_fired = false;
                        }
                        Some(Err(_)) => {
                            state.fc_geo_hits = Vec::new();
                            state.fc_geo_fired = false;
                        }
                        None => {}
                    }
                }
            }
            // Pick a search result.
            for idx in 0..state.fc_geo_hits.len() {
                if ply.is_just_pressed(&format!("fc-hit-{idx}")) {
                    let hit = state.fc_geo_hits[idx].clone();
                    state.user_location = Some(hit.coords);
                    state.forecast_place = hit.label;
                    state.forecast_coords = None; // force refetch for new coords
                    state.forecast_fetch_fired = false;
                    state.forecast = None;
                    state.fc_geo_hits = Vec::new();
                    state.fc_search_text.clear();
                    break;
                }
            }
        }
```
Note: confirm `KeyCode` and `get_char_pressed` / `is_key_pressed` are already imported in `main.rs` (they are — used by the settings location field ~line 2597). If the forecast handling lands inside a scope where `ply` is already mutably borrowed by the UI tree, place it in the same input region that already calls `ply.is_just_pressed(...)` (the post-UI input section, ~line 2380+), not inside the `ply.begin()` closure.

- [ ] **Step 3: Build**

Run: `cargo build -p rustywx`
Expected: compiles; the Task 1 dead-code warnings are gone (functions now used).

- [ ] **Step 4: Run the full test suite**

Run: `cargo test -p rustywx`
Expected: PASS — all existing tests plus Task 1's forecast tests. No regressions.

- [ ] **Step 5: Manual end-to-end verification**

Run: `cargo run -p rustywx` (or the `run` skill). Verify:
- Click "Forecast" → the view shows. With a location already set (KFFC-area default / detected), current conditions (place, temp, icon, feels/wind/humidity) and a 7-day strip appear within a couple seconds.
- Type a city into the search box, press Enter → result rows appear; clicking one switches the forecast to that city (place label updates, data refetches).
- With no location resolved yet → "Detecting location…". Kill network / use a bad state → error line renders (best-effort; not a hard gate).
- Click "Radar" → returns to the scope, unchanged.

- [ ] **Step 6: Commit**

```bash
git add app/src/main.rs
git commit -m "feat: forecast view render, live fetch, and city search

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Notes for the implementer

- **ply alignment/layout tokens:** `main.rs` brings them in via `use ply_engine::prelude::*;`. Before writing new `.layout(...)`/`.align(...)` calls, grep an existing element for the exact token identifiers (`TopToBottom`, `LeftToRight`, `CenterX`, `CenterY`, `Left`, and the top-alignment token). Use the ones already in the file.
- **`glass_panel::glass` signature:** mirror the existing controls-bar call at ~line 930 (`glass_panel::glass(ui.element()...).layout(...).children(...)`).
- **Accessibility builder methods:** mirror existing `.accessibility(|a| a.button(...).checked(...))` usage. If `text_input` isn't available, fall back to `button` (noted in Task 4 Step 1).
- **Coords equality:** `Coords` derives `PartialEq`, so `state.forecast_coords != Some(target)` compiles. Because IDs are built with `{:.4}` precision, two coords equal to 4 decimals share a net request — acceptable.
- **Do not** touch the radar/tropical/settings behavior beyond the three one-line `view_mode = ViewMode::Radar` additions in Task 3.
```
