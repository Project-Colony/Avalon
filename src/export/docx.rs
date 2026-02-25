use std::path::Path;
use anyhow::Result;
use docx_rs::*;

use super::compiler::{CompileContent, CompileOptions, SeparatorType};

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

        // Date line
        let date_str = chrono::Local::now().format("%B %d, %Y").to_string();
        let date_para = Paragraph::new()
            .add_run(
                Run::new()
                    .add_text(&date_str)
                    .size(20)
                    .italic(),
            )
            .align(AlignmentType::Center);
        docx = docx.add_paragraph(date_para);

        // Page break after title
        let break_para = Paragraph::new()
            .add_run(Run::new().add_break(BreakType::Page));
        docx = docx.add_paragraph(break_para);
    }

    let mut prev_was_text = false;

    for (i, content) in contents.iter().enumerate() {
        if content.is_folder {
            // Page break before top-level folders (except the first)
            if content.depth == 0 && i > 0 && options.page_break_between_folders {
                let break_para = Paragraph::new()
                    .add_run(Run::new().add_break(BreakType::Page));
                docx = docx.add_paragraph(break_para);
            }

            // Heading style based on depth
            let heading_size = match content.depth {
                0 => 36_usize,
                1 => 28,
                2 => 24,
                _ => 22,
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
            prev_was_text = false;
        } else {
            // Separator between consecutive documents
            if prev_was_text {
                docx = add_separator(docx, &options.separator);
            }

            // Document text — split into paragraphs and handle inline formatting
            for paragraph_text in content.text.split("\n\n") {
                let trimmed = paragraph_text.trim();
                if trimmed.is_empty() {
                    let empty = Paragraph::new();
                    docx = docx.add_paragraph(empty);
                    continue;
                }

                // Check for heading lines
                if let Some((level, heading_text)) = parse_md_heading(trimmed) {
                    let heading_size = match level {
                        1 => 32_usize,
                        2 => 28,
                        3 => 24,
                        _ => 22,
                    };
                    let para = Paragraph::new()
                        .add_run(
                            Run::new()
                                .add_text(heading_text)
                                .size(heading_size)
                                .bold(),
                        );
                    docx = docx.add_paragraph(para);
                    continue;
                }

                // Check for blockquote
                if trimmed.starts_with("> ") {
                    let quote_text = trimmed.strip_prefix("> ").unwrap_or(trimmed);
                    let para = Paragraph::new()
                        .add_run(
                            Run::new()
                                .add_text(quote_text)
                                .size((options.font_size * 2.0) as usize)
                                .italic(),
                        );
                    docx = docx.add_paragraph(para);
                    continue;
                }

                // Regular paragraph with inline formatting
                let runs = parse_inline_runs(trimmed, options.font_size);
                let mut para = Paragraph::new();
                for run in runs {
                    para = para.add_run(run);
                }
                docx = docx.add_paragraph(para);
            }

            prev_was_text = true;
        }
    }

    let file = std::fs::File::create(path)?;
    docx.build().pack(file)?;
    Ok(())
}

fn add_separator(docx: Docx, sep: &SeparatorType) -> Docx {
    match sep {
        SeparatorType::EmptyLine => {
            docx.add_paragraph(Paragraph::new())
        }
        SeparatorType::PageBreak => {
            docx.add_paragraph(
                Paragraph::new().add_run(Run::new().add_break(BreakType::Page)),
            )
        }
        SeparatorType::SectionBreak => {
            docx.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text("* * *").size(20))
                    .align(AlignmentType::Center),
            )
        }
        SeparatorType::Custom(s) => {
            docx.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(s).size(20))
                    .align(AlignmentType::Center),
            )
        }
        SeparatorType::None => docx,
    }
}

/// Parse a markdown heading line
fn parse_md_heading(line: &str) -> Option<(usize, &str)> {
    if !line.starts_with('#') {
        return None;
    }
    let level = line.chars().take_while(|c| *c == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let text = line[level..].trim();
    if text.is_empty() {
        return None;
    }
    Some((level, text))
}

/// Parse inline markdown formatting into docx Runs
fn parse_inline_runs(text: &str, font_size: f32) -> Vec<Run> {
    let size = (font_size * 2.0) as usize;
    let mut runs = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Bold: **text**
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end) = find_closing_double(&chars, i + 2, '*') {
                // Flush current text
                if !current.is_empty() {
                    runs.push(Run::new().add_text(&current).size(size));
                    current.clear();
                }
                let inner: String = chars[i + 2..end].iter().collect();
                runs.push(Run::new().add_text(&inner).size(size).bold());
                i = end + 2;
                continue;
            }
        }

        // Italic: *text*
        if chars[i] == '*' {
            if let Some(end) = find_closing_single(&chars, i + 1, '*') {
                if !current.is_empty() {
                    runs.push(Run::new().add_text(&current).size(size));
                    current.clear();
                }
                let inner: String = chars[i + 1..end].iter().collect();
                runs.push(Run::new().add_text(&inner).size(size).italic());
                i = end + 1;
                continue;
            }
        }

        current.push(chars[i]);
        i += 1;
    }

    if !current.is_empty() {
        runs.push(Run::new().add_text(&current).size(size));
    }

    if runs.is_empty() {
        runs.push(Run::new().add_text(text).size(size));
    }

    runs
}

fn find_closing_double(chars: &[char], start: usize, c: char) -> Option<usize> {
    let mut i = start;
    while i + 1 < chars.len() {
        if chars[i] == c && chars[i + 1] == c {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn find_closing_single(chars: &[char], start: usize, c: char) -> Option<usize> {
    (start..chars.len()).find(|&i| chars[i] == c)
}
