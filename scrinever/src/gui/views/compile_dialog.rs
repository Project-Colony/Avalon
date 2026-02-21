use iced::widget::{button, column, container, pick_list, row, text, text_input, toggler, Space};
use iced::{Element, Length, Padding};

use crate::export::compiler::{CompileOptions, OutputFormat};
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

    // Options
    let front_matter_toggle = toggler(options.include_front_matter)
        .label("Include front matter (title page)")
        .on_toggle(|val| Message::CompileSetFrontMatter(val));

    let compile_marked_toggle = toggler(options.compile_marked_only)
        .label("Only compile marked documents")
        .on_toggle(|val| Message::CompileSetMarkedOnly(val));

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
        Space::with_height(16),
        front_matter_toggle,
        Space::with_height(8),
        compile_marked_toggle,
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
