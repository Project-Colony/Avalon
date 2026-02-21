use iced::widget::{button, column, container, pick_list, row, text, text_input, toggler, Space};
use iced::{Element, Length, Padding};

use crate::export::compiler::{CompileOptions, OutputFormat, SeparatorType};
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the compile/export dialog
pub fn view(options: &CompileOptions) -> Element<'static, Message> {
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
        |selected| Message::CompileSetFormat(selected),
    )
    .width(Length::Fixed(200.0));

    // Title
    let title_label = text("Title").size(12).color(Theme::TEXT_MUTED);
    let title_input = text_input("Document title...", &options.title)
        .on_input(|val| Message::CompileSetTitle(val))
        .size(14)
        .padding(6);

    // Author
    let author_label = text("Author").size(12).color(Theme::TEXT_MUTED);
    let author_input = text_input("Author name...", &options.author)
        .on_input(|val| Message::CompileSetAuthor(val))
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
        |selected| Message::CompileSetFontFamily(selected),
    )
    .width(Length::Fixed(200.0));

    // Font size
    let font_size_label = text("Font Size").size(12).color(Theme::TEXT_MUTED);
    let font_size_str = format!("{:.0}", options.font_size);
    let font_size_input = text_input("12", &font_size_str)
        .on_input(|val| Message::CompileSetFontSize(val))
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
        |selected| Message::CompileSetSeparator(selected),
    )
    .width(Length::Fixed(200.0));

    // Options
    let front_matter_toggle = toggler(options.include_front_matter)
        .label("Include front matter (title page)")
        .on_toggle(|val| Message::CompileSetFrontMatter(val));

    let compile_marked_toggle = toggler(options.compile_marked_only)
        .label("Only compile marked documents")
        .on_toggle(|val| Message::CompileSetMarkedOnly(val));

    let page_break_toggle = toggler(options.page_break_between_folders)
        .label("Page break between chapters")
        .on_toggle(|val| Message::CompileSetPageBreaks(val));

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

    let content = column![
        header,
        Space::with_height(16),
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
