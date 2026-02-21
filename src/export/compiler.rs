use std::path::Path;
use anyhow::Result;

use crate::core::binder::{Binder, BinderItem, BinderItemKind};

/// Output format for compilation
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone)]
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
        }
    }
}

#[derive(Debug, Clone)]
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

        match options.format {
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
        }
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

fn plain_text_compile(contents: &[CompileContent], options: &CompileOptions) -> Result<String> {
    super::plain_text::compile(contents, options)
}
