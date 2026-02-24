//! Export integrations: compile with statistics and unified import dispatch.
//!
//! Provides higher-level entry points that combine the compiler, manifest,
//! statistics, and import modules into convenient workflows the GUI calls.

use crate::core::binder::{Binder, BinderItem, BinderItemKind};
use crate::export::compiler::{
    CompileContent, CompileManifest, CompileOptions, CompileStatistics,
    OutputFormat, SectionAssembler,
};
use crate::export::fountain;
use crate::export::markdown_import;
use crate::export::web_import;

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
pub fn compile_with_stats(binder: &Binder, options: &CompileOptions) -> CompileResult {
    let contents = collect_contents(binder, options);

    let statistics = CompileStatistics::from_contents(&contents);
    let manifest = CompileManifest::from_contents(&contents, options);
    let validation_issues = options.validate();
    let settings_summary = options.settings_summary();
    let assembled_preview = SectionAssembler::assemble(&contents, options);

    CompileResult {
        statistics,
        manifest,
        settings_summary,
        validation_issues,
        assembled_preview,
    }
}

/// Collect CompileContent from a binder.
fn collect_contents(binder: &Binder, options: &CompileOptions) -> Vec<CompileContent> {
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
// Format information
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
pub fn supported_export_formats() -> Vec<FormatInfo> {
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

/// Detect an output format from a file path.
pub fn detect_format(path: &std::path::Path) -> Option<OutputFormat> {
    OutputFormat::from_path(path)
}

// ---------------------------------------------------------------------------
// Unified import dispatch
// ---------------------------------------------------------------------------

/// Result of an import operation, regardless of source format.
#[derive(Debug, Clone)]
pub struct ImportResult {
    pub items: Vec<BinderItem>,
    pub source_format: String,
    pub metadata_summary: String,
}

/// Import content from various formats using file-extension detection.
pub fn import_content(content: &str, filename: &str) -> ImportResult {
    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "md" | "markdown" => {
            let front_matter = markdown_import::extract_front_matter(content);
            let items = markdown_import::import_markdown(content, filename)
                .unwrap_or_default();
            let meta = match &front_matter {
                Some(fm) => format!(
                    "title={:?} author={:?} tags={}",
                    fm.title, fm.author, fm.tags.join(", "),
                ),
                None => "No front matter".to_string(),
            };
            ImportResult {
                items,
                source_format: "Markdown".to_string(),
                metadata_summary: meta,
            }
        }
        "html" | "htm" => {
            let items = web_import::import_html_content(content, filename)
                .unwrap_or_default();
            let title = web_import::extract_title_from_html(content);
            let headings = web_import::extract_headings(content);
            ImportResult {
                items,
                source_format: "HTML".to_string(),
                metadata_summary: format!(
                    "title={:?} headings={}",
                    title, headings.len(),
                ),
            }
        }
        "fountain" => {
            let sections = fountain::parse_fountain(content);
            let summary = fountain::screenplay_summary(content);
            let items: Vec<BinderItem> = sections
                .into_iter()
                .map(|(title, body)| {
                    let mut item = BinderItem::new_text(&title);
                    if let Some(ref mut doc) = item.document {
                        doc.content = body;
                    }
                    item
                })
                .collect();
            ImportResult {
                items,
                source_format: "Fountain".to_string(),
                metadata_summary: summary,
            }
        }
        _ => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::binder::Binder;

    fn make_binder() -> Binder {
        let mut binder = Binder::default_structure();
        let mut ch1 = BinderItem::new_folder("Part One");
        ch1.include_in_compile = true;
        let mut scene = BinderItem::new_text("Chapter 1");
        if let Some(ref mut doc) = scene.document {
            doc.content = "Hello world. This is a test.".to_string();
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
    fn test_compile_with_stats() {
        let binder = make_binder();
        let opts = make_opts();
        let result = compile_with_stats(&binder, &opts);
        assert!(result.statistics.total_words > 0);
        assert!(!result.assembled_preview.is_empty());
    }

    #[test]
    fn test_supported_export_formats() {
        let formats = supported_export_formats();
        assert!(!formats.is_empty());
        assert!(formats.iter().any(|f| f.display_name == "HTML"));
    }

    #[test]
    fn test_import_content_markdown() {
        let result = import_content("# Chapter\nContent.", "test.md");
        assert_eq!(result.source_format, "Markdown");
    }

    #[test]
    fn test_import_content_html() {
        let result = import_content("<h1>Title</h1><p>Body.</p>", "page.html");
        assert_eq!(result.source_format, "HTML");
    }

    #[test]
    fn test_import_content_plain() {
        let result = import_content("Just text.", "notes.txt");
        assert_eq!(result.source_format, "Plain Text");
    }

    #[test]
    fn test_detect_format() {
        use std::path::Path;
        assert_eq!(detect_format(Path::new("file.pdf")), Some(OutputFormat::Pdf));
        assert_eq!(detect_format(Path::new("file.xyz")), None);
    }
}
