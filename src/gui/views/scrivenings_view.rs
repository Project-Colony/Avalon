use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::core::binder::BinderItem;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render a composite Scrivenings view showing multiple documents concatenated
pub fn view<'a>(items: &[&BinderItem], parent_title: &str) -> Element<'a, Message> {
    let (total_words, total_chars, total_sentences) =
        items
            .iter()
            .filter_map(|i| i.document.as_ref())
            .fold((0usize, 0usize, 0usize), |(w, c, s), d| {
                (
                    w + d.word_count(),
                    c + d.char_count(),
                    s + d
                        .content
                        .chars()
                        .filter(|ch| *ch == '.' || *ch == '!' || *ch == '?')
                        .count(),
                )
            });

    // Reading time
    let reading_min = total_words as f64 / crate::core::READING_WPM;
    let reading_display = if reading_min < 1.0 {
        "<1 min".to_string()
    } else if reading_min < 60.0 {
        format!("~{:.0} min", reading_min)
    } else {
        format!("~{:.1}h", reading_min / 60.0)
    };

    let header = container(
        row![
            text(format!("\u{f02d} Scrivenings: {}", parent_title))
                .size(14)
                .color(Theme::TEXT_SECONDARY),
            Space::with_width(Length::Fill),
            text(format!(
                "{} docs | {} words | {:.1} pg | {}",
                items.len(),
                total_words,
                total_words as f64 / crate::core::WORDS_PER_PAGE as f64,
                reading_display
            ))
            .size(11)
            .color(Theme::TEXT_MUTED),
        ]
        .padding(Padding::from([8, 16])),
    )
    .width(Length::Fill);

    let mut content_col = column![].spacing(0);

    if items.is_empty() {
        content_col = content_col.push(
            container(
                column![
                    Space::with_height(40),
                    text("Select a folder to view its documents in Scrivenings mode.")
                        .size(14)
                        .color(Theme::TEXT_MUTED),
                    Space::with_height(8),
                    text("Scrivenings stitches multiple documents together for seamless reading.")
                        .size(12)
                        .color(Theme::TEXT_MUTED),
                ]
                .align_x(iced::Alignment::Center),
            )
            .padding(24)
            .center_x(Length::Fill),
        );
    } else {
        let mut cumulative_words: usize = 0;

        for (i, item) in items.iter().enumerate() {
            let words = item.document.as_ref().map_or(0, |d| d.word_count());
            cumulative_words += words;

            let chars = item.document.as_ref().map_or(0, |d| d.char_count());

            // Progress through the composite document
            let progress_pct = if total_words > 0 {
                (cumulative_words as f64 / total_words as f64 * 100.0) as usize
            } else {
                0
            };

            // Document title header with section number, status, and label
            let status_text = item
                .metadata
                .status
                .as_ref()
                .map_or_else(String::new, |s| format!(" [{}]", s.name));

            let label_indicator = item
                .metadata
                .label
                .as_ref()
                .map_or_else(String::new, |l| format!(" \u{f111} {}", l.name));

            let label_color = item
                .metadata
                .label
                .as_ref()
                .map_or(Theme::TEXT_ACCENT, |l| l.color.to_iced_color());

            // Section marker
            let section_marker = format!("\u{2503} {}.", i + 1);

            let doc_header = container(
                row![
                    text(section_marker).size(12).color(Theme::TEXT_ACCENT),
                    Space::with_width(6),
                    text(item.title.clone()).size(13).color(label_color),
                    Space::with_width(8),
                    text(label_indicator).size(10).color(label_color),
                    Space::with_width(Length::Fill),
                    text(format!("{} w | {} ch{}", words, chars, status_text))
                        .size(10)
                        .color(Theme::TEXT_MUTED),
                ]
                .align_y(iced::Alignment::Center),
            )
            .padding(Padding::from([12, 24]))
            .width(Length::Fill);

            content_col = content_col.push(doc_header);

            // Synopsis (if any)
            if !item.synopsis.is_empty() {
                let synopsis = container(row![
                    Space::with_width(28),
                    text("\u{f0da}").size(10).color(Theme::TEXT_MUTED),
                    Space::with_width(4),
                    text(item.synopsis.clone()).size(10).color(Theme::TEXT_MUTED),
                ])
                .padding(Padding::from([0, 24]))
                .width(Length::Fill);
                content_col = content_col.push(synopsis);
            }

            // Snapshot indicator
            if item.has_snapshots() {
                let snap_text = container(row![
                    Space::with_width(28),
                    text(format!(
                        "\u{f030} {} snapshot{}",
                        item.snapshot_count(),
                        if item.snapshot_count() == 1 { "" } else { "s" }
                    ))
                    .size(9)
                    .color(Theme::TEXT_MUTED),
                ])
                .padding(Padding::from([0, 24]))
                .width(Length::Fill);
                content_col = content_col.push(snap_text);
            }

            // Document content
            let doc_content = item.document.as_ref().map_or("", |d| d.content.as_str());

            let doc_text = container(text(doc_content.to_string()).size(14).color(Theme::TEXT_PRIMARY))
                .padding(Padding::from([4, 24]))
                .width(Length::Fill);

            content_col = content_col.push(doc_text);

            // Separator between documents with progress indicator
            if i < items.len() - 1 {
                // Mini progress bar
                let bar_width: usize = 20;
                let filled = ((progress_pct as f64 / 100.0) * bar_width as f64) as usize;
                let empty = bar_width.saturating_sub(filled);
                let bar = format!("{}{}", "\u{2501}".repeat(filled), "\u{2500}".repeat(empty));

                let separator = container(
                    row![
                        Space::with_width(24),
                        text(bar).size(8).color(Theme::BORDER),
                        Space::with_width(8),
                        text(format!(
                            "{}% | {} of {} words",
                            progress_pct, cumulative_words, total_words
                        ))
                        .size(9)
                        .color(Theme::TEXT_MUTED),
                        Space::with_width(Length::Fill),
                        text(format!("{} of {} docs", i + 1, items.len()))
                            .size(9)
                            .color(Theme::TEXT_MUTED),
                        Space::with_width(24),
                    ]
                    .align_y(iced::Alignment::Center),
                )
                .padding(Padding::from([8, 0]))
                .width(Length::Fill);

                content_col = content_col.push(separator);
            }
        }
    }

    // Enhanced footer with comprehensive stats
    let avg_words = if !items.is_empty() {
        total_words / items.len()
    } else {
        0
    };

    let footer = container(
        row![
            text(format!(
                "Total: {} words | {} chars | {} sentences | {:.1} pages | {} read",
                total_words,
                total_chars,
                total_sentences,
                total_words as f64 / crate::core::WORDS_PER_PAGE as f64,
                reading_display
            ))
            .size(10)
            .color(Theme::TEXT_MUTED),
            Space::with_width(Length::Fill),
            text(format!("Avg: {} words/doc", avg_words))
                .size(10)
                .color(Theme::TEXT_MUTED),
        ]
        .padding(Padding::from([4, 16])),
    )
    .width(Length::Fill);

    let layout = column![header, scrollable(content_col).height(Length::Fill), footer,];

    container(layout).width(Length::Fill).height(Length::Fill).into()
}
