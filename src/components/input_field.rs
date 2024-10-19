use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph, Widget};
use symbols::border;

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
}

impl Default for InputField {
    fn default() -> Self {
        Self {
            label: "",
            value: String::from("default"),
            hide_value: false,
            state: InputFieldState::Inactive,
            character_limit: 64,
        }
    }
}

impl Widget for &InputField {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let label = format!(" {} ", self.label);
        let block = Block::bordered()
            .border_set(border::ROUNDED)
            .title(label)
            .style(Style::default().bg(Color::DarkGray));

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
