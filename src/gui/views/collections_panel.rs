use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length, Padding};
use uuid::Uuid;

use crate::core::collection::{Collection, CollectionKind};
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
    let manual_count = data
        .collections
        .iter()
        .filter(|c| matches!(c.kind, CollectionKind::Manual))
        .count();
    let smart_count = data
        .collections
        .iter()
        .filter(|c| matches!(c.kind, CollectionKind::Search { .. }))
        .count();
    let total_items: usize = data.collections.iter().map(|c| c.item_ids.len()).sum();

    let header = row![
        text("COLLECTIONS")
            .size(11)
            .color(Theme::TEXT_SECONDARY),
        Space::with_width(8),
        text(format!(
            "{} manual, {} smart | {} total items",
            manual_count, smart_count, total_items
        ))
        .size(10)
        .color(Theme::TEXT_MUTED),
        Space::with_width(Length::Fill),
        button(
            text("Refresh Smart").size(10).color(Theme::TEXT_ACCENT),
        )
        .on_press(Message::RefreshSmartCollections)
        .padding(Padding::from([2, 6])),
    ];

    // New collection input
    let new_input = text_input("New collection name...", &data.new_collection_name)
        .on_input(|val| Message::CollectionNameInput(val))
        .size(12)
        .padding(4)
        .width(Length::FillPortion(3));

    let add_btn = button(
        text("+ Add").size(11).color(Theme::TEXT_ACCENT),
    )
    .on_press(Message::CreateCollection)
    .padding(Padding::from([4, 10]));

    let input_row = row![new_input, Space::with_width(4), add_btn]
        .align_y(iced::Alignment::Center);

    // Collection list
    let mut list = column![].spacing(2);

    for (idx, coll) in data.collections.iter().enumerate() {
        let is_selected = data.selected_collection == Some(coll.id);
        let name_color = if is_selected {
            Theme::TEXT_ACCENT
        } else {
            Theme::TEXT_PRIMARY
        };

        let (kind_icon, kind_label) = match &coll.kind {
            CollectionKind::Manual => ("\u{2630}", "Manual"),
            CollectionKind::Search { query, .. } => {
                if query.is_empty() {
                    ("\u{2606}", "Smart")
                } else {
                    ("\u{2605}", "Smart")
                }
            }
        };

        let selected_marker = if is_selected { "\u{25B6} " } else { "  " };
        let index_text = format!("{}{}.", selected_marker, idx + 1);

        let coll_id = coll.id;
        let coll_row = row![
            text(index_text)
                .size(10)
                .color(Theme::TEXT_MUTED)
                .width(Length::Fixed(36.0)),
            text(kind_icon).size(11).color(Theme::TEXT_MUTED),
            Space::with_width(4),
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
                text("\u{2715}").size(10).color(Theme::ERROR),
            )
            .on_press(Message::DeleteCollection(coll_id))
            .padding(Padding::from([1, 4])),
        ]
        .align_y(iced::Alignment::Center);

        list = list.push(coll_row);
    }

    if data.collections.is_empty() {
        list = list.push(
            text("No collections yet. Create one above, or use Search > Add to Collection.")
                .size(11)
                .color(Theme::TEXT_MUTED),
        );
    }

    let footer = row![
        text("Tip: Drag binder items into a manual collection")
            .size(9)
            .color(Theme::TEXT_MUTED),
        Space::with_width(Length::Fill),
        text("Shortcut: Ctrl+Shift+C")
            .size(9)
            .color(Theme::TEXT_MUTED),
    ];

    let content = column![
        header,
        Space::with_height(4),
        input_row,
        Space::with_height(6),
        scrollable(list).height(Length::Fixed(100.0)),
        Space::with_height(4),
        footer,
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
