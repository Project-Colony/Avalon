use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the name generator panel (bottom panel)
pub fn view(generated_names: &[String]) -> Element<'static, Message> {
    let count_text = if generated_names.is_empty() {
        String::from("No names generated yet")
    } else {
        format!("{} name(s) generated", generated_names.len())
    };

    let header = row![
        text("NAME GENERATOR")
            .size(11)
            .color(Theme::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        text(count_text).size(10).color(Theme::TEXT_MUTED),
    ];

    // Row 1: Basic name types
    let row1 = row![
        text("Type:").size(10).color(Theme::TEXT_MUTED),
        Space::with_width(4),
        gen_button("Male", Message::GenerateName("male".to_string())),
        gen_button("Female", Message::GenerateName("female".to_string())),
        gen_button("Surname", Message::GenerateName("surname".to_string())),
        gen_button("Fantasy", Message::GenerateName("fantasy".to_string())),
        gen_button("Place", Message::GenerateName("place".to_string())),
        gen_button("Sci-Fi", Message::GenerateName("scifi".to_string())),
        gen_button("Medieval", Message::GenerateName("medieval".to_string())),
        Space::with_width(8),
        gen_button("Batch (5)", Message::GenerateNameBatch),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center);

    // Row 2: Culture-specific names
    let culture_label = text("Culture:").size(10).color(Theme::TEXT_MUTED);
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

    // Row 3: Additional categories
    let row3 = row![
        text("Extra:").size(10).color(Theme::TEXT_MUTED),
        Space::with_width(4),
        gen_button("Title", Message::GenerateName("title".to_string())),
        gen_button("Nickname", Message::GenerateName("nickname".to_string())),
        gen_button("Company", Message::GenerateName("company".to_string())),
        gen_button("Ship/Vehicle", Message::GenerateName("vehicle".to_string())),
        gen_button("Tavern/Inn", Message::GenerateName("tavern".to_string())),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center);

    let mut names_list = column![].spacing(2);
    for (idx, name) in generated_names.iter().rev().take(25).enumerate() {
        let name_clone = name.clone();
        let index_str = format!("{}.", idx + 1);
        let name_row = row![
            text(index_str)
                .size(10)
                .color(Theme::TEXT_MUTED)
                .width(Length::Fixed(24.0)),
            text(name.clone()).size(13).color(Theme::TEXT_PRIMARY),
            Space::with_width(Length::Fill),
            button(
                text("Insert").size(10).color(Theme::TEXT_ACCENT),
            )
            .on_press(Message::InsertSynonym(name_clone))
            .padding(Padding::from([2, 6])),
        ]
        .align_y(iced::Alignment::Center);
        names_list = names_list.push(name_row);
    }

    if generated_names.is_empty() {
        names_list = names_list.push(
            text("Click a button above to generate names. Click 'Insert' to add to your document.")
                .size(11)
                .color(Theme::TEXT_MUTED),
        );
    }

    let content = column![
        header,
        Space::with_height(4),
        row1,
        Space::with_height(2),
        row2,
        Space::with_height(2),
        row3,
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
