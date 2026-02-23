use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::core::history::WritingHistory;
use crate::gui::app::Message;
use crate::gui::theme::{self, Theme};

/// Render the writing history panel (bottom panel)
pub fn view(history: &WritingHistory) -> Element<'static, Message> {
    let header = row![
        text("WRITING HISTORY").size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        text(format!("{} day(s) tracked", history.entries.len()))
            .size(10)
            .color(Theme::TEXT_MUTED),
    ];

    // Summary stats
    let total_words = history.total_words_written();
    let total_time = history.total_time_seconds();
    let avg_daily = history.average_words_per_day();
    let streak = history.current_streak();
    let best = history.best_day();

    let total_hours = total_time / 3600;
    let total_mins = (total_time % 3600) / 60;

    // Words per minute productivity
    let wpm = if total_time > 60 {
        format!("{:.1}", total_words as f64 / (total_time as f64 / 60.0))
    } else {
        "-".to_string()
    };

    // Estimated pages written
    let pages = total_words as f64 / 250.0;

    let summary = row![
        stat_item("Total", &format_number(total_words as u64)),
        stat_item("Pages", &format!("{:.1}", pages)),
        stat_item("Avg/Day", &format!("{:.0}", avg_daily)),
        stat_item("Streak", &format!("{} d", streak)),
        stat_item("Time", &format!("{}h {}m", total_hours, total_mins)),
        stat_item("WPM", &wpm),
        stat_item("Best", &best.map(|e| format_number(e.words_written as u64)).unwrap_or_else(|| "-".to_string())),
    ]
    .spacing(12);

    // Recent entries with Unicode bar graph
    let mut entries_col = column![].spacing(2);
    let recent = history.recent(14);

    if recent.is_empty() {
        entries_col = entries_col.push(
            text("No writing history yet. Start writing to track progress!")
                .size(12)
                .color(Theme::TEXT_MUTED),
        );
    }

    // Find max words in recent for bar scaling
    let max_words = recent.iter()
        .map(|e| e.words_written.max(0))
        .max()
        .unwrap_or(1)
        .max(1) as f64;

    for entry in recent.iter().rev() {
        let date_str = entry.date.format("%a %m-%d").to_string();
        let words_color = if entry.words_written > 500 {
            Theme::SUCCESS
        } else if entry.words_written > 0 {
            Theme::TEXT_ACCENT
        } else {
            Theme::TEXT_MUTED
        };

        let words_str = if entry.words_written >= 0 {
            format!("+{}", entry.words_written)
        } else {
            format!("{}", entry.words_written)
        };

        let time_mins = entry.time_spent_seconds / 60;

        // Unicode block bar graph (scaled to max)
        let bar_ratio = entry.words_written.max(0) as f64 / max_words;
        let bar_len = (bar_ratio * 16.0).round() as usize;
        let filled = "\u{2588}".repeat(bar_len);
        let empty = "\u{2591}".repeat(16_usize.saturating_sub(bar_len));
        let bar = format!("{}{}", filled, empty);

        let entry_row = row![
            text(date_str).size(11).color(Theme::TEXT_MUTED).width(Length::Fixed(65.0)),
            text(words_str).size(11).color(words_color).width(Length::Fixed(55.0)),
            text(format!("{}m", time_mins)).size(11).color(Theme::TEXT_SECONDARY).width(Length::Fixed(35.0)),
            text(bar).size(9).color(Theme::TEXT_ACCENT),
        ]
        .spacing(6);

        entries_col = entries_col.push(entry_row);
    }

    // Activity ratio indicator
    let activity = history.activity_ratio();
    let activity_color = if activity >= 0.8 { Theme::SUCCESS }
    else if activity >= 0.5 { Theme::WARNING }
    else { Theme::TEXT_MUTED };
    let activity_pct = format!("{:.0}% active", activity * 100.0);

    let longest = history.longest_streak();
    let week_words = history.words_this_week();

    // Productivity trend and consistency
    let trend = history.productivity_trend();
    let trend_label = history.trend_label();
    let trend_color = if trend >= 1.1 { Theme::SUCCESS }
    else if trend >= 0.9 { Theme::TEXT_ACCENT }
    else { Theme::WARNING };

    let consistency = history.consistency_score();
    let cons_label = history.consistency_label();
    let cons_color = if consistency >= 60.0 { Theme::SUCCESS }
    else if consistency >= 40.0 { Theme::WARNING }
    else { Theme::TEXT_MUTED };

    let extra_stats = row![
        text(format!("Longest streak: {}d", longest)).size(10).color(Theme::TEXT_MUTED),
        Space::with_width(12),
        text(format!("This week: {}", format_number(week_words.max(0) as u64)))
            .size(10).color(Theme::TEXT_ACCENT),
        Space::with_width(12),
        text(activity_pct).size(10).color(activity_color),
        Space::with_width(12),
        text(format!("Trend: {}", trend_label)).size(10).color(trend_color),
        Space::with_width(12),
        text(format!("Consistency: {}", cons_label)).size(10).color(cons_color),
    ];

    let content = column![
        header,
        Space::with_height(4),
        summary,
        Space::with_height(4),
        extra_stats,
        Space::with_height(6),
        scrollable(entries_col).height(Length::Fixed(120.0)),
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .style(theme::panel_style)
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

/// Format large numbers with comma separators
fn format_number(n: u64) -> String {
    if n < 1_000 {
        return n.to_string();
    }
    let s = n.to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}
