use iced::widget::{column, container, row, text, text_input, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Data for the project targets panel
pub struct TargetsData {
    pub project_target: Option<usize>,
    pub current_words: usize,
    pub deadline: String,
    pub session_target: usize,
    pub session_words: i64,
    pub days_remaining: Option<i64>,
    pub words_per_day_needed: Option<usize>,
}

/// Render the project targets panel (bottom panel)
pub fn view(data: &TargetsData) -> Element<'static, Message> {
    let header = text("PROJECT TARGETS")
        .size(11)
        .color(Theme::TEXT_SECONDARY);

    // Project target progress
    let project_progress: Element<'static, Message> = if let Some(target) = data.project_target {
        let pct = if target > 0 {
            (data.current_words as f64 / target as f64 * 100.0).min(100.0)
        } else {
            0.0
        };

        let bar_width: usize = 40;
        let filled = ((pct / 100.0) * bar_width as f64) as usize;
        let empty = bar_width.saturating_sub(filled);
        let bar = format!(
            "\u{2503}{}{}\u{2503} {:.1}%",
            "\u{2588}".repeat(filled),
            "\u{2591}".repeat(empty),
            pct
        );

        let progress_color = if pct >= 100.0 {
            Theme::SUCCESS
        } else if pct >= 75.0 {
            iced::Color::from_rgb(0.3, 0.7, 0.3)
        } else if pct >= 50.0 {
            Theme::WARNING
        } else if pct >= 25.0 {
            iced::Color::from_rgb(0.8, 0.6, 0.2)
        } else {
            Theme::TEXT_SECONDARY
        };

        let remaining = target.saturating_sub(data.current_words);
        let pages_done = data.current_words / 250;
        let pages_total = target / 250;

        column![
            row![
                text("Manuscript Target:").size(12).color(Theme::TEXT_MUTED),
                Space::with_width(8),
                text(format!(
                    "{} / {} words ({} remaining)",
                    format_number(data.current_words),
                    format_number(target),
                    format_number(remaining)
                ))
                .size(12)
                .color(Theme::TEXT_PRIMARY),
            ],
            text(bar).size(12).color(progress_color),
            row![
                text(format!("~{}/{} pages", pages_done, pages_total))
                    .size(10)
                    .color(Theme::TEXT_MUTED),
            ],
        ]
        .spacing(2)
        .into()
    } else {
        text("No project target set. Set one in Project Settings (Ctrl+,)")
            .size(12)
            .color(Theme::TEXT_MUTED)
            .into()
    };

    // Session target progress
    let session_progress: Element<'static, Message> = if data.session_target > 0 {
        let pct = (data.session_words.max(0) as f64 / data.session_target as f64 * 100.0).min(100.0);
        let bar_width: usize = 20;
        let filled = ((pct / 100.0) * bar_width as f64) as usize;
        let empty = bar_width.saturating_sub(filled);
        let bar = format!(
            "\u{2503}{}{}\u{2503} {:.0}%",
            "\u{2588}".repeat(filled),
            "\u{2591}".repeat(empty),
            pct
        );
        let session_remaining = (data.session_target as i64 - data.session_words).max(0);

        let color = if pct >= 100.0 {
            Theme::SUCCESS
        } else {
            Theme::TEXT_SECONDARY
        };

        column![
            row![
                text("Session:").size(11).color(Theme::TEXT_MUTED),
                Space::with_width(4),
                text(format!(
                    "{}/{} words ({} left)",
                    data.session_words.max(0),
                    data.session_target,
                    session_remaining
                ))
                .size(11)
                .color(Theme::TEXT_PRIMARY),
            ],
            text(bar).size(11).color(color),
        ]
        .spacing(1)
        .into()
    } else {
        Space::with_height(0).into()
    };

    // Deadline info
    let deadline_info: Element<'static, Message> = if !data.deadline.is_empty() {
        let deadline_row = row![
            text("\u{f073} Deadline:").size(11).color(Theme::TEXT_MUTED),
            Space::with_width(4),
            text(data.deadline.clone()).size(11).color(Theme::TEXT_PRIMARY),
        ];

        let pace_info: Element<'static, Message> = match (data.days_remaining, data.words_per_day_needed) {
            (Some(days), Some(wpd)) if days > 0 => {
                let weeks = days / 7;
                let remaining_days = days % 7;
                let time_str = if weeks > 0 {
                    format!("{} weeks, {} days remaining", weeks, remaining_days)
                } else {
                    format!("{} days remaining", days)
                };

                let urgency_color = if days <= 7 {
                    Theme::ERROR
                } else if days <= 30 {
                    Theme::WARNING
                } else {
                    Theme::TEXT_SECONDARY
                };

                row![
                    text(time_str).size(11).color(urgency_color),
                    Space::with_width(8),
                    text(format!("{} words/day needed", format_number(wpd)))
                        .size(11)
                        .color(Theme::WARNING),
                    Space::with_width(8),
                    text(format!("(~{} pages/day)", (wpd / 250).max(1)))
                        .size(10)
                        .color(Theme::TEXT_MUTED),
                ]
                .into()
            }
            (Some(days), _) if days <= 0 => {
                text("\u{f071} Deadline has passed!").size(11).color(Theme::ERROR).into()
            }
            _ => Space::with_height(0).into(),
        };

        column![deadline_row, pace_info].spacing(2).into()
    } else {
        Space::with_height(0).into()
    };

    // Deadline input
    let deadline_input_row = row![
        text("Set deadline:").size(11).color(Theme::TEXT_MUTED),
        Space::with_width(4),
        text_input("YYYY-MM-DD", &data.deadline)
            .on_input(|val| Message::SettingsSetDeadline(val))
            .size(11)
            .padding(4)
            .width(Length::Fixed(120.0)),
    ]
    .align_y(iced::Alignment::Center);

    let content = column![
        header,
        Space::with_height(4),
        project_progress,
        Space::with_height(4),
        session_progress,
        Space::with_height(6),
        deadline_info,
        Space::with_height(4),
        deadline_input_row,
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
