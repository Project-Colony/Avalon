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
