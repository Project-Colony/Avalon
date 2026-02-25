#![allow(dead_code)] // Methods used by test code
use std::path::Path;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::core::binder::{Binder, BinderItem, BinderItemKind};

/// Output format for compilation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    PlainText,
    Markdown,
    Html,
    Pdf,
    Latex,
    Docx,
    Epub,
    Rtf,
    Opml,
    Fountain,
}

impl OutputFormat {
    pub fn extension(&self) -> &str {
        match self {
            OutputFormat::PlainText => "txt",
            OutputFormat::Markdown => "md",
            OutputFormat::Html => "html",
            OutputFormat::Pdf => "pdf",
            OutputFormat::Latex => "tex",
            OutputFormat::Docx => "docx",
            OutputFormat::Epub => "epub",
            OutputFormat::Rtf => "rtf",
            OutputFormat::Opml => "opml",
            OutputFormat::Fountain => "fountain",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            OutputFormat::PlainText => "Plain Text",
            OutputFormat::Markdown => "Markdown",
            OutputFormat::Html => "HTML",
            OutputFormat::Pdf => "PDF",
            OutputFormat::Latex => "LaTeX",
            OutputFormat::Docx => "Word (DOCX)",
            OutputFormat::Epub => "ePub",
            OutputFormat::Rtf => "RTF",
            OutputFormat::Opml => "OPML",
            OutputFormat::Fountain => "Fountain",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            OutputFormat::PlainText,
            OutputFormat::Markdown,
            OutputFormat::Html,
            OutputFormat::Pdf,
            OutputFormat::Latex,
            OutputFormat::Docx,
            OutputFormat::Epub,
            OutputFormat::Rtf,
            OutputFormat::Opml,
            OutputFormat::Fountain,
        ]
    }

}

/// Options for compiling/exporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileOptions {
    pub format: OutputFormat,
    pub title: String,
    pub author: String,
    /// Include front matter (title page)
    pub include_front_matter: bool,
    /// Separator between documents
    pub separator: SeparatorType,
    /// Whether to add page breaks between top-level folders
    pub page_break_between_folders: bool,
    /// Include only items marked for compile
    pub compile_marked_only: bool,
    /// Font size for output
    pub font_size: f32,
    /// Font family for output
    pub font_family: String,
    /// Include table of contents
    pub include_toc: bool,
    /// Replace Scrivener-style placeholders (<$n>, <$date>, etc.)
    pub replace_placeholders: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            format: OutputFormat::Markdown,
            title: String::new(),
            author: String::new(),
            include_front_matter: true,
            separator: SeparatorType::EmptyLine,
            page_break_between_folders: true,
            compile_marked_only: true,
            font_size: 12.0,
            font_family: "Times New Roman".to_string(),
            include_toc: false,
            replace_placeholders: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SeparatorType {
    EmptyLine,
    PageBreak,
    SectionBreak,
    Custom(String),
    None,
}

/// The compiler that assembles documents for export
pub struct Compiler;

impl Compiler {
    /// Compile the draft into a single output string
    pub fn compile(binder: &Binder, options: &CompileOptions) -> Result<String> {
        let contents = Self::collect_contents(&binder.draft, options);

        let mut output = match options.format {
            OutputFormat::PlainText => plain_text_compile(&contents, options),
            OutputFormat::Markdown => super::markdown::compile(&contents, options),
            OutputFormat::Html => super::html::compile(&contents, options),
            OutputFormat::Latex => super::latex::compile(&contents, options),
            OutputFormat::Rtf => super::rtf::compile(&contents, options),
            OutputFormat::Fountain => super::fountain::compile(&contents, options),
            OutputFormat::Opml => super::opml::export_opml(binder, &options.title),
            OutputFormat::Pdf => {
                Ok("PDF compilation requires save_to_file()".to_string())
            }
            OutputFormat::Docx => {
                Ok("DOCX compilation requires save_to_file()".to_string())
            }
            OutputFormat::Epub => {
                Ok("ePub compilation requires save_to_file()".to_string())
            }
        }?;

        // Insert table of contents if enabled
        if options.include_toc {
            let sections: Vec<(String, usize)> = contents.iter()
                .filter(|c| c.is_folder || !c.text.is_empty())
                .map(|c| (c.title.clone(), c.depth))
                .collect();

            let toc = match options.format {
                OutputFormat::Html => super::placeholders::generate_toc_html(&sections),
                OutputFormat::Markdown => super::placeholders::generate_toc_markdown(&sections),
                _ => super::placeholders::generate_toc(&sections),
            };

            // Insert TOC after front matter (or at start)
            if options.include_front_matter {
                // Find end of front matter (first double newline)
                if let Some(pos) = output.find("\n\n") {
                    output.insert_str(pos + 2, &toc);
                } else {
                    output = format!("{}\n{}", toc, output);
                }
            } else {
                output = format!("{}{}", toc, output);
            }
        }

        // Replace placeholders if enabled
        if options.replace_placeholders {
            let total_words = contents.iter()
                .map(|c| c.text.split_whitespace().count())
                .sum::<usize>();
            let total_chars = contents.iter()
                .map(|c| c.text.len())
                .sum::<usize>();

            let context = super::placeholders::PlaceholderContext {
                project_title: options.title.clone(),
                author: options.author.clone(),
                word_count: total_words,
                char_count: total_chars,
                page_count: (total_words / crate::core::WORDS_PER_PAGE).max(1),
            };
            output = super::placeholders::replace_placeholders(&output, &context);
        }

        Ok(output)
    }

    /// Save compiled output to a file
    pub fn save_to_file(binder: &Binder, options: &CompileOptions, path: &Path) -> Result<()> {
        let contents = Self::collect_contents(&binder.draft, options);

        match options.format {
            OutputFormat::Pdf => {
                super::pdf::save_pdf(&contents, options, path)
            }
            OutputFormat::Docx => {
                super::docx::save_docx(&contents, options, path)
            }
            OutputFormat::Epub => {
                super::epub::save_epub(&contents, options, path)
            }
            _ => {
                let output = Self::compile(binder, options)?;
                std::fs::write(path, output)?;
                Ok(())
            }
        }
    }

    /// Collect contents from the binder tree in order
    fn collect_contents(item: &BinderItem, options: &CompileOptions) -> Vec<CompileContent> {
        let mut contents = Vec::new();
        Self::collect_recursive(item, options, 0, &mut contents);
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
                // Add folder as a heading
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
            Self::collect_recursive(child, options, depth + 1, contents);
        }
    }
}

/// Content ready for compilation
#[derive(Debug, Clone)]
pub struct CompileContent {
    pub title: String,
    pub text: String,
    pub depth: usize,
    pub is_folder: bool,
}

impl CompileContent {
    /// Word count for this content item
    pub fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }

    /// Character count for this content item
    pub fn char_count(&self) -> usize {
        self.text.len()
    }

    /// Paragraph count for this content item
    pub fn paragraph_count(&self) -> usize {
        self.text.split("\n\n").filter(|p| !p.trim().is_empty()).count()
    }

    /// Sentence count (approximate)
    pub fn sentence_count(&self) -> usize {
        self.text.chars()
            .filter(|c| matches!(c, '.' | '!' | '?'))
            .count()
            .max(if self.text.trim().is_empty() { 0 } else { 1 })
    }

}

impl CompileOptions {
    /// Check if the export configuration is valid
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        if self.title.is_empty() && self.include_front_matter {
            issues.push("Title is empty but front matter is enabled".to_string());
        }
        if self.font_size < 6.0 || self.font_size > 72.0 {
            issues.push(format!("Font size {} is outside reasonable range (6-72)", self.font_size));
        }
        if self.font_family.is_empty() {
            issues.push("Font family is empty".to_string());
        }
        issues
    }

    /// Get a summary of the export settings
    pub fn settings_summary(&self) -> String {
        let mut parts = vec![
            format!("Format: {}", self.format.display_name()),
            format!("Font: {} {}pt", self.font_family, self.font_size),
        ];
        if self.include_front_matter { parts.push("With front matter".to_string()); }
        if self.include_toc { parts.push("With TOC".to_string()); }
        if self.compile_marked_only { parts.push("Marked items only".to_string()); }
        parts.join(", ")
    }
}

impl SeparatorType {
    /// Get the actual separator string to insert between documents
    pub fn separator_string(&self) -> &str {
        match self {
            SeparatorType::EmptyLine => "\n\n",
            SeparatorType::PageBreak => "\n\n---\n\n",
            SeparatorType::SectionBreak => "\n\n***\n\n",
            SeparatorType::Custom(s) => s.as_str(),
            SeparatorType::None => "",
        }
    }

}

/// Statistics gathered from compiled content
#[derive(Debug, Clone)]
pub struct CompileStatistics {
    pub total_words: usize,
}

impl CompileStatistics {
    /// Compute statistics from a set of compile contents
    pub fn from_contents(contents: &[CompileContent]) -> Self {
        let sections: Vec<&CompileContent> = contents.iter().filter(|c| !c.is_folder).collect();

        let total_words: usize = sections.iter().map(|c| c.word_count()).sum();

        Self {
            total_words,
        }
    }
}

/// A structured compile plan describing the assembled output
#[derive(Debug, Clone)]
pub struct CompileManifest {
    pub sections: Vec<ManifestEntry>,
}

/// An entry in the compile manifest describing one section
#[derive(Debug, Clone)]
pub struct ManifestEntry {
    pub title: String,
    pub heading_level: usize,
    pub is_folder: bool,
    pub has_page_break_before: bool,
}

impl CompileManifest {
    /// Build a manifest from collected contents and compile options
    pub fn from_contents(contents: &[CompileContent], options: &CompileOptions) -> Self {
        let mut sections = Vec::new();
        let mut prev_was_folder_at_depth_0 = false;

        for (i, c) in contents.iter().enumerate() {
            let heading_level = (c.depth + 1).min(6); // HTML h1-h6
            let has_page_break_before = if i == 0 {
                false
            } else if c.is_folder && c.depth == 0 && options.page_break_between_folders {
                true
            } else if prev_was_folder_at_depth_0 && options.page_break_between_folders {
                false // Already handled by the folder entry
            } else {
                matches!(options.separator, SeparatorType::PageBreak)
            };

            sections.push(ManifestEntry {
                title: c.title.clone(),
                heading_level,
                is_folder: c.is_folder,
                has_page_break_before,
            });

            prev_was_folder_at_depth_0 = c.is_folder && c.depth == 0;
        }

        Self {
            sections,
        }
    }
}

/// Assembles document sections into a single output string with proper
/// separators, headings, and page breaks.
pub struct SectionAssembler;

impl SectionAssembler {
    /// Assemble contents into a single text with headings and separators
    pub fn assemble(contents: &[CompileContent], options: &CompileOptions) -> String {
        let estimated: usize = contents.iter().map(|c| c.text.len() + c.title.len() + 20).sum();
        let mut output = String::with_capacity(estimated);
        let manifest = CompileManifest::from_contents(contents, options);

        for (i, entry) in manifest.sections.iter().enumerate() {
            let content = &contents[i];

            // Page break
            if entry.has_page_break_before && !output.is_empty() {
                output.push_str("\n\n---\n\n");
            } else if i > 0 && !entry.has_page_break_before && !output.is_empty() {
                output.push_str(options.separator.separator_string());
            }

            // Heading for folders and section titles
            if entry.is_folder {
                let heading = Self::format_heading(&entry.title, entry.heading_level, &options.format);
                output.push_str(&heading);
                output.push('\n');
            } else {
                if !content.title.is_empty() {
                    let heading = Self::format_heading(&content.title, entry.heading_level, &options.format);
                    output.push_str(&heading);
                    output.push('\n');
                }
                if !content.text.is_empty() {
                    output.push_str(&content.text);
                }
            }
        }

        output
    }

    /// Format a heading appropriate for the output format
    fn format_heading(title: &str, level: usize, format: &OutputFormat) -> String {
        match format {
            OutputFormat::Markdown => {
                format!("{} {}", "#".repeat(level), title)
            }
            OutputFormat::Html => {
                format!("<h{}>{}</h{}>", level, title, level)
            }
            OutputFormat::Latex => {
                let cmd = match level {
                    1 => "chapter",
                    2 => "section",
                    3 => "subsection",
                    4 => "subsubsection",
                    _ => "paragraph",
                };
                format!("\\{}{{{}}}",  cmd, title)
            }
            _ => {
                // Plain text: uppercase for top-level, indented for deeper
                if level <= 1 {
                    title.to_uppercase()
                } else {
                    format!("{}{}", "  ".repeat(level - 1), title)
                }
            }
        }
    }
}

fn plain_text_compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    super::plain_text::compile(contents, options)
}

// ── Compile renderer trait for factoring common export logic ──

/// Trait for format-specific rendering. Implement this to add a new export format
/// without duplicating the common iteration, separator, and page-break logic.
///
/// The default `render` method handles the common loop over contents:
/// front matter → (heading | separator | text)* → finalize.
pub trait CompileRenderer {
    /// Render the front matter / title block. Return empty string if not applicable.
    fn render_front_matter(&self, options: &CompileOptions) -> String;

    /// Render a folder heading at the given depth.
    fn render_heading(&self, title: &str, depth: usize) -> String;

    /// Render the separator between consecutive text documents.
    fn render_separator(&self, sep: &SeparatorType) -> String;

    /// Render a page break between top-level sections.
    fn render_page_break(&self) -> String;

    /// Render document text content (may apply format-specific transforms).
    fn render_text(&self, text: &str) -> String;

    /// Wrap the assembled body in any required document envelope (e.g., HTML shell).
    fn finalize(&self, body: String, options: &CompileOptions) -> String {
        let _ = options;
        body
    }

    /// Default rendering pipeline using the trait methods above.
    fn render(&self, contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
        let estimated: usize = contents.iter().map(|c| c.text.len() + c.title.len() + 40).sum();
        let mut body = String::with_capacity(estimated);

        if options.include_front_matter {
            body.push_str(&self.render_front_matter(options));
        }

        let mut prev_was_text = false;

        for (i, content) in contents.iter().enumerate() {
            if content.is_folder {
                if options.page_break_between_folders && content.depth <= 1 && i > 0 {
                    body.push_str(&self.render_page_break());
                }
                body.push_str(&self.render_heading(&content.title, content.depth));
                prev_was_text = false;
            } else if !content.text.is_empty() {
                if prev_was_text {
                    body.push_str(&self.render_separator(&options.separator));
                }
                body.push_str(&self.render_text(&content.text));
                if !content.text.ends_with('\n') {
                    body.push('\n');
                }
                prev_was_text = true;
            }
        }

        Ok(self.finalize(body, options))
    }
}

// ── Shared utility functions used by multiple export modules ──

/// Total word count across all content sections.
pub fn total_word_count(contents: &[CompileContent]) -> usize {
    contents.iter().map(|c| c.text.split_whitespace().count()).sum()
}

/// Escape special characters for XML output (epub, fdx, opml, mobi).
pub fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Escape special characters for HTML output.
pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Create a URL-safe slug from a title (for anchor links / TOC entries).
pub fn slug(title: &str) -> String {
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

/// Decode the five standard XML entities back to characters.
pub fn decode_xml_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}
