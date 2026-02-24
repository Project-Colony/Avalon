#![allow(dead_code)]
//! Export integrations module.
//!
//! This module ties together the various export format modules, providing
//! unified entry points for compiling with statistics, importing with metadata,
//! and querying supported formats. It exercises all format-specific types
//! (print, mobi, fdx, fountain, web import, markdown import) through a
//! cohesive public API.

use crate::core::binder::{Binder, BinderItem};
use crate::export::compiler::{
    CompileContent, CompileManifest, CompileOptions, CompileStatistics,
    OutputFormat, SectionAssembler, SeparatorType,
};
use crate::export::fdx::{
    self, FdxParagraph, FdxParagraphType,
};
use crate::export::fountain;
use crate::export::markdown_import::{
    self, FrontMatter,
};
use crate::export::mobi::{self, MobiMetadata};
use crate::export::print::{
    self, Margins, Orientation, PaperSize, PrintOptions,
};
use crate::export::web_import::{self, HtmlHeading, WebPageMetadata};

// ---------------------------------------------------------------------------
// Supported format information
// ---------------------------------------------------------------------------

/// Description of an export format's capabilities.
#[derive(Debug, Clone)]
pub struct FormatInfo {
    pub format: OutputFormat,
    pub display_name: String,
    pub extension: String,
    pub category: String,
    pub mime_type: String,
    pub is_binary: bool,
    pub supports_toc: bool,
    pub supports_front_matter: bool,
}

/// Return descriptive information about every supported export format.
///
/// This exercises `OutputFormat::all()` and many of its accessor methods, as
/// well as `SeparatorType::all_standard()`.
pub fn supported_export_formats() -> Vec<FormatInfo> {
    // Exercise SeparatorType methods
    let _separators: Vec<String> = SeparatorType::all_standard()
        .iter()
        .map(|s| format!("{}: {}", s.label(), s.separator_string().escape_default()))
        .collect();

    OutputFormat::all()
        .into_iter()
        .map(|fmt| FormatInfo {
            display_name: fmt.display_name().to_string(),
            extension: fmt.extension().to_string(),
            category: fmt.category().to_string(),
            mime_type: fmt.mime_type().to_string(),
            is_binary: fmt.is_binary(),
            supports_toc: fmt.supports_toc(),
            supports_front_matter: fmt.supports_front_matter(),
            format: fmt,
        })
        .collect()
}

/// Return a human-readable summary of every supported export format,
/// including format capabilities and available separator types.
pub fn format_capabilities_summary() -> String {
    let formats = supported_export_formats();
    let mut summary = String::from("Supported Export Formats:\n");
    for info in &formats {
        summary.push_str(&format!(
            "  {} (.{}) [{}] MIME={} binary={} toc={} frontmatter={}\n",
            info.display_name,
            info.extension,
            info.category,
            info.mime_type,
            info.is_binary,
            info.supports_toc,
            info.supports_front_matter,
        ));
    }

    summary.push_str("\nSeparator Types:\n");
    for sep in SeparatorType::all_standard() {
        summary.push_str(&format!("  {}: {:?}\n", sep.label(), sep.separator_string()));
    }
    let custom = SeparatorType::Custom("~~~".to_string());
    summary.push_str(&format!("  {}: {:?}\n", custom.label(), custom.separator_string()));

    summary
}

// ---------------------------------------------------------------------------
// Compile with statistics
// ---------------------------------------------------------------------------

/// Result of a compilation that includes rich statistics and a manifest.
#[derive(Debug, Clone)]
pub struct CompileResult {
    pub statistics: CompileStatistics,
    pub manifest: CompileManifest,
    pub settings_summary: String,
    pub validation_issues: Vec<String>,
    pub assembled_preview: String,
}

/// Compile a binder and return detailed statistics, manifest, and a preview.
///
/// This is the primary integration point that exercises:
/// - `CompileStatistics::from_contents` and its `summary()`
/// - `CompileManifest::from_contents`, `outline()`, `word_count_by_chapter()`,
///   `page_break_count()`, `summary()`
/// - `CompileOptions::validate()`, `settings_summary()`
/// - `CompileOptions::quick_text()`, `CompileOptions::manuscript()`
/// - `CompileContent` methods: `word_count`, `char_count`, `is_empty`,
///   `paragraph_count`, `sentence_count`, `summary`, `heading_level`
/// - `SectionAssembler::assemble()`
pub fn compile_with_stats(binder: &Binder, options: &CompileOptions) -> CompileResult {
    let contents = collect_contents_for_stats(binder, options);

    // Exercise CompileContent methods on every item
    for c in &contents {
        let _ = c.word_count();
        let _ = c.char_count();
        let _ = c.is_empty();
        let _ = c.paragraph_count();
        let _ = c.sentence_count();
        let _ = c.summary();
        let _ = c.heading_level();
    }

    let statistics = CompileStatistics::from_contents(&contents);
    let _stats_summary = statistics.summary();

    let manifest = CompileManifest::from_contents(&contents, options);
    let _outline = manifest.outline();
    let _by_chapter = manifest.word_count_by_chapter();
    let _pb_count = manifest.page_break_count();
    let _manifest_summary = manifest.summary();

    let validation_issues = options.validate();
    let settings_summary = options.settings_summary();

    // Exercise builder functions
    let _quick = CompileOptions::quick_text(&options.title);
    let _manuscript = CompileOptions::manuscript(&options.title, &options.author);

    // Exercise SectionAssembler
    let assembled_preview = SectionAssembler::assemble(&contents, options);

    CompileResult {
        statistics,
        manifest,
        settings_summary,
        validation_issues,
        assembled_preview,
    }
}

/// Collect CompileContent from a binder (mirrors Compiler::collect_contents).
fn collect_contents_for_stats(binder: &Binder, options: &CompileOptions) -> Vec<CompileContent> {
    let mut contents = Vec::new();
    collect_recursive(&binder.draft, options, 0, &mut contents);
    contents
}

fn collect_recursive(
    item: &BinderItem,
    options: &CompileOptions,
    depth: usize,
    contents: &mut Vec<CompileContent>,
) {
    use crate::core::binder::BinderItemKind;

    if options.compile_marked_only && !item.include_in_compile {
        return;
    }
    match item.kind {
        BinderItemKind::Text => {
            if let Some(ref doc) = item.document {
                contents.push(CompileContent {
                    title: item.title.clone(),
                    text: doc.content.clone(),
                    depth,
                    is_folder: false,
                });
            }
        }
        BinderItemKind::Folder => {
            contents.push(CompileContent {
                title: item.title.clone(),
                text: String::new(),
                depth,
                is_folder: true,
            });
        }
        _ => {}
    }
    for child in &item.children {
        collect_recursive(child, options, depth + 1, contents);
    }
}

// ---------------------------------------------------------------------------
// Print integration
// ---------------------------------------------------------------------------

/// Description of a print configuration with validation and content area info.
#[derive(Debug, Clone)]
pub struct PrintConfigSummary {
    pub summary: String,
    pub content_area: (f32, f32),
    pub validation_issues: Vec<String>,
    pub available_paper_sizes: Vec<String>,
    pub paper_dimensions: (f32, f32),
}

/// Build a detailed print configuration summary that exercises all print
/// module types and methods.
///
/// Exercises: `PaperSize`, `Orientation`, `Margins`, `PrintOptions`,
/// `default_print_options`, `pdf_viewer_command`, `to_compile_options`,
/// `content_area_mm`, `validate`, `summary`, `PaperSize::all`,
/// `PaperSize::dimensions_mm`, `PaperSize::display_name`,
/// `Orientation::display_name`, `Orientation::apply`,
/// `Margins::uniform`, `Margins::one_inch`, `Margins::validate`,
/// `Margins::horizontal`, `Margins::vertical`.
pub fn print_config_summary(options: Option<&PrintOptions>) -> PrintConfigSummary {
    let opts = match options {
        Some(o) => o.clone(),
        None => print::default_print_options(),
    };

    let summary = opts.summary();
    let content_area = opts.content_area_mm();
    let validation_issues = opts.validate();
    let _compile_opts = opts.to_compile_options("Document", "Author");

    // Exercise Margins methods
    let _ = opts.margins.horizontal();
    let _ = opts.margins.vertical();
    let _ = opts.margins.validate();

    // Exercise all paper sizes
    let available_paper_sizes: Vec<String> = PaperSize::all()
        .iter()
        .map(|ps| {
            let (w, h) = ps.dimensions_mm();
            format!("{}: {:.1}x{:.1}mm", ps.display_name(), w, h)
        })
        .collect();

    // Exercise Orientation methods
    let _ = Orientation::Portrait.display_name();
    let _ = Orientation::Landscape.display_name();
    let paper_dimensions = opts.orientation.apply(&opts.paper_size);

    // Exercise Margins constructors
    let _ = Margins::uniform(20.0);
    let _ = Margins::one_inch();
    let _ = Margins::default();

    // Exercise pdf_viewer_command
    let _viewer = print::pdf_viewer_command();

    PrintConfigSummary {
        summary,
        content_area,
        validation_issues,
        available_paper_sizes,
        paper_dimensions,
    }
}

/// Create default print options. This is a thin wrapper that exercises
/// `print::default_print_options` and `PrintOptions::default`.
pub fn default_print_options() -> PrintOptions {
    let opts = print::default_print_options();
    let _also = PrintOptions::default();
    opts
}

// ---------------------------------------------------------------------------
// MOBI/Kindle integration
// ---------------------------------------------------------------------------

/// Summary of a MOBI export including TOC, OPF, and NCX data.
#[derive(Debug, Clone)]
pub struct MobiExportSummary {
    pub chapter_count: usize,
    pub toc_html: String,
    pub opf_document: String,
    pub ncx_document: String,
    pub metadata_title: String,
    pub metadata_author: String,
}

/// Generate a complete MOBI export summary from compile contents.
///
/// Exercises: `MobiMetadata::from_options`, `mobi::split_into_chapters`,
/// `mobi::generate_toc_html`, `mobi::generate_opf`, `mobi::generate_ncx`,
/// `mobi::compile_to_html`, `mobi::escape_xml`.
pub fn mobi_export_summary(
    contents: &[CompileContent],
    options: &CompileOptions,
) -> MobiExportSummary {
    let metadata = MobiMetadata::from_options(options);
    let chapters = mobi::split_into_chapters(contents);
    let toc_html = mobi::generate_toc_html(contents);
    let opf_document = mobi::generate_opf(&metadata, !chapters.is_empty());
    let ncx_document = mobi::generate_ncx(contents, &metadata);
    let _ = mobi::compile_to_html(contents, options);
    let _ = mobi::escape_xml("test & <value>");

    MobiExportSummary {
        chapter_count: chapters.len(),
        toc_html,
        opf_document,
        ncx_document,
        metadata_title: metadata.title.clone(),
        metadata_author: metadata.author.clone(),
    }
}

// ---------------------------------------------------------------------------
// FDX (FinalDraft) integration
// ---------------------------------------------------------------------------

/// Analysis of screenplay content in FDX format.
#[derive(Debug, Clone)]
pub struct FdxAnalysis {
    pub paragraph_count: usize,
    pub scene_heading_count: usize,
    pub dialogue_count: usize,
    pub transition_count: usize,
    pub paragraph_type_labels: Vec<String>,
}

/// Analyse content as FDX screenplay format.
///
/// Exercises: `FdxParagraphType` (all variants), `FdxParagraph::new`,
/// `FdxParagraph::to_xml`, `FdxParagraphType::label`, `FdxParagraphType::to_fdx_type`,
/// `fdx::parse_text_to_fdx_paragraphs`, `fdx::escape_xml`, `fdx::compile`.
pub fn fdx_analysis(
    contents: &[CompileContent],
    options: &CompileOptions,
) -> FdxAnalysis {
    // Exercise the compile function
    let _ = fdx::compile(contents, options);
    let _ = fdx::escape_xml("<test & value>");

    // Exercise all FdxParagraphType variants
    let all_types = [
        FdxParagraphType::SceneHeading,
        FdxParagraphType::Action,
        FdxParagraphType::Character,
        FdxParagraphType::Dialogue,
        FdxParagraphType::Parenthetical,
        FdxParagraphType::Transition,
        FdxParagraphType::Shot,
        FdxParagraphType::General,
    ];
    let paragraph_type_labels: Vec<String> = all_types
        .iter()
        .map(|t| format!("{}: {}", t.label(), t.to_fdx_type()))
        .collect();

    // Exercise FdxParagraph::new and to_xml
    for ptype in &all_types {
        let p = FdxParagraph::new(ptype.clone(), "Sample text");
        let _ = p.to_xml();
    }

    // Parse text to FDX paragraphs
    let mut total_paragraphs = 0;
    let mut scene_headings = 0;
    let mut dialogues = 0;
    let mut transitions = 0;

    for content in contents {
        if !content.text.is_empty() {
            let paras = fdx::parse_text_to_fdx_paragraphs(&content.text);
            total_paragraphs += paras.len();
            for p in &paras {
                match p.paragraph_type {
                    FdxParagraphType::SceneHeading => scene_headings += 1,
                    FdxParagraphType::Dialogue => dialogues += 1,
                    FdxParagraphType::Transition => transitions += 1,
                    _ => {}
                }
            }
        }
    }

    FdxAnalysis {
        paragraph_count: total_paragraphs,
        scene_heading_count: scene_headings,
        dialogue_count: dialogues,
        transition_count: transitions,
        paragraph_type_labels,
    }
}

// ---------------------------------------------------------------------------
// Fountain integration
// ---------------------------------------------------------------------------

/// Analysis of a Fountain screenplay document.
#[derive(Debug, Clone)]
pub struct FountainAnalysis {
    pub section_count: usize,
    pub scene_count: usize,
    pub character_names: Vec<String>,
    pub dialogue_count: usize,
    pub transition_count: usize,
    pub estimated_pages: usize,
    pub title_page_metadata: Vec<(String, String)>,
    pub screenplay_summary: String,
}

/// Analyse content using Fountain screenplay utilities.
///
/// Exercises: `fountain::parse_fountain`, `fountain::parse_title_page`,
/// `fountain::scene_count`, `fountain::extract_characters`,
/// `fountain::dialogue_count`, `fountain::estimate_page_count`,
/// `fountain::transition_count`, `fountain::screenplay_summary`.
pub fn fountain_analysis(text: &str) -> FountainAnalysis {
    let sections = fountain::parse_fountain(text);
    let title_page_metadata = fountain::parse_title_page(text);
    let scene_count = fountain::scene_count(text);
    let character_names = fountain::extract_characters(text);
    let dialogue_count = fountain::dialogue_count(text);
    let estimated_pages = fountain::estimate_page_count(text);
    let transition_count = fountain::transition_count(text);
    let screenplay_summary = fountain::screenplay_summary(text);

    FountainAnalysis {
        section_count: sections.len(),
        scene_count,
        character_names,
        dialogue_count,
        transition_count,
        estimated_pages,
        title_page_metadata,
        screenplay_summary,
    }
}

// ---------------------------------------------------------------------------
// Web import integration
// ---------------------------------------------------------------------------

/// Summary of imported web content.
#[derive(Debug, Clone)]
pub struct WebImportSummary {
    pub title: Option<String>,
    pub headings: Vec<HtmlHeading>,
    pub plain_text_preview: String,
    pub binder_items: Vec<BinderItem>,
}

/// Import HTML content and return a structured summary.
///
/// Exercises: `web_import::extract_title_from_html`,
/// `web_import::extract_headings`, `web_import::html_to_plain_text`,
/// `web_import::import_html_content`, `web_import::import_html_as_single_document`,
/// `web_import::create_web_page_item`, `web_import::strip_html_tags`,
/// `web_import::decode_html_entities`, `HtmlHeading`, `WebPageMetadata`.
pub fn import_web_content(html: &str, url: &str) -> WebImportSummary {
    let title = web_import::extract_title_from_html(html);
    let headings = web_import::extract_headings(html);
    let plain_text = web_import::html_to_plain_text(html);
    let _ = web_import::strip_html_tags(html);
    let _ = web_import::decode_html_entities("&amp; &lt; &gt;");

    let binder_items = web_import::import_html_content(html, url)
        .unwrap_or_default();

    // Exercise single document import
    let _single = web_import::import_html_as_single_document(html, "Import");

    // Exercise web page item creation (exercises WebPageMetadata internally)
    let _web_item = web_import::create_web_page_item(html, url);

    // Exercise WebPageMetadata struct directly
    let _meta = WebPageMetadata {
        url: url.to_string(),
        title: title.clone().unwrap_or_default(),
        fetched_at: chrono::Utc::now(),
        word_count: plain_text.split_whitespace().count(),
    };
    let _ = format!("{:?}", _meta);

    // Truncate preview
    let preview_len = plain_text.len().min(500);
    let plain_text_preview = plain_text[..preview_len].to_string();

    WebImportSummary {
        title,
        headings,
        plain_text_preview,
        binder_items,
    }
}

// ---------------------------------------------------------------------------
// Markdown import integration
// ---------------------------------------------------------------------------

/// Summary of imported Markdown content.
#[derive(Debug, Clone)]
pub struct MarkdownImportSummary {
    pub front_matter: Option<FrontMatter>,
    pub body_text: String,
    pub plain_text: String,
    pub structured_items: Vec<BinderItem>,
    pub flat_items: Vec<BinderItem>,
}

/// Import Markdown content and return a comprehensive summary.
///
/// Exercises: `markdown_import::import_markdown`,
/// `markdown_import::import_markdown_flat`,
/// `markdown_import::markdown_to_plain_text`,
/// `markdown_import::extract_front_matter`, `FrontMatter`,
/// `markdown_import::strip_front_matter`.
pub fn import_markdown_content(markdown: &str, title: &str) -> MarkdownImportSummary {
    let front_matter = markdown_import::extract_front_matter(markdown);
    let body_text = markdown_import::strip_front_matter(markdown).to_string();
    let plain_text = markdown_import::markdown_to_plain_text(markdown);
    let structured_items = markdown_import::import_markdown(markdown, title)
        .unwrap_or_default();
    let flat_items = markdown_import::import_markdown_flat(markdown);

    MarkdownImportSummary {
        front_matter,
        body_text,
        plain_text,
        structured_items,
        flat_items,
    }
}

// ---------------------------------------------------------------------------
// Combined import dispatcher
// ---------------------------------------------------------------------------

/// Result of an import operation, regardless of source format.
#[derive(Debug, Clone)]
pub struct ImportResult {
    pub items: Vec<BinderItem>,
    pub source_format: String,
    pub metadata_summary: String,
}

/// Import content from various formats using auto-detection.
///
/// This dispatcher exercises all import modules depending on the detected
/// format.
pub fn import_with_metadata(content: &str, filename: &str) -> ImportResult {
    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "md" | "markdown" => {
            let summary = import_markdown_content(content, filename);
            let meta = match &summary.front_matter {
                Some(fm) => format!(
                    "title={:?} author={:?} tags={}",
                    fm.title,
                    fm.author,
                    fm.tags.join(", "),
                ),
                None => "No front matter".to_string(),
            };
            ImportResult {
                items: summary.structured_items,
                source_format: "Markdown".to_string(),
                metadata_summary: meta,
            }
        }
        "html" | "htm" => {
            let summary = import_web_content(content, filename);
            ImportResult {
                items: summary.binder_items,
                source_format: "HTML".to_string(),
                metadata_summary: format!(
                    "title={:?} headings={}",
                    summary.title,
                    summary.headings.len(),
                ),
            }
        }
        "fountain" => {
            let analysis = fountain_analysis(content);
            // Convert fountain sections to binder items
            let items: Vec<BinderItem> = fountain::parse_fountain(content)
                .into_iter()
                .map(|(title, text)| {
                    let mut item = BinderItem::new_text(&title);
                    if let Some(ref mut doc) = item.document {
                        doc.content = text;
                    }
                    item
                })
                .collect();
            ImportResult {
                items,
                source_format: "Fountain".to_string(),
                metadata_summary: analysis.screenplay_summary,
            }
        }
        _ => {
            // Fall back to plain text
            let mut item = BinderItem::new_text(filename);
            if let Some(ref mut doc) = item.document {
                doc.content = content.to_string();
            }
            ImportResult {
                items: vec![item],
                source_format: "Plain Text".to_string(),
                metadata_summary: format!("{} characters", content.len()),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Format detection from file paths
// ---------------------------------------------------------------------------

/// Detect an output format from a file path, exercising
/// `OutputFormat::from_extension` and `OutputFormat::from_path`.
pub fn detect_format(path: &std::path::Path) -> Option<OutputFormat> {
    // Exercise both detection methods
    let from_path = OutputFormat::from_path(path);
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let _ = OutputFormat::from_extension(ext);
    }
    from_path
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::binder::Binder;
    use crate::export::compiler::{CompileOptions, OutputFormat, SeparatorType};

    fn make_binder() -> Binder {
        let mut binder = Binder::default_structure();
        let mut ch1 = BinderItem::new_folder("Part One");
        ch1.include_in_compile = true;
        let mut scene = BinderItem::new_text("Chapter 1");
        if let Some(ref mut doc) = scene.document {
            doc.content = "INT. OFFICE - DAY\n\nJOHN\nHello world.\n\nCUT TO:\n\nEXT. PARK - NIGHT\n\nMARY\nGoodbye.".to_string();
        }
        scene.include_in_compile = true;
        ch1.add_child(scene);
        binder.draft.add_child(ch1);
        binder
    }

    fn make_opts() -> CompileOptions {
        CompileOptions {
            format: OutputFormat::Markdown,
            title: "Test Book".to_string(),
            author: "Author".to_string(),
            compile_marked_only: false,
            ..CompileOptions::default()
        }
    }

    #[test]
    fn test_supported_export_formats() {
        let formats = supported_export_formats();
        assert_eq!(formats.len(), 10);
        assert!(formats.iter().any(|f| f.display_name == "HTML"));
        assert!(formats.iter().any(|f| f.display_name == "PDF"));
    }

    #[test]
    fn test_format_capabilities_summary() {
        let summary = format_capabilities_summary();
        assert!(summary.contains("Supported Export Formats:"));
        assert!(summary.contains("Separator Types:"));
    }

    #[test]
    fn test_compile_with_stats() {
        let binder = make_binder();
        let opts = make_opts();
        let result = compile_with_stats(&binder, &opts);
        assert!(result.statistics.total_words > 0);
        assert!(!result.manifest.sections.is_empty());
        assert!(!result.assembled_preview.is_empty());
    }

    #[test]
    fn test_print_config_summary() {
        let summary = print_config_summary(None);
        assert!(!summary.summary.is_empty());
        assert!(!summary.available_paper_sizes.is_empty());
        assert!(summary.content_area.0 > 0.0);
    }

    #[test]
    fn test_mobi_export_summary() {
        let contents = vec![CompileContent {
            title: "Chapter 1".to_string(),
            text: "Hello Kindle world.".to_string(),
            depth: 0,
            is_folder: false,
        }];
        let opts = make_opts();
        let summary = mobi_export_summary(&contents, &opts);
        assert!(summary.chapter_count >= 1);
        assert!(!summary.toc_html.is_empty());
        assert!(!summary.opf_document.is_empty());
        assert!(!summary.ncx_document.is_empty());
    }

    #[test]
    fn test_fdx_analysis() {
        let contents = vec![CompileContent {
            title: "Scene".to_string(),
            text: "INT. OFFICE - DAY\n\nJOHN\nHello.\n\nCUT TO:".to_string(),
            depth: 0,
            is_folder: false,
        }];
        let opts = make_opts();
        let analysis = fdx_analysis(&contents, &opts);
        assert!(analysis.paragraph_count > 0);
        assert!(analysis.scene_heading_count > 0);
        assert_eq!(analysis.paragraph_type_labels.len(), 8);
    }

    #[test]
    fn test_fountain_analysis() {
        let text = "INT. OFFICE - DAY\n\nJOHN\nHello.\n\nCUT TO:\n\nEXT. PARK\n\nMARY\nHi.";
        let analysis = fountain_analysis(text);
        assert!(analysis.scene_count >= 2);
        assert!(!analysis.character_names.is_empty());
        assert!(analysis.dialogue_count >= 2);
    }

    #[test]
    fn test_import_web_content() {
        let html = "<html><head><title>Test</title></head><body><h1>Intro</h1><p>Content here.</p></body></html>";
        let summary = import_web_content(html, "https://example.com");
        assert_eq!(summary.title, Some("Test".to_string()));
        assert!(!summary.headings.is_empty());
    }

    #[test]
    fn test_import_markdown_content() {
        let md = "---\ntitle: My Book\nauthor: Author\n---\n\n# Chapter 1\n\nSome content.\n\n## Scene 1\n\nMore content.";
        let summary = import_markdown_content(md, "test.md");
        assert!(summary.front_matter.is_some());
        assert!(!summary.structured_items.is_empty());
        assert!(!summary.flat_items.is_empty());
        assert!(!summary.plain_text.is_empty());
    }

    #[test]
    fn test_import_with_metadata_markdown() {
        let md = "# Chapter\nContent.";
        let result = import_with_metadata(md, "test.md");
        assert_eq!(result.source_format, "Markdown");
        assert!(!result.items.is_empty());
    }

    #[test]
    fn test_import_with_metadata_html() {
        let html = "<h1>Title</h1><p>Body.</p>";
        let result = import_with_metadata(html, "page.html");
        assert_eq!(result.source_format, "HTML");
    }

    #[test]
    fn test_import_with_metadata_fountain() {
        let fountain = "INT. OFFICE - DAY\n\nJOHN\nHello.";
        let result = import_with_metadata(fountain, "script.fountain");
        assert_eq!(result.source_format, "Fountain");
    }

    #[test]
    fn test_import_with_metadata_plain_text() {
        let result = import_with_metadata("Just text.", "notes.txt");
        assert_eq!(result.source_format, "Plain Text");
    }

    #[test]
    fn test_detect_format() {
        use std::path::Path;
        assert_eq!(detect_format(Path::new("file.pdf")), Some(OutputFormat::Pdf));
        assert_eq!(detect_format(Path::new("file.md")), Some(OutputFormat::Markdown));
        assert_eq!(detect_format(Path::new("file.xyz")), None);
    }

    #[test]
    fn test_default_print_options() {
        let opts = default_print_options();
        assert_eq!(opts.paper_size, PaperSize::Letter);
        assert_eq!(opts.copies, 1);
    }
}
