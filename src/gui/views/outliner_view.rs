use std::collections::HashMap;
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};
use uuid::Uuid;

use crate::core::binder::{BinderItem, BinderItemKind};
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the outliner view — a hierarchical table with section numbering
pub fn view(draft: &BinderItem, targets: &HashMap<Uuid, usize>) -> Element<'static, Message> {
    let total_words = draft.total_word_count();
    let doc_count_header = count_text_items(draft);

    let header_info = container(
        row![
            text(format!("\u{f0ea} Outliner: {} docs | {} words | {:.1} pages",
                doc_count_header, total_words, total_words as f64 / 250.0))
                .size(12)
                .color(Theme::TEXT_SECONDARY),
            Space::with_width(Length::Fill),
        ]
        .padding(Padding::from([4, 16]))
    )
    .width(Length::Fill);

    let header_row = container(
        row![
            text("#").size(11).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(1)),
            text("Title").size(11).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(4)),
            text("Synopsis").size(11).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(3)),
            text("Status").size(11).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(2)),
            text("Label").size(11).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(2)),
            text("Words").size(11).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(1)),
            text("Target (%)").size(11).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(2)),
            text("Pgs").size(11).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(1)),
            text("Compile").size(11).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(1)),
        ]
        .spacing(6)
    )
    .padding(Padding::from([8, 16]));

    let mut row_elements: Vec<Element<'static, Message>> = Vec::new();
    let mut counter = vec![0usize];
    collect_outline_rows(draft, 0, targets, &mut row_elements, &mut counter);

    let mut rows = column![].spacing(1);
    for elem in row_elements {
        rows = rows.push(elem);
    }

    // Summary footer with target completion stats
    let doc_count = count_text_items(draft);
    let targeted_count = targets.len();
    let completed_targets = targets.iter()
        .filter(|(id, target)| {
            if let Some(item) = draft.find(id) {
                item.total_word_count() >= **target && **target > 0
            } else {
                false
            }
        })
        .count();

    let compile_count = count_compile_items(draft);

    let footer = container(
        row![
            text(format!("{} docs | {} words | {:.1} pg | {}/{} targets met | {}/{} compile",
                doc_count, total_words, total_words as f64 / 250.0,
                completed_targets, targeted_count,
                compile_count, doc_count))
                .size(10)
                .color(Theme::TEXT_MUTED),
        ]
    )
    .padding(Padding::from([6, 16]));

    let content = column![
        header_info,
        header_row,
        scrollable(rows).height(Length::Fill),
        footer,
    ];

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn count_text_items(item: &BinderItem) -> usize {
    let own = if item.kind == BinderItemKind::Text { 1 } else { 0 };
    let children: usize = item.children.iter().map(count_text_items).sum();
    own + children
}

fn count_compile_items(item: &BinderItem) -> usize {
    let own = if item.kind == BinderItemKind::Text && item.include_in_compile { 1 } else { 0 };
    let children: usize = item.children.iter().map(count_compile_items).sum();
    own + children
}

fn collect_outline_rows(
    item: &BinderItem,
    depth: usize,
    targets: &HashMap<Uuid, usize>,
    rows: &mut Vec<Element<'static, Message>>,
    counter: &mut Vec<usize>,
) {
    let indent = depth as u16 * 16;

    // Build section number
    let section_num = if depth > 0 {
        if let Some(c) = counter.last_mut() { *c += 1; }
        let nums: Vec<String> = counter.iter().map(|c| c.to_string()).collect();
        nums.join(".")
    } else {
        String::new()
    };

    let icon = match item.kind {
        BinderItemKind::Folder => if item.expanded { "\u{f0d7} " } else { "\u{f0da} " },
        BinderItemKind::Text => "\u{2022} ",
        _ => "  ",
    };

    let title_text = format!("{}{}", icon, item.title);
    let word_count = item.total_word_count();

    let status = item.metadata.status.as_ref()
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "-".to_string());

    let label = item.metadata.label.as_ref()
        .map(|l| l.name.clone())
        .unwrap_or_else(|| "-".to_string());

    let label_color = item.metadata.label.as_ref()
        .map(|l| l.color.to_iced_color())
        .unwrap_or(Theme::TEXT_SECONDARY);

    let target = targets.get(&item.id);
    let target_text = target
        .map(|t| t.to_string())
        .unwrap_or_else(|| "-".to_string());

    let progress_text = match target {
        Some(t) if *t > 0 => {
            let pct = (word_count as f64 / *t as f64 * 100.0).min(100.0);
            format!("{:.0}%", pct)
        }
        _ => "-".to_string(),
    };

    let progress_color = match target {
        Some(t) if *t > 0 => {
            let pct = word_count as f64 / *t as f64;
            if pct >= 1.0 {
                Theme::SUCCESS
            } else if pct >= 0.5 {
                Theme::WARNING
            } else {
                Theme::TEXT_SECONDARY
            }
        }
        _ => Theme::TEXT_MUTED,
    };

    let compile_text = if item.include_in_compile { "Yes" } else { "No" };
    let compile_color = if item.include_in_compile {
        Theme::TEXT_SECONDARY
    } else {
        Theme::TEXT_MUTED
    };

    let id = item.id;
    let title_btn = button(
        row![
            Space::with_width(indent),
            text(title_text).size(13).color(Theme::TEXT_PRIMARY),
        ]
    )
    .on_press(Message::SelectBinderItem(id))
    .padding(0)
    .width(Length::FillPortion(4));

    // Synopsis (truncated)
    let synopsis = if item.synopsis.is_empty() {
        "-".to_string()
    } else if item.synopsis.len() > 40 {
        format!("{}...", &item.synopsis[..40])
    } else {
        item.synopsis.clone()
    };

    // Pages estimate
    let pages_text = if word_count > 0 {
        format!("{:.1}", word_count as f64 / 250.0)
    } else {
        "-".to_string()
    };

    let row_content = row![
        text(section_num).size(10).color(Theme::TEXT_MUTED).width(Length::FillPortion(1)),
        title_btn,
        text(synopsis).size(10).color(Theme::TEXT_MUTED).width(Length::FillPortion(3)),
        text(status).size(11).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(2)),
        text(label).size(11).color(label_color).width(Length::FillPortion(2)),
        text(format!("{}", word_count)).size(11).color(Theme::TEXT_SECONDARY).width(Length::FillPortion(1)),
        text(format!("{} ({})", target_text, progress_text)).size(11).color(progress_color).width(Length::FillPortion(2)),
        text(pages_text).size(11).color(Theme::TEXT_MUTED).width(Length::FillPortion(1)),
        text(compile_text).size(11).color(compile_color).width(Length::FillPortion(1)),
    ]
    .spacing(6);

    let row_container = container(row_content)
        .padding(Padding::from([4, 16]))
        .width(Length::Fill);

    rows.push(row_container.into());

    if item.expanded {
        counter.push(0);
        for child in &item.children {
            collect_outline_rows(child, depth + 1, targets, rows, counter);
        }
        counter.pop();
    }
}
