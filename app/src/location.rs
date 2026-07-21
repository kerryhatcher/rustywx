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
