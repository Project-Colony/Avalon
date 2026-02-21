use iced::keyboard;
use iced::widget::{column, container, row, text, text_editor};
use iced::{Element, Length, Padding, Subscription, Task as IcedTask};
use uuid::Uuid;

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

    // Misc
    Tick,
    DismissNotification,
    EscapePressed,
}

impl ScrineverApp {
    pub fn new() -> (Self, IcedTask<Message>) {
        let mut spell_checker = SpellChecker::new();
        spell_checker.try_init();

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
        };

        (app, IcedTask::none())
    }

    pub fn title(&self) -> String {
        let dirty = if self.editor.dirty { " *" } else { "" };
        match &self.project {
            Some(p) => format!("Scrinever - {}{}", p.title, dirty),
            None => "Scrinever".to_string(),
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
                                    if ext == "txt" || ext == "md" || ext == "markdown" || ext == "rtf" {
                                        if let Ok(content) = std::fs::read_to_string(&path) {
                                            let title = path.file_stem()
                                                .and_then(|s| s.to_str())
                                                .unwrap_or("Imported")
                                                .to_string();
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
                        }
                        if count > 0 {
                            self.notification = Some(format!("Imported {} file(s) from ~/Scrinever Import/", count));
                        } else {
                            self.notification = Some("No .txt/.md files found in ~/Scrinever Import/".to_string());
                        }
                    } else {
                        let _ = std::fs::create_dir_all(&import_dir);
                        self.notification = Some("Created ~/Scrinever Import/ — place .txt or .md files there and import again.".to_string());
                    }
                } else {
                    self.notification = Some("Create or open a project first.".to_string());
                }
            }

            // ========== Name generator ==========
            Message::GenerateName(kind) => {
                use crate::core::namegen::NameGenerator;
                self.name_gen_type = kind.clone();
                let name = match kind.as_str() {
                    "male" => NameGenerator::male_name(),
                    "female" => NameGenerator::female_name(),
                    "fantasy" => NameGenerator::fantasy_name(),
                    "place" => NameGenerator::place_name(),
                    _ => NameGenerator::male_name(),
                };
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
            }

            Message::DismissNotification => {
                self.notification = None;
            }

            Message::EscapePressed => {
                if self.fullscreen_editor {
                    self.fullscreen_editor = false;
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
            return views::welcome_screen::view();
        }

        let project = self.project.as_ref().unwrap();

        // Compile dialog (overlay)
        if self.show_compile_dialog {
            return views::compile_dialog::view(&self.compile_options);
        }

        // Settings dialog (overlay)
        if self.show_settings_dialog {
            return views::settings_dialog::view(&project.settings, &project.title);
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
                views::editor_view::view(&self.editor, title)
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
        };

        // Inspector (right panel)
        let inspector = if self.show_inspector {
            self.selected_item
                .and_then(|id| project.binder.find_item(&id))
                .map(|item| {
                    let data = views::inspector_view::InspectorData::from_item(
                        item,
                        &self.notes_text,
                        self.item_targets.get(&item.id).copied(),
                        &project.settings,
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
                Some(views::snapshot_panel::view(snapshots))
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
            BottomPanel::None => None,
        };

        // Status bar
        let stats = Statistics::from_binder(&project.binder);
        let status_bar = views::status_bar::view(
            &stats,
            project.settings.target_word_count,
            self.editor.dirty,
            &project.title,
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
            if modifiers.control() || modifiers.command() {
                match key {
                    keyboard::Key::Character(c) => {
                        let c = c.as_str();
                        match c {
                            "s" => Some(Message::SaveProject),
                            "n" => Some(Message::NewDocument),
                            "f" => Some(Message::ShowBottomPanel(BottomPanel::Search)),
                            "e" => Some(Message::ShowCompileDialog),
                            "i" => Some(Message::ToggleInspector),
                            "," => Some(Message::ShowSettings),
                            _ => None,
                        }
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
