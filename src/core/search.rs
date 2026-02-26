#![allow(dead_code)] // Methods used by test code
use regex::Regex;
use std::sync::Mutex;
use uuid::Uuid;

use super::binder::Binder;

/// Simple single-entry regex cache: avoids recompilation when the user
/// triggers repeated searches with the same pattern (e.g., live search
/// while typing, or scrolling through results).
static REGEX_CACHE: Mutex<Option<(String, Regex)>> = Mutex::new(None);

fn get_or_compile_regex(pattern: &str) -> Option<Regex> {
    let mut cache = REGEX_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((ref cached_pat, ref cached_re)) = *cache {
        if cached_pat == pattern {
            return Some(cached_re.clone());
        }
    }
    match Regex::new(pattern) {
        Ok(re) => {
            *cache = Some((pattern.to_string(), re.clone()));
            Some(re)
        }
        Err(_) => None,
    }
}

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
    /// Maximum number of document results to return (0 = unlimited)
    pub max_results: usize,
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
            max_results: 0,
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
    let regex = match get_or_compile_regex(&pattern) {
        Some(r) => r,
        None => return results,
    };

    let max = options.max_results;
    binder.for_each_item(|item| {
        // Early termination when max_results is reached
        if max > 0 && results.len() >= max {
            return;
        }

        let mut matches = Vec::new();

        // Search title
        if options.search_titles {
            for _m in regex.find_iter(&item.title) {
                matches.push(SearchMatch {
                    context: format!("[Title] {}", item.title),
                });
            }
        }

        // Search content
        if options.search_content {
            if let Some(ref doc) = item.document {
                for line in doc.content.lines() {
                    for _m in regex.find_iter(line) {
                        matches.push(SearchMatch {
                            context: line.to_string(),
                        });
                    }
                }
            }
        }

        // Search synopsis
        if options.search_synopsis && !item.synopsis.is_empty() {
            for _m in regex.find_iter(&item.synopsis) {
                matches.push(SearchMatch {
                    context: format!("[Synopsis] {}", item.synopsis),
                });
            }
        }

        // Search notes
        if options.search_notes {
            if let Some(ref doc) = item.document {
                for _m in regex.find_iter(&doc.notes) {
                    matches.push(SearchMatch {
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
    });

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
            match m.context.char_indices().nth(max_len) {
                None => m.context.clone(),
                Some((byte_idx, _)) => format!("{}...", &m.context[..byte_idx]),
            }
        } else {
            String::new()
        }
    }
}

/// Manages saved searches and search history
#[derive(Debug, Default)]
pub struct SearchManager {
    pub history: Vec<String>,
    pub max_history: usize,
}

impl SearchManager {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            max_history: 50,
        }
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
        self.history
            .iter()
            .filter(|q| q.to_lowercase().starts_with(&lower))
            .map(|q| q.as_str())
            .collect()
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
        let options = SearchOptions {
            query: String::new(),
            ..Default::default()
        };
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
        let binder = make_binder_with_content(vec![("Doc", "Hello hello HELLO")]);
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
        let content_matches: Vec<_> = results[0]
            .matches
            .iter()
            .filter(|m| !m.context.starts_with("[Title]"))
            .collect();
        assert_eq!(content_matches.len(), 1);
    }

    #[test]
    fn test_search_in_title() {
        let binder = make_binder_with_content(vec![("Important Chapter", "Some content here")]);
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
        let binder = make_binder_with_content(vec![("Doc", "Hello world")]);
        let options = SearchOptions {
            query: "zzzzz".to_string(),
            ..Default::default()
        };
        let results = search_binder(&binder, &options);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_result_context() {
        let binder = make_binder_with_content(vec![("Doc", "The quick brown fox jumps over the lazy dog")]);
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
    fn test_search_whole_word() {
        let binder = make_binder_with_content(vec![("Doc", "The cat concatenated strings")]);
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
        let binder = make_binder_with_content(vec![("Doc", "apple 123 banana 456 cherry")]);
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
    fn test_total_match_count() {
        let r1 = SearchResult {
            item_id: Uuid::new_v4(),
            item_title: "Doc1".to_string(),
            matches: vec![
                SearchMatch {
                    context: "abc".to_string(),
                },
                SearchMatch {
                    context: "def".to_string(),
                },
            ],
        };
        let r2 = SearchResult {
            item_id: Uuid::new_v4(),
            item_title: "Doc2".to_string(),
            matches: vec![SearchMatch {
                context: "ghi".to_string(),
            }],
        };
        assert_eq!(total_match_count(&[r1, r2]), 3);
    }

    #[test]
    fn test_document_count_results() {
        let r1 = SearchResult {
            item_id: Uuid::new_v4(),
            item_title: "Doc1".to_string(),
            matches: vec![SearchMatch {
                context: "a".to_string(),
            }],
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
                SearchMatch {
                    context: "x".to_string(),
                },
                SearchMatch {
                    context: "y".to_string(),
                },
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
                SearchMatch {
                    context: "a".to_string(),
                },
                SearchMatch {
                    context: "b".to_string(),
                },
            ],
        };
        assert_eq!(r.match_count(), 2);
    }

    #[test]
    fn test_context_preview() {
        let r = SearchResult {
            item_id: Uuid::new_v4(),
            item_title: "D".to_string(),
            matches: vec![SearchMatch {
                context: "Hello World Extended".to_string(),
            }],
        };
        let preview = r.context_preview(0, 10);
        assert_eq!(preview, "Hello Worl...");
        let full = r.context_preview(0, 100);
        assert_eq!(full, "Hello World Extended");
        let missing = r.context_preview(5, 100);
        assert_eq!(missing, "");
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
        let binder = make_binder_with_content(vec![("Doc", "Line 1 has cat\nLine 2 has dog\nLine 3 has cat again")]);
        let options = SearchOptions {
            query: "cat".to_string(),
            search_content: true,
            search_titles: false,
            ..Default::default()
        };
        let results = search_binder(&binder, &options);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].matches.len(), 2); // Two lines with "cat"
    }

    #[test]
    fn test_search_binder_content_and_title() {
        let binder = make_binder_with_content(vec![("Cat Story", "The dog barked.")]);
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
    fn test_search_manager_history() {
        let mut mgr = SearchManager::new();
        mgr.record_query("hero");
        mgr.record_query("villain");
        mgr.record_query("quest");
        assert_eq!(mgr.history.len(), 3);
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
        assert_eq!(mgr.history.len(), 2);
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
        assert_eq!(mgr.history.len(), 3);
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
}
