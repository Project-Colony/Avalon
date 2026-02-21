use iced::widget::{button, column, container, row, text, text_input, toggler, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Data for the document-level find/replace panel
pub struct FindReplaceData {
    pub find_text: String,
    pub replace_text: String,
    pub match_count: usize,
    pub case_sensitive: bool,
}

/// Render the find/replace panel within current document
pub fn view(data: &FindReplaceData) -> Element<'static, Message> {
    let header = text("FIND & REPLACE IN DOCUMENT")
        .size(11)
        .color(Theme::TEXT_SECONDARY);

    let find_input = text_input("Find in document...", &data.find_text)
        .on_input(|val| Message::DocFindChanged(val))
        .on_submit(Message::DocFindNext)
        .size(12)
        .padding(4)
        .width(Length::FillPortion(3));

    let replace_input = text_input("Replace with...", &data.replace_text)
        .on_input(|val| Message::DocReplaceChanged(val))
        .size(12)
        .padding(4)
        .width(Length::FillPortion(3));

    let match_info = if data.match_count > 0 {
        format!("{} match(es) found", data.match_count)
    } else if !data.find_text.is_empty() {
        "No matches".to_string()
    } else {
        "Type to search...".to_string()
    };

    let match_color = if data.match_count > 0 {
        Theme::SUCCESS
    } else if !data.find_text.is_empty() {
        Theme::WARNING
    } else {
        Theme::TEXT_MUTED
    };

    let find_row = row![
        find_input,
        Space::with_width(4),
        button(text("Prev").size(11).color(Theme::TEXT_ACCENT))
            .on_press(Message::DocFindPrev)
            .padding(Padding::from([4, 8])),
        Space::with_width(2),
        button(text("Next").size(11).color(Theme::TEXT_ACCENT))
            .on_press(Message::DocFindNext)
            .padding(Padding::from([4, 8])),
    ]
    .align_y(iced::Alignment::Center);

    let replace_row = row![
        replace_input,
        Space::with_width(4),
        button(text("Replace").size(11).color(Theme::TEXT_ACCENT))
            .on_press(Message::DocReplaceCurrent)
            .padding(Padding::from([4, 8])),
        Space::with_width(2),
        button(text("Replace All").size(11).color(Theme::WARNING))
            .on_press(Message::DocReplaceAll)
            .padding(Padding::from([4, 8])),
    ]
    .align_y(iced::Alignment::Center);

    let info = row![
        text(match_info)
            .size(10)
            .color(match_color),
        Space::with_width(Length::Fill),
        toggler(data.case_sensitive)
            .label("Aa")
            .on_toggle(|_| Message::DocFindToggleCase),
    ];

    let content = column![
        header,
        Space::with_height(4),
        find_row,
        Space::with_height(4),
        replace_row,
        Space::with_height(4),
        info,
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
