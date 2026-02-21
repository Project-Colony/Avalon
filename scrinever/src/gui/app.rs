use iced::widget::{column, container, row, text_editor};
use iced::{Element, Length, Task as IcedTask};
use uuid::Uuid;

use crate::core::binder::{BinderItem, BinderItemKind};
use crate::core::project::Project;
use crate::core::stats::Statistics;
use crate::editor::EditorState;
use crate::export::compiler::{CompileOptions, OutputFormat};
use crate::spelling::SpellChecker;
use crate::thesaurus::Thesaurus;

use super::views;

/// The active view mode
#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Editor,
    Corkboard,
    Outliner,
}

/// Application state
pub struct ScrineverApp {
    /// The current project (None = welcome screen)
    pub project: Option<Project>,
    /// The currently selected binder item
    pub selected_item: Option<Uuid>,
    /// The editor state
    pub editor: EditorState,
    /// Current view mode
    pub view_mode: ViewMode,
    /// Whether the inspector panel is visible
    pub show_inspector: bool,
    /// Compile dialog state
    pub show_compile_dialog: bool,
    pub compile_options: CompileOptions,
    /// Spell checker
    pub spell_checker: SpellChecker,
    /// Thesaurus
    pub thesaurus: Thesaurus,
    /// Notification message
    pub notification: Option<String>,
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

    // Editor operations
    EditorAction(text_editor::Action),

    // View operations
    SwitchView(ViewMode),
    ToggleInspector,

    // Compile operations
    ShowCompileDialog,
    HideCompileDialog,
    CompileSetFormat(String),
    CompileSetTitle(String),
    CompileSetAuthor(String),
    CompileSetFrontMatter(bool),
    CompileSetMarkedOnly(bool),
    DoCompile,

    // Snapshot operations
    CreateSnapshot,
    RestoreSnapshot(usize),

    // Misc
    Tick,
    DismissNotification,
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
            show_compile_dialog: false,
            compile_options: CompileOptions::default(),
            spell_checker,
            thesaurus: Thesaurus::new(),
            notification: None,
        };

        (app, IcedTask::none())
    }

    pub fn title(&self) -> String {
        match &self.project {
            Some(p) => format!("Scrinever - {}", p.title),
            None => "Scrinever".to_string(),
        }
    }

    pub fn update(&mut self, message: Message) -> IcedTask<Message> {
        match message {
            Message::NewProject => {
                self.project = Some(Project::new("Untitled Project"));
                self.selected_item = None;
                self.editor = EditorState::new();
                if let Some(ref p) = self.project {
                    self.compile_options.title = p.title.clone();
                }
            }

            Message::NewFromTemplate(template_id) => {
                self.project = Some(Project::from_template("Untitled Project", &template_id));
                self.selected_item = None;
                self.editor = EditorState::new();
                if let Some(ref p) = self.project {
                    self.compile_options.title = p.title.clone();
                }
            }

            Message::OpenProject => {
                // In a real app, we'd use a file dialog here
                // For now, show a notification
                self.notification = Some("File dialog not yet implemented. Use project files directly.".to_string());
            }

            Message::SaveProject => {
                if let Some(ref mut project) = self.project {
                    // Sync editor content back to document
                    if let Some(item_id) = self.selected_item {
                        if let Some(item) = project.binder.find_item_mut(&item_id) {
                            if let Some(ref mut doc) = item.document {
                                doc.content = self.editor.text();
                            }
                        }
                    }

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
                    self.project = Some(p);
                    self.selected_item = None;
                    self.editor = EditorState::new();
                }
            }

            Message::SelectBinderItem(id) => {
                // Save current editor content before switching
                if let (Some(ref mut project), Some(prev_id)) = (&mut self.project, self.selected_item) {
                    if let Some(item) = project.binder.find_item_mut(&prev_id) {
                        if let Some(ref mut doc) = item.document {
                            doc.content = self.editor.text();
                        }
                    }
                }

                self.selected_item = Some(id);

                // Load the selected document into the editor
                if let Some(ref project) = self.project {
                    if let Some(item) = project.binder.find_item(&id) {
                        if let Some(ref doc) = item.document {
                            self.editor.load_document(doc);
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
            }

            Message::NewDocument => {
                if let Some(ref mut project) = self.project {
                    let new_item = BinderItem::new_text("New Document");
                    let new_id = new_item.id;
                    project.binder.draft.add_child(new_item);
                    self.selected_item = Some(new_id);

                    if let Some(item) = project.binder.find_item(&new_id) {
                        if let Some(ref doc) = item.document {
                            self.editor.load_document(doc);
                        }
                    }
                }
            }

            Message::NewFolder => {
                if let Some(ref mut project) = self.project {
                    let new_folder = BinderItem::new_folder("New Folder");
                    project.binder.draft.add_child(new_folder);
                }
            }

            Message::DeleteItem(id) => {
                if let Some(ref mut project) = self.project {
                    project.binder.move_to_trash(&id);
                    if self.selected_item == Some(id) {
                        self.selected_item = None;
                        self.editor = EditorState::new();
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

            Message::EditorAction(action) => {
                let is_edit = action.is_edit();
                self.editor.content.perform(action);
                if is_edit {
                    self.editor.mark_dirty();
                }
            }

            Message::SwitchView(mode) => {
                self.view_mode = mode;
            }

            Message::ToggleInspector => {
                self.show_inspector = !self.show_inspector;
            }

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

            Message::DoCompile => {
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

            Message::CreateSnapshot => {
                if let (Some(ref mut project), Some(item_id)) = (&mut self.project, self.selected_item) {
                    // First sync editor content
                    if let Some(item) = project.binder.find_item_mut(&item_id) {
                        if let Some(ref mut doc) = item.document {
                            doc.content = self.editor.text();
                        }
                    }
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

            Message::Tick => {
                // Auto-save timer, etc.
            }

            Message::DismissNotification => {
                self.notification = None;
            }
        }

        IcedTask::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        // If no project is loaded, show welcome screen
        if self.project.is_none() {
            return views::welcome_screen::view();
        }

        let project = self.project.as_ref().unwrap();

        // If compile dialog is visible, show it
        if self.show_compile_dialog {
            return views::compile_dialog::view(&self.compile_options);
        }

        // Toolbar
        let toolbar = views::toolbar::view(&self.view_mode, self.show_inspector);

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
                views::outliner_view::view(&project.binder.draft)
            }
        };

        // Inspector (right panel)
        let inspector = if self.show_inspector {
            self.selected_item
                .and_then(|id| project.binder.find_item(&id))
                .map(|item| {
                    let data = views::inspector_view::InspectorData::from_item(item);
                    views::inspector_view::view(data)
                })
        } else {
            None
        };

        // Status bar
        let stats = Statistics::from_binder(&project.binder);
        let status_bar = views::status_bar::view(
            &stats,
            project.settings.target_word_count,
            self.editor.dirty,
            &project.title,
        );

        // Layout: toolbar on top, then [binder | content | inspector], then status bar
        let mut main_row = row![binder, main_content];
        if let Some(insp) = inspector {
            main_row = main_row.push(insp);
        }

        let layout = column![
            toolbar,
            main_row,
            status_bar,
        ];

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
