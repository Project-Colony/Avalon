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

    // Overused words (words appearing much more than average)
    let avg_freq = if !analysis.most_common_words.is_empty() {
        analysis.most_common_words.iter().map(|(_, c)| *c).sum::<usize>() as f64
            / analysis.most_common_words.len() as f64
    } else {
        1.0
    };
    let overused: Vec<&(String, usize)> = analysis.most_common_words.iter()
        .filter(|(_, c)| *c as f64 > avg_freq * 2.0)
        .take(5)
        .collect();

    let overused_row: Element<'static, Message> = if !overused.is_empty() {
        let mut r = row![
            text("Overused: ").size(11).color(Theme::WARNING),
        ].spacing(4);
        for (word, count) in overused {
            r = r.push(
                text(format!("{}({}x)", word, count)).size(10).color(Theme::WARNING)
            );
        }
        r.into()
    } else {
        Space::with_height(0).into()
    };

    // Vocabulary richness (type-token ratio)
    let ttr = if analysis.word_count > 0 {
        analysis.unique_words as f64 / analysis.word_count as f64 * 100.0
    } else {
        0.0
    };
    let ttr_label = if ttr >= 70.0 { "Rich" }
    else if ttr >= 50.0 { "Moderate" }
    else { "Repetitive" };
    let ttr_color = if ttr >= 70.0 { Theme::SUCCESS }
    else if ttr >= 50.0 { Theme::WARNING }
    else { Theme::ERROR };

    let vocab_row = row![
        text("Vocabulary:").size(11).color(Theme::TEXT_MUTED),
        Space::with_width(4),
        text(format!("{:.1}% unique", ttr)).size(11).color(ttr_color),
        Space::with_width(4),
        text(format!("({})", ttr_label)).size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(16),
        text(format!("Pages: {:.1}", analysis.word_count as f64 / 250.0))
            .size(11).color(Theme::TEXT_MUTED),
    ]
    .align_y(iced::Alignment::Center);

    // Syllable and word length distribution estimate
    let long_words: usize = analysis.most_common_words.iter()
        .filter(|(w, _)| w.len() > 8)
        .count();
    let short_words: usize = analysis.most_common_words.iter()
        .filter(|(w, _)| w.len() <= 4)
        .count();
    let total_common = analysis.most_common_words.len().max(1);

    let complexity_row = row![
        text("Complexity:").size(11).color(Theme::TEXT_MUTED),
        Space::with_width(4),
        text(format!("{}% short", short_words * 100 / total_common)).size(10).color(Theme::SUCCESS),
        Space::with_width(8),
        text(format!("{}% long", long_words * 100 / total_common)).size(10).color(Theme::WARNING),
        Space::with_width(16),
        text(format!("Syllables/word: ~{:.1}", analysis.avg_word_length * 0.6))
            .size(10).color(Theme::TEXT_MUTED),
    ]
    .align_y(iced::Alignment::Center);

    let content = column![
        header,
        Space::with_height(4),
        basic_stats,
        Space::with_height(4),
        avg_stats,
        Space::with_height(4),
        readability,
        Space::with_height(4),
        vocab_row,
        Space::with_height(4),
        complexity_row,
        Space::with_height(4),
        scrollable(common_words),
        overused_row,
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
