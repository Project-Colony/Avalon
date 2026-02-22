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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::compiler::OutputFormat;

    fn make_opts() -> CompileOptions {
        CompileOptions {
            format: OutputFormat::Fountain,
            title: "Test Screenplay".to_string(),
            author: "Screenwriter".to_string(),
            include_front_matter: false,
            separator: crate::export::compiler::SeparatorType::EmptyLine,
            page_break_between_folders: false,
            compile_marked_only: false,
            font_size: 12.0,
            font_family: "Courier".to_string(),
            include_toc: false,
            replace_placeholders: false,
        }
    }

    fn make_content(title: &str, text: &str, is_folder: bool, depth: usize) -> CompileContent {
        CompileContent {
            title: title.to_string(),
            text: text.to_string(),
            depth,
            is_folder,
        }
    }

    #[test]
    fn test_compile_basic() {
        let contents = vec![make_content("Scene 1", "Some action text.", false, 1)];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("Some action text."));
    }

    #[test]
    fn test_compile_with_front_matter() {
        let mut opts = make_opts();
        opts.include_front_matter = true;
        let contents = vec![make_content("Scene", "Text.", false, 1)];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("Title: Test Screenplay"));
        assert!(result.contains("Author: Screenwriter"));
    }

    #[test]
    fn test_compile_folder_as_act() {
        let contents = vec![
            make_content("Act I", "", true, 0),
            make_content("Scene 1", "Action.", false, 1),
        ];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("# ACT I"));
    }

    #[test]
    fn test_is_scene_heading() {
        assert!(is_scene_heading("INT. OFFICE - DAY"));
        assert!(is_scene_heading("EXT. PARK - NIGHT"));
        assert!(is_scene_heading("EST. CITY SKYLINE"));
        assert!(is_scene_heading("I/E. CAR - MOVING"));
        assert!(is_scene_heading(".FLASHBACK"));
        assert!(!is_scene_heading("Regular text"));
    }

    #[test]
    fn test_is_transition() {
        assert!(is_transition("CUT TO:"));
        assert!(is_transition("FADE OUT."));
        assert!(is_transition("FADE IN:"));
        assert!(is_transition("SMASH CUT:"));
        assert!(!is_transition("Regular text"));
    }

    #[test]
    fn test_is_character_cue() {
        assert!(is_character_cue("JOHN"));
        assert!(is_character_cue("MARY JANE"));
        assert!(!is_character_cue("hello"));
        assert!(!is_character_cue("INT. OFFICE"));
        assert!(!is_character_cue(""));
        assert!(!is_character_cue("A")); // Too short
    }

    #[test]
    fn test_scene_count() {
        let input = "INT. OFFICE - DAY\n\nSome action.\n\nEXT. PARK - NIGHT\n\nMore action.";
        assert_eq!(scene_count(input), 2);
    }

    #[test]
    fn test_scene_count_empty() {
        assert_eq!(scene_count("No scenes here."), 0);
    }

    #[test]
    fn test_extract_characters() {
        let input = "INT. OFFICE - DAY\n\nJOHN\nHello there.\n\nMARY\nHi John.\n\nJOHN\nHow are you?";
        let chars = extract_characters(input);
        assert!(chars.contains(&"JOHN".to_string()));
        assert!(chars.contains(&"MARY".to_string()));
        assert_eq!(chars.len(), 2); // JOHN should not be duplicated
    }

    #[test]
    fn test_dialogue_count() {
        let input = "INT. OFFICE\n\nJOHN\nHello.\n\nMARY\nHi.\n";
        assert_eq!(dialogue_count(input), 2);
    }

    #[test]
    fn test_parse_fountain() {
        let input = "INT. OFFICE - DAY\n\nSome action.\n\nEXT. PARK\n\nMore action.";
        let sections = parse_fountain(input);
        assert!(sections.len() >= 2);
    }

    #[test]
    fn test_parse_title_page() {
        let input = "Title: My Script\nAuthor: Writer\nDraft date: 2026-01-01\n\nINT. OFFICE";
        let meta = parse_title_page(input);
        assert_eq!(meta.len(), 3);
        assert_eq!(meta[0].0, "Title");
        assert_eq!(meta[0].1, "My Script");
        assert_eq!(meta[1].0, "Author");
    }

    #[test]
    fn test_estimate_page_count() {
        // 56 lines per page
        let input = (0..120).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n");
        let pages = estimate_page_count(&input);
        assert!(pages >= 2);
    }

    #[test]
    fn test_transition_count() {
        let input = "INT. OFFICE\n\nAction.\n\nCUT TO:\n\nEXT. PARK\n\nFADE OUT.";
        assert_eq!(transition_count(input), 2);
    }

    #[test]
    fn test_screenplay_summary() {
        let input = "INT. OFFICE - DAY\n\nJOHN\nHello.\n\nCUT TO:\n\nEXT. PARK\n\nMARY\nHi.";
        let summary = screenplay_summary(input);
        assert!(summary.contains("scenes"));
        assert!(summary.contains("characters"));
        assert!(summary.contains("dialogue blocks"));
        assert!(summary.contains("pages"));
    }

    #[test]
    fn test_compile_empty() {
        let contents: Vec<CompileContent> = vec![];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.is_empty() || result.trim().is_empty());
    }

    #[test]
    fn test_compile_front_matter_no_author() {
        let mut opts = make_opts();
        opts.include_front_matter = true;
        opts.author = String::new();
        let contents = vec![make_content("Scene", "Text.", false, 1)];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("Title: Test Screenplay"));
        assert!(!result.contains("Author:"));
    }

    #[test]
    fn test_compile_multiple_acts() {
        let contents = vec![
            make_content("Act I", "", true, 0),
            make_content("Scene 1", "Action.", false, 1),
            make_content("Act II", "", true, 0),
            make_content("Scene 2", "More action.", false, 1),
        ];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("# ACT I"));
        assert!(result.contains("# ACT II"));
    }

    #[test]
    fn test_prose_to_fountain_scene_heading() {
        let result = prose_to_fountain("INT. OFFICE - DAY\nSome action.");
        assert!(result.contains("INT. OFFICE - DAY"));
    }

    #[test]
    fn test_prose_to_fountain_transition() {
        let result = prose_to_fountain("CUT TO:");
        assert!(result.contains("CUT TO:"));
    }

    #[test]
    fn test_prose_to_fountain_centered() {
        let result = prose_to_fountain(">CENTERED TEXT<");
        assert!(result.contains(">CENTERED TEXT<"));
    }

    #[test]
    fn test_prose_to_fountain_parenthetical() {
        let result = prose_to_fountain("(whispering)");
        assert!(result.contains("(whispering)"));
    }

    #[test]
    fn test_prose_to_fountain_note() {
        let result = prose_to_fountain("[[This is a note]]");
        assert!(result.contains("[[This is a note]]"));
    }

    #[test]
    fn test_prose_to_fountain_page_break() {
        let result = prose_to_fountain("===");
        assert!(result.contains("==="));
    }

    #[test]
    fn test_is_scene_heading_int_space() {
        assert!(is_scene_heading("INT OFFICE - DAY"));
    }

    #[test]
    fn test_is_scene_heading_ie() {
        assert!(is_scene_heading("I/E CAR - MOVING"));
        assert!(is_scene_heading("I/E. CAR"));
    }

    #[test]
    fn test_is_character_cue_with_extension() {
        assert!(is_character_cue("JOHN (V.O.)"));
        assert!(is_character_cue("MARY (CONT'D)"));
    }

    #[test]
    fn test_extract_characters_strips_extensions() {
        let input = "INT. OFFICE\n\nJOHN (V.O.)\nSome dialogue.\n";
        let chars = extract_characters(input);
        assert!(chars.contains(&"JOHN".to_string()));
    }

    #[test]
    fn test_extract_characters_empty() {
        let chars = extract_characters("Just some action text, no characters.");
        assert!(chars.is_empty());
    }

    #[test]
    fn test_scene_count_forced() {
        // A . prefix forces a scene heading
        let input = ".FLASHBACK - NIGHT\nAction.\n";
        assert_eq!(scene_count(input), 1);
    }

    #[test]
    fn test_parse_fountain_empty() {
        let sections = parse_fountain("");
        assert!(sections.is_empty());
    }

    #[test]
    fn test_parse_title_page_empty() {
        let meta = parse_title_page("");
        assert!(meta.is_empty());
    }

    #[test]
    fn test_parse_title_page_no_colon() {
        let meta = parse_title_page("No colon here\n\nContent");
        assert!(meta.is_empty());
    }

    #[test]
    fn test_dialogue_count_empty() {
        assert_eq!(dialogue_count("No dialogue here."), 0);
    }

    #[test]
    fn test_transition_count_empty() {
        assert_eq!(transition_count("No transitions here."), 0);
    }

    #[test]
    fn test_estimate_page_count_short() {
        assert_eq!(estimate_page_count("Short script."), 1); // Min 1
    }
}
