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

    /// Set the category tag
    pub fn set_category(&mut self, category: &str) {
        self.category = if category.is_empty() {
            None
        } else {
            Some(category.to_string())
        };
    }

    /// Builder: set author
    pub fn with_author(mut self, author: &str) -> Self {
        self.author = author.to_string();
        self
    }

    /// Shift the annotation range by an offset (for text insertions/deletions before it)
    pub fn shift(&mut self, offset: i64) {
        if offset >= 0 {
            if let Ok(off) = usize::try_from(offset) {
                self.start = self.start.saturating_add(off);
                self.end = self.end.saturating_add(off);
            }
        } else if let Ok(off) = usize::try_from(offset.saturating_abs()) {
            self.start = self.start.saturating_sub(off);
            self.end = self.end.saturating_sub(off);
        }
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
        if hex.len() < 6 {
            return iced::Color::from_rgb(0.5, 0.5, 0.5);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotation_new() {
        let ann = Annotation::new(10, 20, "Test comment");
        assert_eq!(ann.start, 10);
        assert_eq!(ann.end, 20);
        assert_eq!(ann.text, "Test comment");
        assert!(!ann.resolved);
    }

    #[test]
    fn test_annotation_with_color() {
        let ann = Annotation::with_color(0, 5, "Note", AnnotationColor::Red);
        assert!(matches!(ann.color, AnnotationColor::Red));
    }

    #[test]
    fn test_toggle_resolved() {
        let mut ann = Annotation::new(0, 10, "Test");
        assert!(!ann.resolved);
        assert!(ann.resolved_at.is_none());

        ann.toggle_resolved();
        assert!(ann.resolved);
        assert!(ann.resolved_at.is_some());

        ann.toggle_resolved();
        assert!(!ann.resolved);
        assert!(ann.resolved_at.is_none());
    }

    #[test]
    fn test_edit_text() {
        let mut ann = Annotation::new(0, 10, "Original");
        assert_eq!(ann.edit_history.len(), 0);
        assert!(ann.edit_history.is_empty());

        ann.edit_text("Updated");
        assert_eq!(ann.text, "Updated");
        assert_eq!(ann.edit_history.len(), 1);
        assert_eq!(ann.edit_history[0].1, "Original");
    }

    #[test]
    fn test_shift() {
        let mut ann = Annotation::new(10, 20, "Test");
        ann.shift(5);
        assert_eq!(ann.start, 15);
        assert_eq!(ann.end, 25);

        ann.shift(-3);
        assert_eq!(ann.start, 12);
        assert_eq!(ann.end, 22);
    }

    #[test]
    fn test_category() {
        let mut ann = Annotation::new(0, 10, "Test");
        assert!(ann.category.is_none());

        ann.set_category("Todo");
        assert_eq!(ann.category, Some("Todo".to_string()));

        ann.set_category("");
        assert!(ann.category.is_none());
    }

    #[test]
    fn test_with_author() {
        let ann = Annotation::new(0, 10, "Test").with_author("John");
        assert_eq!(ann.author, "John");
    }

    #[test]
    fn test_annotation_color_cycle() {
        let color = AnnotationColor::Yellow;
        let next = color.next();
        assert!(matches!(next, AnnotationColor::Blue));
    }

    #[test]
    fn test_annotation_color_hex() {
        let color = AnnotationColor::Red;
        assert_eq!(color.to_hex(), "#e74c3c");
        assert_eq!(color.label(), "Red");
    }

    #[test]
    fn test_annotation_color_all_labels() {
        assert_eq!(AnnotationColor::Yellow.label(), "Yellow");
        assert_eq!(AnnotationColor::Blue.label(), "Blue");
        assert_eq!(AnnotationColor::Green.label(), "Green");
        assert_eq!(AnnotationColor::Red.label(), "Red");
        assert_eq!(AnnotationColor::Purple.label(), "Purple");
    }

    #[test]
    fn test_annotation_color_cycle_full() {
        let start = AnnotationColor::Yellow;
        let b = start.next();
        let g = b.next();
        let r = g.next();
        let p = r.next();
        let y = p.next();
        assert!(matches!(y, AnnotationColor::Yellow)); // Full cycle
    }

    #[test]
    fn test_annotation_color_to_iced() {
        let color = AnnotationColor::Red;
        let iced_color = color.to_iced_color();
        assert!(iced_color.r >= 0.0 && iced_color.r <= 1.0);
        assert!(iced_color.g >= 0.0 && iced_color.g <= 1.0);
        assert!(iced_color.b >= 0.0 && iced_color.b <= 1.0);
    }

    #[test]
    fn test_shift_negative_saturating() {
        let mut ann = Annotation::new(5, 15, "Test");
        ann.shift(-10); // More than start
        assert_eq!(ann.start, 0); // Saturates at 0
        assert_eq!(ann.end, 5);
    }

    #[test]
    fn test_multiple_edits() {
        let mut ann = Annotation::new(0, 10, "Version 1");
        ann.edit_text("Version 2");
        ann.edit_text("Version 3");
        assert_eq!(ann.edit_history.len(), 2);
        assert_eq!(ann.text, "Version 3");
        assert_eq!(ann.edit_history[0].1, "Version 1");
        assert_eq!(ann.edit_history[1].1, "Version 2");
    }
}
