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

    /// Whether this action is a cursor movement (no text modification)
    pub fn is_movement(&self) -> bool {
        matches!(
            self,
            EditorAction::MoveToLineStart
                | EditorAction::MoveToLineEnd
                | EditorAction::MoveToDocStart
                | EditorAction::MoveToDocEnd
                | EditorAction::MoveWordForward
                | EditorAction::MoveWordBackward
                | EditorAction::GoToLine(_)
        )
    }

    /// Whether this action is a selection action
    pub fn is_selection(&self) -> bool {
        matches!(
            self,
            EditorAction::SelectAll
                | EditorAction::SelectWord
                | EditorAction::SelectLine
                | EditorAction::SelectParagraph
        )
    }

    /// Whether this action is a formatting toggle
    pub fn is_formatting(&self) -> bool {
        matches!(
            self,
            EditorAction::ToggleBold
                | EditorAction::ToggleItalic
                | EditorAction::ToggleUnderline
                | EditorAction::ToggleStrikethrough
                | EditorAction::ToggleComment
                | EditorAction::ToUppercase
                | EditorAction::ToLowercase
                | EditorAction::ToTitleCase
        )
    }

    /// Whether this action inserts content
    pub fn is_insertion(&self) -> bool {
        matches!(
            self,
            EditorAction::Insert(_)
                | EditorAction::InsertHeading(_)
                | EditorAction::InsertBlockQuote
                | EditorAction::InsertHorizontalRule
                | EditorAction::InsertFootnote(_)
                | EditorAction::InsertCodeBlock(_)
                | EditorAction::InsertLink { .. }
                | EditorAction::InsertImage { .. }
                | EditorAction::InsertComment(_)
                | EditorAction::InsertPageBreak
                | EditorAction::InsertListItem(_)
                | EditorAction::InsertTable { .. }
                | EditorAction::InsertDateTime(_)
                | EditorAction::SmartPaste(_)
        )
    }

    /// Whether this action can be undone
    pub fn is_undoable(&self) -> bool {
        self.is_editing()
    }

    /// Category string for grouping in menus
    pub fn category(&self) -> &str {
        if self.is_movement() {
            "Navigation"
        } else if self.is_selection() {
            "Selection"
        } else if self.is_formatting() {
            "Formatting"
        } else if self.is_insertion() {
            "Insert"
        } else if matches!(self, EditorAction::Undo | EditorAction::Redo) {
            "History"
        } else if matches!(self, EditorAction::Find(_) | EditorAction::Replace { .. } | EditorAction::ReplaceAll { .. }) {
            "Find & Replace"
        } else {
            "Edit"
        }
    }
}

impl ListStyle {
    /// Markdown prefix for this list style
    pub fn prefix(&self) -> &str {
        match self {
            ListStyle::Bullet => "- ",
            ListStyle::Numbered => "1. ",
            ListStyle::Checkbox => "- [ ] ",
        }
    }

    /// Human-readable label
    pub fn label(&self) -> &str {
        match self {
            ListStyle::Bullet => "Bullet List",
            ListStyle::Numbered => "Numbered List",
            ListStyle::Checkbox => "Checkbox List",
        }
    }
}

impl DateTimeFormat {
    /// Format the current date/time according to this format
    pub fn format_now(&self) -> String {
        let now = chrono::Local::now();
        match self {
            DateTimeFormat::DateOnly => now.format("%Y-%m-%d").to_string(),
            DateTimeFormat::TimeOnly => now.format("%H:%M").to_string(),
            DateTimeFormat::DateTime => now.format("%Y-%m-%d %H:%M").to_string(),
            DateTimeFormat::Iso8601 => now.format("%Y-%m-%dT%H:%M:%S%z").to_string(),
        }
    }

    /// Human-readable label
    pub fn label(&self) -> &str {
        match self {
            DateTimeFormat::DateOnly => "Date Only",
            DateTimeFormat::TimeOnly => "Time Only",
            DateTimeFormat::DateTime => "Date & Time",
            DateTimeFormat::Iso8601 => "ISO 8601",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_description_coverage() {
        // Test that all action variants have non-empty descriptions
        let actions = vec![
            EditorAction::ToggleBold,
            EditorAction::ToggleItalic,
            EditorAction::ToggleUnderline,
            EditorAction::ToggleStrikethrough,
            EditorAction::Undo,
            EditorAction::Redo,
            EditorAction::SelectAll,
            EditorAction::Insert("test".to_string()),
            EditorAction::Find("query".to_string()),
            EditorAction::Replace { find: "a".to_string(), replace: "b".to_string() },
            EditorAction::ReplaceAll { find: "a".to_string(), replace: "b".to_string() },
            EditorAction::InsertHeading(1),
            EditorAction::InsertBlockQuote,
            EditorAction::InsertHorizontalRule,
            EditorAction::InsertFootnote("ref".to_string()),
            EditorAction::InsertCodeBlock(None),
            EditorAction::InsertLink { text: "text".to_string(), url: "url".to_string() },
            EditorAction::InsertImage { alt: "alt".to_string(), path: "path".to_string() },
            EditorAction::InsertComment("comment".to_string()),
            EditorAction::WrapSelection { prefix: "[".to_string(), suffix: "]".to_string() },
            EditorAction::ToUppercase,
            EditorAction::ToLowercase,
            EditorAction::ToTitleCase,
            EditorAction::MoveToLineStart,
            EditorAction::MoveToLineEnd,
            EditorAction::MoveToDocStart,
            EditorAction::MoveToDocEnd,
            EditorAction::MoveWordForward,
            EditorAction::MoveWordBackward,
            EditorAction::SelectWord,
            EditorAction::SelectLine,
            EditorAction::SelectParagraph,
            EditorAction::DeleteLine,
            EditorAction::DuplicateLine,
            EditorAction::MoveLineUp,
            EditorAction::MoveLineDown,
            EditorAction::JoinLines,
            EditorAction::Indent,
            EditorAction::Unindent,
            EditorAction::SetScriptElement("Scene Heading".to_string()),
            EditorAction::InsertPageBreak,
            EditorAction::GoToLine(1),
            EditorAction::TransposeChars,
            EditorAction::SortLines,
            EditorAction::RemoveDuplicateLines,
            EditorAction::ToggleComment,
            EditorAction::InsertListItem(ListStyle::Bullet),
            EditorAction::InsertTable { rows: 3, cols: 3 },
            EditorAction::SmartPaste("text".to_string()),
            EditorAction::InsertDateTime(DateTimeFormat::DateOnly),
            EditorAction::ToggleFocusMode,
        ];

        for action in &actions {
            assert!(!action.description().is_empty(), "Description empty for {:?}", action);
        }
    }

    #[test]
    fn test_is_editing() {
        assert!(EditorAction::ToggleBold.is_editing());
        assert!(EditorAction::Insert("x".to_string()).is_editing());
        assert!(EditorAction::DeleteLine.is_editing());
        assert!(!EditorAction::Find("x".to_string()).is_editing());
        assert!(!EditorAction::SelectAll.is_editing());
        assert!(!EditorAction::MoveToLineStart.is_editing());
        assert!(!EditorAction::GoToLine(1).is_editing());
        assert!(!EditorAction::ToggleFocusMode.is_editing());
    }

    #[test]
    fn test_is_movement() {
        assert!(EditorAction::MoveToLineStart.is_movement());
        assert!(EditorAction::MoveToLineEnd.is_movement());
        assert!(EditorAction::MoveToDocStart.is_movement());
        assert!(EditorAction::MoveToDocEnd.is_movement());
        assert!(EditorAction::MoveWordForward.is_movement());
        assert!(EditorAction::MoveWordBackward.is_movement());
        assert!(EditorAction::GoToLine(5).is_movement());
        assert!(!EditorAction::ToggleBold.is_movement());
        assert!(!EditorAction::SelectAll.is_movement());
    }

    #[test]
    fn test_is_selection() {
        assert!(EditorAction::SelectAll.is_selection());
        assert!(EditorAction::SelectWord.is_selection());
        assert!(EditorAction::SelectLine.is_selection());
        assert!(EditorAction::SelectParagraph.is_selection());
        assert!(!EditorAction::ToggleBold.is_selection());
        assert!(!EditorAction::MoveToLineStart.is_selection());
    }

    #[test]
    fn test_is_formatting() {
        assert!(EditorAction::ToggleBold.is_formatting());
        assert!(EditorAction::ToggleItalic.is_formatting());
        assert!(EditorAction::ToggleUnderline.is_formatting());
        assert!(EditorAction::ToggleStrikethrough.is_formatting());
        assert!(EditorAction::ToggleComment.is_formatting());
        assert!(EditorAction::ToUppercase.is_formatting());
        assert!(EditorAction::ToLowercase.is_formatting());
        assert!(EditorAction::ToTitleCase.is_formatting());
        assert!(!EditorAction::Insert("x".to_string()).is_formatting());
    }

    #[test]
    fn test_is_insertion() {
        assert!(EditorAction::Insert("x".to_string()).is_insertion());
        assert!(EditorAction::InsertHeading(1).is_insertion());
        assert!(EditorAction::InsertBlockQuote.is_insertion());
        assert!(EditorAction::InsertHorizontalRule.is_insertion());
        assert!(EditorAction::InsertFootnote("ref".to_string()).is_insertion());
        assert!(EditorAction::InsertCodeBlock(None).is_insertion());
        assert!(EditorAction::InsertPageBreak.is_insertion());
        assert!(EditorAction::InsertListItem(ListStyle::Bullet).is_insertion());
        assert!(EditorAction::InsertTable { rows: 2, cols: 2 }.is_insertion());
        assert!(EditorAction::SmartPaste("x".to_string()).is_insertion());
        assert!(EditorAction::InsertDateTime(DateTimeFormat::DateOnly).is_insertion());
        assert!(!EditorAction::ToggleBold.is_insertion());
    }

    #[test]
    fn test_is_undoable() {
        assert!(EditorAction::ToggleBold.is_undoable());
        assert!(EditorAction::Insert("x".to_string()).is_undoable());
        assert!(!EditorAction::Find("x".to_string()).is_undoable());
        assert!(!EditorAction::SelectAll.is_undoable());
    }

    #[test]
    fn test_category() {
        assert_eq!(EditorAction::MoveToLineStart.category(), "Navigation");
        assert_eq!(EditorAction::SelectAll.category(), "Selection");
        assert_eq!(EditorAction::ToggleBold.category(), "Formatting");
        assert_eq!(EditorAction::Insert("x".to_string()).category(), "Insert");
        assert_eq!(EditorAction::Undo.category(), "History");
        assert_eq!(EditorAction::Redo.category(), "History");
        assert_eq!(EditorAction::Find("x".to_string()).category(), "Find & Replace");
        assert_eq!(EditorAction::DeleteLine.category(), "Edit");
    }

    #[test]
    fn test_shortcut_hint() {
        assert_eq!(EditorAction::ToggleBold.shortcut_hint(), "Ctrl+B");
        assert_eq!(EditorAction::ToggleItalic.shortcut_hint(), "Ctrl+I");
        assert_eq!(EditorAction::Undo.shortcut_hint(), "Ctrl+Z");
        assert_eq!(EditorAction::Redo.shortcut_hint(), "Ctrl+Shift+Z");
        assert_eq!(EditorAction::SelectAll.shortcut_hint(), "Ctrl+A");
        assert_eq!(EditorAction::Indent.shortcut_hint(), "Tab");
        assert_eq!(EditorAction::DeleteLine.shortcut_hint(), "Ctrl+Shift+K");
        // Actions without shortcuts return ""
        assert_eq!(EditorAction::InsertBlockQuote.shortcut_hint(), "");
    }

    #[test]
    fn test_list_style_prefix() {
        assert_eq!(ListStyle::Bullet.prefix(), "- ");
        assert_eq!(ListStyle::Numbered.prefix(), "1. ");
        assert_eq!(ListStyle::Checkbox.prefix(), "- [ ] ");
    }

    #[test]
    fn test_list_style_label() {
        assert_eq!(ListStyle::Bullet.label(), "Bullet List");
        assert_eq!(ListStyle::Numbered.label(), "Numbered List");
        assert_eq!(ListStyle::Checkbox.label(), "Checkbox List");
    }

    #[test]
    fn test_datetime_format_now() {
        let date = DateTimeFormat::DateOnly.format_now();
        assert!(date.contains("-"), "Date should contain dashes: {}", date);
        assert_eq!(date.len(), 10); // YYYY-MM-DD

        let time = DateTimeFormat::TimeOnly.format_now();
        assert!(time.contains(":"), "Time should contain colon: {}", time);

        let datetime = DateTimeFormat::DateTime.format_now();
        assert!(datetime.contains(" "), "DateTime should contain space: {}", datetime);

        let iso = DateTimeFormat::Iso8601.format_now();
        assert!(iso.contains("T"), "ISO should contain T: {}", iso);
    }

    #[test]
    fn test_datetime_format_label() {
        assert_eq!(DateTimeFormat::DateOnly.label(), "Date Only");
        assert_eq!(DateTimeFormat::TimeOnly.label(), "Time Only");
        assert_eq!(DateTimeFormat::DateTime.label(), "Date & Time");
        assert_eq!(DateTimeFormat::Iso8601.label(), "ISO 8601");
    }
}
