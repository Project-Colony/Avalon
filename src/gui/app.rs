use chrono::{NaiveDate, Utc};
use iced::keyboard;
use iced::widget::{column, container, row, stack, text, Space};
use iced::window;
use iced::{Element, Length, Padding, Subscription, Task as IcedTask};
use std::collections::HashMap;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::core::backup::BackupManager;
use crate::core::binder::{BinderItem, BinderItemKind};
use crate::core::find_replace::{self, FindReplaceOptions};
use crate::core::integrations::ProjectState;
use crate::core::project::Project;
use crate::core::search::{self, SearchOptions};
use crate::core::stats::{SessionStats, Statistics};
use crate::core::PROJECTS_DIR_NAME;
use crate::editor::EditorState;
use crate::export::compiler::{CompileOptions, OutputFormat, SeparatorType};
use crate::spelling::SpellChecker;
use crate::thesaurus::Thesaurus;

use super::helpers;
use super::theme::Theme;
use super::views;

// Re-export message types from the extracted module
pub use super::messages::{BottomPanel, Message, SettingsTab, ToolbarMenu, ViewMode};

// ========== Timing constants ==========
/// How often (in auto-save ticks) to check word count milestones
const MILESTONE_CHECK_INTERVAL: u32 = 5;
/// How often (in seconds) to check daily goal progress
const DAILY_GOAL_CHECK_INTERVAL: u64 = 10;
/// How often (in seconds) to record writing history
const HISTORY_RECORD_INTERVAL: u64 = 60;
/// How often (in auto-save ticks) to auto-refresh smart collections
const COLLECTION_REFRESH_INTERVAL: u32 = 30;
/// Pomodoro break reminder time in seconds (25 minutes)
const POMODORO_BREAK_SECONDS: u64 = 1500;

/// Application state
pub struct ScrineverApp {
    // === Windows ===
    pub main_window: window::Id,
    pub settings_window: Option<window::Id>,
    pub about_window: Option<window::Id>,
    pub settings_active_tab: SettingsTab,

    // === Project ===
    pub project: Option<Project>,
    pub selected_item: Option<Uuid>,
    pub editor: EditorState,

    // === View state ===
    pub view_mode: ViewMode,
    pub show_inspector: bool,
    pub fullscreen_editor: bool,
    pub bottom_panel: BottomPanel,
    pub active_toolbar_menu: Option<ToolbarMenu>,

    // === Dialogs ===
    pub show_compile_dialog: bool,
    pub compile_options: CompileOptions,

    // === Search ===
    pub search_query: String,
    pub search_results: Vec<search::SearchResult>,
    pub search_case_sensitive: bool,
    pub search_whole_word: bool,
    pub search_regex: bool,
    pub replace_text: String,

    // === Thesaurus ===
    pub spell_checker: SpellChecker,
    pub thesaurus: Thesaurus,
    pub thesaurus_query: String,
    pub thesaurus_results: Vec<crate::thesaurus::ThesaurusEntry>,

    // === Document notes ===
    pub notes_text: String,

    // === Target word count per item ===
    pub item_targets: HashMap<Uuid, usize>,

    // === Notification ===
    pub notification: Option<String>,
    pub notification_timer: u32,

    // === Auto-save ===
    pub auto_save_counter: u32,
    /// Saves of the open project since its last automatic backup.
    pub saves_since_backup: u32,

    // === Writing session ===
    pub session_active: bool,
    pub session_stats: SessionStats,
    pub session_start_word_count: usize,
    pub session_goal: usize,
    pub session_goal_text: String,

    // === Name generator ===
    pub generated_names: Vec<String>,
    pub name_gen_type: String,

    // === Project notes (scratch pad) ===
    pub project_notes_text: String,

    // === Collections ===
    pub selected_collection: Option<uuid::Uuid>,
    pub new_collection_name: String,

    // === Annotations ===
    pub annotation_text: String,
    pub annotation_next_color: crate::core::annotation::AnnotationColor,

    // === Recent projects ===
    pub recent_projects: crate::core::recent::RecentProjects,

    // === Folders the import and export dialogs open in ===
    pub last_import_dir: Option<PathBuf>,
    pub last_export_dir: Option<PathBuf>,

    // === Split editor ===
    pub split_editor_item: Option<Uuid>,

    // === Document find/replace ===
    pub doc_find_text: String,
    pub doc_replace_text: String,
    pub doc_find_case_sensitive: bool,
    pub doc_find_whole_word: bool,
    pub doc_find_use_regex: bool,
    pub doc_find_match_count: usize,
    pub doc_find_current_match: usize,
    pub doc_find_positions: Vec<usize>,

    // === Quick reference ===
    pub quick_ref_item: Option<Uuid>,

    // === Script mode ===
    pub script_mode: bool,
    pub current_script_element: Option<crate::core::script::ScriptElement>,
    pub auto_correction: crate::core::script::AutoCorrection,

    // === Compile presets ===
    pub compile_presets: Vec<(String, CompileOptions)>,
    pub footnote_counter: usize,

    // === Project statistics dialog ===
    pub show_project_stats: bool,

    // === Writing goals ===
    pub daily_goal: usize,
    pub daily_goal_text: String,
    pub weekly_goal: usize,
    pub weekly_goal_text: String,

    // === Composition mode ===
    pub composition_mode: bool,

    // === Snapshot comparison ===
    pub selected_snapshot: Option<usize>,

    // === Spell check results ===
    pub spell_check_results: Vec<crate::spelling::SpellSuggestion>,

    // === Writing timer ===
    pub writing_timer: crate::core::timer::WritingTimer,

    // === Validation results ===
    pub validation_result: Option<crate::core::validation::ProjectValidation>,

    // === Word count milestone tracking ===
    pub last_milestone: usize,

    // === Writing prompts ===
    pub writing_prompts_data: views::writing_prompts_panel::WritingPromptsData,

    // === Project subsystem state (comments, revision, corkboard, outliner, search index, targets, etc.) ===
    pub project_state: ProjectState,

    // === Linguistic analysis (on-demand, cached) ===
    pub linguistic_result: Option<crate::core::linguistic::WritingAnalysis>,
}

impl ScrineverApp {
    pub fn new() -> (Self, IcedTask<Message>) {
        let mut spell_checker = SpellChecker::new();
        spell_checker.try_init();
        spell_checker.load_user_dictionary();

        // Open the main window explicitly (daemon mode)
        let (main_id, open_main) = window::open(window::Settings {
            size: iced::Size::new(1280.0, 800.0),
            ..window::Settings::default()
        });

        let app = Self {
            main_window: main_id,
            settings_window: None,
            about_window: None,
            settings_active_tab: SettingsTab::General,
            project: None,
            selected_item: None,
            editor: EditorState::new(),
            view_mode: ViewMode::Editor,
            show_inspector: true,
            fullscreen_editor: false,
            bottom_panel: BottomPanel::None,
            active_toolbar_menu: None,
            show_compile_dialog: false,
            compile_options: CompileOptions::default(),
            search_query: String::new(),
            search_results: Vec::new(),
            search_case_sensitive: false,
            search_whole_word: false,
            search_regex: false,
            replace_text: String::new(),
            spell_checker,
            thesaurus: Thesaurus::new(),
            thesaurus_query: String::new(),
            thesaurus_results: Vec::new(),
            notes_text: String::new(),
            item_targets: HashMap::new(),
            notification: None,
            notification_timer: 0,
            auto_save_counter: 0,
            saves_since_backup: 0,
            session_active: false,
            session_stats: SessionStats::new(),
            session_start_word_count: 0,
            session_goal: 0,
            session_goal_text: String::new(),
            generated_names: Vec::new(),
            name_gen_type: "male".to_string(),
            project_notes_text: String::new(),
            selected_collection: None,
            new_collection_name: String::new(),
            annotation_text: String::new(),
            annotation_next_color: crate::core::annotation::AnnotationColor::Yellow,
            recent_projects: crate::core::recent::RecentProjects::load(),
            last_import_dir: None,
            last_export_dir: None,
            split_editor_item: None,
            doc_find_text: String::new(),
            doc_replace_text: String::new(),
            doc_find_case_sensitive: false,
            doc_find_whole_word: false,
            doc_find_use_regex: false,
            doc_find_match_count: 0,
            doc_find_current_match: 0,
            doc_find_positions: Vec::new(),
            quick_ref_item: None,
            script_mode: false,
            current_script_element: None,
            auto_correction: crate::core::script::AutoCorrection::default(),
            compile_presets: Vec::new(),
            footnote_counter: 0,
            show_project_stats: false,
            daily_goal: 0,
            daily_goal_text: String::new(),
            weekly_goal: 0,
            weekly_goal_text: String::new(),
            composition_mode: false,
            selected_snapshot: None,
            spell_check_results: Vec::new(),
            writing_timer: crate::core::timer::WritingTimer::new(),
            validation_result: None,
            last_milestone: 0,
            writing_prompts_data: views::writing_prompts_panel::WritingPromptsData::new(),
            project_state: ProjectState::new(),
            linguistic_result: None,
        };

        (app, open_main.map(Message::WindowOpened))
    }

    pub fn title(&self, window_id: window::Id) -> String {
        if Some(window_id) == self.settings_window {
            return "Avalon - Settings".to_string();
        }
        if Some(window_id) == self.about_window {
            return "Avalon - About".to_string();
        }
        let dirty = if self.editor.dirty { " *" } else { "" };
        match &self.project {
            Some(p) => format!("Avalon - {}{}", p.title, dirty),
            None => "Avalon".to_string(),
        }
    }

    /// Whether the project with this id is still the open one. File dialogs
    /// do not block the main window, so the user can switch projects while
    /// one is open; its result is dropped when that happened.
    fn is_open_project(&self, id: Uuid) -> bool {
        self.project.as_ref().is_some_and(|p| p.id == id)
    }

    /// Sync the current editor content back to the project's document
    fn sync_editor_to_project(&mut self) {
        if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
            if let Some(item) = project.binder.find_item_mut(&item_id) {
                if let Some(ref mut doc) = item.document {
                    doc.content = self.editor.text();
                }
            }
        }
    }

    /// Get the current total word count for session tracking
    fn current_word_count(&self) -> usize {
        self.project.as_ref().map_or(0, |p| p.binder.total_word_count())
    }

    /// Insert markdown-style wrapping markup (e.g., ** for bold)
    fn insert_markdown_wrap(&mut self, marker: &str) {
        let wrap = format!("{}text{}", marker, marker);
        self.editor.content.perform(iced::widget::text_editor::Action::Edit(
            iced::widget::text_editor::Edit::Paste(std::sync::Arc::new(wrap)),
        ));
        self.editor.mark_dirty();
    }

    pub fn update(&mut self, message: Message) -> IcedTask<Message> {
        // Close toolbar menu on any action except menu toggle itself and ticks
        if !matches!(
            message,
            Message::ToggleToolbarMenu(_) | Message::CloseToolbarMenu | Message::Tick | Message::EscapePressed
        ) {
            self.active_toolbar_menu = None;
        }

        match message {
            // ========== Project operations ==========
            Message::NewProject => {
                self.project = Some(Project::new("Untitled Project"));
                self.selected_item = None;
                self.editor = EditorState::new();
                self.project_notes_text.clear();
                self.generated_names.clear();
                self.last_milestone = 0;
                self.project_state = ProjectState::new();
                self.linguistic_result = None;
                if let Some(ref p) = self.project {
                    self.compile_options.title = p.title.clone();
                }
            }

            Message::NewFromTemplate(template_id) => {
                self.project = Some(Project::from_template("Untitled Project", &template_id));
                self.selected_item = None;
                self.editor = EditorState::new();
                self.project_notes_text.clear();
                self.generated_names.clear();
                self.project_state = ProjectState::new();
                self.linguistic_result = None;
                if let Some(ref p) = self.project {
                    self.compile_options.title = p.title.clone();
                }
            }

            Message::OpenProject => {
                let projects_dir = crate::core::home_dir_or_cwd().join(PROJECTS_DIR_NAME);
                return IcedTask::perform(
                    async move {
                        let mut dialog = rfd::AsyncFileDialog::new().set_title("Open project");
                        if projects_dir.is_dir() {
                            dialog = dialog.set_directory(projects_dir);
                        }
                        let folder = dialog.pick_folder().await?;
                        Some(load_project(folder.path().to_path_buf()).await)
                    },
                    |loaded| Message::ProjectLoaded(Box::new(loaded)),
                );
            }

            Message::SaveProject => {
                self.sync_editor_to_project();
                match &self.project {
                    Some(project) if project.path.is_none() => {
                        let projects_dir = crate::core::home_dir_or_cwd().join(PROJECTS_DIR_NAME);
                        if let Err(e) = std::fs::create_dir_all(&projects_dir) {
                            log::warn!("Failed to create {}: {}", projects_dir.display(), e);
                        }
                        let id = project.id;
                        return IcedTask::perform(
                            pick_save_path("Save project", Some(projects_dir), project.default_dir_name()),
                            move |dir| Message::SaveProjectAs(id, dir),
                        );
                    }
                    Some(_) => self.save_project(None),
                    None => {}
                }
            }

            Message::SaveProjectAs(id, Some(project_dir)) => {
                // Only the same project, still without a folder, takes the chosen one.
                if self.project.as_ref().is_some_and(|p| p.id == id && p.path.is_none()) {
                    self.sync_editor_to_project();
                    self.save_project(Some(&project_dir));
                }
            }

            Message::SaveProjectAs(_, None) => {}

            Message::ProjectLoaded(loaded) => match *loaded {
                Some(Ok(p)) => {
                    self.compile_options.title = p.title.clone();
                    self.project_notes_text = p.project_notes.clone();
                    self.compile_presets = p.compile_presets.clone();
                    self.generated_names.clear();
                    // Initialize word count milestone to current project word count
                    let total_words = p.binder.total_word_count();
                    let milestones = [1000, 5000, 10000, 25000, 50000, 75000, 100000, 150000, 200000];
                    self.last_milestone = milestones
                        .iter()
                        .rev()
                        .find(|&&m| total_words >= m)
                        .copied()
                        .unwrap_or(0);
                    // Initialize project subsystems (search index, file watcher)
                    self.project_state = ProjectState::new();
                    self.project_state.on_project_load(&p.binder, p.path.as_deref());
                    if let Some(path) = &p.path {
                        self.recent_projects.add(&p.title, path.clone());
                        self.recent_projects.save();
                    }
                    let title = p.title.clone();
                    self.project = Some(p);
                    self.saves_since_backup = 0;
                    self.selected_item = None;
                    self.editor = EditorState::new();
                    self.linguistic_result = None;
                    self.notification = Some(format!("Opened project: {}", title));
                }
                Some(Err(e)) => {
                    self.notification = Some(format!("Open error: {}", e));
                }
                // The open dialog was cancelled.
                None => {}
            },

            // ========== Binder operations ==========
            Message::SelectBinderItem(id) => {
                // Save current editor content before switching
                self.sync_editor_to_project();

                // Save notes for current item
                if let (Some(ref mut project), Some(prev_id)) = (&mut self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item_mut(&prev_id) {
                        if let Some(ref mut doc) = item.document {
                            doc.notes = self.notes_text.clone();
                        }
                    }
                }

                self.selected_item = Some(id);

                // Load the selected document into the editor
                if let Some(ref project) = self.project {
                    if let Some(item) = project.binder.find_item(&id) {
                        if let Some(ref doc) = item.document {
                            self.editor.load_document(doc);
                            self.notes_text = doc.notes.clone();
                        } else {
                            self.notes_text.clear();
                        }
                    }
                }
            }

            Message::ToggleBinderItem(id) => {
                if let Some(ref mut project) = self.project {
                    if let Some(item) = project.binder.find_item_mut(&id) {
                        item.expanded = !item.expanded;
                    }
                }
                // Also select the item when toggling a folder
                self.selected_item = Some(id);
            }

            Message::NewDocument => {
                if let Some(ref mut project) = self.project {
                    let new_item = BinderItem::new_text("New Document");
                    let new_id = new_item.id;

                    if let Some(sel_id) = self.selected_item {
                        if let Some(parent) = project.binder.find_item_mut(&sel_id) {
                            if parent.kind == BinderItemKind::Folder {
                                parent.add_child(new_item);
                            } else {
                                project.binder.draft.add_child(new_item);
                            }
                        } else {
                            project.binder.draft.add_child(new_item);
                        }
                    } else {
                        project.binder.draft.add_child(new_item);
                    }

                    self.selected_item = Some(new_id);
                    if let Some(item) = project.binder.find_item(&new_id) {
                        if let Some(ref doc) = item.document {
                            self.editor.load_document(doc);
                            self.notes_text = doc.notes.clone();
                        }
                    }
                }
            }

            Message::NewFolder => {
                if let Some(ref mut project) = self.project {
                    let new_folder = BinderItem::new_folder("New Folder");
                    let new_id = new_folder.id;

                    if let Some(sel_id) = self.selected_item {
                        if let Some(parent) = project.binder.find_item_mut(&sel_id) {
                            if parent.kind == BinderItemKind::Folder {
                                parent.add_child(new_folder);
                            } else {
                                project.binder.draft.add_child(new_folder);
                            }
                        } else {
                            project.binder.draft.add_child(new_folder);
                        }
                    } else {
                        project.binder.draft.add_child(new_folder);
                    }

                    self.selected_item = Some(new_id);
                }
            }

            Message::DeleteItem(id) => {
                if let Some(ref mut project) = self.project {
                    project.binder.move_to_trash(&id);
                    if self.selected_item == Some(id) {
                        self.selected_item = None;
                        self.editor = EditorState::new();
                        self.notes_text.clear();
                    }
                }
            }

            Message::RenameItem(id, new_name) => {
                if let Some(ref mut project) = self.project {
                    if let Some(item) = project.binder.find_item_mut(&id) {
                        item.title = new_name;
                    }
                }
            }

            Message::UpdateSynopsis(id, new_synopsis) => {
                if let Some(ref mut project) = self.project {
                    if let Some(item) = project.binder.find_item_mut(&id) {
                        item.synopsis = new_synopsis;
                    }
                }
            }

            Message::MoveItem {
                item_id,
                target_id,
                position,
            } => {
                if let Some(ref mut project) = self.project {
                    if let Some(item) = project
                        .binder
                        .draft
                        .remove_child(&item_id)
                        .or_else(|| project.binder.research.remove_child(&item_id))
                    {
                        if let Some(target) = project.binder.find_item_mut(&target_id) {
                            target.insert_child(position, item);
                        }
                    }
                }
            }

            Message::MoveItemUp(id) => {
                if let Some(ref mut project) = self.project {
                    project.binder.move_item_up(&id);
                }
            }

            Message::MoveItemDown(id) => {
                if let Some(ref mut project) = self.project {
                    project.binder.move_item_down(&id);
                }
            }

            Message::DuplicateItem(id) => {
                if let Some(ref mut project) = self.project {
                    if let Some(new_id) = project.binder.duplicate_item(&id) {
                        self.selected_item = Some(new_id);
                        self.notification = Some("Item duplicated".to_string());
                    }
                }
            }

            Message::EmptyTrash => {
                if let Some(ref mut project) = self.project {
                    project.binder.empty_trash();
                    self.notification = Some("Trash emptied".to_string());
                }
            }

            Message::ConvertToFolder(id) => {
                if let Some(ref mut project) = self.project {
                    if project.binder.convert_to_folder(&id) {
                        self.notification = Some("Converted to folder".to_string());
                    }
                }
            }

            Message::ConvertToText(id) => {
                if let Some(ref mut project) = self.project {
                    if project.binder.convert_to_text(&id) {
                        self.notification = Some("Converted to text".to_string());
                    }
                }
            }

            Message::SplitDocument => {
                self.sync_editor_to_project();
                if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                    let split_content = {
                        project
                            .binder
                            .find_item(&item_id)
                            .and_then(|item| item.document.as_ref())
                            .map(|doc| doc.content.clone())
                    };

                    if let Some(content) = split_content {
                        // Find a char-boundary-safe midpoint
                        let byte_mid = content.len() / 2;
                        let mid = content.ceil_char_boundary(byte_mid);
                        let split_pos = content[mid..].find("\n\n").map_or(mid, |p| p + mid);

                        if split_pos > 0 && split_pos < content.len() {
                            let first_half = content[..split_pos].to_string();
                            let second_half = content[split_pos..].trim_start().to_string();

                            if let Some(item) = project.binder.find_item_mut(&item_id) {
                                if let Some(ref mut doc) = item.document {
                                    doc.content = first_half;
                                }
                            }

                            let mut new_item = BinderItem::new_text("Split Document");
                            if let Some(ref mut doc) = new_item.document {
                                doc.content = second_half;
                            }
                            project.binder.draft.add_child(new_item);

                            if let Some(item) = project.binder.find_item(&item_id) {
                                if let Some(ref doc) = item.document {
                                    self.editor.load_document(doc);
                                }
                            }

                            self.notification = Some("Document split".to_string());
                        }
                    }
                }
            }

            Message::MergeIntoParent => {
                if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                    let merged = {
                        project
                            .binder
                            .find_item(&item_id)
                            .filter(|item| item.kind == BinderItemKind::Folder && !item.children.is_empty())
                            .map(|item| (item.title.clone(), item.merge_children_content()))
                    };

                    if let Some((title, merged_content)) = merged {
                        if !merged_content.is_empty() {
                            let mut new_item = BinderItem::new_text(&format!("{} (Merged)", title));
                            if let Some(ref mut doc) = new_item.document {
                                doc.content = merged_content;
                            }
                            let new_id = new_item.id;
                            project.binder.draft.add_child(new_item);
                            self.selected_item = Some(new_id);

                            if let Some(item) = project.binder.find_item(&new_id) {
                                if let Some(ref doc) = item.document {
                                    self.editor.load_document(doc);
                                    self.notes_text = doc.notes.clone();
                                }
                            }

                            self.notification = Some("Children merged into new document".to_string());
                        }
                    }
                }
            }

            // ========== Editor operations ==========
            Message::EditorAction(action) => {
                let is_edit = action.is_edit();
                let old_len = self.editor.content.text().len();
                let cursor_before = self.editor.cursor;
                self.editor.content.perform(action);
                if is_edit {
                    self.editor.mark_dirty();
                    let new_text = self.editor.content.text().to_string();
                    let new_len = new_text.len();
                    let delta = new_len as i64 - old_len as i64;

                    if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                        if let Some(item) = project.binder.find_item_mut(&item_id) {
                            // Shift annotation positions when text is edited
                            if delta != 0 {
                                if let Some(ref mut doc) = item.document {
                                    for ann in &mut doc.annotations {
                                        if ann.start >= cursor_before {
                                            ann.shift(delta);
                                        }
                                    }
                                }
                            }

                            // Shift comment anchors for this document
                            if delta != 0 {
                                let mgr = self.project_state.comments_for(item_id);
                                mgr.shift_after(cursor_before, delta);
                            }

                            // Record revision marks when revision mode is active
                            if delta > 0 {
                                self.project_state.record_insertion(
                                    cursor_before,
                                    cursor_before + delta as usize,
                                    &new_text[cursor_before..cursor_before + delta as usize],
                                );
                            } else if delta < 0 {
                                self.project_state.record_deletion(
                                    cursor_before,
                                    cursor_before + (-delta) as usize,
                                    "",
                                );
                            }

                            // Update search index for this document
                            let title = item.title.clone();
                            let content = item.document.as_ref().map_or("", |d| d.content.as_str()).to_string();
                            let notes = item.document.as_ref().map_or("", |d| d.notes.as_str()).to_string();
                            let synopsis = item.synopsis.clone();
                            self.project_state
                                .on_document_edit(item_id, &title, &content, &notes, &synopsis);
                        }
                    }
                }
            }

            // ========== View operations ==========
            Message::SwitchView(ref mode) => {
                // Set up corkboard layout when switching to corkboard view
                if *mode == ViewMode::Corkboard {
                    if let Some(ref project) = self.project {
                        let parent = self
                            .selected_item
                            .and_then(|id| project.binder.find_item(&id))
                            .unwrap_or(&project.binder.draft);
                        let item_ids: Vec<Uuid> = parent.children.iter().map(|c| c.id).collect();
                        self.project_state.arrange_corkboard_grid(&item_ids);
                    }
                }
                self.view_mode = mode.clone();
            }

            Message::ToggleInspector => {
                self.show_inspector = !self.show_inspector;
            }

            Message::ToggleFullscreen => {
                self.fullscreen_editor = !self.fullscreen_editor;
            }

            Message::ShowBottomPanel(panel) => {
                if self.bottom_panel == panel {
                    self.bottom_panel = BottomPanel::None;
                } else {
                    self.bottom_panel = panel;
                }
            }

            // ========== Compile operations ==========
            Message::ShowCompileDialog => {
                self.show_compile_dialog = true;
            }

            Message::HideCompileDialog => {
                self.show_compile_dialog = false;
            }

            Message::CompileSetFormat(format_name) => {
                self.compile_options.format = match format_name.as_str() {
                    "Plain Text" => OutputFormat::PlainText,
                    "Markdown" => OutputFormat::Markdown,
                    "HTML" => OutputFormat::Html,
                    "PDF" => OutputFormat::Pdf,
                    "LaTeX" => OutputFormat::Latex,
                    "Word (DOCX)" => OutputFormat::Docx,
                    "ePub" => OutputFormat::Epub,
                    "RTF" => OutputFormat::Rtf,
                    "OPML" => OutputFormat::Opml,
                    "Fountain" => OutputFormat::Fountain,
                    _ => OutputFormat::Markdown,
                };
            }

            Message::CompileSetTitle(title) => {
                self.compile_options.title = title;
            }

            Message::CompileSetAuthor(author) => {
                self.compile_options.author = author;
            }

            Message::CompileSetFrontMatter(val) => {
                self.compile_options.include_front_matter = val;
            }

            Message::CompileSetMarkedOnly(val) => {
                self.compile_options.compile_marked_only = val;
            }

            Message::CompileSetPageBreaks(val) => {
                self.compile_options.page_break_between_folders = val;
            }

            Message::CompileSetFontFamily(family) => {
                self.compile_options.font_family = family;
            }

            Message::CompileSetFontSize(size_str) => {
                if let Ok(size) = size_str.parse::<f32>() {
                    if size > 0.0 && size <= 72.0 {
                        self.compile_options.font_size = size;
                    }
                }
            }

            Message::CompileSetSeparator(sep_name) => {
                self.compile_options.separator = match sep_name.as_str() {
                    "Empty Line" => SeparatorType::EmptyLine,
                    "Page Break" => SeparatorType::PageBreak,
                    "Section Break" => SeparatorType::SectionBreak,
                    "None" => SeparatorType::None,
                    _ => SeparatorType::EmptyLine,
                };
            }

            Message::DoCompile => {
                if let Some(id) = self.project.as_ref().map(|p| p.id) {
                    let file_name = format!(
                        "{}.{}",
                        crate::core::project::dir_name_for_title(&self.compile_options.title),
                        self.compile_options.format.extension()
                    );
                    return IcedTask::perform(
                        pick_save_path("Compile to", self.last_export_dir.clone(), file_name),
                        move |path| Message::CompileTo(id, path),
                    );
                }
            }

            Message::CompileTo(id, Some(output_path)) if self.is_open_project(id) => {
                self.sync_editor_to_project();
                if let Some(ref project) = self.project {
                    use crate::export::compiler::Compiler;
                    use crate::export::integrations;

                    // Generate compile stats for logging
                    let compile_result = integrations::compile_with_stats(&project.binder, &self.compile_options);
                    log::info!(
                        "Compile: {} words, {} sections, {} validation issues",
                        compile_result.statistics.total_words,
                        compile_result.manifest.sections.len(),
                        compile_result.validation_issues.len(),
                    );

                    self.last_export_dir = output_path.parent().map(Path::to_path_buf);
                    match Compiler::save_to_file(&project.binder, &self.compile_options, &output_path) {
                        Ok(_) => {
                            self.notification = Some(format!("Compiled to {}", output_path.display()));
                            self.show_compile_dialog = false;
                        }
                        Err(e) => {
                            self.notification = Some(format!("Compile error: {}", e));
                        }
                    }
                }
            }

            Message::CompileTo(..) => {}

            // ========== Snapshot operations ==========
            Message::CreateSnapshot => {
                self.sync_editor_to_project();
                if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                    match project.create_snapshot(&item_id, "Manual Snapshot") {
                        Ok(_) => self.notification = Some("Snapshot created".to_string()),
                        Err(e) => self.notification = Some(format!("Snapshot failed: {}", e)),
                    }
                }
            }

            Message::RestoreSnapshot(index) => {
                if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item_mut(&item_id) {
                        if let Some(snapshot) = item.snapshots.get(index) {
                            if let Some(ref mut doc) = item.document {
                                doc.content = snapshot.content.clone();
                                self.editor.load_document(doc);
                            }
                        }
                    }
                }
                self.selected_snapshot = None;
            }

            Message::SelectSnapshot(index) => {
                self.selected_snapshot = if self.selected_snapshot == Some(index) {
                    None
                } else {
                    Some(index)
                };
            }

            Message::CompareSnapshot(index) => {
                self.selected_snapshot = Some(index);
            }

            // ========== Compile options (new) ==========
            Message::CompileSetToc(val) => {
                self.compile_options.include_toc = val;
            }

            Message::CompileSetPlaceholders(val) => {
                self.compile_options.replace_placeholders = val;
            }

            // ========== Search operations ==========
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
            }

            Message::DoSearch => {
                if let Some(ref project) = self.project {
                    // Record search in history
                    if !self.search_query.is_empty() {
                        self.project_state.record_search(&self.search_query);
                    }
                    let options = SearchOptions {
                        query: self.search_query.clone(),
                        case_sensitive: self.search_case_sensitive,
                        whole_word: self.search_whole_word,
                        regex: self.search_regex,
                        search_titles: true,
                        search_content: true,
                        search_notes: true,
                        search_synopsis: true,
                        ..Default::default()
                    };
                    self.search_results = search::search_binder(&project.binder, &options);
                }
            }

            Message::SearchToggleCaseSensitive => {
                self.search_case_sensitive = !self.search_case_sensitive;
            }

            Message::SearchToggleWholeWord => {
                self.search_whole_word = !self.search_whole_word;
            }

            Message::SearchToggleRegex => {
                self.search_regex = !self.search_regex;
            }

            Message::ReplaceTextChanged(text) => {
                self.replace_text = text;
            }

            Message::DoReplaceAll => {
                self.sync_editor_to_project();
                if let Some(ref mut project) = self.project {
                    let options = FindReplaceOptions {
                        query: self.search_query.clone(),
                        replacement: self.replace_text.clone(),
                        case_sensitive: self.search_case_sensitive,
                        whole_word: self.search_whole_word,
                        use_regex: self.search_regex,
                        ..Default::default()
                    };

                    let mut count = 0;
                    project.binder.for_each_item_mut(|item| {
                        if let Some(ref mut doc) = item.document {
                            let (new_content, replacements) = find_replace::replace_in_text(&doc.content, &options);
                            if replacements > 0 {
                                count += 1;
                                doc.content = new_content;
                            }
                        }
                    });

                    // Reload current editor
                    if let Some(item_id) = self.selected_item {
                        if let Some(item) = project.binder.find_item(&item_id) {
                            if let Some(ref doc) = item.document {
                                self.editor.load_document(doc);
                            }
                        }
                    }

                    self.notification = Some(format!("Replaced in {} document(s)", count));
                }
            }

            Message::GoToSearchResult(item_id) => {
                self.selected_item = Some(item_id);
                if let Some(ref project) = self.project {
                    if let Some(item) = project.binder.find_item(&item_id) {
                        if let Some(ref doc) = item.document {
                            self.editor.load_document(doc);
                            self.notes_text = doc.notes.clone();
                        }
                    }
                }
            }

            // ========== Thesaurus operations ==========
            Message::ThesaurusQueryChanged(query) => {
                self.thesaurus_query = query;
            }

            Message::DoThesaurusLookup => {
                self.thesaurus_results = self.thesaurus.lookup(&self.thesaurus_query).to_vec();
            }

            Message::InsertSynonym(word) => {
                self.editor.content.perform(iced::widget::text_editor::Action::Edit(
                    iced::widget::text_editor::Edit::Paste(std::sync::Arc::new(word)),
                ));
                self.editor.mark_dirty();
            }

            // ========== Document notes ==========
            Message::NotesChanged(notes) => {
                self.notes_text = notes;
                if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item_mut(&item_id) {
                        if let Some(ref mut doc) = item.document {
                            doc.notes = self.notes_text.clone();
                        }
                    }
                }
            }

            // ========== Target word count ==========
            Message::SetItemTarget(id, target_str) => {
                if let Ok(target) = target_str.parse::<usize>() {
                    self.item_targets.insert(id, target);
                    // Also set in the proper targets system
                    self.project_state.set_target(id, target);
                } else if target_str.is_empty() {
                    self.item_targets.remove(&id);
                    self.project_state.set_target(id, 0);
                }
            }

            // ========== Metadata ==========
            Message::SetItemStatus(id, status_name) => {
                if let Some(ref mut project) = self.project {
                    if let Some(item) = project.binder.find_item_mut(&id) {
                        if status_name == "None" {
                            item.metadata.status = None;
                        } else {
                            item.metadata.status = Some(crate::core::metadata::Status::new(&status_name));
                        }
                    }
                }
            }

            Message::SetItemLabel(id, label_name) => {
                if let Some(ref mut project) = self.project {
                    if let Some(item) = project.binder.find_item_mut(&id) {
                        if label_name == "None" {
                            item.metadata.label = None;
                        } else {
                            let label = project.settings.labels.iter().find(|l| l.name == label_name).cloned();
                            item.metadata.label = label;
                        }
                    }
                }
            }

            Message::ToggleIncludeInCompile(id) => {
                if let Some(ref mut project) = self.project {
                    if let Some(item) = project.binder.find_item_mut(&id) {
                        item.include_in_compile = !item.include_in_compile;
                    }
                }
            }

            // ========== Settings dialog ==========
            Message::ShowSettings | Message::OpenSettingsWindow => {
                if let Some(id) = self.settings_window {
                    // Already open — focus it
                    return window::gain_focus(id);
                }
                let (id, open_task) = window::open(window::Settings {
                    size: iced::Size::new(780.0, 680.0),
                    ..window::Settings::default()
                });
                self.settings_window = Some(id);
                return open_task.map(Message::WindowOpened);
            }

            Message::HideSettings | Message::CloseSettingsWindow => {
                // Persist settings by saving the project
                self.save_settings();
                if let Some(id) = self.settings_window.take() {
                    return window::close(id);
                }
            }

            Message::OpenAboutWindow => {
                if let Some(id) = self.about_window {
                    // Already open — focus it
                    return window::gain_focus(id);
                }
                let (id, open_task) = window::open(window::Settings {
                    size: iced::Size::new(420.0, 380.0),
                    resizable: false,
                    ..window::Settings::default()
                });
                self.about_window = Some(id);
                return open_task.map(Message::WindowOpened);
            }

            Message::CloseAboutWindow => {
                if let Some(id) = self.about_window.take() {
                    return window::close(id);
                }
            }

            Message::WindowOpened(_id) => {
                // Window created — nothing extra to do
            }

            Message::WindowClosed(id) => {
                // Clean up state when a window is closed externally (e.g. X button)
                if Some(id) == self.settings_window {
                    self.settings_window = None;
                    // Save settings on close
                    self.save_settings();
                } else if Some(id) == self.about_window {
                    self.about_window = None;
                } else if id == self.main_window {
                    // Main window closed — exit the app
                    // Close secondary windows first
                    let mut tasks = Vec::new();
                    if let Some(sw) = self.settings_window.take() {
                        tasks.push(window::close(sw));
                    }
                    if let Some(aw) = self.about_window.take() {
                        tasks.push(window::close(aw));
                    }
                    tasks.push(iced::exit());
                    return IcedTask::batch(tasks);
                }
            }

            Message::MainWindowClosed => {
                return iced::exit();
            }

            Message::SettingsSetProjectTitle(title) => {
                if let Some(ref mut project) = self.project {
                    project.title = title;
                }
            }

            Message::SettingsSetFont(font) => {
                if let Some(ref mut project) = self.project {
                    project.settings.editor_font = font;
                }
            }

            Message::SettingsSetFontSize(size_str) => {
                if let Ok(size) = size_str.parse::<f32>() {
                    if size > 0.0 && size <= 72.0 {
                        if let Some(ref mut project) = self.project {
                            project.settings.editor_font_size = size;
                        }
                    }
                }
            }

            Message::SettingsZoomIn => {
                if let Some(ref mut project) = self.project {
                    project.settings.editor_zoom = (project.settings.editor_zoom + 0.1).min(3.0);
                }
            }

            Message::SettingsZoomOut => {
                if let Some(ref mut project) = self.project {
                    project.settings.editor_zoom = (project.settings.editor_zoom - 0.1).max(0.5);
                }
            }

            Message::SettingsSetTarget(target_str) => {
                if let Some(ref mut project) = self.project {
                    if let Ok(target) = target_str.parse::<usize>() {
                        project.settings.target_word_count = Some(target);
                    } else if target_str.is_empty() {
                        project.settings.target_word_count = None;
                    }
                }
            }

            Message::SettingsSetAutoSave(interval_str) => {
                if let Some(ref mut project) = self.project {
                    if let Ok(secs) = interval_str.parse::<u32>() {
                        project.settings.auto_save_seconds = secs;
                    }
                }
            }

            Message::SettingsToggleWordCount(val) => {
                if let Some(ref mut project) = self.project {
                    project.settings.show_word_count = val;
                }
            }

            Message::SettingsChangeTab(tab) => {
                self.settings_active_tab = tab;
            }

            Message::SettingsSetLineSpacing(val) => {
                if let Ok(spacing) = val.parse::<f32>() {
                    if (0.5..=4.0).contains(&spacing) {
                        if let Some(ref mut project) = self.project {
                            project.settings.line_spacing = spacing;
                        }
                    }
                }
            }

            Message::SettingsSetLineSpacingPreset(preset) => {
                let spacing = match preset.as_str() {
                    "Single" => 1.0,
                    "1.15" => 1.15,
                    "1.5" => 1.5,
                    "Double" => 2.0,
                    _ => 1.5,
                };
                if let Some(ref mut project) = self.project {
                    project.settings.line_spacing = spacing;
                }
            }

            Message::SettingsSetEditorWidth(val) => {
                if let Ok(w) = val.parse::<f32>() {
                    if (30.0..=100.0).contains(&w) {
                        if let Some(ref mut project) = self.project {
                            project.settings.editor_width = w;
                        }
                    }
                }
            }

            Message::SettingsToggleSpellCheck(val) => {
                if let Some(ref mut project) = self.project {
                    project.settings.spell_check_enabled = val;
                }
            }

            Message::SettingsToggleTypewriterScroll(val) => {
                if let Some(ref mut project) = self.project {
                    project.settings.typewriter_scroll = val;
                }
            }

            Message::SettingsToggleShowParagraphMarks(val) => {
                if let Some(ref mut project) = self.project {
                    project.settings.show_paragraph_marks = val;
                }
            }

            Message::SettingsToggleHighContrast(val) => {
                if let Some(ref mut project) = self.project {
                    project.settings.high_contrast = val;
                }
            }

            Message::SettingsToggleLargeUI(val) => {
                if let Some(ref mut project) = self.project {
                    project.settings.large_ui = val;
                }
            }

            Message::SettingsToggleReduceMotion(val) => {
                if let Some(ref mut project) = self.project {
                    project.settings.reduce_motion = val;
                }
            }

            Message::SettingsToggleScreenReaderHints(val) => {
                if let Some(ref mut project) = self.project {
                    project.settings.screen_reader_hints = val;
                }
            }

            Message::SettingsSetUIScale(val) => {
                if let Ok(scale) = val.parse::<f32>() {
                    let clamped = scale.clamp(0.5, 3.0);
                    if let Some(ref mut project) = self.project {
                        project.settings.ui_scale = clamped;
                    }
                    if (clamped - scale).abs() > f32::EPSILON {
                        log::warn!("UI scale {:.2} clamped to {:.2}", scale, clamped);
                    }
                }
            }

            Message::SettingsToggleAutoBackup(val) => {
                if let Some(ref mut project) = self.project {
                    project.settings.auto_backup = val;
                }
            }

            Message::SettingsSetBackupInterval(val) => {
                if let Ok(n) = val.parse::<u32>() {
                    if n > 0 && n <= 100 {
                        if let Some(ref mut project) = self.project {
                            project.settings.backup_interval_saves = n;
                        }
                    }
                }
            }

            Message::SettingsToggleSmartPunctuation(val) => {
                if let Some(ref mut project) = self.project {
                    project.settings.smart_punctuation = val;
                }
            }

            Message::SettingsSetDefaultDocType(val) => {
                if let Some(ref mut project) = self.project {
                    project.settings.default_doc_type = val;
                }
            }

            Message::SettingsToggleShowSynopsis(val) => {
                if let Some(ref mut project) = self.project {
                    project.settings.show_synopsis_in_binder = val;
                }
            }

            Message::SettingsToggleAutoNumbering(val) => {
                if let Some(ref mut project) = self.project {
                    project.settings.auto_numbering = val;
                }
            }

            // ========== Writing session ==========
            Message::SessionToggle => {
                if self.session_active {
                    // Stopping session - show summary
                    self.session_active = false;
                    let elapsed_min = self.session_stats.time_elapsed_seconds as f64 / 60.0;
                    let words = self.session_stats.words_written;
                    let wpm = self.session_stats.words_per_minute;
                    let goal_msg = if self.session_goal > 0 {
                        let pct = (words as f64 / self.session_goal as f64 * 100.0).min(999.9);
                        if pct >= 100.0 {
                            " | Goal reached!".to_string()
                        } else {
                            format!(" | {:.0}% of goal", pct)
                        }
                    } else {
                        String::new()
                    };
                    self.notification = Some(format!(
                        "Session ended: {} words in {:.1}m ({:.1} wpm){}",
                        words, elapsed_min, wpm, goal_msg
                    ));
                    // Record to writing history
                    let current_wc = self.current_word_count();
                    let elapsed = self.session_stats.time_elapsed_seconds;
                    if let Some(ref mut project) = self.project {
                        project.writing_history.record(current_wc, elapsed);
                    }
                } else {
                    // Starting session
                    self.session_active = true;
                    self.session_start_word_count = self.current_word_count();
                    self.session_stats = SessionStats::new();
                    self.notification = Some("Writing session started. Happy writing!".to_string());
                }
            }

            Message::SessionReset => {
                let had_words = self.session_stats.words_written > 0;
                self.session_active = false;
                self.session_stats = SessionStats::new();
                self.session_start_word_count = self.current_word_count();
                if had_words {
                    self.notification = Some("Session reset. Previous session data cleared.".to_string());
                }
            }

            Message::SessionSetGoal(goal_str) => {
                self.session_goal_text = goal_str.clone();
                if let Ok(goal) = goal_str.parse::<usize>() {
                    self.session_goal = goal;
                } else if goal_str.is_empty() {
                    self.session_goal = 0;
                }
            }

            // ========== Import ==========
            Message::ImportFiles => match self.project.as_ref().map(|p| p.id) {
                None => self.notification = Some("Create or open a project first.".to_string()),
                Some(id) => {
                    return IcedTask::perform(
                        pick_import_paths(self.last_import_dir.clone(), "Supported files", IMPORT_EXTENSIONS),
                        move |paths| Message::ImportPaths(id, paths),
                    );
                }
            },

            Message::ImportPaths(id, paths) => {
                if let Some(dir) = paths.first().and_then(|p| p.parent()) {
                    self.last_import_dir = Some(dir.to_path_buf());
                }
                if !paths.is_empty() && self.is_open_project(id) {
                    self.import_files(paths);
                }
            }

            // ========== Name generator ==========
            Message::GenerateName(kind) => {
                use crate::core::namegen::NameGenerator;
                self.name_gen_type = kind.clone();
                let name = NameGenerator::generate_one(&kind);
                self.generated_names.push(name);
            }

            Message::GenerateNameBatch => {
                use crate::core::namegen::NameGenerator;
                let names = NameGenerator::generate_batch(&self.name_gen_type, 5);
                self.generated_names.extend(names);
            }

            // ========== Project notes ==========
            Message::ProjectNotesChanged(notes) => {
                self.project_notes_text = notes.clone();
                if let Some(ref mut project) = self.project {
                    project.project_notes = notes;
                }
            }

            // ========== Collections ==========
            Message::CollectionNameInput(name) => {
                self.new_collection_name = name;
            }

            Message::CreateCollection => {
                if let Some(ref mut project) = self.project {
                    if !self.new_collection_name.is_empty() {
                        let coll = crate::core::collection::Collection::new_manual(&self.new_collection_name);
                        project.collections.push(coll);
                        self.new_collection_name.clear();
                        self.notification = Some("Collection created".to_string());
                    }
                }
            }

            Message::DeleteCollection(coll_id) => {
                if let Some(ref mut project) = self.project {
                    project.collections.retain(|c| c.id != coll_id);
                    if self.selected_collection == Some(coll_id) {
                        self.selected_collection = None;
                    }
                }
            }

            Message::SelectCollection(coll_id) => {
                self.selected_collection = Some(coll_id);
            }

            Message::AddToCollection(coll_id) => {
                if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                    if let Some(coll) = project.collections.iter_mut().find(|c| c.id == coll_id) {
                        coll.add_item(item_id);
                        self.notification = Some(format!("Added to collection '{}'", coll.name));
                    }
                }
            }

            // ========== Bookmarks ==========
            Message::ToggleBookmark(item_id) => {
                if let Some(ref mut project) = self.project {
                    let name = project
                        .binder
                        .find_item(&item_id)
                        .map_or_else(String::new, |i| i.title.clone());
                    project.bookmarks.toggle(item_id, &name);
                }
            }

            // ========== Annotations ==========
            Message::AnnotationTextInput(text) => {
                self.annotation_text = text;
            }

            Message::AddAnnotation => {
                if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                    if !self.annotation_text.is_empty() {
                        if let Some(item) = project.binder.find_item_mut(&item_id) {
                            if let Some(ref mut doc) = item.document {
                                // Use selection range if available, otherwise use cursor position
                                let (start, end) = self
                                    .editor
                                    .selection_range()
                                    .unwrap_or((self.editor.cursor, self.editor.cursor));
                                let mut ann = crate::core::annotation::Annotation::with_color(
                                    start,
                                    end,
                                    &self.annotation_text,
                                    self.annotation_next_color.clone(),
                                );
                                // Set author from compile settings if available
                                if !self.compile_options.author.is_empty() {
                                    ann = ann.with_author(&self.compile_options.author);
                                }
                                doc.annotations.push(ann);
                            }
                        }
                        self.annotation_text.clear();
                        self.notification = Some("Annotation added".to_string());
                    }
                }
            }

            Message::DeleteAnnotation(ann_id) => {
                if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item_mut(&item_id) {
                        if let Some(ref mut doc) = item.document {
                            doc.annotations.retain(|a| a.id != ann_id);
                        }
                    }
                }
                self.notification = Some("Annotation deleted".to_string());
            }

            Message::ToggleAnnotationResolved(ann_id) => {
                if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item_mut(&item_id) {
                        if let Some(ref mut doc) = item.document {
                            if let Some(ann) = doc.annotations.iter_mut().find(|a| a.id == ann_id) {
                                ann.toggle_resolved();
                            }
                        }
                    }
                }
                self.notification = Some("Annotation toggled".to_string());
            }

            Message::EditAnnotation(ann_id, new_text) => {
                if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item_mut(&item_id) {
                        if let Some(ref mut doc) = item.document {
                            if let Some(ann) = doc.annotations.iter_mut().find(|a| a.id == ann_id) {
                                if !new_text.is_empty() {
                                    ann.edit_text(&new_text);
                                    self.notification = Some("Annotation updated".to_string());
                                }
                            }
                        }
                    }
                }
            }

            Message::SetAnnotationColor(ann_id, color_name) => {
                if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item_mut(&item_id) {
                        if let Some(ref mut doc) = item.document {
                            if let Some(ann) = doc.annotations.iter_mut().find(|a| a.id == ann_id) {
                                ann.color = match color_name.as_str() {
                                    "Blue" => crate::core::annotation::AnnotationColor::Blue,
                                    "Green" => crate::core::annotation::AnnotationColor::Green,
                                    "Red" => crate::core::annotation::AnnotationColor::Red,
                                    "Purple" => crate::core::annotation::AnnotationColor::Purple,
                                    _ => crate::core::annotation::AnnotationColor::Yellow,
                                };
                            }
                        }
                    }
                }
            }

            Message::SetAnnotationCategory(ann_id, category) => {
                if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item_mut(&item_id) {
                        if let Some(ref mut doc) = item.document {
                            if let Some(ann) = doc.annotations.iter_mut().find(|a| a.id == ann_id) {
                                ann.set_category(&category);
                            }
                        }
                    }
                }
            }

            Message::CycleAnnotationColor => {
                self.annotation_next_color = self.annotation_next_color.next();
            }

            // ========== Project targets ==========
            Message::SettingsSetDeadline(deadline) => {
                if let Some(ref mut project) = self.project {
                    if deadline.is_empty() {
                        project.settings.target_deadline = None;
                    } else {
                        project.settings.target_deadline = Some(deadline);
                    }
                }
            }

            // ========== Text transforms ==========
            Message::TextToUppercase => {
                let has_selection = self.editor.has_selection();
                if has_selection {
                    // Transform selection only
                    self.editor.to_uppercase(true);
                } else {
                    // Transform whole document
                    self.sync_editor_to_project();
                    if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                        if let Some(item) = project.binder.find_item_mut(&item_id) {
                            if let Some(ref mut doc) = item.document {
                                doc.content = doc.content.to_uppercase();
                                self.editor.load_document(doc);
                                self.editor.mark_dirty();
                            }
                        }
                    }
                }
            }

            Message::TextToLowercase => {
                let has_selection = self.editor.has_selection();
                if has_selection {
                    self.editor.to_lowercase(true);
                } else {
                    self.sync_editor_to_project();
                    if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                        if let Some(item) = project.binder.find_item_mut(&item_id) {
                            if let Some(ref mut doc) = item.document {
                                doc.content = doc.content.to_lowercase();
                                self.editor.load_document(doc);
                                self.editor.mark_dirty();
                            }
                        }
                    }
                }
            }

            Message::TextToTitleCase => {
                let has_selection = self.editor.has_selection();
                if has_selection {
                    self.editor.to_title_case(true);
                } else {
                    self.sync_editor_to_project();
                    if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                        if let Some(item) = project.binder.find_item_mut(&item_id) {
                            if let Some(ref mut doc) = item.document {
                                doc.content = helpers::title_case(&doc.content);
                                self.editor.load_document(doc);
                                self.editor.mark_dirty();
                            }
                        }
                    }
                }
            }

            // ========== Undo/Redo ==========
            Message::Undo => {
                if self.editor.undo() {
                    self.sync_editor_to_project();
                }
            }

            Message::Redo => {
                if self.editor.redo() {
                    self.sync_editor_to_project();
                }
            }

            // ========== Recent projects ==========
            Message::OpenRecentProject(path) => {
                return IcedTask::perform(load_project(path), |loaded| {
                    Message::ProjectLoaded(Box::new(Some(loaded)))
                });
            }

            // ========== Keywords ==========
            Message::SetItemKeywords(id, keywords_str) => {
                if let Some(ref mut project) = self.project {
                    if let Some(item) = project.binder.find_item_mut(&id) {
                        item.metadata.keywords = keywords_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                }
            }

            // ========== Split editor ==========
            Message::OpenInSplitEditor(item_id) => {
                self.split_editor_item = Some(item_id);
            }

            Message::CloseSplitEditor => {
                self.split_editor_item = None;
            }

            // ========== Document find/replace ==========
            Message::DocFindChanged(query) => {
                self.doc_find_text = query;
                self.doc_find_positions.clear();
                self.doc_find_current_match = 0;
                // Find all match positions in current document using find_replace module
                if let (Some(ref project), Some(item_id)) = (&self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item(&item_id) {
                        if let Some(ref doc) = item.document {
                            if !self.doc_find_text.is_empty() {
                                let options = FindReplaceOptions {
                                    query: self.doc_find_text.clone(),
                                    replacement: self.doc_replace_text.clone(),
                                    case_sensitive: self.doc_find_case_sensitive,
                                    whole_word: self.doc_find_whole_word,
                                    use_regex: self.doc_find_use_regex,
                                    ..Default::default()
                                };
                                let matches = find_replace::find_in_text(&doc.content, &options);
                                self.doc_find_positions = matches.iter().map(|m| m.start).collect();
                                self.doc_find_match_count = self.doc_find_positions.len();
                            } else {
                                self.doc_find_match_count = 0;
                            }
                        }
                    }
                }
            }

            Message::DocFindNext => {
                if !self.doc_find_positions.is_empty() {
                    self.doc_find_current_match = (self.doc_find_current_match + 1) % self.doc_find_positions.len();
                    let pos = self.doc_find_positions[self.doc_find_current_match];
                    self.notification = Some(format!(
                        "Match {}/{} at position {}",
                        self.doc_find_current_match + 1,
                        self.doc_find_positions.len(),
                        pos
                    ));
                } else {
                    self.notification = Some("No matches found".to_string());
                }
            }

            Message::DocFindPrev => {
                if !self.doc_find_positions.is_empty() {
                    if self.doc_find_current_match == 0 {
                        self.doc_find_current_match = self.doc_find_positions.len() - 1;
                    } else {
                        self.doc_find_current_match -= 1;
                    }
                    let pos = self.doc_find_positions[self.doc_find_current_match];
                    self.notification = Some(format!(
                        "Match {}/{} at position {}",
                        self.doc_find_current_match + 1,
                        self.doc_find_positions.len(),
                        pos
                    ));
                } else {
                    self.notification = Some("No matches found".to_string());
                }
            }

            Message::DocReplaceCurrent => {
                if !self.doc_find_positions.is_empty() && self.doc_find_current_match < self.doc_find_positions.len() {
                    self.sync_editor_to_project();
                    if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                        if let Some(item) = project.binder.find_item_mut(&item_id) {
                            if let Some(ref mut doc) = item.document {
                                let pos = self.doc_find_positions[self.doc_find_current_match];
                                let options = FindReplaceOptions {
                                    query: self.doc_find_text.clone(),
                                    replacement: self.doc_replace_text.clone(),
                                    case_sensitive: self.doc_find_case_sensitive,
                                    whole_word: self.doc_find_whole_word,
                                    use_regex: self.doc_find_use_regex,
                                    ..Default::default()
                                };
                                if let Some((new_text, _loc)) = find_replace::replace_next(&doc.content, &options, pos)
                                {
                                    doc.content = new_text;
                                    self.editor.load_document(doc);
                                    self.editor.mark_dirty();
                                    // Re-find all matches in the updated content
                                    let matches = find_replace::find_in_text(&doc.content, &options);
                                    self.doc_find_positions = matches.iter().map(|m| m.start).collect();
                                    self.doc_find_match_count = self.doc_find_positions.len();
                                    if self.doc_find_current_match >= self.doc_find_positions.len()
                                        && !self.doc_find_positions.is_empty()
                                    {
                                        self.doc_find_current_match = 0;
                                    }
                                    self.notification =
                                        Some(format!("Replaced match. {} remaining", self.doc_find_positions.len()));
                                }
                            }
                        }
                    }
                }
            }

            Message::DocReplaceAll => {
                self.sync_editor_to_project();
                if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item_mut(&item_id) {
                        if let Some(ref mut doc) = item.document {
                            if !self.doc_find_text.is_empty() {
                                let options = FindReplaceOptions {
                                    query: self.doc_find_text.clone(),
                                    replacement: self.doc_replace_text.clone(),
                                    case_sensitive: self.doc_find_case_sensitive,
                                    whole_word: self.doc_find_whole_word,
                                    use_regex: self.doc_find_use_regex,
                                    ..Default::default()
                                };
                                let (new_text, count) = find_replace::replace_in_text(&doc.content, &options);
                                doc.content = new_text;
                                self.editor.load_document(doc);
                                self.editor.mark_dirty();
                                self.doc_find_positions.clear();
                                self.doc_find_match_count = 0;
                                self.notification = Some(format!("Replaced {} occurrence(s)", count));
                            }
                        }
                    }
                }
            }

            Message::DocReplaceChanged(text) => {
                self.doc_replace_text = text;
            }

            Message::DocFindToggleCase => {
                self.doc_find_case_sensitive = !self.doc_find_case_sensitive;
                // Recount matches using find_replace module
                if !self.doc_find_text.is_empty() {
                    if let (Some(ref project), Some(item_id)) = (&self.project, self.selected_item) {
                        if let Some(item) = project.binder.find_item(&item_id) {
                            if let Some(ref doc) = item.document {
                                let options = FindReplaceOptions {
                                    query: self.doc_find_text.clone(),
                                    replacement: String::new(),
                                    case_sensitive: self.doc_find_case_sensitive,
                                    whole_word: self.doc_find_whole_word,
                                    use_regex: self.doc_find_use_regex,
                                    ..Default::default()
                                };
                                self.doc_find_match_count = find_replace::count_matches(&doc.content, &options);
                            }
                        }
                    }
                }
            }

            Message::DocFindToggleWholeWord => {
                self.doc_find_whole_word = !self.doc_find_whole_word;
            }

            Message::DocFindToggleRegex => {
                self.doc_find_use_regex = !self.doc_find_use_regex;
            }

            // ========== Quick reference ==========
            Message::ShowQuickRef(item_id) => {
                self.quick_ref_item = Some(item_id);
                self.bottom_panel = BottomPanel::QuickRef;
            }

            // ========== Script mode ==========
            Message::ToggleScriptMode => {
                self.script_mode = !self.script_mode;
                if self.script_mode {
                    self.current_script_element = Some(crate::core::script::ScriptElement::Action);
                } else {
                    self.current_script_element = None;
                }
                self.notification = Some(if self.script_mode {
                    "Script mode enabled".to_string()
                } else {
                    "Script mode disabled".to_string()
                });
            }

            Message::SetScriptElement(element_name) => {
                self.current_script_element = match element_name.as_str() {
                    "Scene Heading" => Some(crate::core::script::ScriptElement::SceneHeading),
                    "Action" => Some(crate::core::script::ScriptElement::Action),
                    "Character" => Some(crate::core::script::ScriptElement::Character),
                    "Dialogue" => Some(crate::core::script::ScriptElement::Dialogue),
                    "Parenthetical" => Some(crate::core::script::ScriptElement::Parenthetical),
                    "Transition" => Some(crate::core::script::ScriptElement::Transition),
                    "Shot" => Some(crate::core::script::ScriptElement::Shot),
                    "Note" => Some(crate::core::script::ScriptElement::Note),
                    _ => None,
                };
            }

            // ========== Auto-correction ==========
            Message::ToggleAutoCorrectSmartQuotes => {
                self.auto_correction.smart_quotes = !self.auto_correction.smart_quotes;
            }

            Message::ToggleAutoCorrectEmDashes => {
                self.auto_correction.em_dashes = !self.auto_correction.em_dashes;
            }

            Message::ToggleAutoCorrectEllipsis => {
                self.auto_correction.ellipsis = !self.auto_correction.ellipsis;
            }

            // ========== Document links ==========
            Message::InsertDocLink(target_id) => {
                if let Some(ref project) = self.project {
                    if let Some(target_item) = project.binder.find_item(&target_id) {
                        let link_text = format!("[[{}]]", target_item.title);
                        self.editor.content.perform(iced::widget::text_editor::Action::Edit(
                            iced::widget::text_editor::Edit::Paste(std::sync::Arc::new(link_text)),
                        ));
                        self.editor.mark_dirty();
                        self.notification = Some(format!("Linked to '{}'", target_item.title));
                    }
                }
            }

            // ========== Formatting toolbar ==========
            Message::InsertBold => {
                self.insert_markdown_wrap("**");
            }

            Message::InsertItalic => {
                self.insert_markdown_wrap("*");
            }

            Message::InsertUnderline => {
                self.insert_markdown_wrap("__");
            }

            Message::InsertStrikethrough => {
                self.insert_markdown_wrap("~~");
            }

            Message::InsertHeading(level) => {
                let prefix = "#".repeat(level as usize);
                let markup = format!("{} ", prefix);
                self.editor.content.perform(iced::widget::text_editor::Action::Edit(
                    iced::widget::text_editor::Edit::Paste(std::sync::Arc::new(markup)),
                ));
                self.editor.mark_dirty();
            }

            Message::InsertBlockQuote => {
                self.editor.content.perform(iced::widget::text_editor::Action::Edit(
                    iced::widget::text_editor::Edit::Paste(std::sync::Arc::new("> ".to_string())),
                ));
                self.editor.mark_dirty();
            }

            Message::InsertFootnote => {
                self.footnote_counter += 1;
                let marker = format!("[^{}]", self.footnote_counter);
                self.editor.content.perform(iced::widget::text_editor::Action::Edit(
                    iced::widget::text_editor::Edit::Paste(std::sync::Arc::new(marker.clone())),
                ));
                self.editor.mark_dirty();
                // Add footnote to document
                if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item_mut(&item_id) {
                        if let Some(ref mut doc) = item.document {
                            doc.footnotes.push(crate::core::document::Footnote {
                                marker: self.footnote_counter,
                                text: String::new(),
                                is_endnote: false,
                            });
                        }
                    }
                }
                self.notification = Some(format!("Footnote {} inserted", self.footnote_counter));
            }

            Message::InsertHRule => {
                self.editor.content.perform(iced::widget::text_editor::Action::Edit(
                    iced::widget::text_editor::Edit::Paste(std::sync::Arc::new("\n---\n".to_string())),
                ));
                self.editor.mark_dirty();
            }

            // ========== Compile presets ==========
            Message::SaveCompilePreset(name) => {
                // Remove existing preset with same name
                self.compile_presets.retain(|(n, _)| n != &name);
                self.compile_presets.push((name.clone(), self.compile_options.clone()));
                // Persist to project
                if let Some(ref mut project) = self.project {
                    project.compile_presets = self.compile_presets.clone();
                }
                self.notification = Some(format!("Compile preset '{}' saved", name));
            }

            Message::LoadCompilePreset(name) => {
                if let Some((_, preset)) = self.compile_presets.iter().find(|(n, _)| n == &name) {
                    self.compile_options = preset.clone();
                    self.notification = Some(format!("Loaded preset '{}'", name));
                }
            }

            // ========== Project Statistics ==========
            Message::ShowProjectStats => {
                self.show_project_stats = true;
            }

            Message::HideProjectStats => {
                self.show_project_stats = false;
            }

            // ========== Writing Goals ==========
            Message::SetDailyGoal(val) => {
                self.daily_goal_text = val.clone();
                if let Ok(goal) = val.parse::<usize>() {
                    self.daily_goal = goal;
                } else if val.is_empty() {
                    self.daily_goal = 0;
                }
            }

            Message::SetWeeklyGoal(val) => {
                self.weekly_goal_text = val.clone();
                if let Ok(goal) = val.parse::<usize>() {
                    self.weekly_goal = goal;
                } else if val.is_empty() {
                    self.weekly_goal = 0;
                }
            }

            Message::ResetGoals => {
                self.daily_goal = 0;
                self.daily_goal_text.clear();
                self.weekly_goal = 0;
                self.weekly_goal_text.clear();
                self.notification = Some("Goals reset".to_string());
            }

            // ========== Composition Mode ==========
            Message::ToggleCompositionMode => {
                self.composition_mode = !self.composition_mode;
                self.notification = Some(if self.composition_mode {
                    "Composition mode enabled".to_string()
                } else {
                    "Composition mode disabled".to_string()
                });
            }

            // ========== Copy special ==========
            Message::CopyAsMarkdown => {
                self.sync_editor_to_project();
                if let (Some(ref project), Some(item_id)) = (&self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item(&item_id) {
                        if let Some(ref doc) = item.document {
                            let md = format!("# {}\n\n{}", item.title, doc.content);
                            match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(md)) {
                                Ok(_) => self.notification = Some("Copied as Markdown".to_string()),
                                Err(e) => self.notification = Some(format!("Clipboard error: {}", e)),
                            }
                        }
                    }
                }
            }

            Message::CopyAsHtml => {
                self.sync_editor_to_project();
                if let (Some(ref project), Some(item_id)) = (&self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item(&item_id) {
                        if let Some(ref doc) = item.document {
                            let html = format!(
                                "<html><body><h1>{}</h1>\n{}</body></html>",
                                item.title,
                                doc.content
                                    .split("\n\n")
                                    .map(|p| format!("<p>{}</p>", p.replace('\n', "<br>")))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            );
                            match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(html)) {
                                Ok(_) => self.notification = Some("Copied as HTML".to_string()),
                                Err(e) => self.notification = Some(format!("Clipboard error: {}", e)),
                            }
                        }
                    }
                }
            }

            Message::CopyAsPlainText => {
                self.sync_editor_to_project();
                if let (Some(ref project), Some(item_id)) = (&self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item(&item_id) {
                        if let Some(ref doc) = item.document {
                            match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(doc.content.clone())) {
                                Ok(_) => self.notification = Some("Copied as plain text".to_string()),
                                Err(e) => self.notification = Some(format!("Clipboard error: {}", e)),
                            }
                        }
                    }
                }
            }

            // ========== Search collection ==========
            Message::SaveSearchAsCollection => {
                if let Some(ref mut project) = self.project {
                    if !self.search_results.is_empty() {
                        let mut coll = crate::core::collection::Collection::new_search(
                            &format!("Search: \"{}\"", self.search_query),
                            &self.search_query,
                        );
                        for result in &self.search_results {
                            coll.add_item(result.item_id);
                        }
                        project.collections.push(coll);
                        self.notification = Some(format!(
                            "Saved {} search results as collection",
                            self.search_results.len()
                        ));
                    }
                }
            }

            // ========== Backup ==========
            Message::CreateBackup => {
                if let Some(ref project) = self.project {
                    if let Some(ref path) = project.path {
                        match BackupManager::default_root().and_then(|root| BackupManager::create_backup(path, &root)) {
                            Ok(backup_path) => {
                                self.notification = Some(format!(
                                    "Backup created: {:?}",
                                    backup_path.file_name().unwrap_or_default()
                                ));
                            }
                            Err(e) => {
                                self.notification = Some(format!("Backup error: {:#}", e));
                            }
                        }
                    } else {
                        self.notification = Some("Save the project first before creating a backup.".to_string());
                    }
                }
            }

            Message::RestoreBackup(path) => {
                // The backup is unpacked into a new folder next to the project,
                // which is then opened; the current project folder stays as is.
                if let Some(proj_path) = self.project.as_ref().and_then(|p| p.path.clone()) {
                    match BackupManager::restore_backup(&path, &proj_path) {
                        Ok(restored) => {
                            return IcedTask::perform(load_project(restored), |loaded| {
                                Message::ProjectLoaded(Box::new(Some(loaded)))
                            });
                        }
                        Err(e) => {
                            self.notification = Some(format!("Restore error: {:#}", e));
                        }
                    }
                }
            }

            // ========== OPML Import ==========
            Message::ImportOpml => {
                if let Some(id) = self.project.as_ref().map(|p| p.id) {
                    return IcedTask::perform(
                        pick_import_paths(self.last_import_dir.clone(), "OPML outlines", &["opml"]),
                        move |paths| Message::ImportPaths(id, paths),
                    );
                }
            }

            // ========== Smart Collection ==========
            Message::CreateSmartCollection(query) => {
                if let Some(ref mut project) = self.project {
                    if !query.is_empty() {
                        let mut coll = crate::core::collection::Collection::new_search("Smart: Search", &query);
                        // Auto-populate with matching items
                        let options = crate::core::search::SearchOptions {
                            query: query.clone(),
                            case_sensitive: false,
                            whole_word: false,
                            regex: false,
                            search_titles: true,
                            search_content: true,
                            search_notes: false,
                            search_synopsis: true,
                            ..Default::default()
                        };
                        let results = crate::core::search::search_binder(&project.binder, &options);
                        for result in &results {
                            coll.add_item(result.item_id);
                        }
                        coll.name = format!("Smart: \"{}\" ({} items)", query, results.len());
                        project.collections.push(coll);
                        self.notification = Some(format!("Smart collection created with {} items", results.len()));
                    }
                }
            }

            // ========== Export OPML ==========
            Message::ExportOpml => {
                if let Some(ref project) = self.project {
                    let file_name = format!("{}.opml", crate::core::project::dir_name_for_title(&project.title));
                    let id = project.id;
                    return IcedTask::perform(
                        pick_save_path("Export OPML", self.last_export_dir.clone(), file_name),
                        move |path| Message::ExportOpmlTo(id, path),
                    );
                }
            }

            Message::ExportOpmlTo(id, Some(output_path)) if self.is_open_project(id) => {
                self.sync_editor_to_project();
                if let Some(ref project) = self.project {
                    self.last_export_dir = output_path.parent().map(Path::to_path_buf);
                    match crate::export::opml::export_opml(&project.binder, &project.title) {
                        Ok(opml_content) => match std::fs::write(&output_path, opml_content) {
                            Ok(_) => {
                                self.notification = Some(format!("OPML exported to {}", output_path.display()));
                            }
                            Err(e) => {
                                self.notification = Some(format!("Export error: {}", e));
                            }
                        },
                        Err(e) => {
                            self.notification = Some(format!("OPML error: {}", e));
                        }
                    }
                }
            }

            Message::ExportOpmlTo(..) => {}

            // ========== Print ==========
            Message::PrintCurrent => {
                self.sync_editor_to_project();
                if let (Some(ref project), Some(item_id)) = (&self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item(&item_id) {
                        if let Some(ref doc) = item.document {
                            use crate::export::compiler::{CompileContent, CompileOptions};

                            let contents = vec![CompileContent {
                                title: item.title.clone(),
                                text: doc.content.clone(),
                                is_folder: false,
                                depth: 0,
                            }];

                            let opts = CompileOptions {
                                title: item.title.clone(),
                                ..CompileOptions::default()
                            };
                            let home = crate::core::home_dir_or_cwd();
                            let print_path = home.join(PROJECTS_DIR_NAME).join("print.pdf");
                            if let Some(parent) = print_path.parent() {
                                if let Err(e) = std::fs::create_dir_all(parent) {
                                    log::warn!("Failed to create directory {:?}: {}", parent, e);
                                }
                            }
                            match crate::export::pdf::save_pdf(&contents, &opts, &print_path) {
                                Ok(_) => {
                                    self.notification = Some(format!("PDF saved to {:?} — open to print", print_path));
                                }
                                Err(e) => {
                                    self.notification = Some(format!("Print error: {}", e));
                                }
                            }
                        }
                    }
                }
            }

            Message::PrintProject => {
                self.sync_editor_to_project();
                if let Some(ref project) = self.project {
                    let mut opts = self.compile_options.clone();
                    opts.format = crate::export::compiler::OutputFormat::Pdf;
                    let home = crate::core::home_dir_or_cwd();
                    let print_path = home.join(PROJECTS_DIR_NAME).join(format!(
                        "{}_print.pdf",
                        crate::core::project::dir_name_for_title(&project.title)
                    ));
                    if let Some(parent) = print_path.parent() {
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            log::warn!("Failed to create directory {:?}: {}", parent, e);
                        }
                    }
                    match crate::export::compiler::Compiler::save_to_file(&project.binder, &opts, &print_path) {
                        Ok(_) => {
                            self.notification = Some(format!("Project PDF saved to {:?}", print_path));
                        }
                        Err(e) => {
                            self.notification = Some(format!("Print error: {}", e));
                        }
                    }
                }
            }

            // ========== Spell check ==========
            Message::RunSpellCheck => {
                self.sync_editor_to_project();
                if let (Some(ref project), Some(item_id)) = (&self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item(&item_id) {
                        if let Some(ref doc) = item.document {
                            self.spell_check_results = self.spell_checker.check_text(&doc.content);
                            let count = self.spell_check_results.len();
                            if count == 0 {
                                self.notification = Some("No spelling errors found!".to_string());
                            } else {
                                self.notification = Some(format!("Found {} potential spelling issue(s)", count));
                            }
                            self.bottom_panel = BottomPanel::SpellCheck;
                        }
                    }
                }
            }

            Message::SpellCheckAddWord(word) => {
                self.spell_checker.add_to_dictionary(&word);
                self.spell_check_results
                    .retain(|r| r.word.to_lowercase() != word.to_lowercase());
                if let Err(e) = self.spell_checker.save_user_dictionary() {
                    self.notification = Some(format!("Added \"{}\" but failed to save dictionary: {}", word, e));
                } else {
                    self.notification = Some(format!("Added \"{}\" to dictionary", word));
                }
            }

            Message::SpellCheckReplace(position, misspelled, replacement) => {
                self.sync_editor_to_project();
                if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item_mut(&item_id) {
                        if let Some(ref mut doc) = item.document {
                            // Find the misspelled word at or near the given position and replace it
                            let safe_pos = if position <= doc.content.len() && doc.content.is_char_boundary(position) {
                                position
                            } else {
                                // Snap to nearest valid char boundary
                                doc.content.ceil_char_boundary(position.min(doc.content.len()))
                            };
                            if safe_pos < doc.content.len() {
                                if let Some(start) = doc.content[safe_pos..].find(&misspelled) {
                                    let actual_pos = safe_pos + start;
                                    let end_pos = actual_pos + misspelled.len();
                                    doc.content = format!(
                                        "{}{}{}",
                                        &doc.content[..actual_pos],
                                        replacement,
                                        &doc.content[end_pos..]
                                    );
                                    // Reload editor with updated content
                                    self.editor.load_document(doc);
                                    self.notification =
                                        Some(format!("Replaced \"{}\" with \"{}\"", misspelled, replacement));
                                    // Remove this entry from results
                                    self.spell_check_results
                                        .retain(|r| !(r.word == misspelled && r.position == position));
                                }
                            } // safe_pos < doc.content.len()
                        }
                    }
                }
            }

            Message::SpellCheckRemoveWord(word) => {
                self.spell_checker.remove_from_dictionary(&word);
                if let Err(e) = self.spell_checker.save_user_dictionary() {
                    self.notification = Some(format!("Removed \"{}\" but failed to save: {}", word, e));
                } else {
                    self.notification = Some(format!("Removed \"{}\" from user dictionary", word));
                }
            }

            Message::SpellCheckClearDict => {
                self.spell_checker.clear_user_dictionary();
                if let Err(e) = self.spell_checker.save_user_dictionary() {
                    self.notification = Some(format!("Dictionary cleared but failed to save: {}", e));
                } else {
                    self.notification = Some("User dictionary cleared".to_string());
                }
            }

            Message::ToggleSpellChecker => {
                if self.spell_checker.active {
                    self.spell_checker.active = false;
                    self.spell_check_results.clear();
                    self.notification = Some("Spell checker disabled".to_string());
                } else {
                    self.spell_checker.try_init();
                    if self.spell_checker.active {
                        self.notification = Some(format!(
                            "Spell checker enabled ({} words loaded)",
                            self.spell_checker.dictionary_size()
                        ));
                    } else {
                        self.notification = Some("Failed to initialize spell checker".to_string());
                    }
                }
            }

            // ========== Custom metadata fields ==========
            Message::AddCustomField(id, field_name) => {
                if let Some(ref mut project) = self.project {
                    if let Some(item) = project.binder.find_item_mut(&id) {
                        if !field_name.is_empty() && !item.metadata.custom_metadata.iter().any(|f| f.name == field_name)
                        {
                            item.metadata.custom_metadata.push(crate::core::metadata::CustomField {
                                name: field_name.clone(),
                                value: crate::core::metadata::CustomFieldValue::Text(String::new()),
                            });
                            self.notification = Some(format!("Added field '{}'", field_name));
                        }
                    }
                }
            }

            Message::UpdateCustomField(id, field_name, value) => {
                if let Some(ref mut project) = self.project {
                    if let Some(item) = project.binder.find_item_mut(&id) {
                        if let Some(field) = item.metadata.custom_metadata.iter_mut().find(|f| f.name == field_name) {
                            field.value = crate::core::metadata::CustomFieldValue::Text(value);
                        }
                    }
                }
            }

            Message::RemoveCustomField(id, field_name) => {
                if let Some(ref mut project) = self.project {
                    if let Some(item) = project.binder.find_item_mut(&id) {
                        item.metadata.custom_metadata.retain(|f| f.name != field_name);
                        self.notification = Some(format!("Removed field '{}'", field_name));
                    }
                }
            }

            // ========== Smart collection refresh ==========
            Message::RefreshSmartCollections => {
                if let Some(ref mut project) = self.project {
                    let mut updates = Vec::new();
                    for (i, coll) in project.collections.iter().enumerate() {
                        if let crate::core::collection::CollectionKind::Search {
                            ref query,
                            case_sensitive,
                            whole_word,
                        } = coll.kind
                        {
                            let options = crate::core::search::SearchOptions {
                                query: query.clone(),
                                case_sensitive,
                                whole_word,
                                regex: false,
                                search_titles: true,
                                search_content: true,
                                search_notes: false,
                                search_synopsis: true,
                                ..Default::default()
                            };
                            let results = crate::core::search::search_binder(&project.binder, &options);
                            let item_ids: Vec<Uuid> = results.iter().map(|r| r.item_id).collect();
                            updates.push((i, item_ids, results.len()));
                        }
                    }
                    let mut total_refreshed = 0;
                    for (idx, item_ids, count) in updates {
                        if let Some(coll) = project.collections.get_mut(idx) {
                            coll.item_ids = item_ids;
                            if let crate::core::collection::CollectionKind::Search { ref query, .. } = coll.kind {
                                coll.name = format!("Smart: \"{}\" ({} items)", query, count);
                            }
                            total_refreshed += 1;
                        }
                    }
                    if total_refreshed > 0 {
                        self.notification = Some(format!("Refreshed {} smart collection(s)", total_refreshed));
                    }
                }
            }

            // ========== Composition mode settings ==========
            Message::SettingsSetCompWidth(val) => {
                if let Ok(w) = val.parse::<f32>() {
                    if (10.0..=100.0).contains(&w) {
                        if let Some(ref mut project) = self.project {
                            project.settings.fullscreen_text_width = w;
                        }
                    }
                }
            }

            // ========== Editor text operations ==========
            Message::TransposeChars => {
                self.editor.push_undo();
                self.editor.transpose_chars();
                self.sync_editor_to_project();
            }

            Message::SortLines => {
                self.editor.push_undo();
                self.editor.sort_lines();
                self.sync_editor_to_project();
            }

            Message::RemoveDuplicateLines => {
                self.editor.push_undo();
                self.editor.remove_duplicate_lines();
                self.sync_editor_to_project();
            }

            Message::JoinLines => {
                self.editor.push_undo();
                self.editor.join_lines();
                self.sync_editor_to_project();
            }

            Message::MoveLineUp => {
                self.editor.push_undo();
                self.editor.move_line_up();
                self.sync_editor_to_project();
            }

            Message::MoveLineDown => {
                self.editor.push_undo();
                self.editor.move_line_down();
                self.sync_editor_to_project();
            }

            Message::DeleteLine => {
                self.editor.push_undo();
                self.editor.delete_line();
                self.sync_editor_to_project();
            }

            Message::DuplicateLine => {
                self.editor.push_undo();
                self.editor.duplicate_line();
                self.sync_editor_to_project();
            }

            Message::IndentLine => {
                self.editor.push_undo();
                self.editor.indent_line();
                self.sync_editor_to_project();
            }

            Message::UnindentLine => {
                self.editor.push_undo();
                self.editor.unindent_line();
                self.sync_editor_to_project();
            }

            Message::ToggleComment => {
                self.editor.push_undo();
                self.editor.toggle_comment();
                self.sync_editor_to_project();
            }

            // ========== Insert operations ==========
            Message::InsertListItem(style) => {
                let prefix = match style.as_str() {
                    "numbered" => "1. ",
                    "checkbox" => "- [ ] ",
                    _ => "- ",
                };
                self.editor.content.perform(iced::widget::text_editor::Action::Edit(
                    iced::widget::text_editor::Edit::Paste(std::sync::Arc::new(format!("\n{}", prefix))),
                ));
                self.editor.mark_dirty();
            }

            Message::InsertTable(rows, cols) => {
                let mut table = String::new();
                // Header row
                table.push('|');
                for c in 0..cols {
                    let _ = write!(table, " Column {} |", c + 1);
                }
                table.push('\n');
                // Separator
                table.push('|');
                for _ in 0..cols {
                    table.push_str("----------|");
                }
                table.push('\n');
                // Data rows
                for _ in 0..rows {
                    table.push('|');
                    for _ in 0..cols {
                        table.push_str("          |");
                    }
                    table.push('\n');
                }
                self.editor.content.perform(iced::widget::text_editor::Action::Edit(
                    iced::widget::text_editor::Edit::Paste(std::sync::Arc::new(table)),
                ));
                self.editor.mark_dirty();
            }

            Message::InsertCodeBlock(lang) => {
                let block = if lang.is_empty() {
                    "\n```\n\n```\n".to_string()
                } else {
                    format!("\n```{}\n\n```\n", lang)
                };
                self.editor.content.perform(iced::widget::text_editor::Action::Edit(
                    iced::widget::text_editor::Edit::Paste(std::sync::Arc::new(block)),
                ));
                self.editor.mark_dirty();
            }

            Message::InsertPageBreak => {
                self.editor.content.perform(iced::widget::text_editor::Action::Edit(
                    iced::widget::text_editor::Edit::Paste(std::sync::Arc::new(
                        "\n\n---\n\n<!-- page break -->\n\n".to_string(),
                    )),
                ));
                self.editor.mark_dirty();
            }

            Message::InsertComment => {
                self.editor.content.perform(iced::widget::text_editor::Action::Edit(
                    iced::widget::text_editor::Edit::Paste(std::sync::Arc::new("<!-- comment -->".to_string())),
                ));
                self.editor.mark_dirty();
            }

            Message::InsertDateTime(format) => {
                use crate::editor::actions::DateTimeFormat;
                let fmt = match format.as_str() {
                    "time" => DateTimeFormat::TimeOnly,
                    "datetime" => DateTimeFormat::DateTime,
                    "iso" => DateTimeFormat::Iso8601,
                    _ => DateTimeFormat::DateOnly,
                };
                let text = fmt.format_now();
                self.editor.content.perform(iced::widget::text_editor::Action::Edit(
                    iced::widget::text_editor::Edit::Paste(std::sync::Arc::new(text)),
                ));
                self.editor.mark_dirty();
            }

            Message::InsertLink => {
                let link_text = "[link text](url)";
                self.editor.content.perform(iced::widget::text_editor::Action::Edit(
                    iced::widget::text_editor::Edit::Paste(std::sync::Arc::new(link_text.to_string())),
                ));
                self.editor.mark_dirty();
            }

            Message::InsertImage => {
                let image_text = "![alt text](image_path)";
                self.editor.content.perform(iced::widget::text_editor::Action::Edit(
                    iced::widget::text_editor::Edit::Paste(std::sync::Arc::new(image_text.to_string())),
                ));
                self.editor.mark_dirty();
            }

            // ========== Document templates ==========
            Message::NewDocFromTemplate(template_id) => {
                if let Some(ref mut project) = self.project {
                    if let Some(template) = crate::core::doc_templates::find_template(&template_id) {
                        let item = template.create_item(template.name);
                        let new_id = item.id;

                        if let Some(sel_id) = self.selected_item {
                            if let Some(parent) = project.binder.find_item_mut(&sel_id) {
                                if parent.kind == BinderItemKind::Folder {
                                    parent.add_child(item);
                                } else {
                                    project.binder.draft.add_child(item);
                                }
                            } else {
                                project.binder.draft.add_child(item);
                            }
                        } else {
                            project.binder.draft.add_child(item);
                        }

                        self.selected_item = Some(new_id);
                        if let Some(new_item) = project.binder.find_item(&new_id) {
                            if let Some(ref doc) = new_item.document {
                                self.editor.load_document(doc);
                                self.notes_text = doc.notes.clone();
                            }
                        }
                        self.notification = Some(format!("Created '{}' from template", template.name));
                    }
                }
            }

            // ========== Writing timer ==========
            Message::TimerStart => {
                let word_count = self.current_word_count();
                self.writing_timer.start(word_count);
                self.notification = Some(format!("Timer started: {}", self.writing_timer.preset.label()));
            }

            Message::TimerPause => {
                self.writing_timer.pause();
            }

            Message::TimerResume => {
                self.writing_timer.resume();
            }

            Message::TimerStop => {
                let word_count = self.current_word_count();
                self.writing_timer.stop(word_count);
                self.notification = Some(format!("Timer stopped. {}", self.writing_timer.summary()));
            }

            Message::TimerReset => {
                self.writing_timer.reset();
            }

            Message::TimerSetPreset(preset_name) => {
                use crate::core::timer::TimerPreset;
                let preset = match preset_name.as_str() {
                    "sprint" => TimerPreset::Sprint,
                    "long" => TimerPreset::LongSession,
                    "hour" => TimerPreset::HourSession,
                    _ => TimerPreset::Pomodoro,
                };
                self.writing_timer.set_preset(preset);
            }

            // ========== Project validation ==========
            Message::ShowValidation => {
                self.sync_editor_to_project();
                if let Some(ref project) = self.project {
                    // Use ProjectState for comprehensive validation (includes link health)
                    let result = ProjectState::validate_project(&project.binder);
                    self.notification = Some(result.display());
                    self.validation_result = Some(result);
                    self.bottom_panel = BottomPanel::Validation;
                }
            }

            // ========== Writing prompts ==========
            Message::GenerateWritingPrompt => {
                self.writing_prompts_data.generate_prompt(None);
                self.bottom_panel = BottomPanel::WritingPrompts;
            }
            Message::GenerateWritingPromptCategory(cat_name) => {
                use crate::core::writing_prompts::PromptCategory;
                let category = PromptCategory::all().iter().find(|c| c.label() == cat_name).copied();
                if let Some(cat) = category {
                    self.writing_prompts_data.generate_prompt(Some(cat));
                } else {
                    self.writing_prompts_data.generate_prompt(None);
                }
                self.bottom_panel = BottomPanel::WritingPrompts;
            }
            Message::GenerateCharacter => {
                self.writing_prompts_data.generate_character();
                self.bottom_panel = BottomPanel::WritingPrompts;
            }
            Message::GeneratePlotSeed => {
                self.writing_prompts_data.generate_plot();
                self.bottom_panel = BottomPanel::WritingPrompts;
            }
            Message::GenerateWritingNames => {
                self.writing_prompts_data.generate_names();
                self.bottom_panel = BottomPanel::WritingPrompts;
            }

            // ========== Outliner interactions ==========
            Message::OutlinerToggleExpand(item_id) => {
                self.project_state.outliner.toggle_expand(item_id);
            }

            // ========== Validation auto-fix ==========
            Message::AutoFixValidation => {
                if let Some(ref mut project) = self.project {
                    let result = ProjectState::auto_fix(&mut project.binder);
                    self.notification = Some(result.summary());
                    // Re-run validation to update the panel
                    let validation = ProjectState::validate_project(&project.binder);
                    self.validation_result = Some(validation);
                }
            }

            // ========== Linguistic analysis ==========
            // ========== Misc ==========
            Message::Tick => {
                self.handle_tick();
            }

            Message::DismissNotification => {
                self.notification = None;
                self.notification_timer = 0;
            }

            Message::ToggleToolbarMenu(menu) => {
                if self.active_toolbar_menu.as_ref() == Some(&menu) {
                    self.active_toolbar_menu = None;
                } else {
                    self.active_toolbar_menu = Some(menu);
                }
            }

            Message::CloseToolbarMenu => {
                self.active_toolbar_menu = None;
            }

            Message::EscapePressed => {
                if self.active_toolbar_menu.is_some() {
                    self.active_toolbar_menu = None;
                } else if self.composition_mode {
                    self.composition_mode = false;
                } else if self.fullscreen_editor {
                    self.fullscreen_editor = false;
                } else if self.show_project_stats {
                    self.show_project_stats = false;
                } else if self.show_compile_dialog {
                    self.show_compile_dialog = false;
                } else if self.bottom_panel != BottomPanel::None {
                    self.bottom_panel = BottomPanel::None;
                } else if self.notification.is_some() {
                    self.notification = None;
                }
            }
        }

        IcedTask::none()
    }

    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        // Route to Settings window
        if Some(window_id) == self.settings_window {
            if let Some(ref project) = self.project {
                return views::settings_dialog::view(
                    &project.settings,
                    &project.title,
                    self.script_mode,
                    &self.auto_correction,
                    &self.settings_active_tab,
                );
            }
            // No project open — show placeholder
            return container(
                text("Open a project first to access settings.")
                    .size(14)
                    .color(super::theme::Theme::TEXT_MUTED),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
        }

        // Route to About window
        if Some(window_id) == self.about_window {
            return views::about_dialog::view();
        }

        // === Main window ===

        // Welcome screen (no project loaded)
        let Some(project) = self.project.as_ref() else {
            return views::welcome_screen::view(&self.recent_projects);
        };

        // Compile dialog (overlay)
        if self.show_compile_dialog {
            return views::compile_dialog::view(&self.compile_options, &self.compile_presets);
        }

        // Project statistics dialog (overlay)
        if self.show_project_stats {
            let stats = Statistics::from_binder(&project.binder);
            let all_text = project.binder.all_text();
            let readability = if all_text.split_whitespace().count() >= 50 {
                Some(crate::core::stats::ReadabilityMetrics::from_text(&all_text))
            } else {
                None
            };
            let data = views::project_stats_dialog::ProjectStatsData {
                title: project.title.clone(),
                word_count: stats.word_count,
                char_count: stats.char_count,
                char_no_spaces: stats.char_count_no_spaces,
                paragraph_count: stats.paragraph_count,
                sentence_count: stats.sentence_count,
                page_count: stats.page_count,
                document_count: stats.document_count,
                folder_count: stats.folder_count,
                avg_words_per_doc: stats.average_words_per_document,
                avg_words_per_day: project.writing_history.average_words_per_day(),
                total_writing_days: project.writing_history.entries.len(),
                current_streak: project.writing_history.current_streak(),
                best_day_words: project.writing_history.best_day().map(|d| d.words_written).unwrap_or(0),
                total_time_hours: project.writing_history.total_time_seconds() as f64 / 3600.0,
                reading_time_minutes: stats.word_count as f64 / crate::core::READING_WPM,
                speaking_time_minutes: stats.word_count as f64 / crate::core::SPEAKING_WPM,
                target_words: project.settings.target_word_count,
                deadline: project.settings.target_deadline.clone(),
                days_remaining: project
                    .settings
                    .target_deadline
                    .as_ref()
                    .and_then(|d| NaiveDate::parse_from_str(d, crate::core::DATE_FORMAT).ok())
                    .map(|target_date| {
                        let today = Utc::now().date_naive();
                        (target_date - today).num_days()
                    }),
                words_per_day_needed: {
                    let days_remaining = project
                        .settings
                        .target_deadline
                        .as_ref()
                        .and_then(|d| NaiveDate::parse_from_str(d, crate::core::DATE_FORMAT).ok())
                        .map(|target_date| {
                            let today = Utc::now().date_naive();
                            (target_date - today).num_days()
                        });
                    match (project.settings.target_word_count, days_remaining) {
                        (Some(target), Some(days)) if days > 0 && target > stats.word_count => {
                            Some((target - stats.word_count) / days as usize)
                        }
                        _ => None,
                    }
                },
                // Readability metrics (from all draft content)
                flesch_score: readability.as_ref().map(|r| r.flesch_reading_ease),
                flesch_label: readability.as_ref().map(|r| r.flesch_label().to_string()),
                grade_level: readability.as_ref().map(|r| r.consensus_grade()),
                audience: readability.as_ref().map(|r| r.audience_label().to_string()),
            };
            return views::project_stats_dialog::view(&data);
        }

        // Composition mode (distraction-free writing)
        if self.composition_mode {
            let title = self
                .selected_item
                .and_then(|id| project.binder.find_item(&id))
                .map(|item| item.title.as_str())
                .unwrap_or("");
            let word_count = self.editor.document.word_count();
            let session_words = if self.session_active {
                self.session_stats.words_written
            } else {
                0
            };
            return views::editor_view::view_composition(&self.editor, title, word_count, session_words);
        }

        // Fullscreen editor mode
        if self.fullscreen_editor {
            let title = self
                .selected_item
                .and_then(|id| project.binder.find_item(&id))
                .map(|item| item.title.as_str())
                .unwrap_or("");
            return views::editor_view::view_fullscreen(&self.editor, title);
        }

        // Menu bar
        let toolbar = views::toolbar::menu_bar(&self.active_toolbar_menu);

        // Floating dropdown overlay (if a menu is open)
        let dropdown_overlay = views::toolbar::dropdown_overlay(
            &self.view_mode,
            self.show_inspector,
            self.fullscreen_editor,
            &self.bottom_panel,
            &self.active_toolbar_menu,
        );

        // Binder sidebar
        let binder = views::binder_view::view(
            &project.binder.draft,
            &project.binder.research,
            &project.binder.trash,
            self.selected_item,
        );

        // Main content area
        let main_content: Element<'_, Message> = match self.view_mode {
            ViewMode::Editor => {
                let title = self
                    .selected_item
                    .and_then(|id| project.binder.find_item(&id))
                    .map(|item| item.title.as_str())
                    .unwrap_or("No document selected");

                // Check for split editor mode
                if let Some(split_id) = self.split_editor_item {
                    let secondary_content = project
                        .binder
                        .find_item(&split_id)
                        .and_then(|item| item.document.as_ref())
                        .map(|doc| doc.content.as_str())
                        .unwrap_or("");
                    let secondary_title = project
                        .binder
                        .find_item(&split_id)
                        .map(|item| item.title.as_str())
                        .unwrap_or("Reference");
                    views::split_editor_view::view(&self.editor, title, secondary_content, secondary_title)
                } else {
                    let script_el = self.current_script_element.as_ref().map(|e| e.label());
                    views::editor_view::view(&self.editor, title, self.script_mode, script_el)
                }
            }
            ViewMode::Corkboard => {
                let (items, parent_title) = if let Some(id) = self.selected_item {
                    if let Some(item) = project.binder.find_item(&id) {
                        if item.kind == BinderItemKind::Folder {
                            let children: Vec<&BinderItem> = item.children.iter().collect();
                            (children, item.title.clone())
                        } else {
                            (vec![], "Select a folder".to_string())
                        }
                    } else {
                        (vec![], "Select a folder".to_string())
                    }
                } else {
                    let children: Vec<&BinderItem> = project.binder.draft.children.iter().collect();
                    (children, project.binder.draft.title.clone())
                };
                views::corkboard_view::view(&items, &parent_title)
            }
            ViewMode::Outliner => views::outliner_view::view(
                &project.binder.draft,
                &self.item_targets,
                &self.project_state.outliner.expanded,
            ),
            ViewMode::Scrivenings => {
                let (items, parent_title) = if let Some(id) = self.selected_item {
                    if let Some(item) = project.binder.find_item(&id) {
                        if item.kind == BinderItemKind::Folder {
                            let children: Vec<&BinderItem> =
                                item.children.iter().filter(|c| c.document.is_some()).collect();
                            (children, item.title.clone())
                        } else {
                            (vec![item], item.title.clone())
                        }
                    } else {
                        (vec![], "Select a folder".to_string())
                    }
                } else {
                    let children: Vec<&BinderItem> = project
                        .binder
                        .draft
                        .children
                        .iter()
                        .filter(|c| c.document.is_some())
                        .collect();
                    (children, project.binder.draft.title.clone())
                };
                views::scrivenings_view::view(&items, &parent_title)
            }
        };

        // Inspector (right panel)
        let inspector = if self.show_inspector {
            self.selected_item
                .and_then(|id| project.binder.find_item(&id))
                .map(|item| {
                    let is_bookmarked = project.bookmarks.is_bookmarked(&item.id);
                    let data = views::inspector_view::InspectorData::from_item(
                        item,
                        &self.notes_text,
                        self.item_targets.get(&item.id).copied(),
                        &project.settings,
                        is_bookmarked,
                    );
                    views::inspector_view::view(data)
                })
        } else {
            None
        };

        // Bottom panel
        let bottom: Option<Element<'_, Message>> = match self.bottom_panel {
            BottomPanel::Search => Some(views::search_panel::view(
                &self.search_query,
                &self.search_results,
                self.search_case_sensitive,
                self.search_whole_word,
                self.search_regex,
                &self.replace_text,
            )),
            BottomPanel::Thesaurus => Some(views::thesaurus_panel::view(
                &self.thesaurus_query,
                &self.thesaurus_results,
            )),
            BottomPanel::Snapshots => {
                let snapshots = self
                    .selected_item
                    .and_then(|id| project.binder.find_item(&id))
                    .map(|item| item.snapshots.as_slice())
                    .unwrap_or(&[]);
                let current_content = self
                    .selected_item
                    .and_then(|id| project.binder.find_item(&id))
                    .and_then(|item| item.document.as_ref())
                    .map(|doc| doc.content.as_str())
                    .unwrap_or("");
                Some(views::snapshot_panel::view(
                    snapshots,
                    current_content,
                    self.selected_snapshot,
                ))
            }
            BottomPanel::Session => {
                let session_data = views::session_panel::SessionData {
                    is_active: self.session_active,
                    elapsed_seconds: self.session_stats.time_elapsed_seconds,
                    words_written: self.session_stats.words_written,
                    words_per_minute: self.session_stats.words_per_minute,
                    session_goal: self.session_goal,
                    session_goal_text: self.session_goal_text.clone(),
                };
                Some(views::session_panel::view(&session_data))
            }
            BottomPanel::History => {
                let history = self
                    .project
                    .as_ref()
                    .map(|p| &p.writing_history)
                    .cloned()
                    .unwrap_or_default();
                Some(views::history_panel::view(&history))
            }
            BottomPanel::TextStats => {
                let text_content = self
                    .selected_item
                    .and_then(|id| {
                        self.project
                            .as_ref()
                            .and_then(|p| p.binder.find_item(&id))
                            .and_then(|item| item.document.as_ref())
                            .map(|doc| doc.content.as_str())
                    })
                    .unwrap_or("");
                let analysis = crate::core::stats::TextAnalysis::from_text(text_content);
                Some(views::text_stats_panel::view(&analysis, text_content))
            }
            BottomPanel::NameGen => Some(views::name_generator_panel::view(&self.generated_names)),
            BottomPanel::ProjectNotes => Some(views::project_notes_panel::view(&self.project_notes_text)),
            BottomPanel::Collections => {
                let data = views::collections_panel::CollectionsData {
                    collections: project.collections.clone(),
                    selected_collection: self.selected_collection,
                    new_collection_name: self.new_collection_name.clone(),
                };
                Some(views::collections_panel::view(&data))
            }
            BottomPanel::Bookmarks => Some(views::bookmarks_panel::view(&project.bookmarks)),
            BottomPanel::Annotations => {
                let annotations = self
                    .selected_item
                    .and_then(|id| project.binder.find_item(&id))
                    .and_then(|item| item.document.as_ref())
                    .map(|doc| doc.annotations.as_slice())
                    .unwrap_or(&[]);
                Some(views::annotations_panel::view(annotations, &self.annotation_text))
            }
            BottomPanel::Targets => {
                let current_words = project.binder.total_word_count();
                let deadline = project.settings.target_deadline.clone().unwrap_or_default();
                let days_remaining = project
                    .settings
                    .target_deadline
                    .as_ref()
                    .and_then(|d| NaiveDate::parse_from_str(d, crate::core::DATE_FORMAT).ok())
                    .map(|target_date| {
                        let today = Utc::now().date_naive();
                        (target_date - today).num_days()
                    });
                let words_per_day_needed = match (project.settings.target_word_count, days_remaining) {
                    (Some(target), Some(days)) if days > 0 && target > current_words => {
                        Some((target - current_words) / days as usize)
                    }
                    _ => None,
                };
                // Build per-document target progress
                let word_counts = crate::core::integrations::ProjectState::word_count_map(&project.binder);
                let doc_progress: Vec<(String, _)> = self
                    .project_state
                    .targets
                    .all_progress(&word_counts)
                    .into_iter()
                    .filter_map(|p| project.binder.find_item(&p.doc_id).map(|item| (item.title.clone(), p)))
                    .collect();
                let data = views::targets_panel::TargetsData {
                    project_target: project.settings.target_word_count,
                    current_words,
                    deadline,
                    session_target: self.session_goal,
                    session_words: self.session_stats.words_written,
                    days_remaining,
                    words_per_day_needed,
                    doc_progress,
                };
                Some(views::targets_panel::view(&data))
            }
            BottomPanel::QuickRef => {
                if let Some(ref_id) = self.quick_ref_item {
                    self.project
                        .as_ref()
                        .and_then(|p| p.binder.find_item(&ref_id))
                        .map(|item| {
                            let data = views::quick_reference_panel::QuickRefData::from_item(item);
                            views::quick_reference_panel::view(&data)
                        })
                } else {
                    None
                }
            }
            BottomPanel::FindReplace => {
                let data = views::find_replace_panel::FindReplaceData {
                    find_text: self.doc_find_text.clone(),
                    replace_text: self.doc_replace_text.clone(),
                    match_count: self.doc_find_match_count,
                    case_sensitive: self.doc_find_case_sensitive,
                    current_match: self.doc_find_current_match,
                    whole_word: self.doc_find_whole_word,
                    use_regex: self.doc_find_use_regex,
                };
                Some(views::find_replace_panel::view(&data))
            }
            BottomPanel::WritingGoals => {
                let words_today = if self.session_active {
                    self.session_stats.words_written
                } else {
                    project
                        .writing_history
                        .entries
                        .last()
                        .filter(|e| e.date == Utc::now().date_naive())
                        .map(|e| e.words_written)
                        .unwrap_or(0)
                };
                let words_this_week: i64 = project.writing_history.recent(7).iter().map(|e| e.words_written).sum();
                let data = views::writing_goals_panel::WritingGoalsData {
                    daily_goal: self.daily_goal,
                    daily_goal_text: self.daily_goal_text.clone(),
                    weekly_goal: self.weekly_goal,
                    weekly_goal_text: self.weekly_goal_text.clone(),
                    words_today,
                    words_this_week,
                    streak: project.writing_history.current_streak(),
                    avg_daily: project.writing_history.average_words_per_day(),
                    days_this_week: project.writing_history.recent(7).len(),
                };
                Some(views::writing_goals_panel::view(&data))
            }
            BottomPanel::DocLinks => {
                // Use the links module for proper link parsing and validation
                let mut outgoing = Vec::new();
                let mut incoming = Vec::new();
                let mut broken = Vec::new();
                let current_title = self
                    .selected_item
                    .and_then(|id| project.binder.find_item(&id))
                    .map(|item| item.title.clone())
                    .unwrap_or_default();

                if let Some(item_id) = self.selected_item {
                    if let Some(item) = project.binder.find_item(&item_id) {
                        // Use links module to extract and validate outgoing links
                        let validations = crate::core::links::validate_document_links(item, &project.binder);
                        for v in &validations {
                            match &v.status {
                                crate::core::links::LinkStatus::Valid(target_id) => {
                                    if let Some(target) = project.binder.find_item(target_id) {
                                        outgoing.push(views::doc_links_panel::DocLink {
                                            target_title: target.title.clone(),
                                            target_id: *target_id,
                                            link_text: v.link.link_text.clone(),
                                            display_text: v.link.display_text.clone(),
                                        });
                                    }
                                }
                                crate::core::links::LinkStatus::Broken => {
                                    let suggestions =
                                        crate::core::links::suggest_link_targets(&v.link.link_text, &project.binder);
                                    broken.push(views::doc_links_panel::BrokenDocLink {
                                        link_text: v.link.link_text.clone(),
                                        status: views::doc_links_panel::BrokenLinkStatus::Broken,
                                        suggestions,
                                    });
                                }
                                crate::core::links::LinkStatus::Ambiguous(ids) => {
                                    broken.push(views::doc_links_panel::BrokenDocLink {
                                        link_text: v.link.link_text.clone(),
                                        status: views::doc_links_panel::BrokenLinkStatus::Ambiguous(ids.len()),
                                        suggestions: Vec::new(),
                                    });
                                }
                            }
                        }
                    }

                    // Find incoming links (docs that link to current)
                    for other_item in project.binder.all_items() {
                        if other_item.id == item_id {
                            continue;
                        }
                        if let Some(ref doc) = other_item.document {
                            let links = crate::core::links::extract_links(&doc.content);
                            for link in &links {
                                if link.link_text == current_title {
                                    incoming.push(views::doc_links_panel::DocLink {
                                        target_title: other_item.title.clone(),
                                        target_id: other_item.id,
                                        link_text: format!("[[{}]]", current_title),
                                        display_text: link.display_text.clone(),
                                    });
                                    break; // one entry per source document
                                }
                            }
                        }
                    }
                }

                // Available docs for quick insertion
                let available: Vec<(uuid::Uuid, String)> = project
                    .binder
                    .all_items()
                    .iter()
                    .filter(|i| i.document.is_some() && Some(i.id) != self.selected_item)
                    .map(|i| (i.id, i.title.clone()))
                    .collect();

                Some(views::doc_links_panel::view(&outgoing, &incoming, &broken, &available))
            }
            BottomPanel::Backups => {
                let backups = match (&project.path, BackupManager::default_root()) {
                    (Some(path), Ok(root)) => BackupManager::list_backups(&root, path).unwrap_or_default(),
                    _ => Vec::new(),
                };
                let auto_backup_every = project
                    .settings
                    .auto_backup
                    .then_some(project.settings.backup_interval_saves.max(1));
                Some(views::backup_panel::view(&backups, &project.title, auto_backup_every))
            }
            BottomPanel::SpellCheck => Some(views::spell_check_panel::view(
                &self.spell_check_results,
                self.spell_checker.active,
                self.spell_checker.dictionary_size(),
                self.spell_checker.user_words(),
            )),
            BottomPanel::Timer => {
                let wc = self.current_word_count();
                Some(views::timer_panel::view(&self.writing_timer, wc))
            }
            BottomPanel::Validation => Some(views::validation_panel::view(self.validation_result.as_ref())),
            BottomPanel::Templates => {
                let templates = crate::core::doc_templates::builtin_templates();
                Some(views::templates_panel::view(&templates))
            }
            BottomPanel::WritingPrompts => Some(views::writing_prompts_panel::view(&self.writing_prompts_data)),
            BottomPanel::None => None,
        };

        // Status bar
        let stats = Statistics::from_binder(&project.binder);
        let timer_remaining = if self.writing_timer.is_running() {
            self.writing_timer.remaining_display()
        } else {
            String::new()
        };
        let streak = project.writing_history.current_streak();
        let status_bar = views::status_bar::view(&views::status_bar::StatusBarParams {
            stats: &stats,
            target_words: project.settings.target_word_count,
            is_dirty: self.editor.dirty,
            project_title: &project.title,
            session_active: self.session_active,
            cursor_line: self.editor.current_line(),
            cursor_col: self.editor.current_column(),
            timer_running: self.writing_timer.is_running(),
            timer_remaining: &timer_remaining,
            writing_streak: streak,
        });

        // Notification bar
        let notification_bar: Option<Element<'_, Message>> = self.notification.as_ref().map(|msg| {
            container(
                row![
                    text(msg.clone()).size(12).color(Theme::WARNING),
                    iced::widget::Space::new().width(Length::Fill),
                    iced::widget::button(text("x").size(12).color(Theme::TEXT_MUTED),)
                        .on_press(Message::DismissNotification)
                        .padding(Padding::from([2, 8])),
                ]
                .padding(Padding::from([4, 12])),
            )
            .width(Length::Fill)
            .into()
        });

        // Layout assembly
        let mut main_row = row![binder, main_content];
        if let Some(insp) = inspector {
            main_row = main_row.push(insp);
        }

        let mut layout = column![toolbar];
        if let Some(notif) = notification_bar {
            layout = layout.push(notif);
        }
        layout = layout.push(main_row);
        if let Some(bp) = bottom {
            layout = layout.push(bp);
        }
        layout = layout.push(status_bar);

        // If a dropdown menu is open, stack it as a floating overlay
        if let Some(overlay) = dropdown_overlay {
            // The overlay column: an empty spacer for the menu bar height,
            // then the dropdown floating over the rest of the content
            let floating = column![
                // Spacer matching the menu bar height (~33px)
                Space::new().height(33),
                overlay,
            ];

            stack![
                container(layout).width(Length::Fill).height(Length::Fill),
                container(floating).width(Length::Fill).height(Length::Fill),
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            container(layout).width(Length::Fill).height(Length::Fill).into()
        }
    }

    /// Keyboard shortcuts and auto-save timer
    pub fn subscription(&self) -> Subscription<Message> {
        let key_sub = keyboard::listen().filter_map(|event| {
            let keyboard::Event::KeyPressed { key, modifiers, .. } = event else {
                return None;
            };
            let ctrl = modifiers.control() || modifiers.command();
            let shift = modifiers.shift();

            let alt = modifiers.alt();

            if ctrl && shift {
                // Ctrl+Shift shortcuts
                match key {
                    keyboard::Key::Character(c) => {
                        let c = c.as_str();
                        match c {
                            "i" | "I" => Some(Message::InsertItalic),
                            "x" | "X" => Some(Message::InsertStrikethrough),
                            "f" | "F" => Some(Message::ToggleCompositionMode),
                            "s" | "S" => Some(Message::ShowProjectStats),
                            "g" | "G" => Some(Message::ShowBottomPanel(BottomPanel::WritingGoals)),
                            "t" | "T" => Some(Message::ToggleScriptMode),
                            "k" | "K" => Some(Message::DeleteLine),
                            "d" | "D" => Some(Message::DuplicateLine),
                            "j" | "J" => Some(Message::JoinLines),
                            "l" | "L" => Some(Message::SortLines),
                            "u" | "U" => Some(Message::RemoveDuplicateLines),
                            "b" | "B" => Some(Message::CreateBackup),
                            "e" | "E" => Some(Message::CloseSplitEditor),
                            "n" | "N" => Some(Message::ShowBottomPanel(BottomPanel::NameGen)),
                            "h" | "H" => Some(Message::ShowBottomPanel(BottomPanel::History)),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            } else if ctrl {
                match key {
                    keyboard::Key::Character(c) => {
                        let c = c.as_str();
                        match c {
                            "s" => Some(Message::SaveProject),
                            "n" => Some(Message::NewDocument),
                            "f" => Some(Message::ShowBottomPanel(BottomPanel::Search)),
                            "e" => Some(Message::ShowCompileDialog),
                            "i" => Some(Message::ToggleInspector),
                            "z" => Some(Message::Undo),
                            "y" => Some(Message::Redo),
                            "," => Some(Message::ShowSettings),
                            "b" => Some(Message::InsertBold),
                            "u" => Some(Message::InsertUnderline),
                            "k" => Some(Message::InsertLink),
                            "h" => Some(Message::ShowBottomPanel(BottomPanel::FindReplace)),
                            "d" => Some(Message::ShowBottomPanel(BottomPanel::Annotations)),
                            "g" => Some(Message::ShowBottomPanel(BottomPanel::WritingGoals)),
                            "t" => Some(Message::TransposeChars),
                            "l" => Some(Message::ShowBottomPanel(BottomPanel::DocLinks)),
                            "j" => Some(Message::ShowBottomPanel(BottomPanel::Timer)),
                            "/" => Some(Message::ToggleComment),
                            "5" => Some(Message::ShowBottomPanel(BottomPanel::Snapshots)),
                            "6" => Some(Message::ShowBottomPanel(BottomPanel::Session)),
                            "7" => Some(Message::ShowBottomPanel(BottomPanel::TextStats)),
                            "8" => Some(Message::ShowBottomPanel(BottomPanel::Backups)),
                            "9" => Some(Message::ShowBottomPanel(BottomPanel::Targets)),
                            "1" => Some(Message::SwitchView(ViewMode::Editor)),
                            "2" => Some(Message::SwitchView(ViewMode::Corkboard)),
                            "3" => Some(Message::SwitchView(ViewMode::Outliner)),
                            "4" => Some(Message::SwitchView(ViewMode::Scrivenings)),
                            "p" => Some(Message::ShowBottomPanel(BottomPanel::ProjectNotes)),
                            "m" => Some(Message::ShowBottomPanel(BottomPanel::Bookmarks)),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            } else if alt {
                match key {
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) => Some(Message::MoveLineUp),
                    keyboard::Key::Named(keyboard::key::Named::ArrowDown) => Some(Message::MoveLineDown),
                    keyboard::Key::Character(c) => {
                        let c = c.as_str();
                        match c {
                            "[" => Some(Message::UnindentLine),
                            "]" => Some(Message::IndentLine),
                            "u" => Some(Message::TextToUppercase),
                            "l" => Some(Message::TextToLowercase),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            } else {
                match key {
                    keyboard::Key::Named(keyboard::key::Named::Escape) => Some(Message::EscapePressed),
                    keyboard::Key::Named(keyboard::key::Named::F11) => Some(Message::ToggleFullscreen),
                    keyboard::Key::Named(keyboard::key::Named::F5) => Some(Message::ToggleCompositionMode),
                    keyboard::Key::Named(keyboard::key::Named::F7) => Some(Message::RunSpellCheck),
                    keyboard::Key::Named(keyboard::key::Named::F3) => Some(Message::DocFindNext),
                    keyboard::Key::Named(keyboard::key::Named::F6) => {
                        Some(Message::ShowBottomPanel(BottomPanel::Search))
                    }
                    keyboard::Key::Named(keyboard::key::Named::F8) => Some(Message::ShowValidation),
                    keyboard::Key::Named(keyboard::key::Named::F9) => Some(Message::CreateSnapshot),
                    _ => None,
                }
            }
        });

        let tick_sub = iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick);

        let close_sub = window::close_events().map(Message::WindowClosed);

        Subscription::batch([key_sub, tick_sub, close_sub])
    }

    /// Dark theme
    pub fn theme(&self, _window_id: window::Id) -> iced::Theme {
        iced::Theme::Dark
    }

    // ========== Extracted handler methods (reduce update() size) ==========

    /// Save the open project into its own folder or, the first time it is
    /// saved, into `new_dir`, and record the folder in the recent projects.
    fn save_project(&mut self, new_dir: Option<&Path>) {
        let Some(project) = self.project.as_mut() else {
            return;
        };
        let result = match new_dir {
            Some(dir) => project.save_as(dir),
            None => project.save(),
        };
        match (result, &project.path) {
            (Ok(()), Some(path)) => {
                self.editor.mark_clean();
                self.recent_projects.add(&project.title, path.clone());
                self.recent_projects.save();
                self.notification = Some(format!("Project saved to {}", path.display()));
                self.auto_backup();
            }
            (Ok(()), None) => {}
            (Err(e), _) => {
                self.notification = Some(format!("Save error: {:#}", e));
            }
        }
    }

    /// Count a save of the open project and back it up when its settings
    /// (automatic backups on, every `backup_interval_saves` saves) say so.
    fn auto_backup(&mut self) {
        let Some(project) = &self.project else {
            return;
        };
        let Some(dir) = &project.path else {
            return;
        };
        let result = BackupManager::default_root().and_then(|root| {
            BackupManager::backup_after_save(dir, &root, &project.settings, &mut self.saves_since_backup)
        });
        if let Err(e) = result {
            log::warn!("Auto-backup failed: {:#}", e);
            self.notification = Some(format!("Automatic backup failed: {:#}", e));
        }
    }

    /// Save the project after its settings changed, if it already has a folder.
    fn save_settings(&mut self) {
        if let Some(project) = self.project.as_mut().filter(|p| p.path.is_some()) {
            if let Err(e) = project.save() {
                self.notification = Some(format!("Settings save failed: {:#}", e));
            }
        }
    }

    /// Add the files in `paths` to the draft folder of the open project.
    fn import_files(&mut self, paths: Vec<PathBuf>) {
        let Some(project) = self.project.as_mut() else {
            self.notification = Some("Create or open a project first.".to_string());
            return;
        };
        self.notification = None;
        let mut count = 0;
        for path in paths {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

            // A Scrivener project is a .scriv folder. On most systems the file
            // dialog cannot pick a folder, so it picks the .scrivx file inside.
            let scriv_package = match ext.as_str() {
                "scriv" if path.is_dir() => Some(path.as_path()),
                "scrivx" => path.parent(),
                _ => None,
            };
            if let Some(package) = scriv_package {
                match crate::export::scriv_import::import_scriv(package) {
                    Ok((_info, items)) => {
                        for item in items {
                            project.binder.draft.add_child(item);
                            count += 1;
                        }
                    }
                    Err(e) => {
                        self.notification = Some(format!("Scrivener import error: {}", e));
                    }
                }
                continue;
            }
            let title = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Imported")
                .to_string();
            match ext.as_str() {
                "md" | "markdown" => {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        // Use structured Markdown import (headings -> folders/docs)
                        match crate::export::markdown_import::import_markdown(&content, &title) {
                            Ok(items) => {
                                for item in items {
                                    project.binder.draft.add_child(item);
                                    count += 1;
                                }
                            }
                            Err(_) => {
                                // Fallback to plain import
                                let mut item = BinderItem::new_text(&title);
                                if let Some(ref mut doc) = item.document {
                                    doc.content = content;
                                }
                                project.binder.draft.add_child(item);
                                count += 1;
                            }
                        }
                    }
                }
                "txt" | "rtf" => {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let mut item = BinderItem::new_text(&title);
                        if let Some(ref mut doc) = item.document {
                            doc.content = content;
                        }
                        project.binder.draft.add_child(item);
                        count += 1;
                    }
                }
                "html" | "htm" => {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        // Use structured HTML import (headings -> folders/docs)
                        match crate::export::web_import::import_html_content(&content, &title) {
                            Ok(items) if !items.is_empty() => {
                                for item in items {
                                    project.binder.draft.add_child(item);
                                    count += 1;
                                }
                            }
                            _ => {
                                // Fallback to plain text conversion
                                let plain = crate::export::web_import::html_to_plain_text(&content);
                                let mut item = BinderItem::new_text(&title);
                                if let Some(ref mut doc) = item.document {
                                    doc.content = plain;
                                }
                                project.binder.draft.add_child(item);
                                count += 1;
                            }
                        }
                    }
                }
                "tex" | "latex" => {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let plain = helpers::strip_latex_commands(&content);
                        let mut item = BinderItem::new_text(&title);
                        if let Some(ref mut doc) = item.document {
                            doc.content = plain;
                        }
                        project.binder.draft.add_child(item);
                        count += 1;
                    }
                }
                "fountain" => {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let sections = crate::export::fountain::parse_fountain(&content);
                        for (sec_title, text) in sections {
                            let mut item = BinderItem::new_text(&sec_title);
                            if let Some(ref mut doc) = item.document {
                                doc.content = text;
                            }
                            project.binder.draft.add_child(item);
                            count += 1;
                        }
                    }
                }
                "opml" => {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        match crate::export::opml::import_opml(&content) {
                            Ok(items) => {
                                for item in items {
                                    project.binder.draft.add_child(item);
                                    count += 1;
                                }
                            }
                            Err(e) => {
                                self.notification = Some(format!("OPML parse error: {}", e));
                            }
                        }
                    }
                }
                "docx" => {
                    // Use structured DOCX import (headings -> folders/docs)
                    match crate::export::docx_import::import_docx(&path) {
                        Ok(items) => {
                            for item in items {
                                project.binder.draft.add_child(item);
                                count += 1;
                            }
                        }
                        Err(e) => {
                            // Fallback: create a plain text item with error note
                            let mut item = BinderItem::new_text(&title);
                            if let Some(ref mut doc) = item.document {
                                doc.content = String::new();
                                doc.notes = format!("DOCX import failed: {}", e);
                            }
                            project.binder.draft.add_child(item);
                            count += 1;
                            self.notification = Some(format!("DOCX import error for '{}': {}", title, e));
                        }
                    }
                }
                _ => {}
            }
        }
        if count > 0 {
            self.notification = Some(format!("Imported {} item(s)", count));
        } else if self.notification.is_none() {
            self.notification = Some("Nothing could be imported from the selected files".to_string());
        }
    }

    /// Handle the per-second tick: auto-save, milestones, session timers,
    /// collection refresh, external change polling, and notification dismissal.
    fn handle_tick(&mut self) {
        // Auto-save
        if self.project.is_some() && self.editor.dirty {
            self.auto_save_counter += 1;
            let interval = self
                .project
                .as_ref()
                .map(|p| p.settings.auto_save_seconds)
                .unwrap_or(30);
            if interval > 0 && self.auto_save_counter >= interval {
                self.auto_save_counter = 0;
                self.sync_editor_to_project();
                if let Some(ref mut project) = self.project {
                    // A project that was never saved has no folder yet, and
                    // only the save dialog may choose one.
                    if project.path.is_none() {
                        self.notification = Some("Save the project once to turn on autosave".to_string());
                    } else {
                        match project.save() {
                            Ok(()) => {
                                self.editor.mark_clean();
                                self.auto_backup();
                            }
                            Err(e) => {
                                self.notification = Some(format!("Autosave failed: {:#}", e));
                            }
                        }
                    }
                }
            }
        }

        // Word count milestone detection
        if self.auto_save_counter.is_multiple_of(MILESTONE_CHECK_INTERVAL) {
            if let Some(ref project) = self.project {
                let total_words = project.binder.total_word_count();
                let milestones = [1000, 5000, 10000, 25000, 50000, 75000, 100000, 150000, 200000];
                for &m in &milestones {
                    if total_words >= m && self.last_milestone < m {
                        self.last_milestone = m;
                        let label = if m >= 1000 {
                            format!("{}k", m / 1000)
                        } else {
                            m.to_string()
                        };
                        self.notification = Some(format!("\u{f005} Milestone: {} words! Keep writing!", label));
                        break;
                    }
                }
            }
        }

        // Writing session timer
        if self.session_active {
            let current_words = self.current_word_count();
            let word_delta = current_words as i64 - self.session_start_word_count as i64;
            let prev_words = self.session_stats.words_written;
            self.session_stats
                .update(word_delta, self.session_stats.time_elapsed_seconds + 1);

            if self.session_goal > 0 {
                let new_words = self.session_stats.words_written;
                let goal = self.session_goal as i64;
                if prev_words < goal && new_words >= goal {
                    self.notification = Some(format!(
                        "\u{f00c} Session goal of {} words reached! Keep going!",
                        self.session_goal
                    ));
                }
            }

            if self.daily_goal > 0
                && self
                    .session_stats
                    .time_elapsed_seconds
                    .is_multiple_of(DAILY_GOAL_CHECK_INTERVAL)
            {
                let words_today = self.session_stats.words_written;
                let daily_goal = self.daily_goal as i64;
                if words_today >= daily_goal && (words_today - DAILY_GOAL_CHECK_INTERVAL as i64) < daily_goal {
                    self.notification = Some(format!("\u{f00c} Daily goal of {} words reached!", self.daily_goal));
                }
            }

            if self
                .session_stats
                .time_elapsed_seconds
                .is_multiple_of(HISTORY_RECORD_INTERVAL)
            {
                if let Some(ref mut project) = self.project {
                    project.writing_history.record(current_words, HISTORY_RECORD_INTERVAL);
                }
            }

            if self.session_stats.time_elapsed_seconds == POMODORO_BREAK_SECONDS && self.notification.is_none() {
                self.notification = Some("\u{f0f4} 25 minutes of writing! Consider a short break.".to_string());
            }
        }

        // Writing focus timer tick
        if self.writing_timer.is_running() && self.writing_timer.tick() {
            let word_count = self.current_word_count();
            self.writing_timer.stop(word_count);
            let summary = self.writing_timer.summary();
            self.notification = Some(format!(
                "\u{f00c} Timer completed! {} | Great writing session!",
                summary
            ));
        }

        // Auto-refresh smart collections
        if self.bottom_panel == BottomPanel::Collections
            && self.auto_save_counter.is_multiple_of(COLLECTION_REFRESH_INTERVAL)
        {
            if let Some(ref mut project) = self.project {
                for coll in &mut project.collections {
                    if let crate::core::collection::CollectionKind::Search {
                        ref query,
                        case_sensitive,
                        whole_word,
                    } = coll.kind
                    {
                        let options = crate::core::search::SearchOptions {
                            query: query.clone(),
                            case_sensitive,
                            whole_word,
                            regex: false,
                            search_titles: true,
                            search_content: true,
                            search_notes: false,
                            search_synopsis: true,
                            ..Default::default()
                        };
                        let results = crate::core::search::search_binder(&project.binder, &options);
                        let item_ids: Vec<Uuid> = results.iter().map(|r| r.item_id).collect();
                        coll.item_ids = item_ids;
                    }
                }
            }
        }

        // Poll file watcher for external changes
        if self.project_state.poll_external_changes() {
            self.notification = Some("External changes detected. Consider reloading.".to_string());
            self.project_state.clear_external_changes();
        }

        // Auto-dismiss notification after 8 seconds
        if self.notification.is_some() {
            self.notification_timer = self.notification_timer.saturating_add(1);
            if self.notification_timer >= 8 {
                self.notification = None;
                self.notification_timer = 0;
            }
        } else {
            self.notification_timer = 0;
        }
    }
}

// Free functions (title_case, strip_latex_commands) live in gui/helpers.rs

/// File extensions the Import dialog offers. A `.scriv` package is a folder,
/// so on most systems it is picked through the `.scrivx` file inside it.
const IMPORT_EXTENSIONS: &[&str] = &[
    "txt", "rtf", "md", "markdown", "html", "htm", "tex", "latex", "fountain", "opml", "docx", "scrivx", "scriv",
];

/// Load the project in `dir` off the GUI thread.
async fn load_project(dir: PathBuf) -> Result<Project, String> {
    Project::load_async(dir).await.map_err(|e| format!("{:#}", e))
}

/// Ask where to write a file, starting in `dir` when there is one.
async fn pick_save_path(title: &'static str, dir: Option<PathBuf>, file_name: String) -> Option<PathBuf> {
    let mut dialog = rfd::AsyncFileDialog::new().set_title(title).set_file_name(file_name);
    if let Some(dir) = dir {
        dialog = dialog.set_directory(dir);
    }
    dialog.save_file().await.map(|file| file.path().to_path_buf())
}

/// Ask which files to import, starting in `dir` when there is one. Returns
/// an empty list when the dialog is cancelled.
async fn pick_import_paths(
    dir: Option<PathBuf>,
    filter: &'static str,
    extensions: &'static [&'static str],
) -> Vec<PathBuf> {
    let mut dialog = rfd::AsyncFileDialog::new()
        .set_title("Import")
        .add_filter(filter, extensions);
    if let Some(dir) = dir {
        dialog = dialog.set_directory(dir);
    }
    dialog
        .pick_files()
        .await
        .unwrap_or_default()
        .iter()
        .map(|file| file.path().to_path_buf())
        .collect()
}
