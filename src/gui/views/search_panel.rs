use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length, Padding};

use crate::core::search::{self, SearchResult};
use crate::gui::app::Message;
use crate::gui::theme::{self, Theme};

/// Render the search panel (bottom panel)
pub fn view(
    query: &str,
    results: &[SearchResult],
    case_sensitive: bool,
    whole_word: bool,
    is_regex: bool,
    replace_text: &str,
) -> Element<'static, Message> {
    let header = text("SEARCH & REPLACE").size(11).color(Theme::TEXT_SECONDARY);

    // Search input row
    let search_input = text_input("Search...", query)
        .on_input(Message::SearchQueryChanged)
        .on_submit(Message::DoSearch)
        .size(13)
        .padding(6)
        .width(Length::Fixed(300.0));

    let search_btn = button(text("Search").size(12).color(Theme::TEXT_PRIMARY))
        .on_press(Message::DoSearch)
        .padding(Padding::from([4, 12]));

    // Toggle buttons for search options
    let case_btn = toggle_button("Aa", case_sensitive, Message::SearchToggleCaseSensitive);
    let word_btn = toggle_button("W", whole_word, Message::SearchToggleWholeWord);
    let regex_btn = toggle_button(".*", is_regex, Message::SearchToggleRegex);

    let search_row = row![
        search_input,
        Space::with_width(4),
        search_btn,
        Space::with_width(8),
        case_btn,
        word_btn,
        regex_btn,
    ]
    .spacing(2)
    .align_y(iced::Alignment::Center);

    // Replace row
    let replace_input = text_input("Replace with...", replace_text)
        .on_input(Message::ReplaceTextChanged)
        .size(13)
        .padding(6)
        .width(Length::Fixed(300.0));

    let replace_all_btn = button(text("Replace All").size(12).color(Theme::TEXT_PRIMARY))
        .on_press(Message::DoReplaceAll)
        .padding(Padding::from([4, 12]));

    let replace_row = row![replace_input, Space::with_width(4), replace_all_btn,]
        .spacing(2)
        .align_y(iced::Alignment::Center);

    // Results list - use search module summary functions
    let result_header = text(search::search_summary(results)).size(11).color(Theme::TEXT_MUTED);

    let mut result_list = column![].spacing(2);
    for result in results.iter().take(50) {
        let id = result.item_id;
        let title = result.item_title.clone();
        let match_count = result.match_count();
        let first_context = result.context_preview(0, 80);

        let result_row = button(
            column![
                row![
                    text(title).size(12).color(Theme::TEXT_PRIMARY),
                    Space::with_width(Length::Fill),
                    text(format!("{} match(es)", match_count))
                        .size(10)
                        .color(Theme::TEXT_MUTED),
                ],
                text(first_context).size(11).color(Theme::TEXT_SECONDARY),
            ]
            .spacing(2),
        )
        .on_press(Message::GoToSearchResult(id))
        .padding(Padding::from([4, 8]))
        .width(Length::Fill);

        result_list = result_list.push(result_row);
    }

    // Save as collection button
    let save_coll_btn: Element<'static, Message> = if !results.is_empty() {
        button(text("Save as Collection").size(10).color(Theme::TEXT_ACCENT))
            .on_press(Message::SaveSearchAsCollection)
            .padding(Padding::from([2, 8]))
            .into()
    } else {
        Space::with_height(0).into()
    };

    // Smart collection button
    let smart_coll_btn: Element<'static, Message> = if !query.is_empty() {
        button(text("Create Smart Collection").size(10).color(Theme::TEXT_SECONDARY))
            .on_press(Message::CreateSmartCollection(query.to_string()))
            .padding(Padding::from([2, 8]))
            .into()
    } else {
        Space::with_height(0).into()
    };

    let actions_row = row![save_coll_btn, Space::with_width(4), smart_coll_btn,].spacing(2);

    let content = column![
        header,
        Space::with_height(4),
        search_row,
        Space::with_height(4),
        replace_row,
        Space::with_height(4),
        row![result_header, Space::with_width(Length::Fill), actions_row],
        scrollable(result_list).height(Length::Fixed(120.0)),
    ]
    .padding(Padding::from([8, 12]));

    container(content).style(theme::panel_style).width(Length::Fill).into()
}

fn toggle_button(label: &str, active: bool, message: Message) -> Element<'static, Message> {
    let color = if active { Theme::TEXT_ACCENT } else { Theme::TEXT_MUTED };

    button(text(label.to_string()).size(12).color(color))
        .on_press(message)
        .padding(Padding::from([4, 8]))
        .into()
}
