use serde::{Deserialize, Serialize};

/// Metadata associated with a binder item
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metadata {
    pub label: Option<Label>,
    pub status: Option<Status>,
    pub custom_metadata: Vec<CustomField>,
    pub keywords: Vec<String>,
}

impl Metadata {
    /// Check if a keyword exists (case-insensitive)
    pub fn has_keyword(&self, keyword: &str) -> bool {
        let lower = keyword.to_lowercase();
        self.keywords.iter().any(|k| k.to_lowercase() == lower)
    }

    /// Add a keyword (avoids duplicates)
    pub fn add_keyword(&mut self, keyword: &str) {
        if !self.has_keyword(keyword) {
            self.keywords.push(keyword.to_string());
        }
    }

    /// Remove a keyword (case-insensitive)
    pub fn remove_keyword(&mut self, keyword: &str) {
        let lower = keyword.to_lowercase();
        self.keywords.retain(|k| k.to_lowercase() != lower);
    }

    /// Get a custom field value by name
    pub fn get_custom_field(&self, name: &str) -> Option<&CustomFieldValue> {
        self.custom_metadata
            .iter()
            .find(|f| f.name == name)
            .map(|f| &f.value)
    }

    /// Set a custom field value, updating if it already exists
    pub fn set_custom_field(&mut self, name: &str, value: CustomFieldValue) {
        if let Some(field) = self.custom_metadata.iter_mut().find(|f| f.name == name) {
            field.value = value;
        } else {
            self.custom_metadata.push(CustomField {
                name: name.to_string(),
                value,
            });
        }
    }

    /// Remove a custom field by name
    pub fn remove_custom_field(&mut self, name: &str) {
        self.custom_metadata.retain(|f| f.name != name);
    }

    /// Get the label name, if set
    pub fn label_name(&self) -> Option<&str> {
        self.label.as_ref().map(|l| l.name.as_str())
    }

    /// Get the status name, if set
    pub fn status_name(&self) -> Option<&str> {
        self.status.as_ref().map(|s| s.name.as_str())
    }

    /// Check if any metadata is set
    pub fn is_empty(&self) -> bool {
        self.label.is_none()
            && self.status.is_none()
            && self.custom_metadata.is_empty()
            && self.keywords.is_empty()
    }

    /// Summary string for display
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref label) = self.label {
            parts.push(format!("Label: {}", label.name));
        }
        if let Some(ref status) = self.status {
            parts.push(format!("Status: {}", status.name));
        }
        if !self.keywords.is_empty() {
            parts.push(format!("Keywords: {}", self.keywords.join(", ")));
        }
        if !self.custom_metadata.is_empty() {
            parts.push(format!("{} custom field(s)", self.custom_metadata.len()));
        }
        if parts.is_empty() {
            "No metadata".to_string()
        } else {
            parts.join(" | ")
        }
    }
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

    /// All predefined colors
    pub fn all_predefined() -> Vec<Self> {
        vec![
            LabelColor::Red,
            LabelColor::Orange,
            LabelColor::Yellow,
            LabelColor::Green,
            LabelColor::Blue,
            LabelColor::Purple,
        ]
    }

    /// Display name for the color
    pub fn display_name(&self) -> &str {
        match self {
            LabelColor::Red => "Red",
            LabelColor::Orange => "Orange",
            LabelColor::Yellow => "Yellow",
            LabelColor::Green => "Green",
            LabelColor::Blue => "Blue",
            LabelColor::Purple => "Purple",
            LabelColor::Custom(_) => "Custom",
        }
    }
}

impl Label {
    pub fn new(name: &str, color: LabelColor) -> Self {
        Self {
            name: name.to_string(),
            color,
        }
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

impl CustomFieldValue {
    /// Get the type name as a string
    pub fn type_name(&self) -> &str {
        match self {
            CustomFieldValue::Text(_) => "Text",
            CustomFieldValue::Number(_) => "Number",
            CustomFieldValue::Checkbox(_) => "Checkbox",
            CustomFieldValue::Date(_) => "Date",
            CustomFieldValue::List(_) => "List",
        }
    }

    /// Display value as string
    pub fn display(&self) -> String {
        match self {
            CustomFieldValue::Text(s) => s.clone(),
            CustomFieldValue::Number(n) => format!("{}", n),
            CustomFieldValue::Checkbox(b) => if *b { "Yes" } else { "No" }.to_string(),
            CustomFieldValue::Date(d) => d.clone(),
            CustomFieldValue::List(items) => items.join(", "),
        }
    }

    /// Check if the value is empty/default
    pub fn is_empty(&self) -> bool {
        match self {
            CustomFieldValue::Text(s) => s.is_empty(),
            CustomFieldValue::Number(n) => *n == 0.0,
            CustomFieldValue::Checkbox(b) => !*b,
            CustomFieldValue::Date(d) => d.is_empty(),
            CustomFieldValue::List(items) => items.is_empty(),
        }
    }
}

impl CustomField {
    pub fn text(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: CustomFieldValue::Text(value.to_string()),
        }
    }

    pub fn number(name: &str, value: f64) -> Self {
        Self {
            name: name.to_string(),
            value: CustomFieldValue::Number(value),
        }
    }

    pub fn checkbox(name: &str, value: bool) -> Self {
        Self {
            name: name.to_string(),
            value: CustomFieldValue::Checkbox(value),
        }
    }
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

impl ProjectSettings {
    /// Find a label by name
    pub fn find_label(&self, name: &str) -> Option<&Label> {
        self.labels.iter().find(|l| l.name == name)
    }

    /// Find a status by name
    pub fn find_status(&self, name: &str) -> Option<&Status> {
        self.statuses.iter().find(|s| s.name == name)
    }

    /// Get all label names
    pub fn label_names(&self) -> Vec<&str> {
        self.labels.iter().map(|l| l.name.as_str()).collect()
    }

    /// Get all status names
    pub fn status_names(&self) -> Vec<&str> {
        self.statuses.iter().map(|s| s.name.as_str()).collect()
    }

    /// Check if a target is set and what percentage of progress
    pub fn target_progress(&self, current_words: usize) -> Option<f64> {
        self.target_word_count.map(|target| {
            if target == 0 { 100.0 } else { current_words as f64 / target as f64 * 100.0 }
        })
    }

    /// Days remaining until deadline (None if no deadline set)
    pub fn days_to_deadline(&self) -> Option<i64> {
        self.target_deadline.as_ref().and_then(|d| {
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .ok()
                .map(|deadline| {
                    let today = chrono::Utc::now().date_naive();
                    (deadline - today).num_days()
                })
        })
    }
}
