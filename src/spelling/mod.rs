/// Spell checking module using system dictionaries or bundled Hunspell data.
/// Currently provides a trait-based interface that can be backed by hunspell-rs
/// when dictionary files are available.

/// A spell checking suggestion
#[derive(Debug, Clone)]
pub struct SpellSuggestion {
    pub word: String,
    pub suggestions: Vec<String>,
    pub position: usize,
}

/// Spell checker interface
pub struct SpellChecker {
    /// Words added to the user dictionary
    user_dictionary: Vec<String>,
    /// Whether spell checking is active
    pub active: bool,
}

impl SpellChecker {
    pub fn new() -> Self {
        Self {
            user_dictionary: Vec::new(),
            active: false,
        }
    }

    /// Try to initialize with system hunspell dictionaries
    pub fn try_init(&mut self) -> bool {
        // Look for system dictionaries in common locations
        let dict_paths = [
            "/usr/share/hunspell",
            "/usr/share/myspell",
            "/usr/local/share/hunspell",
        ];

        for _path in &dict_paths {
            // In a full implementation, we'd load the .aff and .dic files
            // For now, we note that dictionaries need to be provided
        }

        self.active = false;
        self.active
    }

    /// Check a single word (basic implementation)
    pub fn check_word(&self, _word: &str) -> bool {
        if !self.active {
            return true; // If spell checker not active, all words pass
        }
        true
    }

    /// Get suggestions for a misspelled word
    pub fn suggest(&self, _word: &str) -> Vec<String> {
        Vec::new()
    }

    /// Check an entire text and return misspelled words with positions
    pub fn check_text(&self, text: &str) -> Vec<SpellSuggestion> {
        if !self.active {
            return Vec::new();
        }

        let mut results = Vec::new();
        let mut pos = 0;

        for word in text.split(|c: char| !c.is_alphabetic()) {
            if !word.is_empty() && !self.check_word(word) && !self.is_in_user_dict(word) {
                results.push(SpellSuggestion {
                    word: word.to_string(),
                    suggestions: self.suggest(word),
                    position: pos,
                });
            }
            pos += word.len() + 1;
        }

        results
    }

    /// Add a word to the user dictionary
    pub fn add_to_dictionary(&mut self, word: &str) {
        let lower = word.to_lowercase();
        if !self.user_dictionary.contains(&lower) {
            self.user_dictionary.push(lower);
        }
    }

    /// Check if a word is in the user dictionary
    fn is_in_user_dict(&self, word: &str) -> bool {
        self.user_dictionary.contains(&word.to_lowercase())
    }
}

impl Default for SpellChecker {
    fn default() -> Self {
        Self::new()
    }
}
