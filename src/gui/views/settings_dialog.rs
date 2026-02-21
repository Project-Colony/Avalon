use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input, toggler, Space};
use iced::{Element, Length, Padding};

use crate::core::metadata::ProjectSettings;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the project settings dialog
pub fn view(settings: &ProjectSettings, project_title: &str) -> Element<'static, Message> {
    let header = text("Project Settings")
        .size(20)
        .color(Theme::TEXT_PRIMARY);

    // Project title
    let title_label = text("Project Title").size(12).color(Theme::TEXT_MUTED);
    let title_input = text_input("Project title...", project_title)
        .on_input(|val| Message::SettingsSetProjectTitle(val))
        .size(14)
        .padding(6);

    // Editor font
    let font_label = text("Editor Font").size(12).color(Theme::TEXT_MUTED);
    let font_options = vec![
        "monospace".to_string(),
        "serif".to_string(),
        "sans-serif".to_string(),
    ];
    let font_picker = pick_list(
        font_options,
        Some(settings.editor_font.clone()),
        |selected| Message::SettingsSetFont(selected),
    )
    .width(Length::Fixed(200.0));

    // Font size
    let font_size_label = text("Font Size").size(12).color(Theme::TEXT_MUTED);
    let font_size_str = format!("{:.0}", settings.editor_font_size);
    let font_size_input = text_input("16", &font_size_str)
        .on_input(|val| Message::SettingsSetFontSize(val))
        .size(14)
        .padding(6)
        .width(Length::Fixed(80.0));

    // Editor zoom
    let zoom_label = text("Editor Zoom").size(12).color(Theme::TEXT_MUTED);
    let zoom_str = format!("{:.0}%", settings.editor_zoom * 100.0);
    let zoom_row = row![
        button(text("-").size(14).color(Theme::TEXT_PRIMARY))
            .on_press(Message::SettingsZoomOut)
            .padding(Padding::from([4, 10])),
        text(zoom_str).size(14).color(Theme::TEXT_PRIMARY),
        button(text("+").size(14).color(Theme::TEXT_PRIMARY))
            .on_press(Message::SettingsZoomIn)
            .padding(Padding::from([4, 10])),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Target word count
    let target_label = text("Project Target Words").size(12).color(Theme::TEXT_MUTED);
    let target_str = settings.target_word_count
        .map(|t| t.to_string())
        .unwrap_or_default();
    let target_input = text_input("e.g. 80000", &target_str)
        .on_input(|val| Message::SettingsSetTarget(val))
        .size(14)
        .padding(6)
        .width(Length::Fixed(150.0));

    // Deadline
    let deadline_label = text("Target Deadline").size(12).color(Theme::TEXT_MUTED);
    let deadline_str = settings.target_deadline.clone().unwrap_or_default();
    let deadline_input = text_input("YYYY-MM-DD", &deadline_str)
        .on_input(|val| Message::SettingsSetDeadline(val))
        .size(14)
        .padding(6)
        .width(Length::Fixed(150.0));

    // Auto-save interval
    let autosave_label = text("Auto-save Interval (seconds)").size(12).color(Theme::TEXT_MUTED);
    let autosave_str = format!("{}", settings.auto_save_seconds);
    let autosave_input = text_input("30", &autosave_str)
        .on_input(|val| Message::SettingsSetAutoSave(val))
        .size(14)
        .padding(6)
        .width(Length::Fixed(80.0));

    // Show word count in footer
    let word_count_toggle = toggler(settings.show_word_count)
        .label("Show word count in status bar")
        .on_toggle(|val| Message::SettingsToggleWordCount(val));

    // Labels section
    let labels_header = text("Labels").size(14).color(Theme::TEXT_SECONDARY);
    let mut labels_col = column![].spacing(2);
    for lbl in &settings.labels {
        let color = lbl.color.to_iced_color();
        labels_col = labels_col.push(
            row![
                text("*").size(14).color(color),
                Space::with_width(4),
                text(lbl.name.clone()).size(12).color(Theme::TEXT_PRIMARY),
            ]
        );
    }

    // Statuses section
    let statuses_header = text("Statuses").size(14).color(Theme::TEXT_SECONDARY);
    let mut statuses_col = column![].spacing(2);
    for st in &settings.statuses {
        statuses_col = statuses_col.push(
            text(format!("  {}", st.name)).size(12).color(Theme::TEXT_PRIMARY),
        );
    }

    // Composition mode settings
    let comp_header = text("Composition Mode").size(14).color(Theme::TEXT_SECONDARY);
    let comp_width_label = text("Text Width (%)").size(12).color(Theme::TEXT_MUTED);
    let comp_width_str = format!("{:.0}", settings.fullscreen_text_width);
    let comp_width_input = text_input("60", &comp_width_str)
        .on_input(|val| {
            if let Ok(w) = val.parse::<f32>() {
                if (10.0..=100.0).contains(&w) {
                    // Accept valid width — handled in settings
                }
            }
            Message::SettingsSetFont(val) // placeholder
        })
        .size(12)
        .padding(4)
        .width(Length::Fixed(60.0));

    // Buttons
    let done_btn = button(
        text("  Done  ").size(14).color(Theme::TEXT_PRIMARY),
    )
    .on_press(Message::HideSettings)
    .padding(Padding::from([8, 20]));

    let content = column![
        header,
        Space::with_height(16),
        title_label,
        title_input,
        Space::with_height(12),
        font_label,
        font_picker,
        Space::with_height(8),
        font_size_label,
        font_size_input,
        Space::with_height(12),
        zoom_label,
        zoom_row,
        Space::with_height(12),
        target_label,
        target_input,
        Space::with_height(8),
        deadline_label,
        deadline_input,
        Space::with_height(12),
        autosave_label,
        autosave_input,
        Space::with_height(12),
        word_count_toggle,
        Space::with_height(16),
        labels_header,
        labels_col,
        Space::with_height(12),
        statuses_header,
        statuses_col,
        Space::with_height(16),
        comp_header,
        comp_width_label,
        comp_width_input,
        Space::with_height(24),
        done_btn,
    ]
    .padding(24)
    .max_width(500);

    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
