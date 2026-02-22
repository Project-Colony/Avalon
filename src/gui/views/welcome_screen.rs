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
        text("Ctrl+K Link | Ctrl+L DocLinks | Ctrl+J Timer | Ctrl+T Transpose").size(11).color(Theme::TEXT_MUTED),
        text("Ctrl+Shift+F Composition | Ctrl+Shift+T Script | Ctrl+Shift+G Goals").size(11).color(Theme::TEXT_MUTED),
        text("Ctrl+Shift+B Backup | Ctrl+Shift+N Name Gen | Ctrl+Shift+H History").size(11).color(Theme::TEXT_MUTED),
        text("Ctrl+5 Snapshots | Ctrl+M Bookmarks | Ctrl+P Project Notes | F7 Spellcheck").size(11).color(Theme::TEXT_MUTED),
        text("F3 Find Next | F6 Search | F8 Validation | F9 Take Snapshot").size(11).color(Theme::TEXT_MUTED),
        text("Alt+U Uppercase | Alt+L Lowercase | Alt+[ Indent | Alt+] Unindent").size(11).color(Theme::TEXT_MUTED),
    ]
    .spacing(2);

    // Tip of the day
    let tips = [
        "Use Collections to organize documents into custom groups outside the binder hierarchy.",
        "Snapshots let you save and compare versions of your document at any point.",
        "The Corkboard view shows index cards for each document in a folder.",
        "Use [[Title]] syntax to create links between your documents.",
        "Set word targets per document or per session in the Targets panel.",
        "Use labels and statuses to track progress on individual documents.",
        "The Scrivenings view stitches multiple documents together for seamless reading.",
        "Project Notes are a quick scratchpad saved with your project.",
        "Writing streaks are tracked automatically — aim for 7+ days for the fire icon!",
        "The Writing Timer (Ctrl+J) has Pomodoro, Sprint, and custom presets.",
        "Use [[Title|display text]] to create links with custom display text.",
        "Ctrl+Shift+S shows comprehensive project statistics with readability scores.",
        "The Text Stats panel (Ctrl+7) analyzes vocabulary richness and overused words.",
        "Use the Name Generator (Ctrl+Shift+N) for character and place names.",
        "Auto-backup creates safety copies every time your project is saved.",
        "F8 runs project validation to find broken links, empty docs, and orphans.",
    ];
    let tip_index = {
        use chrono::Datelike;
        chrono::Local::now().ordinal0() as usize % tips.len()
    };

    let tip_section = column![
        Space::with_height(12),
        row![
            text("\u{1F4A1}").size(12),
            Space::with_width(4),
            text("Tip:").size(11).color(Theme::TEXT_ACCENT),
            Space::with_width(4),
            text(tips[tip_index]).size(11).color(Theme::TEXT_SECONDARY),
        ],
    ];

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
        tip_section,
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
