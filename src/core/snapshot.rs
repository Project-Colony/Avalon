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
#[derive(Debug)]
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

/// Retention policy for automatic snapshot pruning
#[derive(Debug, Clone)]
pub enum RetentionPolicy {
    /// Keep the N most recent snapshots
    KeepRecent(usize),
    /// Keep snapshots from the last N days
    KeepDays(i64),
    /// Keep all snapshots
    KeepAll,
    /// Keep at most N snapshots per day (the most recent of each day)
    PerDay(usize),
}

impl RetentionPolicy {
    /// Apply the policy and return IDs of snapshots to remove
    pub fn snapshots_to_prune<'a>(&self, snapshots: &'a [Snapshot]) -> Vec<&'a Uuid> {
        if snapshots.is_empty() {
            return Vec::new();
        }
        match self {
            RetentionPolicy::KeepRecent(n) => {
                if snapshots.len() <= *n {
                    return Vec::new();
                }
                // Snapshots should be sorted newest-first for this to work properly
                let mut sorted: Vec<&Snapshot> = snapshots.iter().collect();
                sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                sorted[*n..].iter().map(|s| &s.id).collect()
            }
            RetentionPolicy::KeepDays(days) => {
                let cutoff = Utc::now() - chrono::Duration::days(*days);
                snapshots.iter()
                    .filter(|s| s.created_at < cutoff)
                    .map(|s| &s.id)
                    .collect()
            }
            RetentionPolicy::KeepAll => Vec::new(),
            RetentionPolicy::PerDay(max_per_day) => {
                let mut by_day: std::collections::HashMap<String, Vec<&Snapshot>> =
                    std::collections::HashMap::new();
                for s in snapshots {
                    let day = s.created_at.format("%Y-%m-%d").to_string();
                    by_day.entry(day).or_default().push(s);
                }
                let mut to_prune = Vec::new();
                for (_day, mut day_snaps) in by_day {
                    day_snaps.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                    if day_snaps.len() > *max_per_day {
                        for s in &day_snaps[*max_per_day..] {
                            to_prune.push(&s.id);
                        }
                    }
                }
                to_prune
            }
        }
    }

    /// Label for display
    pub fn label(&self) -> String {
        match self {
            RetentionPolicy::KeepRecent(n) => format!("Keep last {}", n),
            RetentionPolicy::KeepDays(d) => format!("Keep {} days", d),
            RetentionPolicy::KeepAll => "Keep all".to_string(),
            RetentionPolicy::PerDay(n) => format!("Max {} per day", n),
        }
    }
}

/// Manages snapshots for a document with retention and tagging
#[derive(Debug, Clone)]
pub struct SnapshotManager {
    pub snapshots: Vec<Snapshot>,
    pub tags: std::collections::HashMap<Uuid, Vec<String>>,
    pub retention: RetentionPolicy,
}

impl SnapshotManager {
    pub fn new(retention: RetentionPolicy) -> Self {
        Self {
            snapshots: Vec::new(),
            tags: std::collections::HashMap::new(),
            retention,
        }
    }

    /// Take a snapshot of a document
    pub fn take_snapshot(&mut self, doc: &Document, title: &str) -> Uuid {
        let snap = Snapshot::from_document(doc, title);
        let id = snap.id;
        self.snapshots.push(snap);
        id
    }

    /// Tag a snapshot
    pub fn tag(&mut self, id: &Uuid, tag: &str) {
        self.tags.entry(*id).or_default().push(tag.to_string());
    }

    /// Remove a tag from a snapshot
    pub fn untag(&mut self, id: &Uuid, tag: &str) {
        if let Some(tags) = self.tags.get_mut(id) {
            tags.retain(|t| t != tag);
        }
    }

    /// Get tags for a snapshot
    pub fn get_tags(&self, id: &Uuid) -> Vec<&str> {
        self.tags.get(id)
            .map(|tags| tags.iter().map(|t| t.as_str()).collect())
            .unwrap_or_default()
    }

    /// Find snapshots by tag
    pub fn find_by_tag(&self, tag: &str) -> Vec<&Snapshot> {
        self.snapshots.iter()
            .filter(|s| {
                self.tags.get(&s.id)
                    .map(|tags| tags.iter().any(|t| t == tag))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get snapshot by ID
    pub fn get(&self, id: &Uuid) -> Option<&Snapshot> {
        self.snapshots.iter().find(|s| &s.id == id)
    }

    /// Count of snapshots
    pub fn count(&self) -> usize {
        self.snapshots.len()
    }

    /// Apply retention policy and remove excess snapshots
    pub fn prune(&mut self) -> usize {
        let to_remove: Vec<Uuid> = self.retention
            .snapshots_to_prune(&self.snapshots)
            .into_iter()
            .cloned()
            .collect();
        let count = to_remove.len();
        for id in &to_remove {
            self.tags.remove(id);
        }
        self.snapshots.retain(|s| !to_remove.contains(&s.id));
        count
    }

    /// Get all snapshots sorted by creation time (newest first)
    pub fn sorted_by_date(&self) -> Vec<&Snapshot> {
        let mut sorted: Vec<&Snapshot> = self.snapshots.iter().collect();
        sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        sorted
    }

    /// Compare two snapshots by ID
    pub fn compare(&self, older_id: &Uuid, newer_id: &Uuid) -> Option<SnapshotComparison> {
        let older = self.get(older_id)?;
        let newer = self.get(newer_id)?;
        Some(SnapshotComparison::from_snapshots(older, newer))
    }

    /// Word count trend across all snapshots (chronological order)
    pub fn word_count_trend(&self) -> Vec<(DateTime<Utc>, usize)> {
        let mut trend: Vec<(DateTime<Utc>, usize)> = self.snapshots
            .iter()
            .map(|s| (s.created_at, s.word_count))
            .collect();
        trend.sort_by_key(|(dt, _)| *dt);
        trend
    }

    /// Get all unique tags
    pub fn all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self.tags.values()
            .flat_map(|t| t.iter().cloned())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        tags.sort();
        tags
    }

    /// Summary
    pub fn summary(&self) -> String {
        if self.snapshots.is_empty() {
            return "No snapshots".to_string();
        }
        let total_tags: usize = self.tags.values().map(|t| t.len()).sum();
        format!(
            "{} snapshots, {} tags, retention: {}",
            self.snapshots.len(), total_tags, self.retention.label()
        )
    }
}

/// Structured comparison between two snapshots
#[derive(Debug)]
pub struct SnapshotComparison {
    pub older_title: String,
    pub newer_title: String,
    pub older_word_count: usize,
    pub newer_word_count: usize,
    pub word_count_delta: i64,
    pub diff_stats: DiffStats,
    pub similarity: f64,
}

impl SnapshotComparison {
    pub fn from_snapshots(older: &Snapshot, newer: &Snapshot) -> Self {
        let chunks = diff_snapshots(older, newer);
        let diff_stats = DiffStats::from_chunks(&chunks);
        let total = diff_stats.total_lines();
        let similarity = if total == 0 { 1.0 } else {
            diff_stats.lines_unchanged as f64 / total as f64
        };

        Self {
            older_title: older.title.clone(),
            newer_title: newer.title.clone(),
            older_word_count: older.word_count,
            newer_word_count: newer.word_count,
            word_count_delta: newer.word_count as i64 - older.word_count as i64,
            diff_stats,
            similarity,
        }
    }

    /// Summary
    pub fn summary(&self) -> String {
        let delta = if self.word_count_delta >= 0 {
            format!("+{}", self.word_count_delta)
        } else {
            format!("{}", self.word_count_delta)
        };
        format!(
            "{} -> {}: {} words ({} delta), {:.0}% similar, {}",
            self.older_title, self.newer_title,
            self.newer_word_count, delta,
            self.similarity * 100.0,
            self.diff_stats.summary()
        )
    }

    /// Whether the newer snapshot is longer
    pub fn grew(&self) -> bool {
        self.word_count_delta > 0
    }

    /// Whether the newer snapshot is shorter
    pub fn shrank(&self) -> bool {
        self.word_count_delta < 0
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

    // ---- SnapshotManager tests ----

    #[test]
    fn test_snapshot_manager_new() {
        let mgr = SnapshotManager::new(RetentionPolicy::KeepAll);
        assert_eq!(mgr.count(), 0);
        assert!(mgr.snapshots.is_empty());
    }

    #[test]
    fn test_snapshot_manager_take_snapshot() {
        let mut mgr = SnapshotManager::new(RetentionPolicy::KeepAll);
        let doc = Document::with_content("Hello world");
        let id = mgr.take_snapshot(&doc, "v1");
        assert_eq!(mgr.count(), 1);
        assert!(mgr.get(&id).is_some());
        assert_eq!(mgr.get(&id).unwrap().word_count, 2);
    }

    #[test]
    fn test_snapshot_manager_tagging() {
        let mut mgr = SnapshotManager::new(RetentionPolicy::KeepAll);
        let doc = Document::with_content("Content");
        let id = mgr.take_snapshot(&doc, "v1");

        mgr.tag(&id, "milestone");
        mgr.tag(&id, "draft");
        assert_eq!(mgr.get_tags(&id).len(), 2);
        assert!(mgr.get_tags(&id).contains(&"milestone"));

        mgr.untag(&id, "draft");
        assert_eq!(mgr.get_tags(&id).len(), 1);
        assert!(!mgr.get_tags(&id).contains(&"draft"));
    }

    #[test]
    fn test_snapshot_manager_find_by_tag() {
        let mut mgr = SnapshotManager::new(RetentionPolicy::KeepAll);
        let doc1 = Document::with_content("First");
        let id1 = mgr.take_snapshot(&doc1, "v1");
        let doc2 = Document::with_content("Second");
        let _id2 = mgr.take_snapshot(&doc2, "v2");

        mgr.tag(&id1, "important");
        let found = mgr.find_by_tag("important");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].title, "v1");

        let not_found = mgr.find_by_tag("nonexistent");
        assert!(not_found.is_empty());
    }

    #[test]
    fn test_snapshot_manager_sorted_by_date() {
        let mut mgr = SnapshotManager::new(RetentionPolicy::KeepAll);
        let doc = Document::with_content("Content");
        mgr.take_snapshot(&doc, "first");
        mgr.take_snapshot(&doc, "second");
        mgr.take_snapshot(&doc, "third");

        let sorted = mgr.sorted_by_date();
        assert_eq!(sorted.len(), 3);
        // Most recent first
        assert!(sorted[0].created_at >= sorted[1].created_at);
        assert!(sorted[1].created_at >= sorted[2].created_at);
    }

    #[test]
    fn test_snapshot_manager_word_count_trend() {
        let mut mgr = SnapshotManager::new(RetentionPolicy::KeepAll);
        let doc1 = Document::with_content("One");
        mgr.take_snapshot(&doc1, "v1");
        let doc2 = Document::with_content("One two three");
        mgr.take_snapshot(&doc2, "v2");
        let doc3 = Document::with_content("One two three four five");
        mgr.take_snapshot(&doc3, "v3");

        let trend = mgr.word_count_trend();
        assert_eq!(trend.len(), 3);
        assert_eq!(trend[0].1, 1);
        assert_eq!(trend[1].1, 3);
        assert_eq!(trend[2].1, 5);
    }

    #[test]
    fn test_snapshot_manager_all_tags() {
        let mut mgr = SnapshotManager::new(RetentionPolicy::KeepAll);
        let doc = Document::with_content("Content");
        let id1 = mgr.take_snapshot(&doc, "v1");
        let id2 = mgr.take_snapshot(&doc, "v2");

        mgr.tag(&id1, "milestone");
        mgr.tag(&id1, "draft");
        mgr.tag(&id2, "milestone");
        mgr.tag(&id2, "final");

        let all = mgr.all_tags();
        assert_eq!(all.len(), 3); // draft, final, milestone (sorted)
        assert_eq!(all[0], "draft");
        assert_eq!(all[1], "final");
        assert_eq!(all[2], "milestone");
    }

    #[test]
    fn test_snapshot_manager_compare() {
        let mut mgr = SnapshotManager::new(RetentionPolicy::KeepAll);
        let doc1 = Document::with_content("Hello world");
        let id1 = mgr.take_snapshot(&doc1, "v1");
        let doc2 = Document::with_content("Hello world, how are you today");
        let id2 = mgr.take_snapshot(&doc2, "v2");

        let cmp = mgr.compare(&id1, &id2).unwrap();
        assert_eq!(cmp.older_title, "v1");
        assert_eq!(cmp.newer_title, "v2");
        assert!(cmp.grew());
        assert!(!cmp.shrank());
        assert!(cmp.word_count_delta > 0);
    }

    #[test]
    fn test_snapshot_manager_compare_not_found() {
        let mgr = SnapshotManager::new(RetentionPolicy::KeepAll);
        let fake = Uuid::new_v4();
        assert!(mgr.compare(&fake, &fake).is_none());
    }

    #[test]
    fn test_snapshot_manager_summary() {
        let mut mgr = SnapshotManager::new(RetentionPolicy::KeepRecent(10));
        assert_eq!(mgr.summary(), "No snapshots");

        let doc = Document::with_content("Content");
        let id = mgr.take_snapshot(&doc, "v1");
        mgr.tag(&id, "test");
        let summary = mgr.summary();
        assert!(summary.contains("1 snapshots"));
        assert!(summary.contains("1 tags"));
        assert!(summary.contains("Keep last 10"));
    }

    // ---- RetentionPolicy tests ----

    #[test]
    fn test_retention_keep_all() {
        let policy = RetentionPolicy::KeepAll;
        let doc = Document::with_content("A");
        let snaps: Vec<Snapshot> = (0..10)
            .map(|i| Snapshot::from_document(&doc, &format!("v{}", i)))
            .collect();
        let to_prune = policy.snapshots_to_prune(&snaps);
        assert!(to_prune.is_empty());
    }

    #[test]
    fn test_retention_keep_recent() {
        let policy = RetentionPolicy::KeepRecent(3);
        let doc = Document::with_content("A");
        let snaps: Vec<Snapshot> = (0..5)
            .map(|i| Snapshot::from_document(&doc, &format!("v{}", i)))
            .collect();
        let to_prune = policy.snapshots_to_prune(&snaps);
        assert_eq!(to_prune.len(), 2); // Remove 2 oldest
    }

    #[test]
    fn test_retention_keep_recent_fewer_than_limit() {
        let policy = RetentionPolicy::KeepRecent(10);
        let doc = Document::with_content("A");
        let snaps: Vec<Snapshot> = (0..3)
            .map(|i| Snapshot::from_document(&doc, &format!("v{}", i)))
            .collect();
        let to_prune = policy.snapshots_to_prune(&snaps);
        assert!(to_prune.is_empty());
    }

    #[test]
    fn test_retention_keep_days() {
        let policy = RetentionPolicy::KeepDays(7);
        // All snapshots are just created, so nothing to prune
        let doc = Document::with_content("A");
        let snaps: Vec<Snapshot> = (0..3)
            .map(|i| Snapshot::from_document(&doc, &format!("v{}", i)))
            .collect();
        let to_prune = policy.snapshots_to_prune(&snaps);
        assert!(to_prune.is_empty());
    }

    #[test]
    fn test_retention_label() {
        assert_eq!(RetentionPolicy::KeepRecent(5).label(), "Keep last 5");
        assert_eq!(RetentionPolicy::KeepDays(30).label(), "Keep 30 days");
        assert_eq!(RetentionPolicy::KeepAll.label(), "Keep all");
        assert_eq!(RetentionPolicy::PerDay(3).label(), "Max 3 per day");
    }

    #[test]
    fn test_retention_empty_snapshots() {
        let policy = RetentionPolicy::KeepRecent(5);
        let snaps: Vec<Snapshot> = vec![];
        assert!(policy.snapshots_to_prune(&snaps).is_empty());
    }

    #[test]
    fn test_prune_with_manager() {
        let mut mgr = SnapshotManager::new(RetentionPolicy::KeepRecent(2));
        let doc = Document::with_content("Content");
        let id1 = mgr.take_snapshot(&doc, "v1");
        mgr.tag(&id1, "old");
        mgr.take_snapshot(&doc, "v2");
        mgr.take_snapshot(&doc, "v3");

        assert_eq!(mgr.count(), 3);
        let pruned = mgr.prune();
        assert_eq!(pruned, 1); // Should remove oldest
        assert_eq!(mgr.count(), 2);
        // Tags for pruned snapshot should be gone
        assert!(mgr.get_tags(&id1).is_empty());
    }

    // ---- SnapshotComparison tests ----

    #[test]
    fn test_snapshot_comparison_grew() {
        let doc1 = Document::with_content("Short");
        let s1 = Snapshot::from_document(&doc1, "v1");
        let doc2 = Document::with_content("A much longer document with more words");
        let s2 = Snapshot::from_document(&doc2, "v2");

        let cmp = SnapshotComparison::from_snapshots(&s1, &s2);
        assert!(cmp.grew());
        assert!(!cmp.shrank());
        assert!(cmp.word_count_delta > 0);
    }

    #[test]
    fn test_snapshot_comparison_shrank() {
        let doc1 = Document::with_content("A longer document with content");
        let s1 = Snapshot::from_document(&doc1, "v1");
        let doc2 = Document::with_content("Short");
        let s2 = Snapshot::from_document(&doc2, "v2");

        let cmp = SnapshotComparison::from_snapshots(&s1, &s2);
        assert!(!cmp.grew());
        assert!(cmp.shrank());
        assert!(cmp.word_count_delta < 0);
    }

    #[test]
    fn test_snapshot_comparison_identical() {
        let doc = Document::with_content("Same content");
        let s1 = Snapshot::from_document(&doc, "v1");
        let s2 = Snapshot::from_document(&doc, "v2");

        let cmp = SnapshotComparison::from_snapshots(&s1, &s2);
        assert!(!cmp.grew());
        assert!(!cmp.shrank());
        assert_eq!(cmp.word_count_delta, 0);
        assert!((cmp.similarity - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_snapshot_comparison_summary() {
        let doc1 = Document::with_content("Hello");
        let s1 = Snapshot::from_document(&doc1, "v1");
        let doc2 = Document::with_content("Hello world");
        let s2 = Snapshot::from_document(&doc2, "v2");

        let cmp = SnapshotComparison::from_snapshots(&s1, &s2);
        let summary = cmp.summary();
        assert!(summary.contains("v1"));
        assert!(summary.contains("v2"));
        assert!(summary.contains("+1"));
    }
}
