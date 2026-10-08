use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::core::timer::{TimerPreset, TimerState, WritingTimer};
use crate::gui::app::Message;
use crate::gui::theme::{self, Theme};

/// Render the writing timer panel (bottom panel)
pub fn view(timer: &WritingTimer, _current_word_count: usize) -> Element<'static, Message> {
    let header = row![
        text("WRITING TIMER").size(11).color(Theme::TEXT_SECONDARY),
        Space::new().width(4),
        text("\u{f017}").size(10),
        Space::new().width(Length::Fill),
        text(format!(
            "{} session{} completed",
            timer.completed_count(),
            if timer.completed_count() == 1 { "" } else { "s" }
        ))
        .size(10)
        .color(Theme::TEXT_MUTED),
    ];

    // Timer display
    let remaining = timer.remaining_display();
    let elapsed = timer.elapsed_display();

    let timer_display = match timer.state() {
        TimerState::Running => text(remaining).size(28).color(Theme::TEXT_PRIMARY),
        TimerState::Paused(_) => text(remaining).size(28).color(Theme::WARNING),
        TimerState::Completed => text("\u{f00c} 00:00").size(28).color(Theme::SUCCESS),
        TimerState::Idle => text(timer.preset.label()).size(20).color(Theme::TEXT_MUTED),
    };

    // Status indicator
    let status = match timer.state() {
        TimerState::Running => text("\u{f111} Running").size(10).color(Theme::SUCCESS),
        TimerState::Paused(_) => text("\u{f04c} Paused").size(10).color(Theme::WARNING),
        TimerState::Completed => text("\u{f00c} Completed!").size(10).color(Theme::SUCCESS),
        TimerState::Idle => text("\u{f10c} Ready").size(10).color(Theme::TEXT_MUTED),
    };

    // Progress bar
    let progress: Element<'static, Message> = if timer.is_active() || *timer.state() == TimerState::Completed {
        let pct = timer.progress();
        let bar_width: usize = 25;
        let filled = (pct * bar_width as f64) as usize;
        let empty = bar_width.saturating_sub(filled);
        let bar_color = if pct >= 1.0 {
            Theme::SUCCESS
        } else if pct >= 0.75 {
            Theme::WARNING
        } else {
            Theme::TEXT_ACCENT
        };

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
        Space::new().height(0).into()
    };

    // Controls
    let controls: Element<'static, Message> = match timer.state() {
        TimerState::Idle => row![button(text("\u{f04b} Start").size(13).color(Theme::SUCCESS),)
            .on_press(Message::TimerStart)
            .padding(Padding::from([6, 16])),]
        .into(),
        TimerState::Running => row![
            button(text("\u{f04c} Pause").size(13).color(Theme::WARNING),)
                .on_press(Message::TimerPause)
                .padding(Padding::from([6, 16])),
            Space::new().width(8),
            button(text("\u{f04d} Stop").size(12).color(Theme::TEXT_MUTED),)
                .on_press(Message::TimerStop)
                .padding(Padding::from([4, 12])),
        ]
        .into(),
        TimerState::Paused(_) => row![
            button(text("\u{f04b} Resume").size(13).color(Theme::SUCCESS),)
                .on_press(Message::TimerResume)
                .padding(Padding::from([6, 16])),
            Space::new().width(8),
            button(text("\u{f04d} Stop").size(12).color(Theme::TEXT_MUTED),)
                .on_press(Message::TimerStop)
                .padding(Padding::from([4, 12])),
        ]
        .into(),
        TimerState::Completed => row![button(text("\u{f021} Reset").size(13).color(Theme::TEXT_SECONDARY),)
            .on_press(Message::TimerReset)
            .padding(Padding::from([6, 16])),]
        .into(),
    };

    // Preset selection (only when idle)
    let presets: Element<'static, Message> = if *timer.state() == TimerState::Idle {
        row![
            button(
                text("Sprint (10m)")
                    .size(10)
                    .color(if timer.preset == TimerPreset::Sprint {
                        Theme::TEXT_ACCENT
                    } else {
                        Theme::TEXT_MUTED
                    })
            )
            .on_press(Message::TimerSetPreset("sprint".to_string()))
            .padding(Padding::from([3, 8])),
            button(
                text("Pomodoro (25m)")
                    .size(10)
                    .color(if timer.preset == TimerPreset::Pomodoro {
                        Theme::TEXT_ACCENT
                    } else {
                        Theme::TEXT_MUTED
                    })
            )
            .on_press(Message::TimerSetPreset("pomodoro".to_string()))
            .padding(Padding::from([3, 8])),
            button(
                text("Long (45m)")
                    .size(10)
                    .color(if timer.preset == TimerPreset::LongSession {
                        Theme::TEXT_ACCENT
                    } else {
                        Theme::TEXT_MUTED
                    })
            )
            .on_press(Message::TimerSetPreset("long".to_string()))
            .padding(Padding::from([3, 8])),
            button(
                text("Hour (60m)")
                    .size(10)
                    .color(if timer.preset == TimerPreset::HourSession {
                        Theme::TEXT_ACCENT
                    } else {
                        Theme::TEXT_MUTED
                    })
            )
            .on_press(Message::TimerSetPreset("hour".to_string()))
            .padding(Padding::from([3, 8])),
        ]
        .spacing(4)
        .into()
    } else {
        Space::new().height(0).into()
    };

    // Elapsed time display when running
    let elapsed_display: Element<'static, Message> = if timer.is_active() {
        text(format!("Elapsed: {}", elapsed))
            .size(11)
            .color(Theme::TEXT_SECONDARY)
            .into()
    } else {
        Space::new().height(0).into()
    };

    // Session history (show past sessions with WPM)
    let history_section: Element<'static, Message> = if !timer.sessions().is_empty() {
        let mut history_col = column![text("Session History").size(10).color(Theme::TEXT_ACCENT),].spacing(2);

        for (i, session) in timer.sessions().iter().rev().take(5).enumerate() {
            let duration_mins = session.duration.as_secs() as f64 / 60.0;
            let wpm = session.words_per_minute();
            let status_icon = if session.completed { "\u{f00c}" } else { "\u{f00d}" };
            let status_color = if session.completed {
                Theme::SUCCESS
            } else {
                Theme::TEXT_MUTED
            };

            history_col = history_col.push(row![
                text(format!("{}.", i + 1))
                    .size(9)
                    .color(Theme::TEXT_MUTED)
                    .width(Length::Fixed(16.0)),
                text(status_icon).size(9).color(status_color),
                Space::new().width(4),
                text(session.preset.label().to_string())
                    .size(9)
                    .color(Theme::TEXT_MUTED),
                Space::new().width(4),
                text(format!("{:.1}m", duration_mins))
                    .size(9)
                    .color(Theme::TEXT_SECONDARY),
                Space::new().width(4),
                text(format!("{} words", session.words_written))
                    .size(9)
                    .color(Theme::TEXT_SECONDARY),
                Space::new().width(4),
                text(format!("{:.1} wpm", wpm)).size(9).color(if wpm > 20.0 {
                    Theme::SUCCESS
                } else if wpm > 10.0 {
                    Theme::WARNING
                } else {
                    Theme::TEXT_MUTED
                }),
            ]);
        }

        if timer.sessions().len() > 5 {
            history_col = history_col.push(
                text(format!("... +{} more session(s)", timer.sessions().len() - 5))
                    .size(9)
                    .color(Theme::TEXT_MUTED),
            );
        }

        history_col.into()
    } else {
        text("No completed sessions yet. Start a timer to begin tracking!")
            .size(10)
            .color(Theme::TEXT_MUTED)
            .into()
    };

    // Aggregate stats
    let aggregate_stats: Element<'static, Message> = if timer.completed_count() > 0 {
        let total_mins = timer.total_time().as_secs() as f64 / 60.0;
        row![
            text(format!("Total: {:.0}m", total_mins))
                .size(10)
                .color(Theme::TEXT_MUTED),
            Space::new().width(8),
            text(format!("{} words", timer.total_words()))
                .size(10)
                .color(Theme::TEXT_MUTED),
            Space::new().width(8),
            text(format!("Avg: {:.1} wpm", timer.avg_wpm()))
                .size(10)
                .color(Theme::TEXT_ACCENT),
        ]
        .into()
    } else {
        Space::new().height(0).into()
    };

    let hint = row![
        text("Shortcut: Ctrl+J to toggle timer panel")
            .size(9)
            .color(Theme::TEXT_MUTED),
        Space::new().width(Length::Fill),
        text(timer.summary()).size(9).color(Theme::TEXT_MUTED),
    ];

    let content = row![
        column![
            header,
            Space::new().height(4),
            timer_display,
            Space::new().height(2),
            status,
            Space::new().height(4),
            progress,
            Space::new().height(4),
            controls,
        ]
        .spacing(2)
        .width(Length::Fixed(220.0)),
        Space::new().width(20),
        column![
            presets,
            Space::new().height(4),
            elapsed_display,
            Space::new().height(4),
            scrollable(history_section).height(Length::Fixed(80.0)),
            Space::new().height(4),
            aggregate_stats,
            Space::new().height(2),
            hint,
        ]
        .spacing(2),
    ]
    .padding(Padding::from([8, 12]));

    container(content).style(theme::panel_style).width(Length::Fill).into()
}
