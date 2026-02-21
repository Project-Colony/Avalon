use anyhow::Result;
use super::compiler::{CompileContent, CompileOptions, SeparatorType};

pub fn compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    let mut output = String::new();

    // Front matter / title page
    if options.include_front_matter && !options.title.is_empty() {
        let title = options.title.to_uppercase();
        output.push('\n');
        output.push_str(&title);
        output.push('\n');
        output.push_str(&"=".repeat(title.len()));
        output.push('\n');

        if !options.author.is_empty() {
            output.push_str(&format!("by {}", options.author));
            output.push('\n');
        }

        output.push('\n');
        output.push_str(&"\u{2500}".repeat(40));
        output.push_str("\n\n");
    }

    let mut prev_was_folder = false;

    for (i, content) in contents.iter().enumerate() {
        if content.is_folder {
            // Page break between top-level folders
            if options.page_break_between_folders && content.depth == 1 && i > 0 {
                output.push_str("\n\n");
                output.push_str(&"\u{2500}".repeat(40));
                output.push_str("\n\n");
            }

            // Folder headings with depth-based decoration
            output.push('\n');
            match content.depth {
                0 | 1 => {
                    let heading = content.title.to_uppercase();
                    output.push_str(&heading);
                    output.push('\n');
                    output.push_str(&"=".repeat(heading.len()));
                }
                2 => {
                    output.push_str(&content.title);
                    output.push('\n');
                    output.push_str(&"-".repeat(content.title.len()));
                }
                _ => {
                    let indent = "  ".repeat(content.depth.saturating_sub(2));
                    output.push_str(&format!("{}* {}", indent, content.title));
                }
            }
            output.push_str("\n\n");
            prev_was_folder = true;
        } else if !content.text.is_empty() {
            // Document separator between non-folder items
            if i > 0 && !prev_was_folder {
                match &options.separator {
                    SeparatorType::EmptyLine => output.push('\n'),
                    SeparatorType::SectionBreak => {
                        output.push_str("\n        * * *\n\n");
                    }
                    SeparatorType::Custom(s) => {
                        output.push_str(&format!("\n{}\n\n", s));
                    }
                    SeparatorType::PageBreak => {
                        output.push_str("\n\n");
                        output.push_str(&"\u{2500}".repeat(40));
                        output.push_str("\n\n");
                    }
                    SeparatorType::None => {}
                }
            }

            // Strip markdown formatting for clean plain text
            let clean = strip_markdown(&content.text);
            output.push_str(&clean);
            output.push_str("\n\n");
            prev_was_folder = false;
        }
    }

    Ok(output.trim_end().to_string())
}

/// Strip basic markdown formatting to produce clean plain text
fn strip_markdown(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    for line in text.lines() {
        let trimmed = line.trim();

        // Convert headings to plain text
        if trimmed.starts_with('#') {
            let content = trimmed.trim_start_matches('#').trim();
            result.push_str(content);
            result.push('\n');
            continue;
        }

        // Strip bold/italic/strikethrough markers
        let mut cleaned = line.to_string();
        cleaned = cleaned.replace("***", "");
        cleaned = cleaned.replace("**", "");
        cleaned = cleaned.replace("~~", "");

        // Remove remaining single emphasis markers
        let chars: Vec<char> = cleaned.chars().collect();
        let mut out = String::with_capacity(cleaned.len());
        for &ch in &chars {
            if ch != '*' && ch != '_' {
                out.push(ch);
            }
        }

        // Convert blockquotes to indented quotes
        let out = if out.trim_start().starts_with("> ") {
            out.replacen("> ", "  ", 1)
        } else {
            out
        };

        result.push_str(&out);
        result.push('\n');
    }

    if result.ends_with('\n') {
        result.pop();
    }

    result
}

/// Word wrap text to a maximum line width
pub fn word_wrap(text: &str, max_width: usize) -> String {
    let mut result = String::new();
    for line in text.lines() {
        if line.len() <= max_width {
            result.push_str(line);
            result.push('\n');
            continue;
        }
        let mut current_len = 0;
        for word in line.split_whitespace() {
            if current_len + word.len() + 1 > max_width && current_len > 0 {
                result.push('\n');
                current_len = 0;
            }
            if current_len > 0 {
                result.push(' ');
                current_len += 1;
            }
            result.push_str(word);
            current_len += word.len();
        }
        result.push('\n');
    }
    if result.ends_with('\n') {
        result.pop();
    }
    result
}

/// Estimate page count from content
pub fn estimate_pages(contents: &[CompileContent]) -> usize {
    let total_words: usize = contents.iter()
        .map(|c| c.text.split_whitespace().count())
        .sum();
    (total_words / 250).max(1)
}

/// Count total words across all content sections
pub fn word_count(contents: &[CompileContent]) -> usize {
    contents.iter().map(|c| c.text.split_whitespace().count()).sum()
}

/// Count total characters across all content sections
pub fn char_count(contents: &[CompileContent]) -> usize {
    contents.iter().map(|c| c.text.len()).sum()
}
