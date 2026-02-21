use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// An annotation (inline comment) attached to a text range in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: Uuid,
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub color: AnnotationColor,
    pub resolved: bool,
    /// When the annotation was resolved (if applicable)
    #[serde(default)]
    pub resolved_at: Option<DateTime<Utc>>,
    /// Edit history: (timestamp, old_text)
    #[serde(default)]
    pub edit_history: Vec<(DateTime<Utc>, String)>,
    /// Category tag for grouping annotations
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnotationColor {
    Yellow,
    Blue,
    Green,
    Red,
    Purple,
}

impl Annotation {
    pub fn new(start: usize, end: usize, text: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            start,
            end,
            text: text.to_string(),
            author: String::new(),
            created_at: Utc::now(),
            color: AnnotationColor::Yellow,
            resolved: false,
            resolved_at: None,
            edit_history: Vec::new(),
            category: None,
        }
    }

    /// Create an annotation with a specific color
    pub fn with_color(start: usize, end: usize, text: &str, color: AnnotationColor) -> Self {
        let mut ann = Self::new(start, end, text);
        ann.color = color;
        ann
    }

    /// Edit the annotation text, preserving history
    pub fn edit_text(&mut self, new_text: &str) {
        let old = std::mem::replace(&mut self.text, new_text.to_string());
        self.edit_history.push((Utc::now(), old));
    }

    /// Toggle resolved status
    pub fn toggle_resolved(&mut self) {
        self.resolved = !self.resolved;
        self.resolved_at = if self.resolved { Some(Utc::now()) } else { None };
    }

    /// Get the annotated text span length
    pub fn span_length(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Check if this annotation overlaps with a text range
    pub fn overlaps(&self, start: usize, end: usize) -> bool {
        self.start < end && self.end > start
    }

    /// Get age as a human-readable string
    pub fn age_string(&self) -> String {
        let age = Utc::now().signed_duration_since(self.created_at);
        if age.num_days() > 0 {
            format!("{}d ago", age.num_days())
        } else if age.num_hours() > 0 {
            format!("{}h ago", age.num_hours())
        } else {
            "recent".to_string()
        }
    }

    /// Check if the annotation text contains a substring (case-insensitive)
    pub fn text_contains(&self, query: &str) -> bool {
        self.text.to_lowercase().contains(&query.to_lowercase())
    }

    /// Get the number of edits this annotation has undergone
    pub fn edit_count(&self) -> usize {
        self.edit_history.len()
    }

    /// Check if this annotation has been edited
    pub fn has_been_edited(&self) -> bool {
        !self.edit_history.is_empty()
    }

    /// Set the category tag
    pub fn set_category(&mut self, category: &str) {
        self.category = if category.is_empty() {
            None
        } else {
            Some(category.to_string())
        };
    }

    /// Get display summary for the annotation
    pub fn summary(&self) -> String {
        let status = if self.resolved { "resolved" } else { "open" };
        let truncated = if self.text.len() > 40 {
            format!("{}...", &self.text[..40])
        } else {
            self.text.clone()
        };
        format!("[{}] {} ({})", status, truncated, self.color.label())
    }

    /// Builder: set author
    pub fn with_author(mut self, author: &str) -> Self {
        self.author = author.to_string();
        self
    }

    /// Check if a character position is inside this annotation
    pub fn contains_position(&self, pos: usize) -> bool {
        pos >= self.start && pos < self.end
    }

    /// Shift the annotation range by an offset (for text insertions/deletions before it)
    pub fn shift(&mut self, offset: i64) {
        if offset >= 0 {
            let off = offset as usize;
            self.start += off;
            self.end += off;
        } else {
            let off = (-offset) as usize;
            self.start = self.start.saturating_sub(off);
            self.end = self.end.saturating_sub(off);
        }
    }
}

/// Available annotation categories
pub fn default_categories() -> Vec<&'static str> {
    vec!["Note", "Todo", "Question", "Research", "Continuity", "Revision"]
}

impl AnnotationColor {
    pub fn to_hex(&self) -> &str {
        match self {
            AnnotationColor::Yellow => "#f1c40f",
            AnnotationColor::Blue => "#3498db",
            AnnotationColor::Green => "#2ecc71",
            AnnotationColor::Red => "#e74c3c",
            AnnotationColor::Purple => "#9b59b6",
        }
    }

    pub fn to_iced_color(&self) -> iced::Color {
        let hex = self.to_hex().trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0;
        iced::Color::from_rgb(r, g, b)
    }

    /// Human-readable label
    pub fn label(&self) -> &str {
        match self {
            AnnotationColor::Yellow => "Yellow",
            AnnotationColor::Blue => "Blue",
            AnnotationColor::Green => "Green",
            AnnotationColor::Red => "Red",
            AnnotationColor::Purple => "Purple",
        }
    }

    /// All available colors
    pub fn all() -> Vec<Self> {
        vec![
            AnnotationColor::Yellow,
            AnnotationColor::Blue,
            AnnotationColor::Green,
            AnnotationColor::Red,
            AnnotationColor::Purple,
        ]
    }

    /// Get the next color in the cycle
    pub fn next(&self) -> Self {
        match self {
            AnnotationColor::Yellow => AnnotationColor::Blue,
            AnnotationColor::Blue => AnnotationColor::Green,
            AnnotationColor::Green => AnnotationColor::Red,
            AnnotationColor::Red => AnnotationColor::Purple,
            AnnotationColor::Purple => AnnotationColor::Yellow,
        }
    }
}
