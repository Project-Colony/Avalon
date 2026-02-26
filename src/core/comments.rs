#![allow(dead_code)] // Methods used by test code
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A margin comment (side bubble) attached to a text range in a document.
///
/// Margin comments appear alongside the text in the editor gutter, similar to
/// Scrivener-style comment bubbles. Each comment is anchored to a specific
/// character range and can have threaded replies, priority levels, and
/// resolution status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    /// Start position (byte offset) in the document text
    pub anchor_start: usize,
    /// End position (byte offset) in the document text
    pub anchor_end: usize,
    /// The comment body text
    pub text: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<String>,
    pub color: CommentColor,
    pub replies: Vec<CommentReply>,
    pub priority: CommentPriority,
}

/// A reply within a comment thread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentReply {
    pub id: Uuid,
    pub author: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
}

/// Available colors for margin comment bubbles
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommentColor {
    Yellow,
    Blue,
    Green,
    Red,
    Purple,
    Orange,
}

/// Priority level for a comment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommentPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Aggregate statistics about comments in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentStatistics {
    pub total: usize,
    pub open: usize,
    pub resolved: usize,
    pub total_replies: usize,
    pub unique_authors: usize,
    pub by_priority: PriorityBreakdown,
    pub by_color: ColorBreakdown,
}

/// Count of comments per priority level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityBreakdown {
    pub low: usize,
    pub normal: usize,
    pub high: usize,
    pub critical: usize,
}

/// Count of comments per color
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorBreakdown {
    pub yellow: usize,
    pub blue: usize,
    pub green: usize,
    pub red: usize,
    pub purple: usize,
    pub orange: usize,
}

/// Manages all margin comments for a single document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommentManager {
    pub comments: Vec<Comment>,
}

// ---------------------------------------------------------------------------
// Comment
// ---------------------------------------------------------------------------

impl Comment {
    /// Create a new open comment anchored to the given text range.
    pub fn new(start: usize, end: usize, text: &str, author: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            anchor_start: start,
            anchor_end: end,
            text: text.to_string(),
            author: author.to_string(),
            created_at: now,
            modified_at: now,
            resolved: false,
            resolved_at: None,
            resolved_by: None,
            color: CommentColor::Yellow,
            replies: Vec::new(),
            priority: CommentPriority::Normal,
        }
    }

    /// Append a threaded reply to this comment.
    pub fn add_reply(&mut self, author: &str, text: &str) {
        self.replies.push(CommentReply {
            id: Uuid::new_v4(),
            author: author.to_string(),
            text: text.to_string(),
            created_at: Utc::now(),
        });
        self.modified_at = Utc::now();
    }

    /// Mark the comment as resolved.
    pub fn resolve(&mut self, by: &str) {
        self.resolved = true;
        self.resolved_at = Some(Utc::now());
        self.resolved_by = Some(by.to_string());
        self.modified_at = Utc::now();
    }

    /// Re-open a previously resolved comment.
    pub fn unresolve(&mut self) {
        self.resolved = false;
        self.resolved_at = None;
        self.resolved_by = None;
        self.modified_at = Utc::now();
    }

    /// Replace the comment body text.
    pub fn edit_text(&mut self, new_text: &str) {
        self.text = new_text.to_string();
        self.modified_at = Utc::now();
    }

    /// Number of replies in the thread.
    pub fn reply_count(&self) -> usize {
        self.replies.len()
    }

    /// Returns `true` if the given character position falls inside the
    /// anchored range (inclusive start, exclusive end).
    pub fn is_anchored_at(&self, pos: usize) -> bool {
        pos >= self.anchor_start && pos < self.anchor_end
    }

    /// Returns `true` if the comment's anchor range overlaps `[start, end)`.
    pub fn overlaps(&self, start: usize, end: usize) -> bool {
        self.anchor_start < end && self.anchor_end > start
    }

    /// Shift the anchor positions by `offset` characters.
    ///
    /// Use a positive offset when text is inserted before the comment and a
    /// negative offset when text is deleted before the comment. Positions
    /// saturate at zero rather than wrapping.
    pub fn shift(&mut self, offset: i64) {
        if offset >= 0 {
            if let Ok(off) = usize::try_from(offset) {
                self.anchor_start = self.anchor_start.saturating_add(off);
                self.anchor_end = self.anchor_end.saturating_add(off);
            }
        } else if let Ok(off) = usize::try_from(offset.saturating_abs()) {
            self.anchor_start = self.anchor_start.saturating_sub(off);
            self.anchor_end = self.anchor_end.saturating_sub(off);
        }
    }

    /// Human-readable string describing how old the comment is.
    pub fn age_string(&self) -> String {
        let age = Utc::now().signed_duration_since(self.created_at);
        if age.num_days() > 0 {
            format!("{}d ago", age.num_days())
        } else if age.num_hours() > 0 {
            format!("{}h ago", age.num_hours())
        } else if age.num_minutes() > 0 {
            format!("{}m ago", age.num_minutes())
        } else {
            "just now".to_string()
        }
    }

    /// Short display summary for sidebar / tooltip rendering.
    pub fn summary(&self) -> String {
        let status = if self.resolved { "resolved" } else { "open" };
        let truncated = match self.text.char_indices().nth(50) {
            Some((byte_idx, _)) => format!("{}...", &self.text[..byte_idx]),
            None => self.text.clone(),
        };
        format!(
            "[{}|{}] {} — {} ({})",
            status,
            self.priority.label(),
            truncated,
            self.author,
            self.color.label(),
        )
    }
}

// ---------------------------------------------------------------------------
// CommentManager
// ---------------------------------------------------------------------------

impl CommentManager {
    /// Create an empty manager.
    pub fn new() -> Self {
        Self {
            comments: Vec::new(),
        }
    }

    /// Add a comment and return its id.
    pub fn add_comment(&mut self, comment: Comment) -> Uuid {
        let id = comment.id;
        self.comments.push(comment);
        id
    }

    /// Remove a comment by id. Returns `true` if it was found and removed.
    pub fn remove_comment(&mut self, id: Uuid) -> bool {
        let before = self.comments.len();
        self.comments.retain(|c| c.id != id);
        self.comments.len() < before
    }

    /// Look up a comment by id.
    pub fn get(&self, id: Uuid) -> Option<&Comment> {
        self.comments.iter().find(|c| c.id == id)
    }

    /// Look up a comment mutably by id.
    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut Comment> {
        self.comments.iter_mut().find(|c| c.id == id)
    }

    /// All comments whose anchor range contains `pos`.
    pub fn comments_at(&self, pos: usize) -> Vec<&Comment> {
        self.comments.iter().filter(|c| c.is_anchored_at(pos)).collect()
    }

    /// All comments whose anchor range overlaps `[start, end)`.
    pub fn comments_in_range(&self, start: usize, end: usize) -> Vec<&Comment> {
        self.comments.iter().filter(|c| c.overlaps(start, end)).collect()
    }

    /// All unresolved (open) comments.
    pub fn open_comments(&self) -> Vec<&Comment> {
        self.comments.iter().filter(|c| !c.resolved).collect()
    }

    /// All resolved comments.
    pub fn resolved_comments(&self) -> Vec<&Comment> {
        self.comments.iter().filter(|c| c.resolved).collect()
    }

    /// Comments by a specific author (case-insensitive).
    pub fn by_author(&self, author: &str) -> Vec<&Comment> {
        let lower = author.to_lowercase();
        self.comments
            .iter()
            .filter(|c| c.author.to_lowercase() == lower)
            .collect()
    }

    /// Comments matching a specific priority.
    pub fn by_priority(&self, priority: &CommentPriority) -> Vec<&Comment> {
        self.comments
            .iter()
            .filter(|c| &c.priority == priority)
            .collect()
    }

    /// Search comment text and replies for a query (case-insensitive).
    pub fn search(&self, query: &str) -> Vec<&Comment> {
        let lower = query.to_lowercase();
        self.comments
            .iter()
            .filter(|c| {
                c.text.to_lowercase().contains(&lower)
                    || c.replies.iter().any(|r| r.text.to_lowercase().contains(&lower))
            })
            .collect()
    }

    /// Resolve every open comment at once.
    pub fn resolve_all(&mut self) {
        for comment in &mut self.comments {
            if !comment.resolved {
                comment.resolved = true;
                comment.resolved_at = Some(Utc::now());
                comment.modified_at = Utc::now();
            }
        }
    }

    /// Shift all comments whose anchor starts at or after `position` by
    /// `offset` characters.
    pub fn shift_after(&mut self, position: usize, offset: i64) {
        for comment in &mut self.comments {
            if comment.anchor_start >= position {
                comment.shift(offset);
            }
        }
    }

    /// Total number of comments.
    pub fn count(&self) -> usize {
        self.comments.len()
    }

    /// Number of unresolved comments.
    pub fn open_count(&self) -> usize {
        self.comments.iter().filter(|c| !c.resolved).count()
    }

    /// Comments sorted by anchor start position (ascending).
    pub fn sorted_by_position(&self) -> Vec<&Comment> {
        let mut sorted: Vec<&Comment> = self.comments.iter().collect();
        sorted.sort_by_key(|c| c.anchor_start);
        sorted
    }

    /// Comments sorted by creation date (newest first).
    pub fn sorted_by_date(&self) -> Vec<&Comment> {
        let mut sorted: Vec<&Comment> = self.comments.iter().collect();
        sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        sorted
    }

    /// Export all comment texts as a list of formatted strings.
    pub fn export_all(&self) -> Vec<String> {
        self.comments
            .iter()
            .map(|c| {
                let status = if c.resolved { "RESOLVED" } else { "OPEN" };
                format!(
                    "[{}] ({}) @{}: {}",
                    status, c.priority.label(), c.author, c.text,
                )
            })
            .collect()
    }

    /// Compute aggregate statistics for the current comment set.
    pub fn statistics(&self) -> CommentStatistics {
        let total = self.comments.len();
        let open = self.comments.iter().filter(|c| !c.resolved).count();
        let resolved = total - open;
        let total_replies: usize = self.comments.iter().map(|c| c.replies.len()).sum();
        let unique_authors = self.unique_authors().len();

        let by_priority = PriorityBreakdown {
            low: self.comments.iter().filter(|c| c.priority == CommentPriority::Low).count(),
            normal: self.comments.iter().filter(|c| c.priority == CommentPriority::Normal).count(),
            high: self.comments.iter().filter(|c| c.priority == CommentPriority::High).count(),
            critical: self.comments.iter().filter(|c| c.priority == CommentPriority::Critical).count(),
        };

        let by_color = ColorBreakdown {
            yellow: self.comments.iter().filter(|c| c.color == CommentColor::Yellow).count(),
            blue: self.comments.iter().filter(|c| c.color == CommentColor::Blue).count(),
            green: self.comments.iter().filter(|c| c.color == CommentColor::Green).count(),
            red: self.comments.iter().filter(|c| c.color == CommentColor::Red).count(),
            purple: self.comments.iter().filter(|c| c.color == CommentColor::Purple).count(),
            orange: self.comments.iter().filter(|c| c.color == CommentColor::Orange).count(),
        };

        CommentStatistics {
            total,
            open,
            resolved,
            total_replies,
            unique_authors,
            by_priority,
            by_color,
        }
    }

    /// Distinct authors across all comments (sorted).
    pub fn unique_authors(&self) -> Vec<String> {
        let mut authors: Vec<String> = self
            .comments
            .iter()
            .filter(|c| !c.author.is_empty())
            .map(|c| c.author.clone())
            .collect();
        authors.sort();
        authors.dedup();
        authors
    }
}

// ---------------------------------------------------------------------------
// CommentColor
// ---------------------------------------------------------------------------

impl CommentColor {
    /// CSS hex value for the color.
    pub fn to_hex(&self) -> &str {
        match self {
            CommentColor::Yellow => "#f1c40f",
            CommentColor::Blue => "#3498db",
            CommentColor::Green => "#2ecc71",
            CommentColor::Red => "#e74c3c",
            CommentColor::Purple => "#9b59b6",
            CommentColor::Orange => "#e67e22",
        }
    }

    /// Human-readable label.
    pub fn label(&self) -> &str {
        match self {
            CommentColor::Yellow => "Yellow",
            CommentColor::Blue => "Blue",
            CommentColor::Green => "Green",
            CommentColor::Red => "Red",
            CommentColor::Purple => "Purple",
            CommentColor::Orange => "Orange",
        }
    }

    /// All available colors.
    pub fn all() -> Vec<Self> {
        vec![
            CommentColor::Yellow,
            CommentColor::Blue,
            CommentColor::Green,
            CommentColor::Red,
            CommentColor::Purple,
            CommentColor::Orange,
        ]
    }

    /// Next color in the cycle (wraps around).
    pub fn next(&self) -> Self {
        match self {
            CommentColor::Yellow => CommentColor::Blue,
            CommentColor::Blue => CommentColor::Green,
            CommentColor::Green => CommentColor::Red,
            CommentColor::Red => CommentColor::Purple,
            CommentColor::Purple => CommentColor::Orange,
            CommentColor::Orange => CommentColor::Yellow,
        }
    }
}

// ---------------------------------------------------------------------------
// CommentPriority
// ---------------------------------------------------------------------------

impl CommentPriority {
    /// Human-readable label.
    pub fn label(&self) -> &str {
        match self {
            CommentPriority::Low => "Low",
            CommentPriority::Normal => "Normal",
            CommentPriority::High => "High",
            CommentPriority::Critical => "Critical",
        }
    }

    /// All available priority levels.
    pub fn all() -> Vec<Self> {
        vec![
            CommentPriority::Low,
            CommentPriority::Normal,
            CommentPriority::High,
            CommentPriority::Critical,
        ]
    }

    /// Numeric importance value (Low = 1, Normal = 2, High = 3, Critical = 4).
    pub fn numeric_value(&self) -> u8 {
        match self {
            CommentPriority::Low => 1,
            CommentPriority::Normal => 2,
            CommentPriority::High => 3,
            CommentPriority::Critical => 4,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Comment construction ------------------------------------------------

    #[test]
    fn test_comment_new() {
        let c = Comment::new(10, 20, "Fix pacing", "Alice");
        assert_eq!(c.anchor_start, 10);
        assert_eq!(c.anchor_end, 20);
        assert_eq!(c.text, "Fix pacing");
        assert_eq!(c.author, "Alice");
        assert!(!c.resolved);
        assert!(c.resolved_at.is_none());
        assert!(c.resolved_by.is_none());
        assert_eq!(c.color, CommentColor::Yellow);
        assert_eq!(c.priority, CommentPriority::Normal);
        assert!(c.replies.is_empty());
    }

    #[test]
    fn test_comment_new_timestamps_match() {
        let c = Comment::new(0, 5, "t", "a");
        // created_at and modified_at should be identical at creation
        assert_eq!(c.created_at, c.modified_at);
    }

    // -- Replies -------------------------------------------------------------

    #[test]
    fn test_add_reply() {
        let mut c = Comment::new(0, 10, "Draft note", "Alice");
        c.add_reply("Bob", "Good point");
        assert_eq!(c.reply_count(), 1);
        assert_eq!(c.replies[0].author, "Bob");
        assert_eq!(c.replies[0].text, "Good point");
    }

    #[test]
    fn test_multiple_replies() {
        let mut c = Comment::new(0, 10, "Note", "Alice");
        c.add_reply("Bob", "Reply 1");
        c.add_reply("Carol", "Reply 2");
        c.add_reply("Dave", "Reply 3");
        assert_eq!(c.reply_count(), 3);
    }

    #[test]
    fn test_reply_has_unique_id() {
        let mut c = Comment::new(0, 10, "Note", "Alice");
        c.add_reply("Bob", "R1");
        c.add_reply("Carol", "R2");
        assert_ne!(c.replies[0].id, c.replies[1].id);
    }

    // -- Resolve / unresolve -------------------------------------------------

    #[test]
    fn test_resolve() {
        let mut c = Comment::new(0, 10, "Fix", "Alice");
        c.resolve("Bob");
        assert!(c.resolved);
        assert!(c.resolved_at.is_some());
        assert_eq!(c.resolved_by, Some("Bob".to_string()));
    }

    #[test]
    fn test_unresolve() {
        let mut c = Comment::new(0, 10, "Fix", "Alice");
        c.resolve("Bob");
        c.unresolve();
        assert!(!c.resolved);
        assert!(c.resolved_at.is_none());
        assert!(c.resolved_by.is_none());
    }

    // -- Edit text -----------------------------------------------------------

    #[test]
    fn test_edit_text() {
        let mut c = Comment::new(0, 10, "Old note", "Alice");
        let before = c.modified_at;
        c.edit_text("New note");
        assert_eq!(c.text, "New note");
        assert!(c.modified_at >= before);
    }

    // -- Anchor queries ------------------------------------------------------

    #[test]
    fn test_is_anchored_at() {
        let c = Comment::new(10, 20, "Note", "Alice");
        assert!(c.is_anchored_at(10));
        assert!(c.is_anchored_at(15));
        assert!(c.is_anchored_at(19));
        assert!(!c.is_anchored_at(20)); // exclusive end
        assert!(!c.is_anchored_at(9));
    }

    #[test]
    fn test_overlaps() {
        let c = Comment::new(10, 20, "Note", "Alice");
        assert!(c.overlaps(15, 25));
        assert!(c.overlaps(5, 15));
        assert!(c.overlaps(12, 18));
        assert!(c.overlaps(0, 100));  // fully contains
        assert!(c.overlaps(10, 20));  // exact match
        assert!(!c.overlaps(20, 30)); // adjacent, no overlap
        assert!(!c.overlaps(0, 10));  // adjacent, no overlap
    }

    // -- Shift ---------------------------------------------------------------

    #[test]
    fn test_shift_positive() {
        let mut c = Comment::new(10, 20, "Note", "Alice");
        c.shift(5);
        assert_eq!(c.anchor_start, 15);
        assert_eq!(c.anchor_end, 25);
    }

    #[test]
    fn test_shift_negative() {
        let mut c = Comment::new(10, 20, "Note", "Alice");
        c.shift(-3);
        assert_eq!(c.anchor_start, 7);
        assert_eq!(c.anchor_end, 17);
    }

    #[test]
    fn test_shift_saturates_at_zero() {
        let mut c = Comment::new(5, 15, "Note", "Alice");
        c.shift(-10);
        assert_eq!(c.anchor_start, 0);
        assert_eq!(c.anchor_end, 5);
    }

    // -- Display helpers -----------------------------------------------------

    #[test]
    fn test_age_string_just_created() {
        let c = Comment::new(0, 10, "Note", "Alice");
        let age = c.age_string();
        assert!(age == "just now" || age.contains("m ago"));
    }

    #[test]
    fn test_summary_open() {
        let c = Comment::new(0, 10, "Check continuity", "Alice");
        let s = c.summary();
        assert!(s.contains("open"));
        assert!(s.contains("Check continuity"));
        assert!(s.contains("Alice"));
        assert!(s.contains("Yellow"));
    }

    #[test]
    fn test_summary_resolved() {
        let mut c = Comment::new(0, 10, "Done", "Alice");
        c.resolve("Bob");
        let s = c.summary();
        assert!(s.contains("resolved"));
    }

    #[test]
    fn test_summary_truncates_long_text() {
        let long = "A".repeat(100);
        let c = Comment::new(0, 10, &long, "Alice");
        let s = c.summary();
        assert!(s.contains("..."));
        assert!(s.len() < 200);
    }

    // -- CommentColor --------------------------------------------------------

    #[test]
    fn test_color_hex_values() {
        assert_eq!(CommentColor::Yellow.to_hex(), "#f1c40f");
        assert_eq!(CommentColor::Blue.to_hex(), "#3498db");
        assert_eq!(CommentColor::Green.to_hex(), "#2ecc71");
        assert_eq!(CommentColor::Red.to_hex(), "#e74c3c");
        assert_eq!(CommentColor::Purple.to_hex(), "#9b59b6");
        assert_eq!(CommentColor::Orange.to_hex(), "#e67e22");
    }

    #[test]
    fn test_color_hex_format() {
        for color in CommentColor::all() {
            let hex = color.to_hex();
            assert!(hex.starts_with('#'));
            assert_eq!(hex.len(), 7);
        }
    }

    #[test]
    fn test_color_labels() {
        assert_eq!(CommentColor::Yellow.label(), "Yellow");
        assert_eq!(CommentColor::Blue.label(), "Blue");
        assert_eq!(CommentColor::Green.label(), "Green");
        assert_eq!(CommentColor::Red.label(), "Red");
        assert_eq!(CommentColor::Purple.label(), "Purple");
        assert_eq!(CommentColor::Orange.label(), "Orange");
    }

    #[test]
    fn test_color_all() {
        let all = CommentColor::all();
        assert_eq!(all.len(), 6);
    }

    #[test]
    fn test_color_next_full_cycle() {
        let start = CommentColor::Yellow;
        let c2 = start.next();
        assert_eq!(c2, CommentColor::Blue);
        let c3 = c2.next();
        assert_eq!(c3, CommentColor::Green);
        let c4 = c3.next();
        assert_eq!(c4, CommentColor::Red);
        let c5 = c4.next();
        assert_eq!(c5, CommentColor::Purple);
        let c6 = c5.next();
        assert_eq!(c6, CommentColor::Orange);
        let c7 = c6.next();
        assert_eq!(c7, CommentColor::Yellow); // wraps around
    }

    // -- CommentPriority -----------------------------------------------------

    #[test]
    fn test_priority_labels() {
        assert_eq!(CommentPriority::Low.label(), "Low");
        assert_eq!(CommentPriority::Normal.label(), "Normal");
        assert_eq!(CommentPriority::High.label(), "High");
        assert_eq!(CommentPriority::Critical.label(), "Critical");
    }

    #[test]
    fn test_priority_all() {
        let all = CommentPriority::all();
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn test_priority_numeric_values() {
        assert_eq!(CommentPriority::Low.numeric_value(), 1);
        assert_eq!(CommentPriority::Normal.numeric_value(), 2);
        assert_eq!(CommentPriority::High.numeric_value(), 3);
        assert_eq!(CommentPriority::Critical.numeric_value(), 4);
    }

    #[test]
    fn test_priority_numeric_ordering() {
        assert!(CommentPriority::Low.numeric_value() < CommentPriority::Normal.numeric_value());
        assert!(CommentPriority::Normal.numeric_value() < CommentPriority::High.numeric_value());
        assert!(CommentPriority::High.numeric_value() < CommentPriority::Critical.numeric_value());
    }

    // -- CommentManager construction -----------------------------------------

    #[test]
    fn test_manager_new() {
        let mgr = CommentManager::new();
        assert_eq!(mgr.count(), 0);
        assert_eq!(mgr.open_count(), 0);
        assert!(mgr.comments.is_empty());
    }

    #[test]
    fn test_manager_default() {
        let mgr = CommentManager::default();
        assert_eq!(mgr.count(), 0);
    }

    // -- Add / remove --------------------------------------------------------

    #[test]
    fn test_manager_add_comment() {
        let mut mgr = CommentManager::new();
        let c = Comment::new(0, 10, "Note", "Alice");
        let id = mgr.add_comment(c);
        assert_eq!(mgr.count(), 1);
        assert!(mgr.get(id).is_some());
    }

    #[test]
    fn test_manager_remove_comment() {
        let mut mgr = CommentManager::new();
        let c = Comment::new(0, 10, "Note", "Alice");
        let id = mgr.add_comment(c);
        assert!(mgr.remove_comment(id));
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_manager_remove_nonexistent() {
        let mut mgr = CommentManager::new();
        assert!(!mgr.remove_comment(Uuid::new_v4()));
    }

    // -- Get / get_mut -------------------------------------------------------

    #[test]
    fn test_manager_get() {
        let mut mgr = CommentManager::new();
        let c = Comment::new(5, 15, "Test", "Alice");
        let id = mgr.add_comment(c);
        let found = mgr.get(id).unwrap();
        assert_eq!(found.text, "Test");
    }

    #[test]
    fn test_manager_get_mut() {
        let mut mgr = CommentManager::new();
        let c = Comment::new(5, 15, "Original", "Alice");
        let id = mgr.add_comment(c);
        mgr.get_mut(id).unwrap().edit_text("Modified");
        assert_eq!(mgr.get(id).unwrap().text, "Modified");
    }

    #[test]
    fn test_manager_get_missing() {
        let mgr = CommentManager::new();
        assert!(mgr.get(Uuid::new_v4()).is_none());
    }

    // -- Positional queries --------------------------------------------------

    #[test]
    fn test_comments_at() {
        let mut mgr = CommentManager::new();
        mgr.add_comment(Comment::new(0, 10, "A", "Alice"));
        mgr.add_comment(Comment::new(5, 15, "B", "Bob"));
        mgr.add_comment(Comment::new(20, 30, "C", "Carol"));

        let at_7 = mgr.comments_at(7);
        assert_eq!(at_7.len(), 2); // A and B
        let at_25 = mgr.comments_at(25);
        assert_eq!(at_25.len(), 1); // C
        let at_18 = mgr.comments_at(18);
        assert_eq!(at_18.len(), 0);
    }

    #[test]
    fn test_comments_in_range() {
        let mut mgr = CommentManager::new();
        mgr.add_comment(Comment::new(0, 10, "A", "Alice"));
        mgr.add_comment(Comment::new(10, 20, "B", "Bob"));
        mgr.add_comment(Comment::new(30, 40, "C", "Carol"));

        let range = mgr.comments_in_range(5, 15);
        assert_eq!(range.len(), 2); // A and B overlap [5, 15)
        let range2 = mgr.comments_in_range(25, 28);
        assert_eq!(range2.len(), 0);
    }

    // -- Filtering -----------------------------------------------------------

    #[test]
    fn test_open_and_resolved_comments() {
        let mut mgr = CommentManager::new();
        let c1 = Comment::new(0, 10, "Open", "Alice");
        let mut c2 = Comment::new(10, 20, "Resolved", "Bob");
        c2.resolve("Carol");
        mgr.add_comment(c1);
        mgr.add_comment(c2);

        assert_eq!(mgr.open_comments().len(), 1);
        assert_eq!(mgr.resolved_comments().len(), 1);
        assert_eq!(mgr.open_count(), 1);
    }

    #[test]
    fn test_by_author() {
        let mut mgr = CommentManager::new();
        mgr.add_comment(Comment::new(0, 5, "A", "Alice"));
        mgr.add_comment(Comment::new(5, 10, "B", "Bob"));
        mgr.add_comment(Comment::new(10, 15, "C", "Alice"));

        assert_eq!(mgr.by_author("Alice").len(), 2);
        assert_eq!(mgr.by_author("Bob").len(), 1);
        assert_eq!(mgr.by_author("Charlie").len(), 0);
    }

    #[test]
    fn test_by_author_case_insensitive() {
        let mut mgr = CommentManager::new();
        mgr.add_comment(Comment::new(0, 5, "A", "Alice"));
        assert_eq!(mgr.by_author("alice").len(), 1);
        assert_eq!(mgr.by_author("ALICE").len(), 1);
    }

    #[test]
    fn test_by_priority() {
        let mut mgr = CommentManager::new();
        let c1 = Comment::new(0, 10, "Normal", "Alice");
        let mut c2 = Comment::new(10, 20, "Critical", "Bob");
        c2.priority = CommentPriority::Critical;
        mgr.add_comment(c1);
        mgr.add_comment(c2);

        assert_eq!(mgr.by_priority(&CommentPriority::Normal).len(), 1);
        assert_eq!(mgr.by_priority(&CommentPriority::Critical).len(), 1);
        assert_eq!(mgr.by_priority(&CommentPriority::Low).len(), 0);
    }

    // -- Search --------------------------------------------------------------

    #[test]
    fn test_search_comment_text() {
        let mut mgr = CommentManager::new();
        mgr.add_comment(Comment::new(0, 10, "Fix the pacing issue", "Alice"));
        mgr.add_comment(Comment::new(10, 20, "Add more detail", "Bob"));
        mgr.add_comment(Comment::new(20, 30, "Pacing is fine here", "Carol"));

        let results = mgr.search("pacing");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_in_replies() {
        let mut mgr = CommentManager::new();
        let mut c = Comment::new(0, 10, "General note", "Alice");
        c.add_reply("Bob", "I think the pacing is off");
        mgr.add_comment(c);

        let results = mgr.search("pacing");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_no_results() {
        let mut mgr = CommentManager::new();
        mgr.add_comment(Comment::new(0, 10, "Hello world", "Alice"));
        let results = mgr.search("zzzzz");
        assert!(results.is_empty());
    }

    // -- Bulk operations -----------------------------------------------------

    #[test]
    fn test_resolve_all() {
        let mut mgr = CommentManager::new();
        mgr.add_comment(Comment::new(0, 10, "A", "Alice"));
        mgr.add_comment(Comment::new(10, 20, "B", "Bob"));
        assert_eq!(mgr.open_count(), 2);

        mgr.resolve_all();
        assert_eq!(mgr.open_count(), 0);
        assert_eq!(mgr.resolved_comments().len(), 2);
    }

    #[test]
    fn test_shift_after() {
        let mut mgr = CommentManager::new();
        mgr.add_comment(Comment::new(0, 10, "Before", "Alice"));
        mgr.add_comment(Comment::new(20, 30, "After", "Bob"));

        mgr.shift_after(15, 5);
        let sorted = mgr.sorted_by_position();
        assert_eq!(sorted[0].anchor_start, 0);  // unshifted
        assert_eq!(sorted[0].anchor_end, 10);
        assert_eq!(sorted[1].anchor_start, 25); // shifted +5
        assert_eq!(sorted[1].anchor_end, 35);
    }

    #[test]
    fn test_shift_after_negative() {
        let mut mgr = CommentManager::new();
        mgr.add_comment(Comment::new(0, 10, "Before", "Alice"));
        mgr.add_comment(Comment::new(20, 30, "After", "Bob"));

        mgr.shift_after(15, -5);
        let sorted = mgr.sorted_by_position();
        assert_eq!(sorted[0].anchor_start, 0);  // unshifted
        assert_eq!(sorted[1].anchor_start, 15); // shifted -5
        assert_eq!(sorted[1].anchor_end, 25);
    }

    // -- Sorting -------------------------------------------------------------

    #[test]
    fn test_sorted_by_position() {
        let mut mgr = CommentManager::new();
        mgr.add_comment(Comment::new(30, 40, "Third", "Alice"));
        mgr.add_comment(Comment::new(0, 10, "First", "Bob"));
        mgr.add_comment(Comment::new(15, 25, "Second", "Carol"));

        let sorted = mgr.sorted_by_position();
        assert_eq!(sorted[0].text, "First");
        assert_eq!(sorted[1].text, "Second");
        assert_eq!(sorted[2].text, "Third");
    }

    #[test]
    fn test_sorted_by_date() {
        let mut mgr = CommentManager::new();
        mgr.add_comment(Comment::new(0, 10, "First", "Alice"));
        mgr.add_comment(Comment::new(10, 20, "Second", "Bob"));

        let sorted = mgr.sorted_by_date();
        assert_eq!(sorted.len(), 2);
        // Newest first — the second comment was created most recently
        assert!(sorted[0].created_at >= sorted[1].created_at);
    }

    // -- Export / statistics / authors ----------------------------------------

    #[test]
    fn test_export_all() {
        let mut mgr = CommentManager::new();
        mgr.add_comment(Comment::new(0, 10, "First note", "Alice"));
        mgr.add_comment(Comment::new(10, 20, "Second note", "Bob"));

        let exported = mgr.export_all();
        assert_eq!(exported.len(), 2);
        assert!(exported[0].contains("First note"));
        assert!(exported[0].contains("OPEN"));
        assert!(exported[1].contains("Second note"));
    }

    #[test]
    fn test_export_all_empty() {
        let mgr = CommentManager::new();
        assert!(mgr.export_all().is_empty());
    }

    #[test]
    fn test_statistics() {
        let mut mgr = CommentManager::new();
        let mut c1 = Comment::new(0, 10, "A", "Alice");
        c1.priority = CommentPriority::High;
        c1.color = CommentColor::Red;
        c1.add_reply("Bob", "Reply");

        let mut c2 = Comment::new(10, 20, "B", "Bob");
        c2.resolve("Carol");

        let mut c3 = Comment::new(20, 30, "C", "Alice");
        c3.priority = CommentPriority::Critical;
        c3.color = CommentColor::Red;

        mgr.add_comment(c1);
        mgr.add_comment(c2);
        mgr.add_comment(c3);

        let stats = mgr.statistics();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.open, 2);
        assert_eq!(stats.resolved, 1);
        assert_eq!(stats.total_replies, 1);
        assert_eq!(stats.unique_authors, 2); // Alice, Bob
        assert_eq!(stats.by_priority.high, 1);
        assert_eq!(stats.by_priority.critical, 1);
        assert_eq!(stats.by_priority.normal, 1);
        assert_eq!(stats.by_priority.low, 0);
        assert_eq!(stats.by_color.red, 2);
        assert_eq!(stats.by_color.yellow, 1);
        assert_eq!(stats.by_color.blue, 0);
    }

    #[test]
    fn test_statistics_empty() {
        let mgr = CommentManager::new();
        let stats = mgr.statistics();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.open, 0);
        assert_eq!(stats.resolved, 0);
        assert_eq!(stats.total_replies, 0);
        assert_eq!(stats.unique_authors, 0);
    }

    #[test]
    fn test_unique_authors() {
        let mut mgr = CommentManager::new();
        mgr.add_comment(Comment::new(0, 5, "A", "Alice"));
        mgr.add_comment(Comment::new(5, 10, "B", "Bob"));
        mgr.add_comment(Comment::new(10, 15, "C", "Alice"));
        mgr.add_comment(Comment::new(15, 20, "D", "")); // empty author

        let authors = mgr.unique_authors();
        assert_eq!(authors.len(), 2);
        assert!(authors.contains(&"Alice".to_string()));
        assert!(authors.contains(&"Bob".to_string()));
    }

    #[test]
    fn test_unique_authors_sorted() {
        let mut mgr = CommentManager::new();
        mgr.add_comment(Comment::new(0, 5, "A", "Zara"));
        mgr.add_comment(Comment::new(5, 10, "B", "Alice"));

        let authors = mgr.unique_authors();
        assert_eq!(authors[0], "Alice");
        assert_eq!(authors[1], "Zara");
    }

    // -- Serialization round-trip --------------------------------------------

    #[test]
    fn test_comment_serde_roundtrip() {
        let mut c = Comment::new(10, 20, "Check this scene", "Alice");
        c.priority = CommentPriority::High;
        c.color = CommentColor::Blue;
        c.add_reply("Bob", "Agreed");

        let json = serde_json::to_string(&c).unwrap();
        let deser: Comment = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.text, "Check this scene");
        assert_eq!(deser.author, "Alice");
        assert_eq!(deser.priority, CommentPriority::High);
        assert_eq!(deser.color, CommentColor::Blue);
        assert_eq!(deser.reply_count(), 1);
    }

    #[test]
    fn test_manager_serde_roundtrip() {
        let mut mgr = CommentManager::new();
        mgr.add_comment(Comment::new(0, 10, "Note one", "Alice"));
        mgr.add_comment(Comment::new(10, 20, "Note two", "Bob"));

        let json = serde_json::to_string(&mgr).unwrap();
        let deser: CommentManager = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.count(), 2);
    }
}
