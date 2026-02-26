use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    fn test_bookmark_list_add_remove() {
        let mut list = BookmarkList::new();
        assert!(list.bookmarks.is_empty());

        let id1 = make_id();
        let id2 = make_id();
        list.add(id1, "First");
        list.add(id2, "Second");
        assert_eq!(list.bookmarks.len(), 2);
        assert!(list.is_bookmarked(&id1));

        list.remove(&id1);
        assert_eq!(list.bookmarks.len(), 1);
        assert!(!list.is_bookmarked(&id1));
    }

    #[test]
    fn test_no_duplicates() {
        let mut list = BookmarkList::new();
        let id = make_id();
        list.add(id, "Test");
        list.add(id, "Test Again");
        assert_eq!(list.bookmarks.len(), 1);
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
    fn test_remove_nonexistent() {
        let mut list = BookmarkList::new();
        let id = make_id();
        list.add(id, "A");
        list.remove(&make_id()); // Remove a different ID
        assert_eq!(list.bookmarks.len(), 1);
    }
}
