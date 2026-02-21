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
}
