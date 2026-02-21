#![allow(dead_code, unused_imports)]

mod core;
mod editor;
mod export;
mod gui;
mod spelling;
mod templates;
mod thesaurus;

use gui::ScrineverApp;

fn main() -> iced::Result {
    env_logger::init();

    iced::application(ScrineverApp::title, ScrineverApp::update, ScrineverApp::view)
        .window_size((1280.0, 800.0))
        .subscription(ScrineverApp::subscription)
        .theme(ScrineverApp::theme)
        .run_with(ScrineverApp::new)
}
