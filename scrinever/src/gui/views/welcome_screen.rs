use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::Message;
use crate::gui::theme::Theme;
use crate::templates::{built_in_templates, TemplateCategory};

/// Render the welcome/start screen
pub fn view() -> Element<'static, Message> {
    let title = text("Scrinever")
        .size(42)
        .color(Theme::TEXT_PRIMARY);

    let subtitle = text("Free Writing Studio")
        .size(16)
        .color(Theme::TEXT_SECONDARY);

    let new_project_btn = button(
        text("  New Project  ").size(16).color(Theme::TEXT_PRIMARY),
    )
    .on_press(Message::NewProject)
    .padding(Padding::from([10, 24]));

    let open_project_btn = button(
        text("  Open Project  ").size(16).color(Theme::TEXT_PRIMARY),
    )
    .on_press(Message::OpenProject)
    .padding(Padding::from([10, 24]));

    let buttons = row![
        new_project_btn,
        Space::with_width(16),
        open_project_btn,
    ];

    // Template categories
    let templates = built_in_templates();
    let categories = [
        TemplateCategory::Fiction,
        TemplateCategory::NonFiction,
        TemplateCategory::Scriptwriting,
        TemplateCategory::Academic,
    ];

    let mut template_section = column![
        Space::with_height(24),
        text("Quick Start Templates").size(16).color(Theme::TEXT_SECONDARY),
        Space::with_height(8),
    ]
    .spacing(4);

    for cat in &categories {
        let cat_templates: Vec<_> = templates.iter()
            .filter(|t| &t.category == cat)
            .collect();

        if !cat_templates.is_empty() {
            let mut cat_row = row![].spacing(8);
            for tmpl in cat_templates {
                let tid = tmpl.template_id.clone();
                let tmpl_btn = button(
                    text(tmpl.name.clone()).size(13).color(Theme::TEXT_PRIMARY),
                )
                .on_press(Message::NewFromTemplate(tid))
                .padding(Padding::from([6, 12]));
                cat_row = cat_row.push(tmpl_btn);
            }

            template_section = template_section
                .push(text(format!("{}", cat)).size(12).color(Theme::TEXT_MUTED))
                .push(cat_row)
                .push(Space::with_height(4));
        }
    }

    let content = column![
        Space::with_height(60),
        title,
        subtitle,
        Space::with_height(32),
        buttons,
        template_section,
    ]
    .align_x(iced::Alignment::Center)
    .padding(40);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .into()
}
