use crate::core::binder::{BinderItem, BinderItemKind};
use crate::core::document::Document;
use anyhow::Result;

/// Import a Markdown file and convert it into a binder tree structure.
/// The document is split on headings: each H1 becomes a top-level folder,
/// each H2 becomes a child text item, etc.
pub fn import_markdown(content: &str, title: &str) -> Result<Vec<BinderItem>> {
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    let sections = parse_markdown_sections(content);

    if sections.is_empty() {
        // No headings found — treat the whole file as one document
        let mut item = BinderItem::new_text(title);
        if let Some(ref mut doc) = item.document {
            doc.content = content.to_string();
        }
        return Ok(vec![item]);
    }

    let mut items = Vec::new();
    let mut current_folder: Option<BinderItem> = None;

    for section in &sections {
        match section.level {
            1 => {
                // Flush previous folder
                if let Some(folder) = current_folder.take() {
                    items.push(folder);
                }
                // Start a new top-level folder
                let mut folder = BinderItem::new_folder(&section.title);
                if !section.content.is_empty() {
                    // H1 has direct content — add as a preface item
                    let mut preface = BinderItem::new_text("Introduction");
                    if let Some(ref mut doc) = preface.document {
                        doc.content = section.content.clone();
                    }
                    folder.add_child(preface);
                }
                current_folder = Some(folder);
            }
            2 => {
                let mut item = BinderItem::new_text(&section.title);
                if let Some(ref mut doc) = item.document {
                    doc.content = section.content.clone();
                }
                if let Some(ref mut folder) = current_folder {
                    folder.add_child(item);
                } else {
                    items.push(item);
                }
            }
            _ => {
                // H3+ treated as text with the heading inline
                let heading_prefix = "#".repeat(section.level);
                let full_content = format!("{} {}\n\n{}", heading_prefix, section.title, section.content);
                let mut item = BinderItem::new_text(&section.title);
                if let Some(ref mut doc) = item.document {
                    doc.content = full_content;
                }
                if let Some(ref mut folder) = current_folder {
                    folder.add_child(item);
                } else {
                    items.push(item);
                }
            }
        }
    }

    // Flush the last folder
    if let Some(folder) = current_folder {
        items.push(folder);
    }

    Ok(items)
}

/// A parsed section from a Markdown document
#[derive(Debug, Clone)]
struct MarkdownSection {
    /// Heading level (1-6)
    level: usize,
    /// Heading text
    title: String,
    /// Content below the heading
    content: String,
}

/// Parse a Markdown document into sections based on headings
fn parse_markdown_sections(content: &str) -> Vec<MarkdownSection> {
    let lines: Vec<&str> = content.lines().collect();
    let mut sections = Vec::new();
    let mut current_title = String::new();
    let mut current_level = 0;
    let mut current_content = String::new();
    let mut in_code_block = false;

    for line in &lines {
        // Track code blocks to avoid treating # in code as headings
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            if current_level > 0 {
                current_content.push_str(line);
                current_content.push('\n');
            }
            continue;
        }

        if in_code_block {
            if current_level > 0 {
                current_content.push_str(line);
                current_content.push('\n');
            }
            continue;
        }

        // Check for ATX-style headings (# Heading)
        if let Some((level, title)) = parse_heading(line) {
            // Save previous section
            if current_level > 0 {
                sections.push(MarkdownSection {
                    level: current_level,
                    title: current_title.clone(),
                    content: current_content.trim().to_string(),
                });
            }
            current_level = level;
            current_title = title;
            current_content = String::new();
        } else if current_level > 0 {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Save the last section
    if current_level > 0 {
        sections.push(MarkdownSection {
            level: current_level,
            title: current_title,
            content: current_content.trim().to_string(),
        });
    }

    sections
}

/// Parse an ATX-style heading line (# Title)
fn parse_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim();
    if !trimmed.starts_with('#') {
        return None;
    }

    let level = trimmed.chars().take_while(|c| *c == '#').count();
    if level > 6 {
        return None;
    }

    let title = trimmed[level..].trim().to_string();
    if title.is_empty() {
        return None;
    }

    // Remove trailing # marks (optional in ATX headings)
    let title = title.trim_end_matches('#').trim().to_string();

    Some((level, title))
}

/// Import a Markdown file as a flat list of documents (one per heading)
pub fn import_markdown_flat(content: &str) -> Vec<BinderItem> {
    let sections = parse_markdown_sections(content);
    if sections.is_empty() {
        return Vec::new();
    }

    sections.iter().map(|section| {
        let mut item = BinderItem::new_text(&section.title);
        if let Some(ref mut doc) = item.document {
            doc.content = section.content.clone();
        }
        item
    }).collect()
}

/// Convert a Markdown string to plain text (strip formatting)
pub fn markdown_to_plain_text(markdown: &str) -> String {
    let mut plain = String::new();
    let mut in_code_block = false;

    for line in markdown.lines() {
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block {
            plain.push_str(line);
            plain.push('\n');
            continue;
        }

        // Strip heading markers
        let line = if line.starts_with('#') {
            let heading_end = line.find(|c: char| c != '#').unwrap_or(0);
            line[heading_end..].trim()
        } else {
            line
        };

        // Strip bold/italic markers
        let line = line
            .replace("**", "")
            .replace("__", "")
            .replace("*", "")
            .replace("_", " ");

        // Strip inline code
        let line = strip_inline_code(&line);

        // Strip links [text](url) -> text
        let line = strip_links(&line);

        // Strip images ![alt](url) -> alt
        let line = strip_images(&line);

        // Strip horizontal rules
        if line.trim() == "---" || line.trim() == "***" || line.trim() == "___" {
            plain.push('\n');
            continue;
        }

        plain.push_str(&line);
        plain.push('\n');
    }

    plain.trim().to_string()
}

fn strip_inline_code(text: &str) -> String {
    let mut result = String::new();
    let mut in_code = false;
    for ch in text.chars() {
        if ch == '`' {
            in_code = !in_code;
        } else {
            result.push(ch);
        }
    }
    result
}

fn strip_links(text: &str) -> String {
    let mut result = text.to_string();
    // Simple regex-free link stripping: [text](url) -> text
    while let Some(start) = result.find('[') {
        if let Some(mid) = result[start..].find("](") {
            if let Some(end) = result[start + mid..].find(')') {
                let link_text = &result[start + 1..start + mid].to_string();
                let before = &result[..start].to_string();
                let after = &result[start + mid + end + 1..].to_string();
                result = format!("{}{}{}", before, link_text, after);
                continue;
            }
        }
        break;
    }
    result
}

fn strip_images(text: &str) -> String {
    let mut result = text.to_string();
    while let Some(start) = result.find("![") {
        if let Some(mid) = result[start..].find("](") {
            if let Some(end) = result[start + mid..].find(')') {
                let alt_text = &result[start + 2..start + mid].to_string();
                let before = &result[..start].to_string();
                let after = &result[start + mid + end + 1..].to_string();
                result = format!("{}{}{}", before, alt_text, after);
                continue;
            }
        }
        break;
    }
    result
}

/// Extract YAML front matter from a Markdown file
pub fn extract_front_matter(content: &str) -> Option<FrontMatter> {
    let trimmed = content.trim();
    if !trimmed.starts_with("---") {
        return None;
    }

    let after_first = &trimmed[3..];
    if let Some(end_pos) = after_first.find("---") {
        let yaml_content = &after_first[..end_pos].trim();
        let mut fm = FrontMatter::default();

        for line in yaml_content.lines() {
            let line = line.trim();
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 1..].trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();

                match key.as_str() {
                    "title" => fm.title = Some(value),
                    "author" => fm.author = Some(value),
                    "date" => fm.date = Some(value),
                    "tags" | "keywords" => {
                        fm.tags = value.split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                    "description" | "summary" => fm.description = Some(value),
                    _ => {}
                }
            }
        }

        return Some(fm);
    }

    None
}

/// Parsed YAML front matter
#[derive(Debug, Clone, Default)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
}

/// Strip front matter from Markdown content, returning the body
pub fn strip_front_matter(content: &str) -> &str {
    let trimmed = content.trim();
    if !trimmed.starts_with("---") {
        return content;
    }
    let after_first = &trimmed[3..];
    if let Some(end_pos) = after_first.find("---") {
        return after_first[end_pos + 3..].trim_start();
    }
    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heading() {
        assert_eq!(parse_heading("# Title"), Some((1, "Title".to_string())));
        assert_eq!(parse_heading("## Sub"), Some((2, "Sub".to_string())));
        assert_eq!(parse_heading("### Deep"), Some((3, "Deep".to_string())));
        assert_eq!(parse_heading("Not a heading"), None);
        assert_eq!(parse_heading(""), None);
        assert_eq!(parse_heading("#"), None); // No title text
    }

    #[test]
    fn test_parse_heading_trailing_hashes() {
        assert_eq!(parse_heading("## Title ##"), Some((2, "Title".to_string())));
    }

    #[test]
    fn test_parse_markdown_sections() {
        let md = "# Chapter 1\nContent here.\n## Scene 1\nScene content.\n## Scene 2\nMore content.";
        let sections = parse_markdown_sections(md);
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].level, 1);
        assert_eq!(sections[0].title, "Chapter 1");
        assert_eq!(sections[1].level, 2);
        assert_eq!(sections[1].title, "Scene 1");
    }

    #[test]
    fn test_parse_markdown_sections_code_block() {
        let md = "# Title\n```\n# Not a heading\n```\nReal content.";
        let sections = parse_markdown_sections(md);
        assert_eq!(sections.len(), 1);
        assert!(sections[0].content.contains("# Not a heading"));
    }

    #[test]
    fn test_import_markdown_single() {
        let md = "# My Document\n\nSome content here.";
        let items = import_markdown(md, "Test").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "My Document");
        assert_eq!(items[0].kind, BinderItemKind::Folder);
    }

    #[test]
    fn test_import_markdown_multi_chapter() {
        let md = "# Chapter 1\n## Scene 1\nContent 1.\n## Scene 2\nContent 2.\n# Chapter 2\n## Scene 3\nContent 3.";
        let items = import_markdown(md, "Book").unwrap();
        assert_eq!(items.len(), 2); // Two H1 folders
        assert_eq!(items[0].children.len(), 2); // Two H2 scenes in Ch1
        assert_eq!(items[1].children.len(), 1); // One H2 scene in Ch2
    }

    #[test]
    fn test_import_markdown_no_headings() {
        let md = "Just some plain text without any headings.";
        let items = import_markdown(md, "Plain").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Plain");
        assert_eq!(items[0].kind, BinderItemKind::Text);
    }

    #[test]
    fn test_import_markdown_empty() {
        let items = import_markdown("", "Empty").unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_import_markdown_flat() {
        let md = "# Title 1\nContent 1.\n# Title 2\nContent 2.";
        let items = import_markdown_flat(md);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Title 1");
    }

    #[test]
    fn test_markdown_to_plain_text() {
        let md = "# Heading\n\n**Bold** and *italic* text.\n\n[Link](http://example.com)";
        let plain = markdown_to_plain_text(md);
        assert!(plain.contains("Heading"));
        assert!(plain.contains("Bold"));
        assert!(plain.contains("Link"));
        assert!(!plain.contains("**"));
        assert!(!plain.contains("http://"));
    }

    #[test]
    fn test_markdown_to_plain_text_code_block() {
        let md = "Normal\n```\ncode block\n```\nMore normal";
        let plain = markdown_to_plain_text(md);
        assert!(plain.contains("code block"));
        assert!(plain.contains("Normal"));
    }

    #[test]
    fn test_extract_front_matter() {
        let md = "---\ntitle: \"My Book\"\nauthor: \"Author Name\"\ndate: \"2026-01-01\"\n---\n\n# Content";
        let fm = extract_front_matter(md).unwrap();
        assert_eq!(fm.title, Some("My Book".to_string()));
        assert_eq!(fm.author, Some("Author Name".to_string()));
    }

    #[test]
    fn test_extract_front_matter_none() {
        let fm = extract_front_matter("# Just a heading");
        assert!(fm.is_none());
    }

    #[test]
    fn test_extract_front_matter_with_tags() {
        let md = "---\ntitle: Test\ntags: fiction, fantasy, novel\n---\nBody";
        let fm = extract_front_matter(md).unwrap();
        assert_eq!(fm.tags.len(), 3);
        assert!(fm.tags.contains(&"fiction".to_string()));
    }

    #[test]
    fn test_strip_front_matter() {
        let md = "---\ntitle: Test\n---\n\nBody content here.";
        let body = strip_front_matter(md);
        assert!(body.starts_with("Body"));
        assert!(!body.contains("---"));
    }

    #[test]
    fn test_strip_front_matter_no_fm() {
        let md = "# Just content";
        let body = strip_front_matter(md);
        assert_eq!(body, md);
    }

    #[test]
    fn test_strip_links() {
        assert_eq!(strip_links("[text](http://url.com)"), "text");
        assert_eq!(strip_links("before [link](url) after"), "before link after");
    }

    #[test]
    fn test_strip_images() {
        assert_eq!(strip_images("![alt](image.png)"), "alt");
    }

    #[test]
    fn test_strip_inline_code() {
        assert_eq!(strip_inline_code("some `code` here"), "some code here");
    }
}
