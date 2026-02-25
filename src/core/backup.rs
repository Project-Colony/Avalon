use std::path::{Path, PathBuf};
use std::fs;
use chrono::Utc;
use anyhow::{Context, Result};

/// Backup manager for Scrinever projects.
/// Creates timestamped backups and manages retention.
pub struct BackupManager;

impl BackupManager {
    /// Create a backup of a project directory.
    /// Returns the path to the backup file.
    pub fn create_backup(project_dir: &Path) -> Result<PathBuf> {
        let backup_dir = Self::backup_directory()?;
        fs::create_dir_all(&backup_dir)?;

        let project_name = project_dir.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("{}_{}.backup.json", project_name, timestamp);
        let backup_path = backup_dir.join(&backup_name);

        // Read project.json and copy it as backup
        let project_json = project_dir.join("project.json");
        if project_json.exists() {
            fs::copy(&project_json, &backup_path)
                .context("Failed to create backup copy")?;
        }

        // Prune old backups (keep last 20)
        Self::prune_backups(&backup_dir, project_name, 20)?;

        Ok(backup_path)
    }

    /// Restore a project from a backup file
    pub fn restore_backup(backup_path: &Path, project_dir: &Path) -> Result<()> {
        let project_json = project_dir.join("project.json");
        fs::copy(backup_path, &project_json)
            .context("Failed to restore backup")?;
        Ok(())
    }

    /// List available backups for a project
    pub fn list_backups(project_name: &str) -> Result<Vec<BackupEntry>> {
        let backup_dir = Self::backup_directory()?;
        let mut entries = Vec::new();

        if !backup_dir.exists() {
            return Ok(entries);
        }

        let prefix = format!("{}_", project_name);
        let dir_entries = fs::read_dir(&backup_dir)
            .context("Failed to read backup directory")?;
        for entry in dir_entries.flatten() {
            let path = entry.path();
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if name.starts_with(&prefix) && name.ends_with(".backup.json") {
                let size = fs::metadata(&path)
                    .map(|m| m.len())
                    .unwrap_or(0);

                // Parse timestamp from filename
                let timestamp_str = name
                    .strip_prefix(&prefix)
                    .and_then(|s| s.strip_suffix(".backup.json"))
                    .unwrap_or("");

                entries.push(BackupEntry {
                    path: path.clone(),
                    timestamp: timestamp_str.to_string(),
                    size_bytes: size,
                });
            }
        }

        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(entries)
    }

    /// Get the backup directory path
    fn backup_directory() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .context("Could not determine home directory")?;
        Ok(home.join(super::BACKUPS_DIR_NAME))
    }

    /// Remove old backups, keeping only the most recent `keep` backups
    fn prune_backups(_backup_dir: &Path, project_name: &str, keep: usize) -> Result<()> {
        let mut backups = Self::list_backups(project_name)?;
        if backups.len() > keep {
            let to_remove = backups.split_off(keep);
            for entry in to_remove {
                if let Err(e) = fs::remove_file(&entry.path) {
                    log::warn!("Failed to prune old backup {:?}: {}", entry.path, e);
                }
            }
        }
        Ok(())
    }

}

/// A single backup entry
#[derive(Debug, Clone)]
pub struct BackupEntry {
    pub path: PathBuf,
    pub timestamp: String,
    pub size_bytes: u64,
}

impl BackupEntry {
    /// Format bytes into human-readable size string
    pub fn format_bytes(bytes: u64) -> String {
        super::format_bytes(bytes)
    }

    pub fn display_size(&self) -> String {
        Self::format_bytes(self.size_bytes)
    }

    pub fn display_timestamp(&self) -> String {
        if self.timestamp.len() >= 15 {
            format!(
                "{}-{}-{} {}:{}:{}",
                &self.timestamp[0..4],
                &self.timestamp[4..6],
                &self.timestamp[6..8],
                &self.timestamp[9..11],
                &self.timestamp[11..13],
                &self.timestamp[13..15],
            )
        } else {
            self.timestamp.clone()
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(timestamp: &str, size: u64) -> BackupEntry {
        BackupEntry {
            path: PathBuf::from("/backups/test.backup.json"),
            timestamp: timestamp.to_string(),
            size_bytes: size,
        }
    }

    #[test]
    fn test_display_size_bytes() {
        let entry = make_entry("20260101_120000", 500);
        assert_eq!(entry.display_size(), "500 bytes");
    }

    #[test]
    fn test_display_size_kilobytes() {
        let entry = make_entry("20260101_120000", 2048);
        assert_eq!(entry.display_size(), "2.0 KB");
    }

    #[test]
    fn test_display_size_megabytes() {
        let entry = make_entry("20260101_120000", 5 * 1024 * 1024);
        assert_eq!(entry.display_size(), "5.0 MB");
    }

    #[test]
    fn test_display_size_gigabytes() {
        let entry = make_entry("20260101_120000", 2 * 1024 * 1024 * 1024);
        assert_eq!(entry.display_size(), "2.00 GB");
    }

    #[test]
    fn test_display_timestamp_valid() {
        let entry = make_entry("20260215_143022", 100);
        assert_eq!(entry.display_timestamp(), "2026-02-15 14:30:22");
    }

    #[test]
    fn test_display_timestamp_short() {
        let entry = make_entry("short", 100);
        assert_eq!(entry.display_timestamp(), "short");
    }

    #[test]
    fn test_format_bytes_static() {
        assert_eq!(BackupEntry::format_bytes(0), "0 bytes");
        assert_eq!(BackupEntry::format_bytes(512), "512 bytes");
        assert_eq!(BackupEntry::format_bytes(1024), "1.0 KB");
        assert_eq!(BackupEntry::format_bytes(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn test_display_size_boundary_kb() {
        let entry = make_entry("20260101_120000", 1023);
        assert!(entry.display_size().contains("bytes"));

        let entry2 = make_entry("20260101_120000", 1024);
        assert!(entry2.display_size().contains("KB"));
    }

    #[test]
    fn test_display_size_boundary_mb() {
        let entry = make_entry("20260101_120000", 1024 * 1024 - 1);
        assert!(entry.display_size().contains("KB"));

        let entry2 = make_entry("20260101_120000", 1024 * 1024);
        assert!(entry2.display_size().contains("MB"));
    }

    #[test]
    fn test_display_size_zero() {
        let entry = make_entry("20260101_120000", 0);
        assert_eq!(entry.display_size(), "0 bytes");
    }

    #[test]
    fn test_format_bytes_matches_display_size() {
        for size in [0u64, 500, 2048, 5 * 1024 * 1024] {
            let entry = make_entry("20260101_120000", size);
            assert_eq!(entry.display_size(), BackupEntry::format_bytes(size));
        }
    }

    #[test]
    fn test_display_timestamp_formats_correctly() {
        let entry = make_entry("20260101_000000", 100);
        assert_eq!(entry.display_timestamp(), "2026-01-01 00:00:00");
    }

    #[test]
    fn test_list_backups_nonexistent_project() {
        let result = BackupManager::list_backups("definitely_nonexistent_project_12345");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
