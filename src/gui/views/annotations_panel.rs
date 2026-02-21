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

    let header = row![
        text("ANNOTATIONS / COMMENTS").size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        text(format!("{} open, {} resolved", open_count, resolved_count))
            .size(10)
            .color(Theme::TEXT_MUTED),
    ];

    // Add annotation input
    let input_row = row![
        text_input("Add a comment...", new_annotation_text)
            .on_input(|val| Message::AnnotationTextInput(val))
            .size(12)
            .padding(4)
            .width(Length::FillPortion(3)),
        Space::with_width(4),
        button(
            text("Add").size(11).color(Theme::TEXT_ACCENT),
        )
        .on_press(Message::AddAnnotation)
        .padding(Padding::from([4, 10])),
    ]
    .align_y(iced::Alignment::Center);

    // Annotation list
    let mut list = column![].spacing(4);

    if annotations.is_empty() {
        list = list.push(
            text("No annotations yet. Add comments to your document.")
                .size(11)
                .color(Theme::TEXT_MUTED),
        );
    }

    for ann in annotations {
        let ann_id = ann.id;
        let color = ann.color.to_iced_color();

        // Color indicator dot + text
        let resolved_icon = if ann.resolved { " [done]" } else { "" };

        let text_color = if ann.resolved {
            Theme::TEXT_MUTED
        } else {
            Theme::TEXT_PRIMARY
        };

        let ann_row = container(
            column![
                row![
                    // Color indicator
                    text("\u{25CF} ").size(12).color(color),
                    text(format!("{}{}", ann.text.clone(), resolved_icon))
                        .size(12)
                        .color(text_color),
                    Space::with_width(Length::Fill),
                    text(ann.created_at.format("%H:%M %m/%d").to_string())
                        .size(9)
                        .color(Theme::TEXT_MUTED),
                ],
                row![
                    text(format!("chars {}-{}", ann.start, ann.end))
                        .size(9)
                        .color(Theme::TEXT_MUTED),
                    Space::with_width(Length::Fill),
                    button(
                        text(if ann.resolved { "Reopen" } else { "Resolve" })
                            .size(9)
                            .color(Theme::TEXT_ACCENT),
                    )
                    .on_press(Message::ToggleAnnotationResolved(ann_id))
                    .padding(Padding::from([1, 4])),
                    Space::with_width(4),
                    button(
                        text("Del").size(9).color(Theme::ERROR),
                    )
                    .on_press(Message::DeleteAnnotation(ann_id))
                    .padding(Padding::from([1, 4])),
                ],
            ]
            .spacing(2)
        )
        .padding(Padding::from([4, 8]));

        list = list.push(ann_row);
    }

    let content = column![
        header,
        Space::with_height(4),
        input_row,
        Space::with_height(6),
        scrollable(list).height(Length::Fixed(120.0)),
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
