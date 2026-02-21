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

        let bar_width: usize = 30;
        let filled = ((pct / 100.0) * bar_width as f64) as usize;
        let empty = bar_width.saturating_sub(filled);
        let bar = format!("[{}{}] {:.1}%", "#".repeat(filled), "-".repeat(empty), pct);

        let progress_color = if pct >= 100.0 {
            Theme::SUCCESS
        } else if pct >= 50.0 {
            Theme::WARNING
        } else {
            Theme::TEXT_SECONDARY
        };

        column![
            row![
                text("Manuscript Target:").size(12).color(Theme::TEXT_MUTED),
                Space::with_width(8),
                text(format!("{} / {} words", data.current_words, target))
                    .size(12)
                    .color(Theme::TEXT_PRIMARY),
            ],
            text(bar).size(12).color(progress_color),
        ]
        .spacing(2)
        .into()
    } else {
        text("No project target set. Set one in Project Settings (Ctrl+,)")
            .size(12)
            .color(Theme::TEXT_MUTED)
            .into()
    };

    // Deadline info
    let deadline_info: Element<'static, Message> = if !data.deadline.is_empty() {
        let deadline_row = row![
            text("Deadline:").size(11).color(Theme::TEXT_MUTED),
            Space::with_width(4),
            text(data.deadline.clone()).size(11).color(Theme::TEXT_PRIMARY),
        ];

        let pace_info: Element<'static, Message> = match (data.days_remaining, data.words_per_day_needed) {
            (Some(days), Some(wpd)) if days > 0 => {
                row![
                    text(format!("{} days remaining", days)).size(11).color(Theme::TEXT_SECONDARY),
                    Space::with_width(8),
                    text(format!("{} words/day needed", wpd)).size(11).color(Theme::WARNING),
                ]
                .into()
            }
            (Some(days), _) if days <= 0 => {
                text("Deadline has passed!").size(11).color(Theme::ERROR).into()
            }
            _ => Space::with_height(0).into(),
        };

        column![deadline_row, pace_info].spacing(2).into()
    } else {
        Space::with_height(0).into()
    };

    // Deadline input
    let deadline_input_row = row![
        text("Deadline (YYYY-MM-DD):").size(11).color(Theme::TEXT_MUTED),
        Space::with_width(4),
        text_input("2025-12-31", &data.deadline)
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
        Space::with_height(8),
        deadline_info,
        Space::with_height(4),
        deadline_input_row,
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
