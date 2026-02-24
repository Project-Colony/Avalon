use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::core::stats::TextAnalysis;
use crate::core::text_analysis as advanced_analysis;
use crate::core::linguistic;
use crate::gui::app::Message;
use crate::gui::theme::{self, Theme};

/// Render the text statistics panel (bottom panel)
pub fn view(analysis: &TextAnalysis, raw_text: &str) -> Element<'static, Message> {
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

    // Advanced readability from text_analysis module (6 formulas)
    let advanced = advanced_analysis::compute_readability(raw_text);
    let readability_color = if advanced.flesch_reading_ease >= 60.0 {
        Theme::SUCCESS
    } else if advanced.flesch_reading_ease >= 30.0 {
        Theme::WARNING
    } else {
        Theme::ERROR
    };

    let readability = row![
        text("Readability:").size(11).color(Theme::TEXT_MUTED),
        Space::with_width(4),
        text(format!("Flesch: {:.1}", advanced.flesch_reading_ease)).size(11).color(readability_color),
        Space::with_width(8),
        text(format!("FK Grade: {:.1}", advanced.flesch_kincaid_grade)).size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(8),
        text(format!("Fog: {:.1}", advanced.gunning_fog)).size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(8),
        text(format!("Reading: {:.0} min", analysis.reading_time_minutes)).size(11).color(Theme::TEXT_MUTED),
        Space::with_width(8),
        text(format!("Speaking: {:.0} min", analysis.speaking_time_minutes)).size(11).color(Theme::TEXT_MUTED),
    ]
    .align_y(iced::Alignment::Center);

    // Additional readability formulas
    let readability_extra = row![
        text("Coleman-Liau:").size(10).color(Theme::TEXT_MUTED),
        Space::with_width(4),
        text(format!("{:.1}", advanced.coleman_liau)).size(10).color(Theme::TEXT_SECONDARY),
        Space::with_width(12),
        text("ARI:").size(10).color(Theme::TEXT_MUTED),
        Space::with_width(4),
        text(format!("{:.1}", advanced.ari)).size(10).color(Theme::TEXT_SECONDARY),
        Space::with_width(12),
        text("SMOG:").size(10).color(Theme::TEXT_MUTED),
        Space::with_width(4),
        text(format!("{:.1}", advanced.smog)).size(10).color(Theme::TEXT_SECONDARY),
    ]
    .align_y(iced::Alignment::Center);

    // Vocabulary metrics from text_analysis module
    let vocab = advanced_analysis::vocabulary_metrics(raw_text);
    let vocab_row = row![
        text("Vocabulary:").size(11).color(Theme::TEXT_MUTED),
        Space::with_width(4),
        text(format!("TTR: {:.1}%", vocab.type_token_ratio * 100.0))
            .size(11)
            .color(if vocab.type_token_ratio >= 0.7 { Theme::SUCCESS }
                else if vocab.type_token_ratio >= 0.5 { Theme::WARNING }
                else { Theme::ERROR }),
        Space::with_width(8),
        text(format!("Hapax: {}", vocab.hapax_legomena)).size(10).color(Theme::TEXT_SECONDARY),
        Space::with_width(8),
        text(format!("Pages: {:.1}", analysis.word_count as f64 / crate::core::WORDS_PER_PAGE as f64))
            .size(11).color(Theme::TEXT_MUTED),
    ]
    .align_y(iced::Alignment::Center);

    // Linguistic analysis (passive voice, clichés, adverbs, etc.)
    let writing = linguistic::analyze_text(raw_text);
    let linguistic_level = writing.readability.level();

    let mut linguistic_row = row![
        text("Style:").size(11).color(Theme::TEXT_MUTED),
        Space::with_width(4),
        text(format!("Level: {}", linguistic_level.label()))
            .size(11)
            .color(Theme::TEXT_SECONDARY),
        Space::with_width(8),
    ]
    .spacing(4);

    if !writing.passive_voice.is_empty() {
        linguistic_row = linguistic_row.push(
            text(format!("{} passive", writing.passive_voice.len()))
                .size(10)
                .color(Theme::WARNING),
        );
        linguistic_row = linguistic_row.push(Space::with_width(8));
    }
    if !writing.adverbs.is_empty() {
        linguistic_row = linguistic_row.push(
            text(format!("{} adverbs", writing.adverbs.len()))
                .size(10)
                .color(Theme::WARNING),
        );
        linguistic_row = linguistic_row.push(Space::with_width(8));
    }
    if !writing.cliches.is_empty() {
        linguistic_row = linguistic_row.push(
            text(format!("{} clichés", writing.cliches.len()))
                .size(10)
                .color(Theme::ERROR),
        );
        linguistic_row = linguistic_row.push(Space::with_width(8));
    }
    if !writing.long_sentences.is_empty() {
        linguistic_row = linguistic_row.push(
            text(format!("{} long sentences", writing.long_sentences.len()))
                .size(10)
                .color(Theme::WARNING),
        );
        linguistic_row = linguistic_row.push(Space::with_width(8));
    }
    if !writing.dialog_tags.is_empty() {
        linguistic_row = linguistic_row.push(
            text(format!("{} dialog tags", writing.dialog_tags.len()))
                .size(10)
                .color(Theme::TEXT_MUTED),
        );
    }

    // Writing analysis summary
    let summary_text = writing.summary();
    let summary_row = text(summary_text).size(10).color(Theme::TEXT_SECONDARY);

    // Top words from text_analysis module
    let top_words = advanced_analysis::top_n_words(raw_text, 10);
    let mut common_words = row![
        text("Top words: ").size(11).color(Theme::TEXT_MUTED),
    ]
    .spacing(4);
    for wf in &top_words {
        common_words = common_words.push(
            text(format!("{}({})", wf.word, wf.count)).size(10).color(Theme::TEXT_SECONDARY)
        );
    }

    // Overused words from text_analysis module
    let overused_words = advanced_analysis::overused_words(raw_text, 5);
    let overused_row: Element<'static, Message> = if !overused_words.is_empty() {
        let mut r = row![
            text("Overused: ").size(11).color(Theme::WARNING),
        ].spacing(4);
        for wf in &overused_words {
            r = r.push(
                text(format!("{}({}x)", wf.word, wf.count)).size(10).color(Theme::WARNING)
            );
        }
        r.into()
    } else {
        Space::with_height(0).into()
    };

    // Sentence analysis
    let longest = advanced_analysis::longest_sentences(raw_text, 3);
    let avg_len = advanced_analysis::average_sentence_length(raw_text);
    let sentence_row = row![
        text("Sentences:").size(11).color(Theme::TEXT_MUTED),
        Space::with_width(4),
        text(format!("Avg: {:.1} words", avg_len)).size(10).color(Theme::TEXT_SECONDARY),
        Space::with_width(8),
        text(format!("Longest: {} words",
            longest.first().map_or(0, |s| s.word_count)
        )).size(10).color(
            if longest.first().map_or(false, |s| s.word_count > 40) { Theme::WARNING }
            else { Theme::TEXT_SECONDARY }
        ),
    ]
    .align_y(iced::Alignment::Center);

    // Paragraph analysis
    let paragraphs = advanced_analysis::paragraph_analysis(raw_text);
    let density_row = row![
        text("Density:").size(11).color(Theme::TEXT_MUTED),
        Space::with_width(4),
        text(format!("{} paragraphs", paragraphs.len())).size(10).color(Theme::TEXT_SECONDARY),
        Space::with_width(8),
        text(format!("Avg: {:.1} words/para",
            if !paragraphs.is_empty() {
                paragraphs.iter().map(|p| p.word_count).sum::<usize>() as f64 / paragraphs.len() as f64
            } else { 0.0 }
        )).size(10).color(Theme::TEXT_SECONDARY),
        Space::with_width(8),
        text(format!("Avg: {:.1} sentences/para",
            if !paragraphs.is_empty() {
                paragraphs.iter().map(|p| p.sentence_count).sum::<usize>() as f64 / paragraphs.len() as f64
            } else { 0.0 }
        )).size(10).color(Theme::TEXT_SECONDARY),
    ]
    .align_y(iced::Alignment::Center);

    // Grade level estimation using advanced readability
    let est_grade = advanced.flesch_kincaid_grade.clamp(0.0, 20.0);
    let est_audience = if est_grade <= 6.0 { "Children / General Public" }
    else if est_grade <= 9.0 { "Young Adults" }
    else if est_grade <= 13.0 { "General Adults" }
    else if est_grade <= 17.0 { "College-educated" }
    else { "Academic / Professional" };
    let grade_color = if est_grade <= 8.0 { Theme::SUCCESS }
    else if est_grade <= 12.0 { Theme::WARNING }
    else { Theme::ERROR };

    let grade_row = row![
        text("Est. Grade:").size(11).color(Theme::TEXT_MUTED),
        Space::with_width(4),
        text(format!("{:.1}", est_grade)).size(12).color(grade_color),
        Space::with_width(4),
        text(format!("({})", est_audience)).size(11).color(Theme::TEXT_SECONDARY),
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
        Space::with_height(2),
        readability_extra,
        Space::with_height(4),
        grade_row,
        Space::with_height(4),
        vocab_row,
        Space::with_height(4),
        sentence_row,
        Space::with_height(4),
        density_row,
        Space::with_height(4),
        linguistic_row,
        Space::with_height(2),
        summary_row,
        Space::with_height(4),
        scrollable(common_words),
        overused_row,
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .style(theme::panel_style)
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
