use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// Entry in the recent projects list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub title: String,
    pub path: PathBuf,
    pub last_opened: DateTime<Utc>,
}

impl RecentProject {
    /// How long ago the project was opened, as a human-readable string
    pub fn age_string(&self) -> String {
        let duration = Utc::now().signed_duration_since(self.last_opened);
        let hours = duration.num_hours();
        if hours < 1 {
            let mins = duration.num_minutes();
            if mins < 1 {
                "just now".to_string()
            } else {
                format!("{}m ago", mins)
            }
        } else if hours < 24 {
            format!("{}h ago", hours)
        } else {
            let days = duration.num_days();
            if days < 7 {
                format!("{}d ago", days)
            } else if days < 30 {
                format!("{}w ago", days / 7)
            } else {
                format!("{}mo ago", days / 30)
            }
        }
    }

    /// Check if the project file still exists on disk
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Get the parent directory name
    pub fn directory_name(&self) -> String {
        self.path
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    /// Get the file extension
    pub fn extension(&self) -> String {
        self.path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default()
    }
}

/// Manages a list of recently opened projects
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecentProjects {
    pub projects: Vec<RecentProject>,
}

impl RecentProjects {
    pub fn new() -> Self {
        Self { projects: Vec::new() }
    }

    pub fn add(&mut self, title: &str, path: PathBuf) {
        // Remove existing entry for this path
        self.projects.retain(|p| p.path != path);

        // Add at the beginning
        self.projects.insert(0, RecentProject {
            title: title.to_string(),
            path,
            last_opened: Utc::now(),
        });

        // Keep only last 20
        self.projects.truncate(20);
    }

    /// Remove a project by path
    pub fn remove(&mut self, path: &PathBuf) {
        self.projects.retain(|p| p.path != *path);
    }

    /// Clear all recent projects
    pub fn clear(&mut self) {
        self.projects.clear();
    }

    /// Remove entries whose files no longer exist on disk
    pub fn prune_missing(&mut self) {
        self.projects.retain(|p| p.path.exists());
    }

    /// Find a project by title (case-insensitive)
    pub fn find_by_title(&self, title: &str) -> Option<&RecentProject> {
        let lower = title.to_lowercase();
        self.projects
            .iter()
            .find(|p| p.title.to_lowercase() == lower)
    }

    /// Number of recent projects
    pub fn count(&self) -> usize {
        self.projects.len()
    }

    /// Check if a path is in the recent list
    pub fn contains_path(&self, path: &PathBuf) -> bool {
        self.projects.iter().any(|p| p.path == *path)
    }

    /// Get the most recently opened project
    pub fn most_recent(&self) -> Option<&RecentProject> {
        self.projects.first()
    }

    /// Load from config file
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(recent) = serde_json::from_str(&data) {
                    return recent;
                }
            }
        }
        Self::new()
    }

    /// Save to config file
    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }

    fn config_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_default();
        home.join(".config").join("scrinever").join("recent.json")
    }
}
