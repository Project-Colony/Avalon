use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length, Padding};

use crate::core::binder::BinderItem;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the corkboard view — index cards on a cork background
pub fn view(items: &[&BinderItem], parent_title: &str) -> Element<'static, Message> {
    let header = container(
        text(format!("Corkboard: {}", parent_title))
            .size(14)
            .color(Theme::TEXT_SECONDARY),
    )
    .padding(Padding::from([8, 16]));

    if items.is_empty() {
        let empty_msg = text("Select a folder to see its cards, or switch to Editor view.")
            .size(13)
            .color(Theme::TEXT_MUTED);

        return container(
            column![header, Space::with_height(40), container(empty_msg).padding(20)]
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    }

    // Build rows of cards (3 per row)
    let mut grid = column![].spacing(12);

    for chunk in items.chunks(3) {
        let mut r = row![].spacing(12);
        for item in chunk {
            r = r.push(render_card(item));
        }
        // Pad remaining slots
        for _ in chunk.len()..3 {
            r = r.push(Space::with_width(220));
        }
        grid = grid.push(r);
    }

    let content = column![
        header,
        scrollable(
            container(grid)
                .padding(20)
                .width(Length::Fill)
        )
        .height(Length::Fill),
    ];

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Render a single index card with editable synopsis
fn render_card(item: &BinderItem) -> Element<'static, Message> {
    let title = item.title.clone();
    let id = item.id;

    let word_count = item.document.as_ref()
        .map(|d| format!("{} words", d.word_count()))
        .unwrap_or_default();

    let status_text = item.metadata.status.as_ref()
        .map(|s| s.name.clone())
        .unwrap_or_default();

    // Card title bar
    let title_bar = container(
        text(title).size(13).color(Theme::CARD_TEXT),
    )
    .padding(Padding::from([6, 8]))
    .width(Length::Fill);

    // Editable synopsis
    let synopsis_input = text_input("Write a synopsis...", &item.synopsis)
        .on_input(move |val| Message::UpdateSynopsis(id, val))
        .size(12)
        .padding(6)
        .width(Length::Fill);

    let synopsis_area = container(synopsis_input)
        .padding(Padding::from([4, 4]))
        .width(Length::Fill)
        .height(Length::Fixed(70.0));

    // Footer with status and word count
    let footer = container(
        row![
            text(word_count).size(10).color(Theme::TEXT_MUTED),
            Space::with_width(Length::Fill),
            text(status_text).size(10).color(Theme::TEXT_MUTED),
        ]
    )
    .padding(Padding::from([4, 8]));

    // Label color indicator
    let label_indicator: Element<'static, Message> = if let Some(ref lbl) = item.metadata.label {
        let color = lbl.color.to_iced_color();
        text(format!(" {} ", lbl.name)).size(9).color(color).into()
    } else {
        Space::with_height(0).into()
    };

    let card_content = column![
        title_bar,
        synopsis_area,
        label_indicator,
        footer,
    ];

    let card_btn = button(card_content)
        .on_press(Message::SelectBinderItem(id))
        .padding(0)
        .width(Length::Fixed(220.0));

    card_btn.into()
}
