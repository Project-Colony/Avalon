use std::fmt::Write;
use anyhow::Result;
use super::compiler::{self, CompileContent, CompileOptions, SeparatorType};

pub fn compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    let estimated_size: usize = contents.iter().map(|c| c.text.len() + c.title.len() + 20).sum();
    let mut output = String::with_capacity(estimated_size);

    // Front matter / title page
    if options.include_front_matter && !options.title.is_empty() {
        let title = options.title.to_uppercase();
        output.push('\n');
        output.push_str(&title);
        output.push('\n');
        output.push_str(&"=".repeat(title.len()));
        output.push('\n');

        if !options.author.is_empty() {
            let _ = write!(output, "by {}", options.author);
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
                    let _ = write!(output, "{}* {}", indent, content.title);
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
                        let _ = write!(output, "\n{}\n\n", s);
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
    compiler::total_word_count(contents)
}

/// Count total characters across all content sections
pub fn char_count(contents: &[CompileContent]) -> usize {
    contents.iter().map(|c| c.text.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::compiler::OutputFormat;

    fn make_opts() -> CompileOptions {
        CompileOptions {
            format: OutputFormat::PlainText,
            title: "Test Book".to_string(),
            author: "Author Name".to_string(),
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
    fn test_compile_simple() {
        let contents = vec![
            make_content("Scene 1", "Hello world.", false, 1),
        ];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("Hello world."));
    }

    #[test]
    fn test_compile_with_front_matter() {
        let mut opts = make_opts();
        opts.include_front_matter = true;
        let contents = vec![
            make_content("Scene", "Content here.", false, 1),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("TEST BOOK"));
        assert!(result.contains("Author Name"));
    }

    #[test]
    fn test_compile_with_folders() {
        let contents = vec![
            make_content("Chapter One", "", true, 1),
            make_content("Scene 1", "The story begins.", false, 2),
        ];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("CHAPTER ONE"));
        assert!(result.contains("The story begins."));
    }

    #[test]
    fn test_strip_markdown() {
        let bold = strip_markdown("**bold**");
        assert!(bold.trim() == "bold");
        let strike = strip_markdown("~~strikethrough~~");
        assert!(strike.trim() == "strikethrough");
        let heading = strip_markdown("# Heading");
        assert!(heading.trim() == "Heading");
    }

    #[test]
    fn test_word_wrap() {
        let text = "This is a long line that should be wrapped at a certain width for readability";
        let wrapped = word_wrap(text, 20);
        for line in wrapped.lines() {
            assert!(line.len() <= 25); // Allow some slack for long words
        }
    }

    #[test]
    fn test_word_count_and_char_count() {
        let contents = vec![
            make_content("A", "one two three", false, 0),
            make_content("B", "four five", false, 0),
        ];
        assert_eq!(word_count(&contents), 5);
        assert!(char_count(&contents) > 0);
    }

    #[test]
    fn test_estimate_pages() {
        let contents = vec![
            make_content("A", &"word ".repeat(500), false, 0),
        ];
        assert_eq!(estimate_pages(&contents), 2);
    }

    #[test]
    fn test_compile_empty() {
        let contents: Vec<CompileContent> = vec![];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_compile_section_break_separator() {
        let mut opts = make_opts();
        opts.separator = SeparatorType::SectionBreak;
        let contents = vec![
            make_content("A", "First.", false, 1),
            make_content("B", "Second.", false, 1),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("* * *"));
    }

    #[test]
    fn test_compile_custom_separator() {
        let mut opts = make_opts();
        opts.separator = SeparatorType::Custom("~~~".to_string());
        let contents = vec![
            make_content("A", "First.", false, 1),
            make_content("B", "Second.", false, 1),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("~~~"));
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
        assert!(result.contains("\u{2500}"));
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
    fn test_compile_folder_depth_2() {
        let contents = vec![
            make_content("Subsection", "", true, 2),
            make_content("Content", "Text here.", false, 2),
        ];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("Subsection"));
        assert!(result.contains("-".repeat(10).as_str())); // Underline with dashes
    }

    #[test]
    fn test_compile_folder_depth_3() {
        let contents = vec![
            make_content("Deep Section", "", true, 3),
            make_content("Content", "Deep text.", false, 3),
        ];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("* Deep Section"));
    }

    #[test]
    fn test_compile_page_break_between_folders() {
        let mut opts = make_opts();
        opts.page_break_between_folders = true;
        let contents = vec![
            make_content("Ch 1", "", true, 1),
            make_content("Scene 1", "Text.", false, 2),
            make_content("Ch 2", "", true, 1),
            make_content("Scene 2", "More text.", false, 2),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("\u{2500}")); // Page break between folders
    }

    #[test]
    fn test_strip_markdown_blockquote() {
        let result = strip_markdown("> This is a quote");
        assert!(result.contains("This is a quote"));
        assert!(!result.contains(">"));
    }

    #[test]
    fn test_strip_markdown_heading_levels() {
        assert!(strip_markdown("## Sub Heading").trim() == "Sub Heading");
        assert!(strip_markdown("### Third Level").trim() == "Third Level");
    }

    #[test]
    fn test_strip_markdown_triple_emphasis() {
        let result = strip_markdown("***bold italic***");
        assert!(result.trim() == "bold italic");
    }

    #[test]
    fn test_strip_markdown_no_formatting() {
        let result = strip_markdown("Plain text without formatting.");
        assert_eq!(result, "Plain text without formatting.");
    }

    #[test]
    fn test_word_wrap_short_line() {
        let text = "short";
        let wrapped = word_wrap(text, 80);
        assert_eq!(wrapped, "short");
    }

    #[test]
    fn test_word_wrap_exact_width() {
        let text = "hello world";
        let wrapped = word_wrap(text, 11);
        assert_eq!(wrapped, "hello world");
    }

    #[test]
    fn test_estimate_pages_empty() {
        let contents: Vec<CompileContent> = vec![];
        assert_eq!(estimate_pages(&contents), 1); // Minimum 1 page
    }

    #[test]
    fn test_word_count_empty() {
        let contents: Vec<CompileContent> = vec![];
        assert_eq!(word_count(&contents), 0);
    }

    #[test]
    fn test_char_count_empty() {
        let contents: Vec<CompileContent> = vec![];
        assert_eq!(char_count(&contents), 0);
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
        assert!(result.contains("TEST BOOK"));
        assert!(!result.contains("by "));
    }

    #[test]
    fn test_compile_front_matter_empty_title() {
        let mut opts = make_opts();
        opts.include_front_matter = true;
        opts.title = String::new();
        let contents = vec![
            make_content("Scene", "Content.", false, 1),
        ];
        let result = compile(&contents, &opts).unwrap();
        // Empty title — front matter should be skipped
        assert!(!result.contains("by "));
    }
}
