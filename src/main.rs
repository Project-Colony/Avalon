mod core;
mod editor;
mod export;
mod gui;
mod spelling;
mod templates;
mod thesaurus;

use gui::ScrineverApp;

/// JetBrains Mono Nerd Font — Regular weight (embedded)
const JETBRAINS_MONO_REGULAR: &[u8] = include_bytes!("../assets/fonts/JetBrainsMonoNerdFont-Regular.ttf");

/// JetBrains Mono Nerd Font — Bold weight (embedded)
const JETBRAINS_MONO_BOLD: &[u8] = include_bytes!("../assets/fonts/JetBrainsMonoNerdFont-Bold.ttf");

fn main() -> iced::Result {
    env_logger::init();

    iced::daemon(ScrineverApp::new, ScrineverApp::update, ScrineverApp::view)
        .title(ScrineverApp::title)
        .subscription(ScrineverApp::subscription)
        .theme(ScrineverApp::theme)
        .font(JETBRAINS_MONO_REGULAR)
        .font(JETBRAINS_MONO_BOLD)
        .default_font(iced::Font {
            family: iced::font::Family::Name("JetBrainsMono Nerd Font"),
            ..iced::Font::DEFAULT
        })
        .run()
}
