// Well-known directory names (relative to the user's home directory)
pub const PROJECTS_DIR_NAME: &str = "Scrinever Projects";
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

/// Get the user's home directory, or fall back to the current directory with a warning.
/// Centralizes the home directory lookup so all callers handle the failure case
/// consistently (log + safe fallback) rather than silently using an empty path.
pub fn home_dir_or_cwd() -> std::path::PathBuf {
    dirs::home_dir().unwrap_or_else(|| {
        log::warn!("Could not determine home directory, falling back to current directory");
        std::path::PathBuf::from(".")
    })
}

/// Write `contents` to `path` without ever leaving a half-written file there.
///
/// The data goes to `<path>.tmp` first, is flushed to disk, and is then renamed
/// over `path`, so a crash mid-save keeps the previous version intact. On
/// failure the temporary file is removed.
pub fn write_atomic(path: &std::path::Path, contents: &[u8]) -> std::io::Result<()> {
    use std::io::Write;

    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = std::path::PathBuf::from(tmp);

    let result = (|| {
        {
            let mut file = std::fs::File::create(&tmp)?;
            file.write_all(contents)?;
            file.sync_all()?;
        }
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

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
pub mod binder;
pub mod collection;
pub mod document;
pub mod metadata;
pub mod project;

// Editor features
pub mod annotation;
pub mod comments;
pub mod find_replace;
pub mod indexer;
pub mod links;
pub mod revision;
pub mod search;

// Document management
pub mod autosave;
pub mod backup;
pub mod doc_templates;
pub mod media_import;
pub mod snapshot;
pub mod watcher;

// Writing tools
pub mod history;
pub mod linguistic;
pub mod namegen;
pub mod script;
pub mod stats;
pub mod targets;
pub mod text_analysis;
pub mod timer;
pub mod validation;
pub mod writing_prompts;

// Views & navigation
pub mod bookmark;
pub mod corkboard;
pub mod outliner;
pub mod recent;

// Application-level configuration
pub mod app_config;

// Integration module (wires together subsystems)
pub mod integrations;
