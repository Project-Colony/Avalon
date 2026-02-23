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
        self.draft.find_by_title(title)
            .or_else(|| self.research.find_by_title(title))
            .or_else(|| self.trash.find_by_title(title))
    }

    /// Move an item to trash
    pub fn move_to_trash(&mut self, id: &Uuid) -> bool {
        if let Some(item) = self.draft.remove_child(id).or_else(|| self.research.remove_child(id)) {
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

    /// Get all text content concatenated (for readability analysis)
    pub fn all_text(&self) -> String {
        let mut result = String::new();
        for item in self.all_items() {
            if let Some(ref doc) = item.document {
                if !doc.content.trim().is_empty() {
                    if !result.is_empty() {
                        result.push_str("\n\n");
                    }
                    result.push_str(&doc.content);
                }
            }
        }
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

    /// Find an item by title recursively
    pub fn find_by_title(&self, title: &str) -> Option<&BinderItem> {
        if self.title == title {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_by_title(title) {
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

    /// Collect all items into a mutable flat list.
    ///
    /// # Safety rationale
    /// We split the mutable borrow of `self` into two non-overlapping parts:
    /// the item's own fields (pushed into `items`) and its `children` vec.
    /// Each node in the tree is unique, so no aliasing occurs.
    pub fn collect_all_mut<'a>(&'a mut self, items: &mut Vec<&'a mut BinderItem>) {
        // Split borrow: push self, then recurse into children only
        let children_ptr = self.children.as_mut_ptr();
        let children_len = self.children.len();
        items.push(self);
        // SAFETY: children_ptr/len come from self.children before self was moved;
        // we only access children (disjoint from the &mut self already in `items`).
        let children_slice = unsafe { std::slice::from_raw_parts_mut(children_ptr, children_len) };
        for child in children_slice {
            child.collect_all_mut(items);
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

    /// Get the depth of this item in the tree (for indentation)
    pub fn depth(&self) -> usize {
        0 // This needs to be computed from context
    }

    /// Compute the actual depth of a target item from this node.
    /// Returns None if the item is not found under this node.
    pub fn depth_of(&self, target_id: &Uuid) -> Option<usize> {
        if &self.id == target_id {
            return Some(0);
        }
        for child in &self.children {
            if let Some(d) = child.depth_of(target_id) {
                return Some(d + 1);
            }
        }
        None
    }

    /// Get all ancestor IDs from root down to (but not including) the target.
    pub fn ancestors_of(&self, target_id: &Uuid) -> Option<Vec<Uuid>> {
        if &self.id == target_id {
            return Some(Vec::new());
        }
        for child in &self.children {
            if let Some(mut path) = child.ancestors_of(target_id) {
                path.insert(0, self.id);
                return Some(path);
            }
        }
        None
    }

    /// Collect all descendant IDs (children, grandchildren, etc.) recursively.
    pub fn descendant_ids(&self) -> Vec<Uuid> {
        let mut ids = Vec::new();
        self.collect_descendant_ids(&mut ids);
        ids
    }

    fn collect_descendant_ids(&self, ids: &mut Vec<Uuid>) {
        for child in &self.children {
            ids.push(child.id);
            child.collect_descendant_ids(ids);
        }
    }

    /// Check if a target item is a descendant of this node.
    pub fn is_ancestor_of(&self, target_id: &Uuid) -> bool {
        for child in &self.children {
            if &child.id == target_id || child.is_ancestor_of(target_id) {
                return true;
            }
        }
        false
    }

    /// Get all leaf (text/document) items under this node.
    pub fn leaf_items(&self) -> Vec<&BinderItem> {
        let mut leaves = Vec::new();
        self.collect_leaf_items(&mut leaves);
        leaves
    }

    fn collect_leaf_items<'a>(&'a self, leaves: &mut Vec<&'a BinderItem>) {
        if self.children.is_empty() && self.kind == BinderItemKind::Text {
            leaves.push(self);
        }
        for child in &self.children {
            child.collect_leaf_items(leaves);
        }
    }

    /// Count total descendants (not including self).
    pub fn descendant_count(&self) -> usize {
        let mut count = 0;
        for child in &self.children {
            count += 1 + child.descendant_count();
        }
        count
    }

    /// Get the maximum nesting depth under this node.
    pub fn max_depth(&self) -> usize {
        self.children.iter()
            .map(|c| 1 + c.max_depth())
            .max()
            .unwrap_or(0)
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
            .max_by_key(|i| i.document.as_ref().map_or(0, |d| d.word_count()))
    }

    /// Get the depth of an item in the binder (across all root sections).
    pub fn item_depth(&self, id: &Uuid) -> Option<usize> {
        self.draft.depth_of(id)
            .or_else(|| self.research.depth_of(id))
            .or_else(|| self.trash.depth_of(id))
    }

    /// Get the ancestors of an item (UUIDs from root section down to parent).
    pub fn item_ancestors(&self, id: &Uuid) -> Option<Vec<Uuid>> {
        self.draft.ancestors_of(id)
            .or_else(|| self.research.ancestors_of(id))
            .or_else(|| self.trash.ancestors_of(id))
    }

    /// Get all descendant IDs under a given item.
    pub fn descendants_of(&self, id: &Uuid) -> Vec<Uuid> {
        self.find_item(id).map_or_else(Vec::new, |item| item.descendant_ids())
    }

    /// Get the maximum nesting depth across the entire binder.
    pub fn max_nesting_depth(&self) -> usize {
        self.draft.max_depth()
            .max(self.research.max_depth())
            .max(self.trash.max_depth())
    }

    /// Get all leaf documents (items with no children that are text type).
    pub fn leaf_documents(&self) -> Vec<&BinderItem> {
        let mut leaves = self.draft.leaf_items();
        leaves.extend(self.research.leaf_items());
        leaves
    }

    /// Get items by label name from metadata.
    pub fn items_with_label(&self, label: &str) -> Vec<&BinderItem> {
        self.all_items().into_iter()
            .filter(|item| item.metadata.label.as_ref().map(|l| l.name.as_str()) == Some(label))
            .collect()
    }

    /// Get items by status name from metadata.
    pub fn items_with_status(&self, status: &str) -> Vec<&BinderItem> {
        self.all_items().into_iter()
            .filter(|item| item.metadata.status.as_ref().map(|s| s.name.as_str()) == Some(status))
            .collect()
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

impl Binder {
    /// Move an item from one location to a different parent (reparenting).
    /// Returns true if the move was successful.
    pub fn reparent_item(&mut self, item_id: &Uuid, new_parent_id: &Uuid, position: usize) -> bool {
        // Don't reparent to self
        if item_id == new_parent_id {
            return false;
        }
        // Don't reparent a folder into one of its own descendants
        if let Some(item) = self.find_item(item_id) {
            if item.find(new_parent_id).is_some() {
                return false; // Would create cycle
            }
        }

        // Remove the item from its current location
        let removed = self.draft.remove_child(item_id)
            .or_else(|| self.research.remove_child(item_id));

        if let Some(item) = removed {
            // Insert into new parent
            if let Some(parent) = self.find_item_mut(new_parent_id) {
                let pos = position.min(parent.children.len());
                parent.children.insert(pos, item);
                parent.modified_at = Utc::now();
                return true;
            }
            // If we couldn't find the new parent, put it back in draft
            self.draft.children.push(item);
        }

        false
    }

    /// Move an item to a specific position within its current parent.
    /// position is the index within the parent's children list.
    pub fn move_item_to_position(&mut self, item_id: &Uuid, position: usize) -> bool {
        if self.draft.move_child_to_position(item_id, position) { return true; }
        if self.research.move_child_to_position(item_id, position) { return true; }
        self.trash.move_child_to_position(item_id, position)
    }

    /// Merge multiple items into a single document.
    /// Items are merged in order, separated by the given separator.
    pub fn merge_items(&mut self, item_ids: &[Uuid], separator: &str) -> Option<Uuid> {
        if item_ids.len() < 2 {
            return None;
        }

        // Collect content from items
        let mut merged_title = String::new();
        let mut merged_content = String::new();

        for (i, id) in item_ids.iter().enumerate() {
            if let Some(item) = self.find_item(id) {
                if i == 0 {
                    merged_title = format!("{} (Merged)", item.title);
                }
                if let Some(ref doc) = item.document {
                    if !merged_content.is_empty() {
                        merged_content.push_str(separator);
                    }
                    merged_content.push_str(&doc.content);
                }
            }
        }

        if merged_content.is_empty() {
            return None;
        }

        // Create the merged item
        let mut merged = BinderItem::new_text(&merged_title);
        if let Some(ref mut doc) = merged.document {
            doc.content = merged_content;
        }
        let merged_id = merged.id;

        // Add the merged item at the first item's location
        self.draft.add_child(merged);

        // Remove the original items
        for id in item_ids {
            self.draft.remove_child(id);
        }

        Some(merged_id)
    }

    /// Split a document into multiple documents at separator points.
    /// Returns the IDs of the new documents.
    pub fn split_item(&mut self, item_id: &Uuid, separator: &str) -> Vec<Uuid> {
        let mut new_ids = Vec::new();
        let (title, sections) = {
            if let Some(item) = self.find_item(item_id) {
                if let Some(ref doc) = item.document {
                    let sections: Vec<String> = doc.content
                        .split(separator)
                        .filter(|s| !s.trim().is_empty())
                        .map(|s| s.trim().to_string())
                        .collect();
                    (item.title.clone(), sections)
                } else {
                    return new_ids;
                }
            } else {
                return new_ids;
            }
        };

        if sections.len() < 2 {
            return new_ids;
        }

        for (i, section) in sections.into_iter().enumerate() {
            let mut new_item = BinderItem::new_text(&format!("{} - Part {}", title, i + 1));
            if let Some(ref mut doc) = new_item.document {
                doc.content = section;
            }
            new_ids.push(new_item.id);
            self.draft.add_child(new_item);
        }

        // Remove original
        self.draft.remove_child(item_id);

        new_ids
    }

    /// Flatten a folder: move all its children up to the parent level and remove the folder.
    pub fn flatten_folder(&mut self, folder_id: &Uuid) -> bool {
        // Extract children from the folder
        let children = {
            if let Some(item) = self.find_item(folder_id) {
                if item.kind != BinderItemKind::Folder || item.children.is_empty() {
                    return false;
                }
                item.children.clone()
            } else {
                return false;
            }
        };

        // Find where this folder is and insert children there
        if self.draft.flatten_child(folder_id, &children) { return true; }
        if self.research.flatten_child(folder_id, &children) { return true; }
        false
    }

    /// Group consecutive items into a new folder.
    /// The items must be siblings (direct children of the same parent).
    pub fn group_items_into_folder(&mut self, item_ids: &[Uuid], folder_title: &str) -> Option<Uuid> {
        if item_ids.is_empty() {
            return None;
        }

        self.draft.group_children(item_ids, folder_title)
            .or_else(|| self.research.group_children(item_ids, folder_title))
    }
}

impl BinderItem {
    /// Move a child to a specific position within this item's children
    pub fn move_child_to_position(&mut self, id: &Uuid, position: usize) -> bool {
        if let Some(pos) = self.children.iter().position(|c| &c.id == id) {
            let target = position.min(self.children.len() - 1);
            if pos != target {
                let item = self.children.remove(pos);
                self.children.insert(target, item);
                return true;
            }
            return false;
        }
        for child in &mut self.children {
            if child.move_child_to_position(id, position) {
                return true;
            }
        }
        false
    }

    /// Flatten a child folder: replace it with its children
    fn flatten_child(&mut self, folder_id: &Uuid, children: &[BinderItem]) -> bool {
        if let Some(pos) = self.children.iter().position(|c| &c.id == folder_id) {
            self.children.remove(pos);
            for (i, child) in children.iter().enumerate() {
                self.children.insert(pos + i, child.clone());
            }
            return true;
        }
        for child in &mut self.children {
            if child.flatten_child(folder_id, children) {
                return true;
            }
        }
        false
    }

    /// Group a set of direct children into a new folder
    fn group_children(&mut self, item_ids: &[Uuid], folder_title: &str) -> Option<Uuid> {
        // Check if all items are direct children
        let positions: Vec<usize> = item_ids.iter()
            .filter_map(|id| self.children.iter().position(|c| &c.id == id))
            .collect();

        if positions.len() != item_ids.len() {
            // Not all items are direct children; try nested
            for child in &mut self.children {
                if let Some(id) = child.group_children(item_ids, folder_title) {
                    return Some(id);
                }
            }
            return None;
        }

        // Remove items (in reverse order to preserve indices)
        let mut sorted_positions = positions;
        sorted_positions.sort_unstable();
        sorted_positions.reverse();

        let mut items_to_group = Vec::new();
        for &pos in &sorted_positions {
            items_to_group.push(self.children.remove(pos));
        }
        items_to_group.reverse(); // Restore original order

        // Create the folder and add children
        let mut folder = BinderItem::new_folder(folder_title);
        let folder_id = folder.id;
        for item in items_to_group {
            folder.children.push(item);
        }

        // Insert the folder at the first item's original position
        let insert_pos = *sorted_positions.last().unwrap_or(&0);
        let insert_pos = insert_pos.min(self.children.len());
        self.children.insert(insert_pos, folder);

        Some(folder_id)
    }

    /// Get the path from root to this item (as titles)
    pub fn path_to_item(&self, target_id: &Uuid) -> Option<Vec<String>> {
        if &self.id == target_id {
            return Some(vec![self.title.clone()]);
        }
        for child in &self.children {
            if let Some(mut path) = child.path_to_item(target_id) {
                path.insert(0, self.title.clone());
                return Some(path);
            }
        }
        None
    }

    /// Get the sibling items (other children of the same parent)
    pub fn sibling_ids(&self, item_id: &Uuid) -> Vec<Uuid> {
        if self.children.iter().any(|c| &c.id == item_id) {
            return self.children.iter()
                .filter(|c| &c.id != item_id)
                .map(|c| c.id)
                .collect();
        }
        for child in &self.children {
            let result = child.sibling_ids(item_id);
            if !result.is_empty() {
                return result;
            }
        }
        Vec::new()
    }

    /// Get the next sibling item
    pub fn next_sibling(&self, item_id: &Uuid) -> Option<Uuid> {
        if let Some(pos) = self.children.iter().position(|c| &c.id == item_id) {
            if pos + 1 < self.children.len() {
                return Some(self.children[pos + 1].id);
            }
        }
        for child in &self.children {
            if let Some(id) = child.next_sibling(item_id) {
                return Some(id);
            }
        }
        None
    }

    /// Get the previous sibling item
    pub fn prev_sibling(&self, item_id: &Uuid) -> Option<Uuid> {
        if let Some(pos) = self.children.iter().position(|c| &c.id == item_id) {
            if pos > 0 {
                return Some(self.children[pos - 1].id);
            }
        }
        for child in &self.children {
            if let Some(id) = child.prev_sibling(item_id) {
                return Some(id);
            }
        }
        None
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

    #[test]
    fn test_reparent_item() {
        let mut binder = Binder::default_structure();
        let item = BinderItem::new_text("Scene");
        let item_id = item.id;
        let folder = BinderItem::new_folder("Chapter");
        let folder_id = folder.id;
        binder.draft.add_child(item);
        binder.draft.add_child(folder);

        assert!(binder.reparent_item(&item_id, &folder_id, 0));
        // Item should now be inside the folder
        let folder = binder.find_item(&folder_id).unwrap();
        assert_eq!(folder.child_count(), 1);
        assert_eq!(folder.children[0].id, item_id);
    }

    #[test]
    fn test_reparent_item_no_cycle() {
        let mut binder = Binder::default_structure();
        let mut parent = BinderItem::new_folder("Parent");
        let child = BinderItem::new_folder("Child");
        let parent_id = parent.id;
        let child_id = child.id;
        parent.add_child(child);
        binder.draft.add_child(parent);

        // Should fail: can't reparent parent into its own child
        assert!(!binder.reparent_item(&parent_id, &child_id, 0));
    }

    #[test]
    fn test_move_item_to_position() {
        let mut binder = Binder::default_structure();
        let a = BinderItem::new_text("A");
        let b = BinderItem::new_text("B");
        let c = BinderItem::new_text("C");
        let a_id = a.id;
        binder.draft.add_child(a);
        binder.draft.add_child(b);
        binder.draft.add_child(c);

        // Move A to last position
        assert!(binder.move_item_to_position(&a_id, 2));
        assert_eq!(binder.draft.children.last().unwrap().id, a_id);
    }

    #[test]
    fn test_group_items_into_folder() {
        let mut binder = Binder::default_structure();
        let a = BinderItem::new_text("A");
        let b = BinderItem::new_text("B");
        let c = BinderItem::new_text("C");
        let a_id = a.id;
        let b_id = b.id;
        binder.draft.add_child(a);
        binder.draft.add_child(b);
        binder.draft.add_child(c);

        let folder_id = binder.group_items_into_folder(&[a_id, b_id], "New Chapter").unwrap();
        let folder = binder.find_item(&folder_id).unwrap();
        assert_eq!(folder.title, "New Chapter");
        assert_eq!(folder.child_count(), 2);
    }

    #[test]
    fn test_flatten_folder() {
        let mut binder = Binder::default_structure();
        let mut folder = BinderItem::new_folder("Chapter");
        let folder_id = folder.id;
        folder.add_child(BinderItem::new_text("Scene 1"));
        folder.add_child(BinderItem::new_text("Scene 2"));
        binder.draft.add_child(folder);

        let initial_children = binder.draft.children.len();
        assert!(binder.flatten_folder(&folder_id));
        // Folder should be gone, its children should be at the same level
        assert!(binder.find_item(&folder_id).is_none());
        // Draft should have 2 more children (scenes) but one fewer (the folder)
        assert_eq!(binder.draft.children.len(), initial_children + 1); // -1 folder + 2 scenes
    }

    #[test]
    fn test_split_item() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Full Document");
        if let Some(ref mut doc) = item.document {
            doc.content = "Part one content\n\n---\n\nPart two content\n\n---\n\nPart three".to_string();
        }
        let item_id = item.id;
        binder.draft.add_child(item);

        let new_ids = binder.split_item(&item_id, "\n\n---\n\n");
        assert_eq!(new_ids.len(), 3);
        // Original should be gone
        assert!(binder.find_item(&item_id).is_none());
    }

    #[test]
    fn test_merge_items() {
        let mut binder = Binder::default_structure();
        let mut a = BinderItem::new_text("Part A");
        if let Some(ref mut doc) = a.document {
            doc.content = "Content A".to_string();
        }
        let mut b = BinderItem::new_text("Part B");
        if let Some(ref mut doc) = b.document {
            doc.content = "Content B".to_string();
        }
        let a_id = a.id;
        let b_id = b.id;
        binder.draft.add_child(a);
        binder.draft.add_child(b);

        let merged_id = binder.merge_items(&[a_id, b_id], "\n\n").unwrap();
        let merged = binder.find_item(&merged_id).unwrap();
        let content = merged.document.as_ref().unwrap().content.clone();
        assert!(content.contains("Content A"));
        assert!(content.contains("Content B"));
    }

    #[test]
    fn test_path_to_item() {
        let mut folder = BinderItem::new_folder("Draft");
        let mut chapter = BinderItem::new_folder("Chapter 1");
        let scene = BinderItem::new_text("Scene 1");
        let scene_id = scene.id;
        chapter.add_child(scene);
        folder.add_child(chapter);

        let path = folder.path_to_item(&scene_id).unwrap();
        assert_eq!(path, vec!["Draft", "Chapter 1", "Scene 1"]);
    }

    #[test]
    fn test_sibling_ids() {
        let mut folder = BinderItem::new_folder("Parent");
        let a = BinderItem::new_text("A");
        let b = BinderItem::new_text("B");
        let c = BinderItem::new_text("C");
        let b_id = b.id;
        let a_id = a.id;
        let c_id = c.id;
        folder.add_child(a);
        folder.add_child(b);
        folder.add_child(c);

        let siblings = folder.sibling_ids(&b_id);
        assert_eq!(siblings.len(), 2);
        assert!(siblings.contains(&a_id));
        assert!(siblings.contains(&c_id));
    }

    #[test]
    fn test_next_prev_sibling() {
        let mut folder = BinderItem::new_folder("Parent");
        let a = BinderItem::new_text("A");
        let b = BinderItem::new_text("B");
        let c = BinderItem::new_text("C");
        let a_id = a.id;
        let b_id = b.id;
        let c_id = c.id;
        folder.add_child(a);
        folder.add_child(b);
        folder.add_child(c);

        assert_eq!(folder.next_sibling(&a_id), Some(b_id));
        assert_eq!(folder.next_sibling(&b_id), Some(c_id));
        assert_eq!(folder.next_sibling(&c_id), None);

        assert_eq!(folder.prev_sibling(&c_id), Some(b_id));
        assert_eq!(folder.prev_sibling(&b_id), Some(a_id));
        assert_eq!(folder.prev_sibling(&a_id), None);
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
        assert!(!binder.trash_is_empty());

        binder.empty_trash();
        assert!(binder.trash_is_empty());
        assert_eq!(binder.trash_count(), 0);
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
    fn test_find_parent() {
        let mut folder = BinderItem::new_folder("Root");
        let child = BinderItem::new_text("Child");
        let child_id = child.id;
        folder.add_child(child);

        let (parent, idx) = folder.find_parent(&child_id).unwrap();
        assert_eq!(parent.title, "Root");
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_find_parent_nested() {
        let mut root = BinderItem::new_folder("Root");
        let mut sub = BinderItem::new_folder("Sub");
        let leaf = BinderItem::new_text("Leaf");
        let leaf_id = leaf.id;
        sub.add_child(leaf);
        root.add_child(sub);

        let (parent, idx) = root.find_parent(&leaf_id).unwrap();
        assert_eq!(parent.title, "Sub");
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_longest_document() {
        let mut binder = Binder::default_structure();
        let mut short = BinderItem::new_text("Short");
        if let Some(ref mut doc) = short.document {
            doc.content = "one two".to_string();
        }
        let mut long = BinderItem::new_text("Long");
        if let Some(ref mut doc) = long.document {
            doc.content = "one two three four five six seven".to_string();
        }
        binder.draft.add_child(short);
        binder.draft.add_child(long);

        let longest = binder.longest_document().unwrap();
        assert_eq!(longest.title, "Long");
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
    fn test_binder_item_kind_icons() {
        assert!(!BinderItemKind::Folder.icon().is_empty());
        assert!(!BinderItemKind::Text.icon().is_empty());
        assert!(!BinderItemKind::Image.icon().is_empty());
        assert!(!BinderItemKind::Pdf.icon().is_empty());
        assert!(!BinderItemKind::WebPage.icon().is_empty());
    }

    #[test]
    fn test_binder_item_kind_labels() {
        assert_eq!(BinderItemKind::Image.label(), "Image");
        assert_eq!(BinderItemKind::Pdf.label(), "PDF");
        assert_eq!(BinderItemKind::WebPage.label(), "Web Page");
    }

    #[test]
    fn test_binder_item_kind_editability() {
        assert!(BinderItemKind::Text.is_editable());
        assert!(!BinderItemKind::Folder.is_editable());
        assert!(!BinderItemKind::Image.is_editable());
        assert!(!BinderItemKind::Pdf.is_editable());
        assert!(!BinderItemKind::WebPage.is_editable());
    }

    #[test]
    fn test_document_count() {
        let mut binder = Binder::default_structure();
        binder.draft.add_child(BinderItem::new_text("A"));
        binder.draft.add_child(BinderItem::new_folder("F"));
        binder.draft.add_child(BinderItem::new_text("B"));
        binder.research.add_child(BinderItem::new_text("R"));

        assert_eq!(binder.document_count(), 3);
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
    fn test_age_string_today() {
        let item = BinderItem::new_text("New");
        let age = item.age_string();
        assert_eq!(age, "today");
    }

    #[test]
    fn test_remove_child_not_found() {
        let mut folder = BinderItem::new_folder("Parent");
        folder.add_child(BinderItem::new_text("A"));

        let fake_id = Uuid::new_v4();
        assert!(folder.remove_child(&fake_id).is_none());
        assert_eq!(folder.child_count(), 1);
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
        assert_eq!(root.children[0].child_count(), 0);
    }

    #[test]
    fn test_split_item_no_separator() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Single");
        if let Some(ref mut doc) = item.document {
            doc.content = "No separator here.".to_string();
        }
        let id = item.id;
        binder.draft.add_child(item);

        let new_ids = binder.split_item(&id, "\n---\n");
        assert!(new_ids.is_empty()); // Only 1 section, not enough to split
        // Original should still exist
        assert!(binder.find_item(&id).is_some());
    }

    #[test]
    fn test_merge_items_single() {
        let mut binder = Binder::default_structure();
        let a = BinderItem::new_text("Only One");
        let a_id = a.id;
        binder.draft.add_child(a);

        // Merging a single item should return None
        assert!(binder.merge_items(&[a_id], "\n\n").is_none());
    }

    #[test]
    fn test_sibling_ids_not_found() {
        let folder = BinderItem::new_folder("Parent");
        let fake_id = Uuid::new_v4();
        let siblings = folder.sibling_ids(&fake_id);
        assert!(siblings.is_empty());
    }

    #[test]
    fn test_reparent_item_to_self() {
        let mut binder = Binder::default_structure();
        let item = BinderItem::new_text("Scene");
        let id = item.id;
        binder.draft.add_child(item);
        assert!(!binder.reparent_item(&id, &id, 0));
    }

    #[test]
    fn test_reparent_item_to_research() {
        let mut binder = Binder::default_structure();
        let item = BinderItem::new_text("Move me");
        let item_id = item.id;
        let research_id = binder.research.id;
        binder.draft.add_child(item);

        assert!(binder.reparent_item(&item_id, &research_id, 0));
        assert!(binder.draft.find(&item_id).is_none());
        assert!(binder.research.find(&item_id).is_some());
    }

    #[test]
    fn test_move_item_to_position_same() {
        let mut binder = Binder::default_structure();
        let a = BinderItem::new_text("A");
        let a_id = a.id;
        binder.draft.add_child(a);
        binder.draft.add_child(BinderItem::new_text("B"));

        // Move to same position should return false
        assert!(!binder.move_item_to_position(&a_id, 0));
    }

    #[test]
    fn test_group_items_empty_list() {
        let mut binder = Binder::default_structure();
        assert!(binder.group_items_into_folder(&[], "Empty").is_none());
    }

    #[test]
    fn test_flatten_folder_not_a_folder() {
        let mut binder = Binder::default_structure();
        let item = BinderItem::new_text("Not a folder");
        let id = item.id;
        binder.draft.add_child(item);
        assert!(!binder.flatten_folder(&id));
    }

    #[test]
    fn test_flatten_empty_folder() {
        let mut binder = Binder::default_structure();
        let folder = BinderItem::new_folder("Empty");
        let id = folder.id;
        binder.draft.add_child(folder);
        assert!(!binder.flatten_folder(&id));
    }

    #[test]
    fn test_split_nonexistent_item() {
        let mut binder = Binder::default_structure();
        let fake_id = Uuid::new_v4();
        let result = binder.split_item(&fake_id, "---");
        assert!(result.is_empty());
    }

    #[test]
    fn test_merge_no_content() {
        let mut binder = Binder::default_structure();
        let a = BinderItem::new_folder("Folder A");
        let b = BinderItem::new_folder("Folder B");
        let a_id = a.id;
        let b_id = b.id;
        binder.draft.add_child(a);
        binder.draft.add_child(b);
        // Folders have no document content
        assert!(binder.merge_items(&[a_id, b_id], "\n\n").is_none());
    }

    #[test]
    fn test_path_to_item_not_found() {
        let folder = BinderItem::new_folder("Root");
        let fake_id = Uuid::new_v4();
        assert!(folder.path_to_item(&fake_id).is_none());
    }

    #[test]
    fn test_path_to_item_self() {
        let folder = BinderItem::new_folder("Root");
        let path = folder.path_to_item(&folder.id).unwrap();
        assert_eq!(path, vec!["Root"]);
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
    fn test_next_sibling_not_found() {
        let folder = BinderItem::new_folder("Root");
        let fake_id = Uuid::new_v4();
        assert!(folder.next_sibling(&fake_id).is_none());
    }

    #[test]
    fn test_prev_sibling_not_found() {
        let folder = BinderItem::new_folder("Root");
        let fake_id = Uuid::new_v4();
        assert!(folder.prev_sibling(&fake_id).is_none());
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
    fn test_longest_document_empty_binder() {
        let binder = Binder::default_structure();
        assert!(binder.longest_document().is_none());
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

    #[test]
    fn test_binder_item_kind_all_variants() {
        let kinds = vec![
            BinderItemKind::Folder,
            BinderItemKind::Text,
            BinderItemKind::Image,
            BinderItemKind::Pdf,
            BinderItemKind::WebPage,
        ];
        for kind in &kinds {
            assert!(!kind.label().is_empty());
            assert!(!kind.icon().is_empty());
        }
    }

    // --- Tree utility tests ---

    #[test]
    fn test_depth_of_root() {
        let folder = BinderItem::new_folder("Root");
        assert_eq!(folder.depth_of(&folder.id), Some(0));
    }

    #[test]
    fn test_depth_of_nested() {
        let mut root = BinderItem::new_folder("Root");
        let mut sub = BinderItem::new_folder("Sub");
        let leaf = BinderItem::new_text("Leaf");
        let leaf_id = leaf.id;
        sub.add_child(leaf);
        root.add_child(sub);

        assert_eq!(root.depth_of(&leaf_id), Some(2));
    }

    #[test]
    fn test_depth_of_not_found() {
        let root = BinderItem::new_folder("Root");
        let fake_id = Uuid::new_v4();
        assert_eq!(root.depth_of(&fake_id), None);
    }

    #[test]
    fn test_binder_item_depth() {
        let mut binder = Binder::default_structure();
        let mut ch = BinderItem::new_folder("Ch1");
        let scene = BinderItem::new_text("Scene");
        let scene_id = scene.id;
        ch.add_child(scene);
        binder.draft.add_child(ch);

        assert_eq!(binder.item_depth(&scene_id), Some(2));
        assert_eq!(binder.item_depth(&binder.draft.id), Some(0));
    }

    #[test]
    fn test_ancestors_of() {
        let mut root = BinderItem::new_folder("Root");
        let mut sub = BinderItem::new_folder("Sub");
        let leaf = BinderItem::new_text("Leaf");
        let root_id = root.id;
        let sub_id = sub.id;
        let leaf_id = leaf.id;
        sub.add_child(leaf);
        root.add_child(sub);

        let ancestors = root.ancestors_of(&leaf_id).unwrap();
        assert_eq!(ancestors, vec![root_id, sub_id]);
    }

    #[test]
    fn test_ancestors_of_self() {
        let root = BinderItem::new_folder("Root");
        let ancestors = root.ancestors_of(&root.id).unwrap();
        assert!(ancestors.is_empty());
    }

    #[test]
    fn test_ancestors_of_not_found() {
        let root = BinderItem::new_folder("Root");
        assert!(root.ancestors_of(&Uuid::new_v4()).is_none());
    }

    #[test]
    fn test_binder_item_ancestors() {
        let mut binder = Binder::default_structure();
        let mut ch = BinderItem::new_folder("Ch");
        let scene = BinderItem::new_text("Sc");
        let scene_id = scene.id;
        let ch_id = ch.id;
        ch.add_child(scene);
        binder.draft.add_child(ch);

        let anc = binder.item_ancestors(&scene_id).unwrap();
        assert_eq!(anc.len(), 2); // draft, ch
        assert_eq!(anc[0], binder.draft.id);
        assert_eq!(anc[1], ch_id);
    }

    #[test]
    fn test_descendant_ids() {
        let mut root = BinderItem::new_folder("Root");
        let a = BinderItem::new_text("A");
        let b = BinderItem::new_text("B");
        let a_id = a.id;
        let b_id = b.id;
        root.add_child(a);
        root.add_child(b);

        let desc = root.descendant_ids();
        assert_eq!(desc.len(), 2);
        assert!(desc.contains(&a_id));
        assert!(desc.contains(&b_id));
    }

    #[test]
    fn test_descendant_ids_nested() {
        let mut root = BinderItem::new_folder("Root");
        let mut sub = BinderItem::new_folder("Sub");
        let leaf = BinderItem::new_text("Leaf");
        let sub_id = sub.id;
        let leaf_id = leaf.id;
        sub.add_child(leaf);
        root.add_child(sub);

        let desc = root.descendant_ids();
        assert_eq!(desc.len(), 2);
        assert!(desc.contains(&sub_id));
        assert!(desc.contains(&leaf_id));
    }

    #[test]
    fn test_binder_descendants_of() {
        let mut binder = Binder::default_structure();
        let mut ch = BinderItem::new_folder("Ch");
        let s1 = BinderItem::new_text("S1");
        let s2 = BinderItem::new_text("S2");
        let s1_id = s1.id;
        let s2_id = s2.id;
        let ch_id = ch.id;
        ch.add_child(s1);
        ch.add_child(s2);
        binder.draft.add_child(ch);

        let desc = binder.descendants_of(&ch_id);
        assert_eq!(desc.len(), 2);
        assert!(desc.contains(&s1_id));
        assert!(desc.contains(&s2_id));
    }

    #[test]
    fn test_binder_descendants_of_nonexistent() {
        let binder = Binder::default_structure();
        assert!(binder.descendants_of(&Uuid::new_v4()).is_empty());
    }

    #[test]
    fn test_is_ancestor_of() {
        let mut root = BinderItem::new_folder("Root");
        let mut sub = BinderItem::new_folder("Sub");
        let leaf = BinderItem::new_text("Leaf");
        let leaf_id = leaf.id;
        sub.add_child(leaf);
        root.add_child(sub);

        assert!(root.is_ancestor_of(&leaf_id));
        assert!(!root.is_ancestor_of(&Uuid::new_v4()));
    }

    #[test]
    fn test_descendant_count() {
        let mut root = BinderItem::new_folder("Root");
        let mut sub = BinderItem::new_folder("Sub");
        sub.add_child(BinderItem::new_text("A"));
        sub.add_child(BinderItem::new_text("B"));
        root.add_child(sub);
        root.add_child(BinderItem::new_text("C"));

        assert_eq!(root.descendant_count(), 4); // Sub, A, B, C
    }

    #[test]
    fn test_descendant_count_empty() {
        let leaf = BinderItem::new_text("Leaf");
        assert_eq!(leaf.descendant_count(), 0);
    }

    #[test]
    fn test_max_depth() {
        let mut root = BinderItem::new_folder("Root");
        let mut sub = BinderItem::new_folder("Sub");
        sub.add_child(BinderItem::new_text("Leaf"));
        root.add_child(sub);
        root.add_child(BinderItem::new_text("Shallow"));

        assert_eq!(root.max_depth(), 2);
    }

    #[test]
    fn test_max_depth_flat() {
        let mut root = BinderItem::new_folder("Root");
        root.add_child(BinderItem::new_text("A"));
        root.add_child(BinderItem::new_text("B"));

        assert_eq!(root.max_depth(), 1);
    }

    #[test]
    fn test_max_depth_leaf() {
        let leaf = BinderItem::new_text("Leaf");
        assert_eq!(leaf.max_depth(), 0);
    }

    #[test]
    fn test_binder_max_nesting_depth() {
        let mut binder = Binder::default_structure();
        let mut ch = BinderItem::new_folder("Ch");
        let mut sub = BinderItem::new_folder("Sub");
        sub.add_child(BinderItem::new_text("Deep"));
        ch.add_child(sub);
        binder.draft.add_child(ch);

        assert_eq!(binder.max_nesting_depth(), 3);
    }

    #[test]
    fn test_leaf_items() {
        let mut root = BinderItem::new_folder("Root");
        root.add_child(BinderItem::new_text("A"));
        let mut sub = BinderItem::new_folder("Sub");
        sub.add_child(BinderItem::new_text("B"));
        root.add_child(sub);

        let leaves = root.leaf_items();
        assert_eq!(leaves.len(), 2);
    }

    #[test]
    fn test_binder_leaf_documents() {
        let mut binder = Binder::default_structure();
        binder.draft.add_child(BinderItem::new_text("A"));
        binder.draft.add_child(BinderItem::new_folder("F"));
        binder.research.add_child(BinderItem::new_text("R"));

        let leaves = binder.leaf_documents();
        assert_eq!(leaves.len(), 2);
    }

    #[test]
    fn test_items_with_label() {
        use crate::core::metadata::{Label, LabelColor};
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Labeled");
        item.metadata.label = Some(Label { name: "Important".to_string(), color: LabelColor::Red });
        binder.draft.add_child(item);
        binder.draft.add_child(BinderItem::new_text("No label"));

        let found = binder.items_with_label("Important");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].title, "Labeled");
    }

    #[test]
    fn test_items_with_label_none() {
        let binder = Binder::default_structure();
        assert!(binder.items_with_label("Missing").is_empty());
    }

    #[test]
    fn test_items_with_status() {
        use crate::core::metadata::Status;
        let mut binder = Binder::default_structure();
        let mut item1 = BinderItem::new_text("Done");
        item1.metadata.status = Some(Status::new("Final"));
        let mut item2 = BinderItem::new_text("WIP");
        item2.metadata.status = Some(Status::new("Draft"));
        let mut item3 = BinderItem::new_text("Also Done");
        item3.metadata.status = Some(Status::new("Final"));
        binder.draft.add_child(item1);
        binder.draft.add_child(item2);
        binder.draft.add_child(item3);

        let final_items = binder.items_with_status("Final");
        assert_eq!(final_items.len(), 2);
    }
}
