use iced::widget::{button, container, mouse_area, row, text, column, stack, Space};
use iced::{Border, Element, Length, Padding};

use crate::gui::app::{BottomPanel, Message, ToolbarMenu, ViewMode};
use crate::gui::theme::{self, Theme};

/// Transparent button style with hover highlight for menu items
fn menu_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(iced::Background::Color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.1))),
        button::Status::Pressed => Some(iced::Background::Color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15))),
        _ => None,
    };

    button::Style {
        background: bg,
        text_color: Theme::TEXT_PRIMARY,
        border: Border {
            color: iced::Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// Render the menu bar (just the category header buttons)
pub fn menu_bar(active_menu: &Option<ToolbarMenu>) -> Element<'static, Message> {
    let file_menu = menu_header("File", &ToolbarMenu::File, active_menu);
    let view_menu = menu_header("View", &ToolbarMenu::View, active_menu);
    let panels_menu = menu_header("Panels", &ToolbarMenu::Panels, active_menu);
    let tools_menu = menu_header("Tools", &ToolbarMenu::Tools, active_menu);

    container(
        row![file_menu, view_menu, panels_menu, tools_menu]
            .spacing(2)
            .padding(Padding::from([4, 8]))
    )
    .style(theme::toolbar_style)
    .width(Length::Fill)
    .into()
}

/// Render the floating dropdown overlay (to be stacked on top of main content).
/// Returns None if no menu is open.
pub fn dropdown_overlay(
    current_view: &ViewMode,
    show_inspector: bool,
    fullscreen: bool,
    bottom_panel: &BottomPanel,
    active_menu: &Option<ToolbarMenu>,
) -> Option<Element<'static, Message>> {
    let (dropdown_content, left_offset) = match active_menu {
        Some(ToolbarMenu::File) => (file_dropdown(), 8),
        Some(ToolbarMenu::View) => (view_dropdown(current_view, show_inspector, fullscreen), 62),
        Some(ToolbarMenu::Panels) => (panels_dropdown(bottom_panel), 116),
        Some(ToolbarMenu::Tools) => (tools_dropdown(), 184),
        None => return None,
    };

    // The floating dropdown: positioned with a left offset to align under its header
    let positioned_dropdown = row![
        Space::with_width(left_offset),
        dropdown_content,
    ];

    // Full-screen click-away layer (behind the dropdown)
    let click_away = mouse_area(
        Space::new(Length::Fill, Length::Fill)
    )
    .on_press(Message::CloseToolbarMenu);

    // Stack: click-away fills the whole screen, dropdown floats on top
    // at its natural width
    let overlay = stack![
        click_away,
        positioned_dropdown,
    ];

    Some(
        container(overlay)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    )
}

/// A menu header button that toggles its dropdown
fn menu_header(label: &str, menu: &ToolbarMenu, active: &Option<ToolbarMenu>) -> Element<'static, Message> {
    let is_open = active.as_ref() == Some(menu);
    let color = if is_open { Theme::TEXT_ACCENT } else { Theme::TEXT_PRIMARY };

    button(
        text(label.to_string()).size(13).color(color),
    )
    .on_press(Message::ToggleToolbarMenu(menu.clone()))
    .padding(Padding::from([5, 12]))
    .style(menu_button_style)
    .into()
}

/// File menu dropdown (vertical)
fn file_dropdown() -> Element<'static, Message> {
    dropdown_panel(column![
        dropdown_btn("New", Message::NewProject),
        dropdown_btn("Open", Message::OpenProject),
        dropdown_btn("Save", Message::SaveProject),
        dropdown_separator(),
        dropdown_btn("Import", Message::ImportFiles),
        dropdown_btn("Import OPML", Message::ImportOpml),
        dropdown_btn("Export OPML", Message::ExportOpml),
        dropdown_separator(),
        dropdown_btn("Print", Message::PrintCurrent),
        dropdown_btn("Print All", Message::PrintProject),
        dropdown_separator(),
        dropdown_btn("Compile", Message::ShowCompileDialog),
    ]
    .spacing(1))
}

/// View menu dropdown (vertical)
fn view_dropdown(current_view: &ViewMode, show_inspector: bool, fullscreen: bool) -> Element<'static, Message> {
    let fullscreen_label = if fullscreen { "Exit Focus" } else { "Focus Mode" };

    dropdown_panel(column![
        view_btn("Editor", ViewMode::Editor, current_view),
        view_btn("Corkboard", ViewMode::Corkboard, current_view),
        view_btn("Outliner", ViewMode::Outliner, current_view),
        view_btn("Scrivenings", ViewMode::Scrivenings, current_view),
        dropdown_separator(),
        toggle_btn("Inspector", show_inspector, Message::ToggleInspector),
        dropdown_btn(fullscreen_label, Message::ToggleFullscreen),
        dropdown_btn("Compose", Message::ToggleCompositionMode),
    ]
    .spacing(1))
}

/// Panels menu dropdown (vertical, with section headers)
fn panels_dropdown(bottom_panel: &BottomPanel) -> Element<'static, Message> {
    dropdown_panel(column![
        section_label("Search"),
        panel_btn("Search", BottomPanel::Search, bottom_panel),
        panel_btn("Find & Replace", BottomPanel::FindReplace, bottom_panel),
        panel_btn("Thesaurus", BottomPanel::Thesaurus, bottom_panel),
        panel_btn("Spell Check", BottomPanel::SpellCheck, bottom_panel),
        dropdown_separator(),
        section_label("Writing"),
        panel_btn("Session", BottomPanel::Session, bottom_panel),
        panel_btn("Timer", BottomPanel::Timer, bottom_panel),
        panel_btn("Goals", BottomPanel::WritingGoals, bottom_panel),
        panel_btn("Targets", BottomPanel::Targets, bottom_panel),
        panel_btn("Stats", BottomPanel::TextStats, bottom_panel),
        dropdown_separator(),
        section_label("History"),
        panel_btn("Snapshots", BottomPanel::Snapshots, bottom_panel),
        panel_btn("History", BottomPanel::History, bottom_panel),
        panel_btn("Backups", BottomPanel::Backups, bottom_panel),
        dropdown_separator(),
        section_label("Organization"),
        panel_btn("Notes", BottomPanel::ProjectNotes, bottom_panel),
        panel_btn("Annotations", BottomPanel::Annotations, bottom_panel),
        panel_btn("Bookmarks", BottomPanel::Bookmarks, bottom_panel),
        panel_btn("Collections", BottomPanel::Collections, bottom_panel),
        panel_btn("Doc Links", BottomPanel::DocLinks, bottom_panel),
        dropdown_separator(),
        section_label("Misc"),
        panel_btn("Name Gen", BottomPanel::NameGen, bottom_panel),
        panel_btn("Templates", BottomPanel::Templates, bottom_panel),
        panel_btn("Validation", BottomPanel::Validation, bottom_panel),
    ]
    .spacing(1))
}

/// Tools menu dropdown (vertical)
fn tools_dropdown() -> Element<'static, Message> {
    dropdown_panel(column![
        dropdown_btn("Settings", Message::ShowSettings),
        dropdown_btn("Project Stats", Message::ShowProjectStats),
    ]
    .spacing(1))
}

/// Styled floating dropdown panel container
fn dropdown_panel(content: iced::widget::Column<'static, Message>) -> Element<'static, Message> {
    container(content)
        .padding(Padding::from([6, 4]))
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(Theme::BG_TOOLBAR)),
            border: iced::Border {
                color: Theme::BORDER,
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: iced::Vector::new(0.0, 3.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        })
        .into()
}

/// Section label (non-clickable header within a dropdown)
fn section_label(label: &str) -> Element<'static, Message> {
    container(
        text(label.to_string()).size(10).color(Theme::TEXT_MUTED),
    )
    .padding(Padding::from([2, 8]))
    .into()
}

/// Horizontal separator line
fn dropdown_separator() -> Element<'static, Message> {
    container(
        container(Space::with_height(1))
            .style(|_theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(Theme::BORDER_SUBTLE)),
                ..Default::default()
            })
    )
    .padding(Padding::from([3, 8]))
    .into()
}

/// Regular dropdown menu item
fn dropdown_btn(label: &str, message: Message) -> Element<'static, Message> {
    button(
        text(label.to_string()).size(12).color(Theme::TEXT_PRIMARY),
    )
    .on_press(message)
    .padding(Padding::from([4, 12]))
    .style(menu_button_style)
    .into()
}

/// View mode menu item (highlighted when active)
fn view_btn(label: &str, mode: ViewMode, current: &ViewMode) -> Element<'static, Message> {
    let is_active = std::mem::discriminant(&mode) == std::mem::discriminant(current);
    let color = if is_active { Theme::TEXT_ACCENT } else { Theme::TEXT_PRIMARY };

    button(
        text(label.to_string()).size(12).color(color),
    )
    .on_press(Message::SwitchView(mode))
    .padding(Padding::from([4, 12]))
    .style(menu_button_style)
    .into()
}

/// Toggle menu item (highlighted when active)
fn toggle_btn(label: &str, active: bool, message: Message) -> Element<'static, Message> {
    let color = if active { Theme::TEXT_ACCENT } else { Theme::TEXT_PRIMARY };

    button(
        text(label.to_string()).size(12).color(color),
    )
    .on_press(message)
    .padding(Padding::from([4, 12]))
    .style(menu_button_style)
    .into()
}

/// Panel toggle menu item (highlighted when active)
fn panel_btn(label: &str, panel: BottomPanel, current: &BottomPanel) -> Element<'static, Message> {
    let is_active = std::mem::discriminant(&panel) == std::mem::discriminant(current);
    let color = if is_active { Theme::TEXT_ACCENT } else { Theme::TEXT_PRIMARY };

    button(
        text(label.to_string()).size(12).color(color),
    )
    .on_press(Message::ShowBottomPanel(panel))
    .padding(Padding::from([4, 12]))
    .style(menu_button_style)
    .into()
}
