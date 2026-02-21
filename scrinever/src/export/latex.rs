use anyhow::Result;
use super::compiler::{CompileContent, CompileOptions};

pub fn compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    let mut output = String::new();

    // LaTeX preamble
    output.push_str("\\documentclass[12pt]{article}\n");
    output.push_str("\\usepackage[utf8]{inputenc}\n");
    output.push_str("\\usepackage[T1]{fontenc}\n");
    output.push_str("\\usepackage{geometry}\n");
    output.push_str("\\geometry{a4paper, margin=1in}\n");
    output.push_str("\\usepackage{setspace}\n");
    output.push_str("\\doublespacing\n\n");

    if !options.title.is_empty() {
        output.push_str(&format!("\\title{{{}}}\n", escape_latex(&options.title)));
    }
    if !options.author.is_empty() {
        output.push_str(&format!("\\author{{{}}}\n", escape_latex(&options.author)));
    }
    output.push_str("\\date{}\n\n");
    output.push_str("\\begin{document}\n\n");

    if options.include_front_matter && !options.title.is_empty() {
        output.push_str("\\maketitle\n");
        output.push_str("\\newpage\n\n");
    }

    for content in contents {
        if content.is_folder {
            let cmd = match content.depth {
                0 => "section",
                1 => "subsection",
                _ => "subsubsection",
            };
            output.push_str(&format!("\\{}{{{}}} \n\n", cmd, escape_latex(&content.title)));
        } else {
            // Convert basic markdown to LaTeX
            let text = markdown_to_latex(&content.text);
            output.push_str(&text);
            output.push_str("\n\n");
        }
    }

    output.push_str("\\end{document}\n");

    Ok(output)
}

/// Escape special LaTeX characters
fn escape_latex(text: &str) -> String {
    text.replace('\\', "\\textbackslash{}")
        .replace('&', "\\&")
        .replace('%', "\\%")
        .replace('$', "\\$")
        .replace('#', "\\#")
        .replace('_', "\\_")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('~', "\\textasciitilde{}")
        .replace('^', "\\textasciicircum{}")
}

/// Basic Markdown to LaTeX conversion
fn markdown_to_latex(text: &str) -> String {
    let mut output = String::new();

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            output.push_str("\n\\par\n");
            continue;
        }

        // Convert bold **text** -> \textbf{text}
        let line = convert_inline_formatting(trimmed);
        output.push_str(&escape_latex(&line));
        output.push('\n');
    }

    output
}

fn convert_inline_formatting(text: &str) -> String {
    // This is a simplified conversion — a full implementation would use a proper parser
    text.to_string()
}
