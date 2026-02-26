use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Pre-computed text statistics from a single pass over the content.
/// Use this instead of calling `word_count()`, `char_count()`, etc. separately
/// when you need multiple stats at once (e.g., status bar rendering).
#[derive(Debug, Clone, Default)]
pub struct TextStats {
    pub word_count: usize,
    pub char_count: usize,
    pub char_count_no_spaces: usize,
    pub paragraph_count: usize,
    pub sentence_count: usize,
    pub line_count: usize,
    pub page_count: f64,
}

impl TextStats {
    /// Compute all text statistics in a single pass.
    pub fn from_text(text: &str) -> Self {
        if text.is_empty() {
            return Self::default();
        }

        let char_count = text.len();
        let word_count = text.split_whitespace().count();
        let char_count_no_spaces = text.chars().filter(|c| !c.is_whitespace()).count();
        let paragraph_count = text.split("\n\n").filter(|p| !p.trim().is_empty()).count();
        let line_count = text.lines().count();
        let sentence_count = text.chars().filter(|c| matches!(c, '.' | '!' | '?')).count().max(1);
        let page_count = word_count as f64 / super::WORDS_PER_PAGE as f64;

        Self {
            word_count,
            char_count,
            char_count_no_spaces,
            paragraph_count,
            sentence_count,
            line_count,
            page_count,
        }
    }
}

/// A Document represents the text content of a binder item.
/// It stores the content as plain text / Markdown with optional rich text spans.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// The raw text content (Markdown-compatible)
    pub content: String,
    /// Rich text spans for formatting beyond Markdown
    pub spans: Vec<TextSpan>,
    /// Notes/comments associated with this document
    pub notes: String,
    /// Document-level references/links
    pub references: Vec<Reference>,
    /// Footnotes / endnotes
    #[serde(default)]
    pub footnotes: Vec<Footnote>,
    /// Inline annotations / comments
    #[serde(default)]
    pub annotations: Vec<crate::core::annotation::Annotation>,
    /// Last modification time
    pub modified_at: DateTime<Utc>,
    /// Cursor position (for restoring editing state)
    pub cursor_position: usize,
}

impl Document {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            spans: Vec::new(),
            notes: String::new(),
            references: Vec::new(),
            footnotes: Vec::new(),
            annotations: Vec::new(),
            modified_at: Utc::now(),
            cursor_position: 0,
        }
    }

    pub fn with_content(content: &str) -> Self {
        let mut doc = Self::new();
        doc.content = content.to_string();
        doc
    }

    /// Get word count
    pub fn word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }

    /// Get character count (with spaces)
    pub fn char_count(&self) -> usize {
        self.content.len()
    }

    /// Get character count (without spaces)
    #[allow(dead_code)] // used only in tests; production uses TextStats
    pub fn char_count_no_spaces(&self) -> usize {
        self.content.chars().filter(|c| !c.is_whitespace()).count()
    }

    /// Get paragraph count
    pub fn paragraph_count(&self) -> usize {
        if self.content.is_empty() {
            return 0;
        }
        self.content.split("\n\n").filter(|p| !p.trim().is_empty()).count()
    }

    /// Get sentence count (approximate)
    pub fn sentence_count(&self) -> usize {
        self.content
            .chars()
            .filter(|c| matches!(c, '.' | '!' | '?'))
            .count()
            .max(if self.content.is_empty() { 0 } else { 1 })
    }

    /// Get line count
    #[allow(dead_code)]
    pub fn line_count(&self) -> usize {
        if self.content.is_empty() {
            return 0;
        }
        self.content.lines().count()
    }

    /// Estimate page count (standard manuscript page)
    pub fn page_count(&self) -> f64 {
        self.word_count() as f64 / super::WORDS_PER_PAGE as f64
    }

    /// Compute all text statistics in a single pass.
    /// Prefer this over calling individual stat methods when you need multiple values.
    pub fn stats(&self) -> TextStats {
        TextStats::from_text(&self.content)
    }

    /// Count the number of annotations
    pub fn annotation_count(&self) -> usize {
        self.annotations.len()
    }

    /// Count open (unresolved) annotations
    pub fn open_annotation_count(&self) -> usize {
        self.annotations.iter().filter(|a| !a.resolved).count()
    }

    /// Count footnotes
    pub fn footnote_count(&self) -> usize {
        self.footnotes.len()
    }

    /// Check if the document has notes
    pub fn has_notes(&self) -> bool {
        !self.notes.trim().is_empty()
    }

    /// Clean a word by keeping only alphanumeric chars and apostrophes, then lowercasing.
    fn clean_word(word: &str) -> String {
        word.chars()
            .filter(|c| c.is_alphanumeric() || *c == '\'')
            .flat_map(char::to_lowercase)
            .collect()
    }

    /// Unique word count
    pub fn unique_word_count(&self) -> usize {
        let mut seen = HashSet::new();
        for word in self.content.split_whitespace() {
            let clean = Self::clean_word(word);
            if !clean.is_empty() {
                seen.insert(clean);
            }
        }
        seen.len()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// A span of styled text within a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSpan {
    pub start: usize,
    pub end: usize,
    pub style: SpanStyle,
}

/// Styling for a text span
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpanStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub font_size: Option<f32>,
    pub font_family: Option<String>,
    pub color: Option<String>,
    pub highlight: Option<String>,
}

/// A reference/link to an external resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub title: String,
    pub url: Option<String>,
    pub path: Option<String>,
    pub notes: String,
}

/// A footnote or endnote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Footnote {
    pub marker: usize,
    pub text: String,
    pub is_endnote: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_new() {
        let doc = Document::new();
        assert!(doc.content.is_empty());
        assert_eq!(doc.word_count(), 0);
        assert_eq!(doc.char_count(), 0);
    }

    #[test]
    fn test_document_with_content() {
        let doc = Document::with_content("Hello world. This is a test.");
        assert_eq!(doc.word_count(), 6);
        assert_eq!(doc.sentence_count(), 2);
    }

    #[test]
    fn test_word_count() {
        let doc = Document::with_content("one two three four five");
        assert_eq!(doc.word_count(), 5);
    }

    #[test]
    fn test_paragraph_count() {
        let doc = Document::with_content("First paragraph.\n\nSecond paragraph.\n\nThird paragraph.");
        assert_eq!(doc.paragraph_count(), 3);
    }

    #[test]
    fn test_sentence_count() {
        let doc = Document::with_content("Hello! How are you? I am fine.");
        assert_eq!(doc.sentence_count(), 3);
    }

    #[test]
    fn test_char_counts() {
        let doc = Document::with_content("a b c");
        assert_eq!(doc.char_count(), 5);
        assert_eq!(doc.char_count_no_spaces(), 3);
    }

    #[test]
    fn test_unique_word_count() {
        let doc = Document::with_content("the cat sat on the mat the cat");
        assert_eq!(doc.unique_word_count(), 5); // the, cat, sat, on, mat
    }

    #[test]
    fn test_page_count() {
        let doc = Document::with_content(&"word ".repeat(500));
        assert!((doc.page_count() - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_line_count() {
        let doc = Document::with_content("line 1\nline 2\nline 3");
        assert_eq!(doc.line_count(), 3);

        let empty = Document::new();
        assert_eq!(empty.line_count(), 0);
    }

    #[test]
    fn test_default_document() {
        let doc = Document::default();
        assert_eq!(doc.word_count(), 0);
    }

    #[test]
    fn test_sentence_count_no_punctuation() {
        let doc = Document::with_content("No ending punctuation here");
        assert_eq!(doc.sentence_count(), 1); // at least 1 for non-empty
    }

    #[test]
    fn test_paragraph_count_empty() {
        let doc = Document::new();
        assert_eq!(doc.paragraph_count(), 0);
    }

    #[test]
    fn test_char_count_no_spaces() {
        let doc = Document::with_content("a b c d");
        assert_eq!(doc.char_count_no_spaces(), 4);
        assert_eq!(doc.char_count(), 7);
    }
}
