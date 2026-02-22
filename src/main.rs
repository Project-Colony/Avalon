#![allow(dead_code, unused_imports)]

mod core;
mod editor;
mod export;
mod gui;
mod spelling;
mod templates;
mod thesaurus;

use gui::ScrineverApp;

/// JetBrains Mono Nerd Font — Regular weight (embedded)
const JETBRAINS_MONO_REGULAR: &[u8] =
    include_bytes!("../assets/fonts/JetBrainsMonoNerdFont-Regular.ttf");

/// JetBrains Mono Nerd Font — Bold weight (embedded)
const JETBRAINS_MONO_BOLD: &[u8] =
    include_bytes!("../assets/fonts/JetBrainsMonoNerdFont-Bold.ttf");

fn main() -> iced::Result {
    env_logger::init();

    iced::application(ScrineverApp::title, ScrineverApp::update, ScrineverApp::view)
        .window_size((1280.0, 800.0))
        .subscription(ScrineverApp::subscription)
        .theme(ScrineverApp::theme)
        .font(JETBRAINS_MONO_REGULAR)
        .font(JETBRAINS_MONO_BOLD)
        .default_font(iced::Font {
            family: iced::font::Family::Name("JetBrainsMono Nerd Font"),
            ..iced::Font::DEFAULT
        })
        .run_with(ScrineverApp::new)
}
