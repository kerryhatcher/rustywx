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
}
