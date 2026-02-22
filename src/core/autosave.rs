use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Auto-save manager with debounce logic.
/// Prevents saving too frequently while ensuring changes are persisted.
#[derive(Debug)]
pub struct AutoSaveManager {
    /// Whether auto-save is enabled
    pub enabled: bool,
    /// Save interval in seconds (minimum time between saves)
    pub interval_seconds: u32,
    /// Debounce delay in milliseconds (wait after last edit before saving)
    pub debounce_ms: u64,
    /// Last time a save was performed
    last_save: Option<Instant>,
    /// Last time the content was modified
    last_edit: Option<Instant>,
    /// Whether there are unsaved changes
    dirty: bool,
    /// Number of saves performed
    save_count: u64,
    /// Number of saves skipped due to debounce
    skip_count: u64,
    /// Auto-backup interval (every N saves)
    pub backup_every_n_saves: u32,
    /// Path where backups are stored
    pub backup_path: Option<PathBuf>,
}

impl AutoSaveManager {
    pub fn new(interval_seconds: u32) -> Self {
        Self {
            enabled: true,
            interval_seconds,
            debounce_ms: 1000, // 1 second debounce by default
            last_save: None,
            last_edit: None,
            dirty: false,
            save_count: 0,
            skip_count: 0,
            backup_every_n_saves: 10,
            backup_path: None,
        }
    }

    /// Mark that a modification has occurred
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.last_edit = Some(Instant::now());
    }

    /// Mark that a save was just performed
    pub fn mark_saved(&mut self) {
        self.dirty = false;
        self.last_save = Some(Instant::now());
        self.save_count += 1;
    }

    /// Check if we should save now (called on tick)
    pub fn should_save(&self) -> bool {
        if !self.enabled || !self.dirty {
            return false;
        }

        let now = Instant::now();

        // Check debounce: don't save if user is still actively editing
        if let Some(last_edit) = self.last_edit {
            if now.duration_since(last_edit) < Duration::from_millis(self.debounce_ms) {
                return false;
            }
        }

        // Check interval: don't save more frequently than interval
        if let Some(last_save) = self.last_save {
            if now.duration_since(last_save) < Duration::from_secs(self.interval_seconds as u64) {
                return false;
            }
        }

        true
    }

    /// Check if a backup should be created (after current save)
    pub fn should_backup(&self) -> bool {
        self.backup_every_n_saves > 0
            && self.save_count > 0
            && self.save_count % self.backup_every_n_saves as u64 == 0
    }

    /// Whether there are unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Get the number of saves performed
    pub fn save_count(&self) -> u64 {
        self.save_count
    }

    /// Get the number of saves skipped
    pub fn skip_count(&self) -> u64 {
        self.skip_count
    }

    /// Record a skipped save (for metrics)
    pub fn record_skip(&mut self) {
        self.skip_count += 1;
    }

    /// Time since last save (if any)
    pub fn time_since_last_save(&self) -> Option<Duration> {
        self.last_save.map(|t| Instant::now().duration_since(t))
    }

    /// Time since last edit (if any)
    pub fn time_since_last_edit(&self) -> Option<Duration> {
        self.last_edit.map(|t| Instant::now().duration_since(t))
    }

    /// Reset all state
    pub fn reset(&mut self) {
        self.last_save = None;
        self.last_edit = None;
        self.dirty = false;
        self.save_count = 0;
        self.skip_count = 0;
    }

    /// Get a status summary
    pub fn status(&self) -> AutoSaveStatus {
        AutoSaveStatus {
            enabled: self.enabled,
            dirty: self.dirty,
            save_count: self.save_count,
            interval_seconds: self.interval_seconds,
            time_since_last_save_secs: self.time_since_last_save()
                .map(|d| d.as_secs()),
        }
    }
}

/// Status snapshot for the auto-save manager
#[derive(Debug, Clone)]
pub struct AutoSaveStatus {
    pub enabled: bool,
    pub dirty: bool,
    pub save_count: u64,
    pub interval_seconds: u32,
    pub time_since_last_save_secs: Option<u64>,
}

impl AutoSaveStatus {
    pub fn display(&self) -> String {
        if !self.enabled {
            return "Auto-save disabled".to_string();
        }
        if self.dirty {
            match self.time_since_last_save_secs {
                Some(secs) => format!("Unsaved changes (last save {}s ago)", secs),
                None => "Unsaved changes".to_string(),
            }
        } else {
            match self.time_since_last_save_secs {
                Some(secs) => format!("Saved ({}s ago)", secs),
                None => "No changes".to_string(),
            }
        }
    }
}

/// Strategy for when to create automatic backups
#[derive(Debug, Clone, PartialEq)]
pub enum BackupStrategy {
    /// Backup after every N saves
    EverySaves(u32),
    /// Backup after a certain amount of time
    AfterDuration(Duration),
    /// Backup when word count changes by a certain amount
    AfterWordCountChange(usize),
    /// No automatic backups
    Never,
}

/// Manages the save queue for async saving
#[derive(Debug)]
pub struct SaveQueue {
    /// Pending save operations
    pending: Vec<SaveOperation>,
    /// Maximum queue size before forcing a save
    max_queue_size: usize,
}

#[derive(Debug, Clone)]
pub struct SaveOperation {
    pub project_path: PathBuf,
    pub timestamp: Instant,
    pub kind: SaveKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SaveKind {
    /// Regular auto-save
    AutoSave,
    /// User-initiated save
    Manual,
    /// Pre-compile save
    PreCompile,
    /// Save before closing
    PreClose,
}

impl SaveQueue {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            max_queue_size: 5,
        }
    }

    /// Add a save operation to the queue
    pub fn enqueue(&mut self, path: &Path, kind: SaveKind) {
        // Replace any existing pending save of the same kind
        self.pending.retain(|op| op.kind != kind);
        self.pending.push(SaveOperation {
            project_path: path.to_path_buf(),
            timestamp: Instant::now(),
            kind,
        });

        // If queue is too large, keep only the most recent
        if self.pending.len() > self.max_queue_size {
            self.pending = self.pending.split_off(self.pending.len() - self.max_queue_size);
        }
    }

    /// Get the next pending save (FIFO)
    pub fn dequeue(&mut self) -> Option<SaveOperation> {
        if self.pending.is_empty() {
            None
        } else {
            Some(self.pending.remove(0))
        }
    }

    /// Check if there are pending saves
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Get the number of pending saves
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Clear all pending saves
    pub fn clear(&mut self) {
        self.pending.clear();
    }

    /// Check if there's a manual save pending (should be prioritized)
    pub fn has_manual_save(&self) -> bool {
        self.pending.iter().any(|op| op.kind == SaveKind::Manual)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autosave_new() {
        let manager = AutoSaveManager::new(30);
        assert!(manager.enabled);
        assert!(!manager.is_dirty());
        assert_eq!(manager.save_count(), 0);
    }

    #[test]
    fn test_autosave_dirty() {
        let mut manager = AutoSaveManager::new(30);
        assert!(!manager.is_dirty());
        manager.mark_dirty();
        assert!(manager.is_dirty());
        manager.mark_saved();
        assert!(!manager.is_dirty());
    }

    #[test]
    fn test_autosave_not_dirty_no_save() {
        let manager = AutoSaveManager::new(30);
        assert!(!manager.should_save());
    }

    #[test]
    fn test_autosave_disabled_no_save() {
        let mut manager = AutoSaveManager::new(30);
        manager.enabled = false;
        manager.mark_dirty();
        assert!(!manager.should_save());
    }

    #[test]
    fn test_autosave_debounce() {
        let mut manager = AutoSaveManager::new(0);
        manager.debounce_ms = 5000; // 5 second debounce
        manager.mark_dirty();
        // Just edited — should NOT save yet due to debounce
        assert!(!manager.should_save());
    }

    #[test]
    fn test_autosave_save_count() {
        let mut manager = AutoSaveManager::new(30);
        manager.mark_dirty();
        manager.mark_saved();
        assert_eq!(manager.save_count(), 1);
        manager.mark_dirty();
        manager.mark_saved();
        assert_eq!(manager.save_count(), 2);
    }

    #[test]
    fn test_autosave_should_backup() {
        let mut manager = AutoSaveManager::new(30);
        manager.backup_every_n_saves = 5;
        for _ in 0..5 {
            manager.mark_dirty();
            manager.mark_saved();
        }
        assert!(manager.should_backup());
    }

    #[test]
    fn test_autosave_reset() {
        let mut manager = AutoSaveManager::new(30);
        manager.mark_dirty();
        manager.mark_saved();
        assert_eq!(manager.save_count(), 1);
        manager.reset();
        assert_eq!(manager.save_count(), 0);
        assert!(!manager.is_dirty());
    }

    #[test]
    fn test_autosave_status_display() {
        let manager = AutoSaveManager::new(30);
        let status = manager.status();
        assert!(status.display().contains("No changes"));

        let mut manager2 = AutoSaveManager::new(30);
        manager2.enabled = false;
        let status2 = manager2.status();
        assert!(status2.display().contains("disabled"));
    }

    #[test]
    fn test_save_queue_new() {
        let queue = SaveQueue::new();
        assert!(!queue.has_pending());
        assert_eq!(queue.pending_count(), 0);
    }

    #[test]
    fn test_save_queue_enqueue_dequeue() {
        let mut queue = SaveQueue::new();
        queue.enqueue(Path::new("/tmp/test"), SaveKind::AutoSave);
        assert!(queue.has_pending());
        assert_eq!(queue.pending_count(), 1);

        let op = queue.dequeue().unwrap();
        assert_eq!(op.kind, SaveKind::AutoSave);
        assert!(!queue.has_pending());
    }

    #[test]
    fn test_save_queue_dedup() {
        let mut queue = SaveQueue::new();
        queue.enqueue(Path::new("/tmp/test"), SaveKind::AutoSave);
        queue.enqueue(Path::new("/tmp/test"), SaveKind::AutoSave);
        assert_eq!(queue.pending_count(), 1);
    }

    #[test]
    fn test_save_queue_manual_save() {
        let mut queue = SaveQueue::new();
        queue.enqueue(Path::new("/tmp/test"), SaveKind::Manual);
        assert!(queue.has_manual_save());
    }

    #[test]
    fn test_save_queue_clear() {
        let mut queue = SaveQueue::new();
        queue.enqueue(Path::new("/tmp/test"), SaveKind::AutoSave);
        queue.enqueue(Path::new("/tmp/test"), SaveKind::Manual);
        assert_eq!(queue.pending_count(), 2);
        queue.clear();
        assert_eq!(queue.pending_count(), 0);
    }

    #[test]
    fn test_autosave_skip_count() {
        let mut manager = AutoSaveManager::new(30);
        assert_eq!(manager.skip_count(), 0);
        manager.record_skip();
        manager.record_skip();
        assert_eq!(manager.skip_count(), 2);
    }

    #[test]
    fn test_autosave_time_since_last_save_none() {
        let manager = AutoSaveManager::new(30);
        assert!(manager.time_since_last_save().is_none());
    }

    #[test]
    fn test_autosave_time_since_last_save_some() {
        let mut manager = AutoSaveManager::new(30);
        manager.mark_dirty();
        manager.mark_saved();
        assert!(manager.time_since_last_save().is_some());
    }

    #[test]
    fn test_autosave_time_since_last_edit_none() {
        let manager = AutoSaveManager::new(30);
        assert!(manager.time_since_last_edit().is_none());
    }

    #[test]
    fn test_autosave_time_since_last_edit_some() {
        let mut manager = AutoSaveManager::new(30);
        manager.mark_dirty();
        assert!(manager.time_since_last_edit().is_some());
    }

    #[test]
    fn test_autosave_status_dirty() {
        let mut manager = AutoSaveManager::new(30);
        manager.mark_dirty();
        let status = manager.status();
        assert!(status.dirty);
        assert!(status.display().contains("Unsaved"));
    }

    #[test]
    fn test_autosave_status_saved() {
        let mut manager = AutoSaveManager::new(30);
        manager.mark_dirty();
        manager.mark_saved();
        let status = manager.status();
        assert!(!status.dirty);
        assert!(status.display().contains("Saved"));
    }

    #[test]
    fn test_autosave_should_backup_not_yet() {
        let mut manager = AutoSaveManager::new(30);
        manager.backup_every_n_saves = 5;
        for _ in 0..3 {
            manager.mark_dirty();
            manager.mark_saved();
        }
        assert!(!manager.should_backup());
    }

    #[test]
    fn test_autosave_backup_path() {
        let mut manager = AutoSaveManager::new(30);
        assert!(manager.backup_path.is_none());
        manager.backup_path = Some(PathBuf::from("/tmp/backups"));
        assert_eq!(manager.backup_path.as_ref().unwrap().to_str().unwrap(), "/tmp/backups");
    }

    #[test]
    fn test_save_queue_different_kinds() {
        let mut queue = SaveQueue::new();
        queue.enqueue(Path::new("/tmp/a"), SaveKind::AutoSave);
        queue.enqueue(Path::new("/tmp/a"), SaveKind::Manual);
        queue.enqueue(Path::new("/tmp/a"), SaveKind::PreCompile);
        queue.enqueue(Path::new("/tmp/a"), SaveKind::PreClose);
        assert_eq!(queue.pending_count(), 4);
    }

    #[test]
    fn test_save_queue_dequeue_fifo() {
        let mut queue = SaveQueue::new();
        queue.enqueue(Path::new("/tmp/a"), SaveKind::AutoSave);
        queue.enqueue(Path::new("/tmp/a"), SaveKind::Manual);
        let first = queue.dequeue().unwrap();
        assert_eq!(first.kind, SaveKind::AutoSave);
        let second = queue.dequeue().unwrap();
        assert_eq!(second.kind, SaveKind::Manual);
    }

    #[test]
    fn test_save_queue_dequeue_empty() {
        let mut queue = SaveQueue::new();
        assert!(queue.dequeue().is_none());
    }

    #[test]
    fn test_save_queue_max_size() {
        let mut queue = SaveQueue::new();
        // Enqueue many different kinds - since dedup only applies to same kind,
        // we need to use different paths
        for i in 0..10 {
            queue.enqueue(Path::new(&format!("/tmp/{}", i)), SaveKind::AutoSave);
        }
        // Should be capped at max_queue_size (5)
        assert!(queue.pending_count() <= 5);
    }

    #[test]
    fn test_save_queue_no_manual_save() {
        let mut queue = SaveQueue::new();
        queue.enqueue(Path::new("/tmp/a"), SaveKind::AutoSave);
        assert!(!queue.has_manual_save());
    }

    #[test]
    fn test_save_kind_equality() {
        assert_eq!(SaveKind::AutoSave, SaveKind::AutoSave);
        assert_ne!(SaveKind::AutoSave, SaveKind::Manual);
        assert_ne!(SaveKind::PreCompile, SaveKind::PreClose);
    }

    #[test]
    fn test_backup_strategy_equality() {
        assert_eq!(BackupStrategy::Never, BackupStrategy::Never);
        assert_eq!(BackupStrategy::EverySaves(10), BackupStrategy::EverySaves(10));
        assert_ne!(BackupStrategy::EverySaves(10), BackupStrategy::EverySaves(5));
        assert_ne!(BackupStrategy::Never, BackupStrategy::EverySaves(1));
    }

    #[test]
    fn test_autosave_debounce_default() {
        let manager = AutoSaveManager::new(30);
        assert_eq!(manager.debounce_ms, 1000);
    }

    #[test]
    fn test_autosave_backup_default() {
        let manager = AutoSaveManager::new(30);
        assert_eq!(manager.backup_every_n_saves, 10);
    }
}
