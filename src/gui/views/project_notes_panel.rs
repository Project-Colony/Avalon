use iced::widget::{column, container, row, text, text_input, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::Message;
use crate::gui::theme::{self, Theme};

/// Render the project notes / scratch pad panel (bottom panel)
pub fn view(notes: &str) -> Element<'static, Message> {
    let word_count = notes.split_whitespace().count();
    let char_count = notes.len();
    let line_count = if notes.is_empty() { 0 } else { notes.lines().count() };
    let sentence_count = notes.chars().filter(|c| *c == '.' || *c == '!' || *c == '?').count();
    let paragraph_count = if notes.is_empty() {
        0
    } else {
        notes.split("\n\n").filter(|p| !p.trim().is_empty()).count()
    };

    let header = row![
        text("PROJECT NOTES").size(11).color(Theme::TEXT_SECONDARY),
        Space::new().width(Length::Fill),
        text(format!(
            "{} words | {} chars | {} lines | {} sentences | {} para",
            word_count, char_count, line_count, sentence_count, paragraph_count
        ))
        .size(10)
        .color(Theme::TEXT_MUTED),
    ];

    let notes_input = text_input("Write project-level notes, ideas, reminders...", notes)
        .on_input(Message::ProjectNotesChanged)
        .size(13)
        .padding(8)
        .width(Length::Fill);

    // Reading time
    let reading_min = word_count as f64 / crate::core::READING_WPM;
    let reading_display = if reading_min < 1.0 {
        "<1m read".to_string()
    } else {
        format!("~{:.0}m read", reading_min)
    };

    // Contextual hints based on content
    let hint_text = if notes.is_empty() {
        "Ideas: character bios, world-building notes, research links, plot outlines, revision notes"
    } else if word_count < 10 {
        "Keep adding notes — they're saved with the project automatically"
    } else if word_count > 500 {
        "These notes are getting long — consider moving sections to Research folder"
    } else {
        "Tip: Use this pad for quick notes. For longer notes, use Research folder in the binder."
    };

    let hint = row![
        text(hint_text).size(9).color(Theme::TEXT_MUTED),
        Space::new().width(Length::Fill),
        text(reading_display).size(9).color(Theme::TEXT_MUTED),
        Space::new().width(8),
        text("\u{f0c7} Auto-saved with project")
            .size(9)
            .color(Theme::TEXT_MUTED),
    ];

    let content = column![
        header,
        Space::new().height(4),
        notes_input,
        Space::new().height(2),
        hint,
    ]
    .padding(Padding::from([8, 12]));

    container(content).style(theme::panel_style).width(Length::Fill).into()
}
