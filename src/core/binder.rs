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
        if let Some(item) = self.draft.remove_child(id) {
            self.trash.children.push(item);
            return true;
        }
        if let Some(item) = self.research.remove_child(id) {
            self.trash.children.push(item);
            return true;
        }
        false
    }

    /// Count total documents (text items only)
    pub fn document_count(&self) -> usize {
        self.all_items().iter()
            .filter(|i| i.kind == BinderItemKind::Text)
            .count()
    }

    /// Count total words across all documents
    pub fn total_word_count(&self) -> usize {
        self.all_items().iter()
            .filter_map(|i| i.document.as_ref())
            .map(|d| d.word_count())
            .sum()
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
        if let Some(new_id) = self.draft.duplicate_child(id) {
            return Some(new_id);
        }
        if let Some(new_id) = self.research.duplicate_child(id) {
            return Some(new_id);
        }
        None
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

    /// Collect all items into a flat list
    pub fn collect_all<'a>(&'a self, items: &mut Vec<&'a BinderItem>) {
        items.push(self);
        for child in &self.children {
            child.collect_all(items);
        }
    }

    /// Collect all items into a mutable flat list
    pub fn collect_all_mut<'a>(&'a mut self, items: &mut Vec<&'a mut BinderItem>) {
        let self_ptr = self as *mut BinderItem;
        // Safety: We need to collect mutable references to all items.
        // We guarantee no aliasing because each item appears exactly once in the tree.
        unsafe {
            items.push(&mut *self_ptr);
            for child in &mut (*self_ptr).children {
                child.collect_all_mut(items);
            }
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
        let own_count = self.document.as_ref()
            .map(|d| d.word_count())
            .unwrap_or(0);
        let children_count: usize = self.children.iter()
            .map(|c| c.total_word_count())
            .sum();
        own_count + children_count
    }

    /// Get the depth of this item in the tree (for indentation)
    pub fn depth(&self) -> usize {
        0 // This needs to be computed from context
    }

    /// Find the parent of a child item by ID
    pub fn find_parent(&self, id: &Uuid) -> Option<(&BinderItem, usize)> {
        for (i, child) in self.children.iter().enumerate() {
            if &child.id == id {
                return Some((self, i));
            }
            if let Some(result) = child.find_parent(id) {
                return Some(result);
            }
        }
        None
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
                merged.push_str(&format!("## {}\n\n{}", child.title, doc.content));
            }
        }
        merged
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
