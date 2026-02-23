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

    /// Get intersection of items with another collection
    pub fn intersection(&self, other: &Collection) -> Vec<Uuid> {
        let other_set: std::collections::HashSet<&Uuid> = other.item_ids.iter().collect();
        self.item_ids.iter().filter(|id| other_set.contains(id)).copied().collect()
    }

    /// Get union of items with another collection (no duplicates)
    pub fn union(&self, other: &Collection) -> Vec<Uuid> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        for id in self.item_ids.iter().chain(other.item_ids.iter()) {
            if seen.insert(*id) {
                result.push(*id);
            }
        }
        result
    }

    /// Get items in this collection but not in the other
    pub fn difference(&self, other: &Collection) -> Vec<Uuid> {
        let other_set: std::collections::HashSet<&Uuid> = other.item_ids.iter().collect();
        self.item_ids.iter().filter(|id| !other_set.contains(id)).copied().collect()
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

/// Manages a set of collections
pub struct CollectionManager {
    pub collections: Vec<Collection>,
}

impl CollectionManager {
    pub fn new() -> Self {
        Self { collections: Vec::new() }
    }

    /// Add a new collection, returning its UUID
    pub fn add(&mut self, collection: Collection) -> Uuid {
        let id = collection.id;
        self.collections.push(collection);
        id
    }

    /// Remove a collection by ID
    pub fn remove(&mut self, id: &Uuid) -> bool {
        let before = self.collections.len();
        self.collections.retain(|c| c.id != *id);
        self.collections.len() < before
    }

    /// Find a collection by ID
    pub fn get(&self, id: &Uuid) -> Option<&Collection> {
        self.collections.iter().find(|c| &c.id == id)
    }

    /// Find a mutable collection by ID
    pub fn get_mut(&mut self, id: &Uuid) -> Option<&mut Collection> {
        self.collections.iter_mut().find(|c| &c.id == id)
    }

    /// Find a collection by name (case-insensitive)
    pub fn find_by_name(&self, name: &str) -> Option<&Collection> {
        let lower = name.to_lowercase();
        self.collections.iter().find(|c| c.name.to_lowercase() == lower)
    }

    /// Get all manual collections
    pub fn manual_collections(&self) -> Vec<&Collection> {
        self.collections.iter().filter(|c| !c.is_smart()).collect()
    }

    /// Get all smart (search) collections
    pub fn smart_collections(&self) -> Vec<&Collection> {
        self.collections.iter().filter(|c| c.is_smart()).collect()
    }

    /// Total number of collections
    pub fn count(&self) -> usize {
        self.collections.len()
    }

    /// Total items across all collections (may include duplicates)
    pub fn total_items(&self) -> usize {
        self.collections.iter().map(|c| c.count()).sum()
    }

    /// Unique items across all collections
    pub fn unique_items(&self) -> usize {
        let all: std::collections::HashSet<Uuid> = self.collections.iter()
            .flat_map(|c| c.item_ids.iter().copied())
            .collect();
        all.len()
    }

    /// Find all collections that contain a given item
    pub fn collections_containing(&self, item_id: &Uuid) -> Vec<&Collection> {
        self.collections.iter().filter(|c| c.contains(item_id)).collect()
    }

    /// Remove an item from all collections
    pub fn remove_item_from_all(&mut self, item_id: &Uuid) {
        for coll in &mut self.collections {
            coll.remove_item(item_id);
        }
    }

    /// Get collections sorted by name
    pub fn sorted_by_name(&self) -> Vec<&Collection> {
        let mut sorted: Vec<&Collection> = self.collections.iter().collect();
        sorted.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        sorted
    }

    /// Get collections sorted by item count (descending)
    pub fn sorted_by_count(&self) -> Vec<&Collection> {
        let mut sorted: Vec<&Collection> = self.collections.iter().collect();
        sorted.sort_by(|a, b| b.count().cmp(&a.count()));
        sorted
    }

    /// Check if any collection contains the item
    pub fn item_in_any_collection(&self, item_id: &Uuid) -> bool {
        self.collections.iter().any(|c| c.contains(item_id))
    }

    /// Get a summary string
    pub fn summary(&self) -> String {
        let manual = self.manual_collections().len();
        let smart = self.smart_collections().len();
        let total = self.total_items();
        format!("{} collections ({} manual, {} smart), {} total items", self.count(), manual, smart, total)
    }

    /// Check if manager is empty
    pub fn is_empty(&self) -> bool {
        self.collections.is_empty()
    }

    /// Duplicate a collection with a new name
    pub fn duplicate(&mut self, id: &Uuid, new_name: &str) -> Option<Uuid> {
        let original = self.get(id)?.clone();
        let dup = Collection {
            id: Uuid::new_v4(),
            name: new_name.to_string(),
            kind: original.kind,
            item_ids: original.item_ids,
        };
        let new_id = dup.id;
        self.collections.push(dup);
        Some(new_id)
    }

    /// Merge two collections into a new one (union of items)
    pub fn merge(&mut self, id_a: &Uuid, id_b: &Uuid, name: &str) -> Option<Uuid> {
        let a = self.get(id_a)?;
        let b = self.get(id_b)?;
        let union = a.union(b);
        let mut merged = Collection::new_manual(name);
        merged.item_ids = union;
        let new_id = merged.id;
        self.collections.push(merged);
        Some(new_id)
    }

    /// Get all collection names
    pub fn names(&self) -> Vec<&str> {
        self.collections.iter().map(|c| c.name.as_str()).collect()
    }

    /// Get the largest collection (by item count)
    pub fn largest(&self) -> Option<&Collection> {
        self.collections.iter().max_by_key(|c| c.count())
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

    #[test]
    fn test_search_query() {
        let manual = Collection::new_manual("Manual");
        assert!(manual.search_query().is_none());

        let search = Collection::new_search("Results", "dragon");
        assert_eq!(search.search_query(), Some("dragon"));
    }

    #[test]
    fn test_search_options() {
        let search = Collection::new_search("Results", "query");
        let (q, cs, ww) = search.search_options().unwrap();
        assert_eq!(q, "query");
        assert!(!cs);
        assert!(!ww);

        let manual = Collection::new_manual("Manual");
        assert!(manual.search_options().is_none());
    }

    #[test]
    fn test_is_smart() {
        let manual = Collection::new_manual("Manual");
        assert!(!manual.is_smart());

        let search = Collection::new_search("Smart", "test");
        assert!(search.is_smart());
    }

    #[test]
    fn test_items_slice() {
        let mut coll = Collection::new_manual("Test");
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        coll.add_item(id1);
        coll.add_item(id2);

        let items = coll.items();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], id1);
        assert_eq!(items[1], id2);
    }

    #[test]
    fn test_add_items_bulk() {
        let mut coll = Collection::new_manual("Test");
        let ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();
        coll.add_items(&ids);
        assert_eq!(coll.count(), 5);
    }

    #[test]
    fn test_add_items_bulk_no_duplicates() {
        let mut coll = Collection::new_manual("Test");
        let id = Uuid::new_v4();
        coll.add_item(id);
        coll.add_items(&[id, id, id]);
        assert_eq!(coll.count(), 1);
    }

    #[test]
    fn test_get() {
        let mut coll = Collection::new_manual("Test");
        let id = Uuid::new_v4();
        coll.add_item(id);

        assert_eq!(coll.get(0), Some(&id));
        assert_eq!(coll.get(1), None);
    }

    #[test]
    fn test_summary() {
        let mut coll = Collection::new_manual("My Collection");
        coll.add_item(Uuid::new_v4());
        coll.add_item(Uuid::new_v4());

        let summary = coll.summary();
        assert!(summary.contains("My Collection"));
        assert!(summary.contains("Manual"));
        assert!(summary.contains("2 items"));
    }

    #[test]
    fn test_summary_smart() {
        let coll = Collection::new_search("Dragon Scenes", "dragon");
        let summary = coll.summary();
        assert!(summary.contains("Smart"));
        assert!(summary.contains("0 items"));
    }

    #[test]
    fn test_rename() {
        let mut coll = Collection::new_manual("Old Name");
        coll.rename("New Name");
        assert_eq!(coll.name, "New Name");
    }

    #[test]
    fn test_move_item_valid() {
        let mut coll = Collection::new_manual("Test");
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        coll.add_item(id1);
        coll.add_item(id2);
        coll.add_item(id3);

        coll.move_item(0, 2); // Move first to last
        assert_eq!(coll.item_at_position(0), Some(&id2));
        assert_eq!(coll.item_at_position(2), Some(&id1));
    }

    #[test]
    fn test_move_item_out_of_bounds() {
        let mut coll = Collection::new_manual("Test");
        let id = Uuid::new_v4();
        coll.add_item(id);

        // Should be a no-op when indices are out of bounds
        coll.move_item(0, 5);
        assert_eq!(coll.count(), 1);
        assert_eq!(coll.item_at_position(0), Some(&id));
    }

    #[test]
    fn test_swap_out_of_bounds() {
        let mut coll = Collection::new_manual("Test");
        let id = Uuid::new_v4();
        coll.add_item(id);

        // Should be a no-op
        coll.swap(0, 5);
        assert_eq!(coll.item_at_position(0), Some(&id));
    }

    #[test]
    fn test_sort_by() {
        let mut coll = Collection::new_manual("Test");
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        coll.add_item(id1);
        coll.add_item(id2);
        coll.add_item(id3);

        // Sort by UUID string representation
        coll.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
        // Just verify it doesn't crash and preserves count
        assert_eq!(coll.count(), 3);
    }

    #[test]
    fn test_kind_label() {
        let coll = Collection::new_manual("Test");
        assert_eq!(coll.kind_label(), "Manual");

        let coll2 = Collection::new_search("Test", "query");
        assert_eq!(coll2.kind_label(), "Smart");
    }

    #[test]
    fn test_collection_kind_icons() {
        let manual = CollectionKind::Manual;
        assert!(!manual.icon().is_empty());

        let search = CollectionKind::Search {
            query: "test".to_string(),
            case_sensitive: false,
            whole_word: false,
        };
        assert!(!search.icon().is_empty());
    }

    #[test]
    fn test_remove_nonexistent_item() {
        let mut coll = Collection::new_manual("Test");
        let id = Uuid::new_v4();
        coll.add_item(id);

        let fake = Uuid::new_v4();
        coll.remove_item(&fake); // Should be no-op
        assert_eq!(coll.count(), 1);
    }

    #[test]
    fn test_position_of_not_found() {
        let coll = Collection::new_manual("Test");
        let fake = Uuid::new_v4();
        assert!(coll.position_of(&fake).is_none());
    }

    // CollectionManager tests

    #[test]
    fn test_manager_new() {
        let mgr = CollectionManager::new();
        assert!(mgr.is_empty());
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_manager_add_remove() {
        let mut mgr = CollectionManager::new();
        let coll = Collection::new_manual("My List");
        let id = mgr.add(coll);
        assert_eq!(mgr.count(), 1);
        assert!(mgr.get(&id).is_some());

        assert!(mgr.remove(&id));
        assert!(mgr.is_empty());
    }

    #[test]
    fn test_manager_remove_nonexistent() {
        let mut mgr = CollectionManager::new();
        assert!(!mgr.remove(&Uuid::new_v4()));
    }

    #[test]
    fn test_manager_get_mut() {
        let mut mgr = CollectionManager::new();
        let coll = Collection::new_manual("Editable");
        let id = mgr.add(coll);
        mgr.get_mut(&id).unwrap().add_item(Uuid::new_v4());
        assert_eq!(mgr.get(&id).unwrap().count(), 1);
    }

    #[test]
    fn test_manager_find_by_name() {
        let mut mgr = CollectionManager::new();
        mgr.add(Collection::new_manual("Favorites"));
        mgr.add(Collection::new_search("Dragon Scenes", "dragon"));

        assert!(mgr.find_by_name("favorites").is_some());
        assert!(mgr.find_by_name("FAVORITES").is_some());
        assert!(mgr.find_by_name("nonexistent").is_none());
    }

    #[test]
    fn test_manager_manual_smart_split() {
        let mut mgr = CollectionManager::new();
        mgr.add(Collection::new_manual("Manual 1"));
        mgr.add(Collection::new_manual("Manual 2"));
        mgr.add(Collection::new_search("Smart 1", "query"));

        assert_eq!(mgr.manual_collections().len(), 2);
        assert_eq!(mgr.smart_collections().len(), 1);
    }

    #[test]
    fn test_manager_total_items() {
        let mut mgr = CollectionManager::new();
        let mut c1 = Collection::new_manual("A");
        c1.add_item(Uuid::new_v4());
        c1.add_item(Uuid::new_v4());
        let mut c2 = Collection::new_manual("B");
        c2.add_item(Uuid::new_v4());
        mgr.add(c1);
        mgr.add(c2);

        assert_eq!(mgr.total_items(), 3);
    }

    #[test]
    fn test_manager_unique_items() {
        let mut mgr = CollectionManager::new();
        let shared_id = Uuid::new_v4();
        let unique_id = Uuid::new_v4();

        let mut c1 = Collection::new_manual("A");
        c1.add_item(shared_id);
        c1.add_item(unique_id);
        let mut c2 = Collection::new_manual("B");
        c2.add_item(shared_id); // same item in both

        mgr.add(c1);
        mgr.add(c2);

        assert_eq!(mgr.total_items(), 3); // counting duplicates
        assert_eq!(mgr.unique_items(), 2); // unique items
    }

    #[test]
    fn test_manager_collections_containing() {
        let mut mgr = CollectionManager::new();
        let item_id = Uuid::new_v4();

        let mut c1 = Collection::new_manual("Has it");
        c1.add_item(item_id);
        let c2 = Collection::new_manual("Doesn't have it");

        mgr.add(c1);
        mgr.add(c2);

        let containing = mgr.collections_containing(&item_id);
        assert_eq!(containing.len(), 1);
        assert_eq!(containing[0].name, "Has it");
    }

    #[test]
    fn test_manager_remove_item_from_all() {
        let mut mgr = CollectionManager::new();
        let item_id = Uuid::new_v4();

        let mut c1 = Collection::new_manual("A");
        c1.add_item(item_id);
        let mut c2 = Collection::new_manual("B");
        c2.add_item(item_id);

        mgr.add(c1);
        mgr.add(c2);

        mgr.remove_item_from_all(&item_id);
        assert!(!mgr.item_in_any_collection(&item_id));
    }

    #[test]
    fn test_manager_sorted_by_name() {
        let mut mgr = CollectionManager::new();
        mgr.add(Collection::new_manual("Zebra"));
        mgr.add(Collection::new_manual("Alpha"));
        mgr.add(Collection::new_manual("Middle"));

        let sorted = mgr.sorted_by_name();
        assert_eq!(sorted[0].name, "Alpha");
        assert_eq!(sorted[1].name, "Middle");
        assert_eq!(sorted[2].name, "Zebra");
    }

    #[test]
    fn test_manager_sorted_by_count() {
        let mut mgr = CollectionManager::new();
        let mut big = Collection::new_manual("Big");
        big.add_item(Uuid::new_v4());
        big.add_item(Uuid::new_v4());
        big.add_item(Uuid::new_v4());

        let small = Collection::new_manual("Small");

        let mut medium = Collection::new_manual("Medium");
        medium.add_item(Uuid::new_v4());

        mgr.add(small);
        mgr.add(big);
        mgr.add(medium);

        let sorted = mgr.sorted_by_count();
        assert_eq!(sorted[0].name, "Big");
        assert_eq!(sorted[1].name, "Medium");
        assert_eq!(sorted[2].name, "Small");
    }

    #[test]
    fn test_manager_item_in_any_collection() {
        let mut mgr = CollectionManager::new();
        let item_id = Uuid::new_v4();

        assert!(!mgr.item_in_any_collection(&item_id));

        let mut c = Collection::new_manual("Test");
        c.add_item(item_id);
        mgr.add(c);

        assert!(mgr.item_in_any_collection(&item_id));
    }

    #[test]
    fn test_manager_summary() {
        let mut mgr = CollectionManager::new();
        mgr.add(Collection::new_manual("A"));
        mgr.add(Collection::new_search("B", "q"));

        let summary = mgr.summary();
        assert!(summary.contains("2 collections"));
        assert!(summary.contains("1 manual"));
        assert!(summary.contains("1 smart"));
    }

    // ---- New collection tests ----

    #[test]
    fn test_intersection() {
        let mut a = Collection::new_manual("A");
        let mut b = Collection::new_manual("B");
        let shared = Uuid::new_v4();
        let only_a = Uuid::new_v4();
        let only_b = Uuid::new_v4();
        a.add_item(shared);
        a.add_item(only_a);
        b.add_item(shared);
        b.add_item(only_b);

        let inter = a.intersection(&b);
        assert_eq!(inter.len(), 1);
        assert!(inter.contains(&shared));
    }

    #[test]
    fn test_intersection_empty() {
        let a = Collection::new_manual("A");
        let b = Collection::new_manual("B");
        assert!(a.intersection(&b).is_empty());
    }

    #[test]
    fn test_union() {
        let mut a = Collection::new_manual("A");
        let mut b = Collection::new_manual("B");
        let shared = Uuid::new_v4();
        let only_a = Uuid::new_v4();
        let only_b = Uuid::new_v4();
        a.add_item(shared);
        a.add_item(only_a);
        b.add_item(shared);
        b.add_item(only_b);

        let un = a.union(&b);
        assert_eq!(un.len(), 3); // shared + only_a + only_b
    }

    #[test]
    fn test_difference() {
        let mut a = Collection::new_manual("A");
        let mut b = Collection::new_manual("B");
        let shared = Uuid::new_v4();
        let only_a = Uuid::new_v4();
        a.add_item(shared);
        a.add_item(only_a);
        b.add_item(shared);

        let diff = a.difference(&b);
        assert_eq!(diff.len(), 1);
        assert!(diff.contains(&only_a));
    }

    #[test]
    fn test_difference_empty() {
        let mut a = Collection::new_manual("A");
        let mut b = Collection::new_manual("B");
        let id = Uuid::new_v4();
        a.add_item(id);
        b.add_item(id);
        assert!(a.difference(&b).is_empty());
    }

    #[test]
    fn test_manager_duplicate() {
        let mut mgr = CollectionManager::new();
        let mut c = Collection::new_manual("Original");
        c.add_item(Uuid::new_v4());
        c.add_item(Uuid::new_v4());
        let orig_id = mgr.add(c);

        let dup_id = mgr.duplicate(&orig_id, "Copy of Original").unwrap();
        assert_ne!(orig_id, dup_id);
        assert_eq!(mgr.get(&dup_id).unwrap().name, "Copy of Original");
        assert_eq!(mgr.get(&dup_id).unwrap().count(), 2);
    }

    #[test]
    fn test_manager_duplicate_nonexistent() {
        let mut mgr = CollectionManager::new();
        assert!(mgr.duplicate(&Uuid::new_v4(), "Nope").is_none());
    }

    #[test]
    fn test_manager_merge() {
        let mut mgr = CollectionManager::new();
        let shared = Uuid::new_v4();

        let mut a = Collection::new_manual("A");
        a.add_item(shared);
        a.add_item(Uuid::new_v4());
        let id_a = mgr.add(a);

        let mut b = Collection::new_manual("B");
        b.add_item(shared);
        b.add_item(Uuid::new_v4());
        let id_b = mgr.add(b);

        let merged_id = mgr.merge(&id_a, &id_b, "Merged").unwrap();
        let merged = mgr.get(&merged_id).unwrap();
        assert_eq!(merged.name, "Merged");
        assert_eq!(merged.count(), 3); // 2 unique from A + 1 unique from B
    }

    #[test]
    fn test_manager_names() {
        let mut mgr = CollectionManager::new();
        mgr.add(Collection::new_manual("Alpha"));
        mgr.add(Collection::new_manual("Beta"));
        let names = mgr.names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"Alpha"));
        assert!(names.contains(&"Beta"));
    }

    #[test]
    fn test_manager_largest() {
        let mut mgr = CollectionManager::new();
        let mut big = Collection::new_manual("Big");
        big.add_item(Uuid::new_v4());
        big.add_item(Uuid::new_v4());
        big.add_item(Uuid::new_v4());
        let mut small = Collection::new_manual("Small");
        small.add_item(Uuid::new_v4());
        mgr.add(small);
        mgr.add(big);

        let largest = mgr.largest().unwrap();
        assert_eq!(largest.name, "Big");
    }

    #[test]
    fn test_manager_largest_empty() {
        let mgr = CollectionManager::new();
        assert!(mgr.largest().is_none());
    }
}
