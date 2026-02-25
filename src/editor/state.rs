use std::collections::{HashSet, VecDeque};
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
    pub undo_stack: VecDeque<UndoEntry>,
    /// Redo stack
    pub redo_stack: VecDeque<UndoEntry>,
    /// Whether the document has unsaved changes
    pub dirty: bool,
    /// Scroll offset (in lines)
    pub scroll_offset: f32,
    /// The iced text editor content
    pub content: iced::widget::text_editor::Content,
}

/// An entry in the undo/redo stack
#[derive(Debug, Clone)]
pub struct UndoEntry {
    pub content: String,
    pub cursor: usize,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            document: Document::new(),
            cursor: 0,
            selection_start: None,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            dirty: false,
            scroll_offset: 0.0,
            content: iced::widget::text_editor::Content::new(),
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

    /// Get the line number (0-based) at the cursor position
    pub fn cursor_line(&self) -> usize {
        let text = &self.document.content;
        let pos = self.cursor.min(text.len());
        text[..pos].matches('\n').count()
    }

    /// Maximum total bytes across all undo entries before evicting old ones.
    const UNDO_MEMORY_BUDGET: usize = 10 * 1024 * 1024; // 10 MiB

    /// Push current state onto undo stack
    pub fn push_undo(&mut self) {
        // Skip if top of undo stack already has identical content
        if let Some(top) = self.undo_stack.back() {
            if top.content == self.document.content && top.cursor == self.cursor {
                return;
            }
        }

        self.undo_stack.push_back(UndoEntry {
            content: self.document.content.clone(),
            cursor: self.cursor,
        });
        self.redo_stack.clear();

        // Limit undo stack by count
        if self.undo_stack.len() > 100 {
            self.undo_stack.pop_front();
        }

        // Limit undo stack by total memory usage
        let mut total_bytes: usize = self.undo_stack.iter().map(|e| e.content.len()).sum();
        while total_bytes > Self::UNDO_MEMORY_BUDGET && self.undo_stack.len() > 1 {
            if let Some(evicted) = self.undo_stack.pop_front() {
                total_bytes -= evicted.content.len();
            }
        }
    }

    /// Undo the last change
    pub fn undo(&mut self) -> bool {
        if let Some(entry) = self.undo_stack.pop_back() {
            self.redo_stack.push_back(UndoEntry {
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
        if let Some(entry) = self.redo_stack.pop_back() {
            self.undo_stack.push_back(UndoEntry {
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

    /// Transpose the two characters around the cursor
    pub fn transpose_chars(&mut self) {
        let text = self.document.content.clone();
        let pos = self.cursor;
        if (pos == 0 || pos >= text.len())
            && (pos < 2 || text.len() < 2) {
                return;
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
    #[allow(clippy::wrong_self_convention)]
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
    #[allow(clippy::wrong_self_convention)]
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
    #[allow(clippy::wrong_self_convention)]
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
        let mut editor = EditorState::new();
        editor.load_document(&doc);
        editor
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
        assert!(!editor.undo_stack.is_empty());
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
        // selection_only=true but no selection -> no change
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
        assert!(!editor.undo_stack.is_empty());
        editor.undo();
        assert_eq!(editor.document.content, "original");
        assert!(!editor.redo_stack.is_empty());
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
        assert!(editor.undo_stack.is_empty());
        editor.undo(); // Should be safe no-op
        assert_eq!(editor.document.content, "test");
    }

    #[test]
    fn test_redo_nothing_to_redo() {
        let mut editor = editor_with("test");
        assert!(editor.redo_stack.is_empty());
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
    fn test_editor_default() {
        let editor = EditorState::default();
        assert!(!editor.dirty);
        assert_eq!(editor.cursor, 0);
        assert!(editor.undo_stack.is_empty());
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
        assert!(editor.undo_stack.is_empty());
        assert!(editor.selection_start.is_none());
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
    fn test_has_selection() {
        let mut editor = editor_with("hello");
        assert!(!editor.has_selection());
        editor.selection_start = Some(0);
        assert!(editor.has_selection());
    }

    #[test]
    fn test_undo_stack_limit() {
        let mut editor = editor_with("a");
        for _ in 0..110 {
            editor.push_undo();
        }
        assert!(editor.undo_stack.len() <= 100);
    }

    #[test]
    fn test_redo_cleared_on_new_action() {
        let mut editor = editor_with("original");
        editor.to_uppercase(false);
        editor.undo();
        assert!(!editor.redo_stack.is_empty());
        // New action should clear redo stack
        editor.push_undo();
        assert!(editor.redo_stack.is_empty());
    }

    #[test]
    fn test_cursor_line_and_column() {
        let mut editor = editor_with("abc\ndef\nghi");
        editor.cursor = 5; // "ef" - line 1, column 1
        assert_eq!(editor.cursor_line(), 1);
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
}
