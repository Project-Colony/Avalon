use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::core::snapshot::{format_unified_diff, inline_diff, DiffChunk, DiffStats, Snapshot};
use crate::gui::app::Message;
use crate::gui::theme::{self, Theme};

/// Render the snapshot panel (bottom panel)
pub fn view(
    snapshots: &[Snapshot],
    current_content: &str,
    selected_snapshot: Option<usize>,
) -> Element<'static, Message> {
    let header_row = row![
        text("SNAPSHOTS").size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(8),
        text(format!("{} snapshot(s)", snapshots.len()))
            .size(10)
            .color(Theme::TEXT_MUTED),
        Space::with_width(Length::Fill),
        text("Shortcut: Ctrl+5").size(9).color(Theme::TEXT_MUTED),
        Space::with_width(8),
        button(text("\u{f030} Take Snapshot").size(12).color(Theme::TEXT_ACCENT),)
            .on_press(Message::CreateSnapshot)
            .padding(Padding::from([4, 12])),
    ];

    // Snapshot list
    let mut list = column![].spacing(4);

    if snapshots.is_empty() {
        list = list.push(
            text("No snapshots yet. Click 'Take Snapshot' to save the current state of the document.")
                .size(12)
                .color(Theme::TEXT_MUTED),
        );
        list = list.push(
            text("Snapshots let you compare and restore previous versions of your work.")
                .size(11)
                .color(Theme::TEXT_MUTED),
        );
    }

    for (i, snapshot) in snapshots.iter().enumerate().rev() {
        let date = snapshot.created_at.format("%Y-%m-%d %H:%M").to_string();
        let age = snapshot.age_string();

        let words = format!("{} words", snapshot.word_count);
        // Calculate similarity to current content
        let similarity = snapshot.similarity(current_content);
        let sim_pct = (similarity * 100.0) as usize;
        let sim_color = if sim_pct >= 90 {
            Theme::SUCCESS
        } else if sim_pct >= 50 {
            Theme::WARNING
        } else {
            Theme::ERROR
        };
        let sim_text = format!("{}% sim", sim_pct);

        let is_selected = selected_snapshot == Some(i);
        let text_color = if is_selected {
            Theme::TEXT_ACCENT
        } else {
            Theme::TEXT_PRIMARY
        };

        let selected_marker = if is_selected { "\u{f0da} " } else { "  " };
        let index_text = format!("{}{}.", selected_marker, i + 1);
        let is_latest = i == snapshots.len() - 1;

        let snapshot_row = container(
            row![
                text(index_text)
                    .size(10)
                    .color(Theme::TEXT_MUTED)
                    .width(Length::Fixed(36.0)),
                button(text(snapshot.title.clone()).size(12).color(text_color),)
                    .on_press(Message::SelectSnapshot(i))
                    .padding(Padding::from([2, 4]))
                    .width(Length::FillPortion(3)),
                if is_latest {
                    text("LATEST").size(9).color(Theme::SUCCESS)
                } else {
                    text("").size(9)
                },
                Space::with_width(8),
                text(date).size(10).color(Theme::TEXT_MUTED),
                Space::with_width(4),
                text(format!("({})", age)).size(9).color(Theme::TEXT_MUTED),
                Space::with_width(8),
                text(words).size(10).color(Theme::TEXT_MUTED),
                Space::with_width(4),
                text(sim_text).size(9).color(sim_color),
                Space::with_width(8),
                button(text("\u{f0e2} Restore").size(10).color(Theme::TEXT_ACCENT),)
                    .on_press(Message::RestoreSnapshot(i))
                    .padding(Padding::from([2, 6])),
                Space::with_width(4),
                button(text("\u{f07e} Diff").size(10).color(Theme::WARNING),)
                    .on_press(Message::CompareSnapshot(i))
                    .padding(Padding::from([2, 6])),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center),
        )
        .padding(Padding::from([2, 8]));

        list = list.push(snapshot_row);
    }

    // Diff view (if a snapshot is selected for comparison)
    let diff_view: Element<'static, Message> = if let Some(idx) = selected_snapshot {
        if let Some(snapshot) = snapshots.get(idx) {
            let diff = snapshot.diff_with(current_content);
            let stats = DiffStats::from_chunks(&diff);
            let current_words = current_content.split_whitespace().count();

            let mut diff_col = column![row![
                text(format!("Comparing: \"{}\" vs Current", snapshot.title))
                    .size(11)
                    .color(Theme::TEXT_SECONDARY),
                Space::with_width(Length::Fill),
                text(format!(
                    "Snapshot: {} words | Current: {} words",
                    snapshot.word_count, current_words
                ))
                .size(10)
                .color(Theme::TEXT_MUTED),
            ],]
            .spacing(1);

            // Show unified diff header
            let unified = format_unified_diff(&diff, &snapshot.title, "Current");
            for line in unified.lines().take(3) {
                diff_col = diff_col.push(text(line.to_string()).size(9).color(Theme::TEXT_MUTED));
            }

            let total_chunks = diff.len();
            let mut shown_equal = 0;

            // Collect removed lines for inline diff pairing
            let mut pending_removed: Option<String> = None;

            for chunk in &diff {
                match chunk {
                    DiffChunk::Equal(line) => {
                        pending_removed = None;
                        shown_equal += 1;
                        if shown_equal <= 2 || total_chunks <= 30 {
                            diff_col = diff_col.push(text(format!("  {}", line)).size(10).color(Theme::TEXT_MUTED));
                        } else if shown_equal == 3 {
                            diff_col = diff_col.push(text("  ...").size(10).color(Theme::TEXT_MUTED));
                        }
                    }
                    DiffChunk::Added(line) => {
                        shown_equal = 0;
                        // Show inline diff if we have a paired removed line
                        if let Some(ref old_line) = pending_removed {
                            let inline_chunks = inline_diff(old_line, line);
                            let marked: String = inline_chunks.iter().map(|c| c.to_marked_string()).collect();
                            diff_col = diff_col.push(text(format!("~ {}", marked)).size(10).color(Theme::WARNING));
                            pending_removed = None;
                        } else {
                            diff_col = diff_col.push(text(format!("+ {}", line)).size(10).color(Theme::SUCCESS));
                        }
                    }
                    DiffChunk::Removed(line) => {
                        shown_equal = 0;
                        // If we already have a pending removed, flush it
                        if let Some(ref old) = pending_removed {
                            diff_col = diff_col.push(text(format!("- {}", old)).size(10).color(Theme::ERROR));
                        }
                        pending_removed = Some(line.clone());
                    }
                }
            }
            // Flush any remaining pending removed line
            if let Some(ref old) = pending_removed {
                diff_col = diff_col.push(text(format!("- {}", old)).size(10).color(Theme::ERROR));
            }

            diff_col = diff_col.push(row![
                text(stats.summary()).size(10).color(Theme::TEXT_SECONDARY),
                Space::with_width(8),
                text(format!("{:.0}% changed", stats.change_percentage()))
                    .size(10)
                    .color(if stats.change_percentage() > 50.0 {
                        Theme::WARNING
                    } else {
                        Theme::TEXT_MUTED
                    }),
            ]);

            scrollable(diff_col).height(Length::Fixed(80.0)).into()
        } else {
            Space::with_height(0).into()
        }
    } else {
        Space::with_height(0).into()
    };

    let content = column![
        header_row,
        Space::with_height(4),
        scrollable(list).height(Length::Fixed(100.0)),
        Space::with_height(4),
        diff_view,
    ]
    .padding(Padding::from([8, 12]));

    container(content).style(theme::panel_style).width(Length::Fill).into()
}
