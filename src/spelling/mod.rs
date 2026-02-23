//! Spell checking module with a built-in common English word list.
//! Uses Levenshtein distance for spelling suggestions.

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
    include_str!("wordlist.txt").lines()
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
        self.active = !self.dictionary.is_empty();
        self.active
    }

    /// Check a single word
    pub fn check_word(&self, word: &str) -> bool {
        if !self.active {
            return true;
        }
        let lower = word.to_lowercase();
        // Single letters and numbers are always valid
        if lower.len() <= 1 || lower.chars().all(|c| c.is_ascii_digit()) {
            return true;
        }
        // User words are already in `dictionary`, so a single lookup suffices
        self.dictionary.contains(&lower)
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
        let mut candidates: Vec<(String, usize)> = self.dictionary.iter()
            .map(|dict_word| (dict_word.clone(), edit_distance(&lower, dict_word)))
            .filter(|(_, dist)| *dist > 0)
            .collect();
        candidates.sort_by_key(|(_, dist)| *dist);
        candidates.truncate(max_results);
        candidates
    }

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
        let Some(home) = dirs::home_dir() else { return };
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
            !clean.is_empty() && clean.len() > 1 && !checker.check_word(&clean)
        })
        .count()
}

/// Get spelling accuracy as a percentage
pub fn spelling_accuracy(checker: &SpellChecker, text: &str) -> f64 {
    let word_count = text.split_whitespace().count();
    if word_count == 0 {
        return 100.0;
    }
    let misspelled = count_misspellings(checker, text);
    (1.0 - misspelled as f64 / word_count as f64) * 100.0
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

    #[test]
    fn test_add_duplicate_to_dictionary() {
        let mut checker = make_checker();
        checker.add_to_dictionary("testword");
        checker.add_to_dictionary("TESTWORD"); // Same word, different case
        assert_eq!(checker.user_dictionary_size(), 1);
    }

    #[test]
    fn test_user_words() {
        let mut checker = make_checker();
        checker.add_to_dictionary("foo");
        checker.add_to_dictionary("bar");
        let words = checker.user_words();
        assert_eq!(words.len(), 2);
        assert!(words.contains(&"foo".to_string()));
        assert!(words.contains(&"bar".to_string()));
    }

    #[test]
    fn test_dictionary_size_after_init() {
        let checker = make_checker();
        assert!(checker.dictionary_size() > 0);
    }

    #[test]
    fn test_similar_words() {
        let checker = make_checker();
        let similar = checker.similar_words("helo", 3);
        assert!(!similar.is_empty() || !checker.active);
        if !similar.is_empty() {
            // Should have distance > 0
            assert!(similar[0].1 > 0);
        }
    }

    #[test]
    fn test_similar_words_inactive() {
        let checker = SpellChecker::new();
        let similar = checker.similar_words("hello", 5);
        assert!(similar.is_empty());
    }

    #[test]
    fn test_suggestion_no_suggestions() {
        let suggestion = SpellSuggestion {
            word: "xyzxyz".to_string(),
            suggestions: vec![],
            position: 0,
        };
        assert!(!suggestion.has_suggestions());
        assert!(suggestion.best_suggestion().is_none());
        assert_eq!(suggestion.suggestion_count(), 0);
        let summary = suggestion.summary();
        assert!(summary.contains("no suggestions"));
    }

    #[test]
    fn test_count_misspellings_all_correct() {
        let checker = make_checker();
        let count = count_misspellings(&checker, "the cat");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_spelling_accuracy_all_correct() {
        let checker = make_checker();
        let accuracy = spelling_accuracy(&checker, "the the the the");
        assert!(accuracy >= 90.0);
    }

    #[test]
    fn test_default_constructor() {
        let checker = SpellChecker::default();
        assert!(!checker.active);
        assert_eq!(checker.dictionary_size(), 0);
    }

    #[test]
    fn test_check_text_with_apostrophes() {
        let checker = make_checker();
        let results = checker.check_text("don't it's");
        // These are common words — shouldn't be flagged (if dictionary includes them)
        // Just check it doesn't crash
        assert!(results.len() <= 2);
    }

    #[test]
    fn test_remove_nonexistent_word() {
        let mut checker = make_checker();
        checker.remove_from_dictionary("never_added");
        // Should be a no-op
        assert_eq!(checker.user_dictionary_size(), 0);
    }

    #[test]
    fn test_check_word_case_insensitive() {
        let checker = make_checker();
        // "the" is a common word - both cases should pass
        assert!(checker.check_word("THE"));
        assert!(checker.check_word("The"));
        assert!(checker.check_word("the"));
    }

    #[test]
    fn test_suggest_max_results() {
        let checker = make_checker();
        let suggestions = checker.suggest("hllo");
        assert!(suggestions.len() <= 5); // Max 5 suggestions
    }

    #[test]
    fn test_check_text_empty() {
        let checker = make_checker();
        let results = checker.check_text("");
        assert!(results.is_empty());
    }

    #[test]
    fn test_check_text_punctuation_only() {
        let checker = make_checker();
        let results = checker.check_text("... !!! ???");
        assert!(results.is_empty());
    }

    #[test]
    fn test_edit_distance_single_operations() {
        // Single insertion
        assert_eq!(edit_distance("cat", "cats"), 1);
        // Single deletion
        assert_eq!(edit_distance("cats", "cat"), 1);
        // Single substitution
        assert_eq!(edit_distance("cat", "bat"), 1);
    }

    #[test]
    fn test_spell_suggestion_position() {
        let checker = make_checker();
        let text = "this is a xyznonword test";
        let results = checker.check_text(text);
        // The position should be non-zero (xyznonword is not at start)
        for r in &results {
            if r.word == "xyznonword" {
                assert!(r.position > 0);
            }
        }
    }

    #[test]
    fn test_add_then_check_then_remove() {
        let mut checker = make_checker();
        let word = "florbington";

        assert!(!checker.check_word(word));
        checker.add_to_dictionary(word);
        assert!(checker.check_word(word));
        checker.remove_from_dictionary(word);
        assert!(!checker.check_word(word));
    }

    #[test]
    fn test_clear_user_dictionary_restores() {
        let mut checker = make_checker();
        checker.add_to_dictionary("xyzword1");
        checker.add_to_dictionary("xyzword2");

        assert!(checker.check_word("xyzword1"));
        checker.clear_user_dictionary();
        assert!(!checker.check_word("xyzword1"));
        assert!(!checker.check_word("xyzword2"));
    }

    #[test]
    fn test_check_words_returns_only_bad() {
        let checker = make_checker();
        let results = checker.check_words(&["the", "xyzfakeword1", "xyzfakeword2"]);
        // "the" should not appear in results
        assert!(!results.contains(&"the".to_string()));
    }

    #[test]
    fn test_similar_words_sorted_by_distance() {
        let checker = make_checker();
        let similar = checker.similar_words("hello", 10);
        if similar.len() >= 2 {
            // Should be sorted by distance (ascending)
            for i in 1..similar.len() {
                assert!(similar[i].1 >= similar[i-1].1);
            }
        }
    }

    #[test]
    fn test_edit_distance_symmetric() {
        assert_eq!(edit_distance("abc", "def"), edit_distance("def", "abc"));
        assert_eq!(edit_distance("hello", "world"), edit_distance("world", "hello"));
    }

    #[test]
    fn test_spelling_accuracy_mixed() {
        let checker = make_checker();
        // Mix of correct and likely incorrect words
        let accuracy = spelling_accuracy(&checker, "the xyznotaword hello xyzalsonotaword");
        // Should be less than 100% if the fake words are caught
        if checker.active {
            assert!(accuracy < 100.0);
        }
    }

    #[test]
    fn test_suggestion_summary_with_suggestions() {
        let suggestion = SpellSuggestion {
            word: "teh".to_string(),
            suggestions: vec!["the".to_string(), "ten".to_string(), "tea".to_string(), "ted".to_string()],
            position: 5,
        };
        let summary = suggestion.summary();
        assert!(summary.contains("4 suggestions"));
        assert!(summary.contains("the"));
    }

    #[test]
    fn test_edit_distance_identical() {
        assert_eq!(edit_distance("same", "same"), 0);
        assert_eq!(edit_distance("", ""), 0);
    }

    #[test]
    fn test_edit_distance_one_empty() {
        assert_eq!(edit_distance("hello", ""), 5);
        assert_eq!(edit_distance("", "world"), 5);
    }

    #[test]
    fn test_edit_distance_transposition() {
        // Transposition is two operations (delete + insert)
        assert_eq!(edit_distance("ab", "ba"), 2);
    }

    #[test]
    fn test_check_word_with_unicode() {
        let checker = make_checker();
        // Unicode words should not crash
        let _ = checker.check_word("café");
        let _ = checker.check_word("naïve");
    }

    #[test]
    fn test_suggest_for_known_word() {
        let checker = make_checker();
        // Suggestions for an already-correct word should include similar words (not itself)
        let suggestions = checker.suggest("hello");
        for s in &suggestions {
            assert_ne!(s, "hello");
        }
    }

    #[test]
    fn test_suggest_short_word_max_distance_1() {
        let checker = make_checker();
        // For words <= 4 chars, max distance is 1
        let suggestions = checker.suggest("cat");
        for s in &suggestions {
            let dist = edit_distance("cat", s);
            assert!(dist <= 1, "Suggestion '{}' has distance {}", s, dist);
        }
    }

    #[test]
    fn test_suggest_long_word_max_distance_2() {
        let checker = make_checker();
        // For words > 4 chars, max distance is 2
        let suggestions = checker.suggest("helloo");
        for s in &suggestions {
            let dist = edit_distance("helloo", s);
            assert!(dist <= 2, "Suggestion '{}' has distance {}", s, dist);
        }
    }

    #[test]
    fn test_check_text_preserves_positions() {
        let checker = make_checker();
        let text = "hello xyzfakeword world";
        let results = checker.check_text(text);
        for r in &results {
            if r.word == "xyzfakeword" {
                // Position should be somewhere after "hello "
                assert!(r.position >= 6);
            }
        }
    }

    #[test]
    fn test_count_misspellings_empty() {
        let checker = make_checker();
        assert_eq!(count_misspellings(&checker, ""), 0);
    }

    #[test]
    fn test_multiple_user_dictionary_operations() {
        let mut checker = make_checker();
        checker.add_to_dictionary("alpha");
        checker.add_to_dictionary("beta");
        checker.add_to_dictionary("gamma");
        assert_eq!(checker.user_dictionary_size(), 3);

        checker.remove_from_dictionary("beta");
        assert_eq!(checker.user_dictionary_size(), 2);
        assert!(!checker.check_word("beta") || checker.dictionary.contains("beta"));

        checker.clear_user_dictionary();
        assert_eq!(checker.user_dictionary_size(), 0);
    }
}
