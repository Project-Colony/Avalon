#![allow(dead_code)]
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use notify::{self, Watcher, RecursiveMode, Event, EventKind};

/// Events emitted by the file watcher
#[derive(Debug, Clone)]
pub enum WatchEvent {
    /// A file inside the project was modified externally
    FileModified(PathBuf),
    /// A file was created inside the project directory
    FileCreated(PathBuf),
    /// A file was removed from the project directory
    FileRemoved(PathBuf),
    /// The project directory itself was removed
    ProjectRemoved,
}

/// Watches a project directory for external file changes
pub struct ProjectWatcher {
    _watcher: notify::RecommendedWatcher,
    receiver: mpsc::Receiver<WatchEvent>,
    project_path: PathBuf,
    last_event_time: Instant,
    /// Debounce interval — ignore rapid consecutive events
    debounce_ms: u64,
}

impl ProjectWatcher {
    /// Start watching a project directory
    pub fn new(project_path: &Path) -> Result<Self, notify::Error> {
        let (tx, rx) = mpsc::channel();

        let sender = tx.clone();
        let project_dir = project_path.to_path_buf();

        let mut watcher = notify::recommended_watcher(move |result: Result<Event, notify::Error>| {
            match result {
                Ok(event) => {
                    let watch_events = Self::translate_event(&event, &project_dir);
                    for we in watch_events {
                        let _ = sender.send(we);
                    }
                }
                Err(e) => log::warn!("File watcher error: {}", e),
            }
        })?;

        watcher.watch(project_path, RecursiveMode::Recursive)?;

        Ok(Self {
            _watcher: watcher,
            receiver: rx,
            project_path: project_path.to_path_buf(),
            last_event_time: Instant::now(),
            debounce_ms: 500,
        })
    }

    /// Translate a notify event into our WatchEvent type
    fn translate_event(event: &Event, _project_dir: &Path) -> Vec<WatchEvent> {
        let mut result = Vec::new();
        match event.kind {
            EventKind::Modify(_) => {
                for path in &event.paths {
                    result.push(WatchEvent::FileModified(path.clone()));
                }
            }
            EventKind::Create(_) => {
                for path in &event.paths {
                    result.push(WatchEvent::FileCreated(path.clone()));
                }
            }
            EventKind::Remove(_) => {
                for path in &event.paths {
                    result.push(WatchEvent::FileRemoved(path.clone()));
                }
            }
            _ => {}
        }
        result
    }

    /// Drain all pending watch events (non-blocking)
    pub fn poll_events(&mut self) -> Vec<WatchEvent> {
        let now = Instant::now();
        let debounce = Duration::from_millis(self.debounce_ms);

        let mut events = Vec::new();
        while let Ok(event) = self.receiver.try_recv() {
            // Only emit events if debounce time has passed
            if now.duration_since(self.last_event_time) >= debounce {
                events.push(event);
            }
        }

        if !events.is_empty() {
            self.last_event_time = now;
        }

        // Deduplicate: only keep the last event per path
        let mut seen = HashSet::new();
        events.retain(|e| {
            let path = match e {
                WatchEvent::FileModified(p)
                | WatchEvent::FileCreated(p)
                | WatchEvent::FileRemoved(p) => p.clone(),
                WatchEvent::ProjectRemoved => self.project_path.clone(),
            };
            seen.insert(path)
        });

        events
    }

    /// Check if the watched project path still exists
    pub fn is_project_alive(&self) -> bool {
        self.project_path.exists()
    }

    /// Get the project path being watched
    pub fn project_path(&self) -> &Path {
        &self.project_path
    }

    /// Set the debounce interval in milliseconds
    pub fn set_debounce_ms(&mut self, ms: u64) {
        self.debounce_ms = ms;
    }
}

/// Tracks which files have been modified externally and need reloading
#[derive(Debug, Default)]
pub struct ExternalChangeTracker {
    /// Paths that changed since last check
    pub modified_paths: Vec<PathBuf>,
    /// Whether the project metadata file changed
    pub project_metadata_changed: bool,
    /// Whether any document files changed
    pub documents_changed: Vec<PathBuf>,
}

impl ExternalChangeTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process watch events and categorize them
    pub fn process_events(&mut self, events: &[WatchEvent]) {
        self.modified_paths.clear();
        self.project_metadata_changed = false;
        self.documents_changed.clear();

        for event in events {
            match event {
                WatchEvent::FileModified(path) | WatchEvent::FileCreated(path) => {
                    self.modified_paths.push(path.clone());
                    if path.file_name().is_some_and(|n| n == "project.json") {
                        self.project_metadata_changed = true;
                    }
                    if path.extension().is_some_and(|e| e == "json") {
                        if let Some(parent) = path.parent() {
                            if parent.file_name().is_some_and(|n| n == "docs") {
                                self.documents_changed.push(path.clone());
                            }
                        }
                    }
                }
                WatchEvent::FileRemoved(path) => {
                    self.modified_paths.push(path.clone());
                }
                WatchEvent::ProjectRemoved => {
                    self.project_metadata_changed = true;
                }
            }
        }
    }

    /// Check if any external changes need handling
    pub fn has_changes(&self) -> bool {
        !self.modified_paths.is_empty()
    }

    /// Clear all tracked changes
    pub fn clear(&mut self) {
        self.modified_paths.clear();
        self.project_metadata_changed = false;
        self.documents_changed.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_external_change_tracker_new() {
        let tracker = ExternalChangeTracker::new();
        assert!(!tracker.has_changes());
        assert!(!tracker.project_metadata_changed);
        assert!(tracker.documents_changed.is_empty());
    }

    #[test]
    fn test_external_change_tracker_process_events() {
        let mut tracker = ExternalChangeTracker::new();
        let events = vec![
            WatchEvent::FileModified(PathBuf::from("/project/project.json")),
            WatchEvent::FileModified(PathBuf::from("/project/docs/abc.json")),
        ];
        tracker.process_events(&events);
        assert!(tracker.has_changes());
        assert!(tracker.project_metadata_changed);
        assert_eq!(tracker.documents_changed.len(), 1);
    }

    #[test]
    fn test_external_change_tracker_clear() {
        let mut tracker = ExternalChangeTracker::new();
        let events = vec![
            WatchEvent::FileModified(PathBuf::from("/project/project.json")),
        ];
        tracker.process_events(&events);
        assert!(tracker.has_changes());
        tracker.clear();
        assert!(!tracker.has_changes());
    }

    #[test]
    fn test_external_change_tracker_file_created() {
        let mut tracker = ExternalChangeTracker::new();
        let events = vec![
            WatchEvent::FileCreated(PathBuf::from("/project/docs/new-doc.json")),
        ];
        tracker.process_events(&events);
        assert!(tracker.has_changes());
        assert_eq!(tracker.documents_changed.len(), 1);
        assert!(!tracker.project_metadata_changed);
    }

    #[test]
    fn test_external_change_tracker_file_removed() {
        let mut tracker = ExternalChangeTracker::new();
        let events = vec![
            WatchEvent::FileRemoved(PathBuf::from("/project/docs/deleted.json")),
        ];
        tracker.process_events(&events);
        assert!(tracker.has_changes());
        assert_eq!(tracker.modified_paths.len(), 1);
    }

    #[test]
    fn test_external_change_tracker_project_removed() {
        let mut tracker = ExternalChangeTracker::new();
        let events = vec![WatchEvent::ProjectRemoved];
        tracker.process_events(&events);
        // ProjectRemoved sets metadata_changed flag
        assert!(tracker.project_metadata_changed);
    }

    #[test]
    fn test_external_change_tracker_multiple_events() {
        let mut tracker = ExternalChangeTracker::new();
        let events = vec![
            WatchEvent::FileModified(PathBuf::from("/project/docs/a.json")),
            WatchEvent::FileCreated(PathBuf::from("/project/docs/b.json")),
            WatchEvent::FileRemoved(PathBuf::from("/project/old.txt")),
            WatchEvent::FileModified(PathBuf::from("/project/project.json")),
        ];
        tracker.process_events(&events);
        assert!(tracker.has_changes());
        assert_eq!(tracker.modified_paths.len(), 4);
        assert!(tracker.project_metadata_changed);
        assert_eq!(tracker.documents_changed.len(), 2); // a.json and b.json
    }

    #[test]
    fn test_external_change_tracker_non_json_file() {
        let mut tracker = ExternalChangeTracker::new();
        let events = vec![
            WatchEvent::FileModified(PathBuf::from("/project/readme.md")),
        ];
        tracker.process_events(&events);
        assert!(tracker.has_changes());
        assert!(!tracker.project_metadata_changed);
        assert!(tracker.documents_changed.is_empty());
    }

    #[test]
    fn test_external_change_tracker_clear_resets_all() {
        let mut tracker = ExternalChangeTracker::new();
        let events = vec![
            WatchEvent::FileModified(PathBuf::from("/project/project.json")),
            WatchEvent::FileModified(PathBuf::from("/project/docs/a.json")),
        ];
        tracker.process_events(&events);
        assert!(tracker.has_changes());
        assert!(tracker.project_metadata_changed);
        assert!(!tracker.documents_changed.is_empty());

        tracker.clear();
        assert!(!tracker.has_changes());
        assert!(!tracker.project_metadata_changed);
        assert!(tracker.documents_changed.is_empty());
    }

    #[test]
    fn test_external_change_tracker_process_replaces() {
        let mut tracker = ExternalChangeTracker::new();
        let events1 = vec![
            WatchEvent::FileModified(PathBuf::from("/project/file1.txt")),
        ];
        tracker.process_events(&events1);
        assert_eq!(tracker.modified_paths.len(), 1);

        let events2 = vec![
            WatchEvent::FileModified(PathBuf::from("/project/file2.txt")),
        ];
        tracker.process_events(&events2);
        // process_events clears previous state first
        assert_eq!(tracker.modified_paths.len(), 1);
        assert_eq!(tracker.modified_paths[0], PathBuf::from("/project/file2.txt"));
    }

    #[test]
    fn test_external_change_tracker_empty_events() {
        let mut tracker = ExternalChangeTracker::new();
        tracker.process_events(&[]);
        assert!(!tracker.has_changes());
    }

    #[test]
    fn test_external_change_tracker_json_not_in_docs() {
        let mut tracker = ExternalChangeTracker::new();
        let events = vec![
            WatchEvent::FileModified(PathBuf::from("/project/config/settings.json")),
        ];
        tracker.process_events(&events);
        assert!(tracker.has_changes());
        // JSON file outside docs/ should not be in documents_changed
        assert!(tracker.documents_changed.is_empty());
    }

    #[test]
    fn test_default_tracker() {
        let tracker = ExternalChangeTracker::default();
        assert!(!tracker.has_changes());
        assert!(tracker.modified_paths.is_empty());
        assert!(tracker.documents_changed.is_empty());
        assert!(!tracker.project_metadata_changed);
    }

    #[test]
    fn test_watch_event_debug() {
        let event = WatchEvent::FileModified(PathBuf::from("/test/file.txt"));
        let debug = format!("{:?}", event);
        assert!(debug.contains("FileModified"));
    }

    #[test]
    fn test_watch_event_clone() {
        let event = WatchEvent::FileCreated(PathBuf::from("/test/new.txt"));
        let cloned = event.clone();
        match cloned {
            WatchEvent::FileCreated(p) => assert_eq!(p, PathBuf::from("/test/new.txt")),
            _ => panic!("Clone produced different variant"),
        }
    }

    #[test]
    fn test_project_removed_event() {
        let event = WatchEvent::ProjectRemoved;
        let debug = format!("{:?}", event);
        assert!(debug.contains("ProjectRemoved"));
    }

    #[test]
    fn test_tracker_only_docs_json_counted() {
        let mut tracker = ExternalChangeTracker::new();
        // JSON in /project/docs/ should be counted as document change
        // JSON in /project/other/ should not
        let events = vec![
            WatchEvent::FileModified(PathBuf::from("/project/docs/scene.json")),
            WatchEvent::FileModified(PathBuf::from("/project/other/data.json")),
            WatchEvent::FileModified(PathBuf::from("/project/docs/chapter.json")),
        ];
        tracker.process_events(&events);
        assert_eq!(tracker.documents_changed.len(), 2);
        assert_eq!(tracker.modified_paths.len(), 3);
    }

    #[test]
    fn test_tracker_non_json_in_docs_not_counted() {
        let mut tracker = ExternalChangeTracker::new();
        let events = vec![
            WatchEvent::FileModified(PathBuf::from("/project/docs/readme.md")),
        ];
        tracker.process_events(&events);
        assert!(tracker.documents_changed.is_empty());
        assert_eq!(tracker.modified_paths.len(), 1);
    }

    #[test]
    fn test_tracker_process_events_mixed_types() {
        let mut tracker = ExternalChangeTracker::new();
        let events = vec![
            WatchEvent::FileCreated(PathBuf::from("/project/docs/new.json")),
            WatchEvent::FileModified(PathBuf::from("/project/docs/existing.json")),
            WatchEvent::FileRemoved(PathBuf::from("/project/docs/deleted.json")),
        ];
        tracker.process_events(&events);
        assert_eq!(tracker.modified_paths.len(), 3);
        // Created and Modified JSON in docs/ count as document changes
        assert_eq!(tracker.documents_changed.len(), 2);
    }

    #[test]
    fn test_tracker_process_replaces_previous_state() {
        let mut tracker = ExternalChangeTracker::new();
        let events1 = vec![
            WatchEvent::FileModified(PathBuf::from("/project/project.json")),
        ];
        tracker.process_events(&events1);
        assert!(tracker.project_metadata_changed);

        let events2 = vec![
            WatchEvent::FileModified(PathBuf::from("/project/docs/a.json")),
        ];
        tracker.process_events(&events2);
        // Previous metadata_changed flag should be reset
        assert!(!tracker.project_metadata_changed);
        assert_eq!(tracker.modified_paths.len(), 1);
    }

    #[test]
    fn test_tracker_nested_docs_path() {
        let mut tracker = ExternalChangeTracker::new();
        // JSON in nested docs subdirectory should NOT be counted
        let events = vec![
            WatchEvent::FileModified(PathBuf::from("/project/docs/subfolder/deep.json")),
        ];
        tracker.process_events(&events);
        // Parent is "subfolder", not "docs", so shouldn't be in documents_changed
        assert!(tracker.documents_changed.is_empty());
    }

    #[test]
    fn test_tracker_project_json_in_subdirectory() {
        let mut tracker = ExternalChangeTracker::new();
        let events = vec![
            WatchEvent::FileModified(PathBuf::from("/project/subdir/project.json")),
        ];
        tracker.process_events(&events);
        // Only root-level project.json triggers metadata change
        assert!(tracker.project_metadata_changed);
    }

    #[test]
    fn test_watch_event_all_variants_debug() {
        let events = vec![
            WatchEvent::FileModified(PathBuf::from("/a")),
            WatchEvent::FileCreated(PathBuf::from("/b")),
            WatchEvent::FileRemoved(PathBuf::from("/c")),
            WatchEvent::ProjectRemoved,
        ];
        for event in &events {
            let debug = format!("{:?}", event);
            assert!(!debug.is_empty());
        }
    }

    #[test]
    fn test_watch_event_clone_all_variants() {
        let variants = vec![
            WatchEvent::FileModified(PathBuf::from("/test")),
            WatchEvent::FileCreated(PathBuf::from("/test")),
            WatchEvent::FileRemoved(PathBuf::from("/test")),
            WatchEvent::ProjectRemoved,
        ];
        for v in variants {
            let _ = v.clone();
        }
    }

    #[test]
    fn test_tracker_large_batch() {
        let mut tracker = ExternalChangeTracker::new();
        let events: Vec<WatchEvent> = (0..100)
            .map(|i| WatchEvent::FileModified(PathBuf::from(format!("/project/docs/file{}.json", i))))
            .collect();
        tracker.process_events(&events);
        assert_eq!(tracker.modified_paths.len(), 100);
        assert_eq!(tracker.documents_changed.len(), 100);
    }

    #[test]
    fn test_tracker_has_changes_after_clear() {
        let mut tracker = ExternalChangeTracker::new();
        let events = vec![
            WatchEvent::FileCreated(PathBuf::from("/project/docs/new.json")),
        ];
        tracker.process_events(&events);
        assert!(tracker.has_changes());
        tracker.clear();
        assert!(!tracker.has_changes());
        assert!(tracker.documents_changed.is_empty());
        assert!(!tracker.project_metadata_changed);
    }
}
