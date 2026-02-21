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

    /// Get the line number (0-based) at the cursor position
    pub fn cursor_line(&self) -> usize {
        let text = &self.document.content;
        let pos = self.cursor.min(text.len());
        text[..pos].matches('\n').count()
    }

    /// Get the column number (0-based) at the cursor position
    pub fn cursor_column(&self) -> usize {
        let text = &self.document.content;
        let pos = self.cursor.min(text.len());
        let last_newline = text[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
        pos - last_newline
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

impl EditorState {
    /// Get the current line text
    pub fn current_line_text(&self) -> String {
        let text = self.text();
        let pos = self.cursor.min(text.len());
        let start = text[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
        let end = text[pos..].find('\n').map(|p| pos + p).unwrap_or(text.len());
        text[start..end].to_string()
    }

    /// Get the current paragraph text (delimited by blank lines)
    pub fn current_paragraph_text(&self) -> String {
        let text = self.text();
        let pos = self.cursor.min(text.len());

        // Find paragraph start
        let mut start = pos;
        while start > 1 {
            if text[..start].ends_with("\n\n") {
                break;
            }
            start -= 1;
        }

        // Find paragraph end
        let end = text[pos..]
            .find("\n\n")
            .map(|p| pos + p)
            .unwrap_or(text.len());

        text[start..end].trim().to_string()
    }

    /// Character count of the document
    pub fn char_count(&self) -> usize {
        self.text().len()
    }

    /// Word count of the document
    pub fn word_count(&self) -> usize {
        self.text().split_whitespace().count()
    }

    /// Paragraph count of the document
    pub fn paragraph_count(&self) -> usize {
        let text = self.text();
        if text.is_empty() {
            return 0;
        }
        text.split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .count()
    }

    /// Sentence count of the document
    pub fn sentence_count(&self) -> usize {
        self.text()
            .chars()
            .filter(|c| *c == '.' || *c == '!' || *c == '?')
            .count()
    }

    /// Get the redo stack depth
    pub fn redo_depth(&self) -> usize {
        self.redo_stack.len()
    }

    /// Check if the document is empty
    pub fn is_empty(&self) -> bool {
        self.text().trim().is_empty()
    }

    /// Get the content length in bytes
    pub fn content_length(&self) -> usize {
        self.text().len()
    }

    /// Clear undo and redo stacks
    pub fn clear_history(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

/// An entry in the undo/redo stack
#[derive(Debug, Clone)]
pub struct UndoEntry {
    pub content: String,
    pub cursor: usize,
}

impl EditorState {
    /// Find all occurrences of a query in the document text
    pub fn find_all(&self, query: &str, case_sensitive: bool) -> Vec<(usize, usize)> {
        let text = self.text();
        if query.is_empty() || text.is_empty() {
            return Vec::new();
        }
        let (search_text, search_query) = if case_sensitive {
            (text.clone(), query.to_string())
        } else {
            (text.to_lowercase(), query.to_lowercase())
        };
        search_text.match_indices(&search_query)
            .map(|(pos, _)| (pos, pos + query.len()))
            .collect()
    }

    /// Replace all occurrences of a query in the document
    pub fn replace_all(&mut self, find: &str, replace: &str, case_sensitive: bool) -> usize {
        if find.is_empty() {
            return 0;
        }
        let text = self.text();
        let new_text = if case_sensitive {
            text.replace(find, replace)
        } else {
            let lower = text.to_lowercase();
            let lower_find = find.to_lowercase();
            let mut result = String::new();
            let mut last_end = 0;
            for (start, _) in lower.match_indices(&lower_find) {
                result.push_str(&text[last_end..start]);
                result.push_str(replace);
                last_end = start + find.len();
            }
            result.push_str(&text[last_end..]);
            result
        };
        let count = self.find_all(find, case_sensitive).len();
        if count > 0 {
            self.push_undo();
            self.document.content = new_text.clone();
            self.content = iced::widget::text_editor::Content::with_text(&new_text);
            self.dirty = true;
        }
        count
    }

    /// Insert text at the current cursor position
    pub fn insert_at_cursor(&mut self, text: &str) {
        self.push_undo();
        let content = self.text();
        let pos = self.cursor.min(content.len());
        let new_content = format!("{}{}{}", &content[..pos], text, &content[pos..]);
        self.document.content = new_content.clone();
        self.content = iced::widget::text_editor::Content::with_text(&new_content);
        self.cursor = pos + text.len();
        self.dirty = true;
    }

    /// Delete the selected text (if any selection exists)
    pub fn delete_selection(&mut self) -> Option<String> {
        if let Some((start, end)) = self.selection_range() {
            self.push_undo();
            let text = self.text();
            let deleted = text[start..end].to_string();
            let new_content = format!("{}{}", &text[..start], &text[end..]);
            self.document.content = new_content.clone();
            self.content = iced::widget::text_editor::Content::with_text(&new_content);
            self.cursor = start;
            self.selection_start = None;
            self.dirty = true;
            Some(deleted)
        } else {
            None
        }
    }

    /// Wrap the selection (or insert at cursor) with a prefix and suffix
    pub fn wrap_selection(&mut self, prefix: &str, suffix: &str) {
        if let Some((start, end)) = self.selection_range() {
            self.push_undo();
            let text = self.text();
            let selected = &text[start..end];
            let new_content = format!(
                "{}{}{}{}{}",
                &text[..start], prefix, selected, suffix, &text[end..]
            );
            self.document.content = new_content.clone();
            self.content = iced::widget::text_editor::Content::with_text(&new_content);
            self.cursor = end + prefix.len() + suffix.len();
            self.selection_start = None;
            self.dirty = true;
        } else {
            self.insert_at_cursor(&format!("{}{}", prefix, suffix));
        }
    }

    /// Get the line at a given line number (0-based)
    pub fn line_at(&self, line: usize) -> Option<String> {
        self.text().lines().nth(line).map(|s| s.to_string())
    }

    /// Get total character count excluding trailing whitespace
    pub fn trimmed_length(&self) -> usize {
        self.text().trim_end().len()
    }

    /// Move cursor to a specific byte position
    pub fn move_cursor_to(&mut self, pos: usize) {
        let text = self.text();
        self.cursor = pos.min(text.len());
        self.selection_start = None;
    }

    /// Select a range of text
    pub fn select_range(&mut self, start: usize, end: usize) {
        let text = self.text();
        self.selection_start = Some(start.min(text.len()));
        self.cursor = end.min(text.len());
    }

    /// Select the entire document
    pub fn select_all(&mut self) {
        let len = self.text().len();
        self.selection_start = Some(0);
        self.cursor = len;
    }

    /// Get the word count for a specific line (0-based)
    pub fn line_word_count(&self, line: usize) -> usize {
        self.line_at(line)
            .map(|l| l.split_whitespace().count())
            .unwrap_or(0)
    }

    /// Get a summary of the editor state for debugging
    pub fn debug_summary(&self) -> String {
        format!(
            "cursor={}, sel={:?}, lines={}, words={}, dirty={}, undo={}, redo={}",
            self.cursor,
            self.selection_start,
            self.line_count(),
            self.word_count(),
            self.dirty,
            self.undo_depth(),
            self.redo_depth(),
        )
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}
