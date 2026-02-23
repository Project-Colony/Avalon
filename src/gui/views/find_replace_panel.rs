use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Element, Length, Padding};

use crate::gui::app::Message;
use crate::gui::theme::{self, Theme};

/// Data for the document-level find/replace panel
pub struct FindReplaceData {
    pub find_text: String,
    pub replace_text: String,
    pub match_count: usize,
    pub case_sensitive: bool,
    pub current_match: usize,
    pub whole_word: bool,
    pub use_regex: bool,
}

impl FindReplaceData {
    pub fn new() -> Self {
        Self {
            find_text: String::new(),
            replace_text: String::new(),
            match_count: 0,
            case_sensitive: false,
            current_match: 0,
            whole_word: false,
            use_regex: false,
        }
    }

    /// Check if we have an active search
    pub fn has_query(&self) -> bool {
        !self.find_text.is_empty()
    }

    /// Check if there are matches to navigate
    pub fn has_matches(&self) -> bool {
        self.match_count > 0
    }

    /// Reset search state
    pub fn clear(&mut self) {
        self.find_text.clear();
        self.replace_text.clear();
        self.match_count = 0;
        self.current_match = 0;
    }
}

/// Render the find/replace panel within current document
pub fn view(data: &FindReplaceData) -> Element<'static, Message> {
    let header = row![
        text("FIND & REPLACE").size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(4),
        text("\u{f002}").size(10),
        Space::with_width(Length::Fill),
        text("Ctrl+F: find | Ctrl+H: replace | Esc: close")
            .size(9)
            .color(Theme::TEXT_MUTED),
    ];

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

    // Match status with color-coded feedback and position indicator
    let (match_icon, match_info, match_color) = if data.match_count > 0 {
        let position_text = if data.match_count == 1 {
            "1 match".to_string()
        } else {
            format!(
                "{} of {} matches",
                (data.current_match + 1).min(data.match_count),
                data.match_count
            )
        };
        ("\u{f00c}", position_text, Theme::SUCCESS)
    } else if !data.find_text.is_empty() {
        ("\u{f00d}", "No matches found".to_string(), Theme::WARNING)
    } else {
        ("\u{2026}", "Type to search...".to_string(), Theme::TEXT_MUTED)
    };

    let find_row = row![
        find_input,
        Space::with_width(4),
        text(match_icon).size(12).color(match_color),
        Space::with_width(4),
        button(text("\u{f062} Prev").size(10).color(Theme::TEXT_ACCENT))
            .on_press(Message::DocFindPrev)
            .padding(Padding::from([4, 8])),
        Space::with_width(2),
        button(text("Next \u{f063}").size(10).color(Theme::TEXT_ACCENT))
            .on_press(Message::DocFindNext)
            .padding(Padding::from([4, 8])),
    ]
    .align_y(iced::Alignment::Center);

    let replace_row = row![
        replace_input,
        Space::with_width(4),
        Space::with_width(14),
        Space::with_width(4),
        button(text("Replace").size(10).color(Theme::TEXT_ACCENT))
            .on_press(Message::DocReplaceCurrent)
            .padding(Padding::from([4, 8])),
        Space::with_width(2),
        button(text("Replace All").size(10).color(Theme::WARNING))
            .on_press(Message::DocReplaceAll)
            .padding(Padding::from([4, 8])),
    ]
    .align_y(iced::Alignment::Center);

    let case_label = if data.case_sensitive { "Aa" } else { "Aa" };
    let case_color = if data.case_sensitive { Theme::TEXT_ACCENT } else { Theme::TEXT_MUTED };
    let word_label = if data.whole_word { "W" } else { "W" };
    let word_color = if data.whole_word { Theme::TEXT_ACCENT } else { Theme::TEXT_MUTED };
    let regex_label = if data.use_regex { ".*" } else { ".*" };
    let regex_color = if data.use_regex { Theme::TEXT_ACCENT } else { Theme::TEXT_MUTED };

    let mode_hint = if data.use_regex {
        "Regex mode active"
    } else if data.whole_word {
        "Whole word matching"
    } else {
        "Substring matching"
    };

    let info = row![
        text(match_info)
            .size(10)
            .color(match_color),
        Space::with_width(12),
        button(text(case_label).size(11).color(case_color))
            .on_press(Message::DocFindToggleCase)
            .padding(Padding::from([2, 6])),
        Space::with_width(4),
        button(text(word_label).size(11).color(word_color))
            .on_press(Message::DocFindToggleWholeWord)
            .padding(Padding::from([2, 6])),
        Space::with_width(4),
        button(text(regex_label).size(11).color(regex_color))
            .on_press(Message::DocFindToggleRegex)
            .padding(Padding::from([2, 6])),
        Space::with_width(8),
        text(mode_hint).size(9).color(Theme::TEXT_MUTED),
        Space::with_width(Length::Fill),
        text("Enter: next | Shift+Enter: prev")
            .size(9)
            .color(Theme::TEXT_MUTED),
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
        .style(theme::panel_style)
        .width(Length::Fill)
        .into()
}
