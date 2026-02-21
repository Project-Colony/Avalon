use iced::widget::{column, container, row, text, text_input, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the project notes / scratch pad panel (bottom panel)
pub fn view(notes: &str) -> Element<'static, Message> {
    let word_count = notes.split_whitespace().count();
    let char_count = notes.len();
    let line_count = if notes.is_empty() { 0 } else { notes.lines().count() };

    let header = row![
        text("PROJECT NOTES").size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        text(format!("{} words | {} chars | {} lines", word_count, char_count, line_count))
            .size(10)
            .color(Theme::TEXT_MUTED),
    ];

    let notes_input = text_input("Write project-level notes, ideas, reminders...", notes)
        .on_input(|val| Message::ProjectNotesChanged(val))
        .size(13)
        .padding(8)
        .width(Length::Fill);

    let hint = row![
        text("Tip: Use this scratch pad for project-wide notes, character ideas, research links, and reminders.")
            .size(9)
            .color(Theme::TEXT_MUTED),
        Space::with_width(Length::Fill),
        text("Saved with project")
            .size(9)
            .color(Theme::TEXT_MUTED),
    ];

    let content = column![
        header,
        Space::with_height(4),
        notes_input,
        Space::with_height(2),
        hint,
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
