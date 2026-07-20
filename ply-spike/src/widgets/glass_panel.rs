//! Frosted-glass panel widget and observatory visual theme.
//!
//! Stage 6 — the "Observatory Look". This module owns:
//!   - `BLUR_SHADER` — the custom GLSL ES 1.00 Gaussian blur fragment shader,
//!     embedded via `include_str!` from `assets/shaders/blur.frag`.
//!   - The teal/cyan accent colour and glass palette constants.
//!   - `glass()` — a reusable styling helper that applies the frosted-glass
//!     background, blur shader, and subtle border to any element builder.

use ply_engine::ElementBuilder;
use ply_engine::prelude::*;

// ── Shader ───────────────────────────────────────────────────────────
//
// Loading mechanism is settled (see stage-6-observatory-look.md):
// `ShaderAsset::Source` + `include_str!` — embeds the GLSL in the binary
// at compile time, zero-copy at runtime, no filesystem/CWD dependency.
// `assets/shaders/blur.frag` is the *input* to `include_str!`, kept in the
// repo as the source of truth (mirrors the Ply built-in shader layout).

/// Custom Gaussian blur fragment shader for the frosted-glass effect.
///
/// Validated by Spike S1. Declares the required `u_resolution` /
/// `u_position` auto-uniforms (see Ply issue #5) and uses fixed 5×5
/// constant loop bounds (required on some GLSL ES 1.00 drivers).
pub const BLUR_SHADER: ShaderAsset = ShaderAsset::Source {
    file_name: "rustywx_blur",
    fragment: include_str!("../../assets/shaders/blur.frag"),
};

// ── Observatory theme ────────────────────────────────────────────────
//
// Colour values mirror `docs/observatory-mockup.html`:
//   --accent:       #0dc5b8  (teal/cyan)
//   --glass-bg:     rgba(18, 22, 30, 0.85)
//   --glass-border: rgba(255, 255, 255, 0.06)

/// Teal/cyan accent colour used on all interactive elements.
pub const ACCENT: Color = Color::u_rgb(0x0d, 0xc5, 0xb8);

/// Accent colour at reduced opacity (for glow backgrounds / hovers).
pub const ACCENT_GLOW: Color = Color::rgba(13.0, 197.0, 184.0, 64.0);

/// Semi-transparent dark glass background: rgba(18, 22, 30, 0.85).
pub const GLASS_BG: Color = Color::rgba(18.0, 22.0, 30.0, 217.0);

/// Subtle light glass border: rgba(255, 255, 255, 0.06).
pub const GLASS_BORDER: Color = Color::rgba(255.0, 255.0, 255.0, 15.0);

/// Dark panel background (solid, for inner cards).
pub const PANEL_BG: Color = Color::u_rgb(0x12, 0x16, 0x1e);

/// Inner card background (slightly lighter than panels).
pub const CARD_BG: Color = Color::u_rgb(0x17, 0x1a, 0x1f);

/// Muted text colour (secondary labels).
pub const TEXT_MUTED: Color = Color::u_rgb(0x9e, 0x95, 0x90);

/// Primary text colour.
pub const TEXT_PRIMARY: Color = Color::u_rgb(0xe8, 0xe0, 0xdc);

/// Apply frosted-glass styling to an element builder: a semi-transparent dark
/// background, rounded corners, and a subtle light border.
///
/// The frosted-glass *look* comes from the semi-transparent dark background
/// composited over the gradient scope behind it — NOT from the blur shader.
/// Ply's `.shader()` captures the element's own children to an offscreen
/// buffer and blurs them, which degrades text readability and can break
/// content rendering inside large panels. `BLUR_SHADER` remains defined and
/// validated (Spike S1) for use on non-text decorative elements, but is not
/// applied to content-bearing panels.
pub fn glass(builder: ElementBuilder<'_, ()>) -> ElementBuilder<'_, ()> {
    builder
        .background_color(GLASS_BG)
        .corner_radius(8.0)
        .border(|b| b.color(GLASS_BORDER).all(1))
}
