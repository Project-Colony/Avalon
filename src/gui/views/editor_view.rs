use iced::widget::{button, column, container, row, scrollable, text, text_editor, Space};
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
    // Document annotations/notes indicator
    let notes_indicator = if editor_state.document.has_notes() {
        "\u{1F4DD} "
    } else {
        ""
    };

    let annotation_text = if editor_state.document.annotation_count() > 0 {
        let open = editor_state.document.open_annotation_count();
        let total = editor_state.document.annotation_count();
        format!(" | \u{1F4AC} {}/{}", open, total)
    } else {
        String::new()
    };

    let footnote_text = if editor_state.document.footnote_count() > 0 {
        format!(" | Fn:{}", editor_state.document.footnote_count())
    } else {
        String::new()
    };

    let header = container(
        row![
            text(format!("{}{}", notes_indicator, title))
                .size(14)
                .color(Theme::TEXT_SECONDARY),
            Space::with_width(8),
            text(format!("{}{}", annotation_text, footnote_text))
                .size(10)
                .color(Theme::TEXT_MUTED),
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
        scrollable(
            row![
                fmt_btn("B", Message::InsertBold),
                fmt_btn("I", Message::InsertItalic),
                fmt_btn("U", Message::InsertUnderline),
                fmt_btn("S", Message::InsertStrikethrough),
                Space::with_width(6),
                fmt_btn("H1", Message::InsertHeading(1)),
                fmt_btn("H2", Message::InsertHeading(2)),
                fmt_btn("H3", Message::InsertHeading(3)),
                Space::with_width(6),
                fmt_btn(">", Message::InsertBlockQuote),
                fmt_btn("Fn", Message::InsertFootnote),
                fmt_btn("--", Message::InsertHRule),
                fmt_btn("```", Message::InsertCodeBlock(String::new())),
                fmt_btn("PgBrk", Message::InsertPageBreak),
                fmt_btn("<!--", Message::InsertComment),
                Space::with_width(6),
                fmt_btn("Link", Message::InsertLink),
                fmt_btn("Img", Message::InsertImage),
                Space::with_width(6),
                fmt_btn("List", Message::InsertListItem("bullet".to_string())),
                fmt_btn("1.", Message::InsertListItem("numbered".to_string())),
                fmt_btn("[ ]", Message::InsertListItem("checkbox".to_string())),
                fmt_btn("Table", Message::InsertTable(3, 3)),
                fmt_btn("Date", Message::InsertDateTime("date".to_string())),
                Space::with_width(6),
                fmt_btn("Dup", Message::DuplicateLine),
                fmt_btn("Del", Message::DeleteLine),
                fmt_btn("Join", Message::JoinLines),
                fmt_btn("Sort", Message::SortLines),
                Space::with_width(Length::Fill),
                fmt_btn("UPPER", Message::TextToUppercase),
                fmt_btn("lower", Message::TextToLowercase),
                fmt_btn("Title", Message::TextToTitleCase),
                Space::with_width(6),
                fmt_btn("CpMD", Message::CopyAsMarkdown),
                fmt_btn("CpHTML", Message::CopyAsHtml),
                fmt_btn("CpTxt", Message::CopyAsPlainText),
            ]
            .spacing(2)
        )
        .direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::new()))
    )
    .padding(Padding::from([2, 16]));

    let editor = text_editor(&editor_state.content)
        .on_action(|action| Message::EditorAction(action))
        .padding(Padding::from([16, 24]))
        .height(Length::Fill);

    let word_count = editor_state.document.word_count();
    let page_est = word_count as f64 / 250.0;
    let para_count = editor_state.document.paragraph_count();
    let sentence_count = editor_state.document.sentence_count();
    let reading_min = word_count as f64 / 250.0;
    let reading_display = if reading_min < 1.0 {
        "<1m".to_string()
    } else if reading_min < 60.0 {
        format!("{:.0}m", reading_min)
    } else {
        format!("{:.1}h", reading_min / 60.0)
    };

    let dirty_indicator = if editor_state.dirty { " \u{2022}" } else { "" };

    // Unique word count for vocabulary richness
    let unique_words = editor_state.document.unique_word_count();
    let richness = if word_count > 0 {
        format!(" | TTR:{:.0}%", unique_words as f64 / word_count as f64 * 100.0)
    } else {
        String::new()
    };

    let stats_text = format!(
        "{}W  |  {}S  |  {}P  |  {:.1}pg  |  ~{} read{}{}",
        word_count, sentence_count, para_count, page_est, reading_display, richness, dirty_indicator
    );
    let stats_bar = container(
        row![
            text(stats_text).size(11).color(Theme::TEXT_MUTED),
            Space::with_width(Length::Fill),
            text(format!("Ln {}, Col {}", editor_state.current_line(), editor_state.current_column()))
                .size(10)
                .color(Theme::TEXT_MUTED),
        ]
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
    let char_count = editor_state.document.char_count();
    let para_count = editor_state.document.paragraph_count();
    let pages = editor_state.document.page_count();

    let footer = container(
        row![
            text(format!("{} words | {} chars | {} para | {:.1} pg",
                word_count, char_count, para_count, pages))
                .size(11)
                .color(Theme::TEXT_MUTED),
            Space::with_width(Length::Fill),
            text(format!("Ln {}", editor_state.cursor_line() + 1))
                .size(10)
                .color(Theme::TEXT_MUTED),
        ]
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
