use iced::widget::{column, container, row, text, text_input, Space};
use iced::{Element, Length, Padding};
use uuid::Uuid;

use crate::core::binder::BinderItem;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Data extracted from a BinderItem for the inspector (all owned)
pub struct InspectorData {
    pub id: Uuid,
    pub title: String,
    pub synopsis: String,
    pub status: String,
    pub label: String,
    pub word_count: String,
    pub char_count: String,
    pub paragraph_count: String,
    pub page_count: String,
    pub children_count: String,
    pub total_word_count: String,
    pub snapshot_count: String,
    pub is_document: bool,
}

impl InspectorData {
    pub fn from_item(item: &BinderItem) -> Self {
        let is_document = item.document.is_some();
        Self {
            id: item.id,
            title: item.title.clone(),
            synopsis: item.synopsis.clone(),
            status: item.metadata.status.as_ref()
                .map(|s| s.name.clone())
                .unwrap_or_else(|| "None".to_string()),
            label: item.metadata.label.as_ref()
                .map(|l| l.name.clone())
                .unwrap_or_else(|| "None".to_string()),
            word_count: item.document.as_ref()
                .map(|d| format!("{}", d.word_count()))
                .unwrap_or_default(),
            char_count: item.document.as_ref()
                .map(|d| format!("{}", d.char_count()))
                .unwrap_or_default(),
            paragraph_count: item.document.as_ref()
                .map(|d| format!("{}", d.paragraph_count()))
                .unwrap_or_default(),
            page_count: item.document.as_ref()
                .map(|d| format!("{:.1}", d.page_count()))
                .unwrap_or_default(),
            children_count: format!("{}", item.children.len()),
            total_word_count: format!("{}", item.total_word_count()),
            snapshot_count: format!("{} snapshot(s)", item.snapshots.len()),
            is_document,
        }
    }
}

/// Render the inspector panel (right sidebar)
pub fn view(data: InspectorData) -> Element<'static, Message> {
    let header = container(
        text("INSPECTOR")
            .size(12)
            .color(Theme::TEXT_SECONDARY),
    )
    .padding(Padding::from([8, 12]));

    let id = data.id;

    let title_label = text("Title").size(11).color(Theme::TEXT_MUTED);
    let title_input = text_input("Title...", &data.title)
        .on_input(move |val| Message::RenameItem(id, val))
        .size(14)
        .padding(6);

    let synopsis_label = text("Synopsis").size(11).color(Theme::TEXT_MUTED);
    let synopsis_input = text_input("Synopsis...", &data.synopsis)
        .on_input(move |val| Message::UpdateSynopsis(id, val))
        .size(13)
        .padding(6);

    let status_label = text("Status").size(11).color(Theme::TEXT_MUTED);
    let status_display = text(data.status).size(13).color(Theme::TEXT_PRIMARY);

    let label_label = text("Label").size(11).color(Theme::TEXT_MUTED);
    let label_display = text(data.label).size(13).color(Theme::TEXT_PRIMARY);

    let stats_header = text("Statistics").size(11).color(Theme::TEXT_MUTED);

    let stats_content = if data.is_document {
        column![
            stat_row("Words", data.word_count),
            stat_row("Characters", data.char_count),
            stat_row("Paragraphs", data.paragraph_count),
            stat_row("Pages (est.)", data.page_count),
        ]
        .spacing(2)
    } else {
        column![
            stat_row("Total Words", data.total_word_count),
            stat_row("Documents", data.children_count),
        ]
        .spacing(2)
    };

    let snapshots_header = text("Snapshots").size(11).color(Theme::TEXT_MUTED);
    let snapshots_info = text(data.snapshot_count).size(12).color(Theme::TEXT_SECONDARY);

    let content = column![
        header,
        Space::with_height(4),
        title_label,
        title_input,
        Space::with_height(8),
        synopsis_label,
        synopsis_input,
        Space::with_height(12),
        status_label,
        status_display,
        Space::with_height(8),
        label_label,
        label_display,
        Space::with_height(16),
        stats_header,
        stats_content,
        Space::with_height(16),
        snapshots_header,
        snapshots_info,
    ]
    .padding(12)
    .width(Length::Fixed(220.0));

    container(content)
        .height(Length::Fill)
        .into()
}

fn stat_row(label: &str, value: String) -> Element<'static, Message> {
    row![
        text(label.to_string()).size(12).color(Theme::TEXT_MUTED).width(Length::FillPortion(1)),
        text(value).size(12).color(Theme::TEXT_PRIMARY).width(Length::FillPortion(1)),
    ]
    .into()
}
