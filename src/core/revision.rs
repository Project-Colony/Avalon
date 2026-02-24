use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Colors for revision passes (Scrivener uses 5 distinct colors).
/// Each pass through the document is assigned a color so the writer
/// can visually distinguish which edits belong to which editing round.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevisionColor {
    Red,      // Pass 1
    Blue,     // Pass 2
    Green,    // Pass 3
    Orange,   // Pass 4
    Purple,   // Pass 5
}

impl RevisionColor {
    /// Return the CSS hex color string for this revision color.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_hex(&self) -> &str {
        match self {
            RevisionColor::Red => "#FF0000",
            RevisionColor::Blue => "#0000FF",
            RevisionColor::Green => "#008000",
            RevisionColor::Orange => "#FFA500",
            RevisionColor::Purple => "#800080",
        }
    }

    /// Return a human-readable label for this color.
    pub fn label(&self) -> &str {
        match self {
            RevisionColor::Red => "Red",
            RevisionColor::Blue => "Blue",
            RevisionColor::Green => "Green",
            RevisionColor::Orange => "Orange",
            RevisionColor::Purple => "Purple",
        }
    }

    /// Return the color for a given pass number (1-indexed), cycling
    /// through the five available colors.
    pub fn for_pass_number(n: usize) -> Self {
        match (n.saturating_sub(1)) % 5 {
            0 => RevisionColor::Red,
            1 => RevisionColor::Blue,
            2 => RevisionColor::Green,
            3 => RevisionColor::Orange,
            4 => RevisionColor::Purple,
            _ => unreachable!(),
        }
    }

    /// Return all five revision colors in order.
    pub fn all() -> Vec<Self> {
        vec![
            RevisionColor::Red,
            RevisionColor::Blue,
            RevisionColor::Green,
            RevisionColor::Orange,
            RevisionColor::Purple,
        ]
    }
}

/// A revision pass represents a single editing session/pass through the document.
/// Writers work through a manuscript multiple times, each time focusing on
/// different aspects (structure, prose, copy editing, etc.). Each pass is
/// tracked with its own color so changes are visually distinguishable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionPass {
    pub id: Uuid,
    pub number: usize,        // Pass 1, 2, 3, etc.
    pub color: RevisionColor,
    pub label: String,         // e.g., "First Draft Review", "Copy Edit"
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub notes: String,
}

/// A single text change tracked within a revision pass.
/// Each mark records what was changed, where, and when, allowing
/// the writer to accept or reject individual edits later.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionMark {
    pub id: Uuid,
    pub pass_id: Uuid,
    pub kind: RevisionMarkKind,
    pub start: usize,
    pub end: usize,
    pub original_text: String,
    pub new_text: String,
    pub timestamp: DateTime<Utc>,
}

/// The kind of change represented by a revision mark.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevisionMarkKind {
    Insertion,
    Deletion,
    Replacement,
    StyleChange,
}

/// Summary statistics about revision activity for a document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionStatistics {
    pub pass_count: usize,
    pub mark_count: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub replacements: usize,
    pub style_changes: usize,
}

/// Manages revision tracking for a document.
/// Holds all revision passes and their associated marks, tracks which
/// pass is currently active, and provides methods for creating, querying,
/// accepting, and rejecting edits.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RevisionTracker {
    pub passes: Vec<RevisionPass>,
    pub marks: Vec<RevisionMark>,
    pub active_pass: Option<Uuid>,
    pub revision_mode_enabled: bool,
}

impl RevisionTracker {
    /// Create a new, empty revision tracker with revision mode disabled.
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            marks: Vec::new(),
            active_pass: None,
            revision_mode_enabled: false,
        }
    }

    /// Start a new revision pass with the given label.
    /// The pass number is determined automatically (next in sequence)
    /// and the color cycles through the five available revision colors.
    /// The new pass is automatically set as the active pass and
    /// revision mode is enabled.
    pub fn start_new_pass(&mut self, label: &str) -> &RevisionPass {
        let number = self.passes.len() + 1;
        let color = RevisionColor::for_pass_number(number);
        let pass = RevisionPass {
            id: Uuid::new_v4(),
            number,
            color,
            label: label.to_string(),
            created_at: Utc::now(),
            completed_at: None,
            notes: String::new(),
        };
        self.active_pass = Some(pass.id);
        self.revision_mode_enabled = true;
        self.passes.push(pass);
        self.passes.last().unwrap()
    }

    /// Mark a revision pass as completed by setting its `completed_at`
    /// timestamp. If the completed pass was the active pass, the active
    /// pass is cleared. Returns `true` if the pass was found and was
    /// not already completed.
    pub fn complete_pass(&mut self, pass_id: &Uuid) -> bool {
        if let Some(pass) = self.passes.iter_mut().find(|p| &p.id == pass_id) {
            if pass.completed_at.is_some() {
                return false;
            }
            pass.completed_at = Some(Utc::now());
            if self.active_pass.as_ref() == Some(pass_id) {
                self.active_pass = None;
            }
            true
        } else {
            false
        }
    }

    /// Set the active revision pass. Only incomplete passes can be
    /// activated. Returns `true` if the pass was found and set active.
    pub fn set_active_pass(&mut self, pass_id: &Uuid) -> bool {
        if let Some(pass) = self.passes.iter().find(|p| &p.id == pass_id) {
            if pass.completed_at.is_some() {
                return false;
            }
            self.active_pass = Some(*pass_id);
            true
        } else {
            false
        }
    }

    /// Return the color of the currently active revision pass, or `None`
    /// if no pass is active.
    pub fn active_pass_color(&self) -> Option<RevisionColor> {
        let active_id = self.active_pass.as_ref()?;
        self.passes
            .iter()
            .find(|p| &p.id == active_id)
            .map(|p| p.color)
    }

    /// Add a revision mark (tracked change) to the currently active pass.
    /// Returns `None` if no pass is active or revision mode is disabled.
    /// Returns `Some(mark_id)` on success.
    pub fn add_mark(
        &mut self,
        kind: RevisionMarkKind,
        start: usize,
        end: usize,
        original: &str,
        new_text: &str,
    ) -> Option<Uuid> {
        if !self.revision_mode_enabled {
            return None;
        }
        let pass_id = self.active_pass?;
        let mark = RevisionMark {
            id: Uuid::new_v4(),
            pass_id,
            kind,
            start,
            end,
            original_text: original.to_string(),
            new_text: new_text.to_string(),
            timestamp: Utc::now(),
        };
        let id = mark.id;
        self.marks.push(mark);
        Some(id)
    }

    /// Return all marks that belong to a given revision pass.
    pub fn marks_for_pass(&self, pass_id: &Uuid) -> Vec<&RevisionMark> {
        self.marks.iter().filter(|m| &m.pass_id == pass_id).collect()
    }

    /// Return all marks whose ranges overlap with the given `[start, end)` range.
    pub fn marks_in_range(&self, start: usize, end: usize) -> Vec<&RevisionMark> {
        self.marks
            .iter()
            .filter(|m| m.start < end && m.end > start)
            .collect()
    }

    /// Accept a single mark by removing it from the tracker.
    /// Accepting means the change is kept in the document text and the
    /// mark is simply discarded. Returns `true` if the mark was found
    /// and removed.
    pub fn accept_mark(&mut self, mark_id: &Uuid) -> bool {
        let before = self.marks.len();
        self.marks.retain(|m| &m.id != mark_id);
        self.marks.len() < before
    }

    /// Reject a single mark by removing and returning it.
    /// The caller can use the returned mark's `original_text` to revert
    /// the change in the document. Returns `None` if the mark was not found.
    pub fn reject_mark(&mut self, mark_id: &Uuid) -> Option<RevisionMark> {
        if let Some(pos) = self.marks.iter().position(|m| &m.id == mark_id) {
            Some(self.marks.remove(pos))
        } else {
            None
        }
    }

    /// Accept all marks in a given revision pass, removing them from
    /// the tracker. Returns the number of marks that were accepted.
    pub fn accept_all_in_pass(&mut self, pass_id: &Uuid) -> usize {
        let before = self.marks.len();
        self.marks.retain(|m| &m.pass_id != pass_id);
        before - self.marks.len()
    }

    /// Return the total number of revision passes.
    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }

    /// Return the total number of tracked revision marks.
    pub fn mark_count(&self) -> usize {
        self.marks.len()
    }

    /// Compute summary statistics about revision activity.
    pub fn statistics(&self) -> RevisionStatistics {
        let mut insertions = 0;
        let mut deletions = 0;
        let mut replacements = 0;
        let mut style_changes = 0;

        for mark in &self.marks {
            match mark.kind {
                RevisionMarkKind::Insertion => insertions += 1,
                RevisionMarkKind::Deletion => deletions += 1,
                RevisionMarkKind::Replacement => replacements += 1,
                RevisionMarkKind::StyleChange => style_changes += 1,
            }
        }

        RevisionStatistics {
            pass_count: self.passes.len(),
            mark_count: self.marks.len(),
            insertions,
            deletions,
            replacements,
            style_changes,
        }
    }

    /// Toggle revision mode on or off.
    pub fn toggle_revision_mode(&mut self) {
        self.revision_mode_enabled = !self.revision_mode_enabled;
    }

    /// Return whether revision mode is currently enabled and there is
    /// an active pass ready for tracking changes.
    pub fn is_active(&self) -> bool {
        self.revision_mode_enabled && self.active_pass.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- RevisionColor tests ----

    #[test]
    fn test_color_to_hex() {
        assert_eq!(RevisionColor::Red.to_hex(), "#FF0000");
        assert_eq!(RevisionColor::Blue.to_hex(), "#0000FF");
        assert_eq!(RevisionColor::Green.to_hex(), "#008000");
        assert_eq!(RevisionColor::Orange.to_hex(), "#FFA500");
        assert_eq!(RevisionColor::Purple.to_hex(), "#800080");
    }

    #[test]
    fn test_color_label() {
        assert_eq!(RevisionColor::Red.label(), "Red");
        assert_eq!(RevisionColor::Blue.label(), "Blue");
        assert_eq!(RevisionColor::Green.label(), "Green");
        assert_eq!(RevisionColor::Orange.label(), "Orange");
        assert_eq!(RevisionColor::Purple.label(), "Purple");
    }

    #[test]
    fn test_color_for_pass_number_basic() {
        assert_eq!(RevisionColor::for_pass_number(1), RevisionColor::Red);
        assert_eq!(RevisionColor::for_pass_number(2), RevisionColor::Blue);
        assert_eq!(RevisionColor::for_pass_number(3), RevisionColor::Green);
        assert_eq!(RevisionColor::for_pass_number(4), RevisionColor::Orange);
        assert_eq!(RevisionColor::for_pass_number(5), RevisionColor::Purple);
    }

    #[test]
    fn test_color_for_pass_number_cycles() {
        assert_eq!(RevisionColor::for_pass_number(6), RevisionColor::Red);
        assert_eq!(RevisionColor::for_pass_number(7), RevisionColor::Blue);
        assert_eq!(RevisionColor::for_pass_number(10), RevisionColor::Purple);
        assert_eq!(RevisionColor::for_pass_number(11), RevisionColor::Red);
    }

    #[test]
    fn test_color_for_pass_number_zero_saturates() {
        // Pass number 0 is unusual but should not panic; saturating_sub
        // maps it to index 0 which is Red.
        let color = RevisionColor::for_pass_number(0);
        assert_eq!(color, RevisionColor::Red);
    }

    #[test]
    fn test_color_all() {
        let all = RevisionColor::all();
        assert_eq!(all.len(), 5);
        assert_eq!(all[0], RevisionColor::Red);
        assert_eq!(all[1], RevisionColor::Blue);
        assert_eq!(all[2], RevisionColor::Green);
        assert_eq!(all[3], RevisionColor::Orange);
        assert_eq!(all[4], RevisionColor::Purple);
    }

    #[test]
    fn test_color_equality() {
        assert_eq!(RevisionColor::Red, RevisionColor::Red);
        assert_ne!(RevisionColor::Red, RevisionColor::Blue);
    }

    // ---- RevisionTracker basic tests ----

    #[test]
    fn test_tracker_new() {
        let tracker = RevisionTracker::new();
        assert_eq!(tracker.pass_count(), 0);
        assert_eq!(tracker.mark_count(), 0);
        assert!(!tracker.revision_mode_enabled);
        assert!(tracker.active_pass.is_none());
        assert!(!tracker.is_active());
    }

    #[test]
    fn test_tracker_default_matches_new() {
        let a = RevisionTracker::new();
        let b = RevisionTracker::default();
        assert_eq!(a.pass_count(), b.pass_count());
        assert_eq!(a.mark_count(), b.mark_count());
        assert_eq!(a.revision_mode_enabled, b.revision_mode_enabled);
        assert_eq!(a.active_pass, b.active_pass);
    }

    #[test]
    fn test_start_new_pass() {
        let mut tracker = RevisionTracker::new();
        let pass = tracker.start_new_pass("First Draft Review");
        assert_eq!(pass.number, 1);
        assert_eq!(pass.color, RevisionColor::Red);
        assert_eq!(pass.label, "First Draft Review");
        assert!(pass.completed_at.is_none());
        assert_eq!(pass.notes, "");
        assert_eq!(tracker.pass_count(), 1);
        assert!(tracker.revision_mode_enabled);
        assert!(tracker.is_active());
    }

    #[test]
    fn test_start_multiple_passes_color_cycle() {
        let mut tracker = RevisionTracker::new();
        let labels = [
            "Pass 1", "Pass 2", "Pass 3", "Pass 4", "Pass 5", "Pass 6",
        ];
        let expected_colors = [
            RevisionColor::Red,
            RevisionColor::Blue,
            RevisionColor::Green,
            RevisionColor::Orange,
            RevisionColor::Purple,
            RevisionColor::Red, // Cycles back
        ];

        for (i, label) in labels.iter().enumerate() {
            let pass = tracker.start_new_pass(label);
            assert_eq!(pass.number, i + 1);
            assert_eq!(pass.color, expected_colors[i]);
        }
        assert_eq!(tracker.pass_count(), 6);
    }

    #[test]
    fn test_complete_pass() {
        let mut tracker = RevisionTracker::new();
        let pass_id = tracker.start_new_pass("Draft Review").id;
        assert!(tracker.is_active());

        let result = tracker.complete_pass(&pass_id);
        assert!(result);
        assert!(tracker.active_pass.is_none());

        // Verify completed_at is set
        let pass = tracker.passes.iter().find(|p| p.id == pass_id).unwrap();
        assert!(pass.completed_at.is_some());
    }

    #[test]
    fn test_complete_pass_already_completed() {
        let mut tracker = RevisionTracker::new();
        let pass_id = tracker.start_new_pass("Draft").id;
        assert!(tracker.complete_pass(&pass_id));
        // Second completion attempt should fail
        assert!(!tracker.complete_pass(&pass_id));
    }

    #[test]
    fn test_complete_pass_not_found() {
        let mut tracker = RevisionTracker::new();
        let fake_id = Uuid::new_v4();
        assert!(!tracker.complete_pass(&fake_id));
    }

    #[test]
    fn test_set_active_pass() {
        let mut tracker = RevisionTracker::new();
        let id1 = tracker.start_new_pass("Pass 1").id;
        let id2 = tracker.start_new_pass("Pass 2").id;

        // Active pass should be the most recently started (Pass 2)
        assert_eq!(tracker.active_pass, Some(id2));

        // Switch to Pass 1
        assert!(tracker.set_active_pass(&id1));
        assert_eq!(tracker.active_pass, Some(id1));
    }

    #[test]
    fn test_set_active_pass_completed_fails() {
        let mut tracker = RevisionTracker::new();
        let pass_id = tracker.start_new_pass("Done").id;
        tracker.complete_pass(&pass_id);

        assert!(!tracker.set_active_pass(&pass_id));
    }

    #[test]
    fn test_set_active_pass_not_found() {
        let mut tracker = RevisionTracker::new();
        let fake_id = Uuid::new_v4();
        assert!(!tracker.set_active_pass(&fake_id));
    }

    #[test]
    fn test_active_pass_color() {
        let mut tracker = RevisionTracker::new();
        assert!(tracker.active_pass_color().is_none());

        tracker.start_new_pass("First");
        assert_eq!(tracker.active_pass_color(), Some(RevisionColor::Red));

        tracker.start_new_pass("Second");
        assert_eq!(tracker.active_pass_color(), Some(RevisionColor::Blue));
    }

    // ---- Marks tests ----

    #[test]
    fn test_add_mark_with_active_pass() {
        let mut tracker = RevisionTracker::new();
        tracker.start_new_pass("Edit");

        let mark_id = tracker.add_mark(
            RevisionMarkKind::Insertion,
            10,
            10,
            "",
            "inserted text",
        );
        assert!(mark_id.is_some());
        assert_eq!(tracker.mark_count(), 1);
    }

    #[test]
    fn test_add_mark_without_active_pass() {
        let mut tracker = RevisionTracker::new();
        // Revision mode is off and no active pass
        let mark_id = tracker.add_mark(
            RevisionMarkKind::Insertion,
            0,
            0,
            "",
            "text",
        );
        assert!(mark_id.is_none());
        assert_eq!(tracker.mark_count(), 0);
    }

    #[test]
    fn test_add_mark_revision_mode_disabled() {
        let mut tracker = RevisionTracker::new();
        tracker.start_new_pass("Edit");
        tracker.toggle_revision_mode(); // Turn off

        let mark_id = tracker.add_mark(
            RevisionMarkKind::Deletion,
            5,
            15,
            "deleted",
            "",
        );
        assert!(mark_id.is_none());
    }

    #[test]
    fn test_marks_for_pass() {
        let mut tracker = RevisionTracker::new();
        let pass1_id = tracker.start_new_pass("Pass 1").id;
        tracker.add_mark(RevisionMarkKind::Insertion, 0, 0, "", "hello");
        tracker.add_mark(RevisionMarkKind::Deletion, 5, 10, "world", "");

        let pass2_id = tracker.start_new_pass("Pass 2").id;
        tracker.add_mark(RevisionMarkKind::Replacement, 0, 5, "old", "new");

        let pass1_marks = tracker.marks_for_pass(&pass1_id);
        assert_eq!(pass1_marks.len(), 2);

        let pass2_marks = tracker.marks_for_pass(&pass2_id);
        assert_eq!(pass2_marks.len(), 1);
    }

    #[test]
    fn test_marks_in_range() {
        let mut tracker = RevisionTracker::new();
        tracker.start_new_pass("Edit");

        // Mark at [0, 5)
        tracker.add_mark(RevisionMarkKind::Insertion, 0, 5, "", "hello");
        // Mark at [10, 20)
        tracker.add_mark(RevisionMarkKind::Deletion, 10, 20, "removed", "");
        // Mark at [30, 40)
        tracker.add_mark(RevisionMarkKind::Replacement, 30, 40, "old", "new");

        // Range [8, 25) should overlap with mark at [10, 20)
        let found = tracker.marks_in_range(8, 25);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].start, 10);

        // Range [0, 50) should overlap with all three
        let all = tracker.marks_in_range(0, 50);
        assert_eq!(all.len(), 3);

        // Range [5, 10) should overlap with nothing
        let none = tracker.marks_in_range(5, 10);
        assert_eq!(none.len(), 0);
    }

    #[test]
    fn test_accept_mark() {
        let mut tracker = RevisionTracker::new();
        tracker.start_new_pass("Edit");
        let mark_id = tracker
            .add_mark(RevisionMarkKind::Insertion, 0, 5, "", "added")
            .unwrap();

        assert_eq!(tracker.mark_count(), 1);
        assert!(tracker.accept_mark(&mark_id));
        assert_eq!(tracker.mark_count(), 0);
    }

    #[test]
    fn test_accept_mark_not_found() {
        let mut tracker = RevisionTracker::new();
        let fake_id = Uuid::new_v4();
        assert!(!tracker.accept_mark(&fake_id));
    }

    #[test]
    fn test_reject_mark() {
        let mut tracker = RevisionTracker::new();
        tracker.start_new_pass("Edit");
        let mark_id = tracker
            .add_mark(RevisionMarkKind::Replacement, 0, 10, "original", "replacement")
            .unwrap();

        let rejected = tracker.reject_mark(&mark_id);
        assert!(rejected.is_some());
        let mark = rejected.unwrap();
        assert_eq!(mark.original_text, "original");
        assert_eq!(mark.new_text, "replacement");
        assert_eq!(tracker.mark_count(), 0);
    }

    #[test]
    fn test_reject_mark_not_found() {
        let mut tracker = RevisionTracker::new();
        let fake_id = Uuid::new_v4();
        assert!(tracker.reject_mark(&fake_id).is_none());
    }

    #[test]
    fn test_accept_all_in_pass() {
        let mut tracker = RevisionTracker::new();
        let pass1_id = tracker.start_new_pass("Pass 1").id;
        tracker.add_mark(RevisionMarkKind::Insertion, 0, 0, "", "a");
        tracker.add_mark(RevisionMarkKind::Insertion, 5, 5, "", "b");
        tracker.add_mark(RevisionMarkKind::Deletion, 10, 15, "c", "");

        let pass2_id = tracker.start_new_pass("Pass 2").id;
        tracker.add_mark(RevisionMarkKind::Replacement, 0, 5, "x", "y");

        assert_eq!(tracker.mark_count(), 4);

        let accepted = tracker.accept_all_in_pass(&pass1_id);
        assert_eq!(accepted, 3);
        assert_eq!(tracker.mark_count(), 1);

        // The remaining mark belongs to pass 2
        let remaining = tracker.marks_for_pass(&pass2_id);
        assert_eq!(remaining.len(), 1);
    }

    #[test]
    fn test_accept_all_in_pass_empty() {
        let mut tracker = RevisionTracker::new();
        let fake_id = Uuid::new_v4();
        assert_eq!(tracker.accept_all_in_pass(&fake_id), 0);
    }

    // ---- Toggle and state tests ----

    #[test]
    fn test_toggle_revision_mode() {
        let mut tracker = RevisionTracker::new();
        assert!(!tracker.revision_mode_enabled);

        tracker.toggle_revision_mode();
        assert!(tracker.revision_mode_enabled);

        tracker.toggle_revision_mode();
        assert!(!tracker.revision_mode_enabled);
    }

    #[test]
    fn test_is_active_requires_both() {
        let mut tracker = RevisionTracker::new();
        // Neither enabled nor active pass
        assert!(!tracker.is_active());

        // Enable mode but no pass
        tracker.toggle_revision_mode();
        assert!(!tracker.is_active());

        // Start a pass (also enables mode)
        tracker.start_new_pass("Edit");
        assert!(tracker.is_active());

        // Disable mode
        tracker.toggle_revision_mode();
        assert!(!tracker.is_active());
    }

    // ---- Statistics tests ----

    #[test]
    fn test_statistics_empty() {
        let tracker = RevisionTracker::new();
        let stats = tracker.statistics();
        assert_eq!(stats.pass_count, 0);
        assert_eq!(stats.mark_count, 0);
        assert_eq!(stats.insertions, 0);
        assert_eq!(stats.deletions, 0);
        assert_eq!(stats.replacements, 0);
        assert_eq!(stats.style_changes, 0);
    }

    #[test]
    fn test_statistics_mixed_marks() {
        let mut tracker = RevisionTracker::new();
        tracker.start_new_pass("Edit");

        tracker.add_mark(RevisionMarkKind::Insertion, 0, 0, "", "a");
        tracker.add_mark(RevisionMarkKind::Insertion, 5, 5, "", "b");
        tracker.add_mark(RevisionMarkKind::Deletion, 10, 15, "old", "");
        tracker.add_mark(RevisionMarkKind::Replacement, 20, 25, "x", "y");
        tracker.add_mark(RevisionMarkKind::StyleChange, 30, 35, "", "");

        let stats = tracker.statistics();
        assert_eq!(stats.pass_count, 1);
        assert_eq!(stats.mark_count, 5);
        assert_eq!(stats.insertions, 2);
        assert_eq!(stats.deletions, 1);
        assert_eq!(stats.replacements, 1);
        assert_eq!(stats.style_changes, 1);
    }

    #[test]
    fn test_statistics_multiple_passes() {
        let mut tracker = RevisionTracker::new();
        tracker.start_new_pass("Pass 1");
        tracker.add_mark(RevisionMarkKind::Insertion, 0, 0, "", "text");

        tracker.start_new_pass("Pass 2");
        tracker.add_mark(RevisionMarkKind::Deletion, 5, 10, "gone", "");
        tracker.add_mark(RevisionMarkKind::Deletion, 15, 20, "also gone", "");

        let stats = tracker.statistics();
        assert_eq!(stats.pass_count, 2);
        assert_eq!(stats.mark_count, 3);
        assert_eq!(stats.insertions, 1);
        assert_eq!(stats.deletions, 2);
    }

    // ---- Serialization round-trip test ----

    #[test]
    fn test_serialization_round_trip() {
        let mut tracker = RevisionTracker::new();
        tracker.start_new_pass("Draft Review");
        tracker.add_mark(RevisionMarkKind::Insertion, 0, 5, "", "hello");
        tracker.add_mark(RevisionMarkKind::Replacement, 10, 20, "old", "new");

        let json = serde_json::to_string(&tracker).unwrap();
        let deserialized: RevisionTracker = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.pass_count(), 1);
        assert_eq!(deserialized.mark_count(), 2);
        assert_eq!(deserialized.passes[0].label, "Draft Review");
        assert!(deserialized.revision_mode_enabled);
    }

    // ---- Edge case and integration tests ----

    #[test]
    fn test_complete_pass_clears_active_only_for_that_pass() {
        let mut tracker = RevisionTracker::new();
        let id1 = tracker.start_new_pass("Pass 1").id;
        let id2 = tracker.start_new_pass("Pass 2").id;

        // Active is Pass 2; completing Pass 1 should not clear active
        assert_eq!(tracker.active_pass, Some(id2));
        tracker.complete_pass(&id1);
        assert_eq!(tracker.active_pass, Some(id2));
    }

    #[test]
    fn test_marks_in_range_boundary_conditions() {
        let mut tracker = RevisionTracker::new();
        tracker.start_new_pass("Edit");

        // Mark exactly at [10, 20)
        tracker.add_mark(RevisionMarkKind::Insertion, 10, 20, "", "text");

        // Range [20, 30) should NOT overlap (end == start is non-overlapping)
        assert_eq!(tracker.marks_in_range(20, 30).len(), 0);

        // Range [0, 10) should NOT overlap
        assert_eq!(tracker.marks_in_range(0, 10).len(), 0);

        // Range [19, 21) should overlap
        assert_eq!(tracker.marks_in_range(19, 21).len(), 1);

        // Range [9, 11) should overlap
        assert_eq!(tracker.marks_in_range(9, 11).len(), 1);
    }

    #[test]
    fn test_full_workflow() {
        let mut tracker = RevisionTracker::new();

        // Start first editing pass
        let pass1_id = tracker.start_new_pass("Structure Edit").id;
        assert!(tracker.is_active());
        assert_eq!(tracker.active_pass_color(), Some(RevisionColor::Red));

        // Make some edits
        let m1 = tracker
            .add_mark(RevisionMarkKind::Deletion, 100, 150, "remove this paragraph", "")
            .unwrap();
        let _m2 = tracker
            .add_mark(RevisionMarkKind::Insertion, 200, 200, "", "new paragraph here")
            .unwrap();
        let m3 = tracker
            .add_mark(RevisionMarkKind::Replacement, 50, 60, "said", "whispered")
            .unwrap();

        assert_eq!(tracker.mark_count(), 3);

        // Reject one change
        let rejected = tracker.reject_mark(&m1).unwrap();
        assert_eq!(rejected.original_text, "remove this paragraph");
        assert_eq!(tracker.mark_count(), 2);

        // Accept one change
        assert!(tracker.accept_mark(&m3));
        assert_eq!(tracker.mark_count(), 1);

        // Complete pass 1
        assert!(tracker.complete_pass(&pass1_id));
        assert!(!tracker.is_active());

        // Start second pass
        let pass2_id = tracker.start_new_pass("Copy Edit").id;
        assert_eq!(tracker.active_pass_color(), Some(RevisionColor::Blue));

        tracker.add_mark(RevisionMarkKind::StyleChange, 10, 20, "", "");
        tracker.add_mark(RevisionMarkKind::Replacement, 30, 35, "their", "there");
        assert_eq!(tracker.mark_count(), 3); // 1 from pass 1 + 2 from pass 2

        // Accept all in pass 2
        let accepted = tracker.accept_all_in_pass(&pass2_id);
        assert_eq!(accepted, 2);
        assert_eq!(tracker.mark_count(), 1); // Still 1 from pass 1

        // Check statistics
        let stats = tracker.statistics();
        assert_eq!(stats.pass_count, 2);
        assert_eq!(stats.mark_count, 1);
        assert_eq!(stats.insertions, 1); // The remaining mark from pass 1
    }
}
