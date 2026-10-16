//! Presentation rules for the spaghetti-models overlay: wind-category colours,
//! per-segment colour, line styles, the km-projection cache, cycle/age text,
//! the disclaimer, and intensity-chart scaling. Pure: no drawing calls.

use crate::adeck::ModelGroup;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adeck::ModelGroup;

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
}
