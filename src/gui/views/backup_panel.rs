use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::core::backup::BackupEntry;
use crate::gui::app::Message;
use crate::gui::theme::{self, Theme};

/// Render the backup management panel
pub fn view(backups: &[BackupEntry], project_name: &str) -> Element<'static, Message> {
    let backup_count = backups.len();
    let total_size: u64 = backups.iter().map(|b| b.size_bytes).sum();
    let total_size_str = crate::core::format_bytes(total_size);

    let header = row![
        text("BACKUP MANAGEMENT").size(11).color(Theme::TEXT_SECONDARY),
        Space::new().width(Length::Fill),
        text(format!(
            "{} backup{} ({})",
            backup_count,
            if backup_count == 1 { "" } else { "s" },
            total_size_str
        ))
        .size(10)
        .color(Theme::TEXT_MUTED),
        Space::new().width(8),
        button(text("\u{f067} Create Backup").size(11).color(Theme::TEXT_ACCENT),)
            .on_press(Message::CreateBackup)
            .padding(Padding::from([4, 10])),
    ];

    let subtitle = row![
        text(format!("Project: {}", project_name))
            .size(10)
            .color(Theme::TEXT_MUTED),
        Space::new().width(Length::Fill),
        text("Auto-backup on save | Last 20 kept")
            .size(9)
            .color(Theme::TEXT_MUTED),
    ];

    let mut list = column![].spacing(2);

    if backups.is_empty() {
        list = list.push(
            container(column![
                text("No backups found.").size(12).color(Theme::TEXT_MUTED),
                Space::new().height(4),
                text("Click 'Create Backup' to save a snapshot of your entire project.")
                    .size(10)
                    .color(Theme::TEXT_MUTED),
            ])
            .padding(Padding::from([8, 0])),
        );
    }

    // Show most recent first with numbering
    for (i, entry) in backups.iter().take(15).enumerate() {
        let path = entry.path.clone();
        let is_latest = i == 0;

        // Parse timestamp string (YYYYMMDD_HHMMSS format) for age calculation
        let age_str = parse_age_from_timestamp(&entry.timestamp);

        let label_color = if is_latest {
            Theme::TEXT_ACCENT
        } else {
            Theme::TEXT_PRIMARY
        };
        let latest_tag: Element<'static, Message> = if is_latest {
            text("LATEST").size(8).color(Theme::SUCCESS).into()
        } else {
            Space::new().width(0).into()
        };

        let entry_row = container(
            row![
                text(format!("{}.", i + 1))
                    .size(10)
                    .color(Theme::TEXT_MUTED)
                    .width(Length::Fixed(22.0)),
                text(entry.display_timestamp())
                    .size(11)
                    .color(label_color)
                    .width(Length::FillPortion(3)),
                latest_tag,
                Space::new().width(4),
                text(age_str)
                    .size(9)
                    .color(Theme::TEXT_MUTED)
                    .width(Length::Fixed(50.0)),
                text(entry.display_size())
                    .size(10)
                    .color(Theme::TEXT_MUTED)
                    .width(Length::Fixed(60.0)),
                button(text("Restore").size(10).color(Theme::WARNING),)
                    .on_press(Message::RestoreBackup(path))
                    .padding(Padding::from([2, 6])),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center),
        )
        .padding(Padding::from([2, 8]));

        list = list.push(entry_row);
    }

    let note = row![
        text("\u{f07b}").size(10),
        Space::new().width(4),
        text(format!("Stored in ~/{}/", crate::core::BACKUPS_DIR_NAME))
            .size(9)
            .color(Theme::TEXT_MUTED),
        Space::new().width(Length::Fill),
        text("Ctrl+Shift+B: quick backup").size(9).color(Theme::TEXT_MUTED),
    ];

    let content = column![
        header,
        subtitle,
        Space::new().height(4),
        scrollable(list).height(Length::Fixed(120.0)),
        Space::new().height(4),
        note,
    ]
    .padding(Padding::from([8, 12]));

    container(content).style(theme::panel_style).width(Length::Fill).into()
}

/// Parse a YYYYMMDD_HHMMSS timestamp string and return age as human readable string
fn parse_age_from_timestamp(ts: &str) -> String {
    if ts.len() < 15 {
        return ts.to_string();
    }
    let year = ts.get(0..4).and_then(|s| s.parse::<i32>().ok()).unwrap_or(2020);
    let month = ts.get(4..6).and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
    let day = ts.get(6..8).and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
    let hour = ts.get(9..11).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    let min = ts.get(11..13).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);

    if let Some(dt) = chrono::NaiveDate::from_ymd_opt(year, month, day).and_then(|d| d.and_hms_opt(hour, min, 0)) {
        let backup_time = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc);
        let age = chrono::Utc::now().signed_duration_since(backup_time);
        if age.num_days() > 0 {
            format!("{}d ago", age.num_days())
        } else if age.num_hours() > 0 {
            format!("{}h ago", age.num_hours())
        } else if age.num_minutes() > 0 {
            format!("{}m ago", age.num_minutes())
        } else {
            "just now".to_string()
        }
    } else {
        ts.to_string()
    }
}
