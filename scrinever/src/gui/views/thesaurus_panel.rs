use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::Message;
use crate::gui::theme::Theme;
use crate::thesaurus::ThesaurusEntry;

/// Render the thesaurus panel (bottom panel)
pub fn view(
    query: &str,
    results: &[ThesaurusEntry],
) -> Element<'static, Message> {
    let header = text("THESAURUS")
        .size(11)
        .color(Theme::TEXT_SECONDARY);

    let input = text_input("Look up a word...", query)
        .on_input(|val| Message::ThesaurusQueryChanged(val))
        .on_submit(Message::DoThesaurusLookup)
        .size(13)
        .padding(6)
        .width(Length::Fixed(300.0));

    let lookup_btn = button(
        text("Look Up").size(12).color(Theme::TEXT_PRIMARY),
    )
    .on_press(Message::DoThesaurusLookup)
    .padding(Padding::from([4, 12]));

    let input_row = row![input, Space::with_width(4), lookup_btn]
        .align_y(iced::Alignment::Center);

    let mut entries = column![].spacing(8);

    if results.is_empty() && !query.is_empty() {
        entries = entries.push(
            text("No entries found. Try loading WordNet data or another word.")
                .size(12)
                .color(Theme::TEXT_MUTED),
        );
    }

    for entry in results.iter().take(10) {
        let pos_text = format!("({})", entry.part_of_speech);
        let def_text = if entry.definition.len() > 120 {
            format!("{}...", &entry.definition[..120])
        } else {
            entry.definition.clone()
        };

        let mut synonyms_row = row![].spacing(4);
        for syn in entry.synonyms.iter().take(8) {
            let word = syn.clone();
            let syn_btn = button(
                text(syn.clone()).size(11).color(Theme::TEXT_ACCENT),
            )
            .on_press(Message::InsertSynonym(word))
            .padding(Padding::from([2, 6]));
            synonyms_row = synonyms_row.push(syn_btn);
        }

        let entry_widget = column![
            row![
                text(pos_text).size(11).color(Theme::TEXT_MUTED),
                Space::with_width(8),
                text(def_text).size(12).color(Theme::TEXT_SECONDARY),
            ],
            row![
                text("Synonyms: ").size(11).color(Theme::TEXT_MUTED),
                synonyms_row,
            ],
        ]
        .spacing(2);

        entries = entries.push(entry_widget);
    }

    let content = column![
        header,
        Space::with_height(4),
        input_row,
        Space::with_height(8),
        scrollable(entries).height(Length::Fixed(140.0)),
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
