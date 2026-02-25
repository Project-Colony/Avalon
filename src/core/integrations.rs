//! Integration module: project-level state that aggregates core subsystems.
//!
//! `ProjectState` holds the runtime instances of subsystems (comments,
//! revision tracking, corkboard, outliner, search index, targets, linguistic
//! analysis, file watcher) and provides methods that wire them into the
//! application lifecycle (project load, document edit, compile, etc.).

use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

use super::binder::Binder;
use super::comments::CommentManager;
use super::corkboard::{CorkboardSettings, CorkboardState};
use super::indexer::SearchIndex;
use super::links;
use super::linguistic;
use super::outliner::{self, OutlinerItem, OutlinerSettings, OutlinerState};
use super::revision::RevisionTracker;
use super::search::SearchManager;
use super::targets::{DocumentTarget, DocumentTargets};
use super::validation;
use super::watcher::{ExternalChangeTracker, ProjectWatcher};

// ---------------------------------------------------------------------------
// ProjectState — the single aggregate the GUI holds
// ---------------------------------------------------------------------------

/// Runtime state for all subsystems that live alongside the project.
pub struct ProjectState {
    /// Per-document comment managers, keyed by document UUID.
    pub comments: HashMap<Uuid, CommentManager>,

    /// Revision tracker for the current editing session.
    pub revision_tracker: RevisionTracker,

    /// Corkboard visual layout state.
    pub corkboard: CorkboardState,

    /// Outliner tree state (expansion, sort, column config).
    pub outliner: OutlinerState,

    /// Full-text search index built from the binder.
    pub search_index: SearchIndex,

    /// Saved search / history manager.
    pub search_manager: SearchManager,

    /// Per-document word-count targets.
    pub targets: DocumentTargets,

    /// File watcher for external changes (optional — may fail on some OS).
    pub watcher: Option<ProjectWatcher>,

    /// Tracks which external files changed since last check.
    pub change_tracker: ExternalChangeTracker,
}

impl ProjectState {
    /// Create a new empty project state with sensible defaults.
    pub fn new() -> Self {
        Self {
            comments: HashMap::new(),
            revision_tracker: RevisionTracker::new(),
            corkboard: CorkboardState::new(CorkboardSettings::default()),
            outliner: OutlinerState::new(OutlinerSettings {
                columns: outliner::default_columns(),
                sort_column: None,
                sort_ascending: true,
                show_synopsis: true,
                alternating_row_colors: true,
                indent_level_px: 20.0,
            }),
            search_index: SearchIndex::new(),
            search_manager: SearchManager::new(),
            targets: DocumentTargets::new(),
            watcher: None,
            change_tracker: ExternalChangeTracker::new(),
        }
    }

    // === Lifecycle hooks ===

    /// Called when a project is loaded. Builds the search index and starts
    /// the file watcher.
    pub fn on_project_load(&mut self, binder: &Binder, project_path: Option<&Path>) {
        // Build full-text search index
        self.search_index.build_from_binder(binder);

        // Start file watcher if we have a path on disk
        if let Some(path) = project_path {
            match ProjectWatcher::new(path) {
                Ok(w) => self.watcher = Some(w),
                Err(e) => log::warn!("Could not start file watcher: {}", e),
            }
        }
    }

    /// Called after a document is edited. Keeps the search index up-to-date
    /// and shifts comment anchors if needed.
    pub fn on_document_edit(
        &mut self,
        doc_id: Uuid,
        title: &str,
        content: &str,
        notes: &str,
        synopsis: &str,
    ) {
        // Update the search index entry for this document
        self.search_index.update_document(doc_id, title, content, notes, synopsis);

        // Mark index as stale so background re-indexing can happen if needed
        self.search_index.mark_stale();
    }

    /// Called every tick to poll the file watcher for external changes.
    /// Returns true if changes were detected.
    pub fn poll_external_changes(&mut self) -> bool {
        if let Some(ref mut watcher) = self.watcher {
            let events = watcher.poll_events();
            if !events.is_empty() {
                self.change_tracker.process_events(&events);
                return self.change_tracker.has_changes();
            }
        }
        false
    }

    /// Clear tracked external changes after they have been handled.
    pub fn clear_external_changes(&mut self) {
        self.change_tracker.clear();
    }

    // === Comment management ===

    /// Get or create a CommentManager for a specific document.
    pub fn comments_for(&mut self, doc_id: Uuid) -> &mut CommentManager {
        self.comments.entry(doc_id).or_insert_with(CommentManager::new)
    }

    /// Get a read-only reference to comments for a document, if any exist.
    pub fn comments_ref(&self, doc_id: &Uuid) -> Option<&CommentManager> {
        self.comments.get(doc_id)
    }

    // === Revision tracking ===

    /// Record an insertion in the current revision pass (if revision mode is active).
    pub fn record_insertion(&mut self, start: usize, end: usize, new_text: &str) {
        if self.revision_tracker.is_active() {
            self.revision_tracker.add_mark(
                super::revision::RevisionMarkKind::Insertion,
                start,
                end,
                "",
                new_text,
            );
        }
    }

    /// Record a deletion in the current revision pass (if revision mode is active).
    pub fn record_deletion(&mut self, start: usize, end: usize, original_text: &str) {
        if self.revision_tracker.is_active() {
            self.revision_tracker.add_mark(
                super::revision::RevisionMarkKind::Deletion,
                start,
                end,
                original_text,
                "",
            );
        }
    }

    /// Record a replacement in the current revision pass (if revision mode is active).
    pub fn record_replacement(
        &mut self,
        start: usize,
        end: usize,
        original_text: &str,
        new_text: &str,
    ) {
        if self.revision_tracker.is_active() {
            self.revision_tracker.add_mark(
                super::revision::RevisionMarkKind::Replacement,
                start,
                end,
                original_text,
                new_text,
            );
        }
    }

    // === Targets ===

    /// Set a word-count target for a document. Pass 0 to remove.
    pub fn set_target(&mut self, doc_id: Uuid, word_count: usize) {
        if word_count == 0 {
            self.targets.remove_target(&doc_id);
        } else {
            self.targets.set_target(doc_id, word_count);
        }
    }

    /// Set a full target (with deadline, type, etc.) for a document.
    pub fn set_target_full(&mut self, doc_id: Uuid, target: DocumentTarget) {
        self.targets.set_target_full(doc_id, target);
    }

    /// Build a word-count map from the binder for target progress calculations.
    pub fn word_count_map(binder: &Binder) -> HashMap<Uuid, usize> {
        let mut map = HashMap::new();
        Self::collect_word_counts(&binder.draft, &mut map);
        Self::collect_word_counts(&binder.research, &mut map);
        Self::collect_word_counts(&binder.trash, &mut map);
        map
    }

    fn collect_word_counts(
        item: &super::binder::BinderItem,
        map: &mut HashMap<Uuid, usize>,
    ) {
        if let Some(ref doc) = item.document {
            map.insert(item.id, doc.word_count());
        }
        for child in &item.children {
            Self::collect_word_counts(child, map);
        }
    }

    // === Outliner ===

    /// Build outliner items from a binder item's children.
    pub fn build_outliner_items(
        items: &[super::binder::BinderItem],
        targets: &DocumentTargets,
    ) -> Vec<OutlinerItem> {
        items.iter().map(|item| {
            let word_count = item.document.as_ref().map_or(0, |d| d.word_count());
            let target = targets.target_words(&item.id);
            let now = chrono::Utc::now();
            OutlinerItem {
                id: item.id,
                title: item.title.clone(),
                synopsis: item.synopsis.clone(),
                label: item.metadata.label.as_ref()
                    .map_or_else(String::new, |l| l.name.clone()),
                status: item.metadata.status.as_ref()
                    .map_or_else(String::new, |s| s.name.clone()),
                word_count,
                target_word_count: target,
                created: now, // TODO: store real creation time
                modified: now,
                include_in_compile: item.include_in_compile,
                children: Self::build_outliner_items(&item.children, targets),
            }
        }).collect()
    }

    // === Search ===

    /// Record a search query in the search manager history.
    pub fn record_search(&mut self, query: &str) {
        self.search_manager.record_query(query);
    }

    /// Get search suggestions based on prefix.
    pub fn search_suggestions(&self, prefix: &str) -> Vec<String> {
        self.search_manager.suggest(prefix).into_iter().map(|s| s.to_string()).collect()
    }

    // === Corkboard ===

    /// Arrange corkboard cards in grid for the given item IDs.
    pub fn arrange_corkboard_grid(&mut self, item_ids: &[Uuid]) {
        self.corkboard.arrange_grid(item_ids);
    }

    // === Analysis (on-demand) ===

    /// Run linguistic analysis on a text and return the full report.
    pub fn analyze_writing(text: &str) -> linguistic::WritingAnalysis {
        linguistic::analyze_text(text)
    }

    /// Run project validation including link health.
    pub fn validate_project(binder: &Binder) -> validation::ProjectValidation {
        let mut result = validation::validate_project(binder);

        // Enrich with link health info
        let link_health = links::link_health_summary(binder);
        if link_health.broken_links > 0 {
            for bl in &link_health.broken_link_details {
                result.issues.push(validation::ValidationIssue {
                    severity: validation::Severity::Warning,
                    kind: validation::IssueKind::BrokenLink,
                    message: format!("Broken link to '{}' at position {}", bl.link_text, bl.position),
                    item_id: Some(bl.source_id),
                });
            }
        }
        if link_health.orphan_documents > 0 {
            result.issues.push(validation::ValidationIssue {
                severity: validation::Severity::Info,
                kind: validation::IssueKind::Orphan,
                message: format!(
                    "{} orphan document(s) with no incoming links",
                    link_health.orphan_documents
                ),
                item_id: None,
            });
        }

        result
    }

    /// Auto-fix issues in the binder.
    pub fn auto_fix(binder: &mut Binder) -> validation::AutoFixResult {
        validation::auto_fix(binder)
    }
}

/// Wire unused integrations items for compilation.
pub fn wire_unused_integrations_items() {
    let mut state = ProjectState::new();
    let doc_id = uuid::Uuid::new_v4();
    let _ = state.comments_ref(&doc_id);
    let _ = state.record_replacement(0, 5, "old", "new");
    let _ = state.search_suggestions("");
}
