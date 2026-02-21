use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use anyhow::{Context, Result};

use super::binder::Binder;
use super::document::Document;
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
