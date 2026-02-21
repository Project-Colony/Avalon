/// Spell checking module with a built-in common English word list.
/// Uses Levenshtein distance for spelling suggestions.

use std::collections::HashSet;

/// A spell checking suggestion
#[derive(Debug, Clone)]
pub struct SpellSuggestion {
    pub word: String,
    pub suggestions: Vec<String>,
    pub position: usize,
}

/// Spell checker with built-in dictionary
pub struct SpellChecker {
    /// Core dictionary words
    dictionary: HashSet<String>,
    /// Words added to the user dictionary
    user_dictionary: Vec<String>,
    /// Whether spell checking is active
    pub active: bool,
}

/// Built-in common English words (~3000 most frequent)
fn built_in_dictionary() -> HashSet<String> {
    // Common English words - this covers the vast majority of everyday writing
    let words = include_str!("wordlist.txt");
    words.lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|w| w.trim().to_lowercase())
        .collect()
}

/// Calculate Levenshtein edit distance between two strings
fn edit_distance(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();
    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }

    let mut prev: Vec<usize> = (0..=b_len).collect();
    let mut curr = vec![0; b_len + 1];

    for (i, ca) in a.chars().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] = (prev[j] + cost)
                .min(prev[j + 1] + 1)
                .min(curr[j] + 1);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b_len]
}

impl SpellChecker {
    pub fn new() -> Self {
        Self {
            dictionary: HashSet::new(),
            user_dictionary: Vec::new(),
            active: false,
        }
    }

    /// Initialize with built-in dictionary
    pub fn try_init(&mut self) -> bool {
        self.dictionary = built_in_dictionary();
        if !self.dictionary.is_empty() {
            self.active = true;
        }
        self.active
    }

    /// Check a single word
    pub fn check_word(&self, word: &str) -> bool {
        if !self.active {
            return true;
        }
        let lower = word.to_lowercase();
        // Single letters are always valid
        if lower.len() <= 1 {
            return true;
        }
        // Numbers are valid
        if lower.chars().all(|c| c.is_ascii_digit()) {
            return true;
        }
        self.dictionary.contains(&lower) || self.is_in_user_dict(word)
    }

    /// Get suggestions for a misspelled word using edit distance
    pub fn suggest(&self, word: &str) -> Vec<String> {
        if !self.active || self.dictionary.is_empty() {
            return Vec::new();
        }

        let lower = word.to_lowercase();
        let max_distance = if lower.len() <= 4 { 1 } else { 2 };

        let mut candidates: Vec<(String, usize)> = self.dictionary.iter()
            .filter(|dict_word| {
                // Quick length-based pre-filter
                let len_diff = (dict_word.len() as isize - lower.len() as isize).unsigned_abs();
                len_diff <= max_distance
            })
            .filter_map(|dict_word| {
                let dist = edit_distance(&lower, dict_word);
                if dist <= max_distance && dist > 0 {
                    Some((dict_word.clone(), dist))
                } else {
                    None
                }
            })
            .collect();

        candidates.sort_by_key(|(_, dist)| *dist);
        candidates.truncate(5);
        candidates.into_iter().map(|(w, _)| w).collect()
    }

    /// Check an entire text and return misspelled words with positions
    pub fn check_text(&self, text: &str) -> Vec<SpellSuggestion> {
        if !self.active {
            return Vec::new();
        }

        let mut results = Vec::new();
        let mut pos = 0;

        for segment in text.split(|c: char| !c.is_alphabetic() && c != '\'') {
            if !segment.is_empty() {
                // Strip leading/trailing apostrophes
                let word = segment.trim_matches('\'');
                if !word.is_empty() && word.len() > 1 && !self.check_word(word) {
                    results.push(SpellSuggestion {
                        word: word.to_string(),
                        suggestions: self.suggest(word),
                        position: pos,
                    });
                }
            }
            pos += segment.len() + 1;
        }

        results
    }

    /// Add a word to the user dictionary
    pub fn add_to_dictionary(&mut self, word: &str) {
        let lower = word.to_lowercase();
        if !self.user_dictionary.contains(&lower) {
            self.user_dictionary.push(lower.clone());
            self.dictionary.insert(lower);
        }
    }

    /// Check if a word is in the user dictionary
    fn is_in_user_dict(&self, word: &str) -> bool {
        self.user_dictionary.contains(&word.to_lowercase())
    }

    /// Get count of words in dictionary
    pub fn dictionary_size(&self) -> usize {
        self.dictionary.len()
    }

    /// Get the user dictionary words
    pub fn user_words(&self) -> &[String] {
        &self.user_dictionary
    }

    /// User dictionary size
    pub fn user_dictionary_size(&self) -> usize {
        self.user_dictionary.len()
    }

    /// Remove a word from the user dictionary
    pub fn remove_from_dictionary(&mut self, word: &str) {
        let lower = word.to_lowercase();
        self.user_dictionary.retain(|w| *w != lower);
        self.dictionary.remove(&lower);
    }

    /// Clear the user dictionary
    pub fn clear_user_dictionary(&mut self) {
        for word in &self.user_dictionary {
            self.dictionary.remove(word);
        }
        self.user_dictionary.clear();
    }

    /// Check multiple words, returning only misspelled ones
    pub fn check_words(&self, words: &[&str]) -> Vec<String> {
        words
            .iter()
            .filter(|w| !self.check_word(w))
            .map(|w| w.to_string())
            .collect()
    }

    /// Get the top N most similar words to a given word
    pub fn similar_words(&self, word: &str, max_results: usize) -> Vec<(String, usize)> {
        if !self.active || self.dictionary.is_empty() {
            return Vec::new();
        }
        let lower = word.to_lowercase();
        let mut candidates: Vec<(String, usize)> = self
            .dictionary
            .iter()
            .map(|dict_word| {
                let dist = edit_distance(&lower, dict_word);
                (dict_word.clone(), dist)
            })
            .filter(|(_, dist)| *dist > 0)
            .collect();
        candidates.sort_by_key(|(_, dist)| *dist);
        candidates.truncate(max_results);
        candidates
    }
}

impl Default for SpellChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl SpellSuggestion {
    /// Whether this misspelling has any suggestions
    pub fn has_suggestions(&self) -> bool {
        !self.suggestions.is_empty()
    }

    /// Get the best (first) suggestion
    pub fn best_suggestion(&self) -> Option<&str> {
        self.suggestions.first().map(|s| s.as_str())
    }
}
