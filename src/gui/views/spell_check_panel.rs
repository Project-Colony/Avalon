use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::Message;
use crate::gui::theme::Theme;
use crate::spelling::SpellSuggestion;

/// Render the spell check results panel
pub fn view(results: &[SpellSuggestion], spell_active: bool, dict_size: usize) -> Element<'static, Message> {
    let header_row = row![
        text("SPELL CHECK")
            .size(11)
            .color(Theme::TEXT_SECONDARY),
        Space::with_width(8),
        text(format!("(Dictionary: {} words)", dict_size))
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
            text("Re-check").size(10).color(Theme::TEXT_ACCENT),
        )
        .on_press(Message::RunSpellCheck)
        .padding(Padding::from([2, 8])),
    ];

    let mut result_list = column![].spacing(3);

    if results.is_empty() {
        if spell_active {
            result_list = result_list.push(
                text("No spelling errors found. Click 'Re-check' to scan the current document.")
                    .size(12)
                    .color(Theme::TEXT_MUTED),
            );
        } else {
            result_list = result_list.push(
                text("Spell checker is disabled. Click 'Enable' to activate.")
                    .size(12)
                    .color(Theme::TEXT_MUTED),
            );
        }
    }

    for result in results {
        let word = result.word.clone();

        let mut result_row = row![
            text(format!("\"{}\"", &result.word))
                .size(12)
                .color(iced::Color::from_rgb(0.9, 0.3, 0.3)),
            Space::with_width(8),
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
                    text(format!(" {} ", suggestion))
                        .size(11)
                        .color(Theme::TEXT_ACCENT),
                );
            }
        }

        result_row = result_row.push(Space::with_width(Length::Fill));
        result_row = result_row.push(
            button(
                text("Add to Dict").size(10).color(Theme::TEXT_SECONDARY),
            )
            .on_press(Message::SpellCheckAddWord(word))
            .padding(Padding::from([1, 6])),
        );

        result_list = result_list.push(result_row);
    }

    let content = column![
        header_row,
        Space::with_height(4),
        text(format!("{} issue(s) found", results.len()))
            .size(11)
            .color(Theme::TEXT_MUTED),
        Space::with_height(4),
        scrollable(result_list).height(Length::Fixed(120.0)),
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
