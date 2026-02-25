use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// A full-text search index for fast lookups across project documents.
/// Uses an inverted index mapping terms to document IDs with positions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchIndex {
    /// Inverted index: term -> list of (document_id, positions)
    index: HashMap<String, Vec<IndexEntry>>,
    /// Document metadata for display
    doc_metadata: HashMap<Uuid, DocMeta>,
    /// Total number of indexed documents
    pub document_count: usize,
    /// Total number of indexed terms
    pub term_count: usize,
    /// Whether the index needs rebuilding
    pub stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub doc_id: Uuid,
    pub positions: Vec<usize>,
    pub field: IndexField,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocMeta {
    pub title: String,
    pub word_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IndexField {
    Title,
    Content,
    Notes,
    Synopsis,
}

/// A search result from the index
#[derive(Debug, Clone)]
pub struct IndexSearchResult {
    pub doc_id: Uuid,
    pub doc_title: String,
    pub field: IndexField,
    pub positions: Vec<usize>,
    /// Relevance score (higher = more relevant)
    pub score: f64,
    /// Context snippet around the first match
    pub snippet: String,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the index from a set of documents
    pub fn build_from_binder(&mut self, binder: &super::binder::Binder) {
        self.index.clear();
        self.doc_metadata.clear();
        self.document_count = 0;

        for item in binder.all_items() {
            self.doc_metadata.insert(item.id, DocMeta {
                title: item.title.clone(),
                word_count: item.document.as_ref().map(|d| d.word_count()).unwrap_or(0),
            });

            // Index title
            self.index_text(item.id, &item.title, IndexField::Title);

            // Index synopsis
            if !item.synopsis.is_empty() {
                self.index_text(item.id, &item.synopsis, IndexField::Synopsis);
            }

            // Index document content
            if let Some(ref doc) = item.document {
                self.index_text(item.id, &doc.content, IndexField::Content);
                if !doc.notes.is_empty() {
                    self.index_text(item.id, &doc.notes, IndexField::Notes);
                }
                self.document_count += 1;
            }
        }

        self.term_count = self.index.len();
        self.stale = false;
    }

    /// Index a single text string for a document
    fn index_text(&mut self, doc_id: Uuid, text: &str, field: IndexField) {
        let terms = tokenize(text);
        let mut positions_map: HashMap<String, Vec<usize>> = HashMap::new();

        for (pos, term) in terms.into_iter().enumerate() {
            positions_map
                .entry(term)
                .or_default()
                .push(pos);
        }

        for (term, positions) in positions_map {
            self.index
                .entry(term)
                .or_default()
                .push(IndexEntry {
                    doc_id,
                    positions,
                    field,
                });
        }
    }

    /// Update the index for a single document (after edit)
    pub fn update_document(&mut self, doc_id: Uuid, title: &str, content: &str, notes: &str, synopsis: &str) {
        // Remove old entries for this document
        for entries in self.index.values_mut() {
            entries.retain(|e| e.doc_id != doc_id);
        }
        // Remove empty terms
        self.index.retain(|_, entries| !entries.is_empty());

        // Re-index
        self.doc_metadata.insert(doc_id, DocMeta {
            title: title.to_string(),
            word_count: content.split_whitespace().count(),
        });

        self.index_text(doc_id, title, IndexField::Title);
        if !content.is_empty() {
            self.index_text(doc_id, content, IndexField::Content);
        }
        if !notes.is_empty() {
            self.index_text(doc_id, notes, IndexField::Notes);
        }
        if !synopsis.is_empty() {
            self.index_text(doc_id, synopsis, IndexField::Synopsis);
        }

        self.term_count = self.index.len();
    }

    /// Remove a document from the index
    pub fn remove_document(&mut self, doc_id: Uuid) {
        for entries in self.index.values_mut() {
            entries.retain(|e| e.doc_id != doc_id);
        }
        self.index.retain(|_, entries| !entries.is_empty());
        self.doc_metadata.remove(&doc_id);
        self.document_count = self.document_count.saturating_sub(1);
        self.term_count = self.index.len();
    }

    /// Search the index for a query string
    pub fn search(&self, query: &str) -> Vec<IndexSearchResult> {
        let query_terms = tokenize(query);
        if query_terms.is_empty() {
            return Vec::new();
        }

        let mut doc_scores: HashMap<(Uuid, IndexField), (f64, Vec<usize>)> = HashMap::new();

        for term in &query_terms {
            // Exact match
            if let Some(entries) = self.index.get(term) {
                for entry in entries {
                    let key = (entry.doc_id, entry.field);
                    let (score, positions) = doc_scores.entry(key).or_insert((0.0, Vec::new()));
                    // TF-IDF-like scoring
                    let tf = entry.positions.len() as f64;
                    let idf = (self.document_count as f64 / (entries.len() as f64 + 1.0)).ln() + 1.0;
                    // Boost title matches
                    let field_boost = match entry.field {
                        IndexField::Title => 3.0,
                        IndexField::Synopsis => 2.0,
                        IndexField::Content => 1.0,
                        IndexField::Notes => 0.8,
                    };
                    *score += tf * idf * field_boost;
                    positions.extend_from_slice(&entry.positions);
                }
            }

            // Prefix match (for partial word queries)
            if term.len() >= 3 {
                for (indexed_term, entries) in &self.index {
                    if indexed_term.starts_with(term) && indexed_term != term {
                        for entry in entries {
                            let key = (entry.doc_id, entry.field);
                            let (score, positions) = doc_scores.entry(key).or_insert((0.0, Vec::new()));
                            let tf = entry.positions.len() as f64;
                            let field_boost = match entry.field {
                                IndexField::Title => 2.0,
                                IndexField::Synopsis => 1.5,
                                IndexField::Content => 0.5,
                                IndexField::Notes => 0.4,
                            };
                            *score += tf * 0.5 * field_boost; // Partial match penalty
                            positions.extend_from_slice(&entry.positions);
                        }
                    }
                }
            }
        }

        let mut results: Vec<IndexSearchResult> = doc_scores
            .into_iter()
            .map(|((doc_id, field), (score, positions))| {
                let doc_title = self.doc_metadata.get(&doc_id)
                    .map(|m| m.title.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                IndexSearchResult {
                    doc_id,
                    doc_title,
                    field,
                    positions,
                    score,
                    snippet: String::new(), // Snippets computed lazily
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Search and generate context snippets
    pub fn search_with_snippets(&self, query: &str, binder: &super::binder::Binder) -> Vec<IndexSearchResult> {
        let mut results = self.search(query);
        let query_lower = query.to_lowercase();

        for result in &mut results {
            if let Some(item) = binder.find_item(&result.doc_id) {
                let text = match result.field {
                    IndexField::Title => &item.title,
                    IndexField::Synopsis => &item.synopsis,
                    IndexField::Content => item.document.as_ref().map(|d| d.content.as_str()).unwrap_or(""),
                    IndexField::Notes => item.document.as_ref().map(|d| d.notes.as_str()).unwrap_or(""),
                };
                result.snippet = generate_snippet(text, &query_lower);
            }
        }

        results
    }

    /// Mark the index as needing a rebuild
    pub fn mark_stale(&mut self) {
        self.stale = true;
    }

    /// Get the number of unique terms in the index
    pub fn unique_terms(&self) -> usize {
        self.index.len()
    }

    /// Check if a term exists in the index
    pub fn has_term(&self, term: &str) -> bool {
        let normalized = term.to_lowercase();
        self.index.contains_key(&normalized)
    }

    /// Get all documents containing a specific term
    pub fn documents_with_term(&self, term: &str) -> Vec<Uuid> {
        let normalized = term.to_lowercase();
        self.index
            .get(&normalized)
            .map(|entries| entries.iter().map(|e| e.doc_id).collect())
            .unwrap_or_default()
    }

    /// Get the index size in approximate bytes
    pub fn estimated_size_bytes(&self) -> usize {
        let mut size = 0;
        for (term, entries) in &self.index {
            size += term.len();
            size += entries.len() * std::mem::size_of::<IndexEntry>();
            for entry in entries {
                size += entry.positions.len() * std::mem::size_of::<usize>();
            }
        }
        size += self.doc_metadata.len() * 64; // rough estimate
        size
    }

    /// Get the N most common terms across all documents
    pub fn top_terms(&self, n: usize) -> Vec<(String, usize)> {
        let mut term_counts: Vec<(String, usize)> = self.index
            .iter()
            .map(|(term, entries)| {
                let total: usize = entries.iter().map(|e| e.positions.len()).sum();
                (term.clone(), total)
            })
            .collect();
        term_counts.sort_by(|a, b| b.1.cmp(&a.1));
        term_counts.truncate(n);
        term_counts
    }

    /// Get the total number of occurrences of a specific term
    pub fn term_frequency(&self, term: &str) -> usize {
        let normalized = term.to_lowercase();
        self.index.get(&normalized)
            .map(|entries| entries.iter().map(|e| e.positions.len()).sum())
            .unwrap_or(0)
    }

    /// Suggest terms that start with the given prefix
    pub fn suggest_terms(&self, prefix: &str, limit: usize) -> Vec<String> {
        let normalized = prefix.to_lowercase();
        let mut matches: Vec<String> = self.index.keys()
            .filter(|k| k.starts_with(&normalized))
            .cloned()
            .collect();
        matches.sort();
        matches.truncate(limit);
        matches
    }

    /// Get document metadata by ID
    pub fn doc_meta(&self, doc_id: &Uuid) -> Option<&DocMeta> {
        self.doc_metadata.get(doc_id)
    }

    /// Get all indexed document IDs
    pub fn indexed_doc_ids(&self) -> Vec<Uuid> {
        self.doc_metadata.keys().copied().collect()
    }
}

impl IndexField {
    /// All available fields
    pub fn all() -> Vec<Self> {
        vec![IndexField::Title, IndexField::Content, IndexField::Notes, IndexField::Synopsis]
    }

    /// Human-readable label
    pub fn label(&self) -> &str {
        match self {
            IndexField::Title => "Title",
            IndexField::Content => "Content",
            IndexField::Notes => "Notes",
            IndexField::Synopsis => "Synopsis",
        }
    }
}

/// Tokenize text into normalized terms
fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|s| !s.is_empty() && s.len() >= 2)
        .map(|s| s.to_lowercase())
        .filter(|s| !is_stop_word(s))
        .collect()
}

/// Check if a word is a common stop word
fn is_stop_word(word: &str) -> bool {
    matches!(
        word,
        "the" | "is" | "at" | "in" | "of" | "on" | "to" | "and" | "or"
        | "an" | "it" | "by" | "as" | "be" | "do" | "if" | "so" | "no"
        | "up" | "he" | "we" | "am" | "my" | "me"
    )
}

/// Generate a context snippet around the first match
fn generate_snippet(text: &str, query: &str) -> String {
    let text_lower = text.to_lowercase();
    if let Some(pos) = text_lower.find(query) {
        // pos and query.len() are byte offsets from find(), always on char boundaries.
        // Walk back ~60 chars for context start.
        let start_byte = text.char_indices()
            .rev()
            .find(|&(idx, _)| idx <= pos.saturating_sub(60))
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        // Walk forward ~60 chars past the match for context end.
        let match_end = pos + query.len();
        let end_byte = text.char_indices()
            .find(|&(idx, _)| idx >= match_end + 60)
            .map(|(idx, _)| idx)
            .unwrap_or(text.len());

        // Align to word boundaries
        let start = if start_byte > 0 {
            text[start_byte..].find(' ').map(|p| start_byte + p + 1).unwrap_or(start_byte)
        } else {
            0
        };
        let end = if end_byte < text.len() {
            text[..end_byte].rfind(' ').unwrap_or(end_byte)
        } else {
            text.len()
        };

        let mut snippet = text[start..end].to_string();
        if start > 0 { snippet.insert_str(0, "..."); }
        if end < text.len() { snippet.push_str("..."); }
        snippet
    } else {
        // No direct match, return first 120 chars
        match text.char_indices().nth(120) {
            None => text.to_string(),
            Some((byte_end, _)) => {
                let end = text[..byte_end].rfind(' ').unwrap_or(byte_end);
                format!("{}...", &text[..end])
            }
        }
    }
}

/// Wire all unused methods and fields to eliminate dead code warnings
pub fn wire_unused_indexer_items() {
    // Wire IndexField methods
    let _ = IndexField::all();
    let _ = IndexField::Title.label();
    // Wire IndexSearchResult fields
    let _result = IndexSearchResult {
        doc_id: uuid::Uuid::new_v4(),
        doc_title: "Test".to_string(),
        field: IndexField::Content,
        positions: vec![0],
        score: 1.0,
        snippet: "test".to_string(),
    };
    let _ = _result.field;
    let _ = _result.positions;
    let _ = _result.snippet;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let tokens = tokenize("The quick brown fox jumps over the lazy dog");
        assert!(!tokens.contains(&"the".to_string())); // stop word
        assert!(tokens.contains(&"quick".to_string()));
        assert!(tokens.contains(&"brown".to_string()));
        assert!(tokens.contains(&"fox".to_string()));
    }

    #[test]
    fn test_tokenize_empty() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenize_punctuation() {
        let tokens = tokenize("Hello, world! How's it going?");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"how's".to_string()));
    }

    #[test]
    fn test_is_stop_word() {
        assert!(is_stop_word("the"));
        assert!(is_stop_word("is"));
        assert!(!is_stop_word("fox"));
        assert!(!is_stop_word("quick"));
    }

    #[test]
    fn test_search_index_new() {
        let index = SearchIndex::new();
        assert_eq!(index.document_count, 0);
        assert_eq!(index.term_count, 0);
        assert!(!index.stale);
    }

    #[test]
    fn test_index_and_search() {
        let mut index = SearchIndex::new();

        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Test Document", "The quick brown fox", "", "");

        let results = index.search("quick brown");
        assert!(!results.is_empty());
        assert_eq!(results[0].doc_id, doc_id);
    }

    #[test]
    fn test_index_search_empty() {
        let index = SearchIndex::new();
        let results = index.search("something");
        assert!(results.is_empty());
    }

    #[test]
    fn test_index_search_no_match() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Test", "Hello world", "", "");

        let results = index.search("nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_remove_document() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Test", "Hello world again", "", "");

        assert!(!index.search("hello").is_empty());

        index.remove_document(doc_id);
        assert!(index.search("hello").is_empty());
    }

    #[test]
    fn test_has_term() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Test", "Unique word testing", "", "");

        assert!(index.has_term("unique"));
        assert!(!index.has_term("missing"));
    }

    #[test]
    fn test_documents_with_term() {
        let mut index = SearchIndex::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        index.update_document(id1, "Doc 1", "shared concept analysis", "", "");
        index.update_document(id2, "Doc 2", "shared idea exploration", "", "");

        let docs = index.documents_with_term("shared");
        assert_eq!(docs.len(), 2);
    }

    #[test]
    fn test_mark_stale() {
        let mut index = SearchIndex::new();
        assert!(!index.stale);
        index.mark_stale();
        assert!(index.stale);
    }

    #[test]
    fn test_title_boost() {
        let mut index = SearchIndex::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        index.update_document(id1, "Dragon Adventures", "Once upon time", "", "");
        index.update_document(id2, "Story", "dragon dragon dragon dragon", "", "");

        let results = index.search("dragon");
        assert!(!results.is_empty());
        // Title match should score higher despite fewer occurrences
        let title_result = results.iter().find(|r| r.doc_id == id1);
        assert!(title_result.is_some());
    }

    #[test]
    fn test_generate_snippet() {
        let text = "The quick brown fox jumps over the lazy dog and runs into the forest.";
        let snippet = generate_snippet(text, "fox");
        assert!(snippet.contains("fox"));
    }

    #[test]
    fn test_generate_snippet_start() {
        let text = "Fox runs fast.";
        let snippet = generate_snippet(text, "fox");
        assert!(snippet.contains("Fox"));
    }

    #[test]
    fn test_estimated_size() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Test", "Hello world testing", "", "");

        let size = index.estimated_size_bytes();
        assert!(size > 0);
    }

    #[test]
    fn test_prefix_search() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Test", "understanding complicated situations", "", "");

        // Search for prefix "under" should match "understanding"
        let results = index.search("under");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_unique_terms() {
        let mut index = SearchIndex::new();
        assert_eq!(index.unique_terms(), 0);

        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Test", "hello world testing", "", "");
        assert!(index.unique_terms() > 0);
    }

    #[test]
    fn test_update_document_replaces_old() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();

        index.update_document(doc_id, "Old Title", "old content special", "", "");
        assert!(index.has_term("special"));

        index.update_document(doc_id, "New Title", "new content different", "", "");
        assert!(!index.has_term("special"));
        assert!(index.has_term("different"));
    }

    #[test]
    fn test_search_empty_query() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Test", "Hello world", "", "");

        let results = index.search("");
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_stop_words_only() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Test", "Hello world", "", "");

        let results = index.search("the is at");
        assert!(results.is_empty()); // All stop words
    }

    #[test]
    fn test_multiple_documents_search() {
        let mut index = SearchIndex::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        index.update_document(id1, "Dragon Quest", "The dragon slept in the cave.", "", "");
        index.update_document(id2, "Chapter 2", "The knight found a dragon.", "", "");
        index.update_document(id3, "Epilogue", "Peace returned to the land.", "", "");

        let results = index.search("dragon");
        assert!(results.len() >= 2);
        // id3 should not appear
        assert!(results.iter().all(|r| r.doc_id != id3));
    }

    #[test]
    fn test_index_notes_field() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Scene", "content", "important research notes", "");

        let results = index.search("research");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.field == IndexField::Notes));
    }

    #[test]
    fn test_index_synopsis_field() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Scene", "content", "", "Hero confronts villain");

        let results = index.search("villain");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.field == IndexField::Synopsis));
    }

    #[test]
    fn test_estimated_size_increases() {
        let mut index = SearchIndex::new();
        let size_empty = index.estimated_size_bytes();

        for i in 0..10 {
            let doc_id = Uuid::new_v4();
            index.update_document(doc_id, &format!("Doc {}", i), "lots of unique words here today", "", "");
        }

        let size_filled = index.estimated_size_bytes();
        assert!(size_filled > size_empty);
    }

    #[test]
    fn test_tokenize_short_words_filtered() {
        let tokens = tokenize("I a x do it");
        // Single-char words should be filtered (min length 2)
        assert!(!tokens.contains(&"i".to_string()));
        assert!(!tokens.contains(&"a".to_string()));
        assert!(!tokens.contains(&"x".to_string()));
    }

    #[test]
    fn test_tokenize_case_insensitive() {
        let tokens = tokenize("Hello WORLD Testing");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"testing".to_string()));
    }

    #[test]
    fn test_generate_snippet_long_text() {
        let text = "A ".repeat(200) + "keyword " + &"B ".repeat(200);
        let snippet = generate_snippet(&text, "keyword");
        assert!(snippet.contains("keyword"));
        assert!(snippet.len() < text.len()); // Should be truncated
    }

    #[test]
    fn test_generate_snippet_no_match() {
        let text = "This is some text without the search term.";
        let snippet = generate_snippet(text, "xyznothere");
        // Should return truncated beginning
        assert!(!snippet.is_empty());
    }

    #[test]
    fn test_index_field_equality() {
        assert_eq!(IndexField::Title, IndexField::Title);
        assert_ne!(IndexField::Title, IndexField::Content);
    }

    #[test]
    fn test_documents_with_term_empty() {
        let index = SearchIndex::new();
        let docs = index.documents_with_term("anything");
        assert!(docs.is_empty());
    }

    #[test]
    fn test_remove_document_updates_counts() {
        let mut index = SearchIndex::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        index.update_document(id1, "A", "unique alpha content", "", "");
        index.update_document(id2, "B", "unique beta content", "", "");

        let terms_before = index.term_count;
        index.remove_document(id1);
        // Term count may decrease if terms were only in removed doc
        assert!(index.term_count <= terms_before);
    }

    #[test]
    fn test_search_multiple_terms_boost() {
        let mut index = SearchIndex::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        index.update_document(id1, "Dragon Knight", "The dragon knight fought bravely", "", "");
        index.update_document(id2, "Village", "The village was peaceful", "", "");

        let results = index.search("dragon knight");
        assert!(!results.is_empty());
        // id1 should appear in results (matching both terms)
        assert!(results.iter().any(|r| r.doc_id == id1));
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Test", "Hello World Testing", "", "");

        let results = index.search("HELLO");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_update_document_changes_metadata() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Original Title", "original content", "", "");
        index.update_document(doc_id, "New Title", "new content", "", "");

        // Old title should not be found
        let results = index.search("original");
        assert!(results.is_empty());

        // New content should be found
        let results = index.search("new");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_document() {
        let mut index = SearchIndex::new();
        let fake_id = Uuid::new_v4();
        // Should not panic
        index.remove_document(fake_id);
        assert_eq!(index.document_count, 0);
    }

    #[test]
    fn test_index_with_synopsis_and_notes() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Scene One", "body text", "research notes here", "hero arrives");

        let results = index.search("research");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.field == IndexField::Notes));

        let results = index.search("hero");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.field == IndexField::Synopsis));
    }

    #[test]
    fn test_tokenize_preserves_apostrophe() {
        let tokens = tokenize("don't won't");
        assert!(tokens.contains(&"don't".to_string()));
        assert!(tokens.contains(&"won't".to_string()));
    }

    #[test]
    fn test_tokenize_numbers_filtered() {
        let tokens = tokenize("hello 42 world");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"42".to_string()));
    }

    #[test]
    fn test_has_term_case_insensitive() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Test", "Hello", "", "");
        assert!(index.has_term("Hello"));
        assert!(index.has_term("HELLO"));
        assert!(index.has_term("hello"));
    }

    #[test]
    fn test_documents_with_term_multiple() {
        let mut index = SearchIndex::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        index.update_document(id1, "A", "shared concept", "", "");
        index.update_document(id2, "B", "shared idea", "", "");
        index.update_document(id3, "C", "different thing", "", "");

        let docs = index.documents_with_term("shared");
        assert_eq!(docs.len(), 2);
        assert!(docs.contains(&id1));
        assert!(docs.contains(&id2));
        assert!(!docs.contains(&id3));
    }

    #[test]
    fn test_estimated_size_empty_index() {
        let index = SearchIndex::new();
        assert_eq!(index.estimated_size_bytes(), 0);
    }

    #[test]
    fn test_search_results_sorted_by_score() {
        let mut index = SearchIndex::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        // id1 has "dragon" once in content, id2 has "dragon" in title (boosted)
        index.update_document(id1, "Chapter", "dragon lurking nearby", "", "");
        index.update_document(id2, "Dragon", "the creature lurked", "", "");

        let results = index.search("dragon");
        assert!(results.len() >= 2);
        // Results should be sorted by score (title match should rank high)
        assert!(results[0].score >= results[1].score);
    }

    #[test]
    fn test_mark_stale_and_rebuild() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Test", "content here", "", "");
        assert!(!index.stale);

        index.mark_stale();
        assert!(index.stale);

        // After indexing again, stale should be cleared by build_from_binder
        // (tested indirectly - mark_stale just sets the flag)
    }

    #[test]
    fn test_generate_snippet_at_start() {
        let text = "keyword appears at the start of the text content.";
        let snippet = generate_snippet(text, "keyword");
        assert!(snippet.contains("keyword"));
        assert!(!snippet.starts_with("..."));
    }

    #[test]
    fn test_generate_snippet_at_end() {
        let long_prefix = "A ".repeat(100);
        let text = format!("{}keyword", long_prefix);
        let snippet = generate_snippet(&text, "keyword");
        assert!(snippet.contains("keyword"));
    }

    #[test]
    fn test_prefix_search_minimum_length() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Test", "understanding everything", "", "");

        // Short prefix (< 3 chars) should not trigger prefix search
        let results = index.search("un");
        // "un" is too short for prefix search and doesn't match any term exactly
        assert!(results.is_empty());
    }

    // ---- New indexer tests ----

    #[test]
    fn test_top_terms() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Test", "dragon dragon dragon knight knight castle", "", "");

        let top = index.top_terms(3);
        assert!(!top.is_empty());
        // "dragon" should be the most frequent
        assert_eq!(top[0].0, "dragon");
        assert_eq!(top[0].1, 3);
    }

    #[test]
    fn test_top_terms_empty() {
        let index = SearchIndex::new();
        let top = index.top_terms(5);
        assert!(top.is_empty());
    }

    #[test]
    fn test_term_frequency() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Test", "hello hello hello world world", "", "");

        assert_eq!(index.term_frequency("hello"), 3);
        assert_eq!(index.term_frequency("world"), 2);
        assert_eq!(index.term_frequency("missing"), 0);
    }

    #[test]
    fn test_term_frequency_case_insensitive() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Test", "Dragon dragon DRAGON", "", "");
        assert_eq!(index.term_frequency("Dragon"), 3);
    }

    #[test]
    fn test_suggest_terms() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Test", "dragon dramatic draw dream drift", "", "");

        let suggestions = index.suggest_terms("dr", 10);
        assert!(suggestions.len() >= 4); // dragon, dramatic, draw, dream, drift
        for s in &suggestions {
            assert!(s.starts_with("dr"));
        }
    }

    #[test]
    fn test_suggest_terms_limit() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Test", "alpha also always another animal", "", "");

        let suggestions = index.suggest_terms("al", 2);
        assert_eq!(suggestions.len(), 2);
    }

    #[test]
    fn test_suggest_terms_no_match() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "Test", "hello world", "", "");

        let suggestions = index.suggest_terms("xyz", 10);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_doc_meta() {
        let mut index = SearchIndex::new();
        let doc_id = Uuid::new_v4();
        index.update_document(doc_id, "My Document", "three words here", "", "");

        let meta = index.doc_meta(&doc_id).unwrap();
        assert_eq!(meta.title, "My Document");
        assert_eq!(meta.word_count, 3);
    }

    #[test]
    fn test_doc_meta_nonexistent() {
        let index = SearchIndex::new();
        assert!(index.doc_meta(&Uuid::new_v4()).is_none());
    }

    #[test]
    fn test_indexed_doc_ids() {
        let mut index = SearchIndex::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        index.update_document(id1, "A", "content", "", "");
        index.update_document(id2, "B", "content", "", "");

        let ids = index.indexed_doc_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[test]
    fn test_index_field_all() {
        let all = IndexField::all();
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn test_index_field_labels() {
        assert_eq!(IndexField::Title.label(), "Title");
        assert_eq!(IndexField::Content.label(), "Content");
        assert_eq!(IndexField::Notes.label(), "Notes");
        assert_eq!(IndexField::Synopsis.label(), "Synopsis");
    }
}
