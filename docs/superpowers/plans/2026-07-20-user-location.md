# User Location Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Resolve the user's location (macOS CoreLocation → IP geo → manual coords/ZIP) and optionally show it as a marker on the scope and/or center the scope on it.

**Architecture:** A `location.rs` module owns pure parsers + a `LocationResolver` state machine that runs the fallback chain across frames (system provider polled, then Ply `net` IP/ZIP fetches polled). A `system_location.rs` module wraps macOS CoreLocation (objc2) with a non-mac stub. The main loop polls the resolver, stores `Option<Coords>` in `AppState`, persists to `Settings`, draws a marker via the existing scope projection, and recenters `pan_km` on demand.

**Tech Stack:** Rust, macroquad/miniquad, ply-engine 1.1.1 (`net`, `storage`), serde_json, objc2 0.6 + objc2-core-location 0.3 (macOS only).

## Global Constraints

- Ply net pattern: `net::get(ID, URL, |c| c.header(...))` fires; `net::request(ID)?.response()?` returns `Option<Result<Resp, _>>`; `resp.text()` is the body. (See `app/src/alerts.rs`.)
- All new `Settings` fields MUST be `#[serde(default)]` for backward-compatible deserialization of existing cached settings.
- No panics on network/permission failure — surface via `state.toast` + `LocationStatus`.
- CoreLocation deps are target-gated: `[target.'cfg(target_os = "macos")'.dependencies]`. No new deps on Linux/WASM.
- Startup never auto-detects: saved `user_lat`/`user_lon` load directly (no network); the chain runs only on the Detect button.
- Manual input overrides the chain; empty input + Detect runs System→IP.
- Marker color is cyan, distinct from alerts/echoes. Center places user at screen center via `pan_km = -offset`.
- Run all commands from repo root `/Users/kerry.hatcher/projects/rustywx`. The binary needs the `app/` cwd for assets: run it as `cd app && ../target/debug/rustywx`.
- Tests use plain `#[test]` + `assert!`/`assert_eq!` (repo style — no framework).

---

### Task 1: location.rs — core types + input parser

**Files:**
- Create: `app/src/location.rs`
- Modify: `app/src/lib.rs` (add `pub mod location;`)

**Interfaces:**
- Produces:
  - `pub struct Coords { pub lat: f64, pub lon: f64 }` (derives `Clone, Copy, PartialEq, Debug`)
  - `pub enum Source { System, Ip, Manual }` (derives `Clone, Copy, PartialEq, Debug`)
  - `pub enum LocationInput { Coords(Coords), Zip(String), Invalid }`
  - `pub fn parse_location_input(s: &str) -> LocationInput`

- [ ] **Step 1: Add module declaration**

In `app/src/lib.rs`, add alongside the other `pub mod` lines:

```rust
pub mod location;
```

- [ ] **Step 2: Write the failing test**

Create `app/src/location.rs`:

```rust
//! User-location resolution: input parsing, IP/ZIP geolocation over Ply `net`,
//! and a fallback-chain state machine (system → IP → manual).

/// A resolved geographic coordinate.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Coords {
    pub lat: f64,
    pub lon: f64,
}

/// Where a resolved fix came from (shown in the settings status line).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Source {
    System,
    Ip,
    Manual,
}

/// Parsed form of the manual-entry text field.
#[derive(Clone, PartialEq, Debug)]
pub enum LocationInput {
    Coords(Coords),
    Zip(String),
    Invalid,
}

/// Parse the manual-entry text into coordinates, a ZIP, or invalid.
///
/// - Two floats separated by comma or whitespace → `Coords` (no network).
/// - Exactly 5 ASCII digits → `Zip`.
/// - Anything else → `Invalid`.
pub fn parse_location_input(s: &str) -> LocationInput {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_coords_and_zip_and_junk() {
        assert_eq!(
            parse_location_input("32.68, -83.35"),
            LocationInput::Coords(Coords { lat: 32.68, lon: -83.35 })
        );
        assert_eq!(
            parse_location_input("32.68 -83.35"),
            LocationInput::Coords(Coords { lat: 32.68, lon: -83.35 })
        );
        assert_eq!(parse_location_input("30301"), LocationInput::Zip("30301".to_string()));
        assert_eq!(parse_location_input("  30301 "), LocationInput::Zip("30301".to_string()));
        assert_eq!(parse_location_input("nope"), LocationInput::Invalid);
        assert_eq!(parse_location_input("1234"), LocationInput::Invalid);
        assert_eq!(parse_location_input("123456"), LocationInput::Invalid);
        assert_eq!(parse_location_input("91.0, 0.0"), LocationInput::Invalid); // lat out of range
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p rustywx location::tests::parses_coords_and_zip_and_junk`
Expected: FAIL / panic in `todo!()`.

- [ ] **Step 4: Implement the parser**

Replace the `todo!()` body:

```rust
pub fn parse_location_input(s: &str) -> LocationInput {
    let s = s.trim();
    if s.is_empty() {
        return LocationInput::Invalid;
    }

    // Try "lat, lon" or "lat lon".
    let parts: Vec<&str> = s
        .split([',', ' ', '\t'])
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() == 2
        && let (Ok(lat), Ok(lon)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>())
    {
        if (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lon) {
            return LocationInput::Coords(Coords { lat, lon });
        }
        return LocationInput::Invalid;
    }

    // Try 5-digit ZIP.
    if s.len() == 5 && s.bytes().all(|b| b.is_ascii_digit()) {
        return LocationInput::Zip(s.to_string());
    }

    LocationInput::Invalid
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p rustywx location::tests::parses_coords_and_zip_and_junk`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add app/src/lib.rs app/src/location.rs
git commit -m "feat: location input parser (coords/ZIP)"
```

---

### Task 2: location.rs — IP + ZIP geolocation fetch/parse

**Files:**
- Modify: `app/src/location.rs`

**Interfaces:**
- Consumes: `Coords` (Task 1).
- Produces:
  - `pub fn parse_ipapi_json(body: &str) -> Option<Coords>`
  - `pub fn parse_zippopotam_json(body: &str) -> Option<Coords>`
  - `pub fn fire_ip()` / `pub fn poll_ip() -> Option<Result<Coords, String>>`
  - `pub fn fire_zip(zip: &str)` / `pub fn poll_zip() -> Option<Result<Coords, String>>`

- [ ] **Step 1: Write the failing tests**

Add to the `tests` module in `app/src/location.rs`:

```rust
    #[test]
    fn parses_ipapi_body() {
        let body = r#"{"ip":"1.2.3.4","city":"Macon","latitude":32.8407,"longitude":-83.6324}"#;
        assert_eq!(parse_ipapi_json(body), Some(Coords { lat: 32.8407, lon: -83.6324 }));
        assert_eq!(parse_ipapi_json(r#"{"error":true}"#), None);
    }

    #[test]
    fn parses_zippopotam_body() {
        let body = r#"{"post code":"30301","country":"United States",
            "places":[{"place name":"Atlanta","latitude":"33.7490","longitude":"-84.3880"}]}"#;
        assert_eq!(parse_zippopotam_json(body), Some(Coords { lat: 33.7490, lon: -84.3880 }));
        assert_eq!(parse_zippopotam_json(r#"{"places":[]}"#), None);
        assert_eq!(parse_zippopotam_json("not json"), None);
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p rustywx location::tests::parses_ipapi_body`
Expected: FAIL with "cannot find function `parse_ipapi_json`".

- [ ] **Step 3: Implement fetch + parse**

Add near the top of `app/src/location.rs` (after the type definitions):

```rust
use serde_json::Value;

const IP_NET_ID: &str = "loc-ip";
const ZIP_NET_ID: &str = "loc-zip";
const IP_URL: &str = "https://ipapi.co/json/";
const USER_AGENT: &str = "rustywx (github.com/rustywx)";

/// ipapi.co returns `{"latitude": <num>, "longitude": <num>, ...}`.
pub fn parse_ipapi_json(body: &str) -> Option<Coords> {
    let v: Value = serde_json::from_str(body).ok()?;
    let lat = v.get("latitude")?.as_f64()?;
    let lon = v.get("longitude")?.as_f64()?;
    Some(Coords { lat, lon })
}

/// zippopotam.us returns `{"places":[{"latitude":"<str>","longitude":"<str>"}]}`.
pub fn parse_zippopotam_json(body: &str) -> Option<Coords> {
    let v: Value = serde_json::from_str(body).ok()?;
    let place = v.get("places")?.as_array()?.first()?;
    let lat = place.get("latitude")?.as_str()?.parse::<f64>().ok()?;
    let lon = place.get("longitude")?.as_str()?.parse::<f64>().ok()?;
    Some(Coords { lat, lon })
}

/// Fire the IP-geolocation request. Idempotent per net-id.
pub fn fire_ip() {
    use ply_engine::prelude::net;
    net::get(IP_NET_ID, IP_URL, |c| c.header("User-Agent", USER_AGENT));
}

/// Poll the IP request. `None` = pending; `Some(Ok/Err)` = done.
pub fn poll_ip() -> Option<Result<Coords, String>> {
    use ply_engine::prelude::net;
    let resp = net::request(IP_NET_ID)?.response()?;
    match resp {
        Ok(r) => Some(parse_ipapi_json(r.text()).ok_or_else(|| "no coords in IP response".to_string())),
        Err(e) => Some(Err(format!("IP geolocation failed: {e}"))),
    }
}

/// Fire a ZIP → coords lookup. Idempotent per net-id.
pub fn fire_zip(zip: &str) {
    use ply_engine::prelude::net;
    let url = format!("https://api.zippopotam.us/us/{zip}");
    net::get(ZIP_NET_ID, &url, |c| c.header("User-Agent", USER_AGENT));
}

/// Poll the ZIP request. `None` = pending; `Some(Ok/Err)` = done.
pub fn poll_zip() -> Option<Result<Coords, String>> {
    use ply_engine::prelude::net;
    let resp = net::request(ZIP_NET_ID)?.response()?;
    match resp {
        Ok(r) => Some(parse_zippopotam_json(r.text()).ok_or_else(|| "ZIP not found".to_string())),
        Err(e) => Some(Err(format!("ZIP lookup failed: {e}"))),
    }
}
```

> If `net::get`'s URL param requires `&'static str` (it does not — `alerts.rs` passes a const, but the signature accepts `&str`), confirm against `alerts.rs`. The `fire_zip` format-string URL is passed by reference; verify it compiles.

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p rustywx location::tests`
Expected: PASS (all location tests).

- [ ] **Step 5: Commit**

```bash
git add app/src/location.rs
git commit -m "feat: IP + ZIP geolocation fetch/parse"
```

---

### Task 3: Settings — persisted location fields

**Files:**
- Modify: `app/src/settings.rs`

**Interfaces:**
- Produces on `Settings`: `user_lat: Option<f64>`, `user_lon: Option<f64>`, `location_input: String`, `show_location: bool`, `center_on_location: bool`.

- [ ] **Step 1: Add fields to the struct**

In `app/src/settings.rs`, inside `pub struct Settings`, after `pub dyslexic_font: bool,`:

```rust
    /// Last resolved user latitude (persisted; loaded at startup without network).
    #[serde(default)]
    pub user_lat: Option<f64>,
    /// Last resolved user longitude.
    #[serde(default)]
    pub user_lon: Option<f64>,
    /// Raw manual-entry text (coords or ZIP), kept so the field redisplays.
    #[serde(default)]
    pub location_input: String,
    /// Whether the user-location marker is shown on the scope.
    #[serde(default)]
    pub show_location: bool,
    /// Whether the scope recenters on the user's location.
    #[serde(default)]
    pub center_on_location: bool,
```

- [ ] **Step 2: Add defaults**

In `impl Default for Settings`, after `dyslexic_font: false,`:

```rust
            user_lat: None,
            user_lon: None,
            location_input: String::new(),
            show_location: false,
            center_on_location: false,
```

- [ ] **Step 3: Extend the back-compat test**

In the `tests` module, add:

```rust
    #[test]
    fn location_fields_default_off() {
        let s = Settings::default();
        assert!(s.user_lat.is_none());
        assert!(!s.show_location);
        assert!(!s.center_on_location);
        assert!(s.location_input.is_empty());
    }

    #[test]
    fn deserializes_settings_without_location_fields() {
        // Simulate an old cached settings blob lacking the new keys.
        let json = r#"{"default_site":"KFFC","poll_interval_secs":120,"nhc_refresh_secs":300,
            "show_borders":true,"show_alerts":true,"show_nhc":false,
            "animation_level":"Full","tdbz_kernel":"Off","dyslexic_font":false}"#;
        let s: Settings = serde_json::from_str(json).expect("back-compat deserialize");
        assert!(s.user_lat.is_none());
        assert!(!s.center_on_location);
    }
```

> Verify the `animation_level` / `tdbz_kernel` string values (`"Full"`, `"Off"`) match those enums' serde representation in `settings.rs`; adjust the JSON literal if their variant names differ.

- [ ] **Step 4: Run tests**

Run: `cargo test -p rustywx settings`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add app/src/settings.rs
git commit -m "feat: persist user-location settings fields"
```

---

### Task 4: macOS CoreLocation provider + stub + deps

**Files:**
- Create: `app/src/system_location.rs`
- Modify: `app/src/lib.rs` (add `pub mod system_location;`)
- Modify: `app/Cargo.toml` (target-gated objc2 deps)

**Interfaces:**
- Consumes: `Coords` (from `crate::location`).
- Produces:
  - `pub enum SystemFix { Pending, Ready(Coords), Unavailable }`
  - `pub struct SystemLocator` with `pub fn start() -> Self` and `pub fn poll(&mut self) -> SystemFix`

- [ ] **Step 1: Add the target-gated dependencies**

In `app/Cargo.toml`, add a new section (after `[dependencies]`):

```toml
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.6"
objc2-core-location = { version = "0.3", features = ["CLLocation", "CLLocationManager"] }
```

- [ ] **Step 2: Add module declaration**

In `app/src/lib.rs`:

```rust
pub mod system_location;
```

- [ ] **Step 3: Write the macOS implementation + non-mac stub**

Create `app/src/system_location.rs`:

```rust
//! System-provided device location.
//!
//! macOS: CoreLocation via objc2 (delegate-free — we poll `CLLocationManager`'s
//! `location` property each frame after requesting authorization).
//! Other platforms: a stub returning `Unavailable` (Linux gpsd goes here later).
//!
//! NOTE: CoreLocation only delivers a fix when the process is a bundled `.app`
//! carrying `NSLocationWhenInUseUsageDescription`. From a bare binary, auth is
//! denied and this reports `Unavailable`, so the caller falls back to IP.

use crate::location::Coords;

/// One poll result from the system locator.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SystemFix {
    Pending,
    Ready(Coords),
    Unavailable,
}

#[cfg(target_os = "macos")]
pub struct SystemLocator {
    manager: objc2::rc::Retained<objc2_core_location::CLLocationManager>,
}

#[cfg(target_os = "macos")]
impl SystemLocator {
    pub fn start() -> Self {
        use objc2_core_location::CLLocationManager;
        // SAFETY: standard CoreLocation setup on the main thread (macroquad's
        // event loop runs the Cocoa run loop that delivers updates).
        unsafe {
            let manager = CLLocationManager::new();
            manager.requestWhenInUseAuthorization();
            manager.startUpdatingLocation();
            SystemLocator { manager }
        }
    }

    pub fn poll(&mut self) -> SystemFix {
        use objc2_core_location::CLAuthorizationStatus;
        // SAFETY: reading authorization status + last location property.
        unsafe {
            let status = self.manager.authorizationStatus();
            let authorized = status == CLAuthorizationStatus::AuthorizedWhenInUse
                || status == CLAuthorizationStatus::AuthorizedAlways;
            if status == CLAuthorizationStatus::Denied
                || status == CLAuthorizationStatus::Restricted
            {
                return SystemFix::Unavailable;
            }
            if authorized
                && let Some(loc) = self.manager.location()
            {
                let c = loc.coordinate();
                return SystemFix::Ready(Coords { lat: c.latitude, lon: c.longitude });
            }
            SystemFix::Pending
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub struct SystemLocator;

#[cfg(not(target_os = "macos"))]
impl SystemLocator {
    /// Stub: no system location provider yet (Linux gpsd goes here later).
    pub fn start() -> Self {
        SystemLocator
    }
    pub fn poll(&mut self) -> SystemFix {
        SystemFix::Unavailable
    }
}
```

- [ ] **Step 4: Verify it builds on this host (macOS)**

Run: `cargo build -p rustywx`
Expected: compiles. If an objc2 method name/signature mismatches (e.g. `authorizationStatus` is a class vs instance method), consult the extracted crate at `/tmp/objc2-core-location-0.3.2/src/generated/CLLocationManager.rs` (instance `authorizationStatus(&self)` exists at line ~161) and adjust.

- [ ] **Step 5: Commit**

```bash
git add app/Cargo.toml app/src/lib.rs app/src/system_location.rs
git commit -m "feat: macOS CoreLocation provider + non-mac stub"
```

---

### Task 5: location.rs — LocationResolver chain

**Files:**
- Modify: `app/src/location.rs`

**Interfaces:**
- Consumes: `Coords`, `Source`, `parse_location_input`, `fire_ip/poll_ip`, `fire_zip/poll_zip` (Tasks 1–2); `SystemLocator`, `SystemFix` (Task 4).
- Produces:
  - `pub enum LocationStatus { Idle, Detecting, Resolved(Coords, Source), Denied, Offline, NotFound, Invalid }`
  - `pub struct LocationResolver`
  - `impl LocationResolver`: `pub fn new() -> Self`, `pub fn status(&self) -> &LocationStatus`, `pub fn detect(&mut self, input: &str, now: f64)`, `pub fn poll(&mut self, now: f64) -> Option<Coords>`

- [ ] **Step 1: Implement the resolver**

Add to `app/src/location.rs`:

```rust
use crate::system_location::{SystemFix, SystemLocator};

/// Detect status, surfaced in the settings status line.
#[derive(Clone, PartialEq, Debug)]
pub enum LocationStatus {
    Idle,
    Detecting,
    Resolved(Coords, Source),
    Denied,
    Offline,
    NotFound,
    Invalid,
}

enum Phase {
    Idle,
    SystemWait { start: f64, locator: SystemLocator },
    IpWait,
    ZipWait,
}

/// System-location timeout before falling back to IP (seconds).
const SYSTEM_TIMEOUT_SECS: f64 = 5.0;

pub struct LocationResolver {
    phase: Phase,
    status: LocationStatus,
}

impl LocationResolver {
    pub fn new() -> Self {
        LocationResolver { phase: Phase::Idle, status: LocationStatus::Idle }
    }

    pub fn status(&self) -> &LocationStatus {
        &self.status
    }

    /// Begin resolution. Manual coords resolve instantly; ZIP fires a lookup;
    /// empty/other input runs the system → IP auto chain.
    pub fn detect(&mut self, input: &str, now: f64) -> Option<Coords> {
        match parse_location_input(input) {
            LocationInput::Coords(c) => {
                self.phase = Phase::Idle;
                self.status = LocationStatus::Resolved(c, Source::Manual);
                Some(c)
            }
            LocationInput::Zip(zip) => {
                fire_zip(&zip);
                self.phase = Phase::ZipWait;
                self.status = LocationStatus::Detecting;
                None
            }
            LocationInput::Invalid => {
                // Empty input → auto chain; non-empty junk → invalid.
                if input.trim().is_empty() {
                    self.phase = Phase::SystemWait { start: now, locator: SystemLocator::start() };
                    self.status = LocationStatus::Detecting;
                } else {
                    self.phase = Phase::Idle;
                    self.status = LocationStatus::Invalid;
                }
                None
            }
        }
    }

    /// Advance the state machine. Returns `Some(coords)` on the frame a fix
    /// is newly resolved (system/IP/ZIP); updates `status` regardless.
    pub fn poll(&mut self, now: f64) -> Option<Coords> {
        match &mut self.phase {
            Phase::Idle => None,
            Phase::SystemWait { start, locator } => {
                match locator.poll() {
                    SystemFix::Ready(c) => {
                        self.phase = Phase::Idle;
                        self.status = LocationStatus::Resolved(c, Source::System);
                        Some(c)
                    }
                    SystemFix::Pending if now - *start < SYSTEM_TIMEOUT_SECS => None,
                    _ => {
                        // Unavailable or timed out → fall back to IP.
                        fire_ip();
                        self.phase = Phase::IpWait;
                        None
                    }
                }
            }
            Phase::IpWait => match poll_ip() {
                None => None,
                Some(Ok(c)) => {
                    self.phase = Phase::Idle;
                    self.status = LocationStatus::Resolved(c, Source::Ip);
                    Some(c)
                }
                Some(Err(_)) => {
                    self.phase = Phase::Idle;
                    self.status = LocationStatus::Offline;
                    None
                }
            },
            Phase::ZipWait => match poll_zip() {
                None => None,
                Some(Ok(c)) => {
                    self.phase = Phase::Idle;
                    self.status = LocationStatus::Resolved(c, Source::Manual);
                    Some(c)
                }
                Some(Err(_)) => {
                    self.phase = Phase::Idle;
                    self.status = LocationStatus::NotFound;
                    None
                }
            },
        }
    }
}

impl Default for LocationResolver {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 2: Add a manual-coords resolve test**

In the `tests` module:

```rust
    #[test]
    fn detect_manual_coords_resolves_instantly() {
        let mut r = LocationResolver::new();
        let got = r.detect("40.0, -75.0", 0.0);
        assert_eq!(got, Some(Coords { lat: 40.0, lon: -75.0 }));
        assert!(matches!(r.status(), LocationStatus::Resolved(_, Source::Manual)));
    }

    #[test]
    fn detect_junk_is_invalid() {
        let mut r = LocationResolver::new();
        assert_eq!(r.detect("banana", 0.0), None);
        assert_eq!(*r.status(), LocationStatus::Invalid);
    }
```

- [ ] **Step 3: Run tests + build**

Run: `cargo test -p rustywx location`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add app/src/location.rs
git commit -m "feat: location resolver fallback chain (system/IP/ZIP)"
```

---

### Task 6: AppState runtime fields + startup restore

**Files:**
- Modify: `app/src/state.rs`
- Modify: `app/src/main.rs` (state init block near line 390–415)

**Interfaces:**
- Consumes: `Coords`, `LocationResolver`, `LocationStatus` (Tasks 1, 5).
- Produces on `AppState`: `user_location: Option<Coords>`, `show_location: bool`, `location_resolver: LocationResolver`, `location_input_focused: bool`.

- [ ] **Step 1: Add fields to AppState**

In `app/src/state.rs`, add near `show_alerts` / `show_borders`:

```rust
    /// Resolved user location (marker + centering source). `None` until set.
    pub user_location: Option<crate::location::Coords>,
    /// Whether the user-location marker is drawn (mirrors the top-bar toggle).
    pub show_location: bool,
    /// Fallback-chain driver for on-demand location detection.
    pub location_resolver: crate::location::LocationResolver,
    /// Whether the settings location text field is capturing keystrokes.
    pub location_input_focused: bool,
```

Add the imports at the top of `state.rs` if not already present:

```rust
use crate::location::{Coords, LocationResolver};
```

(If you prefer fully-qualified paths as written above, skip the `use`.)

- [ ] **Step 2: Initialize in main.rs**

In `app/src/main.rs`, in the `AppState { … }` construction (near `show_alerts: true,`), add:

```rust
        user_location: state_initial_location,
        show_location: settings.show_location,
        location_resolver: rustywx::location::LocationResolver::new(),
        location_input_focused: false,
```

Just before the `AppState {` literal (where `settings` is already loaded), add the startup-restore line:

```rust
    // Restore last known location from settings without any network call.
    let state_initial_location = match (settings.user_lat, settings.user_lon) {
        (Some(lat), Some(lon)) => Some(rustywx::location::Coords { lat, lon }),
        _ => None,
    };
```

> Confirm the exact local variable name holding the loaded settings (`settings`) in the surrounding code and that `show_location`/`user_lat` are read from it. Adjust the field seed for `show_location` to read `settings.show_location`.

- [ ] **Step 3: Build**

Run: `cargo build -p rustywx`
Expected: compiles.

- [ ] **Step 4: Commit**

```bash
git add app/src/state.rs app/src/main.rs
git commit -m "feat: user-location runtime state + startup restore"
```

---

### Task 7: Marker drawing on the scope

**Files:**
- Modify: `app/src/scope.rs` (add `draw_user_location`, extend `draw_scope_to_texture` signature + overlay pass)
- Modify: `app/src/main.rs` (pass the new arg at the call site, ~line 729)

**Interfaces:**
- Consumes: `Coords` (Task 1); `geo::point_to_km_offset`, existing `draw_scope_to_texture`.
- Produces: `draw_scope_to_texture` gains a trailing param `user: Option<Coords>`.

- [ ] **Step 1: Add the marker draw function**

In `app/src/scope.rs`, after `fn draw_alerts(...)`:

```rust
/// Draw the user's location as a cyan crosshair-pin at its projected position.
fn draw_user_location(
    user: Coords,
    site: &RadarSite,
    center_x: f32,
    center_y: f32,
    px_per_km: f32,
) {
    let off = geo::point_to_km_offset(site.lat, site.lon, (user.lat, user.lon));
    let x = center_x + off.x * px_per_km;
    let y = center_y + off.y * px_per_km;
    let cyan = Color::from_rgba(0, 220, 220, 255);
    // Crosshair + center dot — distinct from alert polygons and echoes.
    draw_line(x - 10.0, y, x + 10.0, y, 2.0, cyan);
    draw_line(x, y - 10.0, x, y + 10.0, 2.0, cyan);
    draw_circle(x, y, 4.0, cyan);
    draw_circle_lines(x, y, 9.0, 1.5, cyan);
}
```

> `Coords` import: add `use crate::location::Coords;` at the top of `scope.rs`. Confirm `draw_line`, `draw_circle`, `draw_circle_lines`, `Color::from_rgba` are already imported (macroquad prelude) — `draw_alerts`/`draw_borders` already use similar primitives; reuse their imports.

- [ ] **Step 2: Extend `draw_scope_to_texture`**

Change the signature to add a trailing parameter:

```rust
    nhc: Option<(&NhcBundle, &NhcOverlayState)>,
    user: Option<Coords>,
) {
```

And in the overlay pass, after the NHC block:

```rust
    // ── User location marker ─────────────────────────────────────
    if let Some(user) = user {
        draw_user_location(user, site, center_x, center_y, px_per_km);
    }
```

- [ ] **Step 3: Update the call site in main.rs**

At `app/src/main.rs` ~line 729, add the trailing argument:

```rust
        scope::draw_scope_to_texture(
            state.radar_texture.as_ref(),
            site,
            state.pan_km,
            state.zoom,
            Some((&state.borders, state.show_borders)),
            Some((&state.alerts, state.show_alerts)),
            state.nhc_bundle.as_ref().map(|b| (b, &state.nhc_overlays)),
            if state.show_location { state.user_location } else { None },
        );
```

- [ ] **Step 4: Build + visual smoke**

Run: `cargo build -p rustywx`
Then: `cd app && ../target/debug/rustywx` — (marker won't show until `show_location` is on and a location is set; that wiring lands in Tasks 8–9. Just confirm it builds and runs.)
Expected: compiles, runs.

- [ ] **Step 5: Commit**

```bash
git add app/src/scope.rs app/src/main.rs
git commit -m "feat: draw user-location marker on scope"
```

---

### Task 8: Centering + resolver polling in main loop

**Files:**
- Modify: `app/src/main.rs`

**Interfaces:**
- Consumes: `state.location_resolver`, `state.user_location`, `state.pan_km`, `geo::point_to_km_offset`, `state.cache.save_settings`.
- Produces: `fn recenter_on_user(state: &mut AppState)` helper.

- [ ] **Step 1: Add the recenter helper**

In `app/src/main.rs` (near `fn show_toast`), add:

```rust
/// Set pan so the user's location sits at screen center. No-op if unknown.
fn recenter_on_user(state: &mut AppState) {
    if let Some(user) = state.user_location {
        let site = &geo::RADAR_SITES[state.site_index];
        let off = geo::point_to_km_offset(site.lat, site.lon, (user.lat, user.lon));
        state.pan_km = (-off.x, -off.y);
    }
}
```

- [ ] **Step 2: Poll the resolver each frame**

In the main loop's update section (alongside the alerts/NHC poll blocks, ~line 630), add:

```rust
        // ── Poll the location resolver (Detect in progress) ───────
        if let Some(coords) = state.location_resolver.poll(now) {
            state.user_location = Some(coords);
            state.settings.user_lat = Some(coords.lat);
            state.settings.user_lon = Some(coords.lon);
            state.cache.save_settings(&state.settings);
            if state.settings.center_on_location {
                recenter_on_user(state);
            }
        }
        // Surface resolver failures as a toast (once, when it lands).
        match state.location_resolver.status() {
            rustywx::location::LocationStatus::Offline => {
                state.toast = Some(toast_widget::Toast::new("Location: offline", now));
            }
            rustywx::location::LocationStatus::Denied => {
                state.toast = Some(toast_widget::Toast::new("Location permission denied", now));
            }
            rustywx::location::LocationStatus::NotFound => {
                state.toast = Some(toast_widget::Toast::new("ZIP not found", now));
            }
            _ => {}
        }
```

> The status match above fires every frame while in a terminal error state, which would re-raise the toast. Guard it: only raise when the phase just became terminal. Simplest fix — track a `location_error_shown: bool` on `AppState` set when raised and cleared on the next `detect()`. Add that field (default `false`) in Task 6's state block if not already present, and gate each arm with `&& !state.location_error_shown`, setting it true inside. (If you added it here, `git add app/src/state.rs` too.)

- [ ] **Step 3: Build + run**

Run: `cargo build -p rustywx && cd app && ../target/debug/rustywx`
Expected: compiles, runs (no behavior change yet without UI triggers).

- [ ] **Step 4: Commit**

```bash
git add app/src/main.rs app/src/state.rs
git commit -m "feat: poll location resolver + recenter on user"
```

---

### Task 9: Top-bar "Location" toggle button

**Files:**
- Modify: `app/src/widgets/mod.rs` (add location-pin `nf` glyph)
- Modify: `app/src/main.rs` (button in controls bar + press handler)

**Interfaces:**
- Consumes: `state.show_location`, `state.settings.show_location`, `hover_tint`, `nf`, `SYMBOL_FONT`.
- Produces: button id `"btn-location"`; `nf::LOCATION` glyph constant.

- [ ] **Step 1: Add the Nerd Font glyph**

In `app/src/widgets/mod.rs`, in the `nf` module:

```rust
    pub const LOCATION: &str = "\u{f124}"; // fa-location-arrow
```

> Prefer a map-marker glyph if present in the bundled Nerd Font: `fa-map-marker` is `\u{f041}`. Pick whichever renders (the font is Nerd Fonts Symbols); `\u{f124}` (location-arrow) and `\u{f041}` (map-marker) are both standard FA. Verify visually in Step 4.

- [ ] **Step 2: Add the toggle button to the controls bar**

In `app/src/main.rs`, in the controls-bar `.children(|ui| { … })` (after the NHC button, before the window-controls spacer), add — mirroring the `btn-alerts` pattern:

```rust
                        let loc_bg = hover_tint(
                            &state.hovered_ids,
                            "btn-location",
                            if state.show_location { 0x0dc5b8 } else { 0x1E1B1B },
                            0x1E1B1B,
                        );
                        let loc_label = if state.show_location { "Location \u{f00c}" } else { "Location" };
                        ui.element()
                            .id("btn-location")
                            .width(fit!())
                            .height(fixed!(if is_mobile { 44.0 } else { 24.0 }))
                            .background_color(loc_bg)
                            .corner_radius(4.0)
                            .layout(|layout| layout.padding((0, 8, 0, 8)).align(CenterX, CenterY))
                            .accessibility(|a| a.button("Location").checked(state.show_location))
                            .children(|ui| {
                                ui.text(loc_label, |text| text.font_size(12).color(0xE8E0DC));
                            });
```

> The `\u{f00c}` check glyph inside a body-font string will tofu (see prior glyph work). Match the existing "Borders ✓/Alerts ✓" convention exactly — those use the literal `✓` (U+2713) rendered in the body font. If that renders in this build, use `"Location ✓"`; otherwise split the glyph into a separate `.text()` with `.font(&SYMBOL_FONT)` using `nf::` check. Use whatever the neighboring Alerts button does — copy it verbatim.

- [ ] **Step 3: Add the press handler**

Near the `btn-alerts` handler (~line 1827):

```rust
    if ply.is_just_pressed("btn-location") {
        state.show_location = !state.show_location;
        state.settings.show_location = state.show_location;
        state.cache.save_settings(&state.settings);
    }
```

- [ ] **Step 4: Build + visual verify**

Run: `cargo build -p rustywx && cd app && ../target/debug/rustywx`
Then focus the window and screenshot (`osascript` activate + `screencapture -x`), confirm a "Location" toggle appears in the top bar and the glyph renders (no tofu box). Toggle it on; with no location set yet, no marker shows.

- [ ] **Step 5: Commit**

```bash
git add app/src/widgets/mod.rs app/src/main.rs
git commit -m "feat: top-bar location toggle button"
```

---

### Task 10: Settings "My Location" section (input, detect, center, status)

**Files:**
- Modify: `app/src/widgets/settings.rs` (new section + widget ids + editable field render)
- Modify: `app/src/main.rs` (char capture for the field, press handlers, detect trigger, center-checkbox handling, pass status/input to `settings::draw`)

**Interfaces:**
- Consumes: `state.settings.location_input`, `state.location_input_focused`, `state.location_resolver`, `LocationStatus`, `recenter_on_user`.
- Produces widget ids: `LOCATION_INPUT_ID`, `LOCATION_DETECT_ID`, `CENTER_TOGGLE_ID`.

- [ ] **Step 1: Add widget ids + status formatter**

In `app/src/widgets/settings.rs`, with the other `pub const … _ID`:

```rust
pub const LOCATION_INPUT_ID: &str = "settings-location-input";
pub const LOCATION_DETECT_ID: &str = "settings-location-detect";
pub const CENTER_TOGGLE_ID: &str = "settings-toggle-center";
```

- [ ] **Step 2: Extend `settings::draw` signature + section**

Change `pub fn draw(ui, settings, current_site_id)` to also take the input text, focus flag, and a preformatted status string:

```rust
pub fn draw(
    ui: &mut Ui<'_, ()>,
    settings: &Settings,
    current_site_id: &str,
    location_input: &str,
    location_focused: bool,
    location_status: &str,
) {
```

Bump `modal_h` from `380.0` to `470.0` (room for the new section).

In the content block (after the Dyslexia row, before Animation level), add:

```rust
                    // ── My Location ──────────────────────────────
                    ui.text("My Location", |t| t.font_size(12).color(TEXT_COLOR));
                    row(ui, "Coords or ZIP", |ui| {
                        let border = if location_focused { 0x0dc5b8 } else { 0x333333 };
                        let shown = if location_input.is_empty() && !location_focused {
                            "e.g. 33.75, -84.39 or 30301"
                        } else {
                            location_input
                        };
                        ui.element()
                            .id(LOCATION_INPUT_ID)
                            .width(grow!())
                            .height(fixed!(24.0))
                            .background_color(0x11151c)
                            .border_color(border)
                            .border_width(1.0)
                            .corner_radius(4.0)
                            .layout(|l| l.padding((0, 8, 0, 8)).align(Left, CenterY))
                            .accessibility(|a| a.button("Edit location"))
                            .children(|ui| {
                                ui.text(shown, |t| t.font_size(11).color(
                                    if location_input.is_empty() && !location_focused { MUTED_COLOR } else { TEXT_COLOR }
                                ));
                            });
                    });
                    row(ui, "", |ui| {
                        ui.element()
                            .id(LOCATION_DETECT_ID)
                            .width(fit!())
                            .height(fixed!(24.0))
                            .background_color(INACTIVE_BG)
                            .corner_radius(4.0)
                            .layout(|l| l.padding((0, 10, 0, 10)).align(CenterX, CenterY))
                            .accessibility(|a| a.button("Detect location"))
                            .children(|ui| {
                                ui.text("Detect", |t| t.font_size(11).color(TEXT_COLOR));
                            });
                    });
                    row(ui, "Center map on my location", |ui| {
                        bool_toggle(ui, CENTER_TOGGLE_ID, "Center map on my location", settings.center_on_location);
                    });
                    ui.text(location_status, |t| t.font_size(10).color(MUTED_COLOR));
```

> Verify `border_color` / `border_width` exist on the Ply element builder (grep `app/src/widgets` for prior use). If not available, drop the border and instead tint the background when focused (`if location_focused { 0x1a2730 } else { 0x11151c }`).

- [ ] **Step 3: Update the `settings::draw` call in main.rs**

Find the `settings_widget::draw(...)` call and pass the new args. Build the status string there:

```rust
            let loc_status = match state.location_resolver.status() {
                rustywx::location::LocationStatus::Idle => match state.user_location {
                    Some(c) => format!("{:.4}, {:.4}", c.lat, c.lon),
                    None => "Not set".to_string(),
                },
                rustywx::location::LocationStatus::Detecting => "Detecting…".to_string(),
                rustywx::location::LocationStatus::Resolved(c, src) => {
                    format!("{:.4}, {:.4} ({:?})", c.lat, c.lon, src)
                }
                rustywx::location::LocationStatus::Denied => "Permission denied".to_string(),
                rustywx::location::LocationStatus::Offline => "Offline".to_string(),
                rustywx::location::LocationStatus::NotFound => "ZIP not found".to_string(),
                rustywx::location::LocationStatus::Invalid => "Invalid input".to_string(),
            };
            settings_widget::draw(
                ui,
                &state.settings,
                &geo::RADAR_SITES[state.site_index].id,
                &state.settings.location_input,
                state.location_input_focused,
                &loc_status,
            );
```

> Confirm the exact existing call args (`current_site_id` is likely `geo::RADAR_SITES[state.site_index].id` — match the current code).

- [ ] **Step 4: Add focus + char capture + handlers in main.rs**

In the settings-panel input-handling block (where `USE_CURRENT_SITE_ID` etc. are handled), add:

```rust
        // Focus the location field on click; blur when clicking elsewhere.
        if ply.is_just_pressed(settings_widget::LOCATION_INPUT_ID) {
            state.location_input_focused = true;
        }
        // Capture typing while focused.
        if state.location_input_focused {
            while let Some(ch) = get_char_pressed() {
                if !ch.is_control() && state.settings.location_input.len() < 32 {
                    state.settings.location_input.push(ch);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                state.settings.location_input.pop();
            }
            if is_key_pressed(KeyCode::Enter) {
                state.location_input_focused = false;
                state.location_error_shown = false;
                state.location_resolver.detect(&state.settings.location_input, now);
                state.cache.save_settings(&state.settings);
            }
        }
        if ply.is_just_pressed(settings_widget::LOCATION_DETECT_ID) {
            state.location_input_focused = false;
            state.location_error_shown = false;
            if let Some(c) = state.location_resolver.detect(&state.settings.location_input, now) {
                state.user_location = Some(c);
                state.settings.user_lat = Some(c.lat);
                state.settings.user_lon = Some(c.lon);
                if state.settings.center_on_location { recenter_on_user(state); }
            }
            state.cache.save_settings(&state.settings);
        }
        if ply.is_just_pressed(settings_widget::CENTER_TOGGLE_ID) {
            state.settings.center_on_location = !state.settings.center_on_location;
            state.cache.save_settings(&state.settings);
            if state.settings.center_on_location {
                recenter_on_user(state);
            }
        }
```

> `get_char_pressed`, `KeyCode::Backspace`, `KeyCode::Enter` come from the macroquad prelude already imported for the other key handlers. `now` is the current time already computed in the loop. If the `Escape`-closes-settings handler also fires on Enter/typing, ensure the char-capture block runs before the Escape handler and that typing doesn't close the modal.

- [ ] **Step 5: Build + full manual smoke**

Run: `cargo build -p rustywx && cd app && ../target/debug/rustywx`
Verify (focus window, screenshot between steps):
1. Open Settings → "My Location" section present, modal not clipped.
2. Click the field → border/tint changes; type `33.75, -84.39`; press Enter → status shows `33.7500, -84.3900 (Manual)`.
3. Toggle top-bar "Location" on → cyan crosshair appears at Atlanta on the KJGX/KFFC scope.
4. Check "Center map on my location" → scope recenters so the marker is at screen center.
5. Clear the field, press Detect → status `Detecting…` then `(Ip)` coords (CoreLocation is denied on the bare binary → IP fallback), or a toast on offline.

- [ ] **Step 6: fmt + clippy + full test run**

```bash
cargo fmt -p rustywx
cargo clippy -p rustywx
cargo test -p rustywx
```
Expected: no warnings, all tests pass.

- [ ] **Step 7: Commit**

```bash
git add app/src/widgets/settings.rs app/src/main.rs
git commit -m "feat: settings My Location section (input/detect/center/status)"
```

---

## Self-Review Notes

- **Spec coverage:** providers (Tasks 4–5), IP/ZIP (Task 2), manual coords/ZIP override (Tasks 1, 5, 10), marker (Task 7), centering (Task 8), top-bar toggle (Task 9), settings section + status (Task 10), persistence + startup restore (Tasks 3, 6), privacy (Detect-only egress — Tasks 5, 10), tests (Tasks 1–3, 5). All spec sections map to a task.
- **`location_error_shown` field:** introduced in Task 8 Step 2 note; ensure it is added to `AppState` (default `false`) — fold into Task 6's state block if executing in order, else add in Task 8 and `git add app/src/state.rs`.
- **Verification flags:** each `>` note marks an API detail to confirm against the current code/crate during implementation (net URL lifetime, objc2 method receiver, `border_color` availability, check-glyph rendering, exact `settings::draw` call args). These are confirmations, not open design questions.

## Deferred (not in this plan)

- `.app` bundle + `NSLocationWhenInUseUsageDescription` (unblocks real Mac location).
- Linux gpsd provider (drops into `system_location.rs` behind `#[cfg(target_os="linux")]`).
- Blinking caret / text selection in the location field (minimal capture only).
