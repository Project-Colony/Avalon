// Release builds on Windows are windowed programs. Without this, starting
// Avalon from Explorer or Colony also opens a console window that stays up for
// as long as the editor runs. Debug builds keep the console, so `cargo run`
// still shows the log.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod core;
mod editor;
mod export;
mod gui;
mod spelling;
mod templates;
mod thesaurus;

use gui::ScrineverApp;

/// JetBrains Mono Nerd Font Regular weight (embedded)
const JETBRAINS_MONO_REGULAR: &[u8] = include_bytes!("../assets/fonts/JetBrainsMonoNerdFont-Regular.ttf");

/// JetBrains Mono Nerd Font Bold weight (embedded)
const JETBRAINS_MONO_BOLD: &[u8] = include_bytes!("../assets/fonts/JetBrainsMonoNerdFont-Bold.ttf");

fn main() -> iced::Result {
    // Answered before anything opens a window: the release workflow
    // smoke-tests every built artifact with `--version` on runners that have
    // no display.
    if std::env::args_os().skip(1).any(|arg| arg == "--version" || arg == "-V") {
        #[cfg(windows)]
        attach_parent_console();
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

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

/// A windowed release build starts with no console, so `--version` would print
/// nowhere even when a terminal started it. This borrows that terminal's
/// console. Windows keeps the standard handles a parent passed explicitly, so
/// output redirected to a file or a pipe, as in the release smoke test, still
/// lands there. The call fails harmlessly when there is no parent console.
///
/// Only for `--version`: a process attached to a console ends with it, so a
/// normal launch from a terminal must not tie the editor to that window.
#[cfg(windows)]
fn attach_parent_console() {
    #[link(name = "kernel32")]
    extern "system" {
        fn AttachConsole(process_id: u32) -> i32;
    }
    const ATTACH_PARENT_PROCESS: u32 = u32::MAX;

    // SAFETY: AttachConsole takes a process ID and touches nothing in this
    // process's memory. Its only failure, no console to attach to, is ignored
    // on purpose.
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}
