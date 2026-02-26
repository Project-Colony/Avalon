#![allow(dead_code)] // Methods used by test code
use anyhow::Result;
use printpdf::*;
use std::path::Path;

use super::compiler::{CompileContent, CompileOptions};

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
                                // chars_per_line: approximate glyph count that fits in the usable page width.
                                // Clamped to [1, 500] before casting to avoid overflow on very small font sizes.
    let chars_per_line = if options.font_size > 0.0 {
        (page_width / (options.font_size * 0.2)).clamp(1.0, 500.0) as usize
    } else {
        80 // safe fallback when font_size is zero
    };

    // Title page
    if options.include_front_matter && !options.title.is_empty() {
        current_layer.use_text(
            &options.title,
            options.font_size * 2.0,
            Mm((105.0 - options.title.len() as f32 * options.font_size * 0.5).max(5.0)),
            Mm(200.0),
            &bold_font,
        );

        if !options.author.is_empty() {
            current_layer.use_text(
                &options.author,
                options.font_size,
                Mm((105.0 - options.author.len() as f32 * options.font_size * 0.25).max(5.0)),
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

                    current_layer.use_text(&current_line, options.font_size, Mm(margin_left), Mm(y_position), &font);
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
    layer.use_text(format!("- {} -", page_num), 10.0, Mm(100.0), Mm(15.0), font);
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
