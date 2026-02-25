#![allow(dead_code)] // Methods used by test code
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Errors that can occur during corkboard operations.
#[derive(Debug, Clone, PartialEq)]
pub enum CorkboardError {
    /// The specified card was not found on the corkboard.
    CardNotFound(Uuid),
}

impl std::fmt::Display for CorkboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CorkboardError::CardNotFound(id) => write!(f, "Card not found: {}", id),
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
                    z_order: i32::try_from(i).unwrap_or(i32::MAX),
                }
            })
            .collect();

        self.cards = layouts.clone();
        self.next_z = i32::try_from(item_ids.len()).unwrap_or(i32::MAX);
        layouts
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

    // -- Move tests ---------------------------------------------------------

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
    fn test_toggle_selection_add() {
        let mut state = default_state();
        let ids = make_ids(2);
        state.arrange_grid(&ids);

        state.toggle_selection(ids[0]);
        assert!(state.is_selected(&ids[0]));
        assert!(!state.is_selected(&ids[1]));
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

}
