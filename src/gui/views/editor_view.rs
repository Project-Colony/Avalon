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

    // Script element picker bar (only shown in script mode)
    let script_bar: Element<'a, Message> = if script_mode {
        container(
            row![
                script_el_btn("Scene", "Scene Heading", script_element),
                script_el_btn("Action", "Action", script_element),
                script_el_btn("Char", "Character", script_element),
                script_el_btn("Dial", "Dialogue", script_element),
                script_el_btn("Paren", "Parenthetical", script_element),
                script_el_btn("Trans", "Transition", script_element),
                script_el_btn("Shot", "Shot", script_element),
                script_el_btn("Note", "Note", script_element),
                Space::with_width(8),
                text("Tab: cycle element").size(9).color(Theme::TEXT_MUTED),
            ]
            .spacing(3)
        )
        .padding(Padding::from([2, 16]))
        .into()
    } else {
        Space::with_height(0).into()
    };

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
            Space::with_width(8),
            fmt_btn("CpMD", Message::CopyAsMarkdown),
            fmt_btn("CpHTML", Message::CopyAsHtml),
            fmt_btn("CpTxt", Message::CopyAsPlainText),
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
        script_bar,
        format_bar,
        editor,
        stats_bar,
    ];

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn script_el_btn(label: &str, element_name: &str, current: Option<&str>) -> Element<'static, Message> {
    let is_active = current == Some(element_name);
    let color = if is_active { Theme::TEXT_ACCENT } else { Theme::TEXT_MUTED };
    button(
        text(label.to_string()).size(10).color(color),
    )
    .on_press(Message::SetScriptElement(element_name.to_string()))
    .padding(Padding::from([2, 5]))
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
/// Features: centered text, session stats, ambient fade, word count goal tracking
pub fn view_composition<'a>(editor_state: &'a EditorState, title: &str, word_count: usize, session_words: i64) -> Element<'a, Message> {
    let exit_hint = text("Esc: exit  |  F5: toggle  |  Ctrl+S: save")
        .size(10)
        .color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.25));

    let title_text = text(title.to_string())
        .size(16)
        .color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.5));

    // Reading time estimate
    let reading_min = word_count as f64 / 250.0;

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

    // Session progress
    let session_str = if session_words != 0 {
        let sign = if session_words > 0 { "+" } else { "" };
        format!("  |  Session: {}{}", sign, session_words)
    } else {
        String::new()
    };

    // Paragraph count
    let para_count = editor_state.document.paragraph_count();

    let footer = container(
        row![
            Space::with_width(Length::Fill),
            text(format!(
                "{} words  |  {} para  |  ~{:.1} pages  |  ~{:.0} min read{}",
                word_count, para_count, word_count as f64 / 250.0, reading_min, session_str
            ))
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
