//! Dismissable glass toast/banner for user-facing error messages (Stage 7).
//!
//! Network failures (radar fetch, borders, alerts, NHC) are logged to
//! stderr for diagnostics but are otherwise invisible to the user — Stage 7
//! requires a friendly, non-blocking surface instead of silence or a crash.
//! `Toast` is that surface: a single most-recent message that auto-fades
//! after [`VISIBLE_SECS`], and can also be dismissed early by clicking it.

use crate::widgets::glass_panel::{ACCENT, GLASS_BORDER, TEXT_PRIMARY};
use ply_engine::prelude::*;

/// Click target covering the whole toast — clicking it dismisses early.
pub const DISMISS_ID: &str = "toast-dismiss";

/// How long a toast stays fully visible before fading out.
pub const VISIBLE_SECS: f64 = 5.0;
/// Fade-out duration after [`VISIBLE_SECS`] elapses.
pub const FADE_SECS: f64 = 1.0;

/// A short-lived error/status banner shown over the scope.
#[derive(Clone)]
pub struct Toast {
    pub message: String,
    shown_at: f64,
}

impl Toast {
    pub fn new(message: impl Into<String>, now: f64) -> Self {
        Toast {
            message: message.into(),
            shown_at: now,
        }
    }

    /// Opacity in `[0, 1]` for the current wall-clock time, or `None` once
    /// the toast has fully faded — the caller should drop it in that case.
    pub fn opacity(&self, now: f64) -> Option<f32> {
        let age = now - self.shown_at;
        if age < VISIBLE_SECS {
            Some(1.0)
        } else if age < VISIBLE_SECS + FADE_SECS {
            Some((1.0 - (age - VISIBLE_SECS) / FADE_SECS) as f32)
        } else {
            None
        }
    }
}

/// Category of user-facing error, mapped to a short friendly message by
/// [`friendly_message`]. Raw error details (from `anyhow`/decode errors)
/// stay in the `eprintln!` logs at the call site — never shown to the user.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErrorKind {
    /// Transient fetch failure: borders, alerts, or NHC data.
    Network,
    /// Radar volume fetch/decode failure from the background worker.
    RadarData,
}

/// Map an error category to a short, human-friendly toast message.
pub fn friendly_message(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::Network => "Network error — retrying…",
        ErrorKind::RadarData => "Radar data unavailable — retrying…",
    }
}

/// Draw the toast banner, top-center over the scope, faded by `opacity`.
pub fn draw(ui: &mut Ui<'_, ()>, toast: &Toast, opacity: f32) {
    let panel_w = 360.0;
    let panel_h = 40.0;
    let x = (screen_width() - panel_w) / 2.0;
    let y = 16.0;

    let bg = Color::rgba(18.0, 22.0, 30.0, 230.0 * opacity);
    let border = Color::rgba(
        GLASS_BORDER.r,
        GLASS_BORDER.g,
        GLASS_BORDER.b,
        40.0 * opacity,
    );
    let text_color = Color::rgba(
        TEXT_PRIMARY.r,
        TEXT_PRIMARY.g,
        TEXT_PRIMARY.b,
        255.0 * opacity,
    );
    let accent = Color::rgba(ACCENT.r, ACCENT.g, ACCENT.b, 255.0 * opacity);

    ui.element()
        .id(DISMISS_ID)
        .width(fixed!(panel_w))
        .height(fixed!(panel_h))
        .background_color(bg)
        .corner_radius(8.0)
        .border(|b| b.color(border).all(1))
        .floating(|f| f.offset((x, y)).z_index(400).attach_root())
        .layout(|l| {
            l.direction(LeftToRight)
                .padding(12)
                .gap(8)
                .align(Left, CenterY)
        })
        .children(|ui| {
            ui.text("⚠", |t| t.font_size(14).color(accent));
            ui.text(&toast.message, |t| t.font_size(12).color(text_color));
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn friendly_message_never_leaks_raw_error_text() {
        // Regardless of the underlying anyhow error, the user sees only
        // one of these short canned strings — never the raw `{e}` detail.
        assert_eq!(
            friendly_message(ErrorKind::Network),
            "Network error — retrying…"
        );
        assert_eq!(
            friendly_message(ErrorKind::RadarData),
            "Radar data unavailable — retrying…"
        );
    }

    #[test]
    fn toast_is_fully_visible_then_fades_then_expires() {
        let toast = Toast::new("test", 100.0);
        assert_eq!(toast.opacity(100.0), Some(1.0));
        assert_eq!(toast.opacity(100.0 + VISIBLE_SECS - 0.1), Some(1.0));

        let mid_fade = toast
            .opacity(100.0 + VISIBLE_SECS + FADE_SECS / 2.0)
            .unwrap();
        assert!(mid_fade > 0.0 && mid_fade < 1.0);

        assert_eq!(toast.opacity(100.0 + VISIBLE_SECS + FADE_SECS + 0.1), None);
    }
}
