//! Presentation rules for the spaghetti-models overlay: wind-category colours,
//! per-segment colour, line styles, the km-projection cache, cycle/age text,
//! the disclaimer, and intensity-chart scaling. Pure: no drawing calls.

use crate::adeck::{GuidanceStore, ModelGroup};
use crate::geo;
use ply_engine::prelude::Vec2;

/// Saffir-Simpson wind category of a forecast point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindCategory {
    /// VMAX column was 0 (no value), e.g. TVCN.
    Unknown,
    Depression,
    Storm,
    Cat1,
    Cat2,
    Cat3,
    Cat4,
    Cat5,
}

impl WindCategory {
    /// Legend order: weakest to strongest, then "no intensity".
    pub const LEGEND: [WindCategory; 8] = [
        WindCategory::Depression,
        WindCategory::Storm,
        WindCategory::Cat1,
        WindCategory::Cat2,
        WindCategory::Cat3,
        WindCategory::Cat4,
        WindCategory::Cat5,
        WindCategory::Unknown,
    ];

    pub fn from_kt(kt: Option<u32>) -> Self {
        match kt {
            None => WindCategory::Unknown,
            Some(k) if k < 34 => WindCategory::Depression,
            Some(k) if k < 64 => WindCategory::Storm,
            Some(k) if k < 83 => WindCategory::Cat1,
            Some(k) if k < 96 => WindCategory::Cat2,
            Some(k) if k < 113 => WindCategory::Cat3,
            Some(k) if k < 137 => WindCategory::Cat4,
            Some(_) => WindCategory::Cat5,
        }
    }

    /// Standard Saffir-Simpson scale colours; grey for unknown.
    pub fn rgb(self) -> [u8; 3] {
        match self {
            WindCategory::Unknown => [0x9E, 0x95, 0x90],
            WindCategory::Depression => [0x5E, 0xBA, 0xFF],
            WindCategory::Storm => [0x00, 0xFA, 0xF4],
            WindCategory::Cat1 => [0xFF, 0xFF, 0xCC],
            WindCategory::Cat2 => [0xFF, 0xE7, 0x75],
            WindCategory::Cat3 => [0xFF, 0xC1, 0x40],
            WindCategory::Cat4 => [0xFF, 0x8F, 0x20],
            WindCategory::Cat5 => [0xFF, 0x60, 0x60],
        }
    }

    /// `0xRRGGBB` for Ply `background_color`.
    pub fn hex(self) -> u32 {
        let [r, g, b] = self.rgb();
        (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
    }

    pub fn label(self) -> &'static str {
        match self {
            WindCategory::Unknown => "No intensity",
            WindCategory::Depression => "TD  <34 kt",
            WindCategory::Storm => "TS  34-63 kt",
            WindCategory::Cat1 => "Cat 1  64-82 kt",
            WindCategory::Cat2 => "Cat 2  83-95 kt",
            WindCategory::Cat3 => "Cat 3  96-112 kt",
            WindCategory::Cat4 => "Cat 4  113-136 kt",
            WindCategory::Cat5 => "Cat 5  137+ kt",
        }
    }

    /// Lower bound in kt (intensity-chart bands); `None` for Unknown.
    pub fn min_kt(self) -> Option<u32> {
        match self {
            WindCategory::Unknown => None,
            WindCategory::Depression => Some(0),
            WindCategory::Storm => Some(34),
            WindCategory::Cat1 => Some(64),
            WindCategory::Cat2 => Some(83),
            WindCategory::Cat3 => Some(96),
            WindCategory::Cat4 => Some(113),
            WindCategory::Cat5 => Some(137),
        }
    }
}

/// RGBA for the segment between two consecutive points: coloured by the
/// wind at its start (falling back to its end). CARQ (observed) is plain
/// grey; OFCL is opaque, models slightly translucent.
pub fn segment_rgba(group: ModelGroup, start_kt: Option<u32>, end_kt: Option<u32>) -> [u8; 4] {
    if group == ModelGroup::Observed {
        return [0xB0, 0xB0, 0xB0, 200];
    }
    let [r, g, b] = WindCategory::from_kt(start_kt.or(end_kt)).rgb();
    let a = if group == ModelGroup::Official {
        255
    } else {
        210
    };
    [r, g, b, a]
}

/// Line style per model group.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackStyle {
    pub width_px: f32,
    /// White outline drawn under the line (OFCL only), in px.
    pub underlay_px: Option<f32>,
    /// Draw the tech id at the track's last point.
    pub label: bool,
    /// Higher draws later (on top).
    pub draw_rank: u8,
}

pub fn track_style(group: ModelGroup) -> TrackStyle {
    match group {
        ModelGroup::Official => TrackStyle {
            width_px: 2.5,
            underlay_px: Some(5.0),
            label: true,
            draw_rank: 2,
        },
        ModelGroup::Observed => TrackStyle {
            width_px: 1.5,
            underlay_px: None,
            label: false,
            draw_rank: 1,
        },
        _ => TrackStyle {
            width_px: 1.5,
            underlay_px: None,
            label: true,
            draw_rank: 0,
        },
    }
}

/// Full disclaimer and attribution, one panel line each (no auto-wrap).
/// Wording from `docs/research/spaghetti-models.md` section 4.
pub const DISCLAIMER_LINES: [&str; 6] = [
    "Model guidance from NHC ATCF (aid_public).",
    "Individual models are not official forecasts.",
    "The NHC forecast (OFCL) is the official track.",
    "Data are automated and may contain errors.",
    "Consult NHC and your local NWS office.",
    "Source: NOAA/NHC, public domain.",
];

/// NHC's model catalogue and guidance disclaimer.
pub const MODEL_SUMMARY_URL: &str = "https://www.nhc.noaa.gov/modelsummary.shtml";

/// Data older than this is labelled "(stale)": two missed 15-min refreshes.
pub const STALE_AFTER_SECS: i64 = 30 * 60;

/// `2026100912` -> `"12Z Oct 9"`. Arithmetic on the integer, no string
/// slicing, so malformed values just print `???`.
pub fn format_cycle(init: u32) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let hour = init % 100;
    let day = (init / 100) % 100;
    let month = (init / 10_000) % 100;
    let name = (month as usize)
        .checked_sub(1)
        .and_then(|i| MONTHS.get(i))
        .copied()
        .unwrap_or("???");
    format!("{hour:02}Z {name} {day}")
}

/// Human age of a fetch: "just now", "7 min ago", "2 h 5 min ago".
pub fn format_age(fetched_at_unix: i64, now_unix: i64) -> String {
    let secs = (now_unix - fetched_at_unix).max(0);
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{} min ago", secs / 60)
    } else {
        format!("{} h {} min ago", secs / 3600, (secs % 3600) / 60)
    }
}

pub fn is_stale(fetched_at_unix: i64, now_unix: i64) -> bool {
    now_unix - fetched_at_unix > STALE_AFTER_SECS
}

/// A visible track pre-projected to km offsets from the radar site.
/// Rebuilt only when the site or the store changes, so per frame the draw
/// is a bbox cull plus one multiply-add per vertex (review H5).
#[derive(Clone, Debug)]
pub struct ProjectedTrack {
    pub tech: String,
    pub group: ModelGroup,
    pub km: Vec<Vec2>,
    pub vmax_kt: Vec<Option<u32>>,
    pub min_km: Vec2,
    pub max_km: Vec2,
}

/// Project every default-visible track, ordered by draw rank (models, then
/// CARQ, then OFCL on top).
pub fn project_tracks(
    store: &GuidanceStore,
    origin_lat: f64,
    origin_lon: f64,
) -> Vec<ProjectedTrack> {
    let mut out: Vec<ProjectedTrack> = store
        .iter()
        .flat_map(|g| g.tracks.iter())
        .filter(|t| t.default_visible && t.points.len() >= 2)
        .map(|t| {
            let km: Vec<Vec2> = t
                .points
                .iter()
                .map(|p| geo::point_to_km_offset(origin_lat, origin_lon, (p.lat, p.lon)))
                .collect();
            let min_km = km.iter().copied().reduce(Vec2::min).unwrap_or(Vec2::ZERO);
            let max_km = km.iter().copied().reduce(Vec2::max).unwrap_or(Vec2::ZERO);
            ProjectedTrack {
                tech: t.tech.clone(),
                group: t.group,
                vmax_kt: t.points.iter().map(|p| p.vmax_kt).collect(),
                km,
                min_km,
                max_km,
            }
        })
        .collect();
    out.sort_by_key(|t| track_style(t.group).draw_rank);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adeck::ModelGroup;
    use crate::adeck::{AidTrack, GuidanceStore, StormGuidance, TrackPoint};

    const FIXTURE: &str = include_str!("testdata/aal092026_sample.dat");

    fn fixture_store() -> GuidanceStore {
        let records = crate::adeck::parse_adeck(FIXTURE);
        let mut store = GuidanceStore::default();
        store.insert(crate::adeck::build_guidance(
            "al092026", &records, 2026100912,
        ));
        store
    }

    #[test]
    fn project_tracks_orders_official_last() {
        let tracks = project_tracks(&fixture_store(), 27.0, -87.6);
        let techs: Vec<&str> = tracks.iter().map(|t| t.tech.as_str()).collect();
        assert_eq!(techs, ["AVNI", "HWFI", "TVCN", "CARQ", "OFCL"]);
        let ofcl = tracks.last().unwrap();
        assert!(ofcl.km[0].length() < 0.01, "OFCL tau 0 is at the origin");
        for t in &tracks {
            assert_eq!(t.km.len(), t.vmax_kt.len());
            for p in &t.km {
                assert!(p.x >= t.min_km.x && p.x <= t.max_km.x);
                assert!(p.y >= t.min_km.y && p.y <= t.max_km.y);
            }
        }
    }

    #[test]
    fn project_tracks_skips_hidden_models() {
        let mut store = fixture_store();
        let mut g = store.get("al092026").unwrap().clone();
        g.tracks.push(AidTrack {
            tech: "TABS".into(),
            group: ModelGroup::Trajectory,
            default_visible: false,
            points: g.tracks[0].points.clone(),
        });
        store.insert(g);
        assert!(
            project_tracks(&store, 27.0, -87.6)
                .iter()
                .all(|t| t.tech != "TABS")
        );
    }

    #[test]
    fn projection_is_continuous_across_antimeridian() {
        let point = |lon: f64| TrackPoint {
            tau_h: 0,
            lat: 20.0,
            lon,
            vmax_kt: Some(50),
        };
        let mut store = GuidanceStore::default();
        store.insert(StormGuidance {
            storm_id: "cp012026".into(),
            init: 2026100912,
            has_official: true,
            tracks: vec![AidTrack {
                tech: "AVNI".into(),
                group: ModelGroup::Global,
                default_visible: true,
                points: vec![point(179.5), point(-179.5)],
            }],
            intensity: Vec::new(),
            fetched_at_unix: 0,
        });
        let tracks = project_tracks(&store, 20.0, 179.0);
        let gap = (tracks[0].km[1] - tracks[0].km[0]).length();
        assert!(
            gap < 200.0,
            "1 degree of longitude, not a 360-degree wrap: {gap} km"
        );
    }

    #[test]
    fn wind_category_thresholds_follow_saffir_simpson() {
        let cases = [
            (33, WindCategory::Depression),
            (34, WindCategory::Storm),
            (63, WindCategory::Storm),
            (64, WindCategory::Cat1),
            (82, WindCategory::Cat1),
            (83, WindCategory::Cat2),
            (95, WindCategory::Cat2),
            (96, WindCategory::Cat3),
            (112, WindCategory::Cat3),
            (113, WindCategory::Cat4),
            (136, WindCategory::Cat4),
            (137, WindCategory::Cat5),
        ];
        for (kt, cat) in cases {
            assert_eq!(WindCategory::from_kt(Some(kt)), cat, "{kt} kt");
        }
        assert_eq!(WindCategory::from_kt(None), WindCategory::Unknown);
    }

    #[test]
    fn legend_colors_are_distinct() {
        let mut colors: Vec<u32> = WindCategory::LEGEND.iter().map(|c| c.hex()).collect();
        colors.sort_unstable();
        colors.dedup();
        assert_eq!(colors.len(), 8);
        assert_eq!(WindCategory::Cat5.hex(), 0xFF6060);
    }

    #[test]
    fn segment_color_uses_start_wind_then_end() {
        let start = segment_rgba(ModelGroup::Global, Some(100), Some(20));
        assert_eq!(&start[..3], &WindCategory::Cat3.rgb());
        let fallback = segment_rgba(ModelGroup::Global, None, Some(40));
        assert_eq!(&fallback[..3], &WindCategory::Storm.rgb());
        let unknown = segment_rgba(ModelGroup::Consensus, None, None);
        assert_eq!(&unknown[..3], &WindCategory::Unknown.rgb());
    }

    #[test]
    fn observed_track_is_grey_and_official_is_opaque() {
        assert_eq!(
            segment_rgba(ModelGroup::Observed, Some(140), Some(140)),
            [0xB0, 0xB0, 0xB0, 200]
        );
        assert_eq!(segment_rgba(ModelGroup::Official, Some(50), None)[3], 255);
        assert!(segment_rgba(ModelGroup::Hurricane, Some(50), None)[3] < 255);
    }

    #[test]
    fn official_track_draws_last_and_widest() {
        let ofcl = track_style(ModelGroup::Official);
        let carq = track_style(ModelGroup::Observed);
        let model = track_style(ModelGroup::Hurricane);
        assert!(ofcl.draw_rank > carq.draw_rank && carq.draw_rank > model.draw_rank);
        assert!(ofcl.width_px > model.width_px);
        assert!(ofcl.underlay_px.is_some() && model.underlay_px.is_none());
        assert!(!carq.label && model.label && ofcl.label);
    }

    #[test]
    fn disclaimer_lines_fit_the_panel() {
        // NHC panel content is ~288 px; 10 px Inter is ~5.5 px/char.
        for line in DISCLAIMER_LINES {
            assert!(line.chars().count() <= 48, "{line:?} too long");
        }
        assert!(
            DISCLAIMER_LINES
                .iter()
                .any(|l| l.contains("not official forecasts"))
        );
    }

    #[test]
    fn format_cycle_reads_init_digits() {
        assert_eq!(format_cycle(2026100912), "12Z Oct 9");
        assert_eq!(format_cycle(2026010100), "00Z Jan 1");
        // Garbage never panics.
        assert_eq!(format_cycle(0), "00Z ??? 0");
        assert_eq!(format_cycle(2026130100), "00Z ??? 1");
    }

    #[test]
    fn format_age_buckets() {
        assert_eq!(format_age(1000, 1000), "just now");
        assert_eq!(format_age(1000, 1059), "just now");
        assert_eq!(format_age(1000, 1060), "1 min ago");
        assert_eq!(format_age(1000, 4599), "59 min ago");
        assert_eq!(format_age(1000, 4600), "1 h 0 min ago");
        assert_eq!(format_age(1000, 900), "just now", "clock skew clamps to 0");
    }

    #[test]
    fn is_stale_boundary() {
        assert!(!is_stale(0, STALE_AFTER_SECS));
        assert!(is_stale(0, STALE_AFTER_SECS + 1));
    }
}
