use iced::Color;

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
}

impl Theme {
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
}
