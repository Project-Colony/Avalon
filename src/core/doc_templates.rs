use super::binder::BinderItem;

/// A reusable document template with pre-filled structure
#[derive(Debug, Clone)]
pub struct DocumentTemplate {
    pub id: &'static str,
    pub name: &'static str,
    pub category: TemplateCategory,
    pub description: &'static str,
    /// The template content (pre-filled text)
    pub content: &'static str,
    /// Optional synopsis
    pub synopsis: &'static str,
}

/// Categories for organizing document templates
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateCategory {
    Fiction,
    Nonfiction,
    Screenplay,
    Planning,
    Reference,
}

impl TemplateCategory {
    pub fn label(&self) -> &str {
        match self {
            TemplateCategory::Fiction => "Fiction",
            TemplateCategory::Nonfiction => "Nonfiction",
            TemplateCategory::Screenplay => "Screenplay",
            TemplateCategory::Planning => "Planning",
            TemplateCategory::Reference => "Reference",
        }
    }
}

impl DocumentTemplate {
    /// Create a new BinderItem from this template
    pub fn create_item(&self, title: &str) -> BinderItem {
        let mut item = BinderItem::new_text(title);
        if let Some(ref mut doc) = item.document {
            doc.content = self.content.to_string();
        }
        item.synopsis = self.synopsis.to_string();
        item
    }
}

/// Get all built-in document templates
pub fn builtin_templates() -> Vec<DocumentTemplate> {
    vec![
        // Fiction templates
        DocumentTemplate {
            id: "chapter",
            name: "Chapter",
            category: TemplateCategory::Fiction,
            description: "A standard chapter with a heading placeholder",
            content: "",
            synopsis: "",
        },
        DocumentTemplate {
            id: "scene",
            name: "Scene",
            category: TemplateCategory::Fiction,
            description: "A scene with setting/goal/conflict structure",
            content: "**Setting:** \n\n**Goal:** \n\n**Conflict:** \n\n---\n\n",
            synopsis: "",
        },
        DocumentTemplate {
            id: "character_sheet",
            name: "Character Sheet",
            category: TemplateCategory::Planning,
            description: "A character profile template",
            content: "## Physical Description\n\n- **Age:** \n- **Height:** \n- **Build:** \n- **Hair:** \n- **Eyes:** \n- **Distinguishing features:** \n\n## Personality\n\n- **Traits:** \n- **Strengths:** \n- **Weaknesses:** \n- **Fears:** \n- **Desires:** \n\n## Background\n\n- **Birthplace:** \n- **Education:** \n- **Occupation:** \n- **Family:** \n\n## Story Role\n\n- **Arc:** \n- **Motivation:** \n- **Internal conflict:** \n\n## Notes\n\n",
            synopsis: "Character profile and development notes",
        },
        DocumentTemplate {
            id: "world_building",
            name: "World Building",
            category: TemplateCategory::Planning,
            description: "A world building reference sheet",
            content: "## Geography\n\n\n## History\n\n\n## Culture & Society\n\n\n## Magic / Technology\n\n\n## Politics & Power Structures\n\n\n## Economy\n\n\n## Religion & Beliefs\n\n\n## Languages\n\n\n## Flora & Fauna\n\n\n## Notes\n\n",
            synopsis: "World building reference",
        },
        DocumentTemplate {
            id: "plot_outline",
            name: "Plot Outline",
            category: TemplateCategory::Planning,
            description: "Three-act structure plot outline",
            content: "## Act I — Setup\n\n### Hook\n\n\n### Inciting Incident\n\n\n### First Plot Point\n\n\n## Act II — Confrontation\n\n### Rising Action\n\n\n### Midpoint\n\n\n### Second Plot Point\n\n\n## Act III — Resolution\n\n### Climax\n\n\n### Falling Action\n\n\n### Resolution\n\n",
            synopsis: "Three-act plot structure",
        },
        DocumentTemplate {
            id: "scene_summary",
            name: "Scene Summary Card",
            category: TemplateCategory::Fiction,
            description: "Quick scene summary for the corkboard",
            content: "**POV:** \n**When:** \n**Where:** \n\n**What happens:**\n\n\n**Why it matters:**\n\n",
            synopsis: "",
        },
        DocumentTemplate {
            id: "research_note",
            name: "Research Note",
            category: TemplateCategory::Reference,
            description: "A structured research note with source tracking",
            content: "## Source\n\n- **Title:** \n- **Author:** \n- **URL/Location:** \n- **Date Accessed:** \n\n## Key Points\n\n- \n\n## Quotes\n\n> \n\n## How This Applies\n\n\n## Follow-up Questions\n\n- \n",
            synopsis: "Research note with source attribution",
        },
        DocumentTemplate {
            id: "blog_post",
            name: "Blog Post",
            category: TemplateCategory::Nonfiction,
            description: "A blog post template with SEO structure",
            content: "## Introduction\n\nHook the reader with an opening question or bold statement.\n\n## Main Points\n\n### Point 1\n\n\n### Point 2\n\n\n### Point 3\n\n\n## Conclusion\n\nSummarize key takeaways and include a call to action.\n\n---\n\n**Tags:** \n**Category:** \n",
            synopsis: "",
        },
        DocumentTemplate {
            id: "essay",
            name: "Essay",
            category: TemplateCategory::Nonfiction,
            description: "A five-paragraph essay template",
            content: "## Thesis\n\n\n## Argument 1\n\n\n## Argument 2\n\n\n## Argument 3\n\n\n## Conclusion\n\n",
            synopsis: "",
        },
        DocumentTemplate {
            id: "screenplay_scene",
            name: "Screenplay Scene",
            category: TemplateCategory::Screenplay,
            description: "A screenplay scene with Fountain formatting hints",
            content: "INT./EXT. LOCATION - TIME\n\nAction description.\n\nCHARACTER NAME\n(parenthetical)\nDialogue.\n\n",
            synopsis: "",
        },
        DocumentTemplate {
            id: "interview",
            name: "Interview Notes",
            category: TemplateCategory::Reference,
            description: "Interview transcript template",
            content: "## Interview Details\n\n- **Subject:** \n- **Date:** \n- **Location:** \n- **Duration:** \n\n## Questions & Responses\n\n**Q:** \n**A:** \n\n**Q:** \n**A:** \n\n**Q:** \n**A:** \n\n## Key Observations\n\n\n## Follow-up Items\n\n- \n",
            synopsis: "Interview transcript and notes",
        },
        DocumentTemplate {
            id: "timeline",
            name: "Timeline",
            category: TemplateCategory::Planning,
            description: "A chronological timeline of events",
            content: "## Timeline\n\n| Date/Period | Event | Notes |\n|-------------|-------|-------|\n|  |  |  |\n|  |  |  |\n|  |  |  |\n|  |  |  |\n|  |  |  |\n\n## Key Turning Points\n\n1. \n2. \n3. \n",
            synopsis: "Chronological event timeline",
        },
    ]
}

/// Find a template by ID
pub fn find_template(id: &str) -> Option<DocumentTemplate> {
    builtin_templates().into_iter().find(|t| t.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_templates_count() {
        let templates = builtin_templates();
        assert!(templates.len() >= 10);
    }

    #[test]
    fn test_find_template() {
        assert!(find_template("chapter").is_some());
        assert!(find_template("character_sheet").is_some());
        assert!(find_template("nonexistent").is_none());
    }

    #[test]
    fn test_create_item_from_template() {
        let template = find_template("scene").unwrap();
        let item = template.create_item("Act 1 - Scene 3");
        assert_eq!(item.title, "Act 1 - Scene 3");
        let doc = item.document.as_ref().unwrap();
        assert!(doc.content.contains("Setting"));
        assert!(doc.content.contains("Conflict"));
    }

    #[test]
    fn test_character_sheet_template() {
        let template = find_template("character_sheet").unwrap();
        let item = template.create_item("John Doe");
        let doc = item.document.as_ref().unwrap();
        assert!(doc.content.contains("Physical Description"));
        assert!(doc.content.contains("Personality"));
        assert!(doc.content.contains("Background"));
        assert!(!item.synopsis.is_empty());
    }

    #[test]
    fn test_plot_outline_template() {
        let template = find_template("plot_outline").unwrap();
        let item = template.create_item("My Plot");
        let doc = item.document.as_ref().unwrap();
        assert!(doc.content.contains("Act I"));
        assert!(doc.content.contains("Act II"));
        assert!(doc.content.contains("Act III"));
        assert!(doc.content.contains("Climax"));
    }

    #[test]
    fn test_research_note_template() {
        let template = find_template("research_note").unwrap();
        let item = template.create_item("Source Notes");
        let doc = item.document.as_ref().unwrap();
        assert!(doc.content.contains("Source"));
        assert!(doc.content.contains("Key Points"));
    }

    #[test]
    fn test_template_category_labels() {
        assert_eq!(TemplateCategory::Fiction.label(), "Fiction");
        assert_eq!(TemplateCategory::Nonfiction.label(), "Nonfiction");
        assert_eq!(TemplateCategory::Screenplay.label(), "Screenplay");
        assert_eq!(TemplateCategory::Planning.label(), "Planning");
        assert_eq!(TemplateCategory::Reference.label(), "Reference");
    }

    #[test]
    fn test_all_templates_have_nonempty_names() {
        for template in builtin_templates() {
            assert!(!template.name.is_empty(), "Template {} has empty name", template.id);
            assert!(!template.id.is_empty());
            assert!(
                !template.description.is_empty(),
                "Template {} has empty description",
                template.id
            );
        }
    }

    #[test]
    fn test_template_ids_unique() {
        let templates = builtin_templates();
        let mut ids: Vec<&str> = templates.iter().map(|t| t.id).collect();
        let original_len = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), original_len, "Duplicate template IDs found");
    }

    #[test]
    fn test_world_building_template() {
        let t = find_template("world_building").unwrap();
        let item = t.create_item("Arda");
        let doc = item.document.as_ref().unwrap();
        assert!(doc.content.contains("Geography"));
        assert!(doc.content.contains("Culture"));
        assert!(doc.content.contains("Magic"));
    }

    #[test]
    fn test_blog_post_template() {
        let t = find_template("blog_post").unwrap();
        let item = t.create_item("My Post");
        let doc = item.document.as_ref().unwrap();
        assert!(doc.content.contains("Introduction"));
        assert!(doc.content.contains("Conclusion"));
    }

    #[test]
    fn test_essay_template() {
        let t = find_template("essay").unwrap();
        let item = t.create_item("My Essay");
        let doc = item.document.as_ref().unwrap();
        assert!(doc.content.contains("Thesis"));
        assert!(doc.content.contains("Conclusion"));
    }

    #[test]
    fn test_screenplay_scene_template() {
        let t = find_template("screenplay_scene").unwrap();
        let item = t.create_item("Opening");
        let doc = item.document.as_ref().unwrap();
        assert!(doc.content.contains("INT."));
    }

    #[test]
    fn test_interview_template() {
        let t = find_template("interview").unwrap();
        let item = t.create_item("Interview 1");
        let doc = item.document.as_ref().unwrap();
        assert!(doc.content.contains("Subject"));
        assert!(doc.content.contains("Questions"));
    }

    #[test]
    fn test_timeline_template() {
        let t = find_template("timeline").unwrap();
        let item = t.create_item("Events");
        let doc = item.document.as_ref().unwrap();
        assert!(doc.content.contains("Timeline"));
        assert!(doc.content.contains("Turning Points"));
    }

    #[test]
    fn test_scene_summary_template() {
        let t = find_template("scene_summary").unwrap();
        let item = t.create_item("Scene 1");
        let doc = item.document.as_ref().unwrap();
        assert!(doc.content.contains("POV"));
        assert!(doc.content.contains("What happens"));
    }

    #[test]
    fn test_chapter_template_empty_content() {
        let t = find_template("chapter").unwrap();
        let item = t.create_item("Chapter 1");
        let doc = item.document.as_ref().unwrap();
        assert!(doc.content.is_empty());
    }

    #[test]
    fn test_create_item_sets_synopsis() {
        let t = find_template("character_sheet").unwrap();
        let item = t.create_item("Hero");
        assert!(!item.synopsis.is_empty());
    }

    #[test]
    fn test_create_item_title() {
        let t = find_template("scene").unwrap();
        let item = t.create_item("My Custom Title");
        assert_eq!(item.title, "My Custom Title");
    }
}
