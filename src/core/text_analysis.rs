use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Stop Words
// ---------------------------------------------------------------------------

/// Common English stop words (~50 entries) used for filtering frequency lists.
const STOP_WORDS: &[&str] = &[
    "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
    "of", "with", "by", "from", "is", "it", "was", "were", "are", "be",
    "been", "being", "have", "has", "had", "do", "does", "did", "will",
    "would", "shall", "should", "may", "might", "must", "can", "could",
    "that", "this", "these", "those", "he", "she", "they", "we", "you",
    "not", "no", "so", "if", "as", "its", "i", "me", "my", "his", "her",
];

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Readability scores computed using standard formulas.
#[derive(Debug, Clone, Default)]
pub struct ReadabilityScores {
    /// Flesch-Kincaid Grade Level (US school grade)
    pub flesch_kincaid_grade: f64,
    /// Flesch Reading Ease (0-100, higher = easier)
    pub flesch_reading_ease: f64,
    /// Gunning Fog Index (years of education needed)
    pub gunning_fog: f64,
    /// Coleman-Liau Index
    pub coleman_liau: f64,
    /// Automated Readability Index
    pub ari: f64,
    /// SMOG grade (Simple Measure of Gobbledygook)
    pub smog: f64,
}

/// A word and its frequency in a text.
#[derive(Debug, Clone)]
pub struct WordFrequency {
    /// The word (lowercased, stripped of punctuation).
    pub word: String,
    /// Number of times the word appears.
    pub count: usize,
    /// Percentage of total words this word represents.
    pub percentage: f64,
}

/// Information about a single sentence.
#[derive(Debug, Clone)]
pub struct SentenceInfo {
    /// The sentence text.
    pub text: String,
    /// Number of words in the sentence.
    pub word_count: usize,
    /// Average length of words in the sentence (in characters).
    pub avg_word_length: f64,
}

/// Information about a single paragraph.
#[derive(Debug, Clone)]
pub struct ParagraphInfo {
    /// The paragraph text.
    pub text: String,
    /// Number of words in the paragraph.
    pub word_count: usize,
    /// Number of sentences in the paragraph.
    pub sentence_count: usize,
}

/// Vocabulary richness metrics.
#[derive(Debug, Clone, Default)]
pub struct VocabularyMetrics {
    /// Type-token ratio (unique words / total words, as a value between 0.0 and 1.0).
    pub type_token_ratio: f64,
    /// Number of hapax legomena (words appearing exactly once).
    pub hapax_legomena: usize,
    /// Average word length in characters.
    pub average_word_length: f64,
    /// Number of unique words.
    pub unique_word_count: usize,
}

/// Full text analysis result combining all metrics.
#[derive(Debug, Clone)]
pub struct TextAnalysis {
    /// Readability scores from multiple formulas.
    pub readability: ReadabilityScores,
    /// Word frequency list sorted by count descending.
    pub word_frequencies: Vec<WordFrequency>,
    /// Average sentence length in words.
    pub avg_sentence_length: f64,
    /// Minimum sentence length in words.
    pub min_sentence_length: usize,
    /// Maximum sentence length in words.
    pub max_sentence_length: usize,
    /// Sentence length standard deviation.
    pub sentence_length_std_dev: f64,
    /// Number of paragraphs.
    pub paragraph_count: usize,
    /// Average paragraph length in words.
    pub avg_paragraph_length: f64,
    /// Vocabulary richness metrics.
    pub vocabulary: VocabularyMetrics,
    /// Total word count.
    pub total_words: usize,
    /// Total sentence count.
    pub total_sentences: usize,
}

// ---------------------------------------------------------------------------
// Helper: extract words
// ---------------------------------------------------------------------------

/// Extract words from text, lowercased with punctuation stripped.
fn extract_words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|w| {
            w.to_lowercase()
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_string()
        })
        .filter(|w| !w.is_empty())
        .collect()
}

/// Split text into raw whitespace-delimited tokens (preserving original form).
fn raw_words(text: &str) -> Vec<&str> {
    text.split_whitespace().collect()
}

// ---------------------------------------------------------------------------
// Helper: split into sentences
// ---------------------------------------------------------------------------

/// Split text into sentences using terminal punctuation (., !, ?).
fn split_sentences(text: &str) -> Vec<String> {
    if text.trim().is_empty() {
        return Vec::new();
    }

    let mut sentences = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if ch == '.' || ch == '!' || ch == '?' {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
            current.clear();
        }
    }

    // Handle trailing text without terminal punctuation
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        sentences.push(trimmed);
    }

    sentences
}

// ---------------------------------------------------------------------------
// Helper: split into paragraphs
// ---------------------------------------------------------------------------

/// Split text into paragraphs (separated by blank lines).
fn split_paragraphs(text: &str) -> Vec<String> {
    text.split("\n\n")
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

// ---------------------------------------------------------------------------
// Syllable counting
// ---------------------------------------------------------------------------

/// Count the number of syllables in an English word using vowel-group heuristics.
///
/// Handles silent 'e' at end of word, consecutive vowels counted as one group,
/// and the consonant-le pattern. Returns a minimum of 1.
pub fn syllable_count(word: &str) -> usize {
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

    // Handle silent 'e' at the end
    if lower.ends_with('e') && count > 1 {
        count -= 1;
    }

    // Handle consonant + "le" pattern (e.g., "simple", "bottle")
    if lower.ends_with("le") && chars.len() > 2 {
        let before_le = chars[chars.len() - 3];
        if !vowels.contains(before_le) {
            count += 1;
        }
    }

    count.max(1)
}

/// Returns true if the word has 3 or more syllables (complex word).
fn is_complex_word(word: &str) -> bool {
    syllable_count(word) >= 3
}

// ---------------------------------------------------------------------------
// Readability computation
// ---------------------------------------------------------------------------

/// Compute readability scores for a block of text.
///
/// Uses the standard mathematical definitions for:
/// - Flesch-Kincaid Grade Level
/// - Flesch Reading Ease
/// - Gunning Fog Index
/// - Coleman-Liau Index
/// - Automated Readability Index (ARI)
/// - SMOG Grade
pub fn compute_readability(text: &str) -> ReadabilityScores {
    if text.trim().is_empty() {
        return ReadabilityScores::default();
    }

    let words: Vec<&str> = raw_words(text);
    let word_count = words.len();
    if word_count == 0 {
        return ReadabilityScores::default();
    }

    let sentences = split_sentences(text);
    let sentence_count = sentences.len().max(1);

    // Syllable and complex word counts
    let mut total_syllables: usize = 0;
    let mut complex_word_count: usize = 0;
    for w in &words {
        let s = syllable_count(w);
        total_syllables += s;
        if s >= 3 {
            complex_word_count += 1;
        }
    }

    // Character count (only alphanumeric, for Coleman-Liau and ARI)
    let total_chars: usize = words
        .iter()
        .map(|w| w.chars().filter(|c| c.is_alphanumeric()).count())
        .sum();

    let words_per_sentence = word_count as f64 / sentence_count as f64;
    let syllables_per_word = total_syllables as f64 / word_count as f64;

    // Flesch Reading Ease: 206.835 - 1.015 * (words/sentences) - 84.6 * (syllables/words)
    let flesch_reading_ease =
        206.835 - 1.015 * words_per_sentence - 84.6 * syllables_per_word;

    // Flesch-Kincaid Grade Level: 0.39 * (words/sentences) + 11.8 * (syllables/words) - 15.59
    let flesch_kincaid_grade =
        0.39 * words_per_sentence + 11.8 * syllables_per_word - 15.59;

    // Gunning Fog: 0.4 * (words/sentences + 100 * complex_words/words)
    let complex_pct = complex_word_count as f64 / word_count as f64 * 100.0;
    let gunning_fog = 0.4 * (words_per_sentence + complex_pct);

    // Coleman-Liau: 0.0588 * L - 0.296 * S - 15.8
    //   L = avg number of characters per 100 words
    //   S = avg number of sentences per 100 words
    let l = total_chars as f64 / word_count as f64 * 100.0;
    let s = sentence_count as f64 / word_count as f64 * 100.0;
    let coleman_liau = 0.0588 * l - 0.296 * s - 15.8;

    // Automated Readability Index: 4.71 * (chars/words) + 0.5 * (words/sentences) - 21.43
    let ari = 4.71 * (total_chars as f64 / word_count as f64)
        + 0.5 * words_per_sentence
        - 21.43;

    // SMOG: 1.0430 * sqrt(complex_words * 30 / sentences) + 3.1291
    let smog = if sentence_count >= 3 {
        1.0430 * (complex_word_count as f64 * 30.0 / sentence_count as f64).sqrt()
            + 3.1291
    } else {
        // Fallback for very short texts
        flesch_kincaid_grade
    };

    ReadabilityScores {
        flesch_kincaid_grade,
        flesch_reading_ease,
        gunning_fog,
        coleman_liau,
        ari,
        smog,
    }
}

// ---------------------------------------------------------------------------
// Word frequency analysis
// ---------------------------------------------------------------------------

/// Compute word frequencies for a text, sorted by count descending.
///
/// Words are lowercased and stripped of leading/trailing punctuation.
pub fn word_frequencies(text: &str) -> Vec<WordFrequency> {
    let words = extract_words(text);
    let total = words.len();
    if total == 0 {
        return Vec::new();
    }

    let mut freq_map: HashMap<String, usize> = HashMap::new();
    for w in &words {
        *freq_map.entry(w.clone()).or_insert(0) += 1;
    }

    let mut freqs: Vec<WordFrequency> = freq_map
        .into_iter()
        .map(|(word, count)| WordFrequency {
            word,
            count,
            percentage: count as f64 / total as f64 * 100.0,
        })
        .collect();

    freqs.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.word.cmp(&b.word)));
    freqs
}

/// Return the top N most frequent words in the text.
pub fn top_n_words(text: &str, n: usize) -> Vec<WordFrequency> {
    let mut freqs = word_frequencies(text);
    freqs.truncate(n);
    freqs
}

/// Return words used more than `threshold` times.
pub fn overused_words(text: &str, threshold: usize) -> Vec<WordFrequency> {
    word_frequencies(text)
        .into_iter()
        .filter(|wf| wf.count > threshold)
        .collect()
}

/// Remove common English stop words from a frequency list.
pub fn filter_stop_words(frequencies: &[WordFrequency]) -> Vec<WordFrequency> {
    frequencies
        .iter()
        .filter(|wf| !STOP_WORDS.contains(&wf.word.as_str()))
        .cloned()
        .collect()
}

// ---------------------------------------------------------------------------
// Sentence analysis
// ---------------------------------------------------------------------------

/// Return sentence-by-sentence information.
pub fn sentence_lengths(text: &str) -> Vec<SentenceInfo> {
    let sentences = split_sentences(text);
    sentences
        .into_iter()
        .map(|s| {
            let words: Vec<&str> = s.split_whitespace().collect();
            let wc = words.len();
            let total_char_len: usize = words
                .iter()
                .map(|w| w.chars().filter(|c| c.is_alphanumeric()).count())
                .sum();
            let avg_wl = if wc > 0 {
                total_char_len as f64 / wc as f64
            } else {
                0.0
            };
            SentenceInfo {
                text: s,
                word_count: wc,
                avg_word_length: avg_wl,
            }
        })
        .collect()
}

/// Compute the average sentence length (mean words per sentence).
pub fn average_sentence_length(text: &str) -> f64 {
    let infos = sentence_lengths(text);
    if infos.is_empty() {
        return 0.0;
    }
    let total_words: usize = infos.iter().map(|s| s.word_count).sum();
    total_words as f64 / infos.len() as f64
}

/// Return the N longest sentences, sorted by word count descending.
pub fn longest_sentences(text: &str, n: usize) -> Vec<SentenceInfo> {
    let mut infos = sentence_lengths(text);
    infos.sort_by(|a, b| b.word_count.cmp(&a.word_count));
    infos.truncate(n);
    infos
}

// ---------------------------------------------------------------------------
// Paragraph analysis
// ---------------------------------------------------------------------------

/// Return paragraph-by-paragraph analysis.
pub fn paragraph_analysis(text: &str) -> Vec<ParagraphInfo> {
    let paragraphs = split_paragraphs(text);
    paragraphs
        .into_iter()
        .map(|p| {
            let wc = p.split_whitespace().count();
            let sc = split_sentences(&p).len();
            ParagraphInfo {
                text: p,
                word_count: wc,
                sentence_count: sc,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Vocabulary metrics
// ---------------------------------------------------------------------------

/// Compute vocabulary richness metrics.
pub fn vocabulary_metrics(text: &str) -> VocabularyMetrics {
    let words = extract_words(text);
    let total = words.len();
    if total == 0 {
        return VocabularyMetrics::default();
    }

    let mut freq_map: HashMap<String, usize> = HashMap::new();
    for w in &words {
        *freq_map.entry(w.clone()).or_insert(0) += 1;
    }

    let unique_word_count = freq_map.len();
    let hapax_legomena = freq_map.values().filter(|&&c| c == 1).count();
    let type_token_ratio = unique_word_count as f64 / total as f64;

    let total_char_len: usize = words.iter().map(|w| w.len()).sum();
    let average_word_length = total_char_len as f64 / total as f64;

    VocabularyMetrics {
        type_token_ratio,
        hapax_legomena,
        average_word_length,
        unique_word_count,
    }
}

// ---------------------------------------------------------------------------
// Full analysis
// ---------------------------------------------------------------------------

/// Perform a comprehensive text analysis combining all metrics.
pub fn analyze_text(text: &str) -> TextAnalysis {
    let readability = compute_readability(text);
    let freqs = word_frequencies(text);
    let sent_infos = sentence_lengths(text);
    let para_infos = paragraph_analysis(text);
    let vocab = vocabulary_metrics(text);

    let total_words: usize = sent_infos.iter().map(|s| s.word_count).sum();
    let total_sentences = sent_infos.len();

    let avg_sentence_length = if total_sentences > 0 {
        total_words as f64 / total_sentences as f64
    } else {
        0.0
    };

    let min_sentence_length = sent_infos.iter().map(|s| s.word_count).min().unwrap_or(0);
    let max_sentence_length = sent_infos.iter().map(|s| s.word_count).max().unwrap_or(0);

    let sentence_length_std_dev = if total_sentences > 0 {
        let mean = avg_sentence_length;
        let variance = sent_infos
            .iter()
            .map(|s| {
                let diff = s.word_count as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / total_sentences as f64;
        variance.sqrt()
    } else {
        0.0
    };

    let paragraph_count = para_infos.len();
    let avg_paragraph_length = if paragraph_count > 0 {
        para_infos.iter().map(|p| p.word_count).sum::<usize>() as f64
            / paragraph_count as f64
    } else {
        0.0
    };

    TextAnalysis {
        readability,
        word_frequencies: freqs,
        avg_sentence_length,
        min_sentence_length,
        max_sentence_length,
        sentence_length_std_dev,
        paragraph_count,
        avg_paragraph_length,
        vocabulary: vocab,
        total_words,
        total_sentences,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- syllable_count ----

    #[test]
    fn test_syllable_count_monosyllables() {
        assert_eq!(syllable_count("cat"), 1);
        assert_eq!(syllable_count("dog"), 1);
        assert_eq!(syllable_count("run"), 1);
        assert_eq!(syllable_count("the"), 1);
    }

    #[test]
    fn test_syllable_count_two_syllables() {
        assert_eq!(syllable_count("hello"), 2);
        assert_eq!(syllable_count("water"), 2);
    }

    #[test]
    fn test_syllable_count_multi_syllable() {
        assert!(syllable_count("beautiful") >= 3);
        assert!(syllable_count("extraordinary") >= 4);
        assert!(syllable_count("communication") >= 4);
    }

    #[test]
    fn test_syllable_count_silent_e() {
        assert_eq!(syllable_count("time"), 1);
        assert_eq!(syllable_count("came"), 1);
        assert_eq!(syllable_count("make"), 1);
    }

    #[test]
    fn test_syllable_count_empty_and_punctuation() {
        assert_eq!(syllable_count(""), 1);
        assert_eq!(syllable_count("..."), 1);
    }

    #[test]
    fn test_syllable_count_single_letter() {
        assert_eq!(syllable_count("a"), 1);
        assert_eq!(syllable_count("I"), 1);
    }

    #[test]
    fn test_syllable_count_consecutive_vowels() {
        // "queue" has consecutive vowels treated as one group
        assert_eq!(syllable_count("queue"), 1);
    }

    #[test]
    fn test_syllable_count_with_punctuation() {
        assert_eq!(syllable_count("hello,"), 2);
        assert_eq!(syllable_count("world!"), 1);
    }

    // ---- compute_readability ----

    #[test]
    fn test_readability_empty_text() {
        let scores = compute_readability("");
        assert_eq!(scores.flesch_reading_ease, 0.0);
        assert_eq!(scores.flesch_kincaid_grade, 0.0);
        assert_eq!(scores.gunning_fog, 0.0);
        assert_eq!(scores.coleman_liau, 0.0);
        assert_eq!(scores.ari, 0.0);
        assert_eq!(scores.smog, 0.0);
    }

    #[test]
    fn test_readability_whitespace_only() {
        let scores = compute_readability("   \n\t  ");
        assert_eq!(scores.flesch_reading_ease, 0.0);
    }

    #[test]
    fn test_readability_simple_text() {
        let scores = compute_readability(
            "The cat sat on the mat. The dog ran in the yard. It was a nice day.",
        );
        // Simple text should have high reading ease
        assert!(scores.flesch_reading_ease > 60.0);
        // And low grade level
        assert!(scores.flesch_kincaid_grade < 8.0);
    }

    #[test]
    fn test_readability_complex_text() {
        let scores = compute_readability(
            "The epistemological implications of computational neuroscience \
             fundamentally challenge our understanding of consciousness. \
             Philosophical considerations regarding phenomenological experience \
             suggest that reductionist approaches inadequately capture the \
             complexity of subjective awareness.",
        );
        // Complex text should have low reading ease
        assert!(scores.flesch_reading_ease < 30.0);
        // High grade level
        assert!(scores.flesch_kincaid_grade > 10.0);
    }

    #[test]
    fn test_readability_gunning_fog_positive() {
        let scores = compute_readability("The cat sat on the mat.");
        assert!(scores.gunning_fog >= 0.0);
    }

    #[test]
    fn test_readability_smog_with_enough_sentences() {
        let text = "The cat sat. The dog ran. The bird flew. A fish swam.";
        let scores = compute_readability(text);
        assert!(scores.smog >= 0.0);
    }

    #[test]
    fn test_readability_smog_fallback_short_text() {
        // Fewer than 3 sentences: SMOG falls back to FK grade
        let scores = compute_readability("The cat sat. The dog ran.");
        // SMOG should equal FK grade for short texts
        assert!((scores.smog - scores.flesch_kincaid_grade).abs() < 0.001);
    }

    #[test]
    fn test_readability_single_word() {
        let scores = compute_readability("Hello");
        // Should not panic and should return some values
        assert!(scores.flesch_reading_ease != 0.0 || scores.flesch_kincaid_grade != 0.0);
    }

    // ---- word_frequencies ----

    #[test]
    fn test_word_frequencies_basic() {
        let freqs = word_frequencies("the cat and the dog and the bird");
        assert!(!freqs.is_empty());
        // "the" should be most frequent (3 times)
        assert_eq!(freqs[0].word, "the");
        assert_eq!(freqs[0].count, 3);
    }

    #[test]
    fn test_word_frequencies_empty() {
        let freqs = word_frequencies("");
        assert!(freqs.is_empty());
    }

    #[test]
    fn test_word_frequencies_percentage() {
        let freqs = word_frequencies("hello hello hello hello");
        assert_eq!(freqs.len(), 1);
        assert!((freqs[0].percentage - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_word_frequencies_sorted_desc() {
        let freqs = word_frequencies("one two two three three three");
        // First entry should be "three" with count 3
        assert_eq!(freqs[0].word, "three");
        assert_eq!(freqs[0].count, 3);
        // Then "two" with count 2
        assert_eq!(freqs[1].word, "two");
        assert_eq!(freqs[1].count, 2);
    }

    #[test]
    fn test_word_frequencies_strips_punctuation() {
        let freqs = word_frequencies("hello, world! hello.");
        // "hello" should appear twice, "world" once
        let hello = freqs.iter().find(|f| f.word == "hello").unwrap();
        assert_eq!(hello.count, 2);
        let world = freqs.iter().find(|f| f.word == "world").unwrap();
        assert_eq!(world.count, 1);
    }

    #[test]
    fn test_word_frequencies_case_insensitive() {
        let freqs = word_frequencies("Hello HELLO hello");
        assert_eq!(freqs.len(), 1);
        assert_eq!(freqs[0].count, 3);
    }

    // ---- top_n_words ----

    #[test]
    fn test_top_n_words_basic() {
        let top = top_n_words("alpha beta gamma alpha beta alpha", 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].word, "alpha");
        assert_eq!(top[0].count, 3);
    }

    #[test]
    fn test_top_n_words_n_larger_than_total() {
        let top = top_n_words("hello world", 100);
        assert_eq!(top.len(), 2);
    }

    #[test]
    fn test_top_n_words_empty() {
        let top = top_n_words("", 5);
        assert!(top.is_empty());
    }

    // ---- overused_words ----

    #[test]
    fn test_overused_words_basic() {
        let overused = overused_words("cat cat cat dog dog bird", 2);
        assert_eq!(overused.len(), 1); // only "cat" appears > 2 times
        assert_eq!(overused[0].word, "cat");
        assert_eq!(overused[0].count, 3);
    }

    #[test]
    fn test_overused_words_none_above_threshold() {
        let overused = overused_words("one two three", 5);
        assert!(overused.is_empty());
    }

    #[test]
    fn test_overused_words_empty() {
        let overused = overused_words("", 1);
        assert!(overused.is_empty());
    }

    // ---- filter_stop_words ----

    #[test]
    fn test_filter_stop_words_removes_common() {
        let freqs = word_frequencies("the cat is on the mat");
        let filtered = filter_stop_words(&freqs);
        // "the", "is", "on" should be removed
        assert!(filtered.iter().all(|f| f.word != "the"));
        assert!(filtered.iter().all(|f| f.word != "is"));
        assert!(filtered.iter().all(|f| f.word != "on"));
        // "cat" and "mat" should remain
        assert!(filtered.iter().any(|f| f.word == "cat"));
        assert!(filtered.iter().any(|f| f.word == "mat"));
    }

    #[test]
    fn test_filter_stop_words_empty_input() {
        let filtered = filter_stop_words(&[]);
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_filter_stop_words_all_stop_words() {
        let freqs = word_frequencies("the is was and or but");
        let filtered = filter_stop_words(&freqs);
        assert!(filtered.is_empty());
    }

    // ---- sentence_lengths ----

    #[test]
    fn test_sentence_lengths_basic() {
        let infos = sentence_lengths("The cat sat. The big brown dog ran quickly.");
        assert_eq!(infos.len(), 2);
        assert_eq!(infos[0].word_count, 3);
        assert_eq!(infos[1].word_count, 6);
    }

    #[test]
    fn test_sentence_lengths_empty() {
        let infos = sentence_lengths("");
        assert!(infos.is_empty());
    }

    #[test]
    fn test_sentence_lengths_no_punctuation() {
        let infos = sentence_lengths("hello world how are you");
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].word_count, 5);
    }

    #[test]
    fn test_sentence_lengths_avg_word_length() {
        let infos = sentence_lengths("Hi.");
        assert_eq!(infos.len(), 1);
        // "Hi." has 1 word; alphanumeric chars in "Hi." is 2
        assert!(infos[0].avg_word_length > 0.0);
    }

    // ---- average_sentence_length ----

    #[test]
    fn test_average_sentence_length_basic() {
        let avg = average_sentence_length("One two three. Four five six.");
        assert!((avg - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_average_sentence_length_empty() {
        let avg = average_sentence_length("");
        assert_eq!(avg, 0.0);
    }

    #[test]
    fn test_average_sentence_length_single_sentence() {
        let avg = average_sentence_length("Hello world.");
        assert!((avg - 2.0).abs() < 0.01);
    }

    // ---- longest_sentences ----

    #[test]
    fn test_longest_sentences_basic() {
        let text = "Short. A somewhat longer sentence here. Tiny.";
        let longest = longest_sentences(text, 1);
        assert_eq!(longest.len(), 1);
        assert_eq!(longest[0].word_count, 5);
    }

    #[test]
    fn test_longest_sentences_n_larger_than_total() {
        let text = "First. Second.";
        let longest = longest_sentences(text, 100);
        assert_eq!(longest.len(), 2);
    }

    #[test]
    fn test_longest_sentences_empty() {
        let longest = longest_sentences("", 5);
        assert!(longest.is_empty());
    }

    // ---- paragraph_analysis ----

    #[test]
    fn test_paragraph_analysis_basic() {
        let text = "First paragraph here.\n\nSecond paragraph now.";
        let paras = paragraph_analysis(text);
        assert_eq!(paras.len(), 2);
        assert_eq!(paras[0].word_count, 3);
        assert_eq!(paras[1].word_count, 3);
    }

    #[test]
    fn test_paragraph_analysis_single_paragraph() {
        let text = "Just one paragraph.";
        let paras = paragraph_analysis(text);
        assert_eq!(paras.len(), 1);
        assert_eq!(paras[0].sentence_count, 1);
    }

    #[test]
    fn test_paragraph_analysis_empty() {
        let paras = paragraph_analysis("");
        assert!(paras.is_empty());
    }

    #[test]
    fn test_paragraph_analysis_sentence_count() {
        let text = "First sentence. Second sentence.\n\nThird sentence.";
        let paras = paragraph_analysis(text);
        assert_eq!(paras[0].sentence_count, 2);
        assert_eq!(paras[1].sentence_count, 1);
    }

    // ---- vocabulary_metrics ----

    #[test]
    fn test_vocabulary_metrics_basic() {
        let metrics = vocabulary_metrics("the cat sat on the mat");
        assert_eq!(metrics.unique_word_count, 5); // the, cat, sat, on, mat
        assert!(metrics.type_token_ratio > 0.0);
        assert!(metrics.type_token_ratio <= 1.0);
    }

    #[test]
    fn test_vocabulary_metrics_empty() {
        let metrics = vocabulary_metrics("");
        assert_eq!(metrics.unique_word_count, 0);
        assert_eq!(metrics.type_token_ratio, 0.0);
        assert_eq!(metrics.hapax_legomena, 0);
        assert_eq!(metrics.average_word_length, 0.0);
    }

    #[test]
    fn test_vocabulary_metrics_all_unique() {
        let metrics = vocabulary_metrics("alpha beta gamma delta");
        assert_eq!(metrics.unique_word_count, 4);
        assert!((metrics.type_token_ratio - 1.0).abs() < 0.001);
        assert_eq!(metrics.hapax_legomena, 4); // all appear once
    }

    #[test]
    fn test_vocabulary_metrics_hapax_legomena() {
        let metrics = vocabulary_metrics("one two two three three three");
        assert_eq!(metrics.hapax_legomena, 1); // only "one"
    }

    #[test]
    fn test_vocabulary_metrics_avg_word_length() {
        let metrics = vocabulary_metrics("ab cd ef");
        // All words are 2 characters
        assert!((metrics.average_word_length - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_vocabulary_metrics_repetitive_text() {
        let metrics = vocabulary_metrics("the the the the the");
        assert_eq!(metrics.unique_word_count, 1);
        assert!((metrics.type_token_ratio - 0.2).abs() < 0.01);
    }

    // ---- analyze_text (full analysis) ----

    #[test]
    fn test_analyze_text_basic() {
        let text = "The quick brown fox jumps over the lazy dog. \
                     The dog barked loudly at the fox.";
        let analysis = analyze_text(text);
        assert!(analysis.total_words > 0);
        assert!(analysis.total_sentences > 0);
        assert!(!analysis.word_frequencies.is_empty());
        assert!(analysis.vocabulary.unique_word_count > 0);
    }

    #[test]
    fn test_analyze_text_empty() {
        let analysis = analyze_text("");
        assert_eq!(analysis.total_words, 0);
        assert_eq!(analysis.total_sentences, 0);
        assert!(analysis.word_frequencies.is_empty());
        assert_eq!(analysis.paragraph_count, 0);
    }

    #[test]
    fn test_analyze_text_sentence_stats() {
        let text = "Short. A much longer sentence with many words here.";
        let analysis = analyze_text(text);
        assert!(analysis.max_sentence_length > analysis.min_sentence_length);
        assert!(analysis.sentence_length_std_dev > 0.0);
    }

    #[test]
    fn test_analyze_text_paragraph_count() {
        let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let analysis = analyze_text(text);
        assert_eq!(analysis.paragraph_count, 3);
    }

    #[test]
    fn test_analyze_text_single_word() {
        let analysis = analyze_text("Hello");
        assert_eq!(analysis.total_words, 1);
        assert_eq!(analysis.total_sentences, 1);
        assert_eq!(analysis.word_frequencies.len(), 1);
    }

    #[test]
    fn test_analyze_text_very_long_text() {
        // Generate a long text to verify no panics or overflows
        let sentence = "The quick brown fox jumps over the lazy dog. ";
        let text = sentence.repeat(100);
        let analysis = analyze_text(&text);
        assert!(analysis.total_words > 500);
        assert!(analysis.total_sentences >= 100);
        assert!(analysis.vocabulary.unique_word_count > 0);
        assert!(analysis.readability.flesch_reading_ease != 0.0);
    }

    // ---- edge cases ----

    #[test]
    fn test_readability_scores_default() {
        let scores = ReadabilityScores::default();
        assert_eq!(scores.flesch_kincaid_grade, 0.0);
        assert_eq!(scores.flesch_reading_ease, 0.0);
        assert_eq!(scores.gunning_fog, 0.0);
        assert_eq!(scores.coleman_liau, 0.0);
        assert_eq!(scores.ari, 0.0);
        assert_eq!(scores.smog, 0.0);
    }

    #[test]
    fn test_vocabulary_metrics_default() {
        let metrics = VocabularyMetrics::default();
        assert_eq!(metrics.type_token_ratio, 0.0);
        assert_eq!(metrics.hapax_legomena, 0);
        assert_eq!(metrics.average_word_length, 0.0);
        assert_eq!(metrics.unique_word_count, 0);
    }

    #[test]
    fn test_stop_words_list_size() {
        // Verify we have at least 50 stop words
        assert!(STOP_WORDS.len() >= 50);
    }

    #[test]
    fn test_is_complex_word() {
        assert!(is_complex_word("beautiful"));
        assert!(is_complex_word("extraordinary"));
        assert!(!is_complex_word("cat"));
        assert!(!is_complex_word("hello"));
    }

    #[test]
    fn test_split_sentences_multiple_punctuation() {
        let sentences = split_sentences("Really? Yes! Okay.");
        assert_eq!(sentences.len(), 3);
    }

    #[test]
    fn test_split_paragraphs_basic() {
        let paras = split_paragraphs("A\n\nB\n\nC");
        assert_eq!(paras.len(), 3);
    }

    #[test]
    fn test_extract_words_strips_punctuation() {
        let words = extract_words("hello, world!");
        assert_eq!(words, vec!["hello", "world"]);
    }
}
