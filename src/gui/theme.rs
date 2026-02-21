use iced::Color;

/// Color palette for the Scrinever UI
pub struct Theme;

impl Theme {
    // Background colors
    pub const BG_PRIMARY: Color = Color::from_rgb(0.15, 0.15, 0.18);
    pub const BG_SECONDARY: Color = Color::from_rgb(0.18, 0.18, 0.22);
    pub const BG_TERTIARY: Color = Color::from_rgb(0.22, 0.22, 0.26);
    pub const BG_EDITOR: Color = Color::from_rgb(0.12, 0.12, 0.14);
    pub const BG_CARD: Color = Color::from_rgb(0.20, 0.20, 0.24);
    pub const BG_HOVER: Color = Color::from_rgb(0.25, 0.25, 0.30);
    pub const BG_SELECTED: Color = Color::from_rgb(0.20, 0.30, 0.45);

    // Text colors
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.90, 0.90, 0.92);
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.65, 0.65, 0.70);
    pub const TEXT_MUTED: Color = Color::from_rgb(0.45, 0.45, 0.50);
    pub const TEXT_ACCENT: Color = Color::from_rgb(0.40, 0.70, 0.95);

    // Accent colors
    pub const ACCENT: Color = Color::from_rgb(0.30, 0.60, 0.90);
    pub const ACCENT_HOVER: Color = Color::from_rgb(0.35, 0.65, 0.95);
    pub const SUCCESS: Color = Color::from_rgb(0.30, 0.75, 0.45);
    pub const WARNING: Color = Color::from_rgb(0.90, 0.70, 0.20);
    pub const ERROR: Color = Color::from_rgb(0.90, 0.30, 0.30);

    // Border colors
    pub const BORDER: Color = Color::from_rgb(0.30, 0.30, 0.35);
    pub const BORDER_FOCUSED: Color = Color::from_rgb(0.40, 0.60, 0.90);

    // Corkboard colors
    pub const CORK_BG: Color = Color::from_rgb(0.55, 0.42, 0.30);
    pub const CARD_BG: Color = Color::from_rgb(0.98, 0.95, 0.85);
    pub const CARD_TEXT: Color = Color::from_rgb(0.15, 0.15, 0.15);
    pub const CARD_HEADER: Color = Color::from_rgb(0.85, 0.80, 0.70);

    // Sidebar specific
    pub const SIDEBAR_BG: Color = Color::from_rgb(0.16, 0.17, 0.20);
    pub const SIDEBAR_ITEM_HOVER: Color = Color::from_rgb(0.22, 0.23, 0.27);
}
