use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length, Padding};

use crate::core::binder::BinderItem;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the corkboard view — index cards on a cork background
pub fn view(items: &[&BinderItem], parent_title: &str) -> Element<'static, Message> {
    let item_count = items.len();
    let header = container(
        row![
            text(format!("Corkboard: {}", parent_title))
                .size(14)
                .color(Theme::TEXT_SECONDARY),
            Space::with_width(Length::Fill),
            text(format!("{} cards", item_count))
                .size(11)
                .color(Theme::TEXT_MUTED),
        ]
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

/// Render a single index card with editable synopsis, status color bar, and label
fn render_card(item: &BinderItem) -> Element<'static, Message> {
    let title = item.title.clone();
    let id = item.id;

    let word_count = item.document.as_ref()
        .map(|d| format!("{} words", d.word_count()))
        .unwrap_or_default();

    let status_text = item.metadata.status.as_ref()
        .map(|s| s.name.clone())
        .unwrap_or_default();

    // Status color indicator (colored dot before status text)
    let status_color = match item.metadata.status.as_ref().map(|s| s.name.as_str()) {
        Some("To Do") => Theme::ERROR,
        Some("First Draft") => Theme::WARNING,
        Some("Revised Draft") => Theme::TEXT_ACCENT,
        Some("Final Draft") => Theme::SUCCESS,
        Some("Done") => Theme::SUCCESS,
        _ => Theme::TEXT_MUTED,
    };

    // Include in compile indicator
    let compile_indicator = if item.include_in_compile {
        text("C").size(9).color(Theme::SUCCESS)
    } else {
        text("C").size(9).color(Theme::TEXT_MUTED)
    };

    // Card title bar with label color stripe
    let title_color = if let Some(ref lbl) = item.metadata.label {
        lbl.color.to_iced_color()
    } else {
        Theme::CARD_TEXT
    };

    let title_bar = container(
        row![
            text(title).size(13).color(title_color),
            Space::with_width(Length::Fill),
            compile_indicator,
        ]
        .align_y(iced::Alignment::Center)
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

    // Label color indicator
    let label_indicator: Element<'static, Message> = if let Some(ref lbl) = item.metadata.label {
        let color = lbl.color.to_iced_color();
        container(
            text(format!(" {} ", lbl.name)).size(9).color(color)
        )
        .padding(Padding::from([2, 8]))
        .into()
    } else {
        Space::with_height(0).into()
    };

    // Keywords indicator
    let keywords_line: Element<'static, Message> = if !item.metadata.keywords.is_empty() {
        let kw_text = item.metadata.keywords.iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        let suffix = if item.metadata.keywords.len() > 3 {
            format!("... (+{})", item.metadata.keywords.len() - 3)
        } else {
            String::new()
        };
        text(format!("{}{}", kw_text, suffix)).size(9).color(Theme::TEXT_MUTED).into()
    } else {
        Space::with_height(0).into()
    };

    // Snapshot count indicator
    let snapshot_info: Element<'static, Message> = if !item.snapshots.is_empty() {
        text(format!("\u{1F4F7}{}", item.snapshots.len()))
            .size(9).color(Theme::TEXT_MUTED).into()
    } else {
        Space::with_width(0).into()
    };

    // Footer with status and word count
    let footer = container(
        row![
            text(word_count).size(10).color(Theme::TEXT_MUTED),
            Space::with_width(4),
            snapshot_info,
            Space::with_width(Length::Fill),
            text(status_text).size(10).color(status_color),
        ]
    )
    .padding(Padding::from([4, 8]));

    let card_content = column![
        title_bar,
        synopsis_area,
        label_indicator,
        keywords_line,
        footer,
    ];

    let card_btn = button(card_content)
        .on_press(Message::SelectBinderItem(id))
        .padding(0)
        .width(Length::Fixed(220.0));

    card_btn.into()
}
