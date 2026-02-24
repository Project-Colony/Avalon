#![allow(dead_code)]
use std::io::{Cursor, Read};
use std::path::Path;

use anyhow::{Context, Result};
use zip::ZipArchive;

use crate::core::binder::BinderItem;
#[cfg(test)]
use crate::core::binder::BinderItemKind;

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
        let Some(close_pos) = xml[tag_start..].find('>') else { break; };
        let content_start = tag_start + close_pos + 1;

        // Check for self-closing tag `/>` — content_start is 1 past '>',
        // so the slice includes '>' and we check if '/' precedes it.
        if xml[tag_start..content_start].ends_with("/>") {
            search_from = content_start;
            continue;
        }

        // Find the matching `</w:t>`.
        let Some(end_pos) = xml[content_start..].find("</w:t>") else { break; };
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
            format!("titre{}", level),     // French locale
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
        let Some(end_pos) = xml[p_start..].find("</w:p>") else { break; };
        let p_end = p_start + end_pos + "</w:p>".len();

        let paragraph_xml = &xml[p_start..p_end];

        // Assemble text from all <w:t> runs inside this paragraph.
        let texts = extract_wt_texts(paragraph_xml);
        let text: String = texts.join("");

        let heading_level = detect_heading_level(paragraph_xml);

        paragraphs.push(DocxParagraph {
            text,
            heading_level,
        });

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
        let pos = match xml[search_from..].find(&open) {
            Some(p) => search_from + p,
            None => return None,
        };

        // The character right after the tag name must be '>' or whitespace
        // (for attributes) to avoid matching `<w:pPr>` when looking for `<w:p>`.
        let after = pos + open.len();
        if after >= xml.len() {
            return None;
        }
        let next_char = xml.as_bytes()[after];
        if next_char == b'>' || next_char == b' ' || next_char == b'/' || next_char == b'\n' || next_char == b'\r' || next_char == b'\t' {
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
    let push_text_item = |text_item: Option<BinderItem>,
                          folder: &mut Option<BinderItem>,
                          root: &mut Vec<BinderItem>| {
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
    let data = std::fs::read(path)
        .with_context(|| format!("Failed to read DOCX file: {}", path.display()))?;

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
    let mut archive = ZipArchive::new(cursor)
        .context("Failed to open data as a ZIP archive (is this a valid DOCX file?)")?;

    let xml = read_document_xml(&mut archive)?;
    let paragraphs = parse_paragraphs(&xml);
    let items = paragraphs_to_binder_items(&paragraphs, title);

    Ok(items)
}

/// Extract all text content from a `.docx` file on disk and return it as a
/// single string with paragraphs separated by newlines.
pub fn extract_text_from_docx(path: &Path) -> Result<String> {
    let data = std::fs::read(path)
        .with_context(|| format!("Failed to read DOCX file: {}", path.display()))?;

    extract_text_from_docx_bytes(&data)
}

/// Extract all text content from `.docx` file bytes and return it as a
/// single string with paragraphs separated by newlines.
pub fn extract_text_from_docx_bytes(data: &[u8]) -> Result<String> {
    let cursor = Cursor::new(data);
    let mut archive = ZipArchive::new(cursor)
        .context("Failed to open data as a ZIP archive (is this a valid DOCX file?)")?;

    let xml = read_document_xml(&mut archive)?;
    let paragraphs = parse_paragraphs(&xml);
    let text = paragraphs_to_text(&paragraphs);

    Ok(text)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // --- Helper: build a minimal DOCX ZIP in memory ---

    /// Create a minimal DOCX ZIP archive in memory with the given
    /// `word/document.xml` content.
    fn make_docx_bytes(document_xml: &str) -> Vec<u8> {
        let buf = Vec::new();
        let cursor = Cursor::new(buf);
        let mut zip = zip::ZipWriter::new(cursor);

        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(document_xml.as_bytes()).unwrap();

        // A minimal [Content_Types].xml is expected by some ZIP readers but
        // not strictly required for our parsing; include it anyway.
        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(b"<?xml version=\"1.0\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"></Types>").unwrap();

        let cursor = zip.finish().unwrap();
        cursor.into_inner()
    }

    /// Wrap paragraph XML in a minimal document body.
    fn wrap_body(paragraphs_xml: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    {}
  </w:body>
</w:document>"#,
            paragraphs_xml
        )
    }

    /// Build a simple `<w:p>` with text and optional heading style.
    fn make_paragraph(text: &str, heading: Option<u8>) -> String {
        let style = match heading {
            Some(level) => format!(
                r#"<w:pPr><w:pStyle w:val="Heading{}"/></w:pPr>"#,
                level
            ),
            None => String::new(),
        };
        format!(
            r#"<w:p>{}<w:r><w:t>{}</w:t></w:r></w:p>"#,
            style, text
        )
    }

    // -----------------------------------------------------------------------
    // extract_wt_texts
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_wt_single() {
        let xml = r#"<w:r><w:t>Hello world</w:t></w:r>"#;
        let texts = extract_wt_texts(xml);
        assert_eq!(texts, vec!["Hello world"]);
    }

    #[test]
    fn test_extract_wt_multiple_runs() {
        let xml = r#"<w:r><w:t>Hello </w:t></w:r><w:r><w:t>world</w:t></w:r>"#;
        let texts = extract_wt_texts(xml);
        assert_eq!(texts, vec!["Hello ", "world"]);
    }

    #[test]
    fn test_extract_wt_preserve_space() {
        let xml = r#"<w:t xml:space="preserve"> leading space</w:t>"#;
        let texts = extract_wt_texts(xml);
        assert_eq!(texts, vec![" leading space"]);
    }

    #[test]
    fn test_extract_wt_empty_tags() {
        let xml = r#"<w:t></w:t>"#;
        let texts = extract_wt_texts(xml);
        assert_eq!(texts, vec![""]);
    }

    #[test]
    fn test_extract_wt_no_tags() {
        let xml = r#"<w:r><w:rPr><w:b/></w:rPr></w:r>"#;
        let texts = extract_wt_texts(xml);
        assert!(texts.is_empty());
    }

    #[test]
    fn test_extract_wt_self_closing() {
        // Self-closing <w:t/> should be skipped without error.
        let xml = r#"<w:t/><w:t>actual text</w:t>"#;
        let texts = extract_wt_texts(xml);
        assert_eq!(texts, vec!["actual text"]);
    }

    #[test]
    fn test_extract_wt_with_entities() {
        let xml = r#"<w:t>Tom &amp; Jerry</w:t>"#;
        let texts = extract_wt_texts(xml);
        assert_eq!(texts, vec!["Tom &amp; Jerry"]);
    }

    // -----------------------------------------------------------------------
    // detect_heading_level
    // -----------------------------------------------------------------------

    #[test]
    fn test_detect_heading1() {
        let xml = r#"<w:pPr><w:pStyle w:val="Heading1"/></w:pPr>"#;
        assert_eq!(detect_heading_level(xml), 1);
    }

    #[test]
    fn test_detect_heading2() {
        let xml = r#"<w:pPr><w:pStyle w:val="Heading2"/></w:pPr>"#;
        assert_eq!(detect_heading_level(xml), 2);
    }

    #[test]
    fn test_detect_heading3_to_6() {
        for level in 3u8..=6 {
            let xml = format!(
                r#"<w:pPr><w:pStyle w:val="Heading{}"/></w:pPr>"#,
                level
            );
            assert_eq!(detect_heading_level(&xml), level);
        }
    }

    #[test]
    fn test_detect_heading_case_insensitive() {
        let xml = r#"<w:pPr><w:pStyle w:val="heading1"/></w:pPr>"#;
        assert_eq!(detect_heading_level(xml), 1);
    }

    #[test]
    fn test_detect_heading_with_space() {
        let xml = r#"<w:pPr><w:pStyle w:val="Heading 2"/></w:pPr>"#;
        assert_eq!(detect_heading_level(xml), 2);
    }

    #[test]
    fn test_detect_heading_title_style() {
        let xml = r#"<w:pPr><w:pStyle w:val="Title"/></w:pPr>"#;
        assert_eq!(detect_heading_level(xml), 1);
    }

    #[test]
    fn test_detect_heading_subtitle_style() {
        let xml = r#"<w:pPr><w:pStyle w:val="Subtitle"/></w:pPr>"#;
        assert_eq!(detect_heading_level(xml), 2);
    }

    #[test]
    fn test_detect_heading_normal_paragraph() {
        let xml = r#"<w:pPr><w:pStyle w:val="Normal"/></w:pPr>"#;
        assert_eq!(detect_heading_level(xml), 0);
    }

    #[test]
    fn test_detect_heading_no_style() {
        let xml = r#"<w:r><w:t>Just text</w:t></w:r>"#;
        assert_eq!(detect_heading_level(xml), 0);
    }

    // -----------------------------------------------------------------------
    // parse_paragraphs
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_single_paragraph() {
        let xml = wrap_body(&make_paragraph("Hello world", None));
        let paras = parse_paragraphs(&xml);
        assert_eq!(paras.len(), 1);
        assert_eq!(paras[0].text, "Hello world");
        assert_eq!(paras[0].heading_level, 0);
    }

    #[test]
    fn test_parse_multiple_paragraphs() {
        let body = format!(
            "{}{}{}",
            make_paragraph("First", None),
            make_paragraph("Second", None),
            make_paragraph("Third", None),
        );
        let xml = wrap_body(&body);
        let paras = parse_paragraphs(&xml);
        assert_eq!(paras.len(), 3);
        assert_eq!(paras[0].text, "First");
        assert_eq!(paras[1].text, "Second");
        assert_eq!(paras[2].text, "Third");
    }

    #[test]
    fn test_parse_paragraph_with_heading() {
        let body = format!(
            "{}{}",
            make_paragraph("Chapter One", Some(1)),
            make_paragraph("Some body text.", None),
        );
        let xml = wrap_body(&body);
        let paras = parse_paragraphs(&xml);
        assert_eq!(paras.len(), 2);
        assert_eq!(paras[0].heading_level, 1);
        assert_eq!(paras[0].text, "Chapter One");
        assert_eq!(paras[1].heading_level, 0);
    }

    #[test]
    fn test_parse_paragraphs_empty_doc() {
        let xml = wrap_body("");
        let paras = parse_paragraphs(&xml);
        assert!(paras.is_empty());
    }

    #[test]
    fn test_parse_paragraph_multi_run() {
        let xml = wrap_body(
            r#"<w:p><w:r><w:t>Hello </w:t></w:r><w:r><w:rPr><w:b/></w:rPr><w:t>bold</w:t></w:r><w:r><w:t> text</w:t></w:r></w:p>"#,
        );
        let paras = parse_paragraphs(&xml);
        assert_eq!(paras.len(), 1);
        assert_eq!(paras[0].text, "Hello bold text");
    }

    // -----------------------------------------------------------------------
    // find_tag_start
    // -----------------------------------------------------------------------

    #[test]
    fn test_find_tag_start_distinguishes_wp_from_wppr() {
        let xml = r#"<w:pPr><w:pStyle/></w:pPr><w:p><w:t>text</w:t></w:p>"#;
        let pos = find_tag_start(xml, "w:p").unwrap();
        // Should find <w:p>, NOT <w:pPr>
        assert!(xml[pos..].starts_with("<w:p>"));
    }

    #[test]
    fn test_find_tag_start_with_attributes() {
        let xml = r#"<w:p w:rsidR="001A2B3C"><w:t>text</w:t></w:p>"#;
        let pos = find_tag_start(xml, "w:p").unwrap();
        assert_eq!(pos, 0);
    }

    #[test]
    fn test_find_tag_start_not_found() {
        let xml = r#"<w:r><w:t>text</w:t></w:r>"#;
        assert!(find_tag_start(xml, "w:p").is_none());
    }

    // -----------------------------------------------------------------------
    // decode_xml_entities
    // -----------------------------------------------------------------------

    #[test]
    fn test_decode_xml_entities_basic() {
        assert_eq!(decode_xml_entities("&amp;"), "&");
        assert_eq!(decode_xml_entities("&lt;"), "<");
        assert_eq!(decode_xml_entities("&gt;"), ">");
        assert_eq!(decode_xml_entities("&quot;"), "\"");
        assert_eq!(decode_xml_entities("&apos;"), "'");
    }

    #[test]
    fn test_decode_xml_entities_combined() {
        let input = "Tom &amp; Jerry &lt;3";
        assert_eq!(decode_xml_entities(input), "Tom & Jerry <3");
    }

    #[test]
    fn test_decode_xml_entities_no_entities() {
        assert_eq!(decode_xml_entities("plain text"), "plain text");
    }

    // -----------------------------------------------------------------------
    // paragraphs_to_text
    // -----------------------------------------------------------------------

    #[test]
    fn test_paragraphs_to_text_basic() {
        let paras = vec![
            DocxParagraph { text: "Line one".into(), heading_level: 0 },
            DocxParagraph { text: "Line two".into(), heading_level: 0 },
        ];
        let text = paragraphs_to_text(&paras);
        assert_eq!(text, "Line one\nLine two");
    }

    #[test]
    fn test_paragraphs_to_text_empty() {
        let paras: Vec<DocxParagraph> = Vec::new();
        let text = paragraphs_to_text(&paras);
        assert_eq!(text, "");
    }

    #[test]
    fn test_paragraphs_to_text_with_entities() {
        let paras = vec![
            DocxParagraph { text: "A &amp; B".into(), heading_level: 0 },
        ];
        let text = paragraphs_to_text(&paras);
        assert_eq!(text, "A & B");
    }

    #[test]
    fn test_paragraphs_to_text_with_blank_paragraphs() {
        let paras = vec![
            DocxParagraph { text: "Before".into(), heading_level: 0 },
            DocxParagraph { text: "".into(), heading_level: 0 },
            DocxParagraph { text: "After".into(), heading_level: 0 },
        ];
        let text = paragraphs_to_text(&paras);
        assert_eq!(text, "Before\n\nAfter");
    }

    // -----------------------------------------------------------------------
    // paragraphs_to_binder_items
    // -----------------------------------------------------------------------

    #[test]
    fn test_binder_no_headings_single_doc() {
        let paras = vec![
            DocxParagraph { text: "First paragraph.".into(), heading_level: 0 },
            DocxParagraph { text: "Second paragraph.".into(), heading_level: 0 },
        ];
        let items = paragraphs_to_binder_items(&paras, "My Doc");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "My Doc");
        assert_eq!(items[0].kind, BinderItemKind::Text);
        let content = items[0].document.as_ref().unwrap().content.as_str();
        assert!(content.contains("First paragraph."));
        assert!(content.contains("Second paragraph."));
    }

    #[test]
    fn test_binder_heading1_creates_folder() {
        let paras = vec![
            DocxParagraph { text: "Chapter One".into(), heading_level: 1 },
            DocxParagraph { text: "Body text here.".into(), heading_level: 0 },
        ];
        let items = paragraphs_to_binder_items(&paras, "Test");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Chapter One");
        assert_eq!(items[0].kind, BinderItemKind::Folder);
    }

    #[test]
    fn test_binder_heading2_nested_in_folder() {
        let paras = vec![
            DocxParagraph { text: "Chapter One".into(), heading_level: 1 },
            DocxParagraph { text: "Scene A".into(), heading_level: 2 },
            DocxParagraph { text: "Scene A content.".into(), heading_level: 0 },
            DocxParagraph { text: "Scene B".into(), heading_level: 2 },
            DocxParagraph { text: "Scene B content.".into(), heading_level: 0 },
        ];
        let items = paragraphs_to_binder_items(&paras, "Book");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, BinderItemKind::Folder);
        assert_eq!(items[0].children.len(), 2);
        assert_eq!(items[0].children[0].title, "Scene A");
        assert_eq!(items[0].children[0].kind, BinderItemKind::Text);
        let content = items[0].children[0].document.as_ref().unwrap().content.as_str();
        assert_eq!(content, "Scene A content.");
    }

    #[test]
    fn test_binder_multiple_h1_folders() {
        let paras = vec![
            DocxParagraph { text: "Part I".into(), heading_level: 1 },
            DocxParagraph { text: "Scene 1".into(), heading_level: 2 },
            DocxParagraph { text: "Content 1.".into(), heading_level: 0 },
            DocxParagraph { text: "Part II".into(), heading_level: 1 },
            DocxParagraph { text: "Scene 2".into(), heading_level: 2 },
            DocxParagraph { text: "Content 2.".into(), heading_level: 0 },
        ];
        let items = paragraphs_to_binder_items(&paras, "Novel");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Part I");
        assert_eq!(items[1].title, "Part II");
        assert_eq!(items[0].children.len(), 1);
        assert_eq!(items[1].children.len(), 1);
    }

    #[test]
    fn test_binder_h2_without_h1_becomes_root() {
        let paras = vec![
            DocxParagraph { text: "Section A".into(), heading_level: 2 },
            DocxParagraph { text: "Content A.".into(), heading_level: 0 },
            DocxParagraph { text: "Section B".into(), heading_level: 2 },
            DocxParagraph { text: "Content B.".into(), heading_level: 0 },
        ];
        let items = paragraphs_to_binder_items(&paras, "Doc");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Section A");
        assert_eq!(items[0].kind, BinderItemKind::Text);
        assert_eq!(items[1].title, "Section B");
    }

    #[test]
    fn test_binder_empty_paragraphs() {
        let paras: Vec<DocxParagraph> = Vec::new();
        let items = paragraphs_to_binder_items(&paras, "Empty");
        assert!(items.is_empty());
    }

    #[test]
    fn test_binder_only_empty_text() {
        let paras = vec![
            DocxParagraph { text: "".into(), heading_level: 0 },
            DocxParagraph { text: "  ".into(), heading_level: 0 },
        ];
        let items = paragraphs_to_binder_items(&paras, "Blank");
        // All whitespace/empty content results in empty items list.
        assert!(items.is_empty());
    }

    #[test]
    fn test_binder_heading_with_empty_title() {
        let paras = vec![
            DocxParagraph { text: "".into(), heading_level: 1 },
            DocxParagraph { text: "Some content.".into(), heading_level: 0 },
        ];
        let items = paragraphs_to_binder_items(&paras, "Test");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Untitled Section");
    }

    // -----------------------------------------------------------------------
    // Public API with mock DOCX data
    // -----------------------------------------------------------------------

    #[test]
    fn test_import_docx_bytes_simple() {
        let body = format!(
            "{}{}",
            make_paragraph("Hello", None),
            make_paragraph("World", None),
        );
        let xml = wrap_body(&body);
        let docx = make_docx_bytes(&xml);

        let items = import_docx_bytes(&docx, "Test Doc").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Test Doc");
        assert_eq!(items[0].kind, BinderItemKind::Text);
        let content = items[0].document.as_ref().unwrap().content.as_str();
        assert!(content.contains("Hello"));
        assert!(content.contains("World"));
    }

    #[test]
    fn test_import_docx_bytes_with_headings() {
        let body = format!(
            "{}{}{}{}{}",
            make_paragraph("Chapter 1", Some(1)),
            make_paragraph("Intro", Some(2)),
            make_paragraph("Intro body text.", None),
            make_paragraph("Chapter 2", Some(1)),
            make_paragraph("Chapter 2 body.", None),
        );
        let xml = wrap_body(&body);
        let docx = make_docx_bytes(&xml);

        let items = import_docx_bytes(&docx, "Novel").unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Chapter 1");
        assert_eq!(items[0].kind, BinderItemKind::Folder);
        assert_eq!(items[0].children.len(), 1);
        assert_eq!(items[0].children[0].title, "Intro");
        assert_eq!(items[1].title, "Chapter 2");
    }

    #[test]
    fn test_extract_text_from_docx_bytes_simple() {
        let body = format!(
            "{}{}",
            make_paragraph("Line one.", None),
            make_paragraph("Line two.", None),
        );
        let xml = wrap_body(&body);
        let docx = make_docx_bytes(&xml);

        let text = extract_text_from_docx_bytes(&docx).unwrap();
        assert!(text.contains("Line one."));
        assert!(text.contains("Line two."));
    }

    #[test]
    fn test_extract_text_from_docx_bytes_with_headings() {
        let body = format!(
            "{}{}",
            make_paragraph("My Heading", Some(1)),
            make_paragraph("Body paragraph.", None),
        );
        let xml = wrap_body(&body);
        let docx = make_docx_bytes(&xml);

        let text = extract_text_from_docx_bytes(&docx).unwrap();
        assert!(text.contains("My Heading"));
        assert!(text.contains("Body paragraph."));
    }

    #[test]
    fn test_import_docx_bytes_invalid_zip() {
        let bad_data = b"this is not a zip file";
        let result = import_docx_bytes(bad_data, "Bad");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_text_invalid_zip() {
        let bad_data = b"not a zip";
        let result = extract_text_from_docx_bytes(bad_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_docx_bytes_missing_document_xml() {
        // Create a ZIP with no word/document.xml
        let buf = Vec::new();
        let cursor = Cursor::new(buf);
        let mut zip = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("other.xml", options).unwrap();
        zip.write_all(b"<root/>").unwrap();
        let cursor = zip.finish().unwrap();
        let data = cursor.into_inner();

        let result = import_docx_bytes(&data, "Missing");
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("word/document.xml"));
    }

    #[test]
    fn test_import_docx_nonexistent_file() {
        let path = Path::new("/tmp/nonexistent_file_that_does_not_exist.docx");
        let result = import_docx(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_text_nonexistent_file() {
        let path = Path::new("/tmp/nonexistent_file_for_text_extraction.docx");
        let result = extract_text_from_docx(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_docx_bytes_empty_document() {
        let xml = wrap_body("");
        let docx = make_docx_bytes(&xml);

        let items = import_docx_bytes(&docx, "Empty").unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_extract_text_empty_document() {
        let xml = wrap_body("");
        let docx = make_docx_bytes(&xml);

        let text = extract_text_from_docx_bytes(&docx).unwrap();
        assert_eq!(text, "");
    }

    #[test]
    fn test_import_docx_bytes_complex_structure() {
        // Build a more complex document with H1, H2, H3, body text
        let body = format!(
            "{}{}{}{}{}{}{}{}",
            make_paragraph("Part One", Some(1)),
            make_paragraph("Chapter 1", Some(2)),
            make_paragraph("This is chapter 1 content.", None),
            make_paragraph("Chapter 2", Some(2)),
            make_paragraph("This is chapter 2 content.", None),
            make_paragraph("Part Two", Some(1)),
            make_paragraph("Chapter 3", Some(2)),
            make_paragraph("This is chapter 3 content.", None),
        );
        let xml = wrap_body(&body);
        let docx = make_docx_bytes(&xml);

        let items = import_docx_bytes(&docx, "Epic").unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Part One");
        assert_eq!(items[0].kind, BinderItemKind::Folder);
        assert_eq!(items[0].children.len(), 2);
        assert_eq!(items[0].children[0].title, "Chapter 1");
        assert_eq!(items[0].children[1].title, "Chapter 2");
        assert_eq!(items[1].title, "Part Two");
        assert_eq!(items[1].children.len(), 1);
    }

    #[test]
    fn test_roundtrip_entities_in_text() {
        let body = make_paragraph("Tom &amp; Jerry &lt;3", None);
        let xml = wrap_body(&body);
        let docx = make_docx_bytes(&xml);

        let text = extract_text_from_docx_bytes(&docx).unwrap();
        assert_eq!(text, "Tom & Jerry <3");
    }

    #[test]
    fn test_heading_level_preserved_in_binder() {
        // H3 and above should still become text items, not folders.
        let paras = vec![
            DocxParagraph { text: "Main".into(), heading_level: 1 },
            DocxParagraph { text: "Sub".into(), heading_level: 3 },
            DocxParagraph { text: "Detail.".into(), heading_level: 0 },
        ];
        let items = paragraphs_to_binder_items(&paras, "Test");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, BinderItemKind::Folder);
        assert_eq!(items[0].children.len(), 1);
        assert_eq!(items[0].children[0].title, "Sub");
        assert_eq!(items[0].children[0].kind, BinderItemKind::Text);
    }

    #[test]
    fn test_multiple_text_runs_concatenated() {
        // Ensure multiple <w:r><w:t> elements within one paragraph are joined.
        let para_xml = r#"<w:p>
            <w:r><w:t>Hello </w:t></w:r>
            <w:r><w:rPr><w:b/></w:rPr><w:t>beautiful</w:t></w:r>
            <w:r><w:t xml:space="preserve"> world</w:t></w:r>
        </w:p>"#;
        let xml = wrap_body(para_xml);
        let paras = parse_paragraphs(&xml);
        assert_eq!(paras.len(), 1);
        assert_eq!(paras[0].text, "Hello beautiful world");
    }
}
