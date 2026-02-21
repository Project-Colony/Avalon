use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Data for the writing goals panel
pub struct WritingGoalsData {
    pub daily_goal: usize,
    pub daily_goal_text: String,
    pub weekly_goal: usize,
    pub weekly_goal_text: String,
    pub words_today: i64,
    pub words_this_week: i64,
    pub streak: usize,
    pub avg_daily: f64,
    pub days_this_week: usize,
}

/// Render the writing goals panel
pub fn view(data: &WritingGoalsData) -> Element<'static, Message> {
    let header = text("WRITING GOALS")
        .size(11)
        .color(Theme::TEXT_SECONDARY);

    // Daily goal
    let daily_label = text("Daily Goal").size(11).color(Theme::TEXT_MUTED);
    let daily_input = text_input("e.g. 1000", &data.daily_goal_text)
        .on_input(|val| Message::SetDailyGoal(val))
        .size(12)
        .padding(4)
        .width(Length::Fixed(100.0));

    let daily_progress = if data.daily_goal > 0 {
        let pct = (data.words_today as f64 / data.daily_goal as f64 * 100.0).min(100.0);
        let color = if pct >= 100.0 { Theme::SUCCESS }
        else if pct >= 50.0 { Theme::WARNING }
        else { Theme::TEXT_SECONDARY };
        let bar = progress_bar(pct);
        let remaining = (data.daily_goal as i64 - data.words_today).max(0);
        column![
            text(format!("{}/{} words ({:.0}%)", data.words_today, data.daily_goal, pct))
                .size(11).color(color),
            text(bar).size(10).color(color),
            if pct >= 100.0 {
                text("Goal reached!").size(9).color(Theme::SUCCESS)
            } else {
                text(format!("{} words remaining", remaining)).size(9).color(Theme::TEXT_MUTED)
            },
        ].spacing(1)
    } else {
        column![
            text(format!("{} words today", data.words_today))
                .size(11).color(Theme::TEXT_SECONDARY),
        ]
    };

    // Weekly goal
    let weekly_label = text("Weekly Goal").size(11).color(Theme::TEXT_MUTED);
    let weekly_input = text_input("e.g. 5000", &data.weekly_goal_text)
        .on_input(|val| Message::SetWeeklyGoal(val))
        .size(12)
        .padding(4)
        .width(Length::Fixed(100.0));

    let weekly_progress = if data.weekly_goal > 0 {
        let pct = (data.words_this_week as f64 / data.weekly_goal as f64 * 100.0).min(100.0);
        let color = if pct >= 100.0 { Theme::SUCCESS }
        else if pct >= 50.0 { Theme::WARNING }
        else { Theme::TEXT_SECONDARY };
        let bar = progress_bar(pct);
        let days_left = 7_usize.saturating_sub(data.days_this_week);
        let remaining = (data.weekly_goal as i64 - data.words_this_week).max(0);
        let daily_needed = if days_left > 0 { remaining / days_left as i64 } else { remaining };
        column![
            text(format!("{}/{} words ({:.0}%)", data.words_this_week, data.weekly_goal, pct))
                .size(11).color(color),
            text(bar).size(10).color(color),
            if pct >= 100.0 {
                text("Weekly goal reached!").size(9).color(Theme::SUCCESS)
            } else {
                text(format!("{} needed | ~{}/day for {} days left", remaining, daily_needed, days_left))
                    .size(9).color(Theme::TEXT_MUTED)
            },
        ].spacing(1)
    } else {
        column![
            text(format!("{} words this week", data.words_this_week))
                .size(11).color(Theme::TEXT_SECONDARY),
        ]
    };

    // Streak visualization
    let streak_vis = streak_display(data.streak);
    let streak_color = if data.streak >= 7 { Theme::SUCCESS }
        else if data.streak >= 3 { Theme::WARNING }
        else { Theme::TEXT_SECONDARY };

    // Stats
    let stats = row![
        stat_item("Streak", &format!("{} days", data.streak)),
        stat_item("Avg Daily", &format!("{:.0} words", data.avg_daily)),
        stat_item("Days/Wk", &format!("{}/7", data.days_this_week)),
    ]
    .spacing(16);

    let streak_row = row![
        text("Streak: ").size(10).color(Theme::TEXT_MUTED),
        text(streak_vis).size(10).color(streak_color),
        Space::with_width(8),
        if data.streak >= 7 {
            text("On fire!").size(10).color(Theme::SUCCESS)
        } else if data.streak >= 3 {
            text("Keep going!").size(10).color(Theme::WARNING)
        } else {
            text("Start writing!").size(10).color(Theme::TEXT_MUTED)
        },
    ]
    .align_y(iced::Alignment::Center);

    let reset_btn = button(
        text("Reset Goals").size(10).color(Theme::TEXT_MUTED),
    )
    .on_press(Message::ResetGoals)
    .padding(Padding::from([2, 6]));

    let content = column![
        header,
        Space::with_height(4),
        row![daily_label, Space::with_width(8), daily_input].align_y(iced::Alignment::Center),
        daily_progress,
        Space::with_height(6),
        row![weekly_label, Space::with_width(8), weekly_input].align_y(iced::Alignment::Center),
        weekly_progress,
        Space::with_height(6),
        streak_row,
        Space::with_height(4),
        stats,
        Space::with_height(4),
        reset_btn,
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}

fn stat_item(label: &str, value: &str) -> Element<'static, Message> {
    column![
        text(label.to_string()).size(10).color(Theme::TEXT_MUTED),
        text(value.to_string()).size(12).color(Theme::TEXT_PRIMARY),
    ]
    .spacing(1)
    .into()
}

fn progress_bar(pct: f64) -> String {
    let total: usize = 20;
    let filled = ((pct / 100.0) * total as f64).round() as usize;
    let empty = total.saturating_sub(filled);
    format!("[{}{}]", "\u{2588}".repeat(filled), "\u{2591}".repeat(empty))
}

fn streak_display(streak: usize) -> String {
    let max_display = 14;
    let show = streak.min(max_display);
    let filled = "\u{25CF}".repeat(show); // filled circles
    let empty = "\u{25CB}".repeat(max_display.saturating_sub(show)); // empty circles
    if streak > max_display {
        format!("{}{} +{}", filled, empty, streak - max_display)
    } else {
        format!("{}{}", filled, empty)
    }
}
