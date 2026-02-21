use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};
use uuid::Uuid;

use crate::core::binder::BinderItem;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Data for the quick reference panel
pub struct QuickRefData {
    pub title: String,
    pub content: String,
    pub synopsis: String,
    pub notes: String,
    pub word_count: usize,
    pub item_id: Uuid,
}

impl QuickRefData {
    pub fn from_item(item: &BinderItem) -> Self {
        let content = item.document.as_ref()
            .map(|d| d.content.clone())
            .unwrap_or_default();
        let notes = item.document.as_ref()
            .map(|d| d.notes.clone())
            .unwrap_or_default();
        let word_count = item.document.as_ref()
            .map(|d| d.word_count())
            .unwrap_or(0);

        Self {
            title: item.title.clone(),
            content,
            synopsis: item.synopsis.clone(),
            notes,
            word_count,
            item_id: item.id,
        }
    }
}

/// Render the quick reference bottom panel
pub fn view(data: &QuickRefData) -> Element<'static, Message> {
    let header = row![
        text(format!("QUICK REF: {}", data.title))
            .size(11)
            .color(Theme::TEXT_ACCENT),
        Space::with_width(Length::Fill),
        text(format!("{} words", data.word_count))
            .size(10)
            .color(Theme::TEXT_MUTED),
        Space::with_width(8),
        button(
            text("Open").size(10).color(Theme::TEXT_ACCENT),
        )
        .on_press(Message::SelectBinderItem(data.item_id))
        .padding(Padding::from([2, 6])),
        Space::with_width(4),
        button(
            text("Split").size(10).color(Theme::TEXT_SECONDARY),
        )
        .on_press(Message::OpenInSplitEditor(data.item_id))
        .padding(Padding::from([2, 6])),
    ];

    // Synopsis
    let synopsis_section: Element<'static, Message> = if !data.synopsis.is_empty() {
        column![
            text("Synopsis".to_string()).size(10).color(Theme::TEXT_MUTED),
            text(data.synopsis.clone()).size(11).color(Theme::TEXT_SECONDARY),
            Space::with_height(4),
        ].into()
    } else {
        Space::with_height(0).into()
    };

    // Content preview
    let preview = data.content.chars().take(1000).collect::<String>();
    let content_preview = text(preview)
        .size(12)
        .color(Theme::TEXT_PRIMARY);

    // Notes
    let notes_section: Element<'static, Message> = if !data.notes.is_empty() {
        column![
            Space::with_height(4),
            text("Notes".to_string()).size(10).color(Theme::TEXT_MUTED),
            text(data.notes.clone()).size(11).color(Theme::TEXT_SECONDARY),
        ].into()
    } else {
        Space::with_height(0).into()
    };

    let content = column![
        header,
        Space::with_height(4),
        synopsis_section,
        scrollable(
            column![content_preview, notes_section]
                .padding(Padding::from([4, 0]))
        )
        .height(Length::Fixed(120.0)),
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
