#![allow(dead_code)] // Methods used by test code
use std::io::{Cursor, Read};
use std::path::Path;

use anyhow::{Context, Result};
use zip::ZipArchive;

use crate::core::binder::BinderItem;

// ---------------------------------------------------------------------------
// Internal XML parsing helpers
// ---------------------------------------------------------------------------

/// A parsed paragraph from the DOCX XML.
#[derive(Debug, Clone)]
struct DocxParagraph {
    /// The text content assembled from all `<w:t>` elements.
    text: String,
    /// The heading level (0 = normal paragraph, 1-6 = Heading1-Heading6).
    heading_level: u8,
}

/// Extract the raw XML from `word/document.xml` inside a ZIP archive that has
/// already been opened.
fn read_document_xml<R: Read + std::io::Seek>(archive: &mut ZipArchive<R>) -> Result<String> {
    let mut file = archive
        .by_name("word/document.xml")
        .context("DOCX archive does not contain word/document.xml")?;
    let mut xml = String::new();
    file.read_to_string(&mut xml)
        .context("Failed to read word/document.xml")?;
    Ok(xml)
}

/// Extract all text content between `<w:t ...>` and `</w:t>` tags in a single
/// XML fragment.  Handles both `<w:t>` and `<w:t xml:space="preserve">`.
fn extract_wt_texts(xml: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut search_from = 0;

    while let Some(pos) = xml[search_from..].find("<w:t") {
        let tag_start = search_from + pos;

        // Locate the closing `>` of the opening tag.
        let Some(close_pos) = xml[tag_start..].find('>') else {
            break;
        };
        let content_start = tag_start + close_pos + 1;

        // Check for self-closing tag `/>` — content_start is 1 past '>',
        // so the slice includes '>' and we check if '/' precedes it.
        if xml[tag_start..content_start].ends_with("/>") {
            search_from = content_start;
            continue;
        }

        // Find the matching `</w:t>`.
        let Some(end_pos) = xml[content_start..].find("</w:t>") else {
            break;
        };
        let content_end = content_start + end_pos;

        let text = &xml[content_start..content_end];
        results.push(text.to_string());

        search_from = content_end + "</w:t>".len();
    }

    results
}

/// Detect the heading level from a `<w:pPr>` block by looking for
/// `<w:pStyle w:val="HeadingN"/>` patterns (Heading1 through Heading6).
/// Also recognises some alternative style names used by various DOCX
/// producers: `heading 1`, `Titre1`, etc.  Returns 0 for normal paragraphs.
fn detect_heading_level(paragraph_xml: &str) -> u8 {
    // Look for <w:pStyle w:val="..."/>
    let needle = "<w:pStyle";
    let style_start = match paragraph_xml.find(needle) {
        Some(pos) => pos,
        None => return 0,
    };

    // Extract the w:val attribute value.
    let val_needle = "w:val=\"";
    let val_start = match paragraph_xml[style_start..].find(val_needle) {
        Some(pos) => style_start + pos + val_needle.len(),
        None => return 0,
    };

    let val_end = match paragraph_xml[val_start..].find('"') {
        Some(pos) => val_start + pos,
        None => return 0,
    };

    let style_val = &paragraph_xml[val_start..val_end];

    // Normalize to lowercase for comparison.
    let lower = style_val.to_lowercase();

    // Match "heading1" .. "heading6", "heading 1" .. "heading 6"
    for level in 1u8..=6 {
        let patterns = [
            format!("heading{}", level),
            format!("heading {}", level),
            format!("titre{}", level), // French locale
            format!("titre {}", level),
        ];
        for pattern in &patterns {
            if lower == *pattern {
                return level;
            }
        }
    }

    // Also check for TOC heading, Title, Subtitle — treat Title as heading 1.
    if lower == "title" {
        return 1;
    }
    if lower == "subtitle" {
        return 2;
    }

    0
}

/// Parse the document XML into a list of paragraphs.  Each `<w:p>` element
/// becomes one `DocxParagraph` with its assembled text and heading level.
fn parse_paragraphs(xml: &str) -> Vec<DocxParagraph> {
    let mut paragraphs = Vec::new();
    let mut search_from = 0;

    while let Some(pos) = find_tag_start(&xml[search_from..], "w:p") {
        let p_start = search_from + pos;

        // Find the matching </w:p>.
        let Some(end_pos) = xml[p_start..].find("</w:p>") else {
            break;
        };
        let p_end = p_start + end_pos + "</w:p>".len();

        let paragraph_xml = &xml[p_start..p_end];

        // Assemble text from all <w:t> runs inside this paragraph.
        let texts = extract_wt_texts(paragraph_xml);
        let text: String = texts.join("");

        let heading_level = detect_heading_level(paragraph_xml);

        paragraphs.push(DocxParagraph { text, heading_level });

        search_from = p_end;
    }

    paragraphs
}

/// Find the start position of an XML tag with the given name.
/// This correctly distinguishes `<w:p>` / `<w:p ...>` from `<w:pPr>` etc.
fn find_tag_start(xml: &str, tag_name: &str) -> Option<usize> {
    let open = format!("<{}", tag_name);
    let mut search_from = 0;

    loop {
        let pos = search_from + xml[search_from..].find(&open)?;

        // The character right after the tag name must be '>' or whitespace
        // (for attributes) to avoid matching `<w:pPr>` when looking for `<w:p>`.
        let after = pos + open.len();
        if after >= xml.len() {
            return None;
        }
        let next_char = xml.as_bytes()[after];
        if next_char == b'>'
            || next_char == b' '
            || next_char == b'/'
            || next_char == b'\n'
            || next_char == b'\r'
            || next_char == b'\t'
        {
            return Some(pos);
        }

        search_from = after;
    }
}

/// Decode basic XML entities (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`).
fn decode_xml_entities(text: &str) -> String {
    super::compiler::decode_xml_entities(text)
}

/// Build a flat text string from the parsed paragraphs, joining each non-empty
/// paragraph with a newline separator.
fn paragraphs_to_text(paragraphs: &[DocxParagraph]) -> String {
    let mut lines: Vec<String> = Vec::new();
    for p in paragraphs {
        let decoded = decode_xml_entities(&p.text);
        // Include empty paragraphs as blank lines to preserve spacing.
        lines.push(decoded);
    }

    // Join all lines, then trim trailing whitespace.
    let joined = lines.join("\n");
    let trimmed = joined.trim().to_string();
    trimmed
}

/// Build a binder item tree from parsed paragraphs.
///
/// Strategy:
/// - If no headings are present, all paragraphs become the content of a
///   single text item with the given `title`.
/// - If headings are present:
///   - Heading1 creates a **folder** (top-level structural unit).
///   - Heading2+ creates a **text item** that is nested inside the most
///     recent Heading1 folder (or placed at the root if no Heading1 has
///     been seen yet).
///   - Normal paragraphs accumulate as content for the most recently
///     opened heading item.
fn paragraphs_to_binder_items(paragraphs: &[DocxParagraph], title: &str) -> Vec<BinderItem> {
    let has_headings = paragraphs.iter().any(|p| p.heading_level > 0);

    if !has_headings {
        // No structure detected — single document.
        let text = paragraphs_to_text(paragraphs);
        if text.is_empty() {
            return Vec::new();
        }
        let mut item = BinderItem::new_text(title);
        if let Some(ref mut doc) = item.document {
            doc.content = text;
        }
        return vec![item];
    }

    // We have headings — build a structured tree.
    let mut items: Vec<BinderItem> = Vec::new();
    let mut current_folder: Option<BinderItem> = None;
    let mut current_text_item: Option<BinderItem> = None;
    let mut content_buffer: Vec<String> = Vec::new();

    // Helper closure: flush accumulated content into the current text item.
    let flush_content = |item: &mut Option<BinderItem>, buffer: &mut Vec<String>| {
        if let Some(ref mut text_item) = item {
            let content = buffer.join("\n").trim().to_string();
            if !content.is_empty() {
                if let Some(ref mut doc) = text_item.document {
                    doc.content = decode_xml_entities(&content);
                }
            }
        }
        buffer.clear();
    };

    // Helper: push the current text item into the current folder or root.
    let push_text_item =
        |text_item: Option<BinderItem>, folder: &mut Option<BinderItem>, root: &mut Vec<BinderItem>| {
            if let Some(item) = text_item {
                if let Some(ref mut f) = folder {
                    f.add_child(item);
                } else {
                    root.push(item);
                }
            }
        };

    for para in paragraphs {
        match para.heading_level {
            1 => {
                // Flush any in-progress text item.
                flush_content(&mut current_text_item, &mut content_buffer);
                push_text_item(current_text_item.take(), &mut current_folder, &mut items);

                // Flush previous folder.
                if let Some(folder) = current_folder.take() {
                    items.push(folder);
                }

                // Start a new folder.
                let decoded_title = decode_xml_entities(&para.text);
                let folder_title = if decoded_title.is_empty() {
                    "Untitled Section".to_string()
                } else {
                    decoded_title
                };
                current_folder = Some(BinderItem::new_folder(&folder_title));
            }
            2..=6 => {
                // Flush any in-progress text item.
                flush_content(&mut current_text_item, &mut content_buffer);
                push_text_item(current_text_item.take(), &mut current_folder, &mut items);

                // Start a new text item.
                let decoded_title = decode_xml_entities(&para.text);
                let item_title = if decoded_title.is_empty() {
                    "Untitled".to_string()
                } else {
                    decoded_title
                };
                current_text_item = Some(BinderItem::new_text(&item_title));
            }
            _ => {
                // Normal paragraph — accumulate content.
                content_buffer.push(decode_xml_entities(&para.text));
            }
        }
    }

    // Flush remaining state.
    flush_content(&mut current_text_item, &mut content_buffer);
    push_text_item(current_text_item.take(), &mut current_folder, &mut items);
    if let Some(folder) = current_folder.take() {
        items.push(folder);
    }

    items
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Import a `.docx` file from disk.
///
/// The file is read as a ZIP archive, `word/document.xml` is extracted and
/// parsed, and the paragraphs / headings are converted into a binder item
/// tree.  Heading1 elements become folders; Heading2+ become text items
/// nested inside them.
pub fn import_docx(path: &Path) -> Result<Vec<BinderItem>> {
    let data = std::fs::read(path).with_context(|| format!("Failed to read DOCX file: {}", path.display()))?;

    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Imported Document")
        .to_string();

    import_docx_bytes(&data, &title)
}

/// Import a `.docx` file from raw bytes in memory.
///
/// Works the same as [`import_docx`] but accepts the file contents directly
/// instead of a file path.  The `title` is used as the default document name
/// when the file contains no headings.
pub fn import_docx_bytes(data: &[u8], title: &str) -> Result<Vec<BinderItem>> {
    let cursor = Cursor::new(data);
    let mut archive =
        ZipArchive::new(cursor).context("Failed to open data as a ZIP archive (is this a valid DOCX file?)")?;

    let xml = read_document_xml(&mut archive)?;
    let paragraphs = parse_paragraphs(&xml);
    let items = paragraphs_to_binder_items(&paragraphs, title);

    Ok(items)
}

/// Extract all text content from a `.docx` file on disk and return it as a
/// single string with paragraphs separated by newlines.
pub fn extract_text_from_docx(path: &Path) -> Result<String> {
    let data = std::fs::read(path).with_context(|| format!("Failed to read DOCX file: {}", path.display()))?;

    extract_text_from_docx_bytes(&data)
}

/// Extract all text content from `.docx` file bytes and return it as a
/// single string with paragraphs separated by newlines.
pub fn extract_text_from_docx_bytes(data: &[u8]) -> Result<String> {
    let cursor = Cursor::new(data);
    let mut archive =
        ZipArchive::new(cursor).context("Failed to open data as a ZIP archive (is this a valid DOCX file?)")?;

    let xml = read_document_xml(&mut archive)?;
    let paragraphs = parse_paragraphs(&xml);
    let text = paragraphs_to_text(&paragraphs);

    Ok(text)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
