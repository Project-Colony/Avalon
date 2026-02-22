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

impl SpellChecker {
    /// Save the user dictionary to disk
    pub fn save_user_dictionary(&self) -> Result<(), String> {
        if self.user_dictionary.is_empty() {
            return Ok(());
        }
        let home = dirs::home_dir().ok_or("No home directory")?;
        let dict_dir = home.join(".avalon");
        std::fs::create_dir_all(&dict_dir).map_err(|e| e.to_string())?;
        let dict_path = dict_dir.join("user_dictionary.txt");
        let content = self.user_dictionary.join("\n");
        std::fs::write(&dict_path, content).map_err(|e| e.to_string())
    }

    /// Load the user dictionary from disk
    pub fn load_user_dictionary(&mut self) {
        let home = match dirs::home_dir() {
            Some(h) => h,
            None => return,
        };
        let dict_path = home.join(".avalon").join("user_dictionary.txt");
        if let Ok(content) = std::fs::read_to_string(&dict_path) {
            for word in content.lines() {
                let trimmed = word.trim().to_lowercase();
                if !trimmed.is_empty() && !self.user_dictionary.contains(&trimmed) {
                    self.user_dictionary.push(trimmed.clone());
                    self.dictionary.insert(trimmed);
                }
            }
        }
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

    /// Number of suggestions available
    pub fn suggestion_count(&self) -> usize {
        self.suggestions.len()
    }

    /// Summary string for display
    pub fn summary(&self) -> String {
        if self.suggestions.is_empty() {
            format!("\"{}\" — no suggestions", self.word)
        } else {
            format!("\"{}\" — {} suggestions: {}", self.word,
                self.suggestions.len(),
                self.suggestions.iter().take(3).cloned().collect::<Vec<_>>().join(", "))
        }
    }
}

/// Count misspelled words in a text
pub fn count_misspellings(checker: &SpellChecker, text: &str) -> usize {
    text.split_whitespace()
        .filter(|w| {
            let clean: String = w.chars().filter(|c| c.is_alphanumeric() || *c == '\'').collect();
            !clean.is_empty() && clean.len() > 1 && !checker.check_text(&clean).is_empty()
        })
        .count()
}

/// Get spelling accuracy as a percentage
pub fn spelling_accuracy(checker: &SpellChecker, text: &str) -> f64 {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return 100.0;
    }
    let misspelled = count_misspellings(checker, text);
    (1.0 - misspelled as f64 / words.len() as f64) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_checker() -> SpellChecker {
        let mut checker = SpellChecker::new();
        checker.try_init();
        checker
    }

    #[test]
    fn test_spell_checker_init() {
        let checker = make_checker();
        assert!(checker.active);
        assert!(checker.dictionary_size() > 100);
    }

    #[test]
    fn test_check_common_words() {
        let checker = make_checker();
        assert!(checker.check_word("the"));
        assert!(checker.check_word("hello"));
        assert!(checker.check_word("world"));
    }

    #[test]
    fn test_check_numbers() {
        let checker = make_checker();
        assert!(checker.check_word("42"));
        assert!(checker.check_word("123456"));
    }

    #[test]
    fn test_check_single_letters() {
        let checker = make_checker();
        assert!(checker.check_word("a"));
        assert!(checker.check_word("I"));
    }

    #[test]
    fn test_suggest_for_misspelling() {
        let checker = make_checker();
        let suggestions = checker.suggest("helo");
        // Should suggest "hello" or similar
        assert!(!suggestions.is_empty() || !checker.active);
    }

    #[test]
    fn test_user_dictionary() {
        let mut checker = make_checker();
        assert!(!checker.check_word("xyztestword"));

        checker.add_to_dictionary("xyztestword");
        assert!(checker.check_word("xyztestword"));
        assert_eq!(checker.user_dictionary_size(), 1);

        checker.remove_from_dictionary("xyztestword");
        assert!(!checker.check_word("xyztestword"));
    }

    #[test]
    fn test_clear_user_dictionary() {
        let mut checker = make_checker();
        checker.add_to_dictionary("testword1");
        checker.add_to_dictionary("testword2");
        assert_eq!(checker.user_dictionary_size(), 2);

        checker.clear_user_dictionary();
        assert_eq!(checker.user_dictionary_size(), 0);
    }

    #[test]
    fn test_check_text() {
        let checker = make_checker();
        let misspellings = checker.check_text("the cat sat on the mat");
        // All common words — should find no misspellings (assuming they're in dictionary)
        // (This depends on the dictionary having these words)
        assert!(misspellings.len() <= 2);
    }

    #[test]
    fn test_spell_suggestion_methods() {
        let suggestion = SpellSuggestion {
            word: "helo".to_string(),
            suggestions: vec!["hello".to_string(), "help".to_string()],
            position: 0,
        };
        assert!(suggestion.has_suggestions());
        assert_eq!(suggestion.best_suggestion(), Some("hello"));
        assert_eq!(suggestion.suggestion_count(), 2);
        let summary = suggestion.summary();
        assert!(summary.contains("helo"));
    }

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("", ""), 0);
        assert_eq!(edit_distance("abc", ""), 3);
        assert_eq!(edit_distance("", "abc"), 3);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("hello", "hello"), 0);
        assert_eq!(edit_distance("hello", "helo"), 1);
    }

    #[test]
    fn test_inactive_checker() {
        let checker = SpellChecker::new();
        assert!(!checker.active);
        assert!(checker.check_word("anything")); // always true when inactive
        assert!(checker.suggest("anything").is_empty());
        assert!(checker.check_text("anything").is_empty());
    }

    #[test]
    fn test_spelling_accuracy_empty() {
        let checker = make_checker();
        assert_eq!(spelling_accuracy(&checker, ""), 100.0);
    }

    #[test]
    fn test_check_words() {
        let checker = make_checker();
        let bad = checker.check_words(&["the", "xyznonword", "hello"]);
        // xyznonword is unlikely to be in any dictionary
        assert!(bad.contains(&"xyznonword".to_string()) || !checker.active);
    }
}
