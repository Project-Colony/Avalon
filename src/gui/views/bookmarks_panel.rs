use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};
use uuid::Uuid;

use crate::core::bookmark::BookmarkList;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the bookmarks panel (bottom panel)
pub fn view(bookmarks: &BookmarkList) -> Element<'static, Message> {
    let header = row![
        text("BOOKMARKS").size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        text(format!("{} bookmark(s)", bookmarks.bookmarks.len()))
            .size(10)
            .color(Theme::TEXT_MUTED),
    ];

    let mut list = column![].spacing(2);

    if bookmarks.bookmarks.is_empty() {
        list = list.push(
            text("No bookmarks. Use the star icon in the inspector to bookmark items.")
                .size(11)
                .color(Theme::TEXT_MUTED),
        );
    }

    for bm in &bookmarks.bookmarks {
        let item_id = bm.item_id;
        let bm_row = row![
            button(
                text(bm.name.clone()).size(12).color(Theme::TEXT_PRIMARY),
            )
            .on_press(Message::SelectBinderItem(item_id))
            .padding(Padding::from([2, 6])),
            Space::with_width(Length::Fill),
            text(bm.created_at.format("%Y-%m-%d").to_string())
                .size(10)
                .color(Theme::TEXT_MUTED),
            Space::with_width(4),
            button(
                text("x").size(10).color(Theme::ERROR),
            )
            .on_press(Message::ToggleBookmark(item_id))
            .padding(Padding::from([1, 4])),
        ]
        .align_y(iced::Alignment::Center);

        list = list.push(bm_row);
    }

    let content = column![
        header,
        Space::with_height(4),
        scrollable(list).height(Length::Fixed(120.0)),
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
