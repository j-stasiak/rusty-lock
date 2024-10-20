use crossterm::event;
use ratatui::{DefaultTerminal, Frame};
use std::{cell::RefCell, collections::HashMap, fmt::Debug, io, rc::Rc};

use crate::{
    message_bus::{Message, MessageBus},
    screens::{dashboard::Dashboard, welcome_screen::WelcomeScreen},
};

#[derive(PartialEq, Eq, Debug, Hash)]
pub enum AppState {
    WelcomeScreen,
    Dashboard,
    AddNewPassword,
    Quit,
}
pub trait Screen {
    fn handle_terminal_events(&mut self, event: event::Event, state: &mut AppState);
    fn render(&mut self, frame: &mut Frame);
    fn handle_messages(&mut self, messages: Vec<Message>, state: &mut AppState);
}

pub struct App {
    state: AppState,
    screens_map: HashMap<AppState, Box<dyn Screen>>,
    message_bus: Rc<RefCell<MessageBus>>,
}

impl App {
    pub fn new() -> Self {
        let message_bus = Rc::new(RefCell::new(MessageBus::new()));

        let mut app = App {
            state: AppState::WelcomeScreen,
            screens_map: HashMap::new(),
            message_bus: Rc::clone(&message_bus),
        };

        app.screens_map.insert(
            AppState::WelcomeScreen,
            Box::new(WelcomeScreen::new(Rc::clone(&message_bus))),
        );

        app.screens_map.insert(
            AppState::Dashboard,
            Box::new(Dashboard::new(Rc::clone(&message_bus))),
        );

        app
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while self.state != AppState::Quit {
            let messages = self.message_bus.borrow_mut().poll_messages();
            let screen = self
                .screens_map
                .get_mut(&self.state)
                .expect("No screen for state")
                .as_mut();

            screen.handle_messages(messages, &mut self.state);
            terminal.draw(|frame| screen.render(frame))?;

            let ev = event::read()?;
            screen.handle_terminal_events(ev, &mut self.state);
        }
        Ok(())
    }
}
