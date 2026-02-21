use iced::widget::{column, container, row, scrollable, text, text_editor, Space};
use iced::{Element, Length, Padding};

use crate::core::binder::BinderItem;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Render a composite Scrivenings view showing multiple documents concatenated
pub fn view<'a>(
    items: &[&BinderItem],
    parent_title: &str,
) -> Element<'a, Message> {
    let header = container(
        row![
            text(format!("Scrivenings: {}", parent_title))
                .size(14)
                .color(Theme::TEXT_SECONDARY),
            Space::with_width(Length::Fill),
            text(format!("{} documents", items.len()))
                .size(12)
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
        for (i, item) in items.iter().enumerate() {
            // Document title header
            let doc_header = container(
                text(item.title.clone())
                    .size(13)
                    .color(Theme::TEXT_ACCENT),
            )
            .padding(Padding::from([12, 24]))
            .width(Length::Fill);

            content_col = content_col.push(doc_header);

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

            // Word count for this section
            let words = item.document.as_ref()
                .map(|d| d.word_count())
                .unwrap_or(0);

            let doc_footer = container(
                text(format!("{} words", words))
                    .size(10)
                    .color(Theme::TEXT_MUTED),
            )
            .padding(Padding::from([0, 24]))
            .width(Length::Fill);

            content_col = content_col.push(doc_footer);

            // Separator between documents
            if i < items.len() - 1 {
                let separator = container(
                    text("- - - - - - - - - -")
                        .size(12)
                        .color(Theme::TEXT_MUTED),
                )
                .padding(Padding::from([8, 24]))
                .width(Length::Fill)
                .center_x(Length::Fill);

                content_col = content_col.push(separator);
            }
        }
    }

    // Total word count
    let total_words: usize = items.iter()
        .filter_map(|i| i.document.as_ref())
        .map(|d| d.word_count())
        .sum();

    let footer = container(
        text(format!("Total: {} words | {:.1} pages", total_words, total_words as f64 / 250.0))
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
