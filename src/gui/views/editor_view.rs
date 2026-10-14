use iced::font::Family;
use iced::widget::text::LineHeight;
use iced::widget::{button, column, container, row, scrollable, text, text_editor, Space};
use iced::{Element, Font, Length, Padding};

use crate::core::metadata::ProjectSettings;
use crate::editor::EditorState;
use crate::gui::app::Message;
use crate::gui::theme::{self, Theme};

/// How the editor sets its text, from the project's appearance settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextLook {
    pub font: Font,
    /// The font size times the zoom.
    pub size: f32,
    /// The line height, as a multiple of the size.
    pub line_height: f32,
}

impl TextLook {
    pub fn new(settings: &ProjectSettings) -> Self {
        let family = match settings.editor_font.as_str() {
            "serif" => Some(Family::Serif),
            "sans-serif" => Some(Family::SansSerif),
            _ => None,
        };
        Self {
            // "monospace" keeps the bundled JetBrains Mono the rest of the app uses.
            font: family.map_or(theme::APP_FONT, |family| Font {
                family,
                ..Font::DEFAULT
            }),
            size: settings.editor_font_size * settings.editor_zoom,
            line_height: settings.line_spacing,
        }
    }
}

/// The editable text, set in the project's font, size, zoom and line spacing.
pub fn writing_area<'a>(
    editor_state: &'a EditorState,
    settings: &ProjectSettings,
    padding: Padding,
) -> Element<'a, Message> {
    let look = TextLook::new(settings);
    text_editor(&editor_state.content)
        .on_action(Message::EditorAction)
        .font(look.font)
        .size(look.size)
        .line_height(LineHeight::Relative(look.line_height))
        .padding(padding)
        .height(Length::Fill)
        .into()
}

/// Centre `content` in a column `percent` of the available width wide.
fn text_column(content: Element<'_, Message>, percent: f32) -> Element<'_, Message> {
    let Some((margin, middle)) = column_portions(percent) else {
        return content;
    };
    row![
        Space::new().width(Length::FillPortion(margin)),
        container(content)
            .width(Length::FillPortion(middle))
            .height(Length::Fill),
        Space::new().width(Length::FillPortion(margin)),
    ]
    .height(Length::Fill)
    .into()
}

/// The fill portions of each margin and of the text for a text column
/// `percent` wide, or `None` when the text takes the whole width.
fn column_portions(percent: f32) -> Option<(u16, u16)> {
    let percent = percent.round().clamp(10.0, 100.0) as u16;
    (percent < 100).then(|| (100 - percent, 2 * percent))
}

/// Render the main text editor view
pub fn view<'a>(
    editor_state: &'a EditorState,
    title: &str,
    script_mode: bool,
    script_element: Option<&str>,
    settings: &ProjectSettings,
) -> Element<'a, Message> {
    // Document annotations/notes indicator
    let notes_indicator = if editor_state.document.has_notes() {
        "\u{f044} "
    } else {
        ""
    };

    let annotation_text = if editor_state.document.annotation_count() > 0 {
        let open = editor_state.document.open_annotation_count();
        let total = editor_state.document.annotation_count();
        format!(" | \u{f075} {}/{}", open, total)
    } else {
        String::new()
    };

    let footnote_text = if editor_state.document.footnote_count() > 0 {
        format!(" | Fn:{}", editor_state.document.footnote_count())
    } else {
        String::new()
    };

    let header = container(row![
        text(format!("{}{}", notes_indicator, title))
            .size(14)
            .color(Theme::TEXT_SECONDARY),
        Space::new().width(8),
        text(format!("{}{}", annotation_text, footnote_text))
            .size(10)
            .color(Theme::TEXT_MUTED),
        Space::new().width(Length::Fill),
        if script_mode {
            text(format!("[Script: {}]", script_element.unwrap_or("Action")))
                .size(11)
                .color(Theme::TEXT_ACCENT)
        } else {
            text("".to_string()).size(11)
        },
    ])
    .style(theme::view_header_style)
    .padding(Padding::from([8, 16]))
    .width(Length::Fill);

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
                Space::new().width(8),
                text("Tab: cycle element").size(9).color(Theme::TEXT_MUTED),
            ]
            .spacing(3),
        )
        .padding(Padding::from([2, 16]))
        .into()
    } else {
        Space::new().height(0).into()
    };

    // Formatting toolbar
    let format_bar = container(
        scrollable(
            row![
                fmt_btn("B", Message::InsertBold),
                fmt_btn("I", Message::InsertItalic),
                fmt_btn("U", Message::InsertUnderline),
                fmt_btn("S", Message::InsertStrikethrough),
                Space::new().width(6),
                fmt_btn("H1", Message::InsertHeading(1)),
                fmt_btn("H2", Message::InsertHeading(2)),
                fmt_btn("H3", Message::InsertHeading(3)),
                Space::new().width(6),
                fmt_btn(">", Message::InsertBlockQuote),
                fmt_btn("Fn", Message::InsertFootnote),
                fmt_btn("--", Message::InsertHRule),
                fmt_btn("```", Message::InsertCodeBlock(String::new())),
                fmt_btn("PgBrk", Message::InsertPageBreak),
                fmt_btn("<!--", Message::InsertComment),
                Space::new().width(6),
                fmt_btn("Link", Message::InsertLink),
                fmt_btn("Img", Message::InsertImage),
                Space::new().width(6),
                fmt_btn("List", Message::InsertListItem("bullet".to_string())),
                fmt_btn("1.", Message::InsertListItem("numbered".to_string())),
                fmt_btn("[ ]", Message::InsertListItem("checkbox".to_string())),
                fmt_btn("Table", Message::InsertTable(3, 3)),
                fmt_btn("Date", Message::InsertDateTime("date".to_string())),
                Space::new().width(6),
                fmt_btn("Dup", Message::DuplicateLine),
                fmt_btn("Del", Message::DeleteLine),
                fmt_btn("Join", Message::JoinLines),
                fmt_btn("Sort", Message::SortLines),
                Space::new().width(12),
                fmt_btn("UPPER", Message::TextToUppercase),
                fmt_btn("lower", Message::TextToLowercase),
                fmt_btn("Title", Message::TextToTitleCase),
                Space::new().width(6),
                fmt_btn("CpMD", Message::CopyAsMarkdown),
                fmt_btn("CpHTML", Message::CopyAsHtml),
                fmt_btn("CpTxt", Message::CopyAsPlainText),
            ]
            .spacing(2),
        )
        .direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::new())),
    )
    .style(theme::view_header_style)
    .padding(Padding::from([4, 16]))
    .width(Length::Fill);

    let editor = text_column(
        writing_area(editor_state, settings, Padding::from([16, 24])),
        settings.editor_width,
    );

    let word_count = editor_state.document.word_count();
    let page_est = word_count as f64 / crate::core::WORDS_PER_PAGE as f64;
    let para_count = editor_state.document.paragraph_count();
    let sentence_count = editor_state.document.sentence_count();
    let reading_min = word_count as f64 / crate::core::READING_WPM;
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
    let stats_bar = container(row![
        text(stats_text).size(11).color(Theme::TEXT_MUTED),
        Space::new().width(Length::Fill),
        text(format!(
            "Ln {}, Col {}",
            editor_state.current_line(),
            editor_state.current_column()
        ))
        .size(10)
        .color(Theme::TEXT_MUTED),
    ])
    .style(theme::view_footer_style)
    .padding(Padding::from([4, 16]))
    .width(Length::Fill);

    let content = column![header, script_bar, format_bar, editor, stats_bar,];

    container(content).width(Length::Fill).height(Length::Fill).into()
}

fn script_el_btn(label: &str, element_name: &str, current: Option<&str>) -> Element<'static, Message> {
    let is_active = current == Some(element_name);
    let color = if is_active {
        Theme::TEXT_ACCENT
    } else {
        Theme::TEXT_MUTED
    };
    button(text(label.to_string()).size(10).color(color))
        .on_press(Message::SetScriptElement(element_name.to_string()))
        .padding(Padding::from([2, 5]))
        .into()
}

fn fmt_btn(label: &str, message: Message) -> Element<'static, Message> {
    button(text(label.to_string()).size(11).color(Theme::TEXT_SECONDARY))
        .on_press(message)
        .padding(Padding::from([4, 8]))
        .into()
}

/// Render composition mode — minimal, distraction-free writing environment
/// Features: centered text, session stats, ambient fade, word count goal tracking
pub fn view_composition<'a>(
    editor_state: &'a EditorState,
    title: &str,
    word_count: usize,
    session_words: i64,
    settings: &ProjectSettings,
) -> Element<'a, Message> {
    let exit_hint = text("Esc: exit  |  F5: toggle  |  Ctrl+S: save")
        .size(10)
        .color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.25));

    let title_text = text(title.to_string())
        .size(16)
        .color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.5));

    // Reading time estimate
    let reading_min = word_count as f64 / crate::core::READING_WPM;

    let header = container(row![
        Space::new().width(Length::Fill),
        title_text,
        Space::new().width(Length::Fill),
    ])
    .padding(Padding::from([8, 80]))
    .width(Length::Fill);

    let editor = text_column(
        writing_area(editor_state, settings, Padding::from([32, 24])),
        settings.fullscreen_text_width,
    );

    // Session progress
    let session_str = if session_words != 0 {
        let sign = if session_words > 0 { "+" } else { "" };
        format!("  |  Session: {}{}", sign, session_words)
    } else {
        String::new()
    };

    // Paragraph count
    let para_count = editor_state.document.paragraph_count();

    let footer = container(row![
        Space::new().width(Length::Fill),
        text(format!(
            "{} words  |  {} para  |  ~{:.1} pages  |  ~{:.0} min read{}",
            word_count,
            para_count,
            word_count as f64 / crate::core::WORDS_PER_PAGE as f64,
            reading_min,
            session_str
        ))
        .size(11)
        .color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.4)),
        Space::new().width(Length::Fill),
    ])
    .padding(Padding::from([4, 80]))
    .width(Length::Fill);

    let bottom = container(row![
        Space::new().width(Length::Fill),
        exit_hint,
        Space::new().width(Length::Fill),
    ])
    .padding(Padding::from([2, 80]));

    container(column![header, editor, footer, bottom])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Render fullscreen (distraction-free) editor mode
pub fn view_fullscreen<'a>(
    editor_state: &'a EditorState,
    title: &str,
    settings: &ProjectSettings,
) -> Element<'a, Message> {
    let exit_btn = button(text("Exit Focus Mode").size(12).color(Theme::TEXT_MUTED))
        .on_press(Message::ToggleFullscreen)
        .padding(Padding::from([4, 12]));

    let header = container(
        row![
            Space::new().width(Length::Fill),
            text(title.to_string()).size(14).color(Theme::TEXT_SECONDARY),
            Space::new().width(Length::Fill),
            exit_btn,
        ]
        .padding(Padding::from([4, 16])),
    )
    .width(Length::Fill);

    let editor = writing_area(editor_state, settings, Padding::from([24, 80]));

    let word_count = editor_state.document.word_count();
    let char_count = editor_state.document.char_count();
    let para_count = editor_state.document.paragraph_count();
    let pages = editor_state.document.page_count();

    let footer = container(row![
        text(format!(
            "{} words | {} chars | {} para | {:.1} pg",
            word_count, char_count, para_count, pages
        ))
        .size(11)
        .color(Theme::TEXT_MUTED),
        Space::new().width(Length::Fill),
        text(format!("Ln {}", editor_state.cursor_line() + 1))
            .size(10)
            .color(Theme::TEXT_MUTED),
    ])
    .padding(Padding::from([4, 80]))
    .width(Length::Fill);

    container(column![header, editor, footer])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_look_follows_the_appearance_settings() {
        let mut settings = ProjectSettings::default();
        let default = TextLook::new(&settings);
        assert_eq!(default.font, theme::APP_FONT);
        assert_eq!(default.size, 16.0);
        assert_eq!(default.line_height, 1.5);

        settings.editor_font = "serif".to_string();
        assert_eq!(TextLook::new(&settings).font.family, Family::Serif);
        settings.editor_font = "sans-serif".to_string();
        assert_eq!(TextLook::new(&settings).font.family, Family::SansSerif);

        settings.editor_font_size = 20.0;
        assert_eq!(TextLook::new(&settings).size, 20.0);
        settings.editor_zoom = 1.5;
        assert_eq!(TextLook::new(&settings).size, 30.0);

        settings.line_spacing = 2.0;
        assert_eq!(TextLook::new(&settings).line_height, 2.0);
    }

    #[test]
    fn column_portions_give_the_text_its_share_of_the_width() {
        // Margins of 20 and a text of 160 out of 200: the text takes 80%.
        assert_eq!(column_portions(80.0), Some((20, 160)));
        assert_eq!(column_portions(60.0), Some((40, 120)));
        assert_eq!(column_portions(100.0), None);
        assert_eq!(column_portions(5.0), Some((90, 20)));
    }
}
