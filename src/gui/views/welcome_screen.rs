use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::core::recent::RecentProjects;
use crate::gui::app::Message;
use crate::gui::theme::Theme;
use crate::templates::{built_in_templates, TemplateCategory};

/// Render the welcome/start screen
pub fn view(recent_projects: &RecentProjects) -> Element<'static, Message> {
    let title = text("Avalon")
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

    // Recent projects
    let mut recent_section = column![
        Space::with_height(20),
        text("Recent Projects").size(16).color(Theme::TEXT_SECONDARY),
        Space::with_height(8),
    ]
    .spacing(4);

    if recent_projects.projects.is_empty() {
        recent_section = recent_section.push(
            text("No recent projects.")
                .size(13)
                .color(Theme::TEXT_MUTED),
        );
    } else {
        for rp in recent_projects.projects.iter().take(10) {
            let path = rp.path.clone();
            let recent_btn = button(
                row![
                    text(rp.title.clone()).size(14).color(Theme::TEXT_PRIMARY),
                    Space::with_width(12),
                    text(rp.last_opened.format("%Y-%m-%d %H:%M").to_string())
                        .size(11)
                        .color(Theme::TEXT_MUTED),
                ]
            )
            .on_press(Message::OpenRecentProject(path))
            .padding(Padding::from([6, 12]));
            recent_section = recent_section.push(recent_btn);
        }
    }

    // Template categories
    let templates = built_in_templates();
    let categories = [
        TemplateCategory::Fiction,
        TemplateCategory::NonFiction,
        TemplateCategory::Scriptwriting,
        TemplateCategory::Academic,
    ];

    let mut template_section = column![
        Space::with_height(20),
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

    // Keyboard shortcuts help
    let shortcuts = column![
        Space::with_height(20),
        text("Keyboard Shortcuts").size(14).color(Theme::TEXT_SECONDARY),
        Space::with_height(4),
        text("Ctrl+S Save | Ctrl+N New Doc | Ctrl+F Search | Ctrl+E Compile").size(11).color(Theme::TEXT_MUTED),
        text("Ctrl+I Inspector | Ctrl+Z Undo | Ctrl+Y Redo | Ctrl+, Settings").size(11).color(Theme::TEXT_MUTED),
        text("Ctrl+1-4 Switch View | F11 Focus Mode | F5 Composition Mode").size(11).color(Theme::TEXT_MUTED),
        text("Ctrl+B Bold | Ctrl+U Underline | Ctrl+H Find/Replace | Esc Close Panel").size(11).color(Theme::TEXT_MUTED),
        text("Ctrl+Shift+F Composition | Ctrl+Shift+T Script Mode | Ctrl+Shift+G Goals").size(11).color(Theme::TEXT_MUTED),
    ]
    .spacing(2);

    let version_info = text("Avalon v0.1.0 — Open source writing studio")
        .size(10)
        .color(Theme::TEXT_MUTED);

    let content = column![
        Space::with_height(40),
        title,
        subtitle,
        Space::with_height(24),
        buttons,
        recent_section,
        template_section,
        shortcuts,
        Space::with_height(16),
        version_info,
    ]
    .align_x(iced::Alignment::Center)
    .padding(40);

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .into()
}
