use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input, toggler, Space};
use iced::{Element, Length, Padding};
use uuid::Uuid;

use crate::core::binder::BinderItem;
use crate::core::metadata::ProjectSettings;
use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// Data extracted from a BinderItem for the inspector (all owned)
pub struct InspectorData {
    pub id: Uuid,
    pub title: String,
    pub synopsis: String,
    pub notes: String,
    pub status: String,
    pub label: String,
    pub word_count: String,
    pub char_count: String,
    pub paragraph_count: String,
    pub sentence_count: String,
    pub page_count: String,
    pub children_count: String,
    pub total_word_count: String,
    pub snapshot_count: String,
    pub is_document: bool,
    pub include_in_compile: bool,
    pub target_word_count: Option<usize>,
    pub available_statuses: Vec<String>,
    pub available_labels: Vec<String>,
    pub keywords: Vec<String>,
    pub is_bookmarked: bool,
    pub footnote_count: usize,
    pub reference_count: usize,
    pub custom_fields: Vec<(String, String)>,
}

impl InspectorData {
    pub fn from_item(
        item: &BinderItem,
        notes: &str,
        target: Option<usize>,
        settings: &ProjectSettings,
        is_bookmarked: bool,
    ) -> Self {
        let is_document = item.document.is_some();
        Self {
            id: item.id,
            title: item.title.clone(),
            synopsis: item.synopsis.clone(),
            notes: notes.to_string(),
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
            sentence_count: item.document.as_ref()
                .map(|d| format!("{}", d.sentence_count()))
                .unwrap_or_default(),
            page_count: item.document.as_ref()
                .map(|d| format!("{:.1}", d.page_count()))
                .unwrap_or_default(),
            children_count: format!("{}", item.children.len()),
            total_word_count: format!("{}", item.total_word_count()),
            snapshot_count: format!("{} snapshot(s)", item.snapshots.len()),
            is_document,
            include_in_compile: item.include_in_compile,
            target_word_count: target,
            available_statuses: {
                let mut s: Vec<String> = vec!["None".to_string()];
                s.extend(settings.statuses.iter().map(|st| st.name.clone()));
                s
            },
            available_labels: {
                let mut l: Vec<String> = vec!["None".to_string()];
                l.extend(settings.labels.iter().map(|lb| lb.name.clone()));
                l
            },
            keywords: item.metadata.keywords.clone(),
            is_bookmarked,
            footnote_count: item.document.as_ref()
                .map(|d| d.footnotes.len())
                .unwrap_or(0),
            reference_count: item.document.as_ref()
                .map(|d| d.references.len())
                .unwrap_or(0),
            custom_fields: item.metadata.custom_metadata.iter()
                .map(|f| {
                    let val = match &f.value {
                        crate::core::metadata::CustomFieldValue::Text(t) => t.clone(),
                        crate::core::metadata::CustomFieldValue::Number(n) => format!("{}", n),
                        crate::core::metadata::CustomFieldValue::Checkbox(b) => if *b { "Yes".to_string() } else { "No".to_string() },
                        crate::core::metadata::CustomFieldValue::Date(d) => d.clone(),
                        crate::core::metadata::CustomFieldValue::List(l) => l.join(", "),
                    };
                    (f.name.clone(), val)
                })
                .collect(),
        }
    }
}

/// Render the inspector panel (right sidebar)
pub fn view(data: InspectorData) -> Element<'static, Message> {
    let id = data.id;

    let bookmark_label = if data.is_bookmarked { "Unbookmark" } else { "Bookmark" };
    let bookmark_color = if data.is_bookmarked { Theme::WARNING } else { Theme::TEXT_MUTED };

    let header = container(
        row![
            text("INSPECTOR").size(12).color(Theme::TEXT_SECONDARY),
            Space::with_width(Length::Fill),
            button(
                text(bookmark_label).size(10).color(bookmark_color),
            )
            .on_press(Message::ToggleBookmark(id))
            .padding(Padding::from([2, 6])),
        ]
    )
    .padding(Padding::from([8, 12]));

    // Title
    let title_label = text("Title").size(11).color(Theme::TEXT_MUTED);
    let title_input = text_input("Title...", &data.title)
        .on_input(move |val| Message::RenameItem(id, val))
        .size(14)
        .padding(6);

    // Synopsis
    let synopsis_label = text("Synopsis").size(11).color(Theme::TEXT_MUTED);
    let synopsis_input = text_input("Synopsis...", &data.synopsis)
        .on_input(move |val| Message::UpdateSynopsis(id, val))
        .size(13)
        .padding(6);

    // Status picker
    let status_label = text("Status").size(11).color(Theme::TEXT_MUTED);
    let status_picker = pick_list(
        data.available_statuses,
        Some(data.status),
        move |val| Message::SetItemStatus(id, val),
    )
    .width(Length::Fill);

    // Label picker
    let label_label = text("Label").size(11).color(Theme::TEXT_MUTED);
    let label_picker = pick_list(
        data.available_labels,
        Some(data.label),
        move |val| Message::SetItemLabel(id, val),
    )
    .width(Length::Fill);

    // Include in compile toggle
    let compile_toggle = toggler(data.include_in_compile)
        .label("Include in Compile")
        .on_toggle(move |_| Message::ToggleIncludeInCompile(id));

    // Notes
    let notes_label = text("Notes").size(11).color(Theme::TEXT_MUTED);
    let notes_input = text_input("Document notes...", &data.notes)
        .on_input(|val| Message::NotesChanged(val))
        .size(12)
        .padding(6);

    // Target word count
    let target_label = text("Target Words").size(11).color(Theme::TEXT_MUTED);
    let target_value = data.target_word_count
        .map(|t| t.to_string())
        .unwrap_or_default();
    let target_input = text_input("e.g. 50000", &target_value)
        .on_input(move |val| Message::SetItemTarget(id, val))
        .size(12)
        .padding(6);

    // Progress bar (if target set)
    let progress_row: Element<'static, Message> = if let (Some(target), true) = (data.target_word_count, data.is_document) {
        let words: usize = data.word_count.parse().unwrap_or(0);
        let pct = (words as f64 / target as f64 * 100.0).min(100.0);
        let bar = format!("{:.1}% ({}/{})", pct, words, target);
        text(bar).size(11).color(Theme::SUCCESS).into()
    } else {
        Space::with_height(0).into()
    };

    // Statistics
    let stats_header = text("Statistics").size(11).color(Theme::TEXT_MUTED);

    let stats_content = if data.is_document {
        column![
            stat_row("Words", data.word_count),
            stat_row("Characters", data.char_count),
            stat_row("Paragraphs", data.paragraph_count),
            stat_row("Sentences", data.sentence_count),
            stat_row("Pages (est.)", data.page_count),
        ]
        .spacing(2)
    } else {
        column![
            stat_row("Total Words", data.total_word_count),
            stat_row("Children", data.children_count),
        ]
        .spacing(2)
    };

    // Keywords editing
    let keywords_label = text("Keywords").size(11).color(Theme::TEXT_MUTED);
    let keywords_text = data.keywords.join(", ");
    let keywords_input = text_input("keyword1, keyword2...", &keywords_text)
        .on_input(move |val| Message::SetItemKeywords(id, val))
        .size(11)
        .padding(4);

    // Convert to folder/text buttons
    let convert_row: Element<'static, Message> = if data.is_document {
        button(
            text("Convert to Folder").size(10).color(Theme::TEXT_MUTED),
        )
        .on_press(Message::ConvertToFolder(id))
        .padding(Padding::from([2, 6]))
        .into()
    } else {
        row![
            button(
                text("Merge Children").size(10).color(Theme::TEXT_MUTED),
            )
            .on_press(Message::MergeIntoParent)
            .padding(Padding::from([2, 6])),
        ]
        .into()
    };

    // Split button (only for documents)
    let split_btn: Element<'static, Message> = if data.is_document {
        button(
            text("Split at Midpoint").size(10).color(Theme::TEXT_MUTED),
        )
        .on_press(Message::SplitDocument)
        .padding(Padding::from([2, 6]))
        .into()
    } else {
        Space::with_height(0).into()
    };

    // Snapshots
    let snapshots_header = text("Snapshots").size(11).color(Theme::TEXT_MUTED);
    let snapshots_row = row![
        text(data.snapshot_count).size(12).color(Theme::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        button(
            text("Take").size(11).color(Theme::TEXT_ACCENT),
        )
        .on_press(Message::CreateSnapshot)
        .padding(Padding::from([2, 8])),
    ];

    // Additional info (footnotes, refs)
    let extra_info: Element<'static, Message> = if data.footnote_count > 0 || data.reference_count > 0 {
        column![
            stat_row("Footnotes", format!("{}", data.footnote_count)),
            stat_row("References", format!("{}", data.reference_count)),
        ]
        .spacing(2)
        .into()
    } else {
        Space::with_height(0).into()
    };

    // Custom metadata
    let mut custom_col = column![].spacing(2);
    if !data.custom_fields.is_empty() {
        custom_col = custom_col.push(
            text("Custom Metadata").size(11).color(Theme::TEXT_MUTED)
        );
        for (name, value) in &data.custom_fields {
            custom_col = custom_col.push(stat_row(name, value.clone()));
        }
    }

    // Quick ref button
    let quick_ref_btn = button(
        text("Quick Reference").size(10).color(Theme::TEXT_ACCENT),
    )
    .on_press(Message::ShowQuickRef(id))
    .padding(Padding::from([2, 6]));

    // Split editor button
    let split_editor_btn = button(
        text("Open in Split").size(10).color(Theme::TEXT_SECONDARY),
    )
    .on_press(Message::OpenInSplitEditor(id))
    .padding(Padding::from([2, 6]));

    let content = column![
        header,
        Space::with_height(4),
        title_label,
        title_input,
        Space::with_height(6),
        synopsis_label,
        synopsis_input,
        Space::with_height(8),
        status_label,
        status_picker,
        Space::with_height(6),
        label_label,
        label_picker,
        Space::with_height(8),
        compile_toggle,
        Space::with_height(8),
        notes_label,
        notes_input,
        Space::with_height(8),
        target_label,
        target_input,
        progress_row,
        Space::with_height(12),
        stats_header,
        stats_content,
        extra_info,
        Space::with_height(8),
        keywords_label,
        keywords_input,
        Space::with_height(8),
        custom_col,
        Space::with_height(12),
        snapshots_header,
        snapshots_row,
        Space::with_height(8),
        row![quick_ref_btn, Space::with_width(4), split_editor_btn].spacing(2),
        Space::with_height(4),
        convert_row,
        split_btn,
    ]
    .padding(12)
    .width(Length::Fixed(240.0));

    container(scrollable(content))
        .height(Length::Fill)
        .into()
}

fn stat_row(label: &str, value: String) -> Element<'static, Message> {
    row![
        text(label.to_string()).size(11).color(Theme::TEXT_MUTED).width(Length::FillPortion(1)),
        text(value).size(11).color(Theme::TEXT_PRIMARY).width(Length::FillPortion(1)),
    ]
    .into()
}
