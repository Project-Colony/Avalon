use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::{BottomPanel, Message, ToolbarMenu, ViewMode};
use crate::gui::theme::Theme;

/// Render the menu bar (category headers) + dropdown panel if a menu is open
pub fn view(
    current_view: &ViewMode,
    show_inspector: bool,
    fullscreen: bool,
    bottom_panel: &BottomPanel,
    active_menu: &Option<ToolbarMenu>,
) -> Element<'static, Message> {
    // Menu category buttons
    let file_menu = menu_header("File", &ToolbarMenu::File, active_menu);
    let view_menu = menu_header("View", &ToolbarMenu::View, active_menu);
    let panels_menu = menu_header("Panels", &ToolbarMenu::Panels, active_menu);
    let tools_menu = menu_header("Tools", &ToolbarMenu::Tools, active_menu);

    let menu_bar = container(
        row![
            file_menu,
            view_menu,
            panels_menu,
            tools_menu,
        ]
        .spacing(2)
        .padding(Padding::from([4, 8]))
    )
    .width(Length::Fill);

    // Dropdown panel for the active menu
    let dropdown: Element<'static, Message> = match active_menu {
        Some(ToolbarMenu::File) => file_dropdown(),
        Some(ToolbarMenu::View) => view_dropdown(current_view, show_inspector, fullscreen),
        Some(ToolbarMenu::Panels) => panels_dropdown(bottom_panel),
        Some(ToolbarMenu::Tools) => tools_dropdown(),
        None => Space::with_height(0).into(),
    };

    column![menu_bar, dropdown].into()
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
    .into()
}

/// File menu dropdown: New, Open, Save, Import, Export, Print, Compile
fn file_dropdown() -> Element<'static, Message> {
    dropdown_container(
        row![
            dropdown_btn("New", Message::NewProject),
            dropdown_btn("Open", Message::OpenProject),
            dropdown_btn("Save", Message::SaveProject),
            dropdown_sep(),
            dropdown_btn("Import", Message::ImportFiles),
            dropdown_btn("Import OPML", Message::ImportOpml),
            dropdown_btn("Export OPML", Message::ExportOpml),
            dropdown_sep(),
            dropdown_btn("Print", Message::PrintCurrent),
            dropdown_btn("Print All", Message::PrintProject),
            dropdown_sep(),
            dropdown_btn("Compile", Message::ShowCompileDialog),
        ]
        .spacing(2)
    )
}

/// View menu dropdown: view modes, inspector, focus, compose
fn view_dropdown(current_view: &ViewMode, show_inspector: bool, fullscreen: bool) -> Element<'static, Message> {
    let fullscreen_label = if fullscreen { "Exit Focus" } else { "Focus Mode" };

    dropdown_container(
        row![
            view_btn("Editor", ViewMode::Editor, current_view),
            view_btn("Corkboard", ViewMode::Corkboard, current_view),
            view_btn("Outliner", ViewMode::Outliner, current_view),
            view_btn("Scrivenings", ViewMode::Scrivenings, current_view),
            dropdown_sep(),
            toggle_btn("Inspector", show_inspector, Message::ToggleInspector),
            dropdown_btn(fullscreen_label, Message::ToggleFullscreen),
            dropdown_btn("Compose", Message::ToggleCompositionMode),
        ]
        .spacing(2)
    )
}

/// Panels menu dropdown: all bottom panel toggles
fn panels_dropdown(bottom_panel: &BottomPanel) -> Element<'static, Message> {
    dropdown_container(
        row![
            panel_btn("Search", BottomPanel::Search, bottom_panel),
            panel_btn("Find", BottomPanel::FindReplace, bottom_panel),
            panel_btn("Thesaurus", BottomPanel::Thesaurus, bottom_panel),
            panel_btn("Spell", BottomPanel::SpellCheck, bottom_panel),
            dropdown_sep(),
            panel_btn("Session", BottomPanel::Session, bottom_panel),
            panel_btn("Timer", BottomPanel::Timer, bottom_panel),
            panel_btn("Goals", BottomPanel::WritingGoals, bottom_panel),
            panel_btn("Targets", BottomPanel::Targets, bottom_panel),
            panel_btn("Stats", BottomPanel::TextStats, bottom_panel),
            dropdown_sep(),
            panel_btn("Snapshots", BottomPanel::Snapshots, bottom_panel),
            panel_btn("History", BottomPanel::History, bottom_panel),
            panel_btn("Backups", BottomPanel::Backups, bottom_panel),
            dropdown_sep(),
            panel_btn("Notes", BottomPanel::ProjectNotes, bottom_panel),
            panel_btn("Annot", BottomPanel::Annotations, bottom_panel),
            panel_btn("Bookmarks", BottomPanel::Bookmarks, bottom_panel),
            panel_btn("Collections", BottomPanel::Collections, bottom_panel),
            panel_btn("Links", BottomPanel::DocLinks, bottom_panel),
            dropdown_sep(),
            panel_btn("Names", BottomPanel::NameGen, bottom_panel),
            panel_btn("Templates", BottomPanel::Templates, bottom_panel),
            panel_btn("Validation", BottomPanel::Validation, bottom_panel),
        ]
        .spacing(2)
    )
}

/// Tools menu dropdown: settings, project stats
fn tools_dropdown() -> Element<'static, Message> {
    dropdown_container(
        row![
            dropdown_btn("Settings", Message::ShowSettings),
            dropdown_btn("Project Stats", Message::ShowProjectStats),
        ]
        .spacing(2)
    )
}

/// Wrap dropdown items in a styled container
fn dropdown_container(content: iced::widget::Row<'static, Message>) -> Element<'static, Message> {
    container(content)
        .padding(Padding::from([4, 12]))
        .width(Length::Fill)
        .into()
}

/// Regular dropdown button
fn dropdown_btn(label: &str, message: Message) -> Element<'static, Message> {
    button(
        text(label.to_string()).size(12).color(Theme::TEXT_PRIMARY),
    )
    .on_press(message)
    .padding(Padding::from([3, 8]))
    .into()
}

/// View mode button (highlighted when active)
fn view_btn(label: &str, mode: ViewMode, current: &ViewMode) -> Element<'static, Message> {
    let is_active = std::mem::discriminant(&mode) == std::mem::discriminant(current);
    let color = if is_active { Theme::TEXT_ACCENT } else { Theme::TEXT_SECONDARY };

    button(
        text(label.to_string()).size(12).color(color),
    )
    .on_press(Message::SwitchView(mode))
    .padding(Padding::from([3, 8]))
    .into()
}

/// Toggle button (highlighted when active)
fn toggle_btn(label: &str, active: bool, message: Message) -> Element<'static, Message> {
    let color = if active { Theme::TEXT_ACCENT } else { Theme::TEXT_SECONDARY };

    button(
        text(label.to_string()).size(12).color(color),
    )
    .on_press(message)
    .padding(Padding::from([3, 8]))
    .into()
}

/// Panel toggle button (highlighted when active)
fn panel_btn(label: &str, panel: BottomPanel, current: &BottomPanel) -> Element<'static, Message> {
    let is_active = std::mem::discriminant(&panel) == std::mem::discriminant(current);
    let color = if is_active { Theme::TEXT_ACCENT } else { Theme::TEXT_SECONDARY };

    button(
        text(label.to_string()).size(12).color(color),
    )
    .on_press(Message::ShowBottomPanel(panel))
    .padding(Padding::from([3, 8]))
    .into()
}

/// Visual separator between groups
fn dropdown_sep() -> Element<'static, Message> {
    text(" | ").size(12).color(Theme::TEXT_MUTED).into()
}
