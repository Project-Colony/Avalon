use super::compiler::{CompileContent, CompileOptions, SeparatorType};
use anyhow::Result;
use std::fmt::Write;

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
            write!(output, "by {}", options.author).unwrap();
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
                    write!(output, "{}* {}", indent, content.title).unwrap();
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
                        write!(output, "\n{}\n\n", s).unwrap();
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

#[cfg(test)]
mod tests {
    use super::super::compiler::{CompileContent, CompileOptions};
    use super::*;

    fn default_options() -> CompileOptions {
        CompileOptions {
            include_front_matter: false,
            ..Default::default()
        }
    }

    #[test]
    fn test_compile_basic() {
        let contents = vec![
            CompileContent {
                title: "Ch 1".into(),
                text: String::new(),
                depth: 0,
                is_folder: true,
            },
            CompileContent {
                title: "Scene".into(),
                text: "Hello **world**.".into(),
                depth: 1,
                is_folder: false,
            },
        ];
        let output = compile(&contents, &default_options()).unwrap();
        assert!(output.contains("CH 1"));
        assert!(output.contains("Hello world.")); // markdown stripped
        assert!(!output.contains("**"));
    }

    #[test]
    fn test_compile_with_front_matter() {
        let mut opts = default_options();
        opts.include_front_matter = true;
        opts.title = "My Book".into();
        opts.author = "Author".into();
        let output = compile(&[], &opts).unwrap();
        assert!(output.contains("MY BOOK"));
        assert!(output.contains("by Author"));
    }

    #[test]
    fn test_strip_markdown_headings() {
        assert_eq!(strip_markdown("# Title").trim(), "Title");
        assert_eq!(strip_markdown("## Sub").trim(), "Sub");
    }

    #[test]
    fn test_strip_markdown_bold_italic() {
        assert_eq!(strip_markdown("**bold**").trim(), "bold");
        assert_eq!(strip_markdown("~~struck~~").trim(), "struck");
    }

    #[test]
    fn test_strip_markdown_blockquote() {
        assert_eq!(strip_markdown("> quoted").trim(), "quoted");
    }
}
