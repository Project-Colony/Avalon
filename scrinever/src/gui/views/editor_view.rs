use iced::widget::{button, column, container, row, text, text_editor, Space};
use iced::{Element, Length, Padding};

use crate::editor::EditorState;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the main text editor view
pub fn view<'a>(editor_state: &'a EditorState, title: &str) -> Element<'a, Message> {
    let header = container(
        text(title.to_string())
            .size(14)
            .color(Theme::TEXT_SECONDARY),
    )
    .padding(Padding::from([8, 16]));

    let editor = text_editor(&editor_state.content)
        .on_action(|action| Message::EditorAction(action))
        .padding(Padding::from([16, 24]))
        .height(Length::Fill);

    let word_count = editor_state.document.word_count();
    let char_count = editor_state.document.char_count();
    let page_est = word_count as f64 / 250.0;

    let stats_text = format!(
        "Words: {}  |  Chars: {}  |  Pages: {:.1}",
        word_count, char_count, page_est
    );
    let stats_bar = container(
        text(stats_text)
            .size(12)
            .color(Theme::TEXT_MUTED),
    )
    .padding(Padding::from([4, 16]));

    let content = column![
        header,
        editor,
        stats_bar,
    ];

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Render fullscreen (distraction-free) editor mode
pub fn view_fullscreen<'a>(editor_state: &'a EditorState, title: &str) -> Element<'a, Message> {
    let exit_btn = button(
        text("Exit Focus Mode").size(12).color(Theme::TEXT_MUTED),
    )
    .on_press(Message::ToggleFullscreen)
    .padding(Padding::from([4, 12]));

    let header = container(
        row![
            Space::with_width(Length::Fill),
            text(title.to_string()).size(14).color(Theme::TEXT_SECONDARY),
            Space::with_width(Length::Fill),
            exit_btn,
        ]
        .padding(Padding::from([4, 16]))
    )
    .width(Length::Fill);

    let editor = text_editor(&editor_state.content)
        .on_action(|action| Message::EditorAction(action))
        .padding(Padding::from([24, 80]))
        .height(Length::Fill);

    let word_count = editor_state.document.word_count();
    let footer = container(
        text(format!("{} words", word_count))
            .size(12)
            .color(Theme::TEXT_MUTED),
    )
    .padding(Padding::from([4, 80]))
    .width(Length::Fill);

    container(
        column![header, editor, footer]
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
