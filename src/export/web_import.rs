use crate::core::binder::BinderItem;
use anyhow::Result;

/// A heading extracted from HTML content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlHeading {
    /// Heading level (1-6, corresponding to h1-h6).
    pub level: u8,
    /// The text content of the heading.
    pub text: String,
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
                        if let Some(c) = chars.next() {
                            entity.push(c);
                        }
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
                        let code_point = if let Some(hex) = numeric_part
                            .strip_prefix('x')
                            .or_else(|| numeric_part.strip_prefix('X'))
                        {
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
            None => {
                search_pos = tag_start + 1;
                continue;
            }
        };

        // Find the closing tag
        let close_tag = format!("</h{}>", level);
        let close_start = match lower[open_end..].find(&close_tag) {
            Some(pos) => open_end + pos,
            None => {
                search_pos = open_end;
                continue;
            }
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
