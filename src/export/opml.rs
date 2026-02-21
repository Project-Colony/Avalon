use anyhow::Result;

use crate::core::binder::{Binder, BinderItem, BinderItemKind};

/// Export the project binder as OPML (Outline Processor Markup Language)
pub fn export_opml(binder: &Binder, title: &str) -> Result<String> {
    let mut output = String::new();
    output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    output.push_str("<opml version=\"2.0\">\n");
    output.push_str("  <head>\n");
    output.push_str(&format!("    <title>{}</title>\n", escape_xml(title)));
    output.push_str("  </head>\n");
    output.push_str("  <body>\n");

    // Export draft
    write_opml_item(&binder.draft, 2, &mut output);

    // Export research
    write_opml_item(&binder.research, 2, &mut output);

    output.push_str("  </body>\n");
    output.push_str("</opml>\n");

    Ok(output)
}

fn write_opml_item(item: &BinderItem, depth: usize, output: &mut String) {
    let indent = "  ".repeat(depth);
    let text_attr = escape_xml(&item.title);
    let note_attr = item.synopsis.clone();

    let content_note = item.document.as_ref()
        .map(|d| escape_xml(&d.content))
        .unwrap_or_default();

    let combined_note = if !note_attr.is_empty() && !content_note.is_empty() {
        format!("{}\n\n{}", escape_xml(&note_attr), content_note)
    } else if !note_attr.is_empty() {
        escape_xml(&note_attr)
    } else {
        content_note
    };

    if item.children.is_empty() {
        if combined_note.is_empty() {
            output.push_str(&format!("{}<outline text=\"{}\"/>\n", indent, text_attr));
        } else {
            output.push_str(&format!(
                "{}<outline text=\"{}\" _note=\"{}\"/>\n",
                indent, text_attr, combined_note
            ));
        }
    } else {
        if combined_note.is_empty() {
            output.push_str(&format!("{}<outline text=\"{}\">\n", indent, text_attr));
        } else {
            output.push_str(&format!(
                "{}<outline text=\"{}\" _note=\"{}\">\n",
                indent, text_attr, combined_note
            ));
        }
        for child in &item.children {
            write_opml_item(child, depth + 1, output);
        }
        output.push_str(&format!("{}</outline>\n", indent));
    }
}

/// Import OPML content into a binder structure
pub fn import_opml(content: &str) -> Result<Vec<BinderItem>> {
    let mut items = Vec::new();

    // Simple XML parser for OPML outlines
    let mut in_body = false;
    let mut stack: Vec<BinderItem> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.contains("<body>") || trimmed.contains("<body/>") {
            in_body = true;
            continue;
        }
        if trimmed.contains("</body>") {
            in_body = false;
            // Flush remaining stack
            while let Some(item) = stack.pop() {
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(item);
                } else {
                    items.push(item);
                }
            }
            continue;
        }

        if !in_body {
            continue;
        }

        if trimmed.starts_with("<outline") {
            let text = extract_attr(trimmed, "text").unwrap_or_default();
            let note = extract_attr(trimmed, "_note").unwrap_or_default();

            let is_self_closing = trimmed.ends_with("/>");
            let has_children = !is_self_closing;

            let mut item = if has_children {
                BinderItem::new_folder(&unescape_xml(&text))
            } else {
                let mut i = BinderItem::new_text(&unescape_xml(&text));
                if !note.is_empty() {
                    if let Some(ref mut doc) = i.document {
                        doc.content = unescape_xml(&note);
                    }
                }
                i
            };

            if !note.is_empty() {
                item.synopsis = unescape_xml(&note);
            }

            if is_self_closing {
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(item);
                } else {
                    items.push(item);
                }
            } else {
                stack.push(item);
            }
        } else if trimmed.starts_with("</outline>") {
            if let Some(item) = stack.pop() {
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(item);
                } else {
                    items.push(item);
                }
            }
        }
    }

    Ok(items)
}

fn extract_attr(line: &str, attr_name: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr_name);
    if let Some(start) = line.find(&pattern) {
        let value_start = start + pattern.len();
        if let Some(end) = line[value_start..].find('"') {
            return Some(line[value_start..value_start + end].to_string());
        }
    }
    None
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn unescape_xml(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
}

/// Count the total number of outline items in a binder tree
pub fn count_items(item: &BinderItem) -> usize {
    1 + item.children.iter().map(|c| count_items(c)).sum::<usize>()
}

/// Count the depth of the deepest item in the tree
pub fn max_depth(item: &BinderItem) -> usize {
    if item.children.is_empty() {
        0
    } else {
        1 + item.children.iter().map(|c| max_depth(c)).max().unwrap_or(0)
    }
}

/// Extract all titles from a binder tree into a flat list
pub fn flat_titles(item: &BinderItem, depth: usize) -> Vec<(usize, String)> {
    let mut titles = vec![(depth, item.title.clone())];
    for child in &item.children {
        titles.extend(flat_titles(child, depth + 1));
    }
    titles
}

/// Validate OPML content by checking for required elements
pub fn validate_opml(content: &str) -> Result<()> {
    if !content.contains("<opml") {
        anyhow::bail!("Missing <opml> root element");
    }
    if !content.contains("<head>") || !content.contains("</head>") {
        anyhow::bail!("Missing <head> element");
    }
    if !content.contains("<body>") || !content.contains("</body>") {
        anyhow::bail!("Missing <body> element");
    }
    Ok(())
}
