use uuid::Uuid;
use std::collections::HashSet;
use super::binder::{Binder, BinderItem, BinderItemKind};
use super::links;

/// Run a comprehensive validation pass on a project's binder
pub fn validate_project(binder: &Binder) -> ProjectValidation {
    let mut issues = Vec::new();

    // Check for duplicate IDs
    check_duplicate_ids(binder, &mut issues);

    // Check for empty documents
    check_empty_documents(binder, &mut issues);

    // Check for untitled items
    check_untitled_items(binder, &mut issues);

    // Check for folders without children
    check_empty_folders(binder, &mut issues);

    // Check for broken internal links
    check_broken_links(binder, &mut issues);

    // Check for very long documents (potential performance issue)
    check_document_lengths(binder, &mut issues);

    // Check for orphaned items in trash
    let trash_count = count_items(&binder.trash);

    ProjectValidation {
        issues,
        total_items: count_items(&binder.draft) + count_items(&binder.research) + trash_count,
        trash_items: trash_count,
    }
}

fn count_items(item: &BinderItem) -> usize {
    1 + item.children.iter().map(|c| count_items(c)).sum::<usize>()
}

fn check_duplicate_ids(binder: &Binder, issues: &mut Vec<ValidationIssue>) {
    let all_items = binder.all_items();
    let mut seen = HashSet::new();
    for item in &all_items {
        if !seen.insert(item.id) {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                kind: IssueKind::DuplicateId,
                item_id: Some(item.id),
                message: format!("Duplicate item ID: {} (\"{}\")", item.id, item.title),
            });
        }
    }
}

fn check_empty_documents(binder: &Binder, issues: &mut Vec<ValidationIssue>) {
    for item in binder.all_items() {
        if item.kind == BinderItemKind::Text {
            if let Some(ref doc) = item.document {
                if doc.content.trim().is_empty() {
                    issues.push(ValidationIssue {
                        severity: Severity::Info,
                        kind: IssueKind::EmptyDocument,
                        item_id: Some(item.id),
                        message: format!("Document \"{}\" has no content", item.title),
                    });
                }
            } else {
                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    kind: IssueKind::MissingDocument,
                    item_id: Some(item.id),
                    message: format!("Text item \"{}\" has no Document attached", item.title),
                });
            }
        }
    }
}

fn check_untitled_items(binder: &Binder, issues: &mut Vec<ValidationIssue>) {
    for item in binder.all_items() {
        if item.title.trim().is_empty() {
            issues.push(ValidationIssue {
                severity: Severity::Warning,
                kind: IssueKind::UntitledItem,
                item_id: Some(item.id),
                message: format!("Item {} has no title", item.id),
            });
        }
    }
}

fn check_empty_folders(binder: &Binder, issues: &mut Vec<ValidationIssue>) {
    fn check_folder(item: &BinderItem, issues: &mut Vec<ValidationIssue>) {
        if item.kind == BinderItemKind::Folder && item.children.is_empty() {
            // Skip root folders (Draft, Research, Trash) — they're allowed to be empty
            let is_root = item.title == "Draft" || item.title == "Research"
                || item.title == "Trash" || item.title == "Manuscript"
                || item.title == "Screenplay" || item.title == "Paper"
                || item.title == "Story" || item.title == "Collection"
                || item.title == "Book" || item.title == "Essay"
                || item.title == "Thesis" || item.title == "Journal"
                || item.title == "Blog";
            if !is_root {
                issues.push(ValidationIssue {
                    severity: Severity::Info,
                    kind: IssueKind::EmptyFolder,
                    item_id: Some(item.id),
                    message: format!("Folder \"{}\" is empty", item.title),
                });
            }
        }
        for child in &item.children {
            check_folder(child, issues);
        }
    }

    check_folder(&binder.draft, issues);
    check_folder(&binder.research, issues);
}

fn check_broken_links(binder: &Binder, issues: &mut Vec<ValidationIssue>) {
    let summary = links::link_health_summary(binder);
    for broken in &summary.broken_link_details {
        issues.push(ValidationIssue {
            severity: Severity::Warning,
            kind: IssueKind::BrokenLink,
            item_id: Some(broken.source_id),
            message: format!("Broken link to [[{}]]", broken.link_text),
        });
    }
}

fn check_document_lengths(binder: &Binder, issues: &mut Vec<ValidationIssue>) {
    let word_limit = 50_000; // Documents over 50k words might cause performance issues
    for item in binder.all_items() {
        if let Some(ref doc) = item.document {
            let words = doc.content.split_whitespace().count();
            if words > word_limit {
                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    kind: IssueKind::LargeDocument,
                    item_id: Some(item.id),
                    message: format!(
                        "Document \"{}\" has {} words — consider splitting for better performance",
                        item.title, words
                    ),
                });
            }
        }
    }
}

/// The result of a project validation pass
#[derive(Debug, Clone)]
pub struct ProjectValidation {
    pub issues: Vec<ValidationIssue>,
    pub total_items: usize,
    pub trash_items: usize,
}

impl ProjectValidation {
    pub fn is_clean(&self) -> bool {
        !self.issues.iter().any(|i| i.severity == Severity::Error || i.severity == Severity::Warning)
    }

    pub fn error_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == Severity::Error).count()
    }

    pub fn warning_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == Severity::Warning).count()
    }

    pub fn info_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == Severity::Info).count()
    }

    pub fn display(&self) -> String {
        let errors = self.error_count();
        let warnings = self.warning_count();
        let infos = self.info_count();

        if self.issues.is_empty() {
            return format!("Project is clean ({} items)", self.total_items);
        }

        let mut parts = Vec::new();
        if errors > 0 { parts.push(format!("{} error{}", errors, if errors == 1 { "" } else { "s" })); }
        if warnings > 0 { parts.push(format!("{} warning{}", warnings, if warnings == 1 { "" } else { "s" })); }
        if infos > 0 { parts.push(format!("{} info", infos)); }
        parts.join(", ")
    }
}

/// A single validation issue found in the project
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub kind: IssueKind,
    pub item_id: Option<Uuid>,
    pub message: String,
}

/// Severity of a validation issue
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Type of validation issue
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueKind {
    DuplicateId,
    EmptyDocument,
    MissingDocument,
    UntitledItem,
    EmptyFolder,
    BrokenLink,
    LargeDocument,
    Orphan,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::binder::{Binder, BinderItem};

    #[test]
    fn test_validate_empty_project() {
        let binder = Binder::default_structure();
        let result = validate_project(&binder);
        assert!(result.is_clean());
        assert_eq!(result.error_count(), 0);
    }

    #[test]
    fn test_validate_project_with_content() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Chapter 1");
        if let Some(ref mut doc) = item.document {
            doc.content = "Some actual content.".to_string();
        }
        binder.draft.children.push(item);

        let result = validate_project(&binder);
        assert!(result.is_clean());
    }

    #[test]
    fn test_validate_empty_document() {
        let mut binder = Binder::default_structure();
        let item = BinderItem::new_text("Empty Chapter");
        binder.draft.children.push(item);

        let result = validate_project(&binder);
        assert!(result.issues.iter().any(|i| i.kind == IssueKind::EmptyDocument));
    }

    #[test]
    fn test_validate_untitled_item() {
        let mut binder = Binder::default_structure();
        let item = BinderItem::new_text("");
        binder.draft.children.push(item);

        let result = validate_project(&binder);
        assert!(result.issues.iter().any(|i| i.kind == IssueKind::UntitledItem));
    }

    #[test]
    fn test_validate_empty_folder() {
        let mut binder = Binder::default_structure();
        let folder = BinderItem::new_folder("Empty Act");
        binder.draft.children.push(folder);

        let result = validate_project(&binder);
        assert!(result.issues.iter().any(|i| i.kind == IssueKind::EmptyFolder));
    }

    #[test]
    fn test_validate_broken_link() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Scene");
        if let Some(ref mut doc) = item.document {
            doc.content = "See [[Missing Chapter]].".to_string();
        }
        binder.draft.children.push(item);

        let result = validate_project(&binder);
        assert!(result.issues.iter().any(|i| i.kind == IssueKind::BrokenLink));
    }

    #[test]
    fn test_validate_display_clean() {
        let binder = Binder::default_structure();
        let result = validate_project(&binder);
        assert!(result.display().contains("clean"));
    }

    #[test]
    fn test_validate_display_with_issues() {
        let mut binder = Binder::default_structure();
        let item = BinderItem::new_text("");
        binder.draft.children.push(item);

        let result = validate_project(&binder);
        let display = result.display();
        assert!(display.contains("warning") || display.contains("info"));
    }

    #[test]
    fn test_count_items() {
        let mut folder = BinderItem::new_folder("Root");
        folder.children.push(BinderItem::new_text("A"));
        folder.children.push(BinderItem::new_text("B"));
        assert_eq!(count_items(&folder), 3); // root + 2 children
    }

    #[test]
    fn test_severity_comparison() {
        assert_eq!(Severity::Error, Severity::Error);
        assert_ne!(Severity::Error, Severity::Warning);
        assert_ne!(Severity::Warning, Severity::Info);
    }

    #[test]
    fn test_validation_error_count() {
        let validation = ProjectValidation {
            issues: vec![
                ValidationIssue {
                    severity: Severity::Error,
                    kind: IssueKind::DuplicateId,
                    item_id: None,
                    message: "test".to_string(),
                },
                ValidationIssue {
                    severity: Severity::Warning,
                    kind: IssueKind::EmptyDocument,
                    item_id: None,
                    message: "test".to_string(),
                },
                ValidationIssue {
                    severity: Severity::Info,
                    kind: IssueKind::EmptyFolder,
                    item_id: None,
                    message: "test".to_string(),
                },
            ],
            total_items: 10,
            trash_items: 0,
        };
        assert_eq!(validation.error_count(), 1);
        assert_eq!(validation.warning_count(), 1);
        assert_eq!(validation.info_count(), 1);
        assert!(!validation.is_clean());
    }

    #[test]
    fn test_validate_large_document() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Huge Chapter");
        if let Some(ref mut doc) = item.document {
            // Create a large document (> 100k words)
            doc.content = "word ".repeat(120_000);
        }
        binder.draft.children.push(item);

        let result = validate_project(&binder);
        assert!(result.issues.iter().any(|i| i.kind == IssueKind::LargeDocument));
    }

    #[test]
    fn test_validate_valid_link() {
        let mut binder = Binder::default_structure();
        let target = BinderItem::new_text("Target");
        let mut source = BinderItem::new_text("Source");
        if let Some(ref mut doc) = source.document {
            doc.content = "Link to [[Target]].".to_string();
        }
        binder.draft.children.push(source);
        binder.draft.children.push(target);

        let result = validate_project(&binder);
        assert!(!result.issues.iter().any(|i| i.kind == IssueKind::BrokenLink));
    }

    #[test]
    fn test_validate_multiple_issues() {
        let mut binder = Binder::default_structure();

        // Empty document
        binder.draft.children.push(BinderItem::new_text("Empty"));

        // Untitled
        binder.draft.children.push(BinderItem::new_text(""));

        // Broken link
        let mut linked = BinderItem::new_text("Linked");
        if let Some(ref mut doc) = linked.document {
            doc.content = "See [[Nonexistent]].".to_string();
        }
        binder.draft.children.push(linked);

        let result = validate_project(&binder);
        assert!(result.issues.len() >= 3);
        assert!(result.issues.iter().any(|i| i.kind == IssueKind::EmptyDocument));
        assert!(result.issues.iter().any(|i| i.kind == IssueKind::UntitledItem));
        assert!(result.issues.iter().any(|i| i.kind == IssueKind::BrokenLink));
    }

    #[test]
    fn test_orphan_issue_kind() {
        let issue = ValidationIssue {
            severity: Severity::Info,
            kind: IssueKind::Orphan,
            item_id: None,
            message: "orphan document".to_string(),
        };
        assert_eq!(issue.kind, IssueKind::Orphan);
        assert_eq!(issue.severity, Severity::Info);
    }
}
