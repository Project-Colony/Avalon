use anyhow::Result;
use printpdf::{BuiltinFont, Mm, Op, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt, TextItem};
use std::path::Path;

use super::compiler::{CompileContent, CompileOptions};

/// A4 page size.
const PAGE_WIDTH: Mm = Mm(210.0);
const PAGE_HEIGHT: Mm = Mm(297.0);
/// Baseline of the first line on a page, in mm from the bottom edge.
const TOP_Y: f32 = 270.0;
/// A line whose baseline would fall below this goes to the next page.
const BOTTOM_Y: f32 = 30.0;

pub fn save_pdf(contents: &[CompileContent], options: &CompileOptions, path: &Path) -> Result<()> {
    let mut pages = Pages::default();
    let font = BuiltinFont::TimesRoman;
    let bold_font = BuiltinFont::TimesBold;

    let mut y_position = TOP_Y;
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
        pages.text(
            &options.title,
            options.font_size * 2.0,
            (105.0 - options.title.len() as f32 * options.font_size * 0.5).max(5.0),
            200.0,
            bold_font,
        );

        if !options.author.is_empty() {
            pages.text(
                &options.author,
                options.font_size,
                (105.0 - options.author.len() as f32 * options.font_size * 0.25).max(5.0),
                180.0,
                font,
            );
        }

        // New page for content
        pages.break_page();
        y_position = TOP_Y;
    }

    for content in contents {
        if content.is_folder {
            // Folder heading
            let heading_size = match content.depth {
                0 => options.font_size * 1.5,
                1 => options.font_size * 1.3,
                _ => options.font_size * 1.1,
            };

            if y_position < BOTTOM_Y {
                pages.break_page();
                y_position = TOP_Y;
            }

            pages.text(&content.title, heading_size, margin_left, y_position, bold_font);
            y_position -= line_height * 2.0;
        } else {
            // Document text: wrap lines
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
                        if y_position < BOTTOM_Y {
                            pages.break_page();
                            y_position = TOP_Y;
                        }

                        pages.text(&current_line, options.font_size, margin_left, y_position, font);
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
                    if y_position < BOTTOM_Y {
                        pages.break_page();
                        y_position = TOP_Y;
                    }

                    pages.text(&current_line, options.font_size, margin_left, y_position, font);
                    y_position -= line_height;
                }
            }

            y_position -= line_height; // Extra spacing between documents
        }
    }

    let mut doc = PdfDocument::new(&options.title);
    doc.with_pages(pages.finish());
    let mut warnings = Vec::new();
    let bytes = doc.save(&PdfSaveOptions::default(), &mut warnings);
    for warning in &warnings {
        log::warn!("PDF export: {}", warning.msg);
    }
    std::fs::write(path, bytes)?;
    Ok(())
}

/// Finished pages plus the drawing operations of the page being filled.
#[derive(Default)]
struct Pages {
    done: Vec<PdfPage>,
    ops: Vec<Op>,
}

impl Pages {
    /// Close the current page and start a blank one.
    fn break_page(&mut self) {
        let ops = std::mem::take(&mut self.ops);
        self.done.push(PdfPage::new(PAGE_WIDTH, PAGE_HEIGHT, ops));
    }

    /// Write one line with its baseline at (`x`, `y`) mm from the bottom left corner.
    fn text(&mut self, text: &str, size: f32, x: f32, y: f32, font: BuiltinFont) {
        // One text object per line: inside a text object the cursor moves
        // relative to the previous line, not to the page.
        self.ops.extend([
            Op::StartTextSection,
            Op::SetFont {
                font: PdfFontHandle::Builtin(font),
                size: Pt(size),
            },
            Op::SetTextCursor {
                pos: Point::new(Mm(x), Mm(y)),
            },
            Op::ShowText {
                items: vec![TextItem::Text(text.to_string())],
            },
            Op::EndTextSection,
        ]);
    }

    /// Close the last page and return them all. A document always has at least one page.
    fn finish(mut self) -> Vec<PdfPage> {
        self.break_page();
        self.done
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use printpdf::PdfParseOptions;

    fn render(contents: &[CompileContent], options: &CompileOptions) -> Vec<u8> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.pdf");
        save_pdf(contents, options, &path).unwrap();
        std::fs::read(path).unwrap()
    }

    fn parse(bytes: &[u8]) -> PdfDocument {
        PdfDocument::parse(bytes, &PdfParseOptions::default(), &mut Vec::new()).unwrap()
    }

    /// Every string shown on `page`, in order.
    fn page_text(page: &PdfPage) -> Vec<String> {
        page.ops
            .iter()
            .filter_map(|op| match op {
                Op::ShowText { items } => Some(items),
                _ => None,
            })
            .flatten()
            .filter_map(|item| match item {
                TextItem::Text(text) => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn empty_manuscript_is_one_blank_page() {
        let options = CompileOptions {
            include_front_matter: false,
            ..CompileOptions::default()
        };
        let bytes = render(&[], &options);
        assert!(bytes.starts_with(b"%PDF"));
        assert_eq!(parse(&bytes).pages.len(), 1);
    }

    #[test]
    fn long_manuscript_spans_pages_after_the_title_page() {
        let options = CompileOptions {
            title: "Long Novel".to_string(),
            author: "A. Writer".to_string(),
            ..CompileOptions::default()
        };
        // At the default 12 pt, lines are 6 mm apart and baselines run from
        // 270 mm down to 30 mm: 41 lines per page. 100 lines fill 41 + 41 + 18.
        let text = vec!["Ünïcode café line"; 100].join("\n");
        let contents = [CompileContent {
            title: "Chapter 1".to_string(),
            text,
            depth: 1,
            is_folder: false,
        }];
        let bytes = render(&contents, &options);
        assert!(bytes.starts_with(b"%PDF"));
        let doc = parse(&bytes);
        assert_eq!(doc.pages.len(), 1 + 3);
        assert_eq!(page_text(&doc.pages[0]), ["Long Novel", "A. Writer"]);
        assert_eq!(page_text(&doc.pages[1]).len(), 41);
        assert_eq!(page_text(&doc.pages[3]).len(), 18);
        assert_eq!(page_text(&doc.pages[3])[0], "Ünïcode café line");
    }
}
