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
                    context: format!("[Title] {}", item.title.clone()),
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
                    context: format!("[Synopsis] {}", item.synopsis.clone()),
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
                        context: format!("[Notes] {}", doc.notes.clone()),
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
    let mut seen = std::collections::HashSet::new();
    let mut results = Vec::new();
    for m in regex.find_iter(content) {
        let s = m.as_str().to_string();
        if seen.insert(s.clone()) {
            results.push(s);
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
}
