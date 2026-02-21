use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::core::history::WritingHistory;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the writing history panel (bottom panel)
pub fn view(history: &WritingHistory) -> Element<'static, Message> {
    let header = text("WRITING HISTORY")
        .size(11)
        .color(Theme::TEXT_SECONDARY);

    // Summary stats
    let total_words = history.total_words_written();
    let total_time = history.total_time_seconds();
    let avg_daily = history.average_words_per_day();
    let streak = history.current_streak();
    let best = history.best_day();

    let total_hours = total_time / 3600;
    let total_mins = (total_time % 3600) / 60;

    let summary = row![
        stat_item("Total Written", &format!("{}", total_words)),
        stat_item("Days Recorded", &format!("{}", history.entries.len())),
        stat_item("Avg/Day", &format!("{:.0}", avg_daily)),
        stat_item("Streak", &format!("{} days", streak)),
        stat_item("Time", &format!("{}h {}m", total_hours, total_mins)),
        stat_item("Best Day", &best.map(|e| format!("{}", e.words_written)).unwrap_or_else(|| "-".to_string())),
    ]
    .spacing(16);

    // Recent entries
    let mut entries_col = column![].spacing(2);
    let recent = history.recent(14);

    if recent.is_empty() {
        entries_col = entries_col.push(
            text("No writing history yet. Start writing to track progress!")
                .size(12)
                .color(Theme::TEXT_MUTED),
        );
    }

    for entry in recent.iter().rev() {
        let date_str = entry.date.format("%Y-%m-%d").to_string();
        let words_color = if entry.words_written > 0 { Theme::SUCCESS } else { Theme::TEXT_MUTED };
        let words_str = if entry.words_written >= 0 {
            format!("+{}", entry.words_written)
        } else {
            format!("{}", entry.words_written)
        };

        let time_mins = entry.time_spent_seconds / 60;

        // Simple ASCII bar graph
        let bar_len = (entry.words_written.max(0) as usize / 50).min(20);
        let bar: String = "|".repeat(bar_len);

        let entry_row = row![
            text(date_str).size(11).color(Theme::TEXT_MUTED).width(Length::Fixed(80.0)),
            text(words_str).size(11).color(words_color).width(Length::Fixed(60.0)),
            text(format!("{}m", time_mins)).size(11).color(Theme::TEXT_SECONDARY).width(Length::Fixed(40.0)),
            text(bar).size(11).color(Theme::TEXT_ACCENT),
        ]
        .spacing(8);

        entries_col = entries_col.push(entry_row);
    }

    let content = column![
        header,
        Space::with_height(4),
        summary,
        Space::with_height(8),
        scrollable(entries_col).height(Length::Fixed(120.0)),
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}

fn stat_item(label: &str, value: &str) -> Element<'static, Message> {
    column![
        text(label.to_string()).size(10).color(Theme::TEXT_MUTED),
        text(value.to_string()).size(13).color(Theme::TEXT_PRIMARY),
    ]
    .spacing(1)
    .into()
}
