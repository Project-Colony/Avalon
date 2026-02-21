use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::core::binder::BinderItem;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render a composite Scrivenings view showing multiple documents concatenated
pub fn view<'a>(
    items: &[&BinderItem],
    parent_title: &str,
) -> Element<'a, Message> {
    let total_words: usize = items.iter()
        .filter_map(|i| i.document.as_ref())
        .map(|d| d.word_count())
        .sum();

    let header = container(
        row![
            text(format!("Scrivenings: {}", parent_title))
                .size(14)
                .color(Theme::TEXT_SECONDARY),
            Space::with_width(Length::Fill),
            text(format!("{} docs | {} words | {:.1} pages",
                items.len(), total_words, total_words as f64 / 250.0))
                .size(11)
                .color(Theme::TEXT_MUTED),
        ]
        .padding(Padding::from([8, 16]))
    )
    .width(Length::Fill);

    let mut content_col = column![].spacing(0);

    if items.is_empty() {
        content_col = content_col.push(
            container(
                text("Select a folder to view its documents in Scrivenings mode.")
                    .size(14)
                    .color(Theme::TEXT_MUTED),
            )
            .padding(24)
        );
    } else {
        let mut cumulative_words: usize = 0;

        for (i, item) in items.iter().enumerate() {
            let words = item.document.as_ref()
                .map(|d| d.word_count())
                .unwrap_or(0);
            cumulative_words += words;

            // Document title header with section number and status
            let status_text = item.metadata.status.as_ref()
                .map(|s| format!(" [{}]", s.name))
                .unwrap_or_default();

            let label_color = item.metadata.label.as_ref()
                .map(|l| l.color.to_iced_color())
                .unwrap_or(Theme::TEXT_ACCENT);

            let doc_header = container(
                row![
                    text(format!("{}.", i + 1))
                        .size(11)
                        .color(Theme::TEXT_MUTED),
                    Space::with_width(4),
                    text(item.title.clone())
                        .size(13)
                        .color(label_color),
                    Space::with_width(Length::Fill),
                    text(format!("{} words{}", words, status_text))
                        .size(10)
                        .color(Theme::TEXT_MUTED),
                ]
                .align_y(iced::Alignment::Center)
            )
            .padding(Padding::from([12, 24]))
            .width(Length::Fill);

            content_col = content_col.push(doc_header);

            // Synopsis (if any)
            if !item.synopsis.is_empty() {
                let synopsis = container(
                    text(format!("Synopsis: {}", item.synopsis))
                        .size(10)
                        .color(Theme::TEXT_MUTED),
                )
                .padding(Padding::from([0, 24]))
                .width(Length::Fill);
                content_col = content_col.push(synopsis);
            }

            // Document content
            let doc_content = item.document.as_ref()
                .map(|d| d.content.as_str())
                .unwrap_or("");

            let doc_text = container(
                text(doc_content.to_string())
                    .size(14)
                    .color(Theme::TEXT_PRIMARY),
            )
            .padding(Padding::from([4, 24]))
            .width(Length::Fill);

            content_col = content_col.push(doc_text);

            // Separator between documents
            if i < items.len() - 1 {
                let separator = container(
                    row![
                        Space::with_width(Length::Fill),
                        text("\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}")
                            .size(10)
                            .color(Theme::BORDER),
                        Space::with_width(8),
                        text(format!("{} cumulative", cumulative_words))
                            .size(9)
                            .color(Theme::TEXT_MUTED),
                        Space::with_width(Length::Fill),
                    ]
                    .align_y(iced::Alignment::Center)
                )
                .padding(Padding::from([8, 24]))
                .width(Length::Fill);

                content_col = content_col.push(separator);
            }
        }
    }

    // Footer with reading time
    let reading_min = total_words as f64 / 250.0;
    let footer = container(
        text(format!("Total: {} words | {:.1} pages | ~{:.0} min read",
            total_words, total_words as f64 / 250.0, reading_min))
            .size(12)
            .color(Theme::TEXT_MUTED),
    )
    .padding(Padding::from([4, 16]))
    .width(Length::Fill);

    let layout = column![
        header,
        scrollable(content_col).height(Length::Fill),
        footer,
    ];

    container(layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
