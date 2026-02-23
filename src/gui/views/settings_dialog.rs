use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input, toggler, Space};
use iced::{Element, Length, Padding};

use crate::core::metadata::ProjectSettings;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the project settings dialog
pub fn view(
    settings: &ProjectSettings,
    project_title: &str,
    script_mode: bool,
    auto_correction: &crate::core::script::AutoCorrection,
) -> Element<'static, Message> {
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
    let labels_header = row![
        text("Labels").size(14).color(Theme::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        text(format!("{} defined", settings.labels.len()))
            .size(10).color(Theme::TEXT_MUTED),
    ];
    let mut labels_col = column![].spacing(2);
    for lbl in &settings.labels {
        let color = lbl.color.to_iced_color();
        labels_col = labels_col.push(
            row![
                text("\u{f111}").size(12).color(color),
                Space::with_width(4),
                text(lbl.name.clone()).size(12).color(Theme::TEXT_PRIMARY),
                Space::with_width(Length::Fill),
                text(format!("{:?}", lbl.color)).size(9).color(Theme::TEXT_MUTED),
            ]
            .align_y(iced::Alignment::Center)
        );
    }

    // Statuses section
    let statuses_header = row![
        text("Statuses").size(14).color(Theme::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        text(format!("{} defined", settings.statuses.len()))
            .size(10).color(Theme::TEXT_MUTED),
    ];
    let mut statuses_col = column![].spacing(2);
    for (idx, st) in settings.statuses.iter().enumerate() {
        let status_color = match st.name.as_str() {
            "To Do" => Theme::ERROR,
            "First Draft" => Theme::WARNING,
            "Revised Draft" => Theme::TEXT_ACCENT,
            "Final Draft" => Theme::SUCCESS,
            "Done" => Theme::SUCCESS,
            _ => Theme::TEXT_SECONDARY,
        };
        statuses_col = statuses_col.push(
            row![
                text(format!("{}.", idx + 1)).size(10).color(Theme::TEXT_MUTED),
                Space::with_width(4),
                text(st.name.clone()).size(12).color(status_color),
            ]
        );
    }

    // Composition mode settings
    let comp_header = text("Composition Mode").size(14).color(Theme::TEXT_SECONDARY);
    let comp_width_label = text("Text Width (%)").size(12).color(Theme::TEXT_MUTED);
    let comp_width_str = format!("{:.0}", settings.fullscreen_text_width);
    let comp_width_input = text_input("60", &comp_width_str)
        .on_input(|val| Message::SettingsSetCompWidth(val))
        .size(12)
        .padding(4)
        .width(Length::Fixed(60.0));

    let line_spacing_label = text("Line Spacing").size(12).color(Theme::TEXT_MUTED);
    let line_spacing_str = format!("{:.1}x", settings.line_spacing);
    let line_spacing_display = text(line_spacing_str).size(12).color(Theme::TEXT_PRIMARY);

    // Script mode
    let script_header = text("Script Mode").size(14).color(Theme::TEXT_SECONDARY);
    let script_toggle = toggler(script_mode)
        .label("Enable script/screenplay mode")
        .on_toggle(|_| Message::ToggleScriptMode);

    // Auto-correction
    let autocorrect_header = text("Auto-Correction").size(14).color(Theme::TEXT_SECONDARY);
    let smart_quotes_toggle = toggler(auto_correction.smart_quotes)
        .label("Smart quotes (\u{201C}...\u{201D})")
        .on_toggle(|_| Message::ToggleAutoCorrectSmartQuotes);
    let em_dash_toggle = toggler(auto_correction.em_dashes)
        .label("Em dashes (-- \u{2192} \u{2014})")
        .on_toggle(|_| Message::ToggleAutoCorrectEmDashes);
    let ellipsis_toggle = toggler(auto_correction.ellipsis)
        .label("Ellipsis (... \u{2192} \u{2026})")
        .on_toggle(|_| Message::ToggleAutoCorrectEllipsis);

    // Buttons
    let done_btn = button(
        text("  Done  ").size(14).color(Theme::TEXT_PRIMARY),
    )
    .on_press(Message::CloseSettingsWindow)
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
        Space::with_height(6),
        line_spacing_label,
        line_spacing_display,
        Space::with_height(16),
        script_header,
        script_toggle,
        Space::with_height(16),
        autocorrect_header,
        smart_quotes_toggle,
        Space::with_height(4),
        em_dash_toggle,
        Space::with_height(4),
        ellipsis_toggle,
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
