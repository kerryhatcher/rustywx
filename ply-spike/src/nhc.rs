//! NHC tropical cyclone data: fetches storm metadata, text products, and
//! image products from the NHC CurrentStorms.json API, plus GIS overlays
//! (forecast cones, tracks, watches/warnings) from NOAA's ArcGIS MapServer.
//!
//! Stage 5: Replaces `ureq` with Ply `net`. The fetch is a two-phase
//! fire-and-poll state machine:
//!   Phase 1 — fire CurrentStorms.json + 4 GIS MapServer layer requests.
//!   Phase 2 — when CurrentStorms.json arrives, parse storm metas and fire
//!              per-storm text product, image, and KMZ requests.
//! All requests are polled each frame; the bundle is assembled when every
//! pending request has completed.

use crate::borders::Ring;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// How often to refresh NHC data (every 5 minutes).
pub const POLL_INTERVAL: Duration = Duration::from_secs(300);

/// Base URL for the NHC tropical weather summary MapServer.
const MAPSERVER: &str = "https://mapservices.weather.noaa.gov/tropical/rest/services/tropical/NHC_tropical_weather_summary/MapServer";

/// NHC active storms JSON API.
const CURRENT_STORMS_URL: &str = "https://www.nhc.noaa.gov/CurrentStorms.json";

/// Net request IDs for phase-1 requests.
pub const NET_ID_CURRENT_STORMS: &str = "nhc-current-storms";
pub const NET_ID_GIS_PREFIX: &str = "nhc-gis-"; // + layer number

// ── Data types ──────────────────────────────────────────────────────────

/// Storm metadata from the NHC CurrentStorms.json API.
#[derive(Clone, Serialize, Deserialize)]
pub struct StormMeta {
    pub id: String,
    pub name: String,
    pub classification: String,
    /// Maximum sustained wind in knots.
    pub intensity_kt: u32,
    /// Minimum central pressure in millibars.
    pub pressure_mb: u32,
    /// Current position.
    pub lat: f64,
    pub lon: f64,
    /// Movement direction in degrees (meteorological).
    pub movement_dir_deg: Option<u32>,
    /// Movement speed in knots.
    pub movement_speed_kt: Option<u32>,
    /// ISO 8601 last update timestamp.
    pub last_update: String,
    /// NHC bin number (e.g. "AT2", "EP5").
    pub bin_number: String,
    /// Advisory number.
    pub advisory_num: String,
    /// URL to the NHC graphics page for this storm.
    pub graphics_url: String,
    /// Graphics issuance timestamp as DDHHMM (e.g. "192344") for image URLs.
    pub graphics_issuance: String,
    /// KMZ URL for earliest arrival time of 34-kt winds.
    pub earliest_arrival_kmz: String,
    /// KMZ URL for most likely arrival time of 34-kt winds.
    pub most_likely_arrival_kmz: String,
    /// KMZ URL for 34-kt wind speed probabilities.
    pub wind_probs_34kt_kmz: String,
    /// KMZ URL for 50-kt wind speed probabilities.
    pub wind_probs_50kt_kmz: String,
    /// KMZ URL for 64-kt wind speed probabilities.
    pub wind_probs_64kt_kmz: String,
}

/// A text product (advisory, discussion, etc.) with its content.
#[derive(Clone, Serialize, Deserialize)]
pub struct TextProduct {
    pub title: String,
    pub content: String,
    pub url: String,
}

/// An image product reference with optional downloaded bytes.
#[derive(Clone, Serialize, Deserialize)]
pub struct ImageProduct {
    pub title: String,
    pub url: String,
    /// Raw image bytes (PNG/JPEG), populated after download.
    #[serde(skip)]
    pub data: Option<Vec<u8>>,
}

/// A tropical cyclone with GIS overlays (from MapServer).
#[derive(Clone, Serialize, Deserialize)]
pub struct StormGis {
    pub name: String,
    pub storm_type: String,
    pub basin: String,
    pub advisory_num: String,
    pub advisory_date: String,
    /// Forecast cone polygon rings.
    pub cone: Vec<Ring>,
    /// Forecast track line (single ring of points).
    pub track: Vec<Ring>,
    /// Forecast points: (lat, lon, label) for each point along the track.
    pub points: Vec<(f64, f64, String)>,
    /// Watch/warning coastal segments: (lat, lon) pairs with a type label.
    pub watches_warnings: Vec<(Ring, String)>,
}

/// Complete NHC data bundle.
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct NhcBundle {
    pub metas: Vec<StormMeta>,
    pub gis_storms: Vec<StormGis>,
    pub text_products: Vec<(String, Vec<TextProduct>)>,
    pub image_products: Vec<(String, Vec<ImageProduct>)>,
    /// Wind speed probability contours (34 kt threshold).
    pub wind_probs_34kt: Vec<WindProbContour>,
    /// Wind speed probability contours (50 kt threshold).
    pub wind_probs_50kt: Vec<WindProbContour>,
    /// Wind speed probability contours (64 kt threshold).
    pub wind_probs_64kt: Vec<WindProbContour>,
    /// Earliest reasonable arrival time of 34-kt winds.
    pub earliest_arrival: Vec<ArrivalTimeContour>,
    /// Most likely arrival time of 34-kt winds.
    pub most_likely_arrival: Vec<ArrivalTimeContour>,
}

/// A wind speed probability contour from a KMZ file.
#[derive(Clone, Serialize, Deserialize)]
pub struct WindProbContour {
    /// Human-readable label (e.g. "5-10", "20-30", ">90%").
    pub label: String,
    /// Lower bound of the probability range (0–100).
    pub prob_low: u32,
    /// Upper bound of the probability range (0–100).
    pub prob_high: u32,
    /// Wind threshold in knots (34, 50, or 64).
    pub threshold_kt: u32,
    /// Polygon rings for this contour.
    pub rings: Vec<Ring>,
}

/// An arrival-time contour from a KMZ file.
#[derive(Clone, Serialize, Deserialize)]
pub struct ArrivalTimeContour {
    /// Human-readable label (e.g. "Mon 8 am", "Tue 2 pm").
    pub label: String,
    /// Polygon or line rings.
    pub rings: Vec<Ring>,
}

// ── CurrentStorms.json parsing ──────────────────────────────────────────

#[derive(Deserialize)]
struct CurrentStormsResponse {
    #[serde(rename = "activeStorms")]
    active_storms: Vec<CurrentStorm>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct CurrentStorm {
    id: String,
    #[serde(rename = "binNumber")]
    bin_number: String,
    name: String,
    classification: String,
    intensity: String,
    pressure: String,
    #[serde(rename = "latitudeNumeric")]
    latitude_numeric: f64,
    #[serde(rename = "longitudeNumeric")]
    longitude_numeric: f64,
    #[serde(rename = "movementDir")]
    movement_dir: Option<u32>,
    #[serde(rename = "movementSpeed")]
    movement_speed: Option<u32>,
    #[serde(rename = "lastUpdate")]
    last_update: String,
    #[serde(rename = "publicAdvisory")]
    public_advisory: Option<ProductRef>,
    #[serde(rename = "forecastAdvisory")]
    forecast_advisory: Option<ProductRef>,
    #[serde(rename = "forecastDiscussion")]
    forecast_discussion: Option<ProductRef>,
    #[serde(rename = "windSpeedProbabilities")]
    wind_speed_probabilities: Option<ProductRef>,
    #[serde(rename = "forecastGraphics")]
    forecast_graphics: Option<ProductRef>,
    #[serde(rename = "earliestArrivalTimeTSWindsGIS")]
    earliest_arrival_gis: Option<KmzRef>,
    #[serde(rename = "mostLikelyTimeTSWindsGIS")]
    most_likely_arrival_gis: Option<KmzRef>,
    #[serde(rename = "windSpeedProbabilitiesGIS")]
    wind_speed_probs_gis: Option<WindProbGisRef>,
}

#[derive(Deserialize)]
struct KmzRef {
    #[serde(rename = "kmzFile")]
    kmz_file: String,
}

#[derive(Deserialize)]
struct WindProbGisRef {
    #[serde(rename = "kmzFile34kt")]
    kmz_file_34kt: Option<String>,
    #[serde(rename = "kmzFile50kt")]
    kmz_file_50kt: Option<String>,
    #[serde(rename = "kmzFile64kt")]
    kmz_file_64kt: Option<String>,
}

#[derive(Deserialize)]
struct ProductRef {
    #[serde(rename = "advNum")]
    adv_num: String,
    url: String,
    #[serde(rename = "fileUpdateTime")]
    file_update_time: Option<String>,
}

/// Parse the CurrentStorms.json response body into storm metadata.
pub fn parse_current_storms(body: &str) -> Result<Vec<StormMeta>> {
    let response: CurrentStormsResponse =
        serde_json::from_str(body).map_err(|e| anyhow!("parsing CurrentStorms.json: {e}"))?;

    let metas: Vec<StormMeta> = response
        .active_storms
        .into_iter()
        .map(|s| {
            let adv_num = s
                .public_advisory
                .as_ref()
                .map(|p| p.adv_num.clone())
                .unwrap_or_default();
            let graphics_url = s
                .forecast_graphics
                .as_ref()
                .map(|p| p.url.clone())
                .unwrap_or_default();
            let graphics_issuance = s
                .forecast_graphics
                .as_ref()
                .and_then(|p| p.file_update_time.as_deref())
                .and_then(iso_to_ddhhmm)
                .unwrap_or_default();
            let earliest_arrival_kmz = s
                .earliest_arrival_gis
                .as_ref()
                .map(|a| a.kmz_file.clone())
                .unwrap_or_default();
            let most_likely_arrival_kmz = s
                .most_likely_arrival_gis
                .as_ref()
                .map(|a| a.kmz_file.clone())
                .unwrap_or_default();
            let wind_probs = s.wind_speed_probs_gis.as_ref();
            let wind_probs_34kt_kmz = wind_probs
                .and_then(|w| w.kmz_file_34kt.clone())
                .unwrap_or_default();
            let wind_probs_50kt_kmz = wind_probs
                .and_then(|w| w.kmz_file_50kt.clone())
                .unwrap_or_default();
            let wind_probs_64kt_kmz = wind_probs
                .and_then(|w| w.kmz_file_64kt.clone())
                .unwrap_or_default();
            StormMeta {
                id: s.id,
                name: s.name,
                classification: s.classification,
                intensity_kt: s.intensity.parse().unwrap_or(0),
                pressure_mb: s.pressure.parse().unwrap_or(0),
                lat: s.latitude_numeric,
                lon: s.longitude_numeric,
                movement_dir_deg: s.movement_dir,
                movement_speed_kt: s.movement_speed,
                last_update: s.last_update,
                bin_number: s.bin_number,
                advisory_num: adv_num,
                graphics_url,
                graphics_issuance,
                earliest_arrival_kmz,
                most_likely_arrival_kmz,
                wind_probs_34kt_kmz,
                wind_probs_50kt_kmz,
                wind_probs_64kt_kmz,
            }
        })
        .collect();

    Ok(metas)
}

/// Convert an ISO 8601 timestamp like "2026-07-19T23:44:32.968Z"
/// to the DDHHMM format used in NHC image URLs (e.g. "192344").
fn iso_to_ddhhmm(iso: &str) -> Option<String> {
    let t_pos = iso.find('T')?;
    let date_part = &iso[..t_pos];
    let time_part = &iso[t_pos + 1..];

    let day: &str = date_part.rsplit('-').next()?;
    let hour = &time_part[..2];
    let minute = &time_part[3..5];

    Some(format!("{day}{hour}{minute}"))
}

// ── Text product parsing ────────────────────────────────────────────────

/// Extract text from the first <pre>…</pre> block in HTML.
fn extract_pre_content(html: &str) -> Option<String> {
    let start = html.find("<pre>")? + 5;
    let end = html[start..].find("</pre>")?;
    let raw = &html[start..start + end];
    let cleaned = raw
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&nbsp;", " ");
    Some(cleaned.trim().to_string())
}

/// Crude HTML tag stripper fallback.
fn strip_html(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            out.push(c);
        }
    }
    out.trim().to_string()
}

/// Parse a text product from an HTML response body.
pub fn parse_text_product(title: &str, url: &str, body: &str) -> TextProduct {
    let content = extract_pre_content(body).unwrap_or_else(|| strip_html(body));
    TextProduct {
        title: title.to_string(),
        content,
        url: url.to_string(),
    }
}

/// Construct text product URLs for a storm based on known NHC patterns.
pub fn text_product_urls(meta: &StormMeta) -> Vec<(&'static str, String)> {
    let basin_prefix = if meta.bin_number.starts_with("AT") {
        "AT"
    } else if meta.bin_number.starts_with("EP") {
        "EP"
    } else if meta.bin_number.starts_with("CP") {
        "CP"
    } else {
        return Vec::new();
    };

    let bin_num = &meta.bin_number[2..];

    vec![
        (
            "Public Advisory",
            format!("https://www.nhc.noaa.gov/text/MIATCP{basin_prefix}{bin_num}.shtml"),
        ),
        (
            "Forecast Discussion",
            format!("https://www.nhc.noaa.gov/text/MIATCD{basin_prefix}{bin_num}.shtml"),
        ),
        (
            "Forecast Advisory",
            format!("https://www.nhc.noaa.gov/text/MIATCM{basin_prefix}{bin_num}.shtml"),
        ),
    ]
}

// ── Image product URL construction ───────────────────────────────────────

/// Construct image product URLs for a storm based on known NHC patterns.
pub fn construct_image_products(meta: &StormMeta) -> Vec<ImageProduct> {
    let mut products = Vec::new();

    let storm_id = meta.id.to_uppercase();
    let bin = format_bin(&meta.bin_number);
    let ts = &meta.graphics_issuance;

    if ts.is_empty() {
        return products;
    }

    products.push(ImageProduct {
        title: "5-Day Cone".to_string(),
        url: format!(
            "https://www.nhc.noaa.gov/storm_graphics/{bin}/refresh/{storm_id}_5day_cone+png/{ts}_5day_cone.png"
        ),
        data: None,
    });

    products.push(ImageProduct {
        title: "Key Messages".to_string(),
        url: format!(
            "https://www.nhc.noaa.gov/storm_graphics/{bin}/refresh/{storm_id}_key_messages+png/{ts}_key_messages.png"
        ),
        data: None,
    });

    products.push(ImageProduct {
        title: "Wind Field".to_string(),
        url: format!(
            "https://www.nhc.noaa.gov/storm_graphics/{bin}/refresh/{storm_id}_current_wind+png/{ts}_current_wind.png"
        ),
        data: None,
    });

    products.push(ImageProduct {
        title: "Wind Probs 34kt".to_string(),
        url: format!(
            "https://www.nhc.noaa.gov/storm_graphics/{bin}/refresh/{storm_id}_wind_probs_34_F120+png/{ts}_wind_probs_34_F120.png"
        ),
        data: None,
    });

    products.push(ImageProduct {
        title: "Peak Surge".to_string(),
        url: format!(
            "https://www.nhc.noaa.gov/storm_graphics/{bin}/refresh/{storm_id}_peak_surge+png/{ts}_peak_surge.png"
        ),
        data: None,
    });

    let short_id = &storm_id[..storm_id.len().min(6)];
    products.push(ImageProduct {
        title: "Rainfall".to_string(),
        url: format!(
            "https://www.nhc.noaa.gov/storm_graphics/{bin}/refresh/{short_id}WPCQPF+gif/{ts}_WPCQPF.gif"
        ),
        data: None,
    });

    products
}

/// Format a bin number like "AT2" to "AT02" (with leading zero).
fn format_bin(bin: &str) -> String {
    if bin.len() >= 4 {
        return bin.to_string();
    }
    let split = bin.find(|c: char| c.is_ascii_digit()).unwrap_or(bin.len());
    let letters = &bin[..split];
    let digits = &bin[split..];
    if let Ok(num) = digits.parse::<u32>() {
        format!("{letters}{num:02}")
    } else {
        bin.to_string()
    }
}

// ── KMZ / KML parsing ──────────────────────────────────────────────────

/// Decompress a KMZ (ZIP) byte slice and return the KML content.
pub fn extract_kml_from_kmz(bytes: &[u8]) -> Result<String> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| anyhow!("opening KMZ zip: {e}"))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();
        if name.ends_with(".kml") {
            let mut kml = String::new();
            std::io::Read::read_to_string(&mut file, &mut kml)?;
            return Ok(kml);
        }
    }
    Err(anyhow!("no KML file found in KMZ"))
}

/// Parse wind probability contours from KML content.
pub fn parse_wind_prob_kml(kml: &str, threshold_kt: u32) -> Vec<WindProbContour> {
    let mut contours = Vec::new();

    let mut search_start = 0;
    while let Some(pm_start) = kml[search_start..].find("<Placemark>") {
        let abs_start = search_start + pm_start;
        let Some(pm_end) = kml[abs_start..].find("</Placemark>") else {
            break;
        };
        let abs_end = abs_start + pm_end + "</Placemark>".len();
        let placemark = &kml[abs_start..abs_end];
        search_start = abs_end;

        let Some(name) = extract_xml_text(placemark, "name") else {
            continue;
        };
        let name = name.trim().to_string();
        if name.is_empty() {
            continue;
        }

        let (prob_low, prob_high) = parse_prob_range(&name);
        let rings = extract_kml_rings(placemark);
        if rings.is_empty() {
            continue;
        }

        contours.push(WindProbContour {
            label: name,
            prob_low,
            prob_high,
            threshold_kt,
            rings,
        });
    }

    contours
}

/// Parse arrival time contours from KML content.
pub fn parse_arrival_kml(kml: &str) -> Vec<ArrivalTimeContour> {
    let mut contours = Vec::new();

    let mut search_start = 0;
    while let Some(pm_start) = kml[search_start..].find("<Placemark>") {
        let abs_start = search_start + pm_start;
        let Some(pm_end) = kml[abs_start..].find("</Placemark>") else {
            break;
        };
        let abs_end = abs_start + pm_end + "</Placemark>".len();
        let placemark = &kml[abs_start..abs_end];
        search_start = abs_end;

        // NHC arrival time KML placemarks often lack a <name> element;
        // the label is embedded in the <description> CDATA instead.
        // Fall back to stripping HTML from the description, then to an
        // empty string so the contour is still drawn.
        let label = extract_xml_text(placemark, "name")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                extract_xml_text(placemark, "description")
                    .map(|s| strip_html(&s).trim().to_string())
                    .filter(|s| !s.is_empty())
            })
            .unwrap_or_default();

        let rings = extract_kml_rings(placemark);
        if rings.is_empty() {
            continue;
        }

        contours.push(ArrivalTimeContour { label, rings });
    }

    contours
}

/// Extract the text content of an XML element.
fn extract_xml_text(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)?;
    Some(xml[start..start + end].to_string())
}

/// Parse a probability range string like "5-10", "<5%", ">90%".
fn parse_prob_range(s: &str) -> (u32, u32) {
    let cleaned = s
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace('%', "")
        .trim()
        .to_string();

    if let Some(rest) = cleaned.strip_prefix('<') {
        let val: u32 = rest.trim().parse().unwrap_or(0);
        return (0, val);
    }
    if let Some(rest) = cleaned.strip_prefix('>') {
        let val: u32 = rest.trim().parse().unwrap_or(100);
        return (val, 100);
    }
    if let Some((low, high)) = cleaned.split_once('-') {
        let low: u32 = low.trim().parse().unwrap_or(0);
        let high: u32 = high.trim().parse().unwrap_or(100);
        return (low, high);
    }
    (0, 0)
}

/// Extract polygon/line rings from a KML Placemark fragment.
fn extract_kml_rings(placemark: &str) -> Vec<Ring> {
    let mut rings = Vec::new();

    let mut search = 0;
    while let Some(coord_start) = placemark[search..].find("<coordinates>") {
        let abs_start = search + coord_start + "<coordinates>".len();
        let Some(coord_end) = placemark[abs_start..].find("</coordinates>") else {
            break;
        };
        let abs_end = abs_start + coord_end;
        let coords_str = &placemark[abs_start..abs_end];
        search = abs_end + "</coordinates>".len();

        let points: Vec<(f64, f64)> = coords_str
            .split_whitespace()
            .filter_map(|tuple| {
                let parts: Vec<&str> = tuple.split(',').collect();
                if parts.len() >= 2 {
                    let lon: f64 = parts[0].trim().parse().ok()?;
                    let lat: f64 = parts[1].trim().parse().ok()?;
                    Some((lat, lon))
                } else {
                    None
                }
            })
            .collect();

        if points.len() >= 3 {
            rings.push(points);
        }
    }

    rings
}

// ── GeoJSON parsing helpers (MapServer) ──────────────────────────────────

/// Parse a GeoJSON feature collection from a MapServer layer response.
pub fn parse_gis_layer(body: &str) -> Result<Value> {
    serde_json::from_str(body).map_err(|e| anyhow!("parsing GIS layer: {e}"))
}

/// Parse a GeoJSON Polygon coordinates array into one or more rings.
fn parse_polygon_rings(coords: &Value) -> Option<Vec<Ring>> {
    let rings_json = coords.as_array()?;
    let mut rings = Vec::new();
    for ring_json in rings_json {
        let points: Vec<(f64, f64)> = ring_json
            .as_array()?
            .iter()
            .filter_map(|pt| {
                let arr = pt.as_array()?;
                Some((arr.get(1)?.as_f64()?, arr.first()?.as_f64()?))
            })
            .collect();
        if points.len() >= 3 {
            rings.push(points);
        }
    }
    if rings.is_empty() { None } else { Some(rings) }
}

/// Parse a GeoJSON LineString coordinates array into a ring.
fn parse_line_ring(coords: &Value) -> Option<Ring> {
    let points: Vec<(f64, f64)> = coords
        .as_array()?
        .iter()
        .filter_map(|pt| {
            let arr = pt.as_array()?;
            Some((arr.get(1)?.as_f64()?, arr.first()?.as_f64()?))
        })
        .collect();
    if points.len() >= 2 {
        Some(points)
    } else {
        None
    }
}

fn prop_str(feature: &Value, key: &str) -> String {
    feature
        .get("properties")
        .and_then(|p| p.get(key))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

/// Extract rings from a single GeoJSON feature.
fn extract_rings_for_feature(feature: &Value) -> Vec<Ring> {
    let Some(geometry) = feature.get("geometry") else {
        return Vec::new();
    };
    let Some(geom_type) = geometry.get("type").and_then(Value::as_str) else {
        return Vec::new();
    };
    let Some(coords) = geometry.get("coordinates") else {
        return Vec::new();
    };
    match geom_type {
        "Polygon" => parse_polygon_rings(coords).unwrap_or_default(),
        "MultiPolygon" => {
            let mut rings = Vec::new();
            if let Some(polygons) = coords.as_array() {
                for polygon in polygons {
                    if let Some(r) = parse_polygon_rings(polygon) {
                        rings.extend(r);
                    }
                }
            }
            rings
        }
        "LineString" => parse_line_ring(coords).map(|r| vec![r]).unwrap_or_default(),
        "MultiLineString" => {
            let mut rings = Vec::new();
            if let Some(lines) = coords.as_array() {
                for line in lines {
                    if let Some(ring) = parse_line_ring(line) {
                        rings.push(ring);
                    }
                }
            }
            rings
        }
        _ => Vec::new(),
    }
}

/// Parse all GIS storm data from four MapServer layer JSON responses.
/// Layers: 5 = forecast points, 6 = track, 7 = cone, 8 = watches/warnings.
pub fn parse_gis_storms(
    cone_json: Option<&Value>,
    track_json: Option<&Value>,
    points_json: Option<&Value>,
    ww_json: Option<&Value>,
) -> Vec<StormGis> {
    let mut storms: Vec<StormGis> = Vec::new();

    if let Some(cone) = cone_json
        && let Some(features) = cone.get("features").and_then(Value::as_array)
    {
        for feature in features {
            let name = prop_str(feature, "stormname");
            if name.is_empty() {
                continue;
            }
            storms.push(StormGis {
                name,
                storm_type: prop_str(feature, "stormtype"),
                basin: prop_str(feature, "basin"),
                advisory_num: prop_str(feature, "advisnum"),
                advisory_date: prop_str(feature, "advdate"),
                cone: extract_rings_for_feature(feature),
                track: Vec::new(),
                points: Vec::new(),
                watches_warnings: Vec::new(),
            });
        }
    }

    // Merge track data.
    if let Some(track) = track_json
        && let Some(features) = track.get("features").and_then(Value::as_array)
    {
        for feature in features {
            let name = prop_str(feature, "stormname");
            if let Some(storm) = storms.iter_mut().find(|s| name.contains(&s.name)) {
                storm.track = extract_rings_for_feature(feature);
            }
        }
    }

    // Merge forecast points.
    // The points layer (5) uses a full "Tropical Storm Fausto" style
    // stormname, while the cone layer (7) uses just "Fausto".  Match by
    // checking if any existing storm's name is contained in the points
    // feature's stormname.
    if let Some(points) = points_json
        && let Some(features) = points.get("features").and_then(Value::as_array)
    {
        for feature in features {
            let name = prop_str(feature, "stormname");
            if let Some(storm) = storms.iter_mut().find(|s| name.contains(&s.name))
                && let Some(geom) = feature.get("geometry")
                && let Some(coords) = geom.get("coordinates").and_then(Value::as_array)
                && let (Some(lon), Some(lat)) = (
                    coords.first().and_then(|v| v.as_f64()),
                    coords.get(1).and_then(|v| v.as_f64()),
                )
            {
                let label = prop_str(feature, "datelbl");
                storm.points.push((lat, lon, label));
            }
        }
    }

    // Merge watches/warnings.
    if let Some(ww) = ww_json
        && let Some(features) = ww.get("features").and_then(Value::as_array)
    {
        for feature in features {
            let name = prop_str(feature, "stormname");
            let ww_type = prop_str(feature, "tcww");
            if let Some(storm) = storms.iter_mut().find(|s| name.contains(&s.name)) {
                for ring in extract_rings_for_feature(feature) {
                    storm.watches_warnings.push((ring, ww_type.clone()));
                }
            }
        }
    }

    storms
}

// ── Fetch state machine ──────────────────────────────────────────────────

/// Kind of a phase-2 dependent request.
#[derive(Clone)]
enum PendingKind {
    TextProduct {
        storm_id: String,
        title: String,
        url: String,
    },
    Image {
        storm_id: String,
        index: usize,
    },
    KmzWindProb {
        threshold_kt: u32,
    },
    KmzEarliestArrival,
    KmzMostLikelyArrival,
    /// GIS MapServer layer (cone=7, track=6, points=5, watches/warnings=8).
    GisLayer {
        layer: u32,
    },
}

/// One pending phase-2 request tracked by its Ply net ID.
struct PendingRequest {
    net_id: String,
    kind: PendingKind,
}

/// Phases of the NHC fetch state machine.
enum NhcFetchPhase {
    /// Not fetching.
    Idle,
    /// Phase 1: CurrentStorms.json + 4 GIS layers in flight.
    Phase1,
    /// Phase 2: per-storm text/image/KMZ requests in flight.
    Phase2 {
        metas: Vec<StormMeta>,
        pending: Vec<PendingRequest>,
        results: NhcBundle,
        gis_cone: Option<Value>,
        gis_track: Option<Value>,
        gis_points: Option<Value>,
        gis_ww: Option<Value>,
    },
    /// Complete — bundle ready to consume.
    Complete(NhcBundle),
    /// Error occurred.
    Error(String),
}

/// Top-level NHC fetch state, polled each frame by the app.
pub struct NhcFetchState {
    phase: NhcFetchPhase,
}

impl NhcFetchState {
    pub fn new() -> Self {
        Self {
            phase: NhcFetchPhase::Idle,
        }
    }

    /// Is a fetch currently in progress?
    pub fn is_fetching(&self) -> bool {
        matches!(
            self.phase,
            NhcFetchPhase::Phase1 | NhcFetchPhase::Phase2 { .. }
        )
    }

    /// Start a new fetch cycle (phase 1).  No-op if already fetching.
    pub fn start(&mut self) {
        if self.is_fetching() {
            return;
        }
        use ply_engine::prelude::net;
        net::get(NET_ID_CURRENT_STORMS, CURRENT_STORMS_URL, |c| {
            c.header("User-Agent", "rustywx/0.3")
        });
        for layer in [5, 6, 7, 8] {
            let id = format!("{NET_ID_GIS_PREFIX}{layer}");
            let url = format!("{MAPSERVER}/{layer}/query?where=1%3D1&outFields=*&f=geojson");
            net::get(&id, &url, |c| c);
        }
        self.phase = NhcFetchPhase::Phase1;
    }

    /// Poll the fetch state machine. Returns `Some(bundle)` when the fetch
    /// is complete, or `None` if still in progress.
    pub fn poll(&mut self) -> Option<Result<NhcBundle>> {
        let phase = std::mem::replace(&mut self.phase, NhcFetchPhase::Idle);
        match phase {
            NhcFetchPhase::Idle => None,

            NhcFetchPhase::Phase1 => {
                self.poll_phase1();
                None
            }

            NhcFetchPhase::Phase2 {
                metas,
                pending,
                results,
                gis_cone,
                gis_track,
                gis_points,
                gis_ww,
            } => {
                self.poll_phase2(
                    metas, pending, results, gis_cone, gis_track, gis_points, gis_ww,
                );
                None
            }

            NhcFetchPhase::Complete(bundle) => Some(Ok(bundle)),
            NhcFetchPhase::Error(e) => Some(Err(anyhow!(e))),
        }
    }

    fn poll_phase1(&mut self) {
        use ply_engine::prelude::net;

        // Check CurrentStorms.json.
        let storms_resp = net::request(NET_ID_CURRENT_STORMS).and_then(|r| r.response());
        let gis_cone = net::request(&format!("{NET_ID_GIS_PREFIX}7")).and_then(|r| r.response());
        let gis_track = net::request(&format!("{NET_ID_GIS_PREFIX}6")).and_then(|r| r.response());
        let gis_points = net::request(&format!("{NET_ID_GIS_PREFIX}5")).and_then(|r| r.response());
        let gis_ww = net::request(&format!("{NET_ID_GIS_PREFIX}8")).and_then(|r| r.response());

        // CurrentStorms.json must arrive before we can proceed to phase 2.
        let storms_body = match storms_resp {
            None => {
                // Still pending — put the phase back and wait.
                self.phase = NhcFetchPhase::Phase1;
                return;
            }
            Some(Ok(resp)) => resp.text().to_string(),
            Some(Err(e)) => {
                self.phase = NhcFetchPhase::Error(format!("CurrentStorms.json: {e}"));
                return;
            }
        };

        let metas = match parse_current_storms(&storms_body) {
            Ok(m) => m,
            Err(e) => {
                self.phase = NhcFetchPhase::Error(format!("{e:#}"));
                return;
            }
        };

        // Parse any GIS layers that have arrived.
        let gis_cone_val = gis_cone
            .and_then(|r| r.ok())
            .and_then(|r| parse_gis_layer(r.text()).ok());
        let gis_track_val = gis_track
            .and_then(|r| r.ok())
            .and_then(|r| parse_gis_layer(r.text()).ok());
        let gis_points_val = gis_points
            .and_then(|r| r.ok())
            .and_then(|r| parse_gis_layer(r.text()).ok());
        let gis_ww_val = gis_ww
            .and_then(|r| r.ok())
            .and_then(|r| parse_gis_layer(r.text()).ok());

        // Build phase-2 pending requests.
        let mut pending: Vec<PendingRequest> = Vec::new();
        let mut results = NhcBundle::default();

        // Add GIS layers that haven't arrived yet to the pending list so
        // the completion check waits for them.
        for (layer, arrived) in [
            (7u32, gis_cone_val.is_some()),
            (6, gis_track_val.is_some()),
            (5, gis_points_val.is_some()),
            (8, gis_ww_val.is_some()),
        ] {
            if !arrived {
                let net_id = format!("{NET_ID_GIS_PREFIX}{layer}");
                pending.push(PendingRequest {
                    net_id,
                    kind: PendingKind::GisLayer { layer },
                });
            }
        }

        for meta in &metas {
            // Text products.
            for (title, url) in text_product_urls(meta) {
                let net_id = format!("nhc-text-{}-{title}", meta.id);
                net::get(&net_id, &url, |c| c.header("User-Agent", "rustywx/0.3"));
                pending.push(PendingRequest {
                    net_id,
                    kind: PendingKind::TextProduct {
                        storm_id: meta.id.clone(),
                        title: title.to_string(),
                        url: url.clone(),
                    },
                });
            }

            // Image products.
            let images = construct_image_products(meta);
            for (i, img) in images.iter().enumerate() {
                let net_id = format!("nhc-img-{}-{i}", meta.id);
                net::get(&net_id, &img.url, |c| c);
                pending.push(PendingRequest {
                    net_id,
                    kind: PendingKind::Image {
                        storm_id: meta.id.clone(),
                        index: i,
                    },
                });
            }

            // Store image product metadata (without bytes yet) in results.
            results.image_products.push((meta.id.clone(), images));

            // KMZ: wind probs (only fire once per threshold — basin-wide).
            // We'll deduplicate by checking if we already fired a request for
            // this threshold.
            // Earliest arrival (per storm).
            if !meta.earliest_arrival_kmz.is_empty() {
                let net_id = "nhc-kmz-earliest".to_string();
                net::get(&net_id, &meta.earliest_arrival_kmz, |c| c);
                pending.push(PendingRequest {
                    net_id,
                    kind: PendingKind::KmzEarliestArrival,
                });
            }
            if !meta.most_likely_arrival_kmz.is_empty() {
                let net_id = "nhc-kmz-mostlikely".to_string();
                net::get(&net_id, &meta.most_likely_arrival_kmz, |c| c);
                pending.push(PendingRequest {
                    net_id,
                    kind: PendingKind::KmzMostLikelyArrival,
                });
            }
        }

        // Fire wind-prob KMZs (once per threshold — basin-wide).
        // Use the first storm that has each URL.
        for threshold_kt in [34, 50, 64] {
            let kmz_url = metas
                .iter()
                .map(|m| match threshold_kt {
                    34 => &m.wind_probs_34kt_kmz,
                    50 => &m.wind_probs_50kt_kmz,
                    64 => &m.wind_probs_64kt_kmz,
                    _ => unreachable!(),
                })
                .find(|u| !u.is_empty());

            if let Some(url) = kmz_url {
                let net_id = format!("nhc-kmz-windprob-{threshold_kt}");
                net::get(&net_id, url, |c| c);
                pending.push(PendingRequest {
                    net_id,
                    kind: PendingKind::KmzWindProb { threshold_kt },
                });
            }
        }

        results.metas = metas;

        if pending.is_empty() {
            // No dependent requests — assemble and complete.
            results.gis_storms = parse_gis_storms(
                gis_cone_val.as_ref(),
                gis_track_val.as_ref(),
                gis_points_val.as_ref(),
                gis_ww_val.as_ref(),
            );
            // If GIS layers haven't arrived yet, we still complete with what
            // we have. GIS layers will be empty until the next cycle.
            self.phase = NhcFetchPhase::Complete(results);
        } else {
            self.phase = NhcFetchPhase::Phase2 {
                metas: results.metas.clone(),
                pending,
                results,
                gis_cone: gis_cone_val,
                gis_track: gis_track_val,
                gis_points: gis_points_val,
                gis_ww: gis_ww_val,
            };
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn poll_phase2(
        &mut self,
        _metas: Vec<StormMeta>,
        mut pending: Vec<PendingRequest>,
        mut results: NhcBundle,
        mut gis_cone: Option<Value>,
        mut gis_track: Option<Value>,
        mut gis_points: Option<Value>,
        mut gis_ww: Option<Value>,
    ) {
        use ply_engine::prelude::net;

        // Poll all pending requests.
        let mut still_pending = Vec::new();
        for req in pending.drain(..) {
            let resp = net::request(&req.net_id).and_then(|r| r.response());
            match resp {
                None => {
                    // Still pending.
                    still_pending.push(req);
                }
                Some(Ok(r)) => {
                    // Completed — process based on kind.
                    match req.kind {
                        PendingKind::TextProduct {
                            storm_id,
                            title,
                            url,
                        } => {
                            let product = parse_text_product(&title, &url, r.text());
                            if let Some(entry) = results
                                .text_products
                                .iter_mut()
                                .find(|(id, _)| *id == storm_id)
                            {
                                entry.1.push(product);
                            } else {
                                results.text_products.push((storm_id, vec![product]));
                            }
                        }
                        PendingKind::Image { storm_id, index } => {
                            let bytes = r.bytes().to_vec();
                            if !bytes.is_empty()
                                && let Some(entry) = results
                                    .image_products
                                    .iter_mut()
                                    .find(|(id, _)| *id == storm_id)
                                && index < entry.1.len()
                            {
                                entry.1[index].data = Some(bytes);
                            }
                        }
                        PendingKind::KmzWindProb { threshold_kt } => {
                            match extract_kml_from_kmz(r.bytes()) {
                                Ok(kml) => {
                                    let contours = parse_wind_prob_kml(&kml, threshold_kt);
                                    match threshold_kt {
                                        34 => results.wind_probs_34kt = contours,
                                        50 => results.wind_probs_50kt = contours,
                                        64 => results.wind_probs_64kt = contours,
                                        _ => {}
                                    }
                                }
                                Err(e) => {
                                    eprintln!("nhc: wind prob {threshold_kt}kt KMZ error: {e:#}");
                                }
                            }
                        }
                        PendingKind::KmzEarliestArrival => match extract_kml_from_kmz(r.bytes()) {
                            Ok(kml) => {
                                results.earliest_arrival = parse_arrival_kml(&kml);
                            }
                            Err(e) => {
                                eprintln!("nhc: earliest arrival KMZ error: {e:#}");
                            }
                        },
                        PendingKind::KmzMostLikelyArrival => {
                            match extract_kml_from_kmz(r.bytes()) {
                                Ok(kml) => {
                                    results.most_likely_arrival = parse_arrival_kml(&kml);
                                }
                                Err(e) => {
                                    eprintln!("nhc: most likely arrival KMZ error: {e:#}");
                                }
                            }
                        }
                        PendingKind::GisLayer { layer } => {
                            let parsed = parse_gis_layer(r.text()).ok();
                            match layer {
                                7 => gis_cone = parsed,
                                6 => gis_track = parsed,
                                5 => gis_points = parsed,
                                8 => gis_ww = parsed,
                                _ => {}
                            }
                        }
                    }
                }
                Some(Err(e)) => {
                    eprintln!("nhc: request {} failed: {e}", req.net_id);
                    // Drop the request — it failed.
                }
            }
        }

        if still_pending.is_empty() {
            // All done — assemble GIS storms and complete.
            results.gis_storms = parse_gis_storms(
                gis_cone.as_ref(),
                gis_track.as_ref(),
                gis_points.as_ref(),
                gis_ww.as_ref(),
            );
            self.phase = NhcFetchPhase::Complete(results);
        } else {
            self.phase = NhcFetchPhase::Phase2 {
                metas: _metas,
                pending: still_pending,
                results,
                gis_cone,
                gis_track,
                gis_points,
                gis_ww,
            };
        }
    }
}

impl Default for NhcFetchState {
    fn default() -> Self {
        Self::new()
    }
}

// ── Color helpers for overlays ───────────────────────────────────────────

/// Color for a wind probability contour based on its probability range.
/// Returns [r, g, b, a].
pub fn wind_prob_color(prob_high: u32) -> [u8; 4] {
    match prob_high {
        0..=5 => [0x4a, 0x90, 0xd9, 0x60],
        6..=10 => [0x4a, 0xd9, 0x90, 0x60],
        11..=20 => [0x90, 0xd9, 0x4a, 0x80],
        21..=30 => [0xd9, 0xc9, 0x4a, 0x90],
        31..=40 => [0xd9, 0x90, 0x4a, 0xa0],
        41..=50 => [0xd9, 0x6a, 0x4a, 0xb0],
        51..=60 => [0xd9, 0x4a, 0x4a, 0xc0],
        61..=70 => [0xc0, 0x3a, 0x3a, 0xd0],
        71..=80 => [0xa0, 0x2a, 0x2a, 0xe0],
        81..=90 => [0x80, 0x1a, 0x1a, 0xf0],
        _ => [0x60, 0x00, 0x00, 0xff],
    }
}

/// Color for a watch/warning type. Returns [r, g, b, a].
pub fn watch_warning_color(ww_type: &str) -> [u8; 4] {
    let t = ww_type.to_ascii_lowercase();
    if t.contains("warning") && t.contains("hurricane") {
        [0xff, 0x00, 0x00, 0xff] // red
    } else if t.contains("warning") && t.contains("tropical") {
        [0xff, 0x80, 0x00, 0xff] // orange
    } else if t.contains("watch") && t.contains("hurricane") {
        [0xff, 0x14, 0x93, 0xff] // pink
    } else if t.contains("watch") && t.contains("tropical") {
        [0xff, 0xd7, 0x00, 0xff] // gold
    } else {
        [0xff, 0xff, 0xff, 0x80] // white
    }
}

// ── Unit tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_to_ddhhmm_parses_iso8601() {
        let result = iso_to_ddhhmm("2026-07-19T23:44:32.968Z").unwrap();
        assert_eq!(result, "192344");
    }

    #[test]
    fn iso_to_ddhhmm_handles_no_fractional() {
        let result = iso_to_ddhhmm("2026-07-01T06:30:00Z").unwrap();
        assert_eq!(result, "010630");
    }

    #[test]
    fn iso_to_ddhhmm_rejects_bad_input() {
        assert!(iso_to_ddhhmm("not-a-date").is_none());
        assert!(iso_to_ddhhmm("2026-07-19").is_none()); // no T
    }

    #[test]
    fn format_bin_pads_single_digit() {
        assert_eq!(format_bin("AT2"), "AT02");
        assert_eq!(format_bin("EP5"), "EP05");
        assert_eq!(format_bin("CP1"), "CP01");
    }

    #[test]
    fn format_bin_keeps_multi_digit() {
        assert_eq!(format_bin("AT12"), "AT12");
        assert_eq!(format_bin("AT02"), "AT02");
    }

    #[test]
    fn parse_prob_range_handles_dash() {
        assert_eq!(parse_prob_range("5-10"), (5, 10));
        assert_eq!(parse_prob_range("20-30"), (20, 30));
    }

    #[test]
    fn parse_prob_range_handles_lt_gt() {
        assert_eq!(parse_prob_range("<5%"), (0, 5));
        assert_eq!(parse_prob_range(">90%"), (90, 100));
    }

    #[test]
    fn parse_prob_range_handles_xml_entities() {
        assert_eq!(parse_prob_range("&lt;5%"), (0, 5));
        assert_eq!(parse_prob_range("&gt;90%"), (90, 100));
    }

    #[test]
    fn extract_pre_content_finds_pre_block() {
        let html = "<html><pre>Hello World</pre></html>";
        assert_eq!(extract_pre_content(html).unwrap(), "Hello World");
    }

    #[test]
    fn extract_pre_content_decodes_entities() {
        let html = "<pre>&amp; &lt; &gt;</pre>";
        assert_eq!(extract_pre_content(html).unwrap(), "& < >");
    }

    #[test]
    fn extract_pre_content_returns_none_without_pre() {
        assert!(extract_pre_content("<html>no pre</html>").is_none());
    }

    #[test]
    fn strip_html_removes_tags() {
        assert_eq!(strip_html("<b>Hello</b> <i>World</i>"), "Hello World");
    }

    #[test]
    fn parse_current_storms_empty() {
        let body = r#"{"activeStorms": []}"#;
        let metas = parse_current_storms(body).unwrap();
        assert!(metas.is_empty());
    }

    #[test]
    fn parse_current_storms_one_storm() {
        let body = r#"{
            "activeStorms": [{
                "id": "al022026",
                "binNumber": "AT2",
                "name": "Beryl",
                "classification": "Hurricane",
                "intensity": "100",
                "pressure": "974",
                "latitudeNumeric": 25.3,
                "longitudeNumeric": -85.2,
                "movementDir": 315,
                "movementSpeed": 12,
                "lastUpdate": "2026-07-19T23:44:32.968Z",
                "publicAdvisory": {"advNum": "12", "url": "https://nhc.noaa.gov/text/MIATCPAT2.shtml", "fileUpdateTime": "2026-07-19T23:44:32.968Z"},
                "forecastGraphics": {"advNum": "12", "url": "https://nhc.noaa.gov/graphics", "fileUpdateTime": "2026-07-19T23:44:32.968Z"}
            }]
        }"#;
        let metas = parse_current_storms(body).unwrap();
        assert_eq!(metas.len(), 1);
        assert_eq!(metas[0].name, "Beryl");
        assert_eq!(metas[0].intensity_kt, 100);
        assert_eq!(metas[0].pressure_mb, 974);
        assert_eq!(metas[0].graphics_issuance, "192344");
    }

    #[test]
    fn construct_image_products_returns_empty_without_issuance() {
        let meta = StormMeta {
            id: "al022026".to_string(),
            name: "Test".to_string(),
            classification: "TS".to_string(),
            intensity_kt: 50,
            pressure_mb: 1000,
            lat: 25.0,
            lon: -85.0,
            movement_dir_deg: None,
            movement_speed_kt: None,
            last_update: String::new(),
            bin_number: "AT2".to_string(),
            advisory_num: "5".to_string(),
            graphics_url: String::new(),
            graphics_issuance: String::new(),
            earliest_arrival_kmz: String::new(),
            most_likely_arrival_kmz: String::new(),
            wind_probs_34kt_kmz: String::new(),
            wind_probs_50kt_kmz: String::new(),
            wind_probs_64kt_kmz: String::new(),
        };
        assert!(construct_image_products(&meta).is_empty());
    }

    #[test]
    fn construct_image_products_with_issuance() {
        let meta = StormMeta {
            id: "al022026".to_string(),
            name: "Test".to_string(),
            classification: "HU".to_string(),
            intensity_kt: 100,
            pressure_mb: 974,
            lat: 25.0,
            lon: -85.0,
            movement_dir_deg: None,
            movement_speed_kt: None,
            last_update: String::new(),
            bin_number: "AT2".to_string(),
            advisory_num: "12".to_string(),
            graphics_url: String::new(),
            graphics_issuance: "192344".to_string(),
            earliest_arrival_kmz: String::new(),
            most_likely_arrival_kmz: String::new(),
            wind_probs_34kt_kmz: String::new(),
            wind_probs_50kt_kmz: String::new(),
            wind_probs_64kt_kmz: String::new(),
        };
        let products = construct_image_products(&meta);
        assert!(!products.is_empty());
        assert!(products[0].url.contains("AT02"));
        assert!(products[0].url.contains("192344"));
    }

    #[test]
    fn parse_wind_prob_kml_extracts_contours() {
        let kml = r#"<Placemark><name>5-10%</name><Polygon><outerBoundaryIs><LinearRing><coordinates>-80.0,25.0,0 -81.0,26.0,0 -80.5,27.0,0 -80.0,25.0,0</coordinates></LinearRing></outerBoundaryIs></Polygon></Placemark>"#;
        let contours = parse_wind_prob_kml(kml, 34);
        assert_eq!(contours.len(), 1);
        assert_eq!(contours[0].prob_low, 5);
        assert_eq!(contours[0].prob_high, 10);
        assert_eq!(contours[0].threshold_kt, 34);
        assert_eq!(contours[0].rings.len(), 1);
        assert_eq!(contours[0].rings[0].len(), 4); // 4 points (closed ring)
    }

    #[test]
    fn parse_arrival_kml_extracts_contours() {
        let kml = r#"<Placemark><name>Mon 8 AM</name><Polygon><outerBoundaryIs><LinearRing><coordinates>-80.0,25.0,0 -81.0,26.0,0 -80.5,27.0,0 -80.0,25.0,0</coordinates></LinearRing></outerBoundaryIs></Polygon></Placemark>"#;
        let contours = parse_arrival_kml(kml);
        assert_eq!(contours.len(), 1);
        assert_eq!(contours[0].label, "Mon 8 AM");
    }

    #[test]
    fn extract_kml_rings_skips_short_coords() {
        let placemark =
            r#"<Placemark><coordinates>-80.0,25.0,0 -81.0,26.0,0</coordinates></Placemark>"#;
        let rings = extract_kml_rings(placemark);
        assert!(rings.is_empty()); // only 2 points, need >= 3
    }

    #[test]
    fn watch_warning_color_maps_types() {
        assert_eq!(
            watch_warning_color("HURRICANE WARNING"),
            [0xff, 0x00, 0x00, 0xff]
        );
        assert_eq!(
            watch_warning_color("TROPICAL STORM WARNING"),
            [0xff, 0x80, 0x00, 0xff]
        );
        assert_eq!(
            watch_warning_color("HURRICANE WATCH"),
            [0xff, 0x14, 0x93, 0xff]
        );
        assert_eq!(
            watch_warning_color("TROPICAL STORM WATCH"),
            [0xff, 0xd7, 0x00, 0xff]
        );
    }

    #[test]
    fn wind_prob_color_increases_with_probability() {
        let low = wind_prob_color(5);
        let high = wind_prob_color(90);
        // Higher probability should have more red/opacity
        assert!(high[0] >= low[0]); // red component
        assert!(high[3] >= low[3]); // alpha
    }

    #[test]
    fn parse_gis_storms_from_layers() {
        let cone_json: Value = serde_json::from_str(r#"{
            "features": [{
                "properties": {"stormname": "Beryl", "stormtype": "HU", "basin": "AT", "advisnum": "12", "advdate": "2026-07-19"},
                "geometry": {"type": "Polygon", "coordinates": [[[-80.0, 25.0], [-81.0, 26.0], [-80.5, 27.0], [-80.0, 25.0]]]}
            }]
        }"#).unwrap();
        let storms = parse_gis_storms(Some(&cone_json), None, None, None);
        assert_eq!(storms.len(), 1);
        assert_eq!(storms[0].name, "Beryl");
        assert_eq!(storms[0].cone.len(), 1);
    }

    #[test]
    fn text_product_urls_for_at_basin() {
        let meta = StormMeta {
            id: "al022026".to_string(),
            name: "Test".to_string(),
            classification: "TS".to_string(),
            intensity_kt: 50,
            pressure_mb: 1000,
            lat: 25.0,
            lon: -85.0,
            movement_dir_deg: None,
            movement_speed_kt: None,
            last_update: String::new(),
            bin_number: "AT2".to_string(),
            advisory_num: "5".to_string(),
            graphics_url: String::new(),
            graphics_issuance: String::new(),
            earliest_arrival_kmz: String::new(),
            most_likely_arrival_kmz: String::new(),
            wind_probs_34kt_kmz: String::new(),
            wind_probs_50kt_kmz: String::new(),
            wind_probs_64kt_kmz: String::new(),
        };
        let urls = text_product_urls(&meta);
        assert_eq!(urls.len(), 3);
        assert!(urls[0].1.contains("MIATCPAT2"));
        assert!(urls[1].1.contains("MIATCDAT2"));
        assert!(urls[2].1.contains("MIATCMAT2"));
    }

    #[test]
    fn text_product_urls_unknown_basin_returns_empty() {
        let meta = StormMeta {
            id: "xx012026".to_string(),
            name: "Test".to_string(),
            classification: "TS".to_string(),
            intensity_kt: 50,
            pressure_mb: 1000,
            lat: 25.0,
            lon: -85.0,
            movement_dir_deg: None,
            movement_speed_kt: None,
            last_update: String::new(),
            bin_number: "XX1".to_string(),
            advisory_num: "5".to_string(),
            graphics_url: String::new(),
            graphics_issuance: String::new(),
            earliest_arrival_kmz: String::new(),
            most_likely_arrival_kmz: String::new(),
            wind_probs_34kt_kmz: String::new(),
            wind_probs_50kt_kmz: String::new(),
            wind_probs_64kt_kmz: String::new(),
        };
        assert!(text_product_urls(&meta).is_empty());
    }

    #[test]
    fn parse_text_product_extracts_pre_content() {
        let body = "<html><body><pre>WINDS 100 KT\nPRESSURE 974 MB</pre></body></html>";
        let product = parse_text_product("Public Advisory", "https://example.com", body);
        assert_eq!(product.title, "Public Advisory");
        assert!(product.content.contains("WINDS 100 KT"));
    }

    #[test]
    fn nhc_fetch_state_starts_idle() {
        let state = NhcFetchState::new();
        assert!(!state.is_fetching());
    }

    #[test]
    fn parse_arrival_kml_without_name_element() {
        // NHC arrival time KML uses <description> instead of <name> for labels
        let kml = r#"<Placemark>
    <Snippet maxLines="0">empty</Snippet>
    <styleUrl>#toa_line</styleUrl>
    <description><![CDATA[<table><tr><td>Mon 2 pm</td></tr></table>]]></description>
    <LineString><coordinates>-85.4,27.0,0.0 -85.5,27.1,0.0 -85.6,27.2,0.0 -85.4,27.0,0.0</coordinates></LineString>
</Placemark>"#;
        let contours = parse_arrival_kml(kml);
        assert_eq!(contours.len(), 1);
        assert!(!contours[0].rings.is_empty());
        // Label extracted from description HTML
        assert!(contours[0].label.contains("Mon"));
    }

    #[test]
    fn parse_arrival_kml_no_name_no_description() {
        // Placemark with only coordinates — should still produce a contour
        let kml = r#"<Placemark>
    <LineString><coordinates>-85.4,27.0,0.0 -85.5,27.1,0.0 -85.6,27.2,0.0 -85.4,27.0,0.0</coordinates></LineString>
</Placemark>"#;
        let contours = parse_arrival_kml(kml);
        assert_eq!(contours.len(), 1);
        assert!(contours[0].label.is_empty());
        assert!(!contours[0].rings.is_empty());
    }

    #[test]
    fn parse_gis_storms_matches_points_by_substring() {
        // Points layer uses "Tropical Storm Fausto" but cone uses "Fausto"
        let cone_json: Value = serde_json::from_str(r#"{
            "features": [{
                "properties": {"stormname": "Fausto", "stormtype": "TS", "basin": "EP", "advisnum": "7", "advdate": "2026-07-20"},
                "geometry": {"type": "Polygon", "coordinates": [[[-80.0, 25.0], [-81.0, 26.0], [-80.5, 27.0], [-80.0, 25.0]]]}
            }]
        }"#).unwrap();
        let points_json: Value = serde_json::from_str(
            r#"{
            "features": [{
                "properties": {"stormname": "Tropical Storm Fausto", "datelbl": "8:00 AM Mon"},
                "geometry": {"type": "Point", "coordinates": [-116.4, 13.5]}
            }]
        }"#,
        )
        .unwrap();
        let storms = parse_gis_storms(Some(&cone_json), None, Some(&points_json), None);
        assert_eq!(storms.len(), 1);
        assert_eq!(storms[0].points.len(), 1);
        assert_eq!(storms[0].points[0].2, "8:00 AM Mon");
    }
}
