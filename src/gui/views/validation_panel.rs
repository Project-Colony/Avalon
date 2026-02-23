use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use crate::core::validation::{ProjectValidation, Severity};
use crate::gui::app::Message;
use crate::gui::theme::{self, Theme};

/// Render the project validation panel (bottom panel)
pub fn view(result: Option<&ProjectValidation>) -> Element<'static, Message> {
    let header = row![
        text("PROJECT VALIDATION")
            .size(11)
            .color(Theme::TEXT_SECONDARY),
        Space::with_width(Length::Fill),
        button(
            text("Run Validation").size(11).color(Theme::TEXT_ACCENT),
        )
        .on_press(Message::ShowValidation)
        .padding(Padding::from([4, 12])),
    ];

    let body: Element<'static, Message> = match result {
        Some(validation) => {
            if validation.issues.is_empty() {
                let health_label = if validation.trash_items == 0 {
                    "Excellent project health"
                } else if validation.trash_items <= 5 {
                    "Good project health"
                } else {
                    "Consider emptying trash"
                };

                column![
                    text(format!("Project is clean ({} items)", validation.total_items))
                        .size(13)
                        .color(Theme::SUCCESS),
                    Space::with_height(4),
                    row![
                        text(format!("{} items in trash", validation.trash_items))
                            .size(10)
                            .color(Theme::TEXT_MUTED),
                        Space::with_width(12),
                        text(health_label)
                            .size(10)
                            .color(Theme::SUCCESS),
                    ],
                ]
                .spacing(2)
                .into()
            } else {
                // Summary counts
                let errors = validation.error_count();
                let warnings = validation.warning_count();
                let infos = validation.info_count();

                let summary = row![
                    if errors > 0 {
                        text(format!("{} errors", errors)).size(12).color(Theme::ERROR)
                    } else {
                        text("0 errors").size(12).color(Theme::TEXT_MUTED)
                    },
                    Space::with_width(12),
                    if warnings > 0 {
                        text(format!("{} warnings", warnings)).size(12).color(Theme::WARNING)
                    } else {
                        text("0 warnings").size(12).color(Theme::TEXT_MUTED)
                    },
                    Space::with_width(12),
                    text(format!("{} info", infos)).size(12).color(Theme::TEXT_MUTED),
                    Space::with_width(12),
                    text(format!("({} total items)", validation.total_items))
                        .size(10)
                        .color(Theme::TEXT_MUTED),
                ];

                // Issue list
                let mut issue_list = column![].spacing(2);
                for issue in &validation.issues {
                    let severity_color = match issue.severity {
                        Severity::Error => Theme::ERROR,
                        Severity::Warning => Theme::WARNING,
                        Severity::Info => Theme::TEXT_MUTED,
                    };
                    let severity_icon = match issue.severity {
                        Severity::Error => "E",
                        Severity::Warning => "W",
                        Severity::Info => "I",
                    };

                    let inner_row = row![
                        text(severity_icon.to_string()).size(10).color(severity_color),
                        Space::with_width(6),
                        text(issue.message.clone()).size(11).color(Theme::TEXT_SECONDARY),
                    ];
                    let issue_row: Element<'static, Message> = if let Some(item_id) = issue.item_id {
                        button(inner_row)
                            .on_press(Message::SelectBinderItem(item_id))
                            .padding(Padding::from([2, 4]))
                            .into()
                    } else {
                        inner_row.into()
                    };

                    issue_list = issue_list.push(issue_row);
                }

                column![
                    summary,
                    Space::with_height(6),
                    scrollable(issue_list).height(Length::Fixed(120.0)),
                ]
                .spacing(2)
                .into()
            }
        }
        None => {
            text("Click 'Run Validation' to check your project for issues.")
                .size(12)
                .color(Theme::TEXT_MUTED)
                .into()
        }
    };

    let content = column![
        header,
        Space::with_height(6),
        body,
    ]
    .padding(Padding::from([8, 12]));

    container(content)
        .style(theme::panel_style)
        .width(Length::Fill)
        .into()
}
