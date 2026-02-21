use serde::{Deserialize, Serialize};

/// Metadata associated with a binder item
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metadata {
    pub label: Option<Label>,
    pub status: Option<Status>,
    pub custom_metadata: Vec<CustomField>,
    pub keywords: Vec<String>,
}

/// Color-coded label for organizing items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub name: String,
    pub color: LabelColor,
}

/// Predefined label colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LabelColor {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
    Custom(String), // Hex color
}

impl LabelColor {
    pub fn to_hex(&self) -> &str {
        match self {
            LabelColor::Red => "#e74c3c",
            LabelColor::Orange => "#e67e22",
            LabelColor::Yellow => "#f1c40f",
            LabelColor::Green => "#2ecc71",
            LabelColor::Blue => "#3498db",
            LabelColor::Purple => "#9b59b6",
            LabelColor::Custom(hex) => hex,
        }
    }

    pub fn to_iced_color(&self) -> iced::Color {
        let hex = self.to_hex().trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0;
        iced::Color::from_rgb(r, g, b)
    }
}

/// Status of a document (e.g., "First Draft", "Revised", "Final")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub name: String,
}

impl Status {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    /// Default statuses
    pub fn defaults() -> Vec<Self> {
        vec![
            Self::new("To Do"),
            Self::new("First Draft"),
            Self::new("Revised Draft"),
            Self::new("Final Draft"),
            Self::new("Done"),
        ]
    }
}

/// Custom user-defined metadata field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomField {
    pub name: String,
    pub value: CustomFieldValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomFieldValue {
    Text(String),
    Number(f64),
    Checkbox(bool),
    Date(String),
    List(Vec<String>),
}

/// Project-level settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub labels: Vec<Label>,
    pub statuses: Vec<Status>,
    pub editor_zoom: f32,
    pub editor_font: String,
    pub editor_font_size: f32,
    pub show_word_count: bool,
    pub target_word_count: Option<usize>,
    #[serde(default)]
    pub target_deadline: Option<String>,
    pub auto_save_seconds: u32,
    #[serde(default = "default_fullscreen_bg")]
    pub fullscreen_bg_color: String,
    #[serde(default = "default_fullscreen_width")]
    pub fullscreen_text_width: f32,
    #[serde(default)]
    pub composition_bg_color: Option<String>,
    #[serde(default = "default_line_spacing")]
    pub line_spacing: f32,
}

fn default_fullscreen_bg() -> String {
    "#1a1a1e".to_string()
}

fn default_fullscreen_width() -> f32 {
    60.0
}

fn default_line_spacing() -> f32 {
    1.5
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            labels: vec![
                Label { name: "Concept".into(), color: LabelColor::Red },
                Label { name: "Chapter".into(), color: LabelColor::Blue },
                Label { name: "Scene".into(), color: LabelColor::Green },
                Label { name: "Notes".into(), color: LabelColor::Yellow },
                Label { name: "Character".into(), color: LabelColor::Purple },
                Label { name: "Setting".into(), color: LabelColor::Orange },
            ],
            statuses: Status::defaults(),
            editor_zoom: 1.0,
            editor_font: "monospace".to_string(),
            editor_font_size: 16.0,
            show_word_count: true,
            target_word_count: None,
            target_deadline: None,
            auto_save_seconds: 30,
            fullscreen_bg_color: default_fullscreen_bg(),
            fullscreen_text_width: default_fullscreen_width(),
            composition_bg_color: None,
            line_spacing: default_line_spacing(),
        }
    }
}
