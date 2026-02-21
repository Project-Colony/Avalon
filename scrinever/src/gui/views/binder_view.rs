use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};
use uuid::Uuid;

use crate::core::binder::{BinderItem, BinderItemKind};
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the binder sidebar
pub fn view(
    draft: &BinderItem,
    research: &BinderItem,
    trash: &BinderItem,
    selected_id: Option<Uuid>,
) -> Element<'static, Message> {
    let header = container(
        text("BINDER")
            .size(12)
            .color(Theme::TEXT_SECONDARY),
    )
    .padding(Padding::from([8, 12]));

    let draft_tree = render_section("Draft", draft, selected_id, 0);
    let research_tree = render_section("Research", research, selected_id, 0);
    let trash_tree = render_section("Trash", trash, selected_id, 0);

    let add_btn = button(
        text("+  New Document").size(13).color(Theme::TEXT_SECONDARY),
    )
    .on_press(Message::NewDocument)
    .padding(Padding::from([6, 12]))
    .width(Length::Fill);

    let content = column![
        header,
        scrollable(
            column![
                draft_tree,
                Space::with_height(8),
                research_tree,
                Space::with_height(8),
                trash_tree,
            ]
            .width(Length::Fill)
        )
        .height(Length::Fill),
        container(add_btn).padding(8),
    ]
    .width(Length::Fixed(240.0));

    container(content)
        .height(Length::Fill)
        .into()
}

/// Render a top-level section (Draft, Research, Trash)
fn render_section(
    label: &str,
    item: &BinderItem,
    selected_id: Option<Uuid>,
    depth: usize,
) -> Element<'static, Message> {
    let mut col = column![];

    // Section header
    let icon = if item.expanded { "v " } else { "> " };
    let header_text = format!("{}{}", icon, label.to_uppercase());

    let header_btn = button(
        text(header_text)
            .size(12)
            .color(Theme::TEXT_SECONDARY),
    )
    .on_press(Message::ToggleBinderItem(item.id))
    .padding(Padding::from([4, 12]))
    .width(Length::Fill);

    col = col.push(header_btn);

    // Children (if expanded)
    if item.expanded {
        for child in &item.children {
            col = col.push(render_item(child, selected_id, depth + 1));
        }
    }

    col.into()
}

/// Render a single binder item and its children
fn render_item(
    item: &BinderItem,
    selected_id: Option<Uuid>,
    depth: usize,
) -> Element<'static, Message> {
    let mut col = column![];

    let is_selected = selected_id == Some(item.id);
    let indent = depth as u16 * 16;

    let icon = match item.kind {
        BinderItemKind::Folder => {
            if item.expanded { "📂 " } else { "📁 " }
        }
        BinderItemKind::Text => "📄 ",
        _ => "📎 ",
    };

    let label = format!("{}{}", icon, item.title);

    let label_color = if is_selected {
        Theme::TEXT_PRIMARY
    } else {
        Theme::TEXT_SECONDARY
    };

    let item_text = text(label).size(14).color(label_color);

    let item_btn = button(
        row![
            Space::with_width(indent),
            item_text,
        ]
    )
    .on_press(Message::SelectBinderItem(item.id))
    .padding(Padding::from([3, 8]))
    .width(Length::Fill);

    col = col.push(item_btn);

    // Render children if expanded
    if item.expanded && !item.children.is_empty() {
        for child in &item.children {
            col = col.push(render_item(child, selected_id, depth + 1));
        }
    }

    col.into()
}
