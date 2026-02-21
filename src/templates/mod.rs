use serde::{Deserialize, Serialize};

/// Built-in project templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub template_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TemplateCategory {
    Fiction,
    NonFiction,
    Scriptwriting,
    Academic,
    Miscellaneous,
    Blank,
}

impl std::fmt::Display for TemplateCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateCategory::Fiction => write!(f, "Fiction"),
            TemplateCategory::NonFiction => write!(f, "Non-Fiction"),
            TemplateCategory::Scriptwriting => write!(f, "Scriptwriting"),
            TemplateCategory::Academic => write!(f, "Academic"),
            TemplateCategory::Miscellaneous => write!(f, "Miscellaneous"),
            TemplateCategory::Blank => write!(f, "Blank"),
        }
    }
}

/// Get all built-in templates
pub fn built_in_templates() -> Vec<Template> {
    vec![
        // Blank
        Template {
            name: "Blank".into(),
            description: "An empty project with the default structure.".into(),
            category: TemplateCategory::Blank,
            template_id: "blank".into(),
        },
        // Fiction
        Template {
            name: "Novel".into(),
            description: "A novel template with chapters, scenes, and research folders.".into(),
            category: TemplateCategory::Fiction,
            template_id: "novel".into(),
        },
        Template {
            name: "Novel with Parts".into(),
            description: "A novel template organized by parts, each containing chapters.".into(),
            category: TemplateCategory::Fiction,
            template_id: "novel_with_parts".into(),
        },
        Template {
            name: "Short Story".into(),
            description: "A short story template with a simple structure.".into(),
            category: TemplateCategory::Fiction,
            template_id: "short_story".into(),
        },
        Template {
            name: "Poetry Collection".into(),
            description: "A template for organizing a collection of poems.".into(),
            category: TemplateCategory::Fiction,
            template_id: "poetry".into(),
        },
        // Scriptwriting
        Template {
            name: "Screenplay".into(),
            description: "A screenplay template with three-act structure.".into(),
            category: TemplateCategory::Scriptwriting,
            template_id: "screenplay".into(),
        },
        Template {
            name: "Stage Play".into(),
            description: "A stage play template with acts and scenes.".into(),
            category: TemplateCategory::Scriptwriting,
            template_id: "stage_play".into(),
        },
        // Non-Fiction
        Template {
            name: "Non-Fiction Book".into(),
            description: "A non-fiction book template with front and back matter.".into(),
            category: TemplateCategory::NonFiction,
            template_id: "nonfiction".into(),
        },
        Template {
            name: "Essay".into(),
            description: "A simple essay template.".into(),
            category: TemplateCategory::NonFiction,
            template_id: "essay".into(),
        },
        // Academic
        Template {
            name: "Academic Paper (APA)".into(),
            description: "Academic paper following APA format guidelines.".into(),
            category: TemplateCategory::Academic,
            template_id: "academic".into(),
        },
        Template {
            name: "Research Proposal".into(),
            description: "A research proposal template with standard sections.".into(),
            category: TemplateCategory::Academic,
            template_id: "research_proposal".into(),
        },
        Template {
            name: "Thesis / Dissertation".into(),
            description: "A comprehensive template for theses and dissertations.".into(),
            category: TemplateCategory::Academic,
            template_id: "thesis".into(),
        },
        // Miscellaneous
        Template {
            name: "Recipe Collection".into(),
            description: "Organize your recipes with categories and notes.".into(),
            category: TemplateCategory::Miscellaneous,
            template_id: "recipes".into(),
        },
        Template {
            name: "Journal / Diary".into(),
            description: "A personal journal with monthly sections.".into(),
            category: TemplateCategory::Miscellaneous,
            template_id: "journal".into(),
        },
        Template {
            name: "Blog".into(),
            description: "Blog post collection with drafts and published sections.".into(),
            category: TemplateCategory::Miscellaneous,
            template_id: "blog".into(),
        },
        Template {
            name: "Comic Script".into(),
            description: "A comic book script template with issues and pages.".into(),
            category: TemplateCategory::Scriptwriting,
            template_id: "comic_script".into(),
        },
    ]
}
