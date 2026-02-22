use anyhow::Result;
use pulldown_cmark::{Parser, html::push_html};
use super::compiler::{CompileContent, CompileOptions, SeparatorType};

pub fn compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    let mut body = String::new();
    let mut doc_index = 0usize;

    for (i, content) in contents.iter().enumerate() {
        if content.is_folder {
            let level = (content.depth + 1).min(6);
            let id = slug(&content.title);
            body.push_str(&format!(
                "<h{} id=\"{}\" class=\"folder-heading depth-{}\">{}</h{}>\n",
                level, id, content.depth, escape_html(&content.title), level
            ));
        } else {
            doc_index += 1;

            // Separator between consecutive documents
            if i > 0 && !contents[i - 1].is_folder {
                body.push_str(&separator_html(&options.separator));
            }

            body.push_str(&format!("<div class=\"document\" data-index=\"{}\">\n", doc_index));

            // Convert markdown content to HTML
            let parser = Parser::new(&content.text);
            let mut html_output = String::new();
            push_html(&mut html_output, parser);
            body.push_str(&html_output);

            body.push_str("</div>\n");
        }
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        body {{
            font-family: '{font}', serif;
            font-size: {size}pt;
            max-width: 800px;
            margin: 0 auto;
            padding: 40px 20px;
            line-height: 1.6;
            color: #333;
            background: #fff;
        }}
        h1 {{ text-align: center; margin-bottom: 0.5em; }}
        h2, h3, h4, h5, h6 {{ margin-top: 2em; margin-bottom: 0.5em; }}
        p {{ text-indent: 1.5em; margin: 0.5em 0; }}
        p:first-of-type {{ text-indent: 0; }}
        .author {{ text-align: center; font-style: italic; margin-bottom: 2em; }}
        .date {{ text-align: center; color: #666; margin-bottom: 2em; font-size: 0.9em; }}
        hr {{ margin: 2em 0; border: none; border-top: 1px solid #ccc; }}
        .section-break {{ text-align: center; margin: 2em 0; color: #999; letter-spacing: 0.5em; }}
        .page-break {{ page-break-after: always; margin: 0; border: none; }}
        blockquote {{
            margin: 1em 2em;
            padding: 0.5em 1em;
            border-left: 3px solid #ccc;
            color: #555;
            font-style: italic;
        }}
        pre, code {{
            font-family: 'Courier New', monospace;
            background: #f5f5f5;
            padding: 0.2em 0.4em;
            border-radius: 3px;
            font-size: 0.9em;
        }}
        pre {{ padding: 1em; overflow-x: auto; }}
        pre code {{ background: none; padding: 0; }}
        .footnote {{ font-size: 0.85em; color: #666; }}
        .footnote-ref {{ vertical-align: super; font-size: 0.75em; }}
        .document {{ margin-bottom: 0.5em; }}
        @media print {{
            body {{ max-width: none; padding: 0; }}
            .page-break {{ page-break-after: always; }}
            a {{ color: #333; text-decoration: none; }}
        }}
    </style>
</head>
<body>
{front_matter}{body}
</body>
</html>"#,
        title = escape_html(&options.title),
        font = escape_html(&options.font_family),
        size = options.font_size,
        front_matter = build_front_matter(options),
        body = body,
    );

    Ok(html)
}

fn build_front_matter(options: &CompileOptions) -> String {
    if !options.include_front_matter || options.title.is_empty() {
        return String::new();
    }

    let mut fm = format!("<h1>{}</h1>\n", escape_html(&options.title));

    if !options.author.is_empty() {
        fm.push_str(&format!(
            "<p class=\"author\">by {}</p>\n",
            escape_html(&options.author)
        ));
    }

    fm.push_str(&format!(
        "<p class=\"date\">{}</p>\n",
        chrono::Local::now().format("%B %d, %Y")
    ));

    fm.push_str("<hr>\n");
    fm
}

fn separator_html(sep: &SeparatorType) -> String {
    match sep {
        SeparatorType::EmptyLine => "<br>\n".to_string(),
        SeparatorType::PageBreak => "<div class=\"page-break\"></div>\n".to_string(),
        SeparatorType::SectionBreak => {
            "<p class=\"section-break\">&bull; &bull; &bull;</p>\n".to_string()
        }
        SeparatorType::Custom(s) => format!("<p class=\"section-break\">{}</p>\n", escape_html(s)),
        SeparatorType::None => String::new(),
    }
}

/// Create a URL-safe slug from a title
fn slug(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Escape HTML special characters
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Generate a standalone HTML table of contents
pub fn generate_toc(contents: &[CompileContent]) -> String {
    let mut toc = String::from("<nav class=\"toc\">\n<h2>Table of Contents</h2>\n<ul>\n");

    for content in contents {
        if content.is_folder || !content.text.is_empty() {
            let id = slug(&content.title);
            let indent = "  ".repeat(content.depth);
            toc.push_str(&format!(
                "{}<li><a href=\"#{}\">{}</a></li>\n",
                indent,
                id,
                escape_html(&content.title)
            ));
        }
    }

    toc.push_str("</ul>\n</nav>\n<hr>\n");
    toc
}

/// Count total words across all content sections
pub fn word_count(contents: &[CompileContent]) -> usize {
    contents.iter().map(|c| c.text.split_whitespace().count()).sum()
}

/// Count total characters across all content sections
pub fn char_count(contents: &[CompileContent]) -> usize {
    contents.iter().map(|c| c.text.len()).sum()
}

/// Estimate the output size in bytes for an HTML compilation
pub fn estimate_output_size(contents: &[CompileContent], options: &CompileOptions) -> usize {
    // Base HTML boilerplate + CSS is roughly 1500 bytes
    let base = 1500;
    let front_matter = if options.include_front_matter { 200 } else { 0 };
    // HTML tags roughly double the content size
    let content_size: usize = contents.iter()
        .map(|c| c.text.len() * 2 + c.title.len() + 50)
        .sum();
    base + front_matter + content_size
}

/// Strip HTML tags from a string, returning plain text
pub fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}

/// Extract all heading texts from the compiled content
pub fn extract_headings(contents: &[CompileContent]) -> Vec<(String, usize)> {
    contents.iter()
        .filter(|c| c.is_folder)
        .map(|c| (c.title.clone(), c.depth))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::compiler::OutputFormat;

    fn make_opts() -> CompileOptions {
        CompileOptions {
            format: OutputFormat::Html,
            title: "Test Book".to_string(),
            author: "Test Author".to_string(),
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
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("Hello world."));
        assert!(result.contains("<title>Test Book</title>"));
    }

    #[test]
    fn test_compile_with_front_matter() {
        let mut opts = make_opts();
        opts.include_front_matter = true;
        let contents = vec![make_content("Scene", "Content.", false, 1)];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("<h1>Test Book</h1>"));
        assert!(result.contains("Test Author"));
    }

    #[test]
    fn test_compile_folder_heading() {
        let contents = vec![
            make_content("Chapter One", "", true, 0),
            make_content("Scene 1", "Text here.", false, 1),
        ];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("Chapter One"));
        assert!(result.contains("<h1"));
    }

    #[test]
    fn test_escape_html_special_chars() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("a&b"), "a&amp;b");
        assert_eq!(escape_html("\"quotes\""), "&quot;quotes&quot;");
    }

    #[test]
    fn test_slug_generation() {
        assert_eq!(slug("Chapter One"), "chapter-one");
        assert_eq!(slug("Hello World!"), "hello-world");
        assert_eq!(slug("test"), "test");
    }

    #[test]
    fn test_separator_html_types() {
        assert!(separator_html(&SeparatorType::EmptyLine).contains("<br>"));
        assert!(separator_html(&SeparatorType::PageBreak).contains("page-break"));
        assert!(separator_html(&SeparatorType::SectionBreak).contains("section-break"));
        assert!(separator_html(&SeparatorType::Custom("***".to_string())).contains("***"));
        assert!(separator_html(&SeparatorType::None).is_empty());
    }

    #[test]
    fn test_generate_toc() {
        let contents = vec![
            make_content("Chapter 1", "", true, 0),
            make_content("Scene 1", "Text.", false, 1),
            make_content("Chapter 2", "", true, 0),
        ];
        let toc = generate_toc(&contents);
        assert!(toc.contains("Table of Contents"));
        assert!(toc.contains("Chapter 1"));
        assert!(toc.contains("Chapter 2"));
    }

    #[test]
    fn test_word_and_char_count() {
        let contents = vec![
            make_content("A", "one two three", false, 0),
            make_content("B", "four five", false, 0),
        ];
        assert_eq!(word_count(&contents), 5);
        assert!(char_count(&contents) > 0);
    }

    #[test]
    fn test_estimate_output_size() {
        let contents = vec![make_content("A", "Some text here", false, 0)];
        let size = estimate_output_size(&contents, &make_opts());
        assert!(size > 1500);
    }

    #[test]
    fn test_strip_html_tags() {
        assert_eq!(strip_html_tags("<p>Hello</p>"), "Hello");
        assert_eq!(strip_html_tags("<b>bold</b> text"), "bold text");
        assert_eq!(strip_html_tags("no tags"), "no tags");
    }

    #[test]
    fn test_extract_headings() {
        let contents = vec![
            make_content("Chapter 1", "", true, 0),
            make_content("Scene 1", "text", false, 1),
            make_content("Chapter 2", "", true, 0),
        ];
        let headings = extract_headings(&contents);
        assert_eq!(headings.len(), 2);
        assert_eq!(headings[0].0, "Chapter 1");
        assert_eq!(headings[1].0, "Chapter 2");
    }

    #[test]
    fn test_compile_empty() {
        let contents: Vec<CompileContent> = vec![];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("</html>"));
    }

    #[test]
    fn test_compile_front_matter_no_author() {
        let mut opts = make_opts();
        opts.include_front_matter = true;
        opts.author = String::new();
        let contents = vec![make_content("Scene", "Text.", false, 1)];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("<h1>Test Book</h1>"));
        assert!(!result.contains("class=\"author\""));
    }

    #[test]
    fn test_compile_front_matter_empty_title() {
        let mut opts = make_opts();
        opts.include_front_matter = true;
        opts.title = String::new();
        let contents = vec![make_content("Scene", "Text.", false, 1)];
        let result = compile(&contents, &opts).unwrap();
        // Empty title should skip front matter
        assert!(!result.contains("<h1></h1>"));
    }

    #[test]
    fn test_compile_multiple_docs_with_separator() {
        let mut opts = make_opts();
        opts.separator = SeparatorType::SectionBreak;
        let contents = vec![
            make_content("S1", "First.", false, 1),
            make_content("S2", "Second.", false, 1),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("section-break"));
        assert!(result.contains("First."));
        assert!(result.contains("Second."));
    }

    #[test]
    fn test_compile_page_break_separator() {
        let mut opts = make_opts();
        opts.separator = SeparatorType::PageBreak;
        let contents = vec![
            make_content("S1", "First.", false, 1),
            make_content("S2", "Second.", false, 1),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("page-break"));
    }

    #[test]
    fn test_compile_custom_separator() {
        let mut opts = make_opts();
        opts.separator = SeparatorType::Custom("~~~".to_string());
        let contents = vec![
            make_content("S1", "First.", false, 1),
            make_content("S2", "Second.", false, 1),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("~~~"));
    }

    #[test]
    fn test_compile_no_separator() {
        let mut opts = make_opts();
        opts.separator = SeparatorType::None;
        let contents = vec![
            make_content("S1", "First.", false, 1),
            make_content("S2", "Second.", false, 1),
        ];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("First."));
        assert!(result.contains("Second."));
    }

    #[test]
    fn test_heading_depth_capped_at_6() {
        let contents = vec![make_content("Deep", "", true, 10)];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("<h6"));
    }

    #[test]
    fn test_slug_special_characters() {
        assert_eq!(slug("Hello, World!"), "hello-world");
        assert_eq!(slug("Chapter 1: The Beginning"), "chapter-1-the-beginning");
        assert_eq!(slug("---"), "");
    }

    #[test]
    fn test_escape_html_apostrophe() {
        assert_eq!(escape_html("it's"), "it&#39;s");
    }

    #[test]
    fn test_strip_html_nested_tags() {
        assert_eq!(strip_html_tags("<div><p><b>nested</b></p></div>"), "nested");
    }

    #[test]
    fn test_strip_html_no_tags() {
        assert_eq!(strip_html_tags("plain text"), "plain text");
    }

    #[test]
    fn test_word_count_empty() {
        let contents: Vec<CompileContent> = vec![];
        assert_eq!(word_count(&contents), 0);
    }

    #[test]
    fn test_estimate_output_size_with_front_matter() {
        let contents = vec![make_content("A", "text", false, 0)];
        let mut opts = make_opts();
        opts.include_front_matter = true;
        let size_fm = estimate_output_size(&contents, &opts);
        opts.include_front_matter = false;
        let size_no = estimate_output_size(&contents, &opts);
        assert!(size_fm > size_no);
    }

    #[test]
    fn test_extract_headings_no_folders() {
        let contents = vec![
            make_content("Doc 1", "text", false, 0),
            make_content("Doc 2", "text", false, 0),
        ];
        let headings = extract_headings(&contents);
        assert!(headings.is_empty());
    }

    #[test]
    fn test_generate_toc_empty() {
        let contents: Vec<CompileContent> = vec![];
        let toc = generate_toc(&contents);
        assert!(toc.contains("Table of Contents"));
        assert!(toc.contains("</ul>"));
    }

    #[test]
    fn test_compile_markdown_in_content() {
        let contents = vec![make_content("S", "**bold** and *italic*", false, 0)];
        let result = compile(&contents, &make_opts()).unwrap();
        // pulldown_cmark should convert markdown to HTML
        assert!(result.contains("<strong>bold</strong>"));
        assert!(result.contains("<em>italic</em>"));
    }

    #[test]
    fn test_compile_font_size_in_css() {
        let mut opts = make_opts();
        opts.font_size = 14.0;
        let contents = vec![make_content("S", "Text.", false, 0)];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("14pt"));
    }

    #[test]
    fn test_compile_font_family_in_css() {
        let mut opts = make_opts();
        opts.font_family = "Georgia".to_string();
        let contents = vec![make_content("S", "Text.", false, 0)];
        let result = compile(&contents, &opts).unwrap();
        assert!(result.contains("Georgia"));
    }
}
