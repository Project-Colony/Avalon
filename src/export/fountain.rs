use anyhow::Result;

use super::compiler::{CompileContent, CompileOptions};

/// Compile content to Fountain screenplay format
pub fn compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    let mut output = String::new();

    // Title page metadata
    if options.include_front_matter {
        output.push_str(&format!("Title: {}\n", options.title));
        if !options.author.is_empty() {
            output.push_str(&format!("Author: {}\n", options.author));
        }
        output.push_str(&format!("Draft date: {}\n", chrono::Utc::now().format("%Y-%m-%d")));
        output.push('\n');
    }

    for (i, content) in contents.iter().enumerate() {
        if content.is_folder {
            // Folders become act headings
            if i > 0 {
                output.push_str("\n\n");
            }
            output.push_str(&format!("# {}\n\n", content.title.to_uppercase()));
        } else {
            // Text documents — output as-is (assume Fountain-formatted)
            // Or convert basic prose to Fountain format
            if !content.text.is_empty() {
                output.push_str(&content.text);
                output.push_str("\n\n");
            }
        }
    }

    Ok(output)
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
                sections.push((current_title.clone(), current_content.clone()));
                current_content.clear();
            }
            current_title = trimmed.to_string();
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
