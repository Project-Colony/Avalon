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
