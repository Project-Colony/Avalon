use iced::widget::{container, row, text, Space};
use iced::{Element, Length, Padding};

use crate::core::stats::Statistics;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the status bar at the bottom
pub fn view(
    stats: &Statistics,
    target_words: Option<usize>,
    is_dirty: bool,
    project_title: &str,
    session_active: bool,
) -> Element<'static, Message> {
    let dirty_indicator = if is_dirty { " [modified]" } else { "" };
    let project_info = format!("{}{}", project_title, dirty_indicator);

    let progress = stats.progress_string(target_words);
    let extra = format!(
        "Chars: {} | Sentences: {} | Para: {} | Pages: {:.1} | Docs: {}",
        stats.char_count, stats.sentence_count, stats.paragraph_count,
        stats.page_count, stats.document_count
    );

    let session_indicator = if session_active {
        " | Session active"
    } else {
        ""
    };

    let content = row![
        text(project_info).size(12).color(Theme::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        text(progress).size(12).color(Theme::TEXT_PRIMARY),
        Space::with_width(20),
        text(format!("{}{}", extra, session_indicator)).size(12).color(Theme::TEXT_MUTED),
    ]
    .padding(Padding::from([4, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
