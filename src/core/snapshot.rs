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

    /// Paragraph count at snapshot time
    pub fn paragraph_count(&self) -> usize {
        if self.content.is_empty() {
            0
        } else {
            self.content.split("\n\n").filter(|p| !p.trim().is_empty()).count()
        }
    }

    /// Sentence count at snapshot time (approximate)
    pub fn sentence_count(&self) -> usize {
        self.content.chars().filter(|c| *c == '.' || *c == '!' || *c == '?').count()
    }

    /// Check if content has changed since snapshot
    pub fn has_changed(&self, current: &str) -> bool {
        self.content != current
    }

    /// Get a short label summarizing the snapshot
    pub fn label(&self) -> String {
        format!("{} ({} words, {})", self.title, self.word_count, self.age_string())
    }

    /// Similarity ratio with current content (0.0 - 1.0)
    pub fn similarity(&self, current: &str) -> f64 {
        let diff = self.diff_with(current);
        let stats = DiffStats::from_chunks(&diff);
        if stats.total_lines() == 0 {
            return 1.0;
        }
        stats.lines_unchanged as f64 / stats.total_lines() as f64
    }
}

/// Render a diff as a simple side-by-side summary
pub fn diff_summary(chunks: &[DiffChunk]) -> String {
    let stats = DiffStats::from_chunks(chunks);
    if !stats.has_changes() {
        return "No changes".to_string();
    }
    let mut parts = Vec::new();
    if stats.lines_added > 0 {
        parts.push(format!("{} added", stats.lines_added));
    }
    if stats.lines_removed > 0 {
        parts.push(format!("{} removed", stats.lines_removed));
    }
    if stats.lines_unchanged > 0 {
        parts.push(format!("{} unchanged", stats.lines_unchanged));
    }
    parts.join(", ")
}

/// Compute a patch that can be applied to old content to produce new content
pub fn compute_patch(old: &str, new: &str) -> Vec<PatchOp> {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    let diff = lcs_diff(&old_lines, &new_lines);

    let mut ops = Vec::new();
    let mut old_idx = 0;

    for chunk in &diff {
        match chunk {
            DiffChunk::Equal(_) => {
                old_idx += 1;
            }
            DiffChunk::Added(line) => {
                ops.push(PatchOp::Insert { after_line: old_idx, content: line.clone() });
            }
            DiffChunk::Removed(_) => {
                ops.push(PatchOp::Delete { line: old_idx });
                old_idx += 1;
            }
        }
    }

    ops
}

/// A single patch operation
#[derive(Debug, Clone)]
pub enum PatchOp {
    Insert { after_line: usize, content: String },
    Delete { line: usize },
}

impl PatchOp {
    /// Describe this operation
    pub fn describe(&self) -> String {
        match self {
            PatchOp::Insert { after_line, content } => {
                format!("Insert after line {}: \"{}\"", after_line, content)
            }
            PatchOp::Delete { line } => {
                format!("Delete line {}", line)
            }
        }
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

    #[test]
    fn test_snapshot_age_string() {
        let doc = Document::with_content("Content");
        let snap = Snapshot::from_document(&doc, "Just now");
        let age = snap.age_string();
        // Just created — should be "1m ago" (minimum)
        assert!(age.contains("m ago"));
    }

    #[test]
    fn test_snapshot_empty_content() {
        let doc = Document::new();
        let snap = Snapshot::from_document(&doc, "Empty");
        assert_eq!(snap.word_count, 0);
        assert_eq!(snap.char_count(), 0);
        assert_eq!(snap.line_count(), 0);
    }

    #[test]
    fn test_diff_stats_words_positive() {
        let chunks = vec![
            DiffChunk::Added("new word here".to_string()),
        ];
        let stats = DiffStats::from_chunks(&chunks);
        assert_eq!(stats.words_added, 3);
        let summary = stats.summary();
        assert!(summary.contains("+3 words"));
    }

    #[test]
    fn test_diff_stats_words_negative() {
        let chunks = vec![
            DiffChunk::Removed("deleted these words now".to_string()),
        ];
        let stats = DiffStats::from_chunks(&chunks);
        assert_eq!(stats.words_added, -4);
        let summary = stats.summary();
        assert!(summary.contains("-4 words"));
    }

    #[test]
    fn test_diff_stats_no_word_change() {
        let chunks = vec![
            DiffChunk::Equal("same line".to_string()),
        ];
        let stats = DiffStats::from_chunks(&chunks);
        assert_eq!(stats.words_added, 0);
        let summary = stats.summary();
        assert!(summary.contains("no word change"));
    }

    #[test]
    fn test_diff_stats_total_lines() {
        let chunks = vec![
            DiffChunk::Equal("a".to_string()),
            DiffChunk::Added("b".to_string()),
            DiffChunk::Removed("c".to_string()),
            DiffChunk::Equal("d".to_string()),
        ];
        let stats = DiffStats::from_chunks(&chunks);
        assert_eq!(stats.total_lines(), 4);
        assert_eq!(stats.total_changes(), 2);
    }

    #[test]
    fn test_diff_stats_change_percentage() {
        let chunks = vec![
            DiffChunk::Equal("a".to_string()),
            DiffChunk::Added("b".to_string()),
        ];
        let stats = DiffStats::from_chunks(&chunks);
        assert!((stats.change_percentage() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_diff_stats_change_percentage_empty() {
        let chunks: Vec<DiffChunk> = vec![];
        let stats = DiffStats::from_chunks(&chunks);
        assert_eq!(stats.change_percentage(), 0.0);
    }

    #[test]
    fn test_unified_diff_no_changes() {
        let chunks = vec![
            DiffChunk::Equal("same".to_string()),
        ];
        let unified = format_unified_diff(&chunks, "a", "b");
        assert!(unified.contains("--- a"));
        assert!(unified.contains("+++ b"));
        // No @@ hunk headers since no changes
        assert!(!unified.contains("@@"));
    }

    #[test]
    fn test_inline_diff_completely_different() {
        let chunks = inline_diff("hello world", "foo bar");
        let added = chunks.iter().filter(|c| matches!(c, InlineDiffChunk::Added(_))).count();
        let removed = chunks.iter().filter(|c| matches!(c, InlineDiffChunk::Removed(_))).count();
        assert!(added >= 1);
        assert!(removed >= 1);
    }

    #[test]
    fn test_inline_diff_empty_old() {
        let chunks = inline_diff("", "new content");
        let added = chunks.iter().filter(|c| matches!(c, InlineDiffChunk::Added(_))).count();
        assert_eq!(added, 2);
    }

    #[test]
    fn test_inline_diff_empty_new() {
        let chunks = inline_diff("old content", "");
        let removed = chunks.iter().filter(|c| matches!(c, InlineDiffChunk::Removed(_))).count();
        assert_eq!(removed, 2);
    }

    #[test]
    fn test_inline_diff_chunk_equality() {
        let a = InlineDiffChunk::Equal("hello".to_string());
        let b = InlineDiffChunk::Equal("hello".to_string());
        assert_eq!(a, b);

        let c = InlineDiffChunk::Added("world".to_string());
        assert_ne!(a, c);
    }

    #[test]
    fn test_diff_with_multiline() {
        let doc = Document::with_content("Line 1\nLine 2\nLine 3\nLine 4\nLine 5");
        let snap = Snapshot::from_document(&doc, "v1");
        let new_content = "Line 1\nModified 2\nLine 3\nNew Line\nLine 5";
        let chunks = snap.diff_with(new_content);
        assert!(!chunks.is_empty());

        let stats = snap.diff_stats_with(new_content);
        assert!(stats.has_changes());
        assert_eq!(stats.lines_unchanged, 3); // Line 1, Line 3, Line 5
    }

    #[test]
    fn test_diff_snapshots_identical() {
        let doc = Document::with_content("Same content");
        let snap1 = Snapshot::from_document(&doc, "v1");
        let snap2 = Snapshot::from_document(&doc, "v2");

        let diff = diff_snapshots(&snap1, &snap2);
        assert!(diff.iter().all(|c| matches!(c, DiffChunk::Equal(_))));
    }

    #[test]
    fn test_lcs_diff_both_empty() {
        let old: Vec<&str> = vec![];
        let new: Vec<&str> = vec![];
        let diff = lcs_diff(&old, &new);
        assert!(diff.is_empty());
    }

    #[test]
    fn test_group_into_hunks_empty() {
        let chunks: Vec<DiffChunk> = vec![];
        let hunks = group_into_hunks(&chunks, 3);
        assert!(hunks.is_empty());
    }

    #[test]
    fn test_snapshot_paragraph_count() {
        let doc = Document::with_content("Para one.\n\nPara two.\n\nPara three.");
        let snap = Snapshot::from_document(&doc, "Test");
        assert_eq!(snap.paragraph_count(), 3);
    }

    #[test]
    fn test_snapshot_paragraph_count_empty() {
        let doc = Document::new();
        let snap = Snapshot::from_document(&doc, "Empty");
        assert_eq!(snap.paragraph_count(), 0);
    }

    #[test]
    fn test_snapshot_sentence_count() {
        let doc = Document::with_content("Hello. World! How are you?");
        let snap = Snapshot::from_document(&doc, "Test");
        assert_eq!(snap.sentence_count(), 3);
    }

    #[test]
    fn test_snapshot_has_changed() {
        let doc = Document::with_content("Original");
        let snap = Snapshot::from_document(&doc, "v1");
        assert!(!snap.has_changed("Original"));
        assert!(snap.has_changed("Modified"));
    }

    #[test]
    fn test_snapshot_label() {
        let doc = Document::with_content("Hello world");
        let snap = Snapshot::from_document(&doc, "Draft 1");
        let label = snap.label();
        assert!(label.contains("Draft 1"));
        assert!(label.contains("2 words"));
    }

    #[test]
    fn test_snapshot_similarity_identical() {
        let doc = Document::with_content("Same content");
        let snap = Snapshot::from_document(&doc, "v1");
        assert!((snap.similarity("Same content") - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_snapshot_similarity_different() {
        let doc = Document::with_content("Line one\nLine two\nLine three");
        let snap = Snapshot::from_document(&doc, "v1");
        let sim = snap.similarity("Line one\nDifferent\nLine three");
        assert!(sim > 0.0);
        assert!(sim < 1.0);
    }

    #[test]
    fn test_snapshot_similarity_empty() {
        let doc = Document::new();
        let snap = Snapshot::from_document(&doc, "empty");
        assert!((snap.similarity("") - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_diff_summary_no_changes() {
        let chunks = vec![DiffChunk::Equal("same".to_string())];
        assert_eq!(diff_summary(&chunks), "No changes");
    }

    #[test]
    fn test_diff_summary_with_changes() {
        let chunks = vec![
            DiffChunk::Equal("same".to_string()),
            DiffChunk::Added("new".to_string()),
            DiffChunk::Removed("old".to_string()),
        ];
        let summary = diff_summary(&chunks);
        assert!(summary.contains("1 added"));
        assert!(summary.contains("1 removed"));
        assert!(summary.contains("1 unchanged"));
    }

    #[test]
    fn test_compute_patch() {
        let old = "line1\nline2\nline3";
        let new = "line1\nnew line\nline3";
        let patch = compute_patch(old, new);
        assert!(!patch.is_empty());
    }

    #[test]
    fn test_compute_patch_insertion() {
        let old = "line1\nline2";
        let new = "line1\ninserted\nline2";
        let patch = compute_patch(old, new);
        let inserts: Vec<_> = patch.iter().filter(|op| matches!(op, PatchOp::Insert { .. })).collect();
        assert!(!inserts.is_empty());
    }

    #[test]
    fn test_compute_patch_deletion() {
        let old = "line1\nto_delete\nline2";
        let new = "line1\nline2";
        let patch = compute_patch(old, new);
        let deletes: Vec<_> = patch.iter().filter(|op| matches!(op, PatchOp::Delete { .. })).collect();
        assert!(!deletes.is_empty());
    }

    #[test]
    fn test_patch_op_describe_insert() {
        let op = PatchOp::Insert { after_line: 5, content: "new line".to_string() };
        let desc = op.describe();
        assert!(desc.contains("Insert"));
        assert!(desc.contains("5"));
        assert!(desc.contains("new line"));
    }

    #[test]
    fn test_patch_op_describe_delete() {
        let op = PatchOp::Delete { line: 3 };
        let desc = op.describe();
        assert!(desc.contains("Delete"));
        assert!(desc.contains("3"));
    }

    #[test]
    fn test_compute_patch_no_changes() {
        let text = "line1\nline2";
        let patch = compute_patch(text, text);
        assert!(patch.is_empty());
    }

    #[test]
    fn test_snapshot_from_empty_document() {
        let doc = Document::new();
        let snap = Snapshot::from_document(&doc, "Empty snap");
        assert_eq!(snap.word_count, 0);
        assert_eq!(snap.content, "");
        assert_eq!(snap.title, "Empty snap");
    }

    #[test]
    fn test_snapshot_diff_with_completely_different() {
        let doc = Document::with_content("Old line one\nOld line two");
        let snap = Snapshot::from_document(&doc, "v1");
        let chunks = snap.diff_with("New content entirely\nNothing similar");
        let has_added = chunks.iter().any(|c| matches!(c, DiffChunk::Added(_)));
        let has_removed = chunks.iter().any(|c| matches!(c, DiffChunk::Removed(_)));
        assert!(has_added);
        assert!(has_removed);
    }

    #[test]
    fn test_diff_stats_all_added() {
        let chunks = vec![
            DiffChunk::Added("line 1".to_string()),
            DiffChunk::Added("line 2".to_string()),
            DiffChunk::Added("line 3".to_string()),
        ];
        let stats = DiffStats::from_chunks(&chunks);
        assert_eq!(stats.lines_added, 3);
        assert_eq!(stats.lines_removed, 0);
        assert_eq!(stats.lines_unchanged, 0);
        assert!(stats.words_added > 0);
    }

    #[test]
    fn test_diff_stats_all_removed() {
        let chunks = vec![
            DiffChunk::Removed("line 1".to_string()),
            DiffChunk::Removed("line 2".to_string()),
        ];
        let stats = DiffStats::from_chunks(&chunks);
        assert_eq!(stats.lines_added, 0);
        assert_eq!(stats.lines_removed, 2);
        assert!(stats.words_added < 0);
    }

    #[test]
    fn test_diff_stats_empty_chunks() {
        let chunks: Vec<DiffChunk> = vec![];
        let stats = DiffStats::from_chunks(&chunks);
        assert_eq!(stats.total_changes(), 0);
        assert_eq!(stats.total_lines(), 0);
        assert!(!stats.has_changes());
    }

    #[test]
    fn test_snapshot_similarity_completely_different() {
        let doc = Document::with_content("AAA\nBBB\nCCC");
        let snap = Snapshot::from_document(&doc, "v1");
        let sim = snap.similarity("XXX\nYYY\nZZZ");
        assert!(sim < 0.5);
    }

    #[test]
    fn test_snapshot_has_changed_same() {
        let doc = Document::with_content("Same text");
        let snap = Snapshot::from_document(&doc, "v1");
        assert!(!snap.has_changed("Same text"));
    }

    #[test]
    fn test_snapshot_paragraph_count_single() {
        let doc = Document::with_content("Single paragraph with no double newlines.");
        let snap = Snapshot::from_document(&doc, "v1");
        assert_eq!(snap.paragraph_count(), 1);
    }

    #[test]
    fn test_snapshot_sentence_count_none() {
        let doc = Document::with_content("No sentence endings here");
        let snap = Snapshot::from_document(&doc, "v1");
        assert_eq!(snap.sentence_count(), 0);
    }

    #[test]
    fn test_diff_summary_added_only() {
        let chunks = vec![
            DiffChunk::Added("new".to_string()),
        ];
        let summary = diff_summary(&chunks);
        assert!(summary.contains("1 added"));
        assert!(!summary.contains("removed"));
    }

    #[test]
    fn test_diff_summary_removed_only() {
        let chunks = vec![
            DiffChunk::Removed("old".to_string()),
        ];
        let summary = diff_summary(&chunks);
        assert!(summary.contains("1 removed"));
        assert!(!summary.contains("added"));
    }

    #[test]
    fn test_compute_patch_add_at_end() {
        let old = "line1\nline2";
        let new = "line1\nline2\nline3";
        let patch = compute_patch(old, new);
        assert!(!patch.is_empty());
        let inserts = patch.iter().filter(|op| matches!(op, PatchOp::Insert { .. })).count();
        assert!(inserts >= 1);
    }

    #[test]
    fn test_compute_patch_delete_first() {
        let old = "first\nsecond\nthird";
        let new = "second\nthird";
        let patch = compute_patch(old, new);
        let deletes = patch.iter().filter(|op| matches!(op, PatchOp::Delete { .. })).count();
        assert!(deletes >= 1);
    }

    #[test]
    fn test_lcs_diff_single_line() {
        let old = vec!["only line"];
        let new = vec!["different line"];
        let diff = lcs_diff(&old, &new);
        assert!(!diff.is_empty());
    }

    #[test]
    fn test_inline_diff_partial_overlap() {
        let old = "the cat sat on the mat";
        let new = "the cat lay on the rug";
        let chunks = inline_diff(old, new);
        let equal = chunks.iter().filter(|c| matches!(c, InlineDiffChunk::Equal(_))).count();
        assert!(equal >= 3); // "the", "cat", "on", "the"
    }

    #[test]
    fn test_inline_diff_empty_both() {
        let chunks = inline_diff("", "");
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_unified_diff_format_with_multiple_hunks() {
        // Create a diff with changes far apart to get multiple hunks
        let mut old_lines = Vec::new();
        let mut new_lines = Vec::new();
        for i in 0..20 {
            old_lines.push(format!("line {}", i));
            new_lines.push(format!("line {}", i));
        }
        old_lines[2] = "old line 2".to_string();
        new_lines[2] = "new line 2".to_string();
        old_lines[15] = "old line 15".to_string();
        new_lines[15] = "new line 15".to_string();

        let old_refs: Vec<&str> = old_lines.iter().map(|s| s.as_str()).collect();
        let new_refs: Vec<&str> = new_lines.iter().map(|s| s.as_str()).collect();
        let diff = lcs_diff(&old_refs, &new_refs);
        let unified = format_unified_diff(&diff, "old", "new");
        assert!(unified.contains("@@"));
        assert!(unified.contains("--- old"));
        assert!(unified.contains("+++ new"));
    }

    #[test]
    fn test_diff_snapshots_added_content() {
        let doc1 = Document::with_content("Line 1");
        let snap1 = Snapshot::from_document(&doc1, "v1");
        let doc2 = Document::with_content("Line 1\nLine 2\nLine 3");
        let snap2 = Snapshot::from_document(&doc2, "v2");

        let diff = diff_snapshots(&snap1, &snap2);
        let added = diff.iter().filter(|c| matches!(c, DiffChunk::Added(_))).count();
        assert_eq!(added, 2);
    }
}
