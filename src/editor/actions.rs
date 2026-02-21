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
}
