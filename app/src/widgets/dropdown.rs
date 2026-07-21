//! Searchable, keyboard-navigable dropdown composed from Ply elements.

use ply_engine::prelude::*;

const BUTTON_BACKGROUND: u32 = 0x1E1B1B;
const ACTIVE_BACKGROUND: u32 = 0x3A3533;
const PANEL_BACKGROUND: u32 = 0x1A1D22;
const TEXT_COLOR: u32 = 0xE8E0DC;
const MUTED_COLOR: u32 = 0x9E9590;
const ROW_HEIGHT: f32 = 22.0;
const FILTER_HEIGHT: f32 = 24.0;
const PANEL_PADDING: f32 = 4.0;

/// Stable Ply element IDs and layout settings for one dropdown instance.
#[derive(Clone, Copy)]
pub struct DropdownConfig {
    pub button_id: &'static str,
    pub panel_id: &'static str,
    pub option_id: &'static str,
    pub width: f32,
    pub visible_rows: usize,
    pub panel_offset: (f32, f32),
    pub searchable: bool,
}

impl DropdownConfig {
    fn panel_height(self) -> f32 {
        let filter_height = if self.searchable { FILTER_HEIGHT } else { 0.0 };
        filter_height + self.visible_rows as f32 * ROW_HEIGHT + PANEL_PADDING * 2.0
    }
}

/// One selectable dropdown entry. `source_index` is returned on selection.
pub struct DropdownOption {
    pub source_index: usize,
    pub label: String,
    pub search_text: String,
}

/// Persistent interaction state for a dropdown.
#[derive(Default)]
pub struct DropdownState {
    open: bool,
    filter: String,
    scroll: usize,
    cursor: usize,
}

impl DropdownState {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn close(&mut self) {
        self.open = false;
        self.filter.clear();
        self.scroll = 0;
        self.cursor = 0;
    }

    /// Declare the dropdown button and, when open, its floating panel.
    pub fn draw(
        &self,
        ui: &mut Ui<'_, ()>,
        config: DropdownConfig,
        button_label: &str,
        options: &[DropdownOption],
        selected_index: Option<usize>,
    ) {
        ui.element()
            .id(config.button_id)
            .width(fit!())
            .height(fixed!(24.0))
            .background_color(BUTTON_BACKGROUND)
            .corner_radius(4.0)
            .layout(|layout| layout.padding((0, 8, 0, 8)).gap(4).align(CenterX, CenterY))
            .accessibility(|a| a.button(button_label))
            .children(|ui| {
                ui.text(button_label, |text| text.font_size(12).color(TEXT_COLOR));
                // Caret uses the symbol font (Inter lacks ▾/▴).
                ui.text(if self.open { "▴" } else { "▾" }, |text| {
                    text.font_size(10)
                        .font(&super::SYMBOL_FONT)
                        .color(TEXT_COLOR)
                });
            });

        if !self.open {
            return;
        }

        let filtered = self.filtered_indices(options);
        let visible = self.visible_range(filtered.len(), config.visible_rows);

        ui.element()
            .id(config.panel_id)
            .width(fixed!(config.width))
            .height(fixed!(config.panel_height()))
            .background_color(PANEL_BACKGROUND)
            .corner_radius(6.0)
            .floating(|floating| {
                floating
                    .offset(config.panel_offset)
                    .z_index(100)
                    .attach_root()
            })
            .layout(|layout| {
                layout
                    .direction(TopToBottom)
                    .padding(PANEL_PADDING as u16)
                    .gap(2)
            })
            .children(|ui| {
                if config.searchable {
                    let filter_label = if self.filter.is_empty() {
                        "Type to filter…".to_string()
                    } else {
                        format!("Filter: {}_", self.filter)
                    };
                    ui.element()
                        .width(grow!())
                        .height(fixed!(FILTER_HEIGHT - 2.0))
                        .background_color(BUTTON_BACKGROUND)
                        .corner_radius(3.0)
                        .layout(|layout| layout.padding((0, 6, 0, 6)).align(Left, CenterY))
                        .children(|ui| {
                            ui.text(&filter_label, |text| text.font_size(11).color(MUTED_COLOR));
                        });
                }

                if visible.is_empty() {
                    ui.element()
                        .width(grow!())
                        .height(fixed!(ROW_HEIGHT))
                        .layout(|layout| layout.padding((0, 6, 0, 6)).align(Left, CenterY))
                        .children(|ui| {
                            ui.text("No matches", |text| text.font_size(11).color(MUTED_COLOR));
                        });
                    return;
                }

                for option_position in visible.clone() {
                    let option = &options[filtered[option_position]];
                    let highlighted = option_position == self.cursor
                        || selected_index == Some(option.source_index);
                    ui.element()
                        .id((config.option_id, option.source_index as u32))
                        .width(grow!())
                        .height(fixed!(ROW_HEIGHT))
                        .background_color(if highlighted {
                            ACTIVE_BACKGROUND
                        } else {
                            0x00000000
                        })
                        .corner_radius(3.0)
                        .layout(|layout| layout.padding((0, 6, 0, 6)).align(Left, CenterY))
                        .accessibility(|a| a.button(&option.label))
                        .children(|ui| {
                            ui.text(&option.label, |text| text.font_size(12).color(TEXT_COLOR));
                        });
                }
            });
    }

    /// Process mouse and keyboard input after `Ply::show` and return a selection.
    pub fn handle_input(
        &mut self,
        ply: &Ply<()>,
        config: DropdownConfig,
        options: &[DropdownOption],
    ) -> Option<usize> {
        if ply.is_just_pressed(config.button_id) {
            if self.open {
                self.close();
            } else {
                self.open = true;
                self.filter.clear();
                self.scroll = 0;
                self.cursor = 0;
            }
            return None;
        }

        if !self.open {
            return None;
        }

        if config.searchable {
            while let Some(character) = get_char_pressed() {
                if character.is_ascii_alphanumeric() || character == ' ' || character == '-' {
                    self.filter.push(character);
                    self.scroll = 0;
                    self.cursor = 0;
                }
            }
            if is_key_pressed(KeyCode::Backspace) && !self.filter.is_empty() {
                self.filter.pop();
                self.scroll = 0;
                self.cursor = 0;
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            self.close();
            return None;
        }

        let filtered = self.filtered_indices(options);
        if filtered.is_empty() {
            self.scroll = 0;
            self.cursor = 0;
        } else {
            self.cursor = self.cursor.min(filtered.len() - 1);
            let wheel = mouse_wheel().1;
            if wheel < 0.0 || is_key_pressed(KeyCode::Down) {
                self.cursor = (self.cursor + 1).min(filtered.len() - 1);
            }
            if wheel > 0.0 || is_key_pressed(KeyCode::Up) {
                self.cursor = self.cursor.saturating_sub(1);
            }
            self.keep_cursor_visible(filtered.len(), config.visible_rows);

            let visible = self.visible_range(filtered.len(), config.visible_rows);
            for option_position in visible {
                let option = &options[filtered[option_position]];
                if ply.is_just_pressed((config.option_id, option.source_index as u32)) {
                    let selection = option.source_index;
                    self.close();
                    return Some(selection);
                }
            }

            if is_key_pressed(KeyCode::Enter) {
                let selection = options[filtered[self.cursor]].source_index;
                self.close();
                return Some(selection);
            }
        }

        if is_mouse_button_pressed(MouseButton::Left)
            && !ply.pointer_over(config.panel_id)
            && !ply.pointer_over(config.button_id)
        {
            self.close();
        }

        None
    }

    fn filtered_indices(&self, options: &[DropdownOption]) -> Vec<usize> {
        let filter = self.filter.to_lowercase();
        options
            .iter()
            .enumerate()
            .filter_map(|(index, option)| {
                (filter.is_empty() || option.search_text.to_lowercase().contains(&filter))
                    .then_some(index)
            })
            .collect()
    }

    /// The window of *positions into `filtered`* currently visible, honoring
    /// scroll. Callers index `filtered[position]` to get the option index, so
    /// this returns positions (0..filtered.len()), not the filtered values.
    fn visible_range(&self, filtered_len: usize, visible_rows: usize) -> std::ops::Range<usize> {
        let max_scroll = filtered_len.saturating_sub(visible_rows);
        let scroll = self.scroll.min(max_scroll);
        scroll..(scroll + visible_rows).min(filtered_len)
    }

    fn keep_cursor_visible(&mut self, option_count: usize, visible_rows: usize) {
        if option_count <= visible_rows {
            self.scroll = 0;
        } else if self.cursor < self.scroll {
            self.scroll = self.cursor;
        } else if self.cursor >= self.scroll + visible_rows {
            self.scroll = self.cursor + 1 - visible_rows;
        }
        self.scroll = self.scroll.min(option_count.saturating_sub(visible_rows));
    }
}

#[cfg(test)]
mod tests {
    use super::{DropdownOption, DropdownState};

    #[test]
    fn filtering_matches_label_search_text_case_insensitively() {
        let options = [
            DropdownOption {
                source_index: 1,
                label: "KJGX — Robins AFB".to_string(),
                search_text: "KJGX Robins AFB".to_string(),
            },
            DropdownOption {
                source_index: 2,
                label: "KTLX — Oklahoma City".to_string(),
                search_text: "KTLX Oklahoma City".to_string(),
            },
        ];
        let state = DropdownState {
            filter: "oklahoma".to_string(),
            ..DropdownState::default()
        };
        assert_eq!(state.filtered_indices(&options), vec![1]);
    }

    #[test]
    fn visible_range_maps_through_filtered_without_out_of_bounds() {
        // Regression: visible_range yields POSITIONS into `filtered`; callers
        // do options[filtered[position]]. With a filter active the option
        // indices differ from positions — this must stay in bounds and pick
        // the right options (the old code re-indexed filtered by its values).
        let opt = |i: usize, s: &str| DropdownOption {
            source_index: i,
            label: format!("opt{i}"),
            search_text: s.to_string(),
        };
        let options = [
            opt(0, "alpha"),
            opt(1, "bravo match"),
            opt(2, "charlie"),
            opt(3, "delta"),
            opt(4, "echo match"),
        ];
        let state = DropdownState {
            filter: "match".to_string(),
            ..DropdownState::default()
        };
        let filtered = state.filtered_indices(&options); // [1, 4]
        assert_eq!(filtered, vec![1, 4]);
        let visible = state.visible_range(filtered.len(), 10); // 0..2 (positions)
        let picked: Vec<usize> = visible
            .map(|position| options[filtered[position]].source_index)
            .collect();
        assert_eq!(picked, vec![1, 4]);
    }

    #[test]
    fn cursor_scrolling_keeps_selection_visible() {
        let mut state = DropdownState {
            cursor: 7,
            ..DropdownState::default()
        };
        state.keep_cursor_visible(20, 5);
        assert_eq!(state.scroll, 3);

        state.cursor = 2;
        state.keep_cursor_visible(20, 5);
        assert_eq!(state.scroll, 2);
    }
}
