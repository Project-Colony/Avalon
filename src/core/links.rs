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

/// Validate all links in a single document against the binder
pub fn validate_document_links(
    item: &BinderItem,
    binder: &Binder,
) -> Vec<LinkValidation> {
    let doc = match &item.document {
        Some(d) => d,
        None => return Vec::new(),
    };

    let links = extract_links(&doc.content);
    let all_items = binder.all_items();

    links.into_iter().map(|link| {
        let matches: Vec<Uuid> = all_items.iter()
            .filter(|i| i.title == link.link_text && i.id != item.id)
            .map(|i| i.id)
            .collect();

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
    let mut all_validations = Vec::new();

    for item in binder.all_items() {
        let validations = validate_document_links(item, binder);
        all_validations.extend(validations);
    }

    all_validations
}

/// Get a summary of link health for the project
pub fn link_health_summary(binder: &Binder) -> LinkHealthSummary {
    let validations = validate_all_links(binder);

    let total = validations.len();
    let valid = validations.iter().filter(|v| matches!(v.status, LinkStatus::Valid(_))).count();
    let broken = validations.iter().filter(|v| v.status == LinkStatus::Broken).count();
    let ambiguous = validations.iter().filter(|v| matches!(v.status, LinkStatus::Ambiguous(_))).count();

    // Collect broken links for reporting
    let broken_links: Vec<BrokenLink> = validations.iter()
        .filter(|v| v.status == LinkStatus::Broken)
        .map(|v| BrokenLink {
            source_id: v.source_id,
            link_text: v.link.link_text.clone(),
            position: v.link.start,
        })
        .collect();

    // Find orphan documents (no incoming links)
    let all_items = binder.all_items();
    let linked_ids: std::collections::HashSet<Uuid> = validations.iter()
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

    suggestions.sort_by_key(|(_, dist)| *dist);
    suggestions.into_iter().map(|(t, _)| t).take(5).collect()
}

/// Simple Levenshtein edit distance
fn edit_distance(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();
    let mut dp = vec![vec![0usize; b_len + 1]; a_len + 1];

    for i in 0..=a_len {
        dp[i][0] = i;
    }
    for j in 0..=b_len {
        dp[0][j] = j;
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
}
