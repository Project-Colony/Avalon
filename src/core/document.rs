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

    /// Reading time estimate in minutes (250 WPM)
    pub fn reading_time_minutes(&self) -> f64 {
        self.word_count() as f64 / 250.0
    }

    /// Speaking time estimate in minutes (150 WPM)
    pub fn speaking_time_minutes(&self) -> f64 {
        self.word_count() as f64 / 150.0
    }

    /// Unique word count
    pub fn unique_word_count(&self) -> usize {
        let mut seen = std::collections::HashSet::new();
        for word in self.content.split_whitespace() {
            let clean: String = word.chars()
                .filter(|c| c.is_alphanumeric() || *c == '\'')
                .collect();
            if !clean.is_empty() {
                seen.insert(clean.to_lowercase());
            }
        }
        seen.len()
    }

    /// Average word length in characters
    pub fn avg_word_length(&self) -> f64 {
        let words: Vec<&str> = self.content.split_whitespace().collect();
        if words.is_empty() {
            return 0.0;
        }
        let total_chars: usize = words.iter()
            .map(|w| w.chars().filter(|c| c.is_alphanumeric()).count())
            .sum();
        total_chars as f64 / words.len() as f64
    }

    /// Average sentence length in words (approximate)
    pub fn avg_sentence_length(&self) -> f64 {
        let sentences = self.sentence_count();
        if sentences == 0 {
            return 0.0;
        }
        self.word_count() as f64 / sentences as f64
    }

    /// Count syllables in a word (approximate English syllable counter)
    fn count_syllables(word: &str) -> usize {
        let word = word.to_lowercase();
        if word.len() <= 3 {
            return 1;
        }
        let mut count = 0;
        let mut prev_vowel = false;
        let vowels = ['a', 'e', 'i', 'o', 'u', 'y'];
        for ch in word.chars() {
            let is_vowel = vowels.contains(&ch);
            if is_vowel && !prev_vowel {
                count += 1;
            }
            prev_vowel = is_vowel;
        }
        // Adjust: silent 'e' at end
        if word.ends_with('e') && count > 1 {
            count -= 1;
        }
        count.max(1)
    }

    /// Flesch-Kincaid readability grade level
    /// Higher = more difficult reading; typical novel is 7-9
    pub fn readability_grade(&self) -> f64 {
        let words = self.word_count() as f64;
        let sentences = self.sentence_count() as f64;
        if words == 0.0 || sentences == 0.0 {
            return 0.0;
        }
        let syllables: usize = self.content.split_whitespace()
            .map(|w| Self::count_syllables(w))
            .sum();
        // Flesch-Kincaid Grade Level formula
        0.39 * (words / sentences) + 11.8 * (syllables as f64 / words) - 15.59
    }

    /// Flesch Reading Ease score (0-100, higher = easier to read)
    pub fn reading_ease(&self) -> f64 {
        let words = self.word_count() as f64;
        let sentences = self.sentence_count() as f64;
        if words == 0.0 || sentences == 0.0 {
            return 0.0;
        }
        let syllables: usize = self.content.split_whitespace()
            .map(|w| Self::count_syllables(w))
            .sum();
        206.835 - 1.015 * (words / sentences) - 84.6 * (syllables as f64 / words)
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

impl Reference {
    /// Create a URL reference
    pub fn from_url(title: &str, url: &str) -> Self {
        Self {
            title: title.to_string(),
            url: Some(url.to_string()),
            path: None,
            notes: String::new(),
        }
    }

    /// Create a file path reference
    pub fn from_path(title: &str, path: &str) -> Self {
        Self {
            title: title.to_string(),
            url: None,
            path: Some(path.to_string()),
            notes: String::new(),
        }
    }

    /// Check if this is a web reference
    pub fn is_web(&self) -> bool {
        self.url.is_some()
    }

    /// Check if this is a file reference
    pub fn is_file(&self) -> bool {
        self.path.is_some()
    }
}

/// A footnote or endnote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Footnote {
    pub marker: usize,
    pub text: String,
    pub is_endnote: bool,
}

impl Footnote {
    /// Create a new footnote
    pub fn new(marker: usize, text: &str) -> Self {
        Self {
            marker,
            text: text.to_string(),
            is_endnote: false,
        }
    }

    /// Create a new endnote
    pub fn endnote(marker: usize, text: &str) -> Self {
        Self {
            marker,
            text: text.to_string(),
            is_endnote: true,
        }
    }
}

impl TextSpan {
    /// Create a new span with a given style
    pub fn new(start: usize, end: usize, style: SpanStyle) -> Self {
        Self { start, end, style }
    }

    /// Length of the span in characters
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Check if span is empty
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Check if a position is within this span
    pub fn contains(&self, pos: usize) -> bool {
        pos >= self.start && pos < self.end
    }

    /// Check if two spans overlap
    pub fn overlaps(&self, other: &TextSpan) -> bool {
        self.start < other.end && other.start < self.end
    }
}

impl SpanStyle {
    /// Create a bold style
    pub fn bold() -> Self {
        Self { bold: true, ..Default::default() }
    }

    /// Create an italic style
    pub fn italic() -> Self {
        Self { italic: true, ..Default::default() }
    }

    /// Create a bold + italic style
    pub fn bold_italic() -> Self {
        Self { bold: true, italic: true, ..Default::default() }
    }

    /// Check if this span has any styling applied
    pub fn has_formatting(&self) -> bool {
        self.bold || self.italic || self.underline || self.strikethrough
            || self.font_size.is_some() || self.font_family.is_some()
            || self.color.is_some() || self.highlight.is_some()
    }
}
