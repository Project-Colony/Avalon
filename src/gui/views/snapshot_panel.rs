use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::core::snapshot::Snapshot;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the snapshot panel (bottom panel)
pub fn view(snapshots: &[Snapshot]) -> Element<'static, Message> {
    let header_row = row![
        text("SNAPSHOTS").size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        button(
            text("Take Snapshot").size(12).color(Theme::TEXT_ACCENT),
        )
        .on_press(Message::CreateSnapshot)
        .padding(Padding::from([4, 12])),
    ];

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
        let preview = if snapshot.content.len() > 80 {
            format!("{}...", &snapshot.content[..80])
        } else {
            snapshot.content.clone()
        };

        let snapshot_row = button(
            column![
                row![
                    text(snapshot.title.clone()).size(12).color(Theme::TEXT_PRIMARY),
                    Space::with_width(Length::Fill),
                    text(date).size(10).color(Theme::TEXT_MUTED),
                    Space::with_width(8),
                    text(words).size(10).color(Theme::TEXT_MUTED),
                ],
                text(preview.replace('\n', " ")).size(11).color(Theme::TEXT_SECONDARY),
            ]
            .spacing(2)
        )
        .on_press(Message::RestoreSnapshot(i))
        .padding(Padding::from([6, 8]))
        .width(Length::Fill);

        list = list.push(snapshot_row);
    }

    let content = column![
        header_row,
        Space::with_height(4),
        scrollable(list).height(Length::Fixed(140.0)),
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
