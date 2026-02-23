use std::collections::{HashMap, HashSet};
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
            self.content = iced::widget::text_editor::Content::with_text(&entry.content);
            self.document.content = entry.content;
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
            self.content = iced::widget::text_editor::Content::with_text(&entry.content);
            self.document.content = entry.content;
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

impl EditorState {
    /// Transpose the two characters around the cursor
    pub fn transpose_chars(&mut self) {
        let text = self.document.content.clone();
        let pos = self.cursor;
        if pos == 0 || pos >= text.len() {
            if pos < 2 || text.len() < 2 {
                return;
            }
        }

        let swap_pos = if pos >= text.len() { pos - 2 } else { pos.saturating_sub(1) };
        let bytes = text.as_bytes();
        if swap_pos + 1 >= text.len() {
            return;
        }

        if !bytes[swap_pos].is_ascii() || !bytes[swap_pos + 1].is_ascii() {
            return;
        }

        self.push_undo();
        let mut chars: Vec<u8> = text.bytes().collect();
        chars.swap(swap_pos, swap_pos + 1);
        let new_content = String::from_utf8(chars).unwrap_or(text);
        self.document.content = new_content.clone();
        self.content = iced::widget::text_editor::Content::with_text(&new_content);
        self.cursor = swap_pos + 2;
        self.dirty = true;
    }

    /// Sort selected lines alphabetically, or all lines if no selection
    pub fn sort_lines(&mut self) {
        let text = self.document.content.clone();
        let mut lines: Vec<&str> = text.lines().collect();
        if lines.len() < 2 {
            return;
        }
        self.push_undo();
        lines.sort();
        let new_content = lines.join("\n");
        self.document.content = new_content.clone();
        self.content = iced::widget::text_editor::Content::with_text(&new_content);
        self.dirty = true;
    }

    /// Remove duplicate lines (keeping first occurrence)
    pub fn remove_duplicate_lines(&mut self) {
        let text = self.document.content.clone();
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() < 2 {
            return;
        }
        self.push_undo();
        let mut seen = HashSet::new();
        let unique: Vec<&str> = lines.into_iter()
            .filter(|line| seen.insert(*line))
            .collect();
        let new_content = unique.join("\n");
        self.document.content = new_content.clone();
        self.content = iced::widget::text_editor::Content::with_text(&new_content);
        self.dirty = true;
    }

    /// Join the current line with the next line
    pub fn join_lines(&mut self) {
        let text = self.document.content.clone();
        let pos = self.cursor.min(text.len());

        let line_end = text[pos..].find('\n').map(|p| pos + p);
        if let Some(nl_pos) = line_end {
            self.push_undo();
            let mut new_content = text;
            new_content.replace_range(nl_pos..nl_pos + 1, " ");
            self.document.content = new_content.clone();
            self.content = iced::widget::text_editor::Content::with_text(&new_content);
            self.dirty = true;
        }
    }

    /// Move the current line up (swap with previous line)
    pub fn move_line_up(&mut self) {
        let text = self.document.content.clone();
        let pos = self.cursor.min(text.len());
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() < 2 {
            return;
        }

        let current_line = text[..pos].matches('\n').count();
        if current_line == 0 {
            return;
        }

        self.push_undo();
        let mut new_lines = lines;
        new_lines.swap(current_line, current_line - 1);
        let new_content = new_lines.join("\n");

        let new_cursor = new_lines[..current_line - 1].iter()
            .map(|l| l.len() + 1)
            .sum::<usize>();

        self.document.content = new_content.clone();
        self.content = iced::widget::text_editor::Content::with_text(&new_content);
        self.cursor = new_cursor;
        self.dirty = true;
    }

    /// Move the current line down (swap with next line)
    pub fn move_line_down(&mut self) {
        let text = self.document.content.clone();
        let pos = self.cursor.min(text.len());
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() < 2 {
            return;
        }

        let current_line = text[..pos].matches('\n').count();
        if current_line >= lines.len() - 1 {
            return;
        }

        self.push_undo();
        let mut new_lines = lines;
        new_lines.swap(current_line, current_line + 1);
        let new_content = new_lines.join("\n");

        let new_cursor = new_lines[..current_line + 1].iter()
            .map(|l| l.len() + 1)
            .sum::<usize>();

        self.document.content = new_content.clone();
        self.content = iced::widget::text_editor::Content::with_text(&new_content);
        self.cursor = new_cursor;
        self.dirty = true;
    }

    /// Delete the current line
    pub fn delete_line(&mut self) {
        let text = self.document.content.clone();
        let pos = self.cursor.min(text.len());
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return;
        }

        let current_line = text[..pos].matches('\n').count();
        self.push_undo();

        let mut new_lines: Vec<&str> = lines;
        new_lines.remove(current_line.min(new_lines.len() - 1));
        let new_content = new_lines.join("\n");

        let new_cursor = if new_lines.is_empty() {
            0
        } else {
            let target_line = current_line.min(new_lines.len() - 1);
            new_lines[..target_line].iter()
                .map(|l| l.len() + 1)
                .sum::<usize>()
        };

        self.document.content = new_content.clone();
        self.content = iced::widget::text_editor::Content::with_text(&new_content);
        self.cursor = new_cursor;
        self.dirty = true;
    }

    /// Duplicate the current line
    pub fn duplicate_line(&mut self) {
        let text = self.document.content.clone();
        let pos = self.cursor.min(text.len());
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return;
        }

        let current_line = text[..pos].matches('\n').count();
        let line_idx = current_line.min(lines.len() - 1);

        self.push_undo();
        let mut new_lines = lines;
        let dup = new_lines[line_idx];
        new_lines.insert(line_idx + 1, dup);
        let new_content = new_lines.join("\n");

        let new_cursor = new_lines[..line_idx + 1].iter()
            .map(|l| l.len() + 1)
            .sum::<usize>();

        self.document.content = new_content.clone();
        self.content = iced::widget::text_editor::Content::with_text(&new_content);
        self.cursor = new_cursor;
        self.dirty = true;
    }

    /// Indent the current line (add 4 spaces at start)
    pub fn indent_line(&mut self) {
        let text = self.document.content.clone();
        let pos = self.cursor.min(text.len());

        let line_start = text[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0);

        self.push_undo();
        let new_content = format!("{}    {}", &text[..line_start], &text[line_start..]);
        self.document.content = new_content.clone();
        self.content = iced::widget::text_editor::Content::with_text(&new_content);
        self.cursor = pos + 4;
        self.dirty = true;
    }

    /// Unindent the current line (remove up to 4 spaces from start)
    pub fn unindent_line(&mut self) {
        let text = self.document.content.clone();
        let pos = self.cursor.min(text.len());

        let line_start = text[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0);

        let spaces = text[line_start..].chars().take(4).take_while(|c| *c == ' ').count();
        if spaces == 0 {
            return;
        }

        self.push_undo();
        let new_content = format!("{}{}", &text[..line_start], &text[line_start + spaces..]);
        self.document.content = new_content.clone();
        self.content = iced::widget::text_editor::Content::with_text(&new_content);
        self.cursor = pos.saturating_sub(spaces);
        self.dirty = true;
    }

    /// Toggle a markdown comment around the current line
    pub fn toggle_comment(&mut self) {
        let text = self.document.content.clone();
        let pos = self.cursor.min(text.len());

        let line_start = text[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
        let line_end = text[pos..].find('\n').map(|p| pos + p).unwrap_or(text.len());
        let line = &text[line_start..line_end];

        self.push_undo();
        let new_content = if line.starts_with("<!-- ") && line.ends_with(" -->") {
            let uncommented = &line[5..line.len() - 4];
            format!("{}{}{}", &text[..line_start], uncommented, &text[line_end..])
        } else {
            format!("{}<!-- {} -->{}", &text[..line_start], line, &text[line_end..])
        };
        self.document.content = new_content.clone();
        self.content = iced::widget::text_editor::Content::with_text(&new_content);
        self.dirty = true;
    }

    /// Transform text to uppercase - selection only if selected, otherwise whole document
    pub fn to_uppercase(&mut self, selection_only: bool) {
        if selection_only {
            if let Some((start, end)) = self.selection_range() {
                self.push_undo();
                let text = self.document.content.clone();
                let upper = text[start..end].to_uppercase();
                let new_content = format!("{}{}{}", &text[..start], upper, &text[end..]);
                self.document.content = new_content.clone();
                self.content = iced::widget::text_editor::Content::with_text(&new_content);
                self.cursor = start + upper.len();
                self.selection_start = None;
                self.dirty = true;
                return;
            }
            return; // no selection, do nothing when selection_only
        }
        // Whole document
        self.push_undo();
        let new_content = self.document.content.to_uppercase();
        self.document.content = new_content.clone();
        self.content = iced::widget::text_editor::Content::with_text(&new_content);
        self.dirty = true;
    }

    /// Transform text to lowercase - selection only if selected, otherwise whole document
    pub fn to_lowercase(&mut self, selection_only: bool) {
        if selection_only {
            if let Some((start, end)) = self.selection_range() {
                self.push_undo();
                let text = self.document.content.clone();
                let lower = text[start..end].to_lowercase();
                let new_content = format!("{}{}{}", &text[..start], lower, &text[end..]);
                self.document.content = new_content.clone();
                self.content = iced::widget::text_editor::Content::with_text(&new_content);
                self.cursor = start + lower.len();
                self.selection_start = None;
                self.dirty = true;
                return;
            }
            return; // no selection, do nothing when selection_only
        }
        self.push_undo();
        let new_content = self.document.content.to_lowercase();
        self.document.content = new_content.clone();
        self.content = iced::widget::text_editor::Content::with_text(&new_content);
        self.dirty = true;
    }

    /// Transform text to title case - selection only if selected, otherwise whole document
    pub fn to_title_case(&mut self, selection_only: bool) {
        fn title_case(s: &str) -> String {
            let mut result = String::with_capacity(s.len());
            let mut capitalize_next = true;
            for ch in s.chars() {
                if capitalize_next && ch.is_alphabetic() {
                    for c in ch.to_uppercase() {
                        result.push(c);
                    }
                    capitalize_next = false;
                } else {
                    result.push(ch);
                    if ch == ' ' || ch == '\n' || ch == '\t' || ch == '-' {
                        capitalize_next = true;
                    }
                }
            }
            result
        }

        if selection_only {
            if let Some((start, end)) = self.selection_range() {
                self.push_undo();
                let text = self.document.content.clone();
                let titled = title_case(&text[start..end]);
                let new_content = format!("{}{}{}", &text[..start], titled, &text[end..]);
                self.document.content = new_content.clone();
                self.content = iced::widget::text_editor::Content::with_text(&new_content);
                self.cursor = start + titled.len();
                self.selection_start = None;
                self.dirty = true;
                return;
            }
            return; // no selection, do nothing when selection_only
        }
        self.push_undo();
        let new_content = title_case(&self.document.content);
        self.document.content = new_content.clone();
        self.content = iced::widget::text_editor::Content::with_text(&new_content);
        self.dirty = true;
    }
}

impl EditorState {
    /// Reverse the order of all lines in the document
    pub fn reverse_lines(&mut self) {
        let text = self.document.content.clone();
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() < 2 {
            return;
        }
        self.push_undo();
        let reversed: Vec<&str> = lines.into_iter().rev().collect();
        let new_content = reversed.join("\n");
        self.document.content = new_content.clone();
        self.content = iced::widget::text_editor::Content::with_text(&new_content);
        self.dirty = true;
    }

    /// Strip trailing whitespace from every line
    pub fn strip_trailing_whitespace(&mut self) {
        let text = self.document.content.clone();
        let trimmed: Vec<&str> = text.lines().map(|l| l.trim_end()).collect();
        let new_content = trimmed.join("\n");
        if new_content == text {
            return;
        }
        self.push_undo();
        self.document.content = new_content.clone();
        self.content = iced::widget::text_editor::Content::with_text(&new_content);
        self.dirty = true;
    }

    /// Collapse runs of multiple spaces into single spaces on every line
    pub fn collapse_whitespace(&mut self) {
        let text = self.document.content.clone();
        let mut new_lines = Vec::new();
        let mut changed = false;
        for line in text.lines() {
            let collapsed: String = line.split_whitespace().collect::<Vec<_>>().join(" ");
            if collapsed != line {
                changed = true;
            }
            new_lines.push(collapsed);
        }
        if !changed {
            return;
        }
        self.push_undo();
        let new_content = new_lines.join("\n");
        self.document.content = new_content.clone();
        self.content = iced::widget::text_editor::Content::with_text(&new_content);
        self.dirty = true;
    }

    /// Prepend line numbers to every line (1-indexed)
    pub fn number_lines(&mut self) {
        let text = self.document.content.clone();
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return;
        }
        self.push_undo();
        let width = lines.len().to_string().len();
        let numbered: Vec<String> = lines
            .iter()
            .enumerate()
            .map(|(i, l)| format!("{:>width$}  {}", i + 1, l, width = width))
            .collect();
        let new_content = numbered.join("\n");
        self.document.content = new_content.clone();
        self.content = iced::widget::text_editor::Content::with_text(&new_content);
        self.dirty = true;
    }

    /// Extract all sentences from the document (split by .!?)
    pub fn extract_sentences(&self) -> Vec<String> {
        let text = &self.document.content;
        if text.trim().is_empty() {
            return Vec::new();
        }
        let mut sentences = Vec::new();
        let mut current = String::new();
        for ch in text.chars() {
            current.push(ch);
            if ch == '.' || ch == '!' || ch == '?' {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                }
                current.clear();
            }
        }
        let leftover = current.trim().to_string();
        if !leftover.is_empty() {
            sentences.push(leftover);
        }
        sentences
    }

    /// Get word frequency map (case-insensitive)
    pub fn word_frequency(&self) -> HashMap<String, usize> {
        let mut freq = HashMap::new();
        for word in self.document.content.split_whitespace() {
            let lower = word
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            if !lower.is_empty() {
                *freq.entry(lower).or_insert(0) += 1;
            }
        }
        freq
    }

    /// Get the lengths of all lines
    pub fn line_lengths(&self) -> Vec<usize> {
        self.document.content.lines().map(|l| l.len()).collect()
    }

    /// Get the longest line's length and its 0-based line number
    pub fn longest_line(&self) -> Option<(usize, usize)> {
        self.document
            .content
            .lines()
            .enumerate()
            .max_by_key(|(_, l)| l.len())
            .map(|(i, l)| (i, l.len()))
    }

    /// Get average line length
    pub fn avg_line_length(&self) -> f64 {
        let lengths = self.line_lengths();
        if lengths.is_empty() {
            return 0.0;
        }
        lengths.iter().sum::<usize>() as f64 / lengths.len() as f64
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::Document;

    fn editor_with(content: &str) -> EditorState {
        let doc = Document::with_content(content);
        EditorState::from_document(&doc)
    }

    // Note: tests use document.content directly because iced Content::with_text()
    // may add a trailing newline in the widget representation.

    #[test]
    fn test_transpose_chars() {
        let mut editor = editor_with("abcdef");
        editor.cursor = 2;
        editor.transpose_chars();
        assert_eq!(editor.document.content, "acbdef");
    }

    #[test]
    fn test_transpose_chars_at_start() {
        let mut editor = editor_with("abc");
        editor.cursor = 0;
        editor.transpose_chars();
        assert_eq!(editor.document.content, "abc");
    }

    #[test]
    fn test_sort_lines() {
        let mut editor = editor_with("cherry\napple\nbanana");
        editor.sort_lines();
        assert_eq!(editor.document.content, "apple\nbanana\ncherry");
    }

    #[test]
    fn test_sort_lines_single() {
        let mut editor = editor_with("only one line");
        editor.sort_lines();
        assert_eq!(editor.document.content, "only one line");
    }

    #[test]
    fn test_remove_duplicate_lines() {
        let mut editor = editor_with("apple\nbanana\napple\ncherry\nbanana");
        editor.remove_duplicate_lines();
        assert_eq!(editor.document.content, "apple\nbanana\ncherry");
    }

    #[test]
    fn test_remove_duplicate_lines_no_dups() {
        let mut editor = editor_with("a\nb\nc");
        editor.remove_duplicate_lines();
        assert_eq!(editor.document.content, "a\nb\nc");
    }

    #[test]
    fn test_join_lines() {
        let mut editor = editor_with("line one\nline two\nline three");
        editor.cursor = 3;
        editor.join_lines();
        assert_eq!(editor.document.content, "line one line two\nline three");
    }

    #[test]
    fn test_move_line_up() {
        let mut editor = editor_with("first\nsecond\nthird");
        editor.cursor = 7;
        editor.move_line_up();
        assert_eq!(editor.document.content, "second\nfirst\nthird");
    }

    #[test]
    fn test_move_line_up_first_line() {
        let mut editor = editor_with("first\nsecond");
        editor.cursor = 2;
        editor.move_line_up();
        assert_eq!(editor.document.content, "first\nsecond");
    }

    #[test]
    fn test_move_line_down() {
        let mut editor = editor_with("first\nsecond\nthird");
        editor.cursor = 2;
        editor.move_line_down();
        assert_eq!(editor.document.content, "second\nfirst\nthird");
    }

    #[test]
    fn test_move_line_down_last_line() {
        let mut editor = editor_with("first\nsecond");
        editor.cursor = 8;
        editor.move_line_down();
        assert_eq!(editor.document.content, "first\nsecond");
    }

    #[test]
    fn test_delete_line() {
        let mut editor = editor_with("first\nsecond\nthird");
        editor.cursor = 7;
        editor.delete_line();
        assert_eq!(editor.document.content, "first\nthird");
    }

    #[test]
    fn test_delete_line_first() {
        let mut editor = editor_with("first\nsecond\nthird");
        editor.cursor = 2;
        editor.delete_line();
        assert_eq!(editor.document.content, "second\nthird");
    }

    #[test]
    fn test_duplicate_line() {
        let mut editor = editor_with("first\nsecond\nthird");
        editor.cursor = 2;
        editor.duplicate_line();
        assert_eq!(editor.document.content, "first\nfirst\nsecond\nthird");
    }

    #[test]
    fn test_indent_line() {
        let mut editor = editor_with("hello\nworld");
        editor.cursor = 0;
        editor.indent_line();
        assert_eq!(editor.document.content, "    hello\nworld");
        assert_eq!(editor.cursor, 4);
    }

    #[test]
    fn test_unindent_line() {
        let mut editor = editor_with("    hello\nworld");
        editor.cursor = 6;
        editor.unindent_line();
        assert_eq!(editor.document.content, "hello\nworld");
    }

    #[test]
    fn test_unindent_partial() {
        let mut editor = editor_with("  hello\nworld");
        editor.cursor = 4;
        editor.unindent_line();
        assert_eq!(editor.document.content, "hello\nworld");
    }

    #[test]
    fn test_toggle_comment() {
        let mut editor = editor_with("hello world");
        editor.cursor = 3;
        editor.toggle_comment();
        assert_eq!(editor.document.content, "<!-- hello world -->");

        // Toggle again to uncomment
        editor.cursor = 5;
        editor.toggle_comment();
        assert_eq!(editor.document.content, "hello world");
    }

    #[test]
    fn test_sort_lines_preserves_order() {
        let mut editor = editor_with("z line\na line\nm line");
        editor.sort_lines();
        let lines: Vec<&str> = editor.document.content.lines().collect();
        assert_eq!(lines, vec!["a line", "m line", "z line"]);
    }

    #[test]
    fn test_editor_undo_after_operation() {
        let mut editor = editor_with("first\nsecond");
        editor.cursor = 7;
        editor.delete_line();
        assert_eq!(editor.document.content, "first");
        assert!(editor.can_undo());
        editor.undo();
        assert_eq!(editor.document.content, "first\nsecond");
    }

    #[test]
    fn test_to_uppercase_whole_doc() {
        let mut editor = editor_with("hello world");
        editor.to_uppercase(false);
        assert_eq!(editor.document.content, "HELLO WORLD");
    }

    #[test]
    fn test_to_uppercase_selection() {
        let mut editor = editor_with("hello world");
        editor.selection_start = Some(0);
        editor.cursor = 5;
        editor.to_uppercase(true);
        assert_eq!(editor.document.content, "HELLO world");
    }

    #[test]
    fn test_to_lowercase_whole_doc() {
        let mut editor = editor_with("HELLO WORLD");
        editor.to_lowercase(false);
        assert_eq!(editor.document.content, "hello world");
    }

    #[test]
    fn test_to_lowercase_selection() {
        let mut editor = editor_with("HELLO WORLD");
        editor.selection_start = Some(6);
        editor.cursor = 11;
        editor.to_lowercase(true);
        assert_eq!(editor.document.content, "HELLO world");
    }

    #[test]
    fn test_to_title_case_whole_doc() {
        let mut editor = editor_with("hello world foo");
        editor.to_title_case(false);
        assert_eq!(editor.document.content, "Hello World Foo");
    }

    #[test]
    fn test_to_title_case_selection() {
        let mut editor = editor_with("hello world foo");
        editor.selection_start = Some(6);
        editor.cursor = 11;
        editor.to_title_case(true);
        assert_eq!(editor.document.content, "hello World foo");
    }

    #[test]
    fn test_to_uppercase_no_selection_with_flag() {
        let mut editor = editor_with("test text");
        // selection_only=true but no selection → no change
        editor.to_uppercase(true);
        assert_eq!(editor.document.content, "test text");
    }

    #[test]
    fn test_to_lowercase_no_selection_with_flag() {
        let mut editor = editor_with("TEST TEXT");
        editor.to_lowercase(true);
        assert_eq!(editor.document.content, "TEST TEXT");
    }

    #[test]
    fn test_to_title_case_no_selection_with_flag() {
        let mut editor = editor_with("test text");
        editor.to_title_case(true);
        assert_eq!(editor.document.content, "test text");
    }

    #[test]
    fn test_indent_second_line() {
        let mut editor = editor_with("first\nsecond\nthird");
        editor.cursor = 8; // middle of "second"
        editor.indent_line();
        assert_eq!(editor.document.content, "first\n    second\nthird");
    }

    #[test]
    fn test_unindent_no_spaces() {
        let mut editor = editor_with("no spaces here");
        editor.cursor = 3;
        editor.unindent_line();
        assert_eq!(editor.document.content, "no spaces here");
    }

    #[test]
    fn test_delete_last_line() {
        let mut editor = editor_with("first\nsecond\nthird");
        editor.cursor = 15; // in "third"
        editor.delete_line();
        assert_eq!(editor.document.content, "first\nsecond");
    }

    #[test]
    fn test_delete_only_line() {
        let mut editor = editor_with("only line");
        editor.cursor = 3;
        editor.delete_line();
        assert_eq!(editor.document.content, "");
    }

    #[test]
    fn test_duplicate_last_line() {
        let mut editor = editor_with("first\nlast");
        editor.cursor = 8;
        editor.duplicate_line();
        assert_eq!(editor.document.content, "first\nlast\nlast");
    }

    #[test]
    fn test_move_line_down_middle() {
        let mut editor = editor_with("aaa\nbbb\nccc");
        editor.cursor = 5; // in "bbb"
        editor.move_line_down();
        assert_eq!(editor.document.content, "aaa\nccc\nbbb");
    }

    #[test]
    fn test_join_lines_last_line() {
        let mut editor = editor_with("only");
        editor.cursor = 2;
        editor.join_lines();
        // No newline to join, should stay unchanged
        assert_eq!(editor.document.content, "only");
    }

    #[test]
    fn test_toggle_comment_multiline() {
        let mut editor = editor_with("line1\nline2\nline3");
        editor.cursor = 8; // in "line2"
        editor.toggle_comment();
        assert_eq!(editor.document.content, "line1\n<!-- line2 -->\nline3");
    }

    #[test]
    fn test_transpose_chars_end() {
        let mut editor = editor_with("ab");
        editor.cursor = 2; // at end
        editor.transpose_chars();
        assert_eq!(editor.document.content, "ba");
    }

    #[test]
    fn test_undo_redo_cycle() {
        let mut editor = editor_with("original");
        editor.to_uppercase(false);
        assert_eq!(editor.document.content, "ORIGINAL");
        assert!(editor.can_undo());
        editor.undo();
        assert_eq!(editor.document.content, "original");
        assert!(editor.can_redo());
        editor.redo();
        assert_eq!(editor.document.content, "ORIGINAL");
    }

    #[test]
    fn test_sort_lines_empty() {
        let mut editor = editor_with("");
        editor.sort_lines();
        assert_eq!(editor.document.content, "");
    }

    #[test]
    fn test_remove_duplicates_all_same() {
        let mut editor = editor_with("same\nsame\nsame");
        editor.remove_duplicate_lines();
        assert_eq!(editor.document.content, "same");
    }

    #[test]
    fn test_indent_then_unindent() {
        let mut editor = editor_with("test line");
        editor.cursor = 0;
        editor.indent_line();
        assert_eq!(editor.document.content, "    test line");
        editor.cursor = 4;
        editor.unindent_line();
        assert_eq!(editor.document.content, "test line");
    }

    #[test]
    fn test_dirty_flag_set_on_edit() {
        let mut editor = editor_with("test");
        assert!(!editor.dirty);
        editor.to_uppercase(false);
        assert!(editor.dirty);
    }

    #[test]
    fn test_mark_clean() {
        let mut editor = editor_with("test");
        editor.to_uppercase(false);
        assert!(editor.dirty);
        editor.mark_clean();
        assert!(!editor.dirty);
    }

    #[test]
    fn test_selection_range() {
        let mut editor = editor_with("hello world");
        editor.selection_start = Some(0);
        editor.cursor = 5;
        assert_eq!(editor.selection_range(), Some((0, 5)));

        // Reversed selection
        editor.selection_start = Some(5);
        editor.cursor = 0;
        assert_eq!(editor.selection_range(), Some((0, 5)));
    }

    #[test]
    fn test_selection_range_none() {
        let editor = editor_with("hello");
        assert_eq!(editor.selection_range(), None);
    }

    #[test]
    fn test_current_line_col() {
        let mut editor = editor_with("first\nsecond\nthird");
        editor.cursor = 0;
        assert_eq!(editor.current_line(), 1);
        assert_eq!(editor.current_column(), 1);

        editor.cursor = 8; // "se|cond"
        assert_eq!(editor.current_line(), 2);
        assert_eq!(editor.current_column(), 3);
    }

    // Additional editor tests

    #[test]
    fn test_word_count() {
        let editor = editor_with("Hello world foo bar baz");
        assert_eq!(editor.word_count(), 5);
    }

    #[test]
    fn test_word_count_empty() {
        let editor = editor_with("");
        assert_eq!(editor.word_count(), 0);
    }

    #[test]
    fn test_document_char_count() {
        let editor = editor_with("abc");
        assert_eq!(editor.document.content.len(), 3);
    }

    #[test]
    fn test_document_line_count() {
        let editor = editor_with("a\nb\nc");
        let lines = editor.document.content.matches('\n').count() + 1;
        assert_eq!(lines, 3);
    }

    #[test]
    fn test_document_single_line() {
        let editor = editor_with("one line");
        let lines = editor.document.content.matches('\n').count() + 1;
        assert_eq!(lines, 1);
    }

    #[test]
    fn test_to_lowercase() {
        let mut editor = editor_with("HELLO WORLD");
        editor.to_lowercase(false);
        assert_eq!(editor.document.content, "hello world");
    }

    #[test]
    fn test_to_uppercase_preserves_structure() {
        let mut editor = editor_with("hello\nworld");
        editor.to_uppercase(false);
        assert_eq!(editor.document.content, "HELLO\nWORLD");
    }

    #[test]
    fn test_duplicate_line_first() {
        let mut editor = editor_with("first\nsecond");
        editor.cursor = 0;
        editor.duplicate_line();
        assert_eq!(editor.document.content, "first\nfirst\nsecond");
    }

    #[test]
    fn test_sort_lines_alphabetical() {
        let mut editor = editor_with("cherry\napple\nbanana");
        editor.sort_lines();
        assert_eq!(editor.document.content, "apple\nbanana\ncherry");
    }

    #[test]
    fn test_move_line_up_three_lines() {
        let mut editor = editor_with("aaa\nbbb\nccc");
        editor.cursor = 5; // in "bbb"
        editor.move_line_up();
        assert_eq!(editor.document.content, "bbb\naaa\nccc");
    }

    #[test]
    fn test_move_line_up_noop_at_top() {
        let mut editor = editor_with("first\nsecond");
        editor.cursor = 0;
        editor.move_line_up(); // Should be a no-op
        assert_eq!(editor.document.content, "first\nsecond");
    }

    #[test]
    fn test_move_line_down_noop_at_bottom() {
        let mut editor = editor_with("first\nlast");
        editor.cursor = 6; // in "last"
        editor.move_line_down(); // Should be a no-op
        assert_eq!(editor.document.content, "first\nlast");
    }

    #[test]
    fn test_indent_unindent_empty_line() {
        let mut editor = editor_with("");
        editor.cursor = 0;
        editor.indent_line();
        assert_eq!(editor.document.content, "    ");
        editor.unindent_line();
        assert_eq!(editor.document.content, "");
    }

    #[test]
    fn test_toggle_comment_already_commented() {
        let mut editor = editor_with("<!-- commented -->");
        editor.cursor = 5;
        editor.toggle_comment();
        // Should uncomment
        assert_eq!(editor.document.content, "commented");
    }

    #[test]
    fn test_remove_duplicate_lines_no_dupes() {
        let mut editor = editor_with("one\ntwo\nthree");
        editor.remove_duplicate_lines();
        assert_eq!(editor.document.content, "one\ntwo\nthree");
    }

    #[test]
    fn test_multiple_undo() {
        let mut editor = editor_with("start");
        editor.to_uppercase(false);
        assert_eq!(editor.document.content, "START");
        editor.to_lowercase(false);
        assert_eq!(editor.document.content, "start");

        editor.undo();
        assert_eq!(editor.document.content, "START");
        editor.undo();
        assert_eq!(editor.document.content, "start");
    }

    #[test]
    fn test_undo_nothing_to_undo() {
        let mut editor = editor_with("test");
        assert!(!editor.can_undo());
        editor.undo(); // Should be safe no-op
        assert_eq!(editor.document.content, "test");
    }

    #[test]
    fn test_redo_nothing_to_redo() {
        let mut editor = editor_with("test");
        assert!(!editor.can_redo());
        editor.redo(); // Should be safe no-op
        assert_eq!(editor.document.content, "test");
    }

    #[test]
    fn test_join_lines_basic() {
        let mut editor = editor_with("hello\nworld");
        editor.cursor = 3;
        editor.join_lines();
        assert_eq!(editor.document.content, "hello world");
    }

    #[test]
    fn test_transpose_chars_middle() {
        let mut editor = editor_with("abc");
        editor.cursor = 1;
        editor.transpose_chars();
        assert_eq!(editor.document.content, "bac");
    }

    #[test]
    fn test_current_line_last_line() {
        let mut editor = editor_with("a\nb\nc");
        editor.cursor = 4; // in "c"
        assert_eq!(editor.current_line(), 3);
    }

    #[test]
    fn test_selection_range_same_position() {
        let mut editor = editor_with("hello");
        editor.selection_start = Some(3);
        editor.cursor = 3;
        assert_eq!(editor.selection_range(), Some((3, 3)));
    }

    #[test]
    fn test_find_all_basic() {
        let editor = editor_with("hello world hello");
        // find_all uses text() from iced, which may add trailing \n
        let results = editor.find_all("hello", true);
        assert!(results.len() >= 2);
        assert_eq!(results[0].0, 0); // First match starts at 0
    }

    #[test]
    fn test_find_all_case_insensitive() {
        let editor = editor_with("Hello HELLO hello");
        let results = editor.find_all("hello", false);
        assert!(results.len() >= 3);
    }

    #[test]
    fn test_find_all_case_sensitive() {
        let editor = editor_with("Hello HELLO hello");
        let results = editor.find_all("hello", true);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_find_all_empty_query() {
        let editor = editor_with("hello");
        let results = editor.find_all("", true);
        assert!(results.is_empty());
    }

    #[test]
    fn test_find_all_no_match() {
        let editor = editor_with("hello world");
        let results = editor.find_all("xyz", true);
        assert!(results.is_empty());
    }

    #[test]
    fn test_replace_all_basic() {
        let mut editor = editor_with("foo bar foo baz");
        let count = editor.replace_all("foo", "qux", true);
        assert_eq!(count, 2);
        assert!(editor.document.content.contains("qux bar qux baz"));
    }

    #[test]
    fn test_replace_all_case_insensitive() {
        let mut editor = editor_with("Hello HELLO hello");
        let count = editor.replace_all("hello", "hi", false);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_replace_all_no_match() {
        let mut editor = editor_with("hello world");
        let count = editor.replace_all("xyz", "abc", true);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_replace_all_empty_find() {
        let mut editor = editor_with("hello");
        let count = editor.replace_all("", "abc", true);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_insert_at_cursor_beginning() {
        let mut editor = editor_with("world");
        editor.cursor = 0;
        editor.insert_at_cursor("hello ");
        assert!(editor.document.content.starts_with("hello world"));
    }

    #[test]
    fn test_insert_at_cursor_end() {
        let mut editor = editor_with("hello");
        editor.cursor = 5;
        editor.insert_at_cursor(" world");
        assert!(editor.document.content.contains("hello world"));
    }

    #[test]
    fn test_insert_at_cursor_middle() {
        let mut editor = editor_with("helloworld");
        editor.cursor = 5;
        editor.insert_at_cursor(" ");
        assert!(editor.document.content.contains("hello world"));
    }

    #[test]
    fn test_delete_selection() {
        let mut editor = editor_with("hello world");
        editor.selection_start = Some(5);
        editor.cursor = 11;
        let deleted = editor.delete_selection();
        assert!(deleted.is_some());
        assert!(editor.document.content.starts_with("hello"));
        assert!(!editor.document.content.contains("world"));
    }

    #[test]
    fn test_delete_selection_none() {
        let mut editor = editor_with("hello");
        let deleted = editor.delete_selection();
        assert!(deleted.is_none());
    }

    #[test]
    fn test_wrap_selection_with_selection() {
        let mut editor = editor_with("hello world");
        editor.selection_start = Some(0);
        editor.cursor = 5;
        editor.wrap_selection("**", "**");
        assert!(editor.document.content.starts_with("**hello**"));
    }

    #[test]
    fn test_wrap_selection_no_selection() {
        let mut editor = editor_with("hello");
        editor.cursor = 5;
        editor.wrap_selection("**", "**");
        assert!(editor.document.content.contains("****"));
    }

    #[test]
    fn test_line_at_valid() {
        let editor = editor_with("first\nsecond\nthird");
        assert_eq!(editor.line_at(0), Some("first".to_string()));
        assert_eq!(editor.line_at(1), Some("second".to_string()));
        assert_eq!(editor.line_at(2), Some("third".to_string()));
    }

    #[test]
    fn test_line_at_invalid() {
        let editor = editor_with("only");
        assert_eq!(editor.line_at(5), None);
    }

    #[test]
    fn test_move_cursor_to() {
        let mut editor = editor_with("hello world");
        editor.selection_start = Some(0);
        editor.move_cursor_to(5);
        assert_eq!(editor.cursor, 5);
        assert!(editor.selection_start.is_none());
    }

    #[test]
    fn test_move_cursor_to_beyond_length() {
        let mut editor = editor_with("hi");
        editor.move_cursor_to(100);
        // text() may add trailing \n so cursor is clamped to text() length
        assert!(editor.cursor <= 3);
    }

    #[test]
    fn test_select_range() {
        let mut editor = editor_with("hello world");
        editor.select_range(0, 5);
        assert_eq!(editor.selection_start, Some(0));
        assert_eq!(editor.cursor, 5);
        assert_eq!(editor.selected_text(), Some("hello".to_string()));
    }

    #[test]
    fn test_select_all() {
        let mut editor = editor_with("hello world");
        editor.select_all();
        assert_eq!(editor.selection_start, Some(0));
        // cursor is set to text().len() which may include trailing \n
        assert!(editor.cursor >= 11);
    }

    #[test]
    fn test_paragraph_count() {
        let editor = editor_with("Para 1\n\nPara 2\n\nPara 3");
        assert_eq!(editor.paragraph_count(), 3);
    }

    #[test]
    fn test_paragraph_count_empty() {
        let editor = editor_with("");
        assert_eq!(editor.paragraph_count(), 0);
    }

    #[test]
    fn test_sentence_count() {
        let editor = editor_with("Hello. World! How? Fine.");
        assert_eq!(editor.sentence_count(), 4);
    }

    #[test]
    fn test_is_empty() {
        let editor = editor_with("");
        assert!(editor.is_empty());
        let editor2 = editor_with("  \n  ");
        assert!(editor2.is_empty());
        let editor3 = editor_with("hello");
        assert!(!editor3.is_empty());
    }

    #[test]
    fn test_content_length() {
        let editor = editor_with("hello");
        // content_length() uses text() from iced which may add trailing \n
        assert!(editor.content_length() >= 5);
    }

    #[test]
    fn test_clear_history() {
        let mut editor = editor_with("test");
        editor.to_uppercase(false);
        editor.undo();
        assert!(editor.can_undo() || editor.can_redo());
        editor.clear_history();
        assert!(!editor.can_undo());
        assert!(!editor.can_redo());
    }

    #[test]
    fn test_word_at_cursor() {
        let mut editor = editor_with("hello world");
        editor.cursor = 3; // in "hello"
        assert_eq!(editor.word_at_cursor(), Some("hello".to_string()));
    }

    #[test]
    fn test_word_at_cursor_at_start() {
        let mut editor = editor_with("hello");
        editor.cursor = 0;
        assert!(editor.word_at_cursor().is_none());
    }

    #[test]
    fn test_word_at_cursor_between_words() {
        let mut editor = editor_with("hello world");
        editor.cursor = 6; // start of "world"
        assert_eq!(editor.word_at_cursor(), Some("world".to_string()));
    }

    #[test]
    fn test_current_line_text() {
        let mut editor = editor_with("first line\nsecond line\nthird line");
        editor.cursor = 15; // in "second line"
        assert_eq!(editor.current_line_text(), "second line");
    }

    #[test]
    fn test_current_paragraph_text() {
        let mut editor = editor_with("Para one line one.\nPara one line two.\n\nPara two.");
        editor.cursor = 5; // in first paragraph
        let para = editor.current_paragraph_text();
        assert!(para.contains("Para one"));
    }

    #[test]
    fn test_line_word_count() {
        let editor = editor_with("one two three\nfour five");
        assert_eq!(editor.line_word_count(0), 3);
        assert_eq!(editor.line_word_count(1), 2);
        assert_eq!(editor.line_word_count(5), 0); // non-existent line
    }

    #[test]
    fn test_debug_summary() {
        let editor = editor_with("hello world");
        let summary = editor.debug_summary();
        assert!(summary.contains("cursor="));
        assert!(summary.contains("lines="));
        assert!(summary.contains("words="));
    }

    #[test]
    fn test_undo_stack_limit() {
        let mut editor = editor_with("a");
        for _ in 0..110 {
            editor.push_undo();
        }
        assert!(editor.undo_depth() <= 100);
    }

    #[test]
    fn test_redo_cleared_on_new_action() {
        let mut editor = editor_with("original");
        editor.to_uppercase(false);
        editor.undo();
        assert!(editor.can_redo());
        // New action should clear redo stack
        editor.push_undo();
        assert!(!editor.can_redo());
    }

    #[test]
    fn test_trimmed_length() {
        let editor = editor_with("hello   ");
        assert_eq!(editor.trimmed_length(), 5);
    }

    #[test]
    fn test_cursor_line_and_column() {
        let mut editor = editor_with("abc\ndef\nghi");
        editor.cursor = 5; // "ef" - line 1, column 1
        assert_eq!(editor.cursor_line(), 1);
        assert_eq!(editor.cursor_column(), 1);
    }

    #[test]
    fn test_has_selection() {
        let mut editor = editor_with("hello");
        assert!(!editor.has_selection());
        editor.selection_start = Some(0);
        assert!(editor.has_selection());
    }

    #[test]
    fn test_replace_all_creates_undo() {
        let mut editor = editor_with("foo bar foo");
        assert!(!editor.can_undo());
        editor.replace_all("foo", "baz", true);
        assert!(editor.can_undo());
        editor.undo();
        // After undo, content should be restored (may include trailing \n from iced)
        assert!(editor.document.content.contains("foo bar foo"));
    }

    #[test]
    fn test_editor_default() {
        let editor = EditorState::default();
        assert!(!editor.dirty);
        assert_eq!(editor.cursor, 0);
        assert!(!editor.can_undo());
    }

    #[test]
    fn test_load_document_resets_state() {
        let mut editor = editor_with("hello");
        editor.cursor = 3;
        editor.dirty = true;
        editor.push_undo();
        editor.selection_start = Some(0);

        let new_doc = Document::with_content("new content");
        editor.load_document(&new_doc);
        assert_eq!(editor.cursor, 0);
        assert!(!editor.dirty);
        assert!(!editor.can_undo());
        assert!(editor.selection_start.is_none());
    }

    // ---- New text transform tests ----

    #[test]
    fn test_reverse_lines() {
        let mut editor = editor_with("first\nsecond\nthird");
        editor.reverse_lines();
        assert_eq!(editor.document.content, "third\nsecond\nfirst");
    }

    #[test]
    fn test_reverse_lines_single() {
        let mut editor = editor_with("only one");
        editor.reverse_lines();
        assert_eq!(editor.document.content, "only one");
    }

    #[test]
    fn test_reverse_lines_sets_dirty() {
        let mut editor = editor_with("a\nb");
        assert!(!editor.dirty);
        editor.reverse_lines();
        assert!(editor.dirty);
    }

    #[test]
    fn test_strip_trailing_whitespace() {
        let mut editor = editor_with("hello   \nworld  \nclean");
        editor.strip_trailing_whitespace();
        assert_eq!(editor.document.content, "hello\nworld\nclean");
    }

    #[test]
    fn test_strip_trailing_whitespace_no_change() {
        let mut editor = editor_with("clean\nlines");
        editor.strip_trailing_whitespace();
        assert_eq!(editor.document.content, "clean\nlines");
        assert!(!editor.dirty);
    }

    #[test]
    fn test_collapse_whitespace() {
        let mut editor = editor_with("hello    world\nfoo   bar");
        editor.collapse_whitespace();
        assert_eq!(editor.document.content, "hello world\nfoo bar");
    }

    #[test]
    fn test_collapse_whitespace_no_change() {
        let mut editor = editor_with("clean text\nhere");
        editor.collapse_whitespace();
        assert_eq!(editor.document.content, "clean text\nhere");
        assert!(!editor.dirty);
    }

    #[test]
    fn test_number_lines() {
        let mut editor = editor_with("alpha\nbeta\ngamma");
        editor.number_lines();
        assert_eq!(editor.document.content, "1  alpha\n2  beta\n3  gamma");
    }

    #[test]
    fn test_number_lines_padding() {
        let mut editor = editor_with(
            &(0..12).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n"),
        );
        editor.number_lines();
        // 12 lines → width 2, so lines padded: " 1  line 0"
        assert!(editor.document.content.starts_with(" 1  line 0"));
        assert!(editor.document.content.contains("12  line 11"));
    }

    #[test]
    fn test_number_lines_empty() {
        let mut editor = editor_with("");
        editor.number_lines();
        // Empty doc has no lines from .lines(), so no change
        assert_eq!(editor.document.content, "");
    }

    #[test]
    fn test_extract_sentences() {
        let editor = editor_with("Hello world. How are you? I'm fine! Thanks.");
        let sentences = editor.extract_sentences();
        assert_eq!(sentences.len(), 4);
        assert_eq!(sentences[0], "Hello world.");
        assert_eq!(sentences[1], "How are you?");
        assert_eq!(sentences[2], "I'm fine!");
        assert_eq!(sentences[3], "Thanks.");
    }

    #[test]
    fn test_extract_sentences_empty() {
        let editor = editor_with("");
        let sentences = editor.extract_sentences();
        assert!(sentences.is_empty());
    }

    #[test]
    fn test_extract_sentences_no_terminator() {
        let editor = editor_with("No punctuation at end");
        let sentences = editor.extract_sentences();
        assert_eq!(sentences.len(), 1);
        assert_eq!(sentences[0], "No punctuation at end");
    }

    #[test]
    fn test_word_frequency() {
        let editor = editor_with("hello world hello rust world hello");
        let freq = editor.word_frequency();
        assert_eq!(freq.get("hello"), Some(&3));
        assert_eq!(freq.get("world"), Some(&2));
        assert_eq!(freq.get("rust"), Some(&1));
    }

    #[test]
    fn test_word_frequency_case_insensitive() {
        let editor = editor_with("Hello HELLO hello");
        let freq = editor.word_frequency();
        assert_eq!(freq.get("hello"), Some(&3));
    }

    #[test]
    fn test_word_frequency_empty() {
        let editor = editor_with("");
        let freq = editor.word_frequency();
        assert!(freq.is_empty());
    }

    #[test]
    fn test_line_lengths() {
        let editor = editor_with("hi\nhello\nworld!");
        let lengths = editor.line_lengths();
        assert_eq!(lengths, vec![2, 5, 6]);
    }

    #[test]
    fn test_line_lengths_empty() {
        let editor = editor_with("");
        let lengths = editor.line_lengths();
        // "".lines() yields nothing in Rust
        assert!(lengths.is_empty());
    }

    #[test]
    fn test_longest_line() {
        let editor = editor_with("short\na very long line here\nmed");
        let (line_idx, len) = editor.longest_line().unwrap();
        assert_eq!(line_idx, 1);
        assert_eq!(len, "a very long line here".len());
    }

    #[test]
    fn test_longest_line_empty() {
        let editor = editor_with("");
        // "".lines() yields nothing, so longest_line returns None
        assert!(editor.longest_line().is_none());
    }

    #[test]
    fn test_avg_line_length() {
        let editor = editor_with("aaa\nbbbbbb\nccc");
        // lengths: 3, 6, 3 → avg = 4.0
        let avg = editor.avg_line_length();
        assert!((avg - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_reverse_lines_undo() {
        let mut editor = editor_with("a\nb\nc");
        editor.reverse_lines();
        assert_eq!(editor.document.content, "c\nb\na");
        assert!(editor.can_undo());
        editor.undo();
        assert_eq!(editor.document.content, "a\nb\nc");
    }

    #[test]
    fn test_strip_trailing_whitespace_undo() {
        let mut editor = editor_with("hello   \nworld  ");
        editor.strip_trailing_whitespace();
        assert_eq!(editor.document.content, "hello\nworld");
        editor.undo();
        assert_eq!(editor.document.content, "hello   \nworld  ");
    }

    #[test]
    fn test_number_lines_undo() {
        let mut editor = editor_with("foo\nbar");
        editor.number_lines();
        assert!(editor.document.content.contains("1"));
        editor.undo();
        assert_eq!(editor.document.content, "foo\nbar");
    }

    #[test]
    fn test_collapse_whitespace_undo() {
        let mut editor = editor_with("a   b");
        editor.collapse_whitespace();
        assert_eq!(editor.document.content, "a b");
        editor.undo();
        assert_eq!(editor.document.content, "a   b");
    }
}
