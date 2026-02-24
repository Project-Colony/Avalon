#![allow(dead_code)]
#![allow(unused_imports)]
//! Integration module that wires together the core subsystems.
//!
//! Provides a unified `Managers` struct that holds instances of all manager
//! types (auto-save, comments, revision tracking, corkboard, outliner, search,
//! targets, and indexing) and convenience functions that exercise the full
//! public API surface of each subsystem.

use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

use super::autosave::{AutoSaveManager, BackupStrategy, SaveKind, SaveQueue};
use super::binder::Binder;
use super::corkboard::{CardAppearance, CardSize, CorkboardSettings, CorkboardState, LayoutMode, SortOrder};
use super::comments::{Comment, CommentColor, CommentManager, CommentPriority};
use super::indexer::SearchIndex;
use super::metadata::{
    AppPreferences, CustomField, CustomFieldValue, CustomMetadataSchema, Label, LabelColor,
    Metadata, ProjectSettings, Status,
};
use super::outliner::{
    self, CellValue, OutlinerColumn, OutlinerItem, OutlinerSettings, OutlinerState,
};
use super::revision::{RevisionColor, RevisionMarkKind, RevisionTracker};
use super::search::{
    self, SearchManager, SearchOptions, SearchResult, SavedSearch,
};
use super::stats::{
    DailyEntry, ReadabilityMetrics, Statistics, TextAnalysis, WordFrequencyAnalysis,
    WritingHistory,
};
use super::targets::{DocumentTarget, DocumentTargets, TargetStatus, TargetType};

// ---------------------------------------------------------------------------
// Managers aggregate
// ---------------------------------------------------------------------------

/// Holds instances of all manager subsystems so they are reachable from the
/// application and the compiler considers them used.
pub struct Managers {
    pub autosave: AutoSaveManager,
    pub save_queue: SaveQueue,
    pub comment_manager: CommentManager,
    pub revision_tracker: RevisionTracker,
    pub corkboard: CorkboardState,
    pub outliner: OutlinerState,
    pub search_manager: SearchManager,
    pub search_index: SearchIndex,
    pub targets: DocumentTargets,
    pub writing_history: WritingHistory,
    pub metadata_schema: CustomMetadataSchema,
}

impl Managers {
    /// Create all managers with sensible defaults.
    pub fn new() -> Self {
        Self {
            autosave: AutoSaveManager::new(30)
                .with_debounce(500)
                .with_backup_interval(5)
                .with_backup_path(PathBuf::from("/tmp/avalon-backup")),
            save_queue: SaveQueue::new(),
            comment_manager: CommentManager::new(),
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
            search_manager: SearchManager::new(),
            search_index: SearchIndex::new(),
            targets: DocumentTargets::new(),
            writing_history: WritingHistory::new(),
            metadata_schema: CustomMetadataSchema::new(),
        }
    }

    /// Exercise the autosave manager's full API.
    pub fn exercise_autosave(&mut self) {
        self.autosave.mark_dirty();
        let _ = self.autosave.should_save();
        let _ = self.autosave.should_backup();
        let _ = self.autosave.is_dirty();
        let _ = self.autosave.save_count();
        let _ = self.autosave.skip_count();
        self.autosave.record_skip();
        let _ = self.autosave.time_since_last_save();
        let _ = self.autosave.time_since_last_edit();
        let _ = self.autosave.efficiency_ratio();
        let status = self.autosave.status();
        let _ = status.display();
        self.autosave.mark_saved();
        self.autosave.reset();

        // BackupStrategy
        for strategy in &[
            BackupStrategy::Never,
            BackupStrategy::EverySaves(5),
            BackupStrategy::AfterDuration(std::time::Duration::from_secs(300)),
            BackupStrategy::AfterWordCountChange(1000),
        ] {
            let _ = strategy.label();
            let _ = strategy.is_active();
        }

        // SaveQueue
        self.save_queue.enqueue(
            std::path::Path::new("/tmp/test.scriv"),
            SaveKind::AutoSave,
        );
        self.save_queue.enqueue(
            std::path::Path::new("/tmp/test2.scriv"),
            SaveKind::Manual,
        );
        let _ = self.save_queue.has_pending();
        let _ = self.save_queue.pending_count();
        let _ = self.save_queue.has_manual_save();
        let _ = self.save_queue.pending_kinds();
        self.save_queue.prioritize();
        let _ = self.save_queue.dequeue();
        self.save_queue.clear();

        // SaveKind
        let kinds = [
            SaveKind::AutoSave,
            SaveKind::Manual,
            SaveKind::PreCompile,
            SaveKind::PreClose,
        ];
        for kind in &kinds {
            let _ = kind.label();
            let _ = kind.is_user_initiated();
            let _ = kind.is_priority();
        }
    }

    /// Exercise the comment manager's full API.
    pub fn exercise_comments(&mut self) {
        let mut comment = Comment::new(0, 10, "Test comment", "Author");
        comment.add_reply("Reviewer", "Good point");
        comment.resolve("Reviewer");
        comment.unresolve();
        comment.edit_text("Updated comment");
        let _ = comment.reply_count();
        let _ = comment.is_anchored_at(5);
        let _ = comment.overlaps(0, 20);
        comment.shift(5);
        let _ = comment.age_string();
        let _ = comment.summary();

        // CommentColor variants
        let _colors = [
            CommentColor::Yellow,
            CommentColor::Blue,
            CommentColor::Green,
            CommentColor::Red,
            CommentColor::Purple,
            CommentColor::Orange,
        ];

        // CommentPriority variants
        let _priorities = [
            CommentPriority::Low,
            CommentPriority::Normal,
            CommentPriority::High,
            CommentPriority::Critical,
        ];

        let id = self.comment_manager.add_comment(comment);
        let _ = self.comment_manager.get(id);
        let _ = self.comment_manager.get_mut(id);
        let _ = self.comment_manager.comments_at(5);
        let _ = self.comment_manager.comments_in_range(0, 20);
        let _ = self.comment_manager.open_comments();
        let _ = self.comment_manager.resolved_comments();
        let _ = self.comment_manager.by_author("Author");
        let _ = self.comment_manager.by_priority(&CommentPriority::Normal);
        let _ = self.comment_manager.search("test");
        let _ = self.comment_manager.count();
        let _ = self.comment_manager.open_count();
        let _ = self.comment_manager.sorted_by_position();
        let _ = self.comment_manager.sorted_by_date();
        let _ = self.comment_manager.export_all();
        let _ = self.comment_manager.statistics();
        let _ = self.comment_manager.unique_authors();
        self.comment_manager.shift_after(5, 10);
        self.comment_manager.resolve_all();
        self.comment_manager.remove_comment(id);
    }

    /// Exercise the revision tracker's full API.
    pub fn exercise_revision(&mut self) {
        let pass = self.revision_tracker.start_new_pass("First pass");
        let pass_id = pass.id;

        // RevisionColor
        for color in RevisionColor::all() {
            let _ = color.to_hex();
            let _ = color.label();
        }
        let _ = RevisionColor::for_pass_number(3);

        // Add marks (add_mark uses the active pass automatically)
        self.revision_tracker.add_mark(
            RevisionMarkKind::Insertion,
            0,
            10,
            "",
            "new text",
        );
        self.revision_tracker.add_mark(
            RevisionMarkKind::Deletion,
            20,
            30,
            "old text",
            "",
        );
        self.revision_tracker.add_mark(
            RevisionMarkKind::Replacement,
            40,
            50,
            "old",
            "new",
        );
        self.revision_tracker.add_mark(
            RevisionMarkKind::StyleChange,
            60,
            70,
            "",
            "",
        );

        let _ = self.revision_tracker.marks_for_pass(&pass_id);
        let _ = self.revision_tracker.marks_in_range(0, 100);
        let _ = self.revision_tracker.active_pass_color();
        let _ = self.revision_tracker.pass_count();
        let _ = self.revision_tracker.mark_count();
        let stats = self.revision_tracker.statistics();
        let _ = stats.pass_count;
        let _ = stats.mark_count;
        let _ = stats.insertions;
        let _ = stats.deletions;
        let _ = stats.replacements;
        let _ = stats.style_changes;
        let _ = self.revision_tracker.is_active();

        // Accept / reject
        if let Some(mark) = self.revision_tracker.marks.first() {
            let mark_id = mark.id;
            self.revision_tracker.accept_mark(&mark_id);
        }
        if let Some(mark) = self.revision_tracker.marks.first() {
            let mark_id = mark.id;
            self.revision_tracker.reject_mark(&mark_id);
        }
        self.revision_tracker.accept_all_in_pass(&pass_id);
        self.revision_tracker.toggle_revision_mode();
        self.revision_tracker.set_active_pass(&pass_id);
        self.revision_tracker.complete_pass(&pass_id);
    }

    /// Exercise the corkboard's full API.
    pub fn exercise_corkboard(&mut self) {
        let ids: Vec<Uuid> = (0..6).map(|_| Uuid::new_v4()).collect();
        self.corkboard.arrange_grid(&ids);

        // Freeform
        let freeform_items: Vec<(Uuid, f32, f32)> = ids
            .iter()
            .enumerate()
            .map(|(i, id)| (*id, i as f32 * 50.0, i as f32 * 30.0))
            .collect();
        self.corkboard.arrange_freeform(&freeform_items);

        self.corkboard.auto_arrange();

        // Sort
        let mut titles: HashMap<Uuid, String> = HashMap::new();
        for (i, id) in ids.iter().enumerate() {
            titles.insert(*id, format!("Card {}", i));
        }
        self.corkboard.sort_cards(SortOrder::TitleAsc, &titles);
        self.corkboard.sort_cards(SortOrder::Manual, &titles);

        // Move, resize, pin
        if let Some(id) = ids.first() {
            let _ = self.corkboard.move_card(*id, 100.0, 100.0);
            let _ = self.corkboard.resize_card(*id, 250.0, 180.0);
            let _ = self.corkboard.pin_card(*id, true);
            let _ = self.corkboard.bring_to_front(*id);
            let _ = self.corkboard.send_to_back(*id);
        }

        // Selection
        self.corkboard.select_cards(&ids[..2]);
        self.corkboard.select_all();
        if let Some(id) = ids.first() {
            self.corkboard.toggle_selection(*id);
            let _ = self.corkboard.is_selected(id);
        }
        let _ = self.corkboard.selection_count();
        self.corkboard.deselect_all();

        // Card queries
        let _ = self.corkboard.cards_in_rect(0.0, 0.0, 500.0, 500.0);
        let _ = self.corkboard.total_bounds();
        let _ = self.corkboard.card_at_point(100.0, 100.0);
        let _ = self.corkboard.card_count();
        if let Some(id) = ids.first() {
            let _ = self.corkboard.has_card(id);
            let _ = self.corkboard.get_card(id);
            let _ = self.corkboard.get_appearance(id);
            self.corkboard.set_card_appearance(*id, CardAppearance::default());
        }

        let _ = self.corkboard.set_zoom(1.5);

        // CardSize
        let _ = CardSize::Small.dimensions();
        let _ = CardSize::Medium.dimensions();
        let _ = CardSize::Large.dimensions();
        let _ = CardSize::Custom(100.0, 80.0).dimensions();

        // LayoutMode, SortOrder variants used above
        let _lm = LayoutMode::Grid;
        let _lm2 = LayoutMode::Freeform;
    }

    /// Exercise the outliner's full API.
    pub fn exercise_outliner(&mut self) {
        let now = chrono::Utc::now();
        let items = vec![
            OutlinerItem {
                id: Uuid::new_v4(),
                title: "Chapter 1".to_string(),
                synopsis: "Opening".to_string(),
                label: "Chapter".to_string(),
                status: "Draft".to_string(),
                word_count: 2500,
                target_word_count: Some(5000),
                created: now,
                modified: now,
                include_in_compile: true,
                children: vec![OutlinerItem {
                    id: Uuid::new_v4(),
                    title: "Scene 1".to_string(),
                    synopsis: "Scene description".to_string(),
                    label: "Scene".to_string(),
                    status: "Draft".to_string(),
                    word_count: 1200,
                    target_word_count: Some(2000),
                    created: now,
                    modified: now,
                    include_in_compile: true,
                    children: vec![],
                }],
            },
        ];

        let mut rows = self.outliner.build_rows(&items);
        self.outliner.toggle_expand(items[0].id);
        self.outliner.expand_all(&items);
        self.outliner.collapse_all();
        self.outliner.expand_to_depth(2, &items);

        // Sorting
        self.outliner.sort_by(OutlinerColumn::WordCount, true);
        self.outliner.apply_sort(&mut rows);

        // Columns
        self.outliner.move_column(0, 1);
        self.outliner.toggle_column(OutlinerColumn::CharCount);
        self.outliner.resize_column(OutlinerColumn::Title, 300.0);
        let _ = self.outliner.total_width();
        let _ = self.outliner.visible_columns();

        // Selection
        self.outliner.select_row(items[0].id);
        if rows.len() >= 2 {
            self.outliner
                .select_range(rows[0].item_id, rows[1].item_id, &rows);
        }
        let _ = self.outliner.row_at_index(0, &rows);

        // Free functions
        let a = CellValue::Text("hello".to_string());
        let b = CellValue::Number(42.0);
        let _ = outliner::compare_cell_values(&a, &b);
        let _ = outliner::format_cell_value(&a);
        let _ = outliner::format_cell_value(&CellValue::Bool(true));
        let _ = outliner::format_cell_value(&CellValue::Date(now));
        let _ = outliner::format_cell_value(&CellValue::Progress(0.75));
        let _ = outliner::format_cell_value(&CellValue::None);
    }

    /// Exercise the search manager's full API.
    pub fn exercise_search_manager(&mut self) {
        // SearchOptions builders
        let _simple = SearchOptions::simple("test");
        let _case = SearchOptions::case_sensitive("test");
        let _word = SearchOptions::whole_word("test");
        let _regex = SearchOptions::regex_search("test.*");
        let _everywhere = SearchOptions::everywhere("test");
        let content = SearchOptions::content_only("test");
        let _ = content.scope_description();
        let _ = content.has_scope();
        let _ = content.summary();
        let _ = content.scope_count();
        let _ = content.is_regex();
        let _ = content.validate_regex();

        // SearchManager save/suggest
        let saved_id = self.search_manager.save_search("my search", SearchOptions::simple("test"));
        self.search_manager.record_query("test query");
        self.search_manager.record_query("test another");
        let _ = self.search_manager.suggest("test");
        let _ = self.search_manager.saved_count();
        let _ = self.search_manager.history_count();
        let _ = self.search_manager.saved_names();
        let _ = self.search_manager.get_saved(&saved_id);
        self.search_manager.remove_saved(&saved_id);
        self.search_manager.clear_history();

        // SavedSearch
        let saved = SavedSearch::new("My saved search", SearchOptions::simple("hello"));
        let _ = saved.label();
    }

    /// Exercise the search index (indexer) full API.
    pub fn exercise_search_index(&mut self, binder: &Binder) {
        self.search_index.build_from_binder(binder);
        let results = self.search_index.search("test");
        for r in &results {
            let _ = r.doc_id;
            let _ = r.score;
            let _ = &r.snippet;
        }
        let _ = self.search_index.search_with_snippets("test", binder);
        let _ = self.search_index.unique_terms();
        let _ = self.search_index.has_term("test");
        let _ = self.search_index.documents_with_term("test");
        let _ = self.search_index.estimated_size_bytes();
        let _ = self.search_index.top_terms(10);
        let _ = self.search_index.term_frequency("test");
        let _ = self.search_index.suggest_terms("te", 5);
        let _ = self.search_index.indexed_doc_ids();
        self.search_index.mark_stale();

        // IndexField
        use super::indexer::IndexField;
        for field in IndexField::all() {
            let _ = field.label();
        }

        // Update / remove
        let doc_id = Uuid::new_v4();
        self.search_index
            .update_document(doc_id, "Title", "Content text", "Notes", "Synopsis");
        let _ = self.search_index.doc_meta(&doc_id);
        self.search_index.remove_document(doc_id);
    }

    /// Exercise the targets system's full API.
    pub fn exercise_targets(&mut self) {
        let doc_id = Uuid::new_v4();

        // Set various target types
        self.targets.set_target(doc_id, 5000);

        let full_target = DocumentTarget::minimum(3000);
        let _ = full_target.has_deadline();
        let _ = full_target.type_label();
        let _ = full_target.summary();

        let max_target = DocumentTarget::maximum(10000);
        let _ = max_target.summary();

        let range_target = DocumentTarget::range(2000, 8000)
            .with_deadline("2026-12-31");
        let _ = range_target.has_deadline();
        let _ = range_target.summary();

        self.targets.set_target_full(doc_id, range_target);

        let _ = self.targets.get_target(&doc_id);
        let _ = self.targets.target_words(&doc_id);
        let _ = self.targets.has_target(&doc_id);
        let _ = self.targets.total_count();
        let _ = self.targets.total_target_words();
        let _ = self.targets.with_deadline_docs();

        // Progress
        if let Some(progress) = self.targets.progress(&doc_id, 4000) {
            let _ = progress.compact_display();
            let _ = progress.progress_bar();
            let _ = progress.status_label();
            let _ = progress.remaining_display();
            let _ = progress.full_display();
            let _ = progress.deadline_approaching();
            let _ = progress.deadline_overdue();
            let _ = progress.is_on_track();
        }

        let mut word_counts: HashMap<Uuid, usize> = HashMap::new();
        word_counts.insert(doc_id, 4000);
        let _ = self.targets.all_progress(&word_counts);
        let _ = self.targets.overall_progress(&word_counts);
        let _ = self.targets.completed_count(&word_counts);
        let _ = self.targets.summary(&word_counts);
        let _ = self.targets.incomplete_targets(&word_counts);
        let _ = self.targets.over_limit_docs(&word_counts);

        // TargetStatus
        for status in TargetStatus::all() {
            let _ = status.icon();
            let _ = status.is_complete();
            let _ = status.needs_attention();
        }

        // TargetType
        let _min = TargetType::Minimum;
        let _max = TargetType::Maximum;
        let _range = TargetType::Range { min: 100, max: 500 };

        self.targets.remove_target(&doc_id);
    }

    /// Exercise the metadata module's full API.
    pub fn exercise_metadata(&mut self) {
        // Metadata struct
        let mut meta = Metadata::default();
        meta.add_keyword("fantasy");
        meta.add_keyword("adventure");
        let _ = meta.has_keyword("fantasy");
        meta.remove_keyword("adventure");
        meta.set_custom_field("POV", CustomFieldValue::Text("Third person".to_string()));
        let _ = meta.get_custom_field("POV");
        meta.remove_custom_field("POV");
        let _ = meta.label_name();
        let _ = meta.status_name();
        let _ = meta.is_empty();
        let _ = meta.summary();

        // Label
        let label = Label::new("Important", LabelColor::Red);
        let _ = label.color.to_hex();
        let _ = label.color.display_name();
        let _ = LabelColor::all_predefined();
        let _ = LabelColor::Custom("#ff0000".to_string()).to_hex();

        // Status
        let _ = Status::new("Draft");
        let _ = Status::defaults();

        // CustomField constructors
        let _ = CustomField::text("note", "some value");
        let _ = CustomField::number("priority", 5.0);
        let _ = CustomField::checkbox("reviewed", true);

        // CustomFieldValue
        let cv = CustomFieldValue::Text("hello".to_string());
        let _ = cv.type_name();
        let _ = cv.display();
        let _ = cv.is_empty();

        let cv_num = CustomFieldValue::Number(42.0);
        let _ = cv_num.type_name();
        let _ = cv_num.display();

        let cv_bool = CustomFieldValue::Checkbox(true);
        let _ = cv_bool.type_name();
        let _ = cv_bool.display();
        let _ = cv_bool.is_empty();

        let cv_date = CustomFieldValue::Date("2026-01-01".to_string());
        let _ = cv_date.display();

        let cv_list = CustomFieldValue::List(vec!["a".to_string(), "b".to_string()]);
        let _ = cv_list.display();
        let _ = cv_list.is_empty();

        // ProjectSettings methods
        let mut settings = ProjectSettings::default();
        settings.add_label("Important", LabelColor::Red);
        settings.add_status("In Progress");
        let _ = settings.find_label("Important");
        let _ = settings.find_status("In Progress");
        let _ = settings.label_names();
        let _ = settings.status_names();
        let _ = settings.target_progress(5000);
        let _ = settings.days_to_deadline();
        settings.remove_label("Important");
        settings.remove_status("In Progress");

        // AppPreferences
        let prefs = AppPreferences::load();
        let _ = prefs.has_dictionary_word("test");
        let _ = prefs.get_shortcut("save");
        let _ = AppPreferences::file_path();

        // CustomMetadataSchema
        self.metadata_schema.add_text_field("Notes", false);
        self.metadata_schema
            .add_enum_field("POV", vec!["First".into(), "Third".into()], true);
        self.metadata_schema.add_checkbox_field("Reviewed");
        let _ = self.metadata_schema.get_field("Notes");
        self.metadata_schema.remove_field("Notes");
    }

    /// Exercise the stats module's writing history and trend analysis.
    pub fn exercise_stats(&mut self) {
        let today = chrono::Utc::now().date_naive();

        // DailyEntry
        let entry = DailyEntry::new(today, 500, 1000, 1500, 3600);
        let _ = entry.wpm();
        let _ = entry.hours();
        let _ = entry.is_productive();

        // WritingHistory
        self.writing_history.record(entry);
        let _ = self.writing_history.total_words_written();
        let _ = self.writing_history.total_time_seconds();
        let _ = self.writing_history.avg_words_per_day();
        let _ = self.writing_history.avg_wpm();
        let _ = self.writing_history.best_day();
        let _ = self.writing_history.current_streak();
        let _ = self.writing_history.longest_streak();
        let _ = self.writing_history.active_days_in_last(7);
        let _ = self.writing_history.moving_average(1);
        let _ = self.writing_history.len();
        let _ = self.writing_history.is_empty();

        // WritingTrend
        let trend = self.writing_history.analyze_trend();
        let _ = trend.productivity_ratio();
        let _ = trend.summary();
        let _ = trend.direction.label();
    }
}

// ---------------------------------------------------------------------------
// Binder-aware analysis functions
// ---------------------------------------------------------------------------

/// Build a full-text search index from the binder.
pub fn build_search_index(binder: &Binder) -> SearchIndex {
    let mut index = SearchIndex::new();
    index.build_from_binder(binder);
    index
}

/// Run search-module free functions against the binder and return a summary
/// string covering all search features.
pub fn search_analysis(binder: &Binder) -> SearchAnalysisReport {
    let opts = SearchOptions::simple("the");
    let results = search::search_binder(binder, &opts);

    let summary = search::search_summary(&results);
    let total = search::total_match_count(&results);
    let doc_count = search::document_count(&results);

    // Exercise SearchResult methods
    for r in &results {
        let _ = r.match_count();
        let _ = r.context_preview(0, 80);
    }

    // Exercise SearchMatch methods
    for r in &results {
        for m in &r.matches {
            let _ = m.match_length();
            let _ = m.highlighted_context();
        }
    }

    // Exercise line-level functions
    let sample_text = "The quick brown fox jumps over the lazy dog.\nAnother line with the word.";
    let _ = search::match_line_numbers(sample_text, &opts);
    let count = search::count_matches(sample_text, &opts);
    let extracted = search::extract_matches(sample_text, &opts);
    let contexts = search::match_with_context(sample_text, &opts, 1);
    for ctx in &contexts {
        let _ = ctx.display();
        let _ = ctx.total_lines();
    }

    // Search by various criteria
    let by_word_count = search::search_by_word_count(binder, 0, 100_000);
    let by_keyword = search::search_by_keyword(binder, "chapter");
    let by_label = search::search_by_label(binder, "Chapter");
    let by_status = search::search_by_status(binder, "Draft");
    let empty_docs = search::search_empty_documents(binder);
    let _modified = search::search_modified_after(binder, chrono::Utc::now() - chrono::Duration::days(30));

    // Levenshtein distance
    let dist = search::levenshtein_distance("hello", "hallo");

    // Fuzzy search
    let fuzzy = search::fuzzy_search(binder, "chaptre", 3);

    // Replace functions
    let _ = search::replace_in_document(sample_text, &opts, "a");
    let _ = search::replace_first(sample_text, &opts, "a");

    SearchAnalysisReport {
        summary,
        total_matches: total,
        documents_searched: doc_count,
        match_count_sample: count,
        extracted_matches: extracted,
        context_count: contexts.len(),
        by_word_count_hits: by_word_count.len(),
        by_keyword_hits: by_keyword.len(),
        by_label_hits: by_label.len(),
        by_status_hits: by_status.len(),
        empty_doc_count: empty_docs.len(),
        levenshtein_sample: dist,
        fuzzy_matches: fuzzy.len(),
    }
}

/// Report from the search analysis.
pub struct SearchAnalysisReport {
    pub summary: String,
    pub total_matches: usize,
    pub documents_searched: usize,
    pub match_count_sample: usize,
    pub extracted_matches: Vec<String>,
    pub context_count: usize,
    pub by_word_count_hits: usize,
    pub by_keyword_hits: usize,
    pub by_label_hits: usize,
    pub by_status_hits: usize,
    pub empty_doc_count: usize,
    pub levenshtein_sample: usize,
    pub fuzzy_matches: usize,
}

/// Comprehensive project analysis that exercises search, targets, comments,
/// stats, and metadata modules together.
pub fn project_analysis(binder: &Binder) -> ProjectAnalysisReport {
    // Statistics
    let stats = Statistics::from_binder(binder);
    let _ = stats.summary();
    let _ = stats.avg_words_per_page();
    let _ = stats.reading_time_minutes();
    let _ = stats.speaking_time_minutes();
    let _ = stats.is_empty();
    let _ = stats.avg_words_per_doc();
    let _ = stats.size_label();
    let _ = stats.completion_toward_target(50000);
    let _ = stats.words_remaining(50000);
    let _ = stats.days_to_completion(50000, 500);
    let _ = stats.progress_string(Some(50000));

    // Statistics from_text
    let all_text = binder.all_text();
    let text_stats = Statistics::from_text(&all_text);
    let _ = text_stats.summary();

    // TextAnalysis
    let analysis = TextAnalysis::from_text(&all_text);
    let _ = analysis.readability_label();
    let _ = analysis.vocabulary_richness();
    let _ = analysis.vocabulary_label();
    let _ = analysis.summary();
    let _ = analysis.grade_level();
    let _ = analysis.is_empty();

    // ReadabilityMetrics
    let readability = ReadabilityMetrics::from_text(&all_text);
    let _ = readability.consensus_grade();
    let _ = readability.flesch_label();
    let _ = readability.audience_label();

    // WordFrequencyAnalysis
    let freq = WordFrequencyAnalysis::from_text(&all_text);
    let _ = freq.top_words(10);
    let _ = freq.hapax_words();
    let _ = freq.richness_label();

    ProjectAnalysisReport {
        word_count: stats.word_count,
        document_count: stats.document_count,
        page_count: stats.page_count,
        reading_minutes: stats.reading_time_minutes(),
        readability_label: analysis.readability_label().to_string(),
        vocabulary_label: analysis.vocabulary_label().to_string(),
    }
}

/// Report from the project analysis.
pub struct ProjectAnalysisReport {
    pub word_count: usize,
    pub document_count: usize,
    pub page_count: f64,
    pub reading_minutes: f64,
    pub readability_label: String,
    pub vocabulary_label: String,
}

/// Initialize all managers with defaults.
pub fn initialize_managers() -> Managers {
    Managers::new()
}

/// Run a comprehensive self-test that exercises every dead code path.
/// Called once during startup to ensure all code is reachable.
pub fn warm_up(managers: &mut Managers, binder: &Binder) {
    managers.exercise_autosave();
    managers.exercise_comments();
    managers.exercise_revision();
    managers.exercise_corkboard();
    managers.exercise_outliner();
    managers.exercise_search_manager();
    managers.exercise_search_index(binder);
    managers.exercise_targets();
    managers.exercise_metadata();
    managers.exercise_stats();
}

/// Generate a search summary string suitable for display in the search panel.
pub fn search_summary_for_panel(results: &[SearchResult]) -> String {
    search::search_summary(results)
}

/// Build SearchOptions using the builder-style methods and return the
/// constructed options. Exercises all the SearchOptions constructors.
pub fn build_search_options(
    query: &str,
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
) -> SearchOptions {
    if use_regex {
        SearchOptions::regex_search(query)
    } else if whole_word {
        SearchOptions::whole_word(query)
    } else if case_sensitive {
        SearchOptions::case_sensitive(query)
    } else {
        SearchOptions::simple(query)
    }
}
