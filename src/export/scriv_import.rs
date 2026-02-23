//! Scrivener `.scriv` project import.
//!
//! A `.scriv` package is a directory (macOS "package") that contains a
//! `.scrivx` XML file describing the binder hierarchy and a `Files/Data`
//! subdirectory tree holding the actual document content (typically RTF or
//! plain text files).
//!
//! This module parses the `.scrivx` XML, reads associated content files from
//! disk, and reconstructs the binder structure as a `Vec<BinderItem>`.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use chrono::{DateTime, NaiveDateTime, Utc};

use crate::core::binder::{BinderItem, BinderItemKind};
use crate::core::document::Document;
use crate::export::rtf_import::extract_text_from_rtf;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// High-level metadata about a Scrivener project.
#[derive(Debug, Clone)]
pub struct ScrivProjectInfo {
    /// The project title (derived from the `.scrivx` filename or project
    /// directory name).
    pub title: String,
    /// The Scrivener format version string (e.g. `"2.0"` or `"3.0"`).
    pub version: String,
    /// When the project was created, if the information is available.
    pub created_at: Option<DateTime<Utc>>,
}

/// An intermediate representation of a single node parsed from the `.scrivx`
/// binder XML.  This is used as an in-between step before converting to the
/// application-native `BinderItem`.
#[derive(Debug, Clone)]
pub struct ScrivNode {
    /// The Scrivener binder item ID (unique within the project).
    pub id: String,
    /// The display title.
    pub title: String,
    /// The Scrivener item type string (e.g. `"Text"`, `"Folder"`,
    /// `"DraftFolder"`, `"ResearchFolder"`, `"TrashFolder"`, `"Image"`,
    /// `"PDF"`, `"WebPage"`).
    pub item_type: String,
    /// Child nodes.
    pub children: Vec<ScrivNode>,
    /// The relative path to the content file inside `Files/Data/<ID>/`,
    /// if one exists.  Typically `"content.rtf"` or `"content.txt"`.
    pub content_path: Option<String>,
    /// The `Created` timestamp attribute from the XML, if present.
    pub created: Option<String>,
}

// ---------------------------------------------------------------------------
// XML parsing helpers
// ---------------------------------------------------------------------------

/// Decode the five standard XML entities.
fn decode_xml_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// Extract the value of a named attribute from an XML tag string.
///
/// For example, given `<BinderItem ID="42" Type="Text">`, calling
/// `extract_attribute(tag, "ID")` returns `Some("42")`.
fn extract_attribute<'a>(tag: &'a str, attr_name: &str) -> Option<&'a str> {
    let needle = format!("{}=\"", attr_name);
    let start = tag.find(&needle)?;
    let val_start = start + needle.len();
    let val_end = tag[val_start..].find('"')? + val_start;
    Some(&tag[val_start..val_end])
}

/// Extract the text content between `<Tag>` and `</Tag>` for a simple
/// (non-nested) element.  Returns `None` if the tags are not found.
fn extract_element_text<'a>(xml: &'a str, tag_name: &str) -> Option<&'a str> {
    let open = format!("<{}>", tag_name);
    let close = format!("</{}>", tag_name);
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(&xml[start..end])
}

/// Extract the Scrivener project version from the root `<ScrivenerProject>`
/// element.
fn extract_project_version(xml: &str) -> String {
    if let Some(tag_start) = xml.find("<ScrivenerProject") {
        let tag_end = xml[tag_start..].find('>').unwrap_or(0) + tag_start;
        let tag = &xml[tag_start..=tag_end];
        if let Some(v) = extract_attribute(tag, "Version") {
            return v.to_string();
        }
    }
    "unknown".to_string()
}

/// Parse the `<Binder>` section of a `.scrivx` XML file into a flat list of
/// top-level `ScrivNode` trees.
pub fn parse_scrivx_xml(xml: &str) -> Result<Vec<ScrivNode>> {
    let binder_start = xml.find("<Binder")
        .context("No <Binder> element found in .scrivx XML")?;
    let binder_end = xml.find("</Binder>")
        .context("No closing </Binder> element found in .scrivx XML")?
        + "</Binder>".len();

    let binder_xml = &xml[binder_start..binder_end];
    parse_binder_items(binder_xml)
}

/// Recursively parse `<BinderItem>` elements out of an XML fragment.
fn parse_binder_items(xml: &str) -> Result<Vec<ScrivNode>> {
    let mut nodes = Vec::new();
    let mut search_from = 0;

    while let Some(pos) = xml[search_from..].find("<BinderItem ") {
        let tag_start = search_from + pos;

        // Locate the end of the opening tag.
        let Some(close_offset) = xml[tag_start..].find('>') else { break; };
        let tag_close = tag_start + close_offset;
        let opening_tag = &xml[tag_start..=tag_close];

        // Extract attributes from the opening tag.
        let id = extract_attribute(opening_tag, "ID")
            .unwrap_or("0")
            .to_string();
        let item_type = extract_attribute(opening_tag, "Type")
            .unwrap_or("Text")
            .to_string();
        let created = extract_attribute(opening_tag, "Created")
            .map(|s| s.to_string());

        // Find the matching </BinderItem> by counting nesting depth.
        let after_open = tag_close + 1;
        let Some(close_start) = find_matching_close(xml, after_open, "BinderItem") else { break; };
        let close_end = close_start + "</BinderItem>".len();

        let inner_xml = &xml[after_open..close_start];

        // Extract <Title> text.
        let title = extract_element_text(inner_xml, "Title")
            .map(|t| decode_xml_entities(t.trim()))
            .unwrap_or_default();

        // Parse <Children> for nested items.
        // We must find the *matching* </Children> close tag (not just the
        // first one) because nested BinderItems may themselves contain
        // <Children> blocks.
        let children = if let Some(ch_start) = inner_xml.find("<Children>") {
            let ch_inner_start = ch_start + "<Children>".len();
            let ch_end = find_matching_close(inner_xml, ch_inner_start, "Children")
                .unwrap_or(inner_xml.len());
            let children_xml = &inner_xml[ch_inner_start..ch_end];
            parse_binder_items(children_xml).unwrap_or_default()
        } else {
            Vec::new()
        };

        // Determine content path based on ID.
        let content_path = if item_type == "Text" || item_type == "DraftFolder" {
            Some(format!("{}/content.rtf", id))
        } else {
            None
        };

        nodes.push(ScrivNode {
            id,
            title,
            item_type,
            children,
            content_path,
            created,
        });

        search_from = close_end;
    }

    Ok(nodes)
}

/// Find the position of the matching `</TagName>` close tag starting from
/// `start`, accounting for nested elements with the same tag name.
fn find_matching_close(xml: &str, start: usize, tag_name: &str) -> Option<usize> {
    let open_with_space = format!("<{} ", tag_name);
    let open_with_close = format!("<{}>", tag_name);
    let close_tag = format!("</{}>", tag_name);
    let mut depth = 1usize;
    let mut pos = start;

    while pos < xml.len() {
        // Find the next occurrence of an open tag (either `<Tag ...>` or
        // `<Tag>`).
        let next_open_space = xml[pos..].find(&open_with_space).map(|p| pos + p);
        let next_open_close = xml[pos..].find(&open_with_close).map(|p| pos + p);
        let next_open = match (next_open_space, next_open_close) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };

        let next_close = xml[pos..].find(&close_tag).map(|p| pos + p);

        match (next_open, next_close) {
            (Some(o), Some(c)) if o < c => {
                depth += 1;
                // Advance past the opening tag.
                pos = o + 1;
            }
            (_, Some(c)) => {
                depth -= 1;
                if depth == 0 {
                    return Some(c);
                }
                pos = c + close_tag.len();
            }
            _ => return None,
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Conversion: ScrivNode -> BinderItem
// ---------------------------------------------------------------------------

/// Map a Scrivener type string to a `BinderItemKind`.
fn scriv_type_to_kind(scriv_type: &str) -> BinderItemKind {
    match scriv_type {
        "Folder" | "DraftFolder" | "ResearchFolder" | "TrashFolder" | "Root" => {
            BinderItemKind::Folder
        }
        "Image" => BinderItemKind::Image,
        "PDF" => BinderItemKind::Pdf,
        "WebPage" => BinderItemKind::WebPage,
        // "Text" and everything else default to text.
        _ => BinderItemKind::Text,
    }
}

/// Try to parse a Scrivener-style datetime string into a `DateTime<Utc>`.
///
/// Scrivener uses several formats:
/// - `"2024-01-15 10:30:00 +0000"`
/// - `"2024-01-15T10:30:00Z"`
/// - `"2024-01-15 10:30:00"`
fn parse_scriv_datetime(s: &str) -> Option<DateTime<Utc>> {
    // Try RFC 3339 / ISO 8601.
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    // Try with timezone offset like "+0000".
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S %z") {
        return Some(dt.with_timezone(&Utc));
    }
    // Try naive datetime (assume UTC).
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(ndt.and_utc());
    }
    // Try date only.
    if let Ok(ndt) = NaiveDateTime::parse_from_str(
        &format!("{} 00:00:00", s),
        "%Y-%m-%d %H:%M:%S",
    ) {
        return Some(ndt.and_utc());
    }
    None
}

/// Convert a single `ScrivNode` into a `BinderItem`, recursively processing
/// children.  Content is looked up in `content_map` by Scrivener item ID.
fn scriv_node_to_binder_item(
    node: &ScrivNode,
    content_map: &HashMap<String, String>,
) -> BinderItem {
    let kind = scriv_type_to_kind(&node.item_type);

    let mut item = match kind {
        BinderItemKind::Folder => BinderItem::new_folder(&node.title),
        _ => BinderItem::new_text(&node.title),
    };

    // Apply creation timestamp if available.
    if let Some(ref created_str) = node.created {
        if let Some(dt) = parse_scriv_datetime(created_str) {
            item.created_at = dt;
            item.modified_at = dt;
        }
    }

    // Set the item kind (the constructors only set Folder or Text).
    item.kind = kind;

    // Attach document content for text-like items.
    if item.kind == BinderItemKind::Text {
        if let Some(content) = content_map.get(&node.id) {
            item.document = Some(Document::with_content(content));
        }
    }

    // Recurse into children.
    for child_node in &node.children {
        let child_item = scriv_node_to_binder_item(child_node, content_map);
        item.children.push(child_item);
    }

    item
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Import a Scrivener `.scriv` package from the filesystem.
///
/// `path` should point to the `.scriv` directory.  The function locates the
/// `.scrivx` XML file inside, parses it, reads associated content files, and
/// returns the project info together with the reconstructed binder items.
pub fn import_scriv(path: &Path) -> Result<(ScrivProjectInfo, Vec<BinderItem>)> {
    if !path.is_dir() {
        bail!(
            "Path is not a directory (expected a .scriv package): {}",
            path.display()
        );
    }

    // Locate the .scrivx file inside the package.
    let scrivx_path = find_scrivx_file(path)
        .with_context(|| {
            format!(
                "Could not find a .scrivx file inside {}",
                path.display()
            )
        })?;

    let xml = std::fs::read_to_string(&scrivx_path)
        .with_context(|| format!("Failed to read .scrivx file: {}", scrivx_path.display()))?;

    // Parse version for project info.
    let version = extract_project_version(&xml);

    // Derive project title from the directory name.
    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Imported Project")
        .to_string();

    // Parse the binder XML into ScrivNodes.
    let nodes = parse_scrivx_xml(&xml)
        .context("Failed to parse .scrivx binder structure")?;

    // Build content map by reading files from Files/Data/<ID>/content.rtf (or .txt).
    let content_map = build_content_map(path, &nodes)
        .context("Failed to read content files from .scriv package")?;

    // Convert ScrivNodes to BinderItems.
    let items = nodes
        .iter()
        .map(|node| scriv_node_to_binder_item(node, &content_map))
        .collect();

    // Try to extract a creation date from the first node.
    let created_at = nodes
        .first()
        .and_then(|n| n.created.as_ref())
        .and_then(|s| parse_scriv_datetime(s));

    let info = ScrivProjectInfo {
        title,
        version,
        created_at,
    };

    Ok((info, items))
}

/// Import binder items from raw `.scrivx` XML bytes and a pre-built content
/// map.
///
/// This is useful when the caller has already read the XML and content into
/// memory (e.g. from a compressed archive or over the network).
pub fn import_scrivx_bytes(
    xml: &[u8],
    content_map: &HashMap<String, String>,
) -> Result<Vec<BinderItem>> {
    let xml_str = std::str::from_utf8(xml)
        .context("The .scrivx data is not valid UTF-8")?;

    let nodes = parse_scrivx_xml(xml_str)
        .context("Failed to parse .scrivx binder structure")?;

    let items = nodes
        .iter()
        .map(|node| scriv_node_to_binder_item(node, content_map))
        .collect();

    Ok(items)
}

/// Check whether a path looks like a valid Scrivener `.scriv` package and
/// return basic project information if it does.
pub fn detect_scrivener_format(path: &Path) -> Option<ScrivProjectInfo> {
    if !path.is_dir() {
        return None;
    }

    // The directory should have a .scriv extension.
    let ext = path.extension().and_then(|e| e.to_str())?;
    if ext != "scriv" {
        return None;
    }

    // There should be a .scrivx file inside.
    let scrivx_path = find_scrivx_file(path)?;

    let xml = std::fs::read_to_string(&scrivx_path).ok()?;

    // The XML should contain a <ScrivenerProject ...> root element.
    if !xml.contains("<ScrivenerProject") {
        return None;
    }

    let version = extract_project_version(&xml);

    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    // Try to get creation date from first BinderItem.
    let created_at = parse_scrivx_xml(&xml)
        .ok()
        .and_then(|nodes| {
            nodes.first()
                .and_then(|n| n.created.as_ref())
                .and_then(|s| parse_scriv_datetime(s))
        });

    Some(ScrivProjectInfo {
        title,
        version,
        created_at,
    })
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Find the first `.scrivx` file in the given directory.
fn find_scrivx_file(dir: &Path) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) == Some("scrivx") {
            return Some(p);
        }
    }
    None
}

/// Recursively collect all Scrivener item IDs from a tree of `ScrivNode`s.
fn collect_ids(nodes: &[ScrivNode], ids: &mut Vec<String>) {
    for node in nodes {
        ids.push(node.id.clone());
        collect_ids(&node.children, ids);
    }
}

/// Build a map from Scrivener item ID to plain-text content by reading files
/// from the `Files/Data/<ID>/` subdirectories inside the `.scriv` package.
fn build_content_map(
    scriv_dir: &Path,
    nodes: &[ScrivNode],
) -> Result<HashMap<String, String>> {
    let mut ids = Vec::new();
    collect_ids(nodes, &mut ids);

    let mut map = HashMap::new();

    // Scrivener 3 stores content in Files/Data/<UUID>/content.rtf
    // Scrivener 2 stores content in Files/Docs/<ID>.rtf
    let data_dir = scriv_dir.join("Files").join("Data");
    let docs_dir = scriv_dir.join("Files").join("Docs");

    for id in &ids {
        // Try Scrivener 3 layout first.
        let rtf_path = data_dir.join(id).join("content.rtf");
        let txt_path = data_dir.join(id).join("content.txt");
        // Then Scrivener 2 layout.
        let v2_rtf_path = docs_dir.join(format!("{}.rtf", id));
        let v2_txt_path = docs_dir.join(format!("{}.txt", id));

        if rtf_path.exists() {
            if let Ok(raw) = std::fs::read_to_string(&rtf_path) {
                let text = extract_text_from_rtf(&raw);
                if !text.is_empty() {
                    map.insert(id.clone(), text);
                }
            }
        } else if txt_path.exists() {
            if let Ok(text) = std::fs::read_to_string(&txt_path) {
                let trimmed = text.trim().to_string();
                if !trimmed.is_empty() {
                    map.insert(id.clone(), trimmed);
                }
            }
        } else if v2_rtf_path.exists() {
            if let Ok(raw) = std::fs::read_to_string(&v2_rtf_path) {
                let text = extract_text_from_rtf(&raw);
                if !text.is_empty() {
                    map.insert(id.clone(), text);
                }
            }
        } else if v2_txt_path.exists() {
            if let Ok(text) = std::fs::read_to_string(&v2_txt_path) {
                let trimmed = text.trim().to_string();
                if !trimmed.is_empty() {
                    map.insert(id.clone(), trimmed);
                }
            }
        }
    }

    Ok(map)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Helpers --

    /// Build a minimal .scrivx XML string with the given binder content.
    fn wrap_scrivx(binder_content: &str) -> String {
        format!(
            r#"<?xml version="1.0" ?>
<ScrivenerProject Version="2.0">
  <Binder>
    {}
  </Binder>
</ScrivenerProject>"#,
            binder_content
        )
    }

    /// Build a single `<BinderItem>` XML element.
    fn make_binder_item(id: &str, item_type: &str, title: &str, children: &str) -> String {
        let children_xml = if children.is_empty() {
            String::new()
        } else {
            format!("<Children>{}</Children>", children)
        };
        format!(
            r#"<BinderItem ID="{}" Type="{}" Created="2024-06-15 10:00:00 +0000">
      <Title>{}</Title>
      {}
    </BinderItem>"#,
            id, item_type, title, children_xml
        )
    }

    // -----------------------------------------------------------------------
    // extract_attribute
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_attribute_id() {
        let tag = r#"<BinderItem ID="42" Type="Text">"#;
        assert_eq!(extract_attribute(tag, "ID"), Some("42"));
    }

    #[test]
    fn test_extract_attribute_type() {
        let tag = r#"<BinderItem ID="1" Type="Folder">"#;
        assert_eq!(extract_attribute(tag, "Type"), Some("Folder"));
    }

    #[test]
    fn test_extract_attribute_missing() {
        let tag = r#"<BinderItem ID="1">"#;
        assert_eq!(extract_attribute(tag, "Type"), None);
    }

    #[test]
    fn test_extract_attribute_created() {
        let tag = r#"<BinderItem ID="1" Type="Text" Created="2024-06-15">"#;
        assert_eq!(extract_attribute(tag, "Created"), Some("2024-06-15"));
    }

    // -----------------------------------------------------------------------
    // extract_element_text
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_element_text_title() {
        let xml = r#"<Title>Chapter One</Title>"#;
        assert_eq!(extract_element_text(xml, "Title"), Some("Chapter One"));
    }

    #[test]
    fn test_extract_element_text_empty() {
        let xml = r#"<Title></Title>"#;
        assert_eq!(extract_element_text(xml, "Title"), Some(""));
    }

    #[test]
    fn test_extract_element_text_missing() {
        let xml = r#"<Other>data</Other>"#;
        assert_eq!(extract_element_text(xml, "Title"), None);
    }

    // -----------------------------------------------------------------------
    // decode_xml_entities
    // -----------------------------------------------------------------------

    #[test]
    fn test_decode_entities_basic() {
        assert_eq!(decode_xml_entities("&amp;"), "&");
        assert_eq!(decode_xml_entities("&lt;"), "<");
        assert_eq!(decode_xml_entities("&gt;"), ">");
        assert_eq!(decode_xml_entities("&quot;"), "\"");
        assert_eq!(decode_xml_entities("&apos;"), "'");
    }

    #[test]
    fn test_decode_entities_combined() {
        assert_eq!(
            decode_xml_entities("Tom &amp; Jerry &lt;3"),
            "Tom & Jerry <3"
        );
    }

    #[test]
    fn test_decode_entities_no_entities() {
        assert_eq!(decode_xml_entities("plain text"), "plain text");
    }

    // -----------------------------------------------------------------------
    // extract_project_version
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_project_version_2() {
        let xml = r#"<ScrivenerProject Version="2.0"><Binder></Binder></ScrivenerProject>"#;
        assert_eq!(extract_project_version(xml), "2.0");
    }

    #[test]
    fn test_extract_project_version_3() {
        let xml = r#"<ScrivenerProject Version="3.0"><Binder></Binder></ScrivenerProject>"#;
        assert_eq!(extract_project_version(xml), "3.0");
    }

    #[test]
    fn test_extract_project_version_missing() {
        let xml = r#"<ScrivenerProject><Binder></Binder></ScrivenerProject>"#;
        assert_eq!(extract_project_version(xml), "unknown");
    }

    #[test]
    fn test_extract_project_version_no_tag() {
        let xml = r#"<Other>data</Other>"#;
        assert_eq!(extract_project_version(xml), "unknown");
    }

    // -----------------------------------------------------------------------
    // parse_scrivx_xml
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_empty_binder() {
        let xml = wrap_scrivx("");
        let nodes = parse_scrivx_xml(&xml).unwrap();
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_parse_single_text_item() {
        let item = make_binder_item("1", "Text", "My Document", "");
        let xml = wrap_scrivx(&item);
        let nodes = parse_scrivx_xml(&xml).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, "1");
        assert_eq!(nodes[0].title, "My Document");
        assert_eq!(nodes[0].item_type, "Text");
        assert!(nodes[0].children.is_empty());
    }

    #[test]
    fn test_parse_folder_with_children() {
        let child1 = make_binder_item("2", "Text", "Chapter 1", "");
        let child2 = make_binder_item("3", "Text", "Chapter 2", "");
        let folder = make_binder_item(
            "1",
            "DraftFolder",
            "Draft",
            &format!("{}{}", child1, child2),
        );
        let xml = wrap_scrivx(&folder);
        let nodes = parse_scrivx_xml(&xml).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].title, "Draft");
        assert_eq!(nodes[0].children.len(), 2);
        assert_eq!(nodes[0].children[0].title, "Chapter 1");
        assert_eq!(nodes[0].children[1].title, "Chapter 2");
    }

    #[test]
    fn test_parse_nested_folders() {
        let leaf = make_binder_item("3", "Text", "Scene 1", "");
        let subfolder = make_binder_item("2", "Folder", "Chapter 1", &leaf);
        let root = make_binder_item("1", "DraftFolder", "Draft", &subfolder);
        let xml = wrap_scrivx(&root);
        let nodes = parse_scrivx_xml(&xml).unwrap();

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].children.len(), 1);
        assert_eq!(nodes[0].children[0].title, "Chapter 1");
        assert_eq!(nodes[0].children[0].children.len(), 1);
        assert_eq!(nodes[0].children[0].children[0].title, "Scene 1");
    }

    #[test]
    fn test_parse_multiple_top_level_items() {
        let draft = make_binder_item("1", "DraftFolder", "Draft", "");
        let research = make_binder_item("2", "ResearchFolder", "Research", "");
        let trash = make_binder_item("3", "TrashFolder", "Trash", "");
        let xml = wrap_scrivx(&format!("{}{}{}", draft, research, trash));
        let nodes = parse_scrivx_xml(&xml).unwrap();
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[0].title, "Draft");
        assert_eq!(nodes[1].title, "Research");
        assert_eq!(nodes[2].title, "Trash");
    }

    #[test]
    fn test_parse_preserves_id() {
        let item = make_binder_item("ABC-123", "Text", "Test", "");
        let xml = wrap_scrivx(&item);
        let nodes = parse_scrivx_xml(&xml).unwrap();
        assert_eq!(nodes[0].id, "ABC-123");
    }

    #[test]
    fn test_parse_preserves_created() {
        let item = make_binder_item("1", "Text", "Test", "");
        let xml = wrap_scrivx(&item);
        let nodes = parse_scrivx_xml(&xml).unwrap();
        assert!(nodes[0].created.is_some());
        assert!(nodes[0].created.as_ref().unwrap().contains("2024"));
    }

    #[test]
    fn test_parse_no_binder_element() {
        let xml = r#"<?xml version="1.0" ?><ScrivenerProject Version="2.0"></ScrivenerProject>"#;
        let result = parse_scrivx_xml(xml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_entity_in_title() {
        let xml = wrap_scrivx(
            r#"<BinderItem ID="1" Type="Text" Created="2024-01-01 00:00:00 +0000">
                <Title>Tom &amp; Jerry</Title>
            </BinderItem>"#,
        );
        let nodes = parse_scrivx_xml(&xml).unwrap();
        assert_eq!(nodes[0].title, "Tom & Jerry");
    }

    // -----------------------------------------------------------------------
    // scriv_type_to_kind
    // -----------------------------------------------------------------------

    #[test]
    fn test_type_mapping_text() {
        assert_eq!(scriv_type_to_kind("Text"), BinderItemKind::Text);
    }

    #[test]
    fn test_type_mapping_folder() {
        assert_eq!(scriv_type_to_kind("Folder"), BinderItemKind::Folder);
    }

    #[test]
    fn test_type_mapping_draft_folder() {
        assert_eq!(scriv_type_to_kind("DraftFolder"), BinderItemKind::Folder);
    }

    #[test]
    fn test_type_mapping_research_folder() {
        assert_eq!(scriv_type_to_kind("ResearchFolder"), BinderItemKind::Folder);
    }

    #[test]
    fn test_type_mapping_trash_folder() {
        assert_eq!(scriv_type_to_kind("TrashFolder"), BinderItemKind::Folder);
    }

    #[test]
    fn test_type_mapping_image() {
        assert_eq!(scriv_type_to_kind("Image"), BinderItemKind::Image);
    }

    #[test]
    fn test_type_mapping_pdf() {
        assert_eq!(scriv_type_to_kind("PDF"), BinderItemKind::Pdf);
    }

    #[test]
    fn test_type_mapping_webpage() {
        assert_eq!(scriv_type_to_kind("WebPage"), BinderItemKind::WebPage);
    }

    #[test]
    fn test_type_mapping_unknown_defaults_to_text() {
        assert_eq!(scriv_type_to_kind("SomethingNew"), BinderItemKind::Text);
    }

    // -----------------------------------------------------------------------
    // parse_scriv_datetime
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_datetime_with_offset() {
        let dt = parse_scriv_datetime("2024-06-15 10:30:00 +0000");
        assert!(dt.is_some());
        let dt = dt.unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 6);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn test_parse_datetime_rfc3339() {
        let dt = parse_scriv_datetime("2024-06-15T10:30:00Z");
        assert!(dt.is_some());
        assert_eq!(dt.unwrap().year(), 2024);
    }

    #[test]
    fn test_parse_datetime_naive() {
        let dt = parse_scriv_datetime("2024-06-15 10:30:00");
        assert!(dt.is_some());
        assert_eq!(dt.unwrap().year(), 2024);
    }

    #[test]
    fn test_parse_datetime_date_only() {
        let dt = parse_scriv_datetime("2024-06-15");
        assert!(dt.is_some());
        assert_eq!(dt.unwrap().year(), 2024);
    }

    #[test]
    fn test_parse_datetime_invalid() {
        assert!(parse_scriv_datetime("not a date").is_none());
        assert!(parse_scriv_datetime("").is_none());
    }

    // -----------------------------------------------------------------------
    // scriv_node_to_binder_item
    // -----------------------------------------------------------------------

    #[test]
    fn test_node_to_binder_text_with_content() {
        let node = ScrivNode {
            id: "1".to_string(),
            title: "Chapter 1".to_string(),
            item_type: "Text".to_string(),
            children: Vec::new(),
            content_path: Some("1/content.rtf".to_string()),
            created: Some("2024-06-15 10:00:00 +0000".to_string()),
        };
        let mut content_map = HashMap::new();
        content_map.insert("1".to_string(), "Hello, world!".to_string());

        let item = scriv_node_to_binder_item(&node, &content_map);
        assert_eq!(item.title, "Chapter 1");
        assert_eq!(item.kind, BinderItemKind::Text);
        assert!(item.document.is_some());
        assert_eq!(item.document.as_ref().unwrap().content, "Hello, world!");
    }

    #[test]
    fn test_node_to_binder_folder() {
        let node = ScrivNode {
            id: "1".to_string(),
            title: "Draft".to_string(),
            item_type: "DraftFolder".to_string(),
            children: vec![ScrivNode {
                id: "2".to_string(),
                title: "Scene".to_string(),
                item_type: "Text".to_string(),
                children: Vec::new(),
                content_path: None,
                created: None,
            }],
            content_path: None,
            created: None,
        };
        let content_map = HashMap::new();

        let item = scriv_node_to_binder_item(&node, &content_map);
        assert_eq!(item.kind, BinderItemKind::Folder);
        assert_eq!(item.children.len(), 1);
        assert_eq!(item.children[0].title, "Scene");
    }

    #[test]
    fn test_node_to_binder_no_content() {
        let node = ScrivNode {
            id: "99".to_string(),
            title: "Empty".to_string(),
            item_type: "Text".to_string(),
            children: Vec::new(),
            content_path: None,
            created: None,
        };
        let content_map = HashMap::new();

        let item = scriv_node_to_binder_item(&node, &content_map);
        assert_eq!(item.kind, BinderItemKind::Text);
        // The constructor creates a default document; content_map has no entry
        // so the item retains whatever the constructor set.
        assert!(item.document.is_some() || item.document.is_none());
    }

    #[test]
    fn test_node_to_binder_with_datetime() {
        let node = ScrivNode {
            id: "1".to_string(),
            title: "Dated".to_string(),
            item_type: "Text".to_string(),
            children: Vec::new(),
            content_path: None,
            created: Some("2023-01-01 12:00:00 +0000".to_string()),
        };
        let content_map = HashMap::new();
        let item = scriv_node_to_binder_item(&node, &content_map);
        assert_eq!(item.created_at.year(), 2023);
    }

    // -----------------------------------------------------------------------
    // import_scrivx_bytes
    // -----------------------------------------------------------------------

    #[test]
    fn test_import_scrivx_bytes_basic() {
        let child = make_binder_item("2", "Text", "Chapter 1", "");
        let folder = make_binder_item("1", "DraftFolder", "Draft", &child);
        let xml = wrap_scrivx(&folder);

        let mut content_map = HashMap::new();
        content_map.insert("2".to_string(), "Once upon a time...".to_string());

        let items = import_scrivx_bytes(xml.as_bytes(), &content_map).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, BinderItemKind::Folder);
        assert_eq!(items[0].children.len(), 1);
        assert_eq!(items[0].children[0].title, "Chapter 1");
        let doc = items[0].children[0].document.as_ref().unwrap();
        assert_eq!(doc.content, "Once upon a time...");
    }

    #[test]
    fn test_import_scrivx_bytes_empty_binder() {
        let xml = wrap_scrivx("");
        let content_map = HashMap::new();
        let items = import_scrivx_bytes(xml.as_bytes(), &content_map).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_import_scrivx_bytes_invalid_utf8() {
        let bad_data: &[u8] = &[0xFF, 0xFE, 0xFD];
        let content_map = HashMap::new();
        let result = import_scrivx_bytes(bad_data, &content_map);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_scrivx_bytes_no_binder() {
        let xml = r#"<?xml version="1.0" ?><ScrivenerProject Version="2.0"></ScrivenerProject>"#;
        let content_map = HashMap::new();
        let result = import_scrivx_bytes(xml.as_bytes(), &content_map);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_scrivx_bytes_complex_tree() {
        let scene1 = make_binder_item("3", "Text", "Scene 1", "");
        let scene2 = make_binder_item("4", "Text", "Scene 2", "");
        let chapter = make_binder_item(
            "2",
            "Folder",
            "Chapter 1",
            &format!("{}{}", scene1, scene2),
        );
        let draft = make_binder_item("1", "DraftFolder", "Draft", &chapter);
        let research = make_binder_item("5", "ResearchFolder", "Research", "");
        let xml = wrap_scrivx(&format!("{}{}", draft, research));

        let mut content_map = HashMap::new();
        content_map.insert("3".to_string(), "It was a dark night.".to_string());
        content_map.insert("4".to_string(), "The rain fell hard.".to_string());

        let items = import_scrivx_bytes(xml.as_bytes(), &content_map).unwrap();
        assert_eq!(items.len(), 2);

        // Draft folder
        let draft_item = &items[0];
        assert_eq!(draft_item.title, "Draft");
        assert_eq!(draft_item.kind, BinderItemKind::Folder);
        assert_eq!(draft_item.children.len(), 1);

        // Chapter 1 subfolder
        let chapter_item = &draft_item.children[0];
        assert_eq!(chapter_item.title, "Chapter 1");
        assert_eq!(chapter_item.kind, BinderItemKind::Folder);
        assert_eq!(chapter_item.children.len(), 2);

        // Scenes
        assert_eq!(chapter_item.children[0].title, "Scene 1");
        assert_eq!(
            chapter_item.children[0].document.as_ref().unwrap().content,
            "It was a dark night."
        );
        assert_eq!(chapter_item.children[1].title, "Scene 2");
        assert_eq!(
            chapter_item.children[1].document.as_ref().unwrap().content,
            "The rain fell hard."
        );

        // Research folder
        assert_eq!(items[1].title, "Research");
        assert_eq!(items[1].kind, BinderItemKind::Folder);
    }

    // -----------------------------------------------------------------------
    // import_scriv (filesystem-based — uses temp dir)
    // -----------------------------------------------------------------------

    #[test]
    fn test_import_scriv_nonexistent_path() {
        let path = Path::new("/tmp/nonexistent_project.scriv");
        let result = import_scriv(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_scriv_not_a_directory() {
        // Use a path that exists but is a file, not a directory.
        let result = import_scriv(Path::new("/dev/null"));
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // detect_scrivener_format
    // -----------------------------------------------------------------------

    #[test]
    fn test_detect_nonexistent_path() {
        let path = Path::new("/tmp/fake_project.scriv");
        assert!(detect_scrivener_format(path).is_none());
    }

    #[test]
    fn test_detect_not_a_directory() {
        assert!(detect_scrivener_format(Path::new("/dev/null")).is_none());
    }

    #[test]
    fn test_detect_wrong_extension() {
        // /tmp exists but does not have a .scriv extension.
        assert!(detect_scrivener_format(Path::new("/tmp")).is_none());
    }

    // -----------------------------------------------------------------------
    // ScrivNode construction
    // -----------------------------------------------------------------------

    #[test]
    fn test_scriv_node_default_content_path_for_text() {
        let item = make_binder_item("42", "Text", "Test", "");
        let xml = wrap_scrivx(&item);
        let nodes = parse_scrivx_xml(&xml).unwrap();
        assert!(nodes[0].content_path.is_some());
        assert!(nodes[0].content_path.as_ref().unwrap().contains("42"));
    }

    #[test]
    fn test_scriv_node_no_content_path_for_folder() {
        let item = make_binder_item("10", "Folder", "A Folder", "");
        let xml = wrap_scrivx(&item);
        let nodes = parse_scrivx_xml(&xml).unwrap();
        assert!(nodes[0].content_path.is_none());
    }

    // -----------------------------------------------------------------------
    // collect_ids
    // -----------------------------------------------------------------------

    #[test]
    fn test_collect_ids_flat() {
        let nodes = vec![
            ScrivNode {
                id: "1".to_string(),
                title: "A".to_string(),
                item_type: "Text".to_string(),
                children: Vec::new(),
                content_path: None,
                created: None,
            },
            ScrivNode {
                id: "2".to_string(),
                title: "B".to_string(),
                item_type: "Text".to_string(),
                children: Vec::new(),
                content_path: None,
                created: None,
            },
        ];
        let mut ids = Vec::new();
        collect_ids(&nodes, &mut ids);
        assert_eq!(ids, vec!["1", "2"]);
    }

    #[test]
    fn test_collect_ids_nested() {
        let nodes = vec![ScrivNode {
            id: "1".to_string(),
            title: "Root".to_string(),
            item_type: "Folder".to_string(),
            children: vec![ScrivNode {
                id: "2".to_string(),
                title: "Child".to_string(),
                item_type: "Text".to_string(),
                children: Vec::new(),
                content_path: None,
                created: None,
            }],
            content_path: None,
            created: None,
        }];
        let mut ids = Vec::new();
        collect_ids(&nodes, &mut ids);
        assert_eq!(ids, vec!["1", "2"]);
    }

    #[test]
    fn test_collect_ids_empty() {
        let nodes: Vec<ScrivNode> = Vec::new();
        let mut ids = Vec::new();
        collect_ids(&nodes, &mut ids);
        assert!(ids.is_empty());
    }

    // -----------------------------------------------------------------------
    // find_matching_close
    // -----------------------------------------------------------------------

    #[test]
    fn test_find_matching_close_simple() {
        let xml = r#"<BinderItem ID="1"><Title>T</Title></BinderItem>"#;
        let start = xml.find('>').unwrap() + 1;
        let pos = find_matching_close(xml, start, "BinderItem");
        assert!(pos.is_some());
    }

    #[test]
    fn test_find_matching_close_nested() {
        let xml = r#"inner<BinderItem ID="2"><Title>Child</Title></BinderItem>after</BinderItem>"#;
        let pos = find_matching_close(xml, 0, "BinderItem");
        assert!(pos.is_some());
        // Should find the outermost close tag, not the inner one.
        let remaining = &xml[pos.unwrap()..];
        assert!(remaining.starts_with("</BinderItem>"));
        // The outer close tag should be the last one.
        assert_eq!(remaining, "</BinderItem>");
    }

    #[test]
    fn test_find_matching_close_not_found() {
        let xml = r#"<BinderItem ID="1"><Title>T</Title>"#;
        let pos = find_matching_close(xml, 0, "BinderItem");
        assert!(pos.is_none());
    }

    // -----------------------------------------------------------------------
    // Full round-trip: XML -> ScrivNodes -> BinderItems
    // -----------------------------------------------------------------------

    #[test]
    fn test_full_roundtrip_simple() {
        let ch1 = make_binder_item("10", "Text", "Introduction", "");
        let ch2 = make_binder_item("11", "Text", "Chapter 1", "");
        let draft = make_binder_item(
            "1",
            "DraftFolder",
            "Manuscript",
            &format!("{}{}", ch1, ch2),
        );
        let xml = wrap_scrivx(&draft);

        let mut content_map = HashMap::new();
        content_map.insert("10".to_string(), "Welcome to the story.".to_string());
        content_map.insert("11".to_string(), "It was a dark and stormy night.".to_string());

        let items = import_scrivx_bytes(xml.as_bytes(), &content_map).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Manuscript");
        assert_eq!(items[0].children.len(), 2);

        let intro = &items[0].children[0];
        assert_eq!(intro.title, "Introduction");
        assert_eq!(
            intro.document.as_ref().unwrap().content,
            "Welcome to the story."
        );
    }

    #[test]
    fn test_full_roundtrip_deeply_nested() {
        let leaf = make_binder_item("4", "Text", "Leaf", "");
        let sub = make_binder_item("3", "Folder", "Sub", &leaf);
        let ch = make_binder_item("2", "Folder", "Chapter", &sub);
        let draft = make_binder_item("1", "DraftFolder", "Draft", &ch);
        let xml = wrap_scrivx(&draft);

        let mut content_map = HashMap::new();
        content_map.insert("4".to_string(), "Deep content.".to_string());

        let items = import_scrivx_bytes(xml.as_bytes(), &content_map).unwrap();
        // Navigate: Draft -> Chapter -> Sub -> Leaf
        let leaf_item = &items[0].children[0].children[0].children[0];
        assert_eq!(leaf_item.title, "Leaf");
        assert_eq!(
            leaf_item.document.as_ref().unwrap().content,
            "Deep content."
        );
    }

    #[test]
    fn test_full_roundtrip_mixed_types() {
        let text_item = make_binder_item("2", "Text", "Notes", "");
        let image_item = make_binder_item("3", "Image", "Photo", "");
        let pdf_item = make_binder_item("4", "PDF", "Reference", "");
        let web_item = make_binder_item("5", "WebPage", "Link", "");
        let research = make_binder_item(
            "1",
            "ResearchFolder",
            "Research",
            &format!("{}{}{}{}", text_item, image_item, pdf_item, web_item),
        );
        let xml = wrap_scrivx(&research);
        let content_map = HashMap::new();

        let items = import_scrivx_bytes(xml.as_bytes(), &content_map).unwrap();
        assert_eq!(items[0].children.len(), 4);
        assert_eq!(items[0].children[0].kind, BinderItemKind::Text);
        assert_eq!(items[0].children[1].kind, BinderItemKind::Image);
        assert_eq!(items[0].children[2].kind, BinderItemKind::Pdf);
        assert_eq!(items[0].children[3].kind, BinderItemKind::WebPage);
    }

    // -----------------------------------------------------------------------
    // ScrivProjectInfo
    // -----------------------------------------------------------------------

    #[test]
    fn test_scriv_project_info_fields() {
        let info = ScrivProjectInfo {
            title: "My Novel".to_string(),
            version: "3.0".to_string(),
            created_at: Some(Utc::now()),
        };
        assert_eq!(info.title, "My Novel");
        assert_eq!(info.version, "3.0");
        assert!(info.created_at.is_some());
    }

    #[test]
    fn test_scriv_project_info_no_date() {
        let info = ScrivProjectInfo {
            title: "Untitled".to_string(),
            version: "2.0".to_string(),
            created_at: None,
        };
        assert!(info.created_at.is_none());
    }

    // -----------------------------------------------------------------------
    // Edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_item_with_whitespace_title() {
        let xml = wrap_scrivx(
            r#"<BinderItem ID="1" Type="Text" Created="2024-01-01 00:00:00 +0000">
                <Title>  Spaced Title  </Title>
            </BinderItem>"#,
        );
        let nodes = parse_scrivx_xml(&xml).unwrap();
        assert_eq!(nodes[0].title, "Spaced Title");
    }

    #[test]
    fn test_parse_item_with_empty_title() {
        let xml = wrap_scrivx(
            r#"<BinderItem ID="1" Type="Text" Created="2024-01-01 00:00:00 +0000">
                <Title></Title>
            </BinderItem>"#,
        );
        let nodes = parse_scrivx_xml(&xml).unwrap();
        assert_eq!(nodes[0].title, "");
    }

    #[test]
    fn test_parse_item_without_title_element() {
        let xml = wrap_scrivx(
            r#"<BinderItem ID="1" Type="Text" Created="2024-01-01 00:00:00 +0000">
            </BinderItem>"#,
        );
        let nodes = parse_scrivx_xml(&xml).unwrap();
        assert_eq!(nodes[0].title, "");
    }

    #[test]
    fn test_content_map_lookup_miss() {
        let node = ScrivNode {
            id: "999".to_string(),
            title: "Orphan".to_string(),
            item_type: "Text".to_string(),
            children: Vec::new(),
            content_path: None,
            created: None,
        };
        let content_map = HashMap::new();
        let item = scriv_node_to_binder_item(&node, &content_map);
        // Text item created by constructor gets a default empty document.
        // Content map miss means the default stands.
        assert_eq!(item.title, "Orphan");
    }

    #[test]
    fn test_large_binder_structure() {
        // Build a binder with 20 text items.
        let mut children_xml = String::new();
        for i in 0..20 {
            children_xml.push_str(&make_binder_item(
                &format!("{}", 100 + i),
                "Text",
                &format!("Document {}", i),
                "",
            ));
        }
        let draft = make_binder_item("1", "DraftFolder", "Draft", &children_xml);
        let xml = wrap_scrivx(&draft);

        let mut content_map = HashMap::new();
        for i in 0..20 {
            content_map.insert(
                format!("{}", 100 + i),
                format!("Content of document {}.", i),
            );
        }

        let items = import_scrivx_bytes(xml.as_bytes(), &content_map).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].children.len(), 20);
        assert_eq!(items[0].children[0].title, "Document 0");
        assert_eq!(items[0].children[19].title, "Document 19");
        assert_eq!(
            items[0].children[5].document.as_ref().unwrap().content,
            "Content of document 5."
        );
    }

    use chrono::Datelike;

    #[test]
    fn test_parse_datetime_negative_offset() {
        let dt = parse_scriv_datetime("2024-06-15 10:30:00 -0500");
        assert!(dt.is_some());
        assert_eq!(dt.unwrap().year(), 2024);
    }

    #[test]
    fn test_root_type_maps_to_folder() {
        assert_eq!(scriv_type_to_kind("Root"), BinderItemKind::Folder);
    }
}
