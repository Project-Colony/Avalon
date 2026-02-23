use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input, toggler, Space};
use iced::{Border, Element, Length, Padding};

use crate::core::metadata::ProjectSettings;
use crate::gui::app::{Message, SettingsTab};
use crate::gui::theme::Theme;

// ── Style helpers ────────────────────────────────────────────────────

fn tab_button(icon: &str, label: &str, tab: SettingsTab, active: &SettingsTab) -> Element<'static, Message> {
    let is_active = &tab == active;
    let (bg, color) = if is_active {
        (Some(iced::Background::Color(Theme::BG_SELECTED)), Theme::TEXT_ACCENT)
    } else {
        (None, Theme::TEXT_SECONDARY)
    };

    button(
        text(format!("{}  {}", icon, label)).size(13).color(color),
    )
    .on_press(Message::SettingsChangeTab(tab))
    .padding(Padding::from([9, 16]))
    .width(Length::Fill)
    .style(move |_theme: &iced::Theme, status| {
        let hover_bg = match status {
            button::Status::Hovered if !is_active => {
                Some(iced::Background::Color(Theme::SIDEBAR_ITEM_HOVER))
            }
            _ => bg,
        };
        button::Style {
            background: hover_bg,
            text_color: color,
            border: Border {
                color: if is_active { Theme::ACCENT } else { iced::Color::TRANSPARENT },
                width: 0.0,
                radius: 5.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

fn section_header(label: &str) -> Element<'static, Message> {
    column![
        text(label.to_string()).size(15).color(Theme::TEXT_ACCENT),
        Space::with_height(6),
    ].into()
}

fn subsection_header(label: &str) -> Element<'static, Message> {
    column![
        Space::with_height(2),
        text(label.to_string()).size(13).color(Theme::TEXT_SECONDARY),
        Space::with_height(4),
    ].into()
}

fn setting_label(label: &str) -> Element<'static, Message> {
    text(label.to_string()).size(12).color(Theme::TEXT_MUTED).into()
}

fn setting_description(desc: &str) -> Element<'static, Message> {
    text(desc.to_string()).size(10).color(Theme::TEXT_DISABLED).into()
}

fn divider() -> Element<'static, Message> {
    container(
        container(Space::with_height(1))
            .style(|_theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(Theme::BORDER_SUBTLE)),
                ..Default::default()
            })
            .width(Length::Fill)
    )
    .padding(Padding::from([10, 0]))
    .width(Length::Fill)
    .into()
}

fn toggle_setting(
    enabled: bool,
    label: &'static str,
    desc: &str,
    msg: fn(bool) -> Message,
) -> Element<'static, Message> {
    column![
        toggler(enabled)
            .label(label)
            .on_toggle(msg),
        setting_description(desc),
        Space::with_height(6),
    ]
    .spacing(1)
    .into()
}

fn shortcut_row(keys: &str, desc: &str) -> Element<'static, Message> {
    row![
        container(
            text(keys.to_string()).size(11).color(Theme::TEXT_ACCENT)
        )
        .width(Length::Fixed(170.0))
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(Theme::BG_TERTIARY)),
            border: Border {
                color: Theme::BORDER_SUBTLE,
                width: 1.0,
                radius: 3.0.into(),
            },
            ..Default::default()
        })
        .padding(Padding::from([3, 8])),
        Space::with_width(12),
        text(desc.to_string()).size(12).color(Theme::TEXT_SECONDARY),
    ]
    .align_y(iced::Alignment::Center)
    .padding(Padding::from([2, 0]))
    .into()
}

// ── Main view ────────────────────────────────────────────────────────

pub fn view(
    settings: &ProjectSettings,
    project_title: &str,
    script_mode: bool,
    auto_correction: &crate::core::script::AutoCorrection,
    active_tab: &SettingsTab,
) -> Element<'static, Message> {
    // ── Sidebar navigation ──
    let sidebar = container(
        column![
            text("Settings").size(17).color(Theme::TEXT_PRIMARY),
            Space::with_height(20),
            tab_button("\u{f013}", "General", SettingsTab::General, active_tab),
            Space::with_height(2),
            tab_button("\u{f040}", "Editor", SettingsTab::Editor, active_tab),
            Space::with_height(2),
            tab_button("\u{f1fc}", "Appearance", SettingsTab::Appearance, active_tab),
            Space::with_height(2),
            tab_button("\u{f1c1}", "Compile", SettingsTab::Compile, active_tab),
            Space::with_height(2),
            tab_button("\u{f11c}", "Shortcuts", SettingsTab::Shortcuts, active_tab),
            Space::with_height(Length::Fill),
            button(
                text("  Done  ").size(13).color(Theme::TEXT_PRIMARY),
            )
            .on_press(Message::CloseSettingsWindow)
            .padding(Padding::from([8, 20]))
            .width(Length::Fill),
        ]
        .padding(16)
        .width(Length::Fixed(180.0))
    )
    .style(|_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(Theme::SIDEBAR_BG)),
        border: Border {
            color: Theme::BORDER_SUBTLE,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .height(Length::Fill);

    // ── Content panel ──
    let content: Element<'_, Message> = match active_tab {
        SettingsTab::General => tab_general(settings, project_title),
        SettingsTab::Editor => tab_editor(settings, script_mode, auto_correction),
        SettingsTab::Appearance => tab_appearance(settings),
        SettingsTab::Compile => tab_compile(settings),
        SettingsTab::Shortcuts => tab_shortcuts(),
    };

    let content_panel = container(scrollable(
        container(content)
            .padding(28)
            .width(Length::Fill)
    ))
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(Theme::BG_PRIMARY)),
        ..Default::default()
    });

    row![sidebar, content_panel]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ═══════════════════════════════════════════════════════════════════════
//  GENERAL — Project, Saving, Backups, Accessibility
// ═══════════════════════════════════════════════════════════════════════

fn tab_general(
    settings: &ProjectSettings,
    project_title: &str,
) -> Element<'static, Message> {
    // ── Project ──
    let title_input = text_input("Project title...", project_title)
        .on_input(|val| Message::SettingsSetProjectTitle(val))
        .size(14)
        .padding(8);

    let target_str = settings.target_word_count
        .map(|t| t.to_string())
        .unwrap_or_default();
    let target_input = text_input("e.g. 80000", &target_str)
        .on_input(|val| Message::SettingsSetTarget(val))
        .size(13)
        .padding(6)
        .width(Length::Fixed(160.0));

    let deadline_str = settings.target_deadline.clone().unwrap_or_default();
    let deadline_input = text_input("YYYY-MM-DD", &deadline_str)
        .on_input(|val| Message::SettingsSetDeadline(val))
        .size(13)
        .padding(6)
        .width(Length::Fixed(160.0));

    // ── Saving ──
    let autosave_str = format!("{}", settings.auto_save_seconds);
    let autosave_input = text_input("30", &autosave_str)
        .on_input(|val| Message::SettingsSetAutoSave(val))
        .size(13)
        .padding(6)
        .width(Length::Fixed(80.0));

    // ── Backups ──
    let backup_interval_str = format!("{}", settings.backup_interval_saves);
    let backup_interval_input = text_input("10", &backup_interval_str)
        .on_input(|val| Message::SettingsSetBackupInterval(val))
        .size(13)
        .padding(6)
        .width(Length::Fixed(80.0));

    // ── Accessibility ──
    let ui_scale_str = format!("{:.0}%", settings.ui_scale * 100.0);
    let ui_scale_options = vec![
        "75%".to_string(), "100%".to_string(), "125%".to_string(),
        "150%".to_string(), "200%".to_string(),
    ];
    let ui_scale_picker = pick_list(
        ui_scale_options,
        Some(ui_scale_str),
        |selected| {
            let val = selected.trim_end_matches('%');
            if let Ok(pct) = val.parse::<f32>() {
                Message::SettingsSetUIScale(format!("{}", pct / 100.0))
            } else {
                Message::SettingsSetUIScale("1.0".to_string())
            }
        },
    )
    .width(Length::Fixed(110.0));

    column![
        // ── Project ──
        section_header("Project"),
        setting_label("Project Title"),
        title_input,
        Space::with_height(10),
        row![
            column![
                setting_label("Target Word Count"),
                Space::with_height(4),
                target_input,
                setting_description("Leave empty for no target"),
            ],
            Space::with_width(24),
            column![
                setting_label("Target Deadline"),
                Space::with_height(4),
                deadline_input,
                setting_description("Format: YYYY-MM-DD"),
            ],
        ],

        divider(),

        // ── Saving ──
        section_header("Saving"),
        row![
            column![
                setting_label("Auto-save Interval"),
                Space::with_height(4),
                row![
                    autosave_input,
                    Space::with_width(6),
                    text("seconds").size(12).color(Theme::TEXT_MUTED),
                ].align_y(iced::Alignment::Center),
            ],
        ],

        divider(),

        // ── Backups ──
        section_header("Backups"),
        toggle_setting(
            settings.auto_backup,
            "Automatic backups",
            "Create periodic backup copies of the project",
            Message::SettingsToggleAutoBackup,
        ),
        row![
            column![
                setting_label("Backup frequency"),
                Space::with_height(4),
                row![
                    text("Every").size(12).color(Theme::TEXT_MUTED),
                    Space::with_width(6),
                    backup_interval_input,
                    Space::with_width(6),
                    text("saves").size(12).color(Theme::TEXT_MUTED),
                ].align_y(iced::Alignment::Center),
            ],
        ],

        divider(),

        // ── Accessibility ──
        section_header("Accessibility"),
        setting_label("Interface Scale"),
        Space::with_height(4),
        ui_scale_picker,
        setting_description("Scale all UI elements proportionally"),
        Space::with_height(10),
        toggle_setting(
            settings.high_contrast,
            "High contrast",
            "Sharper borders and stronger color differences",
            Message::SettingsToggleHighContrast,
        ),
        toggle_setting(
            settings.large_ui,
            "Large UI elements",
            "Bigger buttons, toggles, and click targets",
            Message::SettingsToggleLargeUI,
        ),
        toggle_setting(
            settings.reduce_motion,
            "Reduce motion",
            "Disable transitions and animations",
            Message::SettingsToggleReduceMotion,
        ),
        toggle_setting(
            settings.screen_reader_hints,
            "Screen reader hints",
            "Add semantic labels for assistive technology",
            Message::SettingsToggleScreenReaderHints,
        ),
    ]
    .spacing(2)
    .max_width(540)
    .into()
}

// ═══════════════════════════════════════════════════════════════════════
//  EDITOR — Typing, Corrections, Composition, Script mode
// ═══════════════════════════════════════════════════════════════════════

fn tab_editor(
    settings: &ProjectSettings,
    script_mode: bool,
    auto_correction: &crate::core::script::AutoCorrection,
) -> Element<'static, Message> {
    // ── Composition ──
    let comp_width_str = format!("{:.0}", settings.fullscreen_text_width);
    let comp_width_input = text_input("60", &comp_width_str)
        .on_input(|val| Message::SettingsSetCompWidth(val))
        .size(13)
        .padding(6)
        .width(Length::Fixed(80.0));

    column![
        // ── Input ──
        section_header("Input"),
        toggle_setting(
            settings.typewriter_scroll,
            "Typewriter scrolling",
            "Keep the active line vertically centered while typing",
            Message::SettingsToggleTypewriterScroll,
        ),
        toggle_setting(
            settings.spell_check_enabled,
            "Check spelling while typing",
            "Underline misspelled words in real time",
            Message::SettingsToggleSpellCheck,
        ),

        divider(),

        // ── Auto-Correction (Proofing) ──
        section_header("Proofing & Auto-Correct"),
        toggle_setting(
            settings.smart_punctuation,
            "Smart punctuation",
            "Replace straight quotes and hyphens with typographic equivalents",
            Message::SettingsToggleSmartPunctuation,
        ),

        subsection_header("Substitutions"),
        toggler(auto_correction.smart_quotes)
            .label("Curly quotes   \"...\"  \u{2192}  \u{201c}...\u{201d}")
            .on_toggle(|_| Message::ToggleAutoCorrectSmartQuotes),
        Space::with_height(4),
        toggler(auto_correction.em_dashes)
            .label("Em dash   --  \u{2192}  \u{2014}")
            .on_toggle(|_| Message::ToggleAutoCorrectEmDashes),
        Space::with_height(4),
        toggler(auto_correction.ellipsis)
            .label("Ellipsis   ...  \u{2192}  \u{2026}")
            .on_toggle(|_| Message::ToggleAutoCorrectEllipsis),

        divider(),

        // ── Composition mode ──
        section_header("Composition Mode"),
        setting_description("Distraction-free writing environment (F5)"),
        Space::with_height(8),
        setting_label("Text width in composition"),
        Space::with_height(4),
        row![
            comp_width_input,
            Space::with_width(6),
            text("% of screen").size(12).color(Theme::TEXT_MUTED),
        ].align_y(iced::Alignment::Center),

        divider(),

        // ── Script mode ──
        section_header("Screenplay / Script"),
        toggler(script_mode)
            .label("Enable script mode")
            .on_toggle(|_| Message::ToggleScriptMode),
        Space::with_height(2),
        setting_description(
            "Format documents with scene headings, action, character names, \
             dialogue, and transitions. Applies standard screenplay formatting."
        ),
    ]
    .spacing(2)
    .max_width(540)
    .into()
}

// ═══════════════════════════════════════════════════════════════════════
//  APPEARANCE — Typography, Editor Layout, Display
// ═══════════════════════════════════════════════════════════════════════

fn tab_appearance(settings: &ProjectSettings) -> Element<'static, Message> {
    // ── Font ──
    let font_options = vec![
        "monospace".to_string(),
        "serif".to_string(),
        "sans-serif".to_string(),
    ];
    let font_picker = pick_list(
        font_options,
        Some(settings.editor_font.clone()),
        |selected| Message::SettingsSetFont(selected),
    )
    .width(Length::Fixed(180.0));

    let font_size_str = format!("{:.0}", settings.editor_font_size);
    let font_size_input = text_input("16", &font_size_str)
        .on_input(|val| Message::SettingsSetFontSize(val))
        .size(13)
        .padding(6)
        .width(Length::Fixed(70.0));

    // ── Line spacing ──
    let spacing_options = vec![
        "Single".to_string(),
        "1.15".to_string(),
        "1.5".to_string(),
        "Double".to_string(),
    ];
    let current_spacing = match settings.line_spacing {
        s if (s - 1.0).abs() < 0.01 => "Single".to_string(),
        s if (s - 1.15).abs() < 0.01 => "1.15".to_string(),
        s if (s - 1.5).abs() < 0.01 => "1.5".to_string(),
        s if (s - 2.0).abs() < 0.01 => "Double".to_string(),
        s => format!("{:.2}", s),
    };
    let spacing_picker = pick_list(
        spacing_options,
        Some(current_spacing),
        |selected| Message::SettingsSetLineSpacingPreset(selected),
    )
    .width(Length::Fixed(110.0));

    // ── Zoom ──
    let zoom_str = format!("{:.0}%", settings.editor_zoom * 100.0);
    let zoom_row = row![
        button(text("-").size(14).color(Theme::TEXT_PRIMARY))
            .on_press(Message::SettingsZoomOut)
            .padding(Padding::from([4, 12])),
        container(
            text(zoom_str).size(13).color(Theme::TEXT_PRIMARY)
        ).padding(Padding::from([0, 8])),
        button(text("+").size(14).color(Theme::TEXT_PRIMARY))
            .on_press(Message::SettingsZoomIn)
            .padding(Padding::from([4, 12])),
    ]
    .spacing(2)
    .align_y(iced::Alignment::Center);

    // ── Editor width ──
    let editor_width_str = format!("{:.0}", settings.editor_width);
    let editor_width_input = text_input("80", &editor_width_str)
        .on_input(|val| Message::SettingsSetEditorWidth(val))
        .size(13)
        .padding(6)
        .width(Length::Fixed(70.0));

    column![
        // ── Font ──
        section_header("Font"),
        row![
            column![
                setting_label("Font Family"),
                Space::with_height(4),
                font_picker,
            ],
            Space::with_width(24),
            column![
                setting_label("Size (pt)"),
                Space::with_height(4),
                font_size_input,
            ],
        ],
        Space::with_height(12),
        row![
            column![
                setting_label("Line Spacing"),
                Space::with_height(4),
                spacing_picker,
            ],
            Space::with_width(24),
            column![
                setting_label("Editor Zoom"),
                Space::with_height(4),
                zoom_row,
            ],
        ],

        divider(),

        // ── Editor layout ──
        section_header("Layout"),
        setting_label("Editor text width"),
        Space::with_height(4),
        row![
            editor_width_input,
            Space::with_width(6),
            text("% of panel width").size(12).color(Theme::TEXT_MUTED),
        ].align_y(iced::Alignment::Center),
        setting_description("Controls how wide the text column is in the main editor"),

        divider(),

        // ── Display toggles ──
        section_header("Display"),
        toggle_setting(
            settings.show_word_count,
            "Word count in status bar",
            "Show live word and character counts at the bottom",
            Message::SettingsToggleWordCount,
        ),
        toggle_setting(
            settings.show_paragraph_marks,
            "Paragraph marks",
            "Show invisible formatting characters in the editor",
            Message::SettingsToggleShowParagraphMarks,
        ),
    ]
    .spacing(2)
    .max_width(540)
    .into()
}

// ═══════════════════════════════════════════════════════════════════════
//  COMPILE — Labels, Statuses, Compile defaults
// ═══════════════════════════════════════════════════════════════════════

fn tab_compile(settings: &ProjectSettings) -> Element<'static, Message> {
    // ── Labels ──
    let mut labels_col = column![].spacing(4);
    for lbl in &settings.labels {
        let color = lbl.color.to_iced_color();
        labels_col = labels_col.push(
            row![
                text("\u{f111}").size(10).color(color),
                Space::with_width(8),
                text(lbl.name.clone()).size(12).color(Theme::TEXT_PRIMARY),
                Space::with_width(Length::Fill),
                container(
                    text(format!("{:?}", lbl.color)).size(9).color(Theme::TEXT_MUTED)
                )
                .style(|_theme: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(Theme::BG_TERTIARY)),
                    border: Border { color: Theme::BORDER_SUBTLE, width: 1.0, radius: 3.0.into() },
                    ..Default::default()
                })
                .padding(Padding::from([2, 6])),
            ]
            .align_y(iced::Alignment::Center)
        );
    }

    // ── Statuses ──
    let mut statuses_col = column![].spacing(4);
    for (idx, st) in settings.statuses.iter().enumerate() {
        let status_color = Theme::status_color(&st.name);
        statuses_col = statuses_col.push(
            row![
                container(
                    text(format!("{}", idx + 1)).size(10).color(Theme::TEXT_MUTED)
                ).width(Length::Fixed(20.0)),
                container(Space::new(8, 8))
                    .style(move |_theme: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(status_color)),
                        border: Border { color: iced::Color::TRANSPARENT, width: 0.0, radius: 2.0.into() },
                        ..Default::default()
                    }),
                Space::with_width(8),
                text(st.name.clone()).size(12).color(Theme::TEXT_PRIMARY),
            ]
            .align_y(iced::Alignment::Center)
        );
    }

    // ── Default doc type ──
    let doc_type_options = vec![
        "".to_string(),
        "Scene".to_string(),
        "Chapter".to_string(),
        "Part".to_string(),
        "Note".to_string(),
    ];
    let current_doc_type = if settings.default_doc_type.is_empty() {
        "".to_string()
    } else {
        settings.default_doc_type.clone()
    };
    let doc_type_picker = pick_list(
        doc_type_options,
        Some(current_doc_type),
        |selected| Message::SettingsSetDefaultDocType(selected),
    )
    .width(Length::Fixed(160.0))
    .placeholder("None (default)");

    column![
        // ── Labels ──
        section_header("Labels"),
        setting_description("Color-coded tags for organizing binder items"),
        Space::with_height(6),
        labels_col,

        divider(),

        // ── Statuses ──
        section_header("Statuses"),
        setting_description("Track document progress through your workflow"),
        Space::with_height(6),
        statuses_col,

        divider(),

        // ── Compile options ──
        section_header("Compile Defaults"),

        toggle_setting(
            settings.auto_numbering,
            "Auto-number chapters",
            "Automatically prepend chapter numbers during compile",
            Message::SettingsToggleAutoNumbering,
        ),

        toggle_setting(
            settings.show_synopsis_in_binder,
            "Include synopses in binder tooltip",
            "Show the document synopsis when hovering over binder items",
            Message::SettingsToggleShowSynopsis,
        ),

        setting_label("Default new document type"),
        Space::with_height(4),
        doc_type_picker,
        setting_description("Pre-selected type when creating a new document"),
    ]
    .spacing(2)
    .max_width(540)
    .into()
}

// ═══════════════════════════════════════════════════════════════════════
//  SHORTCUTS — Full keyboard reference
// ═══════════════════════════════════════════════════════════════════════

fn tab_shortcuts() -> Element<'static, Message> {
    column![
        section_header("File"),
        shortcut_row("Ctrl+N", "New document"),
        shortcut_row("Ctrl+S", "Save project"),
        shortcut_row("Ctrl+E", "Open compile dialog"),
        shortcut_row("Ctrl+,", "Open settings"),

        divider(),

        section_header("Edit"),
        shortcut_row("Ctrl+Z", "Undo"),
        shortcut_row("Ctrl+Y", "Redo"),
        shortcut_row("Ctrl+B", "Bold"),
        shortcut_row("Ctrl+U", "Underline"),
        shortcut_row("Ctrl+K", "Insert link"),
        shortcut_row("Ctrl+Shift+I", "Italic"),
        shortcut_row("Ctrl+Shift+X", "Strikethrough"),
        shortcut_row("Ctrl+Shift+K", "Delete line"),
        shortcut_row("Ctrl+Shift+D", "Duplicate line"),
        shortcut_row("Ctrl+Shift+J", "Join lines"),

        divider(),

        section_header("Navigation"),
        shortcut_row("Ctrl+F", "Search project"),
        shortcut_row("Ctrl+I", "Toggle inspector"),
        shortcut_row("F3", "Find next"),
        shortcut_row("F6", "Search panel"),

        divider(),

        section_header("View"),
        shortcut_row("F5", "Composition mode"),
        shortcut_row("F11", "Focus mode (fullscreen)"),
        shortcut_row("Ctrl+Shift+F", "Toggle composition"),
        shortcut_row("Ctrl+Shift+S", "Project statistics"),
        shortcut_row("Ctrl+Shift+G", "Writing goals"),

        divider(),

        section_header("Text"),
        shortcut_row("Alt+U", "UPPERCASE"),
        shortcut_row("Alt+L", "lowercase"),
        shortcut_row("Alt+\u{2191}", "Move line up"),
        shortcut_row("Alt+\u{2193}", "Move line down"),
        shortcut_row("Alt+[", "Unindent"),
        shortcut_row("Alt+]", "Indent"),
        shortcut_row("Ctrl+Shift+L", "Sort lines"),
        shortcut_row("Ctrl+Shift+U", "Remove duplicate lines"),

        divider(),

        section_header("Tools"),
        shortcut_row("F7", "Spell check"),
        shortcut_row("F8", "Validation"),
        shortcut_row("F9", "Create snapshot"),
        shortcut_row("Ctrl+Shift+B", "Create backup"),
        shortcut_row("Ctrl+Shift+H", "Writing history"),
        shortcut_row("Ctrl+Shift+N", "Name generator"),
        shortcut_row("Ctrl+Shift+E", "Close split editor"),
        shortcut_row("Ctrl+Shift+T", "Toggle script mode"),

        divider(),

        section_header("General"),
        shortcut_row("Escape", "Close current panel / dialog"),
    ]
    .spacing(2)
    .max_width(540)
    .into()
}
