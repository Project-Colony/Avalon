use std::path::Path;
use anyhow::Result;
use printpdf::*;

use super::compiler::{CompileContent, CompileOptions, SeparatorType};

pub fn save_pdf(contents: &[CompileContent], options: &CompileOptions, path: &Path) -> Result<()> {
    let (doc, page1, layer1) = PdfDocument::new(
        &options.title,
        Mm(210.0), // A4 width
        Mm(297.0), // A4 height
        "Layer 1",
    );

    let font = doc.add_builtin_font(BuiltinFont::TimesRoman)?;
    let bold_font = doc.add_builtin_font(BuiltinFont::TimesBold)?;

    let mut current_layer = doc.get_page(page1).get_layer(layer1);
    let mut y_position = 270.0_f32; // Start from top with margin
    let line_height = options.font_size * 0.5;
    let margin_left = 25.0_f32;
    let page_width = 160.0_f32; // Usable width in mm
    let chars_per_line = (page_width / (options.font_size * 0.2)) as usize;

    // Title page
    if options.include_front_matter && !options.title.is_empty() {
        current_layer.use_text(
            &options.title,
            options.font_size * 2.0,
            Mm(105.0 - (options.title.len() as f32 * options.font_size * 0.5)),
            Mm(200.0),
            &bold_font,
        );

        if !options.author.is_empty() {
            current_layer.use_text(
                &options.author,
                options.font_size,
                Mm(105.0 - (options.author.len() as f32 * options.font_size * 0.25)),
                Mm(180.0),
                &font,
            );
        }

        // New page for content
        let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
        current_layer = doc.get_page(page).get_layer(layer);
        y_position = 270.0;
    }

    for content in contents {
        if content.is_folder {
            // Folder heading
            let heading_size = match content.depth {
                0 => options.font_size * 1.5,
                1 => options.font_size * 1.3,
                _ => options.font_size * 1.1,
            };

            if y_position < 30.0 {
                let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                current_layer = doc.get_page(page).get_layer(layer);
                y_position = 270.0;
            }

            current_layer.use_text(
                &content.title,
                heading_size,
                Mm(margin_left),
                Mm(y_position),
                &bold_font,
            );
            y_position -= line_height * 2.0;
        } else {
            // Document text — wrap lines
            for line in content.text.lines() {
                if line.is_empty() {
                    y_position -= line_height;
                    continue;
                }

                // Simple word wrapping
                let words: Vec<&str> = line.split_whitespace().collect();
                let mut current_line = String::new();

                for word in words {
                    if current_line.len() + word.len() + 1 > chars_per_line {
                        if y_position < 30.0 {
                            let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                            current_layer = doc.get_page(page).get_layer(layer);
                            y_position = 270.0;
                        }

                        current_layer.use_text(
                            &current_line,
                            options.font_size,
                            Mm(margin_left),
                            Mm(y_position),
                            &font,
                        );
                        y_position -= line_height;
                        current_line = word.to_string();
                    } else {
                        if !current_line.is_empty() {
                            current_line.push(' ');
                        }
                        current_line.push_str(word);
                    }
                }

                // Flush remaining text
                if !current_line.is_empty() {
                    if y_position < 30.0 {
                        let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                        current_layer = doc.get_page(page).get_layer(layer);
                        y_position = 270.0;
                    }

                    current_layer.use_text(
                        &current_line,
                        options.font_size,
                        Mm(margin_left),
                        Mm(y_position),
                        &font,
                    );
                    y_position -= line_height;
                }
            }

            y_position -= line_height; // Extra spacing between documents
        }
    }

    doc.save(&mut std::io::BufWriter::new(std::fs::File::create(path)?))?;
    Ok(())
}

/// Add a page number footer to the current layer
fn add_page_number(layer: &PdfLayerReference, page_num: usize, font: &IndirectFontRef) {
    layer.use_text(
        &format!("- {} -", page_num),
        10.0,
        Mm(100.0),
        Mm(15.0),
        font,
    );
}

/// Strip basic markdown formatting from text for PDF rendering
fn strip_markdown(text: &str) -> String {
    let mut result = text.to_string();
    // Remove bold markers
    result = result.replace("**", "");
    // Remove italic markers (single *)
    result = result.replace('*', "");
    // Remove strikethrough
    result = result.replace("~~", "");
    // Remove heading markers
    while result.starts_with('#') {
        result = result.trim_start_matches('#').trim_start().to_string();
    }
    // Remove blockquote markers
    if result.starts_with("> ") {
        result = result[2..].to_string();
    }
    result
}

/// Count total words across all content sections
pub fn word_count(contents: &[CompileContent]) -> usize {
    contents.iter().map(|c| c.text.split_whitespace().count()).sum()
}

/// Count total characters across all content sections
pub fn char_count(contents: &[CompileContent]) -> usize {
    contents.iter().map(|c| c.text.len()).sum()
}

/// Estimate the number of pages for a given set of content
pub fn estimate_pages(contents: &[CompileContent], options: &CompileOptions) -> usize {
    let chars_per_line = (160.0 / (options.font_size * 0.2)) as usize;
    let lines_per_page = (240.0 / (options.font_size * 0.5)) as usize;
    let mut total_lines = 0usize;

    for content in contents {
        if content.is_folder {
            total_lines += 3; // heading + spacing
        } else {
            for line in content.text.lines() {
                if line.is_empty() {
                    total_lines += 1;
                } else {
                    total_lines += (line.len() / chars_per_line).max(1);
                }
            }
            total_lines += 1; // spacing between docs
        }
    }

    (total_lines / lines_per_page).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::compiler::OutputFormat;

    fn make_content(title: &str, text: &str, is_folder: bool, depth: usize) -> CompileContent {
        CompileContent {
            title: title.to_string(),
            text: text.to_string(),
            depth,
            is_folder,
        }
    }

    fn make_opts() -> CompileOptions {
        CompileOptions {
            format: OutputFormat::Pdf,
            title: "Test PDF".to_string(),
            author: "Author".to_string(),
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

    #[test]
    fn test_strip_markdown_bold() {
        assert_eq!(strip_markdown("**bold**"), "bold");
    }

    #[test]
    fn test_strip_markdown_italic() {
        assert_eq!(strip_markdown("*italic*"), "italic");
    }

    #[test]
    fn test_strip_markdown_strikethrough() {
        assert_eq!(strip_markdown("~~strike~~"), "strike");
    }

    #[test]
    fn test_strip_markdown_heading() {
        assert_eq!(strip_markdown("# Heading"), "Heading");
        assert_eq!(strip_markdown("## Sub"), "Sub");
    }

    #[test]
    fn test_strip_markdown_blockquote() {
        assert_eq!(strip_markdown("> Quote text"), "Quote text");
    }

    #[test]
    fn test_strip_markdown_plain() {
        assert_eq!(strip_markdown("Plain text"), "Plain text");
    }

    #[test]
    fn test_word_count() {
        let contents = vec![
            make_content("A", "one two three", false, 0),
            make_content("B", "four", false, 0),
        ];
        assert_eq!(word_count(&contents), 4);
    }

    #[test]
    fn test_char_count() {
        let contents = vec![make_content("A", "hello", false, 0)];
        assert_eq!(char_count(&contents), 5);
    }

    #[test]
    fn test_estimate_pages() {
        let contents = vec![
            make_content("Ch1", "", true, 0),
            make_content("A", &"Some text here.\n".repeat(100), false, 1),
        ];
        let pages = estimate_pages(&contents, &make_opts());
        assert!(pages >= 1);
    }
}
