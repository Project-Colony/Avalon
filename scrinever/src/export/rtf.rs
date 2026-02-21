use anyhow::Result;
use super::compiler::{CompileContent, CompileOptions};

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

    // Color table
    rtf.push_str("{\\colortbl;\\red0\\green0\\blue0;\\red128\\green128\\blue128;}\n");

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

    for content in contents {
        if content.is_folder {
            // Heading
            let heading_fs = match content.depth {
                0 => (fs as f32 * 1.5) as u32,
                1 => (fs as f32 * 1.3) as u32,
                _ => (fs as f32 * 1.1) as u32,
            };
            rtf.push_str(&format!(
                "\\pard\\sb240\\sa120\\fs{} \\b {}\\b0\\par\n",
                heading_fs,
                rtf_escape(&content.title)
            ));
        } else {
            // Body text — split into paragraphs
            for paragraph in content.text.split("\n\n") {
                let trimmed = paragraph.trim();
                if trimmed.is_empty() {
                    rtf.push_str("\\par\n");
                    continue;
                }

                // Handle basic markdown inline formatting
                let text = convert_basic_markdown(trimmed);
                rtf.push_str(&format!("\\pard\\fi360\\sa60\\fs{} {}\\par\n", fs, text));
            }
        }
    }

    rtf.push_str("}\n");
    Ok(rtf)
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
