use std::{
    cell::RefCell,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
    rc::Rc,
};

use crate::{
    app::{AppState, Screen},
    components::{
        input_field::{InputField, InputFieldState},
        password_list::{PasswordList, PasswordListItem},
    },
    crypto_utils::{self},
    message_bus::{Message, MessageBus},
};
use base64::{prelude::BASE64_STANDARD, Engine};
use crossterm::event::{Event, KeyCode, KeyEventKind};
use rand::distributions::{Alphanumeric, DistString};
use ratatui::{
    prelude::*,
    widgets::{
        block::Title, Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph,
    },
};
use secrecy::{ExposeSecret, SecretBox};
use symbols::border;
use windows::Win32::{
    Foundation::{GlobalFree, HANDLE},
    System::{
        DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData},
        Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
    },
};

#[cfg(not(debug_assertions))]
const PASSWORD_PATH: &'static str = env!("LOCALAPPDATA");

#[cfg(debug_assertions)]
const PASSWORD_PATH: &'static str = ".";

#[derive(Copy, Clone)]
enum DisplayInputs {
    GeneratePassword,
    ImportPassword,
}

#[derive(Copy, Clone)]
enum CurrentlyActiveInput {
    Service,
    Password,
}

pub struct Dashboard {
    service_input: InputField,
    password_input: InputField,
    display_inputs: Option<DisplayInputs>,
    active_input: Option<CurrentlyActiveInput>,

    password_list: PasswordList,
    user_password: SecretBox<Vec<u8>>,
}

impl Dashboard {
    pub fn new(_message_bus: Rc<RefCell<MessageBus>>) -> Self {
        let mut service_input = InputField::default();
        service_input.label = "Service";
        service_input.state = InputFieldState::Active;

        let mut password_input = InputField::default();
        password_input.label = "Password";
        password_input.hide_value = true;

        let dashboard = Dashboard {
            user_password: SecretBox::new(Box::new(vec![])),
            service_input,
            password_input,
            display_inputs: None,
            active_input: None,
            password_list: PasswordList {
                items: vec![],
                state: ListState::default(),
            },
        };

        dashboard
    }

    fn decode_password(&self, encoded_password: &str) -> String {
        let decoded = crypto_utils::decrypt(
            BASE64_STANDARD.decode(encoded_password).unwrap().as_slice(),
            self.user_password.expose_secret(),
        );

        String::from_utf8(decoded).unwrap()
    }

    fn select_next(&mut self) {
        self.password_list.state.select_next();
    }
    fn select_previous(&mut self) {
        self.password_list.state.select_previous();
    }

    fn focus_service(&mut self) {
        self.active_input = Some(CurrentlyActiveInput::Service);
        self.service_input.state = InputFieldState::Active;
        self.password_input.state = InputFieldState::Inactive;
    }

    fn focus_password(&mut self) {
        self.active_input = Some(CurrentlyActiveInput::Password);
        self.service_input.state = InputFieldState::Inactive;
        self.password_input.state = InputFieldState::Active;
    }

    fn handle_input_field_event(&mut self, key_code: KeyCode) {
        let active_input = match self.active_input {
            Some(input) => match input {
                CurrentlyActiveInput::Service => &mut self.service_input,
                CurrentlyActiveInput::Password => &mut self.password_input,
            },
            None => return,
        };

        match key_code {
            KeyCode::Esc => {
                self.service_input.clear_value();
                self.password_input.clear_value();

                self.display_inputs = None;
                self.active_input = None;
            }
            KeyCode::Backspace => active_input.remove_character(),
            KeyCode::Left => active_input.move_cursor_left(),
            KeyCode::Right => active_input.move_cursor_right(),
            KeyCode::Home => active_input.reset_cursor(),
            KeyCode::End => active_input.place_cursor_at_end(),
            KeyCode::Char(c) => active_input.add_character(c),
            _ => {}
        }
    }

    fn copy_to_clipboard(&self, text: &str) {
        unsafe {
            // Open the clipboard
            if OpenClipboard(None).is_ok() {
                // Empty the clipboard
                EmptyClipboard();

                // Allocate global memory
                let h_glob = GlobalAlloc(GMEM_MOVEABLE, text.len() + 1).unwrap();
                let p_glob = GlobalLock(h_glob);
                if !p_glob.is_null() {
                    // Copy the text to the allocated memory
                    std::ptr::copy_nonoverlapping(text.as_ptr(), p_glob as *mut u8, text.len());
                    // Add null terminator
                    *((p_glob as *mut u8).add(text.len())) = 0;

                    GlobalUnlock(h_glob);

                    // Set the clipboard data
                    SetClipboardData(1, HANDLE(h_glob.0));
                }
                GlobalFree(h_glob);
                CloseClipboard();
            }
        }
    }

    fn load_passwords_from_file(&mut self, login: String) {
        #[cfg(not(debug_assertions))]
        let directory_base_path = Path::new(PASSWORD_PATH).join("rusty-lock");

        #[cfg(debug_assertions)]
        let directory_base_path = Path::new(PASSWORD_PATH);

        let directory_path = directory_base_path.join("pwds");
        let binding = directory_path.clone();
        let directory_path_display = binding.display();
        let path = directory_path.join(login);
        let display = path.display();

        if !directory_path.exists() {
            match fs::create_dir_all(directory_path) {
                Err(why) => panic!("couldn't create {}: {}", directory_path_display, why),
                Ok(_) => {}
            }
        }

        if !path.exists() {
            match File::create(&path) {
                Ok(_) => {}
                Err(why) => panic!("couldn't create {}: {}", display, why),
            }
        }

        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why),
            Ok(file) => file,
        };

        // Read the file contents into a string, returns `io::Result<usize>`
        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(why) => panic!("couldn't read {}: {}", display, why),
            Ok(_) => {}
        }

        let encrypted_passwords: Vec<(&str, &str)> = s
            .lines()
            .map(|line| {
                let results: Vec<&str> = line.split('=').collect();
                (results[0], results[1])
            })
            .collect();

        self.password_list = PasswordList::from(encrypted_passwords);
        let item = PasswordListItem {
            encrypted_value: BASE64_STANDARD.encode(crypto_utils::encrypt(
                "test",
                self.user_password.expose_secret(),
            )),
            label: String::from("facebook"),
        };

        self.password_list.items.push(item);

        if self.password_list.items.len() > 0 {
            self.password_list.state = ListState::default().with_selected(Some(1));
        }
    }

    fn clear_inputs(&mut self) {
        self.service_input.clear_value();
        self.password_input.clear_value();
    }

    fn submit_generate_password(&mut self) {
        let new_password = Alphanumeric.sample_string(&mut rand::thread_rng(), 20);
        let encoded = BASE64_STANDARD.encode(crypto_utils::encrypt(
            new_password.as_str(),
            self.user_password.expose_secret(),
        ));

        let service_name = self.service_input.get_value();
        self.password_list.items.push(PasswordListItem::from((
            service_name.clone(),
            encoded.clone(),
        )));

        #[cfg(not(debug_assertions))]
        let directory_base_path = Path::new(PASSWORD_PATH).join("rusty-lock");

        #[cfg(debug_assertions)]
        let directory_base_path = Path::new(PASSWORD_PATH);

        let directory_path = directory_base_path.join("pwds").join("asd");

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(directory_path)
            .unwrap();

        if let Err(e) = writeln!(file, "{service_name}={encoded}") {
            eprintln!("Couldn't write to file: {}", e);
        }

        self.clear_inputs();

        self.display_inputs = None;
        self.active_input = None;
    }

    fn submit_import_password(&mut self) {
        if let Some(active) = self.active_input {
            match active {
                CurrentlyActiveInput::Service => self.focus_password(),
                CurrentlyActiveInput::Password => {
                    let encoded = BASE64_STANDARD.encode(crypto_utils::encrypt(
                        self.password_input.get_value().as_str(),
                        self.user_password.expose_secret(),
                    ));

                    let service_name = self.service_input.get_value();
                    self.password_list.items.push(PasswordListItem::from((
                        service_name.clone(),
                        encoded.clone(),
                    )));

                    #[cfg(not(debug_assertions))]
                    let directory_base_path = Path::new(PASSWORD_PATH).join("rusty-lock");

                    #[cfg(debug_assertions)]
                    let directory_base_path = Path::new(PASSWORD_PATH);

                    let directory_path = directory_base_path.join("pwds").join("asd");

                    let mut file = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .open(directory_path)
                        .unwrap();

                    if let Err(e) = writeln!(file, "{service_name}={encoded}") {
                        eprintln!("Couldn't write to file: {}", e);
                    }

                    self.clear_inputs();
                    self.display_inputs = None;
                    self.active_input = None;
                }
            }
        }
    }
}

impl Screen for Dashboard {
    fn handle_terminal_events(
        &mut self,
        event: crossterm::event::Event,
        state: &mut crate::app::AppState,
    ) {
        if let Some(display_inputs) = self.display_inputs {
            match event {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    self.handle_input_field_event(key.code);

                    if key.code == KeyCode::Enter {
                        match display_inputs {
                            DisplayInputs::GeneratePassword => self.submit_generate_password(),
                            DisplayInputs::ImportPassword => self.submit_import_password(),
                        }
                    }
                }
                _ => {}
            }
        } else {
            match event {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Down => self.select_next(),
                    KeyCode::Up => self.select_previous(),
                    KeyCode::Char('c') => {
                        if let Some(password_index) = self.password_list.state.selected() {
                            let encoded_password = self
                                .password_list
                                .items
                                .get(password_index)
                                .unwrap()
                                .encrypted_value
                                .as_str();

                            let decoded_password = self.decode_password(encoded_password);

                            self.copy_to_clipboard(decoded_password.as_str());
                        }
                    }
                    KeyCode::Char('g') => {
                        self.display_inputs = Some(DisplayInputs::GeneratePassword);
                        self.focus_service();
                    }
                    KeyCode::Char('n') => {
                        self.display_inputs = Some(DisplayInputs::ImportPassword);
                        self.focus_service();
                    }
                    KeyCode::Char('q') => {
                        *state = AppState::Quit;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let buf = frame.buffer_mut();

        let title = Title::from(" List of passwords ".bold());
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .border_set(border::THICK);

        let text = Text::from("View and add or copy all your passwords from this screen!");

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

        let block = Block::bordered().border_set(border::THICK);

        let items: Vec<ListItem> = self
            .password_list
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let color = if i % 2 == 0 {
                    Color::LightBlue
                } else {
                    Color::Blue
                };

                ListItem::from(item).fg(color)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, layout_parts[1], buf, &mut self.password_list.state);

        Paragraph::new(
            Line::from(vec![
                "<Down> ".bold(),
                "Select below / ".into(),
                "<Up> ".bold(),
                "Select above / ".into(),
                "<C> ".bold(),
                "Copy selected / ".into(),
                "<N> ".bold(),
                "Add new / ".into(),
                "<G> ".bold(),
                "Generate new / ".into(),
                "<Q> ".bold(),
                "Quit".into(),
            ])
            .style(Style::default().fg(Color::Green)),
        )
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center)
        .render(layout_parts[2], buf);

        if let Some(display) = &self.display_inputs {
            let input_area = Layout::default()
                .direction(Direction::Vertical)
                .horizontal_margin(20)
                .flex(layout::Flex::Center)
                .constraints([Constraint::Length(5), Constraint::Length(5)])
                .split(layout_parts[1]);

            match display {
                DisplayInputs::GeneratePassword => {
                    self.service_input.render(input_area[0], buf);
                }
                DisplayInputs::ImportPassword => {
                    self.service_input.render(input_area[0], buf);
                    self.password_input.render(input_area[1], buf);
                }
            }
        }

        if self.display_inputs.is_some() {
            for input in [&self.service_input, &self.password_input].iter() {
                if let Some(position) = input.cursor_position {
                    frame.set_cursor_position(position);
                }
            }
        }
    }

    fn handle_messages(&mut self, messages: Vec<Message>, _state: &mut crate::app::AppState) {
        for message in messages {
            match message {
                Message::LoginCredentials(login, password) => {
                    let hash = crypto_utils::hash_password(password);

                    self.user_password = SecretBox::new(Box::new(hash.to_vec()));

                    self.load_passwords_from_file(login);
                }
            }
        }
    }
}
