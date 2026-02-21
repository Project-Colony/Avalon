/// Actions that can be performed in the editor
#[derive(Debug, Clone)]
pub enum EditorAction {
    /// Text was edited via the iced text_editor
    Edit(iced::widget::text_editor::Action),
    /// Toggle bold on selection (**text**)
    ToggleBold,
    /// Toggle italic on selection (*text*)
    ToggleItalic,
    /// Toggle underline on selection
    ToggleUnderline,
    /// Toggle strikethrough on selection (~~text~~)
    ToggleStrikethrough,
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
    /// Insert a heading (level 1-6)
    InsertHeading(u8),
    /// Insert a block quote
    InsertBlockQuote,
    /// Insert a horizontal rule
    InsertHorizontalRule,
    /// Insert a footnote reference
    InsertFootnote(String),
    /// Insert a code block with optional language
    InsertCodeBlock(Option<String>),
    /// Insert a link
    InsertLink { text: String, url: String },
    /// Insert an image reference
    InsertImage { alt: String, path: String },
    /// Insert a comment (invisible in output)
    InsertComment(String),
    /// Wrap selection in custom markup
    WrapSelection { prefix: String, suffix: String },
    /// Convert text to uppercase
    ToUppercase,
    /// Convert text to lowercase
    ToLowercase,
    /// Convert text to title case
    ToTitleCase,
    /// Move cursor to line start
    MoveToLineStart,
    /// Move cursor to line end
    MoveToLineEnd,
    /// Move cursor to document start
    MoveToDocStart,
    /// Move cursor to document end
    MoveToDocEnd,
    /// Move cursor one word forward
    MoveWordForward,
    /// Move cursor one word backward
    MoveWordBackward,
    /// Select the current word under cursor
    SelectWord,
    /// Select the current line
    SelectLine,
    /// Select the current paragraph
    SelectParagraph,
    /// Delete the current line
    DeleteLine,
    /// Duplicate the current line
    DuplicateLine,
    /// Move current line up
    MoveLineUp,
    /// Move current line down
    MoveLineDown,
    /// Join current line with next
    JoinLines,
    /// Indent current line or selection
    Indent,
    /// Unindent current line or selection
    Unindent,
    /// Set script element type for screenplay mode
    SetScriptElement(String),
    /// Insert page break marker
    InsertPageBreak,
    /// Go to a specific line number
    GoToLine(usize),
    /// Transpose characters at cursor
    TransposeChars,
    /// Sort selected lines alphabetically
    SortLines,
    /// Remove duplicate lines in selection
    RemoveDuplicateLines,
    /// Toggle line/inline comment
    ToggleComment,
    /// Insert a list item (bullet or numbered)
    InsertListItem(ListStyle),
    /// Insert a table with given dimensions
    InsertTable { rows: usize, cols: usize },
    /// Smart paste (clean up formatting from clipboard)
    SmartPaste(String),
    /// Insert current date/time
    InsertDateTime(DateTimeFormat),
    /// Toggle focus mode (highlight current sentence)
    ToggleFocusMode,
}

/// List style for InsertListItem action
#[derive(Debug, Clone)]
pub enum ListStyle {
    Bullet,
    Numbered,
    Checkbox,
}

/// DateTime format for InsertDateTime action
#[derive(Debug, Clone)]
pub enum DateTimeFormat {
    DateOnly,
    TimeOnly,
    DateTime,
    Iso8601,
}

impl EditorAction {
    /// Human-readable description of the action
    pub fn description(&self) -> &str {
        match self {
            EditorAction::Edit(_) => "Edit text",
            EditorAction::ToggleBold => "Toggle bold",
            EditorAction::ToggleItalic => "Toggle italic",
            EditorAction::ToggleUnderline => "Toggle underline",
            EditorAction::ToggleStrikethrough => "Toggle strikethrough",
            EditorAction::Undo => "Undo",
            EditorAction::Redo => "Redo",
            EditorAction::SelectAll => "Select all",
            EditorAction::Insert(_) => "Insert text",
            EditorAction::Find(_) => "Find",
            EditorAction::Replace { .. } => "Replace",
            EditorAction::ReplaceAll { .. } => "Replace all",
            EditorAction::InsertHeading(_) => "Insert heading",
            EditorAction::InsertBlockQuote => "Insert block quote",
            EditorAction::InsertHorizontalRule => "Insert horizontal rule",
            EditorAction::InsertFootnote(_) => "Insert footnote",
            EditorAction::InsertCodeBlock(_) => "Insert code block",
            EditorAction::InsertLink { .. } => "Insert link",
            EditorAction::InsertImage { .. } => "Insert image",
            EditorAction::InsertComment(_) => "Insert comment",
            EditorAction::WrapSelection { .. } => "Wrap selection",
            EditorAction::ToUppercase => "To uppercase",
            EditorAction::ToLowercase => "To lowercase",
            EditorAction::ToTitleCase => "To title case",
            EditorAction::MoveToLineStart => "Move to line start",
            EditorAction::MoveToLineEnd => "Move to line end",
            EditorAction::MoveToDocStart => "Move to document start",
            EditorAction::MoveToDocEnd => "Move to document end",
            EditorAction::MoveWordForward => "Move word forward",
            EditorAction::MoveWordBackward => "Move word backward",
            EditorAction::SelectWord => "Select word",
            EditorAction::SelectLine => "Select line",
            EditorAction::SelectParagraph => "Select paragraph",
            EditorAction::DeleteLine => "Delete line",
            EditorAction::DuplicateLine => "Duplicate line",
            EditorAction::MoveLineUp => "Move line up",
            EditorAction::MoveLineDown => "Move line down",
            EditorAction::JoinLines => "Join lines",
            EditorAction::Indent => "Indent",
            EditorAction::Unindent => "Unindent",
            EditorAction::SetScriptElement(_) => "Set script element",
            EditorAction::InsertPageBreak => "Insert page break",
            EditorAction::GoToLine(_) => "Go to line",
            EditorAction::TransposeChars => "Transpose characters",
            EditorAction::SortLines => "Sort lines",
            EditorAction::RemoveDuplicateLines => "Remove duplicate lines",
            EditorAction::ToggleComment => "Toggle comment",
            EditorAction::InsertListItem(_) => "Insert list item",
            EditorAction::InsertTable { .. } => "Insert table",
            EditorAction::SmartPaste(_) => "Smart paste",
            EditorAction::InsertDateTime(_) => "Insert date/time",
            EditorAction::ToggleFocusMode => "Toggle focus mode",
        }
    }

    /// Whether this action modifies the document
    pub fn is_editing(&self) -> bool {
        !matches!(
            self,
            EditorAction::Find(_)
                | EditorAction::MoveToLineStart
                | EditorAction::MoveToLineEnd
                | EditorAction::MoveToDocStart
                | EditorAction::MoveToDocEnd
                | EditorAction::MoveWordForward
                | EditorAction::MoveWordBackward
                | EditorAction::SelectWord
                | EditorAction::SelectLine
                | EditorAction::SelectParagraph
                | EditorAction::SelectAll
                | EditorAction::GoToLine(_)
                | EditorAction::ToggleFocusMode
        )
    }

    /// Keyboard shortcut hint for the action
    pub fn shortcut_hint(&self) -> &str {
        match self {
            EditorAction::ToggleBold => "Ctrl+B",
            EditorAction::ToggleItalic => "Ctrl+I",
            EditorAction::ToggleUnderline => "Ctrl+U",
            EditorAction::Undo => "Ctrl+Z",
            EditorAction::Redo => "Ctrl+Shift+Z",
            EditorAction::SelectAll => "Ctrl+A",
            EditorAction::Find(_) => "Ctrl+F",
            EditorAction::Replace { .. } => "Ctrl+H",
            EditorAction::MoveToLineStart => "Home",
            EditorAction::MoveToLineEnd => "End",
            EditorAction::MoveToDocStart => "Ctrl+Home",
            EditorAction::MoveToDocEnd => "Ctrl+End",
            EditorAction::Indent => "Tab",
            EditorAction::Unindent => "Shift+Tab",
            EditorAction::DeleteLine => "Ctrl+Shift+K",
            EditorAction::DuplicateLine => "Ctrl+Shift+D",
            EditorAction::MoveLineUp => "Alt+Up",
            EditorAction::MoveLineDown => "Alt+Down",
            EditorAction::GoToLine(_) => "Ctrl+G",
            EditorAction::ToggleComment => "Ctrl+/",
            _ => "",
        }
    }
}
