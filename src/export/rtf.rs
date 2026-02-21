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
