use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use anyhow::{Context, Result};

use super::binder::Binder;
use super::bookmark::BookmarkList;
use super::collection::Collection;
use super::document::Document;
use super::history::WritingHistory;
use super::snapshot::Snapshot;
use super::metadata::ProjectSettings;
use crate::export::compiler::CompileOptions;

/// A Scrinever project — the top-level container for all writing data.
/// Stored as a directory with structured JSON files inside.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub binder: Binder,
    pub settings: ProjectSettings,
    /// Project-level notes (scratch pad)
    #[serde(default)]
    pub project_notes: String,
    /// Collections (manual lists and saved searches)
    #[serde(default)]
    pub collections: Vec<Collection>,
    /// Writing history (daily word counts)
    #[serde(default)]
    pub writing_history: WritingHistory,
    /// Bookmarks / favorites
    #[serde(default)]
    pub bookmarks: BookmarkList,
    /// Compile presets (name -> options)
    #[serde(default)]
    pub compile_presets: Vec<(String, CompileOptions)>,
    #[serde(skip)]
    pub path: Option<PathBuf>,
}

impl Project {
    /// Create a new empty project
    pub fn new(title: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.to_string(),
            created_at: now,
            modified_at: now,
            binder: Binder::default_structure(),
            settings: ProjectSettings::default(),
            project_notes: String::new(),
            collections: Vec::new(),
            writing_history: WritingHistory::new(),
            bookmarks: BookmarkList::new(),
            compile_presets: Vec::new(),
            path: None,
        }
    }

    /// Create a project from a template
    pub fn from_template(title: &str, template_name: &str) -> Self {
        let mut project = Self::new(title);
        project.apply_template(template_name);
        project
    }

    /// Save the project to disk
    pub fn save(&mut self, base_path: &Path) -> Result<()> {
        let project_dir = base_path.join(format!("{}.scriv", self.title));
        fs::create_dir_all(&project_dir)
            .context("Failed to create project directory")?;

        // Save project metadata
        let meta_path = project_dir.join("project.json");
        self.modified_at = Utc::now();
        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize project")?;
        fs::write(&meta_path, json)
            .context("Failed to write project metadata")?;

        // Save documents
        let docs_dir = project_dir.join("docs");
        fs::create_dir_all(&docs_dir)?;
        self.save_documents(&docs_dir)?;

        // Save snapshots
        let snaps_dir = project_dir.join("snapshots");
        fs::create_dir_all(&snaps_dir)?;
        self.save_snapshots(&snaps_dir)?;

        // Save compile presets (if any)
        let presets_path = project_dir.join("compile_presets.json");
        if let Ok(json) = serde_json::to_string_pretty(&self.compile_presets) {
            let _ = fs::write(&presets_path, json);
        }

        self.path = Some(project_dir);
        Ok(())
    }

    /// Load a project from disk
    pub fn load(project_dir: &Path) -> Result<Self> {
        let meta_path = project_dir.join("project.json");
        let json = fs::read_to_string(&meta_path)
            .context("Failed to read project metadata")?;
        let mut project: Project = serde_json::from_str(&json)
            .context("Failed to parse project metadata")?;

        // Load documents
        let docs_dir = project_dir.join("docs");
        if docs_dir.exists() {
            project.load_documents(&docs_dir)?;
        }

        // Load snapshots
        let snaps_dir = project_dir.join("snapshots");
        if snaps_dir.exists() {
            project.load_snapshots(&snaps_dir)?;
        }

        // Load compile presets
        let presets_path = project_dir.join("compile_presets.json");
        if presets_path.exists() {
            if let Ok(json) = fs::read_to_string(&presets_path) {
                if let Ok(presets) = serde_json::from_str(&json) {
                    project.compile_presets = presets;
                }
            }
        }

        project.path = Some(project_dir.to_path_buf());
        Ok(project)
    }

    /// Save all documents to disk
    fn save_documents(&self, docs_dir: &Path) -> Result<()> {
        for item in self.binder.all_items() {
            if let Some(doc) = &item.document {
                let doc_path = docs_dir.join(format!("{}.json", item.id));
                let json = serde_json::to_string_pretty(doc)?;
                fs::write(doc_path, json)?;
            }
        }
        Ok(())
    }

    /// Load all documents from disk
    fn load_documents(&mut self, docs_dir: &Path) -> Result<()> {
        for item in self.binder.all_items_mut() {
            let doc_path = docs_dir.join(format!("{}.json", item.id));
            if doc_path.exists() {
                let json = fs::read_to_string(&doc_path)?;
                let doc: Document = serde_json::from_str(&json)?;
                item.document = Some(doc);
            }
        }
        Ok(())
    }

    /// Save all snapshots to disk (one file per binder item that has snapshots)
    fn save_snapshots(&self, snaps_dir: &Path) -> Result<()> {
        for item in self.binder.all_items() {
            if !item.snapshots.is_empty() {
                let snap_path = snaps_dir.join(format!("{}.json", item.id));
                let json = serde_json::to_string_pretty(&item.snapshots)?;
                fs::write(snap_path, json)?;
            }
        }
        Ok(())
    }

    /// Load all snapshots from disk
    fn load_snapshots(&mut self, snaps_dir: &Path) -> Result<()> {
        for item in self.binder.all_items_mut() {
            let snap_path = snaps_dir.join(format!("{}.json", item.id));
            if snap_path.exists() {
                let json = fs::read_to_string(&snap_path)?;
                let snapshots: Vec<Snapshot> = serde_json::from_str(&json)?;
                item.snapshots = snapshots;
            }
        }
        Ok(())
    }

    /// Apply a template to the project structure
    fn apply_template(&mut self, template_name: &str) {
        use super::binder::BinderItem;

        match template_name {
            "novel" => {
                let manuscript = BinderItem::new_folder("Manuscript");
                let chapter1 = BinderItem::new_folder("Chapter 1");
                let scene1 = BinderItem::new_text("Scene 1");
                self.binder = Binder {
                    draft: {
                        let mut m = manuscript;
                        let mut c1 = chapter1;
                        c1.children.push(scene1);
                        m.children.push(c1);
                        m
                    },
                    research: {
                        let mut r = BinderItem::new_folder("Research");
                        r.children.push(BinderItem::new_folder("Characters"));
                        r.children.push(BinderItem::new_folder("Places"));
                        r.children.push(BinderItem::new_text("Notes"));
                        r
                    },
                    trash: BinderItem::new_folder("Trash"),
                };
            }
            "screenplay" => {
                let mut draft = BinderItem::new_folder("Screenplay");
                draft.children.push(BinderItem::new_text("Title Page"));
                draft.children.push(BinderItem::new_text("Act I"));
                draft.children.push(BinderItem::new_text("Act II"));
                draft.children.push(BinderItem::new_text("Act III"));

                let mut research = BinderItem::new_folder("Research");
                research.children.push(BinderItem::new_folder("Characters"));
                research.children.push(BinderItem::new_folder("Locations"));
                research.children.push(BinderItem::new_text("Treatment"));

                self.binder = Binder {
                    draft,
                    research,
                    trash: BinderItem::new_folder("Trash"),
                };
            }
            "academic" => {
                let mut draft = BinderItem::new_folder("Paper");
                draft.children.push(BinderItem::new_text("Abstract"));
                draft.children.push(BinderItem::new_text("Introduction"));
                draft.children.push(BinderItem::new_text("Literature Review"));
                draft.children.push(BinderItem::new_text("Methodology"));
                draft.children.push(BinderItem::new_text("Results"));
                draft.children.push(BinderItem::new_text("Discussion"));
                draft.children.push(BinderItem::new_text("Conclusion"));
                draft.children.push(BinderItem::new_text("References"));

                let mut research = BinderItem::new_folder("Research");
                research.children.push(BinderItem::new_folder("Sources"));
                research.children.push(BinderItem::new_text("Notes"));

                self.binder = Binder {
                    draft,
                    research,
                    trash: BinderItem::new_folder("Trash"),
                };
            }
            "novel_with_parts" => {
                let mut draft = BinderItem::new_folder("Manuscript");

                let mut part1 = BinderItem::new_folder("Part I");
                let mut ch1 = BinderItem::new_folder("Chapter 1");
                ch1.children.push(BinderItem::new_text("Scene 1"));
                part1.children.push(ch1);
                draft.children.push(part1);

                let mut part2 = BinderItem::new_folder("Part II");
                let mut ch2 = BinderItem::new_folder("Chapter 2");
                ch2.children.push(BinderItem::new_text("Scene 1"));
                part2.children.push(ch2);
                draft.children.push(part2);

                let mut research = BinderItem::new_folder("Research");
                research.children.push(BinderItem::new_folder("Characters"));
                research.children.push(BinderItem::new_folder("Places"));
                research.children.push(BinderItem::new_folder("World Building"));
                research.children.push(BinderItem::new_text("Notes"));

                self.binder = Binder { draft, research, trash: BinderItem::new_folder("Trash") };
            }
            "short_story" => {
                let mut draft = BinderItem::new_folder("Story");
                draft.children.push(BinderItem::new_text("Opening"));
                draft.children.push(BinderItem::new_text("Middle"));
                draft.children.push(BinderItem::new_text("Climax"));
                draft.children.push(BinderItem::new_text("Resolution"));

                let mut research = BinderItem::new_folder("Research");
                research.children.push(BinderItem::new_text("Character Sketches"));
                research.children.push(BinderItem::new_text("Notes"));

                self.binder = Binder { draft, research, trash: BinderItem::new_folder("Trash") };
            }
            "poetry" => {
                let mut draft = BinderItem::new_folder("Collection");
                let mut section1 = BinderItem::new_folder("Section I");
                section1.children.push(BinderItem::new_text("Poem 1"));
                section1.children.push(BinderItem::new_text("Poem 2"));
                draft.children.push(section1);
                let mut section2 = BinderItem::new_folder("Section II");
                section2.children.push(BinderItem::new_text("Poem 3"));
                draft.children.push(section2);

                let mut research = BinderItem::new_folder("Research");
                research.children.push(BinderItem::new_text("Inspirations"));
                research.children.push(BinderItem::new_text("Notes"));

                self.binder = Binder { draft, research, trash: BinderItem::new_folder("Trash") };
            }
            "stage_play" => {
                let mut draft = BinderItem::new_folder("Play");
                draft.children.push(BinderItem::new_text("Title Page"));
                draft.children.push(BinderItem::new_text("Cast of Characters"));
                let mut act1 = BinderItem::new_folder("Act I");
                act1.children.push(BinderItem::new_text("Scene 1"));
                act1.children.push(BinderItem::new_text("Scene 2"));
                draft.children.push(act1);
                let mut act2 = BinderItem::new_folder("Act II");
                act2.children.push(BinderItem::new_text("Scene 1"));
                act2.children.push(BinderItem::new_text("Scene 2"));
                draft.children.push(act2);

                let mut research = BinderItem::new_folder("Research");
                research.children.push(BinderItem::new_folder("Characters"));
                research.children.push(BinderItem::new_folder("Set Design"));
                research.children.push(BinderItem::new_text("Notes"));

                self.binder = Binder { draft, research, trash: BinderItem::new_folder("Trash") };
            }
            "nonfiction" => {
                let mut draft = BinderItem::new_folder("Book");
                draft.children.push(BinderItem::new_text("Foreword"));
                draft.children.push(BinderItem::new_text("Introduction"));
                let mut ch1 = BinderItem::new_folder("Chapter 1");
                ch1.children.push(BinderItem::new_text("Section 1.1"));
                ch1.children.push(BinderItem::new_text("Section 1.2"));
                draft.children.push(ch1);
                let mut ch2 = BinderItem::new_folder("Chapter 2");
                ch2.children.push(BinderItem::new_text("Section 2.1"));
                draft.children.push(ch2);
                draft.children.push(BinderItem::new_text("Conclusion"));
                draft.children.push(BinderItem::new_text("Bibliography"));

                let mut research = BinderItem::new_folder("Research");
                research.children.push(BinderItem::new_folder("Sources"));
                research.children.push(BinderItem::new_folder("Interviews"));
                research.children.push(BinderItem::new_text("Notes"));

                self.binder = Binder { draft, research, trash: BinderItem::new_folder("Trash") };
            }
            "essay" => {
                let mut draft = BinderItem::new_folder("Essay");
                draft.children.push(BinderItem::new_text("Introduction"));
                draft.children.push(BinderItem::new_text("Body"));
                draft.children.push(BinderItem::new_text("Conclusion"));

                let mut research = BinderItem::new_folder("Research");
                research.children.push(BinderItem::new_text("Sources"));
                research.children.push(BinderItem::new_text("Notes"));

                self.binder = Binder { draft, research, trash: BinderItem::new_folder("Trash") };
            }
            "research_proposal" => {
                let mut draft = BinderItem::new_folder("Proposal");
                draft.children.push(BinderItem::new_text("Title Page"));
                draft.children.push(BinderItem::new_text("Abstract"));
                draft.children.push(BinderItem::new_text("Introduction"));
                draft.children.push(BinderItem::new_text("Research Questions"));
                draft.children.push(BinderItem::new_text("Literature Review"));
                draft.children.push(BinderItem::new_text("Methodology"));
                draft.children.push(BinderItem::new_text("Timeline"));
                draft.children.push(BinderItem::new_text("Budget"));
                draft.children.push(BinderItem::new_text("References"));

                let mut research = BinderItem::new_folder("Research");
                research.children.push(BinderItem::new_folder("Prior Work"));
                research.children.push(BinderItem::new_text("Notes"));

                self.binder = Binder { draft, research, trash: BinderItem::new_folder("Trash") };
            }
            "thesis" => {
                let mut draft = BinderItem::new_folder("Thesis");
                draft.children.push(BinderItem::new_text("Title Page"));
                draft.children.push(BinderItem::new_text("Acknowledgments"));
                draft.children.push(BinderItem::new_text("Abstract"));
                draft.children.push(BinderItem::new_text("Table of Contents"));
                let mut ch1 = BinderItem::new_folder("Chapter 1: Introduction");
                ch1.children.push(BinderItem::new_text("Background"));
                ch1.children.push(BinderItem::new_text("Research Questions"));
                ch1.children.push(BinderItem::new_text("Scope"));
                draft.children.push(ch1);
                let mut ch2 = BinderItem::new_folder("Chapter 2: Literature Review");
                ch2.children.push(BinderItem::new_text("Theoretical Framework"));
                ch2.children.push(BinderItem::new_text("Related Work"));
                draft.children.push(ch2);
                let mut ch3 = BinderItem::new_folder("Chapter 3: Methodology");
                ch3.children.push(BinderItem::new_text("Research Design"));
                ch3.children.push(BinderItem::new_text("Data Collection"));
                ch3.children.push(BinderItem::new_text("Analysis Methods"));
                draft.children.push(ch3);
                draft.children.push(BinderItem::new_text("Chapter 4: Results"));
                draft.children.push(BinderItem::new_text("Chapter 5: Discussion"));
                draft.children.push(BinderItem::new_text("Chapter 6: Conclusion"));
                draft.children.push(BinderItem::new_text("Bibliography"));
                draft.children.push(BinderItem::new_text("Appendices"));

                let mut research = BinderItem::new_folder("Research");
                research.children.push(BinderItem::new_folder("Sources"));
                research.children.push(BinderItem::new_folder("Data"));
                research.children.push(BinderItem::new_folder("Figures"));
                research.children.push(BinderItem::new_text("Notes"));

                self.binder = Binder { draft, research, trash: BinderItem::new_folder("Trash") };
            }
            "recipes" => {
                let mut draft = BinderItem::new_folder("Recipe Book");
                let mut appetizers = BinderItem::new_folder("Appetizers");
                appetizers.children.push(BinderItem::new_text("Recipe 1"));
                draft.children.push(appetizers);
                let mut mains = BinderItem::new_folder("Main Courses");
                mains.children.push(BinderItem::new_text("Recipe 1"));
                draft.children.push(mains);
                let mut desserts = BinderItem::new_folder("Desserts");
                desserts.children.push(BinderItem::new_text("Recipe 1"));
                draft.children.push(desserts);
                let mut drinks = BinderItem::new_folder("Drinks");
                drinks.children.push(BinderItem::new_text("Recipe 1"));
                draft.children.push(drinks);

                let mut research = BinderItem::new_folder("Research");
                research.children.push(BinderItem::new_text("Ingredient Notes"));
                research.children.push(BinderItem::new_text("Technique Notes"));

                self.binder = Binder { draft, research, trash: BinderItem::new_folder("Trash") };
            }
            "journal" => {
                let mut draft = BinderItem::new_folder("Journal");
                let mut jan = BinderItem::new_folder("January");
                jan.children.push(BinderItem::new_text("Entry 1"));
                draft.children.push(jan);
                let mut feb = BinderItem::new_folder("February");
                feb.children.push(BinderItem::new_text("Entry 1"));
                draft.children.push(feb);

                let mut research = BinderItem::new_folder("Research");
                research.children.push(BinderItem::new_text("Reflections"));
                research.children.push(BinderItem::new_text("Goals"));

                self.binder = Binder { draft, research, trash: BinderItem::new_folder("Trash") };
            }
            "blog" => {
                let mut draft = BinderItem::new_folder("Blog");
                let mut drafts = BinderItem::new_folder("Drafts");
                drafts.children.push(BinderItem::new_text("Post Idea 1"));
                drafts.children.push(BinderItem::new_text("Post Idea 2"));
                draft.children.push(drafts);
                let mut published = BinderItem::new_folder("Published");
                published.children.push(BinderItem::new_text("First Post"));
                draft.children.push(published);

                let mut research = BinderItem::new_folder("Research");
                research.children.push(BinderItem::new_text("Topic Ideas"));
                research.children.push(BinderItem::new_text("Style Guide"));

                self.binder = Binder { draft, research, trash: BinderItem::new_folder("Trash") };
            }
            "comic_script" => {
                let mut draft = BinderItem::new_folder("Comic");
                draft.children.push(BinderItem::new_text("Cover"));
                let mut issue1 = BinderItem::new_folder("Issue #1");
                issue1.children.push(BinderItem::new_text("Page 1"));
                issue1.children.push(BinderItem::new_text("Page 2"));
                issue1.children.push(BinderItem::new_text("Page 3"));
                draft.children.push(issue1);
                let mut issue2 = BinderItem::new_folder("Issue #2");
                issue2.children.push(BinderItem::new_text("Page 1"));
                draft.children.push(issue2);

                let mut research = BinderItem::new_folder("Research");
                research.children.push(BinderItem::new_folder("Characters"));
                research.children.push(BinderItem::new_folder("World"));
                research.children.push(BinderItem::new_text("Art References"));

                self.binder = Binder { draft, research, trash: BinderItem::new_folder("Trash") };
            }
            "radio_drama" => {
                let mut draft = BinderItem::new_folder("Radio Drama");
                draft.children.push(BinderItem::new_text("Title Page"));
                draft.children.push(BinderItem::new_text("Cast List"));
                let mut ep1 = BinderItem::new_folder("Episode 1");
                ep1.children.push(BinderItem::new_text("Scene 1"));
                ep1.children.push(BinderItem::new_text("Scene 2"));
                draft.children.push(ep1);

                let mut research = BinderItem::new_folder("Research");
                research.children.push(BinderItem::new_folder("Characters"));
                research.children.push(BinderItem::new_text("Sound Effects Notes"));
                research.children.push(BinderItem::new_text("Music Cues"));

                self.binder = Binder { draft, research, trash: BinderItem::new_folder("Trash") };
            }
            "documentary" => {
                let mut draft = BinderItem::new_folder("Documentary");
                draft.children.push(BinderItem::new_text("Synopsis / Treatment"));
                draft.children.push(BinderItem::new_text("Opening"));
                let mut seg1 = BinderItem::new_folder("Segment 1");
                seg1.children.push(BinderItem::new_text("Narration"));
                seg1.children.push(BinderItem::new_text("Interview Notes"));
                seg1.children.push(BinderItem::new_text("B-Roll List"));
                draft.children.push(seg1);
                let mut seg2 = BinderItem::new_folder("Segment 2");
                seg2.children.push(BinderItem::new_text("Narration"));
                seg2.children.push(BinderItem::new_text("Interview Notes"));
                draft.children.push(seg2);
                draft.children.push(BinderItem::new_text("Closing"));

                let mut research = BinderItem::new_folder("Research");
                research.children.push(BinderItem::new_folder("Interviews"));
                research.children.push(BinderItem::new_folder("Sources"));
                research.children.push(BinderItem::new_text("Shot List"));

                self.binder = Binder { draft, research, trash: BinderItem::new_folder("Trash") };
            }
            "mla_paper" => {
                let mut draft = BinderItem::new_folder("Paper");
                draft.children.push(BinderItem::new_text("Title Page"));
                draft.children.push(BinderItem::new_text("Introduction"));
                draft.children.push(BinderItem::new_text("Body"));
                draft.children.push(BinderItem::new_text("Conclusion"));
                draft.children.push(BinderItem::new_text("Works Cited"));

                let mut research = BinderItem::new_folder("Research");
                research.children.push(BinderItem::new_folder("Sources"));
                research.children.push(BinderItem::new_text("Notes"));
                research.children.push(BinderItem::new_text("Outline"));

                self.binder = Binder { draft, research, trash: BinderItem::new_folder("Trash") };
            }
            "chicago_essay" => {
                let mut draft = BinderItem::new_folder("Essay");
                draft.children.push(BinderItem::new_text("Title Page"));
                draft.children.push(BinderItem::new_text("Introduction"));
                draft.children.push(BinderItem::new_text("Argument"));
                draft.children.push(BinderItem::new_text("Analysis"));
                draft.children.push(BinderItem::new_text("Conclusion"));
                draft.children.push(BinderItem::new_text("Bibliography"));

                let mut research = BinderItem::new_folder("Research");
                research.children.push(BinderItem::new_folder("Primary Sources"));
                research.children.push(BinderItem::new_folder("Secondary Sources"));
                research.children.push(BinderItem::new_text("Notes"));

                self.binder = Binder { draft, research, trash: BinderItem::new_folder("Trash") };
            }
            _ => {} // Keep default structure
        }
    }

    /// Get a document by its binder item ID
    pub fn get_document(&self, item_id: &Uuid) -> Option<&Document> {
        self.binder.find_item(item_id)
            .and_then(|item| item.document.as_ref())
    }

    /// Get a mutable document by its binder item ID
    pub fn get_document_mut(&mut self, item_id: &Uuid) -> Option<&mut Document> {
        self.binder.find_item_mut(item_id)
            .and_then(|item| item.document.as_mut())
    }

    /// Create a snapshot of a document
    pub fn create_snapshot(&mut self, item_id: &Uuid, title: &str) -> Result<()> {
        if let Some(item) = self.binder.find_item_mut(item_id) {
            if let Some(doc) = &item.document {
                let snapshot = Snapshot::from_document(doc, title);
                item.snapshots.push(snapshot);
            }
        }
        Ok(())
    }

    /// Get the total word count across the entire project
    pub fn total_word_count(&self) -> usize {
        self.binder.total_word_count()
    }

    /// Get the total character count across the entire project
    pub fn total_char_count(&self) -> usize {
        self.binder.total_char_count()
    }

    /// Get the total document count
    pub fn document_count(&self) -> usize {
        self.binder.document_count()
    }

    /// Get the total folder count
    pub fn folder_count(&self) -> usize {
        self.binder.folder_count()
    }

    /// Estimate the total page count (250 words per page)
    pub fn estimated_pages(&self) -> usize {
        let words = self.total_word_count();
        if words == 0 { 0 } else { (words / 250).max(1) }
    }

    /// Get the project age as a human-readable string
    pub fn age_string(&self) -> String {
        let duration = Utc::now().signed_duration_since(self.created_at);
        let days = duration.num_days();
        if days == 0 {
            "today".to_string()
        } else if days == 1 {
            "1 day".to_string()
        } else if days < 30 {
            format!("{} days", days)
        } else if days < 365 {
            format!("{} months", days / 30)
        } else {
            format!("{} years", days / 365)
        }
    }

    /// Check if the project has been saved to disk
    pub fn is_saved(&self) -> bool {
        self.path.is_some()
    }

    /// Get the project directory name (without .scriv extension)
    pub fn directory_name(&self) -> String {
        format!("{}.scriv", self.title)
    }

    /// Get the number of collections
    pub fn collection_count(&self) -> usize {
        self.collections.len()
    }

    /// Find a collection by name
    pub fn find_collection(&self, name: &str) -> Option<&Collection> {
        self.collections.iter().find(|c| c.name == name)
    }

    /// Find a mutable collection by name
    pub fn find_collection_mut(&mut self, name: &str) -> Option<&mut Collection> {
        self.collections.iter_mut().find(|c| c.name == name)
    }

    /// Get a summary string for the project
    pub fn summary(&self) -> String {
        let words = self.total_word_count();
        let docs = self.document_count();
        let pages = self.estimated_pages();
        format!(
            "{}: {} words, {} documents, ~{} pages (created {})",
            self.title, words, docs, pages, self.age_string()
        )
    }

    /// Check if the project has any content
    pub fn has_content(&self) -> bool {
        self.total_word_count() > 0
    }

    /// Get the project file path as a string (if saved)
    pub fn path_display(&self) -> String {
        match &self.path {
            Some(p) => p.display().to_string(),
            None => "Unsaved".to_string(),
        }
    }

    /// Get the number of compile presets
    pub fn preset_count(&self) -> usize {
        self.compile_presets.len()
    }

    /// Find a compile preset by name
    pub fn find_preset(&self, name: &str) -> Option<&CompileOptions> {
        self.compile_presets.iter()
            .find(|(n, _)| n == name)
            .map(|(_, opts)| opts)
    }

    /// Add or update a compile preset
    pub fn set_preset(&mut self, name: &str, opts: CompileOptions) {
        if let Some(existing) = self.compile_presets.iter_mut().find(|(n, _)| n == name) {
            existing.1 = opts;
        } else {
            self.compile_presets.push((name.to_string(), opts));
        }
    }

    /// Remove a compile preset by name
    pub fn remove_preset(&mut self, name: &str) -> bool {
        let before = self.compile_presets.len();
        self.compile_presets.retain(|(n, _)| n != name);
        self.compile_presets.len() < before
    }

    /// All available template names (constant slice, no allocation).
    const TEMPLATES: &'static [&'static str] = &[
        "novel", "novel_with_parts", "short_story", "poetry",
        "screenplay", "stage_play", "nonfiction", "essay",
        "academic", "research_proposal", "thesis",
        "recipes", "journal", "blog",
        "comic_script", "radio_drama", "documentary",
        "mla_paper", "chicago_essay",
    ];

    /// Get all available template names
    pub fn available_templates() -> &'static [&'static str] {
        Self::TEMPLATES
    }

    /// Get a comprehensive status summary
    pub fn status_summary(&self) -> String {
        let collections = self.collection_count();
        let presets = self.preset_count();

        let mut parts = vec![
            format!("{} words", self.total_word_count()),
            format!("{} pages", self.estimated_pages()),
            format!("{} documents", self.document_count()),
            format!("{} folders", self.folder_count()),
        ];
        if collections > 0 { parts.push(format!("{} collections", collections)); }
        if presets > 0 { parts.push(format!("{} presets", presets)); }
        if !self.project_notes.is_empty() { parts.push("has project notes".to_string()); }

        parts.join(", ")
    }

    /// Check if a template name is valid
    pub fn is_valid_template(name: &str) -> bool {
        Self::TEMPLATES.contains(&name)
    }

    /// Compute content distribution: returns (folder_title, word_count) pairs for top-level items.
    pub fn content_distribution(&self) -> Vec<(String, usize)> {
        self.binder.draft.children.iter()
            .map(|child| (child.title.clone(), child.total_word_count()))
            .collect()
    }

    /// Analyze the project structure: (max_depth, avg_children, total_items).
    pub fn structure_analysis(&self) -> (usize, f64, usize) {
        let max_depth = self.binder.max_nesting_depth();
        let total = self.binder.item_count();
        let (folder_count, total_children) = self.binder.all_items().into_iter()
            .filter(|i| !i.children.is_empty())
            .fold((0usize, 0usize), |(count, sum), i| (count + 1, sum + i.child_count()));
        let avg_children = if folder_count == 0 {
            0.0
        } else {
            total_children as f64 / folder_count as f64
        };
        (max_depth, avg_children, total)
    }

    /// Writing velocity: average words per recorded history day.
    pub fn writing_velocity(&self) -> f64 {
        let entries = &self.writing_history.entries;
        if entries.is_empty() {
            return 0.0;
        }
        let total_words: i64 = entries.iter().map(|e| e.words_written).sum();
        total_words as f64 / entries.len() as f64
    }

    /// Get the longest and shortest documents by word count.
    pub fn word_count_extremes(&self) -> (Option<(String, usize)>, Option<(String, usize)>) {
        let mut longest: Option<(&str, usize)> = None;
        let mut shortest: Option<(&str, usize)> = None;

        for item in self.binder.all_items() {
            if item.kind != super::binder::BinderItemKind::Text { continue; }
            let wc = match item.document.as_ref() {
                Some(d) => d.word_count(),
                None => continue,
            };
            if wc == 0 { continue; }

            if longest.is_none_or(|(_, best)| wc > best) {
                longest = Some((item.title.as_str(), wc));
            }
            if shortest.is_none_or(|(_, best)| wc < best) {
                shortest = Some((item.title.as_str(), wc));
            }
        }

        (
            longest.map(|(t, wc)| (t.to_string(), wc)),
            shortest.map(|(t, wc)| (t.to_string(), wc)),
        )
    }

    /// Average document word count across all text items with content.
    pub fn avg_document_word_count(&self) -> f64 {
        let (count, total) = self.binder.all_items().into_iter()
            .filter(|i| i.kind == super::binder::BinderItemKind::Text)
            .filter_map(|i| i.document.as_ref().map(|d| d.word_count()))
            .filter(|wc| *wc > 0)
            .fold((0usize, 0usize), |(n, sum), wc| (n + 1, sum + wc));
        if count == 0 {
            return 0.0;
        }
        total as f64 / count as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_project() {
        let project = Project::new("My Novel");
        assert_eq!(project.title, "My Novel");
        assert!(project.collections.is_empty());
        assert!(project.project_notes.is_empty());
        assert!(!project.id.is_nil());
    }

    #[test]
    fn test_project_not_saved() {
        let project = Project::new("Test");
        assert!(!project.is_saved());
        assert_eq!(project.path_display(), "Unsaved");
    }

    #[test]
    fn test_directory_name() {
        let project = Project::new("My Novel");
        assert_eq!(project.directory_name(), "My Novel.scriv");
    }

    #[test]
    fn test_total_word_count_empty() {
        let project = Project::new("Test");
        assert_eq!(project.total_word_count(), 0);
    }

    #[test]
    fn test_document_count() {
        let project = Project::new("Test");
        // Default structure has some documents
        let count = project.document_count();
        assert!(count >= 0);
    }

    #[test]
    fn test_folder_count() {
        let project = Project::new("Test");
        let count = project.folder_count();
        // Default structure should have at least draft, research, trash folders
        assert!(count >= 3);
    }

    #[test]
    fn test_estimated_pages_empty() {
        let project = Project::new("Test");
        assert_eq!(project.estimated_pages(), 0);
    }

    #[test]
    fn test_has_content_empty() {
        let project = Project::new("Test");
        assert!(!project.has_content());
    }

    #[test]
    fn test_collection_count() {
        let mut project = Project::new("Test");
        assert_eq!(project.collection_count(), 0);
        project.collections.push(Collection::new_manual("Test Collection"));
        assert_eq!(project.collection_count(), 1);
    }

    #[test]
    fn test_find_collection() {
        let mut project = Project::new("Test");
        project.collections.push(Collection::new_manual("Favorites"));
        assert!(project.find_collection("Favorites").is_some());
        assert!(project.find_collection("NotFound").is_none());
    }

    #[test]
    fn test_find_collection_mut() {
        let mut project = Project::new("Test");
        project.collections.push(Collection::new_manual("Favorites"));
        let coll = project.find_collection_mut("Favorites");
        assert!(coll.is_some());
    }

    #[test]
    fn test_age_string() {
        let project = Project::new("Test");
        let age = project.age_string();
        assert_eq!(age, "today");
    }

    #[test]
    fn test_summary() {
        let project = Project::new("My Novel");
        let summary = project.summary();
        assert!(summary.contains("My Novel"));
        assert!(summary.contains("words"));
        assert!(summary.contains("documents"));
    }

    #[test]
    fn test_from_template_novel() {
        let project = Project::from_template("Test Novel", "novel");
        assert_eq!(project.title, "Test Novel");
        assert_eq!(project.binder.draft.title, "Manuscript");
    }

    #[test]
    fn test_from_template_screenplay() {
        let project = Project::from_template("Test Script", "screenplay");
        assert_eq!(project.binder.draft.title, "Screenplay");
    }

    #[test]
    fn test_from_template_academic() {
        let project = Project::from_template("Test Paper", "academic");
        assert_eq!(project.binder.draft.title, "Paper");
    }

    #[test]
    fn test_from_template_short_story() {
        let project = Project::from_template("Test Story", "short_story");
        assert_eq!(project.binder.draft.title, "Story");
    }

    #[test]
    fn test_from_template_poetry() {
        let project = Project::from_template("Poems", "poetry");
        assert_eq!(project.binder.draft.title, "Collection");
    }

    #[test]
    fn test_from_template_nonfiction() {
        let project = Project::from_template("Test Book", "nonfiction");
        assert_eq!(project.binder.draft.title, "Book");
    }

    #[test]
    fn test_from_template_essay() {
        let project = Project::from_template("Test Essay", "essay");
        assert_eq!(project.binder.draft.title, "Essay");
    }

    #[test]
    fn test_from_template_thesis() {
        let project = Project::from_template("Test Thesis", "thesis");
        assert_eq!(project.binder.draft.title, "Thesis");
    }

    #[test]
    fn test_from_template_journal() {
        let project = Project::from_template("My Journal", "journal");
        assert_eq!(project.binder.draft.title, "Journal");
    }

    #[test]
    fn test_from_template_blog() {
        let project = Project::from_template("My Blog", "blog");
        assert_eq!(project.binder.draft.title, "Blog");
    }

    #[test]
    fn test_from_template_unknown_keeps_default() {
        let project = Project::from_template("Test", "nonexistent_template");
        // Should keep default structure
        assert!(!project.binder.draft.title.is_empty());
    }

    #[test]
    fn test_project_serialization() {
        let project = Project::new("Serialization Test");
        let json = serde_json::to_string(&project);
        assert!(json.is_ok());
        let json = json.unwrap();
        assert!(json.contains("Serialization Test"));

        let parsed: Result<Project, _> = serde_json::from_str(&json);
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.title, "Serialization Test");
        assert_eq!(parsed.id, project.id);
    }

    #[test]
    fn test_save_and_load_snapshots() {
        use tempfile::tempdir;
        use crate::core::binder::BinderItem;

        let dir = tempdir().unwrap();
        let mut project = Project::new("SnapshotTest");

        // Add a document with content
        let mut item = BinderItem::new_text("Chapter 1");
        if let Some(doc) = &mut item.document {
            doc.content = "Hello world".to_string();
        }
        project.binder.draft.children.push(item);

        // Create a snapshot
        let item_id = project.binder.draft.children.last().unwrap().id;
        let _ = project.create_snapshot(&item_id, "First Draft");

        // Verify snapshot exists in memory
        let item = project.binder.find_item(&item_id).unwrap();
        assert_eq!(item.snapshots.len(), 1);
        assert_eq!(item.snapshots[0].title, "First Draft");
        assert_eq!(item.snapshots[0].content, "Hello world");

        // Save project
        project.save(dir.path()).unwrap();

        // Load project
        let project_dir = dir.path().join("SnapshotTest.scriv");
        let loaded = Project::load(&project_dir).unwrap();

        // Verify snapshot was persisted
        let loaded_item = loaded.binder.find_item(&item_id).unwrap();
        assert_eq!(loaded_item.snapshots.len(), 1);
        assert_eq!(loaded_item.snapshots[0].title, "First Draft");
        assert_eq!(loaded_item.snapshots[0].content, "Hello world");
    }

    #[test]
    fn test_save_and_load_compile_presets() {
        use tempfile::tempdir;
        use crate::export::compiler::{CompileOptions, OutputFormat};

        let dir = tempdir().unwrap();
        let mut project = Project::new("PresetTest");

        // Add compile presets
        let mut opts = CompileOptions::default();
        opts.title = "My Book".to_string();
        opts.format = OutputFormat::Html;
        project.compile_presets.push(("HTML Export".to_string(), opts));

        let mut opts2 = CompileOptions::default();
        opts2.format = OutputFormat::Latex;
        opts2.font_size = 14.0;
        project.compile_presets.push(("LaTeX Export".to_string(), opts2));

        // Save project
        project.save(dir.path()).unwrap();

        // Load project
        let project_dir = dir.path().join("PresetTest.scriv");
        let loaded = Project::load(&project_dir).unwrap();

        assert_eq!(loaded.compile_presets.len(), 2);
        assert_eq!(loaded.compile_presets[0].0, "HTML Export");
        assert_eq!(loaded.compile_presets[0].1.title, "My Book");
        assert!(matches!(loaded.compile_presets[0].1.format, OutputFormat::Html));
        assert_eq!(loaded.compile_presets[1].0, "LaTeX Export");
        assert!((loaded.compile_presets[1].1.font_size - 14.0).abs() < 0.01);
    }

    #[test]
    fn test_save_and_load_annotations() {
        use tempfile::tempdir;
        use crate::core::binder::BinderItem;
        use crate::core::annotation::Annotation;

        let dir = tempdir().unwrap();
        let mut project = Project::new("AnnotationTest");

        // Add a document with annotations
        let mut item = BinderItem::new_text("Chapter 1");
        if let Some(doc) = &mut item.document {
            doc.content = "The quick brown fox jumps over the lazy dog.".to_string();
            doc.annotations.push(Annotation::new(4, 19, "Check this phrasing"));
            doc.annotations.push(Annotation::new(35, 43, "Consider stronger word"));
        }
        project.binder.draft.children.push(item);
        let item_id = project.binder.draft.children.last().unwrap().id;

        // Save project
        project.save(dir.path()).unwrap();

        // Load project
        let project_dir = dir.path().join("AnnotationTest.scriv");
        let loaded = Project::load(&project_dir).unwrap();

        let loaded_item = loaded.binder.find_item(&item_id).unwrap();
        let doc = loaded_item.document.as_ref().unwrap();
        assert_eq!(doc.annotations.len(), 2);
        assert_eq!(doc.annotations[0].text, "Check this phrasing");
        assert_eq!(doc.annotations[0].start, 4);
        assert_eq!(doc.annotations[0].end, 19);
        assert_eq!(doc.annotations[1].text, "Consider stronger word");
    }

    #[test]
    fn test_save_and_load_multiple_snapshots() {
        use tempfile::tempdir;
        use crate::core::binder::BinderItem;

        let dir = tempdir().unwrap();
        let mut project = Project::new("MultiSnapTest");

        let mut item = BinderItem::new_text("Scene 1");
        if let Some(doc) = &mut item.document {
            doc.content = "Version 1".to_string();
        }
        project.binder.draft.children.push(item);
        let item_id = project.binder.draft.children.last().unwrap().id;

        // Create first snapshot
        let _ = project.create_snapshot(&item_id, "Draft 1");

        // Modify and create second snapshot
        if let Some(doc) = project.get_document_mut(&item_id) {
            doc.content = "Version 2 - improved".to_string();
        }
        let _ = project.create_snapshot(&item_id, "Draft 2");

        // Save and reload
        project.save(dir.path()).unwrap();
        let project_dir = dir.path().join("MultiSnapTest.scriv");
        let loaded = Project::load(&project_dir).unwrap();

        let loaded_item = loaded.binder.find_item(&item_id).unwrap();
        assert_eq!(loaded_item.snapshots.len(), 2);
        assert_eq!(loaded_item.snapshots[0].title, "Draft 1");
        assert_eq!(loaded_item.snapshots[0].content, "Version 1");
        assert_eq!(loaded_item.snapshots[1].title, "Draft 2");
        assert_eq!(loaded_item.snapshots[1].content, "Version 2 - improved");
    }

    // New: compile preset tests

    #[test]
    fn test_preset_count_empty() {
        let project = Project::new("Test");
        assert_eq!(project.preset_count(), 0);
    }

    #[test]
    fn test_set_and_find_preset() {
        let mut project = Project::new("Test");
        let opts = CompileOptions::default();
        project.set_preset("My Preset", opts);

        assert_eq!(project.preset_count(), 1);
        assert!(project.find_preset("My Preset").is_some());
        assert!(project.find_preset("Not Found").is_none());
    }

    #[test]
    fn test_set_preset_updates_existing() {
        let mut project = Project::new("Test");
        let mut opts1 = CompileOptions::default();
        opts1.title = "Old".to_string();
        project.set_preset("Preset", opts1);

        let mut opts2 = CompileOptions::default();
        opts2.title = "New".to_string();
        project.set_preset("Preset", opts2);

        assert_eq!(project.preset_count(), 1); // Still only one
        assert_eq!(project.find_preset("Preset").unwrap().title, "New");
    }

    #[test]
    fn test_remove_preset() {
        let mut project = Project::new("Test");
        project.set_preset("A", CompileOptions::default());
        project.set_preset("B", CompileOptions::default());

        assert!(project.remove_preset("A"));
        assert_eq!(project.preset_count(), 1);
        assert!(!project.remove_preset("A")); // Already removed
    }

    // New: template tests

    #[test]
    fn test_available_templates() {
        let templates = Project::available_templates();
        assert!(templates.len() >= 15);
        assert!(templates.contains(&"novel"));
        assert!(templates.contains(&"screenplay"));
        assert!(templates.contains(&"thesis"));
    }

    #[test]
    fn test_is_valid_template() {
        assert!(Project::is_valid_template("novel"));
        assert!(Project::is_valid_template("essay"));
        assert!(!Project::is_valid_template("nonexistent"));
    }

    #[test]
    fn test_all_templates_produce_valid_projects() {
        for template in Project::available_templates() {
            let project = Project::from_template(&format!("Test {}", template), template);
            assert!(!project.binder.draft.title.is_empty(),
                "Template '{}' produced empty draft title", template);
            assert_eq!(project.title, format!("Test {}", template));
        }
    }

    // New: status summary tests

    #[test]
    fn test_status_summary_empty() {
        let project = Project::new("Test");
        let summary = project.status_summary();
        assert!(summary.contains("0 words"));
        assert!(summary.contains("0 pages"));
    }

    #[test]
    fn test_status_summary_with_extras() {
        let mut project = Project::new("Test");
        project.collections.push(Collection::new_manual("Fav"));
        project.set_preset("Default", CompileOptions::default());
        project.project_notes = "Some notes".to_string();

        let summary = project.status_summary();
        assert!(summary.contains("1 collections"));
        assert!(summary.contains("1 presets"));
        assert!(summary.contains("has project notes"));
    }

    // New: additional template structure tests

    #[test]
    fn test_from_template_stage_play() {
        let project = Project::from_template("Test Play", "stage_play");
        assert_eq!(project.binder.draft.title, "Play");
        assert!(project.binder.draft.children.len() >= 3);
    }

    #[test]
    fn test_from_template_research_proposal() {
        let project = Project::from_template("Test", "research_proposal");
        assert_eq!(project.binder.draft.title, "Proposal");
    }

    #[test]
    fn test_from_template_recipes() {
        let project = Project::from_template("Test", "recipes");
        assert_eq!(project.binder.draft.title, "Recipe Book");
    }

    #[test]
    fn test_from_template_comic_script() {
        let project = Project::from_template("Test", "comic_script");
        assert_eq!(project.binder.draft.title, "Comic");
    }

    #[test]
    fn test_from_template_radio_drama() {
        let project = Project::from_template("Test", "radio_drama");
        assert_eq!(project.binder.draft.title, "Radio Drama");
    }

    #[test]
    fn test_from_template_documentary() {
        let project = Project::from_template("Test", "documentary");
        assert_eq!(project.binder.draft.title, "Documentary");
    }

    #[test]
    fn test_from_template_mla_paper() {
        let project = Project::from_template("Test", "mla_paper");
        assert_eq!(project.binder.draft.title, "Paper");
    }

    #[test]
    fn test_from_template_chicago_essay() {
        let project = Project::from_template("Test", "chicago_essay");
        assert_eq!(project.binder.draft.title, "Essay");
    }

    // --- Project analytics tests ---

    #[test]
    fn test_content_distribution_empty() {
        let project = Project::new("Test");
        let dist = project.content_distribution();
        assert!(dist.is_empty() || dist.iter().all(|(_, wc)| *wc == 0));
    }

    #[test]
    fn test_content_distribution_with_items() {
        use crate::core::binder::BinderItem;
        let mut project = Project::new("Test");
        let mut ch1 = BinderItem::new_text("Chapter 1");
        if let Some(doc) = &mut ch1.document {
            doc.content = "one two three four five".to_string();
        }
        let mut ch2 = BinderItem::new_text("Chapter 2");
        if let Some(doc) = &mut ch2.document {
            doc.content = "six seven".to_string();
        }
        project.binder.draft.add_child(ch1);
        project.binder.draft.add_child(ch2);

        let dist = project.content_distribution();
        assert_eq!(dist.len(), 2);
        assert_eq!(dist[0].0, "Chapter 1");
        assert_eq!(dist[0].1, 5);
        assert_eq!(dist[1].0, "Chapter 2");
        assert_eq!(dist[1].1, 2);
    }

    #[test]
    fn test_structure_analysis() {
        let mut project = Project::from_template("Novel", "novel");
        let (max_depth, _avg_children, total) = project.structure_analysis();
        assert!(max_depth >= 1);
        assert!(total >= 3);
    }

    #[test]
    fn test_structure_analysis_empty() {
        let project = Project::new("Empty");
        let (max_depth, _avg, total) = project.structure_analysis();
        assert_eq!(max_depth, 0);
        assert!(total >= 3); // Draft, Research, Trash always present
    }

    #[test]
    fn test_writing_velocity_empty() {
        let project = Project::new("Test");
        assert_eq!(project.writing_velocity(), 0.0);
    }

    #[test]
    fn test_writing_velocity_with_entries() {
        use crate::core::history::DailyEntry;
        use chrono::NaiveDate;

        let mut project = Project::new("Test");
        project.writing_history.entries.push(DailyEntry {
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            word_count_start: 0,
            word_count_end: 1000,
            words_written: 1000,
            time_spent_seconds: 3600,
        });
        project.writing_history.entries.push(DailyEntry {
            date: NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
            word_count_start: 1000,
            word_count_end: 1500,
            words_written: 500,
            time_spent_seconds: 1800,
        });

        let velocity = project.writing_velocity();
        assert!((velocity - 750.0).abs() < 0.01);
    }

    #[test]
    fn test_word_count_extremes_empty() {
        let project = Project::new("Test");
        let (longest, shortest) = project.word_count_extremes();
        assert!(longest.is_none());
        assert!(shortest.is_none());
    }

    #[test]
    fn test_word_count_extremes_with_items() {
        use crate::core::binder::BinderItem;
        let mut project = Project::new("Test");
        let mut short = BinderItem::new_text("Short");
        if let Some(doc) = &mut short.document {
            doc.content = "one two".to_string();
        }
        let mut long = BinderItem::new_text("Long");
        if let Some(doc) = &mut long.document {
            doc.content = "one two three four five six seven eight".to_string();
        }
        project.binder.draft.add_child(short);
        project.binder.draft.add_child(long);

        let (longest, shortest) = project.word_count_extremes();
        assert_eq!(longest.unwrap().0, "Long");
        assert_eq!(shortest.unwrap().0, "Short");
    }

    #[test]
    fn test_avg_document_word_count_empty() {
        let project = Project::new("Test");
        assert_eq!(project.avg_document_word_count(), 0.0);
    }

    #[test]
    fn test_avg_document_word_count() {
        use crate::core::binder::BinderItem;
        let mut project = Project::new("Test");
        let mut a = BinderItem::new_text("A");
        if let Some(doc) = &mut a.document {
            doc.content = "one two three four".to_string();
        }
        let mut b = BinderItem::new_text("B");
        if let Some(doc) = &mut b.document {
            doc.content = "five six".to_string();
        }
        project.binder.draft.add_child(a);
        project.binder.draft.add_child(b);

        let avg = project.avg_document_word_count();
        assert!((avg - 3.0).abs() < 0.01);
    }
}
