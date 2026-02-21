use iced::widget::{button, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::{BottomPanel, Message, ViewMode};
use crate::gui::theme::Theme;

/// Render the main toolbar
pub fn view(
    current_view: &ViewMode,
    show_inspector: bool,
    fullscreen: bool,
    bottom_panel: &BottomPanel,
) -> Element<'static, Message> {
    let new_btn = tool_button("New", Message::NewProject);
    let open_btn = tool_button("Open", Message::OpenProject);
    let save_btn = tool_button("Save", Message::SaveProject);
    let import_btn = tool_button("Import", Message::ImportFiles);

    let sep1 = text(" | ").size(14).color(Theme::TEXT_MUTED);

    // View mode buttons
    let editor_btn = view_button("Editor", ViewMode::Editor, current_view);
    let corkboard_btn = view_button("Cork", ViewMode::Corkboard, current_view);
    let outliner_btn = view_button("Outline", ViewMode::Outliner, current_view);
    let scrivenings_btn = view_button("Scriv", ViewMode::Scrivenings, current_view);

    let sep2 = text(" | ").size(14).color(Theme::TEXT_MUTED);

    let inspector_btn = toggle_tool_button("Insp", show_inspector, Message::ToggleInspector);

    let fullscreen_label = if fullscreen { "Exit" } else { "Focus" };
    let fullscreen_btn = tool_button(fullscreen_label, Message::ToggleFullscreen);

    let sep3 = text(" | ").size(14).color(Theme::TEXT_MUTED);

    // Bottom panel toggles - two rows via scrollable
    let search_btn = panel_button("Search", BottomPanel::Search, bottom_panel);
    let thesaurus_btn = panel_button("Thes", BottomPanel::Thesaurus, bottom_panel);
    let snapshots_btn = panel_button("Snap", BottomPanel::Snapshots, bottom_panel);
    let session_btn = panel_button("Sess", BottomPanel::Session, bottom_panel);
    let history_btn = panel_button("Hist", BottomPanel::History, bottom_panel);
    let stats_btn = panel_button("Stats", BottomPanel::TextStats, bottom_panel);
    let names_btn = panel_button("Names", BottomPanel::NameGen, bottom_panel);
    let notes_btn = panel_button("Notes", BottomPanel::ProjectNotes, bottom_panel);
    let colls_btn = panel_button("Coll", BottomPanel::Collections, bottom_panel);
    let bookmarks_btn = panel_button("Bkmk", BottomPanel::Bookmarks, bottom_panel);
    let targets_btn = panel_button("Targets", BottomPanel::Targets, bottom_panel);
    let annot_btn = panel_button("Annot", BottomPanel::Annotations, bottom_panel);
    let find_btn = panel_button("Find", BottomPanel::FindReplace, bottom_panel);
    let goals_btn = panel_button("Goals", BottomPanel::WritingGoals, bottom_panel);
    let links_btn = panel_button("Links", BottomPanel::DocLinks, bottom_panel);
    let backups_btn = panel_button("Bkups", BottomPanel::Backups, bottom_panel);

    let sep4 = text(" | ").size(14).color(Theme::TEXT_MUTED);

    let compose_btn = tool_button("Compose", Message::ToggleCompositionMode);
    let stats_dialog_btn = tool_button("ProjStats", Message::ShowProjectStats);
    let import_opml_btn = tool_button("Import+", Message::ImportOpml);
    let export_opml_btn = tool_button("OPML", Message::ExportOpml);
    let settings_btn = tool_button("Settings", Message::ShowSettings);
    let compile_btn = tool_button("Compile", Message::ShowCompileDialog);

    let toolbar_content = row![
        new_btn,
        open_btn,
        save_btn,
        import_btn,
        sep1,
        editor_btn,
        corkboard_btn,
        outliner_btn,
        scrivenings_btn,
        sep2,
        inspector_btn,
        fullscreen_btn,
        sep3,
        search_btn,
        thesaurus_btn,
        snapshots_btn,
        session_btn,
        history_btn,
        stats_btn,
        names_btn,
        notes_btn,
        colls_btn,
        bookmarks_btn,
        targets_btn,
        annot_btn,
        find_btn,
        goals_btn,
        links_btn,
        backups_btn,
        sep4,
        compose_btn,
        stats_dialog_btn,
        import_opml_btn,
        export_opml_btn,
        Space::with_width(Length::Fill),
        settings_btn,
        compile_btn,
    ]
    .spacing(2)
    .padding(Padding::from([4, 8]));

    container(
        scrollable(toolbar_content)
            .direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::new()))
    )
    .width(Length::Fill)
    .into()
}

fn tool_button(label: &str, message: Message) -> Element<'static, Message> {
    button(
        text(label.to_string()).size(12).color(Theme::TEXT_PRIMARY),
    )
    .on_press(message)
    .padding(Padding::from([3, 8]))
    .into()
}

fn view_button(label: &str, mode: ViewMode, current: &ViewMode) -> Element<'static, Message> {
    let is_active = std::mem::discriminant(&mode) == std::mem::discriminant(current);
    let color = if is_active {
        Theme::TEXT_ACCENT
    } else {
        Theme::TEXT_SECONDARY
    };

    button(
        text(label.to_string()).size(12).color(color),
    )
    .on_press(Message::SwitchView(mode))
    .padding(Padding::from([3, 8]))
    .into()
}

fn toggle_tool_button(label: &str, active: bool, message: Message) -> Element<'static, Message> {
    let color = if active {
        Theme::TEXT_ACCENT
    } else {
        Theme::TEXT_SECONDARY
    };

    button(
        text(label.to_string()).size(12).color(color),
    )
    .on_press(message)
    .padding(Padding::from([3, 8]))
    .into()
}

fn panel_button(label: &str, panel: BottomPanel, current: &BottomPanel) -> Element<'static, Message> {
    let is_active = std::mem::discriminant(&panel) == std::mem::discriminant(current);
    let color = if is_active {
        Theme::TEXT_ACCENT
    } else {
        Theme::TEXT_SECONDARY
    };

    button(
        text(label.to_string()).size(12).color(color),
    )
    .on_press(Message::ShowBottomPanel(panel))
    .padding(Padding::from([3, 8]))
    .into()
}
