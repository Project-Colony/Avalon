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

    // Action buttons row
    let add_doc_btn = small_action_btn("+Doc", Message::NewDocument);
    let add_folder_btn = small_action_btn("+Folder", Message::NewFolder);

    let action_row = row![
        add_doc_btn,
        Space::with_width(4),
        add_folder_btn,
    ]
    .spacing(2);

    // Selected item actions
    let selected_actions: Element<'static, Message> = if let Some(sel_id) = selected_id {
        let move_up_btn = small_action_btn("Up", Message::MoveItemUp(sel_id));
        let move_down_btn = small_action_btn("Dn", Message::MoveItemDown(sel_id));
        let delete_btn = small_action_btn("Del", Message::DeleteItem(sel_id));
        let dup_btn = small_action_btn("Dup", Message::DuplicateItem(sel_id));

        row![
            move_up_btn,
            move_down_btn,
            delete_btn,
            dup_btn,
        ]
        .spacing(2)
        .into()
    } else {
        Space::with_height(0).into()
    };

    // Empty trash button (only if trash has items)
    let trash_actions: Element<'static, Message> = if !trash.children.is_empty() {
        button(
            text("Empty Trash").size(11).color(Theme::ERROR),
        )
        .on_press(Message::EmptyTrash)
        .padding(Padding::from([3, 8]))
        .width(Length::Fill)
        .into()
    } else {
        Space::with_height(0).into()
    };

    let content = column![
        header,
        scrollable(
            column![
                draft_tree,
                Space::with_height(8),
                research_tree,
                Space::with_height(8),
                trash_tree,
                trash_actions,
            ]
            .width(Length::Fill)
        )
        .height(Length::Fill),
        container(
            column![
                action_row,
                selected_actions,
            ]
            .spacing(4)
        )
        .padding(8),
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
            if item.expanded { "v " } else { "> " }
        }
        BinderItemKind::Text => "  ",
        _ => "  ",
    };

    let label = format!("{}{}", icon, item.title);

    // Apply label color if present
    let label_color = if is_selected {
        Theme::TEXT_PRIMARY
    } else if let Some(ref lbl) = item.metadata.label {
        lbl.color.to_iced_color()
    } else {
        Theme::TEXT_SECONDARY
    };

    let item_text = text(label).size(14).color(label_color);

    // Status indicator
    let status_indicator: Element<'static, Message> = if let Some(ref status) = item.metadata.status {
        let status_color = match status.name.as_str() {
            "Done" | "Final Draft" => Theme::SUCCESS,
            "Revised Draft" => Theme::TEXT_ACCENT,
            "First Draft" => Theme::WARNING,
            "To Do" => Theme::TEXT_MUTED,
            _ => Theme::TEXT_SECONDARY,
        };
        text("*").size(12).color(status_color).into()
    } else {
        Space::with_width(0).into()
    };

    let item_btn = button(
        row![
            Space::with_width(indent),
            item_text,
            Space::with_width(Length::Fill),
            status_indicator,
        ]
    )
    .on_press(if item.kind == BinderItemKind::Folder {
        Message::ToggleBinderItem(item.id)
    } else {
        Message::SelectBinderItem(item.id)
    })
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

fn small_action_btn(label: &str, message: Message) -> Element<'static, Message> {
    button(
        text(label.to_string()).size(11).color(Theme::TEXT_SECONDARY),
    )
    .on_press(message)
    .padding(Padding::from([3, 6]))
    .into()
}
