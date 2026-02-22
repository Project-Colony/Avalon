use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};
use uuid::Uuid;

use crate::core::bookmark::BookmarkList;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the bookmarks panel (bottom panel)
pub fn view(bookmarks: &BookmarkList) -> Element<'static, Message> {
    let count = bookmarks.bookmarks.len();
    let header = row![
        text("BOOKMARKS").size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(8),
        text("\u{2605}").size(12).color(Theme::WARNING),
        Space::with_width(Length::Fill),
        text(format!("{} bookmark{}", count, if count == 1 { "" } else { "s" }))
            .size(10)
            .color(Theme::TEXT_MUTED),
    ];

    let mut list = column![].spacing(2);

    if bookmarks.bookmarks.is_empty() {
        list = list.push(
            container(
                column![
                    text("No bookmarks yet.").size(12).color(Theme::TEXT_MUTED),
                    Space::with_height(4),
                    text("Use the \u{2605} star icon in the inspector to bookmark documents for quick access.")
                        .size(10)
                        .color(Theme::TEXT_MUTED),
                ]
            ).padding(Padding::from([8, 0]))
        );
    }

    for (i, bm) in bookmarks.bookmarks.iter().enumerate() {
        let item_id = bm.item_id;

        // Alternate row numbering for visual clarity
        let index_str = format!("{}.", i + 1);
        let age = chrono::Utc::now().signed_duration_since(bm.created_at);
        let age_str = if age.num_days() > 0 {
            format!("{}d ago", age.num_days())
        } else if age.num_hours() > 0 {
            format!("{}h ago", age.num_hours())
        } else {
            "just now".to_string()
        };

        // Color indicator
        let color_indicator = if let Some(ref color) = bm.color {
            text(format!("[{}]", color)).size(9).color(Theme::TEXT_ACCENT)
        } else {
            text("").size(9)
        };

        let bm_row = row![
            text(index_str).size(10).color(Theme::TEXT_MUTED).width(Length::Fixed(20.0)),
            text("\u{2605}").size(10).color(Theme::WARNING),
            Space::with_width(4),
            color_indicator,
            button(
                text(bm.name.clone()).size(12).color(Theme::TEXT_PRIMARY),
            )
            .on_press(Message::SelectBinderItem(item_id))
            .padding(Padding::from([2, 6])),
            Space::with_width(Length::Fill),
            text(age_str)
                .size(9)
                .color(Theme::TEXT_MUTED),
            Space::with_width(4),
            text(bm.created_at.format("%m-%d").to_string())
                .size(9)
                .color(Theme::TEXT_MUTED),
            Space::with_width(4),
            button(
                text("\u{2715}").size(10).color(Theme::ERROR),
            )
            .on_press(Message::ToggleBookmark(item_id))
            .padding(Padding::from([1, 4])),
        ]
        .align_y(iced::Alignment::Center);

        list = list.push(bm_row);

        // Show note below the bookmark if present
        if let Some(ref note) = bm.note {
            let note_display = if note.len() > 80 {
                format!("  {} ...", &note[..80])
            } else {
                format!("  {}", note)
            };
            list = list.push(
                text(note_display).size(9).color(Theme::TEXT_MUTED)
            );
        }
    }

    // Bookmarks with notes count
    let with_notes = bookmarks.bookmarks.iter().filter(|b| b.note.is_some()).count();
    let note_info = if with_notes > 0 {
        format!(" | {} with notes", with_notes)
    } else {
        String::new()
    };

    let hint = row![
        text("Ctrl+D: bookmark selected item")
            .size(9)
            .color(Theme::TEXT_MUTED),
        Space::with_width(Length::Fill),
        text(format!("Drag to reorder{}", note_info))
            .size(9)
            .color(Theme::TEXT_MUTED),
    ];

    let content = column![
        header,
        Space::with_height(4),
        scrollable(list).height(Length::Fixed(120.0)),
        Space::with_height(2),
        hint,
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
