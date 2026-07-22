//! User-configurable settings, persisted via Ply `storage` (see `cache.rs`).
//!
//! The settings panel (`widgets/settings.rs`) reads/writes this struct;
//! `main.rs` and `data.rs` consume the fields to drive actual behaviour
//! (`animation_level`, `tdbz_kernel`, poll intervals).

use serde::{Deserialize, Serialize};

/// How much motion the Stage 6 "Observatory Look" animations use.
///
/// Consumed by the animation/easing code in `main.rs`: `Full` is the
/// original behaviour, `Subtle` keeps fades but drops the sweep line and
/// spring bounce, `None` renders the final/static state with no motion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationLevel {
    #[default]
    Full,
    Subtle,
    None,
}

/// TDBZ clutter-filter kernel size preset (see `scope.rs`).
///
/// [`TdbzKernel::size`] is passed into `scope::rasterize` from the `main.rs`
/// call site on every re-raster.
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

/// Serde default for [`Settings::show_radar`] (missing in older config files).
fn default_show_radar() -> bool {
    true
}

/// Serde default for scope-decoration toggles (missing in older config files).
fn default_true() -> bool {
    true
}

/// Serde default for [`Settings::cc_gate_threshold`] (missing in older configs).
fn default_cc_threshold() -> f32 {
    0.80
}

/// Serde default for [`Settings::refl_floor_dbz`] (missing in older configs).
fn default_refl_floor() -> f32 {
    7.0
}

/// Serde default for [`Settings::vel_sd_threshold`] (missing in older configs).
fn default_vel_sd() -> f32 {
    7.0
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
    /// Read once at startup and threaded into `data::spawn_worker` as the
    /// healthy-poll `Duration` (see `main.rs`); `data::POLL_INTERVAL`
    /// remains the fallback used before the persisted value loads.
    pub poll_interval_secs: u64,
    /// Seconds between NHC tropical-data refreshes.
    ///
    /// Read every frame in `main.rs`'s NHC-fetch-scheduling block in place
    /// of `nhc::POLL_INTERVAL`.
    pub nhc_refresh_secs: u64,
    /// Whether state borders are shown by default at startup.
    pub show_borders: bool,
    /// Whether NWS watch polygons are shown by default at startup.
    #[serde(default = "default_true", alias = "show_alerts")]
    pub show_watches: bool,
    /// Whether NWS warning polygons are shown by default at startup.
    #[serde(default = "default_true")]
    pub show_warnings: bool,
    /// Whether the NHC tropical panel is shown by default at startup.
    pub show_nhc: bool,
    /// Whether the Radar controls side panel is shown by default at startup.
    /// Defaults `true` (open) so the radar controls stay discoverable after
    /// moving off the main controls bar.
    #[serde(default = "default_show_radar")]
    pub show_radar: bool,
    /// Whether the radar data layer (texture + site markers) is shown at startup.
    #[serde(default = "default_true")]
    pub show_radar_data: bool,
    /// Whether the tropical (NHC) data layer master-gate is on at startup.
    #[serde(default = "default_true")]
    pub show_nhc_data: bool,
    /// Whether the rotating radar sweep beam is drawn.
    #[serde(default = "default_true")]
    pub show_sweep: bool,
    /// Whether the scope range rings and cardinal crosshairs are drawn.
    #[serde(default = "default_true")]
    pub show_scope_rings: bool,
    /// Observatory-look animation intensity.
    pub animation_level: AnimationLevel,
    /// TDBZ clutter-filter kernel size preset.
    pub tdbz_kernel: TdbzKernel,
    /// Render body/label text with the OpenDyslexic font (accessibility).
    #[serde(default)]
    pub dyslexic_font: bool,
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
    /// Whether correlation-coefficient gating suppresses non-meteorological
    /// echo (CC < threshold) from the Reflectivity display. Default on.
    #[serde(default = "default_true")]
    pub cc_gate_enabled: bool,
    /// CC value below which a Reflectivity gate is suppressed when gating is on.
    #[serde(default = "default_cc_threshold")]
    pub cc_gate_threshold: f32,
    /// Whether reflectivity noise-floor cut suppresses weak Reflectivity gates
    /// below a fixed dBZ threshold. Default on.
    #[serde(default = "default_true")]
    pub refl_floor_enabled: bool,
    /// Reflectivity floor (dBZ) below which gates are suppressed when floor is on.
    /// Applied as a per-range maximum with the existing range floor.
    #[serde(default = "default_refl_floor")]
    pub refl_floor_dbz: f32,
    /// Whether Velocity spatial-SD censoring nulls noisy gates (dealias
    /// artifacts / non-meteorological velocity noise). Default on.
    #[serde(default = "default_true")]
    pub vel_sd_censor_enabled: bool,
    /// Local velocity standard-deviation (m/s) above which a gate is
    /// censored when SD-censoring is on.
    #[serde(default = "default_vel_sd")]
    pub vel_sd_threshold: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            default_site: "KFFC".to_string(),
            poll_interval_secs: 120,
            nhc_refresh_secs: 300,
            show_borders: true,
            show_watches: true,
            show_warnings: true,
            show_nhc: false,
            show_radar: false,
            show_radar_data: true,
            show_nhc_data: true,
            show_sweep: true,
            show_scope_rings: true,
            animation_level: AnimationLevel::default(),
            tdbz_kernel: TdbzKernel::default(),
            dyslexic_font: false,
            user_lat: None,
            user_lon: None,
            location_input: String::new(),
            show_location: false,
            center_on_location: false,
            cc_gate_enabled: true,
            cc_gate_threshold: 0.80,
            refl_floor_enabled: true,
            refl_floor_dbz: 7.0,
            vel_sd_censor_enabled: true,
            vel_sd_threshold: 7.0,
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
        assert!(settings.show_watches);
        assert!(settings.show_warnings);
        assert!(!settings.show_nhc);
        assert!(!settings.show_radar);
        assert!(settings.show_radar_data);
        assert!(settings.show_nhc_data);
        assert!(settings.show_sweep);
        assert!(settings.show_scope_rings);
        assert_eq!(settings.animation_level, AnimationLevel::Full);
        assert_eq!(settings.tdbz_kernel, TdbzKernel::Default);
        assert_eq!(settings.tdbz_kernel.size(), 9);
        assert!(settings.cc_gate_enabled);
        assert_eq!(settings.cc_gate_threshold, 0.80);
        assert!(settings.refl_floor_enabled);
        assert_eq!(settings.refl_floor_dbz, 7.0);
        assert!(settings.vel_sd_censor_enabled);
        assert_eq!(settings.vel_sd_threshold, 7.0);
    }

    #[test]
    fn serde_round_trip() {
        let settings = Settings {
            default_site: "KTLX".to_string(),
            poll_interval_secs: 60,
            nhc_refresh_secs: 180,
            show_borders: false,
            show_watches: true,
            show_warnings: false,
            show_nhc: true,
            show_radar: false,
            show_radar_data: false,
            show_nhc_data: true,
            show_sweep: false,
            show_scope_rings: false,
            animation_level: AnimationLevel::Subtle,
            tdbz_kernel: TdbzKernel::Aggressive,
            dyslexic_font: true,
            user_lat: Some(35.5),
            user_lon: Some(-97.5),
            location_input: "Oklahoma City".to_string(),
            show_location: true,
            center_on_location: true,
            cc_gate_enabled: false,
            cc_gate_threshold: 0.85,
            refl_floor_enabled: false,
            refl_floor_dbz: 5.0,
            vel_sd_censor_enabled: false,
            vel_sd_threshold: 8.0,
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
            "animation_level":"Full","tdbz_kernel":"Default","dyslexic_font":false}"#;
        let s: Settings = serde_json::from_str(json).expect("back-compat deserialize");
        assert!(s.user_lat.is_none());
        assert!(!s.center_on_location);
        // Legacy `show_alerts` key aliases into both new toggles.
        assert!(s.show_watches);
        assert!(s.show_warnings);
        // Missing show_radar defaults to open.
        assert!(s.show_radar);
        // Missing data-layer toggles default on.
        assert!(s.show_radar_data);
        assert!(s.show_nhc_data);
        // Missing scope-decoration toggles default on.
        assert!(s.show_sweep);
        assert!(s.show_scope_rings);
        // Missing CC-gate fields default to on/0.80.
        assert!(s.cc_gate_enabled);
        assert_eq!(s.cc_gate_threshold, 0.80);
        // Missing noise-floor fields default to on/7.0.
        assert!(s.refl_floor_enabled);
        assert_eq!(s.refl_floor_dbz, 7.0);
        // Missing velocity-SD fields default to on/7.0.
        assert!(s.vel_sd_censor_enabled);
        assert_eq!(s.vel_sd_threshold, 7.0);
    }
}
