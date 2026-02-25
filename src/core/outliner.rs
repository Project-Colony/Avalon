#![allow(dead_code)] // Methods used by test code
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Columns available in the outliner view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutlinerColumn {
    Title,
    Synopsis,
    Label,
    Status,
    WordCount,
    CharCount,
    TargetWordCount,
    TargetProgress,
    DateCreated,
    DateModified,
    Section,
    IncludeInCompile,
    PageCount,
    ParagraphCount,
    WordFrequency,
}

/// Configuration for a single column in the outliner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnConfig {
    pub column: OutlinerColumn,
    pub visible: bool,
    pub width: f32,
    pub position: usize,
}

/// Settings controlling the outliner's appearance and behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlinerSettings {
    pub columns: Vec<ColumnConfig>,
    pub sort_column: Option<OutlinerColumn>,
    pub sort_ascending: bool,
    pub show_synopsis: bool,
    pub alternating_row_colors: bool,
    pub indent_level_px: f32,
}

/// A binder item with the data needed for the outliner to build rows
#[derive(Debug, Clone)]
pub struct OutlinerItem {
    pub id: Uuid,
    pub children: Vec<OutlinerItem>,
}

/// Full outliner state including settings and expansion
#[derive(Debug, Clone)]
pub struct OutlinerState {
    pub settings: OutlinerSettings,
    pub expanded: HashSet<Uuid>,
}

impl OutlinerState {
    /// Create a new outliner state with the given settings
    pub fn new(settings: OutlinerSettings) -> Self {
        Self {
            settings,
            expanded: HashSet::new(),
        }
    }

    /// Toggle the expanded state of an item
    pub fn toggle_expand(&mut self, id: Uuid) {
        if !self.expanded.remove(&id) {
            self.expanded.insert(id);
        }
    }

    /// Expand all items in the given tree
    pub fn expand_all(&mut self, items: &[OutlinerItem]) {
        for item in items {
            self.expand_all_recursive(item);
        }
    }

    fn expand_all_recursive(&mut self, item: &OutlinerItem) {
        if !item.children.is_empty() {
            self.expanded.insert(item.id);
        }
        for child in &item.children {
            self.expand_all_recursive(child);
        }
    }

    /// Collapse all items
    pub fn collapse_all(&mut self) {
        self.expanded.clear();
    }

    /// Sort rows in place by the given column
    pub fn sort_by(&mut self, column: OutlinerColumn, ascending: bool) {
        self.settings.sort_column = Some(column);
        self.settings.sort_ascending = ascending;
    }

}

/// Returns a sensible default set of columns for the outliner
pub fn default_columns() -> Vec<ColumnConfig> {
    vec![
        ColumnConfig { column: OutlinerColumn::Title, visible: true, width: 250.0, position: 0 },
        ColumnConfig { column: OutlinerColumn::Synopsis, visible: true, width: 300.0, position: 1 },
        ColumnConfig { column: OutlinerColumn::Label, visible: true, width: 100.0, position: 2 },
        ColumnConfig { column: OutlinerColumn::Status, visible: true, width: 100.0, position: 3 },
        ColumnConfig { column: OutlinerColumn::WordCount, visible: true, width: 80.0, position: 4 },
        ColumnConfig { column: OutlinerColumn::CharCount, visible: false, width: 80.0, position: 5 },
        ColumnConfig { column: OutlinerColumn::TargetWordCount, visible: false, width: 100.0, position: 6 },
        ColumnConfig { column: OutlinerColumn::TargetProgress, visible: true, width: 100.0, position: 7 },
        ColumnConfig { column: OutlinerColumn::DateCreated, visible: false, width: 140.0, position: 8 },
        ColumnConfig { column: OutlinerColumn::DateModified, visible: true, width: 140.0, position: 9 },
        ColumnConfig { column: OutlinerColumn::Section, visible: false, width: 100.0, position: 10 },
        ColumnConfig { column: OutlinerColumn::IncludeInCompile, visible: true, width: 80.0, position: 11 },
        ColumnConfig { column: OutlinerColumn::PageCount, visible: false, width: 80.0, position: 12 },
        ColumnConfig { column: OutlinerColumn::ParagraphCount, visible: false, width: 80.0, position: 13 },
        ColumnConfig { column: OutlinerColumn::WordFrequency, visible: false, width: 120.0, position: 14 },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(children: Vec<OutlinerItem>) -> OutlinerItem {
        OutlinerItem {
            id: Uuid::new_v4(),
            children,
        }
    }

    fn make_item_with_id(id: Uuid, children: Vec<OutlinerItem>) -> OutlinerItem {
        OutlinerItem {
            id,
            children,
        }
    }

    fn default_settings() -> OutlinerSettings {
        OutlinerSettings {
            columns: default_columns(),
            sort_column: None,
            sort_ascending: true,
            show_synopsis: true,
            alternating_row_colors: true,
            indent_level_px: 20.0,
        }
    }

    // ---- Expand/Collapse tests ----

    #[test]
    fn test_toggle_expand() {
        let mut state = OutlinerState::new(default_settings());
        let id = Uuid::new_v4();
        assert!(!state.expanded.contains(&id));

        state.toggle_expand(id);
        assert!(state.expanded.contains(&id));

        state.toggle_expand(id);
        assert!(!state.expanded.contains(&id));
    }

    #[test]
    fn test_expand_all() {
        let mut state = OutlinerState::new(default_settings());
        let items = vec![
            make_item(vec![
                make_item(vec![
                    make_item(vec![]),
                ]),
            ]),
            make_item(vec![
                make_item(vec![]),
            ]),
        ];
        state.expand_all(&items);
        // Should have expanded: Part 1, Ch 1, Part 2 (3 nodes with children)
        assert_eq!(state.expanded.len(), 3);
    }

    #[test]
    fn test_expand_all_skips_leaves() {
        let mut state = OutlinerState::new(default_settings());
        let leaf = make_item(vec![]);
        let leaf_id = leaf.id;
        state.expand_all(&[leaf]);
        // Leaf has no children, should not be expanded
        assert!(!state.expanded.contains(&leaf_id));
    }

    #[test]
    fn test_collapse_all() {
        let mut state = OutlinerState::new(default_settings());
        state.expanded.insert(Uuid::new_v4());
        state.expanded.insert(Uuid::new_v4());
        state.expanded.insert(Uuid::new_v4());
        assert_eq!(state.expanded.len(), 3);

        state.collapse_all();
        assert!(state.expanded.is_empty());
    }

    // ---- Sorting tests ----

    #[test]
    fn test_sort_by_sets_settings() {
        let mut state = OutlinerState::new(default_settings());
        state.sort_by(OutlinerColumn::WordCount, false);
        assert_eq!(state.settings.sort_column, Some(OutlinerColumn::WordCount));
        assert!(!state.settings.sort_ascending);
    }

    // ---- Default columns test ----

    #[test]
    fn test_default_columns() {
        let cols = default_columns();
        assert_eq!(cols.len(), 15);
        // Title should be first and visible
        assert_eq!(cols[0].column, OutlinerColumn::Title);
        assert!(cols[0].visible);
        assert_eq!(cols[0].position, 0);
        // All positions should be sequential
        for (i, col) in cols.iter().enumerate() {
            assert_eq!(col.position, i);
        }
    }

    #[test]
    fn test_expand_collapse_roundtrip() {
        let mut state = OutlinerState::new(default_settings());
        let items = vec![
            make_item(vec![
                make_item(vec![]),
            ]),
        ];
        state.expand_all(&items);
        assert_eq!(state.expanded.len(), 1);

        state.collapse_all();
        assert!(state.expanded.is_empty());
    }
}
