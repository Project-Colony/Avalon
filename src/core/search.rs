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
}
