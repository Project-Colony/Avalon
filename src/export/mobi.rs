#![allow(dead_code)]
use std::fmt::Write;
use anyhow::Result;
use pulldown_cmark::{Parser, html::push_html};

use super::compiler::{self, CompileContent, CompileOptions};

/// Metadata for a MOBI/Kindle publication.
#[derive(Debug, Clone)]
pub struct MobiMetadata {
    pub title: String,
    pub author: String,
    pub language: String,
    pub publisher: Option<String>,
    pub isbn: Option<String>,
    pub description: Option<String>,
    pub cover_image: Option<String>,
}

impl MobiMetadata {
    /// Build metadata from compile options, using sensible defaults for
    /// optional fields.
    pub fn from_options(options: &CompileOptions) -> Self {
        Self {
            title: options.title.clone(),
            author: options.author.clone(),
            language: "en".to_string(),
            publisher: None,
            isbn: None,
            description: None,
            cover_image: None,
        }
    }
}

/// A chapter extracted from the flat content list.
#[derive(Debug, Clone)]
pub struct MobiChapter {
    pub title: String,
    pub content: String,
    pub anchor_id: String,
}

pub fn escape_xml(text: &str) -> String { compiler::escape_xml(text) }
fn anchor_id(title: &str) -> String { compiler::slug(title) }

/// Organise flat `CompileContent` items into discrete chapters.
///
/// Folders become chapter boundaries.  Text items that appear before any
/// folder are grouped into an implicit first chapter.  Consecutive text
/// items under the same folder are merged into one chapter.
pub fn split_into_chapters(contents: &[CompileContent]) -> Vec<MobiChapter> {
    let mut chapters: Vec<MobiChapter> = Vec::new();
    let mut current_title = String::new();
    let mut current_html = String::new();
    let mut chapter_idx: usize = 0;

    for content in contents {
        if content.is_folder {
            // Flush accumulated content as a chapter.
            if !current_html.is_empty() {
                let id = if current_title.is_empty() {
                    format!("chapter-{}", chapter_idx)
                } else {
                    anchor_id(&current_title)
                };
                let title = std::mem::replace(&mut current_title, content.title.clone());
                chapters.push(MobiChapter {
                    title,
                    content: std::mem::take(&mut current_html),
                    anchor_id: id,
                });
                chapter_idx += 1;
            } else {
                current_title = content.title.clone();
            }
            let level = (content.depth + 1).min(6);
            let _ = writeln!(
                current_html,
                "<h{level}>{title}</h{level}>",
                level = level,
                title = escape_xml(&content.title),
            );
        } else {
            if current_title.is_empty() && chapters.is_empty() {
                current_title = content.title.clone();
            }
            let parser = Parser::new(&content.text);
            let mut html_buf = String::new();
            push_html(&mut html_buf, parser);
            current_html.push_str(&html_buf);
            current_html.push('\n');
        }
    }

    // Flush the final chapter.
    if !current_html.is_empty() {
        let id = if current_title.is_empty() {
            format!("chapter-{}", chapter_idx)
        } else {
            anchor_id(&current_title)
        };
        chapters.push(MobiChapter {
            title: current_title,
            content: current_html,
            anchor_id: id,
        });
    }

    chapters
}

/// Generate Kindle-optimised HTML from the compiled content.
///
/// The output is a self-contained HTML document with inline CSS that
/// renders well on Kindle devices and in KindleGen/Calibre conversion
/// pipelines.
pub fn compile_to_html(
    contents: &[CompileContent],
    options: &CompileOptions,
) -> Result<String> {
    let metadata = MobiMetadata::from_options(options);
    let chapters = split_into_chapters(contents);

    let mut body = String::new();

    // Front matter (title page).
    if options.include_front_matter && !options.title.is_empty() {
        body.push_str("<div class=\"title-page\">\n");
        let _ = writeln!(body, "<h1 class=\"book-title\">{}</h1>", escape_xml(&metadata.title));
        if !metadata.author.is_empty() {
            let _ = writeln!(body, "<p class=\"book-author\">{}</p>", escape_xml(&metadata.author));
        }
        if let Some(ref desc) = metadata.description {
            let _ = writeln!(body, "<p class=\"book-description\">{}</p>", escape_xml(desc));
        }
        body.push_str("</div>\n<mbp:pagebreak />\n");
    }

    // Table of contents.
    if options.include_toc && !chapters.is_empty() {
        body.push_str(&generate_toc_html(contents));
        body.push_str("<mbp:pagebreak />\n");
    }

    // Chapter content.
    for (i, ch) in chapters.iter().enumerate() {
        let _ = write!(
            body,
            "<a id=\"{}\"></a>\n<div class=\"chapter\">\n{}</div>\n",
            escape_xml(&ch.anchor_id),
            ch.content,
        );
        if i < chapters.len() - 1 {
            body.push_str("<mbp:pagebreak />\n");
        }
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" lang="{lang}" xml:lang="{lang}">
<head>
    <meta http-equiv="Content-Type" content="text/html; charset=utf-8" />
    <title>{title}</title>
    <style type="text/css">
        body {{
            font-family: serif;
            font-size: {size}pt;
            line-height: 1.6;
            margin: 5%;
            text-align: justify;
        }}
        h1 {{ text-align: center; margin-bottom: 0.5em; page-break-before: always; }}
        h2, h3, h4, h5, h6 {{ margin-top: 1.5em; margin-bottom: 0.5em; }}
        p {{ text-indent: 1.5em; margin: 0.2em 0; }}
        p.first, p:first-of-type {{ text-indent: 0; }}
        .title-page {{ text-align: center; margin-top: 30%; }}
        .book-title {{ font-size: 2em; margin-bottom: 0.5em; }}
        .book-author {{ font-style: italic; font-size: 1.2em; margin-bottom: 1em; }}
        .book-description {{ font-size: 0.9em; color: #555; margin-bottom: 2em; }}
        .toc {{ margin: 1em 0; }}
        .toc h2 {{ text-align: center; }}
        .toc ol {{ list-style-type: none; padding-left: 0; }}
        .toc li {{ margin: 0.3em 0; }}
        .toc a {{ text-decoration: none; color: #000; }}
        .chapter {{ margin-bottom: 2em; }}
        blockquote {{
            margin: 1em 2em;
            padding: 0.5em 1em;
            border-left: 3px solid #ccc;
            font-style: italic;
        }}
        pre, code {{
            font-family: monospace;
            font-size: 0.9em;
        }}
    </style>
</head>
<body>
{body}
</body>
</html>"#,
        lang = escape_xml(&metadata.language),
        title = escape_xml(&metadata.title),
        size = options.font_size,
        body = body,
    );

    Ok(html)
}

/// Generate an HTML table of contents with anchor links that point to
/// each chapter.
pub fn generate_toc_html(contents: &[CompileContent]) -> String {
    let chapters = split_into_chapters(contents);
    let mut toc = String::from("<div class=\"toc\">\n<h2>Table of Contents</h2>\n<ol>\n");

    for (i, ch) in chapters.iter().enumerate() {
        let display = if ch.title.is_empty() {
            format!("Chapter {}", i + 1)
        } else {
            ch.title.clone()
        };
        let _ = writeln!(
            toc,
            "  <li><a href=\"#{anchor}\">{title}</a></li>",
            anchor = escape_xml(&ch.anchor_id),
            title = escape_xml(&display),
        );
    }

    toc.push_str("</ol>\n</div>\n");
    toc
}

/// Generate an OPF (Open Packaging Format) manifest document.
///
/// The OPF is used by KindleGen and Calibre to build a complete .mobi
/// file from the HTML content.
pub fn generate_opf(metadata: &MobiMetadata, has_toc: bool) -> String {
    let book_id = format!(
        "urn:uuid:{}",
        // Deterministic ID based on title for reproducible builds.
        simple_uuid_from_title(&metadata.title),
    );

    let mut meta_extras = String::new();
    if let Some(ref publisher) = metadata.publisher {
        let _ = writeln!(meta_extras, "    <dc:publisher>{}</dc:publisher>", escape_xml(publisher));
    }
    if let Some(ref isbn) = metadata.isbn {
        let _ = writeln!(meta_extras, "    <dc:identifier opf:scheme=\"ISBN\">{}</dc:identifier>", escape_xml(isbn));
    }
    if let Some(ref description) = metadata.description {
        let _ = writeln!(meta_extras, "    <dc:description>{}</dc:description>", escape_xml(description));
    }

    let mut manifest_items = String::from(
        "    <item id=\"content\" href=\"content.html\" media-type=\"text/html\" />\n",
    );
    if has_toc {
        manifest_items.push_str(
            "    <item id=\"toc\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\" />\n",
        );
    }
    if metadata.cover_image.is_some() {
        manifest_items.push_str(
            "    <item id=\"cover-image\" href=\"cover.jpg\" media-type=\"image/jpeg\" />\n",
        );
    }

    let toc_attr = if has_toc { " toc=\"toc\"" } else { "" };

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="bookid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:opf="http://www.idpf.org/2007/opf">
    <dc:identifier id="bookid">{book_id}</dc:identifier>
    <dc:title>{title}</dc:title>
    <dc:creator opf:role="aut">{author}</dc:creator>
    <dc:language>{lang}</dc:language>
{extras}  </metadata>
  <manifest>
{manifest}  </manifest>
  <spine{toc_attr}>
    <itemref idref="content" />
  </spine>
</package>"#,
        book_id = escape_xml(&book_id),
        title = escape_xml(&metadata.title),
        author = escape_xml(&metadata.author),
        lang = escape_xml(&metadata.language),
        extras = meta_extras,
        manifest = manifest_items,
        toc_attr = toc_attr,
    )
}

/// Generate an NCX (Navigation Control for XML) document.
///
/// The NCX provides hierarchical navigation data consumed by Kindle
/// readers and other EPUB/MOBI-compatible tools.
pub fn generate_ncx(contents: &[CompileContent], metadata: &MobiMetadata) -> String {
    let chapters = split_into_chapters(contents);
    let book_uid = simple_uuid_from_title(&metadata.title);

    let mut nav_points = String::new();
    for (i, ch) in chapters.iter().enumerate() {
        let display = if ch.title.is_empty() {
            format!("Chapter {}", i + 1)
        } else {
            ch.title.clone()
        };
        let _ = write!(
            nav_points,
            r#"    <navPoint id="navpoint-{idx}" playOrder="{order}">
      <navLabel><text>{title}</text></navLabel>
      <content src="content.html#{anchor}" />
    </navPoint>
"#,
            idx = i + 1,
            order = i + 1,
            title = escape_xml(&display),
            anchor = escape_xml(&ch.anchor_id),
        );
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE ncx PUBLIC "-//NISO//DTD ncx 2005-1//EN" "http://www.daisy.org/z3986/2005/ncx-2005-1.dtd">
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="urn:uuid:{uid}" />
    <meta name="dtb:depth" content="1" />
    <meta name="dtb:totalPageCount" content="0" />
    <meta name="dtb:maxPageNumber" content="0" />
  </head>
  <docTitle><text>{title}</text></docTitle>
  <navMap>
{nav_points}  </navMap>
</ncx>"#,
        uid = escape_xml(&book_uid),
        title = escape_xml(&metadata.title),
        nav_points = nav_points,
    )
}

/// Main compile entry-point.  Produces Kindle-optimised HTML.
pub fn compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    compile_to_html(contents, options)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Produce a deterministic, reproducible UUID-like string from a title.
/// This is intentionally *not* a real UUID; it exists so that repeated
/// builds with the same title yield the same identifier.
fn simple_uuid_from_title(title: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325; // FNV offset basis
    for byte in title.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0100_0000_01b3); // FNV prime
    }
    let hi = hash;
    // Mix in the reversed bytes for the second half.
    let mut hash2: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in title.as_bytes().iter().rev() {
        hash2 ^= *byte as u64;
        hash2 = hash2.wrapping_mul(0x0100_0000_01b3);
    }
    let lo = hash2;
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        (hi >> 32) as u32,
        (hi >> 16) as u16,
        hi as u16,
        (lo >> 48) as u16,
        lo & 0x0000_ffff_ffff_ffff,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::compiler::{OutputFormat, SeparatorType};

    // -- helpers --

    fn make_content(title: &str, text: &str, is_folder: bool, depth: usize) -> CompileContent {
        CompileContent {
            title: title.to_string(),
            text: text.to_string(),
            depth,
            is_folder,
        }
    }

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
            font_family: "serif".to_string(),
            include_toc: false,
            replace_placeholders: false,
        }
    }

    // ----------------------------------------------------------------
    // escape_xml tests
    // ----------------------------------------------------------------

    #[test]
    fn test_escape_xml_ampersand() {
        assert_eq!(escape_xml("a & b"), "a &amp; b");
    }

    #[test]
    fn test_escape_xml_angle_brackets() {
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
    }

    #[test]
    fn test_escape_xml_quotes() {
        assert_eq!(escape_xml("\"hello\""), "&quot;hello&quot;");
    }

    #[test]
    fn test_escape_xml_apostrophe() {
        assert_eq!(escape_xml("it's"), "it&apos;s");
    }

    #[test]
    fn test_escape_xml_all_entities() {
        let result = escape_xml("<a href=\"x\">&'y</a>");
        assert!(result.contains("&lt;"));
        assert!(result.contains("&gt;"));
        assert!(result.contains("&amp;"));
        assert!(result.contains("&quot;"));
        assert!(result.contains("&apos;"));
    }

    #[test]
    fn test_escape_xml_empty_string() {
        assert_eq!(escape_xml(""), "");
    }

    #[test]
    fn test_escape_xml_no_special_chars() {
        assert_eq!(escape_xml("plain text 123"), "plain text 123");
    }

    // ----------------------------------------------------------------
    // MobiMetadata tests
    // ----------------------------------------------------------------

    #[test]
    fn test_metadata_from_options_basic() {
        let opts = make_opts();
        let meta = MobiMetadata::from_options(&opts);
        assert_eq!(meta.title, "Test Book");
        assert_eq!(meta.author, "Test Author");
        assert_eq!(meta.language, "en");
        assert!(meta.publisher.is_none());
        assert!(meta.isbn.is_none());
        assert!(meta.description.is_none());
        assert!(meta.cover_image.is_none());
    }

    #[test]
    fn test_metadata_from_options_empty_author() {
        let mut opts = make_opts();
        opts.author = String::new();
        let meta = MobiMetadata::from_options(&opts);
        assert_eq!(meta.author, "");
    }

    #[test]
    fn test_metadata_clone() {
        let meta = MobiMetadata::from_options(&make_opts());
        let cloned = meta.clone();
        assert_eq!(cloned.title, meta.title);
        assert_eq!(cloned.author, meta.author);
    }

    // ----------------------------------------------------------------
    // split_into_chapters tests
    // ----------------------------------------------------------------

    #[test]
    fn test_split_chapters_basic() {
        let contents = vec![
            make_content("Chapter 1", "", true, 0),
            make_content("Scene 1", "Hello world.", false, 1),
            make_content("Chapter 2", "", true, 0),
            make_content("Scene 2", "Goodbye world.", false, 1),
        ];
        let chapters = split_into_chapters(&contents);
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "Chapter 1");
        assert_eq!(chapters[1].title, "Chapter 2");
    }

    #[test]
    fn test_split_chapters_no_folders() {
        let contents = vec![
            make_content("Scene A", "First.", false, 0),
            make_content("Scene B", "Second.", false, 0),
        ];
        let chapters = split_into_chapters(&contents);
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].title, "Scene A");
        assert!(chapters[0].content.contains("First."));
        assert!(chapters[0].content.contains("Second."));
    }

    #[test]
    fn test_split_chapters_empty_input() {
        let chapters = split_into_chapters(&[]);
        assert!(chapters.is_empty());
    }

    #[test]
    fn test_split_chapters_folders_only() {
        let contents = vec![
            make_content("Part 1", "", true, 0),
            make_content("Part 2", "", true, 0),
        ];
        let chapters = split_into_chapters(&contents);
        // Part 1 has heading HTML but is flushed when Part 2 starts;
        // Part 2 heading is the final pending content.
        assert_eq!(chapters.len(), 2);
    }

    #[test]
    fn test_split_chapters_anchor_ids() {
        let contents = vec![
            make_content("My Chapter", "", true, 0),
            make_content("Scene", "Content.", false, 1),
        ];
        let chapters = split_into_chapters(&contents);
        assert_eq!(chapters[0].anchor_id, "my-chapter");
    }

    #[test]
    fn test_split_chapters_deeply_nested() {
        let contents = vec![
            make_content("Part", "", true, 0),
            make_content("Chapter", "", true, 1),
            make_content("Scene", "Deeply nested text.", false, 2),
        ];
        let chapters = split_into_chapters(&contents);
        assert!(chapters.len() >= 1);
        // The final chapter should contain the scene text.
        let last = chapters.last().unwrap();
        assert!(last.content.contains("Deeply nested text."));
    }

    // ----------------------------------------------------------------
    // compile_to_html tests
    // ----------------------------------------------------------------

    #[test]
    fn test_compile_to_html_basic() {
        let contents = vec![make_content("Scene", "Hello Kindle!", false, 0)];
        let result = compile_to_html(&contents, &make_opts()).unwrap();
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("Hello Kindle!"));
        assert!(result.contains("<title>Test Book</title>"));
    }

    #[test]
    fn test_compile_to_html_empty() {
        let result = compile_to_html(&[], &make_opts()).unwrap();
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("</html>"));
    }

    #[test]
    fn test_compile_to_html_with_front_matter() {
        let mut opts = make_opts();
        opts.include_front_matter = true;
        let contents = vec![make_content("S", "Content.", false, 0)];
        let result = compile_to_html(&contents, &opts).unwrap();
        assert!(result.contains("title-page"));
        assert!(result.contains("book-title"));
        assert!(result.contains("Test Book"));
        assert!(result.contains("Test Author"));
    }

    #[test]
    fn test_compile_to_html_front_matter_no_author() {
        let mut opts = make_opts();
        opts.include_front_matter = true;
        opts.author = String::new();
        let contents = vec![make_content("S", "Text.", false, 0)];
        let result = compile_to_html(&contents, &opts).unwrap();
        assert!(result.contains("book-title"));
        // No <p class="book-author"> element should be emitted.
        assert!(!result.contains("<p class=\"book-author\">"));
    }

    #[test]
    fn test_compile_to_html_with_toc() {
        let mut opts = make_opts();
        opts.include_toc = true;
        let contents = vec![
            make_content("Chapter 1", "", true, 0),
            make_content("Scene", "Text.", false, 1),
        ];
        let result = compile_to_html(&contents, &opts).unwrap();
        assert!(result.contains("Table of Contents"));
        assert!(result.contains("Chapter 1"));
    }

    #[test]
    fn test_compile_to_html_page_breaks_between_chapters() {
        let contents = vec![
            make_content("Ch 1", "", true, 0),
            make_content("S1", "First.", false, 1),
            make_content("Ch 2", "", true, 0),
            make_content("S2", "Second.", false, 1),
        ];
        let result = compile_to_html(&contents, &make_opts()).unwrap();
        assert!(result.contains("mbp:pagebreak"));
    }

    #[test]
    fn test_compile_to_html_font_size() {
        let mut opts = make_opts();
        opts.font_size = 16.0;
        let result = compile_to_html(&[], &opts).unwrap();
        assert!(result.contains("16pt"));
    }

    #[test]
    fn test_compile_to_html_xss_safe() {
        let mut opts = make_opts();
        opts.title = "<script>alert(1)</script>".to_string();
        let result = compile_to_html(&[], &opts).unwrap();
        assert!(!result.contains("<script>alert(1)</script>"));
        assert!(result.contains("&lt;script&gt;"));
    }

    // ----------------------------------------------------------------
    // generate_toc_html tests
    // ----------------------------------------------------------------

    #[test]
    fn test_generate_toc_html_basic() {
        let contents = vec![
            make_content("Chapter 1", "", true, 0),
            make_content("Scene", "text", false, 1),
            make_content("Chapter 2", "", true, 0),
            make_content("Scene 2", "text", false, 1),
        ];
        let toc = generate_toc_html(&contents);
        assert!(toc.contains("Table of Contents"));
        assert!(toc.contains("Chapter 1"));
        assert!(toc.contains("Chapter 2"));
        assert!(toc.contains("<ol>"));
        assert!(toc.contains("</ol>"));
    }

    #[test]
    fn test_generate_toc_html_empty() {
        let toc = generate_toc_html(&[]);
        assert!(toc.contains("Table of Contents"));
        assert!(toc.contains("<ol>"));
        assert!(toc.contains("</ol>"));
    }

    #[test]
    fn test_generate_toc_html_anchor_links() {
        let contents = vec![
            make_content("Intro", "", true, 0),
            make_content("Text", "body", false, 1),
        ];
        let toc = generate_toc_html(&contents);
        assert!(toc.contains("href=\"#intro\""));
    }

    #[test]
    fn test_generate_toc_html_unnamed_chapters() {
        let contents = vec![make_content("", "content", false, 0)];
        let toc = generate_toc_html(&contents);
        assert!(toc.contains("Chapter 1"));
    }

    // ----------------------------------------------------------------
    // generate_opf tests
    // ----------------------------------------------------------------

    #[test]
    fn test_generate_opf_basic() {
        let meta = MobiMetadata::from_options(&make_opts());
        let opf = generate_opf(&meta, false);
        assert!(opf.contains("<?xml version"));
        assert!(opf.contains("<package"));
        assert!(opf.contains("Test Book"));
        assert!(opf.contains("Test Author"));
        assert!(opf.contains("content.html"));
    }

    #[test]
    fn test_generate_opf_with_toc() {
        let meta = MobiMetadata::from_options(&make_opts());
        let opf = generate_opf(&meta, true);
        assert!(opf.contains("toc.ncx"));
        assert!(opf.contains("toc=\"toc\""));
    }

    #[test]
    fn test_generate_opf_without_toc() {
        let meta = MobiMetadata::from_options(&make_opts());
        let opf = generate_opf(&meta, false);
        assert!(!opf.contains("toc.ncx"));
        assert!(!opf.contains("toc=\"toc\""));
    }

    #[test]
    fn test_generate_opf_with_publisher() {
        let mut meta = MobiMetadata::from_options(&make_opts());
        meta.publisher = Some("Avalon Press".to_string());
        let opf = generate_opf(&meta, false);
        assert!(opf.contains("Avalon Press"));
        assert!(opf.contains("dc:publisher"));
    }

    #[test]
    fn test_generate_opf_with_isbn() {
        let mut meta = MobiMetadata::from_options(&make_opts());
        meta.isbn = Some("978-3-16-148410-0".to_string());
        let opf = generate_opf(&meta, false);
        assert!(opf.contains("978-3-16-148410-0"));
        assert!(opf.contains("opf:scheme=\"ISBN\""));
    }

    #[test]
    fn test_generate_opf_with_description() {
        let mut meta = MobiMetadata::from_options(&make_opts());
        meta.description = Some("A thrilling tale.".to_string());
        let opf = generate_opf(&meta, false);
        assert!(opf.contains("A thrilling tale."));
        assert!(opf.contains("dc:description"));
    }

    #[test]
    fn test_generate_opf_with_cover_image() {
        let mut meta = MobiMetadata::from_options(&make_opts());
        meta.cover_image = Some("cover.jpg".to_string());
        let opf = generate_opf(&meta, false);
        assert!(opf.contains("cover-image"));
        assert!(opf.contains("cover.jpg"));
    }

    #[test]
    fn test_generate_opf_language() {
        let meta = MobiMetadata::from_options(&make_opts());
        let opf = generate_opf(&meta, false);
        assert!(opf.contains("<dc:language>en</dc:language>"));
    }

    // ----------------------------------------------------------------
    // generate_ncx tests
    // ----------------------------------------------------------------

    #[test]
    fn test_generate_ncx_basic() {
        let contents = vec![
            make_content("Chapter 1", "", true, 0),
            make_content("Scene", "text", false, 1),
        ];
        let meta = MobiMetadata::from_options(&make_opts());
        let ncx = generate_ncx(&contents, &meta);
        assert!(ncx.contains("<?xml version"));
        assert!(ncx.contains("<ncx"));
        assert!(ncx.contains("navPoint"));
        assert!(ncx.contains("Chapter 1"));
        assert!(ncx.contains("content.html#"));
    }

    #[test]
    fn test_generate_ncx_empty() {
        let meta = MobiMetadata::from_options(&make_opts());
        let ncx = generate_ncx(&[], &meta);
        assert!(ncx.contains("<navMap>"));
        assert!(ncx.contains("</navMap>"));
        assert!(!ncx.contains("navPoint"));
    }

    #[test]
    fn test_generate_ncx_play_order() {
        let contents = vec![
            make_content("Ch 1", "", true, 0),
            make_content("S1", "text", false, 1),
            make_content("Ch 2", "", true, 0),
            make_content("S2", "text", false, 1),
        ];
        let meta = MobiMetadata::from_options(&make_opts());
        let ncx = generate_ncx(&contents, &meta);
        assert!(ncx.contains("playOrder=\"1\""));
        assert!(ncx.contains("playOrder=\"2\""));
    }

    #[test]
    fn test_generate_ncx_doc_title() {
        let meta = MobiMetadata::from_options(&make_opts());
        let ncx = generate_ncx(&[], &meta);
        assert!(ncx.contains("<docTitle><text>Test Book</text></docTitle>"));
    }

    // ----------------------------------------------------------------
    // compile (main entry) tests
    // ----------------------------------------------------------------

    #[test]
    fn test_compile_returns_html() {
        let contents = vec![make_content("Scene", "Some text.", false, 0)];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("Some text."));
    }

    #[test]
    fn test_compile_empty_contents() {
        let result = compile(&[], &make_opts()).unwrap();
        assert!(result.contains("<!DOCTYPE html>"));
    }

    // ----------------------------------------------------------------
    // anchor_id / helper tests
    // ----------------------------------------------------------------

    #[test]
    fn test_anchor_id_basic() {
        assert_eq!(anchor_id("Chapter One"), "chapter-one");
    }

    #[test]
    fn test_anchor_id_special_chars() {
        assert_eq!(anchor_id("Hello, World!"), "hello-world");
    }

    #[test]
    fn test_anchor_id_empty() {
        assert_eq!(anchor_id(""), "");
    }

    #[test]
    fn test_simple_uuid_deterministic() {
        let a = simple_uuid_from_title("My Book");
        let b = simple_uuid_from_title("My Book");
        assert_eq!(a, b);
    }

    #[test]
    fn test_simple_uuid_different_titles() {
        let a = simple_uuid_from_title("Book A");
        let b = simple_uuid_from_title("Book B");
        assert_ne!(a, b);
    }

    // ----------------------------------------------------------------
    // Integration-style tests
    // ----------------------------------------------------------------

    #[test]
    fn test_full_pipeline_with_all_options() {
        let mut opts = make_opts();
        opts.include_front_matter = true;
        opts.include_toc = true;
        opts.title = "Grand Novel".to_string();
        opts.author = "Jane Doe".to_string();

        let contents = vec![
            make_content("Part One", "", true, 0),
            make_content("Chapter 1", "", true, 1),
            make_content("Opening", "It was a dark and stormy night.", false, 2),
            make_content("Part Two", "", true, 0),
            make_content("Chapter 2", "", true, 1),
            make_content("Climax", "The hero prevailed.", false, 2),
        ];

        let html = compile(&contents, &opts).unwrap();
        assert!(html.contains("Grand Novel"));
        assert!(html.contains("Jane Doe"));
        assert!(html.contains("Table of Contents"));
        assert!(html.contains("dark and stormy night"));
        assert!(html.contains("hero prevailed"));
        assert!(html.contains("mbp:pagebreak"));

        let meta = MobiMetadata::from_options(&opts);
        let opf = generate_opf(&meta, true);
        assert!(opf.contains("Grand Novel"));

        let ncx = generate_ncx(&contents, &meta);
        assert!(ncx.contains("navPoint"));
    }

    #[test]
    fn test_compile_preserves_markdown_formatting() {
        let contents = vec![
            make_content("S", "**bold** and *italic*", false, 0),
        ];
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("<strong>bold</strong>"));
        assert!(result.contains("<em>italic</em>"));
    }

    #[test]
    fn test_compile_large_document() {
        let contents: Vec<CompileContent> = (0..100)
            .map(|i| {
                if i % 10 == 0 {
                    make_content(&format!("Part {}", i / 10 + 1), "", true, 0)
                } else {
                    make_content(
                        &format!("Scene {}", i),
                        &format!("Content for scene number {}.", i),
                        false,
                        1,
                    )
                }
            })
            .collect();
        let result = compile(&contents, &make_opts()).unwrap();
        assert!(result.contains("Content for scene number 1."));
        assert!(result.contains("Content for scene number 99."));
    }
}
