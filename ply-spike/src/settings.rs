//! User-configurable settings, persisted via Ply `storage` (see `cache.rs`).
//!
//! Stage 7 foundation: the schema and persistence plumbing land here first;
//! the settings panel (`widgets/settings.rs`) reads/writes this struct, and
//! later Stage 7 tasks consume the fields that aren't wired to behaviour yet
//! (`animation_level`, `tdbz_kernel`, poll intervals).

use serde::{Deserialize, Serialize};

/// How much motion the Stage 6 "Observatory Look" animations use.
///
/// `// Stage 7: consumed by the animation/easing code in main.rs (pending)`
/// — no animation currently reads this; it's stored for that later task.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationLevel {
    #[default]
    Full,
    Subtle,
    None,
}

/// TDBZ clutter-filter kernel size preset (see `scope.rs`).
///
/// `// Stage 7: consumed by the TDBZ filter in scope.rs (pending)` — the
/// filter is currently hardcoded to 9×9; a later task reads
/// [`TdbzKernel::size`] instead.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TdbzKernel {
    Sensitive,
    #[default]
    Default,
    Aggressive,
}

impl TdbzKernel {
    /// Kernel edge length in gates (odd, centered).
    pub fn size(self) -> u32 {
        match self {
            TdbzKernel::Sensitive => 5,
            TdbzKernel::Default => 9,
            TdbzKernel::Aggressive => 13,
        }
    }

    /// Cycle to the next preset (used by the settings panel's control).
    pub fn next(self) -> Self {
        match self {
            TdbzKernel::Sensitive => TdbzKernel::Default,
            TdbzKernel::Default => TdbzKernel::Aggressive,
            TdbzKernel::Aggressive => TdbzKernel::Sensitive,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            TdbzKernel::Sensitive => "Sensitive (5×5)",
            TdbzKernel::Default => "Default (9×9)",
            TdbzKernel::Aggressive => "Aggressive (13×13)",
        }
    }
}

impl AnimationLevel {
    /// Cycle to the next level (used by the settings panel's control).
    pub fn next(self) -> Self {
        match self {
            AnimationLevel::Full => AnimationLevel::Subtle,
            AnimationLevel::Subtle => AnimationLevel::None,
            AnimationLevel::None => AnimationLevel::Full,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AnimationLevel::Full => "Full",
            AnimationLevel::Subtle => "Subtle",
            AnimationLevel::None => "None",
        }
    }
}

/// User-configurable app settings, persisted as `"settings.json"` via
/// [`crate::cache::Cache::save_settings`] / [`crate::cache::Cache::load_settings`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    /// Radar site ID (e.g. `"KFFC"`) used on first launch, before any
    /// explicit site preference has been saved.
    pub default_site: String,
    /// Seconds between background radar-scan polls.
    ///
    /// `// Stage 7: consumed by data.rs POLL_INTERVAL (pending)`
    pub poll_interval_secs: u64,
    /// Seconds between NHC tropical-data refreshes.
    ///
    /// `// Stage 7: consumed by nhc.rs POLL_INTERVAL (pending)`
    pub nhc_refresh_secs: u64,
    /// Whether state borders are shown by default at startup.
    pub show_borders: bool,
    /// Whether NWS alert polygons are shown by default at startup.
    pub show_alerts: bool,
    /// Whether the NHC tropical panel is shown by default at startup.
    pub show_nhc: bool,
    /// Observatory-look animation intensity.
    pub animation_level: AnimationLevel,
    /// TDBZ clutter-filter kernel size preset.
    pub tdbz_kernel: TdbzKernel,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            default_site: "KFFC".to_string(),
            poll_interval_secs: 120,
            nhc_refresh_secs: 300,
            show_borders: true,
            show_alerts: true,
            show_nhc: false,
            animation_level: AnimationLevel::default(),
            tdbz_kernel: TdbzKernel::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_existing_startup_behaviour() {
        let settings = Settings::default();
        assert_eq!(settings.default_site, "KFFC");
        assert!(settings.show_borders);
        assert!(settings.show_alerts);
        assert!(!settings.show_nhc);
        assert_eq!(settings.animation_level, AnimationLevel::Full);
        assert_eq!(settings.tdbz_kernel, TdbzKernel::Default);
        assert_eq!(settings.tdbz_kernel.size(), 9);
    }

    #[test]
    fn serde_round_trip() {
        let settings = Settings {
            default_site: "KTLX".to_string(),
            poll_interval_secs: 60,
            nhc_refresh_secs: 180,
            show_borders: false,
            show_alerts: true,
            show_nhc: true,
            animation_level: AnimationLevel::Subtle,
            tdbz_kernel: TdbzKernel::Aggressive,
        };
        let json = serde_json::to_string(&settings).expect("serialize");
        let restored: Settings = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, settings);
    }

    #[test]
    fn enum_cycles_wrap_around() {
        assert_eq!(AnimationLevel::Full.next(), AnimationLevel::Subtle);
        assert_eq!(AnimationLevel::Subtle.next(), AnimationLevel::None);
        assert_eq!(AnimationLevel::None.next(), AnimationLevel::Full);

        assert_eq!(TdbzKernel::Sensitive.next(), TdbzKernel::Default);
        assert_eq!(TdbzKernel::Default.next(), TdbzKernel::Aggressive);
        assert_eq!(TdbzKernel::Aggressive.next(), TdbzKernel::Sensitive);
    }
}
