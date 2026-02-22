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

impl TemplateCategory {
    /// All categories in display order
    pub fn all() -> Vec<Self> {
        vec![
            TemplateCategory::Blank,
            TemplateCategory::Fiction,
            TemplateCategory::NonFiction,
            TemplateCategory::Scriptwriting,
            TemplateCategory::Academic,
            TemplateCategory::Miscellaneous,
        ]
    }

    /// Get description of the category
    pub fn description(&self) -> &str {
        match self {
            TemplateCategory::Fiction => "Templates for novels, stories, and creative writing",
            TemplateCategory::NonFiction => "Templates for books, essays, and articles",
            TemplateCategory::Scriptwriting => "Templates for screenplays, plays, and scripts",
            TemplateCategory::Academic => "Templates for papers, theses, and research",
            TemplateCategory::Miscellaneous => "Other templates for various purposes",
            TemplateCategory::Blank => "Start with an empty project",
        }
    }

    /// Get an icon for the category
    pub fn icon(&self) -> &str {
        match self {
            TemplateCategory::Fiction => "\u{1F4D6}",
            TemplateCategory::NonFiction => "\u{1F4DA}",
            TemplateCategory::Scriptwriting => "\u{1F3AC}",
            TemplateCategory::Academic => "\u{1F393}",
            TemplateCategory::Miscellaneous => "\u{1F4CB}",
            TemplateCategory::Blank => "\u{1F4C4}",
        }
    }
}

impl Template {
    /// Get a one-line summary
    pub fn summary(&self) -> String {
        format!("{} — {} ({})", self.name, self.description, self.category)
    }

    /// Check if the template matches a search query
    pub fn matches_query(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        self.name.to_lowercase().contains(&q)
            || self.description.to_lowercase().contains(&q)
            || self.template_id.to_lowercase().contains(&q)
    }
}

/// Find a template by its ID
pub fn find_template(id: &str) -> Option<Template> {
    built_in_templates().into_iter().find(|t| t.template_id == id)
}

/// Get templates filtered by category
pub fn templates_by_category(category: &TemplateCategory) -> Vec<Template> {
    built_in_templates().into_iter().filter(|t| &t.category == category).collect()
}

/// Get the total number of built-in templates
pub fn template_count() -> usize {
    built_in_templates().len()
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
        Template {
            name: "Radio Drama".into(),
            description: "A radio drama script template with scenes and SFX cues.".into(),
            category: TemplateCategory::Scriptwriting,
            template_id: "radio_drama".into(),
        },
        Template {
            name: "Documentary Script".into(),
            description: "A documentary script with narration, interviews, and B-roll notes.".into(),
            category: TemplateCategory::Scriptwriting,
            template_id: "documentary".into(),
        },
        Template {
            name: "MLA Paper".into(),
            description: "Academic paper following MLA format guidelines.".into(),
            category: TemplateCategory::Academic,
            template_id: "mla_paper".into(),
        },
        Template {
            name: "Chicago Essay".into(),
            description: "Essay following Chicago Manual of Style guidelines.".into(),
            category: TemplateCategory::Academic,
            template_id: "chicago_essay".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_built_in_templates() {
        let templates = built_in_templates();
        assert!(templates.len() >= 15);
    }

    #[test]
    fn test_find_template() {
        let novel = find_template("novel");
        assert!(novel.is_some());
        assert_eq!(novel.unwrap().name, "Novel");

        let missing = find_template("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_templates_by_category() {
        let fiction = templates_by_category(&TemplateCategory::Fiction);
        assert!(fiction.len() >= 3);
        for t in &fiction {
            assert_eq!(t.category, TemplateCategory::Fiction);
        }

        let scripts = templates_by_category(&TemplateCategory::Scriptwriting);
        assert!(scripts.len() >= 2);
    }

    #[test]
    fn test_template_count() {
        let count = template_count();
        assert!(count >= 15);
        assert_eq!(count, built_in_templates().len());
    }

    #[test]
    fn test_template_summary() {
        let t = find_template("novel").unwrap();
        let summary = t.summary();
        assert!(summary.contains("Novel"));
        assert!(summary.contains("Fiction"));
    }

    #[test]
    fn test_template_matches_query() {
        let t = find_template("novel").unwrap();
        assert!(t.matches_query("novel"));
        assert!(t.matches_query("Novel"));
        assert!(t.matches_query("chapters"));
        assert!(!t.matches_query("zzz"));
    }

    #[test]
    fn test_template_category_all() {
        let all = TemplateCategory::all();
        assert_eq!(all.len(), 6);
    }

    #[test]
    fn test_template_category_descriptions() {
        for cat in TemplateCategory::all() {
            assert!(!cat.description().is_empty());
            assert!(!cat.icon().is_empty());
            assert!(!cat.to_string().is_empty());
        }
    }

    #[test]
    fn test_unique_template_ids() {
        let templates = built_in_templates();
        let mut ids: Vec<&str> = templates.iter().map(|t| t.template_id.as_str()).collect();
        let original_len = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), original_len, "Template IDs must be unique");
    }

    #[test]
    fn test_find_template_blank() {
        let blank = find_template("blank").unwrap();
        assert_eq!(blank.category, TemplateCategory::Blank);
    }

    #[test]
    fn test_find_template_screenplay() {
        let screenplay = find_template("screenplay").unwrap();
        assert_eq!(screenplay.category, TemplateCategory::Scriptwriting);
        assert!(screenplay.description.contains("screenplay"));
    }

    #[test]
    fn test_find_template_academic() {
        let academic = find_template("academic").unwrap();
        assert_eq!(academic.category, TemplateCategory::Academic);
    }

    #[test]
    fn test_templates_by_category_academic() {
        let academic = templates_by_category(&TemplateCategory::Academic);
        assert!(academic.len() >= 3);
        for t in &academic {
            assert_eq!(t.category, TemplateCategory::Academic);
        }
    }

    #[test]
    fn test_templates_by_category_blank() {
        let blank = templates_by_category(&TemplateCategory::Blank);
        assert_eq!(blank.len(), 1);
    }

    #[test]
    fn test_templates_by_category_misc() {
        let misc = templates_by_category(&TemplateCategory::Miscellaneous);
        assert!(misc.len() >= 2);
    }

    #[test]
    fn test_template_category_display() {
        assert_eq!(format!("{}", TemplateCategory::Fiction), "Fiction");
        assert_eq!(format!("{}", TemplateCategory::NonFiction), "Non-Fiction");
        assert_eq!(format!("{}", TemplateCategory::Scriptwriting), "Scriptwriting");
        assert_eq!(format!("{}", TemplateCategory::Academic), "Academic");
        assert_eq!(format!("{}", TemplateCategory::Miscellaneous), "Miscellaneous");
        assert_eq!(format!("{}", TemplateCategory::Blank), "Blank");
    }

    #[test]
    fn test_template_category_icon_not_empty() {
        for cat in TemplateCategory::all() {
            assert!(!cat.icon().is_empty());
        }
    }

    #[test]
    fn test_template_matches_query_case_insensitive() {
        let t = find_template("novel").unwrap();
        assert!(t.matches_query("NOVEL"));
        assert!(t.matches_query("novel"));
        assert!(t.matches_query("Novel"));
    }

    #[test]
    fn test_template_matches_query_by_id() {
        let t = find_template("novel_with_parts").unwrap();
        assert!(t.matches_query("novel_with_parts"));
    }

    #[test]
    fn test_template_matches_query_by_description() {
        let t = find_template("short_story").unwrap();
        assert!(t.matches_query("simple"));
    }

    #[test]
    fn test_all_templates_have_valid_categories() {
        let templates = built_in_templates();
        let all_cats = TemplateCategory::all();
        for t in &templates {
            assert!(all_cats.contains(&t.category),
                "Template '{}' has unrecognized category", t.name);
        }
    }

    #[test]
    fn test_template_serialization() {
        let t = find_template("novel").unwrap();
        let json = serde_json::to_string(&t).unwrap();
        let parsed: Template = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Novel");
        assert_eq!(parsed.template_id, "novel");
    }
}
