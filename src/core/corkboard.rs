#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Errors that can occur during corkboard operations.
#[derive(Debug, Clone, PartialEq)]
pub enum CorkboardError {
    /// The specified card was not found on the corkboard.
    CardNotFound(Uuid),
    /// The requested dimensions are invalid (zero or negative).
    InvalidDimensions,
    /// The requested zoom level is out of range.
    InvalidZoom,
}

impl std::fmt::Display for CorkboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CorkboardError::CardNotFound(id) => write!(f, "Card not found: {}", id),
            CorkboardError::InvalidDimensions => write!(f, "Invalid card dimensions"),
            CorkboardError::InvalidZoom => write!(f, "Invalid zoom level"),
        }
    }
}

impl std::error::Error for CorkboardError {}

pub type Result<T> = std::result::Result<T, CorkboardError>;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// The layout mode for the corkboard.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LayoutMode {
    /// Cards are arranged in a regular grid.
    Grid,
    /// Cards can be placed at arbitrary positions.
    Freeform,
}

/// Predefined card sizes plus a custom option.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CardSize {
    Small,
    Medium,
    Large,
    Custom(f32, f32),
}

impl CardSize {
    /// Return (width, height) for the card size.
    pub fn dimensions(&self) -> (f32, f32) {
        match self {
            CardSize::Small => (150.0, 100.0),
            CardSize::Medium => (200.0, 150.0),
            CardSize::Large => (280.0, 200.0),
            CardSize::Custom(w, h) => (*w, *h),
        }
    }
}

/// Sort order for cards on the corkboard.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SortOrder {
    Manual,
    TitleAsc,
    TitleDesc,
    DateCreatedAsc,
    DateCreatedDesc,
    DateModifiedAsc,
    DateModifiedDesc,
    LabelColor,
    Status,
    WordCount,
}

/// An undoable corkboard operation.
#[derive(Debug, Clone, PartialEq)]
pub enum CorkboardAction {
    MoveCard {
        id: Uuid,
        old_x: f32,
        old_y: f32,
        new_x: f32,
        new_y: f32,
    },
    ResizeCard {
        id: Uuid,
        old_w: f32,
        old_h: f32,
        new_w: f32,
        new_h: f32,
    },
    ReorderCards {
        old_order: Vec<Uuid>,
        new_order: Vec<Uuid>,
    },
    ChangeSettings {
        description: String,
    },
    SelectCards {
        old_selection: Vec<Uuid>,
        new_selection: Vec<Uuid>,
    },
    PinCard {
        id: Uuid,
        pinned: bool,
    },
}

// ---------------------------------------------------------------------------
// Data structs
// ---------------------------------------------------------------------------

/// Global settings for the corkboard view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorkboardSettings {
    pub layout_mode: LayoutMode,
    pub card_size: CardSize,
    pub spacing: f32,
    pub columns: usize,
    pub sort_order: SortOrder,
    pub show_synopsis: bool,
    pub show_label_color: bool,
    pub show_status_stamp: bool,
    pub show_keyword_chips: bool,
}

impl Default for CorkboardSettings {
    fn default() -> Self {
        Self {
            layout_mode: LayoutMode::Grid,
            card_size: CardSize::Medium,
            spacing: 20.0,
            columns: 4,
            sort_order: SortOrder::Manual,
            show_synopsis: true,
            show_label_color: true,
            show_status_stamp: true,
            show_keyword_chips: false,
        }
    }
}

/// Layout information for a single card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardLayout {
    pub item_id: Uuid,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub z_order: i32,
}

/// Per-card visual appearance settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardAppearance {
    pub label_color: Option<String>,
    pub status_stamp: Option<String>,
    pub pinned: bool,
    pub opacity: f32,
}

impl Default for CardAppearance {
    fn default() -> Self {
        Self {
            label_color: None,
            status_stamp: None,
            pinned: false,
            opacity: 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// CorkboardState
// ---------------------------------------------------------------------------

/// The full state of a corkboard view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorkboardState {
    pub settings: CorkboardSettings,
    pub cards: Vec<CardLayout>,
    pub appearances: HashMap<Uuid, CardAppearance>,
    pub selected: Vec<Uuid>,
    pub zoom: f32,
    next_z: i32,
}

impl CorkboardState {
    /// Create a new, empty corkboard with the given settings.
    pub fn new(settings: CorkboardSettings) -> Self {
        Self {
            settings,
            cards: Vec::new(),
            appearances: HashMap::new(),
            selected: Vec::new(),
            zoom: 1.0,
            next_z: 1,
        }
    }

    // -- Layout -------------------------------------------------------------

    /// Compute grid positions for the given item IDs and replace the current
    /// card list. Returns the generated layouts.
    pub fn arrange_grid(&mut self, item_ids: &[Uuid]) -> Vec<CardLayout> {
        let (cw, ch) = self.settings.card_size.dimensions();
        let cols = self.settings.columns.max(1);
        let spacing = self.settings.spacing;

        let layouts: Vec<CardLayout> = item_ids
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let col = i % cols;
                let row = i / cols;
                CardLayout {
                    item_id: *id,
                    x: spacing + col as f32 * (cw + spacing),
                    y: spacing + row as f32 * (ch + spacing),
                    width: cw,
                    height: ch,
                    z_order: i as i32,
                }
            })
            .collect();

        self.cards = layouts.clone();
        self.next_z = item_ids.len() as i32;
        layouts
    }

    /// Set cards to freeform positions. Each tuple is (id, x, y).
    /// Width and height come from the current card size setting.
    pub fn arrange_freeform(&mut self, items: &[(Uuid, f32, f32)]) -> Vec<CardLayout> {
        let (cw, ch) = self.settings.card_size.dimensions();

        let layouts: Vec<CardLayout> = items
            .iter()
            .enumerate()
            .map(|(i, (id, x, y))| CardLayout {
                item_id: *id,
                x: *x,
                y: *y,
                width: cw,
                height: ch,
                z_order: i as i32,
            })
            .collect();

        self.cards = layouts.clone();
        self.next_z = items.len() as i32;
        layouts
    }

    /// Re-flow all unpinned cards into a clean grid, preserving the current
    /// item ordering.
    pub fn auto_arrange(&mut self) {
        let (cw, ch) = self.settings.card_size.dimensions();
        let cols = self.settings.columns.max(1);
        let spacing = self.settings.spacing;

        let mut slot = 0usize;
        for card in &mut self.cards {
            let pinned = self
                .appearances
                .get(&card.item_id)
                .map(|a| a.pinned)
                .unwrap_or(false);

            if !pinned {
                let col = slot % cols;
                let row = slot / cols;
                card.x = spacing + col as f32 * (cw + spacing);
                card.y = spacing + row as f32 * (ch + spacing);
                card.width = cw;
                card.height = ch;
                slot += 1;
            }
        }
    }

    /// Sort cards by the given order and re-arrange them in a grid.
    /// `titles` provides the display title for each item; it is used by
    /// title-based and other external sort keys.
    pub fn sort_cards(&mut self, order: SortOrder, titles: &HashMap<Uuid, String>) {
        // Borrow `appearances` separately so we can use it inside `cards.sort_by`.
        let appearances = &self.appearances;

        match order {
            SortOrder::Manual => { /* keep current order */ }
            SortOrder::TitleAsc => {
                self.cards.sort_by(|a, b| {
                    let ta = titles.get(&a.item_id).map(String::as_str).unwrap_or("");
                    let tb = titles.get(&b.item_id).map(String::as_str).unwrap_or("");
                    ta.to_lowercase().cmp(&tb.to_lowercase())
                });
            }
            SortOrder::TitleDesc => {
                self.cards.sort_by(|a, b| {
                    let ta = titles.get(&a.item_id).map(String::as_str).unwrap_or("");
                    let tb = titles.get(&b.item_id).map(String::as_str).unwrap_or("");
                    tb.to_lowercase().cmp(&ta.to_lowercase())
                });
            }
            SortOrder::LabelColor => {
                self.cards.sort_by(|a, b| {
                    let ca = appearances
                        .get(&a.item_id)
                        .and_then(|ap| ap.label_color.as_deref())
                        .unwrap_or("");
                    let cb = appearances
                        .get(&b.item_id)
                        .and_then(|ap| ap.label_color.as_deref())
                        .unwrap_or("");
                    ca.cmp(cb)
                });
            }
            SortOrder::Status => {
                self.cards.sort_by(|a, b| {
                    let sa = appearances
                        .get(&a.item_id)
                        .and_then(|ap| ap.status_stamp.as_deref())
                        .unwrap_or("");
                    let sb = appearances
                        .get(&b.item_id)
                        .and_then(|ap| ap.status_stamp.as_deref())
                        .unwrap_or("");
                    sa.cmp(sb)
                });
            }
            // For date- and word-count-based sorts we fall through to a
            // by-title sort since we do not have those values available here.
            _ => {
                self.cards.sort_by(|a, b| {
                    let ta = titles.get(&a.item_id).map(String::as_str).unwrap_or("");
                    let tb = titles.get(&b.item_id).map(String::as_str).unwrap_or("");
                    ta.to_lowercase().cmp(&tb.to_lowercase())
                });
            }
        }

        self.settings.sort_order = order;
        self.auto_arrange();
    }

    // -- Card manipulation --------------------------------------------------

    /// Move a card to a new position, returning the undoable action.
    pub fn move_card(&mut self, id: Uuid, new_x: f32, new_y: f32) -> Result<CorkboardAction> {
        let card = self
            .cards
            .iter_mut()
            .find(|c| c.item_id == id)
            .ok_or(CorkboardError::CardNotFound(id))?;

        let action = CorkboardAction::MoveCard {
            id,
            old_x: card.x,
            old_y: card.y,
            new_x,
            new_y,
        };

        card.x = new_x;
        card.y = new_y;
        Ok(action)
    }

    /// Resize a card, returning the undoable action.
    pub fn resize_card(&mut self, id: Uuid, w: f32, h: f32) -> Result<CorkboardAction> {
        if w <= 0.0 || h <= 0.0 {
            return Err(CorkboardError::InvalidDimensions);
        }

        let card = self
            .cards
            .iter_mut()
            .find(|c| c.item_id == id)
            .ok_or(CorkboardError::CardNotFound(id))?;

        let action = CorkboardAction::ResizeCard {
            id,
            old_w: card.width,
            old_h: card.height,
            new_w: w,
            new_h: h,
        };

        card.width = w;
        card.height = h;
        Ok(action)
    }

    /// Pin or unpin a card. Pinned cards are excluded from auto-arrange.
    pub fn pin_card(&mut self, id: Uuid, pinned: bool) -> Result<CorkboardAction> {
        // Ensure the card exists on the board.
        if !self.cards.iter().any(|c| c.item_id == id) {
            return Err(CorkboardError::CardNotFound(id));
        }

        let appearance = self.appearances.entry(id).or_default();
        appearance.pinned = pinned;

        Ok(CorkboardAction::PinCard { id, pinned })
    }

    // -- Z-ordering ---------------------------------------------------------

    /// Bring a card to the front (highest z-order).
    pub fn bring_to_front(&mut self, id: Uuid) -> Result<()> {
        if !self.cards.iter().any(|c| c.item_id == id) {
            return Err(CorkboardError::CardNotFound(id));
        }

        let z = self.next_z;
        self.next_z += 1;

        if let Some(card) = self.cards.iter_mut().find(|c| c.item_id == id) {
            card.z_order = z;
        }
        Ok(())
    }

    /// Send a card to the back (lowest z-order).
    pub fn send_to_back(&mut self, id: Uuid) -> Result<()> {
        if !self.cards.iter().any(|c| c.item_id == id) {
            return Err(CorkboardError::CardNotFound(id));
        }

        let min_z = self.cards.iter().map(|c| c.z_order).min().unwrap_or(0);
        if let Some(card) = self.cards.iter_mut().find(|c| c.item_id == id) {
            card.z_order = min_z - 1;
        }
        Ok(())
    }

    // -- Selection ----------------------------------------------------------

    /// Set the selection to the given IDs, replacing any previous selection.
    pub fn select_cards(&mut self, ids: &[Uuid]) {
        self.selected = ids.to_vec();
    }

    /// Select all cards on the board.
    pub fn select_all(&mut self) {
        self.selected = self.cards.iter().map(|c| c.item_id).collect();
    }

    /// Clear the selection.
    pub fn deselect_all(&mut self) {
        self.selected.clear();
    }

    /// Toggle a card in/out of the selection.
    pub fn toggle_selection(&mut self, id: Uuid) {
        if let Some(pos) = self.selected.iter().position(|i| *i == id) {
            self.selected.remove(pos);
        } else {
            self.selected.push(id);
        }
    }

    /// Return the IDs of all cards that intersect the given rectangle
    /// (marquee / lasso selection).
    pub fn cards_in_rect(&self, x: f32, y: f32, w: f32, h: f32) -> Vec<Uuid> {
        let rx2 = x + w;
        let ry2 = y + h;

        self.cards
            .iter()
            .filter(|c| {
                let cx2 = c.x + c.width;
                let cy2 = c.y + c.height;
                // AABB overlap test
                c.x < rx2 && cx2 > x && c.y < ry2 && cy2 > y
            })
            .map(|c| c.item_id)
            .collect()
    }

    // -- Zoom ---------------------------------------------------------------

    /// Set the zoom level. Must be in the range [0.1, 5.0].
    pub fn set_zoom(&mut self, level: f32) -> Result<()> {
        if !(0.1..=5.0).contains(&level) {
            return Err(CorkboardError::InvalidZoom);
        }
        self.zoom = level;
        Ok(())
    }

    // -- Appearance ---------------------------------------------------------

    /// Set the visual appearance for a card.
    pub fn set_card_appearance(&mut self, id: Uuid, appearance: CardAppearance) {
        self.appearances.insert(id, appearance);
    }

    // -- Queries ------------------------------------------------------------

    /// Return the bounding box (x, y, width, height) that encloses all cards.
    /// Returns (0, 0, 0, 0) when the board is empty.
    pub fn total_bounds(&self) -> (f32, f32, f32, f32) {
        if self.cards.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let min_x = self.cards.iter().map(|c| c.x).fold(f32::INFINITY, f32::min);
        let min_y = self.cards.iter().map(|c| c.y).fold(f32::INFINITY, f32::min);
        let max_x = self
            .cards
            .iter()
            .map(|c| c.x + c.width)
            .fold(f32::NEG_INFINITY, f32::max);
        let max_y = self
            .cards
            .iter()
            .map(|c| c.y + c.height)
            .fold(f32::NEG_INFINITY, f32::max);

        (min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// Hit-test: return the topmost card (highest z-order) at the point.
    pub fn card_at_point(&self, x: f32, y: f32) -> Option<Uuid> {
        self.cards
            .iter()
            .filter(|c| x >= c.x && x <= c.x + c.width && y >= c.y && y <= c.y + c.height)
            .max_by_key(|c| c.z_order)
            .map(|c| c.item_id)
    }

    /// Number of cards currently on the board.
    pub fn card_count(&self) -> usize {
        self.cards.len()
    }

    /// Whether a specific card exists on the board.
    pub fn has_card(&self, id: &Uuid) -> bool {
        self.cards.iter().any(|c| &c.item_id == id)
    }

    /// Get a reference to a card layout by ID.
    pub fn get_card(&self, id: &Uuid) -> Option<&CardLayout> {
        self.cards.iter().find(|c| &c.item_id == id)
    }

    /// Get the appearance for a card (returns default if not explicitly set).
    pub fn get_appearance(&self, id: &Uuid) -> CardAppearance {
        self.appearances.get(id).cloned().unwrap_or_default()
    }

    /// Check if a card is currently selected.
    pub fn is_selected(&self, id: &Uuid) -> bool {
        self.selected.contains(id)
    }

    /// Get the number of selected cards.
    pub fn selection_count(&self) -> usize {
        self.selected.len()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- Helpers ------------------------------------------------------------

    fn default_state() -> CorkboardState {
        CorkboardState::new(CorkboardSettings::default())
    }

    fn make_ids(n: usize) -> Vec<Uuid> {
        (0..n).map(|_| Uuid::new_v4()).collect()
    }

    // -- Grid layout tests --------------------------------------------------

    #[test]
    fn test_arrange_grid_empty() {
        let mut state = default_state();
        let layouts = state.arrange_grid(&[]);
        assert!(layouts.is_empty());
        assert_eq!(state.card_count(), 0);
    }

    #[test]
    fn test_arrange_grid_single_card() {
        let mut state = default_state();
        let ids = make_ids(1);
        let layouts = state.arrange_grid(&ids);

        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].item_id, ids[0]);
        // First card should be at (spacing, spacing)
        assert_eq!(layouts[0].x, state.settings.spacing);
        assert_eq!(layouts[0].y, state.settings.spacing);
    }

    #[test]
    fn test_arrange_grid_positions() {
        let mut state = default_state();
        state.settings.columns = 3;
        state.settings.spacing = 10.0;
        state.settings.card_size = CardSize::Small; // 150x100
        let ids = make_ids(6);
        let layouts = state.arrange_grid(&ids);

        assert_eq!(layouts.len(), 6);

        // Row 0: columns 0, 1, 2
        assert_eq!(layouts[0].x, 10.0);
        assert_eq!(layouts[1].x, 10.0 + 160.0); // 10 + (150 + 10)
        assert_eq!(layouts[2].x, 10.0 + 320.0);

        // Row 1 starts
        assert_eq!(layouts[3].y, 10.0 + 110.0); // 10 + (100 + 10)
    }

    #[test]
    fn test_arrange_grid_custom_card_size() {
        let mut state = default_state();
        state.settings.card_size = CardSize::Custom(100.0, 80.0);
        state.settings.columns = 2;
        state.settings.spacing = 5.0;
        let ids = make_ids(4);
        let layouts = state.arrange_grid(&ids);

        assert_eq!(layouts[0].width, 100.0);
        assert_eq!(layouts[0].height, 80.0);
        // Second column
        assert_eq!(layouts[1].x, 5.0 + 105.0);
        // Second row
        assert_eq!(layouts[2].y, 5.0 + 85.0);
    }

    #[test]
    fn test_arrange_grid_replaces_existing_cards() {
        let mut state = default_state();
        let ids_a = make_ids(3);
        state.arrange_grid(&ids_a);
        assert_eq!(state.card_count(), 3);

        let ids_b = make_ids(5);
        state.arrange_grid(&ids_b);
        assert_eq!(state.card_count(), 5);
    }

    // -- Freeform layout tests ----------------------------------------------

    #[test]
    fn test_arrange_freeform_empty() {
        let mut state = default_state();
        let layouts = state.arrange_freeform(&[]);
        assert!(layouts.is_empty());
    }

    #[test]
    fn test_arrange_freeform_positions() {
        let mut state = default_state();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let items = vec![(id1, 50.0, 100.0), (id2, 300.0, 200.0)];
        let layouts = state.arrange_freeform(&items);

        assert_eq!(layouts.len(), 2);
        assert_eq!(layouts[0].x, 50.0);
        assert_eq!(layouts[0].y, 100.0);
        assert_eq!(layouts[1].x, 300.0);
        assert_eq!(layouts[1].y, 200.0);
    }

    #[test]
    fn test_arrange_freeform_uses_card_size() {
        let mut state = default_state();
        state.settings.card_size = CardSize::Large; // 280x200
        let id = Uuid::new_v4();
        let layouts = state.arrange_freeform(&[(id, 0.0, 0.0)]);

        assert_eq!(layouts[0].width, 280.0);
        assert_eq!(layouts[0].height, 200.0);
    }

    // -- Move / resize tests ------------------------------------------------

    #[test]
    fn test_move_card_success() {
        let mut state = default_state();
        let ids = make_ids(2);
        state.arrange_grid(&ids);

        let action = state.move_card(ids[0], 999.0, 888.0).unwrap();
        assert!(matches!(action, CorkboardAction::MoveCard { new_x: 999.0, new_y: 888.0, .. }));

        let card = state.get_card(&ids[0]).unwrap();
        assert_eq!(card.x, 999.0);
        assert_eq!(card.y, 888.0);
    }

    #[test]
    fn test_move_card_not_found() {
        let mut state = default_state();
        let result = state.move_card(Uuid::new_v4(), 0.0, 0.0);
        assert!(matches!(result, Err(CorkboardError::CardNotFound(_))));
    }

    #[test]
    fn test_resize_card_success() {
        let mut state = default_state();
        let ids = make_ids(1);
        state.arrange_grid(&ids);

        let action = state.resize_card(ids[0], 300.0, 250.0).unwrap();
        assert!(matches!(action, CorkboardAction::ResizeCard { new_w: 300.0, new_h: 250.0, .. }));

        let card = state.get_card(&ids[0]).unwrap();
        assert_eq!(card.width, 300.0);
        assert_eq!(card.height, 250.0);
    }

    #[test]
    fn test_resize_card_invalid_dimensions() {
        let mut state = default_state();
        let ids = make_ids(1);
        state.arrange_grid(&ids);

        assert!(matches!(
            state.resize_card(ids[0], 0.0, 100.0),
            Err(CorkboardError::InvalidDimensions)
        ));
        assert!(matches!(
            state.resize_card(ids[0], 100.0, -5.0),
            Err(CorkboardError::InvalidDimensions)
        ));
    }

    #[test]
    fn test_resize_card_not_found() {
        let mut state = default_state();
        let result = state.resize_card(Uuid::new_v4(), 100.0, 100.0);
        assert!(matches!(result, Err(CorkboardError::CardNotFound(_))));
    }

    // -- Selection tests ----------------------------------------------------

    #[test]
    fn test_select_cards() {
        let mut state = default_state();
        let ids = make_ids(3);
        state.arrange_grid(&ids);

        state.select_cards(&ids[0..2]);
        assert_eq!(state.selection_count(), 2);
        assert!(state.is_selected(&ids[0]));
        assert!(state.is_selected(&ids[1]));
        assert!(!state.is_selected(&ids[2]));
    }

    #[test]
    fn test_select_all() {
        let mut state = default_state();
        let ids = make_ids(4);
        state.arrange_grid(&ids);

        state.select_all();
        assert_eq!(state.selection_count(), 4);
        for id in &ids {
            assert!(state.is_selected(id));
        }
    }

    #[test]
    fn test_deselect_all() {
        let mut state = default_state();
        let ids = make_ids(3);
        state.arrange_grid(&ids);

        state.select_all();
        assert_eq!(state.selection_count(), 3);

        state.deselect_all();
        assert_eq!(state.selection_count(), 0);
    }

    #[test]
    fn test_toggle_selection_add() {
        let mut state = default_state();
        let ids = make_ids(2);
        state.arrange_grid(&ids);

        state.toggle_selection(ids[0]);
        assert!(state.is_selected(&ids[0]));
        assert!(!state.is_selected(&ids[1]));
    }

    #[test]
    fn test_toggle_selection_remove() {
        let mut state = default_state();
        let ids = make_ids(2);
        state.arrange_grid(&ids);

        state.select_cards(&ids);
        state.toggle_selection(ids[0]);
        assert!(!state.is_selected(&ids[0]));
        assert!(state.is_selected(&ids[1]));
    }

    #[test]
    fn test_cards_in_rect() {
        let mut state = default_state();
        state.settings.card_size = CardSize::Custom(100.0, 100.0);
        state.settings.columns = 10;
        state.settings.spacing = 0.0;

        let ids = make_ids(3);
        // Place cards manually via freeform
        state.arrange_freeform(&[
            (ids[0], 0.0, 0.0),      // 0..100, 0..100
            (ids[1], 200.0, 0.0),     // 200..300, 0..100
            (ids[2], 50.0, 50.0),     // 50..150, 50..150
        ]);

        // Rectangle that overlaps cards 0 and 2 but not 1
        let found = state.cards_in_rect(0.0, 0.0, 160.0, 160.0);
        assert!(found.contains(&ids[0]));
        assert!(!found.contains(&ids[1]));
        assert!(found.contains(&ids[2]));
    }

    #[test]
    fn test_cards_in_rect_no_overlap() {
        let mut state = default_state();
        state.settings.card_size = CardSize::Custom(50.0, 50.0);
        let ids = make_ids(1);
        state.arrange_freeform(&[(ids[0], 0.0, 0.0)]); // 0..50, 0..50

        let found = state.cards_in_rect(100.0, 100.0, 50.0, 50.0);
        assert!(found.is_empty());
    }

    // -- Hit testing --------------------------------------------------------

    #[test]
    fn test_card_at_point_hit() {
        let mut state = default_state();
        state.settings.card_size = CardSize::Custom(100.0, 100.0);
        let ids = make_ids(1);
        state.arrange_freeform(&[(ids[0], 10.0, 10.0)]);

        assert_eq!(state.card_at_point(50.0, 50.0), Some(ids[0]));
    }

    #[test]
    fn test_card_at_point_miss() {
        let mut state = default_state();
        state.settings.card_size = CardSize::Custom(100.0, 100.0);
        let ids = make_ids(1);
        state.arrange_freeform(&[(ids[0], 10.0, 10.0)]);

        assert_eq!(state.card_at_point(500.0, 500.0), None);
    }

    #[test]
    fn test_card_at_point_z_order() {
        let mut state = default_state();
        state.settings.card_size = CardSize::Custom(100.0, 100.0);
        let ids = make_ids(2);
        // Two overlapping cards
        state.arrange_freeform(&[(ids[0], 0.0, 0.0), (ids[1], 50.0, 50.0)]);
        // ids[1] has higher z_order (index 1)

        // Point at (60, 60) overlaps both; should return the topmost
        assert_eq!(state.card_at_point(60.0, 60.0), Some(ids[1]));

        // Now bring ids[0] to front
        state.bring_to_front(ids[0]).unwrap();
        assert_eq!(state.card_at_point(60.0, 60.0), Some(ids[0]));
    }

    // -- Z-ordering tests ---------------------------------------------------

    #[test]
    fn test_bring_to_front() {
        let mut state = default_state();
        let ids = make_ids(3);
        state.arrange_grid(&ids);

        state.bring_to_front(ids[0]).unwrap();

        let z0 = state.get_card(&ids[0]).unwrap().z_order;
        let z1 = state.get_card(&ids[1]).unwrap().z_order;
        let z2 = state.get_card(&ids[2]).unwrap().z_order;
        assert!(z0 > z1);
        assert!(z0 > z2);
    }

    #[test]
    fn test_send_to_back() {
        let mut state = default_state();
        let ids = make_ids(3);
        state.arrange_grid(&ids);

        state.send_to_back(ids[2]).unwrap();

        let z0 = state.get_card(&ids[0]).unwrap().z_order;
        let z1 = state.get_card(&ids[1]).unwrap().z_order;
        let z2 = state.get_card(&ids[2]).unwrap().z_order;
        assert!(z2 < z0);
        assert!(z2 < z1);
    }

    #[test]
    fn test_bring_to_front_not_found() {
        let mut state = default_state();
        assert!(matches!(
            state.bring_to_front(Uuid::new_v4()),
            Err(CorkboardError::CardNotFound(_))
        ));
    }

    #[test]
    fn test_send_to_back_not_found() {
        let mut state = default_state();
        assert!(matches!(
            state.send_to_back(Uuid::new_v4()),
            Err(CorkboardError::CardNotFound(_))
        ));
    }

    // -- Zoom tests ---------------------------------------------------------

    #[test]
    fn test_set_zoom_valid() {
        let mut state = default_state();
        assert!(state.set_zoom(2.0).is_ok());
        assert_eq!(state.zoom, 2.0);
    }

    #[test]
    fn test_set_zoom_boundaries() {
        let mut state = default_state();
        assert!(state.set_zoom(0.1).is_ok());
        assert!(state.set_zoom(5.0).is_ok());
    }

    #[test]
    fn test_set_zoom_too_low() {
        let mut state = default_state();
        assert!(matches!(state.set_zoom(0.05), Err(CorkboardError::InvalidZoom)));
    }

    #[test]
    fn test_set_zoom_too_high() {
        let mut state = default_state();
        assert!(matches!(state.set_zoom(10.0), Err(CorkboardError::InvalidZoom)));
    }

    // -- Sorting tests ------------------------------------------------------

    #[test]
    fn test_sort_cards_title_asc() {
        let mut state = default_state();
        let ids = make_ids(3);
        state.arrange_grid(&ids);

        let mut titles = HashMap::new();
        titles.insert(ids[0], "Charlie".to_string());
        titles.insert(ids[1], "Alpha".to_string());
        titles.insert(ids[2], "Bravo".to_string());

        state.sort_cards(SortOrder::TitleAsc, &titles);

        assert_eq!(state.cards[0].item_id, ids[1]); // Alpha
        assert_eq!(state.cards[1].item_id, ids[2]); // Bravo
        assert_eq!(state.cards[2].item_id, ids[0]); // Charlie
    }

    #[test]
    fn test_sort_cards_title_desc() {
        let mut state = default_state();
        let ids = make_ids(3);
        state.arrange_grid(&ids);

        let mut titles = HashMap::new();
        titles.insert(ids[0], "Alpha".to_string());
        titles.insert(ids[1], "Charlie".to_string());
        titles.insert(ids[2], "Bravo".to_string());

        state.sort_cards(SortOrder::TitleDesc, &titles);

        assert_eq!(state.cards[0].item_id, ids[1]); // Charlie
        assert_eq!(state.cards[1].item_id, ids[2]); // Bravo
        assert_eq!(state.cards[2].item_id, ids[0]); // Alpha
    }

    #[test]
    fn test_sort_cards_manual_preserves_order() {
        let mut state = default_state();
        let ids = make_ids(3);
        state.arrange_grid(&ids);

        let original_order: Vec<Uuid> = state.cards.iter().map(|c| c.item_id).collect();
        state.sort_cards(SortOrder::Manual, &HashMap::new());
        let new_order: Vec<Uuid> = state.cards.iter().map(|c| c.item_id).collect();

        assert_eq!(original_order, new_order);
    }

    // -- Pin tests ----------------------------------------------------------

    #[test]
    fn test_pin_card() {
        let mut state = default_state();
        let ids = make_ids(2);
        state.arrange_grid(&ids);

        let action = state.pin_card(ids[0], true).unwrap();
        assert!(matches!(action, CorkboardAction::PinCard { pinned: true, .. }));
        assert!(state.get_appearance(&ids[0]).pinned);
    }

    #[test]
    fn test_pin_card_not_found() {
        let mut state = default_state();
        assert!(matches!(
            state.pin_card(Uuid::new_v4(), true),
            Err(CorkboardError::CardNotFound(_))
        ));
    }

    #[test]
    fn test_auto_arrange_skips_pinned() {
        let mut state = default_state();
        state.settings.card_size = CardSize::Custom(100.0, 100.0);
        state.settings.columns = 2;
        state.settings.spacing = 0.0;
        let ids = make_ids(3);
        state.arrange_grid(&ids);

        // Pin the first card and move it somewhere unusual
        state.pin_card(ids[0], true).unwrap();
        state.move_card(ids[0], 999.0, 999.0).unwrap();

        state.auto_arrange();

        // Pinned card should remain at its custom position
        let pinned = state.get_card(&ids[0]).unwrap();
        assert_eq!(pinned.x, 999.0);
        assert_eq!(pinned.y, 999.0);

        // Unpinned cards should be re-arranged starting from slot 0
        let c1 = state.get_card(&ids[1]).unwrap();
        assert_eq!(c1.x, 0.0);
        assert_eq!(c1.y, 0.0);
    }

    // -- Appearance tests ---------------------------------------------------

    #[test]
    fn test_set_card_appearance() {
        let mut state = default_state();
        let ids = make_ids(1);
        state.arrange_grid(&ids);

        let appearance = CardAppearance {
            label_color: Some("red".to_string()),
            status_stamp: Some("Draft".to_string()),
            pinned: false,
            opacity: 0.8,
        };
        state.set_card_appearance(ids[0], appearance.clone());

        let retrieved = state.get_appearance(&ids[0]);
        assert_eq!(retrieved.label_color, Some("red".to_string()));
        assert_eq!(retrieved.status_stamp, Some("Draft".to_string()));
        assert_eq!(retrieved.opacity, 0.8);
    }

    #[test]
    fn test_get_appearance_default() {
        let state = default_state();
        let fake_id = Uuid::new_v4();
        let appearance = state.get_appearance(&fake_id);
        assert_eq!(appearance.label_color, None);
        assert_eq!(appearance.opacity, 1.0);
        assert!(!appearance.pinned);
    }

    // -- Bounds tests -------------------------------------------------------

    #[test]
    fn test_total_bounds_empty() {
        let state = default_state();
        assert_eq!(state.total_bounds(), (0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_total_bounds_single_card() {
        let mut state = default_state();
        state.settings.card_size = CardSize::Custom(100.0, 80.0);
        let ids = make_ids(1);
        state.arrange_freeform(&[(ids[0], 10.0, 20.0)]);

        let (x, y, w, h) = state.total_bounds();
        assert_eq!(x, 10.0);
        assert_eq!(y, 20.0);
        assert_eq!(w, 100.0);
        assert_eq!(h, 80.0);
    }

    #[test]
    fn test_total_bounds_multiple_cards() {
        let mut state = default_state();
        state.settings.card_size = CardSize::Custom(100.0, 50.0);
        let ids = make_ids(2);
        state.arrange_freeform(&[(ids[0], 0.0, 0.0), (ids[1], 200.0, 100.0)]);

        let (x, y, w, h) = state.total_bounds();
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(w, 300.0); // 200 + 100
        assert_eq!(h, 150.0); // 100 + 50
    }

    // -- CardSize tests -----------------------------------------------------

    #[test]
    fn test_card_size_dimensions() {
        assert_eq!(CardSize::Small.dimensions(), (150.0, 100.0));
        assert_eq!(CardSize::Medium.dimensions(), (200.0, 150.0));
        assert_eq!(CardSize::Large.dimensions(), (280.0, 200.0));
        assert_eq!(CardSize::Custom(42.0, 17.0).dimensions(), (42.0, 17.0));
    }

    // -- Settings defaults test ---------------------------------------------

    #[test]
    fn test_default_settings() {
        let settings = CorkboardSettings::default();
        assert_eq!(settings.layout_mode, LayoutMode::Grid);
        assert_eq!(settings.card_size, CardSize::Medium);
        assert_eq!(settings.spacing, 20.0);
        assert_eq!(settings.columns, 4);
        assert_eq!(settings.sort_order, SortOrder::Manual);
        assert!(settings.show_synopsis);
        assert!(settings.show_label_color);
        assert!(settings.show_status_stamp);
        assert!(!settings.show_keyword_chips);
    }

    // -- Error display test -------------------------------------------------

    #[test]
    fn test_error_display() {
        let id = Uuid::new_v4();
        let err = CorkboardError::CardNotFound(id);
        let msg = format!("{}", err);
        assert!(msg.contains("Card not found"));

        let err2 = CorkboardError::InvalidDimensions;
        assert!(format!("{}", err2).contains("Invalid card dimensions"));

        let err3 = CorkboardError::InvalidZoom;
        assert!(format!("{}", err3).contains("Invalid zoom level"));
    }

    // -- has_card / get_card tests ------------------------------------------

    #[test]
    fn test_has_card() {
        let mut state = default_state();
        let ids = make_ids(2);
        state.arrange_grid(&ids);

        assert!(state.has_card(&ids[0]));
        assert!(!state.has_card(&Uuid::new_v4()));
    }

    #[test]
    fn test_get_card() {
        let mut state = default_state();
        let ids = make_ids(1);
        state.arrange_grid(&ids);

        let card = state.get_card(&ids[0]).unwrap();
        assert_eq!(card.item_id, ids[0]);
        assert!(state.get_card(&Uuid::new_v4()).is_none());
    }

    // -- Sort by label color ------------------------------------------------

    #[test]
    fn test_sort_cards_by_label_color() {
        let mut state = default_state();
        let ids = make_ids(3);
        state.arrange_grid(&ids);

        state.set_card_appearance(ids[0], CardAppearance {
            label_color: Some("red".to_string()),
            ..CardAppearance::default()
        });
        state.set_card_appearance(ids[1], CardAppearance {
            label_color: Some("blue".to_string()),
            ..CardAppearance::default()
        });
        state.set_card_appearance(ids[2], CardAppearance {
            label_color: Some("green".to_string()),
            ..CardAppearance::default()
        });

        state.sort_cards(SortOrder::LabelColor, &HashMap::new());

        // Sorted alphabetically by label: blue, green, red
        assert_eq!(state.cards[0].item_id, ids[1]); // blue
        assert_eq!(state.cards[1].item_id, ids[2]); // green
        assert_eq!(state.cards[2].item_id, ids[0]); // red
    }

    // -- Edge cases ---------------------------------------------------------

    #[test]
    fn test_select_cards_replaces_previous() {
        let mut state = default_state();
        let ids = make_ids(4);
        state.arrange_grid(&ids);

        state.select_cards(&ids[0..2]);
        assert_eq!(state.selection_count(), 2);

        state.select_cards(&ids[2..4]);
        assert_eq!(state.selection_count(), 2);
        assert!(!state.is_selected(&ids[0]));
        assert!(state.is_selected(&ids[2]));
    }

    #[test]
    fn test_card_at_point_on_edge() {
        let mut state = default_state();
        state.settings.card_size = CardSize::Custom(100.0, 100.0);
        let ids = make_ids(1);
        state.arrange_freeform(&[(ids[0], 0.0, 0.0)]);

        // Exact corners should hit
        assert_eq!(state.card_at_point(0.0, 0.0), Some(ids[0]));
        assert_eq!(state.card_at_point(100.0, 100.0), Some(ids[0]));
        // Just outside should miss
        assert_eq!(state.card_at_point(100.1, 50.0), None);
    }

    #[test]
    fn test_grid_single_column() {
        let mut state = default_state();
        state.settings.columns = 1;
        state.settings.spacing = 0.0;
        state.settings.card_size = CardSize::Custom(100.0, 50.0);
        let ids = make_ids(3);
        let layouts = state.arrange_grid(&ids);

        // All cards should be in column 0
        for layout in &layouts {
            assert_eq!(layout.x, 0.0);
        }
        assert_eq!(layouts[0].y, 0.0);
        assert_eq!(layouts[1].y, 50.0);
        assert_eq!(layouts[2].y, 100.0);
    }

    #[test]
    fn test_move_card_returns_old_position() {
        let mut state = default_state();
        let ids = make_ids(1);
        state.arrange_grid(&ids);

        let original_x = state.get_card(&ids[0]).unwrap().x;
        let original_y = state.get_card(&ids[0]).unwrap().y;

        let action = state.move_card(ids[0], 500.0, 600.0).unwrap();
        if let CorkboardAction::MoveCard { old_x, old_y, .. } = action {
            assert_eq!(old_x, original_x);
            assert_eq!(old_y, original_y);
        } else {
            panic!("Expected MoveCard action");
        }
    }

    #[test]
    fn test_resize_card_returns_old_size() {
        let mut state = default_state();
        let ids = make_ids(1);
        state.arrange_grid(&ids);

        let original_w = state.get_card(&ids[0]).unwrap().width;
        let original_h = state.get_card(&ids[0]).unwrap().height;

        let action = state.resize_card(ids[0], 400.0, 300.0).unwrap();
        if let CorkboardAction::ResizeCard { old_w, old_h, .. } = action {
            assert_eq!(old_w, original_w);
            assert_eq!(old_h, original_h);
        } else {
            panic!("Expected ResizeCard action");
        }
    }
}
