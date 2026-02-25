use std::fmt::Write;
use anyhow::Result;
use super::compiler::{self, CompileContent, CompileOptions, SeparatorType};

pub fn compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    let estimated_size: usize = contents.iter().map(|c| c.text.len() + c.title.len() + 20).sum();
    let mut output = String::with_capacity(estimated_size);

    // YAML front matter block (common in Markdown publishing)
    if options.include_front_matter && !options.title.is_empty() {
        output.push_str("---\n");
        let _ = writeln!(output,"title: \"{}\"", escape_yaml(&options.title));
        if !options.author.is_empty() {
            let _ = writeln!(output,"author: \"{}\"", escape_yaml(&options.author));
        }
        let _ = writeln!(output,"date: \"{}\"", chrono::Local::now().format(crate::core::DATE_FORMAT));

        // Word count metadata
        let total_words = compiler::total_word_count(contents);
        let _ = writeln!(output,"wordcount: {}", total_words);

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
                let _ = writeln!(output,
                    "{}- [{}](#{})",
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

fn slug(title: &str) -> String { compiler::slug(title) }
