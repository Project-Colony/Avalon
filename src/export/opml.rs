use anyhow::Result;
use std::fmt::Write;

use crate::core::binder::{Binder, BinderItem};

/// Export the project binder as OPML (Outline Processor Markup Language)
pub fn export_opml(binder: &Binder, title: &str) -> Result<String> {
    let mut output = String::new();
    output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    output.push_str("<opml version=\"2.0\">\n");
    output.push_str("  <head>\n");
    writeln!(output, "    <title>{}</title>", escape_xml(title)).unwrap();
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

    let content_note = item
        .document
        .as_ref()
        .map(|d| escape_xml(&d.content))
        .unwrap_or_default();

    let combined_note = if !item.synopsis.is_empty() && !content_note.is_empty() {
        format!("{}\n\n{}", escape_xml(&item.synopsis), content_note)
    } else if !item.synopsis.is_empty() {
        escape_xml(&item.synopsis)
    } else {
        content_note
    };

    if item.children.is_empty() {
        if combined_note.is_empty() {
            writeln!(output, "{}<outline text=\"{}\"/>", indent, text_attr).unwrap();
        } else {
            writeln!(
                output,
                "{}<outline text=\"{}\" _note=\"{}\"/>",
                indent, text_attr, combined_note
            )
            .unwrap();
        }
    } else {
        if combined_note.is_empty() {
            writeln!(output, "{}<outline text=\"{}\">", indent, text_attr).unwrap();
        } else {
            writeln!(
                output,
                "{}<outline text=\"{}\" _note=\"{}\">",
                indent, text_attr, combined_note
            )
            .unwrap();
        }
        for child in &item.children {
            write_opml_item(child, depth + 1, output);
        }
        writeln!(output, "{}</outline>", indent).unwrap();
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
    super::compiler::escape_xml(s)
}

fn unescape_xml(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::binder::{Binder, BinderItem};

    #[test]
    fn test_export_opml_basic() {
        let binder = Binder::default_structure();
        let output = export_opml(&binder, "Test").unwrap();
        assert!(output.contains("<opml version=\"2.0\">"));
        assert!(output.contains("<title>Test</title>"));
        assert!(output.contains("</opml>"));
    }

    #[test]
    fn test_export_opml_escapes_title() {
        let binder = Binder::default_structure();
        let output = export_opml(&binder, "Tom & Jerry").unwrap();
        assert!(output.contains("Tom &amp; Jerry"));
    }

    #[test]
    fn test_import_opml_empty() {
        let input = r#"<?xml version="1.0"?><opml><head></head><body></body></opml>"#;
        let items = import_opml(input).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_import_opml_single_item() {
        let input = r#"<?xml version="1.0"?>
<opml version="2.0">
<head><title>T</title></head>
<body>
  <outline text="Chapter One"/>
</body>
</opml>"#;
        let items = import_opml(input).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Chapter One");
    }

    #[test]
    fn test_import_opml_nested() {
        let input = r#"<opml>
<head></head>
<body>
  <outline text="Part 1">
    <outline text="Ch 1"/>
    <outline text="Ch 2"/>
  </outline>
</body>
</opml>"#;
        let items = import_opml(input).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].children.len(), 2);
    }

    #[test]
    fn test_roundtrip_export_import() {
        let mut binder = Binder::default_structure();
        let child = BinderItem::new_text("Scene A");
        binder.draft.children.push(child);
        let opml = export_opml(&binder, "Test").unwrap();
        let imported = import_opml(&opml).unwrap();
        // Draft + Research are exported as top-level items
        assert!(!imported.is_empty());
    }

    #[test]
    fn test_unescape_xml() {
        assert_eq!(unescape_xml("&amp;&lt;&gt;&quot;"), "&<>\"");
    }

    #[test]
    fn test_extract_attr() {
        assert_eq!(extract_attr(r#"<outline text="hello"/>"#, "text"), Some("hello".into()));
        assert_eq!(
            extract_attr(r#"<outline text="a" _note="b"/>"#, "_note"),
            Some("b".into())
        );
        assert!(extract_attr("<outline/>", "text").is_none());
    }
}
