use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length, Padding};

use crate::core::binder::BinderItem;
use crate::gui::app::Message;
use crate::gui::theme::{self, Theme};

/// Render the corkboard view — index cards on a cork background
pub fn view(items: &[&BinderItem], parent_title: &str) -> Element<'static, Message> {
    let item_count = items.len();
    let total_words: usize = items.iter()
        .filter_map(|i| i.document.as_ref())
        .map(|d| d.word_count())
        .sum();

    let compile_count = items.iter()
        .filter(|i| i.include_in_compile)
        .count();

    let header = container(
        row![
            text(format!("\u{f0ea} Corkboard: {}", parent_title))
                .size(14)
                .color(Theme::TEXT_SECONDARY),
            Space::with_width(Length::Fill),
            text(format!("{} cards | {} words | {}/{} compile",
                item_count, total_words, compile_count, item_count))
                .size(11)
                .color(Theme::TEXT_MUTED),
        ]
    )
    .style(theme::view_header_style)
    .padding(Padding::from([8, 16]))
    .width(Length::Fill);

    if items.is_empty() {
        let empty_msg = column![
            Space::with_height(40),
            text("Select a folder to see its cards, or switch to Editor view.")
                .size(13)
                .color(Theme::TEXT_MUTED),
            Space::with_height(8),
            text("Each document appears as an index card with its synopsis.")
                .size(11)
                .color(Theme::TEXT_MUTED),
        ]
        .align_x(iced::Alignment::Center);

        return container(
            column![header, container(empty_msg).padding(20).center_x(Length::Fill)]
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

    // Footer with aggregate stats
    let with_synopsis = items.iter().filter(|i| !i.synopsis.is_empty()).count();
    let with_status = items.iter().filter(|i| i.metadata.status.is_some()).count();
    let with_label = items.iter().filter(|i| i.metadata.label.is_some()).count();

    let footer = container(
        row![
            text(format!("{}/{} synopsis | {}/{} status | {}/{} labels",
                with_synopsis, item_count,
                with_status, item_count,
                with_label, item_count))
                .size(10)
                .color(Theme::TEXT_MUTED),
            Space::with_width(Length::Fill),
            text(format!("{:.1} pages", total_words as f64 / 250.0))
                .size(10)
                .color(Theme::TEXT_MUTED),
        ]
        .padding(Padding::from([4, 16]))
    )
    .style(theme::view_footer_style)
    .width(Length::Fill);

    let content = column![
        header,
        scrollable(
            container(grid)
                .padding(20)
                .width(Length::Fill)
        )
        .height(Length::Fill),
        footer,
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
        .map(|d| d.word_count())
        .unwrap_or(0);

    let word_display = if word_count == 0 {
        "empty".to_string()
    } else {
        format!("{} w", word_count)
    };

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
    let compile_icon = if item.include_in_compile {
        "\u{f00c}"  // checkmark
    } else {
        "\u{f00d}"  // cross
    };
    let compile_color = if item.include_in_compile {
        Theme::SUCCESS
    } else {
        Theme::TEXT_MUTED
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
            text(compile_icon).size(10).color(compile_color),
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

    // Label color indicator with name
    let label_indicator: Element<'static, Message> = if let Some(ref lbl) = item.metadata.label {
        let color = lbl.color.to_iced_color();
        container(
            row![
                text("\u{f111}").size(10).color(color),
                Space::with_width(4),
                text(lbl.name.clone()).size(9).color(color),
            ]
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
        container(
            row![
                text("\u{f02b}").size(8),
                Space::with_width(2),
                text(format!("{}{}", kw_text, suffix)).size(9).color(Theme::TEXT_MUTED),
            ]
        )
        .padding(Padding::from([0, 8]))
        .into()
    } else {
        Space::with_height(0).into()
    };

    // Snapshot and notes indicators
    let has_notes = item.document.as_ref()
        .map(|d| !d.notes.trim().is_empty())
        .unwrap_or(false);

    let mut indicator_parts: Vec<String> = Vec::new();
    if !item.snapshots.is_empty() {
        indicator_parts.push(format!("\u{f030}{}", item.snapshots.len()));
    }
    if has_notes {
        indicator_parts.push("\u{f044}".to_string());
    }

    let extra_indicators: Element<'static, Message> = if !indicator_parts.is_empty() {
        text(indicator_parts.join(" "))
            .size(9).color(Theme::TEXT_MUTED).into()
    } else {
        Space::with_width(0).into()
    };

    // Footer with status and word count
    let footer = container(
        row![
            text(word_display).size(10).color(Theme::TEXT_MUTED),
            Space::with_width(4),
            extra_indicators,
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
