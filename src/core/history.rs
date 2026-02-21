use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, Utc};

/// Tracks daily writing history for the project
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WritingHistory {
    pub entries: Vec<DailyEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyEntry {
    pub date: NaiveDate,
    pub word_count_start: usize,
    pub word_count_end: usize,
    pub words_written: i64,
    pub time_spent_seconds: u64,
}

impl WritingHistory {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Record word count for today
    pub fn record(&mut self, current_word_count: usize, seconds_spent: u64) {
        let today = Utc::now().date_naive();

        if let Some(entry) = self.entries.last_mut() {
            if entry.date == today {
                entry.word_count_end = current_word_count;
                entry.words_written = current_word_count as i64 - entry.word_count_start as i64;
                entry.time_spent_seconds += seconds_spent;
                return;
            }
        }

        // New day
        self.entries.push(DailyEntry {
            date: today,
            word_count_start: current_word_count,
            word_count_end: current_word_count,
            words_written: 0,
            time_spent_seconds: seconds_spent,
        });
    }

    /// Get the last N days of history
    pub fn recent(&self, days: usize) -> &[DailyEntry] {
        let start = self.entries.len().saturating_sub(days);
        &self.entries[start..]
    }

    /// Total words written across all recorded days
    pub fn total_words_written(&self) -> i64 {
        self.entries.iter().map(|e| e.words_written).sum()
    }

    /// Total time spent writing in seconds
    pub fn total_time_seconds(&self) -> u64 {
        self.entries.iter().map(|e| e.time_spent_seconds).sum()
    }

    /// Average words per day (over recorded days)
    pub fn average_words_per_day(&self) -> f64 {
        if self.entries.is_empty() {
            return 0.0;
        }
        self.total_words_written() as f64 / self.entries.len() as f64
    }

    /// Best day (most words written)
    pub fn best_day(&self) -> Option<&DailyEntry> {
        self.entries.iter().max_by_key(|e| e.words_written)
    }

    /// Current streak (consecutive days with positive words)
    pub fn current_streak(&self) -> usize {
        let mut streak = 0;
        for entry in self.entries.iter().rev() {
            if entry.words_written > 0 {
                streak += 1;
            } else {
                break;
            }
        }
        streak
    }

    /// Longest streak ever achieved
    pub fn longest_streak(&self) -> usize {
        let mut best = 0;
        let mut current = 0;
        for entry in &self.entries {
            if entry.words_written > 0 {
                current += 1;
                best = best.max(current);
            } else {
                current = 0;
            }
        }
        best
    }

    /// Average words per minute across all sessions
    pub fn average_wpm(&self) -> f64 {
        let total_time = self.total_time_seconds();
        if total_time < 60 {
            return 0.0;
        }
        let total_positive: i64 = self.entries.iter()
            .map(|e| e.words_written.max(0))
            .sum();
        total_positive as f64 / (total_time as f64 / 60.0)
    }

    /// Words written in the last 7 days
    pub fn words_this_week(&self) -> i64 {
        let week = self.recent(7);
        week.iter().map(|e| e.words_written).sum()
    }

    /// Get today's entry, if any
    pub fn today(&self) -> Option<&DailyEntry> {
        let today = Utc::now().date_naive();
        self.entries.iter().rev().find(|e| e.date == today)
    }

    /// Number of days with writing activity
    pub fn active_days(&self) -> usize {
        self.entries.iter().filter(|e| e.words_written > 0).count()
    }
}
