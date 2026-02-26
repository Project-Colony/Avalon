use iced::widget::text_editor;
use iced::window;
use uuid::Uuid;

use crate::core::project::Project;

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
    WritingPrompts,
}

/// Which toolbar dropdown menu is open
#[derive(Debug, Clone, PartialEq)]
pub enum ToolbarMenu {
    File,
    View,
    Panels,
    Tools,
}

/// Active tab in the Settings window
///
/// Inspired by Scrivener (7 panes), Word (10 categories), Ulysses (7 tabs):
/// - General: project identity, saving, labels/statuses
/// - Editor: typing behavior, navigation, composition, script mode
/// - Corrections: proofing, auto-correct substitutions, spell check
/// - Appearance: font, layout, display toggles, UI scale
/// - Backup: auto-backup, frequency, accessibility
/// - Shortcuts: complete keyboard reference
#[derive(Debug, Clone, PartialEq)]
pub enum SettingsTab {
    General,
    Editor,
    Corrections,
    Appearance,
    Backup,
    Shortcuts,
}

/// Messages for the application
#[derive(Debug, Clone)]
#[allow(dead_code)] // Some variants are dispatched by UI views
pub enum Message {
    // Project operations
    NewProject,
    NewFromTemplate(String),
    OpenProject,
    SaveProject,
    ProjectLoaded(Box<Option<Project>>),

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

    // Window management
    OpenSettingsWindow,
    CloseSettingsWindow,
    OpenAboutWindow,
    CloseAboutWindow,
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    MainWindowClosed,

    // Settings
    ShowSettings,
    HideSettings,
    SettingsChangeTab(SettingsTab),
    SettingsSetProjectTitle(String),
    SettingsSetFont(String),
    SettingsSetFontSize(String),
    SettingsZoomIn,
    SettingsZoomOut,
    SettingsSetTarget(String),
    SettingsSetAutoSave(String),
    SettingsToggleWordCount(bool),
    SettingsSetLineSpacing(String),
    SettingsSetLineSpacingPreset(String),
    SettingsSetEditorWidth(String),
    SettingsToggleSpellCheck(bool),
    SettingsToggleTypewriterScroll(bool),
    SettingsToggleShowParagraphMarks(bool),
    SettingsToggleHighContrast(bool),
    SettingsToggleLargeUI(bool),
    SettingsToggleReduceMotion(bool),
    SettingsToggleScreenReaderHints(bool),
    SettingsSetUIScale(String),
    SettingsToggleAutoBackup(bool),
    SettingsSetBackupInterval(String),
    SettingsToggleSmartPunctuation(bool),
    SettingsSetDefaultDocType(String),
    SettingsToggleShowSynopsis(bool),
    SettingsToggleAutoNumbering(bool),

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
    DocFindToggleWholeWord,
    DocFindToggleRegex,

    // Quick reference
    ShowQuickRef(Uuid),

    // Script mode
    ToggleScriptMode,
    SetScriptElement(String),

    // Auto-correction
    ToggleAutoCorrectSmartQuotes,
    ToggleAutoCorrectEmDashes,
    ToggleAutoCorrectEllipsis,

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

    // Smart collection refresh
    RefreshSmartCollections,

    // Composition mode settings
    SettingsSetCompWidth(String),

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

    // Writing prompts
    GenerateWritingPrompt,
    GenerateWritingPromptCategory(String),
    GenerateCharacter,
    GeneratePlotSeed,
    GenerateWritingNames,

    // Toolbar menus
    ToggleToolbarMenu(ToolbarMenu),
    CloseToolbarMenu,

    // Outliner interactions
    OutlinerToggleExpand(Uuid),

    // Validation auto-fix
    AutoFixValidation,

    // Misc
    Tick,
    DismissNotification,
    EscapePressed,
}
