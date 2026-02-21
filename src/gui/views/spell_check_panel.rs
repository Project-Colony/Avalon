use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::Message;
use crate::gui::theme::Theme;
use crate::spelling::SpellSuggestion;

/// Render the spell check results panel
pub fn view(results: &[SpellSuggestion], spell_active: bool, dict_size: usize) -> Element<'static, Message> {
    let status_icon = if spell_active { "\u{2705}" } else { "\u{26AA}" };
    let status_text = if spell_active { "Active" } else { "Inactive" };

    let header_row = row![
        text("SPELL CHECK")
            .size(11)
            .color(Theme::TEXT_SECONDARY),
        Space::with_width(8),
        text(format!("{} {}", status_icon, status_text))
            .size(10)
            .color(if spell_active { Theme::SUCCESS } else { Theme::TEXT_MUTED }),
        Space::with_width(8),
        text(format!("Dictionary: {} words", format_number(dict_size)))
            .size(10)
            .color(Theme::TEXT_MUTED),
        Space::with_width(Length::Fill),
        button(
            text(if spell_active { "Disable" } else { "Enable" })
                .size(10)
                .color(Theme::TEXT_ACCENT),
        )
        .on_press(Message::ToggleSpellChecker)
        .padding(Padding::from([2, 8])),
        Space::with_width(4),
        button(
            text("\u{21BB} Re-check").size(10).color(Theme::TEXT_ACCENT),
        )
        .on_press(Message::RunSpellCheck)
        .padding(Padding::from([2, 8])),
    ];

    // Summary line
    let summary = if results.is_empty() {
        if spell_active {
            text("\u{2713} No spelling errors found")
                .size(11)
                .color(Theme::SUCCESS)
        } else {
            text("Enable spell checker to scan document")
                .size(11)
                .color(Theme::TEXT_MUTED)
        }
    } else {
        let unique_words: Vec<&str> = results.iter().map(|r| r.word.as_str()).collect();
        let with_suggestions = results.iter().filter(|r| !r.suggestions.is_empty()).count();
        text(format!(
            "\u{26A0} {} issue(s) found ({} with suggestions, {} unknown)",
            unique_words.len(),
            with_suggestions,
            unique_words.len() - with_suggestions
        ))
        .size(11)
        .color(Theme::WARNING)
    };

    let mut result_list = column![].spacing(3);

    if results.is_empty() && spell_active {
        result_list = result_list.push(
            text("Click 'Re-check' to scan the current document for spelling errors.")
                .size(12)
                .color(Theme::TEXT_MUTED),
        );
    }

    for (idx, result) in results.iter().enumerate() {
        let word = result.word.clone();
        let index_text = format!("{}.", idx + 1);

        let mut result_row = row![
            text(index_text)
                .size(10)
                .color(Theme::TEXT_MUTED)
                .width(Length::Fixed(24.0)),
            text(format!("\"{}\"", &result.word))
                .size(12)
                .color(iced::Color::from_rgb(0.9, 0.3, 0.3)),
            Space::with_width(4),
            text("\u{2192}").size(11).color(Theme::TEXT_MUTED),
            Space::with_width(4),
        ]
        .spacing(4);

        if result.suggestions.is_empty() {
            result_row = result_row.push(
                text("(no suggestions)")
                    .size(11)
                    .color(Theme::TEXT_MUTED),
            );
        } else {
            for suggestion in &result.suggestions {
                result_row = result_row.push(
                    button(
                        text(suggestion.clone()).size(11).color(Theme::TEXT_ACCENT),
                    )
                    .on_press(Message::InsertSynonym(suggestion.clone()))
                    .padding(Padding::from([1, 4])),
                );
            }
        }

        result_row = result_row.push(Space::with_width(Length::Fill));
        result_row = result_row.push(
            button(
                text("+ Dict").size(10).color(Theme::TEXT_SECONDARY),
            )
            .on_press(Message::SpellCheckAddWord(word))
            .padding(Padding::from([1, 6])),
        );

        result_list = result_list.push(result_row);
    }

    let footer = row![
        text("Click a suggestion to replace, or '+ Dict' to learn the word")
            .size(9)
            .color(Theme::TEXT_MUTED),
        Space::with_width(Length::Fill),
        text("Shortcut: F7")
            .size(9)
            .color(Theme::TEXT_MUTED),
    ];

    let content = column![
        header_row,
        Space::with_height(4),
        summary,
        Space::with_height(4),
        scrollable(result_list).height(Length::Fixed(120.0)),
        Space::with_height(4),
        footer,
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
