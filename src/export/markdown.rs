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

        // Word count metadata
        let total_words: usize = contents.iter().map(|c| c.text.split_whitespace().count()).sum();
        output.push_str(&format!("wordcount: {}\n", total_words));

        output.push_str("---\n\n");

        // Title as H1
        output.push_str(&format!("# {}\n\n", options.title));
        if !options.author.is_empty() {
            output.push_str(&format!("*by {}*\n\n", options.author));
        }
        output.push_str("---\n\n");
    }

    // Table of contents (if enabled)
    if options.include_toc {
        output.push_str("## Table of Contents\n\n");
        for content in contents {
            if content.is_folder || !content.text.is_empty() {
                let indent = "  ".repeat(content.depth);
                let anchor = slug(&content.title);
                output.push_str(&format!(
                    "{}- [{}](#{})\n",
                    indent, content.title, anchor
                ));
            }
        }
        output.push_str("\n---\n\n");
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

/// Create a URL-safe slug from a title (for anchor links)
fn slug(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Count total words across all content sections
pub fn word_count(contents: &[CompileContent]) -> usize {
    contents
        .iter()
        .map(|c| c.text.split_whitespace().count())
        .sum()
}

/// Count total characters across all content sections
pub fn char_count(contents: &[CompileContent]) -> usize {
    contents.iter().map(|c| c.text.len()).sum()
}

/// Estimate page count from content
pub fn estimate_pages(contents: &[CompileContent]) -> usize {
    let total_words: usize = contents.iter()
        .map(|c| c.text.split_whitespace().count())
        .sum();
    (total_words / 250).max(1)
}

/// Extract all headings from markdown content for analysis
pub fn extract_headings(contents: &[CompileContent]) -> Vec<(usize, String)> {
    let mut headings = Vec::new();
    for content in contents {
        if content.is_folder {
            let level = (content.depth + 2).min(6);
            headings.push((level, content.title.clone()));
        }
        for line in content.text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|c| *c == '#').count();
                let text = trimmed[level..].trim().to_string();
                if !text.is_empty() && level <= 6 {
                    headings.push((level, text));
                }
            }
        }
    }
    headings
}
