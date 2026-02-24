use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};
use uuid::Uuid;

use crate::core::binder::BinderItem;
use crate::gui::app::Message;
use crate::gui::theme::{self, Theme};

/// Data for the quick reference panel
pub struct QuickRefData {
    pub title: String,
    pub content: String,
    pub synopsis: String,
    pub notes: String,
    pub word_count: usize,
    pub char_count: usize,
    pub paragraph_count: usize,
    pub item_id: Uuid,
    pub status: String,
    pub label: String,
    pub label_color: iced::Color,
    pub keywords: Vec<String>,
    pub modified_at: String,
    pub reading_time: f64,
}

impl QuickRefData {
    pub fn from_item(item: &BinderItem) -> Self {
        let content = item.document.as_ref()
            .map_or_else(String::new, |d| d.content.clone());
        let notes = item.document.as_ref()
            .map_or_else(String::new, |d| d.notes.clone());
        let word_count = item.document.as_ref()
            .map_or(0, |d| d.word_count());
        let char_count = item.document.as_ref()
            .map_or(0, |d| d.char_count());
        let paragraph_count = item.document.as_ref()
            .map_or(0, |d| d.paragraph_count());
        let modified_at = item.document.as_ref()
            .map_or_else(String::new, |d| d.modified_at.format("%Y-%m-%d %H:%M").to_string());
        let reading_time = word_count as f64 / crate::core::READING_WPM;

        let status = item.metadata.status.as_ref()
            .map_or_else(String::new, |s| s.name.clone());
        let label = item.metadata.label.as_ref()
            .map_or_else(String::new, |l| l.name.clone());
        let label_color = item.metadata.label.as_ref()
            .map_or(Theme::TEXT_MUTED, |l| l.color.to_iced_color());

        Self {
            title: item.title.clone(),
            content,
            synopsis: item.synopsis.clone(),
            notes,
            word_count,
            char_count,
            paragraph_count,
            item_id: item.id,
            status,
            label,
            label_color,
            keywords: item.metadata.keywords.clone(),
            modified_at,
            reading_time,
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
        text(format!("{} words | {} chars | {} para | ~{:.0}m read",
            data.word_count, data.char_count, data.paragraph_count, data.reading_time))
            .size(9)
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

    // Metadata row
    let meta_parts: Vec<String> = [
        if !data.status.is_empty() { Some(format!("Status: {}", data.status)) } else { None },
        if !data.label.is_empty() { Some(format!("Label: {}", data.label)) } else { None },
        if !data.modified_at.is_empty() { Some(format!("Modified: {}", data.modified_at)) } else { None },
    ].iter().filter_map(|x| x.clone()).collect();

    let meta_row: Element<'static, Message> = if !meta_parts.is_empty() {
        let mut r = row![].spacing(8);
        for part in meta_parts {
            r = r.push(text(part).size(10).color(Theme::TEXT_MUTED));
        }
        if !data.keywords.is_empty() {
            let kw_str = data.keywords.join(", ");
            r = r.push(text(format!("Keywords: {}", kw_str)).size(10).color(Theme::TEXT_MUTED));
        }
        r.into()
    } else {
        Space::with_height(0).into()
    };

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

    // Content preview with truncation indicator
    let max_chars = 1000;
    let is_truncated = data.content.len() > max_chars;
    let preview = data.content.chars().take(max_chars).collect::<String>();
    let content_preview = text(preview)
        .size(12)
        .color(Theme::TEXT_PRIMARY);

    let truncation_hint: Element<'static, Message> = if is_truncated {
        text(format!("... [{} more chars]", data.content.len() - max_chars))
            .size(10)
            .color(Theme::TEXT_MUTED)
            .into()
    } else {
        Space::with_height(0).into()
    };

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
        Space::with_height(2),
        meta_row,
        Space::with_height(4),
        synopsis_section,
        scrollable(
            column![content_preview, truncation_hint, notes_section]
                .padding(Padding::from([4, 0]))
        )
        .height(Length::Fixed(120.0)),
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .style(theme::panel_style)
        .width(Length::Fill)
        .into()
}
