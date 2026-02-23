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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

    /// Add a new custom label
    pub fn add_label(&mut self, name: &str, color: LabelColor) {
        if !self.labels.iter().any(|l| l.name == name) {
            self.labels.push(Label::new(name, color));
        }
    }

    /// Remove a label by name
    pub fn remove_label(&mut self, name: &str) {
        self.labels.retain(|l| l.name != name);
    }

    /// Add a new custom status
    pub fn add_status(&mut self, name: &str) {
        if !self.statuses.iter().any(|s| s.name == name) {
            self.statuses.push(Status::new(name));
        }
    }

    /// Remove a status by name
    pub fn remove_status(&mut self, name: &str) {
        self.statuses.retain(|s| s.name != name);
    }

    /// Save settings to a standalone file (for sharing/backup)
    pub fn save_to_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize settings: {}", e))?;
        std::fs::write(path, json)
            .map_err(|e| anyhow::anyhow!("Failed to write settings file: {}", e))?;
        Ok(())
    }

    /// Load settings from a standalone file
    pub fn load_from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read settings file: {}", e))?;
        let settings: ProjectSettings = serde_json::from_str(&json)
            .map_err(|e| anyhow::anyhow!("Failed to parse settings: {}", e))?;
        Ok(settings)
    }
}

/// Application-level preferences (persisted across sessions, independent of project)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPreferences {
    /// Preferred window width
    pub window_width: f32,
    /// Preferred window height
    pub window_height: f32,
    /// Show binder sidebar
    pub show_binder: bool,
    /// Show inspector panel
    pub show_inspector: bool,
    /// Last used export format
    pub last_export_format: String,
    /// Theme preference (light/dark)
    pub theme: String,
    /// Spell checker enabled by default
    pub spell_check_enabled: bool,
    /// Default project template
    pub default_template: String,
    /// Auto-save enabled
    pub auto_save_enabled: bool,
    /// Auto-backup interval (in saves)
    pub auto_backup_interval: u32,
    /// Maximum number of recent projects
    pub max_recent_projects: usize,
    /// Default compile options
    #[serde(default)]
    pub default_compile_author: String,
    /// Custom dictionary words
    #[serde(default)]
    pub custom_dictionary: Vec<String>,
    /// Keyboard shortcut overrides
    #[serde(default)]
    pub shortcut_overrides: std::collections::HashMap<String, String>,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            window_width: 1280.0,
            window_height: 800.0,
            show_binder: true,
            show_inspector: true,
            last_export_format: "Markdown".to_string(),
            theme: "dark".to_string(),
            spell_check_enabled: true,
            default_template: String::new(),
            auto_save_enabled: true,
            auto_backup_interval: 10,
            max_recent_projects: 10,
            default_compile_author: String::new(),
            custom_dictionary: Vec::new(),
            shortcut_overrides: std::collections::HashMap::new(),
        }
    }
}

impl AppPreferences {
    /// Get the preferences file path
    pub fn file_path() -> std::path::PathBuf {
        let home = dirs::home_dir().unwrap_or_default();
        home.join(".avalon").join("preferences.json")
    }

    /// Load preferences from disk (returns default if file doesn't exist)
    pub fn load() -> Self {
        let path = Self::file_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(json) => {
                    serde_json::from_str(&json).unwrap_or_default()
                }
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    /// Save preferences to disk
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::file_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Add a word to the custom dictionary
    pub fn add_dictionary_word(&mut self, word: &str) {
        let lower = word.to_lowercase();
        if !self.custom_dictionary.contains(&lower) {
            self.custom_dictionary.push(lower);
        }
    }

    /// Check if a word is in the custom dictionary
    pub fn has_dictionary_word(&self, word: &str) -> bool {
        let lower = word.to_lowercase();
        self.custom_dictionary.contains(&lower)
    }

    /// Set a shortcut override
    pub fn set_shortcut(&mut self, action: &str, shortcut: &str) {
        self.shortcut_overrides.insert(action.to_string(), shortcut.to_string());
    }

    /// Get a shortcut for an action (returns the override or None)
    pub fn get_shortcut(&self, action: &str) -> Option<&str> {
        self.shortcut_overrides.get(action).map(|s| s.as_str())
    }
}

/// Schema definition for custom metadata fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetadataSchema {
    /// Field definitions
    pub fields: Vec<CustomFieldDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFieldDefinition {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: CustomFieldType,
    /// Default value (as string)
    pub default_value: String,
    /// Whether this field is required
    pub required: bool,
    /// Allowed values (for enum-type fields)
    pub allowed_values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CustomFieldType {
    Text,
    Number,
    Checkbox,
    Date,
    Enum,
}

impl CustomMetadataSchema {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    /// Add a text field definition
    pub fn add_text_field(&mut self, name: &str, required: bool) {
        self.fields.push(CustomFieldDefinition {
            name: name.to_string(),
            field_type: CustomFieldType::Text,
            default_value: String::new(),
            required,
            allowed_values: Vec::new(),
        });
    }

    /// Add an enum field definition with allowed values
    pub fn add_enum_field(&mut self, name: &str, values: Vec<String>, required: bool) {
        self.fields.push(CustomFieldDefinition {
            name: name.to_string(),
            field_type: CustomFieldType::Enum,
            default_value: values.first().cloned().unwrap_or_default(),
            required,
            allowed_values: values,
        });
    }

    /// Add a checkbox field definition
    pub fn add_checkbox_field(&mut self, name: &str) {
        self.fields.push(CustomFieldDefinition {
            name: name.to_string(),
            field_type: CustomFieldType::Checkbox,
            default_value: "false".to_string(),
            required: false,
            allowed_values: Vec::new(),
        });
    }

    /// Get field definition by name
    pub fn get_field(&self, name: &str) -> Option<&CustomFieldDefinition> {
        self.fields.iter().find(|f| f.name == name)
    }

    /// Remove a field definition
    pub fn remove_field(&mut self, name: &str) {
        self.fields.retain(|f| f.name != name);
    }

    /// Validate a field value against its definition
    pub fn validate_field(&self, name: &str, value: &str) -> bool {
        match self.get_field(name) {
            Some(def) => {
                match def.field_type {
                    CustomFieldType::Number => value.parse::<f64>().is_ok(),
                    CustomFieldType::Checkbox => value == "true" || value == "false",
                    CustomFieldType::Enum => def.allowed_values.contains(&value.to_string()),
                    _ => true,
                }
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_keywords() {
        let mut meta = Metadata::default();
        assert!(!meta.has_keyword("test"));

        meta.add_keyword("test");
        assert!(meta.has_keyword("test"));
        assert!(meta.has_keyword("TEST"));

        meta.add_keyword("test"); // duplicate
        assert_eq!(meta.keywords.len(), 1);

        meta.remove_keyword("Test");
        assert!(!meta.has_keyword("test"));
    }

    #[test]
    fn test_custom_fields() {
        let mut meta = Metadata::default();
        meta.set_custom_field("POV", CustomFieldValue::Text("First Person".to_string()));
        assert_eq!(
            meta.get_custom_field("POV"),
            Some(&CustomFieldValue::Text("First Person".to_string()))
        );

        meta.set_custom_field("POV", CustomFieldValue::Text("Third Person".to_string()));
        assert_eq!(meta.custom_metadata.len(), 1);

        meta.remove_custom_field("POV");
        assert!(meta.get_custom_field("POV").is_none());
    }

    #[test]
    fn test_label_color() {
        let label = Label {
            name: "Chapter".to_string(),
            color: LabelColor::Red,
        };
        assert_eq!(label.color.to_hex(), "#e74c3c");
    }

    #[test]
    fn test_label_color_all_predefined() {
        let all = LabelColor::all_predefined();
        assert!(all.len() >= 6);
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
    fn test_find_label_and_status() {
        let settings = ProjectSettings::default();
        let label = settings.find_label("Chapter");
        assert!(label.is_some());

        let status = settings.find_status("First Draft");
        assert!(status.is_some());
    }

    #[test]
    fn test_label_and_status_names() {
        let settings = ProjectSettings::default();
        let label_names = settings.label_names();
        assert!(!label_names.is_empty());
        assert!(label_names.contains(&"Chapter"));

        let status_names = settings.status_names();
        assert!(!status_names.is_empty());
    }

    #[test]
    fn test_target_progress() {
        let mut settings = ProjectSettings::default();
        assert!(settings.target_progress(5000).is_none());

        settings.target_word_count = Some(10000);
        let progress = settings.target_progress(5000).unwrap();
        assert!((progress - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_custom_field_value_types() {
        assert_eq!(CustomFieldValue::Text("hello".into()).type_name(), "Text");
        assert_eq!(CustomFieldValue::Number(42.0).type_name(), "Number");
        assert_eq!(CustomFieldValue::Checkbox(true).type_name(), "Checkbox");
        assert_eq!(CustomFieldValue::Date("2024-01-01".into()).type_name(), "Date");
        assert_eq!(CustomFieldValue::List(vec!["a".into()]).type_name(), "List");
    }

    #[test]
    fn test_metadata_label_and_status() {
        let mut meta = Metadata::default();
        assert!(meta.label_name().is_none());
        assert!(meta.status_name().is_none());

        meta.label = Some(Label { name: "Scene".into(), color: LabelColor::Blue });
        meta.status = Some(Status { name: "Done".into() });
        assert_eq!(meta.label_name(), Some("Scene"));
        assert_eq!(meta.status_name(), Some("Done"));
    }

    #[test]
    fn test_add_remove_label() {
        let mut settings = ProjectSettings::default();
        let initial_count = settings.labels.len();
        settings.add_label("Custom Label", LabelColor::Red);
        assert_eq!(settings.labels.len(), initial_count + 1);
        // No duplicate
        settings.add_label("Custom Label", LabelColor::Blue);
        assert_eq!(settings.labels.len(), initial_count + 1);
        // Remove
        settings.remove_label("Custom Label");
        assert_eq!(settings.labels.len(), initial_count);
    }

    #[test]
    fn test_add_remove_status() {
        let mut settings = ProjectSettings::default();
        let initial_count = settings.statuses.len();
        settings.add_status("In Review");
        assert_eq!(settings.statuses.len(), initial_count + 1);
        settings.add_status("In Review"); // duplicate
        assert_eq!(settings.statuses.len(), initial_count + 1);
        settings.remove_status("In Review");
        assert_eq!(settings.statuses.len(), initial_count);
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
    fn test_app_preferences_default() {
        let prefs = AppPreferences::default();
        assert_eq!(prefs.window_width, 1280.0);
        assert!(prefs.show_binder);
        assert!(prefs.auto_save_enabled);
        assert!(prefs.custom_dictionary.is_empty());
    }

    #[test]
    fn test_app_preferences_dictionary() {
        let mut prefs = AppPreferences::default();
        prefs.add_dictionary_word("Scrinever");
        assert!(prefs.has_dictionary_word("scrinever"));
        prefs.add_dictionary_word("scrinever"); // no duplicate
        assert_eq!(prefs.custom_dictionary.len(), 1);
    }

    #[test]
    fn test_app_preferences_shortcuts() {
        let mut prefs = AppPreferences::default();
        prefs.set_shortcut("save", "Ctrl+S");
        assert_eq!(prefs.get_shortcut("save"), Some("Ctrl+S"));
        assert_eq!(prefs.get_shortcut("undefined"), None);
    }

    #[test]
    fn test_app_preferences_serialization() {
        let prefs = AppPreferences::default();
        let json = serde_json::to_string(&prefs).unwrap();
        let parsed: AppPreferences = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.window_width, 1280.0);
    }

    #[test]
    fn test_custom_metadata_schema() {
        let mut schema = CustomMetadataSchema::new();
        schema.add_text_field("POV", false);
        schema.add_enum_field("Genre", vec!["Fantasy".into(), "Sci-Fi".into()], true);
        schema.add_checkbox_field("Reviewed");

        assert_eq!(schema.fields.len(), 3);
        assert!(schema.get_field("POV").is_some());
        assert!(schema.get_field("NotExists").is_none());

        // Validate
        assert!(schema.validate_field("Genre", "Fantasy"));
        assert!(!schema.validate_field("Genre", "Romance"));
        assert!(schema.validate_field("Reviewed", "true"));
        assert!(!schema.validate_field("Reviewed", "maybe"));

        // Remove
        schema.remove_field("POV");
        assert_eq!(schema.fields.len(), 2);
    }

    #[test]
    fn test_custom_field_value_display() {
        assert_eq!(CustomFieldValue::Text("hello".into()).display(), "hello");
        assert_eq!(CustomFieldValue::Number(42.5).display(), "42.5");
        assert_eq!(CustomFieldValue::Checkbox(true).display(), "Yes");
        assert_eq!(CustomFieldValue::Checkbox(false).display(), "No");
        assert_eq!(CustomFieldValue::Date("2024-01-01".into()).display(), "2024-01-01");
        assert_eq!(
            CustomFieldValue::List(vec!["a".into(), "b".into()]).display(),
            "a, b"
        );
    }

    #[test]
    fn test_custom_field_value_is_empty() {
        assert!(CustomFieldValue::Text(String::new()).is_empty());
        assert!(!CustomFieldValue::Text("hello".into()).is_empty());
        assert!(CustomFieldValue::Number(0.0).is_empty());
        assert!(!CustomFieldValue::Number(1.0).is_empty());
        assert!(CustomFieldValue::Checkbox(false).is_empty());
        assert!(!CustomFieldValue::Checkbox(true).is_empty());
        assert!(CustomFieldValue::Date(String::new()).is_empty());
        assert!(!CustomFieldValue::Date("2024".into()).is_empty());
        assert!(CustomFieldValue::List(vec![]).is_empty());
        assert!(!CustomFieldValue::List(vec!["a".into()]).is_empty());
    }

    #[test]
    fn test_metadata_is_empty() {
        let meta = Metadata::default();
        assert!(meta.is_empty());

        let mut meta2 = Metadata::default();
        meta2.add_keyword("tag");
        assert!(!meta2.is_empty());
    }

    #[test]
    fn test_metadata_summary() {
        let meta = Metadata::default();
        assert_eq!(meta.summary(), "No metadata");

        let mut meta2 = Metadata::default();
        meta2.label = Some(Label::new("Scene", LabelColor::Green));
        meta2.status = Some(Status::new("Done"));
        meta2.add_keyword("important");
        let summary = meta2.summary();
        assert!(summary.contains("Label: Scene"));
        assert!(summary.contains("Status: Done"));
        assert!(summary.contains("Keywords: important"));
    }

    #[test]
    fn test_label_color_display_names() {
        assert_eq!(LabelColor::Red.display_name(), "Red");
        assert_eq!(LabelColor::Orange.display_name(), "Orange");
        assert_eq!(LabelColor::Yellow.display_name(), "Yellow");
        assert_eq!(LabelColor::Green.display_name(), "Green");
        assert_eq!(LabelColor::Blue.display_name(), "Blue");
        assert_eq!(LabelColor::Purple.display_name(), "Purple");
        assert_eq!(LabelColor::Custom("#fff".into()).display_name(), "Custom");
    }

    #[test]
    fn test_label_color_to_iced() {
        for color in LabelColor::all_predefined() {
            let iced = color.to_iced_color();
            assert!(iced.r >= 0.0 && iced.r <= 1.0);
            assert!(iced.g >= 0.0 && iced.g <= 1.0);
            assert!(iced.b >= 0.0 && iced.b <= 1.0);
        }
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
    fn test_custom_field_constructors() {
        let text = CustomField::text("Name", "John");
        assert_eq!(text.name, "Name");
        assert_eq!(text.value, CustomFieldValue::Text("John".into()));

        let num = CustomField::number("Count", 42.0);
        assert_eq!(num.name, "Count");
        assert_eq!(num.value, CustomFieldValue::Number(42.0));

        let check = CustomField::checkbox("Done", true);
        assert_eq!(check.name, "Done");
        assert_eq!(check.value, CustomFieldValue::Checkbox(true));
    }

    #[test]
    fn test_status_defaults() {
        let defaults = Status::defaults();
        assert!(defaults.len() >= 5);
        assert!(defaults.iter().any(|s| s.name == "To Do"));
        assert!(defaults.iter().any(|s| s.name == "Done"));
    }

    #[test]
    fn test_target_progress_zero_target() {
        let mut settings = ProjectSettings::default();
        settings.target_word_count = Some(0);
        let progress = settings.target_progress(5000).unwrap();
        assert_eq!(progress, 100.0);
    }

    #[test]
    fn test_days_to_deadline_none() {
        let settings = ProjectSettings::default();
        assert!(settings.days_to_deadline().is_none());
    }

    #[test]
    fn test_days_to_deadline_invalid() {
        let mut settings = ProjectSettings::default();
        settings.target_deadline = Some("not-a-date".to_string());
        assert!(settings.days_to_deadline().is_none());
    }

    #[test]
    fn test_days_to_deadline_future() {
        let mut settings = ProjectSettings::default();
        settings.target_deadline = Some("2030-12-31".to_string());
        let days = settings.days_to_deadline().unwrap();
        assert!(days > 0);
    }

    #[test]
    fn test_days_to_deadline_past() {
        let mut settings = ProjectSettings::default();
        settings.target_deadline = Some("2020-01-01".to_string());
        let days = settings.days_to_deadline().unwrap();
        assert!(days < 0);
    }

    #[test]
    fn test_schema_validate_number() {
        let mut schema = CustomMetadataSchema::new();
        schema.fields.push(CustomFieldDefinition {
            name: "Age".to_string(),
            field_type: CustomFieldType::Number,
            default_value: "0".to_string(),
            required: true,
            allowed_values: Vec::new(),
        });
        assert!(schema.validate_field("Age", "42"));
        assert!(schema.validate_field("Age", "3.14"));
        assert!(!schema.validate_field("Age", "not a number"));
    }

    #[test]
    fn test_schema_validate_text() {
        let mut schema = CustomMetadataSchema::new();
        schema.add_text_field("Notes", false);
        // Text fields accept any value
        assert!(schema.validate_field("Notes", "anything goes"));
        assert!(schema.validate_field("Notes", ""));
    }

    #[test]
    fn test_schema_validate_unknown_field() {
        let schema = CustomMetadataSchema::new();
        assert!(!schema.validate_field("Unknown", "value"));
    }

    #[test]
    fn test_find_label_not_found() {
        let settings = ProjectSettings::default();
        assert!(settings.find_label("NonexistentLabel").is_none());
    }

    #[test]
    fn test_find_status_not_found() {
        let settings = ProjectSettings::default();
        assert!(settings.find_status("NonexistentStatus").is_none());
    }

    #[test]
    fn test_metadata_summary_with_custom_fields() {
        let mut meta = Metadata::default();
        meta.set_custom_field("POV", CustomFieldValue::Text("First".into()));
        let summary = meta.summary();
        assert!(summary.contains("1 custom field"));
    }

    #[test]
    fn test_metadata_multiple_keywords() {
        let mut meta = Metadata::default();
        meta.add_keyword("fantasy");
        meta.add_keyword("adventure");
        meta.add_keyword("dragons");
        assert_eq!(meta.keywords.len(), 3);
        meta.remove_keyword("adventure");
        assert_eq!(meta.keywords.len(), 2);
        assert!(!meta.has_keyword("adventure"));
    }

    #[test]
    fn test_metadata_custom_field_update() {
        let mut meta = Metadata::default();
        meta.set_custom_field("Version", CustomFieldValue::Number(1.0));
        meta.set_custom_field("Version", CustomFieldValue::Number(2.0));
        assert_eq!(meta.custom_metadata.len(), 1);
        assert_eq!(meta.get_custom_field("Version"), Some(&CustomFieldValue::Number(2.0)));
    }

    #[test]
    fn test_custom_field_value_equality() {
        assert_eq!(
            CustomFieldValue::Text("hello".into()),
            CustomFieldValue::Text("hello".into())
        );
        assert_ne!(
            CustomFieldValue::Text("hello".into()),
            CustomFieldValue::Number(0.0)
        );
    }

    #[test]
    fn test_label_new() {
        let label = Label::new("Test", LabelColor::Green);
        assert_eq!(label.name, "Test");
        assert_eq!(label.color.display_name(), "Green");
    }

    #[test]
    fn test_status_new() {
        let status = Status::new("Review");
        assert_eq!(status.name, "Review");
    }

    #[test]
    fn test_all_label_color_hexes() {
        for color in LabelColor::all_predefined() {
            let hex = color.to_hex();
            assert!(hex.starts_with('#'));
            assert_eq!(hex.len(), 7);
        }
    }

    #[test]
    fn test_schema_add_fields() {
        let mut schema = CustomMetadataSchema::new();
        assert_eq!(schema.fields.len(), 0);
        schema.add_text_field("Title", true);
        schema.add_checkbox_field("Reviewed");
        schema.add_enum_field("Status", vec!["Open".into(), "Closed".into()], false);
        assert_eq!(schema.fields.len(), 3);
        assert_eq!(schema.fields[0].field_type, CustomFieldType::Text);
        assert_eq!(schema.fields[1].field_type, CustomFieldType::Checkbox);
        assert_eq!(schema.fields[2].field_type, CustomFieldType::Enum);
    }

    #[test]
    fn test_schema_remove_field() {
        let mut schema = CustomMetadataSchema::new();
        schema.add_text_field("A", false);
        schema.add_text_field("B", false);
        schema.remove_field("A");
        assert_eq!(schema.fields.len(), 1);
        assert_eq!(schema.fields[0].name, "B");
    }

    #[test]
    fn test_schema_enum_default_value() {
        let mut schema = CustomMetadataSchema::new();
        schema.add_enum_field("Priority", vec!["Low".into(), "Medium".into(), "High".into()], false);
        let field = schema.get_field("Priority").unwrap();
        assert_eq!(field.default_value, "Low");
    }

    #[test]
    fn test_app_preferences_serialization_roundtrip() {
        let mut prefs = AppPreferences::default();
        prefs.add_dictionary_word("avalon");
        prefs.set_shortcut("compile", "Ctrl+Shift+C");
        let json = serde_json::to_string(&prefs).unwrap();
        let parsed: AppPreferences = serde_json::from_str(&json).unwrap();
        assert!(parsed.has_dictionary_word("avalon"));
        assert_eq!(parsed.get_shortcut("compile"), Some("Ctrl+Shift+C"));
    }

    #[test]
    fn test_project_settings_add_duplicate_label() {
        let mut settings = ProjectSettings::default();
        let initial = settings.labels.len();
        settings.add_label("Chapter", LabelColor::Red); // "Chapter" already exists
        assert_eq!(settings.labels.len(), initial); // No duplicate added
    }

    #[test]
    fn test_project_settings_remove_nonexistent() {
        let mut settings = ProjectSettings::default();
        let initial = settings.labels.len();
        settings.remove_label("Nonexistent");
        assert_eq!(settings.labels.len(), initial); // Nothing removed
    }

    #[test]
    fn test_custom_field_list_value() {
        let list = CustomFieldValue::List(vec!["tag1".into(), "tag2".into(), "tag3".into()]);
        assert_eq!(list.display(), "tag1, tag2, tag3");
        assert!(!list.is_empty());
        assert_eq!(list.type_name(), "List");
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
