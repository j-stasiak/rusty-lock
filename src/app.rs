use crossterm::event;
use ratatui::{buffer::Buffer, layout::Rect, DefaultTerminal, Frame};
use std::{collections::HashMap, fmt::Debug, io};

use crate::screens::{dashboard::Dashboard, welcome_screen::WelcomeScreen};

#[derive(PartialEq, Eq, Debug, Hash)]
pub enum AppState {
    WelcomeScreen,
    LoginScreen,
    RegistrationScreen,
    Dashboard,
    AddNewPassword,
    Search,
    Quit,
}
pub trait Screen {
    fn handle_events(&mut self, event: event::Event, state: &mut AppState);
    fn render(&mut self, frame: &mut Frame);
}

pub struct App {
    state: AppState,
    screens_map: HashMap<AppState, Box<dyn Screen>>,
}

impl App {
    pub fn new() -> Self {
        let mut app = App {
            state: AppState::WelcomeScreen,
            screens_map: HashMap::new(),
        };

        app.screens_map
            .insert(AppState::WelcomeScreen, Box::new(WelcomeScreen::new()));

        app.screens_map
            .insert(AppState::Dashboard, Box::new(Dashboard::new()));

        app
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while self.state != AppState::Quit {
            let screen = self
                .screens_map
                .get_mut(&self.state)
                .expect("No screen for state")
                .as_mut();

            terminal.draw(|frame| screen.render(frame))?;

            let ev = event::read()?;
            screen.handle_events(ev, &mut self.state);
        }
        Ok(())
    }
}
