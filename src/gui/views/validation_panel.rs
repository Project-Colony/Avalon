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
            // Health score and grade
            let health = validation.health_score();
            let grade = validation.health_grade();
            let grade_color = if health >= 90.0 { Theme::SUCCESS }
                else if health >= 70.0 { Theme::WARNING }
                else { Theme::ERROR };

            if validation.issues.is_empty() {
                column![
                    row![
                        text(format!("Project is clean ({} items)", validation.total_items))
                            .size(13)
                            .color(Theme::SUCCESS),
                        Space::with_width(12),
                        text(format!("Health: {} ({:.0}%)", grade, health))
                            .size(12)
                            .color(grade_color),
                    ],
                    Space::with_height(4),
                    row![
                        text(format!("{} items in trash", validation.trash_items))
                            .size(10)
                            .color(Theme::TEXT_MUTED),
                        Space::with_width(12),
                        text("No issues found")
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

                let mut summary_row = row![
                    text(format!("Health: {} ({:.0}%)", grade, health))
                        .size(12)
                        .color(grade_color),
                    Space::with_width(12),
                ];
                if errors > 0 {
                    summary_row = summary_row.push(
                        text(format!("{} errors", errors)).size(12).color(Theme::ERROR)
                    );
                } else {
                    summary_row = summary_row.push(
                        text("0 errors").size(12).color(Theme::TEXT_MUTED)
                    );
                }
                summary_row = summary_row.push(Space::with_width(12));
                if warnings > 0 {
                    summary_row = summary_row.push(
                        text(format!("{} warnings", warnings)).size(12).color(Theme::WARNING)
                    );
                } else {
                    summary_row = summary_row.push(
                        text("0 warnings").size(12).color(Theme::TEXT_MUTED)
                    );
                }
                summary_row = summary_row.push(Space::with_width(12));
                summary_row = summary_row.push(
                    text(format!("{} info", infos)).size(12).color(Theme::TEXT_MUTED)
                );
                summary_row = summary_row.push(Space::with_width(12));
                summary_row = summary_row.push(
                    text(format!("({} total items)", validation.total_items))
                        .size(10)
                        .color(Theme::TEXT_MUTED)
                );

                // Auto-fix hint
                let autofix_hint: Element<'static, Message> = if validation.has_auto_fixable() {
                    text(format!("{} issue(s) can be auto-fixed", validation.auto_fixable_count()))
                        .size(10)
                        .color(Theme::TEXT_ACCENT)
                        .into()
                } else {
                    Space::with_height(0).into()
                };

                // Sorted issue list
                let mut issue_list = column![].spacing(2);
                for issue in validation.sorted_issues() {
                    let severity_color = match issue.severity {
                        Severity::Error => Theme::ERROR,
                        Severity::Warning => Theme::WARNING,
                        Severity::Info => Theme::TEXT_MUTED,
                    };
                    let kind_icon = issue.severity.icon();
                    let kind_label = issue.kind.label();

                    let fix_hint = if issue.is_auto_fixable() {
                        " [auto-fixable]"
                    } else {
                        ""
                    };

                    let inner_row = row![
                        text(kind_icon.to_string()).size(10).color(severity_color),
                        Space::with_width(4),
                        text(format!("[{}]", kind_label)).size(9).color(Theme::TEXT_MUTED),
                        Space::with_width(4),
                        text(format!("{}{}", issue.message, fix_hint)).size(11).color(Theme::TEXT_SECONDARY),
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
                    summary_row,
                    autofix_hint,
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
