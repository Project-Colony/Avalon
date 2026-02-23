use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input, toggler, Space};
use iced::{Element, Length, Padding};

use crate::export::compiler::{CompileOptions, OutputFormat, SeparatorType};
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the compile/export dialog
pub fn view(options: &CompileOptions, presets: &[(String, CompileOptions)]) -> Element<'static, Message> {
    let header = text("Compile Project")
        .size(20)
        .color(Theme::TEXT_PRIMARY);

    // Format selection
    let format_label = text("Output Format").size(12).color(Theme::TEXT_MUTED);
    let format_options: Vec<String> = OutputFormat::all()
        .iter()
        .map(|f| f.display_name().to_string())
        .collect();
    let current_format = options.format.display_name().to_string();
    let format_picker = pick_list(
        format_options,
        Some(current_format),
        Message::CompileSetFormat,
    )
    .width(Length::Fixed(200.0));

    // Title
    let title_label = text("Title").size(12).color(Theme::TEXT_MUTED);
    let title_input = text_input("Document title...", &options.title)
        .on_input(Message::CompileSetTitle)
        .size(14)
        .padding(6);

    // Author
    let author_label = text("Author").size(12).color(Theme::TEXT_MUTED);
    let author_input = text_input("Author name...", &options.author)
        .on_input(Message::CompileSetAuthor)
        .size(14)
        .padding(6);

    // Font family
    let font_label = text("Font Family").size(12).color(Theme::TEXT_MUTED);
    let font_options = vec![
        "Times New Roman".to_string(),
        "Arial".to_string(),
        "Courier New".to_string(),
        "Georgia".to_string(),
        "Palatino".to_string(),
    ];
    let font_picker = pick_list(
        font_options,
        Some(options.font_family.clone()),
        Message::CompileSetFontFamily,
    )
    .width(Length::Fixed(200.0));

    // Font size
    let font_size_label = text("Font Size").size(12).color(Theme::TEXT_MUTED);
    let font_size_str = format!("{:.0}", options.font_size);
    let font_size_input = text_input("12", &font_size_str)
        .on_input(Message::CompileSetFontSize)
        .size(14)
        .padding(6)
        .width(Length::Fixed(80.0));

    // Separator type
    let sep_label = text("Section Separator").size(12).color(Theme::TEXT_MUTED);
    let sep_options = vec![
        "Empty Line".to_string(),
        "Page Break".to_string(),
        "Section Break".to_string(),
        "None".to_string(),
    ];
    let current_sep = match options.separator {
        SeparatorType::EmptyLine => "Empty Line",
        SeparatorType::PageBreak => "Page Break",
        SeparatorType::SectionBreak => "Section Break",
        SeparatorType::None => "None",
        SeparatorType::Custom(_) => "Custom",
    };
    let sep_picker = pick_list(
        sep_options,
        Some(current_sep.to_string()),
        Message::CompileSetSeparator,
    )
    .width(Length::Fixed(200.0));

    // Options
    let front_matter_toggle = toggler(options.include_front_matter)
        .label("Include front matter (title page)")
        .on_toggle(Message::CompileSetFrontMatter);

    let compile_marked_toggle = toggler(options.compile_marked_only)
        .label("Only compile marked documents")
        .on_toggle(Message::CompileSetMarkedOnly);

    let page_break_toggle = toggler(options.page_break_between_folders)
        .label("Page break between chapters")
        .on_toggle(Message::CompileSetPageBreaks);

    let toc_toggle = toggler(options.include_toc)
        .label("Include table of contents")
        .on_toggle(Message::CompileSetToc);

    let placeholders_toggle = toggler(options.replace_placeholders)
        .label("Replace placeholders (<$n>, <$date>, etc.)")
        .on_toggle(Message::CompileSetPlaceholders);

    // Buttons
    let compile_btn = button(
        text("  Compile  ").size(14).color(Theme::TEXT_PRIMARY),
    )
    .on_press(Message::DoCompile)
    .padding(Padding::from([8, 20]));

    let cancel_btn = button(
        text("  Cancel  ").size(14).color(Theme::TEXT_SECONDARY),
    )
    .on_press(Message::HideCompileDialog)
    .padding(Padding::from([8, 20]));

    // Presets section
    let presets_label = text("Compile Presets").size(12).color(Theme::TEXT_MUTED);
    let mut presets_row = row![].spacing(4);
    let built_in_presets = vec![
        ("Novel".to_string(), "novel"),
        ("Manuscript".to_string(), "manuscript"),
        ("Screenplay".to_string(), "screenplay"),
    ];
    for (name, _) in &built_in_presets {
        let n = name.clone();
        presets_row = presets_row.push(
            button(
                text(name.clone()).size(10).color(Theme::TEXT_SECONDARY),
            )
            .on_press(Message::LoadCompilePreset(n))
            .padding(Padding::from([2, 6]))
        );
    }
    for (name, _) in presets {
        let n = name.clone();
        presets_row = presets_row.push(
            button(
                text(name.clone()).size(10).color(Theme::TEXT_ACCENT),
            )
            .on_press(Message::LoadCompilePreset(n))
            .padding(Padding::from([2, 6]))
        );
    }

    let save_preset_btn = button(
        text("Save Current as Preset").size(10).color(Theme::TEXT_ACCENT),
    )
    .on_press(Message::SaveCompilePreset("Custom".to_string()))
    .padding(Padding::from([2, 8]));

    let content = column![
        header,
        Space::with_height(8),
        presets_label,
        scrollable(presets_row).direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::new())),
        save_preset_btn,
        Space::with_height(12),
        format_label,
        format_picker,
        Space::with_height(12),
        title_label,
        title_input,
        Space::with_height(8),
        author_label,
        author_input,
        Space::with_height(12),
        row![
            column![font_label, font_picker].spacing(4),
            Space::with_width(16),
            column![font_size_label, font_size_input].spacing(4),
        ]
        .spacing(8),
        Space::with_height(12),
        sep_label,
        sep_picker,
        Space::with_height(16),
        front_matter_toggle,
        Space::with_height(6),
        compile_marked_toggle,
        Space::with_height(6),
        page_break_toggle,
        Space::with_height(6),
        toc_toggle,
        Space::with_height(6),
        placeholders_toggle,
        Space::with_height(24),
        row![
            compile_btn,
            Space::with_width(8),
            cancel_btn,
        ],
    ]
    .padding(24)
    .max_width(500);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
