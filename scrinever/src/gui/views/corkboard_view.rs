use iced::widget::{button, column, container, row, scrollable, text, Space};
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

    // Build rows of cards (3 per row)
    let mut grid = column![].spacing(12);

    for chunk in items.chunks(3) {
        let mut r = row![].spacing(12);
        for item in chunk {
            r = r.push(render_card(item));
        }
        // Pad remaining slots
        for _ in chunk.len()..3 {
            r = r.push(Space::with_width(200));
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

/// Render a single index card
fn render_card(item: &BinderItem) -> Element<'static, Message> {
    let title = item.title.clone();
    let synopsis = if item.synopsis.is_empty() {
        "No synopsis".to_string()
    } else {
        if item.synopsis.len() > 100 {
            format!("{}...", &item.synopsis[..100])
        } else {
            item.synopsis.clone()
        }
    };

    let word_count = item.document.as_ref()
        .map(|d| format!("{} words", d.word_count()))
        .unwrap_or_default();

    let status_text = item.metadata.status.as_ref()
        .map(|s| s.name.clone())
        .unwrap_or_default();

    let card_content = column![
        // Title bar
        container(
            text(title).size(13).color(Theme::CARD_TEXT),
        )
        .padding(Padding::from([6, 8]))
        .width(Length::Fill),
        // Synopsis
        container(
            text(synopsis).size(12).color(Theme::CARD_TEXT),
        )
        .padding(Padding::from([8, 8]))
        .width(Length::Fill)
        .height(Length::Fixed(80.0)),
        // Footer
        container(
            row![
                text(word_count).size(10).color(Theme::TEXT_MUTED),
                Space::with_width(Length::Fill),
                text(status_text).size(10).color(Theme::TEXT_MUTED),
            ]
        )
        .padding(Padding::from([4, 8])),
    ];

    let id = item.id;
    button(card_content)
        .on_press(Message::SelectBinderItem(id))
        .padding(0)
        .width(Length::Fixed(200.0))
        .into()
}
