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
        let mut seen = std::collections::HashSet::new();
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
        }
        self.push_undo();
        let new_content = title_case(&self.document.content);
        self.document.content = new_content.clone();
        self.content = iced::widget::text_editor::Content::with_text(&new_content);
        self.dirty = true;
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
}
