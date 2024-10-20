use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{block::Title, Block, Paragraph},
};
use symbols::border;

use crate::{
    app::{AppState, Screen},
    components::input_field::{InputField, InputFieldState},
};

pub struct WelcomeScreen {
    login_input: InputField,
    password_input: InputField,
}

impl WelcomeScreen {
    pub fn new() -> Self {
        let mut login_input = InputField::default();
        login_input.state = InputFieldState::Active;
        login_input.label = "Login";

        let mut password_input = InputField::default();
        password_input.label = "Password";
        password_input.hide_value = true;

        let screen = WelcomeScreen {
            login_input,
            password_input,
        };

        screen
    }
}

impl Screen for WelcomeScreen {
    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let buf = frame.buffer_mut();

        // Implement rendering logic
        let title = Title::from("Welcome to the App".bold());
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .border_set(border::THICK);

        let text = Text::from("Log in or create an account to continue...");

        let layout_parts = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(area);

        Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center)
            .render(layout_parts[0], buf);

        Block::bordered()
            .border_set(border::THICK)
            .render(layout_parts[1], buf);

        let input_area = Layout::default()
            .direction(Direction::Vertical)
            .flex(layout::Flex::Center)
            .constraints([Constraint::Length(5), Constraint::Length(5)])
            .split(layout_parts[1]);

        let login_area = Rect::new(
            input_area[0].x + input_area[0].width / 3,
            input_area[0].y,
            input_area[0].width / 3,
            input_area[0].height,
        );
        self.login_input.render(login_area, buf);

        let password_area = Rect::new(
            input_area[1].x + input_area[1].width / 3,
            input_area[1].y,
            input_area[1].width / 3,
            input_area[1].height,
        );
        self.password_input.render(password_area, buf);

        for input in [&self.login_input, &self.password_input].iter() {
            if let Some(position) = input.cursor_position {
                frame.set_cursor_position(position);
            }
        }
    }

    fn handle_events(&mut self, event: event::Event, state: &mut AppState) {
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                if key_event.code == KeyCode::Delete {
                    *state = AppState::Dashboard;
                }

                if key_event.code == KeyCode::Char('q') {
                    *state = AppState::Quit;
                }
            }
            _ => {}
        }
    }
}
