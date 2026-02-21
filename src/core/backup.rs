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
        } else if self.size_bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", self.size_bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", self.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0))
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

    /// Parse the timestamp into a chrono NaiveDateTime
    pub fn parsed_timestamp(&self) -> Option<chrono::NaiveDateTime> {
        if self.timestamp.len() >= 15 {
            let date = chrono::NaiveDate::from_ymd_opt(
                self.timestamp[0..4].parse().ok()?,
                self.timestamp[4..6].parse().ok()?,
                self.timestamp[6..8].parse().ok()?,
            )?;
            let time = chrono::NaiveTime::from_hms_opt(
                self.timestamp[9..11].parse().ok()?,
                self.timestamp[11..13].parse().ok()?,
                self.timestamp[13..15].parse().ok()?,
            )?;
            Some(chrono::NaiveDateTime::new(date, time))
        } else {
            None
        }
    }

    /// Age as human-readable string
    pub fn age_string(&self) -> String {
        if let Some(ts) = self.parsed_timestamp() {
            let now = chrono::Utc::now().naive_utc();
            let duration = now.signed_duration_since(ts);
            let hours = duration.num_hours();
            if hours < 1 {
                format!("{}m ago", duration.num_minutes().max(1))
            } else if hours < 24 {
                format!("{}h ago", hours)
            } else {
                let days = duration.num_days();
                if days < 7 {
                    format!("{}d ago", days)
                } else {
                    format!("{}w ago", days / 7)
                }
            }
        } else {
            String::new()
        }
    }

    /// Check if the backup file exists on disk
    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

impl BackupManager {
    /// Delete a specific backup
    pub fn delete_backup(backup_path: &Path) -> Result<()> {
        fs::remove_file(backup_path)
            .context("Failed to delete backup")?;
        Ok(())
    }

    /// Get total size of all backups for a project
    pub fn total_backup_size(project_name: &str) -> Result<u64> {
        let backups = Self::list_backups(project_name)?;
        Ok(backups.iter().map(|b| b.size_bytes).sum())
    }

    /// Get the number of backups for a project
    pub fn backup_count(project_name: &str) -> Result<usize> {
        let backups = Self::list_backups(project_name)?;
        Ok(backups.len())
    }

    /// Get the most recent backup for a project
    pub fn latest_backup(project_name: &str) -> Result<Option<BackupEntry>> {
        let backups = Self::list_backups(project_name)?;
        Ok(backups.into_iter().next())
    }

    /// Get the backup directory path (public accessor)
    pub fn backup_dir_path() -> Result<PathBuf> {
        Self::backup_directory()
    }

    /// Get total backup size as a human-readable string
    pub fn total_backup_size_display(project_name: &str) -> Result<String> {
        let total = Self::total_backup_size(project_name)?;
        Ok(BackupEntry::format_bytes(total))
    }
}

impl BackupEntry {
    /// Format bytes into human-readable size string
    pub fn format_bytes(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Check if this backup is from today
    pub fn is_from_today(&self) -> bool {
        if let Some(ts) = self.parsed_timestamp() {
            let today = chrono::Utc::now().naive_utc().date();
            ts.date() == today
        } else {
            false
        }
    }

    /// Get the project name from the backup filename
    pub fn project_name(&self) -> String {
        self.name
            .rsplit_once('_')
            .and_then(|(prefix, _)| prefix.rsplit_once('_'))
            .map(|(name, _)| name.to_string())
            .unwrap_or_else(|| self.name.clone())
    }
}
