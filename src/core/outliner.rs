use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
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

/// A flattened row in the outliner, representing one binder item
#[derive(Debug, Clone)]
pub struct OutlinerRow {
    pub item_id: Uuid,
    pub depth: usize,
    pub expanded: bool,
    pub values: HashMap<OutlinerColumn, CellValue>,
}

/// A value that can appear in a cell of the outliner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellValue {
    Text(String),
    Number(f64),
    Bool(bool),
    Date(DateTime<Utc>),
    Progress(f64),
    None,
}

/// A binder item with the data needed for the outliner to build rows
#[derive(Debug, Clone)]
pub struct OutlinerItem {
    pub id: Uuid,
    pub title: String,
    pub synopsis: String,
    pub label: String,
    pub status: String,
    pub word_count: usize,
    pub target_word_count: Option<usize>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub include_in_compile: bool,
    pub children: Vec<OutlinerItem>,
}

/// Full outliner state including settings, expansion, and selection
#[derive(Debug, Clone)]
pub struct OutlinerState {
    pub settings: OutlinerSettings,
    pub expanded: HashSet<Uuid>,
    pub selection: HashSet<Uuid>,
}

impl OutlinerState {
    /// Create a new outliner state with the given settings
    pub fn new(settings: OutlinerSettings) -> Self {
        Self {
            settings,
            expanded: HashSet::new(),
            selection: HashSet::new(),
        }
    }

    /// Flatten a tree of items into outliner rows, respecting expansion state.
    /// Performs depth-first traversal, only descending into expanded nodes.
    pub fn build_rows(&self, items: &[OutlinerItem]) -> Vec<OutlinerRow> {
        let mut rows = Vec::new();
        for item in items {
            self.build_rows_recursive(item, 0, &mut rows);
        }
        rows
    }

    fn build_rows_recursive(
        &self,
        item: &OutlinerItem,
        depth: usize,
        rows: &mut Vec<OutlinerRow>,
    ) {
        let is_expanded = self.expanded.contains(&item.id);
        let mut values = HashMap::new();

        values.insert(OutlinerColumn::Title, CellValue::Text(item.title.clone()));
        values.insert(OutlinerColumn::Synopsis, CellValue::Text(item.synopsis.clone()));
        values.insert(OutlinerColumn::Label, CellValue::Text(item.label.clone()));
        values.insert(OutlinerColumn::Status, CellValue::Text(item.status.clone()));
        values.insert(OutlinerColumn::WordCount, CellValue::Number(item.word_count as f64));
        values.insert(OutlinerColumn::CharCount, CellValue::Number(0.0));
        values.insert(
            OutlinerColumn::TargetWordCount,
            match item.target_word_count {
                Some(t) => CellValue::Number(t as f64),
                None => CellValue::None,
            },
        );
        values.insert(
            OutlinerColumn::TargetProgress,
            match item.target_word_count {
                Some(target) if target > 0 => {
                    CellValue::Progress(item.word_count as f64 / target as f64)
                }
                _ => CellValue::None,
            },
        );
        values.insert(OutlinerColumn::DateCreated, CellValue::Date(item.created));
        values.insert(OutlinerColumn::DateModified, CellValue::Date(item.modified));
        values.insert(OutlinerColumn::Section, CellValue::Text(String::new()));
        values.insert(OutlinerColumn::IncludeInCompile, CellValue::Bool(item.include_in_compile));
        values.insert(OutlinerColumn::PageCount, CellValue::Number(item.word_count as f64 / super::WORDS_PER_PAGE as f64));
        values.insert(OutlinerColumn::ParagraphCount, CellValue::Number(0.0));
        values.insert(OutlinerColumn::WordFrequency, CellValue::None);

        rows.push(OutlinerRow {
            item_id: item.id,
            depth,
            expanded: is_expanded,
            values,
        });

        if is_expanded {
            for child in &item.children {
                self.build_rows_recursive(child, depth + 1, rows);
            }
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

    /// Expand items up to and including the given depth (0-indexed).
    /// Depth 0 expands root items, depth 1 expands root and their direct children, etc.
    pub fn expand_to_depth(&mut self, depth: usize, items: &[OutlinerItem]) {
        self.expanded.clear();
        for item in items {
            self.expand_to_depth_recursive(item, 0, depth);
        }
    }

    fn expand_to_depth_recursive(
        &mut self,
        item: &OutlinerItem,
        current_depth: usize,
        max_depth: usize,
    ) {
        if current_depth <= max_depth && !item.children.is_empty() {
            self.expanded.insert(item.id);
        }
        if current_depth < max_depth {
            for child in &item.children {
                self.expand_to_depth_recursive(child, current_depth + 1, max_depth);
            }
        }
    }

    /// Sort rows in place by the given column
    pub fn sort_by(&mut self, column: OutlinerColumn, ascending: bool) {
        self.settings.sort_column = Some(column);
        self.settings.sort_ascending = ascending;
    }

    /// Apply the current sort settings to a vec of rows
    pub fn apply_sort(&self, rows: &mut [OutlinerRow]) {
        if let Some(ref col) = self.settings.sort_column {
            let col = *col;
            let ascending = self.settings.sort_ascending;
            rows.sort_by(|a, b| {
                let val_a = a.values.get(&col).unwrap_or(&CellValue::None);
                let val_b = b.values.get(&col).unwrap_or(&CellValue::None);
                let ord = compare_cell_values(val_a, val_b);
                if ascending { ord } else { ord.reverse() }
            });
        }
    }

    /// Reorder a column from one position to another
    pub fn move_column(&mut self, from_index: usize, to_index: usize) {
        let len = self.settings.columns.len();
        if from_index >= len || to_index >= len || from_index == to_index {
            return;
        }
        let col = self.settings.columns.remove(from_index);
        self.settings.columns.insert(to_index, col);
        // Re-assign position indices
        for (i, cfg) in self.settings.columns.iter_mut().enumerate() {
            cfg.position = i;
        }
    }

    /// Toggle visibility of a column
    pub fn toggle_column(&mut self, column: OutlinerColumn) {
        for cfg in &mut self.settings.columns {
            if cfg.column == column {
                cfg.visible = !cfg.visible;
                return;
            }
        }
    }

    /// Resize a column
    pub fn resize_column(&mut self, column: OutlinerColumn, width: f32) {
        for cfg in &mut self.settings.columns {
            if cfg.column == column {
                cfg.width = width;
                return;
            }
        }
    }

    /// Select a single row, clearing any previous selection
    pub fn select_row(&mut self, id: Uuid) {
        self.selection.clear();
        self.selection.insert(id);
    }

    /// Select a contiguous range of rows between two IDs (inclusive)
    pub fn select_range(&mut self, from: Uuid, to: Uuid, rows: &[OutlinerRow]) {
        let from_idx = rows.iter().position(|r| r.item_id == from);
        let to_idx = rows.iter().position(|r| r.item_id == to);

        if let (Some(start), Some(end)) = (from_idx, to_idx) {
            let (lo, hi) = if start <= end { (start, end) } else { (end, start) };
            self.selection.clear();
            for row in &rows[lo..=hi] {
                self.selection.insert(row.item_id);
            }
        }
    }

    /// Sum of visible column widths
    pub fn total_width(&self) -> f32 {
        self.settings
            .columns
            .iter()
            .filter(|c| c.visible)
            .map(|c| c.width)
            .sum()
    }

    /// Visible columns sorted by position
    pub fn visible_columns(&self) -> Vec<&ColumnConfig> {
        let mut cols: Vec<&ColumnConfig> = self
            .settings
            .columns
            .iter()
            .filter(|c| c.visible)
            .collect();
        cols.sort_by_key(|c| c.position);
        cols
    }

    /// Get a row by its index in the flattened list
    pub fn row_at_index<'a>(&self, index: usize, rows: &'a [OutlinerRow]) -> Option<&'a OutlinerRow> {
        rows.get(index)
    }
}

/// Compare two CellValues for sorting purposes
pub fn compare_cell_values(a: &CellValue, b: &CellValue) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a, b) {
        (CellValue::Text(a), CellValue::Text(b)) => a.to_lowercase().cmp(&b.to_lowercase()),
        (CellValue::Number(a), CellValue::Number(b)) => {
            a.partial_cmp(b).unwrap_or(Ordering::Equal)
        }
        (CellValue::Bool(a), CellValue::Bool(b)) => a.cmp(b),
        (CellValue::Date(a), CellValue::Date(b)) => a.cmp(b),
        (CellValue::Progress(a), CellValue::Progress(b)) => {
            a.partial_cmp(b).unwrap_or(Ordering::Equal)
        }
        (CellValue::None, CellValue::None) => Ordering::Equal,
        // None always sorts last
        (CellValue::None, _) => Ordering::Greater,
        (_, CellValue::None) => Ordering::Less,
        // Cross-type: fall back to discriminant ordering
        _ => discriminant_rank(a).cmp(&discriminant_rank(b)),
    }
}

/// Assign a numeric rank to each CellValue variant for cross-type comparison
fn discriminant_rank(v: &CellValue) -> u8 {
    match v {
        CellValue::Text(_) => 0,
        CellValue::Number(_) => 1,
        CellValue::Bool(_) => 2,
        CellValue::Date(_) => 3,
        CellValue::Progress(_) => 4,
        CellValue::None => 5,
    }
}

/// Format a CellValue for display
pub fn format_cell_value(value: &CellValue) -> String {
    match value {
        CellValue::Text(s) => s.clone(),
        CellValue::Number(n) => {
            if *n == (*n as i64) as f64 {
                format!("{}", *n as i64)
            } else {
                format!("{:.1}", n)
            }
        }
        CellValue::Bool(b) => {
            if *b { "Yes".to_string() } else { "No".to_string() }
        }
        CellValue::Date(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        CellValue::Progress(p) => format!("{:.0}%", p * 100.0),
        CellValue::None => String::new(),
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

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    fn make_item(title: &str, word_count: usize, children: Vec<OutlinerItem>) -> OutlinerItem {
        OutlinerItem {
            id: Uuid::new_v4(),
            title: title.to_string(),
            synopsis: format!("Synopsis for {}", title),
            label: "Chapter".to_string(),
            status: "Draft".to_string(),
            word_count,
            target_word_count: Some(5000),
            created: now(),
            modified: now(),
            include_in_compile: true,
            children,
        }
    }

    fn make_item_with_id(id: Uuid, title: &str, children: Vec<OutlinerItem>) -> OutlinerItem {
        OutlinerItem {
            id,
            title: title.to_string(),
            synopsis: String::new(),
            label: String::new(),
            status: String::new(),
            word_count: 100,
            target_word_count: None,
            created: now(),
            modified: now(),
            include_in_compile: true,
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

    // ---- Row flattening tests ----

    #[test]
    fn test_build_rows_empty() {
        let state = OutlinerState::new(default_settings());
        let rows = state.build_rows(&[]);
        assert!(rows.is_empty());
    }

    #[test]
    fn test_build_rows_flat_list() {
        let state = OutlinerState::new(default_settings());
        let items = vec![
            make_item("Chapter 1", 1000, vec![]),
            make_item("Chapter 2", 2000, vec![]),
            make_item("Chapter 3", 3000, vec![]),
        ];
        let rows = state.build_rows(&items);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].depth, 0);
        assert_eq!(rows[1].depth, 0);
        assert_eq!(rows[2].depth, 0);
    }

    #[test]
    fn test_build_rows_collapsed_children_hidden() {
        let state = OutlinerState::new(default_settings());
        let items = vec![
            make_item("Part 1", 0, vec![
                make_item("Chapter 1", 1000, vec![]),
                make_item("Chapter 2", 2000, vec![]),
            ]),
        ];
        let rows = state.build_rows(&items);
        // Parent is collapsed by default, so only the parent row appears
        assert_eq!(rows.len(), 1);
        assert!(!rows[0].expanded);
    }

    #[test]
    fn test_build_rows_expanded_children_visible() {
        let parent_id = Uuid::new_v4();
        let items = vec![
            make_item_with_id(parent_id, "Part 1", vec![
                make_item("Chapter 1", 1000, vec![]),
                make_item("Chapter 2", 2000, vec![]),
            ]),
        ];
        let mut state = OutlinerState::new(default_settings());
        state.expanded.insert(parent_id);
        let rows = state.build_rows(&items);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].depth, 0);
        assert!(rows[0].expanded);
        assert_eq!(rows[1].depth, 1);
        assert_eq!(rows[2].depth, 1);
    }

    #[test]
    fn test_build_rows_depth_first_order() {
        let root_id = Uuid::new_v4();
        let child_id = Uuid::new_v4();
        let items = vec![
            make_item_with_id(root_id, "Root", vec![
                make_item_with_id(child_id, "Child", vec![
                    make_item("Grandchild", 500, vec![]),
                ]),
            ]),
        ];
        let mut state = OutlinerState::new(default_settings());
        state.expanded.insert(root_id);
        state.expanded.insert(child_id);
        let rows = state.build_rows(&items);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].depth, 0);
        assert_eq!(rows[1].depth, 1);
        assert_eq!(rows[2].depth, 2);
    }

    #[test]
    fn test_build_rows_partial_expansion() {
        let root_id = Uuid::new_v4();
        let child_a_id = Uuid::new_v4();
        let items = vec![
            make_item_with_id(root_id, "Root", vec![
                make_item_with_id(child_a_id, "Child A", vec![
                    make_item("Grandchild A1", 100, vec![]),
                ]),
                make_item("Child B", 200, vec![
                    make_item("Grandchild B1", 100, vec![]),
                ]),
            ]),
        ];
        let mut state = OutlinerState::new(default_settings());
        state.expanded.insert(root_id);
        // Only expand Child A, not Child B
        state.expanded.insert(child_a_id);
        let rows = state.build_rows(&items);
        // Root, Child A, Grandchild A1, Child B (Child B is collapsed so B1 hidden)
        assert_eq!(rows.len(), 4);
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
            make_item("Part 1", 0, vec![
                make_item("Ch 1", 100, vec![
                    make_item("Scene 1", 50, vec![]),
                ]),
            ]),
            make_item("Part 2", 0, vec![
                make_item("Ch 2", 200, vec![]),
            ]),
        ];
        state.expand_all(&items);
        // Should have expanded: Part 1, Ch 1, Part 2 (3 nodes with children)
        assert_eq!(state.expanded.len(), 3);
    }

    #[test]
    fn test_expand_all_skips_leaves() {
        let mut state = OutlinerState::new(default_settings());
        let leaf = make_item("Leaf", 100, vec![]);
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

    #[test]
    fn test_expand_to_depth_zero() {
        let mut state = OutlinerState::new(default_settings());
        let items = vec![
            make_item("Part 1", 0, vec![
                make_item("Ch 1", 100, vec![
                    make_item("Scene 1", 50, vec![]),
                ]),
            ]),
        ];
        state.expand_to_depth(0, &items);
        // Only root-level items with children should be expanded
        assert!(state.expanded.contains(&items[0].id));
        assert!(!state.expanded.contains(&items[0].children[0].id));
    }

    #[test]
    fn test_expand_to_depth_one() {
        let mut state = OutlinerState::new(default_settings());
        let items = vec![
            make_item("Part 1", 0, vec![
                make_item("Ch 1", 100, vec![
                    make_item("Scene 1", 50, vec![]),
                ]),
            ]),
        ];
        state.expand_to_depth(1, &items);
        // Root and depth-1 items with children should be expanded
        assert!(state.expanded.contains(&items[0].id));
        assert!(state.expanded.contains(&items[0].children[0].id));
    }

    // ---- Sorting tests ----

    #[test]
    fn test_sort_by_sets_settings() {
        let mut state = OutlinerState::new(default_settings());
        state.sort_by(OutlinerColumn::WordCount, false);
        assert_eq!(state.settings.sort_column, Some(OutlinerColumn::WordCount));
        assert!(!state.settings.sort_ascending);
    }

    #[test]
    fn test_apply_sort_ascending() {
        let mut state = OutlinerState::new(default_settings());
        state.sort_by(OutlinerColumn::Title, true);

        let items = vec![
            make_item("Zebra", 100, vec![]),
            make_item("Alpha", 200, vec![]),
            make_item("Middle", 150, vec![]),
        ];
        let mut rows = state.build_rows(&items);
        state.apply_sort(&mut rows);

        let titles: Vec<String> = rows.iter().map(|r| {
            match r.values.get(&OutlinerColumn::Title).unwrap() {
                CellValue::Text(t) => t.clone(),
                _ => String::new(),
            }
        }).collect();
        assert_eq!(titles, vec!["Alpha", "Middle", "Zebra"]);
    }

    #[test]
    fn test_apply_sort_descending() {
        let mut state = OutlinerState::new(default_settings());
        state.sort_by(OutlinerColumn::WordCount, false);

        let items = vec![
            make_item("A", 100, vec![]),
            make_item("B", 300, vec![]),
            make_item("C", 200, vec![]),
        ];
        let mut rows = state.build_rows(&items);
        state.apply_sort(&mut rows);

        let counts: Vec<f64> = rows.iter().map(|r| {
            match r.values.get(&OutlinerColumn::WordCount).unwrap() {
                CellValue::Number(n) => *n,
                _ => 0.0,
            }
        }).collect();
        assert_eq!(counts, vec![300.0, 200.0, 100.0]);
    }

    #[test]
    fn test_apply_sort_no_column() {
        let state = OutlinerState::new(default_settings());
        let items = vec![
            make_item("C", 300, vec![]),
            make_item("A", 100, vec![]),
        ];
        let mut rows = state.build_rows(&items);
        let original_order: Vec<Uuid> = rows.iter().map(|r| r.item_id).collect();
        state.apply_sort(&mut rows);
        let after_order: Vec<Uuid> = rows.iter().map(|r| r.item_id).collect();
        assert_eq!(original_order, after_order);
    }

    // ---- Column operations tests ----

    #[test]
    fn test_move_column() {
        let mut state = OutlinerState::new(default_settings());
        let first_col = state.settings.columns[0].column;
        let second_col = state.settings.columns[1].column;

        state.move_column(0, 1);

        assert_eq!(state.settings.columns[0].column, second_col);
        assert_eq!(state.settings.columns[1].column, first_col);
        // Positions must be updated
        assert_eq!(state.settings.columns[0].position, 0);
        assert_eq!(state.settings.columns[1].position, 1);
    }

    #[test]
    fn test_move_column_out_of_bounds() {
        let mut state = OutlinerState::new(default_settings());
        let original: Vec<OutlinerColumn> = state.settings.columns.iter().map(|c| c.column).collect();
        state.move_column(0, 100);
        let after: Vec<OutlinerColumn> = state.settings.columns.iter().map(|c| c.column).collect();
        assert_eq!(original, after);
    }

    #[test]
    fn test_move_column_same_position() {
        let mut state = OutlinerState::new(default_settings());
        let original: Vec<OutlinerColumn> = state.settings.columns.iter().map(|c| c.column).collect();
        state.move_column(2, 2);
        let after: Vec<OutlinerColumn> = state.settings.columns.iter().map(|c| c.column).collect();
        assert_eq!(original, after);
    }

    #[test]
    fn test_toggle_column_visibility() {
        let mut state = OutlinerState::new(default_settings());
        // Title starts visible
        assert!(state.settings.columns.iter().find(|c| c.column == OutlinerColumn::Title).unwrap().visible);
        state.toggle_column(OutlinerColumn::Title);
        assert!(!state.settings.columns.iter().find(|c| c.column == OutlinerColumn::Title).unwrap().visible);
        state.toggle_column(OutlinerColumn::Title);
        assert!(state.settings.columns.iter().find(|c| c.column == OutlinerColumn::Title).unwrap().visible);
    }

    #[test]
    fn test_resize_column() {
        let mut state = OutlinerState::new(default_settings());
        state.resize_column(OutlinerColumn::Title, 400.0);
        let title_cfg = state.settings.columns.iter().find(|c| c.column == OutlinerColumn::Title).unwrap();
        assert!((title_cfg.width - 400.0).abs() < f32::EPSILON);
    }

    // ---- Selection tests ----

    #[test]
    fn test_select_row() {
        let mut state = OutlinerState::new(default_settings());
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        state.select_row(id1);
        assert!(state.selection.contains(&id1));
        assert_eq!(state.selection.len(), 1);

        state.select_row(id2);
        assert!(state.selection.contains(&id2));
        assert!(!state.selection.contains(&id1));
        assert_eq!(state.selection.len(), 1);
    }

    #[test]
    fn test_select_range() {
        let mut state = OutlinerState::new(default_settings());
        let items = vec![
            make_item("A", 100, vec![]),
            make_item("B", 200, vec![]),
            make_item("C", 300, vec![]),
            make_item("D", 400, vec![]),
        ];
        let rows = state.build_rows(&items);
        let id_b = rows[1].item_id;
        let id_d = rows[3].item_id;

        state.select_range(id_b, id_d, &rows);
        assert_eq!(state.selection.len(), 3);
        assert!(state.selection.contains(&id_b));
        assert!(state.selection.contains(&rows[2].item_id));
        assert!(state.selection.contains(&id_d));
    }

    #[test]
    fn test_select_range_reversed() {
        let mut state = OutlinerState::new(default_settings());
        let items = vec![
            make_item("A", 100, vec![]),
            make_item("B", 200, vec![]),
            make_item("C", 300, vec![]),
        ];
        let rows = state.build_rows(&items);
        let id_a = rows[0].item_id;
        let id_c = rows[2].item_id;

        // Select from C to A (reversed)
        state.select_range(id_c, id_a, &rows);
        assert_eq!(state.selection.len(), 3);
    }

    #[test]
    fn test_select_range_invalid_id() {
        let mut state = OutlinerState::new(default_settings());
        let items = vec![make_item("A", 100, vec![])];
        let rows = state.build_rows(&items);
        let fake_id = Uuid::new_v4();
        state.select_range(rows[0].item_id, fake_id, &rows);
        // Selection should remain empty if one ID is not found
        assert!(state.selection.is_empty());
    }

    // ---- Width and visible columns tests ----

    #[test]
    fn test_total_width() {
        let state = OutlinerState::new(default_settings());
        let expected: f32 = state.settings.columns.iter().filter(|c| c.visible).map(|c| c.width).sum();
        assert!((state.total_width() - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_visible_columns() {
        let state = OutlinerState::new(default_settings());
        let vis = state.visible_columns();
        let all_visible_count = state.settings.columns.iter().filter(|c| c.visible).count();
        assert_eq!(vis.len(), all_visible_count);
        // Verify sorted by position
        for i in 1..vis.len() {
            assert!(vis[i - 1].position < vis[i].position);
        }
    }

    #[test]
    fn test_visible_columns_after_toggle() {
        let mut state = OutlinerState::new(default_settings());
        let before = state.visible_columns().len();
        state.toggle_column(OutlinerColumn::Title);
        let after = state.visible_columns().len();
        assert_eq!(after, before - 1);
    }

    // ---- Row at index ----

    #[test]
    fn test_row_at_index() {
        let state = OutlinerState::new(default_settings());
        let items = vec![
            make_item("A", 100, vec![]),
            make_item("B", 200, vec![]),
        ];
        let rows = state.build_rows(&items);
        assert!(state.row_at_index(0, &rows).is_some());
        assert!(state.row_at_index(1, &rows).is_some());
        assert!(state.row_at_index(2, &rows).is_none());
    }

    // ---- CellValue comparison tests ----

    #[test]
    fn test_compare_text_values() {
        let a = CellValue::Text("alpha".to_string());
        let b = CellValue::Text("Beta".to_string());
        // Case-insensitive comparison: alpha < beta
        assert_eq!(compare_cell_values(&a, &b), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_compare_number_values() {
        let a = CellValue::Number(10.0);
        let b = CellValue::Number(20.0);
        assert_eq!(compare_cell_values(&a, &b), std::cmp::Ordering::Less);
        assert_eq!(compare_cell_values(&b, &a), std::cmp::Ordering::Greater);
        assert_eq!(compare_cell_values(&a, &a), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_compare_bool_values() {
        let t = CellValue::Bool(true);
        let f = CellValue::Bool(false);
        assert_eq!(compare_cell_values(&f, &t), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_compare_progress_values() {
        let a = CellValue::Progress(0.5);
        let b = CellValue::Progress(0.75);
        assert_eq!(compare_cell_values(&a, &b), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_compare_none_sorts_last() {
        let text = CellValue::Text("hello".to_string());
        let none = CellValue::None;
        assert_eq!(compare_cell_values(&text, &none), std::cmp::Ordering::Less);
        assert_eq!(compare_cell_values(&none, &text), std::cmp::Ordering::Greater);
        assert_eq!(compare_cell_values(&none, &none), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_compare_different_types() {
        let text = CellValue::Text("hello".to_string());
        let num = CellValue::Number(42.0);
        // Text(0) < Number(1) by discriminant
        assert_eq!(compare_cell_values(&text, &num), std::cmp::Ordering::Less);
    }

    // ---- Format tests ----

    #[test]
    fn test_format_text() {
        assert_eq!(format_cell_value(&CellValue::Text("hello".to_string())), "hello");
    }

    #[test]
    fn test_format_number_integer() {
        assert_eq!(format_cell_value(&CellValue::Number(42.0)), "42");
    }

    #[test]
    fn test_format_number_decimal() {
        assert_eq!(format_cell_value(&CellValue::Number(3.14)), "3.1");
    }

    #[test]
    fn test_format_bool() {
        assert_eq!(format_cell_value(&CellValue::Bool(true)), "Yes");
        assert_eq!(format_cell_value(&CellValue::Bool(false)), "No");
    }

    #[test]
    fn test_format_progress() {
        assert_eq!(format_cell_value(&CellValue::Progress(0.75)), "75%");
    }

    #[test]
    fn test_format_none() {
        assert_eq!(format_cell_value(&CellValue::None), "");
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

    // ---- Edge case tests ----

    #[test]
    fn test_build_rows_values_populated() {
        let state = OutlinerState::new(default_settings());
        let items = vec![make_item("Test", 1500, vec![])];
        let rows = state.build_rows(&items);
        let row = &rows[0];
        match row.values.get(&OutlinerColumn::Title) {
            Some(CellValue::Text(t)) => assert_eq!(t, "Test"),
            _ => panic!("Expected Text value for Title"),
        }
        match row.values.get(&OutlinerColumn::WordCount) {
            Some(CellValue::Number(n)) => assert_eq!(*n, 1500.0),
            _ => panic!("Expected Number value for WordCount"),
        }
        match row.values.get(&OutlinerColumn::IncludeInCompile) {
            Some(CellValue::Bool(b)) => assert!(*b),
            _ => panic!("Expected Bool value for IncludeInCompile"),
        }
    }

    #[test]
    fn test_target_progress_computed() {
        let state = OutlinerState::new(default_settings());
        let items = vec![make_item("Test", 2500, vec![])]; // target = 5000
        let rows = state.build_rows(&items);
        match rows[0].values.get(&OutlinerColumn::TargetProgress) {
            Some(CellValue::Progress(p)) => assert!((*p - 0.5).abs() < 0.001),
            _ => panic!("Expected Progress value for TargetProgress"),
        }
    }

    #[test]
    fn test_target_progress_none_when_no_target() {
        let state = OutlinerState::new(default_settings());
        let mut item = make_item("Test", 1000, vec![]);
        item.target_word_count = None;
        let rows = state.build_rows(&[item]);
        match rows[0].values.get(&OutlinerColumn::TargetProgress) {
            Some(CellValue::None) => {}
            _ => panic!("Expected None for TargetProgress when no target"),
        }
    }

    #[test]
    fn test_expand_collapse_roundtrip() {
        let mut state = OutlinerState::new(default_settings());
        let items = vec![
            make_item("Root", 0, vec![
                make_item("Child", 100, vec![]),
            ]),
        ];
        state.expand_all(&items);
        let rows_expanded = state.build_rows(&items);
        assert_eq!(rows_expanded.len(), 2);

        state.collapse_all();
        let rows_collapsed = state.build_rows(&items);
        assert_eq!(rows_collapsed.len(), 1);
    }

    #[test]
    fn test_multiple_roots_with_expansion() {
        let root1_id = Uuid::new_v4();
        let root2_id = Uuid::new_v4();
        let items = vec![
            make_item_with_id(root1_id, "Root 1", vec![
                make_item("Child 1a", 100, vec![]),
            ]),
            make_item_with_id(root2_id, "Root 2", vec![
                make_item("Child 2a", 200, vec![]),
                make_item("Child 2b", 300, vec![]),
            ]),
        ];
        let mut state = OutlinerState::new(default_settings());
        state.expanded.insert(root2_id);
        let rows = state.build_rows(&items);
        // Root 1 (collapsed) + Root 2 (expanded) + Child 2a + Child 2b
        assert_eq!(rows.len(), 4);
    }

    #[test]
    fn test_format_date_value() {
        use chrono::TimeZone;
        let dt = Utc.with_ymd_and_hms(2025, 6, 15, 10, 30, 0).unwrap();
        let formatted = format_cell_value(&CellValue::Date(dt));
        assert!(formatted.contains("2025-06-15"));
        assert!(formatted.contains("10:30"));
    }

    #[test]
    fn test_resize_column_nonexistent() {
        let mut state = OutlinerState::new(OutlinerSettings {
            columns: vec![],
            sort_column: None,
            sort_ascending: true,
            show_synopsis: true,
            alternating_row_colors: true,
            indent_level_px: 20.0,
        });
        // Should not panic
        state.resize_column(OutlinerColumn::Title, 500.0);
    }

    #[test]
    fn test_toggle_column_nonexistent() {
        let mut state = OutlinerState::new(OutlinerSettings {
            columns: vec![],
            sort_column: None,
            sort_ascending: true,
            show_synopsis: true,
            alternating_row_colors: true,
            indent_level_px: 20.0,
        });
        // Should not panic
        state.toggle_column(OutlinerColumn::Title);
    }

    #[test]
    fn test_total_width_no_visible() {
        let mut state = OutlinerState::new(default_settings());
        for cfg in &mut state.settings.columns {
            cfg.visible = false;
        }
        assert!((state.total_width() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_visible_columns_empty() {
        let mut state = OutlinerState::new(default_settings());
        for cfg in &mut state.settings.columns {
            cfg.visible = false;
        }
        assert!(state.visible_columns().is_empty());
    }

    #[test]
    fn test_select_range_single_item() {
        let mut state = OutlinerState::new(default_settings());
        let items = vec![make_item("Only", 100, vec![])];
        let rows = state.build_rows(&items);
        let id = rows[0].item_id;
        state.select_range(id, id, &rows);
        assert_eq!(state.selection.len(), 1);
        assert!(state.selection.contains(&id));
    }
}

/// Wire unused outliner items for compilation.
pub fn wire_unused_outliner_items() {
    // Reference unused OutlinerRow fields
    let row = OutlinerRow {
        item_id: uuid::Uuid::new_v4(),
        depth: 0,
        expanded: false,
        values: std::collections::HashMap::new(),
    };
    let _ = row.expanded;
    let _ = row.values;

    // Reference unused OutlinerItem fields
    let item = OutlinerItem {
        id: uuid::Uuid::new_v4(),
        title: String::new(),
        synopsis: String::new(),
        label: String::new(),
        status: String::new(),
        word_count: 0,
        target_word_count: None,
        created: chrono::Utc::now(),
        modified: chrono::Utc::now(),
        include_in_compile: true,
        children: vec![],
    };
    let _ = item.synopsis;
}
