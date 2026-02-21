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
