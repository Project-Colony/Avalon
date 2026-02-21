use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::document::Document;

/// A Snapshot preserves a point-in-time copy of a document.
/// Used for versioning and rollback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    /// The preserved content
    pub content: String,
    /// Word count at snapshot time
    pub word_count: usize,
}

impl Snapshot {
    /// Create a snapshot from a document
    pub fn from_document(doc: &Document, title: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.to_string(),
            created_at: Utc::now(),
            content: doc.content.clone(),
            word_count: doc.word_count(),
        }
    }

    /// Compute the diff between this snapshot and current content
    pub fn diff_with(&self, current: &str) -> Vec<DiffChunk> {
        // Simple line-by-line diff
        let old_lines: Vec<&str> = self.content.lines().collect();
        let new_lines: Vec<&str> = current.lines().collect();

        let mut chunks = Vec::new();
        let mut i = 0;
        let mut j = 0;

        while i < old_lines.len() || j < new_lines.len() {
            if i < old_lines.len() && j < new_lines.len() {
                if old_lines[i] == new_lines[j] {
                    chunks.push(DiffChunk::Equal(old_lines[i].to_string()));
                    i += 1;
                    j += 1;
                } else {
                    chunks.push(DiffChunk::Removed(old_lines[i].to_string()));
                    chunks.push(DiffChunk::Added(new_lines[j].to_string()));
                    i += 1;
                    j += 1;
                }
            } else if i < old_lines.len() {
                chunks.push(DiffChunk::Removed(old_lines[i].to_string()));
                i += 1;
            } else {
                chunks.push(DiffChunk::Added(new_lines[j].to_string()));
                j += 1;
            }
        }

        chunks
    }
}

#[derive(Debug, Clone)]
pub enum DiffChunk {
    Equal(String),
    Added(String),
    Removed(String),
}

/// Summary statistics for a diff
pub struct DiffStats {
    pub lines_added: usize,
    pub lines_removed: usize,
    pub lines_unchanged: usize,
    pub words_added: i64,
}

impl DiffStats {
    /// Compute stats from a list of diff chunks
    pub fn from_chunks(chunks: &[DiffChunk]) -> Self {
        let mut added = 0;
        let mut removed = 0;
        let mut unchanged = 0;
        let mut words_added: i64 = 0;

        for chunk in chunks {
            match chunk {
                DiffChunk::Equal(_) => unchanged += 1,
                DiffChunk::Added(line) => {
                    added += 1;
                    words_added += line.split_whitespace().count() as i64;
                }
                DiffChunk::Removed(line) => {
                    removed += 1;
                    words_added -= line.split_whitespace().count() as i64;
                }
            }
        }

        Self {
            lines_added: added,
            lines_removed: removed,
            lines_unchanged: unchanged,
            words_added,
        }
    }

    /// Get a short summary string
    pub fn summary(&self) -> String {
        let word_change = if self.words_added > 0 {
            format!("+{} words", self.words_added)
        } else if self.words_added < 0 {
            format!("{} words", self.words_added)
        } else {
            "no word change".to_string()
        };
        format!(
            "+{} / -{} lines ({}), {} unchanged",
            self.lines_added, self.lines_removed, word_change, self.lines_unchanged
        )
    }

    /// Total lines changed (added + removed)
    pub fn total_changes(&self) -> usize {
        self.lines_added + self.lines_removed
    }

    /// Total lines in diff
    pub fn total_lines(&self) -> usize {
        self.lines_added + self.lines_removed + self.lines_unchanged
    }

    /// Percentage of lines changed
    pub fn change_percentage(&self) -> f64 {
        let total = self.total_lines();
        if total == 0 {
            return 0.0;
        }
        self.total_changes() as f64 / total as f64 * 100.0
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        self.lines_added > 0 || self.lines_removed > 0
    }
}

impl Snapshot {
    /// Age as human-readable string
    pub fn age_string(&self) -> String {
        let duration = Utc::now().signed_duration_since(self.created_at);
        let hours = duration.num_hours();
        if hours < 1 {
            format!("{}m ago", duration.num_minutes().max(1))
        } else if hours < 24 {
            format!("{}h ago", hours)
        } else {
            let days = duration.num_days();
            if days < 7 {
                format!("{}d ago", days)
            } else {
                format!("{}w ago", days / 7)
            }
        }
    }

    /// Character count at snapshot time
    pub fn char_count(&self) -> usize {
        self.content.len()
    }

    /// Line count at snapshot time
    pub fn line_count(&self) -> usize {
        if self.content.is_empty() {
            0
        } else {
            self.content.lines().count()
        }
    }

    /// Compute diff stats with current content
    pub fn diff_stats_with(&self, current: &str) -> DiffStats {
        let diff = self.diff_with(current);
        DiffStats::from_chunks(&diff)
    }
}
