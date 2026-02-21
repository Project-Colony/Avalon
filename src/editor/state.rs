use crate::core::document::Document;

/// The state of the text editor
#[derive(Debug)]
pub struct EditorState {
    /// The document being edited
    pub document: Document,
    /// Current cursor byte position
    pub cursor: usize,
    /// Selection start (if any)
    pub selection_start: Option<usize>,
    /// Undo stack
    pub undo_stack: Vec<UndoEntry>,
    /// Redo stack
    pub redo_stack: Vec<UndoEntry>,
    /// Whether the document has unsaved changes
    pub dirty: bool,
    /// Scroll offset (in lines)
    pub scroll_offset: f32,
    /// The iced text editor content
    pub content: iced::widget::text_editor::Content,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            document: Document::new(),
            cursor: 0,
            selection_start: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            dirty: false,
            scroll_offset: 0.0,
            content: iced::widget::text_editor::Content::new(),
        }
    }

    pub fn from_document(doc: &Document) -> Self {
        Self {
            content: iced::widget::text_editor::Content::with_text(&doc.content),
            document: doc.clone(),
            cursor: 0,
            selection_start: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            dirty: false,
            scroll_offset: 0.0,
        }
    }

    /// Load a new document into the editor
    pub fn load_document(&mut self, doc: &Document) {
        self.document = doc.clone();
        self.content = iced::widget::text_editor::Content::with_text(&doc.content);
        self.cursor = 0;
        self.selection_start = None;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.dirty = false;
        self.scroll_offset = 0.0;
    }

    /// Get the current text content from the iced editor
    pub fn text(&self) -> String {
        self.content.text()
    }

    /// Mark the document as modified
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.document.content = self.text();
    }

    /// Mark the document as saved
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Get the current line number (1-indexed)
    pub fn current_line(&self) -> usize {
        let text = self.text();
        let pos = self.cursor.min(text.len());
        text[..pos].matches('\n').count() + 1
    }

    /// Get the current column number (1-indexed)
    pub fn current_column(&self) -> usize {
        let text = self.text();
        let pos = self.cursor.min(text.len());
        let last_newline = text[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
        pos - last_newline + 1
    }

    /// Get the total number of lines in the document
    pub fn line_count(&self) -> usize {
        self.text().matches('\n').count() + 1
    }

    /// Check whether text is currently selected
    pub fn has_selection(&self) -> bool {
        self.selection_start.is_some()
    }

    /// Get the selected text range, if any
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_start.map(|start| {
            let end = self.cursor;
            if start <= end { (start, end) } else { (end, start) }
        })
    }

    /// Get the currently selected text, if any
    pub fn selected_text(&self) -> Option<String> {
        self.selection_range().map(|(start, end)| {
            let text = self.text();
            text.get(start..end).unwrap_or("").to_string()
        })
    }

    /// Get the word at the current cursor position
    pub fn word_at_cursor(&self) -> Option<String> {
        let text = self.text();
        let pos = self.cursor.min(text.len());
        if text.is_empty() || pos == 0 {
            return None;
        }
        let bytes = text.as_bytes();
        let mut start = pos;
        while start > 0 && bytes.get(start - 1).map_or(false, |b| b.is_ascii_alphanumeric() || *b == b'_') {
            start -= 1;
        }
        let mut end = pos;
        while end < text.len() && bytes.get(end).map_or(false, |b| b.is_ascii_alphanumeric() || *b == b'_') {
            end += 1;
        }
        if start < end {
            Some(text[start..end].to_string())
        } else {
            None
        }
    }

    /// Check if the undo stack has entries
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if the redo stack has entries
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the undo stack depth
    pub fn undo_depth(&self) -> usize {
        self.undo_stack.len()
    }

    /// Push current state onto undo stack
    pub fn push_undo(&mut self) {
        self.undo_stack.push(UndoEntry {
            content: self.document.content.clone(),
            cursor: self.cursor,
        });
        self.redo_stack.clear();

        // Limit undo stack size
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
    }

    /// Undo the last change
    pub fn undo(&mut self) -> bool {
        if let Some(entry) = self.undo_stack.pop() {
            self.redo_stack.push(UndoEntry {
                content: self.document.content.clone(),
                cursor: self.cursor,
            });
            self.document.content = entry.content.clone();
            self.content = iced::widget::text_editor::Content::with_text(&entry.content);
            self.cursor = entry.cursor;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// Redo the last undone change
    pub fn redo(&mut self) -> bool {
        if let Some(entry) = self.redo_stack.pop() {
            self.undo_stack.push(UndoEntry {
                content: self.document.content.clone(),
                cursor: self.cursor,
            });
            self.document.content = entry.content.clone();
            self.content = iced::widget::text_editor::Content::with_text(&entry.content);
            self.cursor = entry.cursor;
            self.dirty = true;
            true
        } else {
            false
        }
    }
}

/// An entry in the undo/redo stack
#[derive(Debug, Clone)]
pub struct UndoEntry {
    pub content: String,
    pub cursor: usize,
}
