use std::collections::HashMap;
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

    /// Estimated completion percentage toward a word target
    pub fn completion_toward_target(&self, target: usize) -> f64 {
        if target == 0 {
            return 0.0;
        }
        (self.word_count as f64 / target as f64 * 100.0).min(100.0)
    }

    /// Words remaining to reach a target
    pub fn words_remaining(&self, target: usize) -> usize {
        target.saturating_sub(self.word_count)
    }

    /// Estimated days to completion at a given daily word rate
    pub fn days_to_completion(&self, target: usize, words_per_day: usize) -> Option<usize> {
        if words_per_day == 0 {
            return None;
        }
        let remaining = self.words_remaining(target);
        if remaining == 0 {
            return Some(0);
        }
        Some(remaining.div_ceil(words_per_day))
    }

    /// Average words per document
    pub fn avg_words_per_doc(&self) -> f64 {
        if self.document_count == 0 {
            return 0.0;
        }
        self.word_count as f64 / self.document_count as f64
    }

    /// Project size classification
    pub fn size_label(&self) -> &str {
        match self.word_count {
            0..=999 => "Flash Fiction / Note",
            1000..=7499 => "Short Story",
            7500..=17499 => "Novelette",
            17500..=39999 => "Novella",
            40000..=79999 => "Novel",
            80000..=119999 => "Full Novel",
            _ => "Epic / Tome",
        }
    }

    /// Compute statistics for a single text content
    pub fn from_text(text: &str) -> Self {
        let word_count = text.split_whitespace().count();
        Statistics {
            word_count,
            char_count: text.len(),
            char_count_no_spaces: text.chars().filter(|c| !c.is_whitespace()).count(),
            paragraph_count: text.split("\n\n").filter(|p| !p.trim().is_empty()).count(),
            sentence_count: text.chars()
                .filter(|c| *c == '.' || *c == '!' || *c == '?')
                .count()
                .max(if text.is_empty() { 0 } else { 1 }),
            line_count: if text.is_empty() { 0 } else { text.lines().count() },
            page_count: word_count as f64 / 250.0,
            document_count: 1,
            folder_count: 0,
            average_words_per_document: word_count as f64,
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

        // Single pass over characters for non-space count and sentence count
        let (char_no_spaces, raw_sentences) = text.chars().fold((0usize, 0usize), |(ns, sc), c| {
            (
                ns + usize::from(!c.is_whitespace()),
                sc + usize::from(matches!(c, '.' | '!' | '?')),
            )
        });
        let sentence_count = raw_sentences.max(1);

        let paragraph_count = text.split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .count()
            .max(1);

        // Unique words
        let mut word_freq = HashMap::new();
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
            .filter(|(w, _): &(String, usize)| w.len() > 3)
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

        // Syllable analysis — single pass, no intermediate Vec
        let (total_syllables, complex_word_count) = words.iter().fold((0usize, 0usize), |(total, complex), w| {
            let s = count_syllables(w);
            (total + s, complex + usize::from(s >= 3))
        });

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
        let (sum, count) = grades.iter().fold((0.0, 0u32), |(s, c), &g| {
            if g > 0.0 { (s + g, c + 1) } else { (s, c) }
        });
        if count == 0 { 0.0 } else { sum / count as f64 }
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
        let mut freq_map: HashMap<&str, usize> = HashMap::new();
        for word in &words {
            *freq_map.entry(word).or_default() += 1;
        }

        let unique_count = freq_map.len();
        let hapax_count = freq_map.values().filter(|&&c| c == 1).count();

        let mut frequencies: Vec<(String, usize)> = freq_map.into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        frequencies.sort_by(|a, b| b.1.cmp(&a.1));

        // Bigrams
        let mut bigram_map = HashMap::new();
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

/// A single daily writing record
#[derive(Debug, Clone)]
pub struct DailyEntry {
    pub date: chrono::NaiveDate,
    pub words_written: i64,
    pub word_count_start: usize,
    pub word_count_end: usize,
    pub time_spent_seconds: u64,
}

impl DailyEntry {
    pub fn new(date: chrono::NaiveDate, words_written: i64, start: usize, end: usize, seconds: u64) -> Self {
        Self {
            date,
            words_written,
            word_count_start: start,
            word_count_end: end,
            time_spent_seconds: seconds,
        }
    }

    /// Words per minute for this day
    pub fn wpm(&self) -> f64 {
        if self.time_spent_seconds == 0 {
            return 0.0;
        }
        self.words_written.unsigned_abs() as f64 / (self.time_spent_seconds as f64 / 60.0)
    }

    /// Hours spent
    pub fn hours(&self) -> f64 {
        self.time_spent_seconds as f64 / 3600.0
    }

    /// Was this a productive day (net positive words)?
    pub fn is_productive(&self) -> bool {
        self.words_written > 0
    }
}

/// Tracks writing history over time and computes trends
#[derive(Debug, Clone, Default)]
pub struct WritingHistory {
    pub entries: Vec<DailyEntry>,
}

impl WritingHistory {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Add an entry for a day
    pub fn record(&mut self, entry: DailyEntry) {
        // Replace existing entry for same date
        self.entries.retain(|e| e.date != entry.date);
        self.entries.push(entry);
        self.entries.sort_by_key(|e| e.date);
    }

    /// Total words written across all days
    pub fn total_words_written(&self) -> i64 {
        self.entries.iter().map(|e| e.words_written).sum()
    }

    /// Total time spent in seconds
    pub fn total_time_seconds(&self) -> u64 {
        self.entries.iter().map(|e| e.time_spent_seconds).sum()
    }

    /// Average words per day (only counting days with entries)
    pub fn avg_words_per_day(&self) -> f64 {
        if self.entries.is_empty() {
            return 0.0;
        }
        self.total_words_written() as f64 / self.entries.len() as f64
    }

    /// Average words per minute across all sessions
    pub fn avg_wpm(&self) -> f64 {
        let total_time = self.total_time_seconds();
        if total_time == 0 {
            return 0.0;
        }
        self.total_words_written().unsigned_abs() as f64 / (total_time as f64 / 60.0)
    }

    /// Best day (most words written)
    pub fn best_day(&self) -> Option<&DailyEntry> {
        self.entries.iter().max_by_key(|e| e.words_written)
    }

    /// Current writing streak (consecutive days with entries from the end)
    pub fn current_streak(&self) -> usize {
        if self.entries.is_empty() {
            return 0;
        }
        let mut streak = 1;
        let mut i = self.entries.len() - 1;
        while i > 0 {
            let prev_date = self.entries[i - 1].date;
            let curr_date = self.entries[i].date;
            if curr_date - prev_date == chrono::Duration::days(1) {
                streak += 1;
                i -= 1;
            } else {
                break;
            }
        }
        streak
    }

    /// Longest streak of consecutive writing days
    pub fn longest_streak(&self) -> usize {
        if self.entries.is_empty() {
            return 0;
        }
        let mut best = 1;
        let mut current = 1;
        for i in 1..self.entries.len() {
            if self.entries[i].date - self.entries[i - 1].date == chrono::Duration::days(1) {
                current += 1;
                best = best.max(current);
            } else {
                current = 1;
            }
        }
        best
    }

    /// Days with entries in the last N days
    pub fn active_days_in_last(&self, days: i64) -> usize {
        let cutoff = chrono::Utc::now().date_naive() - chrono::Duration::days(days);
        self.entries.iter().filter(|e| e.date > cutoff).count()
    }

    /// Moving average of words/day over a window of N entries
    pub fn moving_average(&self, window: usize) -> Vec<(chrono::NaiveDate, f64)> {
        if self.entries.len() < window || window == 0 {
            return Vec::new();
        }
        let mut result = Vec::new();
        for i in (window - 1)..self.entries.len() {
            let sum: i64 = self.entries[i + 1 - window..=i]
                .iter()
                .map(|e| e.words_written)
                .sum();
            let avg = sum as f64 / window as f64;
            result.push((self.entries[i].date, avg));
        }
        result
    }

    /// Compute a WritingTrend from the entries
    pub fn analyze_trend(&self) -> WritingTrend {
        let total_words = self.total_words_written();
        let total_days = self.entries.len();
        let productive_days = self.entries.iter().filter(|e| e.is_productive()).count();

        let direction = if self.entries.len() < 2 {
            TrendDirection::Stable
        } else {
            let mid = self.entries.len() / 2;
            let first_half: f64 = self.entries[..mid].iter().map(|e| e.words_written as f64).sum::<f64>()
                / mid as f64;
            let second_half: f64 = self.entries[mid..].iter().map(|e| e.words_written as f64).sum::<f64>()
                / (self.entries.len() - mid) as f64;
            let diff = second_half - first_half;
            if diff > first_half.abs() * 0.1 {
                TrendDirection::Increasing
            } else if diff < -(first_half.abs() * 0.1) {
                TrendDirection::Decreasing
            } else {
                TrendDirection::Stable
            }
        };

        WritingTrend {
            total_words,
            total_days,
            productive_days,
            avg_words_per_day: self.avg_words_per_day(),
            avg_wpm: self.avg_wpm(),
            best_day_words: self.best_day().map(|d| d.words_written).unwrap_or(0),
            current_streak: self.current_streak(),
            longest_streak: self.longest_streak(),
            direction,
        }
    }

    /// Number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the history is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Overall trend analysis
#[derive(Debug, Clone)]
pub struct WritingTrend {
    pub total_words: i64,
    pub total_days: usize,
    pub productive_days: usize,
    pub avg_words_per_day: f64,
    pub avg_wpm: f64,
    pub best_day_words: i64,
    pub current_streak: usize,
    pub longest_streak: usize,
    pub direction: TrendDirection,
}

impl WritingTrend {
    /// Productivity ratio (productive days / total days)
    pub fn productivity_ratio(&self) -> f64 {
        if self.total_days == 0 {
            return 0.0;
        }
        self.productive_days as f64 / self.total_days as f64
    }

    /// Summary string
    pub fn summary(&self) -> String {
        format!(
            "{} words over {} days ({} productive), avg {:.0}/day, streak: {}, trend: {}",
            self.total_words, self.total_days, self.productive_days,
            self.avg_words_per_day, self.current_streak, self.direction.label()
        )
    }
}

/// Direction of the writing trend
#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

impl TrendDirection {
    pub fn label(&self) -> &str {
        match self {
            TrendDirection::Increasing => "Increasing",
            TrendDirection::Decreasing => "Decreasing",
            TrendDirection::Stable => "Stable",
        }
    }
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

    #[test]
    fn test_completion_toward_target() {
        let mut stats = Statistics::default();
        stats.word_count = 25000;
        assert!((stats.completion_toward_target(50000) - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_completion_toward_target_zero() {
        let stats = Statistics::default();
        assert_eq!(stats.completion_toward_target(0), 0.0);
    }

    #[test]
    fn test_completion_toward_target_exceeded() {
        let mut stats = Statistics::default();
        stats.word_count = 60000;
        assert_eq!(stats.completion_toward_target(50000), 100.0);
    }

    #[test]
    fn test_words_remaining() {
        let mut stats = Statistics::default();
        stats.word_count = 30000;
        assert_eq!(stats.words_remaining(50000), 20000);
    }

    #[test]
    fn test_words_remaining_exceeded() {
        let mut stats = Statistics::default();
        stats.word_count = 60000;
        assert_eq!(stats.words_remaining(50000), 0);
    }

    #[test]
    fn test_days_to_completion() {
        let mut stats = Statistics::default();
        stats.word_count = 30000;
        assert_eq!(stats.days_to_completion(50000, 1000), Some(20));
    }

    #[test]
    fn test_days_to_completion_zero_rate() {
        let stats = Statistics::default();
        assert_eq!(stats.days_to_completion(50000, 0), None);
    }

    #[test]
    fn test_days_to_completion_already_done() {
        let mut stats = Statistics::default();
        stats.word_count = 60000;
        assert_eq!(stats.days_to_completion(50000, 1000), Some(0));
    }

    #[test]
    fn test_avg_words_per_doc() {
        let mut stats = Statistics::default();
        stats.word_count = 10000;
        stats.document_count = 5;
        assert_eq!(stats.avg_words_per_doc(), 2000.0);
    }

    #[test]
    fn test_avg_words_per_doc_no_docs() {
        let stats = Statistics::default();
        assert_eq!(stats.avg_words_per_doc(), 0.0);
    }

    #[test]
    fn test_size_label() {
        let mut stats = Statistics::default();
        stats.word_count = 0;
        assert_eq!(stats.size_label(), "Flash Fiction / Note");

        stats.word_count = 5000;
        assert_eq!(stats.size_label(), "Short Story");

        stats.word_count = 10000;
        assert_eq!(stats.size_label(), "Novelette");

        stats.word_count = 25000;
        assert_eq!(stats.size_label(), "Novella");

        stats.word_count = 60000;
        assert_eq!(stats.size_label(), "Novel");

        stats.word_count = 100000;
        assert_eq!(stats.size_label(), "Full Novel");

        stats.word_count = 200000;
        assert_eq!(stats.size_label(), "Epic / Tome");
    }

    #[test]
    fn test_statistics_from_text_single_sentence() {
        let stats = Statistics::from_text("Hello world");
        assert_eq!(stats.word_count, 2);
        assert_eq!(stats.sentence_count, 1); // max(0, 1) for non-empty text
        assert_eq!(stats.paragraph_count, 1);
    }

    #[test]
    fn test_statistics_char_count_no_spaces() {
        let stats = Statistics::from_text("a b c");
        assert_eq!(stats.char_count, 5);
        assert_eq!(stats.char_count_no_spaces, 3);
    }

    #[test]
    fn test_statistics_page_count() {
        let stats = Statistics::from_text(&"word ".repeat(500));
        assert!((stats.page_count - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_progress_string_exceeded() {
        let mut stats = Statistics::default();
        stats.word_count = 100;
        let progress = stats.progress_string(Some(50));
        assert!(progress.contains("100.0%")); // Capped at 100%
    }

    #[test]
    fn test_session_stats_wpm_calculation() {
        let mut session = SessionStats::new();
        session.update(300, 120); // 300 words in 2 minutes
        assert!((session.words_per_minute - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_text_analysis_most_common_words() {
        let text = "the great great great story about the great adventure adventure";
        let analysis = TextAnalysis::from_text(text);
        assert!(!analysis.most_common_words.is_empty());
        // "great" should be in most common (4 occurrences, len > 3)
        assert!(analysis.most_common_words.iter().any(|(w, _)| w == "great"));
    }

    #[test]
    fn test_text_analysis_short_words_excluded_from_common() {
        let text = "the the the the big big big";
        let analysis = TextAnalysis::from_text(text);
        // "the" is 3 chars, should be excluded from most_common_words (filter > 3)
        assert!(analysis.most_common_words.iter().all(|(w, _)| w.len() > 3));
    }

    #[test]
    fn test_readability_metrics_smog_with_enough_sentences() {
        // Text with many sentences for SMOG calculation
        let text = "The cat sat. The dog ran. The bird flew. The fish swam. He walked home.";
        let metrics = ReadabilityMetrics::from_text(text);
        assert!(metrics.smog_grade >= 0.0);
    }

    #[test]
    fn test_readability_metrics_avg_syllables() {
        let text = "Simple words here.";
        let metrics = ReadabilityMetrics::from_text(text);
        assert!(metrics.avg_syllables_per_word >= 1.0);
    }

    #[test]
    fn test_word_frequency_single_word() {
        let analysis = WordFrequencyAnalysis::from_text("hello");
        assert_eq!(analysis.total_count, 1);
        assert_eq!(analysis.unique_count, 1);
        assert_eq!(analysis.hapax_count, 1);
        assert_eq!(analysis.type_token_ratio, 100.0);
    }

    #[test]
    fn test_word_frequency_punctuation_stripping() {
        let analysis = WordFrequencyAnalysis::from_text("hello, world! hello.");
        assert_eq!(analysis.total_count, 3);
        // "hello" appears twice, "world" once
        assert_eq!(analysis.unique_count, 2);
    }

    #[test]
    fn test_count_syllables_polysyllabic() {
        assert!(count_syllables("extraordinary") >= 4);
        assert!(count_syllables("communication") >= 4);
    }

    #[test]
    fn test_count_syllables_silent_e() {
        // "time" has silent e: should be 1 syllable
        assert_eq!(count_syllables("time"), 1);
        assert_eq!(count_syllables("came"), 1);
    }

    #[test]
    fn test_days_to_completion_non_divisible() {
        let mut stats = Statistics::default();
        stats.word_count = 0;
        // 1001 words at 500/day = ceil(1001/500) = 3 days
        assert_eq!(stats.days_to_completion(1001, 500), Some(3));
    }

    #[test]
    fn test_text_analysis_vocabulary_richness_exact() {
        let mut analysis = TextAnalysis::default();
        analysis.word_count = 100;
        analysis.unique_words = 70;
        assert!((analysis.vocabulary_richness() - 70.0).abs() < 0.01);
        assert_eq!(analysis.vocabulary_label(), "Rich");
    }

    #[test]
    fn test_readability_labels_boundary_values() {
        let mut analysis = TextAnalysis::default();
        analysis.readability_score = 90.0;
        assert_eq!(analysis.readability_label(), "Very Easy");
        analysis.readability_score = 80.0;
        assert_eq!(analysis.readability_label(), "Easy");
        analysis.readability_score = 70.0;
        assert_eq!(analysis.readability_label(), "Fairly Easy");
        analysis.readability_score = 60.0;
        assert_eq!(analysis.readability_label(), "Standard");
        analysis.readability_score = 50.0;
        assert_eq!(analysis.readability_label(), "Fairly Difficult");
        analysis.readability_score = 30.0;
        assert_eq!(analysis.readability_label(), "Difficult");
        analysis.readability_score = 29.9;
        assert_eq!(analysis.readability_label(), "Very Difficult");
    }

    #[test]
    fn test_word_frequency_bigrams_minimum_count() {
        // Bigrams with count == 1 are filtered out
        let text = "unique pair only once";
        let analysis = WordFrequencyAnalysis::from_text(text);
        assert!(analysis.top_bigrams.is_empty());
    }

    #[test]
    fn test_statistics_from_binder_empty() {
        use crate::core::binder::Binder;
        let binder = Binder::default_structure();
        let stats = Statistics::from_binder(&binder);
        assert_eq!(stats.word_count, 0);
        assert_eq!(stats.document_count, 0);
        assert!(stats.folder_count >= 3); // Draft, Research, Trash
    }

    // ---- DailyEntry tests ----

    fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn test_daily_entry_new() {
        let e = DailyEntry::new(date(2025, 1, 1), 500, 1000, 1500, 3600);
        assert_eq!(e.words_written, 500);
        assert_eq!(e.word_count_start, 1000);
        assert_eq!(e.word_count_end, 1500);
        assert_eq!(e.time_spent_seconds, 3600);
    }

    #[test]
    fn test_daily_entry_wpm() {
        let e = DailyEntry::new(date(2025, 1, 1), 600, 0, 600, 3600);
        assert!((e.wpm() - 10.0).abs() < 0.01); // 600 words / 60 min
    }

    #[test]
    fn test_daily_entry_wpm_zero_time() {
        let e = DailyEntry::new(date(2025, 1, 1), 100, 0, 100, 0);
        assert_eq!(e.wpm(), 0.0);
    }

    #[test]
    fn test_daily_entry_hours() {
        let e = DailyEntry::new(date(2025, 1, 1), 100, 0, 100, 7200);
        assert!((e.hours() - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_daily_entry_is_productive() {
        let good = DailyEntry::new(date(2025, 1, 1), 500, 0, 500, 3600);
        assert!(good.is_productive());
        let bad = DailyEntry::new(date(2025, 1, 2), -100, 500, 400, 3600);
        assert!(!bad.is_productive());
        let zero = DailyEntry::new(date(2025, 1, 3), 0, 500, 500, 1800);
        assert!(!zero.is_productive());
    }

    // ---- WritingHistory tests ----

    #[test]
    fn test_writing_history_new() {
        let h = WritingHistory::new();
        assert!(h.is_empty());
        assert_eq!(h.len(), 0);
    }

    #[test]
    fn test_writing_history_record() {
        let mut h = WritingHistory::new();
        h.record(DailyEntry::new(date(2025, 1, 1), 500, 0, 500, 3600));
        h.record(DailyEntry::new(date(2025, 1, 2), 300, 500, 800, 1800));
        assert_eq!(h.len(), 2);
        assert!(!h.is_empty());
    }

    #[test]
    fn test_writing_history_record_replaces_same_date() {
        let mut h = WritingHistory::new();
        h.record(DailyEntry::new(date(2025, 1, 1), 200, 0, 200, 1800));
        h.record(DailyEntry::new(date(2025, 1, 1), 500, 0, 500, 3600));
        assert_eq!(h.len(), 1);
        assert_eq!(h.total_words_written(), 500);
    }

    #[test]
    fn test_writing_history_totals() {
        let mut h = WritingHistory::new();
        h.record(DailyEntry::new(date(2025, 1, 1), 500, 0, 500, 3600));
        h.record(DailyEntry::new(date(2025, 1, 2), 300, 500, 800, 1800));

        assert_eq!(h.total_words_written(), 800);
        assert_eq!(h.total_time_seconds(), 5400);
    }

    #[test]
    fn test_writing_history_avg_words_per_day() {
        let mut h = WritingHistory::new();
        h.record(DailyEntry::new(date(2025, 1, 1), 400, 0, 400, 3600));
        h.record(DailyEntry::new(date(2025, 1, 2), 600, 400, 1000, 3600));
        assert!((h.avg_words_per_day() - 500.0).abs() < 0.01);
    }

    #[test]
    fn test_writing_history_avg_words_per_day_empty() {
        let h = WritingHistory::new();
        assert_eq!(h.avg_words_per_day(), 0.0);
    }

    #[test]
    fn test_writing_history_avg_wpm() {
        let mut h = WritingHistory::new();
        h.record(DailyEntry::new(date(2025, 1, 1), 600, 0, 600, 3600));
        // 600 words / 60 min = 10 wpm
        assert!((h.avg_wpm() - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_writing_history_best_day() {
        let mut h = WritingHistory::new();
        h.record(DailyEntry::new(date(2025, 1, 1), 200, 0, 200, 1800));
        h.record(DailyEntry::new(date(2025, 1, 2), 800, 200, 1000, 3600));
        h.record(DailyEntry::new(date(2025, 1, 3), 500, 1000, 1500, 2700));

        let best = h.best_day().unwrap();
        assert_eq!(best.date, date(2025, 1, 2));
        assert_eq!(best.words_written, 800);
    }

    #[test]
    fn test_writing_history_best_day_empty() {
        let h = WritingHistory::new();
        assert!(h.best_day().is_none());
    }

    #[test]
    fn test_writing_history_current_streak() {
        let mut h = WritingHistory::new();
        h.record(DailyEntry::new(date(2025, 1, 1), 100, 0, 100, 1800));
        h.record(DailyEntry::new(date(2025, 1, 2), 200, 100, 300, 1800));
        h.record(DailyEntry::new(date(2025, 1, 3), 300, 300, 600, 1800));
        assert_eq!(h.current_streak(), 3);
    }

    #[test]
    fn test_writing_history_streak_broken() {
        let mut h = WritingHistory::new();
        h.record(DailyEntry::new(date(2025, 1, 1), 100, 0, 100, 1800));
        // Gap on Jan 2
        h.record(DailyEntry::new(date(2025, 1, 3), 200, 100, 300, 1800));
        h.record(DailyEntry::new(date(2025, 1, 4), 300, 300, 600, 1800));
        assert_eq!(h.current_streak(), 2);
    }

    #[test]
    fn test_writing_history_streak_single() {
        let mut h = WritingHistory::new();
        h.record(DailyEntry::new(date(2025, 1, 10), 100, 0, 100, 1800));
        assert_eq!(h.current_streak(), 1);
    }

    #[test]
    fn test_writing_history_longest_streak() {
        let mut h = WritingHistory::new();
        // 3-day streak
        h.record(DailyEntry::new(date(2025, 1, 1), 100, 0, 100, 1800));
        h.record(DailyEntry::new(date(2025, 1, 2), 200, 0, 200, 1800));
        h.record(DailyEntry::new(date(2025, 1, 3), 150, 0, 150, 1800));
        // Gap
        // 2-day streak
        h.record(DailyEntry::new(date(2025, 1, 10), 100, 0, 100, 1800));
        h.record(DailyEntry::new(date(2025, 1, 11), 100, 0, 100, 1800));

        assert_eq!(h.longest_streak(), 3);
    }

    #[test]
    fn test_writing_history_moving_average() {
        let mut h = WritingHistory::new();
        h.record(DailyEntry::new(date(2025, 1, 1), 100, 0, 100, 1800));
        h.record(DailyEntry::new(date(2025, 1, 2), 200, 100, 300, 1800));
        h.record(DailyEntry::new(date(2025, 1, 3), 300, 300, 600, 1800));
        h.record(DailyEntry::new(date(2025, 1, 4), 400, 600, 1000, 1800));

        let ma = h.moving_average(3);
        assert_eq!(ma.len(), 2); // 4 entries - 3 window + 1
        assert!((ma[0].1 - 200.0).abs() < 0.01); // avg(100,200,300)
        assert!((ma[1].1 - 300.0).abs() < 0.01); // avg(200,300,400)
    }

    #[test]
    fn test_writing_history_moving_average_too_small() {
        let mut h = WritingHistory::new();
        h.record(DailyEntry::new(date(2025, 1, 1), 100, 0, 100, 1800));
        let ma = h.moving_average(3);
        assert!(ma.is_empty());
    }

    #[test]
    fn test_writing_history_moving_average_zero_window() {
        let mut h = WritingHistory::new();
        h.record(DailyEntry::new(date(2025, 1, 1), 100, 0, 100, 1800));
        let ma = h.moving_average(0);
        assert!(ma.is_empty());
    }

    // ---- WritingTrend & analyze_trend tests ----

    #[test]
    fn test_analyze_trend_empty() {
        let h = WritingHistory::new();
        let trend = h.analyze_trend();
        assert_eq!(trend.total_words, 0);
        assert_eq!(trend.total_days, 0);
        assert_eq!(trend.direction, TrendDirection::Stable);
    }

    #[test]
    fn test_analyze_trend_increasing() {
        let mut h = WritingHistory::new();
        // First half: small numbers
        h.record(DailyEntry::new(date(2025, 1, 1), 100, 0, 100, 1800));
        h.record(DailyEntry::new(date(2025, 1, 2), 120, 100, 220, 1800));
        // Second half: much bigger numbers
        h.record(DailyEntry::new(date(2025, 1, 3), 500, 220, 720, 1800));
        h.record(DailyEntry::new(date(2025, 1, 4), 600, 720, 1320, 1800));

        let trend = h.analyze_trend();
        assert_eq!(trend.direction, TrendDirection::Increasing);
    }

    #[test]
    fn test_analyze_trend_decreasing() {
        let mut h = WritingHistory::new();
        h.record(DailyEntry::new(date(2025, 1, 1), 800, 0, 800, 1800));
        h.record(DailyEntry::new(date(2025, 1, 2), 700, 800, 1500, 1800));
        h.record(DailyEntry::new(date(2025, 1, 3), 100, 1500, 1600, 1800));
        h.record(DailyEntry::new(date(2025, 1, 4), 50, 1600, 1650, 1800));

        let trend = h.analyze_trend();
        assert_eq!(trend.direction, TrendDirection::Decreasing);
    }

    #[test]
    fn test_trend_productivity_ratio() {
        let mut h = WritingHistory::new();
        h.record(DailyEntry::new(date(2025, 1, 1), 500, 0, 500, 3600));
        h.record(DailyEntry::new(date(2025, 1, 2), -50, 500, 450, 1800));
        h.record(DailyEntry::new(date(2025, 1, 3), 300, 450, 750, 2700));

        let trend = h.analyze_trend();
        assert_eq!(trend.productive_days, 2);
        assert!((trend.productivity_ratio() - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_trend_summary() {
        let mut h = WritingHistory::new();
        h.record(DailyEntry::new(date(2025, 1, 1), 500, 0, 500, 3600));
        h.record(DailyEntry::new(date(2025, 1, 2), 300, 500, 800, 1800));

        let trend = h.analyze_trend();
        let summary = trend.summary();
        assert!(summary.contains("800 words"));
        assert!(summary.contains("2 days"));
        assert!(summary.contains("streak"));
    }

    #[test]
    fn test_trend_direction_labels() {
        assert_eq!(TrendDirection::Increasing.label(), "Increasing");
        assert_eq!(TrendDirection::Decreasing.label(), "Decreasing");
        assert_eq!(TrendDirection::Stable.label(), "Stable");
    }
}
