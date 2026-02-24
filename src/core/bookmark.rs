#![allow(dead_code)]
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// A bookmark marks a binder item as a favorite for quick access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub item_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    /// Optional color/category for grouping bookmarks
    #[serde(default)]
    pub color: Option<String>,
    /// Optional user note about why this was bookmarked
    #[serde(default)]
    pub note: Option<String>,
}

impl Bookmark {
    pub fn new(item_id: Uuid, name: &str) -> Self {
        Self {
            item_id,
            name: name.to_string(),
            created_at: Utc::now(),
            color: None,
            note: None,
        }
    }

    /// Create a bookmark with a note
    pub fn with_note(item_id: Uuid, name: &str, note: &str) -> Self {
        Self {
            item_id,
            name: name.to_string(),
            created_at: Utc::now(),
            color: None,
            note: if note.is_empty() { None } else { Some(note.to_string()) },
        }
    }

    /// Get the age of this bookmark as a human-readable string
    pub fn age_string(&self) -> String {
        let age = Utc::now().signed_duration_since(self.created_at);
        if age.num_days() > 30 {
            format!("{}mo ago", age.num_days() / 30)
        } else if age.num_days() > 0 {
            format!("{}d ago", age.num_days())
        } else if age.num_hours() > 0 {
            format!("{}h ago", age.num_hours())
        } else {
            "recent".to_string()
        }
    }

    /// Display label combining name, color, and note info
    pub fn display_label(&self) -> String {
        let color_tag = self.color.as_deref().unwrap_or("none");
        let note_flag = if self.note.is_some() { " [note]" } else { "" };
        format!("{} ({}){}", self.name, color_tag, note_flag)
    }

    /// Check if the bookmark has a note
    pub fn has_note(&self) -> bool {
        self.note.is_some()
    }

    /// Check if the bookmark has a color
    pub fn has_color(&self) -> bool {
        self.color.is_some()
    }
}

/// Manages the list of bookmarks for a project
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BookmarkList {
    pub bookmarks: Vec<Bookmark>,
}

impl BookmarkList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, item_id: Uuid, name: &str) {
        if !self.bookmarks.iter().any(|b| b.item_id == item_id) {
            self.bookmarks.push(Bookmark::new(item_id, name));
        }
    }

    pub fn remove(&mut self, item_id: &Uuid) {
        self.bookmarks.retain(|b| &b.item_id != item_id);
    }

    pub fn is_bookmarked(&self, item_id: &Uuid) -> bool {
        self.bookmarks.iter().any(|b| &b.item_id == item_id)
    }

    pub fn toggle(&mut self, item_id: Uuid, name: &str) {
        if self.is_bookmarked(&item_id) {
            self.remove(&item_id);
        } else {
            self.add(item_id, name);
        }
    }

    /// Get a bookmark by item_id
    pub fn get(&self, item_id: &Uuid) -> Option<&Bookmark> {
        self.bookmarks.iter().find(|b| &b.item_id == item_id)
    }

    /// Get bookmarks sorted by creation date (newest first)
    pub fn sorted_by_date(&self) -> Vec<&Bookmark> {
        let mut sorted: Vec<&Bookmark> = self.bookmarks.iter().collect();
        sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        sorted
    }

    /// Get bookmarks sorted by name alphabetically
    pub fn sorted_by_name(&self) -> Vec<&Bookmark> {
        let mut sorted: Vec<&Bookmark> = self.bookmarks.iter().collect();
        sorted.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        sorted
    }

    /// Update the name of a bookmark
    pub fn rename(&mut self, item_id: &Uuid, new_name: &str) {
        if let Some(bm) = self.bookmarks.iter_mut().find(|b| &b.item_id == item_id) {
            bm.name = new_name.to_string();
        }
    }

    /// Total number of bookmarks
    pub fn count(&self) -> usize {
        self.bookmarks.len()
    }

    /// Check if the bookmark list is empty
    pub fn is_empty(&self) -> bool {
        self.bookmarks.is_empty()
    }

    /// Clear all bookmarks
    pub fn clear(&mut self) {
        self.bookmarks.clear();
    }

    /// Search bookmarks by name (case-insensitive)
    pub fn search(&self, query: &str) -> Vec<&Bookmark> {
        let q = query.to_lowercase();
        self.bookmarks.iter().filter(|b| b.name.to_lowercase().contains(&q)).collect()
    }

    /// Set a note on a bookmark
    pub fn set_note(&mut self, item_id: &Uuid, note: &str) {
        if let Some(bm) = self.bookmarks.iter_mut().find(|b| &b.item_id == item_id) {
            bm.note = if note.is_empty() { None } else { Some(note.to_string()) };
        }
    }

    /// Set a color on a bookmark
    pub fn set_color(&mut self, item_id: &Uuid, color: &str) {
        if let Some(bm) = self.bookmarks.iter_mut().find(|b| &b.item_id == item_id) {
            bm.color = if color.is_empty() { None } else { Some(color.to_string()) };
        }
    }

    /// Get all unique colors used by bookmarks
    pub fn used_colors(&self) -> Vec<String> {
        let mut colors: Vec<String> = self.bookmarks.iter()
            .filter_map(|b| b.color.clone())
            .collect();
        colors.sort();
        colors.dedup();
        colors
    }

    /// Get bookmarks filtered by color
    pub fn by_color(&self, color: &str) -> Vec<&Bookmark> {
        self.bookmarks.iter()
            .filter(|b| b.color.as_deref() == Some(color))
            .collect()
    }

    /// Get bookmarks that have notes
    pub fn with_notes(&self) -> Vec<&Bookmark> {
        self.bookmarks.iter()
            .filter(|b| b.note.is_some())
            .collect()
    }

    /// Move a bookmark to a new position in the list
    pub fn reorder(&mut self, from: usize, to: usize) {
        if from < self.bookmarks.len() && to < self.bookmarks.len() {
            let item = self.bookmarks.remove(from);
            self.bookmarks.insert(to, item);
        }
    }

    /// Get mutable access to a bookmark by item_id
    pub fn get_mut(&mut self, item_id: &Uuid) -> Option<&mut Bookmark> {
        self.bookmarks.iter_mut().find(|b| &b.item_id == item_id)
    }

    /// Get all item IDs
    pub fn ids(&self) -> Vec<Uuid> {
        self.bookmarks.iter().map(|b| b.item_id).collect()
    }

    /// Export bookmark names as a list of strings
    pub fn export_names(&self) -> Vec<String> {
        self.bookmarks.iter().map(|b| b.name.clone()).collect()
    }

    /// Get bookmarks grouped by color (None group = no color)
    pub fn grouped_by_color(&self) -> HashMap<Option<String>, Vec<&Bookmark>> {
        let mut groups: HashMap<Option<String>, Vec<&Bookmark>> = HashMap::new();
        for bm in &self.bookmarks {
            groups.entry(bm.color.clone()).or_default().push(bm);
        }
        groups
    }

    /// Search bookmarks by note content (case-insensitive)
    pub fn search_notes(&self, query: &str) -> Vec<&Bookmark> {
        let q = query.to_lowercase();
        self.bookmarks.iter()
            .filter(|b| b.note.as_deref().unwrap_or("").to_lowercase().contains(&q))
            .collect()
    }

    /// Summary of bookmarks
    pub fn summary(&self) -> String {
        let with_notes = self.with_notes().len();
        let colors = self.used_colors().len();
        format!("{} bookmarks ({} with notes, {} colors)", self.count(), with_notes, colors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_id() -> Uuid {
        Uuid::new_v4()
    }

    #[test]
    fn test_bookmark_new() {
        let id = make_id();
        let bm = Bookmark::new(id, "Test");
        assert_eq!(bm.item_id, id);
        assert_eq!(bm.name, "Test");
        assert!(bm.color.is_none());
        assert!(bm.note.is_none());
    }

    #[test]
    fn test_bookmark_with_note() {
        let id = make_id();
        let bm = Bookmark::with_note(id, "Test", "Important chapter");
        assert_eq!(bm.note, Some("Important chapter".to_string()));
    }

    #[test]
    fn test_bookmark_list_add_remove() {
        let mut list = BookmarkList::new();
        assert!(list.is_empty());
        assert_eq!(list.count(), 0);

        let id1 = make_id();
        let id2 = make_id();
        list.add(id1, "First");
        list.add(id2, "Second");
        assert_eq!(list.count(), 2);
        assert!(list.is_bookmarked(&id1));

        list.remove(&id1);
        assert_eq!(list.count(), 1);
        assert!(!list.is_bookmarked(&id1));
    }

    #[test]
    fn test_no_duplicates() {
        let mut list = BookmarkList::new();
        let id = make_id();
        list.add(id, "Test");
        list.add(id, "Test Again");
        assert_eq!(list.count(), 1);
    }

    #[test]
    fn test_toggle() {
        let mut list = BookmarkList::new();
        let id = make_id();
        list.toggle(id, "Test");
        assert!(list.is_bookmarked(&id));
        list.toggle(id, "Test");
        assert!(!list.is_bookmarked(&id));
    }

    #[test]
    fn test_rename() {
        let mut list = BookmarkList::new();
        let id = make_id();
        list.add(id, "Original");
        list.rename(&id, "Renamed");
        assert_eq!(list.get(&id).unwrap().name, "Renamed");
    }

    #[test]
    fn test_search() {
        let mut list = BookmarkList::new();
        list.add(make_id(), "Chapter One");
        list.add(make_id(), "Chapter Two");
        list.add(make_id(), "Epilogue");

        let results = list.search("chapter");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_set_note_and_color() {
        let mut list = BookmarkList::new();
        let id = make_id();
        list.add(id, "Test");
        list.set_note(&id, "A note");
        list.set_color(&id, "red");
        assert_eq!(list.get(&id).unwrap().note, Some("A note".to_string()));
        assert_eq!(list.get(&id).unwrap().color, Some("red".to_string()));
    }

    #[test]
    fn test_clear() {
        let mut list = BookmarkList::new();
        list.add(make_id(), "A");
        list.add(make_id(), "B");
        list.clear();
        assert!(list.is_empty());
    }

    #[test]
    fn test_sorted_by_name() {
        let mut list = BookmarkList::new();
        list.add(make_id(), "Zebra");
        list.add(make_id(), "Alpha");
        list.add(make_id(), "Middle");
        let sorted = list.sorted_by_name();
        assert_eq!(sorted[0].name, "Alpha");
        assert_eq!(sorted[1].name, "Middle");
        assert_eq!(sorted[2].name, "Zebra");
    }

    #[test]
    fn test_reorder() {
        let mut list = BookmarkList::new();
        let id1 = make_id();
        let id2 = make_id();
        let id3 = make_id();
        list.add(id1, "A");
        list.add(id2, "B");
        list.add(id3, "C");
        list.reorder(0, 2);
        assert_eq!(list.bookmarks[0].name, "B");
        assert_eq!(list.bookmarks[2].name, "A");
    }

    #[test]
    fn test_by_color() {
        let mut list = BookmarkList::new();
        let id1 = make_id();
        let id2 = make_id();
        let id3 = make_id();
        list.add(id1, "A");
        list.add(id2, "B");
        list.add(id3, "C");
        list.set_color(&id1, "red");
        list.set_color(&id2, "red");

        let reds = list.by_color("red");
        assert_eq!(reds.len(), 2);
        let colors = list.used_colors();
        assert_eq!(colors.len(), 1);
        assert_eq!(colors[0], "red");
    }

    #[test]
    fn test_bookmark_with_empty_note() {
        let id = make_id();
        let bm = Bookmark::with_note(id, "Test", "");
        assert!(bm.note.is_none());
    }

    #[test]
    fn test_bookmark_age_string() {
        let bm = Bookmark::new(make_id(), "Test");
        assert_eq!(bm.age_string(), "recent");
    }

    #[test]
    fn test_set_note_empty_clears() {
        let mut list = BookmarkList::new();
        let id = make_id();
        list.add(id, "Test");
        list.set_note(&id, "A note");
        assert!(list.get(&id).unwrap().note.is_some());
        list.set_note(&id, "");
        assert!(list.get(&id).unwrap().note.is_none());
    }

    #[test]
    fn test_set_color_empty_clears() {
        let mut list = BookmarkList::new();
        let id = make_id();
        list.add(id, "Test");
        list.set_color(&id, "blue");
        assert!(list.get(&id).unwrap().color.is_some());
        list.set_color(&id, "");
        assert!(list.get(&id).unwrap().color.is_none());
    }

    #[test]
    fn test_with_notes_filter() {
        let mut list = BookmarkList::new();
        let id1 = make_id();
        let id2 = make_id();
        list.add(id1, "A");
        list.add(id2, "B");
        list.set_note(&id1, "Has note");

        let with_notes = list.with_notes();
        assert_eq!(with_notes.len(), 1);
        assert_eq!(with_notes[0].name, "A");
    }

    #[test]
    fn test_get_nonexistent() {
        let list = BookmarkList::new();
        assert!(list.get(&make_id()).is_none());
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut list = BookmarkList::new();
        let id = make_id();
        list.add(id, "A");
        list.remove(&make_id()); // Remove a different ID
        assert_eq!(list.count(), 1);
    }

    #[test]
    fn test_search_empty_query() {
        let mut list = BookmarkList::new();
        list.add(make_id(), "Test");
        let results = list.search("");
        assert_eq!(results.len(), 1); // Empty query matches all
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut list = BookmarkList::new();
        list.add(make_id(), "Chapter One");
        let results = list.search("CHAPTER");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_reorder_out_of_bounds() {
        let mut list = BookmarkList::new();
        let id = make_id();
        list.add(id, "A");
        list.reorder(0, 10); // Out of bounds
        assert_eq!(list.count(), 1); // No change
    }

    #[test]
    fn test_sorted_by_date() {
        let mut list = BookmarkList::new();
        list.add(make_id(), "First");
        list.add(make_id(), "Second");
        let sorted = list.sorted_by_date();
        // Newest first
        assert_eq!(sorted.len(), 2);
    }

    #[test]
    fn test_used_colors_multiple() {
        let mut list = BookmarkList::new();
        let id1 = make_id();
        let id2 = make_id();
        let id3 = make_id();
        list.add(id1, "A");
        list.add(id2, "B");
        list.add(id3, "C");
        list.set_color(&id1, "red");
        list.set_color(&id2, "blue");
        list.set_color(&id3, "red");

        let colors = list.used_colors();
        assert_eq!(colors.len(), 2);
        assert!(colors.contains(&"blue".to_string()));
        assert!(colors.contains(&"red".to_string()));
    }

    #[test]
    fn test_rename_nonexistent() {
        let mut list = BookmarkList::new();
        list.add(make_id(), "A");
        list.rename(&make_id(), "New Name"); // Different ID
        assert_eq!(list.bookmarks[0].name, "A"); // Unchanged
    }

    // ---- New bookmark tests ----

    #[test]
    fn test_display_label_basic() {
        let bm = Bookmark::new(make_id(), "Chapter 1");
        let label = bm.display_label();
        assert!(label.contains("Chapter 1"));
        assert!(label.contains("none"));
    }

    #[test]
    fn test_display_label_with_color_and_note() {
        let id = make_id();
        let mut bm = Bookmark::new(id, "Ch 1");
        bm.color = Some("red".to_string());
        bm.note = Some("Important".to_string());
        let label = bm.display_label();
        assert!(label.contains("red"));
        assert!(label.contains("[note]"));
    }

    #[test]
    fn test_has_note_and_color() {
        let mut bm = Bookmark::new(make_id(), "Test");
        assert!(!bm.has_note());
        assert!(!bm.has_color());
        bm.note = Some("A note".to_string());
        bm.color = Some("blue".to_string());
        assert!(bm.has_note());
        assert!(bm.has_color());
    }

    #[test]
    fn test_get_mut() {
        let mut list = BookmarkList::new();
        let id = make_id();
        list.add(id, "Test");
        let bm = list.get_mut(&id).unwrap();
        bm.name = "Modified".to_string();
        assert_eq!(list.get(&id).unwrap().name, "Modified");
    }

    #[test]
    fn test_get_mut_nonexistent() {
        let mut list = BookmarkList::new();
        assert!(list.get_mut(&make_id()).is_none());
    }

    #[test]
    fn test_ids() {
        let mut list = BookmarkList::new();
        let id1 = make_id();
        let id2 = make_id();
        list.add(id1, "A");
        list.add(id2, "B");
        let ids = list.ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[test]
    fn test_export_names() {
        let mut list = BookmarkList::new();
        list.add(make_id(), "First");
        list.add(make_id(), "Second");
        list.add(make_id(), "Third");
        let names = list.export_names();
        assert_eq!(names, vec!["First", "Second", "Third"]);
    }

    #[test]
    fn test_export_names_empty() {
        let list = BookmarkList::new();
        assert!(list.export_names().is_empty());
    }

    #[test]
    fn test_grouped_by_color() {
        let mut list = BookmarkList::new();
        let id1 = make_id();
        let id2 = make_id();
        let id3 = make_id();
        list.add(id1, "A");
        list.add(id2, "B");
        list.add(id3, "C");
        list.set_color(&id1, "red");
        list.set_color(&id2, "red");

        let groups = list.grouped_by_color();
        assert_eq!(groups.get(&Some("red".to_string())).unwrap().len(), 2);
        assert_eq!(groups.get(&None).unwrap().len(), 1);
    }

    #[test]
    fn test_search_notes() {
        let mut list = BookmarkList::new();
        let id1 = make_id();
        let id2 = make_id();
        let id3 = make_id();
        list.add(id1, "A");
        list.add(id2, "B");
        list.add(id3, "C");
        list.set_note(&id1, "Important chapter");
        list.set_note(&id2, "Review later");

        let results = list.search_notes("important");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "A");
    }

    #[test]
    fn test_search_notes_no_match() {
        let mut list = BookmarkList::new();
        list.add(make_id(), "A");
        let results = list.search_notes("xyz");
        assert!(results.is_empty());
    }

    #[test]
    fn test_summary() {
        let mut list = BookmarkList::new();
        let id1 = make_id();
        let id2 = make_id();
        list.add(id1, "A");
        list.add(id2, "B");
        list.set_note(&id1, "A note");
        list.set_color(&id1, "red");
        let summary = list.summary();
        assert!(summary.contains("2 bookmarks"));
        assert!(summary.contains("1 with notes"));
        assert!(summary.contains("1 colors"));
    }
}
