use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the name generator panel (bottom panel)
pub fn view(generated_names: &[String]) -> Element<'static, Message> {
    let header = text("NAME GENERATOR")
        .size(11)
        .color(Theme::TEXT_SECONDARY);

    // Row 1: Basic name types
    let row1 = row![
        gen_button("Male", Message::GenerateName("male".to_string())),
        gen_button("Female", Message::GenerateName("female".to_string())),
        gen_button("Fantasy", Message::GenerateName("fantasy".to_string())),
        gen_button("Place", Message::GenerateName("place".to_string())),
        gen_button("Sci-Fi", Message::GenerateName("scifi".to_string())),
        Space::with_width(8),
        gen_button("Batch (5)", Message::GenerateNameBatch),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center);

    // Row 2: Culture-specific names
    let culture_label = text("Cultures:").size(10).color(Theme::TEXT_MUTED);
    let row2 = row![
        culture_label,
        Space::with_width(4),
        gen_button("JP(M)", Message::GenerateName("japanese_m".to_string())),
        gen_button("JP(F)", Message::GenerateName("japanese_f".to_string())),
        gen_button("CN(M)", Message::GenerateName("chinese_m".to_string())),
        gen_button("CN(F)", Message::GenerateName("chinese_f".to_string())),
        gen_button("ES(M)", Message::GenerateName("spanish_m".to_string())),
        gen_button("ES(F)", Message::GenerateName("spanish_f".to_string())),
        gen_button("IN(M)", Message::GenerateName("indian_m".to_string())),
        gen_button("IN(F)", Message::GenerateName("indian_f".to_string())),
        gen_button("AR(M)", Message::GenerateName("arabic_m".to_string())),
        gen_button("AR(F)", Message::GenerateName("arabic_f".to_string())),
    ]
    .spacing(3)
    .align_y(iced::Alignment::Center);

    let mut names_list = column![].spacing(2);
    for name in generated_names.iter().rev().take(20) {
        let name_clone = name.clone();
        let name_row = row![
            text(name.clone()).size(13).color(Theme::TEXT_PRIMARY),
            Space::with_width(Length::Fill),
            button(
                text("Insert").size(10).color(Theme::TEXT_ACCENT),
            )
            .on_press(Message::InsertSynonym(name_clone))
            .padding(Padding::from([2, 6])),
        ];
        names_list = names_list.push(name_row);
    }

    let content = column![
        header,
        Space::with_height(4),
        row1,
        Space::with_height(2),
        row2,
        Space::with_height(6),
        scrollable(names_list).height(Length::Fixed(120.0)),
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}

fn gen_button(label: &str, message: Message) -> Element<'static, Message> {
    button(
        text(label.to_string()).size(11).color(Theme::TEXT_PRIMARY),
    )
    .on_press(message)
    .padding(Padding::from([3, 7]))
    .into()
}
