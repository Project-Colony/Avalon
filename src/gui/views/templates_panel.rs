use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::core::doc_templates::DocumentTemplate;
use crate::gui::app::Message;
use crate::gui::theme::{self, Theme};

/// Render the document templates panel (bottom panel)
pub fn view(templates: &[DocumentTemplate]) -> Element<'static, Message> {
    let header = text("DOCUMENT TEMPLATES")
        .size(11)
        .color(Theme::TEXT_SECONDARY);

    let hint = text("Click a template to create a new document from it.")
        .size(10)
        .color(Theme::TEXT_MUTED);

    // Group templates by category
    let categories = ["Fiction", "Nonfiction", "Screenplay", "Planning", "Reference"];

    let mut template_list = column![].spacing(6);

    for category in categories {
        let category_templates: Vec<&DocumentTemplate> = templates.iter()
            .filter(|t| t.category.label() == category)
            .collect();

        if category_templates.is_empty() {
            continue;
        }

        let category_label = text(category.to_string())
            .size(11)
            .color(Theme::TEXT_ACCENT);

        let mut items_row = row![].spacing(6);
        for template in &category_templates {
            let btn = button(
                column![
                    text(template.name.to_string())
                        .size(12)
                        .color(Theme::TEXT_PRIMARY),
                    text(template.description.to_string())
                        .size(9)
                        .color(Theme::TEXT_MUTED),
                ]
                .spacing(1)
            )
            .on_press(Message::NewDocFromTemplate(template.id.to_string()))
            .padding(Padding::from([6, 10]));

            items_row = items_row.push(btn);
        }

        template_list = template_list.push(
            column![
                category_label,
                items_row,
            ]
            .spacing(3)
        );
    }

    let content = column![
        header,
        hint,
        Space::with_height(4),
        scrollable(template_list).height(Length::Fixed(140.0)),
    ]
    .spacing(4)
    .padding(Padding::from([8, 12]));

    container(content)
        .style(theme::panel_style)
        .width(Length::Fill)
        .into()
}
