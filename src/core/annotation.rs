#![allow(dead_code)]
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
        let truncated = match self.text.char_indices().nth(40) {
            Some((byte_idx, _)) => format!("{}...", &self.text[..byte_idx]),
            None => self.text.clone(),
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

    /// Builder: set category
    pub fn with_category(mut self, category: &str) -> Self {
        self.category = if category.is_empty() { None } else { Some(category.to_string()) };
        self
    }

    /// Check if this annotation is in a given category
    pub fn is_in_category(&self, category: &str) -> bool {
        self.category.as_deref() == Some(category)
    }

    /// Check if this annotation is older than the given number of days
    pub fn is_older_than_days(&self, days: i64) -> bool {
        let age = Utc::now().signed_duration_since(self.created_at);
        age.num_days() > days
    }

    /// Get a compact display label
    pub fn label(&self) -> String {
        let status = if self.resolved { "resolved" } else { "open" };
        let truncated = match self.text.char_indices().nth(30) {
            Some((byte_idx, _)) => format!("{}...", &self.text[..byte_idx]),
            None => self.text.clone(),
        };
        format!("[{}|{}] {}", status, self.color.label(), truncated)
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

/// Manages a collection of annotations for a document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnnotationSet {
    pub annotations: Vec<Annotation>,
}

impl AnnotationSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an annotation
    pub fn add(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
    }

    /// Remove an annotation by ID
    pub fn remove(&mut self, id: &Uuid) {
        self.annotations.retain(|a| &a.id != id);
    }

    /// Get an annotation by ID
    pub fn get(&self, id: &Uuid) -> Option<&Annotation> {
        self.annotations.iter().find(|a| &a.id == id)
    }

    /// Get a mutable annotation by ID
    pub fn get_mut(&mut self, id: &Uuid) -> Option<&mut Annotation> {
        self.annotations.iter_mut().find(|a| &a.id == id)
    }

    /// Get all open (unresolved) annotations
    pub fn open(&self) -> Vec<&Annotation> {
        self.annotations.iter().filter(|a| !a.resolved).collect()
    }

    /// Get all resolved annotations
    pub fn resolved(&self) -> Vec<&Annotation> {
        self.annotations.iter().filter(|a| a.resolved).collect()
    }

    /// Get annotations by category
    pub fn by_category(&self, category: &str) -> Vec<&Annotation> {
        self.annotations.iter()
            .filter(|a| a.category.as_deref() == Some(category))
            .collect()
    }

    /// Get annotations by color
    pub fn by_color(&self, color: &AnnotationColor) -> Vec<&Annotation> {
        self.annotations.iter()
            .filter(|a| std::mem::discriminant(&a.color) == std::mem::discriminant(color))
            .collect()
    }

    /// Get annotations overlapping a given range
    pub fn overlapping(&self, start: usize, end: usize) -> Vec<&Annotation> {
        self.annotations.iter().filter(|a| a.overlaps(start, end)).collect()
    }

    /// Get annotations at a specific position
    pub fn at_position(&self, pos: usize) -> Vec<&Annotation> {
        self.annotations.iter().filter(|a| a.contains_position(pos)).collect()
    }

    /// Search annotations by text (case-insensitive)
    pub fn search(&self, query: &str) -> Vec<&Annotation> {
        self.annotations.iter().filter(|a| a.text_contains(query)).collect()
    }

    /// Resolve all annotations
    pub fn resolve_all(&mut self) {
        for ann in &mut self.annotations {
            if !ann.resolved {
                ann.toggle_resolved();
            }
        }
    }

    /// Count of annotations
    pub fn count(&self) -> usize {
        self.annotations.len()
    }

    /// Count of open annotations
    pub fn open_count(&self) -> usize {
        self.annotations.iter().filter(|a| !a.resolved).count()
    }

    /// Count of resolved annotations
    pub fn resolved_count(&self) -> usize {
        self.annotations.iter().filter(|a| a.resolved).count()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.annotations.is_empty()
    }

    /// Get all unique categories used
    pub fn used_categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self.annotations.iter()
            .filter_map(|a| a.category.clone())
            .collect();
        cats.sort();
        cats.dedup();
        cats
    }

    /// Sorted by position (start offset)
    pub fn sorted_by_position(&self) -> Vec<&Annotation> {
        let mut sorted: Vec<&Annotation> = self.annotations.iter().collect();
        sorted.sort_by_key(|a| a.start);
        sorted
    }

    /// Sorted by creation date (newest first)
    pub fn sorted_by_date(&self) -> Vec<&Annotation> {
        let mut sorted: Vec<&Annotation> = self.annotations.iter().collect();
        sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        sorted
    }

    /// Shift all annotations after a position by an offset
    pub fn shift_after(&mut self, position: usize, offset: i64) {
        for ann in &mut self.annotations {
            if ann.start >= position {
                ann.shift(offset);
            }
        }
    }

    /// Summary of annotations
    pub fn summary(&self) -> String {
        format!(
            "{} total ({} open, {} resolved)",
            self.count(), self.open_count(), self.resolved_count()
        )
    }

    /// Get annotations by author
    pub fn by_author(&self, author: &str) -> Vec<&Annotation> {
        let lower = author.to_lowercase();
        self.annotations.iter()
            .filter(|a| a.author.to_lowercase() == lower)
            .collect()
    }

    /// Get all unique authors
    pub fn unique_authors(&self) -> Vec<String> {
        let mut authors: Vec<String> = self.annotations.iter()
            .filter(|a| !a.author.is_empty())
            .map(|a| a.author.clone())
            .collect();
        authors.sort();
        authors.dedup();
        authors
    }

    /// Get the most-edited annotation
    pub fn most_edited(&self) -> Option<&Annotation> {
        self.annotations.iter().max_by_key(|a| a.edit_count())
    }

    /// Get annotations older than a given number of days
    pub fn older_than_days(&self, days: i64) -> Vec<&Annotation> {
        self.annotations.iter()
            .filter(|a| a.is_older_than_days(days))
            .collect()
    }

    /// Get stale open annotations (open and older than N days)
    pub fn stale_open(&self, days: i64) -> Vec<&Annotation> {
        self.annotations.iter()
            .filter(|a| !a.resolved && a.is_older_than_days(days))
            .collect()
    }

    /// Export all annotation texts as a list of strings
    pub fn export_texts(&self) -> Vec<String> {
        self.annotations.iter().map(|a| a.text.clone()).collect()
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
        assert_eq!(ann.span_length(), 10);
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
        assert_eq!(ann.edit_count(), 0);
        assert!(!ann.has_been_edited());

        ann.edit_text("Updated");
        assert_eq!(ann.text, "Updated");
        assert_eq!(ann.edit_count(), 1);
        assert!(ann.has_been_edited());
        assert_eq!(ann.edit_history[0].1, "Original");
    }

    #[test]
    fn test_overlaps() {
        let ann = Annotation::new(10, 20, "Test");
        assert!(ann.overlaps(15, 25));
        assert!(ann.overlaps(5, 15));
        assert!(ann.overlaps(12, 18));
        assert!(!ann.overlaps(20, 30));
        assert!(!ann.overlaps(0, 10));
    }

    #[test]
    fn test_contains_position() {
        let ann = Annotation::new(10, 20, "Test");
        assert!(ann.contains_position(10));
        assert!(ann.contains_position(15));
        assert!(!ann.contains_position(20));
        assert!(!ann.contains_position(9));
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
    fn test_text_contains() {
        let ann = Annotation::new(0, 10, "Fix the typo here");
        assert!(ann.text_contains("typo"));
        assert!(ann.text_contains("TYPO"));
        assert!(!ann.text_contains("bug"));
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
    fn test_summary() {
        let ann = Annotation::new(0, 10, "Short note");
        let summary = ann.summary();
        assert!(summary.contains("open"));
        assert!(summary.contains("Short note"));
    }

    #[test]
    fn test_default_categories() {
        let cats = default_categories();
        assert!(cats.contains(&"Note"));
        assert!(cats.contains(&"Todo"));
        assert!(cats.contains(&"Question"));
    }

    #[test]
    fn test_annotation_color_cycle() {
        let color = AnnotationColor::Yellow;
        let next = color.next();
        assert!(matches!(next, AnnotationColor::Blue));
        let all = AnnotationColor::all();
        assert_eq!(all.len(), 5);
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
    fn test_annotation_color_all_hex() {
        // All hex values should start with '#' and have 7 chars
        for color in AnnotationColor::all() {
            let hex = color.to_hex();
            assert!(hex.starts_with('#'));
            assert_eq!(hex.len(), 7);
        }
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
        for color in AnnotationColor::all() {
            let iced_color = color.to_iced_color();
            assert!(iced_color.r >= 0.0 && iced_color.r <= 1.0);
            assert!(iced_color.g >= 0.0 && iced_color.g <= 1.0);
            assert!(iced_color.b >= 0.0 && iced_color.b <= 1.0);
        }
    }

    #[test]
    fn test_shift_negative_saturating() {
        let mut ann = Annotation::new(5, 15, "Test");
        ann.shift(-10); // More than start
        assert_eq!(ann.start, 0); // Saturates at 0
        assert_eq!(ann.end, 5);
    }

    #[test]
    fn test_age_string() {
        let ann = Annotation::new(0, 10, "Recent");
        let age = ann.age_string();
        assert_eq!(age, "recent"); // Just created
    }

    #[test]
    fn test_summary_long_text() {
        let long_text = "A".repeat(100);
        let ann = Annotation::new(0, 10, &long_text);
        let summary = ann.summary();
        assert!(summary.contains("..."));
        assert!(summary.len() < 100);
    }

    #[test]
    fn test_summary_resolved() {
        let mut ann = Annotation::new(0, 10, "Fixed");
        ann.toggle_resolved();
        let summary = ann.summary();
        assert!(summary.contains("resolved"));
    }

    #[test]
    fn test_multiple_edits() {
        let mut ann = Annotation::new(0, 10, "Version 1");
        ann.edit_text("Version 2");
        ann.edit_text("Version 3");
        assert_eq!(ann.edit_count(), 2);
        assert_eq!(ann.text, "Version 3");
        assert_eq!(ann.edit_history[0].1, "Version 1");
        assert_eq!(ann.edit_history[1].1, "Version 2");
    }

    #[test]
    fn test_span_length_zero() {
        let ann = Annotation::new(5, 5, "Zero span");
        assert_eq!(ann.span_length(), 0);
    }

    #[test]
    fn test_overlaps_same_range() {
        let ann = Annotation::new(10, 20, "Test");
        assert!(ann.overlaps(10, 20)); // Exact same range
    }

    #[test]
    fn test_overlaps_contains() {
        let ann = Annotation::new(10, 20, "Test");
        assert!(ann.overlaps(0, 100)); // Range contains annotation
    }

    #[test]
    fn test_default_categories_count() {
        let cats = default_categories();
        assert_eq!(cats.len(), 6);
        assert!(cats.contains(&"Continuity"));
        assert!(cats.contains(&"Revision"));
        assert!(cats.contains(&"Research"));
    }

    // AnnotationSet tests

    #[test]
    fn test_annotation_set_new() {
        let set = AnnotationSet::new();
        assert!(set.is_empty());
        assert_eq!(set.count(), 0);
    }

    #[test]
    fn test_annotation_set_add_remove() {
        let mut set = AnnotationSet::new();
        let ann = Annotation::new(0, 10, "Note 1");
        let id = ann.id;
        set.add(ann);
        assert_eq!(set.count(), 1);
        assert!(!set.is_empty());

        set.remove(&id);
        assert_eq!(set.count(), 0);
        assert!(set.is_empty());
    }

    #[test]
    fn test_annotation_set_get() {
        let mut set = AnnotationSet::new();
        let ann = Annotation::new(5, 15, "Test");
        let id = ann.id;
        set.add(ann);

        let found = set.get(&id).unwrap();
        assert_eq!(found.text, "Test");
    }

    #[test]
    fn test_annotation_set_get_mut() {
        let mut set = AnnotationSet::new();
        let ann = Annotation::new(5, 15, "Original");
        let id = ann.id;
        set.add(ann);

        let found = set.get_mut(&id).unwrap();
        found.edit_text("Modified");
        assert_eq!(set.get(&id).unwrap().text, "Modified");
    }

    #[test]
    fn test_annotation_set_open_resolved() {
        let mut set = AnnotationSet::new();
        let mut ann1 = Annotation::new(0, 10, "Open");
        let mut ann2 = Annotation::new(10, 20, "Resolved");
        ann2.toggle_resolved();
        set.add(ann1);
        set.add(ann2);

        assert_eq!(set.open_count(), 1);
        assert_eq!(set.resolved_count(), 1);
        assert_eq!(set.open().len(), 1);
        assert_eq!(set.resolved().len(), 1);
    }

    #[test]
    fn test_annotation_set_by_category() {
        let mut set = AnnotationSet::new();
        let mut ann1 = Annotation::new(0, 10, "Note");
        ann1.set_category("Todo");
        let mut ann2 = Annotation::new(10, 20, "Question");
        ann2.set_category("Research");
        let ann3 = Annotation::new(20, 30, "No category");
        set.add(ann1);
        set.add(ann2);
        set.add(ann3);

        assert_eq!(set.by_category("Todo").len(), 1);
        assert_eq!(set.by_category("Research").len(), 1);
        assert_eq!(set.by_category("Missing").len(), 0);
    }

    #[test]
    fn test_annotation_set_by_color() {
        let mut set = AnnotationSet::new();
        set.add(Annotation::with_color(0, 5, "A", AnnotationColor::Red));
        set.add(Annotation::with_color(5, 10, "B", AnnotationColor::Red));
        set.add(Annotation::with_color(10, 15, "C", AnnotationColor::Blue));

        assert_eq!(set.by_color(&AnnotationColor::Red).len(), 2);
        assert_eq!(set.by_color(&AnnotationColor::Blue).len(), 1);
        assert_eq!(set.by_color(&AnnotationColor::Green).len(), 0);
    }

    #[test]
    fn test_annotation_set_overlapping() {
        let mut set = AnnotationSet::new();
        set.add(Annotation::new(0, 10, "First"));
        set.add(Annotation::new(5, 15, "Second"));
        set.add(Annotation::new(20, 30, "Third"));

        let overlapping = set.overlapping(8, 12);
        assert_eq!(overlapping.len(), 2); // First and Second
    }

    #[test]
    fn test_annotation_set_at_position() {
        let mut set = AnnotationSet::new();
        set.add(Annotation::new(0, 10, "First"));
        set.add(Annotation::new(5, 15, "Second"));
        set.add(Annotation::new(20, 30, "Third"));

        let at_7 = set.at_position(7);
        assert_eq!(at_7.len(), 2); // First and Second
        let at_25 = set.at_position(25);
        assert_eq!(at_25.len(), 1); // Third
    }

    #[test]
    fn test_annotation_set_search() {
        let mut set = AnnotationSet::new();
        set.add(Annotation::new(0, 10, "Fix the bug"));
        set.add(Annotation::new(10, 20, "Add feature"));
        set.add(Annotation::new(20, 30, "Another bug fix"));

        let results = set.search("bug");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_annotation_set_resolve_all() {
        let mut set = AnnotationSet::new();
        set.add(Annotation::new(0, 10, "A"));
        set.add(Annotation::new(10, 20, "B"));
        assert_eq!(set.open_count(), 2);

        set.resolve_all();
        assert_eq!(set.open_count(), 0);
        assert_eq!(set.resolved_count(), 2);
    }

    #[test]
    fn test_annotation_set_used_categories() {
        let mut set = AnnotationSet::new();
        let mut a1 = Annotation::new(0, 5, "A");
        a1.set_category("Todo");
        let mut a2 = Annotation::new(5, 10, "B");
        a2.set_category("Research");
        let mut a3 = Annotation::new(10, 15, "C");
        a3.set_category("Todo");
        set.add(a1);
        set.add(a2);
        set.add(a3);

        let cats = set.used_categories();
        assert_eq!(cats.len(), 2); // "Research" and "Todo" (deduped, sorted)
        assert_eq!(cats[0], "Research");
        assert_eq!(cats[1], "Todo");
    }

    #[test]
    fn test_annotation_set_sorted_by_position() {
        let mut set = AnnotationSet::new();
        set.add(Annotation::new(20, 30, "Third"));
        set.add(Annotation::new(0, 10, "First"));
        set.add(Annotation::new(10, 20, "Second"));

        let sorted = set.sorted_by_position();
        assert_eq!(sorted[0].text, "First");
        assert_eq!(sorted[1].text, "Second");
        assert_eq!(sorted[2].text, "Third");
    }

    #[test]
    fn test_annotation_set_sorted_by_date() {
        let mut set = AnnotationSet::new();
        set.add(Annotation::new(0, 10, "First"));
        set.add(Annotation::new(10, 20, "Second"));

        let sorted = set.sorted_by_date();
        // Newest first - both created very close, so just check we get both
        assert_eq!(sorted.len(), 2);
    }

    #[test]
    fn test_annotation_set_shift_after() {
        let mut set = AnnotationSet::new();
        set.add(Annotation::new(0, 10, "Before"));
        set.add(Annotation::new(20, 30, "After"));

        set.shift_after(15, 5);
        let sorted = set.sorted_by_position();
        assert_eq!(sorted[0].start, 0); // Not shifted (before position 15)
        assert_eq!(sorted[1].start, 25); // Shifted by 5
        assert_eq!(sorted[1].end, 35);
    }

    #[test]
    fn test_annotation_set_summary() {
        let mut set = AnnotationSet::new();
        set.add(Annotation::new(0, 10, "A"));
        let mut b = Annotation::new(10, 20, "B");
        b.toggle_resolved();
        set.add(b);

        let summary = set.summary();
        assert!(summary.contains("2 total"));
        assert!(summary.contains("1 open"));
        assert!(summary.contains("1 resolved"));
    }

    // ---- New annotation tests ----

    #[test]
    fn test_with_category_builder() {
        let ann = Annotation::new(0, 10, "Test").with_category("Todo");
        assert_eq!(ann.category, Some("Todo".to_string()));
    }

    #[test]
    fn test_with_category_empty() {
        let ann = Annotation::new(0, 10, "Test").with_category("");
        assert!(ann.category.is_none());
    }

    #[test]
    fn test_is_in_category() {
        let ann = Annotation::new(0, 10, "Test").with_category("Research");
        assert!(ann.is_in_category("Research"));
        assert!(!ann.is_in_category("Todo"));
    }

    #[test]
    fn test_is_older_than_days() {
        let ann = Annotation::new(0, 10, "Test");
        // Just created, not older than 1 day
        assert!(!ann.is_older_than_days(1));
    }

    #[test]
    fn test_label() {
        let ann = Annotation::new(0, 10, "Fix typo");
        let label = ann.label();
        assert!(label.contains("open"));
        assert!(label.contains("Yellow"));
        assert!(label.contains("Fix typo"));
    }

    #[test]
    fn test_label_resolved() {
        let mut ann = Annotation::new(0, 10, "Done");
        ann.toggle_resolved();
        let label = ann.label();
        assert!(label.contains("resolved"));
    }

    #[test]
    fn test_label_truncated() {
        let long_text = "A".repeat(50);
        let ann = Annotation::new(0, 10, &long_text);
        let label = ann.label();
        assert!(label.contains("..."));
    }

    #[test]
    fn test_by_author() {
        let mut set = AnnotationSet::new();
        set.add(Annotation::new(0, 5, "A").with_author("Alice"));
        set.add(Annotation::new(5, 10, "B").with_author("Bob"));
        set.add(Annotation::new(10, 15, "C").with_author("Alice"));

        assert_eq!(set.by_author("Alice").len(), 2);
        assert_eq!(set.by_author("Bob").len(), 1);
        assert_eq!(set.by_author("Charlie").len(), 0);
    }

    #[test]
    fn test_by_author_case_insensitive() {
        let mut set = AnnotationSet::new();
        set.add(Annotation::new(0, 5, "A").with_author("Alice"));
        assert_eq!(set.by_author("alice").len(), 1);
        assert_eq!(set.by_author("ALICE").len(), 1);
    }

    #[test]
    fn test_unique_authors() {
        let mut set = AnnotationSet::new();
        set.add(Annotation::new(0, 5, "A").with_author("Alice"));
        set.add(Annotation::new(5, 10, "B").with_author("Bob"));
        set.add(Annotation::new(10, 15, "C").with_author("Alice"));
        set.add(Annotation::new(15, 20, "D")); // No author

        let authors = set.unique_authors();
        assert_eq!(authors.len(), 2);
        assert!(authors.contains(&"Alice".to_string()));
        assert!(authors.contains(&"Bob".to_string()));
    }

    #[test]
    fn test_most_edited() {
        let mut set = AnnotationSet::new();
        let mut a1 = Annotation::new(0, 5, "A");
        a1.edit_text("A2");
        let mut a2 = Annotation::new(5, 10, "B");
        a2.edit_text("B2");
        a2.edit_text("B3");
        a2.edit_text("B4");
        set.add(a1);
        set.add(a2);

        let most = set.most_edited().unwrap();
        assert_eq!(most.text, "B4");
        assert_eq!(most.edit_count(), 3);
    }

    #[test]
    fn test_most_edited_empty() {
        let set = AnnotationSet::new();
        assert!(set.most_edited().is_none());
    }

    #[test]
    fn test_stale_open() {
        let mut set = AnnotationSet::new();
        set.add(Annotation::new(0, 5, "A"));
        set.add(Annotation::new(5, 10, "B"));
        // Just created, not stale
        assert_eq!(set.stale_open(7).len(), 0);
    }

    #[test]
    fn test_export_texts() {
        let mut set = AnnotationSet::new();
        set.add(Annotation::new(0, 5, "First note"));
        set.add(Annotation::new(5, 10, "Second note"));
        let texts = set.export_texts();
        assert_eq!(texts.len(), 2);
        assert_eq!(texts[0], "First note");
        assert_eq!(texts[1], "Second note");
    }

    #[test]
    fn test_export_texts_empty() {
        let set = AnnotationSet::new();
        assert!(set.export_texts().is_empty());
    }
}
