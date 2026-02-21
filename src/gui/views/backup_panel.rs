use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::core::backup::BackupEntry;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the backup management panel
pub fn view(backups: &[BackupEntry], project_name: &str) -> Element<'static, Message> {
    let header = row![
        text("BACKUP MANAGEMENT").size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        button(
            text("Create Backup Now").size(11).color(Theme::TEXT_ACCENT),
        )
        .on_press(Message::CreateBackup)
        .padding(Padding::from([4, 10])),
    ];

    let subtitle = text(format!("Project: {}", project_name))
        .size(10)
        .color(Theme::TEXT_MUTED);

    let mut list = column![].spacing(2);

    if backups.is_empty() {
        list = list.push(
            text("No backups found. Click 'Create Backup Now' to save a backup.")
                .size(11)
                .color(Theme::TEXT_MUTED),
        );
    }

    for entry in backups.iter().take(10) {
        let path = entry.path.clone();
        let entry_row = container(
            row![
                text(entry.display_timestamp()).size(11).color(Theme::TEXT_PRIMARY)
                    .width(Length::FillPortion(3)),
                text(entry.display_size()).size(10).color(Theme::TEXT_MUTED)
                    .width(Length::FillPortion(1)),
                button(
                    text("Restore").size(10).color(Theme::WARNING),
                )
                .on_press(Message::RestoreBackup(path))
                .padding(Padding::from([2, 6])),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center)
        )
        .padding(Padding::from([2, 8]));

        list = list.push(entry_row);
    }

    let note = text("Backups are stored in ~/Scrinever Backups/. Last 20 kept automatically.")
        .size(9)
        .color(Theme::TEXT_MUTED);

    let content = column![
        header,
        subtitle,
        Space::with_height(4),
        scrollable(list).height(Length::Fixed(120.0)),
        Space::with_height(4),
        note,
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
