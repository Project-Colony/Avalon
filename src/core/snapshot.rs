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

    /// Compute the diff between this snapshot and current content using LCS algorithm
    pub fn diff_with(&self, current: &str) -> Vec<DiffChunk> {
        let old_lines: Vec<&str> = self.content.lines().collect();
        let new_lines: Vec<&str> = current.lines().collect();
        lcs_diff(&old_lines, &new_lines)
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

/// Compute diff using Longest Common Subsequence (Myers-like) algorithm.
/// Produces minimal edit script.
fn lcs_diff(old: &[&str], new: &[&str]) -> Vec<DiffChunk> {
    let n = old.len();
    let m = new.len();

    // Build LCS table
    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in 1..=n {
        for j in 1..=m {
            if old[i - 1] == new[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    // Backtrack to produce diff
    let mut chunks = Vec::new();
    let mut i = n;
    let mut j = m;

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old[i - 1] == new[j - 1] {
            chunks.push(DiffChunk::Equal(old[i - 1].to_string()));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            chunks.push(DiffChunk::Added(new[j - 1].to_string()));
            j -= 1;
        } else if i > 0 {
            chunks.push(DiffChunk::Removed(old[i - 1].to_string()));
            i -= 1;
        }
    }

    chunks.reverse();
    chunks
}

/// Format a diff as a unified diff string (like `diff -u` output)
pub fn format_unified_diff(chunks: &[DiffChunk], old_label: &str, new_label: &str) -> String {
    let mut output = String::new();
    output.push_str(&format!("--- {}\n", old_label));
    output.push_str(&format!("+++ {}\n", new_label));

    // Group changes into hunks
    let hunks = group_into_hunks(chunks, 3);
    for hunk in hunks {
        output.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count
        ));
        for line in &hunk.lines {
            match line {
                DiffChunk::Equal(text) => output.push_str(&format!(" {}\n", text)),
                DiffChunk::Added(text) => output.push_str(&format!("+{}\n", text)),
                DiffChunk::Removed(text) => output.push_str(&format!("-{}\n", text)),
            }
        }
    }

    output
}

/// A hunk in a unified diff
struct DiffHunk {
    old_start: usize,
    old_count: usize,
    new_start: usize,
    new_count: usize,
    lines: Vec<DiffChunk>,
}

/// Group diff chunks into hunks with context lines
fn group_into_hunks(chunks: &[DiffChunk], context: usize) -> Vec<DiffHunk> {
    if chunks.is_empty() {
        return Vec::new();
    }

    // Find change ranges
    let mut change_indices = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        if !matches!(chunk, DiffChunk::Equal(_)) {
            change_indices.push(i);
        }
    }

    if change_indices.is_empty() {
        return Vec::new(); // No changes
    }

    // Group nearby changes into hunks
    let mut hunks = Vec::new();
    let mut hunk_start = change_indices[0].saturating_sub(context);
    let mut hunk_end = (change_indices[0] + context + 1).min(chunks.len());

    for &idx in &change_indices[1..] {
        let proposed_start = idx.saturating_sub(context);
        if proposed_start <= hunk_end {
            // Merge with current hunk
            hunk_end = (idx + context + 1).min(chunks.len());
        } else {
            // Emit current hunk
            hunks.push(build_hunk(chunks, hunk_start, hunk_end));
            hunk_start = proposed_start;
            hunk_end = (idx + context + 1).min(chunks.len());
        }
    }
    hunks.push(build_hunk(chunks, hunk_start, hunk_end));

    hunks
}

fn build_hunk(chunks: &[DiffChunk], start: usize, end: usize) -> DiffHunk {
    let mut old_start = 1;
    let mut new_start = 1;
    // Count lines before the hunk start
    for chunk in &chunks[..start] {
        match chunk {
            DiffChunk::Equal(_) => { old_start += 1; new_start += 1; }
            DiffChunk::Removed(_) => { old_start += 1; }
            DiffChunk::Added(_) => { new_start += 1; }
        }
    }

    let mut old_count = 0;
    let mut new_count = 0;
    let lines: Vec<DiffChunk> = chunks[start..end].to_vec();
    for chunk in &lines {
        match chunk {
            DiffChunk::Equal(_) => { old_count += 1; new_count += 1; }
            DiffChunk::Removed(_) => { old_count += 1; }
            DiffChunk::Added(_) => { new_count += 1; }
        }
    }

    DiffHunk { old_start, old_count, new_start, new_count, lines }
}

/// Compare two snapshots
pub fn diff_snapshots(older: &Snapshot, newer: &Snapshot) -> Vec<DiffChunk> {
    let old_lines: Vec<&str> = older.content.lines().collect();
    let new_lines: Vec<&str> = newer.content.lines().collect();
    lcs_diff(&old_lines, &new_lines)
}

/// Inline diff: find differences within a single line
pub fn inline_diff(old_line: &str, new_line: &str) -> Vec<InlineDiffChunk> {
    let old_words: Vec<&str> = old_line.split_whitespace().collect();
    let new_words: Vec<&str> = new_line.split_whitespace().collect();

    let n = old_words.len();
    let m = new_words.len();
    let mut dp = vec![vec![0u32; m + 1]; n + 1];

    for i in 1..=n {
        for j in 1..=m {
            if old_words[i - 1] == new_words[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut result = Vec::new();
    let mut i = n;
    let mut j = m;

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old_words[i - 1] == new_words[j - 1] {
            result.push(InlineDiffChunk::Equal(old_words[i - 1].to_string()));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            result.push(InlineDiffChunk::Added(new_words[j - 1].to_string()));
            j -= 1;
        } else if i > 0 {
            result.push(InlineDiffChunk::Removed(old_words[i - 1].to_string()));
            i -= 1;
        }
    }

    result.reverse();
    result
}

/// A word-level diff chunk for inline diff display
#[derive(Debug, Clone, PartialEq)]
pub enum InlineDiffChunk {
    Equal(String),
    Added(String),
    Removed(String),
}

impl InlineDiffChunk {
    /// Render as a string with markdown-style markers
    pub fn to_marked_string(&self) -> String {
        match self {
            InlineDiffChunk::Equal(s) => s.clone(),
            InlineDiffChunk::Added(s) => format!("{{+{}+}}", s),
            InlineDiffChunk::Removed(s) => format!("{{-{}-}}", s),
        }
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

    #[test]
    fn test_lcs_diff_insertion() {
        let old = vec!["line1", "line2", "line3"];
        let new = vec!["line1", "inserted", "line2", "line3"];
        let diff = lcs_diff(&old, &new);

        let added_count = diff.iter().filter(|c| matches!(c, DiffChunk::Added(_))).count();
        let equal_count = diff.iter().filter(|c| matches!(c, DiffChunk::Equal(_))).count();
        assert_eq!(added_count, 1);
        assert_eq!(equal_count, 3);
    }

    #[test]
    fn test_lcs_diff_deletion() {
        let old = vec!["line1", "to_delete", "line2"];
        let new = vec!["line1", "line2"];
        let diff = lcs_diff(&old, &new);

        let removed_count = diff.iter().filter(|c| matches!(c, DiffChunk::Removed(_))).count();
        let equal_count = diff.iter().filter(|c| matches!(c, DiffChunk::Equal(_))).count();
        assert_eq!(removed_count, 1);
        assert_eq!(equal_count, 2);
    }

    #[test]
    fn test_lcs_diff_modification() {
        let old = vec!["line1", "old line", "line3"];
        let new = vec!["line1", "new line", "line3"];
        let diff = lcs_diff(&old, &new);

        let added_count = diff.iter().filter(|c| matches!(c, DiffChunk::Added(_))).count();
        let removed_count = diff.iter().filter(|c| matches!(c, DiffChunk::Removed(_))).count();
        assert!(added_count >= 1);
        assert!(removed_count >= 1);
    }

    #[test]
    fn test_lcs_diff_empty_old() {
        let old: Vec<&str> = vec![];
        let new = vec!["line1", "line2"];
        let diff = lcs_diff(&old, &new);

        let added_count = diff.iter().filter(|c| matches!(c, DiffChunk::Added(_))).count();
        assert_eq!(added_count, 2);
    }

    #[test]
    fn test_lcs_diff_empty_new() {
        let old = vec!["line1", "line2"];
        let new: Vec<&str> = vec![];
        let diff = lcs_diff(&old, &new);

        let removed_count = diff.iter().filter(|c| matches!(c, DiffChunk::Removed(_))).count();
        assert_eq!(removed_count, 2);
    }

    #[test]
    fn test_lcs_diff_identical() {
        let lines = vec!["line1", "line2", "line3"];
        let diff = lcs_diff(&lines, &lines);
        let equal_count = diff.iter().filter(|c| matches!(c, DiffChunk::Equal(_))).count();
        assert_eq!(equal_count, 3);
        assert!(diff.iter().all(|c| matches!(c, DiffChunk::Equal(_))));
    }

    #[test]
    fn test_unified_diff_format() {
        let doc = Document::with_content("line1\nold line\nline3");
        let snap = Snapshot::from_document(&doc, "v1");
        let chunks = snap.diff_with("line1\nnew line\nline3");
        let unified = format_unified_diff(&chunks, "snapshot", "current");
        assert!(unified.contains("--- snapshot"));
        assert!(unified.contains("+++ current"));
        assert!(unified.contains("@@"));
    }

    #[test]
    fn test_inline_diff() {
        let old = "The quick brown fox";
        let new = "The slow brown cat";
        let chunks = inline_diff(old, new);

        let has_equal = chunks.iter().any(|c| matches!(c, InlineDiffChunk::Equal(_)));
        let has_changes = chunks.iter().any(|c| !matches!(c, InlineDiffChunk::Equal(_)));
        assert!(has_equal);
        assert!(has_changes);
    }

    #[test]
    fn test_inline_diff_identical() {
        let line = "The same line";
        let chunks = inline_diff(line, line);
        assert!(chunks.iter().all(|c| matches!(c, InlineDiffChunk::Equal(_))));
    }

    #[test]
    fn test_inline_diff_marked_string() {
        let eq = InlineDiffChunk::Equal("hello".to_string());
        let add = InlineDiffChunk::Added("world".to_string());
        let rem = InlineDiffChunk::Removed("old".to_string());
        assert_eq!(eq.to_marked_string(), "hello");
        assert_eq!(add.to_marked_string(), "{+world+}");
        assert_eq!(rem.to_marked_string(), "{-old-}");
    }

    #[test]
    fn test_diff_snapshots() {
        let doc1 = Document::with_content("First version");
        let snap1 = Snapshot::from_document(&doc1, "v1");
        let doc2 = Document::with_content("Second version");
        let snap2 = Snapshot::from_document(&doc2, "v2");

        let diff = diff_snapshots(&snap1, &snap2);
        assert!(!diff.is_empty());
        let has_changes = diff.iter().any(|c| !matches!(c, DiffChunk::Equal(_)));
        assert!(has_changes);
    }
}
