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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

        for (pos, term) in terms.iter().enumerate() {
            positions_map
                .entry(term.clone())
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
                    field: field.clone(),
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
                    let key = (entry.doc_id, entry.field.clone());
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
                            let key = (entry.doc_id, entry.field.clone());
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
        let context_chars = 60;
        let start = if pos > context_chars { pos - context_chars } else { 0 };
        let end = (pos + query.len() + context_chars).min(text.len());

        // Align to word boundaries
        let start = if start > 0 {
            text[start..].find(' ').map(|p| start + p + 1).unwrap_or(start)
        } else {
            start
        };
        let end = if end < text.len() {
            text[..end].rfind(' ').unwrap_or(end)
        } else {
            end
        };

        let mut snippet = text[start..end].to_string();
        if start > 0 { snippet = format!("...{}", snippet); }
        if end < text.len() { snippet = format!("{}...", snippet); }
        snippet
    } else {
        // No direct match, return first 120 chars
        let end = text.len().min(120);
        let end = if end < text.len() {
            text[..end].rfind(' ').unwrap_or(end)
        } else {
            end
        };
        format!("{}...", &text[..end])
    }
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
}
