use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 1. Readability Analysis
// ---------------------------------------------------------------------------

/// Readability level classification based on Flesch Reading Ease score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadabilityLevel {
    VeryEasy,
    Easy,
    FairlyEasy,
    Standard,
    FairlyDifficult,
    Difficult,
    VeryDifficult,
}

impl ReadabilityLevel {
    /// Classify a Flesch Reading Ease score into a readability level.
    pub fn from_score(score: f64) -> Self {
        if score >= 90.0 {
            ReadabilityLevel::VeryEasy
        } else if score >= 80.0 {
            ReadabilityLevel::Easy
        } else if score >= 70.0 {
            ReadabilityLevel::FairlyEasy
        } else if score >= 60.0 {
            ReadabilityLevel::Standard
        } else if score >= 50.0 {
            ReadabilityLevel::FairlyDifficult
        } else if score >= 30.0 {
            ReadabilityLevel::Difficult
        } else {
            ReadabilityLevel::VeryDifficult
        }
    }

    /// Human-readable label for the readability level.
    pub fn label(&self) -> &str {
        match self {
            ReadabilityLevel::VeryEasy => "Very Easy",
            ReadabilityLevel::Easy => "Easy",
            ReadabilityLevel::FairlyEasy => "Fairly Easy",
            ReadabilityLevel::Standard => "Standard",
            ReadabilityLevel::FairlyDifficult => "Fairly Difficult",
            ReadabilityLevel::Difficult => "Difficult",
            ReadabilityLevel::VeryDifficult => "Very Difficult",
        }
    }
}

/// Readability metrics computed from a block of text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadabilityMetrics {
    pub sentence_count: usize,
    pub word_count: usize,
    pub syllable_count: usize,
    pub avg_sentence_length: f64,
    pub avg_word_length: f64,
    pub flesch_reading_ease: f64,
    pub flesch_kincaid_grade: f64,
}

impl Default for ReadabilityMetrics {
    fn default() -> Self {
        Self {
            sentence_count: 0,
            word_count: 0,
            syllable_count: 0,
            avg_sentence_length: 0.0,
            avg_word_length: 0.0,
            flesch_reading_ease: 0.0,
            flesch_kincaid_grade: 0.0,
        }
    }
}

impl ReadabilityMetrics {
    /// Get the readability level for these metrics.
    pub fn level(&self) -> ReadabilityLevel {
        ReadabilityLevel::from_score(self.flesch_reading_ease)
    }
}

/// Estimate the number of syllables in an English word.
///
/// Uses a vowel-group heuristic with adjustments for silent-e, common
/// suffixes, and other English spelling patterns. This is an approximation
/// and will not be perfectly accurate for every word.
pub fn count_syllables(word: &str) -> usize {
    let lower = word
        .trim_matches(|c: char| !c.is_alphabetic())
        .to_lowercase();
    if lower.is_empty() {
        return 1;
    }

    let vowels = "aeiouy";
    let chars: Vec<char> = lower.chars().collect();
    let mut count: usize = 0;
    let mut prev_vowel = false;

    for &ch in &chars {
        if vowels.contains(ch) {
            if !prev_vowel {
                count += 1;
            }
            prev_vowel = true;
        } else {
            prev_vowel = false;
        }
    }

    // Handle silent 'e' at end of word
    if lower.ends_with('e') && count > 1 {
        count -= 1;
    }

    // Common suffixes that add a syllable
    if lower.ends_with("le") && chars.len() > 2 {
        let before_le = chars[chars.len() - 3];
        if !vowels.contains(before_le) {
            count += 1;
        }
    }

    count.max(1)
}

/// Analyze the readability of a text passage.
pub fn analyze_readability(text: &str) -> ReadabilityMetrics {
    if text.trim().is_empty() {
        return ReadabilityMetrics::default();
    }

    let words: Vec<&str> = text.split_whitespace().collect();
    let word_count = words.len();
    if word_count == 0 {
        return ReadabilityMetrics::default();
    }

    let sentence_count = text
        .chars()
        .filter(|c| *c == '.' || *c == '!' || *c == '?')
        .count()
        .max(1);

    let syllable_count: usize = words.iter().map(|w| count_syllables(w)).sum();

    let total_char_len: usize = words
        .iter()
        .map(|w| w.chars().filter(|c| c.is_alphabetic()).count())
        .sum();

    let avg_sentence_length = word_count as f64 / sentence_count as f64;
    let avg_word_length = total_char_len as f64 / word_count as f64;
    let avg_syllables_per_word = syllable_count as f64 / word_count as f64;

    let flesch_reading_ease =
        206.835 - 1.015 * avg_sentence_length - 84.6 * avg_syllables_per_word;
    let flesch_kincaid_grade =
        (0.39 * avg_sentence_length + 11.8 * avg_syllables_per_word - 15.59).max(0.0);

    ReadabilityMetrics {
        sentence_count,
        word_count,
        syllable_count,
        avg_sentence_length,
        avg_word_length,
        flesch_reading_ease,
        flesch_kincaid_grade,
    }
}

// ---------------------------------------------------------------------------
// 2. Passive Voice Detection
// ---------------------------------------------------------------------------

/// A match where passive voice was detected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassiveVoiceMatch {
    /// Byte position of the match in the original text.
    pub position: usize,
    /// Byte length of the matched substring.
    pub length: usize,
    /// The matched text itself.
    pub text: String,
}

/// Detect passive voice constructions in text.
///
/// Looks for patterns like "was/were/been/being/is/are/am + past participle".
/// Past participles are approximated as words ending in "-ed", "-en", "-wn",
/// "-ne", or "-nt" (common English patterns).
pub fn detect_passive_voice(text: &str) -> Vec<PassiveVoiceMatch> {
    let re = Regex::new(
        r"(?i)\b(was|were|been|being|is|are|am)\s+([\w]+(?:ed|en|wn|ne|nt))\b",
    )
    .expect("passive voice regex must compile");

    let mut results = Vec::new();
    for cap in re.captures_iter(text) {
        if let Some(m) = cap.get(0) {
            let matched_text = m.as_str().to_string();
            // Filter out false positives by checking that the second word is
            // not a common non-participle word.
            let participle = cap.get(2).map(|c| c.as_str().to_lowercase()).unwrap_or_default();
            if is_false_positive_participle(&participle) {
                continue;
            }
            results.push(PassiveVoiceMatch {
                position: m.start(),
                length: m.len(),
                text: matched_text,
            });
        }
    }
    results
}

/// Filter out words that match the participle pattern but are not actually
/// past participles.
fn is_false_positive_participle(word: &str) -> bool {
    const FALSE_POSITIVES: &[&str] = &[
        "been", "when", "then", "ten", "men", "women", "often", "even",
        "sudden", "garden", "children", "given", "open", "broken",
        "between", "happen", "listen", "golden", "kitten", "written",
        "chicken", "heaven", "eleven", "dozen", "linen", "oven",
    ];
    FALSE_POSITIVES.contains(&word.to_lowercase().as_str())
}

// ---------------------------------------------------------------------------
// 3. Repeated Word Detection
// ---------------------------------------------------------------------------

/// A warning about a word repeated too many times in close proximity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepetitionWarning {
    /// The repeated word (lowercased).
    pub word: String,
    /// How many times the word appears within the detection window.
    pub count: usize,
    /// Byte positions of each occurrence.
    pub positions: Vec<usize>,
}

/// Detect words that are repeated within a certain word distance.
///
/// - `min_length`: ignore words shorter than this many characters.
/// - `max_distance`: maximum number of words apart two occurrences can be
///    to count as a close repetition.
pub fn detect_repetitions(
    text: &str,
    min_length: usize,
    max_distance: usize,
) -> Vec<RepetitionWarning> {
    // Common English words that are naturally repeated and should be ignored.
    const STOP_WORDS: &[&str] = &[
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
        "of", "with", "by", "from", "is", "it", "was", "were", "are", "be",
        "been", "being", "have", "has", "had", "do", "does", "did", "will",
        "would", "shall", "should", "may", "might", "must", "can", "could",
        "that", "this", "these", "those", "he", "she", "they", "we", "you",
        "his", "her", "its", "our", "your", "their", "not", "no", "so",
        "if", "then", "than", "as", "into", "about",
    ];

    let word_re = Regex::new(r"[a-zA-Z]+").expect("word regex must compile");

    // Collect (word_lowercase, byte_position, word_index)
    let mut word_entries: Vec<(String, usize, usize)> = Vec::new();
    for (idx, m) in word_re.find_iter(text).enumerate() {
        let w = m.as_str().to_lowercase();
        if w.len() >= min_length && !STOP_WORDS.contains(&w.as_str()) {
            word_entries.push((w, m.start(), idx));
        }
    }

    // Group positions by word
    let mut word_map: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
    for (word, pos, idx) in &word_entries {
        word_map
            .entry(word.clone())
            .or_default()
            .push((*pos, *idx));
    }

    let mut warnings = Vec::new();
    for (word, occurrences) in &word_map {
        if occurrences.len() < 2 {
            continue;
        }
        // Check if any pair of occurrences is within max_distance words
        let mut close_positions: Vec<usize> = Vec::new();
        for i in 0..occurrences.len() {
            for j in (i + 1)..occurrences.len() {
                let dist = occurrences[j].1.abs_diff(occurrences[i].1);
                if dist <= max_distance {
                    if !close_positions.contains(&occurrences[i].0) {
                        close_positions.push(occurrences[i].0);
                    }
                    if !close_positions.contains(&occurrences[j].0) {
                        close_positions.push(occurrences[j].0);
                    }
                }
            }
        }
        if close_positions.len() >= 2 {
            close_positions.sort();
            warnings.push(RepetitionWarning {
                word: word.clone(),
                count: close_positions.len(),
                positions: close_positions,
            });
        }
    }

    warnings.sort_by_key(|w| w.positions.first().copied().unwrap_or(0));
    warnings
}

// ---------------------------------------------------------------------------
// 4. Adverb Detection
// ---------------------------------------------------------------------------

/// Detect adverbs in text (words ending in "-ly").
///
/// Returns a list of (byte_position, adverb) pairs. Common non-adverb
/// exceptions such as "family", "only", "early", etc. are excluded.
pub fn detect_adverbs(text: &str) -> Vec<(usize, String)> {
    const EXCEPTIONS: &[&str] = &[
        "family", "only", "early", "daily", "holy", "rely", "reply",
        "apply", "supply", "ally", "belly", "bully", "curly", "fly",
        "folly", "gully", "hilly", "holly", "homely", "imply", "italy",
        "jelly", "jolly", "july", "lily", "lonely", "lovely", "multiply",
        "rally", "silly", "tally", "ugly", "unlikely", "comply", "poly",
        "melancholy", "butterfly", "assembly",
    ];

    let word_re = Regex::new(r"\b([a-zA-Z]+ly)\b").expect("adverb regex must compile");
    let mut results = Vec::new();

    for cap in word_re.captures_iter(text) {
        if let Some(m) = cap.get(1) {
            let word = m.as_str();
            let lower = word.to_lowercase();
            // Must be at least 4 chars (e.g. "sly" is not an adverb in the -ly sense)
            if lower.len() >= 4 && !EXCEPTIONS.contains(&lower.as_str()) {
                results.push((m.start(), word.to_string()));
            }
        }
    }
    results
}

// ---------------------------------------------------------------------------
// 5. Sentence Length Analysis
// ---------------------------------------------------------------------------

/// Warning about a sentence that exceeds the maximum word count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceLengthWarning {
    /// Byte position of the sentence start.
    pub position: usize,
    /// Byte length of the sentence.
    pub length: usize,
    /// Number of words in the sentence.
    pub word_count: usize,
}

/// Detect sentences that exceed a given word count threshold.
pub fn detect_long_sentences(text: &str, max_words: usize) -> Vec<SentenceLengthWarning> {
    let sentence_re =
        Regex::new(r"[^.!?]+[.!?]+").expect("sentence regex must compile");
    let mut warnings = Vec::new();

    for m in sentence_re.find_iter(text) {
        let sentence = m.as_str().trim();
        let wc = sentence.split_whitespace().count();
        if wc > max_words {
            warnings.push(SentenceLengthWarning {
                position: m.start(),
                length: m.len(),
                word_count: wc,
            });
        }
    }

    // Also handle the case where the text has no terminal punctuation
    if warnings.is_empty() && !text.trim().is_empty() {
        let has_terminal = text.chars().any(|c| c == '.' || c == '!' || c == '?');
        if !has_terminal {
            let wc = text.split_whitespace().count();
            if wc > max_words {
                warnings.push(SentenceLengthWarning {
                    position: 0,
                    length: text.len(),
                    word_count: wc,
                });
            }
        }
    }

    warnings
}

// ---------------------------------------------------------------------------
// 7. Cliche Detection
// ---------------------------------------------------------------------------

/// Built-in list of common cliches to detect in prose.
const CLICHES: &[&str] = &[
    "at the end of the day",
    "back to the drawing board",
    "barking up the wrong tree",
    "beat around the bush",
    "better late than never",
    "bite the bullet",
    "break the ice",
    "burning the midnight oil",
    "by the skin of your teeth",
    "costs an arm and a leg",
    "cry over spilt milk",
    "cutting corners",
    "dead as a doornail",
    "easy as pie",
    "every cloud has a silver lining",
    "fit as a fiddle",
    "hit the nail on the head",
    "ignorance is bliss",
    "it takes two to tango",
    "last but not least",
    "let the cat out of the bag",
    "once in a blue moon",
    "piece of cake",
    "raining cats and dogs",
    "the best of both worlds",
    "tip of the iceberg",
    "under the weather",
    "when pigs fly",
    "a blessing in disguise",
    "add insult to injury",
    "the whole nine yards",
    "time flies",
    "actions speak louder than words",
    "a dime a dozen",
    "beat a dead horse",
    "birds of a feather",
    "blood is thicker than water",
];

/// Detect cliches in text.
///
/// Returns a list of (byte_position, cliche) pairs for each cliche found.
pub fn detect_cliches(text: &str) -> Vec<(usize, String)> {
    let lower = text.to_lowercase();
    let mut results = Vec::new();

    for cliche in CLICHES {
        let mut search_from = 0;
        while let Some(pos) = lower[search_from..].find(cliche) {
            let abs_pos = search_from + pos;
            results.push((abs_pos, cliche.to_string()));
            search_from = abs_pos + cliche.len();
        }
    }

    results.sort_by_key(|(pos, _)| *pos);
    results
}

// ---------------------------------------------------------------------------
// 8. Dialog Tag Analysis
// ---------------------------------------------------------------------------

/// Common overused dialog tags (beyond simple "said"/"asked").
const OVERUSED_DIALOG_TAGS: &[&str] = &[
    "exclaimed",
    "shouted",
    "whispered",
    "muttered",
    "murmured",
    "screamed",
    "yelled",
    "bellowed",
    "stammered",
    "stuttered",
    "retorted",
    "snapped",
    "hissed",
    "growled",
    "snarled",
    "barked",
    "wailed",
    "whimpered",
    "sobbed",
    "gasped",
    "sighed",
    "groaned",
    "moaned",
    "chuckled",
    "giggled",
    "laughed",
    "cried",
    "declared",
    "announced",
    "proclaimed",
    "interjected",
    "interrupted",
    "demanded",
    "pleaded",
    "implored",
];

/// Detect overused or unnecessary dialog tags in text.
///
/// Returns (byte_position, tag_word) for each occurrence found.
pub fn detect_said_alternatives(text: &str) -> Vec<(usize, String)> {
    let mut results = Vec::new();

    for tag in OVERUSED_DIALOG_TAGS {
        // Match the tag as a whole word, case-insensitive
        let pattern = format!(r"(?i)\b{}\b", regex::escape(tag));
        let re = Regex::new(&pattern).expect("dialog tag regex must compile");
        for m in re.find_iter(text) {
            results.push((m.start(), m.as_str().to_string()));
        }
    }

    results.sort_by_key(|(pos, _)| *pos);
    results
}

// ---------------------------------------------------------------------------
// 6. Writing Statistics Summary
// ---------------------------------------------------------------------------

/// A comprehensive writing analysis combining all linguistic checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingAnalysis {
    pub readability: ReadabilityMetrics,
    pub passive_voice: Vec<PassiveVoiceMatch>,
    pub repetitions: Vec<RepetitionWarning>,
    pub adverbs: Vec<(usize, String)>,
    pub long_sentences: Vec<SentenceLengthWarning>,
    pub cliches: Vec<(usize, String)>,
    pub dialog_tags: Vec<(usize, String)>,
}

impl WritingAnalysis {
    /// Generate a human-readable summary of the analysis.
    pub fn summary(&self) -> String {
        let level = self.readability.level();
        let mut parts = Vec::new();

        parts.push(format!(
            "Readability: {:.1} ({}) | Grade {:.1}",
            self.readability.flesch_reading_ease,
            level.label(),
            self.readability.flesch_kincaid_grade,
        ));
        parts.push(format!(
            "Words: {} | Sentences: {} | Syllables: {}",
            self.readability.word_count,
            self.readability.sentence_count,
            self.readability.syllable_count,
        ));

        if !self.passive_voice.is_empty() {
            parts.push(format!(
                "Passive voice: {} instance{}",
                self.passive_voice.len(),
                if self.passive_voice.len() == 1 { "" } else { "s" },
            ));
        }
        if !self.repetitions.is_empty() {
            parts.push(format!(
                "Repetitions: {} word{}",
                self.repetitions.len(),
                if self.repetitions.len() == 1 { "" } else { "s" },
            ));
        }
        if !self.adverbs.is_empty() {
            parts.push(format!(
                "Adverbs: {} found",
                self.adverbs.len(),
            ));
        }
        if !self.long_sentences.is_empty() {
            parts.push(format!(
                "Long sentences: {}",
                self.long_sentences.len(),
            ));
        }
        if !self.cliches.is_empty() {
            parts.push(format!(
                "Cliches: {} detected",
                self.cliches.len(),
            ));
        }
        if !self.dialog_tags.is_empty() {
            parts.push(format!(
                "Overused dialog tags: {}",
                self.dialog_tags.len(),
            ));
        }

        parts.join("\n")
    }
}

/// Run all writing quality checks on a block of text.
///
/// Uses sensible defaults: minimum word length of 4 for repetition detection,
/// maximum distance of 50 words, and a long-sentence threshold of 30 words.
pub fn analyze_text(text: &str) -> WritingAnalysis {
    WritingAnalysis {
        readability: analyze_readability(text),
        passive_voice: detect_passive_voice(text),
        repetitions: detect_repetitions(text, 4, 50),
        adverbs: detect_adverbs(text),
        long_sentences: detect_long_sentences(text, 30),
        cliches: detect_cliches(text),
        dialog_tags: detect_said_alternatives(text),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- count_syllables ----

    #[test]
    fn test_syllables_monosyllable() {
        assert_eq!(count_syllables("cat"), 1);
        assert_eq!(count_syllables("dog"), 1);
        assert_eq!(count_syllables("run"), 1);
    }

    #[test]
    fn test_syllables_two_syllables() {
        assert_eq!(count_syllables("hello"), 2);
        assert_eq!(count_syllables("water"), 2);
    }

    #[test]
    fn test_syllables_multi_syllable() {
        assert!(count_syllables("beautiful") >= 3);
        assert!(count_syllables("extraordinary") >= 4);
        assert!(count_syllables("communication") >= 4);
    }

    #[test]
    fn test_syllables_silent_e() {
        assert_eq!(count_syllables("time"), 1);
        assert_eq!(count_syllables("came"), 1);
        assert_eq!(count_syllables("make"), 1);
    }

    #[test]
    fn test_syllables_single_letter() {
        assert_eq!(count_syllables("a"), 1);
        assert_eq!(count_syllables("I"), 1);
    }

    #[test]
    fn test_syllables_empty_and_punctuation() {
        assert_eq!(count_syllables(""), 1);
        assert_eq!(count_syllables("..."), 1);
    }

    // ---- ReadabilityLevel ----

    #[test]
    fn test_readability_level_from_score() {
        assert_eq!(ReadabilityLevel::from_score(95.0), ReadabilityLevel::VeryEasy);
        assert_eq!(ReadabilityLevel::from_score(85.0), ReadabilityLevel::Easy);
        assert_eq!(ReadabilityLevel::from_score(75.0), ReadabilityLevel::FairlyEasy);
        assert_eq!(ReadabilityLevel::from_score(65.0), ReadabilityLevel::Standard);
        assert_eq!(ReadabilityLevel::from_score(55.0), ReadabilityLevel::FairlyDifficult);
        assert_eq!(ReadabilityLevel::from_score(40.0), ReadabilityLevel::Difficult);
        assert_eq!(ReadabilityLevel::from_score(20.0), ReadabilityLevel::VeryDifficult);
    }

    #[test]
    fn test_readability_level_boundary_values() {
        assert_eq!(ReadabilityLevel::from_score(90.0), ReadabilityLevel::VeryEasy);
        assert_eq!(ReadabilityLevel::from_score(89.9), ReadabilityLevel::Easy);
        assert_eq!(ReadabilityLevel::from_score(80.0), ReadabilityLevel::Easy);
        assert_eq!(ReadabilityLevel::from_score(79.9), ReadabilityLevel::FairlyEasy);
        assert_eq!(ReadabilityLevel::from_score(70.0), ReadabilityLevel::FairlyEasy);
        assert_eq!(ReadabilityLevel::from_score(69.9), ReadabilityLevel::Standard);
        assert_eq!(ReadabilityLevel::from_score(60.0), ReadabilityLevel::Standard);
        assert_eq!(ReadabilityLevel::from_score(59.9), ReadabilityLevel::FairlyDifficult);
        assert_eq!(ReadabilityLevel::from_score(50.0), ReadabilityLevel::FairlyDifficult);
        assert_eq!(ReadabilityLevel::from_score(49.9), ReadabilityLevel::Difficult);
        assert_eq!(ReadabilityLevel::from_score(30.0), ReadabilityLevel::Difficult);
        assert_eq!(ReadabilityLevel::from_score(29.9), ReadabilityLevel::VeryDifficult);
    }

    #[test]
    fn test_readability_level_labels() {
        assert_eq!(ReadabilityLevel::VeryEasy.label(), "Very Easy");
        assert_eq!(ReadabilityLevel::Difficult.label(), "Difficult");
        assert_eq!(ReadabilityLevel::VeryDifficult.label(), "Very Difficult");
    }

    // ---- analyze_readability ----

    #[test]
    fn test_readability_empty_text() {
        let m = analyze_readability("");
        assert_eq!(m.word_count, 0);
        assert_eq!(m.sentence_count, 0);
        assert_eq!(m.syllable_count, 0);
        assert_eq!(m.flesch_reading_ease, 0.0);
    }

    #[test]
    fn test_readability_whitespace_only() {
        let m = analyze_readability("   \n\t  ");
        assert_eq!(m.word_count, 0);
    }

    #[test]
    fn test_readability_simple_text() {
        let m = analyze_readability("The cat sat on the mat. The dog ran.");
        assert_eq!(m.word_count, 9);
        assert_eq!(m.sentence_count, 2);
        assert!(m.flesch_reading_ease > 70.0);
        assert!(m.flesch_kincaid_grade < 6.0);
    }

    #[test]
    fn test_readability_complex_text() {
        let text = "The epistemological implications of computational neuroscience \
                     fundamentally challenge our understanding of consciousness.";
        let m = analyze_readability(text);
        assert!(m.flesch_reading_ease < 30.0);
        assert!(m.flesch_kincaid_grade > 12.0);
    }

    #[test]
    fn test_readability_single_sentence_no_punctuation() {
        let m = analyze_readability("hello world");
        assert_eq!(m.sentence_count, 1); // default minimum
        assert_eq!(m.word_count, 2);
    }

    #[test]
    fn test_readability_metrics_level() {
        let m = analyze_readability("The cat sat. The dog ran.");
        assert!(matches!(
            m.level(),
            ReadabilityLevel::VeryEasy | ReadabilityLevel::Easy | ReadabilityLevel::FairlyEasy
        ));
    }

    // ---- detect_passive_voice ----

    #[test]
    fn test_passive_voice_basic() {
        let results = detect_passive_voice("The ball was kicked by the boy.");
        assert!(!results.is_empty());
        assert!(results[0].text.to_lowercase().contains("was kicked"));
    }

    #[test]
    fn test_passive_voice_were() {
        let results = detect_passive_voice("The windows were shattered.");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_passive_voice_being() {
        let results = detect_passive_voice("The house is being painted.");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_passive_voice_no_match() {
        let results = detect_passive_voice("The boy kicked the ball.");
        assert!(results.is_empty());
    }

    #[test]
    fn test_passive_voice_empty() {
        let results = detect_passive_voice("");
        assert!(results.is_empty());
    }

    #[test]
    fn test_passive_voice_position_and_length() {
        let text = "The ball was kicked.";
        let results = detect_passive_voice(text);
        assert!(!results.is_empty());
        let first = &results[0];
        assert!(first.position < text.len());
        assert!(first.length > 0);
        assert_eq!(&text[first.position..first.position + first.length], &first.text);
    }

    // ---- detect_repetitions ----

    #[test]
    fn test_repetitions_basic() {
        let text = "The castle was dark. The castle loomed above.";
        let warnings = detect_repetitions(text, 4, 50);
        assert!(warnings.iter().any(|w| w.word == "castle"));
    }

    #[test]
    fn test_repetitions_no_match_short_words() {
        let text = "A dog and a cat and a bird.";
        let warnings = detect_repetitions(text, 4, 50);
        // "and" is a stop word, "a" is too short
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_repetitions_min_length_filter() {
        let text = "go go go run run run";
        let short_warnings = detect_repetitions(text, 4, 50);
        assert!(short_warnings.is_empty()); // "go" and "run" are < 4 chars
        let long_warnings = detect_repetitions(text, 2, 50);
        assert!(!long_warnings.is_empty());
    }

    #[test]
    fn test_repetitions_empty() {
        let warnings = detect_repetitions("", 4, 50);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_repetitions_positions_ordered() {
        let text = "quickly quickly slowly slowly quickly";
        let warnings = detect_repetitions(text, 4, 50);
        for w in &warnings {
            let positions = &w.positions;
            for window in positions.windows(2) {
                assert!(window[0] < window[1]);
            }
        }
    }

    // ---- detect_adverbs ----

    #[test]
    fn test_adverbs_basic() {
        let results = detect_adverbs("She walked slowly and spoke softly.");
        let words: Vec<&str> = results.iter().map(|(_, w)| w.as_str()).collect();
        assert!(words.contains(&"slowly"));
        assert!(words.contains(&"softly"));
    }

    #[test]
    fn test_adverbs_exceptions_excluded() {
        let results = detect_adverbs("The family was lonely in the holy city.");
        let words: Vec<&str> = results.iter().map(|(_, w)| w.as_str()).collect();
        assert!(!words.contains(&"family"));
        assert!(!words.contains(&"lonely"));
        assert!(!words.contains(&"holy"));
    }

    #[test]
    fn test_adverbs_short_ly_excluded() {
        // "fly" ends in ly but is too short (3 chars)
        let results = detect_adverbs("A fly landed.");
        assert!(results.is_empty());
    }

    #[test]
    fn test_adverbs_empty() {
        let results = detect_adverbs("");
        assert!(results.is_empty());
    }

    #[test]
    fn test_adverbs_position() {
        let text = "He ran quickly.";
        let results = detect_adverbs(text);
        assert_eq!(results.len(), 1);
        let (pos, word) = &results[0];
        assert_eq!(&text[*pos..*pos + word.len()], "quickly");
    }

    // ---- detect_long_sentences ----

    #[test]
    fn test_long_sentences_basic() {
        let long = "word ".repeat(35) + "end.";
        let short = "Short sentence.";
        let text = format!("{} {}", long, short);
        let warnings = detect_long_sentences(&text, 30);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].word_count > 30);
    }

    #[test]
    fn test_long_sentences_all_short() {
        let text = "I am here. He is there. We go now.";
        let warnings = detect_long_sentences(text, 30);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_long_sentences_empty() {
        let warnings = detect_long_sentences("", 30);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_long_sentences_no_punctuation() {
        let text = "word ".repeat(35);
        let warnings = detect_long_sentences(&text, 30);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn test_long_sentences_position_and_length() {
        let text = "Short. ".to_owned() + &"word ".repeat(35) + "end.";
        let warnings = detect_long_sentences(&text, 30);
        assert!(!warnings.is_empty());
        let w = &warnings[0];
        assert!(w.position > 0);
        assert!(w.length > 0);
    }

    // ---- detect_cliches ----

    #[test]
    fn test_cliches_basic() {
        let text = "At the end of the day, we need to bite the bullet.";
        let results = detect_cliches(text);
        assert!(results.len() >= 2);
        let cliche_texts: Vec<&str> = results.iter().map(|(_, c)| c.as_str()).collect();
        assert!(cliche_texts.contains(&"at the end of the day"));
        assert!(cliche_texts.contains(&"bite the bullet"));
    }

    #[test]
    fn test_cliches_case_insensitive() {
        let text = "It was Easy As Pie to find.";
        let results = detect_cliches(text);
        assert!(results.iter().any(|(_, c)| c == "easy as pie"));
    }

    #[test]
    fn test_cliches_none_found() {
        let text = "The algorithm processed the data efficiently.";
        let results = detect_cliches(text);
        assert!(results.is_empty());
    }

    #[test]
    fn test_cliches_empty() {
        let results = detect_cliches("");
        assert!(results.is_empty());
    }

    #[test]
    fn test_cliches_position_correct() {
        let text = "Well, time flies when you are busy.";
        let results = detect_cliches(text);
        assert!(!results.is_empty());
        let (pos, cliche) = &results[0];
        // The cliche text should appear at the reported position (case insensitive)
        let found = &text.to_lowercase()[*pos..*pos + cliche.len()];
        assert_eq!(found, cliche.as_str());
    }

    #[test]
    fn test_cliches_multiple_same() {
        let text = "Time flies, and time flies again.";
        let results = detect_cliches(text);
        let tf_count = results.iter().filter(|(_, c)| c == "time flies").count();
        assert_eq!(tf_count, 2);
    }

    // ---- detect_said_alternatives ----

    #[test]
    fn test_dialog_tags_basic() {
        let text = r#""Stop!" she exclaimed. "No," he whispered."#;
        let results = detect_said_alternatives(text);
        let tags: Vec<&str> = results.iter().map(|(_, t)| t.as_str()).collect();
        assert!(tags.contains(&"exclaimed"));
        assert!(tags.contains(&"whispered"));
    }

    #[test]
    fn test_dialog_tags_case_insensitive() {
        let text = "He SHOUTED across the room.";
        let results = detect_said_alternatives(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.to_lowercase(), "shouted");
    }

    #[test]
    fn test_dialog_tags_none_found() {
        let text = r#""Hello," he said. "Hi," she said."#;
        let results = detect_said_alternatives(text);
        // "said" is not in the overused list
        assert!(results.is_empty());
    }

    #[test]
    fn test_dialog_tags_empty() {
        let results = detect_said_alternatives("");
        assert!(results.is_empty());
    }

    #[test]
    fn test_dialog_tags_position() {
        let text = "She whispered something.";
        let results = detect_said_alternatives(text);
        assert_eq!(results.len(), 1);
        let (pos, tag) = &results[0];
        assert_eq!(&text[*pos..*pos + tag.len()], "whispered");
    }

    // ---- analyze_text (combined) ----

    #[test]
    fn test_analyze_text_empty() {
        let analysis = analyze_text("");
        assert_eq!(analysis.readability.word_count, 0);
        assert!(analysis.passive_voice.is_empty());
        assert!(analysis.repetitions.is_empty());
        assert!(analysis.adverbs.is_empty());
        assert!(analysis.long_sentences.is_empty());
        assert!(analysis.cliches.is_empty());
        assert!(analysis.dialog_tags.is_empty());
    }

    #[test]
    fn test_analyze_text_combines_all_checks() {
        let text = "The ball was kicked slowly. She exclaimed loudly. \
                     At the end of the day, the castle was dark. \
                     The castle loomed. It was easy as pie.";
        let analysis = analyze_text(text);
        assert!(analysis.readability.word_count > 0);
        assert!(!analysis.passive_voice.is_empty());
        assert!(!analysis.adverbs.is_empty());
        assert!(!analysis.cliches.is_empty());
        assert!(!analysis.dialog_tags.is_empty());
    }

    #[test]
    fn test_analyze_text_summary_format() {
        let text = "The ball was kicked slowly. She exclaimed loudly.";
        let analysis = analyze_text(text);
        let summary = analysis.summary();
        assert!(summary.contains("Readability"));
        assert!(summary.contains("Words:"));
        assert!(summary.contains("Sentences:"));
    }

    #[test]
    fn test_analyze_text_summary_clean_text() {
        let text = "The cat sat on the mat.";
        let analysis = analyze_text(text);
        let summary = analysis.summary();
        // Clean text should not mention passive voice, adverbs, etc.
        assert!(!summary.contains("Passive voice"));
        assert!(!summary.contains("Cliches"));
        assert!(!summary.contains("dialog tags"));
    }

    #[test]
    fn test_summary_singular_passive_voice() {
        let text = "The ball was kicked.";
        let analysis = analyze_text(text);
        if analysis.passive_voice.len() == 1 {
            let summary = analysis.summary();
            assert!(summary.contains("1 instance"));
            assert!(!summary.contains("instances"));
        }
    }

    // ---- Integration / edge-case tests ----

    #[test]
    fn test_readability_default_is_zero() {
        let m = ReadabilityMetrics::default();
        assert_eq!(m.word_count, 0);
        assert_eq!(m.flesch_reading_ease, 0.0);
        assert_eq!(m.flesch_kincaid_grade, 0.0);
    }

    #[test]
    fn test_passive_voice_multiple_matches() {
        let text = "The house was built. The car was driven. The food was eaten.";
        let results = detect_passive_voice(text);
        // Should detect at least 2 passive constructions
        assert!(results.len() >= 2);
    }

    #[test]
    fn test_repetitions_count_field() {
        let text = "castle castle castle castle castle";
        let warnings = detect_repetitions(text, 4, 50);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].count, 5);
        assert_eq!(warnings[0].word, "castle");
    }

    #[test]
    fn test_cliches_list_has_minimum() {
        // Verify the built-in cliche list has at least 20 entries.
        assert!(CLICHES.len() >= 20);
    }

    #[test]
    fn test_dialog_tags_list_has_entries() {
        assert!(OVERUSED_DIALOG_TAGS.len() >= 20);
    }

    #[test]
    fn test_readability_avg_sentence_length() {
        let text = "One two three. Four five six.";
        let m = analyze_readability(text);
        assert!((m.avg_sentence_length - 3.0).abs() < 0.01);
    }
}
