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

use anyhow::{bail, Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};

use crate::core::binder::{BinderItem, BinderItemKind};
use crate::core::document::Document;
use crate::export::rtf_import::extract_text_from_rtf;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// High-level metadata about a Scrivener project.
#[derive(Debug, Clone)]
pub struct ScrivProjectInfo {}

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
    /// The `Created` timestamp attribute from the XML, if present.
    pub created: Option<String>,
}

// ---------------------------------------------------------------------------
// XML parsing helpers
// ---------------------------------------------------------------------------

/// Decode the five standard XML entities.
fn decode_xml_entities(text: &str) -> String {
    super::compiler::decode_xml_entities(text)
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

/// Parse the `<Binder>` section of a `.scrivx` XML file into a flat list of
/// top-level `ScrivNode` trees.
pub fn parse_scrivx_xml(xml: &str) -> Result<Vec<ScrivNode>> {
    let binder_start = xml
        .find("<Binder")
        .context("No <Binder> element found in .scrivx XML")?;
    let binder_end = xml
        .find("</Binder>")
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
        let Some(close_offset) = xml[tag_start..].find('>') else {
            break;
        };
        let tag_close = tag_start + close_offset;
        let opening_tag = &xml[tag_start..=tag_close];

        // Extract attributes from the opening tag.
        let id = extract_attribute(opening_tag, "ID").unwrap_or("0").to_string();
        let item_type = extract_attribute(opening_tag, "Type").unwrap_or("Text").to_string();
        let created = extract_attribute(opening_tag, "Created").map(|s| s.to_string());

        // Find the matching </BinderItem> by counting nesting depth.
        let after_open = tag_close + 1;
        let Some(close_start) = find_matching_close(xml, after_open, "BinderItem") else {
            break;
        };
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
            let ch_end = find_matching_close(inner_xml, ch_inner_start, "Children").unwrap_or(inner_xml.len());
            let children_xml = &inner_xml[ch_inner_start..ch_end];
            parse_binder_items(children_xml).unwrap_or_default()
        } else {
            Vec::new()
        };

        nodes.push(ScrivNode {
            id,
            title,
            item_type,
            children,
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
        "Folder" | "DraftFolder" | "ResearchFolder" | "TrashFolder" | "Root" => BinderItemKind::Folder,
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
    if let Ok(ndt) = NaiveDateTime::parse_from_str(&format!("{} 00:00:00", s), "%Y-%m-%d %H:%M:%S") {
        return Some(ndt.and_utc());
    }
    None
}

/// Convert a single `ScrivNode` into a `BinderItem`, recursively processing
/// children.  Content is looked up in `content_map` by Scrivener item ID.
fn scriv_node_to_binder_item(node: &ScrivNode, content_map: &HashMap<String, String>) -> BinderItem {
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
    let scrivx_path =
        find_scrivx_file(path).with_context(|| format!("Could not find a .scrivx file inside {}", path.display()))?;

    let xml = std::fs::read_to_string(&scrivx_path)
        .with_context(|| format!("Failed to read .scrivx file: {}", scrivx_path.display()))?;

    // Parse the binder XML into ScrivNodes.
    let nodes = parse_scrivx_xml(&xml).context("Failed to parse .scrivx binder structure")?;

    // Build content map by reading files from Files/Data/<ID>/content.rtf (or .txt).
    let content_map = build_content_map(path, &nodes).context("Failed to read content files from .scriv package")?;

    // Convert ScrivNodes to BinderItems.
    let items = nodes
        .iter()
        .map(|node| scriv_node_to_binder_item(node, &content_map))
        .collect();

    let info = ScrivProjectInfo {};

    Ok((info, items))
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
fn build_content_map(scriv_dir: &Path, nodes: &[ScrivNode]) -> Result<HashMap<String, String>> {
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
