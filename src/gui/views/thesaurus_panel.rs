use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::Message;
use crate::gui::theme::{self, Theme};
use crate::thesaurus::ThesaurusEntry;

/// Render the thesaurus panel (bottom panel)
pub fn view(
    query: &str,
    results: &[ThesaurusEntry],
) -> Element<'static, Message> {
    let result_count = if results.is_empty() {
        String::new()
    } else {
        let total_synonyms: usize = results.iter().map(|e| e.synonyms.len()).sum();
        format!("{} sense(s), {} synonym(s)", results.len(), total_synonyms)
    };

    let header = row![
        text("THESAURUS")
            .size(11)
            .color(Theme::TEXT_SECONDARY),
        Space::with_width(8),
        text(result_count).size(10).color(Theme::TEXT_MUTED),
        Space::with_width(Length::Fill),
        text("Tip: Select a word in the editor, then open Thesaurus")
            .size(9)
            .color(Theme::TEXT_MUTED),
    ];

    let input = text_input("Look up a word...", query)
        .on_input(Message::ThesaurusQueryChanged)
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
            text("No entries found. Try a different word or check spelling.")
                .size(12)
                .color(Theme::TEXT_MUTED),
        );
    } else if results.is_empty() {
        entries = entries.push(
            text("Type a word above and press Enter or click Look Up.")
                .size(12)
                .color(Theme::TEXT_MUTED),
        );
    }

    for (idx, entry) in results.iter().take(12).enumerate() {
        let pos_text = format!("{}. ({})", idx + 1, entry.part_of_speech);
        let def_text = if entry.definition.len() > 140 {
            format!("{}...", &entry.definition[..140])
        } else {
            entry.definition.clone()
        };

        // Synonyms row — click to insert, right-click hint to look up
        let mut synonyms_row = row![].spacing(4);
        for syn in entry.synonyms.iter().take(10) {
            let word_insert = syn.clone();
            let syn_btn = button(
                text(syn.clone()).size(11).color(Theme::TEXT_ACCENT),
            )
            .on_press(Message::InsertSynonym(word_insert))
            .padding(Padding::from([2, 6]));
            synonyms_row = synonyms_row.push(syn_btn);
        }
        if entry.synonyms.len() > 10 {
            synonyms_row = synonyms_row.push(
                text(format!("+{} more", entry.synonyms.len() - 10))
                    .size(10)
                    .color(Theme::TEXT_MUTED),
            );
        }

        // Antonyms row (if any)
        let antonyms_widget: Element<'static, Message> = if !entry.antonyms.is_empty() {
            let mut ant_row = row![
                text("Antonyms: ").size(10).color(Theme::TEXT_MUTED),
            ]
            .spacing(4);
            for ant in entry.antonyms.iter().take(6) {
                ant_row = ant_row.push(
                    text(ant.clone())
                        .size(11)
                        .color(iced::Color::from_rgb(0.8, 0.4, 0.4)),
                );
            }
            ant_row.into()
        } else {
            Space::with_height(0).into()
        };

        let entry_widget = column![
            row![
                text(pos_text).size(11).color(Theme::WARNING),
                Space::with_width(8),
                text(def_text).size(12).color(Theme::TEXT_SECONDARY),
            ],
            row![
                text("Synonyms: ").size(10).color(Theme::TEXT_MUTED),
                synonyms_row,
            ],
            antonyms_widget,
        ]
        .spacing(2);

        entries = entries.push(entry_widget);
    }

    let footer = row![
        text("Click a synonym to insert it at cursor position")
            .size(9)
            .color(Theme::TEXT_MUTED),
        Space::with_width(Length::Fill),
        text("Shortcut: Ctrl+Shift+T")
            .size(9)
            .color(Theme::TEXT_MUTED),
    ];

    let content = column![
        header,
        Space::with_height(4),
        input_row,
        Space::with_height(8),
        scrollable(entries).height(Length::Fixed(140.0)),
        Space::with_height(4),
        footer,
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .style(theme::panel_style)
        .width(Length::Fill)
        .into()
}
