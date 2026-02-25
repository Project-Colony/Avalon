use std::fmt::Write as _;
use anyhow::Result;

use super::compiler::{CompileContent, CompileOptions};

/// Compile content to Fountain screenplay format
pub fn compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    let estimated_size: usize = contents.iter().map(|c| c.text.len() + c.title.len() + 20).sum();
    let mut output = String::with_capacity(estimated_size);

    // Title page metadata
    if options.include_front_matter {
        let _ = writeln!(output,"Title: {}", options.title);
        if !options.author.is_empty() {
            let _ = writeln!(output,"Author: {}", options.author);
        }
        let _ = writeln!(output,"Draft date: {}", chrono::Utc::now().format(crate::core::DATE_FORMAT));
        output.push_str("Contact:\n");
        output.push('\n');
    }

    for (i, content) in contents.iter().enumerate() {
        if content.is_folder {
            // Folders become act headings
            if i > 0 {
                output.push_str("\n\n");
            }
            let _ = write!(output,"# {}\n\n", content.title.to_uppercase());
        } else {
            // Try to detect Fountain formatting, otherwise convert prose
            if !content.text.is_empty() {
                let converted = prose_to_fountain(&content.text);
                output.push_str(&converted);
                output.push_str("\n\n");
            }
        }
    }

    Ok(output)
}

/// Attempt basic prose-to-Fountain conversion
fn prose_to_fountain(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.is_empty() {
            output.push('\n');
            i += 1;
            continue;
        }

        // Normalize page breaks to Fountain standard; pass everything else through
        if line == "===" || line == "---" {
            output.push_str("===\n");
        } else {
            output.push_str(line);
            output.push('\n');
        }

        i += 1;
    }

    output
}

/// Parse a Fountain document into structured sections.
/// Returns title and a list of (section_name, content) pairs.
pub fn parse_fountain(input: &str) -> Vec<(String, String)> {
    let mut sections = Vec::new();
    let mut current_title = String::from("Untitled");
    let mut current_content = String::new();

    for line in input.lines() {
        let trimmed = line.trim();

        // Scene headings: INT., EXT., or lines starting with .
        if is_scene_heading(trimmed) {
            if !current_content.trim().is_empty() || !sections.is_empty() {
                let title = std::mem::replace(&mut current_title, trimmed.to_string());
                sections.push((title, std::mem::take(&mut current_content)));
            } else {
                current_title = trimmed.to_string();
            }
        }

        current_content.push_str(line);
        current_content.push('\n');
    }

    // Push remaining content
    if !current_content.trim().is_empty() {
        sections.push((current_title, current_content));
    }

    sections
}

/// Check if a line is a Fountain scene heading
fn is_scene_heading(line: &str) -> bool {
    let upper = line.to_uppercase();
    upper.starts_with("INT.")
        || upper.starts_with("INT ")
        || upper.starts_with("EXT.")
        || upper.starts_with("EXT ")
        || upper.starts_with("EST.")
        || upper.starts_with("I/E.")
        || upper.starts_with("I/E ")
        || line.starts_with('.')
}
