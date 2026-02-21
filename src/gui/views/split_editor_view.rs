use iced::widget::{button, column, container, row, text, text_editor, Space};
use iced::{Element, Length, Padding};

use crate::editor::EditorState;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render a split editor view showing two documents side by side
pub fn view<'a>(
    primary_editor: &'a EditorState,
    primary_title: &str,
    secondary_content: &str,
    secondary_title: &str,
) -> Element<'a, Message> {
    // Primary editor (left) - editable
    let primary_words = primary_editor.document.word_count();
    let primary_chars = primary_editor.document.char_count();
    let primary_pages = primary_words as f64 / 250.0;

    let primary_header = container(
        row![
            text("\u{270E}").size(12).color(Theme::TEXT_ACCENT),
            Space::with_width(4),
            text(primary_title.to_string())
                .size(13)
                .color(Theme::TEXT_ACCENT),
            Space::with_width(Length::Fill),
            text("Editing").size(10).color(Theme::SUCCESS),
        ]
        .padding(Padding::from([4, 12]))
    )
    .width(Length::Fill);

    let primary = text_editor(&primary_editor.content)
        .on_action(|action| Message::EditorAction(action))
        .padding(Padding::from([12, 16]))
        .height(Length::Fill);

    let dirty_marker = if primary_editor.dirty { " \u{2022}" } else { "" };
    let primary_footer = container(
        row![
            text(format!("{} words | {} chars | {:.1} pg{}", primary_words, primary_chars, primary_pages, dirty_marker))
                .size(10)
                .color(Theme::TEXT_MUTED),
            Space::with_width(Length::Fill),
            text("Ctrl+S: save").size(9).color(Theme::TEXT_MUTED),
        ]
    )
    .padding(Padding::from([4, 12]));

    let primary_panel = container(
        column![primary_header, primary, primary_footer]
    )
    .width(Length::FillPortion(1))
    .height(Length::Fill);

    // Secondary (right) - read-only reference
    let secondary_words = secondary_content.split_whitespace().count();
    let secondary_chars = secondary_content.len();
    let secondary_paragraphs = if secondary_content.is_empty() {
        0
    } else {
        secondary_content.split("\n\n").filter(|p| !p.trim().is_empty()).count()
    };

    let secondary_header = container(
        row![
            text("\u{1F4D6}").size(12),
            Space::with_width(4),
            text(secondary_title.to_string())
                .size(13)
                .color(Theme::TEXT_SECONDARY),
            Space::with_width(Length::Fill),
            text("Read-only").size(10).color(Theme::TEXT_MUTED),
            Space::with_width(8),
            button(
                text("\u{2715} Close").size(10).color(Theme::TEXT_MUTED),
            )
            .on_press(Message::CloseSplitEditor)
            .padding(Padding::from([2, 6])),
        ]
        .padding(Padding::from([4, 12]))
    )
    .width(Length::Fill);

    let secondary_text = container(
        iced::widget::scrollable(
            container(
                text(secondary_content.to_string())
                    .size(14)
                    .color(Theme::TEXT_PRIMARY),
            )
            .padding(Padding::from([12, 16]))
        )
        .height(Length::Fill)
    )
    .width(Length::Fill)
    .height(Length::Fill);

    let secondary_pages = secondary_words as f64 / 250.0;
    let reading_min = secondary_words as f64 / 250.0;
    let reading_display = if reading_min < 1.0 {
        "<1m".to_string()
    } else {
        format!("{:.0}m", reading_min)
    };

    let secondary_footer = container(
        row![
            text(format!("{} words | {} chars | {} para | {:.1} pg | {} read",
                secondary_words, secondary_chars, secondary_paragraphs,
                secondary_pages, reading_display
            ))
                .size(10)
                .color(Theme::TEXT_MUTED),
            Space::with_width(Length::Fill),
            text("Ctrl+Shift+E: toggle split").size(9).color(Theme::TEXT_MUTED),
        ]
    )
    .padding(Padding::from([4, 12]));

    let secondary_panel = container(
        column![secondary_header, secondary_text, secondary_footer]
    )
    .width(Length::FillPortion(1))
    .height(Length::Fill);

    // Visual divider between panels
    let divider = container(
        text("\u{2502}").size(14).color(Theme::BORDER)
    )
    .width(Length::Fixed(4.0))
    .height(Length::Fill);

    container(
        row![primary_panel, divider, secondary_panel]
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
