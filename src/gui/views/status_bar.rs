use iced::widget::{container, row, text, Space};
use iced::{Element, Length, Padding};

use crate::core::stats::Statistics;
use crate::gui::app::Message;
use crate::gui::theme::{self, Theme};

/// View mode label for the status bar
pub fn view_mode_label(mode: &str) -> &str {
    match mode {
        "editor" => "Editor",
        "corkboard" => "Corkboard",
        "outliner" => "Outliner",
        "scrivenings" => "Scrivenings",
        _ => mode,
    }
}

/// Render the status bar at the bottom of the application
pub fn view(
    stats: &Statistics,
    target_words: Option<usize>,
    is_dirty: bool,
    project_title: &str,
    session_active: bool,
    cursor_line: usize,
    cursor_col: usize,
    timer_running: bool,
    timer_remaining: &str,
    writing_streak: usize,
) -> Element<'static, Message> {
    let dirty_indicator = if is_dirty { " \u{2022}" } else { "" };
    let dirty_color = if is_dirty { Theme::WARNING } else { Theme::TEXT_SECONDARY };
    let project_info = format!("{}{}", project_title, dirty_indicator);

    let progress = stats.progress_string(target_words);

    // Reading time estimate
    let reading_min = stats.word_count as f64 / crate::core::READING_WPM;
    let reading_display = if reading_min < 1.0 {
        "<1m read".to_string()
    } else if reading_min < 60.0 {
        format!("~{:.0}m read", reading_min)
    } else {
        format!("~{:.1}h read", reading_min / 60.0)
    };

    // Compact stats with formatting
    let stats_text = format!(
        "W:{} | Ch:{} | S:{} | P:{} | Pg:{:.1} | {}",
        format_stat(stats.word_count),
        format_stat(stats.char_count),
        format_stat(stats.sentence_count),
        stats.paragraph_count,
        stats.page_count,
        reading_display
    );

    // Session indicator
    let session_indicator: Element<'static, Message> = if session_active {
        row![
            Space::with_width(4),
            text("\u{f017}").size(10),
            Space::with_width(2),
            text("REC").size(9).color(Theme::SUCCESS),
        ].into()
    } else {
        Space::with_width(0).into()
    };

    // Timer indicator
    let timer_indicator: Element<'static, Message> = if timer_running {
        row![
            Space::with_width(4),
            text("\u{f017}").size(10),
            Space::with_width(2),
            text(timer_remaining.to_string()).size(9).color(Theme::TEXT_ACCENT),
        ].into()
    } else {
        Space::with_width(0).into()
    };

    // Cursor position
    let cursor_info = format!("Ln {}, Col {}", cursor_line, cursor_col);

    // Target progress (if set)
    let target_info: Element<'static, Message> = if let Some(target) = target_words {
        if target > 0 {
            let pct = (stats.word_count as f64 / target as f64 * 100.0).min(999.9);
            let remaining = target.saturating_sub(stats.word_count);
            let color = if pct >= 100.0 {
                Theme::SUCCESS
            } else if pct >= 75.0 {
                Theme::TEXT_ACCENT
            } else if pct >= 50.0 {
                Theme::WARNING
            } else {
                Theme::TEXT_SECONDARY
            };

            // Mini progress bar
            let bar_width: usize = 10;
            let filled = ((pct / 100.0) * bar_width as f64) as usize;
            let empty = bar_width.saturating_sub(filled);
            let bar = format!(
                "{}{}",
                "\u{2588}".repeat(filled),
                "\u{2591}".repeat(empty)
            );

            row![
                Space::with_width(8),
                text(bar).size(8).color(color),
                Space::with_width(4),
                text(format!("{:.0}%", pct)).size(10).color(color),
                Space::with_width(4),
                text(format!("({} left)", format_stat(remaining)))
                    .size(9)
                    .color(Theme::TEXT_MUTED),
            ]
            .into()
        } else {
            Space::with_width(0).into()
        }
    } else {
        Space::with_width(0).into()
    };

    // Documents count
    let docs_text = format!(
        "{}doc{}",
        stats.document_count,
        if stats.document_count == 1 { "" } else { "s" }
    );

    // Writing streak indicator
    let streak_indicator: Element<'static, Message> = if writing_streak > 0 {
        let streak_color = if writing_streak >= 7 { Theme::SUCCESS }
        else if writing_streak >= 3 { Theme::WARNING }
        else { Theme::TEXT_MUTED };
        let streak_icon = if writing_streak >= 7 { "\u{f06d}" } else { "\u{f0e7}" };
        row![
            Space::with_width(4),
            text(format!("{} {}d", streak_icon, writing_streak)).size(9).color(streak_color),
        ].into()
    } else {
        Space::with_width(0).into()
    };

    let content = row![
        text(project_info).size(11).color(dirty_color),
        session_indicator,
        timer_indicator,
        streak_indicator,
        Space::with_width(Length::Fill),
        text(progress).size(10).color(Theme::TEXT_PRIMARY),
        target_info,
        Space::with_width(12),
        text(stats_text).size(10).color(Theme::TEXT_MUTED),
        Space::with_width(8),
        text(docs_text).size(10).color(Theme::TEXT_MUTED),
        Space::with_width(8),
        text(cursor_info).size(9).color(Theme::TEXT_MUTED),
        Space::with_width(4),
        text("UTF-8").size(9).color(Theme::TEXT_MUTED),
    ]
    .padding(Padding::from([3, 12]));

    container(content)
        .style(theme::status_bar_style)
        .width(Length::Fill)
        .into()
}

/// Format a number with comma separators for large values
fn format_stat(n: usize) -> String {
    theme::format_compact(n)
}
