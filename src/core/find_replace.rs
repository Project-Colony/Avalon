#![allow(dead_code)]
use uuid::Uuid;
use regex::Regex;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Scope for a find-and-replace operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchScope {
    CurrentDocument,
    SelectedDocuments(Vec<Uuid>),
    EntireProject,
    DraftOnly,
    CustomCollection(String),
}

/// Options that drive a find-and-replace operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindReplaceOptions {
    pub query: String,
    pub replacement: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub use_regex: bool,
    pub scope: SearchScope,
    /// When true, smart-case replacement is applied: the replacement string
    /// mirrors the casing pattern of the matched text.
    pub preserve_case: bool,
}

impl Default for FindReplaceOptions {
    fn default() -> Self {
        Self {
            query: String::new(),
            replacement: String::new(),
            case_sensitive: false,
            whole_word: false,
            use_regex: false,
            scope: SearchScope::EntireProject,
            preserve_case: false,
        }
    }
}

/// A single match location inside a text body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchLocation {
    pub start: usize,
    pub end: usize,
    pub line_number: usize,
    pub column: usize,
    pub context_before: String,
    pub matched_text: String,
    pub context_after: String,
    pub preview_replacement: String,
}

/// All matches found in a single document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindResult {
    pub item_id: Uuid,
    pub item_title: String,
    pub matches: Vec<MatchLocation>,
}

/// The outcome of replacing text in one document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceResult {
    pub item_id: Uuid,
    pub original_text: String,
    pub new_text: String,
    pub replacements_made: usize,
}

/// Report produced by a batch replace across multiple documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchReplaceReport {
    pub results: Vec<ReplaceResult>,
    pub total_replacements: usize,
    pub total_documents_modified: usize,
    pub errors: Vec<(Uuid, String)>,
}

/// Record kept for undo support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplacementRecord {
    pub item_id: Uuid,
    pub original_text: String,
    pub new_text: String,
    pub timestamp: DateTime<Utc>,
}

/// An interactive find-and-replace session that tracks navigation state,
/// current results, and a history of replacements for undo.
#[derive(Debug, Clone)]
pub struct FindReplaceSession {
    pub options: FindReplaceOptions,
    pub results: Vec<FindResult>,
    pub current_result_index: usize,
    pub current_match_index: usize,
    pub history: Vec<ReplacementRecord>,
}

// ---------------------------------------------------------------------------
// Pattern building
// ---------------------------------------------------------------------------

/// Build a compiled `Regex` from the user-supplied options.
pub fn build_regex_pattern(
    query: &str,
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
) -> Result<Regex, String> {
    let pattern = if use_regex {
        query.to_string()
    } else {
        regex::escape(query)
    };

    let pattern = if whole_word {
        format!(r"\b{}\b", pattern)
    } else {
        pattern
    };

    let pattern = if case_sensitive {
        pattern
    } else {
        format!("(?i){}", pattern)
    };

    Regex::new(&pattern).map_err(|e| format!("Invalid regex: {}", e))
}

// ---------------------------------------------------------------------------
// Context helpers
// ---------------------------------------------------------------------------

/// The approximate number of characters of context to capture on each side
/// of a match.
const CONTEXT_CHARS: usize = 40;

/// Extract context around a byte range within `text`.
fn extract_context(text: &str, start: usize, end: usize) -> (String, String) {
    let before_start = if start > CONTEXT_CHARS {
        // Walk back to a safe char boundary.
        let mut idx = start - CONTEXT_CHARS;
        while idx > 0 && !text.is_char_boundary(idx) {
            idx -= 1;
        }
        idx
    } else {
        0
    };

    let after_end = if end + CONTEXT_CHARS < text.len() {
        let mut idx = end + CONTEXT_CHARS;
        while idx < text.len() && !text.is_char_boundary(idx) {
            idx += 1;
        }
        idx
    } else {
        text.len()
    };

    let before = text[before_start..start].to_string();
    let after = text[end..after_end].to_string();
    (before, after)
}

/// Compute (1-indexed line number, 1-indexed column) for a byte offset.
fn line_col_for_offset(text: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for (i, ch) in text.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

// ---------------------------------------------------------------------------
// Core find / replace functions
// ---------------------------------------------------------------------------

/// Find all matches of the query inside a single text body.
pub fn find_in_text(text: &str, options: &FindReplaceOptions) -> Vec<MatchLocation> {
    if options.query.is_empty() {
        return Vec::new();
    }

    let regex = match build_regex_pattern(
        &options.query,
        options.case_sensitive,
        options.whole_word,
        options.use_regex,
    ) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let mut results = Vec::new();
    for m in regex.find_iter(text) {
        let matched_text = m.as_str().to_string();
        let replacement = if options.preserve_case {
            preserve_case_replace(&matched_text, &options.replacement)
        } else if options.use_regex {
            // For regex mode, expand capture groups using the regex itself.
            regex.replace(m.as_str(), options.replacement.as_str()).to_string()
        } else {
            options.replacement.clone()
        };
        let (ctx_before, ctx_after) = extract_context(text, m.start(), m.end());
        let (line, col) = line_col_for_offset(text, m.start());

        results.push(MatchLocation {
            start: m.start(),
            end: m.end(),
            line_number: line,
            column: col,
            context_before: ctx_before,
            matched_text,
            context_after: ctx_after,
            preview_replacement: replacement,
        });
    }
    results
}

/// Search across multiple documents. Each tuple is (id, title, content).
pub fn find_in_documents(
    documents: &[(Uuid, String, String)],
    options: &FindReplaceOptions,
) -> Vec<FindResult> {
    let mut results = Vec::new();
    for (id, title, content) in documents {
        let matches = find_in_text(content, options);
        if !matches.is_empty() {
            results.push(FindResult {
                item_id: *id,
                item_title: title.clone(),
                matches,
            });
        }
    }
    results
}

/// Replace all occurrences in a single text body. Returns the new text and
/// the number of replacements made.
pub fn replace_in_text(text: &str, options: &FindReplaceOptions) -> (String, usize) {
    if options.query.is_empty() {
        return (text.to_string(), 0);
    }

    let regex = match build_regex_pattern(
        &options.query,
        options.case_sensitive,
        options.whole_word,
        options.use_regex,
    ) {
        Ok(r) => r,
        Err(_) => return (text.to_string(), 0),
    };

    if options.preserve_case {
        // We need to apply preserve_case_replace per match, so we walk
        // through matches manually.
        let mut result = String::new();
        let mut last_end = 0usize;
        let mut count = 0usize;
        for m in regex.find_iter(text) {
            result.push_str(&text[last_end..m.start()]);
            result.push_str(&preserve_case_replace(m.as_str(), &options.replacement));
            last_end = m.end();
            count += 1;
        }
        result.push_str(&text[last_end..]);
        (result, count)
    } else {
        let count = regex.find_iter(text).count();
        let new_text = regex.replace_all(text, options.replacement.as_str()).to_string();
        (new_text, count)
    }
}

/// Replace only the next occurrence of the pattern that starts at or after
/// `after_position`. Returns the new full text and the `MatchLocation` that
/// was replaced, or `None` if no match was found.
pub fn replace_next(
    text: &str,
    options: &FindReplaceOptions,
    after_position: usize,
) -> Option<(String, MatchLocation)> {
    if options.query.is_empty() {
        return None;
    }

    let regex = match build_regex_pattern(
        &options.query,
        options.case_sensitive,
        options.whole_word,
        options.use_regex,
    ) {
        Ok(r) => r,
        Err(_) => return None,
    };

    // Find the first match at or after `after_position`.
    let search_start = after_position.min(text.len());
    // Ensure we start at a valid char boundary.
    let mut safe_start = search_start;
    while safe_start < text.len() && !text.is_char_boundary(safe_start) {
        safe_start += 1;
    }

    let m = regex.find(&text[safe_start..])?;
    let abs_start = safe_start + m.start();
    let abs_end = safe_start + m.end();

    let matched_text = m.as_str().to_string();
    let replacement_text = if options.preserve_case {
        preserve_case_replace(&matched_text, &options.replacement)
    } else if options.use_regex {
        regex.replace(m.as_str(), options.replacement.as_str()).to_string()
    } else {
        options.replacement.clone()
    };

    let (ctx_before, ctx_after) = extract_context(text, abs_start, abs_end);
    let (line, col) = line_col_for_offset(text, abs_start);

    let loc = MatchLocation {
        start: abs_start,
        end: abs_end,
        line_number: line,
        column: col,
        context_before: ctx_before,
        matched_text,
        context_after: ctx_after,
        preview_replacement: replacement_text.clone(),
    };

    let mut new_text = String::with_capacity(text.len());
    new_text.push_str(&text[..abs_start]);
    new_text.push_str(&replacement_text);
    new_text.push_str(&text[abs_end..]);

    Some((new_text, loc))
}

/// Batch replace across multiple documents. Mutates the content in place.
pub fn replace_all_in_documents(
    documents: &mut [(Uuid, String, String)],
    options: &FindReplaceOptions,
) -> BatchReplaceReport {
    let mut report = BatchReplaceReport {
        results: Vec::new(),
        total_replacements: 0,
        total_documents_modified: 0,
        errors: Vec::new(),
    };

    for (id, _title, content) in documents.iter_mut() {
        let (new_text, count) = replace_in_text(content, options);
        if count > 0 {
            let original = std::mem::replace(content, new_text);
            report.results.push(ReplaceResult {
                item_id: *id,
                new_text: content.clone(),
                original_text: original,
                replacements_made: count,
            });
            report.total_replacements += count;
            report.total_documents_modified += 1;
        }
    }

    report
}

/// Preview what a single replacement would look like.
pub fn preview_replacement(
    match_loc: &MatchLocation,
    replacement: &str,
    preserve_case: bool,
) -> String {
    if preserve_case {
        preserve_case_replace(&match_loc.matched_text, replacement)
    } else {
        replacement.to_string()
    }
}

/// Smart case-preserving replacement.
///
/// Rules:
/// - If the original is all uppercase, the replacement is uppercased.
/// - If the original starts with an uppercase letter and the rest are
///   lowercase (Title Case), the replacement is title-cased.
/// - Otherwise the replacement is returned as-is (lowercase / mixed stay).
pub fn preserve_case_replace(original: &str, replacement: &str) -> String {
    if original.is_empty() || replacement.is_empty() {
        return replacement.to_string();
    }

    let all_upper = original.chars().all(|c| !c.is_alphabetic() || c.is_uppercase());
    let has_alpha = original.chars().any(|c| c.is_alphabetic());

    if all_upper && has_alpha {
        return replacement.to_uppercase();
    }

    let mut chars = original.chars();
    // Safety: original.is_empty() is checked at the start of the function
    let Some(first) = chars.next() else { return replacement.to_string() };
    let rest_lower = chars.all(|c| !c.is_alphabetic() || c.is_lowercase());
    if first.is_uppercase() && rest_lower {
        // Title Case
        let mut result = String::new();
        for (i, ch) in replacement.chars().enumerate() {
            if i == 0 {
                for uc in ch.to_uppercase() {
                    result.push(uc);
                }
            } else {
                for lc in ch.to_lowercase() {
                    result.push(lc);
                }
            }
        }
        return result;
    }

    // All lowercase or mixed case -- just use replacement as-is.
    replacement.to_string()
}

/// Count the number of matches without building full location data.
pub fn count_matches(text: &str, options: &FindReplaceOptions) -> usize {
    if options.query.is_empty() {
        return 0;
    }
    let regex = match build_regex_pattern(
        &options.query,
        options.case_sensitive,
        options.whole_word,
        options.use_regex,
    ) {
        Ok(r) => r,
        Err(_) => return 0,
    };
    regex.find_iter(text).count()
}

/// Insert highlight markers (tags) around every match in the text.
///
/// For example, `highlight_matches(text, matches, "<b>", "</b>")` wraps
/// each matched span with bold tags.
pub fn highlight_matches(
    text: &str,
    matches: &[MatchLocation],
    before_tag: &str,
    after_tag: &str,
) -> String {
    if matches.is_empty() {
        return text.to_string();
    }

    // Work from back to front so that earlier byte offsets remain valid.
    let mut sorted: Vec<&MatchLocation> = matches.iter().collect();
    sorted.sort_by(|a, b| b.start.cmp(&a.start));

    let mut result = text.to_string();
    for m in sorted {
        if m.end <= result.len() && m.start <= m.end {
            result.insert_str(m.end, after_tag);
            result.insert_str(m.start, before_tag);
        }
    }
    result
}

// ---------------------------------------------------------------------------
// FindReplaceSession
// ---------------------------------------------------------------------------

impl FindReplaceSession {
    /// Create a new session from the given options. Results are initially
    /// empty; the caller should populate them via `find_in_documents` or
    /// similar.
    pub fn new(options: FindReplaceOptions) -> Self {
        Self {
            options,
            results: Vec::new(),
            current_result_index: 0,
            current_match_index: 0,
            history: Vec::new(),
        }
    }

    /// Total number of matches across all result documents.
    fn total_matches(&self) -> usize {
        self.results.iter().map(|r| r.matches.len()).sum()
    }

    /// Flatten index: convert (result_index, match_index) into a single
    /// linear index.
    fn flat_index(&self) -> usize {
        let mut idx = 0;
        for (ri, r) in self.results.iter().enumerate() {
            if ri < self.current_result_index {
                idx += r.matches.len();
            } else {
                idx += self.current_match_index;
                break;
            }
        }
        idx
    }

    /// Navigate to the next match, wrapping around at the end.
    pub fn find_next(&mut self) -> Option<&MatchLocation> {
        if self.results.is_empty() {
            return None;
        }

        // Advance
        let cur_doc = &self.results[self.current_result_index];
        if self.current_match_index + 1 < cur_doc.matches.len() {
            self.current_match_index += 1;
        } else {
            // Move to next document
            self.current_result_index = (self.current_result_index + 1) % self.results.len();
            self.current_match_index = 0;
        }

        self.results
            .get(self.current_result_index)
            .and_then(|r| r.matches.get(self.current_match_index))
    }

    /// Navigate to the previous match, wrapping around at the beginning.
    pub fn find_previous(&mut self) -> Option<&MatchLocation> {
        if self.results.is_empty() {
            return None;
        }

        if self.current_match_index > 0 {
            self.current_match_index -= 1;
        } else {
            // Move to previous document
            if self.current_result_index == 0 {
                self.current_result_index = self.results.len() - 1;
            } else {
                self.current_result_index -= 1;
            }
            let doc = &self.results[self.current_result_index];
            self.current_match_index = doc.matches.len().saturating_sub(1);
        }

        self.results
            .get(self.current_result_index)
            .and_then(|r| r.matches.get(self.current_match_index))
    }

    /// Replace the current match and record the operation for undo.
    /// Returns `None` if there is nothing to replace.
    pub fn replace_current(&mut self) -> Option<ReplacementRecord> {
        if self.results.is_empty() {
            return None;
        }

        let result = self.results.get(self.current_result_index)?;
        let loc = result.matches.get(self.current_match_index)?;

        let record = ReplacementRecord {
            item_id: result.item_id,
            original_text: loc.matched_text.clone(),
            new_text: loc.preview_replacement.clone(),
            timestamp: Utc::now(),
        };

        // Remove the consumed match from the results so the session advances.
        let result_mut = &mut self.results[self.current_result_index];
        result_mut.matches.remove(self.current_match_index);

        // If the document has no more matches, remove it entirely.
        if result_mut.matches.is_empty() {
            self.results.remove(self.current_result_index);
            if !self.results.is_empty() {
                self.current_result_index %= self.results.len();
            } else {
                self.current_result_index = 0;
            }
            self.current_match_index = 0;
        } else if self.current_match_index >= result_mut.matches.len() {
            self.current_match_index = 0;
        }

        self.history.push(record.clone());
        Some(record)
    }

    /// Replace all remaining matches. Returns a `BatchReplaceReport`.
    pub fn replace_all(&mut self) -> BatchReplaceReport {
        let mut report = BatchReplaceReport {
            results: Vec::new(),
            total_replacements: 0,
            total_documents_modified: 0,
            errors: Vec::new(),
        };

        for result in self.results.drain(..) {
            let count = result.matches.len();
            if count > 0 {
                report.total_replacements += count;
                report.total_documents_modified += 1;

                let item_id = result.item_id;
                // Record each replacement for undo, consuming owned strings.
                for loc in result.matches {
                    self.history.push(ReplacementRecord {
                        item_id,
                        original_text: loc.matched_text,
                        new_text: loc.preview_replacement,
                        timestamp: Utc::now(),
                    });
                }

                report.results.push(ReplaceResult {
                    item_id,
                    original_text: String::new(), // Full text not tracked in session
                    new_text: String::new(),
                    replacements_made: count,
                });
            }
        }

        self.current_result_index = 0;
        self.current_match_index = 0;

        report
    }

    /// Return (current_flat_index, total_matches). Both are 0 when empty.
    pub fn current_position(&self) -> (usize, usize) {
        let total = self.total_matches();
        if total == 0 {
            return (0, 0);
        }
        (self.flat_index(), total)
    }

    /// Undo the most recent replacement by popping the last history entry.
    /// Returns the `ReplacementRecord` so the caller can restore the
    /// original text in the document.
    pub fn undo_last(&mut self) -> Option<ReplacementRecord> {
        self.history.pop()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create default options with a query and replacement.
    fn opts(query: &str, replacement: &str) -> FindReplaceOptions {
        FindReplaceOptions {
            query: query.to_string(),
            replacement: replacement.to_string(),
            ..Default::default()
        }
    }

    // ---- basic find tests ----

    #[test]
    fn test_find_basic() {
        let text = "The cat sat on the mat.";
        let o = opts("cat", "dog");
        let matches = find_in_text(text, &o);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_text, "cat");
        assert_eq!(matches[0].start, 4);
        assert_eq!(matches[0].end, 7);
    }

    #[test]
    fn test_find_multiple_matches() {
        let text = "cat dog cat bird cat";
        let o = opts("cat", "x");
        let matches = find_in_text(text, &o);
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_find_empty_query() {
        let text = "hello world";
        let o = opts("", "x");
        let matches = find_in_text(text, &o);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_find_no_match() {
        let text = "hello world";
        let o = opts("xyz", "abc");
        let matches = find_in_text(text, &o);
        assert!(matches.is_empty());
    }

    // ---- case sensitivity tests ----

    #[test]
    fn test_find_case_insensitive() {
        let text = "Hello HELLO hello";
        let o = opts("hello", "x");
        let matches = find_in_text(text, &o);
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_find_case_sensitive() {
        let text = "Hello HELLO hello";
        let mut o = opts("Hello", "x");
        o.case_sensitive = true;
        let matches = find_in_text(text, &o);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_text, "Hello");
    }

    // ---- whole word tests ----

    #[test]
    fn test_find_whole_word() {
        let text = "The cat concatenated strings.";
        let mut o = opts("cat", "dog");
        o.whole_word = true;
        let matches = find_in_text(text, &o);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_text, "cat");
    }

    #[test]
    fn test_find_whole_word_no_match() {
        let text = "concatenation";
        let mut o = opts("cat", "dog");
        o.whole_word = true;
        let matches = find_in_text(text, &o);
        assert!(matches.is_empty());
    }

    // ---- regex mode tests ----

    #[test]
    fn test_find_regex() {
        let text = "abc 123 def 456";
        let mut o = opts(r"\d+", "NUM");
        o.use_regex = true;
        let matches = find_in_text(text, &o);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].matched_text, "123");
        assert_eq!(matches[1].matched_text, "456");
    }

    #[test]
    fn test_find_regex_capture_groups() {
        let text = "John Smith";
        let mut o = opts(r"(\w+)\s(\w+)", "$2, $1");
        o.use_regex = true;
        o.case_sensitive = true;
        let matches = find_in_text(text, &o);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].preview_replacement, "Smith, John");
    }

    #[test]
    fn test_find_invalid_regex() {
        let text = "some text";
        let mut o = opts("[invalid", "x");
        o.use_regex = true;
        let matches = find_in_text(text, &o);
        assert!(matches.is_empty());
    }

    // ---- replace in text tests ----

    #[test]
    fn test_replace_in_text_basic() {
        let text = "The cat sat on the cat mat.";
        let o = opts("cat", "dog");
        let (new_text, count) = replace_in_text(text, &o);
        assert_eq!(new_text, "The dog sat on the dog mat.");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_replace_in_text_no_match() {
        let text = "hello world";
        let o = opts("xyz", "abc");
        let (new_text, count) = replace_in_text(text, &o);
        assert_eq!(new_text, "hello world");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_replace_in_text_empty_query() {
        let text = "hello world";
        let o = opts("", "abc");
        let (new_text, count) = replace_in_text(text, &o);
        assert_eq!(new_text, "hello world");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_replace_in_text_regex() {
        let text = "abc 123 def 456";
        let mut o = opts(r"\d+", "NUM");
        o.use_regex = true;
        let (new_text, count) = replace_in_text(text, &o);
        assert_eq!(new_text, "abc NUM def NUM");
        assert_eq!(count, 2);
    }

    // ---- batch replace tests ----

    #[test]
    fn test_batch_replace() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let mut docs = vec![
            (id1, "Doc1".to_string(), "The cat sat.".to_string()),
            (id2, "Doc2".to_string(), "No match here.".to_string()),
        ];
        let o = opts("cat", "dog");
        let report = replace_all_in_documents(&mut docs, &o);
        assert_eq!(report.total_replacements, 1);
        assert_eq!(report.total_documents_modified, 1);
        assert_eq!(docs[0].2, "The dog sat.");
        assert_eq!(docs[1].2, "No match here."); // unchanged
    }

    #[test]
    fn test_batch_replace_multiple_docs() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let mut docs = vec![
            (id1, "Doc1".to_string(), "cat cat".to_string()),
            (id2, "Doc2".to_string(), "cat".to_string()),
        ];
        let o = opts("cat", "dog");
        let report = replace_all_in_documents(&mut docs, &o);
        assert_eq!(report.total_replacements, 3);
        assert_eq!(report.total_documents_modified, 2);
        assert_eq!(docs[0].2, "dog dog");
        assert_eq!(docs[1].2, "dog");
    }

    // ---- preserve case tests ----

    #[test]
    fn test_preserve_case_all_upper() {
        assert_eq!(preserve_case_replace("HELLO", "world"), "WORLD");
    }

    #[test]
    fn test_preserve_case_title_case() {
        assert_eq!(preserve_case_replace("Hello", "world"), "World");
    }

    #[test]
    fn test_preserve_case_lowercase() {
        assert_eq!(preserve_case_replace("hello", "World"), "World");
    }

    #[test]
    fn test_preserve_case_mixed() {
        // Mixed case: replacement stays as-is.
        assert_eq!(preserve_case_replace("hElLo", "world"), "world");
    }

    #[test]
    fn test_preserve_case_empty_original() {
        assert_eq!(preserve_case_replace("", "world"), "world");
    }

    #[test]
    fn test_preserve_case_empty_replacement() {
        assert_eq!(preserve_case_replace("HELLO", ""), "");
    }

    #[test]
    fn test_replace_with_preserve_case() {
        let text = "Hello HELLO hello";
        let mut o = opts("hello", "world");
        o.preserve_case = true;
        let (new_text, count) = replace_in_text(text, &o);
        assert_eq!(count, 3);
        assert_eq!(new_text, "World WORLD world");
    }

    // ---- session navigation tests ----

    #[test]
    fn test_session_new() {
        let o = opts("cat", "dog");
        let session = FindReplaceSession::new(o);
        assert!(session.results.is_empty());
        assert_eq!(session.current_result_index, 0);
        assert_eq!(session.current_match_index, 0);
        assert!(session.history.is_empty());
    }

    #[test]
    fn test_session_find_next() {
        let o = opts("cat", "dog");
        let mut session = FindReplaceSession::new(o.clone());
        let text = "cat dog cat";
        let matches = find_in_text(text, &o);
        session.results.push(FindResult {
            item_id: Uuid::new_v4(),
            item_title: "Doc".to_string(),
            matches,
        });

        // First find_next goes to match index 1
        let m = session.find_next().unwrap();
        assert_eq!(m.start, 8); // second "cat"

        // Next wraps back to first
        let m = session.find_next().unwrap();
        assert_eq!(m.start, 0);
    }

    #[test]
    fn test_session_find_previous() {
        let o = opts("cat", "dog");
        let mut session = FindReplaceSession::new(o.clone());
        let text = "cat dog cat";
        let matches = find_in_text(text, &o);
        session.results.push(FindResult {
            item_id: Uuid::new_v4(),
            item_title: "Doc".to_string(),
            matches,
        });

        // find_previous from index 0 wraps to the last match.
        let m = session.find_previous().unwrap();
        assert_eq!(m.start, 8); // last "cat"
    }

    #[test]
    fn test_session_find_next_empty() {
        let o = opts("cat", "dog");
        let mut session = FindReplaceSession::new(o);
        assert!(session.find_next().is_none());
    }

    // ---- session replace / undo tests ----

    #[test]
    fn test_session_replace_current() {
        let o = opts("cat", "dog");
        let mut session = FindReplaceSession::new(o.clone());
        let text = "cat dog cat";
        let matches = find_in_text(text, &o);
        session.results.push(FindResult {
            item_id: Uuid::new_v4(),
            item_title: "Doc".to_string(),
            matches,
        });

        let record = session.replace_current().unwrap();
        assert_eq!(record.original_text, "cat");
        assert_eq!(record.new_text, "dog");
        assert_eq!(session.history.len(), 1);
        // One match was removed; one remains.
        assert_eq!(session.results[0].matches.len(), 1);
    }

    #[test]
    fn test_session_replace_current_empty() {
        let o = opts("cat", "dog");
        let mut session = FindReplaceSession::new(o);
        assert!(session.replace_current().is_none());
    }

    #[test]
    fn test_session_undo_last() {
        let o = opts("cat", "dog");
        let mut session = FindReplaceSession::new(o.clone());
        let text = "cat";
        let matches = find_in_text(text, &o);
        session.results.push(FindResult {
            item_id: Uuid::new_v4(),
            item_title: "Doc".to_string(),
            matches,
        });

        session.replace_current();
        let undone = session.undo_last().unwrap();
        assert_eq!(undone.original_text, "cat");
        assert_eq!(undone.new_text, "dog");
        // History should now be empty.
        assert!(session.undo_last().is_none());
    }

    #[test]
    fn test_session_replace_all() {
        let o = opts("cat", "dog");
        let mut session = FindReplaceSession::new(o.clone());

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        session.results.push(FindResult {
            item_id: id1,
            item_title: "Doc1".to_string(),
            matches: find_in_text("cat cat", &o),
        });
        session.results.push(FindResult {
            item_id: id2,
            item_title: "Doc2".to_string(),
            matches: find_in_text("a cat", &o),
        });

        let report = session.replace_all();
        assert_eq!(report.total_replacements, 3);
        assert_eq!(report.total_documents_modified, 2);
        assert!(session.results.is_empty());
        assert_eq!(session.history.len(), 3);
    }

    // ---- current_position tests ----

    #[test]
    fn test_session_current_position_empty() {
        let session = FindReplaceSession::new(opts("x", "y"));
        assert_eq!(session.current_position(), (0, 0));
    }

    #[test]
    fn test_session_current_position() {
        let o = opts("cat", "dog");
        let mut session = FindReplaceSession::new(o.clone());
        session.results.push(FindResult {
            item_id: Uuid::new_v4(),
            item_title: "Doc".to_string(),
            matches: find_in_text("cat dog cat", &o),
        });
        assert_eq!(session.current_position(), (0, 2));
        session.find_next();
        assert_eq!(session.current_position(), (1, 2));
    }

    // ---- context extraction tests ----

    #[test]
    fn test_context_extraction() {
        let text = "The quick brown fox jumps over the lazy dog";
        let o = opts("fox", "cat");
        let matches = find_in_text(text, &o);
        assert_eq!(matches.len(), 1);
        assert!(matches[0].context_before.contains("brown"));
        assert!(matches[0].context_after.contains("jumps"));
    }

    #[test]
    fn test_context_at_start() {
        let text = "cat sat on the mat";
        let o = opts("cat", "dog");
        let matches = find_in_text(text, &o);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].context_before, "");
        assert!(!matches[0].context_after.is_empty());
    }

    #[test]
    fn test_context_at_end() {
        let text = "the mat is a cat";
        let o = opts("cat", "dog");
        let matches = find_in_text(text, &o);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].context_after, "");
        assert!(!matches[0].context_before.is_empty());
    }

    // ---- line/column tests ----

    #[test]
    fn test_line_column_first_line() {
        let text = "Hello cat world";
        let o = opts("cat", "dog");
        let matches = find_in_text(text, &o);
        assert_eq!(matches[0].line_number, 1);
        assert_eq!(matches[0].column, 7); // "Hello " = 6 chars, col 7
    }

    #[test]
    fn test_line_column_second_line() {
        let text = "first line\nsecond cat line";
        let o = opts("cat", "dog");
        let matches = find_in_text(text, &o);
        assert_eq!(matches[0].line_number, 2);
        assert_eq!(matches[0].column, 8); // "second " = 7 chars, col 8
    }

    // ---- highlight tests ----

    #[test]
    fn test_highlight_matches_basic() {
        let text = "The cat sat.";
        let o = opts("cat", "dog");
        let matches = find_in_text(text, &o);
        let highlighted = highlight_matches(text, &matches, "<b>", "</b>");
        assert_eq!(highlighted, "The <b>cat</b> sat.");
    }

    #[test]
    fn test_highlight_matches_multiple() {
        let text = "cat and cat";
        let o = opts("cat", "x");
        let matches = find_in_text(text, &o);
        let highlighted = highlight_matches(text, &matches, "[", "]");
        assert_eq!(highlighted, "[cat] and [cat]");
    }

    #[test]
    fn test_highlight_matches_empty() {
        let text = "no matches here";
        let highlighted = highlight_matches(text, &[], "<b>", "</b>");
        assert_eq!(highlighted, "no matches here");
    }

    // ---- replace_next tests ----

    #[test]
    fn test_replace_next_basic() {
        let text = "cat dog cat bird cat";
        let o = opts("cat", "fish");
        let result = replace_next(text, &o, 0);
        assert!(result.is_some());
        let (new_text, loc) = result.unwrap();
        assert_eq!(new_text, "fish dog cat bird cat");
        assert_eq!(loc.start, 0);
    }

    #[test]
    fn test_replace_next_after_position() {
        let text = "cat dog cat bird cat";
        let o = opts("cat", "fish");
        let result = replace_next(text, &o, 4).unwrap();
        assert_eq!(result.0, "cat dog fish bird cat");
        assert_eq!(result.1.start, 8);
    }

    #[test]
    fn test_replace_next_no_match() {
        let text = "hello world";
        let o = opts("cat", "dog");
        assert!(replace_next(text, &o, 0).is_none());
    }

    // ---- count_matches tests ----

    #[test]
    fn test_count_matches_basic() {
        let text = "cat and cat and cat";
        let o = opts("cat", "");
        assert_eq!(count_matches(text, &o), 3);
    }

    #[test]
    fn test_count_matches_zero() {
        let text = "nothing here";
        let o = opts("xyz", "");
        assert_eq!(count_matches(text, &o), 0);
    }

    #[test]
    fn test_count_matches_empty_query() {
        let text = "something";
        let o = opts("", "");
        assert_eq!(count_matches(text, &o), 0);
    }

    // ---- build_regex_pattern tests ----

    #[test]
    fn test_build_regex_pattern_valid() {
        let r = build_regex_pattern("hello", false, false, false);
        assert!(r.is_ok());
    }

    #[test]
    fn test_build_regex_pattern_invalid() {
        let r = build_regex_pattern("[invalid", false, false, true);
        assert!(r.is_err());
    }

    #[test]
    fn test_build_regex_pattern_special_chars_escaped() {
        // Without regex mode, special chars like '.' should be escaped.
        let r = build_regex_pattern("a.b", false, false, false).unwrap();
        assert!(!r.is_match("axb"));
        assert!(r.is_match("a.b"));
    }

    // ---- preview_replacement tests ----

    #[test]
    fn test_preview_replacement_plain() {
        let loc = MatchLocation {
            start: 0,
            end: 3,
            line_number: 1,
            column: 1,
            context_before: String::new(),
            matched_text: "cat".to_string(),
            context_after: String::new(),
            preview_replacement: "dog".to_string(),
        };
        let preview = preview_replacement(&loc, "dog", false);
        assert_eq!(preview, "dog");
    }

    #[test]
    fn test_preview_replacement_preserve_case() {
        let loc = MatchLocation {
            start: 0,
            end: 3,
            line_number: 1,
            column: 1,
            context_before: String::new(),
            matched_text: "CAT".to_string(),
            context_after: String::new(),
            preview_replacement: String::new(),
        };
        let preview = preview_replacement(&loc, "dog", true);
        assert_eq!(preview, "DOG");
    }

    // ---- find_in_documents test ----

    #[test]
    fn test_find_in_documents() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let docs = vec![
            (id1, "Doc1".to_string(), "The cat sat.".to_string()),
            (id2, "Doc2".to_string(), "No match here.".to_string()),
        ];
        let o = opts("cat", "dog");
        let results = find_in_documents(&docs, &o);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item_id, id1);
        assert_eq!(results[0].matches.len(), 1);
    }

    // ---- edge case: empty text ----

    #[test]
    fn test_find_in_empty_text() {
        let matches = find_in_text("", &opts("cat", "dog"));
        assert!(matches.is_empty());
    }

    #[test]
    fn test_replace_in_empty_text() {
        let (text, count) = replace_in_text("", &opts("cat", "dog"));
        assert_eq!(text, "");
        assert_eq!(count, 0);
    }

    // ---- edge case: overlapping / adjacent matches ----

    #[test]
    fn test_find_adjacent_matches() {
        let text = "aaa";
        let mut o = opts("a", "b");
        o.case_sensitive = true;
        let matches = find_in_text(text, &o);
        assert_eq!(matches.len(), 3);
    }

    // ---- session navigation across multiple documents ----

    #[test]
    fn test_session_navigate_across_documents() {
        let o = opts("cat", "dog");
        let mut session = FindReplaceSession::new(o.clone());

        session.results.push(FindResult {
            item_id: Uuid::new_v4(),
            item_title: "Doc1".to_string(),
            matches: find_in_text("cat", &o),
        });
        session.results.push(FindResult {
            item_id: Uuid::new_v4(),
            item_title: "Doc2".to_string(),
            matches: find_in_text("a cat b", &o),
        });

        // We start at doc 0, match 0. find_next should advance to doc 1.
        let matched = session.find_next().unwrap().matched_text.clone();
        assert_eq!(session.current_result_index, 1);
        assert_eq!(matched, "cat");

        // find_next again wraps back to doc 0.
        let matched = session.find_next().unwrap().matched_text.clone();
        assert_eq!(session.current_result_index, 0);
        assert_eq!(matched, "cat");
    }

    // ---- session: replace all then undo ----

    #[test]
    fn test_session_undo_after_replace_all() {
        let o = opts("cat", "dog");
        let mut session = FindReplaceSession::new(o.clone());
        session.results.push(FindResult {
            item_id: Uuid::new_v4(),
            item_title: "Doc".to_string(),
            matches: find_in_text("cat cat", &o),
        });

        session.replace_all();
        assert_eq!(session.history.len(), 2);

        let r1 = session.undo_last().unwrap();
        assert_eq!(r1.original_text, "cat");
        let r2 = session.undo_last().unwrap();
        assert_eq!(r2.original_text, "cat");
        assert!(session.undo_last().is_none());
    }

    // ---- session: replace_current removes document when empty ----

    #[test]
    fn test_session_replace_current_removes_empty_doc() {
        let o = opts("cat", "dog");
        let mut session = FindReplaceSession::new(o.clone());
        session.results.push(FindResult {
            item_id: Uuid::new_v4(),
            item_title: "Doc".to_string(),
            matches: find_in_text("cat", &o), // single match
        });

        session.replace_current();
        assert!(session.results.is_empty());
    }
}
