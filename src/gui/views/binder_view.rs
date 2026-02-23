use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Color, Element, Length, Padding};
use uuid::Uuid;

use crate::core::binder::{BinderItem, BinderItemKind};
use crate::gui::app::Message;
use crate::gui::theme::{self, Icons, Theme};

/// Section accent colors
const DRAFT_ACCENT: Color = Color::from_rgb(0.35, 0.65, 0.95);
const RESEARCH_ACCENT: Color = Color::from_rgb(0.55, 0.78, 0.45);
const TRASH_ACCENT: Color = Color::from_rgb(0.75, 0.45, 0.45);

/// Render the binder sidebar
pub fn view(
    draft: &BinderItem,
    research: &BinderItem,
    trash: &BinderItem,
    selected_id: Option<Uuid>,
) -> Element<'static, Message> {
    // ── Header ──────────────────────────────────────────
    let header = container(
        row![
            text(Icons::BOOK).size(13).color(Theme::TEXT_ACCENT),
            Space::with_width(8),
            text("Binder")
                .size(12)
                .color(Theme::TEXT_SECONDARY),
        ]
        .align_y(iced::Alignment::Center),
    )
    .style(theme::panel_header_style)
    .padding(Padding::from([10, 14]))
    .width(Length::Fill);

    // ── Section trees ───────────────────────────────────
    let draft_tree = render_section(
        "Draft",
        Icons::PENCIL_SQUARE,
        DRAFT_ACCENT,
        draft,
        selected_id,
    );
    let research_tree = render_section(
        "Research",
        Icons::SEARCH,
        RESEARCH_ACCENT,
        research,
        selected_id,
    );
    let trash_tree = render_section(
        "Trash",
        Icons::BAN,
        TRASH_ACCENT,
        trash,
        selected_id,
    );

    // ── Empty trash button ──────────────────────────────
    let trash_actions: Element<'static, Message> = if !trash.children.is_empty() {
        let trash_count = trash.children.len();
        container(
            button(
                row![
                    text(Icons::TIMES).size(11).color(Theme::ERROR),
                    Space::with_width(6),
                    text(format!("Empty Trash ({})", trash_count))
                        .size(11),
                ]
                .align_y(iced::Alignment::Center),
            )
            .on_press(Message::EmptyTrash)
            .style(theme::binder_danger_btn_style)
            .padding(Padding::from([5, 10]))
            .width(Length::Fill),
        )
        .padding(Padding { top: 0.0, right: 8.0, bottom: 4.0, left: 8.0 })
        .into()
    } else {
        Space::with_height(0).into()
    };

    // ── Separator helper ────────────────────────────────
    let sep = || -> Element<'static, Message> {
        container(Space::with_height(0))
            .style(theme::binder_separator_style)
            .height(1)
            .width(Length::Fill)
            .padding(Padding::from([0, 10]))
            .into()
    };

    // ── Main scrollable content ─────────────────────────
    let tree_content = column![
        draft_tree,
        sep(),
        research_tree,
        sep(),
        trash_tree,
        trash_actions,
        Space::with_height(8),
    ]
    .width(Length::Fill);

    // ── Bottom action bar ───────────────────────────────
    let action_bar = build_action_bar(selected_id);

    let content = column![
        header,
        scrollable(tree_content).height(Length::Fill),
        action_bar,
    ]
    .width(Length::Fixed(250.0));

    container(content)
        .style(theme::sidebar_style)
        .height(Length::Fill)
        .into()
}

/// Render a top-level section (Draft, Research, Trash)
fn render_section(
    label: &str,
    icon: &str,
    accent: Color,
    item: &BinderItem,
    selected_id: Option<Uuid>,
) -> Element<'static, Message> {
    let mut col = column![];

    // ── Section header ──────────────────────────────────
    let chevron = if item.expanded {
        Icons::CARET_DOWN
    } else {
        Icons::CARET_RIGHT
    };

    let doc_count = count_docs(item);
    let total_words = item.total_word_count();

    // Badge with count info
    let badge: Element<'static, Message> = if doc_count > 0 || total_words > 0 {
        let badge_text = if total_words > 0 {
            let word_label = if total_words >= 1000 {
                format!("{:.1}k", total_words as f64 / 1000.0)
            } else {
                total_words.to_string()
            };
            format!("{} {} w", doc_count, word_label)
        } else {
            format!("{}", doc_count)
        };

        container(
            text(badge_text).size(9).color(Theme::TEXT_MUTED),
        )
        .style(theme::binder_badge_style)
        .padding(Padding::from([2, 6]))
        .into()
    } else {
        Space::with_width(0).into()
    };

    let icon_owned = icon.to_string();
    let label_owned = label.to_string();

    let header_row = row![
        text(chevron).size(10).color(Theme::TEXT_MUTED),
        Space::with_width(6),
        text(icon_owned).size(13).color(accent),
        Space::with_width(6),
        text(label_owned).size(12).color(Theme::TEXT_PRIMARY),
        Space::with_width(Length::Fill),
        badge,
    ]
    .align_y(iced::Alignment::Center);

    let header_btn = button(header_row)
        .on_press(Message::ToggleBinderItem(item.id))
        .style(theme::binder_section_btn_style)
        .padding(Padding::from([7, 12]))
        .width(Length::Fill);

    let section_header = container(header_btn)
        .style(theme::binder_section_header_style(accent))
        .padding(Padding::from([4, 6]))
        .width(Length::Fill);

    col = col.push(section_header);

    // ── Children (if expanded) ──────────────────────────
    if item.expanded {
        let mut items_col = column![].spacing(1).padding(Padding::from([2, 0]));
        for child in &item.children {
            items_col = items_col.push(render_item(child, selected_id, 1));
        }
        col = col.push(items_col);
    }

    col.spacing(2).into()
}

/// Count documents in a binder section (recursive)
fn count_docs(item: &BinderItem) -> usize {
    let self_count = if item.kind == BinderItemKind::Text { 1 } else { 0 };
    self_count + item.children.iter().map(|c| count_docs(c)).sum::<usize>()
}

/// Render a single binder item and its children
fn render_item(
    item: &BinderItem,
    selected_id: Option<Uuid>,
    depth: usize,
) -> Element<'static, Message> {
    let mut col = column![];

    let is_selected = selected_id == Some(item.id);
    let indent = (depth as u16).saturating_sub(1) * 14 + 8;

    // ── Icon ────────────────────────────────────────────
    let (icon_str, icon_color) = match item.kind {
        BinderItemKind::Folder => {
            let ic = if item.expanded { Icons::FOLDER_OPEN } else { Icons::FOLDER };
            (ic, Color::from_rgb(0.75, 0.65, 0.40))
        }
        BinderItemKind::Text => (Icons::FILE_TEXT, Color::from_rgb(0.55, 0.70, 0.85)),
        BinderItemKind::Image => (Icons::FILE_IMAGE, Color::from_rgb(0.70, 0.55, 0.80)),
        BinderItemKind::Pdf => (Icons::FILE_PDF, Color::from_rgb(0.85, 0.45, 0.40)),
        BinderItemKind::WebPage => (Icons::GLOBE, Color::from_rgb(0.45, 0.75, 0.65)),
    };

    // ── Folder chevron ──────────────────────────────────
    let chevron: Element<'static, Message> = if item.kind == BinderItemKind::Folder {
        let ch = if item.expanded { Icons::CARET_DOWN } else { Icons::CARET_RIGHT };
        text(ch).size(9).color(Theme::TEXT_MUTED).into()
    } else {
        Space::with_width(9).into()
    };

    // ── Label color ─────────────────────────────────────
    let label_color = if is_selected {
        Theme::TEXT_PRIMARY
    } else if let Some(ref lbl) = item.metadata.label {
        lbl.color.to_iced_color()
    } else {
        Theme::TEXT_SECONDARY
    };

    // ── Title text ──────────────────────────────────────
    let title_text = text(item.title.clone()).size(13).color(label_color);

    // ── Word count (subtle, right-aligned) ──────────────
    let word_info: Element<'static, Message> = if item.kind == BinderItemKind::Text {
        if let Some(ref d) = item.document {
            let wc = d.word_count();
            if wc > 0 {
                text(format!("{}", wc))
                    .size(9)
                    .color(Theme::TEXT_MUTED)
                    .into()
            } else {
                Space::with_width(0).into()
            }
        } else {
            Space::with_width(0).into()
        }
    } else if item.kind == BinderItemKind::Folder && !item.children.is_empty() {
        let total = item.total_word_count();
        if total > 0 {
            let label = if total >= 1000 {
                format!("{:.1}k", total as f64 / 1000.0)
            } else {
                total.to_string()
            };
            text(label).size(9).color(Theme::TEXT_MUTED).into()
        } else {
            Space::with_width(0).into()
        }
    } else {
        Space::with_width(0).into()
    };

    // ── Status dot ──────────────────────────────────────
    let status_dot: Element<'static, Message> = if let Some(ref status) = item.metadata.status {
        let status_color = match status.name.as_str() {
            "Done" | "Final Draft" => Theme::SUCCESS,
            "Revised Draft" => Theme::TEXT_ACCENT,
            "First Draft" => Theme::WARNING,
            "To Do" => Theme::TEXT_MUTED,
            _ => Theme::TEXT_SECONDARY,
        };
        text(Icons::CIRCLE).size(6).color(status_color).into()
    } else {
        Space::with_width(0).into()
    };

    // ── Compile exclusion indicator ─────────────────────
    let compile_indicator: Element<'static, Message> = if !item.include_in_compile {
        text(Icons::BAN).size(8).color(Theme::TEXT_MUTED).into()
    } else {
        Space::with_width(0).into()
    };

    // ── Assemble item row ───────────────────────────────
    let item_row = row![
        Space::with_width(indent),
        chevron,
        Space::with_width(4),
        text(icon_str).size(12).color(if is_selected {
            Theme::lighten(icon_color, 0.15)
        } else {
            icon_color
        }),
        Space::with_width(6),
        title_text,
        Space::with_width(Length::Fill),
        compile_indicator,
        Space::with_width(2),
        status_dot,
        Space::with_width(4),
        word_info,
        Space::with_width(4),
    ]
    .align_y(iced::Alignment::Center);

    let btn_style = if is_selected {
        theme::binder_item_selected_btn_style as fn(&iced::Theme, button::Status) -> button::Style
    } else {
        theme::binder_item_btn_style
    };

    let item_btn = button(item_row)
        .on_press(if item.kind == BinderItemKind::Folder {
            Message::ToggleBinderItem(item.id)
        } else {
            Message::SelectBinderItem(item.id)
        })
        .style(btn_style)
        .padding(Padding::from([4, 4]))
        .width(Length::Fill);

    col = col.push(
        container(item_btn)
            .padding(Padding::from([0, 4]))
    );

    // ── Render children if expanded ─────────────────────
    if item.expanded && !item.children.is_empty() {
        let mut children_col = column![].spacing(1);
        for child in &item.children {
            children_col = children_col.push(render_item(child, selected_id, depth + 1));
        }

        // Wrap children with a left indent guide
        let children_with_guide = row![
            Space::with_width(indent + 6),
            container(Space::with_width(0))
                .style(theme::binder_indent_guide_style)
                .width(1)
                .height(Length::Fill),
            Space::with_width(3),
            children_col.width(Length::Fill),
        ];

        col = col.push(children_with_guide);
    }

    col.into()
}

/// Build the bottom action bar
fn build_action_bar(selected_id: Option<Uuid>) -> Element<'static, Message> {
    // Primary actions (always visible)
    let add_doc = action_btn(Icons::FILE_TEXT, "New Doc", Message::NewDocument);
    let add_folder = action_btn(Icons::FOLDER, "New Folder", Message::NewFolder);

    let primary_row = row![
        add_doc,
        Space::with_width(4),
        add_folder,
    ];

    // Context actions (only if item selected)
    let context_row: Element<'static, Message> = if let Some(sel_id) = selected_id {
        let move_up = icon_btn(Icons::ARROW_UP, Message::MoveItemUp(sel_id));
        let move_down = icon_btn(Icons::ARROW_DOWN, Message::MoveItemDown(sel_id));
        let duplicate = icon_btn(Icons::CLIPBOARD, Message::DuplicateItem(sel_id));
        let delete = danger_icon_btn(Icons::TIMES, Message::DeleteItem(sel_id));

        row![
            move_up,
            Space::with_width(2),
            move_down,
            Space::with_width(6),
            duplicate,
            Space::with_width(Length::Fill),
            delete,
        ]
        .align_y(iced::Alignment::Center)
        .into()
    } else {
        Space::with_height(0).into()
    };

    container(
        column![
            primary_row,
            context_row,
        ]
        .spacing(4),
    )
    .style(theme::binder_action_bar_style)
    .padding(Padding::from([8, 10]))
    .width(Length::Fill)
    .into()
}

/// Action button with icon + label
fn action_btn(icon: &str, label: &str, message: Message) -> Element<'static, Message> {
    button(
        row![
            text(icon.to_string()).size(11).color(Theme::TEXT_ACCENT),
            Space::with_width(5),
            text(label.to_string()).size(11).color(Theme::TEXT_SECONDARY),
        ]
        .align_y(iced::Alignment::Center),
    )
    .on_press(message)
    .style(theme::binder_action_btn_style)
    .padding(Padding::from([4, 8]))
    .into()
}

/// Small icon-only button
fn icon_btn(icon: &str, message: Message) -> Element<'static, Message> {
    button(
        text(icon.to_string()).size(12).color(Theme::TEXT_SECONDARY),
    )
    .on_press(message)
    .style(theme::binder_action_btn_style)
    .padding(Padding::from([4, 7]))
    .into()
}

/// Small icon-only button (danger variant)
fn danger_icon_btn(icon: &str, message: Message) -> Element<'static, Message> {
    button(
        text(icon.to_string()).size(12).color(Theme::ERROR),
    )
    .on_press(message)
    .style(theme::binder_danger_btn_style)
    .padding(Padding::from([4, 7]))
    .into()
}
