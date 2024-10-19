use crate::app::Screen;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{
        block::Title, Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph,
    },
};
use symbols::border;
use windows::Win32::{
    Foundation::{GlobalFree, HANDLE},
    System::{
        DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData},
        Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
    },
};

struct PasswordListItem {
    label: &'static str,
    encrypted_value: String,
}

impl From<&PasswordListItem> for ListItem<'static> {
    fn from(value: &PasswordListItem) -> Self {
        ListItem::new(Line::from(value.label.to_string()))
    }
}

struct PasswordList {
    items: Vec<PasswordListItem>,
    state: ListState,
}

pub struct Dashboard {
    password_list: PasswordList,
}

impl Dashboard {
    pub fn new() -> Self {
        let dashboard = Dashboard {
            password_list: PasswordList {
                items: vec![
                    PasswordListItem {
                        label: "facebook",
                        encrypted_value: String::from("facebook-password"),
                    },
                    PasswordListItem {
                        label: "linkedin",
                        encrypted_value: String::from("linkedin-password"),
                    },
                ],
                state: ListState::default(),
            },
        };

        dashboard
    }
}

impl Dashboard {
    fn select_next(&mut self) {
        self.password_list.state.select_next();
    }
    fn select_previous(&mut self) {
        self.password_list.state.select_previous();
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
                    if SetClipboardData(1, HANDLE(h_glob.0)).is_ok() {
                        CloseClipboard();
                        return;
                    }
                }
                GlobalFree(h_glob);
                CloseClipboard();
            }
        }
    }
}

impl Screen for Dashboard {
    fn handle_events(&mut self, event: crossterm::event::Event, _state: &mut crate::app::AppState) {
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

                        self.copy_to_clipboard(encoded_password);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn render(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
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
                "Select above /".into(),
                "<C> ".bold(),
                "Copy selected /".into(),
                "<N> ".bold(),
                "Add new".into(),
            ])
            .style(Style::default().fg(Color::Green)),
        )
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center)
        .render(layout_parts[2], buf);
    }
}
