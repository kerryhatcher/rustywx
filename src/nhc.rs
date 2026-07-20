//! NHC tropical cyclone data: fetches storm metadata, text products, and
//! image products from the NHC CurrentStorms.json API, plus GIS overlays
//! (forecast cones, tracks, watches/warnings) from NOAA's ArcGIS MapServer.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::mpsc::Sender;
use std::time::Duration;

/// How often to refresh NHC data (every 5 minutes).
const REFRESH_INTERVAL: Duration = Duration::from_secs(300);

/// Base URL for the NHC tropical weather summary MapServer.
const MAPSERVER: &str =
    "https://mapservices.weather.noaa.gov/tropical/rest/services/tropical/NHC_tropical_weather_summary/MapServer";

/// NHC active storms JSON API.
const CURRENT_STORMS_URL: &str = "https://www.nhc.noaa.gov/CurrentStorms.json";

// ── Data types ──────────────────────────────────────────────────────────

/// A single ring (closed polygon or open line) in lat/lon degrees.
pub type Ring = Vec<(f64, f64)>;

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

/// Complete NHC data bundle sent to the UI.
#[derive(Clone, Serialize, Deserialize)]
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

/// Messages sent from the NHC worker to the UI thread.
pub enum Nhcmessage {
    Loaded(NhcBundle),
    Error(String),
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

/// Convert an ISO 8601 timestamp like "2026-07-19T23:44:32.968Z"
/// to the DDHHMM format used in NHC image URLs (e.g. "192344").
fn iso_to_ddhhmm(iso: &str) -> Option<String> {
    // Find the T separator.
    let t_pos = iso.find('T')?;
    let date_part = &iso[..t_pos]; // "2026-07-19"
    let time_part = &iso[t_pos + 1..]; // "23:44:32.968Z"

    // Extract day from date (last 2 digits before T).
    let day: &str = date_part.rsplit('-').next()?;

    // Extract hour and minute from time.
    let hour = &time_part[..2];
    let minute = &time_part[3..5];

    Some(format!("{day}{hour}{minute}"))
}

/// Fetch and parse the CurrentStorms.json API.
fn fetch_current_storms() -> Result<Vec<StormMeta>> {
    crate::logger::log("nhc: fetching CurrentStorms.json…");
    let body = ureq::get(CURRENT_STORMS_URL)
        .header("User-Agent", "rustywx/0.1")
        .call()
        .map_err(|e| anyhow!("fetching CurrentStorms.json: {e}"))?
        .into_body()
        .read_to_string()
        .map_err(|e| anyhow!("reading CurrentStorms.json: {e}"))?;

    let response: CurrentStormsResponse =
        serde_json::from_str(&body).map_err(|e| anyhow!("parsing CurrentStorms.json: {e}"))?;

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
            // Extract DDHHMM from the fileUpdateTime ISO 8601 timestamp.
            // e.g. "2026-07-19T23:44:32.968Z" → "192344"
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

    crate::logger::log(&format!(
        "nhc: parsed {} storm(s) from CurrentStorms.json",
        metas.len()
    ));
    Ok(metas)
}

// ── Text product fetching ────────────────────────────────────────────────

/// Fetch a text product from its .shtml URL and extract the plain text.
fn fetch_text_product(title: &str, url: &str) -> Result<TextProduct> {
    crate::logger::log(&format!("nhc: fetching text product: {title}"));
    let body = ureq::get(url)
        .header("User-Agent", "rustywx/0.1")
        .call()
        .map_err(|e| anyhow!("fetching {title}: {e}"))?
        .into_body()
        .read_to_string()
        .map_err(|e| anyhow!("reading {title}: {e}"))?;

    // NHC text products are HTML with the content in <pre> tags.
    let content = extract_pre_content(&body).unwrap_or_else(|| {
        // Fallback: strip HTML tags
        strip_html(&body)
    });

    Ok(TextProduct {
        title: title.to_string(),
        content,
        url: url.to_string(),
    })
}

/// Extract text from the first <pre>…</pre> block in HTML.
fn extract_pre_content(html: &str) -> Option<String> {
    let start = html.find("<pre>")? + 5;
    let end = html[start..].find("</pre>")?;
    let raw = &html[start..start + end];
    // Decode common HTML entities.
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

/// Fetch all text products for a storm.
fn fetch_text_products(meta: &StormMeta) -> Vec<TextProduct> {
    let mut products = Vec::new();

    // We need to re-fetch the CurrentStorms.json to get the URLs, or
    // we can construct them from the storm ID. Let's use the known URL
    // patterns based on bin number.
    let basin_prefix = if meta.bin_number.starts_with("AT") {
        "AT"
    } else if meta.bin_number.starts_with("EP") {
        "EP"
    } else if meta.bin_number.starts_with("CP") {
        "CP"
    } else {
        return products; // Unknown basin
    };

    let bin_num = &meta.bin_number[2..]; // e.g. "2" from "AT2"

    // Construct text product URLs.
    let advisory_url = format!(
        "https://www.nhc.noaa.gov/text/MIATCP{basin_prefix}{bin_num}.shtml"
    );
    let discussion_url = format!(
        "https://www.nhc.noaa.gov/text/MIATCD{basin_prefix}{bin_num}.shtml"
    );
    let forecast_url = format!(
        "https://www.nhc.noaa.gov/text/MIATCM{basin_prefix}{bin_num}.shtml"
    );

    for (title, url) in [
        ("Public Advisory", &advisory_url),
        ("Forecast Discussion", &discussion_url),
        ("Forecast Advisory", &forecast_url),
    ] {
        match fetch_text_product(title, url) {
            Ok(product) => products.push(product),
            Err(e) => {
                crate::logger::log(&format!(
                    "nhc: failed to fetch {title} for {}: {e:#}",
                    meta.name
                ));
            }
        }
    }

    products
}

// ── Image product URL construction ───────────────────────────────────────

/// Construct image product URLs for a storm based on known NHC patterns.
fn construct_image_products(meta: &StormMeta) -> Vec<ImageProduct> {
    let mut products = Vec::new();

    let storm_id = meta.id.to_uppercase(); // e.g. "AL022026"
    // Bin number needs leading zero: "AT2" → "AT02"
    let bin = format_bin(&meta.bin_number); // e.g. "AT02"
    let ts = &meta.graphics_issuance; // e.g. "192344"

    // If we don't have a valid issuance timestamp, skip image products.
    if ts.is_empty() {
        return products;
    }

    // Image URLs follow:
    // Full-size:  storm_graphics/{bin}/refresh/{stormId}_{product}+png/{ts}_{product}.png
    // Small:      storm_graphics/{bin}/refresh/{stormId}_{product}_sm+png/{ts}_{product}_sm.png

    // Cone graphic (5-day cone with warnings)
    products.push(ImageProduct {
        title: "5-Day Cone".to_string(),
        url: format!(
            "https://www.nhc.noaa.gov/storm_graphics/{bin}/refresh/{storm_id}_5day_cone+png/{ts}_5day_cone.png"
        ),
        data: None,
    });

    // Key Messages
    products.push(ImageProduct {
        title: "Key Messages".to_string(),
        url: format!(
            "https://www.nhc.noaa.gov/storm_graphics/{bin}/refresh/{storm_id}_key_messages+png/{ts}_key_messages.png"
        ),
        data: None,
    });

    // Current wind field
    products.push(ImageProduct {
        title: "Wind Field".to_string(),
        url: format!(
            "https://www.nhc.noaa.gov/storm_graphics/{bin}/refresh/{storm_id}_current_wind+png/{ts}_current_wind.png"
        ),
        data: None,
    });

    // Wind speed probabilities (34kt, 120hr)
    products.push(ImageProduct {
        title: "Wind Probs 34kt".to_string(),
        url: format!(
            "https://www.nhc.noaa.gov/storm_graphics/{bin}/refresh/{storm_id}_wind_probs_34_F120+png/{ts}_wind_probs_34_F120.png"
        ),
        data: None,
    });

    // Peak surge (if applicable)
    products.push(ImageProduct {
        title: "Peak Surge".to_string(),
        url: format!(
            "https://www.nhc.noaa.gov/storm_graphics/{bin}/refresh/{storm_id}_peak_surge+png/{ts}_peak_surge.png"
        ),
        data: None,
    });

    // Rainfall potential (WPC QPF) — note: WPC uses shortened storm ID
    let short_id = &storm_id[..storm_id.len().min(6)]; // "AL022026" → "AL0226"
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
    // Split letters from digits: "AT2" → ("AT", "2")
    let split = bin.find(|c: char| c.is_ascii_digit()).unwrap_or(bin.len());
    let letters = &bin[..split];
    let digits = &bin[split..];
    if let Ok(num) = digits.parse::<u32>() {
        format!("{letters}{num:02}")
    } else {
        bin.to_string()
    }
}

/// Download image bytes for an image product.
fn download_image(product: &ImageProduct) -> Option<Vec<u8>> {
    crate::logger::log(&format!("nhc: downloading image: {}", product.title));
    match ureq::get(&product.url)
        .header("User-Agent", "rustywx/0.1")
        .call()
    {
        Ok(response) => {
            let bytes = response.into_body().read_to_vec().unwrap_or_default();
            if !bytes.is_empty() {
                crate::logger::log(&format!(
                    "nhc: downloaded {} ({} bytes)",
                    product.title,
                    bytes.len()
                ));
                Some(bytes)
            } else {
                crate::logger::log(&format!("nhc: empty image: {}", product.title));
                None
            }
        }
        Err(e) => {
            crate::logger::log(&format!(
                "nhc: failed to download {}: {e}",
                product.title
            ));
            None
        }
    }
}

// ── KMZ / KML parsing ──────────────────────────────────────────────────

/// Download a KMZ file, unzip it, and return the KML content.
fn download_kmz(url: &str) -> Result<String> {
    crate::logger::log(&format!("nhc: downloading KMZ: {url}"));
    let response = ureq::get(url)
        .header("User-Agent", "rustywx/0.1")
        .call()
        .map_err(|e| anyhow!("downloading KMZ: {e}"))?;
    let bytes = response.into_body().read_to_vec()?;

    let cursor = std::io::Cursor::new(bytes);
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| anyhow!("opening KMZ zip: {e}"))?;

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
fn parse_wind_prob_kml(kml: &str, threshold_kt: u32) -> Vec<WindProbContour> {
    let mut contours = Vec::new();

    // Find all Placemark blocks.
    let mut search_start = 0;
    while let Some(pm_start) = kml[search_start..].find("<Placemark>") {
        let abs_start = search_start + pm_start;
        let Some(pm_end) = kml[abs_start..].find("</Placemark>") else {
            break;
        };
        let abs_end = abs_start + pm_end + "</Placemark>".len();
        let placemark = &kml[abs_start..abs_end];
        search_start = abs_end;

        // Extract name.
        let Some(name) = extract_xml_text(placemark, "name") else {
            continue;
        };
        let name = name.trim().to_string();
        if name.is_empty() {
            continue;
        }

        // Parse probability range from name.
        let (prob_low, prob_high) = parse_prob_range(&name);

        // Extract all coordinate rings.
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
fn parse_arrival_kml(kml: &str) -> Vec<ArrivalTimeContour> {
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

        // Extract name.
        let Some(name) = extract_xml_text(placemark, "name") else {
            continue;
        };
        let name = name.trim().to_string();
        if name.is_empty() {
            continue;
        }

        // Skip non-contour placemarks (like point markers).
        let rings = extract_kml_rings(placemark);
        if rings.is_empty() {
            continue;
        }

        contours.push(ArrivalTimeContour {
            label: name,
            rings,
        });
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

    // Find all <coordinates> blocks.
    let mut search = 0;
    while let Some(coord_start) = placemark[search..].find("<coordinates>") {
        let abs_start = search + coord_start + "<coordinates>".len();
        let Some(coord_end) = placemark[abs_start..].find("</coordinates>") else {
            break;
        };
        let abs_end = abs_start + coord_end;
        let coords_str = &placemark[abs_start..abs_end];
        search = abs_end + "</coordinates>".len();

        // Parse coordinate tuples: "lon,lat,alt lon,lat,alt ..."
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

/// Fetch a GeoJSON feature collection from an ArcGIS MapServer layer.
fn fetch_layer(layer_id: u32) -> Result<Value> {
    let url = format!(
        "{MAPSERVER}/{layer_id}/query?where=1%3D1&outFields=*&f=geojson"
    );
    crate::logger::log(&format!("nhc: fetching layer {layer_id}…"));
    let body = ureq::get(&url)
        .header("User-Agent", "rustywx/0.1")
        .call()
        .map_err(|e| anyhow!("fetching layer {layer_id}: {e}"))?
        .into_body()
        .read_to_string()
        .map_err(|e| anyhow!("reading layer {layer_id}: {e}"))?;
    serde_json::from_str(&body).map_err(|e| anyhow!("parsing layer {layer_id}: {e}"))
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
    if rings.is_empty() {
        None
    } else {
        Some(rings)
    }
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
        "LineString" => {
            if let Some(ring) = parse_line_ring(coords) {
                vec![ring]
            } else {
                Vec::new()
            }
        }
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

/// Fetch all GIS storm data from the MapServer.
fn fetch_gis_storms() -> Result<Vec<StormGis>> {
    crate::logger::log("nhc: fetch_gis_storms starting…");

    let cone_json = fetch_layer(7)
        .inspect_err(|e| crate::logger::log(&format!("nhc: cone layer failed: {e:#}")))
        .ok();
    let track_json = fetch_layer(6)
        .inspect_err(|e| crate::logger::log(&format!("nhc: track layer failed: {e:#}")))
        .ok();
    let points_json = fetch_layer(5)
        .inspect_err(|e| crate::logger::log(&format!("nhc: points layer failed: {e:#}")))
        .ok();
    let ww_json = fetch_layer(8)
        .inspect_err(|e| crate::logger::log(&format!("nhc: ww layer failed: {e:#}")))
        .ok();

    let mut storms: Vec<StormGis> = Vec::new();

    if let Some(ref cone) = cone_json {
        let Some(features) = cone.get("features").and_then(Value::as_array) else {
            return Ok(Vec::new());
        };
        for feature in features {
            let name = prop_str(feature, "stormname");
            if name.is_empty() {
                continue;
            }
            let storm_type = prop_str(feature, "stormtype");
            let basin = prop_str(feature, "basin");
            let adv_num = prop_str(feature, "advisnum");
            let adv_date = prop_str(feature, "advdate");
            let cone_rings = extract_rings_for_feature(feature);

            storms.push(StormGis {
                name,
                storm_type,
                basin,
                advisory_num: adv_num,
                advisory_date: adv_date,
                cone: cone_rings,
                track: Vec::new(),
                points: Vec::new(),
                watches_warnings: Vec::new(),
            });
        }
    }

    // Merge track data.
    if let Some(ref track) = track_json
        && let Some(features) = track.get("features").and_then(Value::as_array)
    {
        for feature in features {
            let name = prop_str(feature, "stormname");
            if let Some(storm) = storms.iter_mut().find(|s| s.name == name) {
                storm.track = extract_rings_for_feature(feature);
            }
        }
    }

    // Merge forecast points.
    if let Some(ref points) = points_json
        && let Some(features) = points.get("features").and_then(Value::as_array)
    {
        for feature in features {
            let name = prop_str(feature, "stormname");
            if let Some(storm) = storms.iter_mut().find(|s| s.name == name)
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
    if let Some(ref ww) = ww_json
        && let Some(features) = ww.get("features").and_then(Value::as_array)
    {
        for feature in features {
            let name = prop_str(feature, "stormname");
            let ww_type = prop_str(feature, "tcww");
            if let Some(storm) = storms.iter_mut().find(|s| s.name == name) {
                for ring in extract_rings_for_feature(feature) {
                    storm.watches_warnings.push((ring, ww_type.clone()));
                }
            }
        }
    }

    crate::logger::log(&format!(
        "nhc: parsed {} GIS storm(s)",
        storms.len()
    ));
    Ok(storms)
}

// ── Background worker ────────────────────────────────────────────────────

/// Spawn a background thread that fetches NHC data every 5 minutes and
/// sends it to the UI via `tx`.
pub fn spawn_nhc_worker(
    tx: Sender<Nhcmessage>,
    egui_ctx: egui::Context,
    refresh_rx: std::sync::mpsc::Receiver<()>,
) {
    std::thread::spawn(move || loop {
        crate::logger::log("nhc: worker cycle starting…");

        let mut bundle = NhcBundle {
            metas: Vec::new(),
            gis_storms: Vec::new(),
            text_products: Vec::new(),
            image_products: Vec::new(),
            wind_probs_34kt: Vec::new(),
            wind_probs_50kt: Vec::new(),
            wind_probs_64kt: Vec::new(),
            earliest_arrival: Vec::new(),
            most_likely_arrival: Vec::new(),
        };

        // 1. Fetch storm metadata from CurrentStorms.json.
        match fetch_current_storms() {
            Ok(metas) => {
                // 2. For each storm, fetch text products.
                for meta in &metas {
                    let texts = fetch_text_products(meta);
                    if !texts.is_empty() {
                        bundle
                            .text_products
                            .push((meta.id.clone(), texts));
                    }

                    // 3. Construct and download image products.
                    let mut images = construct_image_products(meta);
                    for img in &mut images {
                        img.data = download_image(img);
                    }
                    bundle
                        .image_products
                        .push((meta.id.clone(), images));
                }

                // 4. Fetch KMZ overlays (wind probs + arrival times).
                // Use the first storm that has KMZ URLs (wind probs are
                // basin-wide, so they're the same for all storms).
                let mut fetched_34kt = false;
                let mut fetched_50kt = false;
                let mut fetched_64kt = false;
                for meta in &metas {
                    if !fetched_34kt && !meta.wind_probs_34kt_kmz.is_empty() {
                        match download_kmz(&meta.wind_probs_34kt_kmz) {
                            Ok(kml) => {
                                bundle.wind_probs_34kt =
                                    parse_wind_prob_kml(&kml, 34);
                                fetched_34kt = true;
                            }
                            Err(e) => crate::logger::log(&format!(
                                "nhc: wind prob 34kt KMZ error: {e:#}"
                            )),
                        }
                    }
                    if !fetched_50kt && !meta.wind_probs_50kt_kmz.is_empty() {
                        match download_kmz(&meta.wind_probs_50kt_kmz) {
                            Ok(kml) => {
                                bundle.wind_probs_50kt =
                                    parse_wind_prob_kml(&kml, 50);
                                fetched_50kt = true;
                            }
                            Err(e) => crate::logger::log(&format!(
                                "nhc: wind prob 50kt KMZ error: {e:#}"
                            )),
                        }
                    }
                    if !fetched_64kt && !meta.wind_probs_64kt_kmz.is_empty() {
                        match download_kmz(&meta.wind_probs_64kt_kmz) {
                            Ok(kml) => {
                                bundle.wind_probs_64kt =
                                    parse_wind_prob_kml(&kml, 64);
                                fetched_64kt = true;
                            }
                            Err(e) => crate::logger::log(&format!(
                                "nhc: wind prob 64kt KMZ error: {e:#}"
                            )),
                        }
                    }
                    // Arrival time KMZs are per-storm.
                    if !meta.earliest_arrival_kmz.is_empty() {
                        match download_kmz(&meta.earliest_arrival_kmz) {
                            Ok(kml) => {
                                let contours = parse_arrival_kml(&kml);
                                bundle.earliest_arrival.extend(contours);
                            }
                            Err(e) => crate::logger::log(&format!(
                                "nhc: earliest arrival KMZ error: {e:#}"
                            )),
                        }
                    }
                    if !meta.most_likely_arrival_kmz.is_empty() {
                        match download_kmz(&meta.most_likely_arrival_kmz) {
                            Ok(kml) => {
                                let contours = parse_arrival_kml(&kml);
                                bundle.most_likely_arrival.extend(contours);
                            }
                            Err(e) => crate::logger::log(&format!(
                                "nhc: most likely arrival KMZ error: {e:#}"
                            )),
                        }
                    }
                }

                bundle.metas = metas;
            }
            Err(e) => {
                crate::logger::log(&format!("nhc: CurrentStorms.json error: {e:#}"));
            }
        }

        // 5. Fetch GIS overlays from MapServer.
        match fetch_gis_storms() {
            Ok(gis) => {
                bundle.gis_storms = gis;
            }
            Err(e) => {
                crate::logger::log(&format!("nhc: GIS fetch error: {e:#}"));
            }
        }

        // Log summary.
        crate::logger::log(&format!(
            "nhc: cycle complete — {} metas, {} gis, {} text sets, {} image sets, {} wind prob contours, {} arrival contours",
            bundle.metas.len(),
            bundle.gis_storms.len(),
            bundle.text_products.len(),
            bundle.image_products.len(),
            bundle.wind_probs_34kt.len() + bundle.wind_probs_50kt.len() + bundle.wind_probs_64kt.len(),
            bundle.earliest_arrival.len() + bundle.most_likely_arrival.len(),
        ));

        // Cache and send.
        crate::cache::save_nhc(&bundle);
        let _ = tx.send(Nhcmessage::Loaded(bundle));
        egui_ctx.request_repaint();

        // Wait for either a refresh trigger or the poll interval.
        let _ = refresh_rx.recv_timeout(REFRESH_INTERVAL);
    });
}
