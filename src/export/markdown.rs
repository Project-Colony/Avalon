use anyhow::Result;
use super::compiler::{CompileContent, CompileOptions, SeparatorType};

pub fn compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    let mut output = String::new();

    // YAML front matter block (common in Markdown publishing)
    if options.include_front_matter && !options.title.is_empty() {
        output.push_str("---\n");
        output.push_str(&format!("title: \"{}\"\n", escape_yaml(&options.title)));
        if !options.author.is_empty() {
            output.push_str(&format!("author: \"{}\"\n", escape_yaml(&options.author)));
        }
        output.push_str(&format!("date: \"{}\"\n", chrono::Local::now().format("%Y-%m-%d")));
        output.push_str("---\n\n");

        // Title as H1
        output.push_str(&format!("# {}\n\n", options.title));
        if !options.author.is_empty() {
            output.push_str(&format!("*by {}*\n\n", options.author));
        }
        output.push_str("---\n\n");
    }

    let mut prev_was_text = false;

    for (i, content) in contents.iter().enumerate() {
        if content.is_folder {
            // Page break before top-level folders (after the first)
            if options.page_break_between_folders && content.depth == 1 && i > 0 {
                output.push_str("\n---\n\n");
            }

            // Folder becomes heading, depth maps: 0->##, 1->##, 2->###, etc.
            let level = (content.depth + 2).min(6);
            let hashes = "#".repeat(level);
            output.push_str(&format!("{} {}\n\n", hashes, content.title));
            prev_was_text = false;
        } else if !content.text.is_empty() {
            // Section separator between consecutive text documents
            if prev_was_text {
                match &options.separator {
                    SeparatorType::EmptyLine => output.push('\n'),
                    SeparatorType::SectionBreak => {
                        output.push_str("\n<center>* * *</center>\n\n");
                    }
                    SeparatorType::PageBreak => {
                        output.push_str("\n---\n\n");
                    }
                    SeparatorType::Custom(s) => {
                        output.push_str(&format!("\n{}\n\n", s));
                    }
                    SeparatorType::None => {}
                }
            }

            output.push_str(&content.text);
            output.push_str("\n\n");
            prev_was_text = true;
        }
    }

    Ok(output.trim_end().to_string())
}

/// Escape special characters in YAML string values
fn escape_yaml(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
}
