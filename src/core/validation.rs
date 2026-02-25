#![allow(dead_code)] // Methods used by test code
use uuid::Uuid;
use std::collections::HashSet;
use super::binder::{Binder, BinderItem, BinderItemKind};
use super::links;

/// Run a comprehensive validation pass on a project's binder
pub fn validate_project(binder: &Binder) -> ProjectValidation {
    let mut issues = Vec::new();

    // Collect all items once for checks that need the flat list
    let all_items = binder.all_items();

    // Check for duplicate IDs (single-pass using pre-collected items)
    check_duplicate_ids_fast(&all_items, &mut issues);

    // Check for empty documents + untitled items + document lengths + metadata (single-pass)
    check_items_single_pass(&all_items, &mut issues);

    // Check for folders without children (uses tree traversal, not flat list)
    check_empty_folders(binder, &mut issues);

    // Check for broken internal links
    check_broken_links(binder, &mut issues);

    // Check for deep nesting (performance/usability concern)
    check_deep_nesting(binder, &mut issues);

    // Check for orphaned items in trash
    let trash_count = count_items(&binder.trash);

    ProjectValidation {
        issues,
        total_items: count_items(&binder.draft) + count_items(&binder.research) + trash_count,
        trash_items: trash_count,
    }
}

fn count_items(item: &BinderItem) -> usize {
    1 + item.children.iter().map(count_items).sum::<usize>()
}

fn check_duplicate_ids_fast(all_items: &[&BinderItem], issues: &mut Vec<ValidationIssue>) {
    let mut seen = HashSet::new();
    for item in all_items {
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

/// Single-pass check for empty documents, untitled items, document lengths, and metadata consistency.
fn check_items_single_pass(all_items: &[&BinderItem], issues: &mut Vec<ValidationIssue>) {
    let word_limit = 50_000;
    let mut total_text = 0usize;
    let mut with_label = 0usize;
    let mut with_status = 0usize;

    for item in all_items {
        // Untitled check
        if item.title.trim().is_empty() {
            issues.push(ValidationIssue {
                severity: Severity::Warning,
                kind: IssueKind::UntitledItem,
                item_id: Some(item.id),
                message: format!("Item {} has no title", item.id),
            });
        }

        if item.kind == BinderItemKind::Text {
            total_text += 1;
            if item.metadata.label.is_some() { with_label += 1; }
            if item.metadata.status.is_some() { with_status += 1; }

            // Empty document / missing document check
            if let Some(ref doc) = item.document {
                if doc.content.trim().is_empty() {
                    issues.push(ValidationIssue {
                        severity: Severity::Info,
                        kind: IssueKind::EmptyDocument,
                        item_id: Some(item.id),
                        message: format!("Document \"{}\" has no content", item.title),
                    });
                }
                // Large document check
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

    // Metadata consistency (only for projects with >= 3 text items)
    if total_text >= 3 {
        if with_label > 0 && with_label < total_text / 2 {
            issues.push(ValidationIssue {
                severity: Severity::Info,
                kind: IssueKind::InconsistentMetadata,
                item_id: None,
                message: format!(
                    "Only {}/{} items have labels — consider labeling all items for better organization",
                    with_label, total_text
                ),
            });
        }
        if with_status > 0 && with_status < total_text / 2 {
            issues.push(ValidationIssue {
                severity: Severity::Info,
                kind: IssueKind::InconsistentMetadata,
                item_id: None,
                message: format!(
                    "Only {}/{} items have status — consider setting status for all items",
                    with_status, total_text
                ),
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


fn check_deep_nesting(binder: &Binder, issues: &mut Vec<ValidationIssue>) {
    let max_depth_limit = 8;
    fn check_depth(item: &BinderItem, depth: usize, limit: usize, issues: &mut Vec<ValidationIssue>) {
        if depth > limit {
            issues.push(ValidationIssue {
                severity: Severity::Info,
                kind: IssueKind::DeepNesting,
                item_id: Some(item.id),
                message: format!(
                    "Item \"{}\" is nested {} levels deep — consider flattening for better organization",
                    item.title, depth
                ),
            });
        }
        for child in &item.children {
            check_depth(child, depth + 1, limit, issues);
        }
    }
    check_depth(&binder.draft, 0, max_depth_limit, issues);
    check_depth(&binder.research, 0, max_depth_limit, issues);
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

    /// Get issues sorted by severity (errors first)
    pub fn sorted_issues(&self) -> Vec<&ValidationIssue> {
        let mut sorted: Vec<&ValidationIssue> = self.issues.iter().collect();
        sorted.sort_by_key(|i| i.severity.weight());
        sorted
    }

    /// Get issues of a specific kind
    pub fn issues_of_kind(&self, kind: &IssueKind) -> Vec<&ValidationIssue> {
        self.issues.iter().filter(|i| &i.kind == kind).collect()
    }

    /// Get issues for a specific item
    pub fn issues_for_item(&self, item_id: Uuid) -> Vec<&ValidationIssue> {
        self.issues.iter().filter(|i| i.item_id == Some(item_id)).collect()
    }

    /// Whether there are auto-fixable issues
    pub fn has_auto_fixable(&self) -> bool {
        self.issues.iter().any(|i| i.is_auto_fixable())
    }

    /// Count of auto-fixable issues
    pub fn auto_fixable_count(&self) -> usize {
        self.issues.iter().filter(|i| i.is_auto_fixable()).count()
    }

    /// Health score (0-100, higher is better)
    pub fn health_score(&self) -> f64 {
        if self.total_items == 0 {
            return 100.0;
        }
        let error_penalty = self.error_count() as f64 * 10.0;
        let warning_penalty = self.warning_count() as f64 * 3.0;
        let info_penalty = self.info_count() as f64 * 1.0;
        let total_penalty = error_penalty + warning_penalty + info_penalty;
        (100.0 - total_penalty).clamp(0.0, 100.0)
    }

    /// Health grade
    pub fn health_grade(&self) -> &str {
        let score = self.health_score();
        if score >= 95.0 { "Excellent" }
        else if score >= 80.0 { "Good" }
        else if score >= 60.0 { "Fair" }
        else { "Needs Attention" }
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
    DeepNesting,
    InconsistentMetadata,
}

impl Severity {
    /// Human-readable label
    pub fn label(&self) -> &str {
        match self {
            Severity::Error => "Error",
            Severity::Warning => "Warning",
            Severity::Info => "Info",
        }
    }

    /// Icon character for UI
    pub fn icon(&self) -> &str {
        match self {
            Severity::Error => "E",
            Severity::Warning => "W",
            Severity::Info => "I",
        }
    }

    /// Sort weight (errors first)
    pub fn weight(&self) -> usize {
        match self {
            Severity::Error => 0,
            Severity::Warning => 1,
            Severity::Info => 2,
        }
    }
}

impl IssueKind {
    /// Human-readable label
    pub fn label(&self) -> &str {
        match self {
            IssueKind::DuplicateId => "Duplicate ID",
            IssueKind::EmptyDocument => "Empty Document",
            IssueKind::MissingDocument => "Missing Document",
            IssueKind::UntitledItem => "Untitled Item",
            IssueKind::EmptyFolder => "Empty Folder",
            IssueKind::BrokenLink => "Broken Link",
            IssueKind::LargeDocument => "Large Document",
            IssueKind::Orphan => "Orphan",
            IssueKind::DeepNesting => "Deep Nesting",
            IssueKind::InconsistentMetadata => "Inconsistent Metadata",
        }
    }

    /// Whether this kind is automatically fixable
    pub fn is_auto_fixable(&self) -> bool {
        matches!(self, IssueKind::EmptyDocument | IssueKind::UntitledItem | IssueKind::EmptyFolder
            | IssueKind::InconsistentMetadata)
    }

    /// Suggested fix description
    pub fn fix_hint(&self) -> &str {
        match self {
            IssueKind::DuplicateId => "Regenerate the item's UUID",
            IssueKind::EmptyDocument => "Add content or delete the document",
            IssueKind::MissingDocument => "Recreate the document data",
            IssueKind::UntitledItem => "Add a title to the item",
            IssueKind::EmptyFolder => "Add items or remove the folder",
            IssueKind::BrokenLink => "Fix or remove the broken link",
            IssueKind::LargeDocument => "Split the document into smaller sections",
            IssueKind::Orphan => "Link to this document or move to trash",
            IssueKind::DeepNesting => "Move deeply nested items closer to the root",
            IssueKind::InconsistentMetadata => "Apply labels or status to remaining items",
        }
    }
}

impl ValidationIssue {
    /// Formatted display string with severity and message
    pub fn display(&self) -> String {
        format!("[{}] {}", self.severity.label(), self.message)
    }

    /// Whether this issue can be automatically fixed
    pub fn is_auto_fixable(&self) -> bool {
        self.kind.is_auto_fixable()
    }
}

/// A record of a single auto-fix that was applied
#[derive(Debug, Clone)]
pub struct FixRecord {
    pub kind: IssueKind,
    pub item_id: Option<Uuid>,
    pub description: String,
}

/// Result of running auto-fix on a binder
#[derive(Debug)]
pub struct AutoFixResult {
    pub fixes_applied: Vec<FixRecord>,
    pub skipped: Vec<ValidationIssue>,
}

impl AutoFixResult {
    pub fn fix_count(&self) -> usize {
        self.fixes_applied.len()
    }

    pub fn skip_count(&self) -> usize {
        self.skipped.len()
    }

    pub fn summary(&self) -> String {
        if self.fixes_applied.is_empty() {
            return "No fixes applied".to_string();
        }
        let mut parts = Vec::new();
        parts.push(format!("{} fixes applied", self.fixes_applied.len()));
        if !self.skipped.is_empty() {
            parts.push(format!("{} skipped (not auto-fixable)", self.skipped.len()));
        }
        parts.join(", ")
    }
}

/// Auto-fix engine: applies fixes to a binder for auto-fixable issues
pub fn auto_fix(binder: &mut Binder) -> AutoFixResult {
    let validation = validate_project(binder);
    let mut fixes_applied = Vec::new();
    let mut skipped = Vec::new();

    for issue in &validation.issues {
        if !issue.is_auto_fixable() {
            skipped.push(issue.clone());
            continue;
        }

        match issue.kind {
            IssueKind::UntitledItem => {
                if let Some(id) = issue.item_id {
                    if let Some(item) = binder.find_item_mut(&id) {
                        let new_title = match item.kind {
                            BinderItemKind::Folder => "Untitled Folder",
                            BinderItemKind::Text => "Untitled Document",
                            _ => "Untitled Item",
                        };
                        item.title = new_title.to_string();
                        fixes_applied.push(FixRecord {
                            kind: IssueKind::UntitledItem,
                            item_id: Some(id),
                            description: format!("Renamed to \"{}\"", new_title),
                        });
                    }
                }
            }
            IssueKind::EmptyDocument => {
                // Don't auto-delete — just mark for the user's attention
                // by adding a placeholder
                if let Some(id) = issue.item_id {
                    if let Some(item) = binder.find_item_mut(&id) {
                        if let Some(ref mut doc) = item.document {
                            if doc.content.trim().is_empty() {
                                doc.content = "[Empty — placeholder added by auto-fix]".to_string();
                                fixes_applied.push(FixRecord {
                                    kind: IssueKind::EmptyDocument,
                                    item_id: Some(id),
                                    description: "Added placeholder content".to_string(),
                                });
                            }
                        }
                    }
                }
            }
            IssueKind::EmptyFolder => {
                // Move empty folders to trash (if they're in draft)
                if let Some(id) = issue.item_id {
                    if binder.move_to_trash(&id) {
                        fixes_applied.push(FixRecord {
                            kind: IssueKind::EmptyFolder,
                            item_id: Some(id),
                            description: "Moved empty folder to trash".to_string(),
                        });
                    }
                }
            }
            // InconsistentMetadata is technically auto-fixable but we skip it
            // because applying default metadata is too opinionated
            _ => {
                skipped.push(issue.clone());
            }
        }
    }

    AutoFixResult {
        fixes_applied,
        skipped,
    }
}

/// Fix only a specific issue kind
pub fn fix_issues_of_kind(binder: &mut Binder, kind: &IssueKind) -> Vec<FixRecord> {
    let validation = validate_project(binder);
    let matching: Vec<&ValidationIssue> = validation.issues.iter()
        .filter(|i| &i.kind == kind && i.is_auto_fixable())
        .collect();

    let mut fixes = Vec::new();
    for issue in matching {
        match issue.kind {
            IssueKind::UntitledItem => {
                if let Some(id) = issue.item_id {
                    if let Some(item) = binder.find_item_mut(&id) {
                        let new_title = match item.kind {
                            BinderItemKind::Folder => "Untitled Folder",
                            BinderItemKind::Text => "Untitled Document",
                            _ => "Untitled Item",
                        };
                        item.title = new_title.to_string();
                        fixes.push(FixRecord {
                            kind: IssueKind::UntitledItem,
                            item_id: Some(id),
                            description: format!("Renamed to \"{}\"", new_title),
                        });
                    }
                }
            }
            IssueKind::EmptyDocument => {
                if let Some(id) = issue.item_id {
                    if let Some(item) = binder.find_item_mut(&id) {
                        if let Some(ref mut doc) = item.document {
                            if doc.content.trim().is_empty() {
                                doc.content = "[Empty — placeholder added by auto-fix]".to_string();
                                fixes.push(FixRecord {
                                    kind: IssueKind::EmptyDocument,
                                    item_id: Some(id),
                                    description: "Added placeholder content".to_string(),
                                });
                            }
                        }
                    }
                }
            }
            IssueKind::EmptyFolder => {
                if let Some(id) = issue.item_id {
                    if binder.move_to_trash(&id) {
                        fixes.push(FixRecord {
                            kind: IssueKind::EmptyFolder,
                            item_id: Some(id),
                            description: "Moved empty folder to trash".to_string(),
                        });
                    }
                }
            }
            _ => {}
        }
    }
    fixes
}

/// Regenerate a duplicate ID (specific fix for DuplicateId issues)
pub fn fix_duplicate_id(binder: &mut Binder, id: &Uuid) -> Option<Uuid> {
    if let Some(item) = binder.find_item_mut(id) {
        let new_id = Uuid::new_v4();
        item.id = new_id;
        Some(new_id)
    } else {
        None
    }
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

    #[test]
    fn test_validate_root_folder_not_flagged() {
        let binder = Binder::default_structure();
        let result = validate_project(&binder);
        assert!(!result.issues.iter().any(|i| {
            i.kind == IssueKind::EmptyFolder && i.message.contains("Draft")
        }));
    }

    #[test]
    fn test_validate_folder_with_children_not_flagged() {
        let mut binder = Binder::default_structure();
        let mut folder = BinderItem::new_folder("Chapter 1");
        let mut item = BinderItem::new_text("Scene 1");
        if let Some(ref mut doc) = item.document {
            doc.content = "Content.".to_string();
        }
        folder.children.push(item);
        binder.draft.children.push(folder);

        let result = validate_project(&binder);
        assert!(!result.issues.iter().any(|i| {
            i.kind == IssueKind::EmptyFolder && i.message.contains("Chapter 1")
        }));
    }

    #[test]
    fn test_display_single_error() {
        let validation = ProjectValidation {
            issues: vec![
                ValidationIssue {
                    severity: Severity::Error,
                    kind: IssueKind::DuplicateId,
                    item_id: None,
                    message: "test".to_string(),
                },
            ],
            total_items: 1,
            trash_items: 0,
        };
        let display = validation.display();
        assert!(display.contains("1 error"));
        assert!(!display.contains("errors"));
    }

    #[test]
    fn test_display_plural() {
        let validation = ProjectValidation {
            issues: vec![
                ValidationIssue {
                    severity: Severity::Warning,
                    kind: IssueKind::EmptyDocument,
                    item_id: None,
                    message: "a".to_string(),
                },
                ValidationIssue {
                    severity: Severity::Warning,
                    kind: IssueKind::UntitledItem,
                    item_id: None,
                    message: "b".to_string(),
                },
            ],
            total_items: 5,
            trash_items: 0,
        };
        let display = validation.display();
        assert!(display.contains("2 warnings"));
    }

    #[test]
    fn test_is_clean_info_only() {
        let validation = ProjectValidation {
            issues: vec![
                ValidationIssue {
                    severity: Severity::Info,
                    kind: IssueKind::EmptyDocument,
                    item_id: None,
                    message: "info".to_string(),
                },
            ],
            total_items: 1,
            trash_items: 0,
        };
        assert!(validation.is_clean());
    }

    #[test]
    fn test_total_items_count() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("A");
        if let Some(ref mut doc) = item.document {
            doc.content = "Content.".to_string();
        }
        binder.draft.children.push(item);

        let result = validate_project(&binder);
        assert!(result.total_items >= 4);
    }

    #[test]
    fn test_trash_items_count() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Trashed");
        if let Some(ref mut doc) = item.document {
            doc.content = "Content.".to_string();
        }
        binder.trash.children.push(item);

        let result = validate_project(&binder);
        assert!(result.trash_items >= 1);
    }

    #[test]
    fn test_issue_kind_equality() {
        assert_eq!(IssueKind::DuplicateId, IssueKind::DuplicateId);
        assert_ne!(IssueKind::DuplicateId, IssueKind::EmptyDocument);
        assert_ne!(IssueKind::BrokenLink, IssueKind::LargeDocument);
    }

    // New: Severity tests

    #[test]
    fn test_severity_labels() {
        assert_eq!(Severity::Error.label(), "Error");
        assert_eq!(Severity::Warning.label(), "Warning");
        assert_eq!(Severity::Info.label(), "Info");
    }

    #[test]
    fn test_severity_icons() {
        assert_eq!(Severity::Error.icon(), "E");
        assert_eq!(Severity::Warning.icon(), "W");
        assert_eq!(Severity::Info.icon(), "I");
    }

    #[test]
    fn test_severity_weight_ordering() {
        assert!(Severity::Error.weight() < Severity::Warning.weight());
        assert!(Severity::Warning.weight() < Severity::Info.weight());
    }

    // New: IssueKind tests

    #[test]
    fn test_issue_kind_labels() {
        assert_eq!(IssueKind::DuplicateId.label(), "Duplicate ID");
        assert_eq!(IssueKind::EmptyDocument.label(), "Empty Document");
        assert_eq!(IssueKind::BrokenLink.label(), "Broken Link");
        assert_eq!(IssueKind::LargeDocument.label(), "Large Document");
        assert_eq!(IssueKind::Orphan.label(), "Orphan");
    }

    #[test]
    fn test_issue_kind_auto_fixable() {
        assert!(IssueKind::EmptyDocument.is_auto_fixable());
        assert!(IssueKind::UntitledItem.is_auto_fixable());
        assert!(IssueKind::EmptyFolder.is_auto_fixable());
        assert!(!IssueKind::DuplicateId.is_auto_fixable());
        assert!(!IssueKind::BrokenLink.is_auto_fixable());
        assert!(!IssueKind::LargeDocument.is_auto_fixable());
    }

    #[test]
    fn test_issue_kind_fix_hints() {
        for kind in [IssueKind::DuplicateId, IssueKind::EmptyDocument,
            IssueKind::MissingDocument, IssueKind::UntitledItem,
            IssueKind::EmptyFolder, IssueKind::BrokenLink,
            IssueKind::LargeDocument, IssueKind::Orphan] {
            assert!(!kind.fix_hint().is_empty());
        }
    }

    // New: ValidationIssue tests

    #[test]
    fn test_validation_issue_display() {
        let issue = ValidationIssue {
            severity: Severity::Error,
            kind: IssueKind::DuplicateId,
            item_id: None,
            message: "Duplicate found".to_string(),
        };
        assert_eq!(issue.display(), "[Error] Duplicate found");
    }

    #[test]
    fn test_validation_issue_auto_fixable() {
        let fixable = ValidationIssue {
            severity: Severity::Info,
            kind: IssueKind::EmptyDocument,
            item_id: None,
            message: "empty".to_string(),
        };
        assert!(fixable.is_auto_fixable());

        let not_fixable = ValidationIssue {
            severity: Severity::Error,
            kind: IssueKind::DuplicateId,
            item_id: None,
            message: "dupe".to_string(),
        };
        assert!(!not_fixable.is_auto_fixable());
    }

    // New: ProjectValidation method tests

    #[test]
    fn test_sorted_issues() {
        let validation = ProjectValidation {
            issues: vec![
                ValidationIssue { severity: Severity::Info, kind: IssueKind::EmptyFolder, item_id: None, message: "a".to_string() },
                ValidationIssue { severity: Severity::Error, kind: IssueKind::DuplicateId, item_id: None, message: "b".to_string() },
                ValidationIssue { severity: Severity::Warning, kind: IssueKind::BrokenLink, item_id: None, message: "c".to_string() },
            ],
            total_items: 5,
            trash_items: 0,
        };

        let sorted = validation.sorted_issues();
        assert_eq!(sorted[0].severity, Severity::Error);
        assert_eq!(sorted[1].severity, Severity::Warning);
        assert_eq!(sorted[2].severity, Severity::Info);
    }

    #[test]
    fn test_issues_of_kind() {
        let validation = ProjectValidation {
            issues: vec![
                ValidationIssue { severity: Severity::Info, kind: IssueKind::EmptyDocument, item_id: None, message: "a".to_string() },
                ValidationIssue { severity: Severity::Info, kind: IssueKind::EmptyDocument, item_id: None, message: "b".to_string() },
                ValidationIssue { severity: Severity::Warning, kind: IssueKind::BrokenLink, item_id: None, message: "c".to_string() },
            ],
            total_items: 5,
            trash_items: 0,
        };

        assert_eq!(validation.issues_of_kind(&IssueKind::EmptyDocument).len(), 2);
        assert_eq!(validation.issues_of_kind(&IssueKind::BrokenLink).len(), 1);
        assert_eq!(validation.issues_of_kind(&IssueKind::DuplicateId).len(), 0);
    }

    #[test]
    fn test_issues_for_item() {
        let id = Uuid::new_v4();
        let validation = ProjectValidation {
            issues: vec![
                ValidationIssue { severity: Severity::Info, kind: IssueKind::EmptyDocument, item_id: Some(id), message: "a".to_string() },
                ValidationIssue { severity: Severity::Warning, kind: IssueKind::UntitledItem, item_id: Some(id), message: "b".to_string() },
                ValidationIssue { severity: Severity::Warning, kind: IssueKind::BrokenLink, item_id: None, message: "c".to_string() },
            ],
            total_items: 5,
            trash_items: 0,
        };

        assert_eq!(validation.issues_for_item(id).len(), 2);
        assert_eq!(validation.issues_for_item(Uuid::new_v4()).len(), 0);
    }

    #[test]
    fn test_has_auto_fixable() {
        let validation_with = ProjectValidation {
            issues: vec![
                ValidationIssue { severity: Severity::Info, kind: IssueKind::EmptyDocument, item_id: None, message: "fix me".to_string() },
            ],
            total_items: 1,
            trash_items: 0,
        };
        assert!(validation_with.has_auto_fixable());
        assert_eq!(validation_with.auto_fixable_count(), 1);

        let validation_without = ProjectValidation {
            issues: vec![
                ValidationIssue { severity: Severity::Error, kind: IssueKind::DuplicateId, item_id: None, message: "no fix".to_string() },
            ],
            total_items: 1,
            trash_items: 0,
        };
        assert!(!validation_without.has_auto_fixable());
        assert_eq!(validation_without.auto_fixable_count(), 0);
    }

    #[test]
    fn test_health_score_perfect() {
        let validation = ProjectValidation {
            issues: vec![],
            total_items: 10,
            trash_items: 0,
        };
        assert_eq!(validation.health_score(), 100.0);
        assert_eq!(validation.health_grade(), "Excellent");
    }

    #[test]
    fn test_health_score_with_issues() {
        let validation = ProjectValidation {
            issues: vec![
                ValidationIssue { severity: Severity::Error, kind: IssueKind::DuplicateId, item_id: None, message: "e".to_string() },
                ValidationIssue { severity: Severity::Warning, kind: IssueKind::BrokenLink, item_id: None, message: "w".to_string() },
            ],
            total_items: 10,
            trash_items: 0,
        };
        let score = validation.health_score();
        assert!(score < 100.0);
        assert!(score > 0.0);
    }

    #[test]
    fn test_health_score_empty_project() {
        let validation = ProjectValidation {
            issues: vec![],
            total_items: 0,
            trash_items: 0,
        };
        assert_eq!(validation.health_score(), 100.0);
    }

    #[test]
    fn test_health_grade_levels() {
        // Excellent: 100 - 0 = 100
        let excellent = ProjectValidation { issues: vec![], total_items: 1, trash_items: 0 };
        assert_eq!(excellent.health_grade(), "Excellent");

        // Good: 100 - (3*3 + 1*5) = 100 - 14 = 86
        let good = ProjectValidation {
            issues: vec![
                ValidationIssue { severity: Severity::Warning, kind: IssueKind::BrokenLink, item_id: None, message: "w1".to_string() },
                ValidationIssue { severity: Severity::Warning, kind: IssueKind::BrokenLink, item_id: None, message: "w2".to_string() },
                ValidationIssue { severity: Severity::Warning, kind: IssueKind::BrokenLink, item_id: None, message: "w3".to_string() },
                ValidationIssue { severity: Severity::Info, kind: IssueKind::EmptyFolder, item_id: None, message: "i1".to_string() },
                ValidationIssue { severity: Severity::Info, kind: IssueKind::EmptyFolder, item_id: None, message: "i2".to_string() },
                ValidationIssue { severity: Severity::Info, kind: IssueKind::EmptyFolder, item_id: None, message: "i3".to_string() },
                ValidationIssue { severity: Severity::Info, kind: IssueKind::EmptyFolder, item_id: None, message: "i4".to_string() },
                ValidationIssue { severity: Severity::Info, kind: IssueKind::EmptyFolder, item_id: None, message: "i5".to_string() },
            ],
            total_items: 20,
            trash_items: 0,
        };
        assert_eq!(good.health_grade(), "Good");
    }

    // --- New validation check tests ---

    #[test]
    fn test_validate_deep_nesting_not_triggered() {
        let mut binder = Binder::default_structure();
        let mut folder = BinderItem::new_folder("Level 1");
        let mut sub = BinderItem::new_folder("Level 2");
        let mut item = BinderItem::new_text("Leaf");
        if let Some(ref mut doc) = item.document {
            doc.content = "Content.".to_string();
        }
        sub.add_child(item);
        folder.add_child(sub);
        binder.draft.add_child(folder);

        let result = validate_project(&binder);
        assert!(!result.issues.iter().any(|i| i.kind == IssueKind::DeepNesting));
    }

    #[test]
    fn test_validate_deep_nesting_triggered() {
        let mut binder = Binder::default_structure();
        // Build 10 levels deep
        let mut current = BinderItem::new_text("Deep Leaf");
        if let Some(ref mut doc) = current.document {
            doc.content = "Content.".to_string();
        }
        for i in (0..10).rev() {
            let mut folder = BinderItem::new_folder(&format!("Level {}", i));
            folder.add_child(current);
            current = folder;
        }
        binder.draft.add_child(current);

        let result = validate_project(&binder);
        assert!(result.issues.iter().any(|i| i.kind == IssueKind::DeepNesting));
    }

    #[test]
    fn test_validate_inconsistent_metadata_not_triggered_small() {
        // Small projects (< 3 text items) should not trigger
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Single");
        if let Some(ref mut doc) = item.document {
            doc.content = "Content.".to_string();
        }
        binder.draft.add_child(item);

        let result = validate_project(&binder);
        assert!(!result.issues.iter().any(|i| i.kind == IssueKind::InconsistentMetadata));
    }

    #[test]
    fn test_validate_inconsistent_metadata_triggered() {
        use crate::core::metadata::{Label, LabelColor};
        let mut binder = Binder::default_structure();

        // Create 6 items, only 1 with label
        for i in 0..6 {
            let mut item = BinderItem::new_text(&format!("Item {}", i));
            if let Some(ref mut doc) = item.document {
                doc.content = format!("Content for item {}", i);
            }
            if i == 0 {
                item.metadata.label = Some(Label { name: "Important".to_string(), color: LabelColor::Red });
            }
            binder.draft.add_child(item);
        }

        let result = validate_project(&binder);
        assert!(result.issues.iter().any(|i| i.kind == IssueKind::InconsistentMetadata));
    }

    #[test]
    fn test_new_issue_kinds() {
        assert_eq!(IssueKind::DeepNesting.label(), "Deep Nesting");
        assert_eq!(IssueKind::InconsistentMetadata.label(), "Inconsistent Metadata");
        assert!(!IssueKind::DeepNesting.is_auto_fixable());
        assert!(IssueKind::InconsistentMetadata.is_auto_fixable());
        assert!(!IssueKind::DeepNesting.fix_hint().is_empty());
        assert!(!IssueKind::InconsistentMetadata.fix_hint().is_empty());
    }

    // ---- Auto-fix engine tests ----

    #[test]
    fn test_auto_fix_untitled_item() {
        let mut binder = Binder::default_structure();
        let item = BinderItem::new_text("");
        let id = item.id;
        binder.draft.children.push(item);

        // Verify the issue exists
        let pre = validate_project(&binder);
        assert!(pre.issues.iter().any(|i| i.kind == IssueKind::UntitledItem));

        // Apply auto-fix
        let result = auto_fix(&mut binder);
        assert!(result.fix_count() > 0);

        // Verify title was set
        let fixed_item = binder.find_item(&id).unwrap();
        assert_eq!(fixed_item.title, "Untitled Document");
    }

    #[test]
    fn test_auto_fix_empty_document() {
        let mut binder = Binder::default_structure();
        let item = BinderItem::new_text("Empty Doc");
        let id = item.id;
        binder.draft.children.push(item);

        let result = auto_fix(&mut binder);
        let empty_fixes: Vec<_> = result.fixes_applied.iter()
            .filter(|f| f.kind == IssueKind::EmptyDocument)
            .collect();
        assert!(!empty_fixes.is_empty());

        // Verify placeholder was added
        let fixed_item = binder.find_item(&id).unwrap();
        assert!(fixed_item.document.as_ref().unwrap().content.contains("placeholder"));
    }

    #[test]
    fn test_auto_fix_empty_folder() {
        let mut binder = Binder::default_structure();
        let folder = BinderItem::new_folder("Empty Act");
        binder.draft.children.push(folder);

        let result = auto_fix(&mut binder);
        let folder_fixes: Vec<_> = result.fixes_applied.iter()
            .filter(|f| f.kind == IssueKind::EmptyFolder)
            .collect();
        assert!(!folder_fixes.is_empty());
    }

    #[test]
    fn test_auto_fix_skips_non_fixable() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Scene");
        if let Some(ref mut doc) = item.document {
            doc.content = "See [[Missing Chapter]].".to_string();
        }
        binder.draft.children.push(item);

        let result = auto_fix(&mut binder);
        let broken_link_skips: Vec<_> = result.skipped.iter()
            .filter(|i| i.kind == IssueKind::BrokenLink)
            .collect();
        assert!(!broken_link_skips.is_empty());
    }

    #[test]
    fn test_auto_fix_clean_project() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Good Chapter");
        if let Some(ref mut doc) = item.document {
            doc.content = "This chapter has content.".to_string();
        }
        binder.draft.children.push(item);

        let result = auto_fix(&mut binder);
        assert_eq!(result.fix_count(), 0);
    }

    #[test]
    fn test_auto_fix_result_summary() {
        let result = AutoFixResult {
            fixes_applied: vec![
                FixRecord { kind: IssueKind::UntitledItem, item_id: None, description: "fixed".to_string() },
            ],
            skipped: vec![
                ValidationIssue { severity: Severity::Error, kind: IssueKind::DuplicateId, item_id: None, message: "skip".to_string() },
            ],
        };
        let summary = result.summary();
        assert!(summary.contains("1 fixes applied"));
        assert!(summary.contains("1 skipped"));
    }

    #[test]
    fn test_auto_fix_result_empty_summary() {
        let result = AutoFixResult {
            fixes_applied: vec![],
            skipped: vec![],
        };
        assert_eq!(result.summary(), "No fixes applied");
    }

    #[test]
    fn test_fix_issues_of_kind_untitled() {
        let mut binder = Binder::default_structure();
        binder.draft.children.push(BinderItem::new_text(""));
        binder.draft.children.push(BinderItem::new_text(""));

        let fixes = fix_issues_of_kind(&mut binder, &IssueKind::UntitledItem);
        assert_eq!(fixes.len(), 2);
        assert!(fixes.iter().all(|f| f.kind == IssueKind::UntitledItem));
    }

    #[test]
    fn test_fix_issues_of_kind_wrong_kind() {
        let mut binder = Binder::default_structure();
        binder.draft.children.push(BinderItem::new_text("")); // untitled, not broken link

        let fixes = fix_issues_of_kind(&mut binder, &IssueKind::BrokenLink);
        assert!(fixes.is_empty());
    }

    #[test]
    fn test_fix_duplicate_id() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Doc");
        if let Some(ref mut doc) = item.document {
            doc.content = "Content.".to_string();
        }
        let old_id = item.id;
        binder.draft.children.push(item);

        let new_id = fix_duplicate_id(&mut binder, &old_id).unwrap();
        assert_ne!(old_id, new_id);
        // Old ID should no longer be found
        assert!(binder.find_item(&old_id).is_none());
        // New ID should be found
        assert!(binder.find_item(&new_id).is_some());
    }

    #[test]
    fn test_fix_duplicate_id_not_found() {
        let mut binder = Binder::default_structure();
        let fake = Uuid::new_v4();
        assert!(fix_duplicate_id(&mut binder, &fake).is_none());
    }

    #[test]
    fn test_fix_record_has_description() {
        let rec = FixRecord {
            kind: IssueKind::UntitledItem,
            item_id: Some(Uuid::new_v4()),
            description: "Renamed to Untitled Document".to_string(),
        };
        assert!(!rec.description.is_empty());
        assert_eq!(rec.kind, IssueKind::UntitledItem);
        assert!(rec.item_id.is_some());
    }
}
