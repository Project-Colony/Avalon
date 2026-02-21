use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::core::binder::{BinderItem, BinderItemKind};
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the outliner view — a hierarchical table
pub fn view(draft: &BinderItem) -> Element<'static, Message> {
    let header_row = container(
        row![
            text("Title").size(12).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(3)),
            text("Status").size(12).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(1)),
            text("Label").size(12).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(1)),
            text("Words").size(12).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(1)),
            text("Target").size(12).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(1)),
            text("Progress").size(12).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(1)),
        ]
        .spacing(8)
    )
    .padding(Padding::from([8, 16]));

    let mut row_elements: Vec<Element<'static, Message>> = Vec::new();
    collect_outline_rows(draft, 0, &mut row_elements);

    let mut rows = column![].spacing(1);
    for elem in row_elements {
        rows = rows.push(elem);
    }

    let content = column![
        header_row,
        scrollable(rows).height(Length::Fill),
    ];

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn collect_outline_rows(
    item: &BinderItem,
    depth: usize,
    rows: &mut Vec<Element<'static, Message>>,
) {
    let indent = depth as u16 * 20;

    let icon = match item.kind {
        BinderItemKind::Folder => "📁 ",
        BinderItemKind::Text => "📄 ",
        _ => "📎 ",
    };

    let title_text = format!("{}{}", icon, item.title);
    let word_count = item.total_word_count();

    let status = item.metadata.status.as_ref()
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "—".to_string());

    let label = item.metadata.label.as_ref()
        .map(|l| l.name.clone())
        .unwrap_or_else(|| "—".to_string());

    let row_content = row![
        row![
            Space::with_width(indent),
            text(title_text).size(13).color(Theme::TEXT_PRIMARY),
        ].width(Length::FillPortion(3)),
        text(status).size(12).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(1)),
        text(label).size(12).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(1)),
        text(format!("{}", word_count)).size(12).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(1)),
        text("—").size(12).color(Theme::TEXT_MUTED).width(Length::FillPortion(1)),
        text("—").size(12).color(Theme::TEXT_MUTED).width(Length::FillPortion(1)),
    ]
    .spacing(8);

    let row_container = container(row_content)
        .padding(Padding::from([4, 16]))
        .width(Length::Fill);

    rows.push(row_container.into());

    if item.expanded {
        for child in &item.children {
            collect_outline_rows(child, depth + 1, rows);
        }
    }
}
