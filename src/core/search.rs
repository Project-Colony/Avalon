use std::collections::HashSet;
use uuid::Uuid;
use regex::Regex;

use super::binder::Binder;

/// Search result from a project-wide search
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub item_id: Uuid,
    pub item_title: String,
    pub matches: Vec<SearchMatch>,
}

/// A single match within a document
#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub line_number: usize,
    pub start: usize,
    pub end: usize,
    pub context: String,
}

/// Options for searching
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub query: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub regex: bool,
    pub search_titles: bool,
    pub search_content: bool,
    pub search_notes: bool,
    pub search_synopsis: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            query: String::new(),
            case_sensitive: false,
            whole_word: false,
            regex: false,
            search_titles: true,
            search_content: true,
            search_notes: false,
            search_synopsis: false,
        }
    }
}

/// Search across the entire binder
pub fn search_binder(binder: &Binder, options: &SearchOptions) -> Vec<SearchResult> {
    let mut results = Vec::new();

    if options.query.is_empty() {
        return results;
    }

    let pattern = build_pattern(options);
    let regex = match Regex::new(&pattern) {
        Ok(r) => r,
        Err(_) => return results,
    };

    for item in binder.all_items() {
        let mut matches = Vec::new();

        // Search title
        if options.search_titles {
            for m in regex.find_iter(&item.title) {
                matches.push(SearchMatch {
                    line_number: 0,
                    start: m.start(),
                    end: m.end(),
                    context: format!("[Title] {}", item.title),
                });
            }
        }

        // Search content
        if options.search_content {
            if let Some(ref doc) = item.document {
                for (line_num, line) in doc.content.lines().enumerate() {
                    for m in regex.find_iter(line) {
                        matches.push(SearchMatch {
                            line_number: line_num + 1,
                            start: m.start(),
                            end: m.end(),
                            context: line.to_string(),
                        });
                    }
                }
            }
        }

        // Search synopsis
        if options.search_synopsis && !item.synopsis.is_empty() {
            for m in regex.find_iter(&item.synopsis) {
                matches.push(SearchMatch {
                    line_number: 0,
                    start: m.start(),
                    end: m.end(),
                    context: format!("[Synopsis] {}", item.synopsis),
                });
            }
        }

        // Search notes
        if options.search_notes {
            if let Some(ref doc) = item.document {
                for m in regex.find_iter(&doc.notes) {
                    matches.push(SearchMatch {
                        line_number: 0,
                        start: m.start(),
                        end: m.end(),
                        context: format!("[Notes] {}", doc.notes),
                    });
                }
            }
        }

        if !matches.is_empty() {
            results.push(SearchResult {
                item_id: item.id,
                item_title: item.title.clone(),
                matches,
            });
        }
    }

    results
}

/// Build a regex pattern from search options
fn build_pattern(options: &SearchOptions) -> String {
    let query = if options.regex {
        options.query.clone()
    } else {
        regex::escape(&options.query)
    };

    let query = if options.whole_word {
        format!(r"\b{}\b", query)
    } else {
        query
    };

    if options.case_sensitive {
        query
    } else {
        format!("(?i){}", query)
    }
}

/// Replace text in a document
pub fn replace_in_document(content: &str, options: &SearchOptions, replacement: &str) -> String {
    let pattern = build_pattern(options);
    if let Ok(regex) = Regex::new(&pattern) {
        regex.replace_all(content, replacement).to_string()
    } else {
        content.to_string()
    }
}

/// Replace the first occurrence of the search pattern
pub fn replace_first(content: &str, options: &SearchOptions, replacement: &str) -> String {
    let pattern = build_pattern(options);
    if let Ok(regex) = Regex::new(&pattern) {
        regex.replace(content, replacement).to_string()
    } else {
        content.to_string()
    }
}

/// Count total matches across all results
pub fn total_match_count(results: &[SearchResult]) -> usize {
    results.iter().map(|r| r.matches.len()).sum()
}

/// Count documents with matches
pub fn document_count(results: &[SearchResult]) -> usize {
    results.len()
}

/// Search summary as a human-readable string
pub fn search_summary(results: &[SearchResult]) -> String {
    let total = total_match_count(results);
    let docs = document_count(results);
    if total == 0 {
        "No matches found".to_string()
    } else {
        format!("{} match(es) in {} document(s)", total, docs)
    }
}

impl SearchResult {
    /// Total number of matches in this result
    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// Get context lines around a match (with surrounding text)
    pub fn context_preview(&self, match_index: usize, max_len: usize) -> String {
        if let Some(m) = self.matches.get(match_index) {
            if m.context.len() <= max_len {
                m.context.clone()
            } else {
                format!("{}...", &m.context[..max_len])
            }
        } else {
            String::new()
        }
    }
}

impl SearchMatch {
    /// Length of the matched text
    pub fn match_length(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Highlighted context with match wrapped in brackets
    pub fn highlighted_context(&self) -> String {
        if self.start < self.context.len() && self.end <= self.context.len() {
            format!(
                "{}[{}]{}",
                &self.context[..self.start],
                &self.context[self.start..self.end],
                &self.context[self.end..]
            )
        } else {
            self.context.clone()
        }
    }
}

impl SearchOptions {
    /// Create options for a simple text search
    pub fn simple(query: &str) -> Self {
        Self {
            query: query.to_string(),
            case_sensitive: false,
            whole_word: false,
            regex: false,
            search_titles: true,
            search_content: true,
            search_notes: false,
            search_synopsis: false,
        }
    }

    /// Create options for case-sensitive search
    pub fn case_sensitive(query: &str) -> Self {
        Self {
            query: query.to_string(),
            case_sensitive: true,
            whole_word: false,
            regex: false,
            search_titles: true,
            search_content: true,
            search_notes: false,
            search_synopsis: false,
        }
    }

    /// Create options for whole-word search
    pub fn whole_word(query: &str) -> Self {
        Self {
            query: query.to_string(),
            case_sensitive: false,
            whole_word: true,
            regex: false,
            search_titles: true,
            search_content: true,
            search_notes: false,
            search_synopsis: false,
        }
    }

    /// Create options for regex search
    pub fn regex_search(query: &str) -> Self {
        Self {
            query: query.to_string(),
            case_sensitive: false,
            whole_word: false,
            regex: true,
            search_titles: true,
            search_content: true,
            search_notes: false,
            search_synopsis: false,
        }
    }

    /// Create options that search everywhere
    pub fn everywhere(query: &str) -> Self {
        Self {
            query: query.to_string(),
            case_sensitive: false,
            whole_word: false,
            regex: false,
            search_titles: true,
            search_content: true,
            search_notes: true,
            search_synopsis: true,
        }
    }

    /// Describe the search scope as a string
    pub fn scope_description(&self) -> String {
        let mut scopes = Vec::new();
        if self.search_titles {
            scopes.push("titles");
        }
        if self.search_content {
            scopes.push("content");
        }
        if self.search_notes {
            scopes.push("notes");
        }
        if self.search_synopsis {
            scopes.push("synopsis");
        }
        scopes.join(", ")
    }

    /// Create options for content-only search
    pub fn content_only(query: &str) -> Self {
        Self {
            query: query.to_string(),
            case_sensitive: false,
            whole_word: false,
            regex: false,
            search_titles: false,
            search_content: true,
            search_notes: false,
            search_synopsis: false,
        }
    }

    /// Check if this search has any scope enabled
    pub fn has_scope(&self) -> bool {
        self.search_titles || self.search_content || self.search_notes || self.search_synopsis
    }

    /// Summary of search configuration
    pub fn summary(&self) -> String {
        let mut flags = Vec::new();
        if self.case_sensitive { flags.push("case-sensitive"); }
        if self.whole_word { flags.push("whole-word"); }
        if self.regex { flags.push("regex"); }
        format!("\"{}\" in {} {}", self.query, self.scope_description(),
            if flags.is_empty() { String::new() } else { format!("({})", flags.join(", ")) })
    }

    /// Count how many scopes are enabled
    pub fn scope_count(&self) -> usize {
        let mut count = 0;
        if self.search_titles { count += 1; }
        if self.search_content { count += 1; }
        if self.search_notes { count += 1; }
        if self.search_synopsis { count += 1; }
        count
    }

    /// Check if this is a regex search
    pub fn is_regex(&self) -> bool {
        self.regex
    }

    /// Validate that the regex pattern is valid (returns error message if not)
    pub fn validate_regex(&self) -> Option<String> {
        if !self.regex {
            return None;
        }
        match Regex::new(&build_pattern(self)) {
            Ok(_) => None,
            Err(e) => Some(format!("Invalid regex: {}", e)),
        }
    }
}

/// Get line numbers of all matches in content
pub fn match_line_numbers(content: &str, options: &SearchOptions) -> Vec<usize> {
    let pattern = build_pattern(options);
    let regex = match Regex::new(&pattern) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    content.lines().enumerate()
        .filter(|(_, line)| regex.is_match(line))
        .map(|(i, _)| i + 1)
        .collect()
}

/// Get the match count in a single string
pub fn count_matches(text: &str, options: &SearchOptions) -> usize {
    let pattern = build_pattern(options);
    let regex = match Regex::new(&pattern) {
        Ok(r) => r,
        Err(_) => return 0,
    };
    regex.find_iter(text).count()
}

/// Extract all unique matched strings from content
pub fn extract_matches(content: &str, options: &SearchOptions) -> Vec<String> {
    let pattern = build_pattern(options);
    let regex = match Regex::new(&pattern) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let mut seen = HashSet::new();
    let mut results = Vec::new();
    for m in regex.find_iter(content) {
        let s = m.as_str();
        if seen.insert(s.to_string()) {
            results.push(s.to_string());
        }
    }
    results
}

/// Get context around a match (N lines before and after)
pub fn match_with_context(content: &str, options: &SearchOptions, context_lines: usize) -> Vec<MatchContext> {
    let pattern = build_pattern(options);
    let regex = match Regex::new(&pattern) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut results = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if regex.is_match(line) {
            let start = i.saturating_sub(context_lines);
            let end = (i + context_lines + 1).min(lines.len());
            let before: Vec<String> = lines[start..i].iter().map(|s| s.to_string()).collect();
            let after: Vec<String> = lines[i + 1..end].iter().map(|s| s.to_string()).collect();
            results.push(MatchContext {
                line_number: i + 1,
                matched_line: line.to_string(),
                before,
                after,
            });
        }
    }

    results
}

/// Search for items by word count range.
pub fn search_by_word_count(binder: &Binder, min: usize, max: usize) -> Vec<(Uuid, String, usize)> {
    binder.all_items().into_iter()
        .filter(|item| item.kind == super::binder::BinderItemKind::Text)
        .filter_map(|item| {
            item.document.as_ref().map(|doc| {
                let wc = doc.word_count();
                (item.id, item.title.clone(), wc)
            })
        })
        .filter(|(_, _, wc)| *wc >= min && *wc <= max)
        .collect()
}

/// Search for items by keyword in metadata.
pub fn search_by_keyword(binder: &Binder, keyword: &str) -> Vec<(Uuid, String)> {
    binder.all_items().into_iter()
        .filter(|item| item.metadata.has_keyword(keyword))
        .map(|item| (item.id, item.title.clone()))
        .collect()
}

/// Search for items with a specific label.
pub fn search_by_label(binder: &Binder, label: &str) -> Vec<(Uuid, String)> {
    binder.all_items().into_iter()
        .filter(|item| item.metadata.label.as_ref().map(|l| l.name.as_str()) == Some(label))
        .map(|item| (item.id, item.title.clone()))
        .collect()
}

/// Search for items with a specific status.
pub fn search_by_status(binder: &Binder, status: &str) -> Vec<(Uuid, String)> {
    binder.all_items().into_iter()
        .filter(|item| item.metadata.status.as_ref().map(|s| s.name.as_str()) == Some(status))
        .map(|item| (item.id, item.title.clone()))
        .collect()
}

/// Search for items modified after a given date.
pub fn search_modified_after(binder: &Binder, date: chrono::DateTime<chrono::Utc>) -> Vec<(Uuid, String)> {
    binder.all_items().into_iter()
        .filter(|item| item.modified_at > date)
        .map(|item| (item.id, item.title.clone()))
        .collect()
}

/// Search for empty documents (text items with no content).
pub fn search_empty_documents(binder: &Binder) -> Vec<(Uuid, String)> {
    binder.all_items().into_iter()
        .filter(|item| item.kind == super::binder::BinderItemKind::Text)
        .filter(|item| {
            item.document.as_ref().is_none_or(|doc| doc.content.trim().is_empty())
        })
        .map(|item| (item.id, item.title.clone()))
        .collect()
}

/// A match with surrounding context lines
#[derive(Debug, Clone)]
pub struct MatchContext {
    pub line_number: usize,
    pub matched_line: String,
    pub before: Vec<String>,
    pub after: Vec<String>,
}

impl MatchContext {
    /// Format as a displayable block
    pub fn display(&self) -> String {
        let mut output = String::new();
        for (i, line) in self.before.iter().enumerate() {
            let num = self.line_number - self.before.len() + i;
            output.push_str(&format!("  {:>4} | {}\n", num, line));
        }
        output.push_str(&format!("> {:>4} | {}\n", self.line_number, self.matched_line));
        for (i, line) in self.after.iter().enumerate() {
            output.push_str(&format!("  {:>4} | {}\n", self.line_number + 1 + i, line));
        }
        output
    }

    /// Total lines in context (before + match + after)
    pub fn total_lines(&self) -> usize {
        self.before.len() + 1 + self.after.len()
    }
}

/// Compute the Levenshtein (edit) distance between two strings
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let n = a_chars.len();
    let m = b_chars.len();

    if n == 0 { return m; }
    if m == 0 { return n; }

    let mut prev = vec![0usize; m + 1];
    let mut curr = vec![0usize; m + 1];

    for (j, val) in prev.iter_mut().enumerate().take(m + 1) {
        *val = j;
    }

    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[m]
}

/// A fuzzy search result with similarity score
#[derive(Debug, Clone)]
pub struct FuzzyMatch {
    pub item_id: Uuid,
    pub item_title: String,
    pub matched_field: String,
    pub matched_text: String,
    pub distance: usize,
    pub score: f64, // 0.0 to 1.0, higher = better match
}

/// Perform a fuzzy search across the binder. Returns items whose titles
/// or content words are within `max_distance` edits of the query.
pub fn fuzzy_search(binder: &Binder, query: &str, max_distance: usize) -> Vec<FuzzyMatch> {
    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    for item in binder.all_items() {
        // Check title
        let title_lower = item.title.to_lowercase();
        let dist = levenshtein_distance(&query_lower, &title_lower);
        let max_len = query_lower.len().max(title_lower.len());
        if dist <= max_distance && max_len > 0 {
            matches.push(FuzzyMatch {
                item_id: item.id,
                item_title: item.title.clone(),
                matched_field: "title".to_string(),
                matched_text: item.title.clone(),
                distance: dist,
                score: 1.0 - (dist as f64 / max_len as f64),
            });
        }

        // Check content words
        if let Some(ref doc) = item.document {
            for word in doc.content.split_whitespace() {
                let word_lower = word.to_lowercase();
                // Only check words of similar length to avoid pointless comparisons
                if word_lower.len().abs_diff(query_lower.len()) > max_distance {
                    continue;
                }
                let dist = levenshtein_distance(&query_lower, &word_lower);
                if dist <= max_distance && dist > 0 {
                    // Only add if exact match wasn't already found
                    let max_len = query_lower.len().max(word_lower.len());
                    matches.push(FuzzyMatch {
                        item_id: item.id,
                        item_title: item.title.clone(),
                        matched_field: "content".to_string(),
                        matched_text: word.to_string(),
                        distance: dist,
                        score: 1.0 - (dist as f64 / max_len as f64),
                    });
                    break; // Only one match per item per word
                }
            }
        }
    }

    matches.sort_by(|a, b| a.distance.cmp(&b.distance)
        .then_with(|| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)));
    matches
}

/// A saved search (bookmark)
#[derive(Debug, Clone)]
pub struct SavedSearch {
    pub id: Uuid,
    pub name: String,
    pub options: SearchOptions,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl SavedSearch {
    pub fn new(name: &str, options: SearchOptions) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            options,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.name, self.options.summary())
    }
}

/// Manages saved searches and search history
#[derive(Debug, Default)]
pub struct SearchManager {
    pub saved_searches: Vec<SavedSearch>,
    pub history: Vec<String>,
    pub max_history: usize,
}

impl SearchManager {
    pub fn new() -> Self {
        Self {
            saved_searches: Vec::new(),
            history: Vec::new(),
            max_history: 50,
        }
    }

    /// Save a search
    pub fn save_search(&mut self, name: &str, options: SearchOptions) -> Uuid {
        let search = SavedSearch::new(name, options);
        let id = search.id;
        self.saved_searches.push(search);
        id
    }

    /// Remove a saved search
    pub fn remove_saved(&mut self, id: &Uuid) -> bool {
        let len_before = self.saved_searches.len();
        self.saved_searches.retain(|s| &s.id != id);
        self.saved_searches.len() < len_before
    }

    /// Get a saved search by ID
    pub fn get_saved(&self, id: &Uuid) -> Option<&SavedSearch> {
        self.saved_searches.iter().find(|s| &s.id == id)
    }

    /// Record a query in history (most recent first, deduplicates)
    pub fn record_query(&mut self, query: &str) {
        self.history.retain(|q| q != query);
        self.history.insert(0, query.to_string());
        if self.history.len() > self.max_history {
            self.history.truncate(self.max_history);
        }
    }

    /// Get history entries matching a prefix (for autocomplete)
    pub fn suggest(&self, prefix: &str) -> Vec<&str> {
        let lower = prefix.to_lowercase();
        self.history.iter()
            .filter(|q| q.to_lowercase().starts_with(&lower))
            .map(|q| q.as_str())
            .collect()
    }

    /// Clear search history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Saved search count
    pub fn saved_count(&self) -> usize {
        self.saved_searches.len()
    }

    /// Search history count
    pub fn history_count(&self) -> usize {
        self.history.len()
    }

    /// Get all saved search names
    pub fn saved_names(&self) -> Vec<&str> {
        self.saved_searches.iter().map(|s| s.name.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::binder::{Binder, BinderItem};

    fn make_binder_with_content(items: Vec<(&str, &str)>) -> Binder {
        let mut binder = Binder::default_structure();
        for (title, content) in items {
            let mut item = BinderItem::new_text(title);
            if let Some(ref mut doc) = item.document {
                doc.content = content.to_string();
            }
            binder.draft.add_child(item);
        }
        binder
    }

    #[test]
    fn test_search_empty_query() {
        let binder = make_binder_with_content(vec![("Doc1", "Hello world")]);
        let options = SearchOptions { query: String::new(), ..Default::default() };
        let results = search_binder(&binder, &options);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_basic() {
        let binder = make_binder_with_content(vec![
            ("Chapter 1", "The cat sat on the mat."),
            ("Chapter 2", "The dog ran in the park."),
        ]);
        let options = SearchOptions {
            query: "the".to_string(),
            case_sensitive: false,
            ..Default::default()
        };
        let results = search_binder(&binder, &options);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_case_sensitive() {
        let binder = make_binder_with_content(vec![
            ("Doc", "Hello hello HELLO"),
        ]);
        let options = SearchOptions {
            query: "Hello".to_string(),
            case_sensitive: true,
            search_content: true,
            search_titles: false,
            ..Default::default()
        };
        let results = search_binder(&binder, &options);
        assert!(!results.is_empty());
        // Should find exactly 1 match for case-sensitive "Hello"
        let content_matches: Vec<_> = results[0].matches.iter()
            .filter(|m| !m.context.starts_with("[Title]"))
            .collect();
        assert_eq!(content_matches.len(), 1);
    }

    #[test]
    fn test_search_in_title() {
        let binder = make_binder_with_content(vec![
            ("Important Chapter", "Some content here"),
        ]);
        let options = SearchOptions {
            query: "important".to_string(),
            search_titles: true,
            search_content: false,
            ..Default::default()
        };
        let results = search_binder(&binder, &options);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_no_results() {
        let binder = make_binder_with_content(vec![
            ("Doc", "Hello world"),
        ]);
        let options = SearchOptions {
            query: "zzzzz".to_string(),
            ..Default::default()
        };
        let results = search_binder(&binder, &options);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_result_context() {
        let binder = make_binder_with_content(vec![
            ("Doc", "The quick brown fox jumps over the lazy dog"),
        ]);
        let options = SearchOptions {
            query: "fox".to_string(),
            search_content: true,
            search_titles: false,
            ..Default::default()
        };
        let results = search_binder(&binder, &options);
        assert_eq!(results.len(), 1);
        assert!(results[0].matches[0].context.contains("fox"));
    }

    #[test]
    fn test_replace_in_document() {
        let options = SearchOptions {
            query: "cat".to_string(),
            case_sensitive: false,
            ..Default::default()
        };
        let result = replace_in_document("The cat sat. The cat ran.", &options, "dog");
        assert_eq!(result, "The dog sat. The dog ran.");
    }

    #[test]
    fn test_replace_case_sensitive() {
        let options = SearchOptions {
            query: "Cat".to_string(),
            case_sensitive: true,
            ..Default::default()
        };
        let result = replace_in_document("Cat cat CAT", &options, "Dog");
        assert_eq!(result, "Dog cat CAT");
    }

    #[test]
    fn test_search_options_defaults() {
        let opts = SearchOptions::default();
        assert!(opts.query.is_empty());
        assert!(!opts.case_sensitive);
        assert!(!opts.whole_word);
        assert!(opts.search_titles);
        assert!(opts.search_content);
        assert!(opts.has_scope());
    }

    #[test]
    fn test_content_only_options() {
        let opts = SearchOptions::content_only("test");
        assert_eq!(opts.query, "test");
        assert!(!opts.search_titles);
        assert!(opts.search_content);
    }

    #[test]
    fn test_search_match_length() {
        let m = SearchMatch {
            line_number: 1,
            start: 10,
            end: 15,
            context: "Some context".to_string(),
        };
        assert_eq!(m.match_length(), 5);
    }

    #[test]
    fn test_search_options_summary() {
        let opts = SearchOptions {
            query: "hello".to_string(),
            case_sensitive: true,
            ..Default::default()
        };
        let summary = opts.summary();
        assert!(summary.contains("hello"));
        assert!(summary.contains("case-sensitive"));
    }

    #[test]
    fn test_search_whole_word() {
        let binder = make_binder_with_content(vec![
            ("Doc", "The cat concatenated strings"),
        ]);
        let options = SearchOptions {
            query: "cat".to_string(),
            whole_word: true,
            search_content: true,
            search_titles: false,
            ..Default::default()
        };
        let results = search_binder(&binder, &options);
        assert_eq!(results.len(), 1);
        // Should match "cat" but not "concatenated"
        assert_eq!(results[0].matches.len(), 1);
    }

    #[test]
    fn test_search_regex() {
        let binder = make_binder_with_content(vec![
            ("Doc", "apple 123 banana 456 cherry"),
        ]);
        let options = SearchOptions {
            query: r"\d+".to_string(),
            regex: true,
            search_content: true,
            search_titles: false,
            ..Default::default()
        };
        let results = search_binder(&binder, &options);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].matches.len(), 2); // "123" and "456"
    }

    #[test]
    fn test_search_in_synopsis() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Chapter");
        item.synopsis = "The protagonist arrives".to_string();
        binder.draft.add_child(item);

        let options = SearchOptions {
            query: "protagonist".to_string(),
            search_content: false,
            search_titles: false,
            search_synopsis: true,
            ..Default::default()
        };
        let results = search_binder(&binder, &options);
        assert_eq!(results.len(), 1);
        assert!(results[0].matches[0].context.contains("[Synopsis]"));
    }

    #[test]
    fn test_search_in_notes() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Chapter");
        if let Some(ref mut doc) = item.document {
            doc.notes = "Research: ancient Egypt".to_string();
        }
        binder.draft.add_child(item);

        let options = SearchOptions {
            query: "egypt".to_string(),
            search_content: false,
            search_titles: false,
            search_notes: true,
            ..Default::default()
        };
        let results = search_binder(&binder, &options);
        assert_eq!(results.len(), 1);
        assert!(results[0].matches[0].context.contains("[Notes]"));
    }

    #[test]
    fn test_replace_first() {
        let options = SearchOptions::simple("cat");
        let result = replace_first("The cat and the cat", &options, "dog");
        assert_eq!(result, "The dog and the cat");
    }

    #[test]
    fn test_replace_all() {
        let options = SearchOptions::simple("cat");
        let result = replace_in_document("cat cat cat", &options, "dog");
        assert_eq!(result, "dog dog dog");
    }

    #[test]
    fn test_replace_regex() {
        let options = SearchOptions::regex_search(r"\d+");
        let result = replace_in_document("abc 123 def 456", &options, "NUM");
        assert_eq!(result, "abc NUM def NUM");
    }

    #[test]
    fn test_total_match_count() {
        let r1 = SearchResult {
            item_id: Uuid::new_v4(),
            item_title: "Doc1".to_string(),
            matches: vec![
                SearchMatch { line_number: 1, start: 0, end: 3, context: "abc".to_string() },
                SearchMatch { line_number: 2, start: 0, end: 3, context: "def".to_string() },
            ],
        };
        let r2 = SearchResult {
            item_id: Uuid::new_v4(),
            item_title: "Doc2".to_string(),
            matches: vec![
                SearchMatch { line_number: 1, start: 0, end: 3, context: "ghi".to_string() },
            ],
        };
        assert_eq!(total_match_count(&[r1, r2]), 3);
    }

    #[test]
    fn test_document_count_results() {
        let r1 = SearchResult {
            item_id: Uuid::new_v4(),
            item_title: "Doc1".to_string(),
            matches: vec![
                SearchMatch { line_number: 1, start: 0, end: 3, context: "a".to_string() },
            ],
        };
        assert_eq!(document_count(&[r1]), 1);
        assert_eq!(document_count(&[]), 0);
    }

    #[test]
    fn test_search_summary_no_matches() {
        let summary = search_summary(&[]);
        assert_eq!(summary, "No matches found");
    }

    #[test]
    fn test_search_summary_with_matches() {
        let r = SearchResult {
            item_id: Uuid::new_v4(),
            item_title: "D".to_string(),
            matches: vec![
                SearchMatch { line_number: 1, start: 0, end: 1, context: "x".to_string() },
                SearchMatch { line_number: 2, start: 0, end: 1, context: "y".to_string() },
            ],
        };
        let summary = search_summary(&[r]);
        assert!(summary.contains("2 match(es)"));
        assert!(summary.contains("1 document(s)"));
    }

    #[test]
    fn test_search_result_match_count() {
        let r = SearchResult {
            item_id: Uuid::new_v4(),
            item_title: "D".to_string(),
            matches: vec![
                SearchMatch { line_number: 1, start: 0, end: 1, context: "a".to_string() },
                SearchMatch { line_number: 2, start: 0, end: 1, context: "b".to_string() },
            ],
        };
        assert_eq!(r.match_count(), 2);
    }

    #[test]
    fn test_context_preview() {
        let r = SearchResult {
            item_id: Uuid::new_v4(),
            item_title: "D".to_string(),
            matches: vec![
                SearchMatch { line_number: 1, start: 0, end: 5, context: "Hello World Extended".to_string() },
            ],
        };
        let preview = r.context_preview(0, 10);
        assert_eq!(preview, "Hello Worl...");
        let full = r.context_preview(0, 100);
        assert_eq!(full, "Hello World Extended");
        let missing = r.context_preview(5, 100);
        assert_eq!(missing, "");
    }

    #[test]
    fn test_match_length() {
        let m = SearchMatch { line_number: 1, start: 5, end: 12, context: "x".to_string() };
        assert_eq!(m.match_length(), 7);
    }

    #[test]
    fn test_match_length_saturating() {
        let m = SearchMatch { line_number: 1, start: 10, end: 5, context: "x".to_string() };
        assert_eq!(m.match_length(), 0);
    }

    #[test]
    fn test_highlighted_context() {
        let m = SearchMatch {
            line_number: 1,
            start: 4,
            end: 7,
            context: "The cat sat".to_string(),
        };
        assert_eq!(m.highlighted_context(), "The [cat] sat");
    }

    #[test]
    fn test_highlighted_context_out_of_bounds() {
        let m = SearchMatch {
            line_number: 1,
            start: 100,
            end: 200,
            context: "short".to_string(),
        };
        assert_eq!(m.highlighted_context(), "short");
    }

    #[test]
    fn test_search_options_simple() {
        let opts = SearchOptions::simple("hello");
        assert_eq!(opts.query, "hello");
        assert!(!opts.case_sensitive);
        assert!(opts.search_titles);
        assert!(opts.search_content);
    }

    #[test]
    fn test_search_options_everywhere() {
        let opts = SearchOptions::everywhere("hello");
        assert!(opts.search_titles);
        assert!(opts.search_content);
        assert!(opts.search_notes);
        assert!(opts.search_synopsis);
        assert_eq!(opts.scope_count(), 4);
    }

    #[test]
    fn test_search_options_case_sensitive() {
        let opts = SearchOptions::case_sensitive("Hello");
        assert!(opts.case_sensitive);
        assert_eq!(opts.query, "Hello");
    }

    #[test]
    fn test_search_options_whole_word() {
        let opts = SearchOptions::whole_word("cat");
        assert!(opts.whole_word);
    }

    #[test]
    fn test_search_options_regex_search() {
        let opts = SearchOptions::regex_search(r"\d+");
        assert!(opts.regex);
        assert!(opts.is_regex());
    }

    #[test]
    fn test_scope_description() {
        let opts = SearchOptions::simple("x");
        let desc = opts.scope_description();
        assert!(desc.contains("titles"));
        assert!(desc.contains("content"));
    }

    #[test]
    fn test_scope_count() {
        let opts = SearchOptions::content_only("x");
        assert_eq!(opts.scope_count(), 1);
    }

    #[test]
    fn test_has_scope_false() {
        let opts = SearchOptions {
            query: "test".to_string(),
            search_titles: false,
            search_content: false,
            search_notes: false,
            search_synopsis: false,
            ..Default::default()
        };
        assert!(!opts.has_scope());
        assert_eq!(opts.scope_count(), 0);
    }

    #[test]
    fn test_validate_regex_valid() {
        let opts = SearchOptions::regex_search(r"\d+");
        assert!(opts.validate_regex().is_none());
    }

    #[test]
    fn test_validate_regex_invalid() {
        let opts = SearchOptions::regex_search(r"[invalid");
        assert!(opts.validate_regex().is_some());
    }

    #[test]
    fn test_validate_regex_non_regex() {
        let opts = SearchOptions::simple("hello");
        assert!(opts.validate_regex().is_none());
    }

    #[test]
    fn test_match_line_numbers() {
        let content = "line one\nfind me\nline three\nfind again";
        let opts = SearchOptions::simple("find");
        let lines = match_line_numbers(content, &opts);
        assert_eq!(lines, vec![2, 4]);
    }

    #[test]
    fn test_match_line_numbers_no_match() {
        let content = "line one\nline two";
        let opts = SearchOptions::simple("missing");
        let lines = match_line_numbers(content, &opts);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_count_matches() {
        let text = "cat and cat and cat";
        let opts = SearchOptions::simple("cat");
        assert_eq!(count_matches(text, &opts), 3);
    }

    #[test]
    fn test_count_matches_zero() {
        let text = "no matches here";
        let opts = SearchOptions::simple("xyz");
        assert_eq!(count_matches(text, &opts), 0);
    }

    #[test]
    fn test_extract_matches() {
        let content = "Hello hello HELLO world World";
        let opts = SearchOptions::simple("hello");
        let matches = extract_matches(content, &opts);
        // Case-insensitive, so matches all forms but unique strings
        assert!(matches.len() >= 1);
    }

    #[test]
    fn test_extract_matches_case_sensitive() {
        let content = "Hello hello HELLO";
        let opts = SearchOptions::case_sensitive("Hello");
        let matches = extract_matches(content, &opts);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], "Hello");
    }

    #[test]
    fn test_match_with_context() {
        let content = "line 1\nline 2\ntarget line\nline 4\nline 5";
        let opts = SearchOptions::simple("target");
        let contexts = match_with_context(content, &opts, 1);
        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0].line_number, 3);
        assert_eq!(contexts[0].before.len(), 1);
        assert_eq!(contexts[0].after.len(), 1);
        assert_eq!(contexts[0].before[0], "line 2");
        assert_eq!(contexts[0].after[0], "line 4");
    }

    #[test]
    fn test_match_with_context_first_line() {
        let content = "target line\nline 2\nline 3";
        let opts = SearchOptions::simple("target");
        let contexts = match_with_context(content, &opts, 2);
        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0].before.len(), 0); // No lines before
        assert_eq!(contexts[0].after.len(), 2);
    }

    #[test]
    fn test_match_with_context_last_line() {
        let content = "line 1\nline 2\ntarget line";
        let opts = SearchOptions::simple("target");
        let contexts = match_with_context(content, &opts, 2);
        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0].after.len(), 0); // No lines after
        assert_eq!(contexts[0].before.len(), 2);
    }

    #[test]
    fn test_match_context_display() {
        let ctx = MatchContext {
            line_number: 5,
            matched_line: "found here".to_string(),
            before: vec!["before line".to_string()],
            after: vec!["after line".to_string()],
        };
        let display = ctx.display();
        assert!(display.contains(">"));
        assert!(display.contains("found here"));
        assert!(display.contains("before line"));
        assert!(display.contains("after line"));
    }

    #[test]
    fn test_match_context_total_lines() {
        let ctx = MatchContext {
            line_number: 3,
            matched_line: "match".to_string(),
            before: vec!["a".to_string(), "b".to_string()],
            after: vec!["c".to_string()],
        };
        assert_eq!(ctx.total_lines(), 4);
    }

    #[test]
    fn test_replace_invalid_regex() {
        let options = SearchOptions {
            query: "[invalid".to_string(),
            regex: true,
            ..Default::default()
        };
        // Should return original content when regex is invalid
        let result = replace_in_document("test content", &options, "replacement");
        assert_eq!(result, "test content");
    }

    #[test]
    fn test_search_invalid_regex() {
        let binder = make_binder_with_content(vec![("Doc", "content")]);
        let options = SearchOptions {
            query: "[invalid".to_string(),
            regex: true,
            ..Default::default()
        };
        let results = search_binder(&binder, &options);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_multiline_content() {
        let binder = make_binder_with_content(vec![
            ("Doc", "Line 1 has cat\nLine 2 has dog\nLine 3 has cat again"),
        ]);
        let options = SearchOptions {
            query: "cat".to_string(),
            search_content: true,
            search_titles: false,
            ..Default::default()
        };
        let results = search_binder(&binder, &options);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].matches.len(), 2); // Two lines with "cat"
        assert_eq!(results[0].matches[0].line_number, 1);
        assert_eq!(results[0].matches[1].line_number, 3);
    }

    #[test]
    fn test_search_summary_options() {
        let opts = SearchOptions {
            query: "test".to_string(),
            case_sensitive: true,
            whole_word: true,
            regex: true,
            ..Default::default()
        };
        let summary = opts.summary();
        assert!(summary.contains("case-sensitive"));
        assert!(summary.contains("whole-word"));
        assert!(summary.contains("regex"));
    }

    // --- New search feature tests ---

    #[test]
    fn test_search_by_word_count() {
        let mut binder = Binder::default_structure();
        let mut short = BinderItem::new_text("Short");
        if let Some(ref mut doc) = short.document {
            doc.content = "one two".to_string();
        }
        let mut long = BinderItem::new_text("Long");
        if let Some(ref mut doc) = long.document {
            doc.content = "one two three four five six seven eight nine ten".to_string();
        }
        binder.draft.add_child(short);
        binder.draft.add_child(long);

        let results = search_by_word_count(&binder, 5, 20);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, "Long");
        assert_eq!(results[0].2, 10);
    }

    #[test]
    fn test_search_by_word_count_empty() {
        let binder = Binder::default_structure();
        let results = search_by_word_count(&binder, 100, 500);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_by_word_count_all() {
        let mut binder = Binder::default_structure();
        let mut a = BinderItem::new_text("A");
        if let Some(ref mut doc) = a.document {
            doc.content = "one two three".to_string();
        }
        binder.draft.add_child(a);

        let results = search_by_word_count(&binder, 0, 1000);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_by_keyword() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Tagged");
        item.metadata.add_keyword("important");
        let mut item2 = BinderItem::new_text("Other");
        item2.metadata.add_keyword("trivial");
        binder.draft.add_child(item);
        binder.draft.add_child(item2);

        let results = search_by_keyword(&binder, "important");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, "Tagged");
    }

    #[test]
    fn test_search_by_keyword_not_found() {
        let binder = Binder::default_structure();
        assert!(search_by_keyword(&binder, "nonexistent").is_empty());
    }

    #[test]
    fn test_search_by_label() {
        use crate::core::metadata::{Label, LabelColor};
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Red Item");
        item.metadata.label = Some(Label { name: "Red".to_string(), color: LabelColor::Red });
        binder.draft.add_child(item);
        binder.draft.add_child(BinderItem::new_text("No label"));

        let results = search_by_label(&binder, "Red");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, "Red Item");
    }

    #[test]
    fn test_search_by_label_not_found() {
        let binder = Binder::default_structure();
        assert!(search_by_label(&binder, "Missing").is_empty());
    }

    #[test]
    fn test_search_by_status() {
        use crate::core::metadata::Status;
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Done");
        item.metadata.status = Some(Status::new("Final"));
        binder.draft.add_child(item);

        let results = search_by_status(&binder, "Final");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_by_status_not_found() {
        let binder = Binder::default_structure();
        assert!(search_by_status(&binder, "Missing").is_empty());
    }

    #[test]
    fn test_search_modified_after() {
        use chrono::{Duration, Utc};
        let mut binder = Binder::default_structure();
        binder.draft.add_child(BinderItem::new_text("Recent"));

        let past = Utc::now() - Duration::days(1);
        let results = search_modified_after(&binder, past);
        // All items are freshly created, so all should match
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_modified_after_future() {
        use chrono::{Duration, Utc};
        let binder = Binder::default_structure();
        let future = Utc::now() + Duration::days(1);
        let results = search_modified_after(&binder, future);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_empty_documents() {
        let mut binder = Binder::default_structure();
        binder.draft.add_child(BinderItem::new_text("Empty Doc")); // new text has empty content
        let mut filled = BinderItem::new_text("Filled");
        if let Some(ref mut doc) = filled.document {
            doc.content = "Some content here".to_string();
        }
        binder.draft.add_child(filled);

        let results = search_empty_documents(&binder);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, "Empty Doc");
    }

    #[test]
    fn test_search_empty_documents_none() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Has Content");
        if let Some(ref mut doc) = item.document {
            doc.content = "Content here".to_string();
        }
        binder.draft.add_child(item);

        let results = search_empty_documents(&binder);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_binder_content_and_title() {
        let binder = make_binder_with_content(vec![
            ("Cat Story", "The dog barked."),
        ]);
        let options = SearchOptions {
            query: "cat".to_string(),
            search_titles: true,
            search_content: true,
            ..Default::default()
        };
        let results = search_binder(&binder, &options);
        assert_eq!(results.len(), 1);
        // Should find in title
        assert!(results[0].matches.iter().any(|m| m.context.contains("[Title]")));
    }

    #[test]
    fn test_search_binder_multiple_docs() {
        let binder = make_binder_with_content(vec![
            ("Doc1", "alpha beta gamma"),
            ("Doc2", "delta epsilon"),
            ("Doc3", "alpha zeta"),
        ]);
        let options = SearchOptions::content_only("alpha");
        let results = search_binder(&binder, &options);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_binder_whole_word_in_notes() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Doc");
        if let Some(ref mut doc) = item.document {
            doc.notes = "The cat concatenated strings".to_string();
        }
        binder.draft.add_child(item);

        let options = SearchOptions {
            query: "cat".to_string(),
            whole_word: true,
            search_content: false,
            search_titles: false,
            search_notes: true,
            ..Default::default()
        };
        let results = search_binder(&binder, &options);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].matches.len(), 1); // Only "cat", not "concatenated"
    }

    #[test]
    fn test_extract_matches_empty() {
        let matches = extract_matches("no match here", &SearchOptions::simple("xyz"));
        assert!(matches.is_empty());
    }

    #[test]
    fn test_count_matches_regex() {
        let opts = SearchOptions::regex_search(r"\b\w{4}\b");
        let count = count_matches("The cats like some fish", &opts);
        assert!(count >= 2); // cats, like, some, fish
    }

    // ---- Levenshtein distance tests ----

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_empty() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", "abc"), 3);
    }

    #[test]
    fn test_levenshtein_substitution() {
        assert_eq!(levenshtein_distance("cat", "car"), 1);
        assert_eq!(levenshtein_distance("cat", "cut"), 1);
    }

    #[test]
    fn test_levenshtein_insertion() {
        assert_eq!(levenshtein_distance("cat", "cats"), 1);
    }

    #[test]
    fn test_levenshtein_deletion() {
        assert_eq!(levenshtein_distance("cats", "cat"), 1);
    }

    #[test]
    fn test_levenshtein_multiple_edits() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    }

    #[test]
    fn test_levenshtein_completely_different() {
        assert_eq!(levenshtein_distance("abc", "xyz"), 3);
    }

    // ---- Fuzzy search tests ----

    #[test]
    fn test_fuzzy_search_exact_match() {
        let binder = make_binder_with_content(vec![
            ("Chapter One", "The hero arrived."),
        ]);
        let results = fuzzy_search(&binder, "Chapter One", 0);
        assert!(!results.is_empty());
        assert_eq!(results[0].distance, 0);
        assert!((results[0].score - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_fuzzy_search_close_match() {
        let binder = make_binder_with_content(vec![
            ("Chapter One", "The hero arrived."),
            ("Chapter Two", "The villain escaped."),
        ]);
        // "Chaptor" is 1 edit from "Chapter" in the title
        let results = fuzzy_search(&binder, "Chaptor One", 2);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_fuzzy_search_no_match() {
        let binder = make_binder_with_content(vec![
            ("Chapter One", "The hero arrived."),
        ]);
        let results = fuzzy_search(&binder, "XYZXYZXYZ", 1);
        assert!(results.is_empty());
    }

    #[test]
    fn test_fuzzy_search_content_word() {
        let binder = make_binder_with_content(vec![
            ("Doc", "The adventurer explored the cave."),
        ]);
        // "adventrer" is 1 edit from "adventurer"
        let results = fuzzy_search(&binder, "adventrer", 2);
        let content_matches: Vec<_> = results.iter()
            .filter(|m| m.matched_field == "content")
            .collect();
        assert!(!content_matches.is_empty());
    }

    #[test]
    fn test_fuzzy_search_sorted_by_distance() {
        let binder = make_binder_with_content(vec![
            ("Cat", ""),
            ("Car", ""),
            ("Bat", ""),
        ]);
        let results = fuzzy_search(&binder, "Cat", 2);
        assert!(results.len() >= 2);
        // Should be sorted by distance (exact match first)
        assert!(results[0].distance <= results[results.len() - 1].distance);
    }

    // ---- SavedSearch tests ----

    #[test]
    fn test_saved_search_new() {
        let opts = SearchOptions::simple("hero");
        let saved = SavedSearch::new("Find hero", opts);
        assert_eq!(saved.name, "Find hero");
        assert_eq!(saved.options.query, "hero");
    }

    #[test]
    fn test_saved_search_label() {
        let opts = SearchOptions::simple("quest");
        let saved = SavedSearch::new("My search", opts);
        let label = saved.label();
        assert!(label.contains("My search"));
        assert!(label.contains("quest"));
    }

    // ---- SearchManager tests ----

    #[test]
    fn test_search_manager_new() {
        let mgr = SearchManager::new();
        assert_eq!(mgr.saved_count(), 0);
        assert_eq!(mgr.history_count(), 0);
    }

    #[test]
    fn test_search_manager_save_and_get() {
        let mut mgr = SearchManager::new();
        let opts = SearchOptions::simple("test");
        let id = mgr.save_search("Test search", opts);
        assert_eq!(mgr.saved_count(), 1);

        let saved = mgr.get_saved(&id).unwrap();
        assert_eq!(saved.name, "Test search");
    }

    #[test]
    fn test_search_manager_remove_saved() {
        let mut mgr = SearchManager::new();
        let id = mgr.save_search("Temp", SearchOptions::simple("temp"));
        assert_eq!(mgr.saved_count(), 1);
        assert!(mgr.remove_saved(&id));
        assert_eq!(mgr.saved_count(), 0);
    }

    #[test]
    fn test_search_manager_remove_nonexistent() {
        let mut mgr = SearchManager::new();
        assert!(!mgr.remove_saved(&Uuid::new_v4()));
    }

    #[test]
    fn test_search_manager_history() {
        let mut mgr = SearchManager::new();
        mgr.record_query("hero");
        mgr.record_query("villain");
        mgr.record_query("quest");
        assert_eq!(mgr.history_count(), 3);
        // Most recent first
        assert_eq!(mgr.history[0], "quest");
        assert_eq!(mgr.history[1], "villain");
        assert_eq!(mgr.history[2], "hero");
    }

    #[test]
    fn test_search_manager_history_dedup() {
        let mut mgr = SearchManager::new();
        mgr.record_query("hero");
        mgr.record_query("villain");
        mgr.record_query("hero"); // duplicate
        assert_eq!(mgr.history_count(), 2);
        assert_eq!(mgr.history[0], "hero"); // Most recent
        assert_eq!(mgr.history[1], "villain");
    }

    #[test]
    fn test_search_manager_history_max() {
        let mut mgr = SearchManager::new();
        mgr.max_history = 3;
        mgr.record_query("a");
        mgr.record_query("b");
        mgr.record_query("c");
        mgr.record_query("d");
        assert_eq!(mgr.history_count(), 3);
        assert_eq!(mgr.history[0], "d");
    }

    #[test]
    fn test_search_manager_suggest() {
        let mut mgr = SearchManager::new();
        mgr.record_query("chapter one");
        mgr.record_query("chapter two");
        mgr.record_query("character");
        mgr.record_query("villain");

        let suggestions = mgr.suggest("chap");
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.contains(&"chapter one"));
        assert!(suggestions.contains(&"chapter two"));
    }

    #[test]
    fn test_search_manager_suggest_case_insensitive() {
        let mut mgr = SearchManager::new();
        mgr.record_query("Hero Quest");
        let suggestions = mgr.suggest("hero");
        assert_eq!(suggestions.len(), 1);
    }

    #[test]
    fn test_search_manager_clear_history() {
        let mut mgr = SearchManager::new();
        mgr.record_query("test");
        mgr.clear_history();
        assert_eq!(mgr.history_count(), 0);
    }

    #[test]
    fn test_search_manager_saved_names() {
        let mut mgr = SearchManager::new();
        mgr.save_search("Search A", SearchOptions::simple("a"));
        mgr.save_search("Search B", SearchOptions::simple("b"));
        let names = mgr.saved_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"Search A"));
        assert!(names.contains(&"Search B"));
    }
}
