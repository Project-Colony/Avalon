use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Length, Padding};

use crate::core::timer::{TimerPreset, TimerState, WritingTimer};
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the writing timer panel (bottom panel)
pub fn view(timer: &WritingTimer) -> Element<'static, Message> {
    let header = text("WRITING TIMER")
        .size(11)
        .color(Theme::TEXT_SECONDARY);

    // Timer display
    let remaining = timer.remaining_display();
    let elapsed = timer.elapsed_display();

    let timer_display = match timer.state() {
        TimerState::Running => {
            text(remaining)
                .size(28)
                .color(Theme::TEXT_PRIMARY)
        }
        TimerState::Paused(_) => {
            text(remaining)
                .size(28)
                .color(Theme::WARNING)
        }
        TimerState::Completed => {
            text("00:00")
                .size(28)
                .color(Theme::SUCCESS)
        }
        TimerState::Idle => {
            text(timer.preset.label())
                .size(20)
                .color(Theme::TEXT_MUTED)
        }
    };

    // Status indicator
    let status = match timer.state() {
        TimerState::Running => text("Running").size(10).color(Theme::SUCCESS),
        TimerState::Paused(_) => text("Paused").size(10).color(Theme::WARNING),
        TimerState::Completed => text("Completed!").size(10).color(Theme::SUCCESS),
        TimerState::Idle => text("Ready").size(10).color(Theme::TEXT_MUTED),
    };

    // Progress bar
    let progress: Element<'static, Message> = if timer.is_active() || *timer.state() == TimerState::Completed {
        let pct = timer.progress();
        let bar_width: usize = 25;
        let filled = (pct * bar_width as f64) as usize;
        let empty = bar_width.saturating_sub(filled);
        let bar_color = if pct >= 1.0 { Theme::SUCCESS }
        else if pct >= 0.75 { Theme::WARNING }
        else { Theme::TEXT_ACCENT };

        text(format!(
            "{}{} {:.0}%",
            "\u{2588}".repeat(filled),
            "\u{2591}".repeat(empty),
            pct * 100.0
        ))
        .size(11)
        .color(bar_color)
        .into()
    } else {
        Space::with_height(0).into()
    };

    // Controls
    let controls: Element<'static, Message> = match timer.state() {
        TimerState::Idle => {
            row![
                button(
                    text("Start").size(13).color(Theme::SUCCESS),
                )
                .on_press(Message::TimerStart)
                .padding(Padding::from([6, 16])),
            ]
            .into()
        }
        TimerState::Running => {
            row![
                button(
                    text("Pause").size(13).color(Theme::WARNING),
                )
                .on_press(Message::TimerPause)
                .padding(Padding::from([6, 16])),
                Space::with_width(8),
                button(
                    text("Stop").size(12).color(Theme::TEXT_MUTED),
                )
                .on_press(Message::TimerStop)
                .padding(Padding::from([4, 12])),
            ]
            .into()
        }
        TimerState::Paused(_) => {
            row![
                button(
                    text("Resume").size(13).color(Theme::SUCCESS),
                )
                .on_press(Message::TimerResume)
                .padding(Padding::from([6, 16])),
                Space::with_width(8),
                button(
                    text("Stop").size(12).color(Theme::TEXT_MUTED),
                )
                .on_press(Message::TimerStop)
                .padding(Padding::from([4, 12])),
            ]
            .into()
        }
        TimerState::Completed => {
            row![
                button(
                    text("Reset").size(13).color(Theme::TEXT_SECONDARY),
                )
                .on_press(Message::TimerReset)
                .padding(Padding::from([6, 16])),
            ]
            .into()
        }
    };

    // Preset selection (only when idle)
    let presets: Element<'static, Message> = if *timer.state() == TimerState::Idle {
        row![
            button(text("Sprint (10m)").size(10).color(
                if timer.preset == TimerPreset::Sprint { Theme::TEXT_ACCENT } else { Theme::TEXT_MUTED }
            ))
            .on_press(Message::TimerSetPreset("sprint".to_string()))
            .padding(Padding::from([3, 8])),
            button(text("Pomodoro (25m)").size(10).color(
                if timer.preset == TimerPreset::Pomodoro { Theme::TEXT_ACCENT } else { Theme::TEXT_MUTED }
            ))
            .on_press(Message::TimerSetPreset("pomodoro".to_string()))
            .padding(Padding::from([3, 8])),
            button(text("Long (45m)").size(10).color(
                if timer.preset == TimerPreset::LongSession { Theme::TEXT_ACCENT } else { Theme::TEXT_MUTED }
            ))
            .on_press(Message::TimerSetPreset("long".to_string()))
            .padding(Padding::from([3, 8])),
            button(text("Hour (60m)").size(10).color(
                if timer.preset == TimerPreset::HourSession { Theme::TEXT_ACCENT } else { Theme::TEXT_MUTED }
            ))
            .on_press(Message::TimerSetPreset("hour".to_string()))
            .padding(Padding::from([3, 8])),
        ]
        .spacing(4)
        .into()
    } else {
        Space::with_height(0).into()
    };

    // Session summary
    let summary = text(timer.summary())
        .size(10)
        .color(Theme::TEXT_MUTED);

    // Elapsed time display when running
    let elapsed_display: Element<'static, Message> = if timer.is_active() {
        text(format!("Elapsed: {}", elapsed))
            .size(11)
            .color(Theme::TEXT_SECONDARY)
            .into()
    } else {
        Space::with_height(0).into()
    };

    let content = row![
        column![
            header,
            Space::with_height(4),
            timer_display,
            Space::with_height(2),
            status,
            Space::with_height(4),
            progress,
            Space::with_height(4),
            controls,
        ]
        .spacing(2)
        .width(Length::Fixed(220.0)),
        Space::with_width(20),
        column![
            presets,
            Space::with_height(8),
            elapsed_display,
            Space::with_height(4),
            summary,
        ]
        .spacing(4),
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
