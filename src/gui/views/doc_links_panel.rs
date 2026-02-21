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
    let header = text("DOCUMENT LINKS")
        .size(11)
        .color(Theme::TEXT_SECONDARY);

    // Outgoing links (from current document)
    let mut outgoing_col = column![
        text(format!("Outgoing ({}):", outgoing_links.len()))
            .size(11)
            .color(Theme::TEXT_MUTED),
    ].spacing(2);

    if outgoing_links.is_empty() {
        outgoing_col = outgoing_col.push(
            text("No outgoing links. Use [[Title]] to link.").size(10).color(Theme::TEXT_MUTED)
        );
    }

    for link in outgoing_links {
        let id = link.target_id;
        outgoing_col = outgoing_col.push(
            button(
                text(format!("-> {}", link.target_title)).size(11).color(Theme::TEXT_ACCENT),
            )
            .on_press(Message::SelectBinderItem(id))
            .padding(Padding::from([2, 4]))
        );
    }

    // Incoming links (to current document)
    let mut incoming_col = column![
        text(format!("Incoming ({}):", incoming_links.len()))
            .size(11)
            .color(Theme::TEXT_MUTED),
    ].spacing(2);

    if incoming_links.is_empty() {
        incoming_col = incoming_col.push(
            text("No documents link to this one.").size(10).color(Theme::TEXT_MUTED)
        );
    }

    for link in incoming_links {
        let id = link.target_id;
        incoming_col = incoming_col.push(
            button(
                text(format!("<- {}", link.target_title)).size(11).color(Theme::TEXT_SECONDARY),
            )
            .on_press(Message::SelectBinderItem(id))
            .padding(Padding::from([2, 4]))
        );
    }

    // Quick link insertion (available docs)
    let mut insert_col = column![
        text("Insert Link:").size(11).color(Theme::TEXT_MUTED),
    ].spacing(1);

    for (id, title) in available_docs.iter().take(15) {
        let target_id = *id;
        insert_col = insert_col.push(
            button(
                text(format!("  {}", title)).size(10).color(Theme::TEXT_SECONDARY),
            )
            .on_press(Message::InsertDocLink(target_id))
            .padding(Padding::from([1, 4]))
        );
    }

    let content = column![
        header,
        Space::with_height(4),
        outgoing_col,
        Space::with_height(6),
        incoming_col,
        Space::with_height(6),
        scrollable(insert_col).height(Length::Fixed(60.0)),
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .width(Length::Fill)
        .into()
}
