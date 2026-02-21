use iced::widget::{column, container, row, text, text_input, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the project notes / scratch pad panel (bottom panel)
pub fn view(notes: &str) -> Element<'static, Message> {
    let header = row![
        text("PROJECT NOTES / SCRATCH PAD").size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        text("Persistent notes for the whole project").size(10).color(Theme::TEXT_MUTED),
    ];

    let notes_input = text_input("Write project-level notes here...", notes)
        .on_input(|val| Message::ProjectNotesChanged(val))
        .size(13)
        .padding(8)
        .width(Length::Fill);

    let content = column![
        header,
        Space::with_height(4),
        notes_input,
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
