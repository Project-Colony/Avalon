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

        project.path = Some(project_dir.to_path_buf());
        Ok(project)
    }

    /// Save all documents to disk
    fn save_documents(&self, docs_dir: &Path) -> Result<()> {
        for item in self.binder.all_items() {
            if let Some(ref doc) = item.document {
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
            if let Some(ref doc) = item.document {
                let snapshot = Snapshot::from_document(doc, title);
                item.snapshots.push(snapshot);
            }
        }
        Ok(())
    }
}
