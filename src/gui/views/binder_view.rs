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

    // Section header with item counts
    let icon = if item.expanded { "\u{25BE} " } else { "\u{25B8} " };
    let doc_count = count_docs(item);
    let folder_count = count_folders(item);
    let total_words = item.total_word_count();
    let count_info = if doc_count > 0 || folder_count > 0 {
        let mut parts = Vec::new();
        if doc_count > 0 { parts.push(format!("{}d", doc_count)); }
        if folder_count > 0 { parts.push(format!("{}f", folder_count)); }
        if total_words > 0 {
            let word_label = if total_words >= 1000 {
                format!("{:.1}k", total_words as f64 / 1000.0)
            } else {
                total_words.to_string()
            };
            parts.push(format!("{}w", word_label));
        }
        format!(" ({})", parts.join(" "))
    } else {
        String::new()
    };
    let header_text = format!("{}{}{}", icon, label.to_uppercase(), count_info);

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

/// Count documents in a binder section (recursive)
fn count_docs(item: &BinderItem) -> usize {
    let self_count = if item.kind == BinderItemKind::Text { 1 } else { 0 };
    self_count + item.children.iter().map(|c| count_docs(c)).sum::<usize>()
}

/// Count folders in a binder section (recursive, excluding root)
fn count_folders(item: &BinderItem) -> usize {
    item.children.iter().map(|c| {
        let self_count = if c.kind == BinderItemKind::Folder { 1 } else { 0 };
        self_count + count_folders(c)
    }).sum::<usize>()
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
            if item.expanded { "\u{1F4C2} " } else { "\u{1F4C1} " }
        }
        BinderItemKind::Text => "\u{1F4C4} ",
        BinderItemKind::Image => "\u{1F5BC} ",
        BinderItemKind::Pdf => "\u{1F4D1} ",
        BinderItemKind::WebPage => "\u{1F310} ",
    };

    // Compile indicator
    let compile_icon = if item.include_in_compile { "" } else { "\u{2298}" }; // circled minus for excluded

    // Show word count for documents
    let word_info = if item.kind == BinderItemKind::Text {
        item.document.as_ref()
            .map(|d| {
                let wc = d.word_count();
                if wc > 0 { format!(" ({})", wc) } else { String::new() }
            })
            .unwrap_or_default()
    } else if item.kind == BinderItemKind::Folder && !item.children.is_empty() {
        let total = item.total_word_count();
        if total > 0 { format!(" [{}]", total) } else { String::new() }
    } else {
        String::new()
    };

    let label = format!("{}{}{}", icon, item.title, word_info);

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

    let compile_indicator: Element<'static, Message> = if !compile_icon.is_empty() {
        text(compile_icon).size(10).color(Theme::TEXT_MUTED).into()
    } else {
        Space::with_width(0).into()
    };

    let item_btn = button(
        row![
            Space::with_width(indent),
            item_text,
            Space::with_width(Length::Fill),
            compile_indicator,
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
