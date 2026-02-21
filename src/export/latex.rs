use anyhow::Result;
use super::compiler::{CompileContent, CompileOptions, SeparatorType};

pub fn compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    let mut output = String::new();

    // LaTeX preamble
    output.push_str("\\documentclass[12pt]{article}\n");
    output.push_str("\\usepackage[utf8]{inputenc}\n");
    output.push_str("\\usepackage[T1]{fontenc}\n");
    output.push_str("\\usepackage{geometry}\n");
    output.push_str("\\geometry{a4paper, margin=1in}\n");
    output.push_str("\\usepackage{setspace}\n");
    output.push_str("\\usepackage{hyperref}\n");
    output.push_str("\\usepackage{fancyhdr}\n");
    output.push_str("\\usepackage{graphicx}\n");
    output.push_str("\\usepackage{longtable}\n");
    output.push_str("\\usepackage{enumitem}\n");
    output.push_str("\\doublespacing\n");
    output.push_str("\\pagestyle{fancy}\n");
    output.push_str("\\fancyhf{}\n");
    output.push_str("\\fancyfoot[C]{\\thepage}\n");

    // Header with author/title
    if !options.title.is_empty() {
        output.push_str(&format!(
            "\\fancyhead[R]{{\\textit{{{}}}}}\n",
            escape_latex(&options.title)
        ));
    }

    output.push('\n');

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
        output.push_str("\\thispagestyle{empty}\n");
        output.push_str("\\newpage\n\n");
    }

    let mut prev_was_text = false;

    for content in contents {
        if content.is_folder {
            let cmd = match content.depth {
                0 => "section",
                1 => "subsection",
                2 => "subsubsection",
                3 => "paragraph",
                _ => "subparagraph",
            };
            output.push_str(&format!(
                "\\{}{{{}}} \n\n",
                cmd,
                escape_latex(&content.title)
            ));
            prev_was_text = false;
        } else {
            // Separator between consecutive text documents
            if prev_was_text {
                output.push_str(&separator_latex(&options.separator));
            }

            // Convert basic markdown to LaTeX
            let text = markdown_to_latex(&content.text);
            output.push_str(&text);
            output.push_str("\n\n");
            prev_was_text = true;
        }
    }

    output.push_str("\\end{document}\n");

    Ok(output)
}

fn separator_latex(sep: &SeparatorType) -> String {
    match sep {
        SeparatorType::EmptyLine => "\\bigskip\n\n".to_string(),
        SeparatorType::PageBreak => "\\newpage\n\n".to_string(),
        SeparatorType::SectionBreak => {
            "\\begin{center}\n$\\ast$ \\quad $\\ast$ \\quad $\\ast$\n\\end{center}\n\n"
                .to_string()
        }
        SeparatorType::Custom(s) => {
            format!(
                "\\begin{{center}}\n{}\n\\end{{center}}\n\n",
                escape_latex(s)
            )
        }
        SeparatorType::None => String::new(),
    }
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
    let mut in_blockquote = false;

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            if in_blockquote {
                output.push_str("\\end{quote}\n");
                in_blockquote = false;
            }
            output.push_str("\n\\par\n");
            continue;
        }

        // Headings
        if let Some(heading) = parse_heading(trimmed) {
            if in_blockquote {
                output.push_str("\\end{quote}\n");
                in_blockquote = false;
            }
            output.push_str(&heading);
            output.push('\n');
            continue;
        }

        // Blockquote
        if trimmed.starts_with("> ") {
            if !in_blockquote {
                output.push_str("\\begin{quote}\n");
                in_blockquote = true;
            }
            let content = convert_inline_formatting(&trimmed[2..]);
            output.push_str(&content);
            output.push('\n');
            continue;
        } else if in_blockquote {
            output.push_str("\\end{quote}\n");
            in_blockquote = false;
        }

        // Horizontal rule
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            output.push_str("\\begin{center}\\rule{0.5\\linewidth}{0.4pt}\\end{center}\n");
            continue;
        }

        // Unordered list item
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let content = convert_inline_formatting(&trimmed[2..]);
            output.push_str(&format!("\\textbullet\\ {}\n", content));
            continue;
        }

        // Regular paragraph
        let converted = convert_inline_formatting(trimmed);
        output.push_str(&converted);
        output.push('\n');
    }

    if in_blockquote {
        output.push_str("\\end{quote}\n");
    }

    output
}

/// Parse a markdown heading line into LaTeX
fn parse_heading(line: &str) -> Option<String> {
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

    let cmd = match level {
        1 => "section",
        2 => "subsection",
        3 => "subsubsection",
        4 => "paragraph",
        _ => "subparagraph",
    };
    Some(format!("\\{}*{{{}}}", cmd, escape_latex(text)))
}

/// Convert inline markdown formatting to LaTeX
fn convert_inline_formatting(text: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Bold: **text**
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end) = find_closing(&chars, i + 2, '*', '*') {
                let inner: String = chars[i + 2..end].iter().collect();
                result.push_str(&format!("\\textbf{{{}}}", escape_latex(&inner)));
                i = end + 2;
                continue;
            }
        }

        // Italic: *text*
        if chars[i] == '*' {
            if let Some(end) = find_closing_single(&chars, i + 1, '*') {
                let inner: String = chars[i + 1..end].iter().collect();
                result.push_str(&format!("\\textit{{{}}}", escape_latex(&inner)));
                i = end + 1;
                continue;
            }
        }

        // Strikethrough: ~~text~~
        if i + 1 < len && chars[i] == '~' && chars[i + 1] == '~' {
            if let Some(end) = find_closing(&chars, i + 2, '~', '~') {
                let inner: String = chars[i + 2..end].iter().collect();
                result.push_str(&format!("\\sout{{{}}}", escape_latex(&inner)));
                i = end + 2;
                continue;
            }
        }

        // Inline code: `text`
        if chars[i] == '`' {
            if let Some(end) = find_closing_single(&chars, i + 1, '`') {
                let inner: String = chars[i + 1..end].iter().collect();
                result.push_str(&format!("\\texttt{{{}}}", escape_latex(&inner)));
                i = end + 1;
                continue;
            }
        }

        // Regular character — escape LaTeX specials individually
        let c = chars[i];
        match c {
            '&' => result.push_str("\\&"),
            '%' => result.push_str("\\%"),
            '$' => result.push_str("\\$"),
            '#' => result.push_str("\\#"),
            '_' => result.push_str("\\_"),
            '{' => result.push_str("\\{"),
            '}' => result.push_str("\\}"),
            '~' => result.push_str("\\textasciitilde{}"),
            '^' => result.push_str("\\textasciicircum{}"),
            _ => result.push(c),
        }
        i += 1;
    }

    result
}

/// Find closing double-char delimiter (e.g. **)
fn find_closing(chars: &[char], start: usize, c1: char, c2: char) -> Option<usize> {
    let mut i = start;
    while i + 1 < chars.len() {
        if chars[i] == c1 && chars[i + 1] == c2 {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Find closing single-char delimiter (e.g. *)
fn find_closing_single(chars: &[char], start: usize, c: char) -> Option<usize> {
    for i in start..chars.len() {
        if chars[i] == c {
            return Some(i);
        }
    }
    None
}

/// Count total words across all content sections
pub fn word_count(contents: &[CompileContent]) -> usize {
    contents.iter().map(|c| c.text.split_whitespace().count()).sum()
}

/// Estimate page count (LaTeX with double spacing, A4, 1-inch margins ~ 250 words/page)
pub fn estimate_pages(contents: &[CompileContent]) -> usize {
    let total_words = word_count(contents);
    (total_words / 250).max(1)
}

/// Estimate the output size in bytes for a LaTeX compilation
pub fn estimate_output_size(contents: &[CompileContent], options: &CompileOptions) -> usize {
    // LaTeX preamble ~600 bytes
    let base = 600;
    let front_matter = if options.include_front_matter { 100 } else { 0 };
    // LaTeX escaping and commands roughly double the content size
    let content_size: usize = contents.iter()
        .map(|c| c.text.len() * 2 + c.title.len() + 40)
        .sum();
    base + front_matter + content_size
}

/// List all LaTeX packages used in the preamble
pub fn required_packages() -> Vec<&'static str> {
    vec![
        "inputenc", "fontenc", "geometry", "setspace",
        "hyperref", "fancyhdr", "graphicx", "longtable", "enumitem",
    ]
}
