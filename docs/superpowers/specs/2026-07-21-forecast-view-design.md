# Forecast View — Design

**Date:** 2026-07-21
**Branch:** port/ply-engine
**Status:** Approved for planning

## Summary

Add a third top-bar item, **Forecast**, alongside Radar and Tropical. Unlike
those (which open side panels over the radar scope), Forecast is a *view mode*:
it swaps the entire content area from the radar scope to a full-screen,
non-map forecast screen showing current conditions and a 7-day outlook for the
app's location, with an in-view search box to look up any other location.

## Decisions (locked)

- **Data source:** Open-Meteo (free, no API key, global coverage).
  - Forecast API: `https://api.open-meteo.com/v1/forecast`
  - Geocoding API: `https://geocoding-api.open-meteo.com/v1/search`
- **Presentation:** full-screen mode swap. Top bar stays; radar scope, radar
  sweep, all overlays, and both side panels are hidden while Forecast is active.
- **Location:** defaults to the app's existing `user_location`; a search box in
  the view looks up any city via Open-Meteo geocoding and switches the forecast
  to the picked result.
- **Units:** °F / mph hardcoded (US-oriented app). No unit toggle in v1.
- **Persistence:** view mode is not persisted; app always launches in Radar.

## Architecture

New module `app/src/forecast.rs`, modeled on `app/src/alerts.rs`: fire-and-poll
over `ply_engine::prelude::net`, plus parse functions with unit tests. No new
dependencies (`ply net` + `serde_json` + existing `location::Coords`).

### `forecast.rs`

Net request IDs: `"forecast"` and `"geo-search"` (distinct so both can be in
flight independently, and independent of the existing `alerts` / `loc-*` IDs).

Endpoints:

- Forecast (per coords):
  ```
  https://api.open-meteo.com/v1/forecast
    ?latitude={lat}&longitude={lon}
    &current=temperature_2m,apparent_temperature,relative_humidity_2m,weather_code,wind_speed_10m,is_day
    &daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max
    &temperature_unit=fahrenheit&wind_speed_unit=mph&timezone=auto&forecast_days=7
  ```
- Geocode search (per query string):
  ```
  https://geocoding-api.open-meteo.com/v1/search?name={q}&count=5&language=en&format=json
  ```

Types:

```rust
pub struct Forecast {
    pub place: String,          // display label; from geocode pick or "lat, lon"
    pub current: Current,
    pub days: Vec<Day>,         // up to 7
}
pub struct Current {
    pub temp: f64,              // °F
    pub feels_like: f64,        // °F
    pub humidity: i64,          // %
    pub wind: f64,              // mph
    pub code: i64,              // WMO weather code
    pub is_day: bool,
}
pub struct Day {
    pub weekday: String,        // "Mon", "Tue", … derived from ISO date
    pub code: i64,              // WMO weather code
    pub hi: f64,                // °F
    pub lo: f64,                // °F
    pub precip_pct: i64,        // %
}
pub struct GeoHit {
    pub label: String,          // "Atlanta, Georgia, United States"
    pub coords: Coords,
}
```

Functions:

- `fire_forecast(coords: Coords)` — `net::get("forecast", url, ...)`. Idempotent
  per current call; caller re-fires when target coords change.
- `poll_forecast() -> Option<Result<Forecast>>` — parse when the response lands.
- `fire_geo(query: &str)`, `poll_geo() -> Option<Result<Vec<GeoHit>>>`.
- `parse_forecast(json: &str) -> Result<Forecast>` — pure, unit-tested.
- `parse_geo(json: &str) -> Result<Vec<GeoHit>>` — pure, unit-tested.
- `wmo_icon(code: i64, is_day: bool) -> (&'static str, &'static str)` — returns
  `(glyph, label)` for a WMO weather code (e.g. `(icon, "Partly cloudy")`).
  Uses the existing symbol font glyphs where available; unit-tested for the
  documented code buckets (0 clear, 1-3 cloud gradient, 45/48 fog, 51-67 rain,
  71-77 snow, 80-82 showers, 95-99 thunderstorm).
- `weekday_from_iso(date: &str) -> String` — "YYYY-MM-DD" → "Mon". Unit-tested.

### State (`app/src/state.rs`)

```rust
pub enum ViewMode { Radar, Forecast }   // default Radar
```

New `AppState` fields:

- `view_mode: ViewMode`
- `forecast: Option<Forecast>`
- `forecast_coords: Option<Coords>`  — coords the current `forecast` was fetched for
- `forecast_fetch_fired: bool`       — a forecast request is in flight
- `forecast_error: Option<String>`   — last fetch/parse error, for inline display
- `fc_search_text: String`
- `fc_search_focused: bool`
- `fc_geo_hits: Vec<GeoHit>`
- `fc_geo_fired: bool`

All initialized in the `AppState { … }` constructor in `main.rs`.

### Top bar (`app/src/main.rs`)

Add a **Forecast** button to the Panels group (after Tropical). It toggles the
view mode:

- Click when in Radar → `view_mode = Forecast`; close `radar_panel_open` and
  `nhc_show_panel` (mode is exclusive with the scope panels).
- Click when in Forecast → `view_mode = Radar`.
- Clicking the existing Radar or Tropical panel buttons also sets
  `view_mode = Radar` (returning to the scope) before applying their normal
  panel-open behavior.
- Active styling: teal tint + `✓` when `view_mode == Forecast` (mirrors the
  Layers-group ✓ idiom, since it *is* an active-state toggle, not a panel opener).

### Render swap (`app/src/main.rs`)

Gate on `state.view_mode`:

- **`Forecast`:** skip the radar scope draw, radar sweep, all overlays, and both
  side-panel blocks. Skip scope pan/zoom/click/hit-test input handling. Render a
  full-area glass forecast element under the top bar:
  1. **Search row** — text field bound to `fc_search_text` (reuse the settings
     text-field input pattern) + a results dropdown listing `fc_geo_hits`;
     picking a hit sets target coords and clears results.
  2. **Current conditions** — place name, large temperature, weather icon +
     label, then feels-like / wind / humidity.
  3. **7-day strip** — one cell per `Day`: weekday, icon, hi/lo, precip %.
  - States: no location yet → "Detecting location…"; in-flight and no data yet →
    "Loading forecast…"; `forecast_error` set → error line + implicit retry on
    next coords change / re-entry.
- **`Radar`:** existing behavior, unchanged.

### Data flow

Each frame while `view_mode == Forecast`:

1. Determine target coords: a search pick overrides; otherwise `user_location`.
2. If target is `Some` and `forecast_coords != target` and not already fired for
   it → `fire_forecast(target)`, set `forecast_fetch_fired`, record intended
   coords.
3. `poll_forecast()`:
   - `Some(Ok(f))` → store `forecast`, set `forecast_coords`, clear error/fired.
   - `Some(Err(e))` → set `forecast_error`, clear fired.
   - `None` → still pending.
4. Search: on submit (Enter) with non-empty text → `fire_geo(text)`, set
   `fc_geo_fired`. `poll_geo()` fills `fc_geo_hits`.

## Error handling

- Network / parse failures surface as `forecast_error` shown inline in the view
  (no toast — the view is the focus). Geocode failures clear the results list
  and leave the current forecast intact.
- Empty geocode results → "No matches" line under the search box.

## Testing

Unit tests in `forecast.rs` (matching `alerts.rs` test style, no framework):

- `parse_forecast` against a captured Open-Meteo fixture → asserts current
  fields and day count / first-day hi/lo/precip.
- `parse_geo` against a fixture → asserts label composition and coords.
- `wmo_icon` → asserts label for representative codes in each bucket.
- `weekday_from_iso` → asserts a known date maps to the right weekday.

## Out of scope (deferred)

- Unit toggle (°C/km·h). Add when requested.
- Hourly forecast.
- Persisting last view mode or last searched location.
- Caching forecast to disk (live poll only; refetch on coords change / re-entry).
