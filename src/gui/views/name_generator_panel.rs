use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the name generator panel (bottom panel)
pub fn view(generated_names: &[String]) -> Element<'static, Message> {
    let header = text("NAME GENERATOR")
        .size(11)
        .color(Theme::TEXT_SECONDARY);

    let male_btn = gen_button("Male Name", Message::GenerateName("male".to_string()));
    let female_btn = gen_button("Female Name", Message::GenerateName("female".to_string()));
    let fantasy_btn = gen_button("Fantasy Name", Message::GenerateName("fantasy".to_string()));
    let place_btn = gen_button("Place Name", Message::GenerateName("place".to_string()));
    let batch_btn = gen_button("Batch (5)", Message::GenerateNameBatch);

    let button_row = row![
        male_btn,
        female_btn,
        fantasy_btn,
        place_btn,
        Space::with_width(12),
        batch_btn,
    ]
    .spacing(4)
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
        button_row,
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
        text(label.to_string()).size(12).color(Theme::TEXT_PRIMARY),
    )
    .on_press(message)
    .padding(Padding::from([4, 10]))
    .into()
}
