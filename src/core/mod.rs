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
