use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length, Padding};
use uuid::Uuid;

use crate::core::collection::Collection;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Data needed for the collections panel
pub struct CollectionsData {
    pub collections: Vec<Collection>,
    pub selected_collection: Option<Uuid>,
    pub new_collection_name: String,
}

/// Render the collections bottom panel
pub fn view(data: &CollectionsData) -> Element<'static, Message> {
    let header = row![
        text("COLLECTIONS").size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        button(
            text("Refresh Smart").size(10).color(Theme::TEXT_ACCENT),
        )
        .on_press(Message::RefreshSmartCollections)
        .padding(Padding::from([2, 6])),
        Space::with_width(4),
        text(format!("{} collection(s)", data.collections.len()))
            .size(10)
            .color(Theme::TEXT_MUTED),
    ];

    // New collection input
    let new_input = text_input("New collection name...", &data.new_collection_name)
        .on_input(|val| Message::CollectionNameInput(val))
        .size(12)
        .padding(4)
        .width(Length::FillPortion(3));

    let add_btn = button(
        text("Add").size(11).color(Theme::TEXT_ACCENT),
    )
    .on_press(Message::CreateCollection)
    .padding(Padding::from([4, 10]));

    let input_row = row![new_input, Space::with_width(4), add_btn]
        .align_y(iced::Alignment::Center);

    // Collection list
    let mut list = column![].spacing(2);
    for coll in &data.collections {
        let is_selected = data.selected_collection == Some(coll.id);
        let name_color = if is_selected { Theme::TEXT_ACCENT } else { Theme::TEXT_PRIMARY };

        let kind_label = match &coll.kind {
            crate::core::collection::CollectionKind::Manual => "Manual",
            crate::core::collection::CollectionKind::Search { .. } => "Search",
        };

        let coll_id = coll.id;
        let coll_row = row![
            button(
                text(coll.name.clone()).size(12).color(name_color),
            )
            .on_press(Message::SelectCollection(coll_id))
            .padding(Padding::from([2, 6])),
            Space::with_width(Length::Fill),
            text(format!("{} ({} items)", kind_label, coll.item_ids.len()))
                .size(10)
                .color(Theme::TEXT_MUTED),
            Space::with_width(4),
            button(
                text("x").size(10).color(Theme::ERROR),
            )
            .on_press(Message::DeleteCollection(coll_id))
            .padding(Padding::from([1, 4])),
        ]
        .align_y(iced::Alignment::Center);

        list = list.push(coll_row);
    }

    if data.collections.is_empty() {
        list = list.push(
            text("No collections yet. Create one above.")
                .size(11)
                .color(Theme::TEXT_MUTED),
        );
    }

    let content = column![
        header,
        Space::with_height(4),
        input_row,
        Space::with_height(6),
        scrollable(list).height(Length::Fixed(100.0)),
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
