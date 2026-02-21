use anyhow::Result;
use pulldown_cmark::{Parser, html::push_html};
use super::compiler::{CompileContent, CompileOptions};

pub fn compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    let mut body = String::new();

    for content in contents {
        if content.is_folder {
            let level = (content.depth + 2).min(6);
            body.push_str(&format!("<h{}>{}</h{}>\n", level, content.title, level));
        } else {
            // Convert markdown content to HTML
            let parser = Parser::new(&content.text);
            let mut html_output = String::new();
            push_html(&mut html_output, parser);
            body.push_str(&html_output);
            body.push('\n');
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
        }}
        h1 {{ text-align: center; margin-bottom: 0.5em; }}
        h2, h3, h4 {{ margin-top: 2em; }}
        p {{ text-indent: 1.5em; margin: 0.5em 0; }}
        p:first-of-type {{ text-indent: 0; }}
        .author {{ text-align: center; font-style: italic; margin-bottom: 2em; }}
        hr {{ margin: 2em 0; border: none; border-top: 1px solid #ccc; }}
    </style>
</head>
<body>
{front_matter}{body}
</body>
</html>"#,
        title = options.title,
        font = options.font_family,
        size = options.font_size,
        front_matter = if options.include_front_matter && !options.title.is_empty() {
            format!(
                "<h1>{}</h1>\n{}<hr>\n",
                options.title,
                if options.author.is_empty() {
                    String::new()
                } else {
                    format!("<p class=\"author\">{}</p>\n", options.author)
                }
            )
        } else {
            String::new()
        },
        body = body,
    );

    Ok(html)
}
