use iced::widget::{button, container, row, text, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::{Message, ViewMode};
use crate::gui::theme::Theme;

/// Render the main toolbar
pub fn view(current_view: &ViewMode, show_inspector: bool) -> Element<'static, Message> {
    let new_btn = tool_button("New", Message::NewProject);
    let open_btn = tool_button("Open", Message::OpenProject);
    let save_btn = tool_button("Save", Message::SaveProject);

    let separator = text(" | ").size(14).color(Theme::TEXT_MUTED);

    let editor_btn = view_button("Editor", ViewMode::Editor, current_view);
    let corkboard_btn = view_button("Corkboard", ViewMode::Corkboard, current_view);
    let outliner_btn = view_button("Outliner", ViewMode::Outliner, current_view);

    let separator2 = text(" | ").size(14).color(Theme::TEXT_MUTED);

    let inspector_label = if show_inspector { "Hide Inspector" } else { "Show Inspector" };
    let inspector_btn = tool_button(inspector_label, Message::ToggleInspector);

    let compile_btn = tool_button("Compile", Message::ShowCompileDialog);

    let toolbar_content = row![
        new_btn,
        open_btn,
        save_btn,
        separator,
        editor_btn,
        corkboard_btn,
        outliner_btn,
        separator2,
        inspector_btn,
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
