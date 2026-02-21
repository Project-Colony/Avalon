use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};
use uuid::Uuid;

use crate::gui::app::Message;
use crate::gui::theme::Theme;

/// A resolved document link
pub struct DocLink {
    pub target_title: String,
    pub target_id: Uuid,
    pub link_text: String,
}

/// Render the document links panel (shows [[links]] in current doc)
pub fn view(
    outgoing_links: &[DocLink],
    incoming_links: &[DocLink],
    available_docs: &[(Uuid, String)],
) -> Element<'static, Message> {
    let total_links = outgoing_links.len() + incoming_links.len();
    let header = row![
        text("DOCUMENT LINKS").size(11).color(Theme::TEXT_SECONDARY),
        Space::with_width(4),
        text("\u{1F517}").size(10),
        Space::with_width(Length::Fill),
        text(format!("{} total link{}", total_links, if total_links == 1 { "" } else { "s" }))
            .size(10)
            .color(Theme::TEXT_MUTED),
    ];

    // Outgoing links (from current document)
    let outgoing_label_color = if outgoing_links.is_empty() { Theme::TEXT_MUTED } else { Theme::TEXT_ACCENT };
    let mut outgoing_col = column![
        text(format!("\u{2192} Outgoing ({})", outgoing_links.len()))
            .size(11)
            .color(outgoing_label_color),
    ].spacing(2);

    if outgoing_links.is_empty() {
        outgoing_col = outgoing_col.push(
            text("No outgoing links. Use [[Title]] syntax to create links.")
                .size(10)
                .color(Theme::TEXT_MUTED)
        );
    }

    for (i, link) in outgoing_links.iter().enumerate() {
        let id = link.target_id;
        let display = if link.link_text != link.target_title && !link.link_text.is_empty() {
            format!("\u{2192} {} (\"{}\")", link.target_title, link.link_text)
        } else {
            format!("\u{2192} {}", link.target_title)
        };
        outgoing_col = outgoing_col.push(
            row![
                text(format!("{}.", i + 1)).size(9).color(Theme::TEXT_MUTED).width(Length::Fixed(16.0)),
                button(
                    text(display).size(11).color(Theme::TEXT_ACCENT),
                )
                .on_press(Message::SelectBinderItem(id))
                .padding(Padding::from([2, 4])),
            ].align_y(iced::Alignment::Center)
        );
    }

    // Incoming links (backlinks to current document)
    let incoming_label_color = if incoming_links.is_empty() { Theme::TEXT_MUTED } else { Theme::SUCCESS };
    let mut incoming_col = column![
        text(format!("\u{2190} Backlinks ({})", incoming_links.len()))
            .size(11)
            .color(incoming_label_color),
    ].spacing(2);

    if incoming_links.is_empty() {
        incoming_col = incoming_col.push(
            text("No documents link to this one (orphan).")
                .size(10)
                .color(Theme::TEXT_MUTED)
        );
    }

    for (i, link) in incoming_links.iter().enumerate() {
        let id = link.target_id;
        incoming_col = incoming_col.push(
            row![
                text(format!("{}.", i + 1)).size(9).color(Theme::TEXT_MUTED).width(Length::Fixed(16.0)),
                button(
                    text(format!("\u{2190} {}", link.target_title)).size(11).color(Theme::TEXT_SECONDARY),
                )
                .on_press(Message::SelectBinderItem(id))
                .padding(Padding::from([2, 4])),
            ].align_y(iced::Alignment::Center)
        );
    }

    // Quick link insertion (available docs to link to)
    let mut insert_col = column![
        text(format!("Insert Link ({} available):", available_docs.len()))
            .size(11)
            .color(Theme::TEXT_MUTED),
    ].spacing(1);

    if available_docs.is_empty() {
        insert_col = insert_col.push(
            text("No documents available to link.").size(10).color(Theme::TEXT_MUTED)
        );
    }

    for (id, title) in available_docs.iter().take(20) {
        let target_id = *id;
        // Check if already linked
        let already_linked = outgoing_links.iter().any(|l| l.target_id == target_id);
        let icon = if already_linked { "\u{2713} " } else { "  " };
        let color = if already_linked { Theme::TEXT_MUTED } else { Theme::TEXT_SECONDARY };

        insert_col = insert_col.push(
            button(
                text(format!("{}{}", icon, title)).size(10).color(color),
            )
            .on_press(Message::InsertDocLink(target_id))
            .padding(Padding::from([1, 4]))
        );
    }

    let hint = text("Syntax: [[Document Title]] or [[Title|display text]]")
        .size(9)
        .color(Theme::TEXT_MUTED);

    let content = column![
        header,
        Space::with_height(4),
        outgoing_col,
        Space::with_height(6),
        incoming_col,
        Space::with_height(6),
        scrollable(insert_col).height(Length::Fixed(60.0)),
        Space::with_height(2),
        hint,
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
