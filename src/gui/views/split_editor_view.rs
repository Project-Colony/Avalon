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
    // Primary editor (left)
    let primary_header = container(
        row![
            text(primary_title.to_string())
                .size(13)
                .color(Theme::TEXT_ACCENT),
            Space::with_width(Length::Fill),
            text("Primary").size(10).color(Theme::TEXT_MUTED),
        ]
        .padding(Padding::from([4, 12]))
    )
    .width(Length::Fill);

    let primary = text_editor(&primary_editor.content)
        .on_action(|action| Message::EditorAction(action))
        .padding(Padding::from([12, 16]))
        .height(Length::Fill);

    let primary_words = primary_editor.document.word_count();
    let primary_footer = container(
        text(format!("{} words", primary_words))
            .size(11)
            .color(Theme::TEXT_MUTED),
    )
    .padding(Padding::from([4, 12]));

    let primary_panel = container(
        column![primary_header, primary, primary_footer]
    )
    .width(Length::FillPortion(1))
    .height(Length::Fill);

    // Secondary (right) - read-only text display
    let secondary_header = container(
        row![
            text(secondary_title.to_string())
                .size(13)
                .color(Theme::TEXT_SECONDARY),
            Space::with_width(Length::Fill),
            text("Reference").size(10).color(Theme::TEXT_MUTED),
            Space::with_width(8),
            button(
                text("Close").size(10).color(Theme::TEXT_MUTED),
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

    let secondary_words = secondary_content.split_whitespace().count();
    let secondary_footer = container(
        text(format!("{} words", secondary_words))
            .size(11)
            .color(Theme::TEXT_MUTED),
    )
    .padding(Padding::from([4, 12]));

    let secondary_panel = container(
        column![secondary_header, secondary_text, secondary_footer]
    )
    .width(Length::FillPortion(1))
    .height(Length::Fill);

    // Divider
    let divider = container(Space::with_width(2))
        .width(Length::Fixed(2.0))
        .height(Length::Fill);

    container(
        row![primary_panel, divider, secondary_panel]
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
