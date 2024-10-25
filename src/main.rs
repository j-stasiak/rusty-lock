pub mod app;
pub mod components;
pub mod crypto_utils;
pub mod message_bus;
pub mod screens;

use app::App;
use std::io;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new();
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}
