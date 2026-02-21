use iced::widget::{container, row, text, Space};
use iced::{Element, Length, Padding};

use crate::core::stats::Statistics;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the status bar at the bottom of the application
pub fn view(
    stats: &Statistics,
    target_words: Option<usize>,
    is_dirty: bool,
    project_title: &str,
    session_active: bool,
) -> Element<'static, Message> {
    let dirty_indicator = if is_dirty { " \u{2022}" } else { "" };
    let dirty_color = if is_dirty { Theme::WARNING } else { Theme::TEXT_SECONDARY };
    let project_info = format!("{}{}", project_title, dirty_indicator);

    let progress = stats.progress_string(target_words);

    // Reading time estimate
    let reading_min = stats.word_count as f64 / 250.0;

    // Compact stats
    let stats_text = format!(
        "W:{} | Ch:{} | S:{} | P:{} | Pg:{:.1} | ~{:.0}m",
        stats.word_count, stats.char_count, stats.sentence_count,
        stats.paragraph_count, stats.page_count, reading_min
    );

    // Session indicator
    let session_indicator: Element<'static, Message> = if session_active {
        row![
            Space::with_width(4),
            text("\u{23F1}").size(10),
            Space::with_width(2),
            text("REC").size(9).color(Theme::SUCCESS),
        ].into()
    } else {
        Space::with_width(0).into()
    };

    // Target progress (if set)
    let target_info: Element<'static, Message> = if let Some(target) = target_words {
        if target > 0 {
            let pct = (stats.word_count as f64 / target as f64 * 100.0).min(999.9);
            let remaining = target.saturating_sub(stats.word_count);
            let color = if pct >= 100.0 { Theme::SUCCESS }
                else if pct >= 75.0 { Theme::TEXT_ACCENT }
                else if pct >= 50.0 { Theme::WARNING }
                else { Theme::TEXT_SECONDARY };
            row![
                Space::with_width(8),
                text(format!("{:.0}%", pct)).size(10).color(color),
                Space::with_width(4),
                text(format!("({} left)", remaining)).size(9).color(Theme::TEXT_MUTED),
            ].into()
        } else {
            Space::with_width(0).into()
        }
    } else {
        Space::with_width(0).into()
    };

    // Documents count
    let docs_text = format!("{}doc{}", stats.document_count, if stats.document_count == 1 { "" } else { "s" });

    let content = row![
        text(project_info).size(11).color(dirty_color),
        session_indicator,
        Space::with_width(Length::Fill),
        text(progress).size(10).color(Theme::TEXT_PRIMARY),
        target_info,
        Space::with_width(12),
        text(stats_text).size(10).color(Theme::TEXT_MUTED),
        Space::with_width(8),
        text(docs_text).size(10).color(Theme::TEXT_MUTED),
        Space::with_width(8),
        text("UTF-8").size(9).color(Theme::TEXT_MUTED),
    ]
    .padding(Padding::from([3, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
