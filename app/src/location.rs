//! User-location resolution: input parsing, IP/ZIP geolocation over Ply `net`,
//! and a fallback-chain state machine (system → IP → manual).

use serde_json::Value;

const IP_NET_ID: &str = "loc-ip";
const ZIP_NET_ID: &str = "loc-zip";
const IP_URL: &str = "https://ipapi.co/json/";
const USER_AGENT: &str = "rustywx (github.com/rustywx)";

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
}
