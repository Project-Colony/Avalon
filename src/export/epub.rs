use std::path::Path;
use std::fs;
use std::io::Write;
use std::fmt::Write as FmtWrite;
use anyhow::Result;
use pulldown_cmark::{Parser, html::push_html};

use super::compiler::{self, CompileContent, CompileOptions};

/// Save compiled content as an ePub file.
/// ePub is essentially a ZIP with XHTML content, metadata, and a manifest.
pub fn save_epub(contents: &[CompileContent], options: &CompileOptions, path: &Path) -> Result<()> {
    let file = fs::File::create(path)?;
    let mut zip = zip::ZipWriter::new(file);
    let zip_options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);
    let zip_options_deflated = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // 1. mimetype (must be first, uncompressed)
    zip.start_file("mimetype", zip_options)?;
    zip.write_all(b"application/epub+zip")?;

    // 2. META-INF/container.xml
    zip.start_file("META-INF/container.xml", zip_options_deflated)?;
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#)?;

    // 3. Build chapter XHTML files
    let mut chapters: Vec<(String, String, String)> = Vec::new(); // (filename, title, html_content)
    let mut current_chapter_title = String::new();
    let mut current_chapter_html = String::new();
    let mut chapter_idx = 0;

    for content in contents {
        if content.is_folder {
            // Flush previous chapter
            if !current_chapter_html.is_empty() {
                let filename = format!("chapter{}.xhtml", chapter_idx);
                let title = std::mem::replace(&mut current_chapter_title, content.title.clone());
                chapters.push((filename, title, std::mem::take(&mut current_chapter_html)));
                chapter_idx += 1;
            } else {
                current_chapter_title = content.title.clone();
            }
            let level = (content.depth + 1).min(6);
            let _ = writeln!(current_chapter_html,
                "<h{}>{}</h{}>",
                level, escape_xml(&content.title), level
            );
        } else {
            if current_chapter_title.is_empty() {
                current_chapter_title = content.title.clone();
            }
            // Convert markdown to HTML
            let parser = Parser::new(&content.text);
            let mut html_output = String::new();
            push_html(&mut html_output, parser);
            current_chapter_html.push_str(&html_output);
            current_chapter_html.push('\n');
        }
    }

    // Flush last chapter
    if !current_chapter_html.is_empty() {
        let filename = format!("chapter{}.xhtml", chapter_idx);
        chapters.push((filename, current_chapter_title, current_chapter_html));
    }

    // If no chapters were generated, create a placeholder
    if chapters.is_empty() {
        chapters.push((
            "chapter0.xhtml".to_string(),
            options.title.clone(),
            "<p>Empty document</p>".to_string(),
        ));
    }

    // 4. Write chapter files
    for (filename, _title, body) in &chapters {
        zip.start_file(format!("OEBPS/{}", filename), zip_options_deflated)?;
        let xhtml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{title}</title>
    <style type="text/css">
        body {{ font-family: serif; font-size: {size}pt; line-height: 1.6; margin: 1em; }}
        h1, h2, h3 {{ margin-top: 1.5em; }}
        p {{ text-indent: 1.5em; margin: 0.3em 0; }}
    </style>
</head>
<body>
{body}
</body>
</html>"#,
            title = escape_xml(&options.title),
            size = options.font_size,
            body = body,
        );
        zip.write_all(xhtml.as_bytes())?;
    }

    // 5. Title page
    if options.include_front_matter && !options.title.is_empty() {
        zip.start_file("OEBPS/titlepage.xhtml", zip_options_deflated)?;
        let title_page = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>{title}</title></head>
<body>
<h1 style="text-align:center; margin-top:40%">{title}</h1>
{author}
</body>
</html>"#,
            title = escape_xml(&options.title),
            author = if options.author.is_empty() {
                String::new()
            } else {
                format!(
                    "<p style=\"text-align:center; font-style:italic\">{}</p>",
                    escape_xml(&options.author)
                )
            },
        );
        zip.write_all(title_page.as_bytes())?;
    }

    // 6. content.opf (package document)
    zip.start_file("OEBPS/content.opf", zip_options_deflated)?;
    let book_id = uuid::Uuid::new_v4();

    let mut manifest_items = String::new();
    let mut spine_items = String::new();

    if options.include_front_matter && !options.title.is_empty() {
        manifest_items.push_str(
            "    <item id=\"titlepage\" href=\"titlepage.xhtml\" media-type=\"application/xhtml+xml\"/>\n"
        );
        spine_items.push_str("    <itemref idref=\"titlepage\"/>\n");
    }

    for (i, (filename, _, _)) in chapters.iter().enumerate() {
        let _ = writeln!(manifest_items,
            "    <item id=\"ch{}\" href=\"{}\" media-type=\"application/xhtml+xml\"/>",
            i, filename
        );
        let _ = writeln!(spine_items,"    <itemref idref=\"ch{}\"/>", i);
    }

    // Table of contents nav
    manifest_items.push_str(
        "    <item id=\"toc\" href=\"toc.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>\n"
    );

    let opf = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="bookid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="bookid">urn:uuid:{id}</dc:identifier>
    <dc:title>{title}</dc:title>
    <dc:creator>{author}</dc:creator>
    <dc:language>en</dc:language>
    <meta property="dcterms:modified">{date}</meta>
  </metadata>
  <manifest>
{manifest}  </manifest>
  <spine>
{spine}  </spine>
</package>"#,
        id = book_id,
        title = escape_xml(&options.title),
        author = escape_xml(&options.author),
        date = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
        manifest = manifest_items,
        spine = spine_items,
    );
    zip.write_all(opf.as_bytes())?;

    // 7. TOC (table of contents)
    zip.start_file("OEBPS/toc.xhtml", zip_options_deflated)?;
    let mut toc_entries = String::new();
    for (i, (filename, title, _)) in chapters.iter().enumerate() {
        let display_title = if title.is_empty() {
            format!("Chapter {}", i + 1)
        } else {
            title.clone()
        };
        let _ = writeln!(toc_entries,
            "      <li><a href=\"{}\">{}</a></li>",
            filename,
            escape_xml(&display_title)
        );
    }

    let toc = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><title>Table of Contents</title></head>
<body>
  <nav epub:type="toc">
    <h1>Table of Contents</h1>
    <ol>
{entries}    </ol>
  </nav>
</body>
</html>"#,
        entries = toc_entries,
    );
    zip.write_all(toc.as_bytes())?;

    zip.finish()?;
    Ok(())
}

fn escape_xml(s: &str) -> String { compiler::escape_xml(s) }
