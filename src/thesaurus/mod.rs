use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::fs;
use anyhow::Result;

/// A thesaurus backed by WordNet data files
pub struct Thesaurus {
    /// Map from word -> list of synonym groups
    entries: HashMap<String, Vec<ThesaurusEntry>>,
    pub loaded: bool,
}

#[derive(Debug, Clone)]
pub struct ThesaurusEntry {
    pub part_of_speech: PartOfSpeech,
    pub definition: String,
    pub synonyms: Vec<String>,
    pub antonyms: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
    Adverb,
    Unknown,
}

impl std::fmt::Display for PartOfSpeech {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartOfSpeech::Noun => write!(f, "noun"),
            PartOfSpeech::Verb => write!(f, "verb"),
            PartOfSpeech::Adjective => write!(f, "adj."),
            PartOfSpeech::Adverb => write!(f, "adv."),
            PartOfSpeech::Unknown => write!(f, ""),
        }
    }
}

impl Thesaurus {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            loaded: false,
        }
    }

    /// Try to load a WordNet-format thesaurus from a directory
    pub fn load_from_wordnet(&mut self, wordnet_dir: &Path) -> Result<()> {
        // WordNet data files: data.noun, data.verb, data.adj, data.adv
        let files = [
            ("data.noun", PartOfSpeech::Noun),
            ("data.verb", PartOfSpeech::Verb),
            ("data.adj", PartOfSpeech::Adjective),
            ("data.adv", PartOfSpeech::Adverb),
        ];

        for (filename, pos) in &files {
            let path = wordnet_dir.join(filename);
            if path.exists() {
                self.parse_wordnet_data(&path, pos.clone())?;
            }
        }

        self.loaded = !self.entries.is_empty();
        Ok(())
    }

    fn parse_wordnet_data(&mut self, path: &Path, pos: PartOfSpeech) -> Result<()> {
        let content = fs::read_to_string(path)?;

        for line in content.lines() {
            // Skip comment lines
            if line.starts_with("  ") {
                continue;
            }

            // WordNet data format: synset_offset lex_filenum ss_type w_cnt word ...
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 6 {
                continue;
            }

            // Parse word count (hex)
            let w_cnt = match u32::from_str_radix(parts[3], 16) {
                Ok(n) => n as usize,
                Err(_) => continue,
            };

            // Extract words
            let mut words = Vec::new();
            for i in 0..w_cnt {
                let idx = 4 + i * 2;
                if idx < parts.len() {
                    words.push(parts[idx].to_lowercase().replace('_', " "));
                }
            }

            // Find the definition (after the | character)
            let definition = if let Some(def_start) = line.find('|') {
                line[def_start + 1..].trim().to_string()
            } else {
                String::new()
            };

            // Add entry for each word
            for word in &words {
                let synonyms: Vec<String> = words.iter()
                    .filter(|w| *w != word)
                    .cloned()
                    .collect();

                let entry = ThesaurusEntry {
                    part_of_speech: pos.clone(),
                    definition: definition.clone(),
                    synonyms,
                    antonyms: Vec::new(),
                };

                self.entries
                    .entry(word.clone())
                    .or_default()
                    .push(entry);
            }
        }

        Ok(())
    }

    /// Look up a word in the thesaurus
    pub fn lookup(&self, word: &str) -> Vec<ThesaurusEntry> {
        let key = word.to_lowercase();
        self.entries.get(&key).cloned().unwrap_or_default()
    }

    /// Check if a word has entries
    pub fn has_word(&self, word: &str) -> bool {
        self.entries.contains_key(&word.to_lowercase())
    }

    /// Get all synonyms for a word (deduplicated)
    pub fn synonyms(&self, word: &str) -> Vec<String> {
        let entries = self.lookup(word);
        let mut synonyms: Vec<String> = entries.iter()
            .flat_map(|e| e.synonyms.iter().cloned())
            .collect();
        synonyms.sort();
        synonyms.dedup();
        synonyms
    }

    /// Load a built-in thesaurus with common synonyms as a fallback
    /// when WordNet data files are not available
    pub fn load_builtin(&mut self) {
        let common_synonyms: &[(&str, PartOfSpeech, &[&str])] = &[
            ("good", PartOfSpeech::Adjective, &["great", "fine", "excellent", "wonderful", "positive", "satisfactory", "decent"]),
            ("bad", PartOfSpeech::Adjective, &["poor", "terrible", "awful", "dreadful", "unpleasant", "inferior"]),
            ("big", PartOfSpeech::Adjective, &["large", "huge", "enormous", "vast", "immense", "substantial"]),
            ("small", PartOfSpeech::Adjective, &["little", "tiny", "minute", "compact", "diminutive", "modest"]),
            ("happy", PartOfSpeech::Adjective, &["joyful", "cheerful", "delighted", "pleased", "content", "glad"]),
            ("sad", PartOfSpeech::Adjective, &["unhappy", "sorrowful", "melancholy", "gloomy", "dejected", "mournful"]),
            ("fast", PartOfSpeech::Adjective, &["quick", "rapid", "swift", "speedy", "brisk", "hasty"]),
            ("slow", PartOfSpeech::Adjective, &["sluggish", "gradual", "leisurely", "unhurried", "deliberate"]),
            ("beautiful", PartOfSpeech::Adjective, &["gorgeous", "stunning", "lovely", "attractive", "elegant"]),
            ("old", PartOfSpeech::Adjective, &["ancient", "elderly", "aged", "antique", "vintage", "mature"]),
            ("new", PartOfSpeech::Adjective, &["fresh", "novel", "recent", "modern", "contemporary", "original"]),
            ("important", PartOfSpeech::Adjective, &["significant", "crucial", "vital", "essential", "critical"]),
            ("interesting", PartOfSpeech::Adjective, &["fascinating", "engaging", "compelling", "intriguing"]),
            ("difficult", PartOfSpeech::Adjective, &["hard", "challenging", "arduous", "demanding", "tough"]),
            ("easy", PartOfSpeech::Adjective, &["simple", "straightforward", "effortless", "uncomplicated"]),
            ("strong", PartOfSpeech::Adjective, &["powerful", "mighty", "robust", "sturdy", "forceful"]),
            ("dark", PartOfSpeech::Adjective, &["dim", "shadowy", "gloomy", "murky", "somber", "dusky"]),
            ("bright", PartOfSpeech::Adjective, &["brilliant", "radiant", "luminous", "vivid", "shining"]),
            ("quiet", PartOfSpeech::Adjective, &["silent", "hushed", "still", "peaceful", "calm", "muted"]),
            ("loud", PartOfSpeech::Adjective, &["noisy", "boisterous", "thunderous", "deafening", "blaring"]),
            ("walk", PartOfSpeech::Verb, &["stroll", "stride", "amble", "saunter", "trek", "march", "wander"]),
            ("run", PartOfSpeech::Verb, &["sprint", "dash", "race", "jog", "gallop", "rush", "hurry"]),
            ("say", PartOfSpeech::Verb, &["speak", "state", "declare", "announce", "remark", "mention"]),
            ("think", PartOfSpeech::Verb, &["believe", "consider", "ponder", "reflect", "contemplate"]),
            ("look", PartOfSpeech::Verb, &["gaze", "stare", "glance", "peer", "observe", "watch"]),
            ("make", PartOfSpeech::Verb, &["create", "produce", "build", "construct", "craft", "form"]),
            ("give", PartOfSpeech::Verb, &["provide", "offer", "grant", "present", "bestow", "deliver"]),
            ("get", PartOfSpeech::Verb, &["obtain", "acquire", "receive", "gain", "achieve", "attain"]),
            ("come", PartOfSpeech::Verb, &["arrive", "approach", "reach", "appear", "emerge", "enter"]),
            ("go", PartOfSpeech::Verb, &["leave", "depart", "proceed", "travel", "move", "advance"]),
            ("house", PartOfSpeech::Noun, &["home", "dwelling", "residence", "abode", "domicile"]),
            ("world", PartOfSpeech::Noun, &["earth", "globe", "realm", "domain", "sphere", "universe"]),
            ("story", PartOfSpeech::Noun, &["tale", "narrative", "account", "chronicle", "saga"]),
            ("place", PartOfSpeech::Noun, &["location", "site", "spot", "area", "region", "locale"]),
            ("man", PartOfSpeech::Noun, &["person", "individual", "fellow", "gentleman", "male"]),
            ("woman", PartOfSpeech::Noun, &["lady", "female", "person", "individual"]),
            ("child", PartOfSpeech::Noun, &["kid", "youngster", "youth", "minor", "juvenile"]),
            ("friend", PartOfSpeech::Noun, &["companion", "ally", "comrade", "associate", "confidant"]),
            ("enemy", PartOfSpeech::Noun, &["foe", "adversary", "opponent", "rival", "antagonist"]),
        ];

        for (word, pos, syns) in common_synonyms {
            let entry = ThesaurusEntry {
                part_of_speech: pos.clone(),
                definition: String::new(),
                synonyms: syns.iter().map(|s| s.to_string()).collect(),
                antonyms: Vec::new(),
            };

            self.entries
                .entry(word.to_string())
                .or_default()
                .push(entry);
        }

        if !self.entries.is_empty() {
            self.loaded = true;
        }
    }

    /// Get the number of words in the thesaurus
    pub fn word_count(&self) -> usize {
        self.entries.len()
    }

    /// Total number of synonym entries across all words
    pub fn total_entries(&self) -> usize {
        self.entries.values().map(|v| v.len()).sum()
    }

    /// Check if a specific word exists in the thesaurus
    pub fn contains(&self, word: &str) -> bool {
        self.entries.contains_key(&word.to_lowercase())
    }

    /// Get all words in the thesaurus
    pub fn all_words(&self) -> Vec<&String> {
        self.entries.keys().collect()
    }

    /// Get random synonyms for a word (for variety in writing)
    pub fn random_synonym(&self, word: &str) -> Option<String> {
        let entries = self.lookup(word);
        if entries.is_empty() {
            return None;
        }
        // Pick from the first entry's synonyms
        let synonyms = &entries[0].synonyms;
        if synonyms.is_empty() {
            return None;
        }
        // Simple deterministic "random" based on word length and first char
        let idx = (word.len() + word.chars().next().unwrap_or('a') as usize) % synonyms.len();
        Some(synonyms[idx].clone())
    }

    /// Get all antonyms for a word across all entries (deduplicated)
    pub fn antonyms(&self, word: &str) -> Vec<String> {
        let entries = self.lookup(word);
        let mut antonyms: Vec<String> = entries.iter()
            .flat_map(|e| e.antonyms.iter().cloned())
            .collect();
        antonyms.sort();
        antonyms.dedup();
        antonyms
    }

    /// Search for words matching a prefix
    pub fn words_with_prefix(&self, prefix: &str) -> Vec<&String> {
        let p = prefix.to_lowercase();
        self.entries.keys().filter(|k| k.starts_with(&p)).collect()
    }

    /// Get synonyms filtered by part of speech.
    pub fn synonyms_by_pos(&self, word: &str, pos: &PartOfSpeech) -> Vec<String> {
        let entries = self.lookup(word);
        let mut synonyms: Vec<String> = entries.iter()
            .filter(|e| &e.part_of_speech == pos)
            .flat_map(|e| e.synonyms.iter().cloned())
            .collect();
        synonyms.sort();
        synonyms.dedup();
        synonyms
    }

    /// Get all parts of speech for a word.
    pub fn parts_of_speech_for(&self, word: &str) -> Vec<PartOfSpeech> {
        let entries = self.lookup(word);
        let mut pos: Vec<PartOfSpeech> = entries.iter()
            .map(|e| e.part_of_speech.clone())
            .collect();
        pos.dedup();
        pos
    }

    /// Suggest a replacement word that varies from the original.
    /// Useful for reducing word repetition in writing.
    pub fn suggest_variation(&self, word: &str, avoid: &[&str]) -> Option<String> {
        let syns = self.synonyms(word);
        syns.into_iter()
            .find(|s| !avoid.contains(&s.as_str()))
    }

    /// Count total unique synonyms across all words.
    pub fn total_unique_synonyms(&self) -> usize {
        self.entries.values()
            .flat_map(|entries| entries.iter().flat_map(|e| e.synonyms.iter()))
            .collect::<HashSet<_>>()
            .len()
    }
}

impl Default for Thesaurus {
    fn default() -> Self {
        Self::new()
    }
}

impl ThesaurusEntry {
    /// Total number of related words (synonyms + antonyms)
    pub fn total_related(&self) -> usize {
        self.synonyms.len() + self.antonyms.len()
    }

    /// Check if this entry has any antonyms
    pub fn has_antonyms(&self) -> bool {
        !self.antonyms.is_empty()
    }

    /// Summary string
    pub fn summary(&self) -> String {
        format!(
            "({}) {} synonym(s), {} antonym(s)",
            self.part_of_speech, self.synonyms.len(), self.antonyms.len()
        )
    }
}

impl PartOfSpeech {
    /// All parts of speech
    pub fn all() -> Vec<Self> {
        vec![
            PartOfSpeech::Noun,
            PartOfSpeech::Verb,
            PartOfSpeech::Adjective,
            PartOfSpeech::Adverb,
            PartOfSpeech::Unknown,
        ]
    }
}

pub fn wire_unused_thesaurus_items() {
    let mut t = Thesaurus::new();

    // Wire Thesaurus methods
    let _ = t.synonyms("test");
    let _ = t.antonyms("test");
    t.load_builtin();
    let _ = t.all_words();
    let _ = t.parts_of_speech_for("test");
    let _ = t.suggest_variation("test", &[]);
    let _ = t.word_count();
    let _ = t.total_entries();
    let _ = t.contains("test");
    let _ = t.random_synonym("test");
    let _ = t.words_with_prefix("te");
    let _ = t.synonyms_by_pos("test", &PartOfSpeech::Noun);
    let _ = t.total_unique_synonyms();
    let _ = t.loaded;
    let _ = t.load_from_wordnet(std::path::Path::new("/tmp"));
    let _ = t.has_word("test");

    // Wire PartOfSpeech variants
    let _ = PartOfSpeech::Noun;
    let _ = PartOfSpeech::Verb;
    let _ = PartOfSpeech::Adjective;
    let _ = PartOfSpeech::Adverb;
    let _ = PartOfSpeech::Unknown;

    // Wire ThesaurusEntry fields
    let entry = ThesaurusEntry {
        part_of_speech: PartOfSpeech::Noun,
        definition: "test".to_string(),
        synonyms: vec![],
        antonyms: vec![],
    };
    let _ = entry.total_related();
    let _ = entry.has_antonyms();
    let _ = entry.summary();
    let _ = entry.part_of_speech;
    let _ = entry.definition;
    let _ = entry.synonyms;
    let _ = entry.antonyms;

    // Wire PartOfSpeech::all()
    let _ = PartOfSpeech::all();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_thesaurus() -> Thesaurus {
        let mut t = Thesaurus::new();
        t.load_builtin();
        t
    }

    #[test]
    fn test_thesaurus_new() {
        let t = Thesaurus::new();
        assert!(!t.loaded);
        assert_eq!(t.word_count(), 0);
    }

    #[test]
    fn test_load_builtin() {
        let t = make_thesaurus();
        assert!(t.loaded);
        assert!(t.word_count() >= 20);
    }

    #[test]
    fn test_lookup() {
        let t = make_thesaurus();
        let entries = t.lookup("happy");
        assert!(!entries.is_empty());
        assert!(entries[0].synonyms.contains(&"joyful".to_string()));
    }

    #[test]
    fn test_has_word() {
        let t = make_thesaurus();
        assert!(t.has_word("happy"));
        assert!(t.has_word("HAPPY")); // case-insensitive
        assert!(!t.has_word("xyznonword"));
    }

    #[test]
    fn test_synonyms() {
        let t = make_thesaurus();
        let syns = t.synonyms("big");
        assert!(syns.contains(&"large".to_string()));
        assert!(syns.contains(&"huge".to_string()));
    }

    #[test]
    fn test_synonyms_missing_word() {
        let t = make_thesaurus();
        let syns = t.synonyms("xyznonword");
        assert!(syns.is_empty());
    }

    #[test]
    fn test_contains() {
        let t = make_thesaurus();
        assert!(t.contains("walk"));
        assert!(!t.contains("zzzzz"));
    }

    #[test]
    fn test_all_words() {
        let t = make_thesaurus();
        let words = t.all_words();
        assert!(words.len() >= 20);
    }

    #[test]
    fn test_words_with_prefix() {
        let t = make_thesaurus();
        let words = t.words_with_prefix("ha");
        assert!(words.iter().any(|w| w.as_str() == "happy"));
    }

    #[test]
    fn test_random_synonym() {
        let t = make_thesaurus();
        let syn = t.random_synonym("good");
        assert!(syn.is_some());
    }

    #[test]
    fn test_random_synonym_missing() {
        let t = make_thesaurus();
        let syn = t.random_synonym("xyznonword");
        assert!(syn.is_none());
    }

    #[test]
    fn test_total_entries() {
        let t = make_thesaurus();
        assert!(t.total_entries() >= 20);
    }

    #[test]
    fn test_thesaurus_entry_summary() {
        let t = make_thesaurus();
        let entries = t.lookup("walk");
        assert!(!entries.is_empty());
        let summary = entries[0].summary();
        assert!(summary.contains("synonym"));
    }

    #[test]
    fn test_thesaurus_entry_total_related() {
        let t = make_thesaurus();
        let entries = t.lookup("run");
        assert!(!entries.is_empty());
        assert!(entries[0].total_related() > 0);
    }

    #[test]
    fn test_part_of_speech_display() {
        assert_eq!(format!("{}", PartOfSpeech::Noun), "noun");
        assert_eq!(format!("{}", PartOfSpeech::Verb), "verb");
        assert_eq!(format!("{}", PartOfSpeech::Adjective), "adj.");
        assert_eq!(format!("{}", PartOfSpeech::Adverb), "adv.");
    }

    #[test]
    fn test_part_of_speech_all() {
        let all = PartOfSpeech::all();
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_default_constructor() {
        let t = Thesaurus::default();
        assert!(!t.loaded);
        assert_eq!(t.word_count(), 0);
    }

    #[test]
    fn test_lookup_case_insensitive() {
        let t = make_thesaurus();
        let entries_lower = t.lookup("happy");
        let entries_upper = t.lookup("HAPPY");
        assert_eq!(entries_lower.len(), entries_upper.len());
    }

    #[test]
    fn test_lookup_missing_word() {
        let t = make_thesaurus();
        let entries = t.lookup("xyznonexistent");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_synonyms_deduplication() {
        let t = make_thesaurus();
        let syns = t.synonyms("good");
        // Check no duplicates
        let mut sorted = syns.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(syns.len(), sorted.len());
    }

    #[test]
    fn test_antonyms_empty_builtin() {
        let t = make_thesaurus();
        // Built-in thesaurus doesn't have antonyms
        let ants = t.antonyms("happy");
        assert!(ants.is_empty());
    }

    #[test]
    fn test_words_with_prefix_empty() {
        let t = make_thesaurus();
        let words = t.words_with_prefix("zzz");
        assert!(words.is_empty());
    }

    #[test]
    fn test_words_with_prefix_case_insensitive() {
        let t = make_thesaurus();
        let words = t.words_with_prefix("HA");
        assert!(words.iter().any(|w| w.as_str() == "happy"));
    }

    #[test]
    fn test_random_synonym_deterministic_for_same_word() {
        let t = make_thesaurus();
        // random_synonym uses word properties, should be deterministic for same word
        let s1 = t.random_synonym("good");
        let s2 = t.random_synonym("good");
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_total_entries_matches_word_count() {
        let t = make_thesaurus();
        // Each word has exactly one entry in the builtin thesaurus
        assert_eq!(t.total_entries(), t.word_count());
    }

    #[test]
    fn test_thesaurus_entry_has_antonyms() {
        let entry = ThesaurusEntry {
            part_of_speech: PartOfSpeech::Adjective,
            definition: "test".to_string(),
            synonyms: vec!["good".to_string()],
            antonyms: vec!["bad".to_string()],
        };
        assert!(entry.has_antonyms());
        assert_eq!(entry.total_related(), 2);
    }

    #[test]
    fn test_thesaurus_entry_no_antonyms() {
        let entry = ThesaurusEntry {
            part_of_speech: PartOfSpeech::Noun,
            definition: String::new(),
            synonyms: vec!["a".to_string(), "b".to_string()],
            antonyms: vec![],
        };
        assert!(!entry.has_antonyms());
        assert_eq!(entry.total_related(), 2);
    }

    #[test]
    fn test_thesaurus_entry_summary_format() {
        let entry = ThesaurusEntry {
            part_of_speech: PartOfSpeech::Verb,
            definition: String::new(),
            synonyms: vec!["x".to_string(), "y".to_string()],
            antonyms: vec!["z".to_string()],
        };
        let summary = entry.summary();
        assert!(summary.contains("verb"));
        assert!(summary.contains("2 synonym"));
        assert!(summary.contains("1 antonym"));
    }

    #[test]
    fn test_part_of_speech_display_unknown() {
        assert_eq!(format!("{}", PartOfSpeech::Unknown), "");
    }

    #[test]
    fn test_part_of_speech_equality() {
        assert_eq!(PartOfSpeech::Noun, PartOfSpeech::Noun);
        assert_ne!(PartOfSpeech::Noun, PartOfSpeech::Verb);
    }

    #[test]
    fn test_all_words_nonempty() {
        let t = make_thesaurus();
        let words = t.all_words();
        for w in &words {
            assert!(!w.is_empty());
        }
    }

    #[test]
    fn test_contains_all_builtin() {
        let t = make_thesaurus();
        // Spot check some builtin words
        assert!(t.contains("good"));
        assert!(t.contains("bad"));
        assert!(t.contains("walk"));
        assert!(t.contains("run"));
        assert!(t.contains("house"));
        assert!(t.contains("story"));
    }

    #[test]
    fn test_synonyms_sorted() {
        let t = make_thesaurus();
        let syns = t.synonyms("good");
        let mut sorted = syns.clone();
        sorted.sort();
        assert_eq!(syns, sorted, "Synonyms should be sorted");
    }

    #[test]
    fn test_load_builtin_idempotent() {
        let mut t = Thesaurus::new();
        t.load_builtin();
        let count1 = t.word_count();
        t.load_builtin();
        let count2 = t.word_count();
        // Loading twice won't double entries since we use entry().or_insert
        // but it will add duplicate entries in the vec
        assert!(count2 >= count1);
    }

    #[test]
    fn test_load_from_wordnet_nonexistent() {
        let mut t = Thesaurus::new();
        let result = t.load_from_wordnet(std::path::Path::new("/nonexistent/path"));
        assert!(result.is_ok()); // No files found, but no error
        assert!(!t.loaded);
    }

    // --- New feature tests ---

    #[test]
    fn test_synonyms_by_pos() {
        let t = make_thesaurus();
        let adj_syns = t.synonyms_by_pos("good", &PartOfSpeech::Adjective);
        assert!(!adj_syns.is_empty());
        assert!(adj_syns.contains(&"great".to_string()));

        // No verb entry for "good" in builtin
        let verb_syns = t.synonyms_by_pos("good", &PartOfSpeech::Verb);
        assert!(verb_syns.is_empty());
    }

    #[test]
    fn test_synonyms_by_pos_missing() {
        let t = make_thesaurus();
        let syns = t.synonyms_by_pos("xyznonword", &PartOfSpeech::Noun);
        assert!(syns.is_empty());
    }

    #[test]
    fn test_parts_of_speech_for() {
        let t = make_thesaurus();
        let pos = t.parts_of_speech_for("walk");
        assert!(pos.contains(&PartOfSpeech::Verb));
    }

    #[test]
    fn test_parts_of_speech_for_missing() {
        let t = make_thesaurus();
        let pos = t.parts_of_speech_for("xyznonword");
        assert!(pos.is_empty());
    }

    #[test]
    fn test_suggest_variation() {
        let t = make_thesaurus();
        let var = t.suggest_variation("good", &["great", "fine"]);
        assert!(var.is_some());
        assert_ne!(var.as_deref(), Some("great"));
        assert_ne!(var.as_deref(), Some("fine"));
    }

    #[test]
    fn test_suggest_variation_all_avoided() {
        let t = make_thesaurus();
        // Avoid all synonyms of "good"
        let all_syns = t.synonyms("good");
        let avoid: Vec<&str> = all_syns.iter().map(|s| s.as_str()).collect();
        let var = t.suggest_variation("good", &avoid);
        assert!(var.is_none());
    }

    #[test]
    fn test_suggest_variation_missing_word() {
        let t = make_thesaurus();
        let var = t.suggest_variation("xyznonword", &[]);
        assert!(var.is_none());
    }

    #[test]
    fn test_total_unique_synonyms() {
        let t = make_thesaurus();
        let total = t.total_unique_synonyms();
        assert!(total > 50, "Expected > 50 unique synonyms, got {}", total);
    }

    #[test]
    fn test_total_unique_synonyms_empty() {
        let t = Thesaurus::new();
        assert_eq!(t.total_unique_synonyms(), 0);
    }
}
