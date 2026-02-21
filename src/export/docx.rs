use std::path::Path;
use anyhow::Result;
use docx_rs::*;

use super::compiler::{CompileContent, CompileOptions};

pub fn save_docx(contents: &[CompileContent], options: &CompileOptions, path: &Path) -> Result<()> {
    let mut docx = Docx::new();

    // Title page
    if options.include_front_matter && !options.title.is_empty() {
        let title_para = Paragraph::new()
            .add_run(
                Run::new()
                    .add_text(&options.title)
                    .size(48)
                    .bold(),
            )
            .align(AlignmentType::Center);
        docx = docx.add_paragraph(title_para);

        if !options.author.is_empty() {
            let author_para = Paragraph::new()
                .add_run(
                    Run::new()
                        .add_text(&options.author)
                        .size(24)
                        .italic(),
                )
                .align(AlignmentType::Center);
            docx = docx.add_paragraph(author_para);
        }

        // Page break after title
        let break_para = Paragraph::new()
            .add_run(Run::new().add_break(BreakType::Page));
        docx = docx.add_paragraph(break_para);
    }

    for (i, content) in contents.iter().enumerate() {
        if content.is_folder {
            // Heading style based on depth
            let heading_size = match content.depth {
                0 => 36_usize,
                1 => 28,
                _ => 24,
            };

            let heading_para = Paragraph::new()
                .add_run(
                    Run::new()
                        .add_text(&content.title)
                        .size(heading_size)
                        .bold(),
                );
            docx = docx.add_paragraph(heading_para);

            // Extra space after heading
            let space = Paragraph::new();
            docx = docx.add_paragraph(space);
        } else {
            // Document text — split into paragraphs
            for line in content.text.split("\n\n") {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    let empty = Paragraph::new();
                    docx = docx.add_paragraph(empty);
                    continue;
                }

                let para = Paragraph::new()
                    .add_run(
                        Run::new()
                            .add_text(trimmed)
                            .size((options.font_size * 2.0) as usize),
                    );
                docx = docx.add_paragraph(para);
            }

            // Separator between documents
            if i < contents.len() - 1 && options.page_break_between_folders {
                // empty line separator
                let sep = Paragraph::new();
                docx = docx.add_paragraph(sep);
            }
        }
    }

    let file = std::fs::File::create(path)?;
    docx.build().pack(file)?;
    Ok(())
}
