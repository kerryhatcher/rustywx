//! Reusable Ply UI controls used by the radar application.

use ply_engine::prelude::FontAsset;

/// Font for symbol glyphs (✕, ⚙, arrows, ⚠) that Inter lacks.
pub static SYMBOL_FONT: FontAsset = FontAsset::Path("assets/fonts/DejaVuSansMono.ttf");

pub mod collapsing;
pub mod dropdown;
pub mod glass_panel;
pub mod settings;
pub mod shortcuts;
pub mod toast;
pub mod toggle;
