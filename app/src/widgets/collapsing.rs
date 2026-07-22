//! Reusable expanding/collapsing section for future detail panels.

use ply_engine::prelude::*;

const HEADER_BACKGROUND: u32 = 0x1E1B1B;
const BODY_BACKGROUND: u32 = 0x171A1F;
const TEXT_COLOR: u32 = 0xE8E0DC;

/// Persistent state for a collapsible section.
pub struct CollapsingState {
    open: bool,
}

impl CollapsingState {
    pub fn new(open: bool) -> Self {
        Self { open }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Declare the header and, when expanded, its body content.
    pub fn draw(
        &self,
        ui: &mut Ui<'_, crate::widgets::ChartWidget>,
        header_id: &'static str,
        title: &str,
        body: impl FnOnce(&mut Ui<'_, crate::widgets::ChartWidget>),
    ) {
        ui.element()
            .width(grow!())
            .height(fit!())
            .layout(|layout| layout.direction(TopToBottom).gap(2))
            .children(|ui| {
                ui.element()
                    .id(header_id)
                    .width(grow!())
                    .height(fixed!(26.0))
                    .background_color(HEADER_BACKGROUND)
                    .corner_radius(4.0)
                    .layout(|layout| layout.padding((0, 8, 0, 8)).align(Left, CenterY))
                    .accessibility(|a| a.button(title).checked(self.open))
                    .children(|ui| {
                        let arrow = if self.open { '▾' } else { '▸' };
                        ui.text(&format!("{arrow} {title}"), |text| {
                            text.font_size(12).color(TEXT_COLOR)
                        });
                    });

                if self.open {
                    ui.element()
                        .width(grow!())
                        .height(fit!())
                        .background_color(BODY_BACKGROUND)
                        .corner_radius(4.0)
                        .layout(|layout| layout.padding(8))
                        .children(body);
                }
            });
    }

    /// Toggle the section after the Ply frame has been shown.
    pub fn handle_input(&mut self, ply: &Ply<crate::widgets::ChartWidget>, header_id: &'static str) -> bool {
        if ply.is_just_pressed(header_id) {
            self.open = !self.open;
            true
        } else {
            false
        }
    }
}

impl Default for CollapsingState {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::CollapsingState;

    #[test]
    fn starts_in_requested_state() {
        assert!(CollapsingState::new(true).is_open());
        assert!(!CollapsingState::default().is_open());
    }
}
