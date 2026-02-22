use anyhow::Result;
use super::compiler::{CompileContent, CompileOptions, SeparatorType};

/// Compile to RTF (Rich Text Format)
pub fn compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    let mut rtf = String::new();

    // RTF header
    rtf.push_str("{\\rtf1\\ansi\\ansicpg1252\\deff0\n");

    // Font table
    rtf.push_str("{\\fonttbl\n");
    rtf.push_str(&format!("{{\\f0\\froman\\fcharset0 {};}}\n", rtf_escape(&options.font_family)));
    rtf.push_str("{\\f1\\fswiss\\fcharset0 Arial;}\n");
    rtf.push_str("{\\f2\\fmodern\\fcharset0 Courier New;}\n");
    rtf.push_str("}\n");

    // Color table (black, gray, red for annotations, blue for links)
    rtf.push_str("{\\colortbl;\\red0\\green0\\blue0;\\red128\\green128\\blue128;\\red200\\green50\\blue50;\\red50\\green50\\blue200;}\n");

    // Default font size (in half-points)
    let fs = (options.font_size * 2.0) as u32;
    rtf.push_str(&format!("\\f0\\fs{}\n", fs));

    // Title page
    if options.include_front_matter && !options.title.is_empty() {
        let title_fs = fs * 2;
        rtf.push_str(&format!(
            "\\pard\\qc\\fs{} \\b {}\\b0\\par\n",
            title_fs,
            rtf_escape(&options.title)
        ));
        if !options.author.is_empty() {
            rtf.push_str(&format!(
                "\\pard\\qc\\fs{} \\i {}\\i0\\par\n",
                fs,
                rtf_escape(&options.author)
            ));
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
            rtf.push_str(&format!(
                "\\pard\\sb240\\sa120\\fs{} \\b {}\\b0\\par\n",
                heading_fs,
                rtf_escape(&content.title)
            ));
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
                if trimmed.starts_with("> ") {
                    let quote_text = convert_basic_markdown(&trimmed[2..]);
                    rtf.push_str(&format!(
                        "\\pard\\li720\\ri720\\sa60\\fs{} \\i {}\\i0\\par\n",
                        fs, quote_text
                    ));
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
                    rtf.push_str(&format!(
                        "\\pard\\sb120\\sa60\\fs{} \\b {}\\b0\\par\n",
                        h_fs,
                        rtf_escape(heading_text)
                    ));
                    continue;
                }

                // Handle basic markdown inline formatting
                let text = convert_basic_markdown(trimmed);
                rtf.push_str(&format!("\\pard\\fi360\\sa60\\fs{} {}\\par\n", fs, text));
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
        SeparatorType::SectionBreak => {
            "\\pard\\qc\\sa120\\sb120 * * *\\par\n".to_string()
        }
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
                result.push_str(&format!("\\u{}?", c as i32));
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
            } else {
                if in_italic {
                    result.push_str("\\i0 ");
                    in_italic = false;
                } else {
                    result.push_str("\\i ");
                    in_italic = true;
                }
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

/// Estimate the output size in bytes for an RTF compilation
pub fn estimate_output_size(contents: &[CompileContent], options: &CompileOptions) -> usize {
    // RTF header + font table + color table ~300 bytes
    let base = 300;
    let front_matter = if options.include_front_matter { 150 } else { 0 };
    // RTF control words roughly triple the content size
    let content_size: usize = contents.iter()
        .map(|c| c.text.len() * 3 + c.title.len() + 80)
        .sum();
    base + front_matter + content_size
}

/// Count total words across all content sections
pub fn word_count(contents: &[CompileContent]) -> usize {
    contents.iter().map(|c| c.text.split_whitespace().count()).sum()
}

/// Count total characters across all content sections
pub fn char_count(contents: &[CompileContent]) -> usize {
    contents.iter().map(|c| c.text.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_options() -> CompileOptions {
        CompileOptions {
            title: "Test Book".to_string(),
            author: "Author".to_string(),
            include_front_matter: true,
            ..Default::default()
        }
    }

    fn make_content(title: &str, text: &str, is_folder: bool) -> CompileContent {
        CompileContent {
            title: title.to_string(),
            text: text.to_string(),
            is_folder,
            depth: 0,
        }
    }

    #[test]
    fn test_rtf_escape_basic() {
        assert_eq!(rtf_escape("hello"), "hello");
        assert_eq!(rtf_escape("a\\b"), "a\\\\b");
        assert_eq!(rtf_escape("a{b}"), "a\\{b\\}");
    }

    #[test]
    fn test_rtf_escape_unicode() {
        let result = rtf_escape("caf\u{00E9}");
        assert!(result.contains("\\u233?"));
    }

    #[test]
    fn test_compile_empty() {
        let options = make_options();
        let result = compile(&[], &options).unwrap();
        assert!(result.starts_with("{\\rtf1"));
        assert!(result.ends_with("}\n"));
    }

    #[test]
    fn test_compile_with_content() {
        let options = make_options();
        let contents = vec![
            make_content("Chapter 1", "Hello world.", false),
        ];
        let result = compile(&contents, &options).unwrap();
        assert!(result.contains("Hello world."));
        assert!(result.contains("Test Book"));
        assert!(result.contains("Author"));
    }

    #[test]
    fn test_compile_no_front_matter() {
        let mut options = make_options();
        options.include_front_matter = false;
        let contents = vec![
            make_content("Ch1", "Text here.", false),
        ];
        let result = compile(&contents, &options).unwrap();
        assert!(!result.contains("Test Book")); // No title page
        assert!(result.contains("Text here."));
    }

    #[test]
    fn test_compile_folder_heading() {
        let options = make_options();
        let contents = vec![
            make_content("Act One", "", true),
            make_content("Scene 1", "First scene.", false),
        ];
        let result = compile(&contents, &options).unwrap();
        assert!(result.contains("Act One"));
        assert!(result.contains("First scene."));
    }

    #[test]
    fn test_convert_markdown_bold() {
        let result = convert_basic_markdown("Hello **bold** text");
        assert!(result.contains("\\b "));
        assert!(result.contains("\\b0 "));
    }

    #[test]
    fn test_convert_markdown_italic() {
        let result = convert_basic_markdown("Hello *italic* text");
        assert!(result.contains("\\i "));
        assert!(result.contains("\\i0 "));
    }

    #[test]
    fn test_separator_types() {
        assert!(separator_rtf(&SeparatorType::EmptyLine).contains("\\par"));
        assert!(separator_rtf(&SeparatorType::PageBreak).contains("\\page"));
        assert!(separator_rtf(&SeparatorType::SectionBreak).contains("* * *"));
        assert!(separator_rtf(&SeparatorType::None).is_empty());
        assert!(separator_rtf(&SeparatorType::Custom("---".to_string())).contains("---"));
    }

    #[test]
    fn test_word_count() {
        let contents = vec![
            make_content("Ch1", "one two three", false),
            make_content("Ch2", "four five", false),
        ];
        assert_eq!(word_count(&contents), 5);
    }

    #[test]
    fn test_char_count_fn() {
        let contents = vec![
            make_content("Ch1", "abc", false),
            make_content("Ch2", "de", false),
        ];
        assert_eq!(char_count(&contents), 5);
    }

    #[test]
    fn test_estimate_output_size() {
        let contents = vec![
            make_content("Ch1", "Some text here.", false),
        ];
        let options = make_options();
        let size = estimate_output_size(&contents, &options);
        assert!(size > 300); // Base + some content
    }

    #[test]
    fn test_compile_with_front_matter_no_author() {
        let mut options = make_options();
        options.author = String::new();
        let contents = vec![make_content("Ch1", "Text.", false)];
        let result = compile(&contents, &options).unwrap();
        assert!(result.contains("Test Book"));
        assert!(!result.contains("\\i Author")); // No author italic block
    }

    #[test]
    fn test_compile_multiple_docs_separator() {
        let options = make_options();
        let contents = vec![
            make_content("Ch1", "First.", false),
            make_content("Ch2", "Second.", false),
        ];
        let result = compile(&contents, &options).unwrap();
        assert!(result.contains("First."));
        assert!(result.contains("Second."));
    }

    #[test]
    fn test_compile_page_break_separator() {
        let mut options = make_options();
        options.separator = SeparatorType::PageBreak;
        let contents = vec![
            make_content("Ch1", "First.", false),
            make_content("Ch2", "Second.", false),
        ];
        let result = compile(&contents, &options).unwrap();
        assert!(result.contains("\\page"));
    }

    #[test]
    fn test_compile_section_break_separator() {
        let mut options = make_options();
        options.separator = SeparatorType::SectionBreak;
        let contents = vec![
            make_content("Ch1", "First.", false),
            make_content("Ch2", "Second.", false),
        ];
        let result = compile(&contents, &options).unwrap();
        assert!(result.contains("* * *"));
    }

    #[test]
    fn test_compile_no_separator() {
        let mut options = make_options();
        options.separator = SeparatorType::None;
        let contents = vec![
            make_content("Ch1", "First.", false),
            make_content("Ch2", "Second.", false),
        ];
        let result = compile(&contents, &options).unwrap();
        assert!(result.contains("First."));
        assert!(result.contains("Second."));
    }

    #[test]
    fn test_compile_blockquote() {
        let options = make_options();
        let contents = vec![make_content("Ch1", "> A quote here.", false)];
        let result = compile(&contents, &options).unwrap();
        assert!(result.contains("A quote here."));
        assert!(result.contains("\\li720")); // Indentation for blockquote
    }

    #[test]
    fn test_compile_heading_in_content() {
        let options = make_options();
        let contents = vec![make_content("Ch1", "# Section Title", false)];
        let result = compile(&contents, &options).unwrap();
        assert!(result.contains("Section Title"));
        assert!(result.contains("\\b ")); // Bold for heading
    }

    #[test]
    fn test_convert_markdown_unclosed_bold() {
        let result = convert_basic_markdown("**unclosed bold");
        assert!(result.contains("\\b "));
        assert!(result.contains("\\b0 ")); // Should auto-close
    }

    #[test]
    fn test_convert_markdown_unclosed_italic() {
        let result = convert_basic_markdown("*unclosed italic");
        assert!(result.contains("\\i "));
        assert!(result.contains("\\i0 ")); // Should auto-close
    }

    #[test]
    fn test_rtf_escape_backslash() {
        assert_eq!(rtf_escape("a\\b\\c"), "a\\\\b\\\\c");
    }

    #[test]
    fn test_estimate_size_front_matter_difference() {
        let contents = vec![make_content("Ch1", "text", false)];
        let mut opts = make_options();
        opts.include_front_matter = true;
        let size_fm = estimate_output_size(&contents, &opts);
        opts.include_front_matter = false;
        let size_no = estimate_output_size(&contents, &opts);
        assert!(size_fm > size_no);
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
    fn test_compile_empty_paragraphs() {
        let options = make_options();
        let contents = vec![make_content("Ch1", "First\n\n\n\nSecond", false)];
        let result = compile(&contents, &options).unwrap();
        assert!(result.contains("First"));
        assert!(result.contains("Second"));
    }
}
