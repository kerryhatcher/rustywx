# User Location — Design

**Date:** 2026-07-20
**Status:** Approved (brainstorming)
**Branch:** port/ply-engine

## Goal

Let the app know where the user is, then optionally:

1. **Display** the user's location as a marker on the radar scope, and/or
2. **Center** the scope on the user's location.

Both are independent toggles. Location is obtained through a fallback
provider chain: **system location** (macOS CoreLocation now; Linux gpsd
later) → **IP geolocation** → **manual entry** (lat/lon or US ZIP, an
explicit override).

## Non-goals

- No `.app` bundle in this pass (CoreLocation therefore falls through to IP
  during bare-binary dev runs — see Constraints).
- No Linux gpsd implementation yet (stub provider with the final interface).
- No continuous location tracking / movement following. One-shot resolve on
  demand; re-resolve only when the user asks or input changes.
- No reverse geocoding, no address display beyond the raw coords/ZIP.

## Constraints & realities

- **Native app, no browser geolocation.** All network calls go through the
  existing Ply `net` fire-and-poll model (same as `alerts.rs` / `nhc.rs`):
  `net::get(ID, URL, |c| …)` fires; `net::request(ID)?.response()?` polls each
  frame.
- **CoreLocation needs a bundled `.app`.** `CLLocationManager` only delivers a
  fix when the process is a bundled `.app` carrying an
  `NSLocationWhenInUseUsageDescription` Info.plist key. From a bare
  `cargo run` binary, macOS reports authorization *denied* and `location()`
  stays nil, so the chain falls through to IP. Mac location begins working the
  moment rustywx is packaged as a `.app` with that key — no code change
  needed. The fallback makes the un-bundled case graceful.
- **Privacy:** IP is only sent to a third party (ipapi.co) when the user
  presses **Detect**. Manual coord entry never touches the network. ZIP entry
  hits zippopotam.us only on submit.
- **Cross-platform:** IP + ZIP + manual work on macOS, Linux, and WASM.
  System provider is `#[cfg]`-gated per OS.

## Architecture

### New module: `app/src/location.rs`

Core types and the resolve flow.

```rust
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Coords { pub lat: f64, pub lon: f64 }

/// Where a resolved fix came from (for the status line).
pub enum LocationStatus {
    Idle,
    Detecting,
    Resolved(Coords, Source),   // Source: System | Ip | Manual
    Denied,                     // system auth denied/restricted
    Offline,                    // net error
    NotFound,                   // bad ZIP / empty IP result
}
```

- **Input parser** `parse_location_input(&str) -> LocationInput`:
  - Two floats separated by comma or whitespace → `Coords` (no network).
  - Exactly 5 ASCII digits → `Zip(String)`.
  - Anything else → `Invalid`.
- **ZIP lookup:** `net::get("loc-zip", "https://api.zippopotam.us/us/{zip}", …)`;
  parse `places[0].latitude` / `longitude` (strings → f64).
- **IP lookup:** `net::get("loc-ip", "https://ipapi.co/json/", …)`; parse
  `latitude` / `longitude` (numbers).
- Pure parse functions kept free of I/O for unit testing.

### System provider

Trait-free: a single function per platform, selected by `#[cfg]`, with the
same signature so the chain calls one name.

```rust
// system_location.rs
pub enum SystemFix { Pending, Ready(Coords), Unavailable }

// macOS: holds a CLLocationManager, polled each frame.
#[cfg(target_os = "macos")] pub struct SystemLocator { /* objc2 manager */ }
#[cfg(not(target_os = "macos"))] pub struct SystemLocator; // stub
```

- **macOS** (`#[cfg(target_os="macos")]`), via `objc2` + `objc2-core-location`:
  - `SystemLocator::start()` → create `CLLocationManager`, call
    `requestWhenInUseAuthorization()` + `startUpdatingLocation()`.
  - `poll() -> SystemFix`: check `authorizationStatus`; if denied/restricted →
    `Unavailable`. If `manager.location()` is `Some` → `Ready(coords)`. Else
    `Pending`. **Delegate-free** — we poll the `location` property rather than
    implementing a `CLLocationManagerDelegate` subclass.
  - Caller enforces a ~5 s timeout: still `Pending` after timeout →
    treat as `Unavailable`, fall to IP.
  - `CLLocationManager` is not `Send`; it lives in a non-`Send` field of the
    single-threaded frame loop — fine (macroquad main loop is one thread).
- **Linux / other:** stub `poll()` returns `Unavailable` immediately. gpsd
  drops in here later behind `#[cfg(target_os="linux")]` with the same enum.

### The chain

**Startup:** if `user_lat`/`user_lon` are persisted, load them straight into
`state.user_location` (`Source::Manual`/last known) — no network, no chain.
`location_input` is restored only to redisplay in the field. The chain never
auto-runs on launch; it runs only when the user presses **Detect**.

`Detect` pressed:

1. **Manual override:** if the input box parses to `Coords` → use directly,
   `Source::Manual`, done (no chain, no network).
2. **Manual ZIP:** if it parses to `Zip` → fire ZIP lookup, `Source::Manual`.
3. **Auto (empty input / explicit "Detect"):** start `SystemLocator`; poll
   until `Ready` (→ done) or `Unavailable`/timeout → fire IP lookup →
   `Ready`/`Offline`/`NotFound`.

Only one detect runs at a time (guard flag). Results and status write into
`state.location_status`; on `Resolved` we store `Coords` in
`state.user_location` and persist to settings.

## State & settings

`AppState` (runtime):
- `user_location: Option<Coords>`
- `show_location: bool` (mirrors the top-bar toggle, seeded from settings)
- `location_status: LocationStatus`
- `system_locator: Option<SystemLocator>` (live during an auto-detect)
- detect-in-flight guard + start timestamp

`Settings` (persisted, every field `#[serde(default)]` for back-compat):
- `user_lat: Option<f64>`, `user_lon: Option<f64>`
- `location_input: String` (raw text so it redisplays)
- `show_location: bool`
- `center_on_location: bool`

Persisted via the existing `cache.save_settings`.

## UI

### Top controls bar
A **"Location"** toggle button (Nerd Font location-pin glyph, new `nf`
constant) beside Alerts / Tropical, styled like the existing overlay toggles.
Toggles `show_location` (marker visibility) and persists it. Button id
`btn-location`.

### Settings panel — new "My Location" section
- **Text input** (`lat, lon` or ZIP) bound to `location_input`.
- **"Detect" button** — runs the chain.
- **"Center map on my location" checkbox** → `center_on_location`.
- **Status line** — from `LocationStatus`: e.g. `32.68, -83.35 (IP)`,
  `Detecting…`, `Location denied`, `Offline`, `ZIP not found`.

Modal height grows to fit the new section (as prior settings additions did).

## Drawing the marker

New `scope::draw_user_location(coords, site, center_x, center_y, px_per_km)`,
called from the same overlay pass as `draw_alerts`. Projects the user's
lat/lon through the existing `geo::point_to_km_offset(site.lat, site.lon,
(lat, lon))` path, then draws a distinct **cyan pin/crosshair** (visually
separate from alert polygons and radar echoes). Drawn only when
`show_location` and `user_location.is_some()`.

## Centering

`center_on_location` true → set `pan_km = -offset`, where
`offset = point_to_km_offset(site.lat, site.lon, user)`. This places the user
at screen center given the projection in `scope.rs`
(`center_x = screen/2 + pan_km.x*px_per_km`; marker at
`center_x + off.x*px_per_km`; zero when `pan_km = -off`).

Re-applied when: coords change, the center checkbox flips on, or the selected
radar site changes. Manual pan/zoom afterward is preserved until the next
trigger (we do not force-recenter every frame).

## Error handling

All failure paths set `LocationStatus` for the settings status line **and**
raise the existing toast (`state.toast`) with a short message: offline,
location denied, ZIP not found, invalid input. No panics; network errors are
already `Result`-typed in the net poll pattern.

## Testing

Pure-function unit tests (asserts, no framework — matches repo style):
- `parse_location_input`: `"32.68, -83.35"` → coords; `"30301"` → ZIP;
  `"32.68 -83.35"` → coords; `"nope"` / `"1234"` / `"123456"` → invalid.
- `parse_ipapi_json`: sample ipapi.co body → coords.
- `parse_zippopotam_json`: sample zippopotam body → coords; empty `places` →
  NotFound.
- Centering math: `pan_km` chosen for a known site+user puts the user at
  screen center (reuse `point_to_km_offset`).

Smoke: `cargo build`, run, Detect (expect IP fallback on bare binary), toggle
marker, toggle center.

## Dependencies

```toml
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "..."
objc2-core-location = "..."
objc2-foundation = "..."   # if needed for NSError/types
```

Versions pinned during implementation against what's current on crates.io.
No new deps on Linux/WASM (IP/ZIP use existing `net` + `serde_json`).

## Files touched

- `app/src/location.rs` — **new**: types, parser, IP/ZIP fetch + parse, chain.
- `app/src/system_location.rs` — **new**: macOS CoreLocation + non-mac stub.
- `app/src/scope.rs` — add `draw_user_location`; call in overlay pass.
- `app/src/settings.rs` — new persisted fields + defaults + test update.
- `app/src/state.rs` — new runtime fields.
- `app/src/widgets/settings.rs` — "My Location" section + widget ids.
- `app/src/widgets/mod.rs` — location-pin `nf` glyph constant.
- `app/src/main.rs` — top-bar toggle button, press handlers, detect chain
  polling, centering trigger, marker draw wiring, module decls.
- `app/Cargo.toml` — target-gated objc2 deps.

## Open items deferred

- `.app` bundle + Info.plist usage-description key (unblocks real Mac
  location). Tracked separately.
- Linux gpsd provider.
