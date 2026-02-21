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
}

/// Manages the list of bookmarks for a project
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BookmarkList {
    pub bookmarks: Vec<Bookmark>,
}

impl BookmarkList {
    pub fn new() -> Self {
        Self { bookmarks: Vec::new() }
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
}
