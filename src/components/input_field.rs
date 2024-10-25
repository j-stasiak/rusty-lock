use ratatui::prelude::*;
use ratatui::widgets::{Block, Padding, Paragraph, Widget};
use secrecy::zeroize::Zeroize;
use symbols::border;

#[derive(PartialEq, Copy, Clone)]
pub enum InputFieldState {
    Active,
    Inactive,
    Disabled,
}

#[derive(Clone)]
pub struct InputField {
    value: String,
    pub label: &'static str,
    pub hide_value: bool,
    pub state: InputFieldState,
    pub character_limit: u8,
    pub cursor_position: Option<Position>,
    default_cursor_position: Position,
    cursor_index: usize,
}

impl Default for InputField {
    fn default() -> Self {
        let character_limit: u8 = 32;
        let field = InputField {
            label: "",
            hide_value: false,
            state: InputFieldState::Inactive,
            character_limit,
            value: String::with_capacity(character_limit.into()),
            cursor_position: None,
            cursor_index: 0,
            default_cursor_position: Default::default(),
        };

        field
    }
}

impl Widget for &mut InputField {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let label = format!(" {} ", self.label);
        let block = Block::bordered()
            .padding(Padding::new(2, 2, 1, 0))
            .border_set(border::ROUNDED)
            .title(label);

        if self.cursor_position.is_none() && self.state == InputFieldState::Active {
            self.cursor_position = Some(Position::new(area.x + 3, area.y + 2));
            self.default_cursor_position = Position::new(area.x + 3, area.y + 2);
        }

        let display_value = if self.hide_value {
            let chars: Vec<u8> = self.value.clone().chars().map(|_| b'*').collect();
            Text::from(String::from_utf8(chars).unwrap())
        } else {
            Text::from(self.value.clone())
        };

        Paragraph::new(display_value)
            .block(block)
            .alignment(Alignment::Left)
            .style(Style::default().bg(Color::DarkGray))
            .render(area, buf);
    }
}

impl InputField {
    pub fn clear_value(&mut self) {
        if self.hide_value {
            self.value.zeroize();
        }
        self.value.clear();
        self.cursor_index = 0;
        self.cursor_position = None;
    }

    pub fn get_value(&self) -> String {
        self.value.clone()
    }

    pub fn add_character(&mut self, new_char: char) {
        if self.value.len() < self.character_limit.into() {
            let index = self.byte_index();
            self.value.insert(index, new_char);
            self.move_cursor_right();
        }
    }

    pub fn remove_character(&mut self) {
        let is_not_cursor_leftmost = self.cursor_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.value.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.value.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.value = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position.is_some() && self.cursor_index < self.value.len() {
            let position = self.cursor_position.unwrap();
            self.cursor_position = Some(Position::new(
                (position.x + 1).min(self.default_cursor_position.x + self.character_limit as u16),
                position.y,
            ));
            let index = self.cursor_index.saturating_add(1);
            self.cursor_index = self.clamp_cursor(index);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position.is_some() {
            let position = self.cursor_position.unwrap();
            self.cursor_position = Some(Position::new(
                (position.x - 1).max(self.default_cursor_position.x),
                position.y,
            ));
            let index = self.cursor_index.saturating_sub(1);
            self.cursor_index = self.clamp_cursor(index);
        }
    }

    pub fn reset_cursor(&mut self) {
        if self.cursor_position.is_some() {
            let position = self.cursor_position.unwrap();
            self.cursor_position = Some(Position::new(self.default_cursor_position.x, position.y));
            self.cursor_index = 0;
        }
    }

    pub fn place_cursor_at_end(&mut self) {
        if let Some(position) = self.cursor_position {
            let end_position: usize =
                self.default_cursor_position.x as usize + self.value.chars().count();

            // Attempt to convert end_position to u16
            if let Ok(end_u16) = end_position.try_into() {
                self.cursor_position = Some(Position::new(end_u16, position.y));
                self.cursor_index = self.value.len();
            }
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.value.chars().count())
    }

    fn byte_index(&self) -> usize {
        self.value
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor_index)
            .unwrap_or(self.value.len())
    }
}
