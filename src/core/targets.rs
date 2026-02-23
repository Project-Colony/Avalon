use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// Per-document word count targets and progress tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentTargets {
    /// Map of document ID -> target configuration
    targets: HashMap<Uuid, DocumentTarget>,
}

/// Target configuration for a single document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTarget {
    /// Target word count
    pub word_count: usize,
    /// Optional deadline
    pub deadline: Option<String>,
    /// Whether to show a progress bar in the binder
    pub show_in_binder: bool,
    /// Whether to notify when target is reached
    pub notify_on_complete: bool,
    /// Target type
    pub target_type: TargetType,
}

/// Types of writing targets
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TargetType {
    /// Minimum word count (write at least this many)
    Minimum,
    /// Maximum word count (don't exceed this many)
    Maximum,
    /// Exact range (between min and max)
    Range { min: usize, max: usize },
}

/// Progress status for a target
#[derive(Debug, Clone)]
pub struct TargetProgress {
    pub doc_id: Uuid,
    pub current_words: usize,
    pub target_words: usize,
    pub percentage: f64,
    pub status: TargetStatus,
    pub words_remaining: i64,
    pub days_remaining: Option<i64>,
    pub words_per_day_needed: Option<usize>,
}

/// Status of target completion
#[derive(Debug, Clone, PartialEq)]
pub enum TargetStatus {
    /// Haven't started (0% progress)
    NotStarted,
    /// In progress (1-99%)
    InProgress,
    /// Approaching target (90-99%)
    AlmostDone,
    /// Target reached or exceeded
    Complete,
    /// Over the maximum limit
    OverLimit,
}

impl DocumentTargets {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a word count target for a document
    pub fn set_target(&mut self, doc_id: Uuid, word_count: usize) {
        self.targets.insert(doc_id, DocumentTarget {
            word_count,
            deadline: None,
            show_in_binder: true,
            notify_on_complete: true,
            target_type: TargetType::Minimum,
        });
    }

    /// Set a target with full configuration
    pub fn set_target_full(&mut self, doc_id: Uuid, target: DocumentTarget) {
        self.targets.insert(doc_id, target);
    }

    /// Remove a target for a document
    pub fn remove_target(&mut self, doc_id: &Uuid) {
        self.targets.remove(doc_id);
    }

    /// Get a target for a document
    pub fn get_target(&self, doc_id: &Uuid) -> Option<&DocumentTarget> {
        self.targets.get(doc_id)
    }

    /// Get target word count for a document (convenience method)
    pub fn target_words(&self, doc_id: &Uuid) -> Option<usize> {
        self.targets.get(doc_id).map(|t| t.word_count)
    }

    /// Check if a document has a target
    pub fn has_target(&self, doc_id: &Uuid) -> bool {
        self.targets.contains_key(doc_id)
    }

    /// Calculate progress for a single document
    pub fn progress(&self, doc_id: &Uuid, current_words: usize) -> Option<TargetProgress> {
        let target = self.targets.get(doc_id)?;

        let percentage = if target.word_count > 0 {
            (current_words as f64 / target.word_count as f64 * 100.0).min(100.0)
        } else {
            100.0
        };

        let status = match &target.target_type {
            TargetType::Minimum => {
                if current_words == 0 {
                    TargetStatus::NotStarted
                } else if current_words >= target.word_count {
                    TargetStatus::Complete
                } else if percentage >= 90.0 {
                    TargetStatus::AlmostDone
                } else {
                    TargetStatus::InProgress
                }
            }
            TargetType::Maximum => {
                if current_words > target.word_count {
                    TargetStatus::OverLimit
                } else if current_words == 0 {
                    TargetStatus::NotStarted
                } else {
                    TargetStatus::InProgress
                }
            }
            TargetType::Range { min, max } => {
                if current_words >= *min && current_words <= *max {
                    TargetStatus::Complete
                } else if current_words > *max {
                    TargetStatus::OverLimit
                } else if current_words == 0 {
                    TargetStatus::NotStarted
                } else {
                    TargetStatus::InProgress
                }
            }
        };

        let words_remaining = target.word_count as i64 - current_words as i64;

        let days_remaining = target.deadline.as_ref().and_then(|d| {
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .ok()
                .map(|deadline| {
                    let today = chrono::Utc::now().date_naive();
                    (deadline - today).num_days()
                })
        });

        let words_per_day_needed = match (words_remaining > 0, days_remaining) {
            (true, Some(days)) if days > 0 => {
                Some((words_remaining as usize) / days as usize)
            }
            _ => None,
        };

        Some(TargetProgress {
            doc_id: *doc_id,
            current_words,
            target_words: target.word_count,
            percentage,
            status,
            words_remaining,
            days_remaining,
            words_per_day_needed,
        })
    }

    /// Get all targets with their progress
    pub fn all_progress(&self, word_counts: &HashMap<Uuid, usize>) -> Vec<TargetProgress> {
        self.targets.keys()
            .filter_map(|id| {
                let current = word_counts.get(id).copied().unwrap_or(0);
                self.progress(id, current)
            })
            .collect()
    }

    /// Get overall project progress across all targets
    pub fn overall_progress(&self, word_counts: &HashMap<Uuid, usize>) -> f64 {
        if self.targets.is_empty() {
            return 0.0;
        }
        let total_target: usize = self.targets.values().map(|t| t.word_count).sum();
        let total_current: usize = self.targets.keys()
            .map(|id| word_counts.get(id).copied().unwrap_or(0))
            .sum();
        if total_target == 0 {
            return 100.0;
        }
        (total_current as f64 / total_target as f64 * 100.0).min(100.0)
    }

    /// Count of completed targets
    pub fn completed_count(&self, word_counts: &HashMap<Uuid, usize>) -> usize {
        self.targets.keys()
            .filter(|id| {
                let current = word_counts.get(id).copied().unwrap_or(0);
                self.progress(id, current)
                    .map(|p| p.status == TargetStatus::Complete)
                    .unwrap_or(false)
            })
            .count()
    }

    /// Total number of targets
    pub fn total_count(&self) -> usize {
        self.targets.len()
    }

    /// Get a textual summary of all targets
    pub fn summary(&self, word_counts: &HashMap<Uuid, usize>) -> String {
        let total = self.total_count();
        if total == 0 {
            return "No targets set".to_string();
        }
        let completed = self.completed_count(word_counts);
        let overall = self.overall_progress(word_counts);
        format!(
            "{}/{} targets complete ({:.0}% overall)",
            completed, total, overall
        )
    }

    /// Get documents that have a deadline set
    pub fn with_deadline_docs(&self) -> Vec<Uuid> {
        self.targets
            .iter()
            .filter(|(_, t)| t.has_deadline())
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get document IDs whose targets are not yet complete
    pub fn incomplete_targets(&self, word_counts: &HashMap<Uuid, usize>) -> Vec<Uuid> {
        self.targets
            .keys()
            .filter(|id| {
                let current = word_counts.get(id).copied().unwrap_or(0);
                self.progress(id, current)
                    .map(|p| !p.status.is_complete())
                    .unwrap_or(true)
            })
            .copied()
            .collect()
    }

    /// Get the total target word count across all documents
    pub fn total_target_words(&self) -> usize {
        self.targets.values().map(|t| t.word_count).sum()
    }

    /// Get documents that are over their target limit
    pub fn over_limit_docs(&self, word_counts: &HashMap<Uuid, usize>) -> Vec<Uuid> {
        self.targets.keys()
            .filter(|id| {
                let current = word_counts.get(id).copied().unwrap_or(0);
                self.progress(id, current)
                    .map(|p| p.status == TargetStatus::OverLimit)
                    .unwrap_or(false)
            })
            .copied()
            .collect()
    }
}

impl TargetProgress {
    /// Format the progress as a compact string
    pub fn compact_display(&self) -> String {
        format!(
            "{}/{} ({:.0}%)",
            self.current_words, self.target_words, self.percentage
        )
    }

    /// Get a progress bar string (10 chars wide)
    pub fn progress_bar(&self) -> String {
        let filled = (self.percentage / 10.0).round() as usize;
        let empty = 10 - filled.min(10);
        format!(
            "[{}{}] {:.0}%",
            "#".repeat(filled.min(10)),
            "-".repeat(empty),
            self.percentage
        )
    }

    /// Status as a short label
    pub fn status_label(&self) -> &str {
        match self.status {
            TargetStatus::NotStarted => "Not Started",
            TargetStatus::InProgress => "In Progress",
            TargetStatus::AlmostDone => "Almost Done",
            TargetStatus::Complete => "Complete",
            TargetStatus::OverLimit => "Over Limit",
        }
    }

    /// Words remaining as a display string
    pub fn remaining_display(&self) -> String {
        if self.words_remaining > 0 {
            format!("{} words remaining", self.words_remaining)
        } else if self.words_remaining == 0 {
            "Target reached!".to_string()
        } else {
            format!("{} words over target", -self.words_remaining)
        }
    }

    /// Format with deadline info
    pub fn full_display(&self) -> String {
        let mut display = self.compact_display();
        if let Some(days) = self.days_remaining {
            display.push_str(&format!(", {} days left", days));
        }
        if let Some(wpd) = self.words_per_day_needed {
            display.push_str(&format!(", {} words/day needed", wpd));
        }
        display
    }

    /// Check if deadline is approaching (less than 7 days)
    pub fn deadline_approaching(&self) -> bool {
        self.days_remaining.map_or(false, |d| d > 0 && d <= 7)
    }

    /// Check if deadline is overdue
    pub fn deadline_overdue(&self) -> bool {
        self.days_remaining.map_or(false, |d| d < 0)
    }

    /// Whether the target is on track (enough daily capacity to finish in time)
    pub fn is_on_track(&self) -> bool {
        match (self.status == TargetStatus::Complete, self.words_per_day_needed) {
            (true, _) => true,
            (_, Some(wpd)) => wpd <= 2000, // Reasonable daily target
            (_, None) => self.days_remaining.is_none(), // No deadline = on track
        }
    }
}

impl DocumentTarget {
    /// Create a simple minimum target
    pub fn minimum(word_count: usize) -> Self {
        Self {
            word_count,
            deadline: None,
            show_in_binder: true,
            notify_on_complete: true,
            target_type: TargetType::Minimum,
        }
    }

    /// Create a maximum target
    pub fn maximum(word_count: usize) -> Self {
        Self {
            word_count,
            deadline: None,
            show_in_binder: true,
            notify_on_complete: true,
            target_type: TargetType::Maximum,
        }
    }

    /// Create a range target
    pub fn range(min: usize, max: usize) -> Self {
        Self {
            word_count: max,
            deadline: None,
            show_in_binder: true,
            notify_on_complete: true,
            target_type: TargetType::Range { min, max },
        }
    }

    /// Set deadline
    pub fn with_deadline(mut self, deadline: &str) -> Self {
        self.deadline = Some(deadline.to_string());
        self
    }

    /// Check if this target has a deadline
    pub fn has_deadline(&self) -> bool {
        self.deadline.is_some()
    }

    /// Get the target type as a label
    pub fn type_label(&self) -> &str {
        match &self.target_type {
            TargetType::Minimum => "Minimum",
            TargetType::Maximum => "Maximum",
            TargetType::Range { .. } => "Range",
        }
    }

    /// Get a summary of the target configuration
    pub fn summary(&self) -> String {
        let type_str = match &self.target_type {
            TargetType::Minimum => format!("min {} words", self.word_count),
            TargetType::Maximum => format!("max {} words", self.word_count),
            TargetType::Range { min, max } => format!("{}-{} words", min, max),
        };
        if let Some(ref dl) = self.deadline {
            format!("{} by {}", type_str, dl)
        } else {
            type_str
        }
    }
}

impl TargetType {
    /// Display label for the target type
    pub fn label(&self) -> &str {
        match self {
            TargetType::Minimum => "Minimum",
            TargetType::Maximum => "Maximum",
            TargetType::Range { .. } => "Range",
        }
    }
}

impl TargetStatus {
    /// Get all possible status variants
    pub fn all() -> Vec<TargetStatus> {
        vec![
            TargetStatus::NotStarted,
            TargetStatus::InProgress,
            TargetStatus::AlmostDone,
            TargetStatus::Complete,
            TargetStatus::OverLimit,
        ]
    }

    /// Get a short emoji representation
    pub fn icon(&self) -> &str {
        match self {
            TargetStatus::NotStarted => "-",
            TargetStatus::InProgress => ">",
            TargetStatus::AlmostDone => "!",
            TargetStatus::Complete => "+",
            TargetStatus::OverLimit => "X",
        }
    }

    /// Check if this represents a completed target
    pub fn is_complete(&self) -> bool {
        matches!(self, TargetStatus::Complete)
    }

    /// Check if this needs attention
    pub fn needs_attention(&self) -> bool {
        matches!(self, TargetStatus::AlmostDone | TargetStatus::OverLimit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_targets_new() {
        let targets = DocumentTargets::new();
        assert_eq!(targets.total_count(), 0);
    }

    #[test]
    fn test_set_and_get_target() {
        let mut targets = DocumentTargets::new();
        let id = Uuid::new_v4();
        targets.set_target(id, 5000);
        assert!(targets.has_target(&id));
        assert_eq!(targets.target_words(&id), Some(5000));
    }

    #[test]
    fn test_remove_target() {
        let mut targets = DocumentTargets::new();
        let id = Uuid::new_v4();
        targets.set_target(id, 5000);
        targets.remove_target(&id);
        assert!(!targets.has_target(&id));
    }

    #[test]
    fn test_progress_not_started() {
        let mut targets = DocumentTargets::new();
        let id = Uuid::new_v4();
        targets.set_target(id, 5000);
        let progress = targets.progress(&id, 0).unwrap();
        assert_eq!(progress.status, TargetStatus::NotStarted);
        assert_eq!(progress.percentage, 0.0);
    }

    #[test]
    fn test_progress_in_progress() {
        let mut targets = DocumentTargets::new();
        let id = Uuid::new_v4();
        targets.set_target(id, 1000);
        let progress = targets.progress(&id, 500).unwrap();
        assert_eq!(progress.status, TargetStatus::InProgress);
        assert!((progress.percentage - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_progress_almost_done() {
        let mut targets = DocumentTargets::new();
        let id = Uuid::new_v4();
        targets.set_target(id, 1000);
        let progress = targets.progress(&id, 950).unwrap();
        assert_eq!(progress.status, TargetStatus::AlmostDone);
    }

    #[test]
    fn test_progress_complete() {
        let mut targets = DocumentTargets::new();
        let id = Uuid::new_v4();
        targets.set_target(id, 1000);
        let progress = targets.progress(&id, 1000).unwrap();
        assert_eq!(progress.status, TargetStatus::Complete);
    }

    #[test]
    fn test_progress_over_minimum() {
        let mut targets = DocumentTargets::new();
        let id = Uuid::new_v4();
        targets.set_target(id, 1000);
        let progress = targets.progress(&id, 1500).unwrap();
        assert_eq!(progress.status, TargetStatus::Complete);
    }

    #[test]
    fn test_progress_maximum_over_limit() {
        let mut targets = DocumentTargets::new();
        let id = Uuid::new_v4();
        targets.set_target_full(id, DocumentTarget::maximum(500));
        let progress = targets.progress(&id, 600).unwrap();
        assert_eq!(progress.status, TargetStatus::OverLimit);
    }

    #[test]
    fn test_progress_range() {
        let mut targets = DocumentTargets::new();
        let id = Uuid::new_v4();
        targets.set_target_full(id, DocumentTarget::range(100, 500));

        let in_range = targets.progress(&id, 300).unwrap();
        assert_eq!(in_range.status, TargetStatus::Complete);

        let over = targets.progress(&id, 600).unwrap();
        assert_eq!(over.status, TargetStatus::OverLimit);

        let under = targets.progress(&id, 50).unwrap();
        assert_eq!(under.status, TargetStatus::InProgress);
    }

    #[test]
    fn test_overall_progress() {
        let mut targets = DocumentTargets::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        targets.set_target(id1, 1000);
        targets.set_target(id2, 1000);

        let mut word_counts = HashMap::new();
        word_counts.insert(id1, 500);
        word_counts.insert(id2, 500);

        let overall = targets.overall_progress(&word_counts);
        assert!((overall - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_completed_count() {
        let mut targets = DocumentTargets::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        targets.set_target(id1, 100);
        targets.set_target(id2, 100);

        let mut word_counts = HashMap::new();
        word_counts.insert(id1, 100);
        word_counts.insert(id2, 50);

        assert_eq!(targets.completed_count(&word_counts), 1);
    }

    #[test]
    fn test_progress_bar() {
        let progress = TargetProgress {
            doc_id: Uuid::new_v4(),
            current_words: 500,
            target_words: 1000,
            percentage: 50.0,
            status: TargetStatus::InProgress,
            words_remaining: 500,
            days_remaining: None,
            words_per_day_needed: None,
        };
        let bar = progress.progress_bar();
        assert!(bar.contains("#####"));
        assert!(bar.contains("50%"));
    }

    #[test]
    fn test_compact_display() {
        let progress = TargetProgress {
            doc_id: Uuid::new_v4(),
            current_words: 750,
            target_words: 1000,
            percentage: 75.0,
            status: TargetStatus::InProgress,
            words_remaining: 250,
            days_remaining: None,
            words_per_day_needed: None,
        };
        let display = progress.compact_display();
        assert!(display.contains("750/1000"));
    }

    #[test]
    fn test_remaining_display() {
        let mut progress = TargetProgress {
            doc_id: Uuid::new_v4(),
            current_words: 750,
            target_words: 1000,
            percentage: 75.0,
            status: TargetStatus::InProgress,
            words_remaining: 250,
            days_remaining: None,
            words_per_day_needed: None,
        };
        assert!(progress.remaining_display().contains("250 words remaining"));

        progress.words_remaining = 0;
        assert!(progress.remaining_display().contains("Target reached"));

        progress.words_remaining = -50;
        assert!(progress.remaining_display().contains("50 words over target"));
    }

    #[test]
    fn test_document_target_builders() {
        let min_target = DocumentTarget::minimum(5000);
        assert_eq!(min_target.target_type, TargetType::Minimum);
        assert_eq!(min_target.word_count, 5000);

        let max_target = DocumentTarget::maximum(3000);
        assert_eq!(max_target.target_type, TargetType::Maximum);

        let range_target = DocumentTarget::range(100, 500);
        assert_eq!(range_target.target_type, TargetType::Range { min: 100, max: 500 });

        let with_deadline = DocumentTarget::minimum(1000).with_deadline("2026-12-31");
        assert_eq!(with_deadline.deadline, Some("2026-12-31".to_string()));
    }

    #[test]
    fn test_serialization() {
        let mut targets = DocumentTargets::new();
        let id = Uuid::new_v4();
        targets.set_target(id, 5000);

        let json = serde_json::to_string(&targets).unwrap();
        let parsed: DocumentTargets = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.target_words(&id), Some(5000));
    }

    #[test]
    fn test_document_target_has_deadline() {
        let target = DocumentTarget::minimum(1000);
        assert!(!target.has_deadline());
        let target_dl = DocumentTarget::minimum(1000).with_deadline("2026-12-31");
        assert!(target_dl.has_deadline());
    }

    #[test]
    fn test_document_target_type_label() {
        assert_eq!(DocumentTarget::minimum(100).type_label(), "Minimum");
        assert_eq!(DocumentTarget::maximum(100).type_label(), "Maximum");
        assert_eq!(DocumentTarget::range(50, 100).type_label(), "Range");
    }

    #[test]
    fn test_document_target_summary() {
        let t = DocumentTarget::minimum(5000);
        assert_eq!(t.summary(), "min 5000 words");

        let t = DocumentTarget::maximum(3000);
        assert_eq!(t.summary(), "max 3000 words");

        let t = DocumentTarget::range(100, 500);
        assert_eq!(t.summary(), "100-500 words");

        let t = DocumentTarget::minimum(1000).with_deadline("2026-06-01");
        assert!(t.summary().contains("by 2026-06-01"));
    }

    #[test]
    fn test_target_status_icon() {
        assert_eq!(TargetStatus::NotStarted.icon(), "-");
        assert_eq!(TargetStatus::InProgress.icon(), ">");
        assert_eq!(TargetStatus::AlmostDone.icon(), "!");
        assert_eq!(TargetStatus::Complete.icon(), "+");
        assert_eq!(TargetStatus::OverLimit.icon(), "X");
    }

    #[test]
    fn test_target_status_is_complete() {
        assert!(TargetStatus::Complete.is_complete());
        assert!(!TargetStatus::InProgress.is_complete());
        assert!(!TargetStatus::NotStarted.is_complete());
    }

    #[test]
    fn test_target_status_needs_attention() {
        assert!(TargetStatus::AlmostDone.needs_attention());
        assert!(TargetStatus::OverLimit.needs_attention());
        assert!(!TargetStatus::InProgress.needs_attention());
        assert!(!TargetStatus::Complete.needs_attention());
    }

    #[test]
    fn test_target_progress_full_display() {
        let progress = TargetProgress {
            doc_id: Uuid::new_v4(),
            current_words: 500,
            target_words: 1000,
            percentage: 50.0,
            status: TargetStatus::InProgress,
            words_remaining: 500,
            days_remaining: Some(10),
            words_per_day_needed: Some(50),
        };
        let display = progress.full_display();
        assert!(display.contains("500/1000"));
        assert!(display.contains("10 days left"));
        assert!(display.contains("50 words/day needed"));
    }

    #[test]
    fn test_target_progress_deadline_approaching() {
        let mut progress = TargetProgress {
            doc_id: Uuid::new_v4(),
            current_words: 500,
            target_words: 1000,
            percentage: 50.0,
            status: TargetStatus::InProgress,
            words_remaining: 500,
            days_remaining: Some(5),
            words_per_day_needed: Some(100),
        };
        assert!(progress.deadline_approaching());

        progress.days_remaining = Some(14);
        assert!(!progress.deadline_approaching());

        progress.days_remaining = None;
        assert!(!progress.deadline_approaching());
    }

    #[test]
    fn test_target_progress_deadline_overdue() {
        let mut progress = TargetProgress {
            doc_id: Uuid::new_v4(),
            current_words: 500,
            target_words: 1000,
            percentage: 50.0,
            status: TargetStatus::InProgress,
            words_remaining: 500,
            days_remaining: Some(-3),
            words_per_day_needed: None,
        };
        assert!(progress.deadline_overdue());

        progress.days_remaining = Some(5);
        assert!(!progress.deadline_overdue());
    }

    #[test]
    fn test_over_limit_docs() {
        let mut targets = DocumentTargets::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        targets.set_target_full(id1, DocumentTarget::maximum(100));
        targets.set_target_full(id2, DocumentTarget::maximum(200));

        let mut word_counts = HashMap::new();
        word_counts.insert(id1, 150); // Over limit
        word_counts.insert(id2, 100); // Under limit

        let over = targets.over_limit_docs(&word_counts);
        assert_eq!(over.len(), 1);
        assert_eq!(over[0], id1);
    }

    #[test]
    fn test_all_progress() {
        let mut targets = DocumentTargets::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        targets.set_target(id1, 100);
        targets.set_target(id2, 200);

        let mut word_counts = HashMap::new();
        word_counts.insert(id1, 50);
        word_counts.insert(id2, 200);

        let progress = targets.all_progress(&word_counts);
        assert_eq!(progress.len(), 2);
    }

    #[test]
    fn test_overall_progress_empty() {
        let targets = DocumentTargets::new();
        let word_counts = HashMap::new();
        assert_eq!(targets.overall_progress(&word_counts), 0.0);
    }

    #[test]
    fn test_status_label() {
        let progress = TargetProgress {
            doc_id: Uuid::new_v4(),
            current_words: 0,
            target_words: 100,
            percentage: 0.0,
            status: TargetStatus::NotStarted,
            words_remaining: 100,
            days_remaining: None,
            words_per_day_needed: None,
        };
        assert_eq!(progress.status_label(), "Not Started");
    }

    #[test]
    fn test_progress_zero_target() {
        let mut targets = DocumentTargets::new();
        let id = Uuid::new_v4();
        targets.set_target(id, 0);
        let progress = targets.progress(&id, 0).unwrap();
        assert_eq!(progress.percentage, 100.0);
    }

    #[test]
    fn test_get_target() {
        let mut targets = DocumentTargets::new();
        let id = Uuid::new_v4();
        targets.set_target_full(id, DocumentTarget::range(100, 500));
        let t = targets.get_target(&id).unwrap();
        assert_eq!(t.type_label(), "Range");
    }

    #[test]
    fn test_targets_summary_empty() {
        let targets = DocumentTargets::new();
        let wc = HashMap::new();
        assert_eq!(targets.summary(&wc), "No targets set");
    }

    #[test]
    fn test_targets_summary_with_data() {
        let mut targets = DocumentTargets::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        targets.set_target(id1, 100);
        targets.set_target(id2, 200);

        let mut wc = HashMap::new();
        wc.insert(id1, 100); // complete
        wc.insert(id2, 50);  // in progress

        let summary = targets.summary(&wc);
        assert!(summary.contains("1/2 targets complete"));
    }

    #[test]
    fn test_with_deadline_docs() {
        let mut targets = DocumentTargets::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        targets.set_target_full(id1, DocumentTarget::minimum(100).with_deadline("2026-12-31"));
        targets.set_target(id2, 200); // no deadline

        let dl_docs = targets.with_deadline_docs();
        assert_eq!(dl_docs.len(), 1);
        assert_eq!(dl_docs[0], id1);
    }

    #[test]
    fn test_incomplete_targets() {
        let mut targets = DocumentTargets::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        targets.set_target(id1, 100);
        targets.set_target(id2, 200);

        let mut wc = HashMap::new();
        wc.insert(id1, 100); // complete
        wc.insert(id2, 50);  // incomplete

        let incomplete = targets.incomplete_targets(&wc);
        assert_eq!(incomplete.len(), 1);
        assert_eq!(incomplete[0], id2);
    }

    #[test]
    fn test_total_target_words() {
        let mut targets = DocumentTargets::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        targets.set_target(id1, 1000);
        targets.set_target(id2, 2000);
        assert_eq!(targets.total_target_words(), 3000);
    }

    #[test]
    fn test_total_target_words_empty() {
        let targets = DocumentTargets::new();
        assert_eq!(targets.total_target_words(), 0);
    }

    #[test]
    fn test_target_type_label() {
        assert_eq!(TargetType::Minimum.label(), "Minimum");
        assert_eq!(TargetType::Maximum.label(), "Maximum");
        assert_eq!(TargetType::Range { min: 10, max: 20 }.label(), "Range");
    }

    #[test]
    fn test_target_status_all() {
        let all = TargetStatus::all();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&TargetStatus::NotStarted));
        assert!(all.contains(&TargetStatus::InProgress));
        assert!(all.contains(&TargetStatus::AlmostDone));
        assert!(all.contains(&TargetStatus::Complete));
        assert!(all.contains(&TargetStatus::OverLimit));
    }

    #[test]
    fn test_is_on_track_complete() {
        let progress = TargetProgress {
            doc_id: Uuid::new_v4(),
            current_words: 1000,
            target_words: 1000,
            percentage: 100.0,
            status: TargetStatus::Complete,
            words_remaining: 0,
            days_remaining: None,
            words_per_day_needed: None,
        };
        assert!(progress.is_on_track());
    }

    #[test]
    fn test_is_on_track_reasonable_pace() {
        let progress = TargetProgress {
            doc_id: Uuid::new_v4(),
            current_words: 500,
            target_words: 1000,
            percentage: 50.0,
            status: TargetStatus::InProgress,
            words_remaining: 500,
            days_remaining: Some(10),
            words_per_day_needed: Some(50),
        };
        assert!(progress.is_on_track());
    }

    #[test]
    fn test_is_on_track_unreasonable_pace() {
        let progress = TargetProgress {
            doc_id: Uuid::new_v4(),
            current_words: 100,
            target_words: 10000,
            percentage: 1.0,
            status: TargetStatus::InProgress,
            words_remaining: 9900,
            days_remaining: Some(2),
            words_per_day_needed: Some(4950),
        };
        assert!(!progress.is_on_track());
    }

    #[test]
    fn test_is_on_track_no_deadline() {
        let progress = TargetProgress {
            doc_id: Uuid::new_v4(),
            current_words: 500,
            target_words: 1000,
            percentage: 50.0,
            status: TargetStatus::InProgress,
            words_remaining: 500,
            days_remaining: None,
            words_per_day_needed: None,
        };
        assert!(progress.is_on_track());
    }
}
