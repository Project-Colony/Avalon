use std::collections::HashMap;
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

    /// Total number of days tracked
    pub fn total_days(&self) -> usize {
        self.entries.len()
    }

    /// Activity ratio (fraction of tracked days with positive words)
    pub fn activity_ratio(&self) -> f64 {
        if self.entries.is_empty() {
            return 0.0;
        }
        self.active_days() as f64 / self.entries.len() as f64
    }

    /// Words written in the last 30 days
    pub fn words_this_month(&self) -> i64 {
        let month = self.recent(30);
        month.iter().map(|e| e.words_written).sum()
    }

    /// Most productive day of the week (0=Mon, 6=Sun)
    pub fn most_productive_weekday(&self) -> Option<chrono::Weekday> {
        use chrono::Datelike;
        let mut totals: HashMap<chrono::Weekday, i64> = HashMap::new();
        for entry in &self.entries {
            *totals.entry(entry.date.weekday()).or_insert(0) += entry.words_written.max(0);
        }
        totals.into_iter().max_by_key(|(_, v)| *v).map(|(k, _)| k)
    }

    /// Check if user has written today
    pub fn wrote_today(&self) -> bool {
        self.today().map_or(false, |e| e.words_written > 0)
    }

    /// Average writing time per session in minutes
    pub fn average_session_minutes(&self) -> f64 {
        let active: Vec<_> = self.entries.iter().filter(|e| e.time_spent_seconds > 0).collect();
        if active.is_empty() {
            return 0.0;
        }
        let total: u64 = active.iter().map(|e| e.time_spent_seconds).sum();
        (total as f64 / 60.0) / active.len() as f64
    }

    /// Format total time as a human-readable string
    pub fn total_time_display(&self) -> String {
        let secs = self.total_time_seconds();
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        if hours > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}m", mins)
        }
    }

    /// Get a summary string for display
    pub fn summary(&self) -> String {
        format!(
            "{} words over {} days ({} active), streak: {}",
            self.total_words_written(),
            self.total_days(),
            self.active_days(),
            self.current_streak()
        )
    }

    /// Get the worst day (fewest words written, among active days)
    pub fn worst_active_day(&self) -> Option<&DailyEntry> {
        self.entries.iter()
            .filter(|e| e.words_written > 0)
            .min_by_key(|e| e.words_written)
    }

    /// Total pages written (assuming 250 words/page)
    pub fn total_pages(&self) -> f64 {
        self.total_words_written().max(0) as f64 / 250.0
    }

    /// Variance in daily word counts (for consistency analysis)
    pub fn daily_variance(&self) -> f64 {
        if self.entries.len() < 2 {
            return 0.0;
        }
        let avg = self.average_words_per_day();
        let sum_sq: f64 = self.entries.iter()
            .map(|e| {
                let diff = e.words_written as f64 - avg;
                diff * diff
            })
            .sum();
        sum_sq / self.entries.len() as f64
    }

    /// Standard deviation of daily word counts
    pub fn daily_std_dev(&self) -> f64 {
        self.daily_variance().sqrt()
    }

    /// Consistency score (0-100, higher = more consistent writing)
    /// Based on coefficient of variation: lower variation = higher consistency
    pub fn consistency_score(&self) -> f64 {
        let avg = self.average_words_per_day();
        if avg <= 0.0 || self.entries.len() < 2 {
            return 0.0;
        }
        let cv = self.daily_std_dev() / avg;
        // Convert to a 0-100 score: CV of 0 = 100, CV of 2+ = 0
        ((1.0 - cv / 2.0) * 100.0).max(0.0).min(100.0)
    }

    /// Consistency label based on score
    pub fn consistency_label(&self) -> &str {
        let score = self.consistency_score();
        if score >= 80.0 { "Very Consistent" }
        else if score >= 60.0 { "Consistent" }
        else if score >= 40.0 { "Moderate" }
        else if score >= 20.0 { "Irregular" }
        else { "Very Irregular" }
    }

    /// Average time per word in seconds
    pub fn avg_seconds_per_word(&self) -> f64 {
        let total_words = self.total_words_written();
        if total_words <= 0 {
            return 0.0;
        }
        self.total_time_seconds() as f64 / total_words as f64
    }

    /// Estimate days to reach a word count target at current pace
    pub fn estimated_days_to_target(&self, target_words: usize, current_words: usize) -> Option<f64> {
        let avg = self.average_words_per_day();
        if avg <= 0.0 || current_words >= target_words {
            return None;
        }
        let remaining = target_words - current_words;
        Some(remaining as f64 / avg)
    }

    /// Words written in the last N days (configurable)
    pub fn words_in_last_n_days(&self, n: usize) -> i64 {
        self.recent(n).iter().map(|e| e.words_written).sum()
    }

    /// Productivity trend: average of last 7 days vs previous 7 days
    /// Returns a ratio (>1 = improving, <1 = declining)
    pub fn productivity_trend(&self) -> f64 {
        if self.entries.len() < 14 {
            return 1.0;
        }
        let recent_7: i64 = self.recent(7).iter().map(|e| e.words_written.max(0)).sum();
        let len = self.entries.len();
        let prev_7: i64 = self.entries[len - 14..len - 7].iter()
            .map(|e| e.words_written.max(0))
            .sum();
        if prev_7 == 0 { return 1.0; }
        recent_7 as f64 / prev_7 as f64
    }

    /// Productivity trend label
    pub fn trend_label(&self) -> &str {
        let trend = self.productivity_trend();
        if trend >= 1.5 { "Accelerating" }
        else if trend >= 1.1 { "Improving" }
        else if trend >= 0.9 { "Stable" }
        else if trend >= 0.5 { "Declining" }
        else { "Stalling" }
    }
}

impl DailyEntry {
    /// Time spent as a human-readable string
    pub fn time_display(&self) -> String {
        let mins = self.time_spent_seconds / 60;
        if mins >= 60 {
            format!("{}h {}m", mins / 60, mins % 60)
        } else {
            format!("{}m", mins)
        }
    }

    /// Words per minute for this session
    pub fn wpm(&self) -> f64 {
        if self.time_spent_seconds < 60 {
            return 0.0;
        }
        self.words_written.max(0) as f64 / (self.time_spent_seconds as f64 / 60.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(date: NaiveDate, words: i64, time: u64) -> DailyEntry {
        DailyEntry {
            date,
            word_count_start: 1000,
            word_count_end: (1000 + words.max(0)) as usize,
            words_written: words,
            time_spent_seconds: time,
        }
    }

    #[test]
    fn test_writing_history_new() {
        let history = WritingHistory::new();
        assert!(history.entries.is_empty());
        assert_eq!(history.total_words_written(), 0);
        assert_eq!(history.total_days(), 0);
        assert_eq!(history.current_streak(), 0);
    }

    #[test]
    fn test_record_words() {
        let mut history = WritingHistory::new();
        history.record(100, 60);
        assert_eq!(history.total_days(), 1);
        // Record again for the same day
        history.record(200, 60);
        assert_eq!(history.total_days(), 1);
    }

    #[test]
    fn test_total_words_written() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 500, 3600));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 300, 1800));
        assert_eq!(history.total_words_written(), 800);
    }

    #[test]
    fn test_average_words_per_day() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 400, 3600));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 200, 1800));
        assert!((history.average_words_per_day() - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_current_streak() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 100, 60));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 0, 0));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(), 200, 120));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 4).unwrap(), 150, 90));
        assert_eq!(history.current_streak(), 2);
    }

    #[test]
    fn test_longest_streak() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 100, 60));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 200, 60));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(), 300, 60));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 4).unwrap(), 0, 0));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(), 100, 60));
        assert_eq!(history.longest_streak(), 3);
    }

    #[test]
    fn test_best_day() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 100, 60));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 500, 120));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(), 200, 90));
        let best = history.best_day().unwrap();
        assert_eq!(best.words_written, 500);
    }

    #[test]
    fn test_recent() {
        let mut history = WritingHistory::new();
        for i in 1..=10 {
            history.entries.push(make_entry(
                NaiveDate::from_ymd_opt(2024, 1, i).unwrap(), i as i64 * 100, 60));
        }
        let recent = history.recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].words_written, 800);
    }

    #[test]
    fn test_active_days() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 100, 60));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 0, 0));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(), 200, 60));
        assert_eq!(history.active_days(), 2);
        assert_eq!(history.total_days(), 3);
    }

    #[test]
    fn test_activity_ratio() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 100, 60));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 0, 0));
        assert!((history.activity_ratio() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_total_time_display() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 100, 7200));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 200, 1800));
        let display = history.total_time_display();
        assert_eq!(display, "2h 30m");
    }

    #[test]
    fn test_daily_entry_wpm() {
        let entry = make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 300, 600);
        assert!((entry.wpm() - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_daily_entry_time_display() {
        let entry = make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 100, 5400);
        assert_eq!(entry.time_display(), "1h 30m");
    }

    #[test]
    fn test_summary() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 500, 3600));
        let summary = history.summary();
        assert!(summary.contains("500 words"));
        assert!(summary.contains("1 days"));
    }

    #[test]
    fn test_wrote_today() {
        let mut history = WritingHistory::new();
        assert!(!history.wrote_today());

        // Record something for today
        history.record(100, 60);
        // Initially words_written is 0 for the first record (start == end)
        assert!(!history.wrote_today());

        // Record a word count increase
        history.record(200, 60);
        assert!(history.wrote_today());
    }

    #[test]
    fn test_today_entry() {
        let mut history = WritingHistory::new();
        assert!(history.today().is_none());

        history.record(100, 60);
        assert!(history.today().is_some());
    }

    #[test]
    fn test_average_wpm_short_session() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 100, 30));
        // Less than 60 seconds should return 0
        assert_eq!(history.average_wpm(), 0.0);
    }

    #[test]
    fn test_average_wpm() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 300, 600)); // 30 wpm
        let wpm = history.average_wpm();
        assert!((wpm - 30.0).abs() < 0.1);
    }

    #[test]
    fn test_words_this_week() {
        let mut history = WritingHistory::new();
        for i in 1..=7 {
            history.entries.push(make_entry(
                NaiveDate::from_ymd_opt(2024, 1, i).unwrap(), 100, 60));
        }
        assert_eq!(history.words_this_week(), 700);
    }

    #[test]
    fn test_words_this_month() {
        let mut history = WritingHistory::new();
        for i in 1..=10 {
            history.entries.push(make_entry(
                NaiveDate::from_ymd_opt(2024, 1, i).unwrap(), 50, 60));
        }
        assert_eq!(history.words_this_month(), 500);
    }

    #[test]
    fn test_most_productive_weekday() {
        let mut history = WritingHistory::new();
        // Monday
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 100, 60));
        // Tuesday
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 500, 120));
        // Wednesday
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(), 200, 90));

        let day = history.most_productive_weekday().unwrap();
        assert_eq!(day, chrono::Weekday::Tue);
    }

    #[test]
    fn test_most_productive_weekday_empty() {
        let history = WritingHistory::new();
        assert!(history.most_productive_weekday().is_none());
    }

    #[test]
    fn test_average_session_minutes() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 100, 1800)); // 30 min
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 200, 3600)); // 60 min

        let avg = history.average_session_minutes();
        assert!((avg - 45.0).abs() < 0.01);
    }

    #[test]
    fn test_average_session_minutes_empty() {
        let history = WritingHistory::new();
        assert_eq!(history.average_session_minutes(), 0.0);
    }

    #[test]
    fn test_average_session_minutes_skip_zero_time() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 100, 600));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 0, 0)); // No time
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(), 200, 600));

        let avg = history.average_session_minutes();
        // Only 2 active sessions: (600 + 600) / 60 / 2 = 10
        assert!((avg - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_daily_entry_wpm_zero_time() {
        let entry = make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 300, 30);
        assert_eq!(entry.wpm(), 0.0); // Less than 60 seconds
    }

    #[test]
    fn test_daily_entry_time_display_short() {
        let entry = make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 100, 300);
        assert_eq!(entry.time_display(), "5m");
    }

    #[test]
    fn test_total_time_display_minutes_only() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 100, 1800));
        assert_eq!(history.total_time_display(), "30m");
    }

    #[test]
    fn test_activity_ratio_empty() {
        let history = WritingHistory::new();
        assert_eq!(history.activity_ratio(), 0.0);
    }

    #[test]
    fn test_activity_ratio_all_active() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 100, 60));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 200, 60));
        assert!((history.activity_ratio() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_total_time_seconds() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 100, 3600));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 200, 1800));
        assert_eq!(history.total_time_seconds(), 5400);
    }

    #[test]
    fn test_average_words_per_day_empty() {
        let history = WritingHistory::new();
        assert_eq!(history.average_words_per_day(), 0.0);
    }

    #[test]
    fn test_best_day_empty() {
        let history = WritingHistory::new();
        assert!(history.best_day().is_none());
    }

    #[test]
    fn test_streak_all_positive() {
        let mut history = WritingHistory::new();
        for i in 1..=5 {
            history.entries.push(make_entry(
                NaiveDate::from_ymd_opt(2024, 1, i).unwrap(), 100, 60));
        }
        assert_eq!(history.current_streak(), 5);
        assert_eq!(history.longest_streak(), 5);
    }

    #[test]
    fn test_worst_active_day() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 500, 60));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 50, 30));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(), 200, 60));
        let worst = history.worst_active_day().unwrap();
        assert_eq!(worst.words_written, 50);
    }

    #[test]
    fn test_worst_active_day_empty() {
        let history = WritingHistory::new();
        assert!(history.worst_active_day().is_none());
    }

    #[test]
    fn test_total_pages() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 500, 60));
        assert!((history.total_pages() - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_daily_variance() {
        let mut history = WritingHistory::new();
        // All same = 0 variance
        for i in 1..=5 {
            history.entries.push(make_entry(
                NaiveDate::from_ymd_opt(2024, 1, i).unwrap(), 100, 60));
        }
        assert!((history.daily_variance()).abs() < 0.01);
    }

    #[test]
    fn test_daily_variance_different() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 100, 60));
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 500, 60));
        assert!(history.daily_variance() > 0.0);
    }

    #[test]
    fn test_daily_std_dev() {
        let mut history = WritingHistory::new();
        for i in 1..=3 {
            history.entries.push(make_entry(
                NaiveDate::from_ymd_opt(2024, 1, i).unwrap(), 100, 60));
        }
        assert!((history.daily_std_dev()).abs() < 0.01);
    }

    #[test]
    fn test_consistency_score_perfect() {
        let mut history = WritingHistory::new();
        for i in 1..=10 {
            history.entries.push(make_entry(
                NaiveDate::from_ymd_opt(2024, 1, i).unwrap(), 100, 60));
        }
        assert!((history.consistency_score() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_consistency_score_empty() {
        let history = WritingHistory::new();
        assert_eq!(history.consistency_score(), 0.0);
    }

    #[test]
    fn test_consistency_label() {
        let mut history = WritingHistory::new();
        for i in 1..=10 {
            history.entries.push(make_entry(
                NaiveDate::from_ymd_opt(2024, 1, i).unwrap(), 100, 60));
        }
        assert_eq!(history.consistency_label(), "Very Consistent");
    }

    #[test]
    fn test_avg_seconds_per_word() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 100, 600));
        // 600 seconds / 100 words = 6 seconds per word
        assert!((history.avg_seconds_per_word() - 6.0).abs() < 0.01);
    }

    #[test]
    fn test_avg_seconds_per_word_empty() {
        let history = WritingHistory::new();
        assert_eq!(history.avg_seconds_per_word(), 0.0);
    }

    #[test]
    fn test_estimated_days_to_target() {
        let mut history = WritingHistory::new();
        for i in 1..=10 {
            history.entries.push(make_entry(
                NaiveDate::from_ymd_opt(2024, 1, i).unwrap(), 100, 60));
        }
        // Average: 100 words/day, need 500 more to reach 1500 from 1000
        let days = history.estimated_days_to_target(1500, 1000).unwrap();
        assert!((days - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_estimated_days_to_target_already_met() {
        let mut history = WritingHistory::new();
        history.entries.push(make_entry(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 100, 60));
        assert!(history.estimated_days_to_target(1000, 1500).is_none());
    }

    #[test]
    fn test_estimated_days_to_target_no_progress() {
        let history = WritingHistory::new();
        assert!(history.estimated_days_to_target(1000, 0).is_none());
    }

    #[test]
    fn test_words_in_last_n_days() {
        let mut history = WritingHistory::new();
        for i in 1..=10 {
            history.entries.push(make_entry(
                NaiveDate::from_ymd_opt(2024, 1, i).unwrap(), 100, 60));
        }
        assert_eq!(history.words_in_last_n_days(5), 500);
        assert_eq!(history.words_in_last_n_days(3), 300);
    }

    #[test]
    fn test_productivity_trend_stable() {
        let mut history = WritingHistory::new();
        for i in 1..=14 {
            history.entries.push(make_entry(
                NaiveDate::from_ymd_opt(2024, 1, i).unwrap(), 100, 60));
        }
        assert!((history.productivity_trend() - 1.0).abs() < 0.01);
        assert_eq!(history.trend_label(), "Stable");
    }

    #[test]
    fn test_productivity_trend_insufficient_data() {
        let mut history = WritingHistory::new();
        for i in 1..=5 {
            history.entries.push(make_entry(
                NaiveDate::from_ymd_opt(2024, 1, i).unwrap(), 100, 60));
        }
        assert!((history.productivity_trend() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_productivity_trend_improving() {
        let mut history = WritingHistory::new();
        // Prev 7 days: 50 words/day
        for i in 1..=7 {
            history.entries.push(make_entry(
                NaiveDate::from_ymd_opt(2024, 1, i).unwrap(), 50, 60));
        }
        // Recent 7 days: 200 words/day
        for i in 8..=14 {
            history.entries.push(make_entry(
                NaiveDate::from_ymd_opt(2024, 1, i).unwrap(), 200, 60));
        }
        assert!(history.productivity_trend() > 1.5);
        assert_eq!(history.trend_label(), "Accelerating");
    }

    #[test]
    fn test_trend_label_declining() {
        let mut history = WritingHistory::new();
        // Prev 7 days: 200 words/day
        for i in 1..=7 {
            history.entries.push(make_entry(
                NaiveDate::from_ymd_opt(2024, 1, i).unwrap(), 200, 60));
        }
        // Recent 7 days: 50 words/day
        for i in 8..=14 {
            history.entries.push(make_entry(
                NaiveDate::from_ymd_opt(2024, 1, i).unwrap(), 50, 60));
        }
        assert!(history.productivity_trend() < 0.5);
        assert_eq!(history.trend_label(), "Stalling");
    }
}
