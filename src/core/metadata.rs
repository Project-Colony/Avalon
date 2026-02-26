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
        if hex.len() < 6 {
            return iced::Color::from_rgb(0.5, 0.5, 0.5);
        }
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
        Self { name: name.to_string() }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

    // --- Appearance ---
    #[serde(default = "default_editor_width")]
    pub editor_width: f32,

    // --- Editor behavior ---
    #[serde(default)]
    pub typewriter_scroll: bool,
    #[serde(default)]
    pub show_paragraph_marks: bool,
    #[serde(default = "default_true")]
    pub spell_check_enabled: bool,
    #[serde(default = "default_true")]
    pub smart_punctuation: bool,
    #[serde(default)]
    pub default_doc_type: String,
    #[serde(default = "default_true")]
    pub show_synopsis_in_binder: bool,
    #[serde(default)]
    pub auto_numbering: bool,

    // --- Accessibility ---
    #[serde(default)]
    pub high_contrast: bool,
    #[serde(default)]
    pub large_ui: bool,
    #[serde(default)]
    pub reduce_motion: bool,
    #[serde(default)]
    pub screen_reader_hints: bool,
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,

    // --- Backup ---
    #[serde(default = "default_true")]
    pub auto_backup: bool,
    #[serde(default = "default_backup_interval")]
    pub backup_interval_saves: u32,
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

fn default_editor_width() -> f32 {
    80.0
}

fn default_true() -> bool {
    true
}

fn default_ui_scale() -> f32 {
    1.0
}

fn default_backup_interval() -> u32 {
    10
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            labels: vec![
                Label {
                    name: "Concept".into(),
                    color: LabelColor::Red,
                },
                Label {
                    name: "Chapter".into(),
                    color: LabelColor::Blue,
                },
                Label {
                    name: "Scene".into(),
                    color: LabelColor::Green,
                },
                Label {
                    name: "Notes".into(),
                    color: LabelColor::Yellow,
                },
                Label {
                    name: "Character".into(),
                    color: LabelColor::Purple,
                },
                Label {
                    name: "Setting".into(),
                    color: LabelColor::Orange,
                },
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
            editor_width: default_editor_width(),
            typewriter_scroll: false,
            show_paragraph_marks: false,
            spell_check_enabled: true,
            smart_punctuation: true,
            default_doc_type: String::new(),
            show_synopsis_in_binder: true,
            auto_numbering: false,
            high_contrast: false,
            large_ui: false,
            reduce_motion: false,
            screen_reader_hints: false,
            ui_scale: default_ui_scale(),
            auto_backup: true,
            backup_interval_saves: default_backup_interval(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_color() {
        let label = Label {
            name: "Chapter".to_string(),
            color: LabelColor::Red,
        };
        assert_eq!(label.color.to_hex(), "#e74c3c");
    }

    #[test]
    fn test_project_settings_default() {
        let settings = ProjectSettings::default();
        assert_eq!(settings.editor_font_size, 16.0);
        assert_eq!(settings.editor_zoom, 1.0);
        assert!(settings.show_word_count);
        assert!(!settings.labels.is_empty());
        assert!(!settings.statuses.is_empty());
    }

    #[test]
    fn test_settings_serialization() {
        let settings = ProjectSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let parsed: ProjectSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.editor_font_size, 16.0);
        assert_eq!(parsed.labels.len(), settings.labels.len());
    }

    #[test]
    fn test_custom_field_value_equality() {
        assert_eq!(
            CustomFieldValue::Text("hello".into()),
            CustomFieldValue::Text("hello".into())
        );
        assert_ne!(CustomFieldValue::Text("hello".into()), CustomFieldValue::Number(0.0));
    }

    #[test]
    fn test_status_new() {
        let status = Status::new("Review");
        assert_eq!(status.name, "Review");
    }

    #[test]
    fn test_status_defaults() {
        let defaults = Status::defaults();
        assert!(defaults.len() >= 5);
        assert!(defaults.iter().any(|s| s.name == "To Do"));
        assert!(defaults.iter().any(|s| s.name == "Done"));
    }

    #[test]
    fn test_label_color_custom_hex() {
        let custom = LabelColor::Custom("#ff5500".to_string());
        assert_eq!(custom.to_hex(), "#ff5500");
        let iced = custom.to_iced_color();
        assert!((iced.r - 1.0).abs() < 0.01);
        assert!((iced.g - 85.0 / 255.0).abs() < 0.01);
    }

    #[test]
    fn test_project_settings_line_spacing() {
        let settings = ProjectSettings::default();
        assert!((settings.line_spacing - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_project_settings_fullscreen_defaults() {
        let settings = ProjectSettings::default();
        assert_eq!(settings.fullscreen_bg_color, "#1a1a1e");
        assert!((settings.fullscreen_text_width - 60.0).abs() < 0.01);
    }
}
