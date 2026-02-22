use crate::core::binder::{BinderItem, BinderItemKind};
use crate::core::document::Document;
use anyhow::Result;
use chrono::{DateTime, Utc};

/// A heading extracted from HTML content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlHeading {
    /// Heading level (1-6, corresponding to h1-h6).
    pub level: u8,
    /// The text content of the heading.
    pub text: String,
}

/// Metadata about an imported web page.
#[derive(Debug, Clone)]
pub struct WebPageMetadata {
    /// The URL the page was fetched from.
    pub url: String,
    /// The page title (from the <title> tag or provided).
    pub title: String,
    /// When the page was fetched/imported.
    pub fetched_at: DateTime<Utc>,
    /// Word count of the extracted plain text.
    pub word_count: usize,
}

/// Strip all HTML tags from a string, keeping only the text content between tags.
///
/// This performs a simple state-machine parse: anything between `<` and `>` is
/// considered a tag and removed. Content outside tags is preserved as-is.
pub fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    result
}

/// Decode common HTML entities into their plain-text equivalents.
///
/// Handles named entities (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, `&nbsp;`)
/// as well as numeric character references in decimal (`&#NNN;`) and
/// hexadecimal (`&#xHH;`) form.
pub fn decode_html_entities(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '&' {
            // Collect entity content up to ';'
            let mut entity = String::new();
            let mut found_semicolon = false;
            // Limit entity length to avoid runaway consumption
            for _ in 0..12 {
                match chars.peek() {
                    Some(&';') => {
                        chars.next();
                        found_semicolon = true;
                        break;
                    }
                    Some(_) => {
                        entity.push(chars.next().unwrap());
                    }
                    None => break,
                }
            }

            if found_semicolon {
                match entity.as_str() {
                    "amp" => result.push('&'),
                    "lt" => result.push('<'),
                    "gt" => result.push('>'),
                    "quot" => result.push('"'),
                    "apos" => result.push('\''),
                    "nbsp" => result.push(' '),
                    _ if entity.starts_with('#') => {
                        let numeric_part = &entity[1..];
                        let code_point = if let Some(hex) = numeric_part.strip_prefix('x').or_else(|| numeric_part.strip_prefix('X')) {
                            u32::from_str_radix(hex, 16).ok()
                        } else {
                            numeric_part.parse::<u32>().ok()
                        };
                        if let Some(cp) = code_point {
                            if let Some(c) = char::from_u32(cp) {
                                result.push(c);
                            } else {
                                // Invalid code point, reproduce original
                                result.push('&');
                                result.push('#');
                                result.push_str(numeric_part);
                                result.push(';');
                            }
                        } else {
                            result.push('&');
                            result.push_str(&entity);
                            result.push(';');
                        }
                    }
                    _ => {
                        // Unknown entity, reproduce as-is
                        result.push('&');
                        result.push_str(&entity);
                        result.push(';');
                    }
                }
            } else {
                // No semicolon found, output '&' and whatever was consumed
                result.push('&');
                result.push_str(&entity);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Extract the content of the `<title>` tag from an HTML document.
///
/// Returns `None` if no `<title>` tag is found. The search is case-insensitive.
/// The returned title has HTML entities decoded and is trimmed.
pub fn extract_title_from_html(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start_tag = "<title>";
    let end_tag = "</title>";

    let start = lower.find(start_tag)?;
    let content_start = start + start_tag.len();
    let end = lower[content_start..].find(end_tag)?;

    let raw_title = &html[content_start..content_start + end];
    let title = decode_html_entities(raw_title.trim());

    if title.is_empty() {
        None
    } else {
        Some(title)
    }
}

/// Extract all headings (h1-h6) from HTML content.
///
/// Returns a vector of `HtmlHeading` structs preserving document order.
/// Heading text has HTML tags stripped and entities decoded.
pub fn extract_headings(html: &str) -> Vec<HtmlHeading> {
    let mut headings = Vec::new();
    let lower = html.to_lowercase();
    let mut search_pos = 0;

    while search_pos < lower.len() {
        // Look for opening heading tags <h1> through <h6>
        let mut best_match: Option<(usize, u8)> = None;

        for level in 1u8..=6 {
            let open_tag = format!("<h{}", level);
            if let Some(pos) = lower[search_pos..].find(&open_tag) {
                let abs_pos = search_pos + pos;
                // Verify the tag is properly formed (next char after <hN should be > or space for attributes)
                let after_tag = abs_pos + open_tag.len();
                if after_tag < lower.len() {
                    let next_ch = lower.as_bytes()[after_tag];
                    if next_ch == b'>' || next_ch == b' ' || next_ch == b'\t' || next_ch == b'\n' {
                        match best_match {
                            None => best_match = Some((abs_pos, level)),
                            Some((best_pos, _)) if abs_pos < best_pos => {
                                best_match = Some((abs_pos, level));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        let (tag_start, level) = match best_match {
            Some(m) => m,
            None => break,
        };

        // Find the end of the opening tag
        let open_end = match lower[tag_start..].find('>') {
            Some(pos) => tag_start + pos + 1,
            None => { search_pos = tag_start + 1; continue; }
        };

        // Find the closing tag
        let close_tag = format!("</h{}>", level);
        let close_start = match lower[open_end..].find(&close_tag) {
            Some(pos) => open_end + pos,
            None => { search_pos = open_end; continue; }
        };

        let raw_content = &html[open_end..close_start];
        let text = decode_html_entities(&strip_html_tags(raw_content)).trim().to_string();

        if !text.is_empty() {
            headings.push(HtmlHeading { level, text });
        }

        search_pos = close_start + close_tag.len();
    }

    headings
}

/// Convert HTML content to plain text with structural whitespace.
///
/// This handles common block-level elements (`<p>`, `<br>`, `<li>`, `<div>`,
/// headings, `<blockquote>`, `<hr>`) by inserting appropriate newlines.
/// All HTML tags are removed and entities are decoded.
pub fn html_to_plain_text(html: &str) -> String {
    let mut text = html.to_string();

    // Remove <script> and <style> blocks entirely
    text = remove_tag_block(&text, "script");
    text = remove_tag_block(&text, "style");

    // Insert newlines for block-level elements before stripping tags
    // Paragraph breaks
    text = replace_tag_case_insensitive(&text, "</p>", "\n\n");
    text = replace_tag_case_insensitive(&text, "<p>", "");
    text = replace_tag_case_insensitive(&text, "<p ", ""); // <p class="...">

    // Line breaks
    text = replace_tag_case_insensitive(&text, "<br>", "\n");
    text = replace_tag_case_insensitive(&text, "<br/>", "\n");
    text = replace_tag_case_insensitive(&text, "<br />", "\n");

    // List items
    text = replace_tag_case_insensitive(&text, "<li>", "  - ");
    text = replace_tag_case_insensitive(&text, "</li>", "\n");

    // Lists themselves get spacing
    text = replace_tag_case_insensitive(&text, "<ul>", "\n");
    text = replace_tag_case_insensitive(&text, "</ul>", "\n");
    text = replace_tag_case_insensitive(&text, "<ol>", "\n");
    text = replace_tag_case_insensitive(&text, "</ol>", "\n");

    // Divs
    text = replace_tag_case_insensitive(&text, "</div>", "\n");

    // Headings get paragraph spacing
    for level in 1..=6 {
        let close = format!("</h{}>", level);
        text = replace_tag_case_insensitive(&text, &close, "\n\n");
    }

    // Blockquote
    text = replace_tag_case_insensitive(&text, "<blockquote>", "\n");
    text = replace_tag_case_insensitive(&text, "</blockquote>", "\n");

    // Horizontal rule
    text = replace_tag_case_insensitive(&text, "<hr>", "\n---\n");
    text = replace_tag_case_insensitive(&text, "<hr/>", "\n---\n");
    text = replace_tag_case_insensitive(&text, "<hr />", "\n---\n");

    // Strip remaining tags
    text = strip_html_tags(&text);

    // Decode entities
    text = decode_html_entities(&text);

    // Clean up excessive whitespace
    // Collapse multiple blank lines to at most two newlines
    let mut cleaned = String::with_capacity(text.len());
    let mut consecutive_newlines = 0;
    for ch in text.chars() {
        if ch == '\n' {
            consecutive_newlines += 1;
            if consecutive_newlines <= 2 {
                cleaned.push('\n');
            }
        } else if ch == '\r' {
            // Skip carriage returns
        } else {
            consecutive_newlines = 0;
            cleaned.push(ch);
        }
    }

    // Trim each line and remove leading/trailing whitespace from the whole result
    let lines: Vec<&str> = cleaned.lines().map(|l| l.trim()).collect();
    let result = lines.join("\n");

    result.trim().to_string()
}

/// Import HTML content into a structured binder tree based on headings.
///
/// H1 headings become folders, and content under each heading becomes a text
/// document child. If no headings are found, the entire content is imported
/// as a single text item. Content between headings of the same or higher
/// level is grouped under the preceding heading.
pub fn import_html_content(html: &str, title: &str) -> Result<Vec<BinderItem>> {
    let plain_text = html_to_plain_text(html);

    if plain_text.trim().is_empty() {
        return Ok(Vec::new());
    }

    let headings = extract_headings(html);

    if headings.is_empty() {
        // No headings: import as a single document
        let mut item = BinderItem::new_text(title);
        if let Some(ref mut doc) = item.document {
            doc.content = plain_text;
        }
        return Ok(vec![item]);
    }

    // Split the plain text by heading text to create sections
    // We use the headings to structure the content
    let sections = split_html_into_sections(html, &headings);
    let mut items: Vec<BinderItem> = Vec::new();
    let mut current_folder: Option<BinderItem> = None;

    for section in &sections {
        match section.level {
            1 => {
                // Flush previous folder
                if let Some(folder) = current_folder.take() {
                    items.push(folder);
                }
                let mut folder = BinderItem::new_folder(&section.title);
                if !section.content.is_empty() {
                    let mut preface = BinderItem::new_text("Introduction");
                    if let Some(ref mut doc) = preface.document {
                        doc.content = section.content.clone();
                    }
                    folder.add_child(preface);
                }
                current_folder = Some(folder);
            }
            _ => {
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
        }
    }

    // Flush the last folder
    if let Some(folder) = current_folder {
        items.push(folder);
    }

    Ok(items)
}

/// Import HTML content as a single flat document without structural splitting.
///
/// The entire HTML body is converted to plain text and stored as one document.
/// The document's notes field is populated with the source HTML for reference.
pub fn import_html_as_single_document(html: &str, title: &str) -> BinderItem {
    let plain_text = html_to_plain_text(html);

    let mut item = BinderItem::new_text(title);
    if let Some(ref mut doc) = item.document {
        doc.content = plain_text;
        doc.notes = format!("Imported from HTML source.\nOriginal length: {} bytes", html.len());
    }
    item
}

/// Create a `BinderItem` with `WebPage` kind from HTML content and a URL.
///
/// The item's kind is set to `BinderItemKind::WebPage`, and the document
/// content is the extracted plain text. The notes field stores the source URL,
/// fetch time, and word count metadata. The synopsis is set to a content preview.
pub fn create_web_page_item(html: &str, url: &str) -> BinderItem {
    let plain_text = html_to_plain_text(html);
    let page_title = extract_title_from_html(html)
        .unwrap_or_else(|| title_from_url(url));

    let metadata = WebPageMetadata {
        url: url.to_string(),
        title: page_title.clone(),
        fetched_at: Utc::now(),
        word_count: plain_text.split_whitespace().count(),
    };

    let mut item = BinderItem::new_text(&page_title);
    item.kind = BinderItemKind::WebPage;

    if let Some(ref mut doc) = item.document {
        doc.content = plain_text;
        doc.notes = format!(
            "Source URL: {}\nFetched: {}\nWord count: {}",
            metadata.url,
            metadata.fetched_at.format("%Y-%m-%d %H:%M:%S UTC"),
            metadata.word_count,
        );
    }

    // Set synopsis to a preview of the content
    let preview_len = 200;
    let content = item.document.as_ref().map(|d| &d.content).cloned().unwrap_or_default();
    if content.len() > preview_len {
        item.synopsis = format!("{}...", content[..preview_len].trim_end());
    } else {
        item.synopsis = content;
    }

    item
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// A parsed section from an HTML document (heading + content below it).
#[derive(Debug, Clone)]
struct HtmlSection {
    level: u8,
    title: String,
    content: String,
}

/// Split HTML into sections by extracting text between headings.
fn split_html_into_sections(html: &str, headings: &[HtmlHeading]) -> Vec<HtmlSection> {
    let lower = html.to_lowercase();
    let mut sections = Vec::new();

    // Find positions of each heading in the original HTML
    let mut heading_positions: Vec<(usize, usize, &HtmlHeading)> = Vec::new();
    let mut search_from = 0;

    for heading in headings {
        let open_tag = format!("<h{}", heading.level);
        if let Some(rel_pos) = lower[search_from..].find(&open_tag) {
            let abs_pos = search_from + rel_pos;
            // Find end of closing tag
            let close_tag = format!("</h{}>", heading.level);
            if let Some(close_rel) = lower[abs_pos..].find(&close_tag) {
                let end_pos = abs_pos + close_rel + close_tag.len();
                heading_positions.push((abs_pos, end_pos, heading));
                search_from = end_pos;
            }
        }
    }

    // Extract content between headings
    for (i, &(_, heading_end, heading)) in heading_positions.iter().enumerate() {
        let content_end = if i + 1 < heading_positions.len() {
            heading_positions[i + 1].0
        } else {
            html.len()
        };

        let raw_content = &html[heading_end..content_end];
        let plain_content = html_to_plain_text(raw_content);

        sections.push(HtmlSection {
            level: heading.level,
            title: heading.text.clone(),
            content: plain_content.trim().to_string(),
        });
    }

    sections
}

/// Remove entire tag blocks (e.g., <script>...</script>, <style>...</style>).
fn remove_tag_block(html: &str, tag: &str) -> String {
    let mut result = html.to_string();
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);

    loop {
        let lower = result.to_lowercase();
        let start = match lower.find(&open) {
            Some(pos) => pos,
            None => break,
        };
        let end = match lower[start..].find(&close) {
            Some(pos) => start + pos + close.len(),
            None => break,
        };
        result = format!("{}{}", &result[..start], &result[end..]);
    }

    result
}

/// Case-insensitive tag replacement.
fn replace_tag_case_insensitive(text: &str, tag: &str, replacement: &str) -> String {
    let tag_lower = tag.to_lowercase();
    let text_lower = text.to_lowercase();
    let mut result = String::with_capacity(text.len());
    let mut pos = 0;

    while pos < text.len() {
        if let Some(found) = text_lower[pos..].find(&tag_lower) {
            result.push_str(&text[pos..pos + found]);
            result.push_str(replacement);
            pos += found + tag.len();
        } else {
            result.push_str(&text[pos..]);
            break;
        }
    }

    result
}

/// Extract a reasonable title from a URL.
fn title_from_url(url: &str) -> String {
    // Strip protocol
    let without_protocol = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);

    // Take the domain part
    let domain = without_protocol.split('/').next().unwrap_or(without_protocol);

    // Strip www. prefix
    let domain = domain.strip_prefix("www.").unwrap_or(domain);

    if domain.is_empty() {
        "Web Page".to_string()
    } else {
        domain.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // strip_html_tags tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_strip_simple_tags() {
        assert_eq!(strip_html_tags("<p>Hello</p>"), "Hello");
    }

    #[test]
    fn test_strip_nested_tags() {
        assert_eq!(
            strip_html_tags("<div><p>Hello <b>World</b></p></div>"),
            "Hello World"
        );
    }

    #[test]
    fn test_strip_tags_with_attributes() {
        assert_eq!(
            strip_html_tags("<a href=\"http://example.com\">Link</a>"),
            "Link"
        );
    }

    #[test]
    fn test_strip_tags_empty_input() {
        assert_eq!(strip_html_tags(""), "");
    }

    #[test]
    fn test_strip_tags_no_tags() {
        assert_eq!(strip_html_tags("Just plain text"), "Just plain text");
    }

    #[test]
    fn test_strip_self_closing_tags() {
        assert_eq!(strip_html_tags("Line 1<br/>Line 2"), "Line 1Line 2");
    }

    #[test]
    fn test_strip_tags_preserves_whitespace() {
        assert_eq!(strip_html_tags("<p>Hello</p> <p>World</p>"), "Hello World");
    }

    // -----------------------------------------------------------------------
    // decode_html_entities tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_decode_amp() {
        assert_eq!(decode_html_entities("Tom &amp; Jerry"), "Tom & Jerry");
    }

    #[test]
    fn test_decode_lt_gt() {
        assert_eq!(decode_html_entities("1 &lt; 2 &gt; 0"), "1 < 2 > 0");
    }

    #[test]
    fn test_decode_quot_apos() {
        assert_eq!(
            decode_html_entities("&quot;Hello&quot; &apos;World&apos;"),
            "\"Hello\" 'World'"
        );
    }

    #[test]
    fn test_decode_nbsp() {
        assert_eq!(decode_html_entities("Hello&nbsp;World"), "Hello World");
    }

    #[test]
    fn test_decode_numeric_decimal() {
        // &#65; = 'A'
        assert_eq!(decode_html_entities("&#65;BC"), "ABC");
    }

    #[test]
    fn test_decode_numeric_hex() {
        // &#x41; = 'A'
        assert_eq!(decode_html_entities("&#x41;BC"), "ABC");
    }

    #[test]
    fn test_decode_numeric_hex_uppercase() {
        // &#X41; = 'A'
        assert_eq!(decode_html_entities("&#X41;BC"), "ABC");
    }

    #[test]
    fn test_decode_multiple_entities() {
        assert_eq!(
            decode_html_entities("&lt;div&gt;&amp;&lt;/div&gt;"),
            "<div>&</div>"
        );
    }

    #[test]
    fn test_decode_no_entities() {
        assert_eq!(decode_html_entities("Hello World"), "Hello World");
    }

    #[test]
    fn test_decode_ampersand_without_semicolon() {
        // Bare ampersand not followed by a valid entity should pass through
        assert_eq!(decode_html_entities("A & B"), "A & B");
    }

    #[test]
    fn test_decode_unknown_entity() {
        assert_eq!(decode_html_entities("&foobar;"), "&foobar;");
    }

    #[test]
    fn test_decode_unicode_entity() {
        // &#169; = copyright sign
        assert_eq!(decode_html_entities("&#169;"), "\u{00A9}");
    }

    // -----------------------------------------------------------------------
    // extract_title_from_html tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_title_basic() {
        let html = "<html><head><title>My Page</title></head><body></body></html>";
        assert_eq!(extract_title_from_html(html), Some("My Page".to_string()));
    }

    #[test]
    fn test_extract_title_case_insensitive() {
        let html = "<HTML><HEAD><TITLE>Upper Case</TITLE></HEAD></HTML>";
        assert_eq!(extract_title_from_html(html), Some("Upper Case".to_string()));
    }

    #[test]
    fn test_extract_title_with_entities() {
        let html = "<title>Tom &amp; Jerry</title>";
        assert_eq!(
            extract_title_from_html(html),
            Some("Tom & Jerry".to_string())
        );
    }

    #[test]
    fn test_extract_title_missing() {
        let html = "<html><body><h1>No title tag here</h1></body></html>";
        assert_eq!(extract_title_from_html(html), None);
    }

    #[test]
    fn test_extract_title_empty() {
        let html = "<title></title>";
        assert_eq!(extract_title_from_html(html), None);
    }

    #[test]
    fn test_extract_title_whitespace_only() {
        let html = "<title>   </title>";
        assert_eq!(extract_title_from_html(html), None);
    }

    // -----------------------------------------------------------------------
    // extract_headings tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_headings_basic() {
        let html = "<h1>Main Title</h1><p>Content</p><h2>Subtitle</h2>";
        let headings = extract_headings(html);
        assert_eq!(headings.len(), 2);
        assert_eq!(headings[0].level, 1);
        assert_eq!(headings[0].text, "Main Title");
        assert_eq!(headings[1].level, 2);
        assert_eq!(headings[1].text, "Subtitle");
    }

    #[test]
    fn test_extract_headings_all_levels() {
        let html = "<h1>H1</h1><h2>H2</h2><h3>H3</h3><h4>H4</h4><h5>H5</h5><h6>H6</h6>";
        let headings = extract_headings(html);
        assert_eq!(headings.len(), 6);
        for (i, h) in headings.iter().enumerate() {
            assert_eq!(h.level, (i + 1) as u8);
        }
    }

    #[test]
    fn test_extract_headings_with_attributes() {
        let html = r#"<h1 class="title" id="main">Title</h1>"#;
        let headings = extract_headings(html);
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].text, "Title");
    }

    #[test]
    fn test_extract_headings_with_nested_tags() {
        let html = "<h1><b>Bold</b> Title</h1>";
        let headings = extract_headings(html);
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].text, "Bold Title");
    }

    #[test]
    fn test_extract_headings_with_entities() {
        let html = "<h2>Tom &amp; Jerry</h2>";
        let headings = extract_headings(html);
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].text, "Tom & Jerry");
    }

    #[test]
    fn test_extract_headings_no_headings() {
        let html = "<p>Just a paragraph</p>";
        let headings = extract_headings(html);
        assert!(headings.is_empty());
    }

    #[test]
    fn test_extract_headings_empty_heading() {
        let html = "<h1></h1><h2>Valid</h2>";
        let headings = extract_headings(html);
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].text, "Valid");
    }

    // -----------------------------------------------------------------------
    // html_to_plain_text tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_plain_text_paragraphs() {
        let html = "<p>First paragraph.</p><p>Second paragraph.</p>";
        let text = html_to_plain_text(html);
        assert!(text.contains("First paragraph."));
        assert!(text.contains("Second paragraph."));
        // Should have blank line between paragraphs
        assert!(text.contains("\n\n"));
    }

    #[test]
    fn test_plain_text_line_breaks() {
        let html = "Line 1<br>Line 2<br/>Line 3";
        let text = html_to_plain_text(html);
        assert!(text.contains("Line 1\nLine 2\nLine 3"));
    }

    #[test]
    fn test_plain_text_list_items() {
        let html = "<ul><li>Item 1</li><li>Item 2</li><li>Item 3</li></ul>";
        let text = html_to_plain_text(html);
        assert!(text.contains("- Item 1"));
        assert!(text.contains("- Item 2"));
        assert!(text.contains("- Item 3"));
    }

    #[test]
    fn test_plain_text_strips_script() {
        let html = "<p>Hello</p><script>alert('xss');</script><p>World</p>";
        let text = html_to_plain_text(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
        assert!(!text.contains("alert"));
        assert!(!text.contains("script"));
    }

    #[test]
    fn test_plain_text_strips_style() {
        let html = "<style>body { color: red; }</style><p>Content</p>";
        let text = html_to_plain_text(html);
        assert!(text.contains("Content"));
        assert!(!text.contains("color"));
    }

    #[test]
    fn test_plain_text_entities() {
        let html = "<p>Tom &amp; Jerry &lt;3</p>";
        let text = html_to_plain_text(html);
        assert!(text.contains("Tom & Jerry <3"));
    }

    #[test]
    fn test_plain_text_empty_html() {
        assert_eq!(html_to_plain_text(""), "");
    }

    #[test]
    fn test_plain_text_headings_get_spacing() {
        let html = "<h1>Title</h1><p>Content below heading.</p>";
        let text = html_to_plain_text(html);
        assert!(text.contains("Title"));
        assert!(text.contains("Content below heading."));
    }

    #[test]
    fn test_plain_text_horizontal_rule() {
        let html = "<p>Before</p><hr><p>After</p>";
        let text = html_to_plain_text(html);
        assert!(text.contains("Before"));
        assert!(text.contains("---"));
        assert!(text.contains("After"));
    }

    // -----------------------------------------------------------------------
    // import_html_content tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_import_html_with_headings() {
        let html = "<h1>Chapter 1</h1><p>Content for chapter 1.</p><h2>Section A</h2><p>Section A content.</p>";
        let items = import_html_content(html, "Test").unwrap();
        assert!(!items.is_empty());
        // Should have a folder for the H1
        assert_eq!(items[0].kind, BinderItemKind::Folder);
        assert_eq!(items[0].title, "Chapter 1");
    }

    #[test]
    fn test_import_html_no_headings() {
        let html = "<p>Just some content without headings.</p>";
        let items = import_html_content(html, "Untitled").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, BinderItemKind::Text);
        assert_eq!(items[0].title, "Untitled");
    }

    #[test]
    fn test_import_html_empty() {
        let items = import_html_content("", "Empty").unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_import_html_multiple_h1() {
        let html = "<h1>Part 1</h1><p>Content 1.</p><h1>Part 2</h1><p>Content 2.</p>";
        let items = import_html_content(html, "Book").unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Part 1");
        assert_eq!(items[1].title, "Part 2");
    }

    #[test]
    fn test_import_html_h2_under_h1() {
        let html = "<h1>Main</h1><h2>Sub 1</h2><p>Content 1.</p><h2>Sub 2</h2><p>Content 2.</p>";
        let items = import_html_content(html, "Doc").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Main");
        // H2 items should be children of the H1 folder
        assert!(items[0].children.len() >= 2);
    }

    // -----------------------------------------------------------------------
    // import_html_as_single_document tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_import_single_document() {
        let html = "<h1>Title</h1><p>Some content here.</p>";
        let item = import_html_as_single_document(html, "My Document");
        assert_eq!(item.kind, BinderItemKind::Text);
        assert_eq!(item.title, "My Document");
        let doc = item.document.as_ref().unwrap();
        assert!(doc.content.contains("Some content here."));
        assert!(doc.notes.contains("Imported from HTML source."));
    }

    #[test]
    fn test_import_single_document_notes_contain_length() {
        let html = "<p>Hello</p>";
        let item = import_html_as_single_document(html, "Test");
        let doc = item.document.as_ref().unwrap();
        assert!(doc.notes.contains(&html.len().to_string()));
    }

    // -----------------------------------------------------------------------
    // create_web_page_item tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_create_web_page_item_basic() {
        let html = "<html><head><title>Example Page</title></head><body><p>Hello World</p></body></html>";
        let item = create_web_page_item(html, "https://example.com/page");
        assert_eq!(item.kind, BinderItemKind::WebPage);
        assert_eq!(item.title, "Example Page");
        let doc = item.document.as_ref().unwrap();
        assert!(doc.content.contains("Hello World"));
        assert!(doc.notes.contains("https://example.com/page"));
    }

    #[test]
    fn test_create_web_page_item_no_title() {
        let html = "<p>No title tag here</p>";
        let item = create_web_page_item(html, "https://www.example.com/path");
        assert_eq!(item.kind, BinderItemKind::WebPage);
        // Should fall back to domain-based title
        assert_eq!(item.title, "example.com");
    }

    #[test]
    fn test_create_web_page_item_word_count_in_notes() {
        let html = "<p>one two three four five</p>";
        let item = create_web_page_item(html, "https://example.com");
        let doc = item.document.as_ref().unwrap();
        assert!(doc.notes.contains("Word count: 5"));
    }

    #[test]
    fn test_create_web_page_item_synopsis() {
        let html = "<p>A short page.</p>";
        let item = create_web_page_item(html, "https://example.com");
        assert!(!item.synopsis.is_empty());
    }

    #[test]
    fn test_create_web_page_item_long_synopsis_truncated() {
        let long_content = "word ".repeat(100);
        let html = format!("<p>{}</p>", long_content);
        let item = create_web_page_item(&html, "https://example.com");
        // Synopsis should be truncated with "..."
        assert!(item.synopsis.len() <= 210); // 200 + "..."
    }

    // -----------------------------------------------------------------------
    // WebPageMetadata tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_web_page_metadata_fields() {
        let metadata = WebPageMetadata {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            fetched_at: Utc::now(),
            word_count: 42,
        };
        assert_eq!(metadata.url, "https://example.com");
        assert_eq!(metadata.title, "Example");
        assert_eq!(metadata.word_count, 42);
    }

    // -----------------------------------------------------------------------
    // HtmlHeading tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_html_heading_struct() {
        let heading = HtmlHeading {
            level: 2,
            text: "Section Title".to_string(),
        };
        assert_eq!(heading.level, 2);
        assert_eq!(heading.text, "Section Title");
    }

    #[test]
    fn test_html_heading_equality() {
        let h1 = HtmlHeading { level: 1, text: "Title".to_string() };
        let h2 = HtmlHeading { level: 1, text: "Title".to_string() };
        let h3 = HtmlHeading { level: 2, text: "Title".to_string() };
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    // -----------------------------------------------------------------------
    // Helper function tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_title_from_url_basic() {
        assert_eq!(title_from_url("https://example.com/page"), "example.com");
    }

    #[test]
    fn test_title_from_url_www() {
        assert_eq!(title_from_url("https://www.example.com"), "example.com");
    }

    #[test]
    fn test_title_from_url_http() {
        assert_eq!(title_from_url("http://blog.example.org/post"), "blog.example.org");
    }

    #[test]
    fn test_title_from_url_no_protocol() {
        assert_eq!(title_from_url("example.com/page"), "example.com");
    }

    #[test]
    fn test_remove_tag_block_script() {
        let html = "Before<script>var x = 1;</script>After";
        assert_eq!(remove_tag_block(html, "script"), "BeforeAfter");
    }

    #[test]
    fn test_remove_tag_block_style() {
        let html = "Before<style>body{}</style>After";
        assert_eq!(remove_tag_block(html, "style"), "BeforeAfter");
    }

    #[test]
    fn test_remove_tag_block_multiple() {
        let html = "A<script>1</script>B<script>2</script>C";
        assert_eq!(remove_tag_block(html, "script"), "ABC");
    }

    #[test]
    fn test_replace_tag_case_insensitive_mixed_case() {
        let html = "Hello<BR>World<br>End";
        let result = replace_tag_case_insensitive(html, "<br>", "\n");
        assert_eq!(result, "Hello\nWorld\nEnd");
    }

    // -----------------------------------------------------------------------
    // Integration / complex scenario tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_full_html_page_import() {
        let html = r#"
            <html>
            <head><title>Test Article</title></head>
            <body>
                <h1>Introduction</h1>
                <p>This is the introduction paragraph.</p>
                <h2>Background</h2>
                <p>Some background information.</p>
                <h2>Details</h2>
                <p>The detailed content goes here.</p>
                <h1>Conclusion</h1>
                <p>Final thoughts.</p>
            </body>
            </html>
        "#;

        let items = import_html_content(html, "Article").unwrap();
        assert_eq!(items.len(), 2); // Two H1 sections
        assert_eq!(items[0].title, "Introduction");
        assert_eq!(items[1].title, "Conclusion");
        // Introduction folder should have children for its H2 headings
        assert!(items[0].children.len() >= 2);
    }

    #[test]
    fn test_complex_entity_decoding() {
        let text = "&#60;tag&#62; &#x26; &#x3C;other&#x3E;";
        let decoded = decode_html_entities(text);
        assert_eq!(decoded, "<tag> & <other>");
    }

    #[test]
    fn test_html_with_mixed_content() {
        let html = r#"
            <div>
                <h1>Title</h1>
                <p>Paragraph with <b>bold</b> and <i>italic</i> text.</p>
                <ul>
                    <li>Item one</li>
                    <li>Item two</li>
                </ul>
                <blockquote>A quote</blockquote>
            </div>
        "#;

        let text = html_to_plain_text(html);
        assert!(text.contains("Title"));
        assert!(text.contains("bold"));
        assert!(text.contains("italic"));
        assert!(text.contains("- Item one"));
        assert!(text.contains("- Item two"));
        assert!(text.contains("A quote"));
    }

    #[test]
    fn test_web_page_item_notes_contain_fetched_time() {
        let html = "<title>Page</title><p>Content</p>";
        let item = create_web_page_item(html, "https://example.com");
        let doc = item.document.as_ref().unwrap();
        assert!(doc.notes.contains("Fetched:"));
        assert!(doc.notes.contains("UTC"));
    }

    #[test]
    fn test_import_only_whitespace_html() {
        let html = "<p>   </p><div>  </div>";
        let items = import_html_content(html, "Whitespace").unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_plain_text_collapse_newlines() {
        let html = "<p>A</p><p></p><p></p><p></p><p>B</p>";
        let text = html_to_plain_text(html);
        // Should not have more than 2 consecutive newlines
        assert!(!text.contains("\n\n\n"));
    }

    #[test]
    fn test_strip_html_tags_malformed() {
        // Unclosed tag: should still work
        assert_eq!(strip_html_tags("<p>Hello"), "Hello");
    }

    #[test]
    fn test_decode_entities_empty_string() {
        assert_eq!(decode_html_entities(""), "");
    }

    #[test]
    fn test_extract_headings_case_insensitive() {
        let html = "<H1>Upper</H1><h2>Lower</h2>";
        let headings = extract_headings(html);
        assert_eq!(headings.len(), 2);
        assert_eq!(headings[0].text, "Upper");
        assert_eq!(headings[1].text, "Lower");
    }
}
