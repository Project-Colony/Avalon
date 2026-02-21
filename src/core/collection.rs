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
}
