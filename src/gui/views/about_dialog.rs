use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the About window content
pub fn view() -> Element<'static, Message> {
    let logo = text("Avalon").size(32).color(Theme::TEXT_ACCENT);

    let subtitle = text("A modern writing environment")
        .size(14)
        .color(Theme::TEXT_SECONDARY);

    let version = text(format!("Version {}", env!("CARGO_PKG_VERSION")))
        .size(12)
        .color(Theme::TEXT_MUTED);

    let separator = text("\u{2500}".repeat(40)).size(10).color(Theme::BORDER);

    let description = text(
        "Avalon is a powerful writing application inspired by Scrivener, \
         built with Rust and iced. Designed for novelists, screenwriters, \
         and anyone working on long-form writing projects.",
    )
    .size(12)
    .color(Theme::TEXT_PRIMARY);

    let features_header = text("Key Features").size(13).color(Theme::TEXT_SECONDARY);

    let features = text(
        "\u{2022} Binder-based document organization\n\
         \u{2022} Corkboard and Outliner views\n\
         \u{2022} Snapshots and revision tracking\n\
         \u{2022} Compile to multiple formats\n\
         \u{2022} Writing goals and session tracking\n\
         \u{2022} Spell check and thesaurus\n\
         \u{2022} Script/screenplay mode",
    )
    .size(11)
    .color(Theme::TEXT_PRIMARY);

    let copyright = text("\u{00A9} 2024-2026 Avalon Contributors")
        .size(10)
        .color(Theme::TEXT_MUTED);

    let built_with = text("Built with Rust & iced").size(10).color(Theme::TEXT_MUTED);

    let close_btn = button(text("  Close  ").size(13).color(Theme::TEXT_PRIMARY))
        .on_press(Message::CloseAboutWindow)
        .padding(Padding::from([6, 18]));

    let content = column![
        logo,
        subtitle,
        Space::new().height(4),
        version,
        Space::new().height(10),
        separator,
        Space::new().height(10),
        description,
        Space::new().height(12),
        features_header,
        Space::new().height(4),
        features,
        Space::new().height(16),
        row![copyright, Space::new().width(Length::Fill), built_with],
        Space::new().height(12),
        close_btn,
    ]
    .padding(24)
    .spacing(2)
    .max_width(400);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
