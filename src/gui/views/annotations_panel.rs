use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length, Padding};
use uuid::Uuid;

use crate::core::annotation::Annotation;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

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
        text("\u{1F4DD}").size(10),
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
            text("\u{2795} Add").size(11).color(Theme::TEXT_ACCENT),
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
                    text("Add comments, notes, and reminders to your document text.")
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
        text("Select text in editor, then add annotation to mark it")
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
        .width(Length::Fill)
        .into()
}

fn render_annotation(ann: &Annotation) -> Element<'static, Message> {
    let ann_id = ann.id;
    let color = ann.color.to_iced_color();
    let is_resolved = ann.resolved;

    let text_color = if is_resolved {
        Theme::TEXT_MUTED
    } else {
        Theme::TEXT_PRIMARY
    };

    // Status indicator
    let status_icon = if is_resolved { "\u{2713}" } else { "\u{25CF}" };

    // Category tag
    let category_tag: Element<'static, Message> = if let Some(ref cat) = ann.category {
        text(format!("[{}]", cat)).size(9).color(Theme::TEXT_ACCENT).into()
    } else {
        Space::with_width(0).into()
    };

    // Edit count
    let edit_count: Element<'static, Message> = if !ann.edit_history.is_empty() {
        text(format!("edited {}x", ann.edit_history.len()))
            .size(8)
            .color(Theme::TEXT_MUTED)
            .into()
    } else {
        Space::with_width(0).into()
    };

    // Age display
    let age = ann.age_string();

    // Author display
    let author_display: Element<'static, Message> = if !ann.author.is_empty() {
        text(format!("by {}", ann.author)).size(8).color(Theme::TEXT_MUTED).into()
    } else {
        Space::with_width(0).into()
    };

    container(
        column![
            row![
                text(format!("{} ", status_icon)).size(12).color(color),
                text(ann.text.clone()).size(12).color(text_color),
                Space::with_width(4),
                category_tag,
                Space::with_width(Length::Fill),
                text(age).size(9).color(Theme::TEXT_MUTED),
            ],
            row![
                text(format!("span {}-{}", ann.start, ann.end))
                    .size(9)
                    .color(Theme::TEXT_MUTED),
                Space::with_width(4),
                author_display,
                Space::with_width(4),
                edit_count,
                Space::with_width(Length::Fill),
                button(
                    text(if is_resolved { "Reopen" } else { "\u{2713} Resolve" })
                        .size(9)
                        .color(if is_resolved { Theme::TEXT_MUTED } else { Theme::SUCCESS }),
                )
                .on_press(Message::ToggleAnnotationResolved(ann_id))
                .padding(Padding::from([1, 4])),
                Space::with_width(4),
                button(
                    text("\u{2715}").size(9).color(Theme::ERROR),
                )
                .on_press(Message::DeleteAnnotation(ann_id))
                .padding(Padding::from([1, 4])),
            ],
        ]
        .spacing(2)
    )
    .padding(Padding::from([4, 8]))
    .into()
}
