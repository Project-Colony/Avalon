use super::compiler::{CompileContent, CompileOptions, SeparatorType};
use anyhow::Result;
use std::fmt::Write;

/// Compile to RTF (Rich Text Format)
pub fn compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    let estimated_size: usize = contents
        .iter()
        .map(|c| c.text.len() + c.title.len() + 60)
        .sum::<usize>()
        + 512;
    let mut rtf = String::with_capacity(estimated_size);

    // RTF header
    rtf.push_str("{\\rtf1\\ansi\\ansicpg1252\\deff0\n");

    // Font table
    rtf.push_str("{\\fonttbl\n");
    writeln!(rtf, "{{\\f0\\froman\\fcharset0 {};}}", rtf_escape(&options.font_family)).unwrap();
    rtf.push_str("{\\f1\\fswiss\\fcharset0 Arial;}\n");
    rtf.push_str("{\\f2\\fmodern\\fcharset0 Courier New;}\n");
    rtf.push_str("}\n");

    // Color table (black, gray, red for annotations, blue for links)
    rtf.push_str("{\\colortbl;\\red0\\green0\\blue0;\\red128\\green128\\blue128;\\red200\\green50\\blue50;\\red50\\green50\\blue200;}\n");

    // Default font size (in half-points)
    let fs = (options.font_size * 2.0) as u32;
    writeln!(rtf, "\\f0\\fs{}", fs).unwrap();

    // Title page
    if options.include_front_matter && !options.title.is_empty() {
        let title_fs = fs * 2;
        writeln!(
            rtf,
            "\\pard\\qc\\fs{} \\b {}\\b0\\par",
            title_fs,
            rtf_escape(&options.title)
        )
        .unwrap();
        if !options.author.is_empty() {
            writeln!(rtf, "\\pard\\qc\\fs{} \\i {}\\i0\\par", fs, rtf_escape(&options.author)).unwrap();
        }
        rtf.push_str("\\page\n");
    }

    let mut prev_was_text = false;

    for content in contents {
        if content.is_folder {
            // Page break before top-level folders
            if content.depth == 0 && prev_was_text && options.page_break_between_folders {
                rtf.push_str("\\page\n");
            }

            // Heading
            let heading_fs = match content.depth {
                0 => (fs as f32 * 1.5) as u32,
                1 => (fs as f32 * 1.3) as u32,
                2 => (fs as f32 * 1.1) as u32,
                _ => fs,
            };
            writeln!(
                rtf,
                "\\pard\\sb240\\sa120\\fs{} \\b {}\\b0\\par",
                heading_fs,
                rtf_escape(&content.title)
            )
            .unwrap();
            prev_was_text = false;
        } else {
            // Separator between consecutive text docs
            if prev_was_text {
                rtf.push_str(&separator_rtf(&options.separator));
            }

            // Body text — split into paragraphs
            for paragraph in content.text.split("\n\n") {
                let trimmed = paragraph.trim();
                if trimmed.is_empty() {
                    rtf.push_str("\\par\n");
                    continue;
                }

                // Handle blockquotes
                if let Some(quoted) = trimmed.strip_prefix("> ") {
                    let quote_text = convert_basic_markdown(quoted);
                    writeln!(rtf, "\\pard\\li720\\ri720\\sa60\\fs{} \\i {}\\i0\\par", fs, quote_text).unwrap();
                    continue;
                }

                // Handle heading lines within markdown
                if trimmed.starts_with('#') {
                    let level = trimmed.chars().take_while(|c| *c == '#').count();
                    let heading_text = trimmed[level..].trim();
                    let h_fs = match level {
                        1 => (fs as f32 * 1.5) as u32,
                        2 => (fs as f32 * 1.3) as u32,
                        _ => (fs as f32 * 1.1) as u32,
                    };
                    writeln!(
                        rtf,
                        "\\pard\\sb120\\sa60\\fs{} \\b {}\\b0\\par",
                        h_fs,
                        rtf_escape(heading_text)
                    )
                    .unwrap();
                    continue;
                }

                // Handle basic markdown inline formatting
                let text = convert_basic_markdown(trimmed);
                writeln!(rtf, "\\pard\\fi360\\sa60\\fs{} {}\\par", fs, text).unwrap();
            }
            prev_was_text = true;
        }
    }

    rtf.push_str("}\n");
    Ok(rtf)
}

fn separator_rtf(sep: &SeparatorType) -> String {
    match sep {
        SeparatorType::EmptyLine => "\\par\\par\n".to_string(),
        SeparatorType::PageBreak => "\\page\n".to_string(),
        SeparatorType::SectionBreak => "\\pard\\qc\\sa120\\sb120 * * *\\par\n".to_string(),
        SeparatorType::Custom(s) => {
            format!("\\pard\\qc\\sa120\\sb120 {}\\par\n", rtf_escape(s))
        }
        SeparatorType::None => String::new(),
    }
}

/// Escape special RTF characters
fn rtf_escape(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' => result.push_str("\\\\"),
            '{' => result.push_str("\\{"),
            '}' => result.push_str("\\}"),
            c if c as u32 > 127 => {
                write!(result, "\\u{}?", c as i32).unwrap();
            }
            c => result.push(c),
        }
    }
    result
}

/// Convert basic Markdown formatting to RTF
fn convert_basic_markdown(text: &str) -> String {
    let escaped = rtf_escape(text);

    // Bold: **text** -> \b text\b0
    let mut result = String::new();
    let mut chars = escaped.chars().peekable();
    let mut in_bold = false;
    let mut in_italic = false;

    while let Some(ch) = chars.next() {
        if ch == '*' {
            if chars.peek() == Some(&'*') {
                chars.next(); // consume second *
                if in_bold {
                    result.push_str("\\b0 ");
                    in_bold = false;
                } else {
                    result.push_str("\\b ");
                    in_bold = true;
                }
            } else if in_italic {
                result.push_str("\\i0 ");
                in_italic = false;
            } else {
                result.push_str("\\i ");
                in_italic = true;
            }
        } else {
            result.push(ch);
        }
    }

    // Close any unclosed formatting
    if in_bold {
        result.push_str("\\b0 ");
    }
    if in_italic {
        result.push_str("\\i0 ");
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
    fn test_compile_basic_structure() {
        let contents = vec![
            CompileContent {
                title: "Ch1".into(),
                text: String::new(),
                depth: 0,
                is_folder: true,
            },
            CompileContent {
                title: "Scene".into(),
                text: "Hello.".into(),
                depth: 1,
                is_folder: false,
            },
        ];
        let output = compile(&contents, &default_options()).unwrap();
        assert!(output.starts_with("{\\rtf1\\ansi"));
        assert!(output.ends_with("}\n"));
        assert!(output.contains("Ch1"));
        assert!(output.contains("Hello."));
    }

    #[test]
    fn test_compile_with_front_matter() {
        let mut opts = default_options();
        opts.include_front_matter = true;
        opts.title = "My Book".into();
        opts.author = "Author".into();
        let output = compile(&[], &opts).unwrap();
        assert!(output.contains("My Book"));
        assert!(output.contains("Author"));
        assert!(output.contains("\\page"));
    }

    #[test]
    fn test_rtf_escape() {
        assert_eq!(rtf_escape("a\\b{c}"), "a\\\\b\\{c\\}");
    }

    #[test]
    fn test_rtf_escape_unicode() {
        let result = rtf_escape("café");
        assert!(result.contains("\\u"));
    }

    #[test]
    fn test_convert_basic_markdown_bold() {
        let result = convert_basic_markdown("**bold** text");
        assert!(result.contains("\\b "));
        assert!(result.contains("\\b0 "));
    }

    #[test]
    fn test_convert_basic_markdown_italic() {
        let result = convert_basic_markdown("*italic* text");
        assert!(result.contains("\\i "));
        assert!(result.contains("\\i0 "));
    }
}
