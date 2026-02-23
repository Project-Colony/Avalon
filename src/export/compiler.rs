use std::fmt::Write;
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

    /// Whether this format supports table of contents
    pub fn supports_toc(&self) -> bool {
        matches!(self, OutputFormat::Html | OutputFormat::Markdown | OutputFormat::Latex
            | OutputFormat::Pdf | OutputFormat::Docx | OutputFormat::Epub)
    }

    /// Whether this format supports front matter
    pub fn supports_front_matter(&self) -> bool {
        !matches!(self, OutputFormat::Opml)
    }

    /// Category label for grouping in UI
    pub fn category(&self) -> &str {
        match self {
            OutputFormat::PlainText | OutputFormat::Markdown => "Text",
            OutputFormat::Html | OutputFormat::Latex => "Markup",
            OutputFormat::Pdf | OutputFormat::Docx | OutputFormat::Epub => "Document",
            OutputFormat::Rtf => "Legacy",
            OutputFormat::Opml => "Outline",
            OutputFormat::Fountain => "Screenplay",
        }
    }

    /// Check if this format requires save_to_file (binary formats)
    pub fn is_binary(&self) -> bool {
        matches!(self, OutputFormat::Pdf | OutputFormat::Docx | OutputFormat::Epub)
    }

    /// MIME type for this format
    pub fn mime_type(&self) -> &str {
        match self {
            OutputFormat::PlainText => "text/plain",
            OutputFormat::Markdown => "text/markdown",
            OutputFormat::Html => "text/html",
            OutputFormat::Pdf => "application/pdf",
            OutputFormat::Latex => "application/x-latex",
            OutputFormat::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            OutputFormat::Epub => "application/epub+zip",
            OutputFormat::Rtf => "application/rtf",
            OutputFormat::Opml => "text/x-opml",
            OutputFormat::Fountain => "text/plain",
        }
    }

    /// Parse an OutputFormat from a file extension string
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().trim_start_matches('.') {
            "txt" | "text" => Some(OutputFormat::PlainText),
            "md" | "markdown" => Some(OutputFormat::Markdown),
            "html" | "htm" => Some(OutputFormat::Html),
            "pdf" => Some(OutputFormat::Pdf),
            "tex" | "latex" => Some(OutputFormat::Latex),
            "docx" => Some(OutputFormat::Docx),
            "epub" => Some(OutputFormat::Epub),
            "rtf" => Some(OutputFormat::Rtf),
            "opml" => Some(OutputFormat::Opml),
            "fountain" => Some(OutputFormat::Fountain),
            _ => None,
        }
    }

    /// Parse an OutputFormat from a file path
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(Self::from_extension)
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
                page_count: (total_words / 250).max(1),
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

    /// Check if this content item is empty
    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty() && !self.is_folder
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

    /// Summary of this content item
    pub fn summary(&self) -> String {
        let kind = if self.is_folder { "Folder" } else { "Document" };
        format!("{}: \"{}\" ({} words, depth {})", kind, self.title, self.word_count(), self.depth)
    }

    /// Compute heading level based on depth (h1 = depth 0, capped at h6)
    pub fn heading_level(&self) -> usize {
        (self.depth + 1).min(6)
    }
}

impl CompileOptions {
    /// Build options for a quick plain text export
    pub fn quick_text(title: &str) -> Self {
        Self {
            format: OutputFormat::PlainText,
            title: title.to_string(),
            include_front_matter: false,
            include_toc: false,
            replace_placeholders: false,
            compile_marked_only: false,
            ..Default::default()
        }
    }

    /// Build options for a manuscript-style export
    pub fn manuscript(title: &str, author: &str) -> Self {
        Self {
            format: OutputFormat::Markdown,
            title: title.to_string(),
            author: author.to_string(),
            include_front_matter: true,
            separator: SeparatorType::PageBreak,
            page_break_between_folders: true,
            compile_marked_only: true,
            include_toc: true,
            replace_placeholders: true,
            ..Default::default()
        }
    }

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

    /// Human-readable label
    pub fn label(&self) -> &str {
        match self {
            SeparatorType::EmptyLine => "Empty Line",
            SeparatorType::PageBreak => "Page Break",
            SeparatorType::SectionBreak => "Section Break (***)",
            SeparatorType::Custom(_) => "Custom",
            SeparatorType::None => "None",
        }
    }

    /// All standard separator types
    pub fn all_standard() -> Vec<Self> {
        vec![
            SeparatorType::EmptyLine,
            SeparatorType::PageBreak,
            SeparatorType::SectionBreak,
            SeparatorType::None,
        ]
    }
}

/// Statistics gathered from compiled content
#[derive(Debug, Clone)]
pub struct CompileStatistics {
    pub total_words: usize,
    pub total_chars: usize,
    pub total_paragraphs: usize,
    pub total_sentences: usize,
    pub section_count: usize,
    pub folder_count: usize,
    pub avg_words_per_section: f64,
    pub longest_section: Option<(String, usize)>,
    pub shortest_section: Option<(String, usize)>,
    pub estimated_pages: usize,
    pub estimated_reading_minutes: usize,
}

impl CompileStatistics {
    /// Compute statistics from a set of compile contents
    pub fn from_contents(contents: &[CompileContent]) -> Self {
        let sections: Vec<&CompileContent> = contents.iter().filter(|c| !c.is_folder).collect();
        let folders: Vec<&CompileContent> = contents.iter().filter(|c| c.is_folder).collect();

        let total_words: usize = sections.iter().map(|c| c.word_count()).sum();
        let total_chars: usize = sections.iter().map(|c| c.char_count()).sum();
        let total_paragraphs: usize = sections.iter().map(|c| c.paragraph_count()).sum();
        let total_sentences: usize = sections.iter().map(|c| c.sentence_count()).sum();

        let avg_words_per_section = if sections.is_empty() {
            0.0
        } else {
            total_words as f64 / sections.len() as f64
        };

        let longest_section = sections
            .iter()
            .max_by_key(|c| c.word_count())
            .map(|c| (c.title.clone(), c.word_count()));

        let shortest_section = sections
            .iter()
            .filter(|c| c.word_count() > 0)
            .min_by_key(|c| c.word_count())
            .map(|c| (c.title.clone(), c.word_count()));

        Self {
            total_words,
            total_chars,
            total_paragraphs,
            total_sentences,
            section_count: sections.len(),
            folder_count: folders.len(),
            avg_words_per_section,
            longest_section,
            shortest_section,
            estimated_pages: if total_words == 0 { 0 } else { (total_words / 250).max(1) },
            estimated_reading_minutes: if total_words == 0 { 0 } else { (total_words / 200).max(1) },
        }
    }

    /// Format as a compact summary string
    pub fn summary(&self) -> String {
        format!(
            "{} words, {} pages, ~{} min read ({} sections, {} folders)",
            self.total_words,
            self.estimated_pages,
            self.estimated_reading_minutes,
            self.section_count,
            self.folder_count,
        )
    }
}

/// A structured compile plan describing the assembled output
#[derive(Debug, Clone)]
pub struct CompileManifest {
    pub title: String,
    pub author: String,
    pub format: OutputFormat,
    pub sections: Vec<ManifestEntry>,
    pub total_words: usize,
    pub total_sections: usize,
    pub total_folders: usize,
}

/// An entry in the compile manifest describing one section
#[derive(Debug, Clone)]
pub struct ManifestEntry {
    pub title: String,
    pub depth: usize,
    pub heading_level: usize,
    pub is_folder: bool,
    pub word_count: usize,
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
                depth: c.depth,
                heading_level,
                is_folder: c.is_folder,
                word_count: c.word_count(),
                has_page_break_before,
            });

            prev_was_folder_at_depth_0 = c.is_folder && c.depth == 0;
        }

        let total_words = contents.iter().filter(|c| !c.is_folder).map(|c| c.word_count()).sum();
        let total_sections = contents.iter().filter(|c| !c.is_folder).count();
        let total_folders = contents.iter().filter(|c| c.is_folder).count();

        Self {
            title: options.title.clone(),
            author: options.author.clone(),
            format: options.format.clone(),
            sections,
            total_words,
            total_sections,
            total_folders,
        }
    }

    /// Generate a textual outline of the compile plan
    pub fn outline(&self) -> String {
        let mut out = String::new();
        for entry in &self.sections {
            let indent = "  ".repeat(entry.depth);
            let kind = if entry.is_folder { "+" } else { "-" };
            let pb = if entry.has_page_break_before { " [PAGE BREAK]" } else { "" };
            let _ = writeln!(out,
                "{}{} {} ({} words){}",
                indent, kind, entry.title, entry.word_count, pb
            );
        }
        out
    }

    /// Get word counts grouped by top-level section (depth 0 folders)
    pub fn word_count_by_chapter(&self) -> Vec<(String, usize)> {
        let mut chapters: Vec<(String, usize)> = Vec::new();
        let mut current_chapter: Option<(String, usize)> = None;

        for entry in &self.sections {
            if entry.is_folder && entry.depth == 0 {
                if let Some(ch) = current_chapter.take() {
                    chapters.push(ch);
                }
                current_chapter = Some((entry.title.clone(), 0));
            } else if let Some(ref mut ch) = current_chapter {
                ch.1 += entry.word_count;
            } else {
                // Content before any folder
                if chapters.is_empty() && !entry.is_folder {
                    chapters.push((entry.title.clone(), entry.word_count));
                }
            }
        }
        if let Some(ch) = current_chapter {
            chapters.push(ch);
        }
        chapters
    }

    /// Entries that have page breaks
    pub fn page_break_count(&self) -> usize {
        self.sections.iter().filter(|s| s.has_page_break_before).count()
    }

    /// Summary of the manifest
    pub fn summary(&self) -> String {
        format!(
            "\"{}\" by {} — {} format, {} words, {} sections, {} folders, {} page breaks",
            self.title, self.author, self.format.display_name(),
            self.total_words, self.total_sections, self.total_folders,
            self.page_break_count(),
        )
    }
}

/// Assembles document sections into a single output string with proper
/// separators, headings, and page breaks.
pub struct SectionAssembler;

impl SectionAssembler {
    /// Assemble contents into a single text with headings and separators
    pub fn assemble(contents: &[CompileContent], options: &CompileOptions) -> String {
        let mut output = String::new();
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_output_format_extension() {
        assert_eq!(OutputFormat::PlainText.extension(), "txt");
        assert_eq!(OutputFormat::Markdown.extension(), "md");
        assert_eq!(OutputFormat::Html.extension(), "html");
        assert_eq!(OutputFormat::Pdf.extension(), "pdf");
        assert_eq!(OutputFormat::Latex.extension(), "tex");
        assert_eq!(OutputFormat::Docx.extension(), "docx");
        assert_eq!(OutputFormat::Epub.extension(), "epub");
    }

    #[test]
    fn test_output_format_display_name() {
        assert_eq!(OutputFormat::PlainText.display_name(), "Plain Text");
        assert_eq!(OutputFormat::Html.display_name(), "HTML");
        assert_eq!(OutputFormat::Docx.display_name(), "Word (DOCX)");
    }

    #[test]
    fn test_all_formats() {
        let all = OutputFormat::all();
        assert_eq!(all.len(), 10);
    }

    #[test]
    fn test_is_binary() {
        assert!(OutputFormat::Pdf.is_binary());
        assert!(OutputFormat::Docx.is_binary());
        assert!(OutputFormat::Epub.is_binary());
        assert!(!OutputFormat::Html.is_binary());
        assert!(!OutputFormat::Markdown.is_binary());
    }

    #[test]
    fn test_mime_types() {
        assert_eq!(OutputFormat::Html.mime_type(), "text/html");
        assert_eq!(OutputFormat::Pdf.mime_type(), "application/pdf");
        assert_eq!(OutputFormat::Markdown.mime_type(), "text/markdown");
    }

    #[test]
    fn test_compile_options_default() {
        let opts = CompileOptions::default();
        assert_eq!(opts.format, OutputFormat::Markdown);
        assert!(opts.include_front_matter);
        assert!(opts.compile_marked_only);
        assert_eq!(opts.font_size, 12.0);
    }

    #[test]
    fn test_compile_content() {
        let content = CompileContent {
            title: "Test Chapter".to_string(),
            text: "Hello world this is content".to_string(),
            depth: 0,
            is_folder: false,
        };
        assert_eq!(content.word_count(), 5);
        assert!(!content.is_empty());
    }

    #[test]
    fn test_compile_content_empty() {
        let content = CompileContent {
            title: "Empty".to_string(),
            text: "  \n  ".to_string(),
            depth: 0,
            is_folder: false,
        };
        assert!(content.is_empty());
    }

    #[test]
    fn test_separator_type_labels() {
        assert_eq!(SeparatorType::EmptyLine.label(), "Empty Line");
        assert_eq!(SeparatorType::PageBreak.label(), "Page Break");
        assert_eq!(SeparatorType::None.label(), "None");
    }

    #[test]
    fn test_separator_all_standard() {
        let all = SeparatorType::all_standard();
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn test_collect_contents() {
        let mut binder = Binder::default_structure();
        let mut item1 = BinderItem::new_text("Chapter 1");
        if let Some(ref mut doc) = item1.document {
            doc.content = "Hello world".to_string();
        }
        item1.include_in_compile = true;
        binder.draft.add_child(item1);

        let mut item2 = BinderItem::new_text("Chapter 2");
        if let Some(ref mut doc) = item2.document {
            doc.content = "Second chapter".to_string();
        }
        item2.include_in_compile = true;
        binder.draft.add_child(item2);

        let opts = CompileOptions::default();
        let contents = Compiler::collect_contents(&binder.draft, &opts);
        // Draft folder + 2 chapters
        assert!(contents.len() >= 2);
    }

    #[test]
    fn test_compile_plain_text() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Test");
        if let Some(ref mut doc) = item.document {
            doc.content = "Hello world".to_string();
        }
        item.include_in_compile = true;
        binder.draft.add_child(item);

        let mut opts = CompileOptions::default();
        opts.format = OutputFormat::PlainText;
        opts.include_front_matter = false;
        opts.include_toc = false;
        opts.replace_placeholders = false;

        let result = Compiler::compile(&binder, &opts);
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("Hello world"));
    }

    #[test]
    fn test_compile_markdown() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Chapter One");
        if let Some(ref mut doc) = item.document {
            doc.content = "The story begins.".to_string();
        }
        item.include_in_compile = true;
        binder.draft.add_child(item);

        let mut opts = CompileOptions::default();
        opts.format = OutputFormat::Markdown;
        opts.title = "My Book".to_string();
        opts.include_front_matter = true;
        opts.include_toc = false;
        opts.replace_placeholders = false;

        let result = Compiler::compile(&binder, &opts);
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("The story begins"));
    }

    #[test]
    fn test_compile_html() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Scene");
        if let Some(ref mut doc) = item.document {
            doc.content = "Some text here.".to_string();
        }
        item.include_in_compile = true;
        binder.draft.add_child(item);

        let mut opts = CompileOptions::default();
        opts.format = OutputFormat::Html;
        opts.include_toc = false;
        opts.replace_placeholders = false;

        let result = Compiler::compile(&binder, &opts);
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("<") && html.contains(">"));
    }

    #[test]
    fn test_output_format_rtf_extension() {
        assert_eq!(OutputFormat::Rtf.extension(), "rtf");
        assert_eq!(OutputFormat::Opml.extension(), "opml");
        assert_eq!(OutputFormat::Fountain.extension(), "fountain");
    }

    #[test]
    fn test_output_format_all_display_names() {
        for fmt in OutputFormat::all() {
            let name = fmt.display_name();
            assert!(!name.is_empty(), "Display name for {:?} is empty", fmt);
        }
    }

    #[test]
    fn test_output_format_all_extensions() {
        for fmt in OutputFormat::all() {
            let ext = fmt.extension();
            assert!(!ext.is_empty());
            assert!(!ext.contains('.'), "Extension should not contain dot");
        }
    }

    #[test]
    fn test_output_format_all_mime_types() {
        for fmt in OutputFormat::all() {
            let mime = fmt.mime_type();
            assert!(mime.contains('/'), "MIME type should contain /: {}", mime);
        }
    }

    #[test]
    fn test_is_binary_text_formats() {
        assert!(!OutputFormat::PlainText.is_binary());
        assert!(!OutputFormat::Markdown.is_binary());
        assert!(!OutputFormat::Html.is_binary());
        assert!(!OutputFormat::Latex.is_binary());
        assert!(!OutputFormat::Rtf.is_binary());
        assert!(!OutputFormat::Opml.is_binary());
        assert!(!OutputFormat::Fountain.is_binary());
    }

    #[test]
    fn test_compile_content_word_count() {
        let content = CompileContent {
            title: "Title".to_string(),
            text: "one two three four five".to_string(),
            depth: 0,
            is_folder: false,
        };
        assert_eq!(content.word_count(), 5);
        assert_eq!(content.char_count(), "one two three four five".len());
    }

    #[test]
    fn test_compile_content_folder_not_empty() {
        let content = CompileContent {
            title: "Folder".to_string(),
            text: String::new(),
            depth: 0,
            is_folder: true,
        };
        // Folders with empty text are not considered "empty" because is_folder is true
        assert!(!content.is_empty());
    }

    #[test]
    fn test_separator_type_custom_label() {
        let sep = SeparatorType::Custom("---".to_string());
        assert_eq!(sep.label(), "Custom");
    }

    #[test]
    fn test_separator_type_section_break_label() {
        assert_eq!(SeparatorType::SectionBreak.label(), "Section Break (***)");
    }

    #[test]
    fn test_compile_options_default_values() {
        let opts = CompileOptions::default();
        assert_eq!(opts.font_family, "Times New Roman");
        assert!(opts.replace_placeholders);
        assert!(!opts.include_toc);
        assert!(opts.page_break_between_folders);
    }

    #[test]
    fn test_compile_marked_only_filters() {
        let mut binder = Binder::default_structure();

        let mut item1 = BinderItem::new_text("Included");
        if let Some(ref mut doc) = item1.document {
            doc.content = "This is included.".to_string();
        }
        item1.include_in_compile = true;
        binder.draft.add_child(item1);

        let mut item2 = BinderItem::new_text("Excluded");
        if let Some(ref mut doc) = item2.document {
            doc.content = "This is excluded.".to_string();
        }
        item2.include_in_compile = false;
        binder.draft.add_child(item2);

        let mut opts = CompileOptions::default();
        opts.format = OutputFormat::PlainText;
        opts.compile_marked_only = true;
        opts.include_front_matter = false;
        opts.include_toc = false;
        opts.replace_placeholders = false;

        let result = Compiler::compile(&binder, &opts).unwrap();
        assert!(result.contains("included"));
        assert!(!result.contains("excluded"));
    }

    #[test]
    fn test_compile_all_items_when_not_marked_only() {
        let mut binder = Binder::default_structure();

        let mut item1 = BinderItem::new_text("First");
        if let Some(ref mut doc) = item1.document {
            doc.content = "First content.".to_string();
        }
        item1.include_in_compile = true;
        binder.draft.add_child(item1);

        let mut item2 = BinderItem::new_text("Second");
        if let Some(ref mut doc) = item2.document {
            doc.content = "Second content.".to_string();
        }
        item2.include_in_compile = false;
        binder.draft.add_child(item2);

        let mut opts = CompileOptions::default();
        opts.format = OutputFormat::PlainText;
        opts.compile_marked_only = false;
        opts.include_front_matter = false;
        opts.include_toc = false;
        opts.replace_placeholders = false;

        let result = Compiler::compile(&binder, &opts).unwrap();
        assert!(result.contains("First content"));
        assert!(result.contains("Second content"));
    }

    #[test]
    fn test_compile_empty_binder() {
        let binder = Binder::default_structure();
        let mut opts = CompileOptions::default();
        opts.format = OutputFormat::PlainText;
        opts.include_front_matter = false;
        opts.include_toc = false;
        opts.replace_placeholders = false;
        opts.compile_marked_only = false;

        let result = Compiler::compile(&binder, &opts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_latex() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Chapter");
        if let Some(ref mut doc) = item.document {
            doc.content = "LaTeX content here.".to_string();
        }
        item.include_in_compile = true;
        binder.draft.add_child(item);

        let mut opts = CompileOptions::default();
        opts.format = OutputFormat::Latex;
        opts.include_front_matter = false;
        opts.include_toc = false;
        opts.replace_placeholders = false;

        let result = Compiler::compile(&binder, &opts);
        assert!(result.is_ok());
        let tex = result.unwrap();
        assert!(tex.contains("LaTeX content"));
    }

    #[test]
    fn test_compile_rtf() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Chapter");
        if let Some(ref mut doc) = item.document {
            doc.content = "RTF output.".to_string();
        }
        item.include_in_compile = true;
        binder.draft.add_child(item);

        let mut opts = CompileOptions::default();
        opts.format = OutputFormat::Rtf;
        opts.include_front_matter = false;
        opts.include_toc = false;
        opts.replace_placeholders = false;

        let result = Compiler::compile(&binder, &opts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_binary_formats_message() {
        let binder = Binder::default_structure();
        let mut opts = CompileOptions::default();
        opts.include_toc = false;
        opts.replace_placeholders = false;
        opts.compile_marked_only = false;

        for fmt in [OutputFormat::Pdf, OutputFormat::Docx, OutputFormat::Epub] {
            opts.format = fmt;
            let result = Compiler::compile(&binder, &opts).unwrap();
            assert!(result.contains("requires save_to_file"));
        }
    }

    #[test]
    fn test_compile_with_toc() {
        let mut binder = Binder::default_structure();
        let mut ch1 = BinderItem::new_folder("Chapter One");
        ch1.include_in_compile = true;
        let mut scene = BinderItem::new_text("Scene 1");
        if let Some(ref mut doc) = scene.document {
            doc.content = "Text of scene one.".to_string();
        }
        scene.include_in_compile = true;
        ch1.add_child(scene);
        binder.draft.add_child(ch1);

        let mut opts = CompileOptions::default();
        opts.format = OutputFormat::PlainText;
        opts.include_front_matter = true;
        opts.title = "My Book".to_string();
        opts.include_toc = true;
        opts.replace_placeholders = false;

        let result = Compiler::compile(&binder, &opts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_with_placeholders() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Chapter");
        if let Some(ref mut doc) = item.document {
            doc.content = "Word count: <$wc>".to_string();
        }
        item.include_in_compile = true;
        binder.draft.add_child(item);

        let mut opts = CompileOptions::default();
        opts.format = OutputFormat::PlainText;
        opts.include_front_matter = false;
        opts.include_toc = false;
        opts.replace_placeholders = true;
        opts.title = "Test".to_string();

        let result = Compiler::compile(&binder, &opts);
        assert!(result.is_ok());
    }

    // New CompileContent tests

    #[test]
    fn test_compile_content_paragraph_count() {
        let content = CompileContent {
            title: "Test".to_string(),
            text: "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.".to_string(),
            depth: 0,
            is_folder: false,
        };
        assert_eq!(content.paragraph_count(), 3);
    }

    #[test]
    fn test_compile_content_sentence_count() {
        let content = CompileContent {
            title: "Test".to_string(),
            text: "First sentence. Second sentence! Third sentence?".to_string(),
            depth: 0,
            is_folder: false,
        };
        assert_eq!(content.sentence_count(), 3);
    }

    #[test]
    fn test_compile_content_summary() {
        let content = CompileContent {
            title: "Chapter 1".to_string(),
            text: "Hello world".to_string(),
            depth: 1,
            is_folder: false,
        };
        let summary = content.summary();
        assert!(summary.contains("Document"));
        assert!(summary.contains("Chapter 1"));
        assert!(summary.contains("2 words"));
        assert!(summary.contains("depth 1"));
    }

    #[test]
    fn test_compile_content_summary_folder() {
        let content = CompileContent {
            title: "Part One".to_string(),
            text: String::new(),
            depth: 0,
            is_folder: true,
        };
        assert!(content.summary().contains("Folder"));
    }

    // CompileOptions builder tests

    #[test]
    fn test_quick_text_options() {
        let opts = CompileOptions::quick_text("My Story");
        assert_eq!(opts.format, OutputFormat::PlainText);
        assert_eq!(opts.title, "My Story");
        assert!(!opts.include_front_matter);
        assert!(!opts.include_toc);
        assert!(!opts.replace_placeholders);
        assert!(!opts.compile_marked_only);
    }

    #[test]
    fn test_manuscript_options() {
        let opts = CompileOptions::manuscript("My Novel", "Author");
        assert_eq!(opts.format, OutputFormat::Markdown);
        assert_eq!(opts.title, "My Novel");
        assert_eq!(opts.author, "Author");
        assert!(opts.include_front_matter);
        assert!(opts.include_toc);
        assert!(opts.compile_marked_only);
    }

    #[test]
    fn test_validate_options_valid() {
        let mut opts = CompileOptions::default();
        opts.title = "My Book".to_string();
        let issues = opts.validate();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_validate_options_empty_title_with_front_matter() {
        let opts = CompileOptions::default(); // title is empty, front matter is true
        let issues = opts.validate();
        assert!(issues.iter().any(|i| i.contains("Title is empty")));
    }

    #[test]
    fn test_validate_options_bad_font_size() {
        let mut opts = CompileOptions::default();
        opts.title = "Book".to_string();
        opts.font_size = 200.0;
        let issues = opts.validate();
        assert!(issues.iter().any(|i| i.contains("Font size")));
    }

    #[test]
    fn test_settings_summary() {
        let opts = CompileOptions::manuscript("Book", "Author");
        let summary = opts.settings_summary();
        assert!(summary.contains("Markdown"));
        assert!(summary.contains("With front matter"));
        assert!(summary.contains("With TOC"));
    }

    // OutputFormat new method tests

    #[test]
    fn test_format_supports_toc() {
        assert!(OutputFormat::Html.supports_toc());
        assert!(OutputFormat::Markdown.supports_toc());
        assert!(OutputFormat::Latex.supports_toc());
        assert!(OutputFormat::Pdf.supports_toc());
        assert!(!OutputFormat::PlainText.supports_toc());
        assert!(!OutputFormat::Rtf.supports_toc());
        assert!(!OutputFormat::Fountain.supports_toc());
    }

    #[test]
    fn test_format_supports_front_matter() {
        assert!(OutputFormat::Html.supports_front_matter());
        assert!(OutputFormat::Markdown.supports_front_matter());
        assert!(!OutputFormat::Opml.supports_front_matter());
    }

    #[test]
    fn test_format_category() {
        assert_eq!(OutputFormat::PlainText.category(), "Text");
        assert_eq!(OutputFormat::Markdown.category(), "Text");
        assert_eq!(OutputFormat::Html.category(), "Markup");
        assert_eq!(OutputFormat::Pdf.category(), "Document");
        assert_eq!(OutputFormat::Fountain.category(), "Screenplay");
        assert_eq!(OutputFormat::Opml.category(), "Outline");
        assert_eq!(OutputFormat::Rtf.category(), "Legacy");
    }

    // SeparatorType tests

    #[test]
    fn test_separator_string() {
        assert_eq!(SeparatorType::EmptyLine.separator_string(), "\n\n");
        assert!(SeparatorType::PageBreak.separator_string().contains("---"));
        assert!(SeparatorType::SectionBreak.separator_string().contains("***"));
        assert_eq!(SeparatorType::None.separator_string(), "");
    }

    #[test]
    fn test_separator_custom_string() {
        let sep = SeparatorType::Custom("~~~".to_string());
        assert_eq!(sep.separator_string(), "~~~");
    }

    // Compile with different formats

    #[test]
    fn test_compile_fountain() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Scene");
        if let Some(ref mut doc) = item.document {
            doc.content = "INT. OFFICE - DAY\n\nJOHN enters.".to_string();
        }
        item.include_in_compile = true;
        binder.draft.add_child(item);

        let mut opts = CompileOptions::default();
        opts.format = OutputFormat::Fountain;
        opts.include_toc = false;
        opts.replace_placeholders = false;
        opts.include_front_matter = false;

        let result = Compiler::compile(&binder, &opts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_opml() {
        let mut binder = Binder::default_structure();
        let mut item = BinderItem::new_text("Note");
        if let Some(ref mut doc) = item.document {
            doc.content = "Some notes.".to_string();
        }
        item.include_in_compile = true;
        binder.draft.add_child(item);

        let mut opts = CompileOptions::default();
        opts.format = OutputFormat::Opml;
        opts.include_toc = false;
        opts.replace_placeholders = false;
        opts.compile_marked_only = false;

        let result = Compiler::compile(&binder, &opts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_with_nested_folders() {
        let mut binder = Binder::default_structure();

        let mut folder = BinderItem::new_folder("Part One");
        folder.include_in_compile = true;

        let mut ch1 = BinderItem::new_text("Chapter 1");
        if let Some(ref mut doc) = ch1.document {
            doc.content = "Content of chapter 1.".to_string();
        }
        ch1.include_in_compile = true;

        let mut ch2 = BinderItem::new_text("Chapter 2");
        if let Some(ref mut doc) = ch2.document {
            doc.content = "Content of chapter 2.".to_string();
        }
        ch2.include_in_compile = true;

        folder.add_child(ch1);
        folder.add_child(ch2);
        binder.draft.add_child(folder);

        let mut opts = CompileOptions::default();
        opts.format = OutputFormat::PlainText;
        opts.include_front_matter = false;
        opts.include_toc = false;
        opts.replace_placeholders = false;

        let result = Compiler::compile(&binder, &opts).unwrap();
        assert!(result.contains("chapter 1"));
        assert!(result.contains("chapter 2"));
    }

    #[test]
    fn test_output_format_equality() {
        assert_eq!(OutputFormat::Pdf, OutputFormat::Pdf);
        assert_ne!(OutputFormat::Pdf, OutputFormat::Html);
    }

    // ---- New: CompileStatistics tests ----

    #[test]
    fn test_compile_statistics_basic() {
        let contents = vec![
            CompileContent { title: "Ch1".into(), text: "Hello world foo bar.".into(), depth: 0, is_folder: false },
            CompileContent { title: "Ch2".into(), text: "Second chapter here.".into(), depth: 0, is_folder: false },
            CompileContent { title: "Part".into(), text: String::new(), depth: 0, is_folder: true },
        ];
        let stats = CompileStatistics::from_contents(&contents);
        assert_eq!(stats.section_count, 2);
        assert_eq!(stats.folder_count, 1);
        assert_eq!(stats.total_words, 7);
        assert!(stats.total_chars > 0);
        assert!(stats.avg_words_per_section > 0.0);
    }

    #[test]
    fn test_compile_statistics_empty() {
        let contents: Vec<CompileContent> = vec![];
        let stats = CompileStatistics::from_contents(&contents);
        assert_eq!(stats.total_words, 0);
        assert_eq!(stats.section_count, 0);
        assert_eq!(stats.folder_count, 0);
        assert_eq!(stats.estimated_pages, 0);
        assert_eq!(stats.estimated_reading_minutes, 0);
        assert!(stats.longest_section.is_none());
        assert!(stats.shortest_section.is_none());
    }

    #[test]
    fn test_compile_statistics_longest_shortest() {
        let contents = vec![
            CompileContent { title: "Short".into(), text: "Two words.".into(), depth: 0, is_folder: false },
            CompileContent { title: "Long".into(), text: "This is a much longer section with many more words in it.".into(), depth: 0, is_folder: false },
        ];
        let stats = CompileStatistics::from_contents(&contents);
        let (longest_title, _) = stats.longest_section.unwrap();
        assert_eq!(longest_title, "Long");
        let (shortest_title, _) = stats.shortest_section.unwrap();
        assert_eq!(shortest_title, "Short");
    }

    #[test]
    fn test_compile_statistics_summary() {
        let contents = vec![
            CompileContent { title: "Scene".into(), text: "Hello world.".into(), depth: 0, is_folder: false },
        ];
        let stats = CompileStatistics::from_contents(&contents);
        let summary = stats.summary();
        assert!(summary.contains("2 words"));
        assert!(summary.contains("1 sections"));
    }

    #[test]
    fn test_compile_statistics_pages_and_reading_time() {
        let long_text = "word ".repeat(1000);
        let contents = vec![
            CompileContent { title: "Big".into(), text: long_text, depth: 0, is_folder: false },
        ];
        let stats = CompileStatistics::from_contents(&contents);
        assert_eq!(stats.estimated_pages, 4); // 1000 / 250
        assert_eq!(stats.estimated_reading_minutes, 5); // 1000 / 200
    }

    // ---- New: OutputFormat::from_extension / from_path tests ----

    #[test]
    fn test_format_from_extension() {
        assert_eq!(OutputFormat::from_extension("txt"), Some(OutputFormat::PlainText));
        assert_eq!(OutputFormat::from_extension("text"), Some(OutputFormat::PlainText));
        assert_eq!(OutputFormat::from_extension("md"), Some(OutputFormat::Markdown));
        assert_eq!(OutputFormat::from_extension("markdown"), Some(OutputFormat::Markdown));
        assert_eq!(OutputFormat::from_extension("html"), Some(OutputFormat::Html));
        assert_eq!(OutputFormat::from_extension("htm"), Some(OutputFormat::Html));
        assert_eq!(OutputFormat::from_extension("pdf"), Some(OutputFormat::Pdf));
        assert_eq!(OutputFormat::from_extension("tex"), Some(OutputFormat::Latex));
        assert_eq!(OutputFormat::from_extension("latex"), Some(OutputFormat::Latex));
        assert_eq!(OutputFormat::from_extension("docx"), Some(OutputFormat::Docx));
        assert_eq!(OutputFormat::from_extension("epub"), Some(OutputFormat::Epub));
        assert_eq!(OutputFormat::from_extension("rtf"), Some(OutputFormat::Rtf));
        assert_eq!(OutputFormat::from_extension("opml"), Some(OutputFormat::Opml));
        assert_eq!(OutputFormat::from_extension("fountain"), Some(OutputFormat::Fountain));
    }

    #[test]
    fn test_format_from_extension_case_insensitive() {
        assert_eq!(OutputFormat::from_extension("TXT"), Some(OutputFormat::PlainText));
        assert_eq!(OutputFormat::from_extension("MD"), Some(OutputFormat::Markdown));
        assert_eq!(OutputFormat::from_extension("Html"), Some(OutputFormat::Html));
    }

    #[test]
    fn test_format_from_extension_with_dot() {
        assert_eq!(OutputFormat::from_extension(".pdf"), Some(OutputFormat::Pdf));
        assert_eq!(OutputFormat::from_extension(".md"), Some(OutputFormat::Markdown));
    }

    #[test]
    fn test_format_from_extension_unknown() {
        assert_eq!(OutputFormat::from_extension("xyz"), None);
        assert_eq!(OutputFormat::from_extension(""), None);
        assert_eq!(OutputFormat::from_extension("jpg"), None);
    }

    #[test]
    fn test_format_from_path() {
        let path = PathBuf::from("/some/file.md");
        assert_eq!(OutputFormat::from_path(&path), Some(OutputFormat::Markdown));

        let path = PathBuf::from("document.pdf");
        assert_eq!(OutputFormat::from_path(&path), Some(OutputFormat::Pdf));

        let path = PathBuf::from("output.html");
        assert_eq!(OutputFormat::from_path(&path), Some(OutputFormat::Html));
    }

    #[test]
    fn test_format_from_path_no_extension() {
        let path = PathBuf::from("noext");
        assert_eq!(OutputFormat::from_path(&path), None);
    }

    #[test]
    fn test_format_from_path_unknown_ext() {
        let path = PathBuf::from("image.png");
        assert_eq!(OutputFormat::from_path(&path), None);
    }

    #[test]
    fn test_format_roundtrip() {
        for fmt in OutputFormat::all() {
            let ext = fmt.extension();
            let parsed = OutputFormat::from_extension(ext).unwrap();
            assert_eq!(parsed, fmt, "Roundtrip failed for {:?}", fmt);
        }
    }

    // ---- Compile Manifest tests ----

    fn sample_contents() -> Vec<CompileContent> {
        vec![
            CompileContent { title: "Part One".into(), text: String::new(), depth: 0, is_folder: true },
            CompileContent { title: "Chapter 1".into(), text: "The hero set out on the journey.".into(), depth: 1, is_folder: false },
            CompileContent { title: "Chapter 2".into(), text: "The villain appeared from the shadows of the old castle.".into(), depth: 1, is_folder: false },
            CompileContent { title: "Part Two".into(), text: String::new(), depth: 0, is_folder: true },
            CompileContent { title: "Chapter 3".into(), text: "A battle ensued between the hero and the villain.".into(), depth: 1, is_folder: false },
        ]
    }

    #[test]
    fn test_compile_manifest_from_contents() {
        let contents = sample_contents();
        let opts = CompileOptions::default();
        let manifest = CompileManifest::from_contents(&contents, &opts);

        assert_eq!(manifest.sections.len(), 5);
        assert_eq!(manifest.total_sections, 3); // 3 text items
        assert_eq!(manifest.total_folders, 2);  // 2 folders
        assert!(manifest.total_words > 0);
    }

    #[test]
    fn test_compile_manifest_heading_levels() {
        let contents = sample_contents();
        let opts = CompileOptions::default();
        let manifest = CompileManifest::from_contents(&contents, &opts);

        // Part One at depth 0 -> heading level 1
        assert_eq!(manifest.sections[0].heading_level, 1);
        // Chapter 1 at depth 1 -> heading level 2
        assert_eq!(manifest.sections[1].heading_level, 2);
    }

    #[test]
    fn test_compile_manifest_page_breaks() {
        let contents = sample_contents();
        let mut opts = CompileOptions::default();
        opts.page_break_between_folders = true;
        let manifest = CompileManifest::from_contents(&contents, &opts);

        // First section should never have page break
        assert!(!manifest.sections[0].has_page_break_before);
        // Part Two (second folder at depth 0) should have page break
        assert!(manifest.sections[3].has_page_break_before);
    }

    #[test]
    fn test_compile_manifest_outline() {
        let contents = sample_contents();
        let opts = CompileOptions::default();
        let manifest = CompileManifest::from_contents(&contents, &opts);
        let outline = manifest.outline();

        assert!(outline.contains("+ Part One"));
        assert!(outline.contains("  - Chapter 1"));
        assert!(outline.contains("  - Chapter 2"));
        assert!(outline.contains("+ Part Two"));
        assert!(outline.contains("  - Chapter 3"));
    }

    #[test]
    fn test_compile_manifest_word_count_by_chapter() {
        let contents = sample_contents();
        let opts = CompileOptions::default();
        let manifest = CompileManifest::from_contents(&contents, &opts);
        let chapters = manifest.word_count_by_chapter();

        assert_eq!(chapters.len(), 2); // Part One and Part Two
        assert_eq!(chapters[0].0, "Part One");
        assert!(chapters[0].1 > 0); // Sum of Ch1 + Ch2 words
        assert_eq!(chapters[1].0, "Part Two");
        assert!(chapters[1].1 > 0); // Sum of Ch3 words
    }

    #[test]
    fn test_compile_manifest_summary() {
        let contents = sample_contents();
        let opts = CompileOptions::default();
        let manifest = CompileManifest::from_contents(&contents, &opts);
        let summary = manifest.summary();

        assert!(summary.contains("Markdown"));
        assert!(summary.contains("3 sections"));
        assert!(summary.contains("2 folders"));
    }

    #[test]
    fn test_compile_manifest_empty() {
        let contents: Vec<CompileContent> = vec![];
        let opts = CompileOptions::default();
        let manifest = CompileManifest::from_contents(&contents, &opts);

        assert_eq!(manifest.total_words, 0);
        assert_eq!(manifest.total_sections, 0);
        assert_eq!(manifest.total_folders, 0);
        assert!(manifest.outline().is_empty());
        assert!(manifest.word_count_by_chapter().is_empty());
    }

    // ---- SectionAssembler tests ----

    #[test]
    fn test_section_assembler_plain_text() {
        let contents = vec![
            CompileContent { title: "Scene 1".into(), text: "The morning arrived.".into(), depth: 0, is_folder: false },
            CompileContent { title: "Scene 2".into(), text: "The hero departed.".into(), depth: 0, is_folder: false },
        ];
        let mut opts = CompileOptions::default();
        opts.format = OutputFormat::PlainText;
        let result = SectionAssembler::assemble(&contents, &opts);

        assert!(result.contains("SCENE 1"));
        assert!(result.contains("The morning arrived."));
        assert!(result.contains("SCENE 2"));
        assert!(result.contains("The hero departed."));
    }

    #[test]
    fn test_section_assembler_markdown_headings() {
        let contents = vec![
            CompileContent { title: "Part One".into(), text: String::new(), depth: 0, is_folder: true },
            CompileContent { title: "Chapter 1".into(), text: "Content here.".into(), depth: 1, is_folder: false },
        ];
        let mut opts = CompileOptions::default();
        opts.format = OutputFormat::Markdown;
        let result = SectionAssembler::assemble(&contents, &opts);

        assert!(result.contains("# Part One"));
        assert!(result.contains("## Chapter 1"));
        assert!(result.contains("Content here."));
    }

    #[test]
    fn test_section_assembler_html_headings() {
        let contents = vec![
            CompileContent { title: "Title".into(), text: String::new(), depth: 0, is_folder: true },
            CompileContent { title: "Sub".into(), text: "Text.".into(), depth: 1, is_folder: false },
        ];
        let mut opts = CompileOptions::default();
        opts.format = OutputFormat::Html;
        let result = SectionAssembler::assemble(&contents, &opts);

        assert!(result.contains("<h1>Title</h1>"));
        assert!(result.contains("<h2>Sub</h2>"));
    }

    #[test]
    fn test_section_assembler_latex_headings() {
        let contents = vec![
            CompileContent { title: "Introduction".into(), text: String::new(), depth: 0, is_folder: true },
            CompileContent { title: "Subsec".into(), text: "Body.".into(), depth: 2, is_folder: false },
        ];
        let mut opts = CompileOptions::default();
        opts.format = OutputFormat::Latex;
        let result = SectionAssembler::assemble(&contents, &opts);

        assert!(result.contains("\\chapter{Introduction}"));
        assert!(result.contains("\\subsection{Subsec}"));
    }

    #[test]
    fn test_section_assembler_page_breaks() {
        let contents = vec![
            CompileContent { title: "Part 1".into(), text: String::new(), depth: 0, is_folder: true },
            CompileContent { title: "Ch1".into(), text: "First.".into(), depth: 1, is_folder: false },
            CompileContent { title: "Part 2".into(), text: String::new(), depth: 0, is_folder: true },
            CompileContent { title: "Ch2".into(), text: "Second.".into(), depth: 1, is_folder: false },
        ];
        let mut opts = CompileOptions::default();
        opts.format = OutputFormat::Markdown;
        opts.page_break_between_folders = true;
        let result = SectionAssembler::assemble(&contents, &opts);

        // Should contain page break before Part 2
        assert!(result.contains("---"));
    }

    #[test]
    fn test_section_assembler_empty() {
        let contents: Vec<CompileContent> = vec![];
        let opts = CompileOptions::default();
        let result = SectionAssembler::assemble(&contents, &opts);
        assert!(result.is_empty());
    }

    // ---- CompileContent heading_level tests ----

    #[test]
    fn test_compile_content_heading_level() {
        let c0 = CompileContent { title: "A".into(), text: String::new(), depth: 0, is_folder: false };
        assert_eq!(c0.heading_level(), 1);

        let c3 = CompileContent { title: "B".into(), text: String::new(), depth: 3, is_folder: false };
        assert_eq!(c3.heading_level(), 4);

        // Capped at 6
        let c10 = CompileContent { title: "C".into(), text: String::new(), depth: 10, is_folder: false };
        assert_eq!(c10.heading_level(), 6);
    }

    #[test]
    fn test_manifest_page_break_count() {
        let contents = sample_contents();
        let mut opts = CompileOptions::default();
        opts.page_break_between_folders = true;
        let manifest = CompileManifest::from_contents(&contents, &opts);
        assert!(manifest.page_break_count() >= 1);
    }
}
