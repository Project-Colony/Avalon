use iced::widget::{button, container};
use iced::{Background, Border, Color, Shadow, Vector};

/// Nerd Font icon constants (JetBrains Mono Nerd Font)
///
/// All icons use codepoints from Font Awesome (nf-fa-*) and other ranges
/// bundled in the Nerd Font patched version of JetBrains Mono.
pub struct Icons;

impl Icons {
    // ── File & document ──────────────────────────────────────
    pub const FOLDER: &str = "\u{f07b}";          // nf-fa-folder
    pub const FOLDER_OPEN: &str = "\u{f07c}";     // nf-fa-folder_open
    pub const FILE_TEXT: &str = "\u{f15c}";        // nf-fa-file_text
    pub const FILE_PDF: &str = "\u{f1c1}";         // nf-fa-file_pdf_o
    pub const FILE_IMAGE: &str = "\u{f1c5}";       // nf-fa-file_image_o
    pub const FILE_CODE: &str = "\u{f1c9}";        // nf-fa-file_code_o
    pub const CLIPBOARD: &str = "\u{f0ea}";        // nf-fa-clipboard
    pub const SAVE: &str = "\u{f0c7}";             // nf-fa-floppy_o
    pub const BOOK: &str = "\u{f02d}";             // nf-fa-book
    pub const BOOK_OPEN: &str = "\u{f518}";        // nf-fa-book_open (FA5)
    pub const GLOBE: &str = "\u{f0ac}";            // nf-fa-globe
    pub const LINK: &str = "\u{f0c1}";             // nf-fa-link
    pub const TAG: &str = "\u{f02b}";              // nf-fa-tag

    // ── Actions ──────────────────────────────────────────────
    pub const PLUS: &str = "\u{f067}";             // nf-fa-plus
    pub const SEARCH: &str = "\u{f002}";           // nf-fa-search
    pub const PENCIL: &str = "\u{f040}";           // nf-fa-pencil
    pub const CAMERA: &str = "\u{f030}";           // nf-fa-camera
    pub const UNDO: &str = "\u{f0e2}";             // nf-fa-undo
    pub const REFRESH: &str = "\u{f021}";          // nf-fa-refresh
    pub const BAN: &str = "\u{f05e}";              // nf-fa-ban

    // ── Status / feedback ────────────────────────────────────
    pub const CHECK: &str = "\u{f00c}";            // nf-fa-check
    pub const CHECK_CIRCLE: &str = "\u{f058}";     // nf-fa-check_circle
    pub const TIMES: &str = "\u{f00d}";            // nf-fa-times
    pub const WARNING: &str = "\u{f071}";          // nf-fa-warning
    pub const INFO_CIRCLE: &str = "\u{f05a}";      // nf-fa-info_circle
    pub const COMMENT: &str = "\u{f075}";          // nf-fa-comment
    pub const STAR: &str = "\u{f005}";             // nf-fa-star
    pub const STAR_O: &str = "\u{f006}";           // nf-fa-star_o

    // ── Media / playback ─────────────────────────────────────
    pub const PLAY: &str = "\u{f04b}";             // nf-fa-play
    pub const PAUSE: &str = "\u{f04c}";            // nf-fa-pause
    pub const STOP: &str = "\u{f04d}";             // nf-fa-stop
    pub const CLOCK: &str = "\u{f017}";            // nf-fa-clock_o
    pub const COFFEE: &str = "\u{f0f4}";           // nf-fa-coffee
    pub const FIRE: &str = "\u{f06d}";             // nf-fa-fire
    pub const BOLT: &str = "\u{f0e7}";             // nf-fa-bolt

    // ── Navigation / arrows ──────────────────────────────────
    pub const ARROW_RIGHT: &str = "\u{f061}";      // nf-fa-arrow_right
    pub const ARROW_LEFT: &str = "\u{f060}";       // nf-fa-arrow_left
    pub const ARROW_UP: &str = "\u{f062}";         // nf-fa-arrow_up
    pub const ARROW_DOWN: &str = "\u{f063}";       // nf-fa-arrow_down
    pub const CARET_RIGHT: &str = "\u{f0da}";      // nf-fa-caret_right
    pub const CARET_DOWN: &str = "\u{f0d7}";       // nf-fa-caret_down
    pub const ARROWS_H: &str = "\u{f07e}";         // nf-fa-arrows_h

    // ── Shapes ───────────────────────────────────────────────
    pub const CIRCLE: &str = "\u{f111}";           // nf-fa-circle
    pub const CIRCLE_O: &str = "\u{f10c}";         // nf-fa-circle_o
    pub const BARS: &str = "\u{f0c9}";             // nf-fa-bars

    // ── Categories (template icons) ──────────────────────────
    pub const FILM: &str = "\u{f008}";             // nf-fa-film
    pub const GRADUATION: &str = "\u{f19d}";       // nf-fa-graduation_cap
    pub const NEWSPAPER: &str = "\u{f1ea}";        // nf-fa-newspaper_o
    pub const CALENDAR: &str = "\u{f073}";         // nf-fa-calendar
    pub const LIGHTBULB: &str = "\u{f0eb}";        // nf-fa-lightbulb_o
    pub const TROPHY: &str = "\u{f091}";           // nf-fa-trophy
    pub const PAINT_BRUSH: &str = "\u{f1fc}";      // nf-fa-paint_brush
    pub const STICKY_NOTE: &str = "\u{f249}";      // nf-fa-sticky_note
    pub const PENCIL_SQUARE: &str = "\u{f044}";    // nf-fa-pencil_square_o
}

/// Color palette for the Scrinever UI
/// Dark theme inspired by professional writing applications
pub struct Theme;

impl Theme {
    // ── Background colors ──────────────────────────────
    pub const BG_PRIMARY: Color = Color::from_rgb(0.15, 0.15, 0.18);
    pub const BG_SECONDARY: Color = Color::from_rgb(0.18, 0.18, 0.22);
    pub const BG_TERTIARY: Color = Color::from_rgb(0.22, 0.22, 0.26);
    pub const BG_EDITOR: Color = Color::from_rgb(0.12, 0.12, 0.14);
    pub const BG_CARD: Color = Color::from_rgb(0.20, 0.20, 0.24);
    pub const BG_HOVER: Color = Color::from_rgb(0.25, 0.25, 0.30);
    pub const BG_SELECTED: Color = Color::from_rgb(0.20, 0.30, 0.45);
    pub const BG_TOOLBAR: Color = Color::from_rgb(0.17, 0.17, 0.21);
    pub const BG_DIALOG: Color = Color::from_rgb(0.16, 0.16, 0.20);
    pub const BG_INPUT: Color = Color::from_rgb(0.13, 0.13, 0.16);
    pub const BG_STATUS_BAR: Color = Color::from_rgb(0.14, 0.14, 0.17);

    // ── Text colors ────────────────────────────────────
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.90, 0.90, 0.92);
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.65, 0.65, 0.70);
    pub const TEXT_MUTED: Color = Color::from_rgb(0.45, 0.45, 0.50);
    pub const TEXT_ACCENT: Color = Color::from_rgb(0.40, 0.70, 0.95);
    pub const TEXT_DISABLED: Color = Color::from_rgb(0.35, 0.35, 0.38);
    pub const TEXT_LINK: Color = Color::from_rgb(0.45, 0.75, 1.00);

    // ── Accent / semantic colors ───────────────────────
    pub const ACCENT: Color = Color::from_rgb(0.30, 0.60, 0.90);
    pub const ACCENT_HOVER: Color = Color::from_rgb(0.35, 0.65, 0.95);
    pub const SUCCESS: Color = Color::from_rgb(0.30, 0.75, 0.45);
    pub const WARNING: Color = Color::from_rgb(0.90, 0.70, 0.20);
    pub const ERROR: Color = Color::from_rgb(0.90, 0.30, 0.30);
    pub const INFO: Color = Color::from_rgb(0.35, 0.65, 0.90);

    // ── Diff / code review colors ──────────────────────
    pub const DIFF_ADDED: Color = Color::from_rgb(0.20, 0.55, 0.30);
    pub const DIFF_REMOVED: Color = Color::from_rgb(0.60, 0.22, 0.22);
    pub const DIFF_CHANGED: Color = Color::from_rgb(0.55, 0.50, 0.20);

    // ── Border colors ──────────────────────────────────
    pub const BORDER: Color = Color::from_rgb(0.30, 0.30, 0.35);
    pub const BORDER_FOCUSED: Color = Color::from_rgb(0.40, 0.60, 0.90);
    pub const BORDER_SUBTLE: Color = Color::from_rgb(0.24, 0.24, 0.28);

    // ── Corkboard colors ───────────────────────────────
    pub const CORK_BG: Color = Color::from_rgb(0.55, 0.42, 0.30);
    pub const CARD_BG: Color = Color::from_rgb(0.98, 0.95, 0.85);
    pub const CARD_TEXT: Color = Color::from_rgb(0.15, 0.15, 0.15);
    pub const CARD_HEADER: Color = Color::from_rgb(0.85, 0.80, 0.70);
    pub const CARD_SHADOW: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.3);

    // ── Sidebar specific ───────────────────────────────
    pub const SIDEBAR_BG: Color = Color::from_rgb(0.16, 0.17, 0.20);
    pub const SIDEBAR_ITEM_HOVER: Color = Color::from_rgb(0.22, 0.23, 0.27);

    // ── Annotation highlight colors ────────────────────
    pub const HIGHLIGHT_YELLOW: Color = Color::from_rgba(0.95, 0.77, 0.06, 0.3);
    pub const HIGHLIGHT_BLUE: Color = Color::from_rgba(0.20, 0.60, 0.86, 0.3);
    pub const HIGHLIGHT_GREEN: Color = Color::from_rgba(0.18, 0.80, 0.44, 0.3);
    pub const HIGHLIGHT_RED: Color = Color::from_rgba(0.91, 0.30, 0.24, 0.3);
    pub const HIGHLIGHT_PURPLE: Color = Color::from_rgba(0.61, 0.35, 0.71, 0.3);

    // ── Script mode colors ─────────────────────────────
    pub const SCRIPT_SCENE: Color = Color::from_rgb(0.50, 0.75, 0.50);
    pub const SCRIPT_ACTION: Color = Color::from_rgb(0.85, 0.85, 0.85);
    pub const SCRIPT_CHARACTER: Color = Color::from_rgb(0.70, 0.60, 0.95);
    pub const SCRIPT_DIALOGUE: Color = Color::from_rgb(0.90, 0.85, 0.70);
    pub const SCRIPT_TRANSITION: Color = Color::from_rgb(0.60, 0.60, 0.70);

    /// Parse a hex color string (e.g. "#3498db") to an iced Color
    pub fn from_hex(hex: &str) -> Color {
        let hex = hex.trim_start_matches('#');
        if hex.len() < 6 {
            return Self::TEXT_MUTED;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0;
        Color::from_rgb(r, g, b)
    }

    /// Dim a color by a factor (0.0 = black, 1.0 = unchanged)
    pub fn dim(color: Color, factor: f32) -> Color {
        Color::from_rgb(color.r * factor, color.g * factor, color.b * factor)
    }

    /// Lighten a color toward white
    pub fn lighten(color: Color, amount: f32) -> Color {
        Color::from_rgb(
            (color.r + amount).min(1.0),
            (color.g + amount).min(1.0),
            (color.b + amount).min(1.0),
        )
    }

    /// Make a color semi-transparent
    pub fn with_alpha(color: Color, alpha: f32) -> Color {
        Color::from_rgba(color.r, color.g, color.b, alpha)
    }

    /// Blend two colors by a given ratio (0.0 = color_a, 1.0 = color_b)
    pub fn blend(color_a: Color, color_b: Color, ratio: f32) -> Color {
        let r = ratio.clamp(0.0, 1.0);
        Color::from_rgb(
            color_a.r * (1.0 - r) + color_b.r * r,
            color_a.g * (1.0 - r) + color_b.g * r,
            color_a.b * (1.0 - r) + color_b.b * r,
        )
    }

    /// Convert an iced Color to a hex string (#RRGGBB)
    pub fn to_hex(color: Color) -> String {
        let r = (color.r * 255.0) as u8;
        let g = (color.g * 255.0) as u8;
        let b = (color.b * 255.0) as u8;
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }

    /// Get a label color by index (cycling through predefined colors)
    pub fn label_color(index: usize) -> Color {
        const LABEL_COLORS: [Color; 8] = [
            Color::from_rgb(0.91, 0.30, 0.24), // Red
            Color::from_rgb(0.90, 0.49, 0.13), // Orange
            Color::from_rgb(0.95, 0.77, 0.06), // Yellow
            Color::from_rgb(0.18, 0.80, 0.44), // Green
            Color::from_rgb(0.20, 0.60, 0.86), // Blue
            Color::from_rgb(0.61, 0.35, 0.71), // Purple
            Color::from_rgb(0.58, 0.29, 0.16), // Brown
            Color::from_rgb(0.50, 0.55, 0.60), // Gray
        ];
        LABEL_COLORS[index % LABEL_COLORS.len()]
    }

    /// Get a progress bar color based on percentage (red->yellow->green)
    pub fn progress_color(pct: f64) -> Color {
        if pct >= 100.0 {
            Self::SUCCESS
        } else if pct >= 75.0 {
            Color::from_rgb(0.30, 0.70, 0.35)
        } else if pct >= 50.0 {
            Self::WARNING
        } else if pct >= 25.0 {
            Color::from_rgb(0.85, 0.55, 0.20)
        } else {
            Color::from_rgb(0.75, 0.35, 0.25)
        }
    }

    /// Desaturate a color (move toward gray)
    pub fn desaturate(color: Color, amount: f32) -> Color {
        let gray = color.r * 0.299 + color.g * 0.587 + color.b * 0.114;
        let a = amount.clamp(0.0, 1.0);
        Color::from_rgb(
            color.r * (1.0 - a) + gray * a,
            color.g * (1.0 - a) + gray * a,
            color.b * (1.0 - a) + gray * a,
        )
    }

    /// Invert a color
    pub fn invert(color: Color) -> Color {
        Color::from_rgb(1.0 - color.r, 1.0 - color.g, 1.0 - color.b)
    }

    /// Calculate perceived brightness (0.0 = black, 1.0 = white)
    pub fn brightness(color: Color) -> f32 {
        color.r * 0.299 + color.g * 0.587 + color.b * 0.114
    }

    /// Get a contrasting text color (white or black) for a given background
    pub fn contrast_text(bg: Color) -> Color {
        if Self::brightness(bg) > 0.5 {
            Color::from_rgb(0.1, 0.1, 0.1)
        } else {
            Color::from_rgb(0.95, 0.95, 0.95)
        }
    }

    /// Get a status color for writing status labels
    pub fn status_color(status: &str) -> Color {
        match status {
            "To Do" => Self::ERROR,
            "In Progress" | "First Draft" => Self::WARNING,
            "Revised Draft" => Self::TEXT_ACCENT,
            "Final Draft" | "Done" => Self::SUCCESS,
            _ => Self::TEXT_SECONDARY,
        }
    }

    /// Get all highlight colors as a list
    pub fn highlight_colors() -> Vec<(&'static str, Color)> {
        vec![
            ("Yellow", Self::HIGHLIGHT_YELLOW),
            ("Blue", Self::HIGHLIGHT_BLUE),
            ("Green", Self::HIGHLIGHT_GREEN),
            ("Red", Self::HIGHLIGHT_RED),
            ("Purple", Self::HIGHLIGHT_PURPLE),
        ]
    }

    /// Get all script element colors as a list
    pub fn script_colors() -> Vec<(&'static str, Color)> {
        vec![
            ("Scene Heading", Self::SCRIPT_SCENE),
            ("Action", Self::SCRIPT_ACTION),
            ("Character", Self::SCRIPT_CHARACTER),
            ("Dialogue", Self::SCRIPT_DIALOGUE),
            ("Transition", Self::SCRIPT_TRANSITION),
        ]
    }
}

// ── Shared UI Style Functions ────────────────────────────────────────
//
// These provide consistent container styles used across all views.
// Every panel, sidebar, dialog, and card should use these functions
// instead of defining ad-hoc inline styles.

/// Style for bottom panels (search, timer, session, annotations, etc.)
pub fn panel_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Theme::BG_SECONDARY)),
        border: Border {
            color: Theme::BORDER_SUBTLE,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Style for the sidebar (binder, inspector)
pub fn sidebar_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Theme::SIDEBAR_BG)),
        border: Border {
            color: Theme::BORDER_SUBTLE,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Style for panel section headers (the colored top strip)
pub fn panel_header_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Theme::BG_TERTIARY)),
        border: Border {
            color: Theme::BORDER_SUBTLE,
            width: 0.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Style for cards within panels (stat blocks, template cards, entries)
pub fn card_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Theme::BG_CARD)),
        border: Border {
            color: Theme::BORDER_SUBTLE,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Style for elevated cards with a subtle accent border
pub fn accent_card_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.20, 0.35, 0.55, 0.15))),
        border: Border {
            color: Color::from_rgba(0.30, 0.60, 0.90, 0.30),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Style for full-screen dialogs (compile, settings, project stats)
pub fn dialog_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Theme::BG_DIALOG)),
        border: Border {
            color: Theme::BORDER,
            width: 1.0,
            radius: 10.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        ..Default::default()
    }
}

/// Style for the status bar at the bottom
pub fn status_bar_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Theme::BG_STATUS_BAR)),
        border: Border {
            color: Theme::BORDER_SUBTLE,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Style for the toolbar / menu bar area
pub fn toolbar_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Theme::BG_TOOLBAR)),
        border: Border {
            color: Theme::BORDER_SUBTLE,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Style for view headers (editor header, corkboard header, etc.)
pub fn view_header_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Theme::BG_TERTIARY)),
        border: Border {
            color: Theme::BORDER_SUBTLE,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Style for view footers (editor stats bar, corkboard footer, etc.)
pub fn view_footer_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Theme::BG_PRIMARY)),
        border: Border {
            color: Theme::BORDER_SUBTLE,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Style for separator lines within panels
pub fn separator_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Theme::BORDER_SUBTLE)),
        ..Default::default()
    }
}

/// Style for the editor content area
pub fn editor_bg_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Theme::BG_EDITOR)),
        ..Default::default()
    }
}

/// Style for the welcome screen hero section
pub fn hero_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Theme::BG_EDITOR)),
        border: Border {
            color: Theme::BORDER_SUBTLE,
            width: 0.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}

/// Style for the footer section on welcome screen
pub fn footer_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.14, 0.14, 0.17, 0.8))),
        border: Border {
            color: Theme::BORDER_SUBTLE,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

// ── Binder-specific styles ──────────────────────────────────────────

/// Style for binder section headers (Draft, Research, Trash)
pub fn binder_section_header_style(accent: Color) -> impl Fn(&iced::Theme) -> container::Style {
    move |_theme: &iced::Theme| container::Style {
        background: Some(Background::Color(Color::from_rgba(
            accent.r * 0.15,
            accent.g * 0.15,
            accent.b * 0.15,
            0.6,
        ))),
        border: Border {
            color: Color::from_rgba(accent.r, accent.g, accent.b, 0.25),
            width: 0.0,
            radius: 5.0.into(),
        },
        ..Default::default()
    }
}

/// Style for the binder item count badge
pub fn binder_badge_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.07))),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

/// Button style for binder section headers
pub fn binder_section_btn_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.05))),
        button::Status::Pressed => Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08))),
        _ => None,
    };
    button::Style {
        background: bg,
        text_color: Theme::TEXT_PRIMARY,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 5.0.into(),
        },
        ..Default::default()
    }
}

/// Button style for a binder tree item (not selected)
pub fn binder_item_btn_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(Background::Color(Theme::SIDEBAR_ITEM_HOVER)),
        button::Status::Pressed => Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.10))),
        _ => None,
    };
    button::Style {
        background: bg,
        text_color: Theme::TEXT_SECONDARY,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// Button style for a selected binder tree item
pub fn binder_item_selected_btn_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg_color = match status {
        button::Status::Hovered => Color::from_rgba(0.22, 0.35, 0.55, 0.50),
        _ => Color::from_rgba(0.20, 0.32, 0.50, 0.40),
    };
    button::Style {
        background: Some(Background::Color(bg_color)),
        text_color: Theme::TEXT_PRIMARY,
        border: Border {
            color: Color::from_rgba(0.35, 0.60, 0.90, 0.40),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// Button style for binder action buttons (bottom toolbar)
pub fn binder_action_btn_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.10))),
        button::Status::Pressed => Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.15))),
        _ => Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.04))),
    };
    button::Style {
        background: bg,
        text_color: Theme::TEXT_SECONDARY,
        border: Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// Button style for destructive actions (empty trash, delete)
pub fn binder_danger_btn_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(Background::Color(Color::from_rgba(0.90, 0.30, 0.30, 0.20))),
        button::Status::Pressed => Some(Background::Color(Color::from_rgba(0.90, 0.30, 0.30, 0.30))),
        _ => Some(Background::Color(Color::from_rgba(0.90, 0.30, 0.30, 0.08))),
    };
    button::Style {
        background: bg,
        text_color: Theme::ERROR,
        border: Border {
            color: Color::from_rgba(0.90, 0.30, 0.30, 0.20),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// Container style for the binder bottom action bar
pub fn binder_action_bar_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.14, 0.15, 0.18))),
        border: Border {
            color: Theme::BORDER_SUBTLE,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Thin horizontal separator for between binder sections
pub fn binder_separator_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
        ..Default::default()
    }
}

/// Style for the indentation guide line
pub fn binder_indent_guide_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
        ..Default::default()
    }
}

// ── Inspector-specific styles ───────────────────────────────────────

/// Style for the inspector header bar
pub fn inspector_header_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.13, 0.14, 0.17))),
        border: Border {
            color: Color::from_rgba(0.30, 0.60, 0.90, 0.25),
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Style for inspector section cards (subtle grouped containers)
pub fn inspector_section_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.025))),
        border: Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Style for inspector separator lines
pub fn inspector_separator_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
        ..Default::default()
    }
}

/// Button style for inspector action buttons
pub fn inspector_btn_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.10))),
        button::Status::Pressed => Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.14))),
        _ => Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.04))),
    };
    button::Style {
        background: bg,
        text_color: Theme::TEXT_SECONDARY,
        border: Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            width: 1.0,
            radius: 5.0.into(),
        },
        ..Default::default()
    }
}

/// Button style for inspector accent/primary action buttons
pub fn inspector_accent_btn_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(Background::Color(Color::from_rgba(0.30, 0.60, 0.90, 0.25))),
        button::Status::Pressed => Some(Background::Color(Color::from_rgba(0.30, 0.60, 0.90, 0.35))),
        _ => Some(Background::Color(Color::from_rgba(0.30, 0.60, 0.90, 0.12))),
    };
    button::Style {
        background: bg,
        text_color: Theme::TEXT_ACCENT,
        border: Border {
            color: Color::from_rgba(0.30, 0.60, 0.90, 0.30),
            width: 1.0,
            radius: 5.0.into(),
        },
        ..Default::default()
    }
}

/// Button style for the inspector bookmark toggle
pub fn inspector_bookmark_style(is_active: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme: &iced::Theme, status: button::Status| {
        let base = if is_active { Theme::WARNING } else { Theme::TEXT_MUTED };
        let bg = match status {
            button::Status::Hovered => Some(Background::Color(Color::from_rgba(base.r, base.g, base.b, 0.15))),
            button::Status::Pressed => Some(Background::Color(Color::from_rgba(base.r, base.g, base.b, 0.25))),
            _ => None,
        };
        button::Style {
            background: bg,
            text_color: base,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    }
}

/// Button style for small inline buttons (+Field, x remove)
pub fn inspector_inline_btn_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08))),
        button::Status::Pressed => Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.12))),
        _ => None,
    };
    button::Style {
        background: bg,
        text_color: Theme::TEXT_ACCENT,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 3.0.into(),
        },
        ..Default::default()
    }
}

/// Button style for small danger inline buttons (remove field)
pub fn inspector_danger_btn_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(Background::Color(Color::from_rgba(0.90, 0.30, 0.30, 0.15))),
        button::Status::Pressed => Some(Background::Color(Color::from_rgba(0.90, 0.30, 0.30, 0.25))),
        _ => None,
    };
    button::Style {
        background: bg,
        text_color: Theme::ERROR,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 3.0.into(),
        },
        ..Default::default()
    }
}

/// Style for progress bar background track
pub fn inspector_progress_bg_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 3.0.into(),
        },
        ..Default::default()
    }
}

/// Style for progress bar fill with dynamic color
pub fn inspector_progress_fill_style(color: Color) -> impl Fn(&iced::Theme) -> container::Style {
    move |_theme: &iced::Theme| container::Style {
        background: Some(Background::Color(Color::from_rgba(color.r, color.g, color.b, 0.75))),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 3.0.into(),
        },
        ..Default::default()
    }
}

// ── Shared Utility Functions ────────────────────────────────────────

/// Format a number with abbreviated suffixes (1.2k, 3.5M)
pub fn format_count(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{},{:03}", n / 1000, n % 1000)
    } else {
        n.to_string()
    }
}

/// Format a number with compact suffixes for status bars
pub fn format_compact(n: usize) -> String {
    if n < 1000 {
        n.to_string()
    } else if n < 1_000_000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    }
}

/// Build a text progress bar string
pub fn progress_bar_text(pct: f64, width: usize) -> String {
    let filled = ((pct / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!(
        "{}{}",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(empty),
    )
}

