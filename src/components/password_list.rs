use ratatui::{
    text::Line,
    widgets::{ListItem, ListState},
};

pub struct PasswordListItem {
    pub label: String,
    pub encrypted_value: String,
}

impl From<(String, String)> for PasswordListItem {
    fn from(value: (String, String)) -> Self {
        PasswordListItem {
            label: value.0.clone(),
            encrypted_value: value.1.clone(),
        }
    }
}

impl From<(&str, &str)> for PasswordListItem {
    fn from(value: (&str, &str)) -> Self {
        PasswordListItem {
            label: value.0.to_string(),
            encrypted_value: value.1.to_string(),
        }
    }
}

impl From<&(&str, &str)> for PasswordListItem {
    fn from(value: &(&str, &str)) -> Self {
        PasswordListItem {
            label: value.0.to_string(),
            encrypted_value: value.1.to_string(),
        }
    }
}

impl From<&PasswordListItem> for ListItem<'static> {
    fn from(value: &PasswordListItem) -> Self {
        ListItem::new(Line::from(value.label.to_string()))
    }
}

pub struct PasswordList {
    pub items: Vec<PasswordListItem>,
    pub state: ListState,
}

impl From<Vec<(&str, &str)>> for PasswordList {
    fn from(value: Vec<(&str, &str)>) -> Self {
        let items: Vec<PasswordListItem> = value.iter().map(PasswordListItem::from).collect();
        PasswordList {
            items,
            state: ListState::default(),
        }
    }
}
