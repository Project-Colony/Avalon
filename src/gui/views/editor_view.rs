use iced::widget::{button, column, container, row, text, text_editor, Space};
use iced::{Element, Length, Padding};

use crate::editor::EditorState;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the main text editor view
pub fn view<'a>(
    editor_state: &'a EditorState,
    title: &str,
    script_mode: bool,
    script_element: Option<&str>,
) -> Element<'a, Message> {
    let header = container(
        row![
            text(title.to_string())
                .size(14)
                .color(Theme::TEXT_SECONDARY),
            Space::with_width(Length::Fill),
            if script_mode {
                text(format!("[Script: {}]", script_element.unwrap_or("Action")))
                    .size(11)
                    .color(Theme::TEXT_ACCENT)
            } else {
                text("".to_string()).size(11)
            },
        ]
    )
    .padding(Padding::from([8, 16]));

    // Formatting toolbar
    let format_bar = container(
        row![
            fmt_btn("B", Message::InsertBold),
            fmt_btn("I", Message::InsertItalic),
            fmt_btn("U", Message::InsertUnderline),
            fmt_btn("S", Message::InsertStrikethrough),
            Space::with_width(8),
            fmt_btn("H1", Message::InsertHeading(1)),
            fmt_btn("H2", Message::InsertHeading(2)),
            fmt_btn("H3", Message::InsertHeading(3)),
            Space::with_width(8),
            fmt_btn(">", Message::InsertBlockQuote),
            fmt_btn("Fn", Message::InsertFootnote),
            fmt_btn("--", Message::InsertHRule),
            Space::with_width(Length::Fill),
            fmt_btn("UPPER", Message::TextToUppercase),
            fmt_btn("lower", Message::TextToLowercase),
            fmt_btn("Title", Message::TextToTitleCase),
        ]
        .spacing(2)
    )
    .padding(Padding::from([2, 16]));

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
        format_bar,
        editor,
        stats_bar,
    ];

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn fmt_btn(label: &str, message: Message) -> Element<'static, Message> {
    button(
        text(label.to_string()).size(11).color(Theme::TEXT_SECONDARY),
    )
    .on_press(message)
    .padding(Padding::from([2, 6]))
    .into()
}

/// Render composition mode — minimal, distraction-free writing environment
pub fn view_composition<'a>(editor_state: &'a EditorState, title: &str, word_count: usize, session_words: i64) -> Element<'a, Message> {
    let exit_hint = text("Press Esc to exit  |  F5 to toggle")
        .size(10)
        .color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.3));

    let title_text = text(title.to_string())
        .size(16)
        .color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.5));

    let header = container(
        row![
            Space::with_width(Length::Fill),
            title_text,
            Space::with_width(Length::Fill),
        ]
    )
    .padding(Padding::from([8, 80]))
    .width(Length::Fill);

    let editor = text_editor(&editor_state.content)
        .on_action(|action| Message::EditorAction(action))
        .padding(Padding::from([32, 120]))
        .height(Length::Fill);

    let session_str = if session_words != 0 {
        let sign = if session_words > 0 { "+" } else { "" };
        format!("  |  Session: {}{}", sign, session_words)
    } else {
        String::new()
    };

    let footer = container(
        row![
            Space::with_width(Length::Fill),
            text(format!("{} words  |  ~{:.1} pages{}", word_count, word_count as f64 / 250.0, session_str))
                .size(11)
                .color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.4)),
            Space::with_width(Length::Fill),
        ]
    )
    .padding(Padding::from([4, 80]))
    .width(Length::Fill);

    let bottom = container(
        row![
            Space::with_width(Length::Fill),
            exit_hint,
            Space::with_width(Length::Fill),
        ]
    )
    .padding(Padding::from([2, 80]));

    container(
        column![header, editor, footer, bottom]
    )
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
