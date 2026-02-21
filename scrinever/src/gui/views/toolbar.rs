use iced::widget::{button, container, row, text, Space};
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

    let sep1 = text(" | ").size(14).color(Theme::TEXT_MUTED);

    let editor_btn = view_button("Editor", ViewMode::Editor, current_view);
    let corkboard_btn = view_button("Corkboard", ViewMode::Corkboard, current_view);
    let outliner_btn = view_button("Outliner", ViewMode::Outliner, current_view);

    let sep2 = text(" | ").size(14).color(Theme::TEXT_MUTED);

    let inspector_label = if show_inspector { "Inspector" } else { "Inspector" };
    let inspector_btn = toggle_tool_button(inspector_label, show_inspector, Message::ToggleInspector);

    let fullscreen_label = if fullscreen { "Exit Focus" } else { "Focus" };
    let fullscreen_btn = tool_button(fullscreen_label, Message::ToggleFullscreen);

    let sep3 = text(" | ").size(14).color(Theme::TEXT_MUTED);

    // Bottom panel toggles
    let search_btn = panel_button("Search", BottomPanel::Search, bottom_panel);
    let thesaurus_btn = panel_button("Thesaurus", BottomPanel::Thesaurus, bottom_panel);
    let snapshots_btn = panel_button("Snapshots", BottomPanel::Snapshots, bottom_panel);

    let compile_btn = tool_button("Compile", Message::ShowCompileDialog);

    let toolbar_content = row![
        new_btn,
        open_btn,
        save_btn,
        sep1,
        editor_btn,
        corkboard_btn,
        outliner_btn,
        sep2,
        inspector_btn,
        fullscreen_btn,
        sep3,
        search_btn,
        thesaurus_btn,
        snapshots_btn,
        Space::with_width(Length::Fill),
        compile_btn,
    ]
    .spacing(4)
    .padding(Padding::from([4, 8]));

    container(toolbar_content)
        .width(Length::Fill)
        .into()
}

fn tool_button(label: &str, message: Message) -> Element<'static, Message> {
    button(
        text(label.to_string()).size(13).color(Theme::TEXT_PRIMARY),
    )
    .on_press(message)
    .padding(Padding::from([4, 10]))
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
        text(label.to_string()).size(13).color(color),
    )
    .on_press(Message::SwitchView(mode))
    .padding(Padding::from([4, 10]))
    .into()
}

fn toggle_tool_button(label: &str, active: bool, message: Message) -> Element<'static, Message> {
    let color = if active {
        Theme::TEXT_ACCENT
    } else {
        Theme::TEXT_SECONDARY
    };

    button(
        text(label.to_string()).size(13).color(color),
    )
    .on_press(message)
    .padding(Padding::from([4, 10]))
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
        text(label.to_string()).size(13).color(color),
    )
    .on_press(Message::ShowBottomPanel(panel))
    .padding(Padding::from([4, 10]))
    .into()
}
