use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A Collection groups binder items outside the normal binder hierarchy.
/// Like Scrivener, collections can be either manual or search-based.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: Uuid,
    pub name: String,
    pub kind: CollectionKind,
    pub item_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollectionKind {
    /// Manually curated list of items
    Manual,
    /// Auto-populated from a saved search
    Search { query: String, case_sensitive: bool, whole_word: bool },
}

impl Collection {
    pub fn new_manual(name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            kind: CollectionKind::Manual,
            item_ids: Vec::new(),
        }
    }

    pub fn new_search(name: &str, query: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            kind: CollectionKind::Search {
                query: query.to_string(),
                case_sensitive: false,
                whole_word: false,
            },
            item_ids: Vec::new(),
        }
    }

    pub fn add_item(&mut self, id: Uuid) {
        if !self.item_ids.contains(&id) {
            self.item_ids.push(id);
        }
    }

    pub fn remove_item(&mut self, id: &Uuid) {
        self.item_ids.retain(|i| i != id);
    }

    /// Check if this collection contains a specific item
    pub fn contains(&self, id: &Uuid) -> bool {
        self.item_ids.contains(id)
    }

    /// Get the item count
    pub fn count(&self) -> usize {
        self.item_ids.len()
    }

    /// Check if this is a search (smart) collection
    pub fn is_smart(&self) -> bool {
        matches!(self.kind, CollectionKind::Search { .. })
    }

    /// Get the search query if this is a smart collection
    pub fn search_query(&self) -> Option<&str> {
        match &self.kind {
            CollectionKind::Search { query, .. } => Some(query.as_str()),
            CollectionKind::Manual => None,
        }
    }

    /// Clear all items from the collection
    pub fn clear(&mut self) {
        self.item_ids.clear();
    }

    /// Reorder: move an item to a new position
    pub fn move_item(&mut self, from: usize, to: usize) {
        if from < self.item_ids.len() && to < self.item_ids.len() {
            let item = self.item_ids.remove(from);
            self.item_ids.insert(to, item);
        }
    }

    /// Get display info about collection kind
    pub fn kind_label(&self) -> &str {
        match &self.kind {
            CollectionKind::Manual => "Manual",
            CollectionKind::Search { .. } => "Smart",
        }
    }

    /// Rename the collection
    pub fn rename(&mut self, new_name: &str) {
        self.name = new_name.to_string();
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.item_ids.is_empty()
    }

    /// Get item IDs as a slice
    pub fn items(&self) -> &[Uuid] {
        &self.item_ids
    }

    /// Add multiple items at once
    pub fn add_items(&mut self, ids: &[Uuid]) {
        for id in ids {
            self.add_item(*id);
        }
    }

    /// Get the search options if this is a smart collection
    pub fn search_options(&self) -> Option<(&str, bool, bool)> {
        match &self.kind {
            CollectionKind::Search {
                query,
                case_sensitive,
                whole_word,
            } => Some((query.as_str(), *case_sensitive, *whole_word)),
            CollectionKind::Manual => None,
        }
    }

    /// Sort items by a custom comparator
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&Uuid, &Uuid) -> std::cmp::Ordering,
    {
        self.item_ids.sort_by(compare);
    }

    /// Get item at a specific index
    pub fn get(&self, index: usize) -> Option<&Uuid> {
        self.item_ids.get(index)
    }

    /// Summary string for display
    pub fn summary(&self) -> String {
        let kind = self.kind_label();
        format!("{} ({}, {} items)", self.name, kind, self.count())
    }

    /// Check if an item is at a specific position
    pub fn item_at_position(&self, pos: usize) -> Option<&Uuid> {
        self.item_ids.get(pos)
    }

    /// Find the position of an item in the collection
    pub fn position_of(&self, id: &Uuid) -> Option<usize> {
        self.item_ids.iter().position(|i| i == id)
    }

    /// Swap two items by their positions
    pub fn swap(&mut self, a: usize, b: usize) {
        if a < self.item_ids.len() && b < self.item_ids.len() {
            self.item_ids.swap(a, b);
        }
    }

    /// Remove duplicate item IDs (preserving first occurrence)
    pub fn dedup(&mut self) {
        let mut seen = std::collections::HashSet::new();
        self.item_ids.retain(|id| seen.insert(*id));
    }

    /// Keep only items that pass the predicate
    pub fn retain<F>(&mut self, f: F) where F: FnMut(&Uuid) -> bool {
        self.item_ids.retain(f);
    }
}

impl CollectionKind {
    /// Human-readable label
    pub fn label(&self) -> &str {
        match self {
            CollectionKind::Manual => "Manual",
            CollectionKind::Search { .. } => "Smart",
        }
    }

    /// Icon character for UI display
    pub fn icon(&self) -> &str {
        match self {
            CollectionKind::Manual => "\u{2630}",
            CollectionKind::Search { .. } => "\u{2605}",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_new() {
        let coll = Collection::new_manual("Test Collection");
        assert_eq!(coll.name, "Test Collection");
        assert!(coll.item_ids.is_empty());
        assert!(coll.is_empty());
        assert!(matches!(coll.kind, CollectionKind::Manual));
    }

    #[test]
    fn test_collection_new_search() {
        let coll = Collection::new_search("Search Results", "query");
        assert!(matches!(coll.kind, CollectionKind::Search { .. }));
        assert_eq!(coll.kind.label(), "Smart");
    }

    #[test]
    fn test_add_remove_items() {
        let mut coll = Collection::new_manual("Test");
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        coll.add_item(id1);
        coll.add_item(id2);
        assert_eq!(coll.item_ids.len(), 2);
        assert!(coll.contains(&id1));

        coll.remove_item(&id1);
        assert_eq!(coll.item_ids.len(), 1);
        assert!(!coll.contains(&id1));
    }

    #[test]
    fn test_no_duplicate_items() {
        let mut coll = Collection::new_manual("Test");
        let id = Uuid::new_v4();
        coll.add_item(id);
        coll.add_item(id);
        assert_eq!(coll.item_ids.len(), 1);
    }

    #[test]
    fn test_item_at_position() {
        let mut coll = Collection::new_manual("Test");
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        coll.add_item(id1);
        coll.add_item(id2);

        assert_eq!(coll.item_at_position(0), Some(&id1));
        assert_eq!(coll.item_at_position(1), Some(&id2));
        assert_eq!(coll.item_at_position(2), None);
    }

    #[test]
    fn test_position_of() {
        let mut coll = Collection::new_manual("Test");
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        coll.add_item(id1);
        coll.add_item(id2);

        assert_eq!(coll.position_of(&id1), Some(0));
        assert_eq!(coll.position_of(&id2), Some(1));
    }

    #[test]
    fn test_swap() {
        let mut coll = Collection::new_manual("Test");
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        coll.add_item(id1);
        coll.add_item(id2);
        coll.swap(0, 1);
        assert_eq!(coll.item_at_position(0), Some(&id2));
        assert_eq!(coll.item_at_position(1), Some(&id1));
    }

    #[test]
    fn test_dedup() {
        let mut coll = Collection::new_manual("Test");
        let id = Uuid::new_v4();
        coll.item_ids.push(id);
        coll.item_ids.push(id);
        coll.item_ids.push(id);
        coll.dedup();
        assert_eq!(coll.item_ids.len(), 1);
    }

    #[test]
    fn test_retain() {
        let mut coll = Collection::new_manual("Test");
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        coll.add_item(id1);
        coll.add_item(id2);
        coll.add_item(id3);
        coll.retain(|id| *id != id2);
        assert_eq!(coll.item_ids.len(), 2);
        assert!(!coll.contains(&id2));
    }

    #[test]
    fn test_clear() {
        let mut coll = Collection::new_manual("Test");
        coll.add_item(Uuid::new_v4());
        coll.add_item(Uuid::new_v4());
        coll.clear();
        assert!(coll.is_empty());
    }

    #[test]
    fn test_collection_kind_labels() {
        let manual = CollectionKind::Manual;
        assert_eq!(manual.label(), "Manual");
        assert_eq!(manual.icon(), "\u{2630}");

        let search = CollectionKind::Search {
            query: "test".to_string(),
            case_sensitive: false,
            whole_word: false,
        };
        assert_eq!(search.label(), "Smart");
    }
}
