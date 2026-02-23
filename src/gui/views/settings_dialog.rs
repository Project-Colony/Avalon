use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input, toggler, Space};
use iced::{Border, Element, Length, Padding};

use crate::core::metadata::ProjectSettings;
use crate::gui::app::{Message, SettingsTab};
use crate::gui::theme::Theme;

// ── Shared style helpers ─────────────────────────────────────────────

fn tab_button(label: &str, tab: SettingsTab, active: &SettingsTab) -> Element<'static, Message> {
    let is_active = &tab == active;
    let (bg, color, border_color) = if is_active {
        (
            Some(iced::Background::Color(Theme::BG_SELECTED)),
            Theme::TEXT_ACCENT,
            Theme::ACCENT,
        )
    } else {
        (None, Theme::TEXT_SECONDARY, iced::Color::TRANSPARENT)
    };

    button(
        row![
            text(label.to_string()).size(13).color(color),
        ]
        .padding(Padding::from([0, 4]))
    )
    .on_press(Message::SettingsChangeTab(tab))
    .padding(Padding::from([8, 16]))
    .width(Length::Fill)
    .style(move |_theme: &iced::Theme, _status| {
        button::Style {
            background: bg,
            text_color: color,
            border: Border {
                color: border_color,
                width: if is_active { 0.0 } else { 0.0 },
                radius: 6.0.into(),
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
    .padding(Padding::from([8, 0]))
    .width(Length::Fill)
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
    // ── Sidebar ──
    let sidebar = container(
        column![
            text("Settings").size(16).color(Theme::TEXT_PRIMARY),
            Space::with_height(16),
            tab_button("\u{f013}  General", SettingsTab::General, active_tab),
            Space::with_height(2),
            tab_button("\u{f1fc}  Appearance", SettingsTab::Appearance, active_tab),
            Space::with_height(2),
            tab_button("\u{f06e}  Accessibility", SettingsTab::Accessibility, active_tab),
            Space::with_height(2),
            tab_button("\u{f040}  Avalon", SettingsTab::Avalon, active_tab),
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
        SettingsTab::Appearance => tab_appearance(settings),
        SettingsTab::Accessibility => tab_accessibility(settings),
        SettingsTab::Avalon => tab_avalon(settings, script_mode, auto_correction),
    };

    let content_panel = container(scrollable(
        container(content)
            .padding(24)
            .width(Length::Fill)
    ))
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(Theme::BG_PRIMARY)),
        ..Default::default()
    });

    // ── Layout: sidebar | content ──
    row![sidebar, content_panel]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ═══════════════════════════════════════════════════════════════════════
// TAB: General
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

    let auto_backup_toggle = toggler(settings.auto_backup)
        .label("Enable automatic backups")
        .on_toggle(|val| Message::SettingsToggleAutoBackup(val));

    let backup_interval_str = format!("{}", settings.backup_interval_saves);
    let backup_interval_input = text_input("10", &backup_interval_str)
        .on_input(|val| Message::SettingsSetBackupInterval(val))
        .size(13)
        .padding(6)
        .width(Length::Fixed(80.0));

    // ── Labels ──
    let mut labels_col = column![].spacing(3);
    for lbl in &settings.labels {
        let color = lbl.color.to_iced_color();
        labels_col = labels_col.push(
            row![
                text("\u{f111}").size(10).color(color),
                Space::with_width(6),
                text(lbl.name.clone()).size(12).color(Theme::TEXT_PRIMARY),
                Space::with_width(Length::Fill),
                text(format!("{:?}", lbl.color)).size(9).color(Theme::TEXT_MUTED),
            ]
            .align_y(iced::Alignment::Center)
        );
    }

    // ── Statuses ──
    let mut statuses_col = column![].spacing(3);
    for (idx, st) in settings.statuses.iter().enumerate() {
        let status_color = Theme::status_color(&st.name);
        statuses_col = statuses_col.push(
            row![
                text(format!("{}.", idx + 1)).size(10).color(Theme::TEXT_MUTED),
                Space::with_width(6),
                text(st.name.clone()).size(12).color(status_color),
            ]
        );
    }

    column![
        section_header("Project"),
        setting_label("Project Title"),
        title_input,
        Space::with_height(10),
        row![
            column![
                setting_label("Target Word Count"),
                Space::with_height(4),
                target_input,
            ],
            Space::with_width(20),
            column![
                setting_label("Target Deadline"),
                Space::with_height(4),
                deadline_input,
            ],
        ],
        Space::with_height(16),
        divider(),

        section_header("Saving & Backups"),
        row![
            column![
                setting_label("Auto-save Interval (seconds)"),
                Space::with_height(4),
                autosave_input,
            ],
            Space::with_width(20),
            column![
                setting_label("Backup Every N Saves"),
                Space::with_height(4),
                backup_interval_input,
                setting_description("Creates a backup after this many saves"),
            ],
        ],
        Space::with_height(8),
        auto_backup_toggle,
        Space::with_height(16),
        divider(),

        section_header("Labels"),
        setting_description("Color-coded labels for organizing binder items"),
        Space::with_height(4),
        labels_col,
        Space::with_height(16),
        divider(),

        section_header("Statuses"),
        setting_description("Document workflow statuses"),
        Space::with_height(4),
        statuses_col,
    ]
    .spacing(2)
    .max_width(520)
    .into()
}

// ═══════════════════════════════════════════════════════════════════════
// TAB: Appearance
// ═══════════════════════════════════════════════════════════════════════

fn tab_appearance(settings: &ProjectSettings) -> Element<'static, Message> {
    // ── Typography ──
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
    .width(Length::Fixed(200.0));

    let font_size_str = format!("{:.0}", settings.editor_font_size);
    let font_size_input = text_input("16", &font_size_str)
        .on_input(|val| Message::SettingsSetFontSize(val))
        .size(13)
        .padding(6)
        .width(Length::Fixed(80.0));

    // ── Line spacing presets ──
    let spacing_options = vec![
        "Single".to_string(),
        "1.15".to_string(),
        "1.5".to_string(),
        "Double".to_string(),
    ];
    let current_spacing_label = match settings.line_spacing {
        s if (s - 1.0).abs() < 0.01 => "Single".to_string(),
        s if (s - 1.15).abs() < 0.01 => "1.15".to_string(),
        s if (s - 1.5).abs() < 0.01 => "1.5".to_string(),
        s if (s - 2.0).abs() < 0.01 => "Double".to_string(),
        s => format!("{:.2}", s),
    };
    let spacing_picker = pick_list(
        spacing_options,
        Some(current_spacing_label),
        |selected| Message::SettingsSetLineSpacingPreset(selected),
    )
    .width(Length::Fixed(120.0));

    // ── Zoom ──
    let zoom_str = format!("{:.0}%", settings.editor_zoom * 100.0);
    let zoom_row = row![
        button(text("-").size(14).color(Theme::TEXT_PRIMARY))
            .on_press(Message::SettingsZoomOut)
            .padding(Padding::from([4, 12])),
        container(
            text(zoom_str).size(14).color(Theme::TEXT_PRIMARY)
        ).padding(Padding::from([0, 8])),
        button(text("+").size(14).color(Theme::TEXT_PRIMARY))
            .on_press(Message::SettingsZoomIn)
            .padding(Padding::from([4, 12])),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center);

    // ── Editor width ──
    let editor_width_str = format!("{:.0}", settings.editor_width);
    let editor_width_input = text_input("80", &editor_width_str)
        .on_input(|val| Message::SettingsSetEditorWidth(val))
        .size(13)
        .padding(6)
        .width(Length::Fixed(80.0));

    // ── Composition mode ──
    let comp_width_str = format!("{:.0}", settings.fullscreen_text_width);
    let comp_width_input = text_input("60", &comp_width_str)
        .on_input(|val| Message::SettingsSetCompWidth(val))
        .size(13)
        .padding(6)
        .width(Length::Fixed(80.0));

    // ── Status bar ──
    let word_count_toggle = toggler(settings.show_word_count)
        .label("Show word count in status bar")
        .on_toggle(|val| Message::SettingsToggleWordCount(val));

    let paragraph_toggle = toggler(settings.show_paragraph_marks)
        .label("Show paragraph marks")
        .on_toggle(|val| Message::SettingsToggleShowParagraphMarks(val));

    column![
        section_header("Typography"),
        row![
            column![
                setting_label("Editor Font"),
                Space::with_height(4),
                font_picker,
            ],
            Space::with_width(20),
            column![
                setting_label("Font Size"),
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
            Space::with_width(20),
            column![
                setting_label("Zoom"),
                Space::with_height(4),
                zoom_row,
            ],
        ],
        Space::with_height(16),
        divider(),

        section_header("Editor Layout"),
        row![
            column![
                setting_label("Editor Text Width (%)"),
                setting_description("Percentage of the editor area used for text"),
                Space::with_height(4),
                editor_width_input,
            ],
            Space::with_width(20),
            column![
                setting_label("Composition Text Width (%)"),
                setting_description("Text width in distraction-free mode"),
                Space::with_height(4),
                comp_width_input,
            ],
        ],
        Space::with_height(16),
        divider(),

        section_header("Display"),
        word_count_toggle,
        Space::with_height(6),
        paragraph_toggle,
    ]
    .spacing(2)
    .max_width(520)
    .into()
}

// ═══════════════════════════════════════════════════════════════════════
// TAB: Accessibility
// ═══════════════════════════════════════════════════════════════════════

fn tab_accessibility(settings: &ProjectSettings) -> Element<'static, Message> {
    let ui_scale_str = format!("{:.0}%", settings.ui_scale * 100.0);
    let ui_scale_options = vec![
        "75%".to_string(),
        "100%".to_string(),
        "125%".to_string(),
        "150%".to_string(),
        "200%".to_string(),
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
    .width(Length::Fixed(120.0));

    let high_contrast_toggle = toggler(settings.high_contrast)
        .label("High contrast mode")
        .on_toggle(|val| Message::SettingsToggleHighContrast(val));

    let large_ui_toggle = toggler(settings.large_ui)
        .label("Large UI elements")
        .on_toggle(|val| Message::SettingsToggleLargeUI(val));

    let reduce_motion_toggle = toggler(settings.reduce_motion)
        .label("Reduce motion and animations")
        .on_toggle(|val| Message::SettingsToggleReduceMotion(val));

    let screen_reader_toggle = toggler(settings.screen_reader_hints)
        .label("Screen reader hints")
        .on_toggle(|val| Message::SettingsToggleScreenReaderHints(val));

    column![
        section_header("Visual"),
        setting_label("UI Scale"),
        setting_description("Scale all interface elements"),
        Space::with_height(4),
        ui_scale_picker,
        Space::with_height(12),
        high_contrast_toggle,
        setting_description("Increase contrast between UI elements for better visibility"),
        Space::with_height(8),
        large_ui_toggle,
        setting_description("Use larger buttons, text, and controls throughout the interface"),
        Space::with_height(16),
        divider(),

        section_header("Motion & Animation"),
        reduce_motion_toggle,
        setting_description("Disable transitions, fades, and smooth scrolling"),
        Space::with_height(16),
        divider(),

        section_header("Assistive Technology"),
        screen_reader_toggle,
        setting_description("Add extra labels and hints for screen readers"),
        Space::with_height(16),
        divider(),

        // Keyboard info section
        section_header("Keyboard Navigation"),
        setting_description("Avalon supports full keyboard navigation throughout the interface."),
        Space::with_height(8),
        setting_label("Key shortcuts"),
        Space::with_height(4),
        shortcut_row("Ctrl+S", "Save project"),
        shortcut_row("Ctrl+,", "Open settings"),
        shortcut_row("Ctrl+F", "Search"),
        shortcut_row("Ctrl+Z / Ctrl+Y", "Undo / Redo"),
        shortcut_row("Ctrl+B / Ctrl+I", "Bold / Italic"),
        shortcut_row("F11", "Toggle fullscreen"),
        shortcut_row("F5", "Composition mode"),
        shortcut_row("Escape", "Close panel/dialog"),
    ]
    .spacing(2)
    .max_width(520)
    .into()
}

fn shortcut_row(shortcut: &str, description: &str) -> Element<'static, Message> {
    row![
        container(
            text(shortcut.to_string()).size(11).color(Theme::TEXT_ACCENT)
        )
        .width(Length::Fixed(160.0))
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
        text(description.to_string()).size(12).color(Theme::TEXT_SECONDARY),
    ]
    .align_y(iced::Alignment::Center)
    .padding(Padding::from([2, 0]))
    .into()
}

// ═══════════════════════════════════════════════════════════════════════
// TAB: Avalon (Editor & Deep Customization)
// ═══════════════════════════════════════════════════════════════════════

fn tab_avalon(
    settings: &ProjectSettings,
    script_mode: bool,
    auto_correction: &crate::core::script::AutoCorrection,
) -> Element<'static, Message> {
    // ── Editor behavior ──
    let typewriter_toggle = toggler(settings.typewriter_scroll)
        .label("Typewriter scrolling")
        .on_toggle(|val| Message::SettingsToggleTypewriterScroll(val));

    let spell_check_toggle = toggler(settings.spell_check_enabled)
        .label("Live spell checking")
        .on_toggle(|val| Message::SettingsToggleSpellCheck(val));

    let smart_punct_toggle = toggler(settings.smart_punctuation)
        .label("Smart punctuation")
        .on_toggle(|val| Message::SettingsToggleSmartPunctuation(val));

    // ── Binder ──
    let synopsis_toggle = toggler(settings.show_synopsis_in_binder)
        .label("Show document synopses in binder")
        .on_toggle(|val| Message::SettingsToggleShowSynopsis(val));

    let auto_number_toggle = toggler(settings.auto_numbering)
        .label("Auto-number chapters on compile")
        .on_toggle(|val| Message::SettingsToggleAutoNumbering(val));

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

    // ── Script mode ──
    let script_toggle = toggler(script_mode)
        .label("Enable script/screenplay mode")
        .on_toggle(|_| Message::ToggleScriptMode);

    // ── Auto-correction ──
    let smart_quotes_toggle = toggler(auto_correction.smart_quotes)
        .label("Smart quotes  \u{201C}...\u{201D}")
        .on_toggle(|_| Message::ToggleAutoCorrectSmartQuotes);
    let em_dash_toggle = toggler(auto_correction.em_dashes)
        .label("Em dashes  -- \u{2192} \u{2014}")
        .on_toggle(|_| Message::ToggleAutoCorrectEmDashes);
    let ellipsis_toggle = toggler(auto_correction.ellipsis)
        .label("Ellipsis  ... \u{2192} \u{2026}")
        .on_toggle(|_| Message::ToggleAutoCorrectEllipsis);

    column![
        section_header("Editor Behavior"),
        typewriter_toggle,
        setting_description("Keep the cursor line centered in the editor viewport"),
        Space::with_height(6),
        spell_check_toggle,
        setting_description("Highlight misspelled words as you type"),
        Space::with_height(6),
        smart_punct_toggle,
        setting_description("Auto-replace straight quotes, dashes, and ellipsis"),
        Space::with_height(16),
        divider(),

        section_header("Binder & Organization"),
        synopsis_toggle,
        Space::with_height(6),
        auto_number_toggle,
        Space::with_height(10),
        setting_label("Default New Document Type"),
        setting_description("Type assigned to newly created documents"),
        Space::with_height(4),
        doc_type_picker,
        Space::with_height(16),
        divider(),

        section_header("Script Mode"),
        script_toggle,
        setting_description("Format documents as screenplays with scene headings, action, dialogue, etc."),
        Space::with_height(16),
        divider(),

        section_header("Auto-Correction"),
        setting_description("Automatic text replacements applied while typing"),
        Space::with_height(8),
        smart_quotes_toggle,
        Space::with_height(4),
        em_dash_toggle,
        Space::with_height(4),
        ellipsis_toggle,
    ]
    .spacing(2)
    .max_width(520)
    .into()
}
