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

    /// Whether this action requires a text selection to be meaningful
    pub fn requires_selection(&self) -> bool {
        matches!(
            self,
            EditorAction::WrapSelection { .. }
                | EditorAction::ToUppercase
                | EditorAction::ToLowercase
                | EditorAction::ToTitleCase
                | EditorAction::SortLines
                | EditorAction::RemoveDuplicateLines
        )
    }

    /// Menu path for nested menu organization (e.g., "Format > Case")
    pub fn menu_path(&self) -> Vec<&str> {
        match self {
            EditorAction::ToggleBold
            | EditorAction::ToggleItalic
            | EditorAction::ToggleUnderline
            | EditorAction::ToggleStrikethrough => vec!["Format", "Style"],

            EditorAction::ToUppercase
            | EditorAction::ToLowercase
            | EditorAction::ToTitleCase => vec!["Format", "Case"],

            EditorAction::InsertHeading(_) => vec!["Insert", "Heading"],
            EditorAction::InsertListItem(_) => vec!["Insert", "List"],

            EditorAction::MoveLineUp
            | EditorAction::MoveLineDown
            | EditorAction::DeleteLine
            | EditorAction::DuplicateLine
            | EditorAction::JoinLines => vec!["Edit", "Line"],

            EditorAction::SortLines
            | EditorAction::RemoveDuplicateLines => vec!["Edit", "Lines"],

            EditorAction::Indent | EditorAction::Unindent => vec!["Edit", "Indentation"],

            _ => vec![self.category()],
        }
    }

    /// Get a combined batch description for a list of actions
    pub fn batch_description(actions: &[EditorAction]) -> String {
        if actions.is_empty() {
            return "No actions".to_string();
        }
        if actions.len() == 1 {
            return actions[0].description().to_string();
        }
        let editing: usize = actions.iter().filter(|a| a.is_editing()).count();
        let nav: usize = actions.iter().filter(|a| a.is_movement()).count();
        format!("{} actions ({} edits, {} navigation)", actions.len(), editing, nav)
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

    /// All available list styles
    pub fn all() -> Vec<Self> {
        vec![ListStyle::Bullet, ListStyle::Numbered, ListStyle::Checkbox]
    }

    /// Continuation prefix for subsequent items (numbered lists increment)
    pub fn continuation_prefix(&self, index: usize) -> String {
        match self {
            ListStyle::Bullet => "- ".to_string(),
            ListStyle::Numbered => format!("{}. ", index + 1),
            ListStyle::Checkbox => "- [ ] ".to_string(),
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

    /// All available date/time formats
    pub fn all() -> Vec<Self> {
        vec![
            DateTimeFormat::DateOnly,
            DateTimeFormat::TimeOnly,
            DateTimeFormat::DateTime,
            DateTimeFormat::Iso8601,
        ]
    }

    /// Format string pattern (for display to users)
    pub fn pattern(&self) -> &str {
        match self {
            DateTimeFormat::DateOnly => "YYYY-MM-DD",
            DateTimeFormat::TimeOnly => "HH:MM",
            DateTimeFormat::DateTime => "YYYY-MM-DD HH:MM",
            DateTimeFormat::Iso8601 => "YYYY-MM-DDTHH:MM:SS+ZZZZ",
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

    #[test]
    fn test_replace_is_editing() {
        assert!(EditorAction::Replace { find: "a".into(), replace: "b".into() }.is_editing());
        assert!(EditorAction::ReplaceAll { find: "a".into(), replace: "b".into() }.is_editing());
    }

    #[test]
    fn test_replace_category() {
        assert_eq!(EditorAction::Replace { find: "a".into(), replace: "b".into() }.category(), "Find & Replace");
        assert_eq!(EditorAction::ReplaceAll { find: "a".into(), replace: "b".into() }.category(), "Find & Replace");
    }

    #[test]
    fn test_wrap_selection_is_editing() {
        assert!(EditorAction::WrapSelection { prefix: "[".into(), suffix: "]".into() }.is_editing());
    }

    #[test]
    fn test_insert_link_is_insertion() {
        assert!(EditorAction::InsertLink { text: "t".into(), url: "u".into() }.is_insertion());
        assert!(EditorAction::InsertImage { alt: "a".into(), path: "p".into() }.is_insertion());
        assert!(EditorAction::InsertComment("c".into()).is_insertion());
    }

    #[test]
    fn test_smart_paste_is_insertion() {
        assert!(EditorAction::SmartPaste("pasted text".into()).is_insertion());
    }

    #[test]
    fn test_movement_is_not_editing() {
        assert!(!EditorAction::MoveToDocStart.is_editing());
        assert!(!EditorAction::MoveToDocEnd.is_editing());
        assert!(!EditorAction::MoveWordForward.is_editing());
        assert!(!EditorAction::MoveWordBackward.is_editing());
    }

    #[test]
    fn test_sort_lines_is_editing() {
        assert!(EditorAction::SortLines.is_editing());
        assert!(EditorAction::RemoveDuplicateLines.is_editing());
        assert!(EditorAction::TransposeChars.is_editing());
        assert!(EditorAction::JoinLines.is_editing());
    }

    #[test]
    fn test_indent_unindent_editing() {
        assert!(EditorAction::Indent.is_editing());
        assert!(EditorAction::Unindent.is_editing());
    }

    #[test]
    fn test_insert_table_is_insertion() {
        assert!(EditorAction::InsertTable { rows: 3, cols: 4 }.is_insertion());
    }

    #[test]
    fn test_set_script_element_is_editing() {
        assert!(EditorAction::SetScriptElement("Action".into()).is_editing());
    }

    #[test]
    fn test_action_debug_format() {
        let action = EditorAction::ToggleBold;
        let debug = format!("{:?}", action);
        assert!(debug.contains("ToggleBold"));
    }

    #[test]
    fn test_action_clone() {
        let action = EditorAction::Insert("hello".to_string());
        let cloned = action.clone();
        assert_eq!(cloned.description(), "Insert text");
    }

    #[test]
    fn test_list_style_debug_clone() {
        let bullet = ListStyle::Bullet;
        let cloned = bullet.clone();
        assert_eq!(cloned.prefix(), "- ");
        let debug = format!("{:?}", bullet);
        assert!(debug.contains("Bullet"));
    }

    #[test]
    fn test_datetime_format_debug_clone() {
        let fmt = DateTimeFormat::Iso8601;
        let cloned = fmt.clone();
        assert_eq!(cloned.label(), "ISO 8601");
        let debug = format!("{:?}", fmt);
        assert!(debug.contains("Iso8601"));
    }

    #[test]
    fn test_all_shortcut_hints_non_panicking() {
        let actions = vec![
            EditorAction::ToggleBold,
            EditorAction::ToggleItalic,
            EditorAction::ToggleUnderline,
            EditorAction::Undo,
            EditorAction::Redo,
            EditorAction::SelectAll,
            EditorAction::Find("q".into()),
            EditorAction::Replace { find: "a".into(), replace: "b".into() },
            EditorAction::MoveToLineStart,
            EditorAction::MoveToLineEnd,
            EditorAction::MoveToDocStart,
            EditorAction::MoveToDocEnd,
            EditorAction::Indent,
            EditorAction::Unindent,
            EditorAction::DeleteLine,
            EditorAction::DuplicateLine,
            EditorAction::MoveLineUp,
            EditorAction::MoveLineDown,
            EditorAction::GoToLine(1),
            EditorAction::ToggleComment,
            EditorAction::InsertBlockQuote,
            EditorAction::SortLines,
            EditorAction::TransposeChars,
        ];
        for action in &actions {
            let _ = action.shortcut_hint();
        }
    }

    #[test]
    fn test_category_completeness() {
        let actions: Vec<EditorAction> = vec![
            EditorAction::MoveToLineStart,
            EditorAction::MoveToDocEnd,
            EditorAction::GoToLine(1),
            EditorAction::SelectAll,
            EditorAction::SelectWord,
            EditorAction::ToggleBold,
            EditorAction::ToUppercase,
            EditorAction::Insert("x".into()),
            EditorAction::InsertHeading(1),
            EditorAction::Undo,
            EditorAction::Redo,
            EditorAction::Find("x".into()),
            EditorAction::Replace { find: "a".into(), replace: "b".into() },
            EditorAction::DeleteLine,
            EditorAction::SortLines,
        ];
        let categories: Vec<&str> = actions.iter().map(|a| a.category()).collect();
        assert!(categories.contains(&"Navigation"));
        assert!(categories.contains(&"Selection"));
        assert!(categories.contains(&"Formatting"));
        assert!(categories.contains(&"Insert"));
        assert!(categories.contains(&"History"));
        assert!(categories.contains(&"Find & Replace"));
        assert!(categories.contains(&"Edit"));
    }

    #[test]
    fn test_focus_mode_not_editing() {
        assert!(!EditorAction::ToggleFocusMode.is_editing());
        assert!(!EditorAction::ToggleFocusMode.is_movement());
        assert!(!EditorAction::ToggleFocusMode.is_selection());
        assert!(!EditorAction::ToggleFocusMode.is_formatting());
        assert!(!EditorAction::ToggleFocusMode.is_insertion());
    }

    #[test]
    fn test_insert_code_block_with_language() {
        let action = EditorAction::InsertCodeBlock(Some("rust".into()));
        assert!(action.is_insertion());
        assert_eq!(action.description(), "Insert code block");
    }

    #[test]
    fn test_insert_heading_levels() {
        for level in 1..=6 {
            let action = EditorAction::InsertHeading(level);
            assert!(action.is_insertion());
            assert!(action.is_editing());
            assert_eq!(action.description(), "Insert heading");
        }
    }

    #[test]
    fn test_page_break_properties() {
        let action = EditorAction::InsertPageBreak;
        assert!(action.is_insertion());
        assert!(action.is_editing());
        assert!(action.is_undoable());
        assert_eq!(action.category(), "Insert");
    }

    // ---- New tests for batch_description, requires_selection, menu_path, list/datetime helpers ----

    #[test]
    fn test_requires_selection() {
        assert!(EditorAction::WrapSelection { prefix: "[".into(), suffix: "]".into() }.requires_selection());
        assert!(EditorAction::ToUppercase.requires_selection());
        assert!(EditorAction::ToLowercase.requires_selection());
        assert!(EditorAction::ToTitleCase.requires_selection());
        assert!(EditorAction::SortLines.requires_selection());
        assert!(EditorAction::RemoveDuplicateLines.requires_selection());
        assert!(!EditorAction::ToggleBold.requires_selection());
        assert!(!EditorAction::Undo.requires_selection());
        assert!(!EditorAction::DeleteLine.requires_selection());
    }

    #[test]
    fn test_menu_path_formatting() {
        let path = EditorAction::ToggleBold.menu_path();
        assert_eq!(path, vec!["Format", "Style"]);
    }

    #[test]
    fn test_menu_path_case() {
        let path = EditorAction::ToUppercase.menu_path();
        assert_eq!(path, vec!["Format", "Case"]);
    }

    #[test]
    fn test_menu_path_headings() {
        let path = EditorAction::InsertHeading(2).menu_path();
        assert_eq!(path, vec!["Insert", "Heading"]);
    }

    #[test]
    fn test_menu_path_line_edit() {
        let path = EditorAction::MoveLineUp.menu_path();
        assert_eq!(path, vec!["Edit", "Line"]);
    }

    #[test]
    fn test_menu_path_indent() {
        let path = EditorAction::Indent.menu_path();
        assert_eq!(path, vec!["Edit", "Indentation"]);
    }

    #[test]
    fn test_menu_path_fallback() {
        let path = EditorAction::Undo.menu_path();
        assert_eq!(path, vec!["History"]);
    }

    #[test]
    fn test_batch_description_empty() {
        let desc = EditorAction::batch_description(&[]);
        assert_eq!(desc, "No actions");
    }

    #[test]
    fn test_batch_description_single() {
        let actions = vec![EditorAction::ToggleBold];
        let desc = EditorAction::batch_description(&actions);
        assert_eq!(desc, "Toggle bold");
    }

    #[test]
    fn test_batch_description_multiple() {
        let actions = vec![
            EditorAction::ToggleBold,
            EditorAction::MoveToDocStart,
            EditorAction::ToUppercase,
        ];
        let desc = EditorAction::batch_description(&actions);
        assert!(desc.contains("3 actions"));
        assert!(desc.contains("2 edits"));
        assert!(desc.contains("1 navigation"));
    }

    #[test]
    fn test_list_style_all() {
        let all = ListStyle::all();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_list_style_continuation_prefix() {
        assert_eq!(ListStyle::Bullet.continuation_prefix(0), "- ");
        assert_eq!(ListStyle::Bullet.continuation_prefix(5), "- ");
        assert_eq!(ListStyle::Numbered.continuation_prefix(0), "1. ");
        assert_eq!(ListStyle::Numbered.continuation_prefix(4), "5. ");
        assert_eq!(ListStyle::Checkbox.continuation_prefix(0), "- [ ] ");
    }

    #[test]
    fn test_datetime_format_all() {
        let all = DateTimeFormat::all();
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn test_datetime_format_pattern() {
        assert_eq!(DateTimeFormat::DateOnly.pattern(), "YYYY-MM-DD");
        assert_eq!(DateTimeFormat::TimeOnly.pattern(), "HH:MM");
        assert!(DateTimeFormat::DateTime.pattern().contains("YYYY"));
        assert!(DateTimeFormat::Iso8601.pattern().contains("T"));
    }
}
