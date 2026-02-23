use std::fmt::Write;
use anyhow::Result;
use super::compiler::{CompileContent, CompileOptions, SeparatorType};

pub fn compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    let mut output = String::new();

    // YAML front matter block (common in Markdown publishing)
    if options.include_front_matter && !options.title.is_empty() {
        output.push_str("---\n");
        let _ = write!(output,"title: \"{}\"\n", escape_yaml(&options.title));
        if !options.author.is_empty() {
            let _ = write!(output,"author: \"{}\"\n", escape_yaml(&options.author));
        }
        let _ = write!(output,"date: \"{}\"\n", chrono::Local::now().format("%Y-%m-%d"));

        // Word count metadata
        let total_words: usize = contents.iter().map(|c| c.text.split_whitespace().count()).sum();
        let _ = write!(output,"wordcount: {}\n", total_words);

        output.push_str("---\n\n");

        // Title as H1
        let _ = write!(output,"# {}\n\n", options.title);
        if !options.author.is_empty() {
            let _ = write!(output,"*by {}*\n\n", options.author);
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
                let _ = write!(output,
                    "{}- [{}](#{})\n",
                    indent, content.title, anchor
                );
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
            let _ = write!(output,"{} {}\n\n", hashes, content.title);
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
                        let _ = write!(output,"\n{}\n\n", s);
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

    #[test]
    fn test_compile_empty() {
        let contents: Vec<CompileContent> = vec![];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_compile_page_break_separator() {
        let mut opts = make_opts();
        opts.separator = SeparatorType::PageBreak;
        let contents = vec![
            make_content("A", "First.", false, 1),
            make_content("B", "Second.", false, 1),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("---"));
    }

    #[test]
    fn test_compile_custom_separator() {
        let mut opts = make_opts();
        opts.separator = SeparatorType::Custom("===".to_string());
        let contents = vec![
            make_content("A", "First.", false, 1),
            make_content("B", "Second.", false, 1),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("==="));
    }

    #[test]
    fn test_compile_no_separator() {
        let mut opts = make_opts();
        opts.separator = SeparatorType::None;
        let contents = vec![
            make_content("A", "First.", false, 1),
            make_content("B", "Second.", false, 1),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("First."));
        assert!(result.contains("Second."));
    }

    #[test]
    fn test_compile_toc() {
        let mut opts = make_opts();
        opts.include_toc = true;
        let contents = vec![
            make_content("Ch 1", "", true, 1),
            make_content("Scene 1", "Text.", false, 2),
            make_content("Ch 2", "", true, 1),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("## Table of Contents"));
        assert!(result.contains("[Ch 1]"));
        assert!(result.contains("[Ch 2]"));
    }

    #[test]
    fn test_compile_page_break_between_folders() {
        let mut opts = make_opts();
        opts.page_break_between_folders = true;
        let contents = vec![
            make_content("Chapter 1", "", true, 1),
            make_content("Scene", "Text.", false, 2),
            make_content("Chapter 2", "", true, 1),
        ];
        let result = compile(&contents, &opts).unwrap();
        // Should have --- between chapters
        let parts: Vec<&str> = result.split("---").collect();
        assert!(parts.len() >= 2);
    }

    #[test]
    fn test_compile_heading_depth_capping() {
        let contents = vec![
            make_content("Level 5", "", true, 4),
            make_content("Level 6", "", true, 5),
            make_content("Level 7 (capped)", "", true, 6),
        ];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("###### Level 5"));
        // Depth 5+2=7, capped to 6
        assert!(result.contains("###### Level 6"));
    }

    #[test]
    fn test_compile_front_matter_yaml() {
        let mut opts = make_opts();
        opts.include_front_matter = true;
        let contents = vec![
            make_content("Scene", "Two words.", false, 1),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("wordcount: 2"));
        assert!(result.contains("author: \"Test Author\""));
    }

    #[test]
    fn test_compile_front_matter_no_author() {
        let mut opts = make_opts();
        opts.include_front_matter = true;
        opts.author = String::new();
        let contents = vec![
            make_content("Scene", "Content.", false, 1),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("title:"));
        assert!(!result.contains("author:"));
    }

    #[test]
    fn test_slug_edge_cases() {
        assert_eq!(slug(""), "");
        assert_eq!(slug("---"), "");
        assert_eq!(slug("Hello!!! World???"), "hello-world");
        assert_eq!(slug("123"), "123");
    }

    #[test]
    fn test_escape_yaml_empty() {
        assert_eq!(escape_yaml(""), "");
    }

    #[test]
    fn test_char_count() {
        let contents = vec![
            make_content("A", "hello", false, 0),
            make_content("B", "world", false, 0),
        ];
        assert_eq!(char_count(&contents), 10);
    }

    #[test]
    fn test_estimate_pages() {
        let contents = vec![
            make_content("A", &"word ".repeat(750), false, 0),
        ];
        assert_eq!(estimate_pages(&contents), 3);
    }

    #[test]
    fn test_estimate_pages_empty() {
        let contents: Vec<CompileContent> = vec![];
        assert_eq!(estimate_pages(&contents), 1);
    }

    #[test]
    fn test_extract_headings_from_inline() {
        let contents = vec![
            make_content("Scene", "# Title\n## Sub\nText\n### Deep", false, 0),
        ];
        let headings = extract_headings(&contents);
        assert_eq!(headings.len(), 3);
        assert_eq!(headings[0], (1, "Title".to_string()));
        assert_eq!(headings[1], (2, "Sub".to_string()));
        assert_eq!(headings[2], (3, "Deep".to_string()));
    }

    #[test]
    fn test_extract_headings_empty() {
        let contents: Vec<CompileContent> = vec![];
        let headings = extract_headings(&contents);
        assert!(headings.is_empty());
    }

    #[test]
    fn test_word_count_empty() {
        let contents: Vec<CompileContent> = vec![];
        assert_eq!(word_count(&contents), 0);
    }
}
