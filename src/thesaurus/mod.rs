use std::collections::HashMap;
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
                    .or_insert_with(Vec::new)
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
}

impl Default for Thesaurus {
    fn default() -> Self {
        Self::new()
    }
}

impl Thesaurus {
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
                .or_insert_with(Vec::new)
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

    /// Get all antonyms for a word across all entries
    pub fn antonyms(&self, word: &str) -> Vec<String> {
        let entries = self.lookup(word);
        let mut antonyms = Vec::new();
        for entry in entries {
            for ant in &entry.antonyms {
                if !antonyms.contains(ant) {
                    antonyms.push(ant.clone());
                }
            }
        }
        antonyms
    }

    /// Search for words matching a prefix
    pub fn words_with_prefix(&self, prefix: &str) -> Vec<&String> {
        let p = prefix.to_lowercase();
        self.entries.keys().filter(|k| k.starts_with(&p)).collect()
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
