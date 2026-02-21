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
