use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length, Padding};
use uuid::Uuid;

use crate::core::annotation::{Annotation, AnnotationColor};
use crate::gui::app::Message;
use crate::gui::theme::{self, Theme};

/// Render the annotations panel (bottom panel)
pub fn view(
    annotations: &[Annotation],
    new_annotation_text: &str,
) -> Element<'static, Message> {
    let total = annotations.len();
    let resolved_count = annotations.iter().filter(|a| a.resolved).count();
    let open_count = total - resolved_count;

    // Count by category
    let categorized = annotations.iter()
        .filter_map(|a| a.category.as_ref())
        .count();

    let header = row![
        text("ANNOTATIONS").size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(4),
        text("\u{f044}").size(10),
        Space::with_width(Length::Fill),
        text(format!("{} open", open_count))
            .size(10)
            .color(if open_count > 0 { Theme::WARNING } else { Theme::TEXT_MUTED }),
        Space::with_width(6),
        text(format!("{} resolved", resolved_count))
            .size(10)
            .color(if resolved_count > 0 { Theme::SUCCESS } else { Theme::TEXT_MUTED }),
    ];

    // Add annotation input
    let input_row = row![
        text_input("Add a comment or note...", new_annotation_text)
            .on_input(|val| Message::AnnotationTextInput(val))
            .size(12)
            .padding(4)
            .width(Length::FillPortion(3)),
        Space::with_width(4),
        button(
            text("\u{f067} Add").size(11).color(Theme::TEXT_ACCENT),
        )
        .on_press(Message::AddAnnotation)
        .padding(Padding::from([4, 10])),
    ]
    .align_y(iced::Alignment::Center);

    // Annotation list - show open first, then resolved
    let mut list = column![].spacing(4);

    if annotations.is_empty() {
        list = list.push(
            container(
                column![
                    text("No annotations yet.").size(12).color(Theme::TEXT_MUTED),
                    Space::with_height(4),
                    text("Select text in the editor, then add a comment with color and category.")
                        .size(10)
                        .color(Theme::TEXT_MUTED),
                ]
            ).padding(Padding::from([4, 0]))
        );
    }

    // Open annotations first
    for ann in annotations.iter().filter(|a| !a.resolved) {
        list = list.push(render_annotation(ann));
    }

    // Then resolved (dimmed)
    if resolved_count > 0 {
        list = list.push(
            text(format!("\u{2500}\u{2500}\u{2500} Resolved ({}) \u{2500}\u{2500}\u{2500}", resolved_count))
                .size(9)
                .color(Theme::TEXT_MUTED)
        );
        for ann in annotations.iter().filter(|a| a.resolved) {
            list = list.push(render_annotation(ann));
        }
    }

    let hint = row![
        text("Select text, then add annotation | Click color dot to change color")
            .size(9)
            .color(Theme::TEXT_MUTED),
        Space::with_width(Length::Fill),
        text(format!("{} total | {} categorized", total, categorized))
            .size(9)
            .color(Theme::TEXT_MUTED),
    ];

    let content = column![
        header,
        Space::with_height(4),
        input_row,
        Space::with_height(6),
        scrollable(list).height(Length::Fixed(120.0)),
        Space::with_height(2),
        hint,
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .style(theme::panel_style)
        .width(Length::Fill)
        .into()
}

fn render_annotation(ann: &Annotation) -> Element<'static, Message> {
    let ann_id = ann.id;
    let color = ann.color.to_iced_color();
    let is_resolved = ann.resolved;
    let next_color_label = ann.color.next().label().to_string();
    let ann_text_str = ann.text.clone();
    let ann_start = ann.start;
    let ann_end = ann.end;
    let ann_author = ann.author.clone();
    let ann_category = ann.category.clone();
    let edit_history_len = ann.edit_history.len();
    let color_label_str = ann.color.label().to_string();
    let age = ann.age_string();

    let text_color = if is_resolved {
        Theme::TEXT_MUTED
    } else {
        Theme::TEXT_PRIMARY
    };

    // Status indicator - clickable to cycle color
    let status_icon = if is_resolved { "\u{f00c}" } else { "\u{f111}" };

    // Color dot button - click to cycle to next color
    let color_btn: Element<'static, Message> = if !is_resolved {
        button(
            text(format!("{} ", status_icon)).size(12).color(color),
        )
        .on_press(Message::SetAnnotationColor(ann_id, next_color_label))
        .padding(Padding::from([0, 2]))
        .into()
    } else {
        text(format!("{} ", status_icon)).size(12).color(color).into()
    };

    // Category tag with dropdown-like buttons
    let category_display: Element<'static, Message> = if let Some(cat) = ann_category {
        // Show current category as clickable button to clear it
        button(
            text(format!("[{}]", cat)).size(9).color(Theme::TEXT_ACCENT),
        )
        .on_press(Message::SetAnnotationCategory(ann_id, String::new()))
        .padding(Padding::from([0, 2]))
        .into()
    } else if !is_resolved {
        // Show category assignment buttons
        let mut cat_row = row![].spacing(2);
        for cat in &["Note", "Todo", "Question", "Research", "Continuity", "Revision"] {
            let cat_str = cat.to_string();
            cat_row = cat_row.push(
                button(
                    text(*cat).size(8).color(Theme::TEXT_MUTED),
                )
                .on_press(Message::SetAnnotationCategory(ann_id, cat_str))
                .padding(Padding::from([0, 3])),
            );
        }
        cat_row.into()
    } else {
        Space::with_width(0).into()
    };

    // Edit count
    let edit_count: Element<'static, Message> = if edit_history_len > 0 {
        text(format!("edited {}x", edit_history_len))
            .size(8)
            .color(Theme::TEXT_MUTED)
            .into()
    } else {
        Space::with_width(0).into()
    };

    // Author display
    let author_display: Element<'static, Message> = if !ann_author.is_empty() {
        text(format!("by {}", ann_author)).size(8).color(Theme::TEXT_MUTED).into()
    } else {
        Space::with_width(0).into()
    };

    // Annotation text display
    let ann_text_el: Element<'static, Message> =
        text(ann_text_str).size(12).color(text_color).into();

    // Color label
    let color_label_el = text(color_label_str)
        .size(8)
        .color(color);

    container(
        column![
            row![
                color_btn,
                ann_text_el,
                Space::with_width(4),
                color_label_el,
                Space::with_width(Length::Fill),
                text(age).size(9).color(Theme::TEXT_MUTED),
            ],
            row![
                text(format!("span {}-{}", ann_start, ann_end))
                    .size(9)
                    .color(Theme::TEXT_MUTED),
                Space::with_width(4),
                author_display,
                Space::with_width(4),
                edit_count,
                Space::with_width(Length::Fill),
                button(
                    text(if is_resolved { "Reopen" } else { "\u{f00c} Resolve" })
                        .size(9)
                        .color(if is_resolved { Theme::TEXT_MUTED } else { Theme::SUCCESS }),
                )
                .on_press(Message::ToggleAnnotationResolved(ann_id))
                .padding(Padding::from([1, 4])),
                Space::with_width(4),
                button(
                    text("\u{f00d}").size(9).color(Theme::ERROR),
                )
                .on_press(Message::DeleteAnnotation(ann_id))
                .padding(Padding::from([1, 4])),
            ],
            category_display,
        ]
        .spacing(2)
    )
    .padding(Padding::from([4, 8]))
    .into()
}
