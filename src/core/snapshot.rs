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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::Document;

    #[test]
    fn test_snapshot_from_document() {
        let doc = Document::with_content("Hello world");
        let snap = Snapshot::from_document(&doc, "Test snapshot");
        assert_eq!(snap.title, "Test snapshot");
        assert_eq!(snap.content, "Hello world");
        assert_eq!(snap.word_count, 2);
    }

    #[test]
    fn test_snapshot_char_line_count() {
        let doc = Document::with_content("Line one\nLine two\nLine three");
        let snap = Snapshot::from_document(&doc, "Test");
        assert_eq!(snap.char_count(), 28);
        assert_eq!(snap.line_count(), 3);
    }

    #[test]
    fn test_diff_equal() {
        let doc = Document::with_content("Hello world");
        let snap = Snapshot::from_document(&doc, "Test");
        let chunks = snap.diff_with("Hello world");
        assert_eq!(chunks.len(), 1);
        assert!(matches!(chunks[0], DiffChunk::Equal(_)));
    }

    #[test]
    fn test_diff_added_lines() {
        let doc = Document::with_content("Line one");
        let snap = Snapshot::from_document(&doc, "Test");
        let chunks = snap.diff_with("Line one\nLine two");
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn test_diff_stats() {
        let doc = Document::with_content("Line one\nLine two");
        let snap = Snapshot::from_document(&doc, "Test");
        let stats = snap.diff_stats_with("Line one\nLine three\nLine four");
        assert!(stats.has_changes());
        assert!(stats.lines_added > 0 || stats.lines_removed > 0);
    }

    #[test]
    fn test_diff_stats_summary() {
        let chunks = vec![
            DiffChunk::Equal("same".to_string()),
            DiffChunk::Removed("old line".to_string()),
            DiffChunk::Added("new line".to_string()),
        ];
        let stats = DiffStats::from_chunks(&chunks);
        assert_eq!(stats.lines_added, 1);
        assert_eq!(stats.lines_removed, 1);
        assert_eq!(stats.lines_unchanged, 1);
        assert!(stats.has_changes());

        let summary = stats.summary();
        assert!(summary.contains("+1"));
        assert!(summary.contains("-1"));
    }

    #[test]
    fn test_diff_stats_no_changes() {
        let chunks = vec![
            DiffChunk::Equal("line1".to_string()),
            DiffChunk::Equal("line2".to_string()),
        ];
        let stats = DiffStats::from_chunks(&chunks);
        assert!(!stats.has_changes());
        assert_eq!(stats.total_changes(), 0);
        assert!((stats.change_percentage() - 0.0).abs() < 0.01);
    }
}
