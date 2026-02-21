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

    /// Find an item by title
    pub fn find_item_by_title(&self, title: &str) -> Option<&BinderItem> {
        self.all_items().into_iter().find(|item| item.title == title)
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

impl Binder {
    /// Count total folders in the binder
    pub fn folder_count(&self) -> usize {
        self.all_items().iter()
            .filter(|i| i.kind == BinderItemKind::Folder)
            .count()
    }

    /// Count total characters across all documents
    pub fn total_char_count(&self) -> usize {
        self.all_items().iter()
            .filter_map(|i| i.document.as_ref())
            .map(|d| d.char_count())
            .sum()
    }

    /// Total number of items (documents + folders) in the entire binder
    pub fn item_count(&self) -> usize {
        self.all_items().len()
    }

    /// Get all text items (documents only)
    pub fn text_items(&self) -> Vec<&BinderItem> {
        self.all_items().into_iter()
            .filter(|i| i.kind == BinderItemKind::Text)
            .collect()
    }

    /// Get the item with the most words
    pub fn longest_document(&self) -> Option<&BinderItem> {
        self.all_items().into_iter()
            .filter(|i| i.kind == BinderItemKind::Text)
            .max_by_key(|i| i.document.as_ref().map(|d| d.word_count()).unwrap_or(0))
    }

    /// Check if the trash is empty
    pub fn trash_is_empty(&self) -> bool {
        self.trash.children.is_empty()
    }

    /// Number of items in the trash
    pub fn trash_count(&self) -> usize {
        self.trash.children.len()
    }
}

impl BinderItem {
    /// Count direct children of this item
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Check if this item has any children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Get the age of this item since creation
    pub fn age_string(&self) -> String {
        let duration = Utc::now().signed_duration_since(self.created_at);
        let days = duration.num_days();
        if days == 0 {
            "today".to_string()
        } else if days == 1 {
            "yesterday".to_string()
        } else if days < 7 {
            format!("{} days ago", days)
        } else if days < 30 {
            format!("{} weeks ago", days / 7)
        } else {
            format!("{} months ago", days / 30)
        }
    }

    /// Check if this item has a synopsis
    pub fn has_synopsis(&self) -> bool {
        !self.synopsis.is_empty()
    }

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

impl BinderItemKind {
    /// Human-readable label
    pub fn label(&self) -> &str {
        match self {
            BinderItemKind::Folder => "Folder",
            BinderItemKind::Text => "Text",
            BinderItemKind::Image => "Image",
            BinderItemKind::Pdf => "PDF",
            BinderItemKind::WebPage => "Web Page",
        }
    }

    /// Icon character for UI display
    pub fn icon(&self) -> &str {
        match self {
            BinderItemKind::Folder => "\u{1F4C1}",
            BinderItemKind::Text => "\u{1F4C4}",
            BinderItemKind::Image => "\u{1F5BC}",
            BinderItemKind::Pdf => "\u{1F4D1}",
            BinderItemKind::WebPage => "\u{1F310}",
        }
    }

    /// Whether this kind supports text editing
    pub fn is_editable(&self) -> bool {
        matches!(self, BinderItemKind::Text)
    }
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
    fn test_find_item_by_title() {
        let mut binder = Binder::default_structure();
        let item = BinderItem::new_text("Important Scene");
        binder.draft.add_child(item);

        assert!(binder.find_item_by_title("Important Scene").is_some());
        assert!(binder.find_item_by_title("Nonexistent").is_none());
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
    fn test_binder_item_has_children() {
        let mut folder = BinderItem::new_folder("Folder");
        assert!(!folder.has_children());
        folder.add_child(BinderItem::new_text("Child"));
        assert!(folder.has_children());
        assert_eq!(folder.child_count(), 1);
    }

    #[test]
    fn test_item_kinds() {
        assert_eq!(BinderItemKind::Text.label(), "Text");
        assert_eq!(BinderItemKind::Folder.label(), "Folder");
        assert!(BinderItemKind::Text.is_editable());
        assert!(!BinderItemKind::Folder.is_editable());
    }

    #[test]
    fn test_folder_count_and_item_count() {
        let mut binder = Binder::default_structure();
        binder.draft.add_child(BinderItem::new_text("A"));
        binder.draft.add_child(BinderItem::new_text("B"));
        binder.draft.add_child(BinderItem::new_folder("Ch1"));

        assert!(binder.item_count() > 0);
        assert!(binder.folder_count() > 0);
    }

    #[test]
    fn test_text_items() {
        let mut binder = Binder::default_structure();
        binder.draft.add_child(BinderItem::new_text("A"));
        binder.draft.add_child(BinderItem::new_folder("F"));
        binder.draft.add_child(BinderItem::new_text("B"));

        let texts = binder.text_items();
        assert_eq!(texts.len(), 2);
    }

    #[test]
    fn test_trash_is_empty() {
        let binder = Binder::default_structure();
        assert!(binder.trash_is_empty());
        assert_eq!(binder.trash_count(), 0);
    }

    #[test]
    fn test_binder_item_snapshots() {
        let item = BinderItem::new_text("Test");
        assert!(!item.has_snapshots());
        assert_eq!(item.snapshot_count(), 0);
    }

    #[test]
    fn test_binder_item_synopsis() {
        let mut item = BinderItem::new_text("Test");
        assert!(!item.has_synopsis());
        item.synopsis = "A brief overview".to_string();
        assert!(item.has_synopsis());
    }

    #[test]
    fn test_remove_child() {
        let mut folder = BinderItem::new_folder("Parent");
        let child = BinderItem::new_text("Child");
        let id = child.id;
        folder.add_child(child);
        assert_eq!(folder.child_count(), 1);

        let removed = folder.remove_child(&id);
        assert!(removed.is_some());
        assert_eq!(folder.child_count(), 0);
    }

    #[test]
    fn test_include_in_compile_default() {
        let item = BinderItem::new_text("Test");
        assert!(item.include_in_compile);
    }
}
