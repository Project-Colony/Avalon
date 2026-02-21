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
}
