#[derive(Clone)]
pub enum Message {
    LoginCredentials(String, String),
}

pub struct MessageBus {
    messages: Vec<Message>,
}

impl MessageBus {
    pub fn new() -> Self {
        MessageBus { messages: vec![] }
    }

    pub fn submit_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn poll_messages(&mut self) -> Vec<Message> {
        self.messages.drain(..).collect()
    }
}
