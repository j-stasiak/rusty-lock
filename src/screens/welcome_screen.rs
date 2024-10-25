use std::{cell::RefCell, rc::Rc};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{block::Title, Block, Paragraph},
};
use symbols::border;

use crate::{
    app::{AppState, Screen},
    components::input_field::{InputField, InputFieldState},
    message_bus::{Message, MessageBus},
};

pub struct WelcomeScreen {
    login_input: InputField,
    password_input: InputField,
    active_field: ActiveField,
    message_bus: Rc<RefCell<MessageBus>>,
}

enum ActiveField {
    Login,
    Password,
}

impl WelcomeScreen {
    pub fn new(message_bus: Rc<RefCell<MessageBus>>) -> Self {
        let mut login_input = InputField::default();
        login_input.label = "Login";
        login_input.state = InputFieldState::Active;

        let mut password_input = InputField::default();
        password_input.label = "Password";
        password_input.hide_value = true;

        WelcomeScreen {
            login_input,
            password_input,
            active_field: ActiveField::Login,
            message_bus,
        }
    }

    fn handle_submit(&mut self, state: &mut AppState) {
        match self.active_field {
            ActiveField::Login => {
                self.focus_password();
            }
            ActiveField::Password => {
                self.message_bus
                    .borrow_mut()
                    .submit_message(Message::LoginCredentials(
                        self.login_input.get_value(),
                        self.password_input.get_value(),
                    ));

                self.login_input.clear_value();
                self.password_input.clear_value();

                *state = AppState::Dashboard
            }
        }
    }

    fn focus_login(&mut self) {
        self.active_field = ActiveField::Login;
        self.login_input.state = InputFieldState::Active;
        self.password_input.state = InputFieldState::Inactive;
    }

    fn focus_password(&mut self) {
        self.active_field = ActiveField::Password;
        self.login_input.state = InputFieldState::Inactive;
        self.password_input.state = InputFieldState::Active;
    }

    fn handle_input_field_event(&mut self, key_code: KeyCode) {
        let active_input = match self.active_field {
            ActiveField::Login => &mut self.login_input,
            ActiveField::Password => &mut self.password_input,
        };

        match key_code {
            KeyCode::Backspace => active_input.remove_character(),
            KeyCode::Left => active_input.move_cursor_left(),
            KeyCode::Right => active_input.move_cursor_right(),
            KeyCode::Home => active_input.reset_cursor(),
            KeyCode::End => active_input.place_cursor_at_end(),

            KeyCode::Char(c) => active_input.add_character(c),
            _ => {}
        }
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

    fn handle_terminal_events(&mut self, event: event::Event, state: &mut AppState) {
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_input_field_event(key_event.code);

                if key_event.code == KeyCode::PageUp {
                    self.focus_login();
                }

                if key_event.code == KeyCode::PageDown {
                    self.focus_password();
                }

                if key_event.code == KeyCode::Enter {
                    self.handle_submit(state);
                }

                if key_event.code == KeyCode::Char('q') {
                    *state = AppState::Quit;
                }
            }
            _ => {}
        }
    }

    fn handle_messages(&mut self, message: Vec<Message>, state: &mut AppState) {}
}
