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
