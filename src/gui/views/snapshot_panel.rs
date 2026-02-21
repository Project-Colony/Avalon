use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::core::snapshot::{DiffChunk, Snapshot};
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the snapshot panel (bottom panel)
pub fn view(snapshots: &[Snapshot], current_content: &str, selected_snapshot: Option<usize>) -> Element<'static, Message> {
    let header_row = row![
        text("SNAPSHOTS").size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        button(
            text("Take Snapshot").size(12).color(Theme::TEXT_ACCENT),
        )
        .on_press(Message::CreateSnapshot)
        .padding(Padding::from([4, 12])),
    ];

    // Snapshot list
    let mut list = column![].spacing(4);

    if snapshots.is_empty() {
        list = list.push(
            text("No snapshots yet. Click 'Take Snapshot' to save the current state.")
                .size(12)
                .color(Theme::TEXT_MUTED),
        );
    }

    for (i, snapshot) in snapshots.iter().enumerate().rev() {
        let date = snapshot.created_at.format("%Y-%m-%d %H:%M").to_string();
        let words = format!("{} words", snapshot.word_count);
        let is_selected = selected_snapshot == Some(i);
        let text_color = if is_selected { Theme::TEXT_ACCENT } else { Theme::TEXT_PRIMARY };

        let snapshot_row = container(
            row![
                button(
                    text(snapshot.title.clone()).size(12).color(text_color),
                )
                .on_press(Message::SelectSnapshot(i))
                .padding(Padding::from([2, 4]))
                .width(Length::FillPortion(3)),
                text(date).size(10).color(Theme::TEXT_MUTED),
                Space::with_width(8),
                text(words).size(10).color(Theme::TEXT_MUTED),
                Space::with_width(8),
                button(
                    text("Restore").size(10).color(Theme::TEXT_ACCENT),
                )
                .on_press(Message::RestoreSnapshot(i))
                .padding(Padding::from([2, 6])),
                Space::with_width(4),
                button(
                    text("Diff").size(10).color(Theme::WARNING),
                )
                .on_press(Message::CompareSnapshot(i))
                .padding(Padding::from([2, 6])),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center)
        )
        .padding(Padding::from([2, 8]));

        list = list.push(snapshot_row);
    }

    // Diff view (if a snapshot is selected for comparison)
    let diff_view: Element<'static, Message> = if let Some(idx) = selected_snapshot {
        if let Some(snapshot) = snapshots.get(idx) {
            let diff = snapshot.diff_with(current_content);
            let mut diff_col = column![
                text(format!("Comparing: \"{}\" vs Current", snapshot.title))
                    .size(11)
                    .color(Theme::TEXT_SECONDARY),
            ].spacing(1);

            let mut added = 0;
            let mut removed = 0;
            let mut unchanged = 0;

            for chunk in &diff {
                match chunk {
                    DiffChunk::Equal(line) => {
                        unchanged += 1;
                        if unchanged <= 3 || diff.len() <= 20 {
                            diff_col = diff_col.push(
                                text(format!("  {}", line))
                                    .size(10)
                                    .color(Theme::TEXT_MUTED),
                            );
                        }
                    }
                    DiffChunk::Added(line) => {
                        added += 1;
                        diff_col = diff_col.push(
                            text(format!("+ {}", line))
                                .size(10)
                                .color(Theme::SUCCESS),
                        );
                    }
                    DiffChunk::Removed(line) => {
                        removed += 1;
                        diff_col = diff_col.push(
                            text(format!("- {}", line))
                                .size(10)
                                .color(Theme::ERROR),
                        );
                    }
                }
            }

            diff_col = diff_col.push(
                text(format!("Summary: +{} added, -{} removed, {} unchanged lines", added, removed, unchanged))
                    .size(10)
                    .color(Theme::TEXT_SECONDARY),
            );

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

    container(content)
        .width(Length::Fill)
        .into()
}
