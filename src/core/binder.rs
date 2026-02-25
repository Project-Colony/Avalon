use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::document::Document;
use super::metadata::Metadata;
use super::snapshot::Snapshot;

/// The Binder is the organizational backbone of a Scrinever project.
/// It's a tree structure that holds all documents, folders, and their hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binder {
    /// The Draft/Manuscript folder — contains the actual writing
    pub draft: BinderItem,
    /// The Research folder — contains reference materials
    pub research: BinderItem,
    /// The Trash folder — contains deleted items
    pub trash: BinderItem,
}

impl Binder {
    /// Create a default binder structure
    pub fn default_structure() -> Self {
        Self {
            draft: BinderItem::new_folder("Draft"),
            research: BinderItem::new_folder("Research"),
            trash: BinderItem::new_folder("Trash"),
        }
    }

    /// Find an item by ID anywhere in the binder
    pub fn find_item(&self, id: &Uuid) -> Option<&BinderItem> {
        self.draft.find(id)
            .or_else(|| self.research.find(id))
            .or_else(|| self.trash.find(id))
    }

    /// Find a mutable item by ID anywhere in the binder
    pub fn find_item_mut(&mut self, id: &Uuid) -> Option<&mut BinderItem> {
        if let Some(item) = self.draft.find_mut(id) {
            return Some(item);
        }
        if let Some(item) = self.research.find_mut(id) {
            return Some(item);
        }
        if let Some(item) = self.trash.find_mut(id) {
            return Some(item);
        }
        None
    }

    /// Get all items as a flat list (for iteration)
    pub fn all_items(&self) -> Vec<&BinderItem> {
        let mut items = Vec::new();
        self.draft.collect_all(&mut items);
        self.research.collect_all(&mut items);
        self.trash.collect_all(&mut items);
        items
    }

    /// Get all items as mutable flat list
    pub fn all_items_mut(&mut self) -> Vec<&mut BinderItem> {
        let mut items = Vec::new();
        self.draft.collect_all_mut(&mut items);
        self.research.collect_all_mut(&mut items);
        self.trash.collect_all_mut(&mut items);
        items
    }

    /// Move an item to trash
    pub fn move_to_trash(&mut self, id: &Uuid) -> bool {
        if let Some(item) = self.draft.remove_child(id).or_else(|| self.research.remove_child(id)) {
            self.trash.children.push(item);
            return true;
        }
        false
    }

    /// Visit each item in the binder tree without allocating a Vec.
    pub fn for_each_item<F: FnMut(&BinderItem)>(&self, mut f: F) {
        self.draft.visit(&mut f);
        self.research.visit(&mut f);
        self.trash.visit(&mut f);
    }

    /// Visit each item mutably in the binder tree without allocating a Vec.
    pub fn for_each_item_mut<F: FnMut(&mut BinderItem)>(&mut self, mut f: F) {
        self.draft.visit_mut(&mut f);
        self.research.visit_mut(&mut f);
        self.trash.visit_mut(&mut f);
    }

    /// Visit each item mutably with early-return on error.
    pub fn try_for_each_item_mut<E, F>(&mut self, mut f: F) -> std::result::Result<(), E>
    where
        F: FnMut(&mut BinderItem) -> std::result::Result<(), E>,
    {
        self.draft.try_visit_mut(&mut f)?;
        self.research.try_visit_mut(&mut f)?;
        self.trash.try_visit_mut(&mut f)?;
        Ok(())
    }

    /// Count total words across all documents
    pub fn total_word_count(&self) -> usize {
        let mut total = 0;
        self.for_each_item(|i| {
            if let Some(ref doc) = i.document {
                total += doc.word_count();
            }
        });
        total
    }

    /// Get all text content concatenated (for readability analysis)
    pub fn all_text(&self) -> String {
        let mut result = String::with_capacity(self.total_char_count());
        self.for_each_item(|item| {
            if let Some(ref doc) = item.document {
                if !doc.content.trim().is_empty() {
                    if !result.is_empty() {
                        result.push_str("\n\n");
                    }
                    result.push_str(&doc.content);
                }
            }
        });
        result
    }

    /// Move an item up in its parent's children list
    pub fn move_item_up(&mut self, id: &Uuid) -> bool {
        if self.draft.move_child_up(id) { return true; }
        if self.research.move_child_up(id) { return true; }
        self.trash.move_child_up(id)
    }

    /// Move an item down in its parent's children list
    pub fn move_item_down(&mut self, id: &Uuid) -> bool {
        if self.draft.move_child_down(id) { return true; }
        if self.research.move_child_down(id) { return true; }
        self.trash.move_child_down(id)
    }

    /// Empty the trash permanently
    pub fn empty_trash(&mut self) {
        self.trash.children.clear();
    }

    /// Duplicate an item (creates a copy next to the original)
    pub fn duplicate_item(&mut self, id: &Uuid) -> Option<Uuid> {
        self.draft.duplicate_child(id)
            .or_else(|| self.research.duplicate_child(id))
    }

    /// Convert a text item into a folder (keeps content as a child document)
    pub fn convert_to_folder(&mut self, id: &Uuid) -> bool {
        if let Some(item) = self.find_item_mut(id) {
            if item.kind == BinderItemKind::Text {
                item.kind = BinderItemKind::Folder;
                item.expanded = true;
                return true;
            }
        }
        false
    }

    /// Convert a folder into a text item (merges children content)
    pub fn convert_to_text(&mut self, id: &Uuid) -> bool {
        if let Some(item) = self.find_item_mut(id) {
            if item.kind == BinderItemKind::Folder && item.children.is_empty() {
                item.kind = BinderItemKind::Text;
                if item.document.is_none() {
                    item.document = Some(Document::new());
                }
                return true;
            }
        }
        false
    }
}

/// A single item in the Binder tree — can be a folder or a text document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinderItem {
    pub id: Uuid,
    pub title: String,
    pub kind: BinderItemKind,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub metadata: Metadata,
    pub children: Vec<BinderItem>,
    #[serde(skip)]
    pub document: Option<Document>,
    pub snapshots: Vec<Snapshot>,
    pub expanded: bool,
    pub include_in_compile: bool,
    /// Synopsis text for the corkboard
    pub synopsis: String,
}

impl BinderItem {
    /// Create a new folder
    pub fn new_folder(title: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.to_string(),
            kind: BinderItemKind::Folder,
            created_at: now,
            modified_at: now,
            metadata: Metadata::default(),
            children: Vec::new(),
            document: None,
            snapshots: Vec::new(),
            expanded: true,
            include_in_compile: true,
            synopsis: String::new(),
        }
    }

    /// Create a new text document
    pub fn new_text(title: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.to_string(),
            kind: BinderItemKind::Text,
            created_at: now,
            modified_at: now,
            metadata: Metadata::default(),
            children: Vec::new(),
            document: Some(Document::new()),
            snapshots: Vec::new(),
            expanded: false,
            include_in_compile: true,
            synopsis: String::new(),
        }
    }

    /// Find an item by ID recursively
    pub fn find(&self, id: &Uuid) -> Option<&BinderItem> {
        if &self.id == id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find(id) {
                return Some(found);
            }
        }
        None
    }

    /// Find a mutable item by ID recursively
    pub fn find_mut(&mut self, id: &Uuid) -> Option<&mut BinderItem> {
        if &self.id == id {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_mut(id) {
                return Some(found);
            }
        }
        None
    }

    /// Remove a child by ID (returns the removed item)
    pub fn remove_child(&mut self, id: &Uuid) -> Option<BinderItem> {
        if let Some(pos) = self.children.iter().position(|c| &c.id == id) {
            return Some(self.children.remove(pos));
        }
        for child in &mut self.children {
            if let Some(removed) = child.remove_child(id) {
                return Some(removed);
            }
        }
        None
    }

    /// Visit this item and all descendants without allocating a Vec.
    pub fn visit<F: FnMut(&BinderItem)>(&self, f: &mut F) {
        f(self);
        for child in &self.children {
            child.visit(f);
        }
    }

    /// Visit this item and all descendants mutably without allocating a Vec.
    /// Safe because `f(self)` releases its borrow before we access `self.children`.
    pub fn visit_mut<F: FnMut(&mut BinderItem)>(&mut self, f: &mut F) {
        f(self);
        for child in &mut self.children {
            child.visit_mut(f);
        }
    }

    /// Visit this item and all descendants mutably with early-return on error.
    pub fn try_visit_mut<E, F>(&mut self, f: &mut F) -> std::result::Result<(), E>
    where
        F: FnMut(&mut BinderItem) -> std::result::Result<(), E>,
    {
        f(self)?;
        for child in &mut self.children {
            child.try_visit_mut(f)?;
        }
        Ok(())
    }

    /// Collect all items into a flat list
    pub fn collect_all<'a>(&'a self, items: &mut Vec<&'a BinderItem>) {
        items.push(self);
        for child in &self.children {
            child.collect_all(items);
        }
    }

    /// Collect all items into a mutable flat list.
    ///
    /// This uses raw pointers because Rust's borrow checker cannot express
    /// simultaneous `&mut item` and `&mut item.children` across function
    /// boundaries. Prefer `visit_mut` or `for_each_item_mut` when possible.
    ///
    /// # Safety invariant
    /// Each node in the binder tree appears exactly once (it is a tree, not a DAG),
    /// so no aliasing occurs between the mutable references produced.
    pub fn collect_all_mut<'a>(&'a mut self, items: &mut Vec<&'a mut BinderItem>) {
        let mut stack: Vec<*mut BinderItem> = vec![self as *mut BinderItem];
        while let Some(ptr) = stack.pop() {
            // SAFETY: Each node in the tree appears exactly once, so no aliasing.
            // The pointers all originate from `&'a mut self` and its children.
            let node = unsafe { &mut *ptr };
            for child in node.children.iter_mut().rev() {
                stack.push(child as *mut BinderItem);
            }
            items.push(node);
        }
    }

    /// Add a child item
    pub fn add_child(&mut self, item: BinderItem) {
        self.children.push(item);
        self.modified_at = Utc::now();
    }

    /// Insert a child at a specific position
    pub fn insert_child(&mut self, index: usize, item: BinderItem) {
        let index = index.min(self.children.len());
        self.children.insert(index, item);
        self.modified_at = Utc::now();
    }

    /// Get the total word count for this item and all children
    pub fn total_word_count(&self) -> usize {
        let own_count = self.document.as_ref().map_or(0, |d| d.word_count());
        let children_count: usize = self.children.iter()
            .map(|c| c.total_word_count())
            .sum();
        own_count + children_count
    }

    /// Move a direct or nested child up in its parent's children list
    pub fn move_child_up(&mut self, id: &Uuid) -> bool {
        // Check direct children
        if let Some(pos) = self.children.iter().position(|c| &c.id == id) {
            if pos > 0 {
                self.children.swap(pos, pos - 1);
                return true;
            }
            return false;
        }
        // Check nested children
        for child in &mut self.children {
            if child.move_child_up(id) {
                return true;
            }
        }
        false
    }

    /// Move a direct or nested child down in its parent's children list
    pub fn move_child_down(&mut self, id: &Uuid) -> bool {
        // Check direct children
        if let Some(pos) = self.children.iter().position(|c| &c.id == id) {
            if pos < self.children.len() - 1 {
                self.children.swap(pos, pos + 1);
                return true;
            }
            return false;
        }
        // Check nested children
        for child in &mut self.children {
            if child.move_child_down(id) {
                return true;
            }
        }
        false
    }

    /// Duplicate a child item (deep clone with new IDs), insert after original
    pub fn duplicate_child(&mut self, id: &Uuid) -> Option<Uuid> {
        // Check direct children
        if let Some(pos) = self.children.iter().position(|c| &c.id == id) {
            let mut clone = self.children[pos].deep_clone();
            let new_id = clone.id;
            clone.title = format!("{} (Copy)", clone.title);
            self.children.insert(pos + 1, clone);
            return Some(new_id);
        }
        // Check nested children
        for child in &mut self.children {
            if let Some(new_id) = child.duplicate_child(id) {
                return Some(new_id);
            }
        }
        None
    }

    /// Deep clone: clone item with new UUIDs for self and all children
    pub fn deep_clone(&self) -> Self {
        let mut item = self.clone();
        item.id = Uuid::new_v4();
        for child in &mut item.children {
            *child = child.deep_clone();
        }
        item
    }

    /// Merge all children's content into a single string
    pub fn merge_children_content(&self) -> String {
        let mut merged = String::new();
        for child in &self.children {
            if let Some(ref doc) = child.document {
                if !merged.is_empty() {
                    merged.push_str("\n\n---\n\n");
                }
                merged.push_str("## ");
                merged.push_str(&child.title);
                merged.push_str("\n\n");
                merged.push_str(&doc.content);
            }
        }
        merged
    }
}

impl Binder {
    /// Count total characters across all documents
    pub fn total_char_count(&self) -> usize {
        let mut total = 0;
        self.for_each_item(|i| {
            if let Some(d) = &i.document {
                total += d.char_count();
            }
        });
        total
    }
}

impl BinderItem {
    /// Check if this item has snapshots
    pub fn has_snapshots(&self) -> bool {
        !self.snapshots.is_empty()
    }

    /// Get the number of snapshots
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }
}

/// The type of a binder item
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinderItemKind {
    Folder,
    Text,
    Image,
    Pdf,
    WebPage,
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binder_default_structure() {
        let binder = Binder::default_structure();
        assert_eq!(binder.draft.title, "Draft");
        assert_eq!(binder.research.title, "Research");
        assert_eq!(binder.trash.title, "Trash");
    }

    #[test]
    fn test_new_text_item() {
        let item = BinderItem::new_text("Chapter 1");
        assert_eq!(item.title, "Chapter 1");
        assert!(matches!(item.kind, BinderItemKind::Text));
        assert!(item.document.is_some());
        assert!(item.children.is_empty());
    }

    #[test]
    fn test_new_folder_item() {
        let item = BinderItem::new_folder("Part I");
        assert_eq!(item.title, "Part I");
        assert!(matches!(item.kind, BinderItemKind::Folder));
        assert!(item.document.is_none());
    }

    #[test]
    fn test_add_and_find_child() {
        let mut binder = Binder::default_structure();
        let item = BinderItem::new_text("Scene 1");
        let id = item.id;
        binder.draft.add_child(item);

        assert!(binder.find_item(&id).is_some());
        assert_eq!(binder.find_item(&id).unwrap().title, "Scene 1");
    }

    #[test]
    fn test_move_to_trash() {
        let mut binder = Binder::default_structure();
        let item = BinderItem::new_text("Delete Me");
        let id = item.id;
        binder.draft.add_child(item);

        assert!(binder.move_to_trash(&id));
        // Should now be in trash, not in draft
        assert!(binder.draft.find(&id).is_none());
        assert!(binder.trash.find(&id).is_some());
    }

    #[test]
    fn test_all_items() {
        let mut binder = Binder::default_structure();
        binder.draft.add_child(BinderItem::new_text("A"));
        binder.draft.add_child(BinderItem::new_text("B"));
        binder.research.add_child(BinderItem::new_text("Ref"));

        let all = binder.all_items();
        // Draft + A + B + Research + Ref + Trash
        assert!(all.len() >= 6);
    }

    #[test]
    fn test_item_word_count() {
        let mut item = BinderItem::new_text("Test");
        if let Some(ref mut doc) = item.document {
            doc.content = "one two three four five".to_string();
        }
        assert_eq!(item.total_word_count(), 5);
    }

    #[test]
    fn test_nested_structure() {
        let mut binder = Binder::default_structure();
        let mut folder = BinderItem::new_folder("Chapter 1");
        folder.add_child(BinderItem::new_text("Scene 1"));
        folder.add_child(BinderItem::new_text("Scene 2"));
        binder.draft.add_child(folder);

        let all = binder.all_items();
        // Draft, Chapter1, Scene1, Scene2, Research, Trash
        assert!(all.len() >= 6);
    }

    #[test]
    fn test_binder_item_snapshots() {
        let item = BinderItem::new_text("Test");
        assert!(!item.has_snapshots());
        assert_eq!(item.snapshot_count(), 0);
    }

    #[test]
    fn test_remove_child() {
        let mut folder = BinderItem::new_folder("Parent");
        let child = BinderItem::new_text("Child");
        let id = child.id;
        folder.add_child(child);
        assert_eq!(folder.children.len(), 1);

        let removed = folder.remove_child(&id);
        assert!(removed.is_some());
        assert_eq!(folder.children.len(), 0);
    }

    #[test]
    fn test_include_in_compile_default() {
        let item = BinderItem::new_text("Test");
        assert!(item.include_in_compile);
    }

    #[test]
    fn test_move_child_up_already_first() {
        let mut binder = Binder::default_structure();
        let a = BinderItem::new_text("A");
        let a_id = a.id;
        binder.draft.add_child(a);
        binder.draft.add_child(BinderItem::new_text("B"));

        // A is already first — move_up should return false
        assert!(!binder.move_item_up(&a_id));
        assert_eq!(binder.draft.children[0].id, a_id);
    }

    #[test]
    fn test_move_child_down_already_last() {
        let mut binder = Binder::default_structure();
        binder.draft.add_child(BinderItem::new_text("A"));
        let b = BinderItem::new_text("B");
        let b_id = b.id;
        binder.draft.add_child(b);

        // B is already last — move_down should return false
        assert!(!binder.move_item_down(&b_id));
        assert_eq!(binder.draft.children[1].id, b_id);
    }

    #[test]
    fn test_move_item_up_down_success() {
        let mut binder = Binder::default_structure();
        let a = BinderItem::new_text("A");
        let b = BinderItem::new_text("B");
        let a_id = a.id;
        let b_id = b.id;
        binder.draft.add_child(a);
        binder.draft.add_child(b);

        // Move B up
        assert!(binder.move_item_up(&b_id));
        assert_eq!(binder.draft.children[0].id, b_id);
        assert_eq!(binder.draft.children[1].id, a_id);

        // Move B back down
        assert!(binder.move_item_down(&b_id));
        assert_eq!(binder.draft.children[0].id, a_id);
        assert_eq!(binder.draft.children[1].id, b_id);
    }

    #[test]
    fn test_duplicate_item() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Original");
        if let Some(ref mut doc) = item.document {
            doc.content = "Some content.".to_string();
        }
        let orig_id = item.id;
        binder.draft.add_child(item);

        let dup_id = binder.duplicate_item(&orig_id).unwrap();
        assert_ne!(dup_id, orig_id);
        let dup = binder.find_item(&dup_id).unwrap();
        assert!(dup.title.contains("(Copy)"));
        assert_eq!(binder.draft.children.len(), 2);
    }

    #[test]
    fn test_deep_clone_new_ids() {
        let mut folder = BinderItem::new_folder("Parent");
        let child = BinderItem::new_text("Child");
        let child_id = child.id;
        folder.add_child(child);

        let cloned = folder.deep_clone();
        assert_ne!(cloned.id, folder.id);
        assert_ne!(cloned.children[0].id, child_id);
        assert_eq!(cloned.title, folder.title);
        assert_eq!(cloned.children[0].title, "Child");
    }

    #[test]
    fn test_convert_to_folder() {
        let mut binder = Binder::default_structure();
        let item = BinderItem::new_text("Scene");
        let id = item.id;
        binder.draft.add_child(item);

        assert!(binder.convert_to_folder(&id));
        let converted = binder.find_item(&id).unwrap();
        assert_eq!(converted.kind, BinderItemKind::Folder);
        assert!(converted.expanded);
    }

    #[test]
    fn test_convert_to_folder_already_folder() {
        let mut binder = Binder::default_structure();
        let folder = BinderItem::new_folder("Chapter");
        let id = folder.id;
        binder.draft.add_child(folder);

        // Already a folder — should return false
        assert!(!binder.convert_to_folder(&id));
    }

    #[test]
    fn test_convert_to_text_empty_folder() {
        let mut binder = Binder::default_structure();
        let folder = BinderItem::new_folder("Empty");
        let id = folder.id;
        binder.draft.add_child(folder);

        assert!(binder.convert_to_text(&id));
        let converted = binder.find_item(&id).unwrap();
        assert_eq!(converted.kind, BinderItemKind::Text);
        assert!(converted.document.is_some());
    }

    #[test]
    fn test_convert_to_text_nonempty_folder() {
        let mut binder = Binder::default_structure();
        let mut folder = BinderItem::new_folder("Chapter");
        folder.add_child(BinderItem::new_text("Scene 1"));
        let id = folder.id;
        binder.draft.add_child(folder);

        // Folder with children — should return false
        assert!(!binder.convert_to_text(&id));
    }

    #[test]
    fn test_empty_trash() {
        let mut binder = Binder::default_structure();
        let item = BinderItem::new_text("Trash me");
        let id = item.id;
        binder.draft.add_child(item);
        binder.move_to_trash(&id);
        assert!(!binder.trash.children.is_empty());

        binder.empty_trash();
        assert!(binder.trash.children.is_empty());
        assert_eq!(binder.trash.children.len(), 0);
    }

    #[test]
    fn test_move_to_trash_from_research() {
        let mut binder = Binder::default_structure();
        let item = BinderItem::new_text("Ref doc");
        let id = item.id;
        binder.research.add_child(item);

        assert!(binder.move_to_trash(&id));
        assert!(binder.research.find(&id).is_none());
        assert!(binder.trash.find(&id).is_some());
    }

    #[test]
    fn test_move_to_trash_nonexistent() {
        let mut binder = Binder::default_structure();
        let fake_id = Uuid::new_v4();
        assert!(!binder.move_to_trash(&fake_id));
    }

    #[test]
    fn test_merge_children_content() {
        let mut folder = BinderItem::new_folder("Chapter");
        let mut s1 = BinderItem::new_text("Scene 1");
        if let Some(ref mut doc) = s1.document {
            doc.content = "Hello".to_string();
        }
        let mut s2 = BinderItem::new_text("Scene 2");
        if let Some(ref mut doc) = s2.document {
            doc.content = "World".to_string();
        }
        folder.add_child(s1);
        folder.add_child(s2);

        let merged = folder.merge_children_content();
        assert!(merged.contains("Hello"));
        assert!(merged.contains("World"));
        assert!(merged.contains("Scene 1"));
        assert!(merged.contains("Scene 2"));
    }

    #[test]
    fn test_total_word_count_nested() {
        let mut folder = BinderItem::new_folder("Root");
        let mut a = BinderItem::new_text("A");
        if let Some(ref mut doc) = a.document {
            doc.content = "one two three".to_string();
        }
        let mut sub = BinderItem::new_folder("Sub");
        let mut b = BinderItem::new_text("B");
        if let Some(ref mut doc) = b.document {
            doc.content = "four five".to_string();
        }
        sub.add_child(b);
        folder.add_child(a);
        folder.add_child(sub);

        assert_eq!(folder.total_word_count(), 5);
    }

    #[test]
    fn test_all_text() {
        let mut binder = Binder::default_structure();
        let mut a = BinderItem::new_text("A");
        if let Some(ref mut doc) = a.document {
            doc.content = "First doc".to_string();
        }
        let mut b = BinderItem::new_text("B");
        if let Some(ref mut doc) = b.document {
            doc.content = "Second doc".to_string();
        }
        binder.draft.add_child(a);
        binder.draft.add_child(b);

        let text = binder.all_text();
        assert!(text.contains("First doc"));
        assert!(text.contains("Second doc"));
    }

    #[test]
    fn test_insert_child_at_position() {
        let mut folder = BinderItem::new_folder("Parent");
        folder.add_child(BinderItem::new_text("A"));
        folder.add_child(BinderItem::new_text("C"));

        let b = BinderItem::new_text("B");
        let b_id = b.id;
        folder.insert_child(1, b);

        assert_eq!(folder.children.len(), 3);
        assert_eq!(folder.children[1].id, b_id);
        assert_eq!(folder.children[1].title, "B");
    }

    #[test]
    fn test_insert_child_out_of_bounds() {
        let mut folder = BinderItem::new_folder("Parent");
        folder.add_child(BinderItem::new_text("A"));

        let b = BinderItem::new_text("B");
        let b_id = b.id;
        folder.insert_child(999, b); // Way out of bounds

        // Should clamp to end
        assert_eq!(folder.children.len(), 2);
        assert_eq!(folder.children[1].id, b_id);
    }

    #[test]
    fn test_total_word_count_binder() {
        let mut binder = Binder::default_structure();
        let mut a = BinderItem::new_text("A");
        if let Some(ref mut doc) = a.document {
            doc.content = "one two three".to_string();
        }
        let mut b = BinderItem::new_text("B");
        if let Some(ref mut doc) = b.document {
            doc.content = "four five".to_string();
        }
        binder.draft.add_child(a);
        binder.draft.add_child(b);

        assert_eq!(binder.total_word_count(), 5);
    }

    #[test]
    fn test_find_item_mut() {
        let mut binder = Binder::default_structure();
        let item = BinderItem::new_text("Editable");
        let id = item.id;
        binder.draft.add_child(item);

        let found = binder.find_item_mut(&id).unwrap();
        found.title = "Modified".to_string();

        assert_eq!(binder.find_item(&id).unwrap().title, "Modified");
    }

    #[test]
    fn test_find_item_not_found() {
        let binder = Binder::default_structure();
        let fake_id = Uuid::new_v4();
        assert!(binder.find_item(&fake_id).is_none());
    }

    #[test]
    fn test_remove_child_not_found() {
        let mut folder = BinderItem::new_folder("Parent");
        folder.add_child(BinderItem::new_text("A"));

        let fake_id = Uuid::new_v4();
        assert!(folder.remove_child(&fake_id).is_none());
        assert_eq!(folder.children.len(), 1);
    }

    #[test]
    fn test_remove_nested_child() {
        let mut root = BinderItem::new_folder("Root");
        let mut sub = BinderItem::new_folder("Sub");
        let leaf = BinderItem::new_text("Leaf");
        let leaf_id = leaf.id;
        sub.add_child(leaf);
        root.add_child(sub);

        let removed = root.remove_child(&leaf_id);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().title, "Leaf");
        assert_eq!(root.children[0].children.len(), 0);
    }

    #[test]
    fn test_deep_clone_preserves_structure() {
        let mut root = BinderItem::new_folder("Root");
        let mut sub = BinderItem::new_folder("Sub");
        sub.add_child(BinderItem::new_text("Leaf A"));
        sub.add_child(BinderItem::new_text("Leaf B"));
        root.add_child(sub);

        let cloned = root.deep_clone();
        assert_ne!(cloned.id, root.id);
        assert_eq!(cloned.children.len(), 1);
        assert_eq!(cloned.children[0].children.len(), 2);
        assert_ne!(cloned.children[0].id, root.children[0].id);
    }

    #[test]
    fn test_merge_children_content_empty() {
        let folder = BinderItem::new_folder("Empty");
        assert!(folder.merge_children_content().is_empty());
    }

    #[test]
    fn test_total_char_count() {
        let mut binder = Binder::default_structure();
        let mut a = BinderItem::new_text("A");
        if let Some(ref mut doc) = a.document {
            doc.content = "Hello".to_string();
        }
        binder.draft.add_child(a);
        assert!(binder.total_char_count() > 0);
    }

    #[test]
    fn test_duplicate_item_not_found() {
        let mut binder = Binder::default_structure();
        let fake_id = Uuid::new_v4();
        assert!(binder.duplicate_item(&fake_id).is_none());
    }

    #[test]
    fn test_move_child_up_nested() {
        let mut binder = Binder::default_structure();
        let mut folder = BinderItem::new_folder("Ch");
        let a = BinderItem::new_text("A");
        let b = BinderItem::new_text("B");
        let b_id = b.id;
        folder.add_child(a);
        folder.add_child(b);
        binder.draft.add_child(folder);

        assert!(binder.move_item_up(&b_id));
    }

    #[test]
    fn test_move_child_down_nested() {
        let mut binder = Binder::default_structure();
        let mut folder = BinderItem::new_folder("Ch");
        let a = BinderItem::new_text("A");
        let b = BinderItem::new_text("B");
        let a_id = a.id;
        folder.add_child(a);
        folder.add_child(b);
        binder.draft.add_child(folder);

        assert!(binder.move_item_down(&a_id));
    }

    #[test]
    fn test_all_text_empty() {
        let binder = Binder::default_structure();
        assert!(binder.all_text().is_empty());
    }

    #[test]
    fn test_find_item_in_trash() {
        let mut binder = Binder::default_structure();
        let item = BinderItem::new_text("Trashed");
        let id = item.id;
        binder.trash.add_child(item);
        assert!(binder.find_item(&id).is_some());
    }

    #[test]
    fn test_find_item_mut_in_research() {
        let mut binder = Binder::default_structure();
        let item = BinderItem::new_text("Research item");
        let id = item.id;
        binder.research.add_child(item);
        let found = binder.find_item_mut(&id).unwrap();
        found.title = "Renamed".to_string();
        assert_eq!(binder.find_item(&id).unwrap().title, "Renamed");
    }

}
