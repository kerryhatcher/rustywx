//! Reusable Ply UI controls used by the radar application.

use crate::forecast::Hour;
use ply_engine::prelude::FontAsset;

/// Icon font for UI symbols (Nerd Fonts Symbols-only). Glyphs are Private-Use
/// codepoints (`\u{f0…}`) — see the `nf` module for the ones in use. Inter has
/// no symbol coverage, so every icon must render with this font.
pub static SYMBOL_FONT: FontAsset = FontAsset::Bytes {
    file_name: "SymbolsNerdFontMono-Regular.ttf",
    data: include_bytes!("../../assets/fonts/SymbolsNerdFontMono-Regular.ttf"),
};

/// Nerd Font glyph codepoints used across the UI, named for grep-ability.
pub mod nf {
    pub const CLOSE: &str = "\u{f00d}"; // fa-times
    pub const GEAR: &str = "\u{f013}"; // fa-gear
    pub const WARNING: &str = "\u{f071}"; // fa-warning
    pub const HURRICANE: &str = "\u{f0898}"; // md-weather-hurricane

    // Weather glyphs (Material Design Icons — same set as HURRICANE).
    // Verify rendering against the SYMBOL_FONT; the paired text label in
    // `forecast::wmo_icon` carries meaning even if a glyph shows as tofu.
    pub const WX_SUNNY: &str = "\u{f0599}"; // md-weather-sunny
    pub const WX_NIGHT: &str = "\u{f0594}"; // md-weather-night
    pub const WX_PARTLY: &str = "\u{f0595}"; // md-weather-partly-cloudy
    pub const WX_CLOUDY: &str = "\u{f0590}"; // md-weather-cloudy
    pub const WX_FOG: &str = "\u{f0591}"; // md-weather-fog
    pub const WX_RAINY: &str = "\u{f0597}"; // md-weather-rainy
    pub const WX_POURING: &str = "\u{f0596}"; // md-weather-pouring
    pub const WX_SNOWY: &str = "\u{f0598}"; // md-weather-snowy
    pub const WX_LIGHTNING: &str = "\u{f0593}"; // md-weather-lightning
    pub const EXTERNAL_LINK: &str = "\u{f08e}"; // fa-external-link
    pub const REFRESH: &str = "\u{f021}"; // fa-refresh
    pub const RADAR: &str = "\u{f0bce}"; // md-radar
    pub const CHEVRON_DOWN: &str = "\u{f078}"; // fa-chevron-down
    pub const CHEVRON_UP: &str = "\u{f077}"; // fa-chevron-up
    pub const CHEVRON_RIGHT: &str = "\u{f054}"; // fa-chevron-right
    pub const EXPAND: &str = "\u{f065}"; // fa-expand (enter fullscreen)
    pub const COMPRESS: &str = "\u{f066}"; // fa-compress (exit fullscreen)
}

/// Payload for Ply's custom-draw hook (`Ply::show`'s `handle_custom_command`).
/// Carries the data an element needs the app to render itself via macroquad.
#[derive(Clone, Debug, Default)]
pub enum ChartWidget {
    /// Nothing custom to draw (the default for ordinary elements).
    #[default]
    None,
    /// Hourly rain-chance line graph — the hours to plot.
    RainChart(Vec<Hour>),
}

pub mod collapsing;
pub mod dropdown;
pub mod glass_panel;
pub mod settings;
pub mod shortcuts;
pub mod toast;
pub mod toggle;
