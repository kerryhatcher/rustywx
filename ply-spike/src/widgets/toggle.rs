//! Mutually exclusive button group widget.

use ply_engine::prelude::*;

const INACTIVE_BACKGROUND: u32 = 0x1E1B1B;
const ACTIVE_BACKGROUND: u32 = 0x3F684A;
const TEXT_COLOR: u32 = 0xE8E0DC;

/// One option in a toggle group.
pub struct ToggleOption<T> {
    pub id: &'static str,
    pub label: &'static str,
    pub value: T,
}

/// Draw a mutually exclusive group of compact buttons.
pub fn draw<T: Copy + PartialEq>(ui: &mut Ui<'_, ()>, selected: T, options: &[ToggleOption<T>]) {
    for option in options {
        ui.element()
            .id(option.id)
            .width(fit!())
            .height(fixed!(24.0))
            .background_color(if selected == option.value {
                ACTIVE_BACKGROUND
            } else {
                INACTIVE_BACKGROUND
            })
            .corner_radius(4.0)
            .layout(|layout| layout.padding((0, 8, 0, 8)).align(CenterX, CenterY))
            .children(|ui| {
                ui.text(option.label, |text| text.font_size(12).color(TEXT_COLOR));
            });
    }
}

/// Return the value of the option pressed during the current frame.
pub fn pressed<T: Copy>(ply: &Ply<()>, options: &[ToggleOption<T>]) -> Option<T> {
    options
        .iter()
        .find(|option| ply.is_just_pressed(option.id))
        .map(|option| option.value)
}
