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
    let dirty_indicator = if is_dirty { " *" } else { "" };
    let project_info = format!("{}{}", project_title, dirty_indicator);

    let progress = stats.progress_string(target_words);

    // Reading time estimate
    let reading_min = stats.word_count as f64 / 250.0;

    let extra = format!(
        "Ch: {} | Sent: {} | Para: {} | Pg: {:.1} | ~{:.0}m read | Docs: {}",
        stats.char_count, stats.sentence_count, stats.paragraph_count,
        stats.page_count, reading_min, stats.document_count
    );

    let session_str = if session_active {
        " | SESSION"
    } else {
        ""
    };

    let content = row![
        text(project_info).size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        text(progress).size(11).color(Theme::TEXT_PRIMARY),
        Space::with_width(12),
        text(format!("{}{}", extra, session_str)).size(11).color(Theme::TEXT_MUTED),
    ]
    .padding(Padding::from([3, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
