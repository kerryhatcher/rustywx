//! Geographic utilities — no egui dependency. Copied from rustywx.

use ply_engine::prelude::Vec2;

/// KJGX (Robins AFB, GA) antenna location — kept for backward compat.
pub const KJGX_LAT: f64 = 32.6755;
pub const KJGX_LON: f64 = -83.3511;

/// A NEXRAD radar site.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct RadarSite {
    pub id: &'static str,
    pub name: &'static str,
    pub lat: f64,
    pub lon: f64,
}

pub const RADAR_SITES: &[RadarSite] = &[
    // ... (keep existing sites) ...
    RadarSite {
        id: "KABR",
        name: "Aberdeen, SD",
        lat: 45.4558,
        lon: -98.4132,
    },
    RadarSite {
        id: "KABX",
        name: "Albuquerque, NM",
        lat: 35.1497,
        lon: -106.8240,
    },
    RadarSite {
        id: "KAKQ",
        name: "Wakefield, VA",
        lat: 36.9840,
        lon: -77.0074,
    },
    RadarSite {
        id: "KAMA",
        name: "Amarillo, TX",
        lat: 35.2334,
        lon: -101.7093,
    },
    RadarSite {
        id: "KAMX",
        name: "Miami, FL",
        lat: 25.6111,
        lon: -80.4127,
    },
    RadarSite {
        id: "KAPX",
        name: "Gaylord, MI",
        lat: 44.9071,
        lon: -84.7198,
    },
    RadarSite {
        id: "KARX",
        name: "La Crosse, WI",
        lat: 43.8227,
        lon: -91.1915,
    },
    RadarSite {
        id: "KATX",
        name: "Seattle, WA",
        lat: 48.1946,
        lon: -122.4958,
    },
    RadarSite {
        id: "KBBX",
        name: "Oroville, CA",
        lat: 39.4957,
        lon: -121.6317,
    },
    RadarSite {
        id: "KBGM",
        name: "Binghamton, NY",
        lat: 42.1997,
        lon: -75.9847,
    },
    RadarSite {
        id: "KBHX",
        name: "Eureka, CA",
        lat: 40.4987,
        lon: -124.2923,
    },
    RadarSite {
        id: "KBIS",
        name: "Bismarck, ND",
        lat: 46.7709,
        lon: -100.7605,
    },
    RadarSite {
        id: "KBLX",
        name: "Billings, MT",
        lat: 45.8538,
        lon: -108.6068,
    },
    RadarSite {
        id: "KBMX",
        name: "Birmingham, AL",
        lat: 33.1722,
        lon: -86.7698,
    },
    RadarSite {
        id: "KBOX",
        name: "Boston, MA",
        lat: 41.9559,
        lon: -71.1369,
    },
    RadarSite {
        id: "KBRO",
        name: "Brownsville, TX",
        lat: 25.9160,
        lon: -97.4190,
    },
    RadarSite {
        id: "KBUF",
        name: "Buffalo, NY",
        lat: 42.9488,
        lon: -78.7368,
    },
    RadarSite {
        id: "KCAE",
        name: "Columbia, SC",
        lat: 33.9488,
        lon: -81.1184,
    },
    RadarSite {
        id: "KCBW",
        name: "Caribou, ME",
        lat: 46.0392,
        lon: -67.8066,
    },
    RadarSite {
        id: "KCBX",
        name: "Boise, ID",
        lat: 43.4902,
        lon: -116.2360,
    },
    RadarSite {
        id: "KCCX",
        name: "State College, PA",
        lat: 40.9234,
        lon: -78.0039,
    },
    RadarSite {
        id: "KCLE",
        name: "Cleveland, OH",
        lat: 41.4132,
        lon: -81.8598,
    },
    RadarSite {
        id: "KCLX",
        name: "Charleston, SC",
        lat: 32.6555,
        lon: -81.0423,
    },
    RadarSite {
        id: "KCRP",
        name: "Corpus Christi, TX",
        lat: 27.7840,
        lon: -97.5112,
    },
    RadarSite {
        id: "KCXX",
        name: "Burlington, VT",
        lat: 44.5110,
        lon: -73.1664,
    },
    RadarSite {
        id: "KCYS",
        name: "Cheyenne, WY",
        lat: 41.1519,
        lon: -104.8060,
    },
    RadarSite {
        id: "KDAX",
        name: "Sacramento, CA",
        lat: 38.5012,
        lon: -121.6778,
    },
    RadarSite {
        id: "KDDC",
        name: "Dodge City, KS",
        lat: 37.7608,
        lon: -99.9689,
    },
    RadarSite {
        id: "KDFX",
        name: "Laughlin AFB, TX",
        lat: 29.2730,
        lon: -100.2802,
    },
    RadarSite {
        id: "KDGX",
        name: "Jackson, MS",
        lat: 32.2797,
        lon: -89.9846,
    },
    RadarSite {
        id: "KDIX",
        name: "Philadelphia, PA",
        lat: 39.9471,
        lon: -74.4109,
    },
    RadarSite {
        id: "KDLH",
        name: "Duluth, MN",
        lat: 46.8369,
        lon: -92.2097,
    },
    RadarSite {
        id: "KDMX",
        name: "Des Moines, IA",
        lat: 41.7312,
        lon: -93.7229,
    },
    RadarSite {
        id: "KDOX",
        name: "Dover AFB, DE",
        lat: 38.8258,
        lon: -75.4401,
    },
    RadarSite {
        id: "KDTX",
        name: "Detroit, MI",
        lat: 42.7000,
        lon: -83.4718,
    },
    RadarSite {
        id: "KDVN",
        name: "Davenport, IA",
        lat: 41.6116,
        lon: -90.5809,
    },
    RadarSite {
        id: "KDYX",
        name: "Abilene, TX",
        lat: 32.5386,
        lon: -99.2543,
    },
    RadarSite {
        id: "KEAX",
        name: "Kansas City, MO",
        lat: 38.8102,
        lon: -94.2644,
    },
    RadarSite {
        id: "KEMX",
        name: "Tucson, AZ",
        lat: 31.8937,
        lon: -110.6304,
    },
    RadarSite {
        id: "KENX",
        name: "Albany, NY",
        lat: 42.5866,
        lon: -74.0640,
    },
    RadarSite {
        id: "KEOX",
        name: "Fort Rucker, AL",
        lat: 31.4606,
        lon: -85.4593,
    },
    RadarSite {
        id: "KEPZ",
        name: "El Paso, TX",
        lat: 31.8731,
        lon: -106.6979,
    },
    RadarSite {
        id: "KESX",
        name: "Las Vegas, NV",
        lat: 35.7012,
        lon: -114.8918,
    },
    RadarSite {
        id: "KEVX",
        name: "Eglin AFB, FL",
        lat: 30.5650,
        lon: -85.9216,
    },
    RadarSite {
        id: "KEWX",
        name: "Austin/San Antonio, TX",
        lat: 29.7031,
        lon: -98.0285,
    },
    RadarSite {
        id: "KEYX",
        name: "Edwards AFB, CA",
        lat: 35.0979,
        lon: -117.5609,
    },
    RadarSite {
        id: "KFCX",
        name: "Blacksburg, VA",
        lat: 37.0242,
        lon: -80.2736,
    },
    RadarSite {
        id: "KFDR",
        name: "Frederick, OK",
        lat: 34.3620,
        lon: -98.9766,
    },
    RadarSite {
        id: "KFDX",
        name: "Cannon AFB, NM",
        lat: 34.6342,
        lon: -103.6186,
    },
    RadarSite {
        id: "KFFC",
        name: "Atlanta, GA",
        lat: 33.3636,
        lon: -84.5659,
    },
    RadarSite {
        id: "KFSD",
        name: "Sioux Falls, SD",
        lat: 43.5877,
        lon: -96.7294,
    },
    RadarSite {
        id: "KFSX",
        name: "Flagstaff, AZ",
        lat: 34.5744,
        lon: -111.1984,
    },
    RadarSite {
        id: "KFTG",
        name: "Denver, CO",
        lat: 39.7867,
        lon: -104.5458,
    },
    RadarSite {
        id: "KFWS",
        name: "Fort Worth, TX",
        lat: 32.5730,
        lon: -97.3032,
    },
    RadarSite {
        id: "KGGW",
        name: "Glasgow, MT",
        lat: 48.2064,
        lon: -106.6247,
    },
    RadarSite {
        id: "KGJX",
        name: "Grand Junction, CO",
        lat: 39.0620,
        lon: -108.2137,
    },
    RadarSite {
        id: "KGLD",
        name: "Goodland, KS",
        lat: 39.3668,
        lon: -101.6927,
    },
    RadarSite {
        id: "KGRB",
        name: "Green Bay, WI",
        lat: 44.4985,
        lon: -88.1111,
    },
    RadarSite {
        id: "KGRK",
        name: "Fort Hood, TX",
        lat: 30.7218,
        lon: -97.3830,
    },
    RadarSite {
        id: "KGRR",
        name: "Grand Rapids, MI",
        lat: 42.8939,
        lon: -85.5448,
    },
    RadarSite {
        id: "KGSP",
        name: "Greer, SC",
        lat: 34.8833,
        lon: -82.2199,
    },
    RadarSite {
        id: "KGWX",
        name: "Columbus AFB, MS",
        lat: 33.8968,
        lon: -88.3293,
    },
    RadarSite {
        id: "KGYX",
        name: "Portland, ME",
        lat: 43.8914,
        lon: -70.2564,
    },
    RadarSite {
        id: "KHDX",
        name: "Holloman AFB, NM",
        lat: 33.0769,
        lon: -106.1201,
    },
    RadarSite {
        id: "KHGX",
        name: "Houston/Galveston, TX",
        lat: 29.4719,
        lon: -95.0789,
    },
    RadarSite {
        id: "KHNX",
        name: "Hanford, CA",
        lat: 36.3142,
        lon: -119.6321,
    },
    RadarSite {
        id: "KHPX",
        name: "Fort Campbell, KY",
        lat: 36.7370,
        lon: -87.2854,
    },
    RadarSite {
        id: "KHTX",
        name: "Huntsville, AL",
        lat: 34.9306,
        lon: -86.0837,
    },
    RadarSite {
        id: "KHUN",
        name: "Huntsville, AL",
        lat: 34.9306,
        lon: -86.0837,
    },
    RadarSite {
        id: "KICT",
        name: "Wichita, KS",
        lat: 37.6546,
        lon: -97.4431,
    },
    RadarSite {
        id: "KICX",
        name: "Cedar City, UT",
        lat: 37.5909,
        lon: -112.8622,
    },
    RadarSite {
        id: "KILN",
        name: "Wilmington, OH",
        lat: 39.4202,
        lon: -83.8217,
    },
    RadarSite {
        id: "KILX",
        name: "Lincoln, IL",
        lat: 40.1505,
        lon: -89.3368,
    },
    RadarSite {
        id: "KIND",
        name: "Indianapolis, IN",
        lat: 39.7075,
        lon: -86.2803,
    },
    RadarSite {
        id: "KINX",
        name: "Tulsa, OK",
        lat: 36.1751,
        lon: -95.5642,
    },
    RadarSite {
        id: "KIWA",
        name: "Phoenix, AZ",
        lat: 33.2891,
        lon: -111.6700,
    },
    RadarSite {
        id: "KJAX",
        name: "Jacksonville, FL",
        lat: 30.4847,
        lon: -81.7019,
    },
    RadarSite {
        id: "KJGX",
        name: "Robins AFB, GA",
        lat: 32.6755,
        lon: -83.3509,
    },
    RadarSite {
        id: "KJKL",
        name: "Jackson, KY",
        lat: 37.5908,
        lon: -83.3130,
    },
    RadarSite {
        id: "KLBB",
        name: "Lubbock, TX",
        lat: 33.6541,
        lon: -101.8142,
    },
    RadarSite {
        id: "KLCH",
        name: "Lake Charles, LA",
        lat: 30.1254,
        lon: -93.2161,
    },
    RadarSite {
        id: "KLIX",
        name: "New Orleans, LA",
        lat: 30.3367,
        lon: -89.8256,
    },
    RadarSite {
        id: "KLNX",
        name: "North Platte, NE",
        lat: 41.9580,
        lon: -100.5759,
    },
    RadarSite {
        id: "KLOT",
        name: "Chicago, IL",
        lat: 41.6045,
        lon: -88.0844,
    },
    RadarSite {
        id: "KLRX",
        name: "Elko, NV",
        lat: 40.7397,
        lon: -116.8026,
    },
    RadarSite {
        id: "KLSX",
        name: "St. Louis, MO",
        lat: 38.6987,
        lon: -90.6828,
    },
    RadarSite {
        id: "KLTX",
        name: "Wilmington, NC",
        lat: 33.9892,
        lon: -78.4291,
    },
    RadarSite {
        id: "KLVX",
        name: "Louisville, KY",
        lat: 38.2604,
        lon: -85.7447,
    },
    RadarSite {
        id: "KLWX",
        name: "Sterling, VA",
        lat: 38.9754,
        lon: -77.4778,
    },
    RadarSite {
        id: "KLZK",
        name: "Little Rock, AR",
        lat: 34.8365,
        lon: -92.2621,
    },
    RadarSite {
        id: "KMAF",
        name: "Midland/Odessa, TX",
        lat: 31.9434,
        lon: -102.1894,
    },
    RadarSite {
        id: "KMAX",
        name: "Medford, OR",
        lat: 42.0811,
        lon: -122.7173,
    },
    RadarSite {
        id: "KMBX",
        name: "Minot AFB, ND",
        lat: 48.3930,
        lon: -100.8644,
    },
    RadarSite {
        id: "KMHX",
        name: "Morehead City, NC",
        lat: 34.7760,
        lon: -76.8762,
    },
    RadarSite {
        id: "KMKX",
        name: "Milwaukee, WI",
        lat: 42.9678,
        lon: -88.5506,
    },
    RadarSite {
        id: "KMLB",
        name: "Melbourne, FL",
        lat: 28.1132,
        lon: -80.6541,
    },
    RadarSite {
        id: "KMOB",
        name: "Mobile, AL",
        lat: 30.6795,
        lon: -88.2398,
    },
    RadarSite {
        id: "KMPX",
        name: "Minneapolis, MN",
        lat: 44.8488,
        lon: -93.5655,
    },
    RadarSite {
        id: "KMQT",
        name: "Marquette, MI",
        lat: 46.5311,
        lon: -87.5488,
    },
    RadarSite {
        id: "KMRX",
        name: "Morristown, TN",
        lat: 36.1685,
        lon: -83.4018,
    },
    RadarSite {
        id: "KMSX",
        name: "Missoula, MT",
        lat: 47.0413,
        lon: -113.9864,
    },
    RadarSite {
        id: "KMTX",
        name: "Salt Lake City, UT",
        lat: 41.2628,
        lon: -112.4480,
    },
    RadarSite {
        id: "KMUX",
        name: "San Francisco, CA",
        lat: 37.1552,
        lon: -121.8984,
    },
    RadarSite {
        id: "KMVX",
        name: "Grand Forks, ND",
        lat: 47.5279,
        lon: -97.3258,
    },
    RadarSite {
        id: "KMXX",
        name: "Maxwell AFB, AL",
        lat: 32.5367,
        lon: -85.7898,
    },
    RadarSite {
        id: "KNKX",
        name: "San Diego, CA",
        lat: 32.9190,
        lon: -117.0419,
    },
    RadarSite {
        id: "KNQA",
        name: "Memphis, TN",
        lat: 35.3448,
        lon: -89.8735,
    },
    RadarSite {
        id: "KNWX",
        name: "New Underwood, SD",
        lat: 44.0711,
        lon: -102.0900,
    },
    RadarSite {
        id: "KOAX",
        name: "Omaha, NE",
        lat: 41.3203,
        lon: -96.3668,
    },
    RadarSite {
        id: "KOHX",
        name: "Nashville, TN",
        lat: 36.2472,
        lon: -86.5625,
    },
    RadarSite {
        id: "KOKX",
        name: "New York City, NY",
        lat: 40.8655,
        lon: -72.8639,
    },
    RadarSite {
        id: "KOTX",
        name: "Spokane, WA",
        lat: 47.6804,
        lon: -117.6268,
    },
    RadarSite {
        id: "KPAH",
        name: "Paducah, KY",
        lat: 37.0684,
        lon: -88.7720,
    },
    RadarSite {
        id: "KPBZ",
        name: "Pittsburgh, PA",
        lat: 40.5317,
        lon: -80.2180,
    },
    RadarSite {
        id: "KPDT",
        name: "Pendleton, OR",
        lat: 45.6906,
        lon: -118.8530,
    },
    RadarSite {
        id: "KPOE",
        name: "Fort Polk, LA",
        lat: 31.1557,
        lon: -92.9763,
    },
    RadarSite {
        id: "KPUX",
        name: "Pueblo, CO",
        lat: 38.4596,
        lon: -104.1816,
    },
    RadarSite {
        id: "KRAX",
        name: "Raleigh/Durham, NC",
        lat: 35.6655,
        lon: -78.4898,
    },
    RadarSite {
        id: "KRGX",
        name: "Reno, NV",
        lat: 39.7542,
        lon: -119.4620,
    },
    RadarSite {
        id: "KRIW",
        name: "Riverton, WY",
        lat: 43.0661,
        lon: -108.4773,
    },
    RadarSite {
        id: "KRLX",
        name: "Charleston, WV",
        lat: 38.3111,
        lon: -81.7229,
    },
    RadarSite {
        id: "KRTX",
        name: "Portland, OR",
        lat: 45.7150,
        lon: -122.9651,
    },
    RadarSite {
        id: "KSFX",
        name: "Pocatello/Idaho Falls, ID",
        lat: 43.1056,
        lon: -112.6860,
    },
    RadarSite {
        id: "KSGF",
        name: "Springfield, MO",
        lat: 37.2352,
        lon: -93.4006,
    },
    RadarSite {
        id: "KSHV",
        name: "Shreveport, LA",
        lat: 32.4508,
        lon: -93.8412,
    },
    RadarSite {
        id: "KSJT",
        name: "San Angelo, TX",
        lat: 31.3713,
        lon: -100.4925,
    },
    RadarSite {
        id: "KSOX",
        name: "Santa Ana, CA",
        lat: 33.8177,
        lon: -117.6360,
    },
    RadarSite {
        id: "KSRX",
        name: "Fort Smith, AR",
        lat: 35.2905,
        lon: -94.3619,
    },
    RadarSite {
        id: "KTBW",
        name: "Tampa Bay, FL",
        lat: 27.7055,
        lon: -82.4018,
    },
    RadarSite {
        id: "KTFX",
        name: "Great Falls, MT",
        lat: 47.4596,
        lon: -111.3855,
    },
    RadarSite {
        id: "KTLH",
        name: "Tallahassee, FL",
        lat: 30.3976,
        lon: -84.3289,
    },
    RadarSite {
        id: "KTLX",
        name: "Oklahoma City, OK",
        lat: 35.3334,
        lon: -97.2778,
    },
    RadarSite {
        id: "KTWX",
        name: "Topeka, KS",
        lat: 38.9969,
        lon: -96.2326,
    },
    RadarSite {
        id: "KTYX",
        name: "Fort Drum, NY",
        lat: 43.7557,
        lon: -75.6799,
    },
    RadarSite {
        id: "KUDX",
        name: "Rapid City, SD",
        lat: 44.1248,
        lon: -102.8298,
    },
    RadarSite {
        id: "KUEX",
        name: "Hastings, NE",
        lat: 40.3210,
        lon: -98.4418,
    },
    RadarSite {
        id: "KVAX",
        name: "Moody AFB, GA",
        lat: 30.8904,
        lon: -83.0019,
    },
    RadarSite {
        id: "KVBX",
        name: "Vandenberg AFB, CA",
        lat: 34.8384,
        lon: -120.3981,
    },
    RadarSite {
        id: "KVNX",
        name: "Vance AFB, OK",
        lat: 36.7407,
        lon: -98.1279,
    },
    RadarSite {
        id: "KVTX",
        name: "Los Angeles, CA",
        lat: 34.4116,
        lon: -119.1790,
    },
    RadarSite {
        id: "KVWX",
        name: "Evansville, IN",
        lat: 38.2604,
        lon: -87.7246,
    },
    RadarSite {
        id: "KYUX",
        name: "Yuma, AZ",
        lat: 32.4953,
        lon: -114.6567,
    },
];

/// Compute range (km) and bearing (degrees from north) from a radar site
/// to a target lat/lon. Uses the haversine formula.
pub fn range_bearing(site_lat: f64, site_lon: f64, target_lat: f64, target_lon: f64) -> (f64, f64) {
    let earth_radius_km = 6371.0;
    let dlat = (target_lat - site_lat).to_radians();
    let dlon = (target_lon - site_lon).to_radians();
    let lat1 = site_lat.to_radians();
    let lat2 = target_lat.to_radians();

    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    let range_km = earth_radius_km * c;

    let y = dlon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();
    let bearing_deg = y.atan2(x).to_degrees().rem_euclid(360.0);

    (range_km, bearing_deg)
}

/// Convert polar (bearing, range) to screen-space offset (x right, y up).
pub fn polar_to_offset(bearing_deg: f32, range_km: f32, px_per_km: f32) -> (f32, f32) {
    let rad = bearing_deg.to_radians();
    let dx = rad.sin() * range_km * px_per_km;
    let dy = -rad.cos() * range_km * px_per_km;
    (dx, dy)
}

/// Convert a lat/lon point to a km-scale planar offset from an origin
/// (typically a radar site) using the same azimuthal-equidistant
/// convention as `polar_to_offset` with `px_per_km = 1.0`.
pub fn point_to_km_offset(origin_lat: f64, origin_lon: f64, (lat, lon): (f64, f64)) -> Vec2 {
    let (range_km, bearing_deg) = range_bearing(origin_lat, origin_lon, lat, lon);
    let theta = bearing_deg.to_radians();
    Vec2::new(
        (range_km * theta.sin()) as f32,
        (-range_km * theta.cos()) as f32,
    )
}

/// Return the portions of the segment `a` -> `b` that lie inside or on a
/// circle centered at the origin with radius `r`. This lets large alert or
/// border polygons that only clip through the scope still draw their visible
/// chord.
pub fn circle_subsegments(a: Vec2, b: Vec2, r: f32) -> Vec<(Vec2, Vec2)> {
    let d = b - a;
    let aa = d.length_squared();
    const EPS: f32 = 1e-5;

    // Degenerate segment; treat as inside if the point is within the circle.
    if aa < EPS {
        if a.length_squared() <= r * r + EPS {
            return vec![(a, b)];
        }
        return Vec::new();
    }

    let mut ts = Vec::new();
    if a.length_squared() <= r * r + EPS {
        ts.push(0.0);
    }
    if b.length_squared() <= r * r + EPS {
        ts.push(1.0);
    }

    // Solve |a + t*d|^2 = r^2 for t.
    let ad = a.dot(d);
    let c = a.length_squared() - r * r;
    let disc_sq = ad * ad - aa * c;
    if disc_sq >= 0.0 {
        let disc = disc_sq.sqrt();
        let t1 = (-ad - disc) / aa;
        let t2 = (-ad + disc) / aa;
        if (0.0..=1.0).contains(&t1) {
            ts.push(t1);
        }
        if (0.0..=1.0).contains(&t2) {
            ts.push(t2);
        }
    }

    ts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut out = Vec::new();
    for window in ts.windows(2) {
        let t_mid = (window[0] + window[1]) * 0.5;
        let mid = a + d * t_mid;
        if mid.length_squared() <= r * r + EPS {
            out.push((a + d * window[0], a + d * window[1]));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // KJGX (East Central Georgia) coordinates — used in range/bearing test.
    const KJGX_LAT: f64 = 32.6755;
    const KJGX_LON: f64 = -83.3511;

    #[test]
    fn macon_range_bearing_from_kjgx() {
        let (range_km, bearing_deg) = range_bearing(KJGX_LAT, KJGX_LON, 32.8407, -83.6324);
        assert!((31.0..34.0).contains(&range_km), "range {range_km}");
        assert!(
            (302.0..308.0).contains(&bearing_deg),
            "bearing {bearing_deg}"
        );
    }

    #[test]
    fn range_bearing_due_north() {
        let (range_km, bearing_deg) = range_bearing(32.0, -83.0, 33.0, -83.0);
        assert!((range_km - 111.2).abs() < 1.0, "range {range_km}");
        assert!(bearing_deg.abs() < 0.01 || (bearing_deg - 360.0).abs() < 0.01);
    }

    #[test]
    fn polar_offsets_cardinal_directions() {
        // North: straight up the screen (negative y).
        let (x, y) = polar_to_offset(0.0, 10.0, 2.0);
        assert!(x.abs() < 1e-4 && (y + 20.0).abs() < 1e-4, "north ({x},{y})");
        // East: +x.
        let (x, y) = polar_to_offset(90.0, 10.0, 2.0);
        assert!((x - 20.0).abs() < 1e-3 && y.abs() < 1e-3, "east ({x},{y})");
        // South: +y (screen y grows downward).
        let (x, y) = polar_to_offset(180.0, 10.0, 2.0);
        assert!(x.abs() < 1e-3 && (y - 20.0).abs() < 1e-3, "south ({x},{y})");
    }

    #[test]
    fn keeps_both_endpoints_inside_circle() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 0.0);
        let segs = circle_subsegments(a, b, 20.0);
        assert_eq!(segs.len(), 1);
        assert!((segs[0].0 - a).length() < 1e-4);
        assert!((segs[0].1 - b).length() < 1e-4);
    }

    #[test]
    fn clips_outside_endpoint_to_circle() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(30.0, 0.0);
        let segs = circle_subsegments(a, b, 20.0);
        assert_eq!(segs.len(), 1);
        assert!((segs[0].0 - a).length() < 1e-4);
        assert!((segs[0].1 - Vec2::new(20.0, 0.0)).length() < 1e-4);
    }

    #[test]
    fn both_outside_but_crossing_produces_chord() {
        let a = Vec2::new(-30.0, 0.0);
        let b = Vec2::new(30.0, 0.0);
        let segs = circle_subsegments(a, b, 20.0);
        assert_eq!(segs.len(), 1);
        assert!((segs[0].0 - Vec2::new(-20.0, 0.0)).length() < 1e-4);
        assert!((segs[0].1 - Vec2::new(20.0, 0.0)).length() < 1e-4);
    }

    #[test]
    fn fully_outside_segment_yields_empty() {
        let a = Vec2::new(30.0, 0.0);
        let b = Vec2::new(40.0, 0.0);
        assert!(circle_subsegments(a, b, 20.0).is_empty());
    }
}
