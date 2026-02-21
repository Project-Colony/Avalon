use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::core::stats::TextAnalysis;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render the text statistics panel (bottom panel)
pub fn view(analysis: &TextAnalysis) -> Element<'static, Message> {
    let header = text("TEXT STATISTICS")
        .size(11)
        .color(Theme::TEXT_SECONDARY);

    // Basic stats
    let basic_stats = row![
        stat_col("Words", &format!("{}", analysis.word_count)),
        stat_col("Unique", &format!("{}", analysis.unique_words)),
        stat_col("Chars", &format!("{}", analysis.char_count)),
        stat_col("No Spaces", &format!("{}", analysis.char_no_spaces)),
        stat_col("Sentences", &format!("{}", analysis.sentence_count)),
        stat_col("Paragraphs", &format!("{}", analysis.paragraph_count)),
    ]
    .spacing(16);

    // Averages
    let avg_stats = row![
        stat_col("Avg Word Len", &format!("{:.1}", analysis.avg_word_length)),
        stat_col("Avg Sentence", &format!("{:.1} words", analysis.avg_sentence_length)),
        stat_col("Avg Paragraph", &format!("{:.1} words", analysis.avg_paragraph_length)),
    ]
    .spacing(16);

    // Readability
    let readability_color = if analysis.readability_score >= 60.0 {
        Theme::SUCCESS
    } else if analysis.readability_score >= 30.0 {
        Theme::WARNING
    } else {
        Theme::ERROR
    };

    let readability = row![
        text("Readability:").size(11).color(Theme::TEXT_MUTED),
        Space::with_width(4),
        text(format!("{:.1}", analysis.readability_score)).size(12).color(readability_color),
        Space::with_width(4),
        text(format!("({})", analysis.readability_label())).size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(16),
        text(format!("Reading: {:.0} min", analysis.reading_time_minutes)).size(11).color(Theme::TEXT_MUTED),
        Space::with_width(8),
        text(format!("Speaking: {:.0} min", analysis.speaking_time_minutes)).size(11).color(Theme::TEXT_MUTED),
    ]
    .align_y(iced::Alignment::Center);

    // Most common words
    let mut common_words = row![
        text("Top words: ").size(11).color(Theme::TEXT_MUTED),
    ]
    .spacing(4);
    for (word, count) in analysis.most_common_words.iter().take(10) {
        common_words = common_words.push(
            text(format!("{}({})", word, count)).size(10).color(Theme::TEXT_SECONDARY)
        );
    }

    let content = column![
        header,
        Space::with_height(4),
        basic_stats,
        Space::with_height(4),
        avg_stats,
        Space::with_height(4),
        readability,
        Space::with_height(4),
        scrollable(common_words),
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}

fn stat_col(label: &str, value: &str) -> Element<'static, Message> {
    column![
        text(label.to_string()).size(10).color(Theme::TEXT_MUTED),
        text(value.to_string()).size(13).color(Theme::TEXT_PRIMARY),
    ]
    .spacing(1)
    .into()
}
