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
        output.push_str("Contact:\n");
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
    let mut output = String::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.is_empty() {
            output.push('\n');
            i += 1;
            continue;
        }

        // Already a scene heading
        if is_scene_heading(line) {
            output.push_str(line);
            output.push('\n');
        }
        // Already a transition (ends with TO:)
        else if is_transition(line) {
            output.push_str(line);
            output.push('\n');
        }
        // ALL CAPS line followed by non-empty = character cue
        else if is_character_cue(line) && i + 1 < lines.len() && !lines[i + 1].trim().is_empty() {
            output.push_str(line);
            output.push('\n');
        }
        // Centered text
        else if line.starts_with('>') && line.ends_with('<') {
            output.push_str(line);
            output.push('\n');
        }
        // Parenthetical
        else if line.starts_with('(') && line.ends_with(')') {
            output.push_str(line);
            output.push('\n');
        }
        // Note
        else if line.starts_with("[[") && line.ends_with("]]") {
            output.push_str(line);
            output.push('\n');
        }
        // Page break
        else if line == "===" || line == "---" {
            output.push_str("===\n");
        }
        // Regular action text
        else {
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

/// Extract title page metadata from a Fountain document
pub fn parse_title_page(input: &str) -> Vec<(String, String)> {
    let mut meta = Vec::new();
    for line in input.lines() {
        // Title page ends at first blank line
        if line.trim().is_empty() {
            break;
        }
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim().to_string();
            let value = line[colon_pos + 1..].trim().to_string();
            if !key.is_empty() {
                meta.push((key, value));
            }
        }
    }
    meta
}

/// Count scenes in a Fountain document
pub fn scene_count(input: &str) -> usize {
    input.lines().filter(|l| is_scene_heading(l.trim())).count()
}

/// Extract all character names from a Fountain document
pub fn extract_characters(input: &str) -> Vec<String> {
    let mut characters = Vec::new();
    let lines: Vec<&str> = input.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if is_character_cue(trimmed) {
            // Verify next line exists and is not empty (i.e. dialogue follows)
            if i + 1 < lines.len() && !lines[i + 1].trim().is_empty() {
                // Strip parenthetical extensions like (V.O.), (O.S.), (CONT'D)
                let name = trimmed
                    .split('(')
                    .next()
                    .unwrap_or(trimmed)
                    .trim()
                    .to_string();
                if !name.is_empty() && !characters.contains(&name) {
                    characters.push(name);
                }
            }
        }
    }

    characters
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

/// Check if a line is a transition (e.g. CUT TO:, FADE OUT.)
fn is_transition(line: &str) -> bool {
    let upper = line.trim().to_uppercase();
    upper.ends_with("TO:")
        || upper == "FADE OUT."
        || upper == "FADE IN:"
        || upper == "SMASH CUT:"
        || upper.starts_with('>')
}

/// Check if a line looks like a character cue (ALL CAPS, not a scene heading)
fn is_character_cue(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.len() < 2 {
        return false;
    }
    if is_scene_heading(trimmed) || is_transition(trimmed) {
        return false;
    }
    // Must be all uppercase letters (allowing spaces, periods, parentheses)
    let alpha_chars: Vec<char> = trimmed.chars().filter(|c| c.is_alphabetic()).collect();
    !alpha_chars.is_empty() && alpha_chars.iter().all(|c| c.is_uppercase())
}

/// Count dialogue blocks in a Fountain document
pub fn dialogue_count(input: &str) -> usize {
    let lines: Vec<&str> = input.lines().collect();
    let mut count = 0;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if is_character_cue(trimmed) && i + 1 < lines.len() && !lines[i + 1].trim().is_empty() {
            count += 1;
        }
    }
    count
}

/// Estimate page count for a screenplay (industry standard: ~1 page per minute)
/// Uses the rough heuristic of ~56 lines per page
pub fn estimate_page_count(input: &str) -> usize {
    let line_count = input.lines().count();
    (line_count / 56).max(1)
}

/// Count transitions in a Fountain document
pub fn transition_count(input: &str) -> usize {
    input.lines().filter(|l| is_transition(l.trim())).count()
}

/// Get a summary of the screenplay structure
pub fn screenplay_summary(input: &str) -> String {
    let scenes = scene_count(input);
    let chars = extract_characters(input);
    let dialogues = dialogue_count(input);
    let pages = estimate_page_count(input);
    format!(
        "{} scenes, {} characters, {} dialogue blocks, ~{} pages",
        scenes, chars.len(), dialogues, pages
    )
}
