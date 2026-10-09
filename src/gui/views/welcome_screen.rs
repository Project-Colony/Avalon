#![allow(dead_code)] // Methods used by test code
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Background, Border, Color, Element, Length, Padding};

use crate::core::recent::RecentProjects;
use crate::gui::app::Message;
use crate::gui::theme::{Icons, Theme};
use crate::templates::{built_in_templates, TemplateCategory};

// ── Style helpers ────────────────────────────────────────────────────

fn card_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Theme::BG_SECONDARY)),
        border: Border {
            color: Theme::BORDER_SUBTLE,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

fn accent_card_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.20, 0.35, 0.55, 0.15))),
        border: Border {
            color: Color::from_rgba(0.30, 0.60, 0.90, 0.30),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

fn hero_bg_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Theme::BG_EDITOR)),
        border: Border {
            color: Theme::BORDER_SUBTLE,
            width: 0.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

fn footer_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.14, 0.14, 0.17, 0.8))),
        border: Border {
            color: Theme::BORDER_SUBTLE,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

fn template_card_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Theme::BG_TERTIARY)),
        border: Border {
            color: Theme::BORDER_SUBTLE,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

// ── Section headers ──────────────────────────────────────────────────

fn section_header(label: &str) -> Element<'static, Message> {
    row![
        text(label.to_string()).size(14).color(Theme::TEXT_SECONDARY),
        Space::new().width(Length::Fill),
    ]
    .into()
}

// ── Main view ────────────────────────────────────────────────────────

/// Render the welcome/start screen
pub fn view(recent_projects: &RecentProjects) -> Element<'static, Message> {
    // ── Hero section ─────────────────────────────────────────────
    let hero = container(
        column![
            Space::new().height(28),
            text("Avalon").size(48).color(Theme::TEXT_PRIMARY),
            text("Free Writing Studio").size(15).color(Theme::TEXT_MUTED),
            Space::new().height(24),
            row![
                button(
                    row![
                        text(Icons::PLUS).size(16).color(Theme::TEXT_ACCENT),
                        Space::new().width(8),
                        text("New Project").size(15).color(Theme::TEXT_PRIMARY),
                    ]
                    .align_y(iced::Alignment::Center)
                )
                .on_press(Message::NewProject)
                .padding(Padding::from([12, 28])),
                Space::new().width(12),
                button(
                    row![
                        text(Icons::FOLDER_OPEN).size(14).color(Theme::TEXT_ACCENT),
                        Space::new().width(8),
                        text("Open Project").size(15).color(Theme::TEXT_PRIMARY),
                    ]
                    .align_y(iced::Alignment::Center)
                )
                .on_press(Message::OpenProject)
                .padding(Padding::from([12, 28])),
            ],
            Space::new().height(28),
        ]
        .align_x(iced::Alignment::Center),
    )
    .style(hero_bg_style)
    .width(Length::Fill)
    .center_x(Length::Fill)
    .padding(Padding::from([0, 20]));

    // ── Recent Projects section ──────────────────────────────────
    let recent_content = build_recent_section(recent_projects);

    let recent_card = container(
        column![
            section_header("Recent Projects"),
            Space::new().height(8),
            recent_content,
        ]
        .width(Length::Fill),
    )
    .style(card_style)
    .padding(Padding::from([16, 20]))
    .width(Length::Fill);

    // ── Templates section ────────────────────────────────────────
    let templates_content = build_templates_section();

    let templates_card = container(
        column![
            section_header("Start from Template"),
            Space::new().height(8),
            templates_content,
        ]
        .width(Length::Fill),
    )
    .style(card_style)
    .padding(Padding::from([16, 20]))
    .width(Length::Fill);

    // ── Two-column layout ────────────────────────────────────────
    let main_content = row![
        container(recent_card).width(Length::FillPortion(1)),
        Space::new().width(16),
        container(templates_card).width(Length::FillPortion(1)),
    ]
    .width(Length::Fill);

    // ── Footer: tip + version ────────────────────────────────────
    let footer = build_footer();

    // ── Assemble everything ──────────────────────────────────────
    let page = column![
        hero,
        Space::new().height(20),
        main_content,
        Space::new().height(16),
        footer,
        Space::new().height(8),
    ]
    .padding(Padding::from([24, 40]));

    container(scrollable(page))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .into()
}

// ── Recent projects list ─────────────────────────────────────────────

fn build_recent_section(recent_projects: &RecentProjects) -> Element<'static, Message> {
    if recent_projects.projects.is_empty() {
        return column![
            Space::new().height(20),
            text("No recent projects yet.").size(13).color(Theme::TEXT_MUTED),
            Space::new().height(4),
            text("Create a new project or open an existing one to get started.")
                .size(12)
                .color(Theme::TEXT_MUTED),
            Space::new().height(20),
        ]
        .align_x(iced::Alignment::Center)
        .into();
    }

    let mut list = column![].spacing(4);
    for rp in recent_projects.projects.iter().take(8) {
        let path = rp.path.clone();
        let age = rp.age_string();
        let dir = rp.directory_name();

        let entry = button(
            row![
                column![
                    text(rp.title.clone()).size(14).color(Theme::TEXT_PRIMARY),
                    text(if dir.is_empty() {
                        rp.path.to_string_lossy().to_string()
                    } else {
                        dir
                    })
                    .size(11)
                    .color(Theme::TEXT_MUTED),
                ]
                .spacing(2)
                .width(Length::Fill),
                text(age).size(11).color(Theme::TEXT_MUTED),
            ]
            .align_y(iced::Alignment::Center)
            .width(Length::Fill),
        )
        .on_press(Message::OpenRecentProject(path))
        .padding(Padding::from([8, 12]))
        .width(Length::Fill);

        list = list.push(entry);
    }

    list.into()
}

// ── Templates grid ───────────────────────────────────────────────────

fn build_templates_section() -> Element<'static, Message> {
    let templates = built_in_templates();
    let categories = [
        TemplateCategory::Fiction,
        TemplateCategory::NonFiction,
        TemplateCategory::Scriptwriting,
        TemplateCategory::Academic,
        TemplateCategory::Miscellaneous,
    ];

    let mut sections = column![].spacing(12);

    for cat in &categories {
        let cat_templates: Vec<_> = templates.iter().filter(|t| &t.category == cat).collect();

        if cat_templates.is_empty() {
            continue;
        }

        let icon = cat.icon().to_string();
        let cat_name = cat.to_string();
        let cat_header = row![
            text(icon).size(13),
            Space::new().width(6),
            text(cat_name).size(13).color(Theme::TEXT_ACCENT),
        ];

        let mut template_col = column![].spacing(4);
        for tmpl in &cat_templates {
            let tid = tmpl.template_id.clone();
            let tmpl_btn = button(
                container(
                    column![
                        text(tmpl.name.clone()).size(13).color(Theme::TEXT_PRIMARY),
                        text(tmpl.description.clone()).size(11).color(Theme::TEXT_MUTED),
                    ]
                    .spacing(2),
                )
                .style(template_card_style)
                .padding(Padding::from([8, 12]))
                .width(Length::Fill),
            )
            .on_press(Message::NewFromTemplate(tid))
            .padding(0)
            .width(Length::Fill);

            template_col = template_col.push(tmpl_btn);
        }

        sections = sections.push(cat_header).push(template_col);
    }

    scrollable(sections).height(Length::Fill).into()
}

// ── Footer ───────────────────────────────────────────────────────────

fn build_footer() -> Element<'static, Message> {
    let tips = [
        "Use Collections to organize documents into custom groups outside the binder hierarchy.",
        "Snapshots let you save and compare versions of your document at any point.",
        "The Corkboard view shows index cards for each document in a folder.",
        "Use [[Title]] syntax to create links between your documents.",
        "Set word targets per document or per session in the Targets panel.",
        "Use labels and statuses to track progress on individual documents.",
        "The Scrivenings view stitches multiple documents together for seamless reading.",
        "Project Notes are a quick scratchpad saved with your project.",
        "The Writing Timer (Ctrl+J) has Pomodoro, Sprint, and custom presets.",
        "Ctrl+Shift+S shows comprehensive project statistics with readability scores.",
        "Use the Name Generator (Ctrl+Shift+N) for character and place names.",
        "Auto-backup zips your project every few saves; Restore opens a backup as a copy.",
        "F8 runs project validation to find broken links, empty docs, and orphans.",
    ];
    let tip_index = {
        use chrono::Datelike;
        chrono::Local::now().ordinal0() as usize % tips.len()
    };

    container(
        row![
            text(Icons::LIGHTBULB).size(11).color(Theme::TEXT_ACCENT),
            Space::new().width(4),
            text("Tip:").size(11).color(Theme::TEXT_ACCENT),
            Space::new().width(6),
            text(tips[tip_index]).size(11).color(Theme::TEXT_SECONDARY),
            Space::new().width(Length::Fill),
            text("Avalon v0.1.0").size(10).color(Theme::TEXT_MUTED),
        ]
        .align_y(iced::Alignment::Center),
    )
    .style(footer_style)
    .padding(Padding::from([10, 16]))
    .width(Length::Fill)
    .into()
}
