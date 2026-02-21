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
        if let Ok(dir_entries) = fs::read_dir(&backup_dir) {
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
                        name: name.clone(),
                        timestamp: timestamp_str.to_string(),
                        size_bytes: size,
                    });
                }
            }
        }

        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(entries)
    }

    /// Get the backup directory path
    fn backup_directory() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .context("Could not determine home directory")?;
        Ok(home.join("Scrinever Backups"))
    }

    /// Remove old backups, keeping only the most recent `keep` backups
    fn prune_backups(backup_dir: &Path, project_name: &str, keep: usize) -> Result<()> {
        let mut backups = Self::list_backups(project_name)?;
        if backups.len() > keep {
            // Remove oldest backups
            let to_remove = backups.split_off(keep);
            for entry in to_remove {
                let _ = fs::remove_file(&entry.path);
            }
        }
        let _ = backup_dir; // suppress unused warning
        Ok(())
    }
}

/// A single backup entry
#[derive(Debug, Clone)]
pub struct BackupEntry {
    pub path: PathBuf,
    pub name: String,
    pub timestamp: String,
    pub size_bytes: u64,
}

impl BackupEntry {
    pub fn display_size(&self) -> String {
        if self.size_bytes < 1024 {
            format!("{} B", self.size_bytes)
        } else if self.size_bytes < 1024 * 1024 {
            format!("{:.1} KB", self.size_bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", self.size_bytes as f64 / (1024.0 * 1024.0))
        }
    }

    pub fn display_timestamp(&self) -> String {
        // Parse YYYYMMDD_HHMMSS format
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
