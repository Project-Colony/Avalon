use super::compiler::{self, CompileContent, CompileOptions, SeparatorType};
use anyhow::Result;
use std::fmt::Write;

pub fn compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    let estimated_size: usize = contents.iter().map(|c| c.text.len() + c.title.len() + 20).sum();
    let mut output = String::with_capacity(estimated_size);

    // YAML front matter block (common in Markdown publishing)
    if options.include_front_matter && !options.title.is_empty() {
        output.push_str("---\n");
        writeln!(output, "title: \"{}\"", escape_yaml(&options.title)).unwrap();
        if !options.author.is_empty() {
            writeln!(output, "author: \"{}\"", escape_yaml(&options.author)).unwrap();
        }
        writeln!(
            output,
            "date: \"{}\"",
            chrono::Local::now().format(crate::core::DATE_FORMAT)
        )
        .unwrap();

        // Word count metadata
        let total_words = compiler::total_word_count(contents);
        writeln!(output, "wordcount: {}", total_words).unwrap();

        output.push_str("---\n\n");

        // Title as H1
        write!(output, "# {}\n\n", options.title).unwrap();
        if !options.author.is_empty() {
            write!(output, "*by {}*\n\n", options.author).unwrap();
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
                writeln!(output, "{}- [{}](#{})", indent, content.title, anchor).unwrap();
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
            write!(output, "{} {}\n\n", hashes, content.title).unwrap();
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
                        write!(output, "\n{}\n\n", s).unwrap();
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
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn slug(title: &str) -> String {
    compiler::slug(title)
}

#[cfg(test)]
mod tests {
    use super::super::compiler::{CompileContent, CompileOptions, SeparatorType};
    use super::*;

    fn sample_contents() -> Vec<CompileContent> {
        vec![
            CompileContent {
                title: "Part One".into(),
                text: String::new(),
                depth: 0,
                is_folder: true,
            },
            CompileContent {
                title: "Chapter 1".into(),
                text: "First paragraph.".into(),
                depth: 1,
                is_folder: false,
            },
        ]
    }

    fn default_options() -> CompileOptions {
        CompileOptions {
            include_front_matter: false,
            ..Default::default()
        }
    }

    #[test]
    fn test_compile_basic() {
        let output = compile(&sample_contents(), &default_options()).unwrap();
        assert!(output.contains("## Part One"));
        assert!(output.contains("First paragraph."));
    }

    #[test]
    fn test_compile_front_matter_yaml() {
        let mut opts = default_options();
        opts.include_front_matter = true;
        opts.title = "My Novel".into();
        opts.author = "Jane".into();
        let output = compile(&sample_contents(), &opts).unwrap();
        assert!(output.contains("---"));
        assert!(output.contains("title: \"My Novel\""));
        assert!(output.contains("author: \"Jane\""));
        assert!(output.contains("# My Novel"));
    }

    #[test]
    fn test_compile_with_toc() {
        let mut opts = default_options();
        opts.include_toc = true;
        let output = compile(&sample_contents(), &opts).unwrap();
        assert!(output.contains("## Table of Contents"));
        assert!(output.contains("[Part One]"));
    }

    #[test]
    fn test_compile_empty() {
        let output = compile(&[], &default_options()).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_separator_custom() {
        let contents = vec![
            CompileContent {
                title: "A".into(),
                text: "Text A".into(),
                depth: 0,
                is_folder: false,
            },
            CompileContent {
                title: "B".into(),
                text: "Text B".into(),
                depth: 0,
                is_folder: false,
            },
        ];
        let mut opts = default_options();
        opts.separator = SeparatorType::Custom("~~~".into());
        let output = compile(&contents, &opts).unwrap();
        assert!(output.contains("~~~"));
    }

    #[test]
    fn test_escape_yaml() {
        assert_eq!(escape_yaml("hello \"world\""), "hello \\\"world\\\"");
        assert_eq!(escape_yaml("back\\slash"), "back\\\\slash");
    }
}
