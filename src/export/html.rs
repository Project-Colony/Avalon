use std::fmt::Write;
use anyhow::Result;
use pulldown_cmark::{Parser, html::push_html};
use super::compiler::{self, CompileContent, CompileOptions, SeparatorType};

pub fn compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    let estimated_size: usize = contents.iter().map(|c| c.text.len() + c.title.len() + 100).sum();
    let mut body = String::with_capacity(estimated_size);
    let mut doc_index = 0usize;

    for (i, content) in contents.iter().enumerate() {
        if content.is_folder {
            let level = (content.depth + 1).min(6);
            let id = slug(&content.title);
            let _ = writeln!(body,
                "<h{} id=\"{}\" class=\"folder-heading depth-{}\">{}</h{}>",
                level, id, content.depth, escape_html(&content.title), level
            );
        } else {
            doc_index += 1;

            // Separator between consecutive documents
            if i > 0 && !contents[i - 1].is_folder {
                body.push_str(&separator_html(&options.separator));
            }

            let _ = writeln!(body,"<div class=\"document\" data-index=\"{}\">", doc_index);

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
        let _ = writeln!(fm,
            "<p class=\"author\">by {}</p>",
            escape_html(&options.author)
        );
    }

    let _ = writeln!(fm,
        "<p class=\"date\">{}</p>",
        chrono::Local::now().format("%B %d, %Y")
    );

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

fn slug(title: &str) -> String { compiler::slug(title) }
fn escape_html(text: &str) -> String { compiler::escape_html(text) }
