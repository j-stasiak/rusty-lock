use ratatui::prelude::*;
use ratatui::widgets::{Block, Padding, Paragraph, Widget};
use symbols::border;

#[derive(PartialEq, Copy, Clone)]
pub enum InputFieldState {
    Active,
    Inactive,
    Disabled,
}

pub struct InputField {
    value: String,
    pub label: &'static str,
    pub hide_value: bool,
    pub state: InputFieldState,
    pub character_limit: u8,
    pub cursor_position: Option<Position>,
}

impl Default for InputField {
    fn default() -> Self {
        let character_limit: u8 = 64;
        let field = InputField {
            label: "",
            hide_value: false,
            state: InputFieldState::Inactive,
            character_limit,
            value: String::with_capacity(character_limit.into()),
            cursor_position: None,
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
            .title(label)
            .style(Style::default().bg(Color::DarkGray));

        if self.cursor_position.is_none() && self.state == InputFieldState::Active {
            self.cursor_position = Some(Position::new(area.x + 2, area.y + 2));
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
            .render(area, buf);
    }
}

impl InputField {
    pub fn add_character(&mut self, c: char) {
        if self.character_limit as usize <= self.value.len() {
            self.value.insert(self.value.len(), c);
            if self.cursor_position.is_some() {
                let position = self.cursor_position.unwrap();
                self.cursor_position = Some(Position::new(position.x + 1, position.y))
            }
        }
    }

    pub fn remove_character(&mut self, index: usize) {
        self.value.remove(index);

        if self.cursor_position.is_some() {
            let position = self.cursor_position.unwrap();
            self.cursor_position = Some(Position::new(position.x - 1, position.y))
        }
    }
}
