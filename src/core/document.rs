use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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
        self.content.chars()
            .filter(|c| *c == '.' || *c == '!' || *c == '?')
            .count()
            .max(if self.content.is_empty() { 0 } else { 1 })
    }

    /// Get line count
    pub fn line_count(&self) -> usize {
        if self.content.is_empty() {
            return 0;
        }
        self.content.lines().count()
    }

    /// Estimate page count (250 words per page)
    pub fn page_count(&self) -> f64 {
        self.word_count() as f64 / 250.0
    }

    /// Update content and refresh modification time
    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.modified_at = Utc::now();
    }

    /// Insert text at a given byte position
    pub fn insert_text(&mut self, position: usize, text: &str) {
        let pos = position.min(self.content.len());
        self.content.insert_str(pos, text);
        self.modified_at = Utc::now();
    }

    /// Delete text in a range
    pub fn delete_range(&mut self, start: usize, end: usize) {
        let start = start.min(self.content.len());
        let end = end.min(self.content.len());
        if start < end {
            self.content.drain(start..end);
            self.modified_at = Utc::now();
        }
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for SpanStyle {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            font_size: None,
            font_family: None,
            color: None,
            highlight: None,
        }
    }
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
