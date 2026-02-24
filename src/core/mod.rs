// Well-known directory names (relative to the user's home directory)
pub const PROJECTS_DIR_NAME: &str = "Scrinever Projects";
pub const OUTPUT_DIR_NAME: &str = "Scrinever Output";
pub const IMPORT_DIR_NAME: &str = "Scrinever Import";
pub const BACKUPS_DIR_NAME: &str = "Scrinever Backups";

/// File extension for project directories.
pub const PROJECT_EXTENSION: &str = "scriv";

/// ISO 8601 date format used for serialization and display.
pub const DATE_FORMAT: &str = "%Y-%m-%d";

// Standard typographical constants
/// Average words per page (standard manuscript page).
/// Used for page-count estimation and reading-time calculations.
pub const WORDS_PER_PAGE: usize = 250;

/// Average reading speed in words per minute.
pub const READING_WPM: f64 = 250.0;

/// Average speaking speed in words per minute.
pub const SPEAKING_WPM: f64 = 150.0;

/// Format a byte count into a human-readable size string (e.g. "1.5 MB").
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;
    if bytes < KB {
        format!("{} bytes", bytes)
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    }
}

// Core data structures
pub mod project;
pub mod binder;
pub mod document;
pub mod metadata;
pub mod collection;

// Editor features
pub mod search;
pub mod indexer;
pub mod find_replace;
pub mod links;
pub mod annotation;
pub mod comments;
pub mod revision;

// Document management
pub mod snapshot;
pub mod autosave;
pub mod backup;
pub mod doc_templates;
pub mod watcher;
pub mod media_import;

// Writing tools
pub mod stats;
pub mod targets;
pub mod timer;
pub mod history;
pub mod linguistic;
pub mod text_analysis;
pub mod validation;
pub mod writing_prompts;
pub mod namegen;
pub mod script;

// Views & navigation
pub mod corkboard;
pub mod outliner;
pub mod bookmark;
pub mod recent;

// Integration module (wires together subsystems)
pub mod integrations;
