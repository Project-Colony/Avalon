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
