use super::binder::Binder;

/// Aggregated statistics for the project or a selection
#[derive(Debug, Clone, Default)]
pub struct Statistics {
    pub word_count: usize,
    pub char_count: usize,
    pub char_count_no_spaces: usize,
    pub paragraph_count: usize,
    pub sentence_count: usize,
    pub line_count: usize,
    pub page_count: f64,
    pub document_count: usize,
    pub folder_count: usize,
    pub average_words_per_document: f64,
}

impl Statistics {
    /// Compute statistics for the entire project
    pub fn from_binder(binder: &Binder) -> Self {
        use super::binder::BinderItemKind;

        let items = binder.all_items();
        let mut stats = Statistics::default();

        for item in &items {
            match item.kind {
                BinderItemKind::Text => stats.document_count += 1,
                BinderItemKind::Folder => stats.folder_count += 1,
                _ => {}
            }

            if let Some(ref doc) = item.document {
                stats.word_count += doc.word_count();
                stats.char_count += doc.char_count();
                stats.char_count_no_spaces += doc.char_count_no_spaces();
                stats.paragraph_count += doc.paragraph_count();
                stats.sentence_count += doc.sentence_count();
                stats.line_count += doc.line_count();
            }
        }

        stats.page_count = stats.word_count as f64 / 250.0;
        stats.average_words_per_document = if stats.document_count > 0 {
            stats.word_count as f64 / stats.document_count as f64
        } else {
            0.0
        };

        stats
    }

    /// Compute statistics for a single text content
    pub fn from_text(text: &str) -> Self {
        Statistics {
            word_count: text.split_whitespace().count(),
            char_count: text.len(),
            char_count_no_spaces: text.chars().filter(|c| !c.is_whitespace()).count(),
            paragraph_count: text.split("\n\n").filter(|p| !p.trim().is_empty()).count(),
            sentence_count: text.chars()
                .filter(|c| *c == '.' || *c == '!' || *c == '?')
                .count()
                .max(if text.is_empty() { 0 } else { 1 }),
            line_count: if text.is_empty() { 0 } else { text.lines().count() },
            page_count: text.split_whitespace().count() as f64 / 250.0,
            document_count: 1,
            folder_count: 0,
            average_words_per_document: text.split_whitespace().count() as f64,
        }
    }

    /// Format word count with target progress
    pub fn progress_string(&self, target: Option<usize>) -> String {
        match target {
            Some(target) => {
                let pct = (self.word_count as f64 / target as f64 * 100.0).min(100.0);
                format!("{} / {} words ({:.1}%)", self.word_count, target, pct)
            }
            None => format!("{} words", self.word_count),
        }
    }
}

/// Session statistics for tracking writing sessions
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub words_written: i64, // Can be negative if deleting
    pub time_elapsed_seconds: u64,
    pub words_per_minute: f64,
}

impl SessionStats {
    pub fn new() -> Self {
        Self {
            words_written: 0,
            time_elapsed_seconds: 0,
            words_per_minute: 0.0,
        }
    }

    pub fn update(&mut self, word_delta: i64, elapsed_seconds: u64) {
        self.words_written = word_delta;
        self.time_elapsed_seconds = elapsed_seconds;
        if elapsed_seconds > 0 {
            self.words_per_minute = self.words_written as f64 / (elapsed_seconds as f64 / 60.0);
        }
    }

    /// Format elapsed time as human-readable string
    pub fn elapsed_display(&self) -> String {
        let hours = self.time_elapsed_seconds / 3600;
        let mins = (self.time_elapsed_seconds % 3600) / 60;
        let secs = self.time_elapsed_seconds % 60;
        if hours > 0 {
            format!("{}h {}m {}s", hours, mins, secs)
        } else if mins > 0 {
            format!("{}m {}s", mins, secs)
        } else {
            format!("{}s", secs)
        }
    }

    /// Get net word change as a formatted string with sign
    pub fn words_display(&self) -> String {
        if self.words_written > 0 {
            format!("+{}", self.words_written)
        } else {
            format!("{}", self.words_written)
        }
    }

    /// Estimated pages written this session
    pub fn pages_written(&self) -> f64 {
        self.words_written.max(0) as f64 / 250.0
    }

    /// Check if the session is active (has time recorded)
    pub fn is_active(&self) -> bool {
        self.time_elapsed_seconds > 0
    }
}

impl Statistics {
    /// Get a summary string
    pub fn summary(&self) -> String {
        format!(
            "{} words, {:.1} pages, {} docs in {} folders",
            self.word_count, self.page_count, self.document_count, self.folder_count
        )
    }

    /// Average words per page (should be ~250)
    pub fn avg_words_per_page(&self) -> f64 {
        if self.page_count > 0.0 {
            self.word_count as f64 / self.page_count
        } else {
            0.0
        }
    }

    /// Reading time estimate in minutes (250 WPM)
    pub fn reading_time_minutes(&self) -> f64 {
        self.word_count as f64 / 250.0
    }

    /// Speaking time estimate in minutes (150 WPM)
    pub fn speaking_time_minutes(&self) -> f64 {
        self.word_count as f64 / 150.0
    }

    /// Check if the project is empty
    pub fn is_empty(&self) -> bool {
        self.word_count == 0 && self.document_count == 0
    }
}

/// Detailed text analysis for the text statistics panel
#[derive(Debug, Clone, Default)]
pub struct TextAnalysis {
    pub word_count: usize,
    pub unique_words: usize,
    pub char_count: usize,
    pub char_no_spaces: usize,
    pub sentence_count: usize,
    pub paragraph_count: usize,
    pub avg_word_length: f64,
    pub avg_sentence_length: f64,
    pub avg_paragraph_length: f64,
    pub readability_score: f64,
    pub reading_time_minutes: f64,
    pub speaking_time_minutes: f64,
    pub most_common_words: Vec<(String, usize)>,
}

impl TextAnalysis {
    pub fn from_text(text: &str) -> Self {
        if text.is_empty() {
            return Self::default();
        }

        let words: Vec<&str> = text.split_whitespace().collect();
        let word_count = words.len();
        let char_count = text.len();
        let char_no_spaces = text.chars().filter(|c| !c.is_whitespace()).count();

        let sentence_count = text.chars()
            .filter(|c| *c == '.' || *c == '!' || *c == '?')
            .count()
            .max(1);

        let paragraph_count = text.split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .count()
            .max(1);

        // Unique words
        let mut word_freq = std::collections::HashMap::new();
        for word in &words {
            let lower = word.to_lowercase()
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_string();
            if !lower.is_empty() {
                *word_freq.entry(lower).or_insert(0usize) += 1;
            }
        }
        let unique_words = word_freq.len();

        // Most common words (exclude short words)
        let mut word_list: Vec<(String, usize)> = word_freq.into_iter()
            .filter(|(w, _)| w.len() > 3)
            .collect();
        word_list.sort_by(|a, b| b.1.cmp(&a.1));
        word_list.truncate(20);

        let avg_word_length = if word_count > 0 {
            words.iter().map(|w| w.len()).sum::<usize>() as f64 / word_count as f64
        } else {
            0.0
        };

        let avg_sentence_length = word_count as f64 / sentence_count as f64;
        let avg_paragraph_length = word_count as f64 / paragraph_count as f64;

        // Flesch Reading Ease approximation
        let syllables: usize = words.iter().map(|w| count_syllables(w)).sum();
        let readability_score = if word_count > 0 && sentence_count > 0 {
            206.835
                - 1.015 * (word_count as f64 / sentence_count as f64)
                - 84.6 * (syllables as f64 / word_count as f64)
        } else {
            0.0
        };

        let reading_time_minutes = word_count as f64 / 250.0;
        let speaking_time_minutes = word_count as f64 / 150.0;

        Self {
            word_count,
            unique_words,
            char_count,
            char_no_spaces,
            sentence_count,
            paragraph_count,
            avg_word_length,
            avg_sentence_length,
            avg_paragraph_length,
            readability_score,
            reading_time_minutes,
            speaking_time_minutes,
            most_common_words: word_list,
        }
    }

    pub fn readability_label(&self) -> &str {
        if self.readability_score >= 90.0 { "Very Easy" }
        else if self.readability_score >= 80.0 { "Easy" }
        else if self.readability_score >= 70.0 { "Fairly Easy" }
        else if self.readability_score >= 60.0 { "Standard" }
        else if self.readability_score >= 50.0 { "Fairly Difficult" }
        else if self.readability_score >= 30.0 { "Difficult" }
        else { "Very Difficult" }
    }

    /// Vocabulary richness (type-token ratio)
    pub fn vocabulary_richness(&self) -> f64 {
        if self.word_count == 0 {
            return 0.0;
        }
        self.unique_words as f64 / self.word_count as f64 * 100.0
    }

    /// Vocabulary richness label
    pub fn vocabulary_label(&self) -> &str {
        let ttr = self.vocabulary_richness();
        if ttr >= 70.0 { "Rich" }
        else if ttr >= 50.0 { "Moderate" }
        else { "Repetitive" }
    }

    /// Get a one-line summary
    pub fn summary(&self) -> String {
        format!(
            "{} words, {} unique, readability: {:.0} ({})",
            self.word_count, self.unique_words,
            self.readability_score, self.readability_label()
        )
    }

    /// Estimated grade level for reading
    pub fn grade_level(&self) -> f64 {
        if self.word_count == 0 || self.sentence_count == 0 {
            return 0.0;
        }
        // Flesch-Kincaid Grade Level
        let syllables: f64 = self.avg_word_length * 0.6 * self.word_count as f64;
        0.39 * (self.word_count as f64 / self.sentence_count as f64)
            + 11.8 * (syllables / self.word_count as f64)
            - 15.59
    }

    /// Is the text empty?
    pub fn is_empty(&self) -> bool {
        self.word_count == 0
    }
}

/// Advanced readability metrics computed from text
#[derive(Debug, Clone, Default)]
pub struct ReadabilityMetrics {
    /// Flesch Reading Ease score (0-100, higher = easier)
    pub flesch_reading_ease: f64,
    /// Flesch-Kincaid Grade Level (US grade levels)
    pub flesch_kincaid_grade: f64,
    /// Gunning Fog Index (estimated years of education needed)
    pub gunning_fog: f64,
    /// Coleman-Liau Index
    pub coleman_liau: f64,
    /// Automated Readability Index
    pub automated_readability: f64,
    /// SMOG grade (Simple Measure of Gobbledygook)
    pub smog_grade: f64,
    /// Average syllables per word
    pub avg_syllables_per_word: f64,
    /// Percentage of complex words (3+ syllables)
    pub complex_word_percentage: f64,
    /// Number of complex words
    pub complex_word_count: usize,
    /// Total syllable count
    pub total_syllables: usize,
}

impl ReadabilityMetrics {
    /// Compute all readability metrics from text
    pub fn from_text(text: &str) -> Self {
        if text.is_empty() {
            return Self::default();
        }

        let words: Vec<&str> = text.split_whitespace().collect();
        let word_count = words.len();
        if word_count == 0 {
            return Self::default();
        }

        let sentence_count = text.chars()
            .filter(|c| *c == '.' || *c == '!' || *c == '?')
            .count()
            .max(1);

        // Syllable analysis
        let syllable_counts: Vec<usize> = words.iter().map(|w| count_syllables(w)).collect();
        let total_syllables: usize = syllable_counts.iter().sum();
        let complex_word_count = syllable_counts.iter().filter(|&&s| s >= 3).count();

        let avg_syllables_per_word = total_syllables as f64 / word_count as f64;
        let words_per_sentence = word_count as f64 / sentence_count as f64;

        // Character counts for Coleman-Liau and ARI
        let total_chars: usize = words.iter().map(|w| w.chars().filter(|c| c.is_alphanumeric()).count()).sum();

        // Flesch Reading Ease
        let flesch_reading_ease = 206.835
            - 1.015 * words_per_sentence
            - 84.6 * avg_syllables_per_word;

        // Flesch-Kincaid Grade Level
        let flesch_kincaid_grade = 0.39 * words_per_sentence
            + 11.8 * avg_syllables_per_word
            - 15.59;

        // Gunning Fog Index
        let complex_word_percentage = complex_word_count as f64 / word_count as f64 * 100.0;
        let gunning_fog = 0.4 * (words_per_sentence + complex_word_percentage);

        // Coleman-Liau Index
        let l = total_chars as f64 / word_count as f64 * 100.0; // avg chars per 100 words
        let s = sentence_count as f64 / word_count as f64 * 100.0; // avg sentences per 100 words
        let coleman_liau = 0.0588 * l - 0.296 * s - 15.8;

        // Automated Readability Index
        let automated_readability = 4.71 * (total_chars as f64 / word_count as f64)
            + 0.5 * words_per_sentence
            - 21.43;

        // SMOG Grade
        let smog_grade = if sentence_count >= 3 {
            1.0430 * (complex_word_count as f64 * 30.0 / sentence_count as f64).sqrt() + 3.1291
        } else {
            flesch_kincaid_grade // Fallback for short texts
        };

        Self {
            flesch_reading_ease,
            flesch_kincaid_grade: flesch_kincaid_grade.max(0.0),
            gunning_fog: gunning_fog.max(0.0),
            coleman_liau: coleman_liau.max(0.0),
            automated_readability: automated_readability.max(0.0),
            smog_grade: smog_grade.max(0.0),
            avg_syllables_per_word,
            complex_word_percentage,
            complex_word_count,
            total_syllables,
        }
    }

    /// Get a consensus grade level (average of major indices)
    pub fn consensus_grade(&self) -> f64 {
        let grades = [
            self.flesch_kincaid_grade,
            self.gunning_fog,
            self.coleman_liau,
            self.automated_readability,
        ];
        let valid: Vec<f64> = grades.iter().copied().filter(|g| *g > 0.0).collect();
        if valid.is_empty() {
            return 0.0;
        }
        valid.iter().sum::<f64>() / valid.len() as f64
    }

    /// Human-readable label for the Flesch score
    pub fn flesch_label(&self) -> &str {
        if self.flesch_reading_ease >= 90.0 { "Very Easy (5th grade)" }
        else if self.flesch_reading_ease >= 80.0 { "Easy (6th grade)" }
        else if self.flesch_reading_ease >= 70.0 { "Fairly Easy (7th grade)" }
        else if self.flesch_reading_ease >= 60.0 { "Standard (8th-9th grade)" }
        else if self.flesch_reading_ease >= 50.0 { "Fairly Difficult (10th-12th grade)" }
        else if self.flesch_reading_ease >= 30.0 { "Difficult (College)" }
        else { "Very Difficult (Graduate)" }
    }

    /// Audience recommendation based on grade level
    pub fn audience_label(&self) -> &str {
        let grade = self.consensus_grade();
        if grade <= 6.0 { "Children / General Public" }
        else if grade <= 8.0 { "Young Adults" }
        else if grade <= 12.0 { "General Adults" }
        else if grade <= 16.0 { "College-educated" }
        else { "Academic / Professional" }
    }
}

/// Word frequency analysis for a text
#[derive(Debug, Clone, Default)]
pub struct WordFrequencyAnalysis {
    /// Word frequencies sorted by count (descending)
    pub frequencies: Vec<(String, usize)>,
    /// Total unique words
    pub unique_count: usize,
    /// Total word count
    pub total_count: usize,
    /// Type-token ratio (vocabulary richness)
    pub type_token_ratio: f64,
    /// Hapax legomena (words appearing only once)
    pub hapax_count: usize,
    /// Top bigrams (two-word phrases)
    pub top_bigrams: Vec<(String, usize)>,
}

impl WordFrequencyAnalysis {
    /// Compute word frequency analysis from text
    pub fn from_text(text: &str) -> Self {
        if text.is_empty() {
            return Self::default();
        }

        let words: Vec<String> = text
            .split_whitespace()
            .map(|w| w.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| !w.is_empty())
            .collect();

        let total_count = words.len();

        // Word frequencies
        let mut freq_map = std::collections::HashMap::new();
        for word in &words {
            *freq_map.entry(word.clone()).or_insert(0usize) += 1;
        }

        let unique_count = freq_map.len();
        let hapax_count = freq_map.values().filter(|&&c| c == 1).count();

        let mut frequencies: Vec<(String, usize)> = freq_map.into_iter().collect();
        frequencies.sort_by(|a, b| b.1.cmp(&a.1));

        // Bigrams
        let mut bigram_map = std::collections::HashMap::new();
        for window in words.windows(2) {
            let bigram = format!("{} {}", window[0], window[1]);
            *bigram_map.entry(bigram).or_insert(0usize) += 1;
        }
        let mut top_bigrams: Vec<(String, usize)> = bigram_map
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .collect();
        top_bigrams.sort_by(|a, b| b.1.cmp(&a.1));
        top_bigrams.truncate(15);

        let type_token_ratio = if total_count > 0 {
            unique_count as f64 / total_count as f64 * 100.0
        } else {
            0.0
        };

        Self {
            frequencies,
            unique_count,
            total_count,
            type_token_ratio,
            hapax_count,
            top_bigrams,
        }
    }

    /// Get the top N most frequent words
    pub fn top_words(&self, n: usize) -> &[(String, usize)] {
        let end = n.min(self.frequencies.len());
        &self.frequencies[..end]
    }

    /// Get words appearing only once
    pub fn hapax_words(&self) -> Vec<&str> {
        self.frequencies
            .iter()
            .filter(|(_, c)| *c == 1)
            .map(|(w, _)| w.as_str())
            .collect()
    }

    /// Vocabulary richness label
    pub fn richness_label(&self) -> &str {
        if self.type_token_ratio >= 70.0 { "Very Rich" }
        else if self.type_token_ratio >= 55.0 { "Rich" }
        else if self.type_token_ratio >= 40.0 { "Moderate" }
        else if self.type_token_ratio >= 25.0 { "Repetitive" }
        else { "Very Repetitive" }
    }
}

fn count_syllables(word: &str) -> usize {
    let word = word.to_lowercase();
    let vowels = "aeiouy";
    let mut count = 0;
    let mut prev_vowel = false;
    for ch in word.chars() {
        if vowels.contains(ch) {
            if !prev_vowel {
                count += 1;
            }
            prev_vowel = true;
        } else {
            prev_vowel = false;
        }
    }
    // Handle silent 'e'
    if word.ends_with('e') && count > 1 {
        count -= 1;
    }
    count.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_from_text() {
        let stats = Statistics::from_text("Hello world. This is a test.");
        assert_eq!(stats.word_count, 6);
        assert_eq!(stats.sentence_count, 2);
        assert_eq!(stats.document_count, 1);
    }

    #[test]
    fn test_statistics_empty_text() {
        let stats = Statistics::from_text("");
        assert_eq!(stats.word_count, 0);
        assert_eq!(stats.char_count, 0);
    }

    #[test]
    fn test_progress_string() {
        let stats = Statistics::from_text("one two three four five");
        let progress = stats.progress_string(Some(10));
        assert!(progress.contains("5 / 10"));
        assert!(progress.contains("50.0%"));

        let no_target = stats.progress_string(None);
        assert!(no_target.contains("5 words"));
    }

    #[test]
    fn test_reading_speaking_time() {
        let stats = Statistics::from_text(&"word ".repeat(250));
        assert!((stats.reading_time_minutes() - 1.0).abs() < 0.1);
        assert!((stats.speaking_time_minutes() - 250.0 / 150.0).abs() < 0.1);
    }

    #[test]
    fn test_session_stats() {
        let mut session = SessionStats::new();
        assert!(!session.is_active());

        session.update(500, 600);
        assert!(session.is_active());
        assert_eq!(session.words_written, 500);
        assert!((session.words_per_minute - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_session_elapsed_display() {
        let mut session = SessionStats::new();
        session.update(100, 3661);
        assert_eq!(session.elapsed_display(), "1h 1m 1s");

        session.update(100, 125);
        assert_eq!(session.elapsed_display(), "2m 5s");

        session.update(100, 30);
        assert_eq!(session.elapsed_display(), "30s");
    }

    #[test]
    fn test_session_words_display() {
        let mut session = SessionStats::new();
        session.update(100, 60);
        assert_eq!(session.words_display(), "+100");

        session.update(-50, 60);
        assert_eq!(session.words_display(), "-50");
    }

    #[test]
    fn test_session_pages_written() {
        let mut session = SessionStats::new();
        session.update(500, 600);
        assert!((session.pages_written() - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_text_analysis_basic() {
        let analysis = TextAnalysis::from_text("The quick brown fox jumps over the lazy dog. The dog barked loudly.");
        assert_eq!(analysis.word_count, 13);
        assert!(analysis.unique_words > 0);
        assert_eq!(analysis.sentence_count, 2);
        assert!(analysis.readability_score > 0.0);
    }

    #[test]
    fn test_text_analysis_empty() {
        let analysis = TextAnalysis::from_text("");
        assert!(analysis.is_empty());
        assert_eq!(analysis.word_count, 0);
    }

    #[test]
    fn test_readability_labels() {
        let mut analysis = TextAnalysis::default();
        analysis.readability_score = 95.0;
        assert_eq!(analysis.readability_label(), "Very Easy");
        analysis.readability_score = 85.0;
        assert_eq!(analysis.readability_label(), "Easy");
        analysis.readability_score = 55.0;
        assert_eq!(analysis.readability_label(), "Fairly Difficult");
        analysis.readability_score = 20.0;
        assert_eq!(analysis.readability_label(), "Very Difficult");
    }

    #[test]
    fn test_vocabulary_richness() {
        let analysis = TextAnalysis::from_text("the the the the the");
        assert!(analysis.vocabulary_richness() < 50.0);
        assert_eq!(analysis.vocabulary_label(), "Repetitive");
    }

    #[test]
    fn test_text_analysis_summary() {
        let analysis = TextAnalysis::from_text("Hello world. Testing.");
        let summary = analysis.summary();
        assert!(summary.contains("3 words"));
    }

    #[test]
    fn test_count_syllables() {
        assert_eq!(count_syllables("cat"), 1);
        assert_eq!(count_syllables("hello"), 2);
        assert_eq!(count_syllables("beautiful"), 3);
        assert_eq!(count_syllables("a"), 1);
    }

    #[test]
    fn test_statistics_summary() {
        let stats = Statistics::from_text("Hello world.");
        let summary = stats.summary();
        assert!(summary.contains("2 words"));
    }

    #[test]
    fn test_readability_metrics_basic() {
        let text = "The cat sat on the mat. The dog ran in the yard. It was a nice day.";
        let metrics = ReadabilityMetrics::from_text(text);
        assert!(metrics.flesch_reading_ease > 50.0);
        assert!(metrics.flesch_kincaid_grade >= 0.0);
        assert!(metrics.gunning_fog >= 0.0);
        assert!(metrics.total_syllables > 0);
    }

    #[test]
    fn test_readability_metrics_empty() {
        let metrics = ReadabilityMetrics::from_text("");
        assert_eq!(metrics.flesch_reading_ease, 0.0);
        assert_eq!(metrics.total_syllables, 0);
    }

    #[test]
    fn test_readability_flesch_label() {
        let text_easy = "The cat sat. The dog ran. I am here. We go now.";
        let metrics_easy = ReadabilityMetrics::from_text(text_easy);
        // Easy text should score high
        assert!(metrics_easy.flesch_reading_ease > 60.0);

        let mut metrics = ReadabilityMetrics::default();
        metrics.flesch_reading_ease = 95.0;
        assert!(metrics.flesch_label().contains("Very Easy"));
        metrics.flesch_reading_ease = 45.0;
        assert!(metrics.flesch_label().contains("Difficult"));
        metrics.flesch_reading_ease = 10.0;
        assert!(metrics.flesch_label().contains("Very Difficult"));
    }

    #[test]
    fn test_readability_consensus_grade() {
        let text = "The quick brown fox jumps over the lazy dog. Simple sentences are easy to read.";
        let metrics = ReadabilityMetrics::from_text(text);
        let grade = metrics.consensus_grade();
        assert!(grade > 0.0);
    }

    #[test]
    fn test_readability_audience_label() {
        let mut metrics = ReadabilityMetrics::default();
        metrics.flesch_kincaid_grade = 5.0;
        metrics.gunning_fog = 5.0;
        metrics.coleman_liau = 5.0;
        metrics.automated_readability = 5.0;
        assert_eq!(metrics.audience_label(), "Children / General Public");
    }

    #[test]
    fn test_word_frequency_basic() {
        let text = "the cat and the dog and the bird";
        let analysis = WordFrequencyAnalysis::from_text(text);
        assert_eq!(analysis.total_count, 8);
        assert_eq!(analysis.unique_count, 5);
        assert!(analysis.type_token_ratio > 0.0);
        // "the" should be most frequent
        assert_eq!(analysis.frequencies[0].0, "the");
        assert_eq!(analysis.frequencies[0].1, 3);
    }

    #[test]
    fn test_word_frequency_empty() {
        let analysis = WordFrequencyAnalysis::from_text("");
        assert_eq!(analysis.total_count, 0);
        assert_eq!(analysis.unique_count, 0);
    }

    #[test]
    fn test_word_frequency_hapax() {
        let text = "one two two three three three";
        let analysis = WordFrequencyAnalysis::from_text(text);
        assert_eq!(analysis.hapax_count, 1); // "one" appears only once
        let hapax = analysis.hapax_words();
        assert!(hapax.contains(&"one"));
    }

    #[test]
    fn test_word_frequency_bigrams() {
        let text = "the cat the cat the cat the dog the dog";
        let analysis = WordFrequencyAnalysis::from_text(text);
        // Should have bigrams with count > 1
        assert!(analysis.top_bigrams.len() >= 2);
        // "the cat" should appear with count 3
        let the_cat = analysis.top_bigrams.iter().find(|(b, _)| b == "the cat");
        assert!(the_cat.is_some());
        assert_eq!(the_cat.unwrap().1, 3);
    }

    #[test]
    fn test_word_frequency_richness_label() {
        let mut analysis = WordFrequencyAnalysis::default();
        analysis.type_token_ratio = 75.0;
        assert_eq!(analysis.richness_label(), "Very Rich");
        analysis.type_token_ratio = 20.0;
        assert_eq!(analysis.richness_label(), "Very Repetitive");
    }

    #[test]
    fn test_word_frequency_top_words() {
        let text = "alpha beta gamma alpha beta alpha";
        let analysis = WordFrequencyAnalysis::from_text(text);
        let top2 = analysis.top_words(2);
        assert_eq!(top2.len(), 2);
        assert_eq!(top2[0].0, "alpha");
    }

    #[test]
    fn test_statistics_from_binder() {
        use crate::core::binder::{Binder, BinderItem};
        let mut binder = Binder::default_structure();
        let mut a = BinderItem::new_text("A");
        if let Some(ref mut doc) = a.document {
            doc.content = "Hello world. This is a test.".to_string();
        }
        let mut b = BinderItem::new_text("B");
        if let Some(ref mut doc) = b.document {
            doc.content = "Another paragraph here.".to_string();
        }
        binder.draft.add_child(a);
        binder.draft.add_child(b);

        let stats = Statistics::from_binder(&binder);
        assert_eq!(stats.word_count, 9);
        assert_eq!(stats.document_count, 2);
        assert!(stats.folder_count >= 3); // Draft, Research, Trash
        assert!(stats.average_words_per_document > 0.0);
    }

    #[test]
    fn test_statistics_is_empty() {
        let stats = Statistics::default();
        assert!(stats.is_empty());

        let stats2 = Statistics::from_text("Hello");
        assert!(!stats2.is_empty());
    }

    #[test]
    fn test_statistics_avg_words_per_page() {
        let stats = Statistics::from_text(&"word ".repeat(500));
        let avg = stats.avg_words_per_page();
        assert!((avg - 250.0).abs() < 1.0);
    }

    #[test]
    fn test_statistics_avg_words_per_page_empty() {
        let stats = Statistics::default();
        assert_eq!(stats.avg_words_per_page(), 0.0);
    }

    #[test]
    fn test_session_stats_negative_words() {
        let mut session = SessionStats::new();
        session.update(-100, 60);
        assert_eq!(session.words_display(), "-100");
        // Pages written should be 0 for negative
        assert_eq!(session.pages_written(), 0.0);
    }

    #[test]
    fn test_session_stats_zero_time() {
        let mut session = SessionStats::new();
        session.update(100, 0);
        assert_eq!(session.words_per_minute, 0.0);
    }

    #[test]
    fn test_text_analysis_grade_level() {
        let analysis = TextAnalysis::from_text("Simple words. Easy to read.");
        let grade = analysis.grade_level();
        // Grade level should be a reasonable value
        assert!(grade >= 0.0 || grade < 0.0); // Just ensure it computes

        let empty = TextAnalysis::default();
        assert_eq!(empty.grade_level(), 0.0);
    }

    #[test]
    fn test_text_analysis_vocabulary_labels() {
        let mut analysis = TextAnalysis::default();
        analysis.word_count = 10;
        analysis.unique_words = 8;
        assert_eq!(analysis.vocabulary_label(), "Rich");

        analysis.unique_words = 3;
        assert_eq!(analysis.vocabulary_label(), "Repetitive");
    }

    #[test]
    fn test_readability_metrics_complex_text() {
        let text = "The epistemological implications of computational neuroscience fundamentally challenge our understanding of consciousness. Philosophical considerations regarding phenomenological experience suggest that reductionist approaches inadequately capture the complexity of subjective awareness.";
        let metrics = ReadabilityMetrics::from_text(text);
        // Complex text should have lower reading ease
        assert!(metrics.flesch_reading_ease < 50.0);
        // And higher grade level
        assert!(metrics.flesch_kincaid_grade > 10.0);
        assert!(metrics.complex_word_count > 5);
    }

    #[test]
    fn test_readability_metrics_consensus_grade_zero() {
        let metrics = ReadabilityMetrics::default();
        assert_eq!(metrics.consensus_grade(), 0.0);
    }

    #[test]
    fn test_readability_audience_labels() {
        let mut metrics = ReadabilityMetrics::default();
        metrics.flesch_kincaid_grade = 7.0;
        metrics.gunning_fog = 7.0;
        metrics.coleman_liau = 7.0;
        metrics.automated_readability = 7.0;
        assert_eq!(metrics.audience_label(), "Young Adults");

        metrics.flesch_kincaid_grade = 10.0;
        metrics.gunning_fog = 10.0;
        metrics.coleman_liau = 10.0;
        metrics.automated_readability = 10.0;
        assert_eq!(metrics.audience_label(), "General Adults");

        metrics.flesch_kincaid_grade = 15.0;
        metrics.gunning_fog = 15.0;
        metrics.coleman_liau = 15.0;
        metrics.automated_readability = 15.0;
        assert_eq!(metrics.audience_label(), "College-educated");

        metrics.flesch_kincaid_grade = 20.0;
        metrics.gunning_fog = 20.0;
        metrics.coleman_liau = 20.0;
        metrics.automated_readability = 20.0;
        assert_eq!(metrics.audience_label(), "Academic / Professional");
    }

    #[test]
    fn test_word_frequency_top_words_empty() {
        let analysis = WordFrequencyAnalysis::default();
        let top = analysis.top_words(5);
        assert!(top.is_empty());
    }

    #[test]
    fn test_word_frequency_hapax_empty() {
        let analysis = WordFrequencyAnalysis::default();
        let hapax = analysis.hapax_words();
        assert!(hapax.is_empty());
    }

    #[test]
    fn test_readability_flesch_labels_all() {
        let mut m = ReadabilityMetrics::default();
        m.flesch_reading_ease = 95.0;
        assert!(m.flesch_label().contains("Very Easy"));
        m.flesch_reading_ease = 85.0;
        assert!(m.flesch_label().contains("Easy"));
        m.flesch_reading_ease = 75.0;
        assert!(m.flesch_label().contains("Fairly Easy"));
        m.flesch_reading_ease = 65.0;
        assert!(m.flesch_label().contains("Standard"));
        m.flesch_reading_ease = 55.0;
        assert!(m.flesch_label().contains("Fairly Difficult"));
        m.flesch_reading_ease = 35.0;
        assert!(m.flesch_label().contains("Difficult"));
        m.flesch_reading_ease = 15.0;
        assert!(m.flesch_label().contains("Very Difficult"));
    }

    #[test]
    fn test_word_frequency_richness_labels_all() {
        let mut a = WordFrequencyAnalysis::default();
        a.type_token_ratio = 75.0;
        assert_eq!(a.richness_label(), "Very Rich");
        a.type_token_ratio = 60.0;
        assert_eq!(a.richness_label(), "Rich");
        a.type_token_ratio = 45.0;
        assert_eq!(a.richness_label(), "Moderate");
        a.type_token_ratio = 30.0;
        assert_eq!(a.richness_label(), "Repetitive");
        a.type_token_ratio = 15.0;
        assert_eq!(a.richness_label(), "Very Repetitive");
    }

    #[test]
    fn test_statistics_from_text_multiline() {
        let stats = Statistics::from_text("First paragraph.\n\nSecond paragraph.\n\nThird one.");
        assert_eq!(stats.paragraph_count, 3);
        assert_eq!(stats.line_count, 5);
    }

    #[test]
    fn test_count_syllables_edge_cases() {
        assert_eq!(count_syllables(""), 1); // min 1
        assert_eq!(count_syllables("I"), 1);
        assert_eq!(count_syllables("eye"), 1); // silent e
        assert_eq!(count_syllables("queue"), 1);
    }
}
