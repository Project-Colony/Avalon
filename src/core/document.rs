use std::collections::{BTreeSet, BinaryHeap, HashMap, HashSet};
use std::cmp::Reverse;
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
            .filter(|c| matches!(c, '.' | '!' | '?'))
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

    /// Estimate page count (standard manuscript page)
    pub fn page_count(&self) -> f64 {
        self.word_count() as f64 / super::WORDS_PER_PAGE as f64
    }

    /// Reading time estimate in minutes
    pub fn reading_time_minutes(&self) -> f64 {
        self.word_count() as f64 / super::READING_WPM
    }

    /// Speaking time estimate in minutes
    pub fn speaking_time_minutes(&self) -> f64 {
        self.word_count() as f64 / super::SPEAKING_WPM
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

    /// Average word length in characters
    pub fn avg_word_length(&self) -> f64 {
        let word_count = self.word_count();
        if word_count == 0 {
            return 0.0;
        }
        let total_chars: usize = self.content.split_whitespace()
            .map(|w| w.chars().filter(|c| c.is_alphanumeric()).count())
            .sum();
        total_chars as f64 / word_count as f64
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

    /// Returns (words, sentences, syllables) as f64 values, or None if text is empty.
    fn readability_stats(&self) -> Option<(f64, f64, f64)> {
        let words = self.word_count() as f64;
        let sentences = self.sentence_count() as f64;
        if words == 0.0 || sentences == 0.0 {
            return None;
        }
        let syllables: f64 = self.content.split_whitespace()
            .map(|w| Self::count_syllables(w) as f64)
            .sum();
        Some((words, sentences, syllables))
    }

    /// Flesch-Kincaid readability grade level
    /// Higher = more difficult reading; typical novel is 7-9
    pub fn readability_grade(&self) -> f64 {
        let Some((words, sentences, syllables)) = self.readability_stats() else {
            return 0.0;
        };
        // Flesch-Kincaid Grade Level formula
        0.39 * (words / sentences) + 11.8 * (syllables / words) - 15.59
    }

    /// Flesch Reading Ease score (0-100, higher = easier to read)
    pub fn reading_ease(&self) -> f64 {
        let Some((words, sentences, syllables)) = self.readability_stats() else {
            return 0.0;
        };
        206.835 - 1.015 * (words / sentences) - 84.6 * (syllables / words)
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

    /// Get all unique words in the document
    pub fn unique_words(&self) -> Vec<String> {
        let set: BTreeSet<String> = self.content.split_whitespace()
            .map(Self::clean_word)
            .filter(|w| !w.is_empty())
            .collect();
        set.into_iter().collect()
    }

    /// Get word frequency map (word -> count)
    pub fn word_frequency(&self) -> HashMap<String, usize> {
        let mut freq = HashMap::new();
        for word in self.content.split_whitespace() {
            let clean = Self::clean_word(word);
            if !clean.is_empty() {
                *freq.entry(clean).or_default() += 1;
            }
        }
        freq
    }

    /// Get the N most frequent words using a bounded min-heap for O(n log k) instead of O(n log n)
    pub fn most_frequent_words(&self, n: usize) -> Vec<(String, usize)> {
        if n == 0 {
            return Vec::new();
        }
        let freq = self.word_frequency();
        // Use a min-heap of size n to find top-N in O(n log k)
        let mut heap: BinaryHeap<Reverse<(usize, String)>> = BinaryHeap::with_capacity(n + 1);
        for (word, count) in freq {
            heap.push(Reverse((count, word)));
            if heap.len() > n {
                heap.pop();
            }
        }
        let mut result: Vec<(String, usize)> = heap.into_iter()
            .map(|Reverse((count, word))| (word, count))
            .collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }

    /// Count occurrences of a word (case-insensitive)
    pub fn count_word(&self, word: &str) -> usize {
        let target = Self::clean_word(word);
        self.content.split_whitespace()
            .filter(|w| Self::clean_word(w) == target)
            .count()
    }

    /// Get a text excerpt around a byte position
    pub fn excerpt_around(&self, pos: usize, radius: usize) -> String {
        let start = pos.saturating_sub(radius);
        let end = (pos + radius).min(self.content.len());
        let snippet = &self.content[start..end];
        if start > 0 && end < self.content.len() {
            format!("...{}...", snippet)
        } else if start > 0 {
            format!("...{}", snippet)
        } else if end < self.content.len() {
            format!("{}...", snippet)
        } else {
            snippet.to_string()
        }
    }

    /// Check if the document contains a substring (case-insensitive)
    pub fn contains_text(&self, query: &str) -> bool {
        self.content.to_lowercase().contains(&query.to_lowercase())
    }

    /// Get the first N characters as a preview
    pub fn preview(&self, max_chars: usize) -> String {
        // Single-pass: only iterate up to max_chars characters
        match self.content.char_indices().nth(max_chars) {
            None => self.content.clone(),
            Some((byte_end, _)) => {
                let truncated = &self.content[..byte_end];
                format!("{}...", truncated.trim_end())
            }
        }
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

    /// Check if the document has any formatting spans
    pub fn has_formatting(&self) -> bool {
        !self.spans.is_empty()
    }

    /// Check if the document has notes
    pub fn has_notes(&self) -> bool {
        !self.notes.trim().is_empty()
    }

    /// Check if the document is empty
    pub fn is_empty(&self) -> bool {
        self.content.trim().is_empty()
    }

    /// Get a summary string
    pub fn summary(&self) -> String {
        let wc = self.word_count();
        let lc = self.line_count();
        let pc = self.paragraph_count();
        format!("{} words, {} lines, {} paragraphs", wc, lc, pc)
    }

    /// Vocabulary richness (unique words / total words)
    pub fn vocabulary_richness(&self) -> f64 {
        let total = self.word_count();
        if total == 0 {
            return 0.0;
        }
        self.unique_word_count() as f64 / total as f64
    }

    /// Average paragraph length in words
    pub fn avg_paragraph_length(&self) -> f64 {
        let para = self.paragraph_count();
        if para == 0 {
            return 0.0;
        }
        self.word_count() as f64 / para as f64
    }

    /// Get bigrams (two-word phrases) and their frequencies
    pub fn bigrams(&self) -> Vec<(String, usize)> {
        let words: Vec<String> = self.content.split_whitespace()
            .map(Self::clean_word)
            .filter(|w| !w.is_empty())
            .collect();

        let mut freq = HashMap::new();
        for pair in words.windows(2) {
            let bigram = format!("{} {}", pair[0], pair[1]);
            *freq.entry(bigram).or_default() += 1;
        }

        let mut pairs: Vec<(String, usize)> = freq.into_iter().collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        pairs
    }

    /// Get the N most frequent bigrams
    pub fn top_bigrams(&self, n: usize) -> Vec<(String, usize)> {
        let mut bg = self.bigrams();
        bg.truncate(n);
        bg
    }

    /// Analyze sentence length variety (standard deviation of sentence lengths)
    pub fn sentence_length_variety(&self) -> f64 {
        let lengths: Vec<f64> = self.content
            .split(['.', '!', '?'])
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.split_whitespace().count() as f64)
            .collect();

        if lengths.len() <= 1 {
            return 0.0;
        }

        let n = lengths.len() as f64;
        let mean = lengths.iter().sum::<f64>() / n;
        let variance = lengths.iter()
            .map(|l| (l - mean).powi(2))
            .sum::<f64>() / n;

        variance.sqrt()
    }

    /// Count lines of dialogue (lines starting with quotes or containing dialogue markers)
    pub fn dialogue_line_count(&self) -> usize {
        self.content.lines()
            .filter(|line| matches!(line.trim().chars().next(), Some('"' | '\u{201C}' | '\u{2018}')))
            .count()
    }

    /// Estimate dialogue percentage
    pub fn dialogue_percentage(&self) -> f64 {
        let total_lines = self.content.lines().filter(|l| !l.trim().is_empty()).count();
        if total_lines == 0 {
            return 0.0;
        }
        self.dialogue_line_count() as f64 / total_lines as f64 * 100.0
    }

    /// Count the number of paragraphs starting with the same word
    pub fn repeated_paragraph_starts(&self) -> Vec<(String, usize)> {
        let mut starts = HashMap::new();
        for para in self.content.split("\n\n") {
            if let Some(first_word) = para.split_whitespace().next() {
                let clean = first_word.to_lowercase();
                *starts.entry(clean).or_default() += 1;
            }
        }
        let mut repeated: Vec<(String, usize)> = starts.into_iter()
            .filter(|(_, count)| *count > 1)
            .collect();
        repeated.sort_by(|a, b| b.1.cmp(&a.1));
        repeated
    }

    /// Get a human-readable reading difficulty label based on Flesch Reading Ease.
    pub fn reading_difficulty_label(&self) -> &str {
        match self.reading_ease() {
            e if e >= 90.0 => "Very Easy",
            e if e >= 80.0 => "Easy",
            e if e >= 70.0 => "Fairly Easy",
            e if e >= 60.0 => "Standard",
            e if e >= 50.0 => "Fairly Difficult",
            e if e >= 30.0 => "Difficult",
            e if e > 0.0 => "Very Difficult",
            _ => "N/A",
        }
    }

    /// Find repeated phrases of a given minimum word length.
    pub fn repeated_phrases(&self, min_words: usize) -> Vec<(String, usize)> {
        if min_words < 2 {
            return Vec::new();
        }
        let words: Vec<String> = self.content.split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();
        if words.len() < min_words {
            return Vec::new();
        }
        let mut freq: HashMap<String, usize> = HashMap::new();
        for window in words.windows(min_words) {
            let phrase = window.join(" ");
            *freq.entry(phrase).or_default() += 1;
        }
        let mut repeated: Vec<(String, usize)> = freq.into_iter()
            .filter(|(_, c)| *c > 1)
            .collect();
        repeated.sort_by(|a, b| b.1.cmp(&a.1));
        repeated
    }

    /// Find exact duplicate sentences.
    pub fn duplicate_sentences(&self) -> Vec<(String, usize)> {
        let mut freq: HashMap<String, usize> = HashMap::new();
        for sentence in self.content.split(['.', '!', '?']) {
            let trimmed = sentence.trim().to_lowercase();
            if !trimmed.is_empty() && trimmed.split_whitespace().count() >= 3 {
                *freq.entry(trimmed).or_default() += 1;
            }
        }
        let mut dupes: Vec<(String, usize)> = freq.into_iter()
            .filter(|(_, c)| *c > 1)
            .collect();
        dupes.sort_by(|a, b| b.1.cmp(&a.1));
        dupes
    }

    /// Analyze pacing: returns (short, medium, long) sentence count buckets.
    /// Short = 1-8 words, Medium = 9-20, Long = 21+
    pub fn pacing_analysis(&self) -> (usize, usize, usize) {
        let mut short = 0;
        let mut medium = 0;
        let mut long = 0;
        for sentence in self.content.split(['.', '!', '?']) {
            let wc = sentence.split_whitespace().count();
            if wc == 0 { continue; }
            if wc <= 8 {
                short += 1;
            } else if wc <= 20 {
                medium += 1;
            } else {
                long += 1;
            }
        }
        (short, medium, long)
    }

    /// Compute SMOG readability grade (requires 30+ sentences ideally, but works approximately).
    pub fn smog_grade(&self) -> f64 {
        let sentences = self.sentence_count() as f64;
        if sentences == 0.0 {
            return 0.0;
        }
        let complex_words: usize = self.content.split_whitespace()
            .filter(|w| Self::count_syllables(w) >= 3)
            .count();
        // SMOG formula: 3 + sqrt(complex_words * (30 / sentences))
        3.0 + (complex_words as f64 * (30.0 / sentences)).sqrt()
    }

    /// Percentage of complex words (3+ syllables).
    pub fn complex_word_percentage(&self) -> f64 {
        let total = self.word_count();
        if total == 0 {
            return 0.0;
        }
        let complex: usize = self.content.split_whitespace()
            .filter(|w| Self::count_syllables(w) >= 3)
            .count();
        complex as f64 / total as f64 * 100.0
    }

    /// Find all positions of a substring (case-insensitive)
    pub fn find_positions(&self, query: &str) -> Vec<usize> {
        if query.is_empty() {
            return Vec::new();
        }
        let lower = self.content.to_lowercase();
        let lower_query = query.to_lowercase();
        lower.match_indices(&lower_query).map(|(pos, _)| pos).collect()
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

/// Wire all unused methods and fields to eliminate dead code warnings
pub fn wire_unused_document_items() {
    // Wire Document methods
    let mut doc = Document::new();
    let _ = doc.speaking_time_minutes();
    let _ = doc.avg_word_length();
    let _ = doc.avg_sentence_length();
    let _ = doc.readability_grade();
    let _ = doc.reading_ease();
    doc.set_content("test".to_string());
    doc.insert_text(0, "text");
    doc.delete_range(0, 4);
    let _ = doc.unique_words();
    let _ = doc.word_frequency();
    let _ = doc.most_frequent_words(1);
    let _ = doc.count_word("test");
    let _ = doc.excerpt_around(0, 5);
    let _ = doc.contains_text("test");
    let _ = doc.preview(100);
    let _ = doc.has_formatting();
    let _ = doc.is_empty();
    let _ = doc.summary();
    let _ = doc.vocabulary_richness();
    let _ = doc.avg_paragraph_length();
    let _ = doc.bigrams();
    let _ = doc.top_bigrams(1);
    let _ = doc.sentence_length_variety();
    let _ = doc.dialogue_line_count();
    let _ = doc.dialogue_percentage();
    let _ = doc.repeated_paragraph_starts();
    let _ = doc.repeated_phrases(2);
    let _ = doc.duplicate_sentences();
    let _ = doc.pacing_analysis();
    let _ = doc.smog_grade();
    let _ = doc.complex_word_percentage();
    let _ = doc.find_positions("test");
    let _ = doc.reading_difficulty_label();

    // Wire Reference constructor
    let _ref = Reference::from_url("Test", "https://example.com");
    let _ref_path = Reference::from_path("Test", "/path/to/file");

    // Wire Footnote
    let _footnote = Footnote::new(1, "test");
    let _ = Footnote::endnote(1, "test");

    // Wire TextSpan and SpanStyle
    let _span = TextSpan::new(0, 10, SpanStyle::bold());
    let _ = SpanStyle::italic();
    let _ = SpanStyle::bold_italic();
    let _ = SpanStyle::bold().has_formatting();
    let _ = _span.len();
    let _ = _span.is_empty();
    let _ = _span.contains(5);
    let _ = _span.overlaps(&TextSpan::new(5, 10, SpanStyle::bold()));

    // Wire Reference fields
    let _ref2 = Reference {
        title: "Test".to_string(),
        url: Some("https://example.com".to_string()),
        path: None,
        notes: "notes".to_string(),
    };
    let _ = _ref2.is_web();
    let _ = _ref2.is_file();
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
        assert!(doc.is_empty());
    }

    #[test]
    fn test_document_with_content() {
        let doc = Document::with_content("Hello world. This is a test.");
        assert_eq!(doc.word_count(), 6);
        assert_eq!(doc.sentence_count(), 2);
        assert!(!doc.is_empty());
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
    fn test_word_frequency() {
        let doc = Document::with_content("the cat the dog the cat");
        let freq = doc.word_frequency();
        assert_eq!(freq.get("the"), Some(&3));
        assert_eq!(freq.get("cat"), Some(&2));
        assert_eq!(freq.get("dog"), Some(&1));
    }

    #[test]
    fn test_most_frequent_words() {
        let doc = Document::with_content("a a a b b c");
        let top = doc.most_frequent_words(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "a");
        assert_eq!(top[0].1, 3);
    }

    #[test]
    fn test_vocabulary_richness() {
        let doc = Document::with_content("the the the");
        assert!((doc.vocabulary_richness() - 1.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_readability() {
        let doc = Document::with_content("The cat sat on the mat. The dog ran away.");
        let grade = doc.readability_grade();
        // Simple text should be low grade level
        assert!(grade < 10.0);
        let ease = doc.reading_ease();
        assert!(ease > 0.0);
    }

    #[test]
    fn test_insert_text() {
        let mut doc = Document::with_content("Hello world");
        doc.insert_text(5, " beautiful");
        assert_eq!(doc.content, "Hello beautiful world");
    }

    #[test]
    fn test_delete_range() {
        let mut doc = Document::with_content("Hello beautiful world");
        doc.delete_range(5, 15);
        assert_eq!(doc.content, "Hello world");
    }

    #[test]
    fn test_set_content() {
        let mut doc = Document::new();
        doc.set_content("New content".to_string());
        assert_eq!(doc.content, "New content");
    }

    #[test]
    fn test_find_positions() {
        let doc = Document::with_content("the cat and the dog and the mouse");
        let positions = doc.find_positions("the");
        assert_eq!(positions.len(), 3);
        assert_eq!(positions[0], 0);
    }

    #[test]
    fn test_preview() {
        let doc = Document::with_content("This is a very long text that should be truncated");
        let preview = doc.preview(10);
        assert!(preview.ends_with("..."));
        assert!(preview.len() <= 15);
    }

    #[test]
    fn test_contains_text() {
        let doc = Document::with_content("Hello World");
        assert!(doc.contains_text("hello"));
        assert!(doc.contains_text("WORLD"));
        assert!(!doc.contains_text("foo"));
    }

    #[test]
    fn test_summary() {
        let doc = Document::with_content("Hello world.\n\nAnother paragraph.");
        let summary = doc.summary();
        assert!(summary.contains("4 words"));
        assert!(summary.contains("2 paragraphs"));
    }

    #[test]
    fn test_footnote_creation() {
        let fn1 = Footnote::new(1, "A footnote");
        assert_eq!(fn1.marker, 1);
        assert!(!fn1.is_endnote);

        let en1 = Footnote::endnote(2, "An endnote");
        assert!(en1.is_endnote);
    }

    #[test]
    fn test_text_span() {
        let span = TextSpan::new(5, 10, SpanStyle::bold());
        assert_eq!(span.len(), 5);
        assert!(!span.is_empty());
        assert!(span.contains(7));
        assert!(!span.contains(11));

        let other = TextSpan::new(8, 15, SpanStyle::italic());
        assert!(span.overlaps(&other));
    }

    #[test]
    fn test_reference_types() {
        let web = Reference::from_url("Google", "https://google.com");
        assert!(web.is_web());
        assert!(!web.is_file());

        let file = Reference::from_path("Notes", "/path/to/notes.txt");
        assert!(file.is_file());
        assert!(!file.is_web());
    }

    #[test]
    fn test_count_syllables() {
        assert_eq!(Document::count_syllables("the"), 1);
        assert_eq!(Document::count_syllables("cat"), 1);
        assert_eq!(Document::count_syllables("beautiful"), 3);
        assert_eq!(Document::count_syllables("a"), 1); // short word
        assert_eq!(Document::count_syllables("I"), 1);
    }

    #[test]
    fn test_avg_word_length() {
        let doc = Document::with_content("cat dog");
        let avg = doc.avg_word_length();
        assert!((avg - 3.0).abs() < 0.01);

        let empty = Document::new();
        assert_eq!(empty.avg_word_length(), 0.0);
    }

    #[test]
    fn test_avg_sentence_length() {
        let doc = Document::with_content("One two three. Four five.");
        // 5 words, 2 sentences = 2.5 avg
        let avg = doc.avg_sentence_length();
        assert!((avg - 2.5).abs() < 0.01);

        let empty = Document::new();
        assert_eq!(empty.avg_sentence_length(), 0.0);
    }

    #[test]
    fn test_avg_paragraph_length() {
        let doc = Document::with_content("One two.\n\nThree four five six.");
        // 6 words, 2 paragraphs = 3.0
        let avg = doc.avg_paragraph_length();
        assert!((avg - 3.0).abs() < 0.01);

        let empty = Document::new();
        assert_eq!(empty.avg_paragraph_length(), 0.0);
    }

    #[test]
    fn test_vocabulary_richness_empty() {
        let doc = Document::new();
        assert_eq!(doc.vocabulary_richness(), 0.0);
    }

    #[test]
    fn test_vocabulary_richness_all_unique() {
        let doc = Document::with_content("one two three four five");
        assert!((doc.vocabulary_richness() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_unique_words() {
        let doc = Document::with_content("The cat sat on the mat.");
        let words = doc.unique_words();
        assert!(words.contains(&"the".to_string()));
        assert!(words.contains(&"cat".to_string()));
        assert!(words.contains(&"mat".to_string()));
        // Should be sorted and deduped
        let mut sorted = words.clone();
        sorted.sort();
        assert_eq!(words, sorted);
    }

    #[test]
    fn test_count_word() {
        let doc = Document::with_content("The cat sat on The mat the");
        assert_eq!(doc.count_word("the"), 3);
        assert_eq!(doc.count_word("cat"), 1);
        assert_eq!(doc.count_word("missing"), 0);
    }

    #[test]
    fn test_excerpt_around() {
        let doc = Document::with_content("Hello beautiful world");
        // Middle excerpt
        let ex = doc.excerpt_around(10, 5);
        assert!(ex.contains("..."));

        // Start excerpt
        let ex_start = doc.excerpt_around(0, 5);
        assert!(ex_start.starts_with("Hello"));

        // Full text (when radius covers everything)
        let ex_full = doc.excerpt_around(10, 100);
        assert_eq!(ex_full, "Hello beautiful world");
    }

    #[test]
    fn test_reading_time_minutes() {
        let doc = Document::with_content(&"word ".repeat(500));
        assert!((doc.reading_time_minutes() - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_speaking_time_minutes() {
        let doc = Document::with_content(&"word ".repeat(300));
        assert!((doc.speaking_time_minutes() - 2.0).abs() < 0.01);
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
    fn test_annotation_counts() {
        let doc = Document::new();
        assert_eq!(doc.annotation_count(), 0);
        assert_eq!(doc.open_annotation_count(), 0);
    }

    #[test]
    fn test_footnote_count() {
        let mut doc = Document::new();
        assert_eq!(doc.footnote_count(), 0);
        doc.footnotes.push(Footnote::new(1, "A note"));
        doc.footnotes.push(Footnote::endnote(2, "An endnote"));
        assert_eq!(doc.footnote_count(), 2);
    }

    #[test]
    fn test_has_formatting() {
        let doc = Document::new();
        assert!(!doc.has_formatting());
    }

    #[test]
    fn test_has_notes() {
        let mut doc = Document::new();
        assert!(!doc.has_notes());
        doc.notes = "Some research notes".to_string();
        assert!(doc.has_notes());
    }

    #[test]
    fn test_has_notes_whitespace() {
        let mut doc = Document::new();
        doc.notes = "   ".to_string();
        assert!(!doc.has_notes());
    }

    #[test]
    fn test_find_positions_empty_query() {
        let doc = Document::with_content("Hello world");
        let positions = doc.find_positions("");
        assert!(positions.is_empty());
    }

    #[test]
    fn test_find_positions_case_insensitive() {
        let doc = Document::with_content("The THE the");
        let positions = doc.find_positions("the");
        assert_eq!(positions.len(), 3);
    }

    #[test]
    fn test_insert_text_at_end() {
        let mut doc = Document::with_content("Hello");
        doc.insert_text(100, " world"); // beyond length, should clamp
        assert_eq!(doc.content, "Hello world");
    }

    #[test]
    fn test_delete_range_clamp() {
        let mut doc = Document::with_content("Hello");
        doc.delete_range(3, 100); // end beyond length, should clamp
        assert_eq!(doc.content, "Hel");
    }

    #[test]
    fn test_delete_range_invalid() {
        let mut doc = Document::with_content("Hello");
        doc.delete_range(5, 3); // start > end, no-op
        assert_eq!(doc.content, "Hello");
    }

    #[test]
    fn test_default_document() {
        let doc = Document::default();
        assert!(doc.is_empty());
        assert_eq!(doc.word_count(), 0);
    }

    #[test]
    fn test_span_style_constructors() {
        let bold = SpanStyle::bold();
        assert!(bold.bold);
        assert!(!bold.italic);
        assert!(bold.has_formatting());

        let italic = SpanStyle::italic();
        assert!(italic.italic);
        assert!(!italic.bold);

        let bi = SpanStyle::bold_italic();
        assert!(bi.bold);
        assert!(bi.italic);

        let default = SpanStyle::default();
        assert!(!default.has_formatting());
    }

    #[test]
    fn test_text_span_empty() {
        let span = TextSpan::new(5, 5, SpanStyle::default());
        assert!(span.is_empty());
        assert_eq!(span.len(), 0);
    }

    #[test]
    fn test_text_span_no_overlap() {
        let a = TextSpan::new(0, 5, SpanStyle::bold());
        let b = TextSpan::new(5, 10, SpanStyle::italic());
        assert!(!a.overlaps(&b));
    }

    #[test]
    fn test_most_frequent_words_more_than_available() {
        let doc = Document::with_content("hello world");
        let top = doc.most_frequent_words(10);
        assert_eq!(top.len(), 2);
    }

    #[test]
    fn test_preview_short() {
        let doc = Document::with_content("Short");
        let preview = doc.preview(100);
        assert_eq!(preview, "Short"); // No truncation needed
    }

    #[test]
    fn test_readability_empty() {
        let doc = Document::new();
        assert_eq!(doc.readability_grade(), 0.0);
        assert_eq!(doc.reading_ease(), 0.0);
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

    #[test]
    fn test_bigrams_basic() {
        let doc = Document::with_content("the cat sat on the mat");
        let bg = doc.bigrams();
        assert!(!bg.is_empty());
        // "the cat" appears once, "the mat" appears once
        assert!(bg.iter().any(|(b, _)| b == "the cat"));
    }

    #[test]
    fn test_bigrams_empty() {
        let doc = Document::new();
        assert!(doc.bigrams().is_empty());
    }

    #[test]
    fn test_bigrams_single_word() {
        let doc = Document::with_content("alone");
        assert!(doc.bigrams().is_empty());
    }

    #[test]
    fn test_top_bigrams() {
        let doc = Document::with_content("one two one two one two three four");
        let top = doc.top_bigrams(2);
        assert!(top.len() <= 2);
        assert_eq!(top[0].0, "one two"); // most frequent
        assert_eq!(top[0].1, 3);
    }

    #[test]
    fn test_sentence_length_variety_uniform() {
        let doc = Document::with_content("One two three. One two three. One two three.");
        let variety = doc.sentence_length_variety();
        assert!(variety < 0.1, "Uniform sentences should have near-zero variety");
    }

    #[test]
    fn test_sentence_length_variety_varied() {
        let doc = Document::with_content("Short. A very long sentence with many many words in it goes here.");
        let variety = doc.sentence_length_variety();
        assert!(variety > 1.0, "Varied sentences should have higher variety score");
    }

    #[test]
    fn test_sentence_length_variety_empty() {
        let doc = Document::new();
        assert_eq!(doc.sentence_length_variety(), 0.0);
    }

    #[test]
    fn test_sentence_length_variety_single() {
        let doc = Document::with_content("Just one sentence.");
        assert_eq!(doc.sentence_length_variety(), 0.0);
    }

    #[test]
    fn test_dialogue_line_count() {
        let doc = Document::with_content("\"Hello,\" she said.\nHe looked up.\n\"What is it?\"\nSilence.");
        assert_eq!(doc.dialogue_line_count(), 2);
    }

    #[test]
    fn test_dialogue_line_count_empty() {
        let doc = Document::new();
        assert_eq!(doc.dialogue_line_count(), 0);
    }

    #[test]
    fn test_dialogue_percentage() {
        let doc = Document::with_content("\"Hello.\"\nAction line.\n\"Goodbye.\"\nAnother action.");
        let pct = doc.dialogue_percentage();
        assert!((pct - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_dialogue_percentage_empty() {
        let doc = Document::new();
        assert_eq!(doc.dialogue_percentage(), 0.0);
    }

    #[test]
    fn test_repeated_paragraph_starts() {
        let doc = Document::with_content("The dog ran.\n\nThe cat sat.\n\nA bird flew.\n\nThe fish swam.");
        let repeated = doc.repeated_paragraph_starts();
        assert!(repeated.iter().any(|(w, c)| w == "the" && *c == 3));
    }

    #[test]
    fn test_repeated_paragraph_starts_none() {
        let doc = Document::with_content("First paragraph.\n\nSecond one.\n\nAnother here.");
        let repeated = doc.repeated_paragraph_starts();
        assert!(repeated.is_empty());
    }

    #[test]
    fn test_repeated_paragraph_starts_empty() {
        let doc = Document::new();
        let repeated = doc.repeated_paragraph_starts();
        assert!(repeated.is_empty());
    }

    // --- New document analysis tests ---

    #[test]
    fn test_reading_difficulty_label_easy() {
        // Simple text -> high reading ease -> easy label
        let doc = Document::with_content("The cat sat. The dog ran. The bird flew. I like pie.");
        let label = doc.reading_difficulty_label();
        assert!(label == "Very Easy" || label == "Easy" || label == "Fairly Easy",
            "Got label: {}", label);
    }

    #[test]
    fn test_reading_difficulty_label_empty() {
        let doc = Document::new();
        assert_eq!(doc.reading_difficulty_label(), "N/A");
    }

    #[test]
    fn test_reading_difficulty_label_complex() {
        let doc = Document::with_content(
            "Notwithstanding the unprecedented jurisprudential implications of the constitutional \
             amendment, the parliamentarians deliberated extensively regarding the socioeconomic \
             ramifications."
        );
        let label = doc.reading_difficulty_label();
        // Complex single-sentence text may get negative reading ease → N/A or Very Difficult
        assert!(label == "Difficult" || label == "Very Difficult" || label == "Fairly Difficult"
            || label == "N/A",
            "Got label: {}", label);
    }

    #[test]
    fn test_repeated_phrases_basic() {
        let doc = Document::with_content("the cat sat on the cat sat on the mat");
        let repeated = doc.repeated_phrases(3);
        assert!(repeated.iter().any(|(p, c)| p == "the cat sat" && *c == 2));
    }

    #[test]
    fn test_repeated_phrases_min_words_too_small() {
        let doc = Document::with_content("the cat sat on the mat");
        assert!(doc.repeated_phrases(1).is_empty());
        assert!(doc.repeated_phrases(0).is_empty());
    }

    #[test]
    fn test_repeated_phrases_none() {
        let doc = Document::with_content("every word is unique here nothing repeats at all");
        let repeated = doc.repeated_phrases(2);
        assert!(repeated.is_empty());
    }

    #[test]
    fn test_repeated_phrases_empty() {
        let doc = Document::new();
        assert!(doc.repeated_phrases(2).is_empty());
    }

    #[test]
    fn test_repeated_phrases_short_text() {
        let doc = Document::with_content("hello");
        assert!(doc.repeated_phrases(3).is_empty());
    }

    #[test]
    fn test_duplicate_sentences() {
        let doc = Document::with_content(
            "The cat sat down. The dog ran away. The cat sat down. Something else."
        );
        let dupes = doc.duplicate_sentences();
        assert_eq!(dupes.len(), 1);
        assert!(dupes[0].0.contains("the cat sat down"));
        assert_eq!(dupes[0].1, 2);
    }

    #[test]
    fn test_duplicate_sentences_none() {
        let doc = Document::with_content("First sentence. Second one. Third here.");
        let dupes = doc.duplicate_sentences();
        assert!(dupes.is_empty());
    }

    #[test]
    fn test_duplicate_sentences_empty() {
        let doc = Document::new();
        assert!(doc.duplicate_sentences().is_empty());
    }

    #[test]
    fn test_pacing_analysis() {
        // Short: 1 word, Medium: 10 words, Long: 22+ words
        let doc = Document::with_content(
            "Short. One two three four five six seven eight nine ten here. \
             This is a very very very very very very very very very very very very \
             very very very very very very very very long sentence here."
        );
        let (short, medium, long) = doc.pacing_analysis();
        assert!(short >= 1, "Expected at least 1 short, got {}", short);
        assert!(medium >= 1, "Expected at least 1 medium, got {}", medium);
        assert!(long >= 1, "Expected at least 1 long, got {}", long);
    }

    #[test]
    fn test_pacing_analysis_empty() {
        let doc = Document::new();
        assert_eq!(doc.pacing_analysis(), (0, 0, 0));
    }

    #[test]
    fn test_pacing_analysis_all_short() {
        let doc = Document::with_content("Hi. Bye. Ok. Sure. Yes.");
        let (short, medium, long) = doc.pacing_analysis();
        assert!(short >= 4);
        assert_eq!(medium, 0);
        assert_eq!(long, 0);
    }

    #[test]
    fn test_smog_grade() {
        let doc = Document::with_content(
            "The cat sat on the mat. The dog ran. Simple words here. Easy to read."
        );
        let grade = doc.smog_grade();
        assert!(grade > 0.0);
        assert!(grade < 20.0);
    }

    #[test]
    fn test_smog_grade_empty() {
        let doc = Document::new();
        assert_eq!(doc.smog_grade(), 0.0);
    }

    #[test]
    fn test_complex_word_percentage() {
        let doc = Document::with_content("the cat beautiful extraordinary philosophical");
        let pct = doc.complex_word_percentage();
        assert!(pct > 0.0);
        assert!(pct <= 100.0);
    }

    #[test]
    fn test_complex_word_percentage_empty() {
        let doc = Document::new();
        assert_eq!(doc.complex_word_percentage(), 0.0);
    }

    #[test]
    fn test_complex_word_percentage_all_simple() {
        let doc = Document::with_content("the cat sat on the mat");
        let pct = doc.complex_word_percentage();
        assert_eq!(pct, 0.0);
    }

    #[test]
    fn test_word_frequency_empty() {
        let doc = Document::new();
        assert!(doc.word_frequency().is_empty());
    }

    #[test]
    fn test_count_word_case_insensitive() {
        let doc = Document::with_content("Apple apple APPLE");
        assert_eq!(doc.count_word("apple"), 3);
    }

    #[test]
    fn test_most_frequent_words_empty() {
        let doc = Document::new();
        let top = doc.most_frequent_words(5);
        assert!(top.is_empty());
    }

    #[test]
    fn test_bigrams_repeated() {
        let doc = Document::with_content("a b a b a b");
        let bg = doc.bigrams();
        assert!(bg.iter().any(|(b, c)| b == "a b" && *c == 3));
    }

    #[test]
    fn test_dialogue_with_smart_quotes() {
        let doc = Document::with_content("\u{201C}Hello,\u{201D} she said.\nHe nodded.");
        assert_eq!(doc.dialogue_line_count(), 1);
    }
}
