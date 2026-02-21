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
}
