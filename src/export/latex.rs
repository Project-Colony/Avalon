use std::fmt::Write as _;
use anyhow::Result;
use super::compiler::{self, CompileContent, CompileOptions, SeparatorType};

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
        let _ = writeln!(output,
            "\\fancyhead[R]{{\\textit{{{}}}}}",
            escape_latex(&options.title)
        );
    }

    output.push('\n');

    if !options.title.is_empty() {
        let _ = writeln!(output,"\\title{{{}}}", escape_latex(&options.title));
    }
    if !options.author.is_empty() {
        let _ = writeln!(output,"\\author{{{}}}", escape_latex(&options.author));
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
            let _ = write!(output,
                "\\{}{{{}}} \n\n",
                cmd,
                escape_latex(&content.title)
            );
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
        if let Some(quoted) = trimmed.strip_prefix("> ") {
            if !in_blockquote {
                output.push_str("\\begin{quote}\n");
                in_blockquote = true;
            }
            let content = convert_inline_formatting(quoted);
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
            let _ = writeln!(output,"\\textbullet\\ {}", content);
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
                let _ = write!(result,"\\textbf{{{}}}", escape_latex(&inner));
                i = end + 2;
                continue;
            }
        }

        // Italic: *text*
        if chars[i] == '*' {
            if let Some(end) = find_closing_single(&chars, i + 1, '*') {
                let inner: String = chars[i + 1..end].iter().collect();
                let _ = write!(result,"\\textit{{{}}}", escape_latex(&inner));
                i = end + 1;
                continue;
            }
        }

        // Strikethrough: ~~text~~
        if i + 1 < len && chars[i] == '~' && chars[i + 1] == '~' {
            if let Some(end) = find_closing(&chars, i + 2, '~', '~') {
                let inner: String = chars[i + 2..end].iter().collect();
                let _ = write!(result,"\\sout{{{}}}", escape_latex(&inner));
                i = end + 2;
                continue;
            }
        }

        // Inline code: `text`
        if chars[i] == '`' {
            if let Some(end) = find_closing_single(&chars, i + 1, '`') {
                let inner: String = chars[i + 1..end].iter().collect();
                let _ = write!(result,"\\texttt{{{}}}", escape_latex(&inner));
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
    (start..chars.len()).find(|&i| chars[i] == c)
}

/// Count total words across all content sections
pub fn word_count(contents: &[CompileContent]) -> usize {
    compiler::total_word_count(contents)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::compiler::OutputFormat;

    fn make_opts() -> CompileOptions {
        CompileOptions {
            format: OutputFormat::Latex,
            title: "Test Book".to_string(),
            author: "Author Name".to_string(),
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

    fn make_content(title: &str, text: &str, is_folder: bool, depth: usize) -> CompileContent {
        CompileContent {
            title: title.to_string(),
            text: text.to_string(),
            depth,
            is_folder,
        }
    }

    #[test]
    fn test_compile_basic() {
        let contents = vec![make_content("Scene 1", "Hello world.", false, 1)];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("\\documentclass"));
        assert!(result.contains("\\begin{document}"));
        assert!(result.contains("\\end{document}"));
        assert!(result.contains("Hello world."));
    }

    #[test]
    fn test_compile_with_front_matter() {
        let mut opts = make_opts();
        opts.include_front_matter = true;
        let contents = vec![make_content("Scene", "Text.", false, 1)];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("\\maketitle"));
        assert!(result.contains("\\title{Test Book}"));
        assert!(result.contains("\\author{Author Name}"));
    }

    #[test]
    fn test_compile_folder_depths() {
        let contents = vec![
            make_content("L0", "", true, 0),
            make_content("L1", "", true, 1),
            make_content("L2", "", true, 2),
            make_content("L3", "", true, 3),
        ];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("\\section{L0}"));
        assert!(result.contains("\\subsection{L1}"));
        assert!(result.contains("\\subsubsection{L2}"));
        assert!(result.contains("\\paragraph{L3}"));
    }

    #[test]
    fn test_escape_latex() {
        assert!(escape_latex("$100").contains("\\$"));
        assert!(escape_latex("50%").contains("\\%"));
        assert!(escape_latex("a&b").contains("\\&"));
        assert!(escape_latex("c#d").contains("\\#"));
        assert!(escape_latex("a_b").contains("\\_"));
    }

    #[test]
    fn test_inline_bold() {
        let result = convert_inline_formatting("**bold**");
        assert!(result.contains("\\textbf{bold}"));
    }

    #[test]
    fn test_inline_italic() {
        let result = convert_inline_formatting("*italic*");
        assert!(result.contains("\\textit{italic}"));
    }

    #[test]
    fn test_inline_code() {
        let result = convert_inline_formatting("`code`");
        assert!(result.contains("\\texttt{code}"));
    }

    #[test]
    fn test_parse_heading() {
        assert!(parse_heading("# Title").unwrap().contains("\\section*{Title}"));
        assert!(parse_heading("## Sub").unwrap().contains("\\subsection*{Sub}"));
        assert!(parse_heading("Not heading").is_none());
    }

    #[test]
    fn test_separator_types() {
        assert!(separator_latex(&SeparatorType::EmptyLine).contains("\\bigskip"));
        assert!(separator_latex(&SeparatorType::PageBreak).contains("\\newpage"));
        assert!(separator_latex(&SeparatorType::SectionBreak).contains("\\ast"));
        assert!(separator_latex(&SeparatorType::None).is_empty());
    }

    #[test]
    fn test_word_count_and_pages() {
        let contents = vec![make_content("A", &"word ".repeat(500), false, 0)];
        assert_eq!(word_count(&contents), 500);
        assert_eq!(estimate_pages(&contents), 2);
    }

    #[test]
    fn test_required_packages() {
        let pkgs = required_packages();
        assert!(pkgs.contains(&"inputenc"));
        assert!(pkgs.contains(&"hyperref"));
        assert!(pkgs.len() >= 9);
    }

    #[test]
    fn test_compile_empty() {
        let contents: Vec<CompileContent> = vec![];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("\\documentclass"));
        assert!(result.contains("\\end{document}"));
    }

    #[test]
    fn test_compile_no_front_matter() {
        let opts = make_opts();
        let contents = vec![make_content("Scene", "Text.", false, 0)];
        let result = compile(&contents, &opts).unwrap();
        assert!(!result.contains("\\maketitle"));
    }

    #[test]
    fn test_compile_deep_folder() {
        let contents = vec![make_content("Deep", "", true, 5)];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("\\subparagraph{Deep}"));
    }

    #[test]
    fn test_markdown_to_latex_heading() {
        let result = markdown_to_latex("# Title\n\nSome text.");
        assert!(result.contains("\\section*{Title}"));
    }

    #[test]
    fn test_markdown_to_latex_blockquote() {
        let result = markdown_to_latex("> A quote here");
        assert!(result.contains("\\begin{quote}"));
        assert!(result.contains("\\end{quote}"));
    }

    #[test]
    fn test_markdown_to_latex_hr() {
        let result = markdown_to_latex("---");
        assert!(result.contains("\\rule"));
    }

    #[test]
    fn test_markdown_to_latex_list_item() {
        let result = markdown_to_latex("- Item one");
        assert!(result.contains("\\textbullet"));
    }

    #[test]
    fn test_inline_strikethrough() {
        let result = convert_inline_formatting("~~struck~~");
        assert!(result.contains("\\sout{struck}"));
    }

    #[test]
    fn test_separator_custom() {
        let result = separator_latex(&SeparatorType::Custom("***".to_string()));
        assert!(result.contains("\\begin{center}"));
    }

    #[test]
    fn test_compile_multiple_docs_with_separator() {
        let mut opts = make_opts();
        opts.separator = SeparatorType::PageBreak;
        let contents = vec![
            make_content("A", "First.", false, 0),
            make_content("B", "Second.", false, 0),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("\\newpage"));
    }

    #[test]
    fn test_escape_latex_tilde_caret() {
        assert!(escape_latex("a~b").contains("\\textasciitilde{}"));
        assert!(escape_latex("a^b").contains("\\textasciicircum{}"));
    }

    #[test]
    fn test_escape_latex_braces() {
        assert!(escape_latex("{test}").contains("\\{"));
        assert!(escape_latex("{test}").contains("\\}"));
    }

    #[test]
    fn test_parse_heading_empty() {
        assert!(parse_heading("#").is_none());
        assert!(parse_heading("# ").is_none());
    }

    #[test]
    fn test_estimate_output_size() {
        let contents = vec![make_content("A", "text", false, 0)];
        let size = estimate_output_size(&contents, &make_opts());
        assert!(size > 600);
    }

    #[test]
    fn test_estimate_output_size_front_matter() {
        let contents = vec![make_content("A", "text", false, 0)];
        let mut opts = make_opts();
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
    fn test_estimate_pages_empty() {
        let contents: Vec<CompileContent> = vec![];
        assert_eq!(estimate_pages(&contents), 1);
    }

    #[test]
    fn test_compile_title_in_header() {
        let opts = make_opts();
        let contents = vec![make_content("Scene", "text.", false, 0)];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("\\fancyhead[R]"));
        assert!(result.contains("Test Book"));
    }

    #[test]
    fn test_compile_no_title() {
        let mut opts = make_opts();
        opts.title = String::new();
        let contents = vec![make_content("Scene", "text.", false, 0)];
        let result = compile(&contents, &opts).unwrap();
        assert!(!result.contains("\\title{}"));
    }

    #[test]
    fn test_compile_multiple_text_docs_with_separator() {
        let mut opts = make_opts();
        opts.separator = SeparatorType::EmptyLine;
        let contents = vec![
            make_content("A", "First text.", false, 0),
            make_content("B", "Second text.", false, 0),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("\\bigskip"));
        assert!(result.contains("First text."));
        assert!(result.contains("Second text."));
    }

    #[test]
    fn test_compile_section_break_separator() {
        let mut opts = make_opts();
        opts.separator = SeparatorType::SectionBreak;
        let contents = vec![
            make_content("A", "First.", false, 0),
            make_content("B", "Second.", false, 0),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("\\ast"));
    }

    #[test]
    fn test_compile_no_separator() {
        let mut opts = make_opts();
        opts.separator = SeparatorType::None;
        let contents = vec![
            make_content("A", "First.", false, 0),
            make_content("B", "Second.", false, 0),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("First."));
        assert!(result.contains("Second."));
        assert!(!result.contains("\\bigskip"));
        assert!(!result.contains("\\newpage\n\nSecond"));
    }

    #[test]
    fn test_compile_custom_separator() {
        let mut opts = make_opts();
        opts.separator = SeparatorType::Custom("***".to_string());
        let contents = vec![
            make_content("A", "First.", false, 0),
            make_content("B", "Second.", false, 0),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("\\begin{center}"));
    }

    #[test]
    fn test_escape_latex_backslash() {
        let result = escape_latex("a\\b");
        // Note: escape_latex replaces \ with \textbackslash{}, then { and } get
        // escaped by subsequent replacement passes, yielding \textbackslash\{\}
        assert!(result.contains("\\textbackslash\\{\\}"));
    }

    #[test]
    fn test_escape_latex_all_specials() {
        let input = "100$ 50% a&b c#d e_f {g} ~h ^i";
        let result = escape_latex(input);
        assert!(result.contains("\\$"));
        assert!(result.contains("\\%"));
        assert!(result.contains("\\&"));
        assert!(result.contains("\\#"));
        assert!(result.contains("\\_"));
        assert!(result.contains("\\{"));
        assert!(result.contains("\\}"));
        assert!(result.contains("\\textasciitilde{}"));
        assert!(result.contains("\\textasciicircum{}"));
    }

    #[test]
    fn test_parse_heading_levels() {
        let h1 = parse_heading("# Title").unwrap();
        assert!(h1.contains("\\section*{Title}"));

        let h2 = parse_heading("## Sub").unwrap();
        assert!(h2.contains("\\subsection*{Sub}"));

        let h3 = parse_heading("### SubSub").unwrap();
        assert!(h3.contains("\\subsubsection*{SubSub}"));

        let h4 = parse_heading("#### Para").unwrap();
        assert!(h4.contains("\\paragraph*{Para}"));

        let h5 = parse_heading("##### Deep").unwrap();
        assert!(h5.contains("\\subparagraph*{Deep}"));
    }

    #[test]
    fn test_parse_heading_too_deep() {
        // 7 # is invalid (only 1-6 supported)
        let result = parse_heading("####### Way too deep");
        assert!(result.is_none());
    }

    #[test]
    fn test_markdown_to_latex_list_items() {
        let result = markdown_to_latex("- Item one\n* Item two");
        assert!(result.contains("\\textbullet"));
    }

    #[test]
    fn test_markdown_to_latex_horizontal_rules() {
        for rule in &["---", "***", "___"] {
            let result = markdown_to_latex(rule);
            assert!(result.contains("\\rule"), "Failed for rule: {}", rule);
        }
    }

    #[test]
    fn test_markdown_to_latex_consecutive_blockquotes() {
        let result = markdown_to_latex("> Line one\n> Line two");
        // Should open quote once and include both lines
        let begin_count = result.matches("\\begin{quote}").count();
        let end_count = result.matches("\\end{quote}").count();
        assert_eq!(begin_count, 1);
        assert_eq!(end_count, 1);
    }

    #[test]
    fn test_markdown_to_latex_empty_lines() {
        let result = markdown_to_latex("Paragraph one.\n\nParagraph two.");
        assert!(result.contains("\\par"));
    }

    #[test]
    fn test_inline_formatting_nested() {
        let result = convert_inline_formatting("normal **bold `code`** end");
        assert!(result.contains("\\textbf{"));
    }

    #[test]
    fn test_inline_formatting_no_close() {
        // Unclosed formatting should just pass through as literal
        let result = convert_inline_formatting("unclosed **bold");
        assert!(result.contains("unclosed"));
    }

    #[test]
    fn test_compile_structure() {
        let opts = make_opts();
        let result = compile(&[], &opts).unwrap();
        assert!(result.contains("\\documentclass"));
        assert!(result.contains("\\usepackage"));
        assert!(result.contains("\\begin{document}"));
        assert!(result.contains("\\end{document}"));
        assert!(result.contains("\\doublespacing"));
        assert!(result.contains("\\pagestyle{fancy}"));
    }

    #[test]
    fn test_compile_folder_then_text() {
        let opts = make_opts();
        let contents = vec![
            make_content("Chapter One", "", true, 0),
            make_content("Scene 1", "First scene text.", false, 1),
            make_content("Scene 2", "Second scene text.", false, 1),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("\\section{Chapter One}"));
        assert!(result.contains("First scene text."));
        assert!(result.contains("Second scene text."));
    }

    #[test]
    fn test_required_packages_list() {
        let pkgs = required_packages();
        assert!(pkgs.contains(&"geometry"));
        assert!(pkgs.contains(&"setspace"));
        assert!(pkgs.contains(&"fancyhdr"));
        assert!(pkgs.contains(&"graphicx"));
        assert!(pkgs.contains(&"longtable"));
        assert!(pkgs.contains(&"enumitem"));
    }

    #[test]
    fn test_word_count_empty_text() {
        let contents = vec![make_content("A", "", false, 0)];
        assert_eq!(word_count(&contents), 0);
    }

    #[test]
    fn test_estimate_pages_short_text() {
        let contents = vec![make_content("A", "one two three", false, 0)];
        assert_eq!(estimate_pages(&contents), 1); // min 1
    }

    #[test]
    fn test_estimate_output_size_grows() {
        let small = vec![make_content("A", "short", false, 0)];
        let big = vec![make_content("A", &"word ".repeat(1000), false, 0)];
        let opts = make_opts();
        assert!(estimate_output_size(&big, &opts) > estimate_output_size(&small, &opts));
    }

    #[test]
    fn test_compile_no_author() {
        let mut opts = make_opts();
        opts.author = String::new();
        opts.include_front_matter = true;
        let contents = vec![make_content("Scene", "text.", false, 0)];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("\\maketitle"));
        assert!(!result.contains("\\author{"));
    }

    #[test]
    fn test_find_closing_not_found() {
        let chars: Vec<char> = "no close here".chars().collect();
        assert!(find_closing(&chars, 0, '*', '*').is_none());
    }

    #[test]
    fn test_find_closing_single_not_found() {
        let chars: Vec<char> = "no close here".chars().collect();
        assert!(find_closing_single(&chars, 0, '*').is_none());
    }
}
