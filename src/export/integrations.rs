//! Export integrations: compile with statistics and unified import dispatch.
//!
//! Provides higher-level entry points that combine the compiler, manifest,
//! statistics, and import modules into convenient workflows the GUI calls.

use crate::core::binder::{Binder, BinderItem, BinderItemKind};
use crate::export::compiler::{
    CompileContent, CompileManifest, CompileOptions, CompileStatistics,
};

// ---------------------------------------------------------------------------
// Compile with statistics
// ---------------------------------------------------------------------------

/// Result of a compilation that includes rich statistics and a manifest.
#[derive(Debug, Clone)]
pub struct CompileResult {
    pub statistics: CompileStatistics,
    pub manifest: CompileManifest,
    pub validation_issues: Vec<String>,
}

/// Compile a binder and return detailed statistics, manifest, and a preview.
pub fn compile_with_stats(binder: &Binder, options: &CompileOptions) -> CompileResult {
    let contents = collect_contents(binder, options);

    let statistics = CompileStatistics::from_contents(&contents);
    let manifest = CompileManifest::from_contents(&contents, options);
    let validation_issues = options.validate();

    CompileResult {
        statistics,
        manifest,
        validation_issues,
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
