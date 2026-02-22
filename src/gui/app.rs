use iced::keyboard;
use iced::widget::{column, container, row, text, text_editor};
use iced::{Element, Length, Padding, Subscription, Task as IcedTask};
use uuid::Uuid;
use chrono;

use crate::core::binder::{BinderItem, BinderItemKind};
use crate::core::document::Document;
use crate::core::project::Project;
use crate::core::search::{self, SearchOptions};
use crate::core::stats::{SessionStats, Statistics};
use crate::editor::EditorState;
use crate::export::compiler::{CompileOptions, OutputFormat, SeparatorType};
use crate::spelling::SpellChecker;
use crate::thesaurus::Thesaurus;

use super::theme::Theme;
use super::views;

/// The active view mode
#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Editor,
    Corkboard,
    Outliner,
    Scrivenings,
}

/// Which bottom panel is visible
#[derive(Debug, Clone, PartialEq)]
pub enum BottomPanel {
    None,
    Search,
    Thesaurus,
    Snapshots,
    Session,
    History,
    TextStats,
    NameGen,
    ProjectNotes,
    Collections,
    Bookmarks,
    Annotations,
    Targets,
    QuickRef,
    FindReplace,
    WritingGoals,
    DocLinks,
    Backups,
    SpellCheck,
    Timer,
    Validation,
    Templates,
}

/// Application state
pub struct ScrineverApp {
    // === Project ===
    pub project: Option<Project>,
    pub selected_item: Option<Uuid>,
    pub editor: EditorState,

    // === View state ===
    pub view_mode: ViewMode,
    pub show_inspector: bool,
    pub fullscreen_editor: bool,
    pub bottom_panel: BottomPanel,

    // === Dialogs ===
    pub show_compile_dialog: bool,
    pub show_settings_dialog: bool,
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
    pub item_targets: std::collections::HashMap<Uuid, usize>,

    // === Notification ===
    pub notification: Option<String>,

    // === Auto-save ===
    pub auto_save_counter: u32,

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

    // === Recent projects ===
    pub recent_projects: crate::core::recent::RecentProjects,

    // === Split editor ===
    pub split_editor_item: Option<Uuid>,

    // === Document find/replace ===
    pub doc_find_text: String,
    pub doc_replace_text: String,
    pub doc_find_case_sensitive: bool,
    pub doc_find_match_count: usize,
    pub doc_find_current_match: usize,
    pub doc_find_positions: Vec<usize>,

    // === Quick reference ===
    pub quick_ref_item: Option<Uuid>,

    // === Script mode ===
    pub script_mode: bool,
    pub current_script_element: Option<crate::core::script::ScriptElement>,
    pub auto_correction: crate::core::script::AutoCorrection,

    // === Revision tracking ===
    pub current_revision: Option<crate::core::script::RevisionLevel>,

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
}

/// Messages for the application
#[derive(Debug, Clone)]
pub enum Message {
    // Project operations
    NewProject,
    NewFromTemplate(String),
    OpenProject,
    SaveProject,
    ProjectLoaded(Option<Project>),

    // Binder operations
    SelectBinderItem(Uuid),
    ToggleBinderItem(Uuid),
    NewDocument,
    NewFolder,
    DeleteItem(Uuid),
    RenameItem(Uuid, String),
    UpdateSynopsis(Uuid, String),
    MoveItem { item_id: Uuid, target_id: Uuid, position: usize },
    MoveItemUp(Uuid),
    MoveItemDown(Uuid),
    DuplicateItem(Uuid),
    EmptyTrash,
    ConvertToFolder(Uuid),
    ConvertToText(Uuid),
    SplitDocument,
    MergeIntoParent,

    // Editor operations
    EditorAction(text_editor::Action),

    // View operations
    SwitchView(ViewMode),
    ToggleInspector,
    ToggleFullscreen,
    ShowBottomPanel(BottomPanel),

    // Compile operations
    ShowCompileDialog,
    HideCompileDialog,
    CompileSetFormat(String),
    CompileSetTitle(String),
    CompileSetAuthor(String),
    CompileSetFrontMatter(bool),
    CompileSetMarkedOnly(bool),
    CompileSetPageBreaks(bool),
    CompileSetFontFamily(String),
    CompileSetFontSize(String),
    CompileSetSeparator(String),
    DoCompile,

    // Snapshot operations
    CreateSnapshot,
    RestoreSnapshot(usize),

    // Search operations
    SearchQueryChanged(String),
    DoSearch,
    SearchToggleCaseSensitive,
    SearchToggleWholeWord,
    SearchToggleRegex,
    ReplaceTextChanged(String),
    DoReplaceAll,
    GoToSearchResult(Uuid),

    // Thesaurus operations
    ThesaurusQueryChanged(String),
    DoThesaurusLookup,
    InsertSynonym(String),

    // Document notes
    NotesChanged(String),

    // Target word count
    SetItemTarget(Uuid, String),

    // Metadata
    SetItemStatus(Uuid, String),
    SetItemLabel(Uuid, String),
    ToggleIncludeInCompile(Uuid),

    // Settings dialog
    ShowSettings,
    HideSettings,
    SettingsSetProjectTitle(String),
    SettingsSetFont(String),
    SettingsSetFontSize(String),
    SettingsZoomIn,
    SettingsZoomOut,
    SettingsSetTarget(String),
    SettingsSetAutoSave(String),
    SettingsToggleWordCount(bool),

    // Writing session
    SessionToggle,
    SessionReset,
    SessionSetGoal(String),

    // Import
    ImportFiles,

    // Name generator
    GenerateName(String),
    GenerateNameBatch,

    // Project notes
    ProjectNotesChanged(String),

    // Keywords
    SetItemKeywords(Uuid, String),

    // Collections
    CollectionNameInput(String),
    CreateCollection,
    DeleteCollection(Uuid),
    SelectCollection(Uuid),
    AddToCollection(Uuid),

    // Bookmarks
    ToggleBookmark(Uuid),

    // Annotations
    AnnotationTextInput(String),
    AddAnnotation,
    DeleteAnnotation(Uuid),
    ToggleAnnotationResolved(Uuid),
    EditAnnotation(Uuid, String),
    SetAnnotationColor(Uuid, String),
    SetAnnotationCategory(Uuid, String),
    CycleAnnotationColor,

    // Project targets
    SettingsSetDeadline(String),

    // Text transforms
    TextToUppercase,
    TextToLowercase,
    TextToTitleCase,

    // Undo/Redo
    Undo,
    Redo,

    // Recent projects
    OpenRecentProject(std::path::PathBuf),

    // Split editor
    OpenInSplitEditor(Uuid),
    CloseSplitEditor,

    // Document find/replace
    DocFindChanged(String),
    DocFindNext,
    DocFindPrev,
    DocReplaceCurrent,
    DocReplaceAll,
    DocReplaceChanged(String),
    DocFindToggleCase,

    // Quick reference
    ShowQuickRef(Uuid),

    // Script mode
    ToggleScriptMode,
    SetScriptElement(String),

    // Auto-correction
    ToggleAutoCorrectSmartQuotes,
    ToggleAutoCorrectEmDashes,
    ToggleAutoCorrectEllipsis,

    // Revision level
    SetRevisionLevel(String),

    // Document links
    InsertDocLink(Uuid),

    // Formatting toolbar
    InsertBold,
    InsertItalic,
    InsertUnderline,
    InsertStrikethrough,
    InsertHeading(u8),
    InsertBlockQuote,
    InsertFootnote,
    InsertHRule,

    // Compile presets
    SaveCompilePreset(String),
    LoadCompilePreset(String),

    // Project statistics
    ShowProjectStats,
    HideProjectStats,

    // Writing goals
    SetDailyGoal(String),
    SetWeeklyGoal(String),
    ResetGoals,

    // Composition mode
    ToggleCompositionMode,

    // Copy special
    CopyAsMarkdown,
    CopyAsHtml,
    CopyAsPlainText,

    // Search results to collection
    SaveSearchAsCollection,

    // Backup
    CreateBackup,
    RestoreBackup(std::path::PathBuf),

    // OPML import
    ImportOpml,

    // Smart collection
    CreateSmartCollection(String),

    // Snapshot comparison
    SelectSnapshot(usize),
    CompareSnapshot(usize),

    // Compile options (new)
    CompileSetToc(bool),
    CompileSetPlaceholders(bool),

    // Export OPML
    ExportOpml,

    // Print
    PrintCurrent,
    PrintProject,

    // Spell check
    RunSpellCheck,
    SpellCheckAddWord(String),
    SpellCheckReplace(usize, String, String),
    SpellCheckRemoveWord(String),
    SpellCheckClearDict,
    ToggleSpellChecker,

    // Custom metadata fields
    AddCustomField(Uuid, String),
    UpdateCustomField(Uuid, String, String),
    RemoveCustomField(Uuid, String),

    // Corkboard card color
    SetCardColor(Uuid, String),

    // Smart collection refresh
    RefreshSmartCollections,

    // Composition mode settings
    SettingsSetCompWidth(String),
    SettingsSetCompBgColor(String),

    // Editor text operations
    TransposeChars,
    SortLines,
    RemoveDuplicateLines,
    JoinLines,
    MoveLineUp,
    MoveLineDown,
    DeleteLine,
    DuplicateLine,
    IndentLine,
    UnindentLine,
    ToggleComment,

    // Insert operations
    InsertListItem(String),
    InsertTable(usize, usize),
    InsertCodeBlock(String),
    InsertPageBreak,
    InsertComment,
    InsertDateTime(String),
    SmartPaste(String),
    InsertLink,
    InsertImage,

    // Document templates
    NewDocFromTemplate(String),

    // Writing timer
    TimerStart,
    TimerPause,
    TimerResume,
    TimerStop,
    TimerReset,
    TimerSetPreset(String),

    // Project validation
    ShowValidation,

    // Misc
    Tick,
    DismissNotification,
    EscapePressed,
}

impl ScrineverApp {
    pub fn new() -> (Self, IcedTask<Message>) {
        let mut spell_checker = SpellChecker::new();
        spell_checker.try_init();
        spell_checker.load_user_dictionary();

        let app = Self {
            project: None,
            selected_item: None,
            editor: EditorState::new(),
            view_mode: ViewMode::Editor,
            show_inspector: true,
            fullscreen_editor: false,
            bottom_panel: BottomPanel::None,
            show_compile_dialog: false,
            show_settings_dialog: false,
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
            item_targets: std::collections::HashMap::new(),
            notification: None,
            auto_save_counter: 0,
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
            recent_projects: crate::core::recent::RecentProjects::load(),
            split_editor_item: None,
            doc_find_text: String::new(),
            doc_replace_text: String::new(),
            doc_find_case_sensitive: false,
            doc_find_match_count: 0,
            doc_find_current_match: 0,
            doc_find_positions: Vec::new(),
            quick_ref_item: None,
            script_mode: false,
            current_script_element: None,
            auto_correction: crate::core::script::AutoCorrection::default(),
            current_revision: None,
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
        };

        (app, IcedTask::none())
    }

    pub fn title(&self) -> String {
        let dirty = if self.editor.dirty { " *" } else { "" };
        match &self.project {
            Some(p) => format!("Avalon - {}{}", p.title, dirty),
            None => "Avalon".to_string(),
        }
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
        self.project.as_ref()
            .map(|p| p.binder.total_word_count())
            .unwrap_or(0)
    }

    /// Insert markdown-style wrapping markup (e.g., ** for bold)
    fn insert_markdown_wrap(&mut self, marker: &str) {
        let wrap = format!("{}text{}", marker, marker);
        self.editor.content.perform(
            iced::widget::text_editor::Action::Edit(
                iced::widget::text_editor::Edit::Paste(
                    std::sync::Arc::new(wrap)
                )
            )
        );
        self.editor.mark_dirty();
    }

    pub fn update(&mut self, message: Message) -> IcedTask<Message> {
        match message {
            // ========== Project operations ==========
            Message::NewProject => {
                self.project = Some(Project::new("Untitled Project"));
                self.selected_item = None;
                self.editor = EditorState::new();
                self.project_notes_text.clear();
                self.generated_names.clear();
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
                if let Some(ref p) = self.project {
                    self.compile_options.title = p.title.clone();
                }
            }

            Message::OpenProject => {
                let home = dirs::home_dir().unwrap_or_default();
                let projects_dir = home.join("Scrinever Projects");
                if projects_dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_dir() && path.extension().map(|e| e == "scriv").unwrap_or(false) {
                                match Project::load(&path) {
                                    Ok(p) => {
                                        self.compile_options.title = p.title.clone();
                                        self.project_notes_text = p.project_notes.clone();
                                        self.generated_names.clear();
                                        self.project = Some(p);
                                        self.selected_item = None;
                                        self.editor = EditorState::new();
                                        self.notification = Some(format!("Opened project from {:?}", path));
                                        return IcedTask::none();
                                    }
                                    Err(e) => {
                                        self.notification = Some(format!("Load error: {}", e));
                                    }
                                }
                            }
                        }
                    }
                    self.notification = Some("No .scriv projects found in ~/Scrinever Projects/".to_string());
                } else {
                    self.notification = Some("No projects directory found. Create a project first.".to_string());
                }
            }

            Message::SaveProject => {
                self.sync_editor_to_project();
                if let Some(ref mut project) = self.project {
                    let home = dirs::home_dir().unwrap_or_default();
                    let save_dir = home.join("Scrinever Projects");
                    match project.save(&save_dir) {
                        Ok(_) => {
                            self.editor.mark_clean();
                            if let Some(ref path) = project.path {
                                self.recent_projects.add(&project.title, path.clone());
                                self.recent_projects.save();
                            }
                            self.notification = Some(format!("Project saved to {:?}", save_dir));
                        }
                        Err(e) => {
                            self.notification = Some(format!("Save error: {}", e));
                        }
                    }
                }
            }

            Message::ProjectLoaded(project) => {
                if let Some(p) = project {
                    self.compile_options.title = p.title.clone();
                    self.project_notes_text = p.project_notes.clone();
                    self.compile_presets = p.compile_presets.clone();
                    self.generated_names.clear();
                    self.project = Some(p);
                    self.selected_item = None;
                    self.editor = EditorState::new();
                }
            }

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

            Message::MoveItem { item_id, target_id, position } => {
                if let Some(ref mut project) = self.project {
                    if let Some(item) = project.binder.draft.remove_child(&item_id)
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
                        project.binder.find_item(&item_id)
                            .and_then(|item| item.document.as_ref())
                            .map(|doc| doc.content.clone())
                    };

                    if let Some(content) = split_content {
                        let mid = content.len() / 2;
                        let split_pos = content[mid..].find("\n\n")
                            .map(|p| p + mid)
                            .unwrap_or(mid);

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
                        project.binder.find_item(&item_id)
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
                self.editor.content.perform(action);
                if is_edit {
                    self.editor.mark_dirty();
                }
            }

            // ========== View operations ==========
            Message::SwitchView(mode) => {
                self.view_mode = mode;
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
                self.sync_editor_to_project();
                if let Some(ref project) = self.project {
                    use crate::export::compiler::Compiler;

                    let home = dirs::home_dir().unwrap_or_default();
                    let output_dir = home.join("Scrinever Output");
                    let _ = std::fs::create_dir_all(&output_dir);
                    let filename = format!(
                        "{}.{}",
                        self.compile_options.title.replace(' ', "_"),
                        self.compile_options.format.extension()
                    );
                    let output_path = output_dir.join(&filename);

                    match Compiler::save_to_file(&project.binder, &self.compile_options, &output_path) {
                        Ok(_) => {
                            self.notification = Some(format!("Compiled to {:?}", output_path));
                            self.show_compile_dialog = false;
                        }
                        Err(e) => {
                            self.notification = Some(format!("Compile error: {}", e));
                        }
                    }
                }
            }

            // ========== Snapshot operations ==========
            Message::CreateSnapshot => {
                self.sync_editor_to_project();
                if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                    let _ = project.create_snapshot(&item_id, "Manual Snapshot");
                    self.notification = Some("Snapshot created".to_string());
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
                    let options = SearchOptions {
                        query: self.search_query.clone(),
                        case_sensitive: self.search_case_sensitive,
                        whole_word: self.search_whole_word,
                        regex: self.search_regex,
                        search_titles: true,
                        search_content: true,
                        search_notes: true,
                        search_synopsis: true,
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
                    let options = SearchOptions {
                        query: self.search_query.clone(),
                        case_sensitive: self.search_case_sensitive,
                        whole_word: self.search_whole_word,
                        regex: self.search_regex,
                        search_titles: false,
                        search_content: true,
                        search_notes: false,
                        search_synopsis: false,
                    };

                    let mut count = 0;
                    for item in project.binder.all_items_mut() {
                        if let Some(ref mut doc) = item.document {
                            let new_content = search::replace_in_document(
                                &doc.content,
                                &options,
                                &self.replace_text,
                            );
                            if new_content != doc.content {
                                count += 1;
                                doc.content = new_content;
                            }
                        }
                    }

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
                self.thesaurus_results = self.thesaurus.lookup(&self.thesaurus_query);
            }

            Message::InsertSynonym(word) => {
                self.editor.content.perform(
                    iced::widget::text_editor::Action::Edit(
                        iced::widget::text_editor::Edit::Paste(
                            std::sync::Arc::new(word)
                        )
                    )
                );
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
                } else if target_str.is_empty() {
                    self.item_targets.remove(&id);
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
                            let label = project.settings.labels.iter()
                                .find(|l| l.name == label_name)
                                .cloned();
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
            Message::ShowSettings => {
                self.show_settings_dialog = true;
            }

            Message::HideSettings => {
                self.show_settings_dialog = false;
                // Persist settings by saving the project
                if let Some(ref mut project) = self.project {
                    if let Some(path) = project.path.clone() {
                        if let Some(parent) = path.parent() {
                            let _ = project.save(parent);
                        }
                    }
                }
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

            // ========== Writing session ==========
            Message::SessionToggle => {
                self.session_active = !self.session_active;
                if self.session_active {
                    self.session_start_word_count = self.current_word_count();
                    self.session_stats = SessionStats::new();
                }
            }

            Message::SessionReset => {
                self.session_active = false;
                self.session_stats = SessionStats::new();
                self.session_start_word_count = self.current_word_count();
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
            Message::ImportFiles => {
                if let Some(ref mut project) = self.project {
                    let home = dirs::home_dir().unwrap_or_default();
                    let import_dir = home.join("Scrinever Import");
                    if import_dir.exists() {
                        let mut count = 0;
                        if let Ok(entries) = std::fs::read_dir(&import_dir) {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                if path.is_file() {
                                    let ext = path.extension()
                                        .and_then(|e| e.to_str())
                                        .unwrap_or("")
                                        .to_lowercase();
                                    let title = path.file_stem()
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
                                                let plain = strip_html_tags(&content);
                                                let mut item = BinderItem::new_text(&title);
                                                if let Some(ref mut doc) = item.document {
                                                    doc.content = plain;
                                                }
                                                project.binder.draft.add_child(item);
                                                count += 1;
                                            }
                                        }
                                        "tex" | "latex" => {
                                            if let Ok(content) = std::fs::read_to_string(&path) {
                                                let plain = strip_latex_commands(&content);
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
                                            // Import DOCX as plain text (basic extraction)
                                            if let Ok(content) = std::fs::read_to_string(&path) {
                                                let mut item = BinderItem::new_text(&title);
                                                if let Some(ref mut doc) = item.document {
                                                    doc.content = content;
                                                }
                                                project.binder.draft.add_child(item);
                                                count += 1;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        if count > 0 {
                            self.notification = Some(format!("Imported {} file(s) from ~/Scrinever Import/", count));
                        } else {
                            self.notification = Some("No importable files found in ~/Scrinever Import/. Supported: txt, md, html, tex, fountain, opml".to_string());
                        }
                    } else {
                        let _ = std::fs::create_dir_all(&import_dir);
                        self.notification = Some("Created ~/Scrinever Import/ — place files there and import again.".to_string());
                    }
                } else {
                    self.notification = Some("Create or open a project first.".to_string());
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
                    let name = project.binder.find_item(&item_id)
                        .map(|i| i.title.clone())
                        .unwrap_or_default();
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
                                let (start, end) = self.editor.selection_range()
                                    .unwrap_or((self.editor.cursor, self.editor.cursor));
                                let ann = crate::core::annotation::Annotation::new(
                                    start,
                                    end,
                                    &self.annotation_text,
                                );
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
                // Cycle the color that will be used for the next annotation
                // This is a UI convenience - stored as a temporary state
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

            Message::TextToLowercase => {
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

            Message::TextToTitleCase => {
                self.sync_editor_to_project();
                if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item_mut(&item_id) {
                        if let Some(ref mut doc) = item.document {
                            doc.content = title_case(&doc.content);
                            self.editor.load_document(doc);
                            self.editor.mark_dirty();
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
                match Project::load(&path) {
                    Ok(p) => {
                        self.compile_options.title = p.title.clone();
                        self.project_notes_text = p.project_notes.clone();
                        self.generated_names.clear();
                        self.recent_projects.add(&p.title, path);
                        self.recent_projects.save();
                        self.project = Some(p);
                        self.selected_item = None;
                        self.editor = EditorState::new();
                    }
                    Err(e) => {
                        self.notification = Some(format!("Load error: {}", e));
                    }
                }
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
                // Find all match positions in current document
                if let (Some(ref project), Some(item_id)) = (&self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item(&item_id) {
                        if let Some(ref doc) = item.document {
                            if !self.doc_find_text.is_empty() {
                                let content = if self.doc_find_case_sensitive {
                                    doc.content.clone()
                                } else {
                                    doc.content.to_lowercase()
                                };
                                let query = if self.doc_find_case_sensitive {
                                    self.doc_find_text.clone()
                                } else {
                                    self.doc_find_text.to_lowercase()
                                };
                                for (pos, _) in content.match_indices(&query) {
                                    self.doc_find_positions.push(pos);
                                }
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
                                let find_len = self.doc_find_text.len();
                                if pos + find_len <= doc.content.len() {
                                    doc.content = format!(
                                        "{}{}{}",
                                        &doc.content[..pos],
                                        self.doc_replace_text,
                                        &doc.content[pos + find_len..]
                                    );
                                    self.editor.load_document(doc);
                                    self.editor.mark_dirty();
                                    // Recalculate positions
                                    let len_diff = self.doc_replace_text.len() as i64 - find_len as i64;
                                    self.doc_find_positions.remove(self.doc_find_current_match);
                                    // Adjust subsequent positions
                                    for p in self.doc_find_positions.iter_mut() {
                                        if *p > pos {
                                            *p = (*p as i64 + len_diff).max(0) as usize;
                                        }
                                    }
                                    self.doc_find_match_count = self.doc_find_positions.len();
                                    if self.doc_find_current_match >= self.doc_find_positions.len() && !self.doc_find_positions.is_empty() {
                                        self.doc_find_current_match = 0;
                                    }
                                    self.notification = Some(format!(
                                        "Replaced match. {} remaining",
                                        self.doc_find_positions.len()
                                    ));
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
                                if self.doc_find_case_sensitive {
                                    doc.content = doc.content.replace(&self.doc_find_text, &self.doc_replace_text);
                                } else {
                                    // Case-insensitive replace
                                    let lower_content = doc.content.to_lowercase();
                                    let lower_find = self.doc_find_text.to_lowercase();
                                    let mut result = String::new();
                                    let mut last_end = 0;
                                    for (start, _) in lower_content.match_indices(&lower_find) {
                                        result.push_str(&doc.content[last_end..start]);
                                        result.push_str(&self.doc_replace_text);
                                        last_end = start + self.doc_find_text.len();
                                    }
                                    result.push_str(&doc.content[last_end..]);
                                    doc.content = result;
                                }
                                self.editor.load_document(doc);
                                self.editor.mark_dirty();
                                self.doc_find_match_count = 0;
                                self.notification = Some("All occurrences replaced".to_string());
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
                // Recount matches
                if !self.doc_find_text.is_empty() {
                    if let (Some(ref project), Some(item_id)) = (&self.project, self.selected_item) {
                        if let Some(item) = project.binder.find_item(&item_id) {
                            if let Some(ref doc) = item.document {
                                let content = if self.doc_find_case_sensitive {
                                    doc.content.clone()
                                } else {
                                    doc.content.to_lowercase()
                                };
                                let query = if self.doc_find_case_sensitive {
                                    self.doc_find_text.clone()
                                } else {
                                    self.doc_find_text.to_lowercase()
                                };
                                self.doc_find_match_count = content.matches(&query).count();
                            }
                        }
                    }
                }
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
                self.notification = Some(
                    if self.script_mode { "Script mode enabled".to_string() }
                    else { "Script mode disabled".to_string() }
                );
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

            // ========== Revision tracking ==========
            Message::SetRevisionLevel(level_name) => {
                self.current_revision = match level_name.as_str() {
                    "Revision 1" => Some(crate::core::script::RevisionLevel::First),
                    "Revision 2" => Some(crate::core::script::RevisionLevel::Second),
                    "Revision 3" => Some(crate::core::script::RevisionLevel::Third),
                    "Revision 4" => Some(crate::core::script::RevisionLevel::Fourth),
                    "Revision 5" => Some(crate::core::script::RevisionLevel::Fifth),
                    "None" | _ => None,
                };
            }

            // ========== Document links ==========
            Message::InsertDocLink(target_id) => {
                if let Some(ref project) = self.project {
                    if let Some(target_item) = project.binder.find_item(&target_id) {
                        let link_text = format!("[[{}]]", target_item.title);
                        self.editor.content.perform(
                            iced::widget::text_editor::Action::Edit(
                                iced::widget::text_editor::Edit::Paste(
                                    std::sync::Arc::new(link_text)
                                )
                            )
                        );
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
                self.editor.content.perform(
                    iced::widget::text_editor::Action::Edit(
                        iced::widget::text_editor::Edit::Paste(
                            std::sync::Arc::new(markup)
                        )
                    )
                );
                self.editor.mark_dirty();
            }

            Message::InsertBlockQuote => {
                self.editor.content.perform(
                    iced::widget::text_editor::Action::Edit(
                        iced::widget::text_editor::Edit::Paste(
                            std::sync::Arc::new("> ".to_string())
                        )
                    )
                );
                self.editor.mark_dirty();
            }

            Message::InsertFootnote => {
                self.footnote_counter += 1;
                let marker = format!("[^{}]", self.footnote_counter);
                self.editor.content.perform(
                    iced::widget::text_editor::Action::Edit(
                        iced::widget::text_editor::Edit::Paste(
                            std::sync::Arc::new(marker.clone())
                        )
                    )
                );
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
                self.editor.content.perform(
                    iced::widget::text_editor::Action::Edit(
                        iced::widget::text_editor::Edit::Paste(
                            std::sync::Arc::new("\n---\n".to_string())
                        )
                    )
                );
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
                self.notification = Some(
                    if self.composition_mode { "Composition mode enabled".to_string() }
                    else { "Composition mode disabled".to_string() }
                );
            }

            // ========== Copy special ==========
            Message::CopyAsMarkdown => {
                self.sync_editor_to_project();
                if let (Some(ref project), Some(item_id)) = (&self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item(&item_id) {
                        if let Some(ref doc) = item.document {
                            let md = format!("# {}\n\n{}", item.title, doc.content);
                            let _ = arboard::Clipboard::new()
                                .and_then(|mut cb| cb.set_text(md));
                            self.notification = Some("Copied as Markdown".to_string());
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
                                doc.content.split("\n\n")
                                    .map(|p| format!("<p>{}</p>", p.replace('\n', "<br>")))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            );
                            let _ = arboard::Clipboard::new()
                                .and_then(|mut cb| cb.set_text(html));
                            self.notification = Some("Copied as HTML".to_string());
                        }
                    }
                }
            }

            Message::CopyAsPlainText => {
                self.sync_editor_to_project();
                if let (Some(ref project), Some(item_id)) = (&self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item(&item_id) {
                        if let Some(ref doc) = item.document {
                            let _ = arboard::Clipboard::new()
                                .and_then(|mut cb| cb.set_text(doc.content.clone()));
                            self.notification = Some("Copied as plain text".to_string());
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
                        match crate::core::backup::BackupManager::create_backup(path) {
                            Ok(backup_path) => {
                                self.notification = Some(format!("Backup created: {:?}", backup_path.file_name().unwrap_or_default()));
                            }
                            Err(e) => {
                                self.notification = Some(format!("Backup error: {}", e));
                            }
                        }
                    } else {
                        self.notification = Some("Save the project first before creating a backup.".to_string());
                    }
                }
            }

            Message::RestoreBackup(path) => {
                if let Some(ref project) = self.project {
                    if let Some(ref proj_path) = project.path {
                        match crate::core::backup::BackupManager::restore_backup(&path, proj_path) {
                            Ok(_) => {
                                // Reload the project
                                match crate::core::project::Project::load(proj_path) {
                                    Ok(p) => {
                                        self.compile_options.title = p.title.clone();
                                        self.project_notes_text = p.project_notes.clone();
                                        self.project = Some(p);
                                        self.selected_item = None;
                                        self.editor = EditorState::new();
                                        self.notification = Some("Backup restored successfully".to_string());
                                    }
                                    Err(e) => {
                                        self.notification = Some(format!("Restore error: {}", e));
                                    }
                                }
                            }
                            Err(e) => {
                                self.notification = Some(format!("Restore error: {}", e));
                            }
                        }
                    }
                }
            }

            // ========== OPML Import ==========
            Message::ImportOpml => {
                if let Some(ref mut project) = self.project {
                    let home = dirs::home_dir().unwrap_or_default();
                    let import_dir = home.join("Scrinever Import");
                    if import_dir.exists() {
                        let mut count = 0;
                        if let Ok(entries) = std::fs::read_dir(&import_dir) {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                if path.is_file() {
                                    let ext = path.extension()
                                        .and_then(|e| e.to_str())
                                        .unwrap_or("")
                                        .to_lowercase();
                                    if ext == "opml" {
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
                                    } else if ext == "fountain" {
                                        if let Ok(content) = std::fs::read_to_string(&path) {
                                            let sections = crate::export::fountain::parse_fountain(&content);
                                            for (title, text) in sections {
                                                let mut item = BinderItem::new_text(&title);
                                                if let Some(ref mut doc) = item.document {
                                                    doc.content = text;
                                                }
                                                project.binder.draft.add_child(item);
                                                count += 1;
                                            }
                                        }
                                    } else if ext == "html" || ext == "htm" {
                                        if let Ok(content) = std::fs::read_to_string(&path) {
                                            let title = path.file_stem()
                                                .and_then(|s| s.to_str())
                                                .unwrap_or("Imported HTML")
                                                .to_string();
                                            // Strip HTML tags for plain text import
                                            let plain = strip_html_tags(&content);
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
                        }
                        if count > 0 {
                            self.notification = Some(format!("Imported {} item(s)", count));
                        } else {
                            self.notification = Some("No importable files found in ~/Scrinever Import/".to_string());
                        }
                    } else {
                        let _ = std::fs::create_dir_all(&import_dir);
                        self.notification = Some("Created ~/Scrinever Import/ — place files there and import again.".to_string());
                    }
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
                self.sync_editor_to_project();
                if let Some(ref project) = self.project {
                    let home = dirs::home_dir().unwrap_or_default();
                    let output_dir = home.join("Scrinever Output");
                    let _ = std::fs::create_dir_all(&output_dir);
                    let filename = format!("{}.opml", project.title.replace(' ', "_"));
                    let output_path = output_dir.join(&filename);

                    match crate::export::opml::export_opml(&project.binder, &project.title) {
                        Ok(opml_content) => {
                            match std::fs::write(&output_path, opml_content) {
                                Ok(_) => {
                                    self.notification = Some(format!("OPML exported to {:?}", output_path));
                                }
                                Err(e) => {
                                    self.notification = Some(format!("Export error: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            self.notification = Some(format!("OPML error: {}", e));
                        }
                    }
                }
            }

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
                            let mut opts = CompileOptions::default();
                            opts.title = item.title.clone();
                            let home = dirs::home_dir().unwrap_or_default();
                            let print_path = home.join("Scrinever Projects").join("print.pdf");
                            let _ = std::fs::create_dir_all(print_path.parent().unwrap());
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
                    let home = dirs::home_dir().unwrap_or_default();
                    let print_path = home.join("Scrinever Projects").join(format!("{}_print.pdf", project.title));
                    let _ = std::fs::create_dir_all(print_path.parent().unwrap());
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
                self.spell_check_results.retain(|r| r.word.to_lowercase() != word.to_lowercase());
                let _ = self.spell_checker.save_user_dictionary();
                self.notification = Some(format!("Added \"{}\" to dictionary", word));
            }

            Message::SpellCheckReplace(position, misspelled, replacement) => {
                self.sync_editor_to_project();
                if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item_mut(&item_id) {
                        if let Some(ref mut doc) = item.document {
                            // Find the misspelled word at or near the given position and replace it
                            if let Some(start) = doc.content[position..].find(&misspelled) {
                                let actual_pos = position + start;
                                let end_pos = actual_pos + misspelled.len();
                                doc.content = format!(
                                    "{}{}{}",
                                    &doc.content[..actual_pos],
                                    replacement,
                                    &doc.content[end_pos..]
                                );
                                // Reload editor with updated content
                                self.editor.load_document(doc);
                                self.notification = Some(format!(
                                    "Replaced \"{}\" with \"{}\"",
                                    misspelled, replacement
                                ));
                                // Remove this entry from results
                                self.spell_check_results.retain(|r| {
                                    !(r.word == misspelled && r.position == position)
                                });
                            }
                        }
                    }
                }
            }

            Message::SpellCheckRemoveWord(word) => {
                self.spell_checker.remove_from_dictionary(&word);
                let _ = self.spell_checker.save_user_dictionary();
                self.notification = Some(format!("Removed \"{}\" from user dictionary", word));
            }

            Message::SpellCheckClearDict => {
                self.spell_checker.clear_user_dictionary();
                let _ = self.spell_checker.save_user_dictionary();
                self.notification = Some("User dictionary cleared".to_string());
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
                        if !field_name.is_empty() && !item.metadata.custom_metadata.iter().any(|f| f.name == field_name) {
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

            // ========== Corkboard card color ==========
            Message::SetCardColor(id, color_name) => {
                if let Some(ref mut project) = self.project {
                    if let Some(item) = project.binder.find_item_mut(&id) {
                        let color = match color_name.as_str() {
                            "Red" => crate::core::metadata::LabelColor::Red,
                            "Orange" => crate::core::metadata::LabelColor::Orange,
                            "Yellow" => crate::core::metadata::LabelColor::Yellow,
                            "Green" => crate::core::metadata::LabelColor::Green,
                            "Blue" => crate::core::metadata::LabelColor::Blue,
                            "Purple" => crate::core::metadata::LabelColor::Purple,
                            _ => crate::core::metadata::LabelColor::Blue,
                        };
                        if let Some(ref mut label) = item.metadata.label {
                            label.color = color;
                        } else {
                            item.metadata.label = Some(crate::core::metadata::Label {
                                name: color_name,
                                color,
                            });
                        }
                    }
                }
            }

            // ========== Smart collection refresh ==========
            Message::RefreshSmartCollections => {
                if let Some(ref mut project) = self.project {
                    let mut updates = Vec::new();
                    for (i, coll) in project.collections.iter().enumerate() {
                        if let crate::core::collection::CollectionKind::Search { ref query, case_sensitive, whole_word } = coll.kind {
                            let options = crate::core::search::SearchOptions {
                                query: query.clone(),
                                case_sensitive,
                                whole_word,
                                regex: false,
                                search_titles: true,
                                search_content: true,
                                search_notes: false,
                                search_synopsis: true,
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

            Message::SettingsSetCompBgColor(color) => {
                if let Some(ref mut project) = self.project {
                    project.settings.composition_bg_color = Some(color);
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
                self.editor.content.perform(
                    iced::widget::text_editor::Action::Edit(
                        iced::widget::text_editor::Edit::Paste(
                            std::sync::Arc::new(format!("\n{}", prefix))
                        )
                    )
                );
                self.editor.mark_dirty();
            }

            Message::InsertTable(rows, cols) => {
                let mut table = String::new();
                // Header row
                table.push('|');
                for c in 0..cols {
                    table.push_str(&format!(" Column {} |", c + 1));
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
                self.editor.content.perform(
                    iced::widget::text_editor::Action::Edit(
                        iced::widget::text_editor::Edit::Paste(
                            std::sync::Arc::new(table)
                        )
                    )
                );
                self.editor.mark_dirty();
            }

            Message::InsertCodeBlock(lang) => {
                let block = if lang.is_empty() {
                    "\n```\n\n```\n".to_string()
                } else {
                    format!("\n```{}\n\n```\n", lang)
                };
                self.editor.content.perform(
                    iced::widget::text_editor::Action::Edit(
                        iced::widget::text_editor::Edit::Paste(
                            std::sync::Arc::new(block)
                        )
                    )
                );
                self.editor.mark_dirty();
            }

            Message::InsertPageBreak => {
                self.editor.content.perform(
                    iced::widget::text_editor::Action::Edit(
                        iced::widget::text_editor::Edit::Paste(
                            std::sync::Arc::new("\n\n---\n\n<!-- page break -->\n\n".to_string())
                        )
                    )
                );
                self.editor.mark_dirty();
            }

            Message::InsertComment => {
                self.editor.content.perform(
                    iced::widget::text_editor::Action::Edit(
                        iced::widget::text_editor::Edit::Paste(
                            std::sync::Arc::new("<!-- comment -->".to_string())
                        )
                    )
                );
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
                self.editor.content.perform(
                    iced::widget::text_editor::Action::Edit(
                        iced::widget::text_editor::Edit::Paste(
                            std::sync::Arc::new(text)
                        )
                    )
                );
                self.editor.mark_dirty();
            }

            Message::SmartPaste(text) => {
                // Clean up pasted text: normalize whitespace, fix smart quotes, etc.
                let cleaned = text
                    .replace('\u{201C}', "\"")  // left double quote
                    .replace('\u{201D}', "\"")  // right double quote
                    .replace('\u{2018}', "'")   // left single quote
                    .replace('\u{2019}', "'")   // right single quote
                    .replace('\u{2013}', "--")  // en dash
                    .replace('\u{2014}', "---") // em dash
                    .replace('\u{2026}', "...") // ellipsis
                    .replace("\r\n", "\n")       // Windows line endings
                    .replace('\r', "\n");        // Old Mac line endings
                self.editor.content.perform(
                    iced::widget::text_editor::Action::Edit(
                        iced::widget::text_editor::Edit::Paste(
                            std::sync::Arc::new(cleaned)
                        )
                    )
                );
                self.editor.mark_dirty();
            }

            Message::InsertLink => {
                let link_text = "[link text](url)";
                self.editor.content.perform(
                    iced::widget::text_editor::Action::Edit(
                        iced::widget::text_editor::Edit::Paste(
                            std::sync::Arc::new(link_text.to_string())
                        )
                    )
                );
                self.editor.mark_dirty();
            }

            Message::InsertImage => {
                let image_text = "![alt text](image_path)";
                self.editor.content.perform(
                    iced::widget::text_editor::Action::Edit(
                        iced::widget::text_editor::Edit::Paste(
                            std::sync::Arc::new(image_text.to_string())
                        )
                    )
                );
                self.editor.mark_dirty();
            }

            // ========== Document templates ==========
            Message::NewDocFromTemplate(template_id) => {
                if let Some(ref mut project) = self.project {
                    if let Some(template) = crate::core::doc_templates::find_template(&template_id) {
                        let item = template.create_item(&template.name);
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
                if let Some(ref project) = self.project {
                    let result = crate::core::validation::validate_project(&project.binder);
                    self.notification = Some(result.display());
                    self.validation_result = Some(result);
                }
            }

            // ========== Misc ==========
            Message::Tick => {
                // Auto-save
                if self.project.is_some() && self.editor.dirty {
                    self.auto_save_counter += 1;
                    let interval = self.project.as_ref()
                        .map(|p| p.settings.auto_save_seconds)
                        .unwrap_or(30);
                    if interval > 0 && self.auto_save_counter >= interval {
                        self.auto_save_counter = 0;
                        self.sync_editor_to_project();
                        if let Some(ref mut project) = self.project {
                            let home = dirs::home_dir().unwrap_or_default();
                            let save_dir = home.join("Scrinever Projects");
                            if project.save(&save_dir).is_ok() {
                                self.editor.mark_clean();
                                // Auto-backup on save (every 10th auto-save)
                                if let Some(ref path) = project.path {
                                    let _ = crate::core::backup::BackupManager::create_backup(path);
                                }
                            }
                        }
                    }
                }

                // Writing session timer
                if self.session_active {
                    let current_words = self.current_word_count();
                    let word_delta = current_words as i64 - self.session_start_word_count as i64;
                    self.session_stats.update(word_delta, self.session_stats.time_elapsed_seconds + 1);

                    // Record writing history every 60 seconds
                    if self.session_stats.time_elapsed_seconds % 60 == 0 {
                        if let Some(ref mut project) = self.project {
                            project.writing_history.record(current_words, 60);
                        }
                    }
                }

                // Writing focus timer tick
                if self.writing_timer.is_running() {
                    if self.writing_timer.tick() {
                        self.notification = Some("Timer completed! Great writing session!".to_string());
                    }
                }

                // Auto-refresh smart collections every 30 seconds when collections panel is open
                if self.bottom_panel == BottomPanel::Collections && self.auto_save_counter % 30 == 0 {
                    if let Some(ref mut project) = self.project {
                        for coll in &mut project.collections {
                            if let crate::core::collection::CollectionKind::Search { ref query, case_sensitive, whole_word } = coll.kind {
                                let options = crate::core::search::SearchOptions {
                                    query: query.clone(),
                                    case_sensitive,
                                    whole_word,
                                    regex: false,
                                    search_titles: true,
                                    search_content: true,
                                    search_notes: false,
                                    search_synopsis: true,
                                };
                                let results = crate::core::search::search_binder(&project.binder, &options);
                                let item_ids: Vec<Uuid> = results.iter().map(|r| r.item_id).collect();
                                coll.item_ids = item_ids;
                            }
                        }
                    }
                }
            }

            Message::DismissNotification => {
                self.notification = None;
            }

            Message::EscapePressed => {
                if self.composition_mode {
                    self.composition_mode = false;
                } else if self.fullscreen_editor {
                    self.fullscreen_editor = false;
                } else if self.show_project_stats {
                    self.show_project_stats = false;
                } else if self.show_compile_dialog {
                    self.show_compile_dialog = false;
                } else if self.show_settings_dialog {
                    self.show_settings_dialog = false;
                } else if self.bottom_panel != BottomPanel::None {
                    self.bottom_panel = BottomPanel::None;
                } else if self.notification.is_some() {
                    self.notification = None;
                }
            }
        }

        IcedTask::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        // Welcome screen
        if self.project.is_none() {
            return views::welcome_screen::view(&self.recent_projects);
        }

        let project = self.project.as_ref().unwrap();

        // Compile dialog (overlay)
        if self.show_compile_dialog {
            return views::compile_dialog::view(&self.compile_options, &self.compile_presets);
        }

        // Settings dialog (overlay)
        if self.show_settings_dialog {
            return views::settings_dialog::view(
                &project.settings,
                &project.title,
                self.script_mode,
                &self.auto_correction,
            );
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
                best_day_words: project.writing_history.best_day()
                    .map(|d| d.words_written)
                    .unwrap_or(0),
                total_time_hours: project.writing_history.total_time_seconds() as f64 / 3600.0,
                reading_time_minutes: stats.word_count as f64 / 250.0,
                speaking_time_minutes: stats.word_count as f64 / 150.0,
                target_words: project.settings.target_word_count,
                deadline: project.settings.target_deadline.clone(),
                days_remaining: project.settings.target_deadline.as_ref()
                    .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                    .map(|target_date| {
                        let today = chrono::Utc::now().date_naive();
                        (target_date - today).num_days()
                    }),
                words_per_day_needed: {
                    let days_remaining = project.settings.target_deadline.as_ref()
                        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                        .map(|target_date| {
                            let today = chrono::Utc::now().date_naive();
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
            let title = self.selected_item
                .and_then(|id| project.binder.find_item(&id))
                .map(|item| item.title.as_str())
                .unwrap_or("");
            let word_count = self.editor.document.word_count();
            let session_words = if self.session_active { self.session_stats.words_written } else { 0 };
            return views::editor_view::view_composition(&self.editor, title, word_count, session_words);
        }

        // Fullscreen editor mode
        if self.fullscreen_editor {
            let title = self.selected_item
                .and_then(|id| project.binder.find_item(&id))
                .map(|item| item.title.as_str())
                .unwrap_or("");
            return views::editor_view::view_fullscreen(&self.editor, title);
        }

        // Toolbar
        let toolbar = views::toolbar::view(
            &self.view_mode,
            self.show_inspector,
            self.fullscreen_editor,
            &self.bottom_panel,
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
                let title = self.selected_item
                    .and_then(|id| project.binder.find_item(&id))
                    .map(|item| item.title.as_str())
                    .unwrap_or("No document selected");

                // Check for split editor mode
                if let Some(split_id) = self.split_editor_item {
                    let secondary_content = project.binder.find_item(&split_id)
                        .and_then(|item| item.document.as_ref())
                        .map(|doc| doc.content.as_str())
                        .unwrap_or("");
                    let secondary_title = project.binder.find_item(&split_id)
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
            ViewMode::Outliner => {
                views::outliner_view::view(&project.binder.draft, &self.item_targets)
            }
            ViewMode::Scrivenings => {
                let (items, parent_title) = if let Some(id) = self.selected_item {
                    if let Some(item) = project.binder.find_item(&id) {
                        if item.kind == BinderItemKind::Folder {
                            let children: Vec<&BinderItem> = item.children.iter()
                                .filter(|c| c.document.is_some())
                                .collect();
                            (children, item.title.clone())
                        } else {
                            (vec![item], item.title.clone())
                        }
                    } else {
                        (vec![], "Select a folder".to_string())
                    }
                } else {
                    let children: Vec<&BinderItem> = project.binder.draft.children.iter()
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
                let snapshots = self.selected_item
                    .and_then(|id| project.binder.find_item(&id))
                    .map(|item| item.snapshots.as_slice())
                    .unwrap_or(&[]);
                let current_content = self.selected_item
                    .and_then(|id| project.binder.find_item(&id))
                    .and_then(|item| item.document.as_ref())
                    .map(|doc| doc.content.as_str())
                    .unwrap_or("");
                Some(views::snapshot_panel::view(snapshots, current_content, self.selected_snapshot))
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
                let history = self.project.as_ref()
                    .map(|p| &p.writing_history)
                    .cloned()
                    .unwrap_or_default();
                Some(views::history_panel::view(&history))
            }
            BottomPanel::TextStats => {
                let text_content = self.selected_item
                    .and_then(|id| {
                        self.project.as_ref()
                            .and_then(|p| p.binder.find_item(&id))
                            .and_then(|item| item.document.as_ref())
                            .map(|doc| doc.content.as_str())
                    })
                    .unwrap_or("");
                let analysis = crate::core::stats::TextAnalysis::from_text(text_content);
                Some(views::text_stats_panel::view(&analysis))
            }
            BottomPanel::NameGen => {
                Some(views::name_generator_panel::view(&self.generated_names))
            }
            BottomPanel::ProjectNotes => {
                Some(views::project_notes_panel::view(&self.project_notes_text))
            }
            BottomPanel::Collections => {
                let data = views::collections_panel::CollectionsData {
                    collections: project.collections.clone(),
                    selected_collection: self.selected_collection,
                    new_collection_name: self.new_collection_name.clone(),
                };
                Some(views::collections_panel::view(&data))
            }
            BottomPanel::Bookmarks => {
                Some(views::bookmarks_panel::view(&project.bookmarks))
            }
            BottomPanel::Annotations => {
                let annotations = self.selected_item
                    .and_then(|id| project.binder.find_item(&id))
                    .and_then(|item| item.document.as_ref())
                    .map(|doc| doc.annotations.as_slice())
                    .unwrap_or(&[]);
                Some(views::annotations_panel::view(annotations, &self.annotation_text))
            }
            BottomPanel::Targets => {
                let current_words = project.binder.total_word_count();
                let deadline = project.settings.target_deadline.clone().unwrap_or_default();
                let days_remaining = project.settings.target_deadline.as_ref()
                    .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                    .map(|target_date| {
                        let today = chrono::Utc::now().date_naive();
                        (target_date - today).num_days()
                    });
                let words_per_day_needed = match (project.settings.target_word_count, days_remaining) {
                    (Some(target), Some(days)) if days > 0 && target > current_words => {
                        Some((target - current_words) / days as usize)
                    }
                    _ => None,
                };
                let data = views::targets_panel::TargetsData {
                    project_target: project.settings.target_word_count,
                    current_words,
                    deadline,
                    session_target: self.session_goal,
                    session_words: self.session_stats.words_written,
                    days_remaining,
                    words_per_day_needed,
                };
                Some(views::targets_panel::view(&data))
            }
            BottomPanel::QuickRef => {
                if let Some(ref_id) = self.quick_ref_item {
                    self.project.as_ref()
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
                    whole_word: false,
                    use_regex: false,
                };
                Some(views::find_replace_panel::view(&data))
            }
            BottomPanel::WritingGoals => {
                let words_today = if self.session_active {
                    self.session_stats.words_written
                } else {
                    project.writing_history.entries.last()
                        .filter(|e| e.date == chrono::Utc::now().date_naive())
                        .map(|e| e.words_written)
                        .unwrap_or(0)
                };
                let words_this_week: i64 = project.writing_history.recent(7)
                    .iter()
                    .map(|e| e.words_written)
                    .sum();
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
                let current_title = self.selected_item
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
                                        });
                                    }
                                }
                                crate::core::links::LinkStatus::Broken => {
                                    let suggestions = crate::core::links::suggest_link_targets(
                                        &v.link.link_text,
                                        &project.binder,
                                    );
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
                        if other_item.id == item_id { continue; }
                        if let Some(ref doc) = other_item.document {
                            let links = crate::core::links::extract_links(&doc.content);
                            for link in &links {
                                if link.link_text == current_title {
                                    incoming.push(views::doc_links_panel::DocLink {
                                        target_title: other_item.title.clone(),
                                        target_id: other_item.id,
                                        link_text: format!("[[{}]]", current_title),
                                    });
                                    break; // one entry per source document
                                }
                            }
                        }
                    }
                }

                // Available docs for quick insertion
                let available: Vec<(uuid::Uuid, String)> = project.binder.all_items()
                    .iter()
                    .filter(|i| i.document.is_some() && Some(i.id) != self.selected_item)
                    .map(|i| (i.id, i.title.clone()))
                    .collect();

                Some(views::doc_links_panel::view(&outgoing, &incoming, &broken, &available))
            }
            BottomPanel::Backups => {
                let backups = crate::core::backup::BackupManager::list_backups(
                    &project.title.replace(' ', "_")
                ).unwrap_or_default();
                Some(views::backup_panel::view(&backups, &project.title))
            }
            BottomPanel::SpellCheck => {
                Some(views::spell_check_panel::view(
                    &self.spell_check_results,
                    self.spell_checker.active,
                    self.spell_checker.dictionary_size(),
                    self.spell_checker.user_words(),
                ))
            }
            BottomPanel::Timer => {
                Some(views::timer_panel::view(&self.writing_timer))
            }
            BottomPanel::Validation => {
                Some(views::validation_panel::view(self.validation_result.as_ref()))
            }
            BottomPanel::Templates => {
                let templates = crate::core::doc_templates::builtin_templates();
                Some(views::templates_panel::view(&templates))
            }
            BottomPanel::None => None,
        };

        // Status bar
        let stats = Statistics::from_binder(&project.binder);
        let status_bar = views::status_bar::view(
            &stats,
            project.settings.target_word_count,
            self.editor.dirty,
            &project.title,
            self.session_active,
        );

        // Notification bar
        let notification_bar: Option<Element<'_, Message>> = self.notification.as_ref().map(|msg| {
            container(
                row![
                    text(msg.clone()).size(12).color(Theme::WARNING),
                    iced::widget::Space::with_width(Length::Fill),
                    iced::widget::button(
                        text("x").size(12).color(Theme::TEXT_MUTED),
                    ).on_press(Message::DismissNotification).padding(Padding::from([2, 8])),
                ]
                .padding(Padding::from([4, 12]))
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

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Keyboard shortcuts and auto-save timer
    pub fn subscription(&self) -> Subscription<Message> {
        let key_sub = keyboard::on_key_press(|key, modifiers| {
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
                            "h" => Some(Message::ShowBottomPanel(BottomPanel::FindReplace)),
                            "d" => Some(Message::ShowBottomPanel(BottomPanel::Annotations)),
                            "g" => Some(Message::ShowBottomPanel(BottomPanel::WritingGoals)),
                            "t" => Some(Message::TransposeChars),
                            "/" => Some(Message::ToggleComment),
                            "1" => Some(Message::SwitchView(ViewMode::Editor)),
                            "2" => Some(Message::SwitchView(ViewMode::Corkboard)),
                            "3" => Some(Message::SwitchView(ViewMode::Outliner)),
                            "4" => Some(Message::SwitchView(ViewMode::Scrivenings)),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            } else if alt {
                match key {
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                        Some(Message::MoveLineUp)
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                        Some(Message::MoveLineDown)
                    }
                    _ => None,
                }
            } else {
                match key {
                    keyboard::Key::Named(keyboard::key::Named::Escape) => {
                        Some(Message::EscapePressed)
                    }
                    keyboard::Key::Named(keyboard::key::Named::F11) => {
                        Some(Message::ToggleFullscreen)
                    }
                    keyboard::Key::Named(keyboard::key::Named::F5) => {
                        Some(Message::ToggleCompositionMode)
                    }
                    keyboard::Key::Named(keyboard::key::Named::F7) => {
                        Some(Message::RunSpellCheck)
                    }
                    _ => None,
                }
            }
        });

        let tick_sub = iced::time::every(std::time::Duration::from_secs(1))
            .map(|_| Message::Tick);

        Subscription::batch([key_sub, tick_sub])
    }

    /// Dark theme
    pub fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}

/// Convert text to Title Case
fn title_case(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    let rest: String = chars.collect();
                    format!("{}{}", upper, rest.to_lowercase())
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Strip HTML tags from content for plain text import
fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_script = false;

    let lower = html.to_lowercase();
    let chars: Vec<char> = html.chars().collect();
    let lower_chars: Vec<char> = lower.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        if !in_tag && i + 7 < lower_chars.len() {
            let slice: String = lower_chars[i..i + 7].iter().collect();
            if slice == "<script" {
                in_script = true;
            }
        }
        if in_script && i + 8 < lower_chars.len() {
            let slice: String = lower_chars[i..i + 9].iter().collect();
            if slice == "</script>" {
                in_script = false;
                i += 9;
                continue;
            }
        }

        if in_script {
            i += 1;
            continue;
        }

        if chars[i] == '<' {
            in_tag = true;
            // Convert block elements to newlines
            if i + 2 < lower_chars.len() {
                let next_two: String = lower_chars[i + 1..i + 3.min(lower_chars.len())].iter().collect();
                if next_two.starts_with('p') || next_two.starts_with('b') || next_two.starts_with('h')
                    || next_two.starts_with('l') || next_two.starts_with('d')
                    || next_two.starts_with('t')
                {
                    result.push('\n');
                }
            }
        } else if chars[i] == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(chars[i]);
        }
        i += 1;
    }

    // Clean up excessive newlines
    let mut cleaned = String::new();
    let mut prev_was_newline = false;
    for ch in result.chars() {
        if ch == '\n' {
            if !prev_was_newline {
                cleaned.push('\n');
            }
            prev_was_newline = true;
        } else {
            prev_was_newline = false;
            cleaned.push(ch);
        }
    }

    // Unescape common HTML entities
    cleaned = cleaned.replace("&amp;", "&");
    cleaned = cleaned.replace("&lt;", "<");
    cleaned = cleaned.replace("&gt;", ">");
    cleaned = cleaned.replace("&quot;", "\"");
    cleaned = cleaned.replace("&#39;", "'");
    cleaned = cleaned.replace("&nbsp;", " ");

    cleaned.trim().to_string()
}

/// Strip LaTeX commands from content for plain text import
fn strip_latex_commands(latex: &str) -> String {
    let mut result = String::new();
    let mut i = 0;
    let chars: Vec<char> = latex.chars().collect();

    while i < chars.len() {
        if chars[i] == '\\' {
            // Skip the command name
            i += 1;
            // Check for \begin{...} and \end{...} — skip the whole thing
            let mut cmd = String::new();
            while i < chars.len() && chars[i].is_alphanumeric() {
                cmd.push(chars[i]);
                i += 1;
            }
            // Convert some commands to their content
            match cmd.as_str() {
                "section" | "subsection" | "subsubsection" | "chapter" | "part" => {
                    result.push('\n');
                    result.push('\n');
                    // Skip the {title} — extract the text inside
                    if i < chars.len() && chars[i] == '{' {
                        i += 1; // skip {
                        let mut depth = 1;
                        while i < chars.len() && depth > 0 {
                            if chars[i] == '{' { depth += 1; }
                            else if chars[i] == '}' { depth -= 1; }
                            if depth > 0 { result.push(chars[i]); }
                            i += 1;
                        }
                    }
                    result.push('\n');
                }
                "textbf" | "textit" | "emph" | "underline" | "textsf" | "texttt" => {
                    // Extract content from braces
                    if i < chars.len() && chars[i] == '{' {
                        i += 1;
                        let mut depth = 1;
                        while i < chars.len() && depth > 0 {
                            if chars[i] == '{' { depth += 1; }
                            else if chars[i] == '}' { depth -= 1; }
                            if depth > 0 { result.push(chars[i]); }
                            i += 1;
                        }
                    }
                }
                "begin" | "end" => {
                    // Skip {environment}
                    if i < chars.len() && chars[i] == '{' {
                        i += 1;
                        let mut env = String::new();
                        while i < chars.len() && chars[i] != '}' {
                            env.push(chars[i]);
                            i += 1;
                        }
                        if i < chars.len() { i += 1; } // skip }
                        if env == "itemize" || env == "enumerate" || env == "description" {
                            result.push('\n');
                        }
                    }
                }
                "item" => {
                    result.push('\n');
                    result.push_str("  - ");
                }
                "par" | "newline" | "linebreak" => {
                    result.push('\n');
                }
                _ => {
                    // Skip unknown commands and their optional/required args
                    if i < chars.len() && chars[i] == '{' {
                        let mut depth = 1;
                        i += 1;
                        while i < chars.len() && depth > 0 {
                            if chars[i] == '{' { depth += 1; }
                            else if chars[i] == '}' { depth -= 1; }
                            i += 1;
                        }
                    }
                }
            }
        } else if chars[i] == '{' || chars[i] == '}' {
            // Skip bare braces
            i += 1;
        } else if chars[i] == '%' {
            // Skip LaTeX comments
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if chars[i] == '$' {
            // Skip math mode
            i += 1;
            if i < chars.len() && chars[i] == '$' {
                // Display math $$...$$
                i += 1;
                while i + 1 < chars.len() && !(chars[i] == '$' && chars[i + 1] == '$') {
                    result.push(chars[i]);
                    i += 1;
                }
                if i + 1 < chars.len() { i += 2; }
            } else {
                // Inline math $...$
                while i < chars.len() && chars[i] != '$' {
                    result.push(chars[i]);
                    i += 1;
                }
                if i < chars.len() { i += 1; }
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    // Clean up multiple blank lines
    let mut cleaned = String::new();
    let mut blank_count = 0;
    for line in result.lines() {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 2 {
                cleaned.push('\n');
            }
        } else {
            blank_count = 0;
            cleaned.push_str(line);
            cleaned.push('\n');
        }
    }

    cleaned.trim().to_string()
}
