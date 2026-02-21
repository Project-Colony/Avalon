use iced::widget::{column, container, text, text_editor};
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

    let stats_text = format!("Words: {}  |  Characters: {}", word_count, char_count);
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
