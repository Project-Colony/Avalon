use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Data for the writing session panel
pub struct SessionData {
    pub is_active: bool,
    pub elapsed_seconds: u64,
    pub words_written: i64,
    pub words_per_minute: f64,
    pub session_goal: usize,
    pub session_goal_text: String,
}

/// Render the writing session panel (bottom panel)
pub fn view(data: &SessionData) -> Element<'static, Message> {
    let status_indicator = if data.is_active {
        "\u{23F1} REC"
    } else {
        "\u{23F8} PAUSED"
    };
    let status_color = if data.is_active { Theme::SUCCESS } else { Theme::TEXT_MUTED };

    let header = row![
        text("WRITING SESSION")
            .size(11)
            .color(Theme::TEXT_SECONDARY),
        Space::with_width(8),
        text(status_indicator).size(10).color(status_color),
    ];

    // Timer display
    let hours = data.elapsed_seconds / 3600;
    let minutes = (data.elapsed_seconds % 3600) / 60;
    let seconds = data.elapsed_seconds % 60;
    let timer_text = if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    };

    let timer_display = text(timer_text)
        .size(28)
        .color(if data.is_active { Theme::TEXT_PRIMARY } else { Theme::TEXT_MUTED });

    // Start/Stop button
    let toggle_btn = if data.is_active {
        button(
            text("\u{23F8}  Pause  ").size(13).color(Theme::WARNING),
        )
        .on_press(Message::SessionToggle)
        .padding(Padding::from([6, 16]))
    } else {
        button(
            text("\u{25B6}  Start  ").size(13).color(Theme::SUCCESS),
        )
        .on_press(Message::SessionToggle)
        .padding(Padding::from([6, 16]))
    };

    let reset_btn = button(
        text("\u{21BB} Reset").size(12).color(Theme::TEXT_MUTED),
    )
    .on_press(Message::SessionReset)
    .padding(Padding::from([4, 12]));

    let controls = row![
        toggle_btn,
        Space::with_width(8),
        reset_btn,
    ]
    .align_y(iced::Alignment::Center);

    // Stats
    let words_color = if data.words_written >= 0 { Theme::SUCCESS } else { Theme::ERROR };
    let words_text = if data.words_written >= 0 {
        format!("+{} words", data.words_written)
    } else {
        format!("{} words", data.words_written)
    };

    let wpm_text = format!("{:.1} wpm", data.words_per_minute);

    // Estimated pages
    let pages = if data.words_written > 0 {
        format!("~{:.1} pages", data.words_written as f64 / 250.0)
    } else {
        String::from("0 pages")
    };

    // Time per word estimate
    let time_per_word = if data.words_written > 0 && data.elapsed_seconds > 0 {
        let spw = data.elapsed_seconds as f64 / data.words_written as f64;
        format!("{:.1}s/word", spw)
    } else {
        String::new()
    };

    let stats_row = row![
        text(words_text).size(14).color(words_color),
        Space::with_width(16),
        text(wpm_text).size(14).color(Theme::TEXT_SECONDARY),
        Space::with_width(16),
        text(pages).size(12).color(Theme::TEXT_MUTED),
        Space::with_width(16),
        text(time_per_word).size(11).color(Theme::TEXT_MUTED),
    ];

    // Session goal
    let goal_label = text("Session Goal:").size(11).color(Theme::TEXT_MUTED);
    let goal_input = text_input("e.g. 1000", &data.session_goal_text)
        .on_input(|val| Message::SessionSetGoal(val))
        .size(12)
        .padding(4)
        .width(Length::Fixed(100.0));

    let goal_progress: Element<'static, Message> = if data.session_goal > 0 {
        let pct = (data.words_written.max(0) as f64 / data.session_goal as f64 * 100.0).min(100.0);
        let remaining = (data.session_goal as i64 - data.words_written).max(0);

        let progress_color = if pct >= 100.0 {
            Theme::SUCCESS
        } else if pct >= 75.0 {
            iced::Color::from_rgb(0.3, 0.7, 0.3)
        } else if pct >= 50.0 {
            Theme::WARNING
        } else {
            Theme::TEXT_SECONDARY
        };

        let bar_width: usize = 20;
        let filled = ((pct / 100.0) * bar_width as f64) as usize;
        let empty = bar_width.saturating_sub(filled);
        let bar = format!(
            "{}{} {:.0}%",
            "\u{2588}".repeat(filled),
            "\u{2591}".repeat(empty),
            pct
        );

        let completion_icon = if pct >= 100.0 { "\u{2713} " } else { "" };

        column![
            text(format!("{}{}", completion_icon, bar)).size(11).color(progress_color),
            text(format!("{} words remaining", remaining))
                .size(10)
                .color(Theme::TEXT_MUTED),
        ]
        .spacing(1)
        .into()
    } else {
        text("(no goal set)")
            .size(10)
            .color(Theme::TEXT_MUTED)
            .into()
    };

    let goal_row = row![
        goal_label,
        Space::with_width(8),
        goal_input,
        Space::with_width(12),
        goal_progress,
    ]
    .align_y(iced::Alignment::Center);

    // Productivity tip based on elapsed time
    let tip: Element<'static, Message> = if data.is_active && data.elapsed_seconds >= 1500 {
        // 25 minutes (Pomodoro)
        text("\u{2615} Consider a short break! You've been writing for 25+ minutes.")
            .size(10)
            .color(Theme::WARNING)
            .into()
    } else if !data.is_active && data.elapsed_seconds == 0 {
        text("Start a session to track your writing speed and word count.")
            .size(10)
            .color(Theme::TEXT_MUTED)
            .into()
    } else {
        Space::with_height(0).into()
    };

    let content = row![
        column![
            header,
            Space::with_height(4),
            timer_display,
            Space::with_height(4),
            controls,
        ]
        .spacing(2)
        .width(Length::Fixed(200.0)),
        Space::with_width(20),
        column![
            stats_row,
            Space::with_height(6),
            goal_row,
            Space::with_height(4),
            tip,
        ]
        .spacing(4),
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
