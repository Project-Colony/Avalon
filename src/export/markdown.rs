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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::compiler::OutputFormat;

    fn make_opts() -> CompileOptions {
        CompileOptions {
            format: OutputFormat::Markdown,
            title: "Test Book".to_string(),
            author: "Test Author".to_string(),
            include_front_matter: false,
            separator: SeparatorType::EmptyLine,
            page_break_between_folders: false,
            compile_marked_only: false,
            font_size: 12.0,
            font_family: "Times New Roman".to_string(),
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
    fn test_compile_simple_text() {
        let contents = vec![
            make_content("Scene", "Hello world.", false, 1),
        ];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("Hello world."));
    }

    #[test]
    fn test_compile_with_front_matter() {
        let mut opts = make_opts();
        opts.include_front_matter = true;
        let contents = vec![
            make_content("Scene", "Text here.", false, 1),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("---"));
        assert!(result.contains("title: \"Test Book\""));
        assert!(result.contains("# Test Book"));
    }

    #[test]
    fn test_compile_with_folders() {
        let contents = vec![
            make_content("Chapter 1", "", true, 1),
            make_content("Scene A", "The story starts.", false, 2),
        ];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("### Chapter 1"));
        assert!(result.contains("The story starts."));
    }

    #[test]
    fn test_slug() {
        assert_eq!(slug("Hello World"), "hello-world");
        assert_eq!(slug("Chapter 1: The Beginning"), "chapter-1-the-beginning");
        assert_eq!(slug("It's A Test!"), "it-s-a-test");
    }

    #[test]
    fn test_escape_yaml() {
        assert_eq!(escape_yaml("plain"), "plain");
        assert_eq!(escape_yaml("with \"quotes\""), "with \\\"quotes\\\"");
        assert_eq!(escape_yaml("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn test_word_count() {
        let contents = vec![
            make_content("A", "one two three", false, 0),
            make_content("B", "four five", false, 0),
        ];
        assert_eq!(word_count(&contents), 5);
    }

    #[test]
    fn test_extract_headings() {
        let contents = vec![
            make_content("Part I", "", true, 0),
            make_content("Scene", "## Sub Heading\n\nText here.", false, 1),
        ];
        let headings = extract_headings(&contents);
        assert!(headings.len() >= 2);
    }

    #[test]
    fn test_section_separator() {
        let mut opts = make_opts();
        opts.separator = SeparatorType::SectionBreak;
        let contents = vec![
            make_content("A", "First text.", false, 1),
            make_content("B", "Second text.", false, 1),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("* * *"));
    }
}
