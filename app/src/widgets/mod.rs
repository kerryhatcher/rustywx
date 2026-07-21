//! Reusable Ply UI controls used by the radar application.

use ply_engine::prelude::FontAsset;

/// Icon font for UI symbols (Nerd Fonts Symbols-only). Glyphs are Private-Use
/// codepoints (`\u{f0…}`) — see the `nf` module for the ones in use. Inter has
/// no symbol coverage, so every icon must render with this font.
pub static SYMBOL_FONT: FontAsset = FontAsset::Path("assets/fonts/SymbolsNerdFontMono-Regular.ttf");

/// Nerd Font glyph codepoints used across the UI, named for grep-ability.
pub mod nf {
    pub const CLOSE: &str = "\u{f00d}"; // fa-times
    pub const GEAR: &str = "\u{f013}"; // fa-gear
    pub const WARNING: &str = "\u{f071}"; // fa-warning
    pub const HURRICANE: &str = "\u{f0898}"; // md-weather-hurricane
    pub const EXTERNAL_LINK: &str = "\u{f08e}"; // fa-external-link
    pub const REFRESH: &str = "\u{f021}"; // fa-refresh
    pub const RADAR: &str = "\u{f0bce}"; // md-radar
    pub const CHEVRON_DOWN: &str = "\u{f078}"; // fa-chevron-down
    pub const CHEVRON_UP: &str = "\u{f077}"; // fa-chevron-up
    pub const CHEVRON_RIGHT: &str = "\u{f054}"; // fa-chevron-right
    pub const EXPAND: &str = "\u{f065}"; // fa-expand (enter fullscreen)
    pub const COMPRESS: &str = "\u{f066}"; // fa-compress (exit fullscreen)
}

pub mod collapsing;
pub mod dropdown;
pub mod glass_panel;
pub mod settings;
pub mod shortcuts;
pub mod toast;
pub mod toggle;
