/// Actions that can be performed in the editor
#[derive(Debug, Clone)]
pub enum EditorAction {
    /// Text was edited via the iced text_editor
    Edit(iced::widget::text_editor::Action),
    /// Toggle bold on selection
    ToggleBold,
    /// Toggle italic on selection
    ToggleItalic,
    /// Toggle underline on selection
    ToggleUnderline,
    /// Undo last change
    Undo,
    /// Redo last undone change
    Redo,
    /// Select all text
    SelectAll,
    /// Insert text at cursor
    Insert(String),
    /// Find text
    Find(String),
    /// Replace text
    Replace { find: String, replace: String },
    /// Replace all occurrences
    ReplaceAll { find: String, replace: String },
}
