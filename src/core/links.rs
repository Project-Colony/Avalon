#![allow(dead_code)] // Methods used by test code
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use super::binder::{Binder, BinderItem};

/// A parsed internal document link
#[derive(Debug, Clone, PartialEq)]
pub struct DocLink {
    /// The raw link text as found in the content (e.g. "Chapter 1")
    pub link_text: String,
    /// Display text override (if using [[Title|display]] syntax)
    pub display_text: Option<String>,
    /// Byte offset in the source document where the link starts
    pub start: usize,
    /// Byte offset in the source document where the link ends
    pub end: usize,
}

/// The result of validating a link
#[derive(Debug, Clone)]
pub struct LinkValidation {
    /// The link being validated
    pub link: DocLink,
    /// The source document that contains the link
    pub source_id: Uuid,
    /// Whether the link resolves to a valid target
    pub status: LinkStatus,
}

/// Status of a validated link
#[derive(Debug, Clone, PartialEq)]
pub enum LinkStatus {
    /// Link resolves to a document
    Valid(Uuid),
    /// Link target not found — broken
    Broken,
    /// Link is ambiguous (multiple documents with the same title)
    Ambiguous(Vec<Uuid>),
}

/// Extract all [[...]] links from a document's content
pub fn extract_links(content: &str) -> Vec<DocLink> {
    let mut links = Vec::new();
    let bytes = content.as_bytes();
    let len = bytes.len();
    let mut pos = 0;

    while pos + 1 < len {
        if bytes[pos] == b'[' && bytes[pos + 1] == b'[' {
            let start = pos;
            let inner_start = pos + 2;
            // Find closing ]]
            if let Some(close_offset) = content[inner_start..].find("]]") {
                let inner = &content[inner_start..inner_start + close_offset];
                let end = inner_start + close_offset + 2;

                // Check for display text: [[Title|display]]
                let (link_text, display_text) = if let Some(pipe_pos) = inner.find('|') {
                    (
                        inner[..pipe_pos].trim().to_string(),
                        Some(inner[pipe_pos + 1..].trim().to_string()),
                    )
                } else {
                    (inner.trim().to_string(), None)
                };

                if !link_text.is_empty() {
                    links.push(DocLink {
                        link_text,
                        display_text,
                        start,
                        end,
                    });
                }
                pos = end;
            } else {
                pos += 2;
            }
        } else {
            pos += 1;
        }
    }

    links
}

/// Build a title-to-IDs index for O(1) link resolution
fn build_title_index(all_items: &[&BinderItem]) -> HashMap<String, Vec<Uuid>> {
    let mut index: HashMap<String, Vec<Uuid>> = HashMap::new();
    for item in all_items {
        index.entry(item.title.clone()).or_default().push(item.id);
    }
    index
}

/// Validate all links in a single document against the binder
pub fn validate_document_links(
    item: &BinderItem,
    binder: &Binder,
) -> Vec<LinkValidation> {
    let all_items = binder.all_items();
    let title_index = build_title_index(&all_items);
    validate_document_links_with_index(item, &title_index)
}

/// Validate all links in a single document using a pre-built title index (avoids repeated traversals)
fn validate_document_links_with_index(
    item: &BinderItem,
    title_index: &HashMap<String, Vec<Uuid>>,
) -> Vec<LinkValidation> {
    let doc = match &item.document {
        Some(d) => d,
        None => return Vec::new(),
    };

    let links = extract_links(&doc.content);

    links.into_iter().map(|link| {
        let matches: Vec<Uuid> = title_index
            .get(&link.link_text)
            .map(|ids| ids.iter().copied().filter(|id| *id != item.id).collect())
            .unwrap_or_default();

        let status = match matches.len() {
            0 => LinkStatus::Broken,
            1 => LinkStatus::Valid(matches[0]),
            _ => LinkStatus::Ambiguous(matches),
        };

        LinkValidation {
            link,
            source_id: item.id,
            status,
        }
    }).collect()
}

/// Validate all links across the entire project
pub fn validate_all_links(binder: &Binder) -> Vec<LinkValidation> {
    let all_items = binder.all_items();
    let title_index = build_title_index(&all_items);
    let mut all_validations = Vec::new();

    for item in &all_items {
        let validations = validate_document_links_with_index(item, &title_index);
        all_validations.extend(validations);
    }

    all_validations
}

/// Get a summary of link health for the project
pub fn link_health_summary(binder: &Binder) -> LinkHealthSummary {
    let all_items = binder.all_items();
    let title_index = build_title_index(&all_items);

    let mut all_validations = Vec::new();
    for item in &all_items {
        let validations = validate_document_links_with_index(item, &title_index);
        all_validations.extend(validations);
    }

    let total = all_validations.len();
    let valid = all_validations.iter().filter(|v| matches!(v.status, LinkStatus::Valid(_))).count();
    let ambiguous = all_validations.iter().filter(|v| matches!(v.status, LinkStatus::Ambiguous(_))).count();

    // Collect broken links for reporting
    let broken_links: Vec<BrokenLink> = all_validations.iter()
        .filter(|v| v.status == LinkStatus::Broken)
        .map(|v| BrokenLink {
            source_id: v.source_id,
            link_text: v.link.link_text.clone(),
            position: v.link.start,
        })
        .collect();

    // Find orphan documents (no incoming links) — reuse all_items from above
    let linked_ids: HashSet<Uuid> = all_validations.iter()
        .filter_map(|v| match &v.status {
            LinkStatus::Valid(id) => Some(*id),
            _ => None,
        })
        .collect();

    let orphans: Vec<Uuid> = all_items.iter()
        .filter(|i| i.document.is_some() && !linked_ids.contains(&i.id))
        .map(|i| i.id)
        .collect();

    LinkHealthSummary {
        total_links: total,
        valid_links: valid,
        broken_links: broken_links.len(),
        ambiguous_links: ambiguous,
        orphan_documents: orphans.len(),
        broken_link_details: broken_links,
        orphan_ids: orphans,
    }
}

/// Summary of link health across a project
#[derive(Debug, Clone)]
pub struct LinkHealthSummary {
    pub total_links: usize,
    pub valid_links: usize,
    pub broken_links: usize,
    pub ambiguous_links: usize,
    pub orphan_documents: usize,
    pub broken_link_details: Vec<BrokenLink>,
    pub orphan_ids: Vec<Uuid>,
}

impl LinkHealthSummary {
    pub fn is_healthy(&self) -> bool {
        self.broken_links == 0 && self.ambiguous_links == 0
    }

    pub fn display(&self) -> String {
        if self.total_links == 0 {
            return "No internal links".to_string();
        }
        let mut parts = vec![
            format!("{} link{}", self.total_links, if self.total_links == 1 { "" } else { "s" }),
        ];
        if self.broken_links > 0 {
            parts.push(format!("{} broken", self.broken_links));
        }
        if self.ambiguous_links > 0 {
            parts.push(format!("{} ambiguous", self.ambiguous_links));
        }
        if self.orphan_documents > 0 {
            parts.push(format!("{} orphan doc{}", self.orphan_documents,
                if self.orphan_documents == 1 { "" } else { "s" }));
        }
        parts.join(", ")
    }

    /// Health score as a percentage (0-100)
    pub fn health_score(&self) -> f64 {
        if self.total_links == 0 {
            return 100.0;
        }
        (self.valid_links as f64 / self.total_links as f64) * 100.0
    }

    /// Health grade label
    pub fn health_grade(&self) -> &str {
        let score = self.health_score();
        if score >= 100.0 { "Excellent" }
        else if score >= 80.0 { "Good" }
        else if score >= 50.0 { "Needs Work" }
        else { "Poor" }
    }

    /// Whether there are broken links that need fixing
    pub fn needs_attention(&self) -> bool {
        self.broken_links > 0 || self.ambiguous_links > 0
    }
}

/// A broken link that needs attention
#[derive(Debug, Clone)]
pub struct BrokenLink {
    pub source_id: Uuid,
    pub link_text: String,
    pub position: usize,
}

/// Suggest possible matches for a broken link title
pub fn suggest_link_targets(broken_title: &str, binder: &Binder) -> Vec<String> {
    let all_items = binder.all_items();
    let lower = broken_title.to_lowercase();

    // Find items whose title is close to the broken link
    let mut suggestions: Vec<(String, usize)> = all_items.iter()
        .filter(|i| i.document.is_some())
        .filter_map(|i| {
            let item_lower = i.title.to_lowercase();
            // Check substring match or prefix match
            if item_lower.contains(&lower) || lower.contains(&item_lower) {
                Some((i.title.clone(), 0))
            } else {
                let dist = edit_distance(&lower, &item_lower);
                if dist <= 3 {
                    Some((i.title.clone(), dist))
                } else {
                    None
                }
            }
        })
        .collect();

    // Deduplicate by title (multiple items can share a title)
    suggestions.sort_by(|(a_title, a_dist), (b_title, b_dist)| a_dist.cmp(b_dist).then(a_title.cmp(b_title)));
    suggestions.dedup_by(|(a, _), (b, _)| a == b);
    suggestions.into_iter().map(|(t, _)| t).take(5).collect()
}

impl DocLink {
    /// Get the display text, falling back to link_text
    pub fn display(&self) -> &str {
        self.display_text.as_deref().unwrap_or(&self.link_text)
    }

    /// Byte length of the link in the original content
    pub fn byte_len(&self) -> usize {
        self.end - self.start
    }

    /// Whether this link uses a display text override
    pub fn has_display_override(&self) -> bool {
        self.display_text.is_some()
    }
}

impl LinkStatus {
    /// Check if the link is valid
    pub fn is_valid(&self) -> bool {
        matches!(self, LinkStatus::Valid(_))
    }

    /// Check if the link is broken
    pub fn is_broken(&self) -> bool {
        matches!(self, LinkStatus::Broken)
    }

    /// Check if the link is ambiguous
    pub fn is_ambiguous(&self) -> bool {
        matches!(self, LinkStatus::Ambiguous(_))
    }

    /// Get the target ID if the link is valid
    pub fn target_id(&self) -> Option<Uuid> {
        match self {
            LinkStatus::Valid(id) => Some(*id),
            _ => None,
        }
    }

    /// Human-readable label
    pub fn label(&self) -> &str {
        match self {
            LinkStatus::Valid(_) => "Valid",
            LinkStatus::Broken => "Broken",
            LinkStatus::Ambiguous(_) => "Ambiguous",
        }
    }
}

impl LinkValidation {
    /// Convenience: is the link valid?
    pub fn is_valid(&self) -> bool {
        self.status.is_valid()
    }

    /// Convenience: is the link broken?
    pub fn is_broken(&self) -> bool {
        self.status.is_broken()
    }
}

/// Count total links in a content string
pub fn count_links(content: &str) -> usize {
    extract_links(content).len()
}

/// Get all unique link targets from a content string
pub fn unique_link_targets(content: &str) -> Vec<String> {
    let links = extract_links(content);
    let mut targets: Vec<String> = links.into_iter().map(|l| l.link_text).collect();
    targets.sort();
    targets.dedup();
    targets
}

/// Replace a link in content with new text
pub fn replace_link(content: &str, old_target: &str, new_target: &str) -> String {
    let pattern = format!("[[{}]]", old_target);
    let replacement = format!("[[{}]]", new_target);
    content.replace(&pattern, &replacement)
}

/// Simple Levenshtein edit distance
fn edit_distance(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();
    let mut dp = vec![vec![0usize; b_len + 1]; a_len + 1];

    for (i, row) in dp.iter_mut().enumerate().take(a_len + 1) {
        row[0] = i;
    }
    for (j, val) in dp[0].iter_mut().enumerate().take(b_len + 1) {
        *val = j;
    }

    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_bytes[i - 1] == b_bytes[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[a_len][b_len]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::binder::{Binder, BinderItem};

    #[test]
    fn test_extract_links_simple() {
        let content = "See [[Chapter 1]] for details.";
        let links = extract_links(content);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].link_text, "Chapter 1");
        assert!(links[0].display_text.is_none());
    }

    #[test]
    fn test_extract_links_with_display_text() {
        let content = "See [[Chapter 1|the first chapter]] for details.";
        let links = extract_links(content);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].link_text, "Chapter 1");
        assert_eq!(links[0].display_text, Some("the first chapter".to_string()));
    }

    #[test]
    fn test_extract_links_multiple() {
        let content = "See [[Intro]] and also [[Conclusion]].";
        let links = extract_links(content);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].link_text, "Intro");
        assert_eq!(links[1].link_text, "Conclusion");
    }

    #[test]
    fn test_extract_links_no_links() {
        let content = "No links here.";
        let links = extract_links(content);
        assert!(links.is_empty());
    }

    #[test]
    fn test_extract_links_empty_brackets() {
        let content = "Empty [[ ]] link.";
        let links = extract_links(content);
        assert!(links.is_empty()); // Empty link text should be skipped
    }

    #[test]
    fn test_extract_links_unclosed() {
        let content = "Broken [[link without closing.";
        let links = extract_links(content);
        assert!(links.is_empty());
    }

    #[test]
    fn test_extract_links_positions() {
        let content = "See [[Chapter 1]].";
        let links = extract_links(content);
        assert_eq!(links[0].start, 4);
        assert_eq!(links[0].end, 17);
        assert_eq!(&content[links[0].start..links[0].end], "[[Chapter 1]]");
    }

    #[test]
    fn test_validate_document_links_valid() {
        let mut binder = Binder::default_structure();

        let mut item1 = BinderItem::new_text("Scene 1");
        if let Some(ref mut doc) = item1.document {
            doc.content = "This links to [[Scene 2]].".to_string();
        }
        let item2 = BinderItem::new_text("Scene 2");

        binder.draft.children.push(item1.clone());
        binder.draft.children.push(item2);

        let validations = validate_document_links(&binder.draft.children[0], &binder);
        assert_eq!(validations.len(), 1);
        assert!(matches!(validations[0].status, LinkStatus::Valid(_)));
    }

    #[test]
    fn test_validate_document_links_broken() {
        let mut binder = Binder::default_structure();

        let mut item = BinderItem::new_text("Scene 1");
        if let Some(ref mut doc) = item.document {
            doc.content = "This links to [[Nonexistent Scene]].".to_string();
        }
        binder.draft.children.push(item.clone());

        let validations = validate_document_links(&binder.draft.children[0], &binder);
        assert_eq!(validations.len(), 1);
        assert_eq!(validations[0].status, LinkStatus::Broken);
    }

    #[test]
    fn test_validate_all_links() {
        let mut binder = Binder::default_structure();

        let mut item1 = BinderItem::new_text("Scene 1");
        if let Some(ref mut doc) = item1.document {
            doc.content = "See [[Scene 2]].".to_string();
        }
        let mut item2 = BinderItem::new_text("Scene 2");
        if let Some(ref mut doc) = item2.document {
            doc.content = "Back to [[Scene 1]].".to_string();
        }

        binder.draft.children.push(item1);
        binder.draft.children.push(item2);

        let validations = validate_all_links(&binder);
        assert_eq!(validations.len(), 2);
        assert!(validations.iter().all(|v| matches!(v.status, LinkStatus::Valid(_))));
    }

    #[test]
    fn test_link_health_summary_healthy() {
        let mut binder = Binder::default_structure();

        let mut item1 = BinderItem::new_text("A");
        if let Some(ref mut doc) = item1.document {
            doc.content = "See [[B]].".to_string();
        }
        let mut item2 = BinderItem::new_text("B");
        if let Some(ref mut doc) = item2.document {
            doc.content = "See [[A]].".to_string();
        }

        binder.draft.children.push(item1);
        binder.draft.children.push(item2);

        let summary = link_health_summary(&binder);
        assert_eq!(summary.total_links, 2);
        assert_eq!(summary.valid_links, 2);
        assert_eq!(summary.broken_links, 0);
        assert!(summary.is_healthy());
    }

    #[test]
    fn test_link_health_summary_broken() {
        let mut binder = Binder::default_structure();

        let mut item = BinderItem::new_text("A");
        if let Some(ref mut doc) = item.document {
            doc.content = "See [[Missing]].".to_string();
        }
        binder.draft.children.push(item);

        let summary = link_health_summary(&binder);
        assert_eq!(summary.broken_links, 1);
        assert!(!summary.is_healthy());
        assert_eq!(summary.broken_link_details[0].link_text, "Missing");
    }

    #[test]
    fn test_link_health_display() {
        let mut binder = Binder::default_structure();

        let mut item = BinderItem::new_text("A");
        if let Some(ref mut doc) = item.document {
            doc.content = "See [[B]] and [[Missing]].".to_string();
        }
        let item2 = BinderItem::new_text("B");
        binder.draft.children.push(item);
        binder.draft.children.push(item2);

        let summary = link_health_summary(&binder);
        let display = summary.display();
        assert!(display.contains("2 links"));
        assert!(display.contains("1 broken"));
    }

    #[test]
    fn test_link_health_no_links() {
        let binder = Binder::default_structure();
        let summary = link_health_summary(&binder);
        assert_eq!(summary.display(), "No internal links");
    }

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("hello", "hello"), 0);
        assert_eq!(edit_distance("", "abc"), 3);
        assert_eq!(edit_distance("abc", ""), 3);
    }

    #[test]
    fn test_suggest_link_targets() {
        let mut binder = Binder::default_structure();
        binder.draft.children.push(BinderItem::new_text("Chapter 1"));
        binder.draft.children.push(BinderItem::new_text("Chapter 2"));
        binder.draft.children.push(BinderItem::new_text("Epilogue"));

        let suggestions = suggest_link_targets("Chapter", &binder);
        assert!(suggestions.len() >= 2);
        assert!(suggestions.contains(&"Chapter 1".to_string()));
        assert!(suggestions.contains(&"Chapter 2".to_string()));
    }

    #[test]
    fn test_suggest_link_targets_typo() {
        let mut binder = Binder::default_structure();
        binder.draft.children.push(BinderItem::new_text("Introduction"));

        let suggestions = suggest_link_targets("Introducton", &binder);
        assert!(suggestions.contains(&"Introduction".to_string()));
    }

    #[test]
    fn test_extract_links_adjacent() {
        let content = "[[A]][[B]]";
        let links = extract_links(content);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].link_text, "A");
        assert_eq!(links[1].link_text, "B");
    }

    #[test]
    fn test_extract_links_whitespace_trimmed() {
        let content = "[[  Chapter 1  ]]";
        let links = extract_links(content);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].link_text, "Chapter 1");
    }

    #[test]
    fn test_validate_ambiguous_link() {
        let mut binder = Binder::default_structure();
        // Two items with the same title
        let mut source = BinderItem::new_text("Source");
        if let Some(ref mut doc) = source.document {
            doc.content = "See [[Duplicate]].".to_string();
        }
        binder.draft.children.push(source);
        binder.draft.children.push(BinderItem::new_text("Duplicate"));
        binder.draft.children.push(BinderItem::new_text("Duplicate"));

        let validations = validate_document_links(&binder.draft.children[0], &binder);
        assert_eq!(validations.len(), 1);
        assert!(matches!(validations[0].status, LinkStatus::Ambiguous(_)));
    }

    #[test]
    fn test_validate_no_document() {
        let binder = Binder::default_structure();
        let folder = BinderItem::new_folder("Folder");
        // Folder has no document — should return empty
        let validations = validate_document_links(&folder, &binder);
        assert!(validations.is_empty());
    }

    #[test]
    fn test_link_health_orphan_detection() {
        let mut binder = Binder::default_structure();
        // Create items where one links to the other but a third is orphaned
        let mut a = BinderItem::new_text("A");
        if let Some(ref mut doc) = a.document {
            doc.content = "See [[B]].".to_string();
        }
        let b = BinderItem::new_text("B");
        let orphan = BinderItem::new_text("Orphan");
        binder.draft.children.push(a);
        binder.draft.children.push(b);
        binder.draft.children.push(orphan);

        let summary = link_health_summary(&binder);
        // Orphan should be detected (A and Orphan are never linked to)
        assert!(summary.orphan_documents > 0);
    }

    #[test]
    fn test_link_health_display_orphans() {
        let mut binder = Binder::default_structure();
        let orphan = BinderItem::new_text("Lone Doc");
        binder.draft.children.push(orphan);

        let summary = link_health_summary(&binder);
        let display = summary.display();
        assert!(display.contains("No internal links"));
    }

    #[test]
    fn test_edit_distance_same() {
        assert_eq!(edit_distance("test", "test"), 0);
    }

    #[test]
    fn test_edit_distance_single_char() {
        assert_eq!(edit_distance("a", "b"), 1);
        assert_eq!(edit_distance("a", "a"), 0);
    }

    #[test]
    fn test_suggest_link_targets_no_match() {
        let mut binder = Binder::default_structure();
        binder.draft.children.push(BinderItem::new_text("Alpha"));

        let suggestions = suggest_link_targets("Completely Different Long Title", &binder);
        // Should not match (edit distance > 3 and no substring match)
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_link_status_equality() {
        assert_eq!(LinkStatus::Broken, LinkStatus::Broken);
        let id = Uuid::new_v4();
        assert_eq!(LinkStatus::Valid(id), LinkStatus::Valid(id));
        assert_ne!(LinkStatus::Broken, LinkStatus::Valid(id));
    }

    #[test]
    fn test_doc_link_equality() {
        let a = DocLink {
            link_text: "Test".to_string(),
            display_text: None,
            start: 0,
            end: 8,
        };
        let b = DocLink {
            link_text: "Test".to_string(),
            display_text: None,
            start: 0,
            end: 8,
        };
        assert_eq!(a, b);
    }

    // DocLink method tests

    #[test]
    fn test_doc_link_display_no_override() {
        let link = DocLink {
            link_text: "Chapter 1".to_string(),
            display_text: None,
            start: 0,
            end: 13,
        };
        assert_eq!(link.display(), "Chapter 1");
        assert!(!link.has_display_override());
    }

    #[test]
    fn test_doc_link_display_with_override() {
        let link = DocLink {
            link_text: "Chapter 1".to_string(),
            display_text: Some("the beginning".to_string()),
            start: 0,
            end: 27,
        };
        assert_eq!(link.display(), "the beginning");
        assert!(link.has_display_override());
    }

    #[test]
    fn test_doc_link_byte_len() {
        let link = DocLink {
            link_text: "Test".to_string(),
            display_text: None,
            start: 4,
            end: 12,
        };
        assert_eq!(link.byte_len(), 8);
    }

    // LinkStatus method tests

    #[test]
    fn test_link_status_is_valid() {
        let valid = LinkStatus::Valid(Uuid::new_v4());
        assert!(valid.is_valid());
        assert!(!valid.is_broken());
        assert!(!valid.is_ambiguous());
    }

    #[test]
    fn test_link_status_is_broken() {
        let broken = LinkStatus::Broken;
        assert!(broken.is_broken());
        assert!(!broken.is_valid());
    }

    #[test]
    fn test_link_status_is_ambiguous() {
        let ambig = LinkStatus::Ambiguous(vec![Uuid::new_v4()]);
        assert!(ambig.is_ambiguous());
        assert!(!ambig.is_valid());
    }

    #[test]
    fn test_link_status_target_id() {
        let id = Uuid::new_v4();
        assert_eq!(LinkStatus::Valid(id).target_id(), Some(id));
        assert_eq!(LinkStatus::Broken.target_id(), None);
        assert_eq!(LinkStatus::Ambiguous(vec![]).target_id(), None);
    }

    #[test]
    fn test_link_status_label() {
        assert_eq!(LinkStatus::Valid(Uuid::new_v4()).label(), "Valid");
        assert_eq!(LinkStatus::Broken.label(), "Broken");
        assert_eq!(LinkStatus::Ambiguous(vec![]).label(), "Ambiguous");
    }

    // LinkValidation convenience tests

    #[test]
    fn test_link_validation_convenience() {
        let validation = LinkValidation {
            link: DocLink { link_text: "X".into(), display_text: None, start: 0, end: 5 },
            source_id: Uuid::new_v4(),
            status: LinkStatus::Broken,
        };
        assert!(validation.is_broken());
        assert!(!validation.is_valid());
    }

    // LinkHealthSummary tests

    #[test]
    fn test_health_score_perfect() {
        let mut binder = Binder::default_structure();
        let mut a = BinderItem::new_text("A");
        if let Some(ref mut doc) = a.document {
            doc.content = "See [[B]].".to_string();
        }
        let mut b = BinderItem::new_text("B");
        if let Some(ref mut doc) = b.document {
            doc.content = "See [[A]].".to_string();
        }
        binder.draft.children.push(a);
        binder.draft.children.push(b);

        let summary = link_health_summary(&binder);
        assert_eq!(summary.health_score(), 100.0);
        assert_eq!(summary.health_grade(), "Excellent");
        assert!(!summary.needs_attention());
    }

    #[test]
    fn test_health_score_no_links() {
        let binder = Binder::default_structure();
        let summary = link_health_summary(&binder);
        assert_eq!(summary.health_score(), 100.0);
        assert_eq!(summary.health_grade(), "Excellent");
    }

    #[test]
    fn test_health_score_with_broken() {
        let mut binder = Binder::default_structure();
        let mut a = BinderItem::new_text("A");
        if let Some(ref mut doc) = a.document {
            doc.content = "See [[B]] and [[Missing]].".to_string();
        }
        let b = BinderItem::new_text("B");
        binder.draft.children.push(a);
        binder.draft.children.push(b);

        let summary = link_health_summary(&binder);
        assert_eq!(summary.health_score(), 50.0);
        assert!(summary.needs_attention());
    }

    #[test]
    fn test_health_grade_levels() {
        // Test via direct construction
        let excellent = LinkHealthSummary {
            total_links: 10, valid_links: 10, broken_links: 0,
            ambiguous_links: 0, orphan_documents: 0,
            broken_link_details: vec![], orphan_ids: vec![],
        };
        assert_eq!(excellent.health_grade(), "Excellent");

        let good = LinkHealthSummary {
            total_links: 10, valid_links: 9, broken_links: 1,
            ambiguous_links: 0, orphan_documents: 0,
            broken_link_details: vec![], orphan_ids: vec![],
        };
        assert_eq!(good.health_grade(), "Good");

        let needs_work = LinkHealthSummary {
            total_links: 10, valid_links: 5, broken_links: 5,
            ambiguous_links: 0, orphan_documents: 0,
            broken_link_details: vec![], orphan_ids: vec![],
        };
        assert_eq!(needs_work.health_grade(), "Needs Work");

        let poor = LinkHealthSummary {
            total_links: 10, valid_links: 2, broken_links: 8,
            ambiguous_links: 0, orphan_documents: 0,
            broken_link_details: vec![], orphan_ids: vec![],
        };
        assert_eq!(poor.health_grade(), "Poor");
    }

    // Utility function tests

    #[test]
    fn test_count_links() {
        assert_eq!(count_links("No links"), 0);
        assert_eq!(count_links("[[A]] and [[B]]"), 2);
        assert_eq!(count_links("[[A]][[B]][[C]]"), 3);
    }

    #[test]
    fn test_unique_link_targets() {
        let targets = unique_link_targets("[[A]] and [[B]] and [[A]] again");
        assert_eq!(targets.len(), 2);
        assert!(targets.contains(&"A".to_string()));
        assert!(targets.contains(&"B".to_string()));
    }

    #[test]
    fn test_unique_link_targets_empty() {
        let targets = unique_link_targets("No links here");
        assert!(targets.is_empty());
    }

    #[test]
    fn test_replace_link() {
        let content = "See [[Old Title]] for details.";
        let result = replace_link(content, "Old Title", "New Title");
        assert_eq!(result, "See [[New Title]] for details.");
        assert!(!result.contains("Old Title"));
    }

    #[test]
    fn test_replace_link_multiple_occurrences() {
        let content = "[[A]] and [[A]] again";
        let result = replace_link(content, "A", "B");
        assert_eq!(result, "[[B]] and [[B]] again");
    }

    #[test]
    fn test_replace_link_no_match() {
        let content = "See [[Chapter 1]].";
        let result = replace_link(content, "Missing", "New");
        assert_eq!(result, content); // Unchanged
    }

    #[test]
    fn test_extract_links_nested_brackets() {
        // Single bracket should not be treated as link
        let content = "Array [0] and [[Link]]";
        let links = extract_links(content);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].link_text, "Link");
    }

    #[test]
    fn test_extract_links_with_special_chars() {
        let content = "[[Scene: The Beginning]]";
        let links = extract_links(content);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].link_text, "Scene: The Beginning");
    }

    #[test]
    fn test_link_health_display_ambiguous() {
        let summary = LinkHealthSummary {
            total_links: 3, valid_links: 1, broken_links: 1,
            ambiguous_links: 1, orphan_documents: 0,
            broken_link_details: vec![], orphan_ids: vec![],
        };
        let display = summary.display();
        assert!(display.contains("3 links"));
        assert!(display.contains("1 broken"));
        assert!(display.contains("1 ambiguous"));
    }

    #[test]
    fn test_link_health_display_singular() {
        let summary = LinkHealthSummary {
            total_links: 1, valid_links: 1, broken_links: 0,
            ambiguous_links: 0, orphan_documents: 1,
            broken_link_details: vec![], orphan_ids: vec![],
        };
        let display = summary.display();
        assert!(display.contains("1 link")); // singular
        assert!(display.contains("1 orphan doc")); // singular
    }

    #[test]
    fn test_edit_distance_empty_strings() {
        assert_eq!(edit_distance("", ""), 0);
    }

    #[test]
    fn test_suggest_link_targets_case_insensitive() {
        let mut binder = Binder::default_structure();
        binder.draft.children.push(BinderItem::new_text("CHAPTER ONE"));

        let suggestions = suggest_link_targets("chapter one", &binder);
        assert!(suggestions.contains(&"CHAPTER ONE".to_string()));
    }
}
