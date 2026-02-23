use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input, toggler, Space};
use iced::{Element, Length, Padding};
use uuid::Uuid;

use crate::core::binder::BinderItem;
use crate::core::metadata::ProjectSettings;
use crate::gui::app::Message;
use crate::gui::theme::{self, Icons, Theme};

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

// ── Helper widgets ──────────────────────────────────────────────────

/// Thin horizontal separator line
fn separator() -> Element<'static, Message> {
    container(Space::with_height(0))
        .width(Length::Fill)
        .height(1)
        .style(theme::inspector_separator_style)
        .into()
}

/// Section header with icon and label
fn section_header(icon: &str, label: &str) -> Element<'static, Message> {
    text(format!("{}  {}", icon, label))
        .size(10)
        .color(Theme::TEXT_ACCENT)
        .into()
}

/// Field label (smaller, muted)
fn field_label(label: &str) -> Element<'static, Message> {
    text(label.to_string())
        .size(10)
        .color(Theme::TEXT_MUTED)
        .into()
}

/// Statistic row with label and value
fn stat_row(label: &str, value: String) -> Element<'static, Message> {
    row![
        text(label.to_string())
            .size(11)
            .color(Theme::TEXT_MUTED)
            .width(Length::FillPortion(3)),
        text(value)
            .size(11)
            .color(Theme::TEXT_PRIMARY)
            .width(Length::FillPortion(2)),
    ]
    .align_y(iced::Alignment::Center)
    .into()
}

/// Wrap content in a section card container
fn section_card(content: impl Into<Element<'static, Message>>) -> Element<'static, Message> {
    container(content)
        .width(Length::Fill)
        .padding(Padding::from([8, 10]))
        .style(theme::inspector_section_style)
        .into()
}

// ── Main view ───────────────────────────────────────────────────────

/// Render the inspector panel (right sidebar)
pub fn view(data: InspectorData) -> Element<'static, Message> {
    let id = data.id;

    // ── Header ──────────────────────────────────────────────────
    let bookmark_icon = if data.is_bookmarked {
        Icons::STAR
    } else {
        Icons::STAR_O
    };

    let header = container(
        row![
            text(format!("{}  INSPECTOR", Icons::INFO_CIRCLE))
                .size(11)
                .color(Theme::TEXT_ACCENT),
            Space::with_width(Length::Fill),
            button(
                text(bookmark_icon)
                    .size(12)
                    .line_height(1.0)
                    .color(if data.is_bookmarked { Theme::WARNING } else { Theme::TEXT_MUTED }),
            )
            .on_press(Message::ToggleBookmark(id))
            .style(theme::inspector_bookmark_style(data.is_bookmarked))
            .padding(Padding { top: 3.0, right: 6.0, bottom: 3.0, left: 2.0 }),
        ]
        .align_y(iced::Alignment::Center)
    )
    .width(Length::Fill)
    .padding(Padding::from([10, 14]))
    .style(theme::inspector_header_style);

    // ── Document info (Title + Synopsis) ────────────────────────
    let title_input = text_input("Title...", &data.title)
        .on_input(move |val| Message::RenameItem(id, val))
        .size(14)
        .padding(6);

    let synopsis_input = text_input("Synopsis...", &data.synopsis)
        .on_input(move |val| Message::UpdateSynopsis(id, val))
        .size(12)
        .padding(6);

    let doc_info_section = column![
        field_label("Title"),
        title_input,
        Space::with_height(6),
        field_label("Synopsis"),
        synopsis_input,
    ]
    .spacing(2);

    // ── Metadata section (Status, Label, Compile) ───────────────
    let status_picker = pick_list(
        data.available_statuses,
        Some(data.status),
        move |val| Message::SetItemStatus(id, val),
    )
    .width(Length::Fill);

    let label_picker = pick_list(
        data.available_labels,
        Some(data.label),
        move |val| Message::SetItemLabel(id, val),
    )
    .width(Length::Fill);

    let compile_toggle = toggler(data.include_in_compile)
        .label("Include in Compile")
        .on_toggle(move |_| Message::ToggleIncludeInCompile(id));

    let metadata_section = section_card(
        column![
            section_header(Icons::TAG, "METADATA"),
            Space::with_height(6),
            field_label("Status"),
            status_picker,
            Space::with_height(6),
            field_label("Label"),
            label_picker,
            Space::with_height(8),
            compile_toggle,
        ]
        .spacing(2)
    );

    // ── Target & Progress ───────────────────────────────────────
    let target_value = data.target_word_count
        .map(|t| t.to_string())
        .unwrap_or_default();
    let target_input = text_input("e.g. 50000", &target_value)
        .on_input(move |val| Message::SetItemTarget(id, val))
        .size(12)
        .padding(6);

    let progress_row: Element<'static, Message> = if let (Some(target), true) = (data.target_word_count, data.is_document) {
        let words: usize = data.word_count.parse().unwrap_or(0);
        let pct = (words as f64 / target as f64 * 100.0).min(100.0);
        let progress_color = Theme::progress_color(pct);
        let bar_text = theme::progress_bar_text(pct, 18);

        column![
            Space::with_height(4),
            text(bar_text).size(10).color(progress_color),
            text(format!("{:.1}%  ({}/{})", pct, words, target))
                .size(10)
                .color(Theme::TEXT_MUTED),
        ]
        .spacing(2)
        .into()
    } else {
        Space::with_height(0).into()
    };

    let target_section = section_card(
        column![
            section_header(Icons::BOLT, "TARGET"),
            Space::with_height(6),
            field_label("Word Count Goal"),
            target_input,
            progress_row,
        ]
        .spacing(2)
    );

    // ── Statistics ──────────────────────────────────────────────
    let stats_content: Element<'static, Message> = if data.is_document {
        let mut stats = column![
            stat_row("Words", data.word_count.clone()),
            stat_row("Characters", data.char_count),
            stat_row("Paragraphs", data.paragraph_count),
            stat_row("Sentences", data.sentence_count),
            stat_row("Pages (est.)", data.page_count),
        ]
        .spacing(3);

        if data.footnote_count > 0 || data.reference_count > 0 {
            stats = stats.push(Space::with_height(2));
            if data.footnote_count > 0 {
                stats = stats.push(stat_row("Footnotes", format!("{}", data.footnote_count)));
            }
            if data.reference_count > 0 {
                stats = stats.push(stat_row("References", format!("{}", data.reference_count)));
            }
        }

        stats.into()
    } else {
        column![
            stat_row("Total Words", data.total_word_count),
            stat_row("Children", data.children_count),
        ]
        .spacing(3)
        .into()
    };

    let stats_section = section_card(
        column![
            section_header(Icons::BARS, "STATISTICS"),
            Space::with_height(6),
            stats_content,
        ]
        .spacing(0)
    );

    // ── Notes & Keywords ────────────────────────────────────────
    let notes_input = text_input("Document notes...", &data.notes)
        .on_input(|val| Message::NotesChanged(val))
        .size(12)
        .padding(6);

    let keywords_text = data.keywords.join(", ");
    let keywords_input = text_input("keyword1, keyword2...", &keywords_text)
        .on_input(move |val| Message::SetItemKeywords(id, val))
        .size(11)
        .padding(5);

    let notes_section = column![
        field_label(&format!("{}  Notes", Icons::PENCIL)),
        notes_input,
        Space::with_height(8),
        field_label(&format!("{}  Keywords", Icons::TAG)),
        keywords_input,
    ]
    .spacing(2);

    // ── Custom Metadata ─────────────────────────────────────────
    let mut custom_inner = column![].spacing(4);
    custom_inner = custom_inner.push(
        row![
            section_header(Icons::PENCIL_SQUARE, "CUSTOM METADATA"),
            Space::with_width(Length::Fill),
            button(
                text(format!("{} Add", Icons::PLUS)).size(9).color(Theme::TEXT_ACCENT),
            )
            .on_press(Message::AddCustomField(id, "New Field".to_string()))
            .style(theme::inspector_inline_btn_style)
            .padding(Padding::from([2, 6])),
        ]
        .align_y(iced::Alignment::Center)
    );
    for (name, value) in &data.custom_fields {
        let field_name = name.clone();
        let field_name2 = name.clone();
        let field_val = value.clone();
        custom_inner = custom_inner.push(
            row![
                text(field_name.clone())
                    .size(10)
                    .color(Theme::TEXT_MUTED)
                    .width(Length::FillPortion(2)),
                text_input("value...", &field_val)
                    .on_input(move |val| Message::UpdateCustomField(id, field_name.clone(), val))
                    .size(10)
                    .padding(3)
                    .width(Length::FillPortion(3)),
                button(
                    text(Icons::TIMES).size(9).color(Theme::ERROR),
                )
                .on_press(Message::RemoveCustomField(id, field_name2))
                .style(theme::inspector_danger_btn_style)
                .padding(Padding::from([2, 4])),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center)
        );
    }

    let custom_section = section_card(custom_inner);

    // ── Snapshots ───────────────────────────────────────────────
    let snapshots_section = section_card(
        column![
            section_header(Icons::CAMERA, "SNAPSHOTS"),
            Space::with_height(6),
            row![
                text(data.snapshot_count.clone())
                    .size(11)
                    .color(Theme::TEXT_SECONDARY),
                Space::with_width(Length::Fill),
                button(
                    text(format!("{} Take", Icons::CAMERA)).size(10),
                )
                .on_press(Message::CreateSnapshot)
                .style(theme::inspector_accent_btn_style)
                .padding(Padding::from([4, 10])),
            ]
            .align_y(iced::Alignment::Center),
        ]
        .spacing(0)
    );

    // ── Actions ─────────────────────────────────────────────────
    let quick_ref_btn = button(
        text(format!("{} Quick Reference", Icons::BOOK)).size(10),
    )
    .on_press(Message::ShowQuickRef(id))
    .style(theme::inspector_accent_btn_style)
    .padding(Padding::from([5, 10]))
    .width(Length::Fill);

    let split_editor_btn = button(
        text(format!("{} Open in Split", Icons::ARROWS_H)).size(10),
    )
    .on_press(Message::OpenInSplitEditor(id))
    .style(theme::inspector_btn_style)
    .padding(Padding::from([5, 10]))
    .width(Length::Fill);

    let convert_btn: Element<'static, Message> = if data.is_document {
        button(
            text(format!("{} Convert to Folder", Icons::FOLDER)).size(10),
        )
        .on_press(Message::ConvertToFolder(id))
        .style(theme::inspector_btn_style)
        .padding(Padding::from([5, 10]))
        .width(Length::Fill)
        .into()
    } else {
        button(
            text(format!("{} Merge Children", Icons::FILE_TEXT)).size(10),
        )
        .on_press(Message::MergeIntoParent)
        .style(theme::inspector_btn_style)
        .padding(Padding::from([5, 10]))
        .width(Length::Fill)
        .into()
    };

    let split_btn: Element<'static, Message> = if data.is_document {
        button(
            text(format!("{} Split at Midpoint", Icons::ARROWS_H)).size(10),
        )
        .on_press(Message::SplitDocument)
        .style(theme::inspector_btn_style)
        .padding(Padding::from([5, 10]))
        .width(Length::Fill)
        .into()
    } else {
        Space::with_height(0).into()
    };

    let actions_section = column![
        section_header(Icons::BOLT, "ACTIONS"),
        Space::with_height(6),
        quick_ref_btn,
        split_editor_btn,
        convert_btn,
        split_btn,
    ]
    .spacing(4);

    // ── Assemble all sections ───────────────────────────────────
    let content = column![
        header,
        Space::with_height(10),
        doc_info_section,
        Space::with_height(8),
        separator(),
        Space::with_height(8),
        metadata_section,
        Space::with_height(8),
        separator(),
        Space::with_height(8),
        target_section,
        Space::with_height(6),
        stats_section,
        Space::with_height(8),
        separator(),
        Space::with_height(8),
        notes_section,
        Space::with_height(8),
        separator(),
        Space::with_height(8),
        custom_section,
        Space::with_height(6),
        snapshots_section,
        Space::with_height(8),
        separator(),
        Space::with_height(8),
        actions_section,
        Space::with_height(16),
    ]
    .padding(Padding { top: 0.0, right: 12.0, bottom: 12.0, left: 12.0 })
    .width(Length::Fixed(240.0));

    container(scrollable(content))
        .style(theme::sidebar_style)
        .height(Length::Fill)
        .into()
}
