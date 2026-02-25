use std::collections::HashSet;
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
        Self::default()
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
        let Some(path) = Self::config_path() else {
            return Self::default();
        };
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(data) => {
                    match serde_json::from_str(&data) {
                        Ok(recent) => return recent,
                        Err(e) => log::warn!("Failed to parse recent projects file: {}", e),
                    }
                }
                Err(e) => log::warn!("Failed to read recent projects file: {}", e),
            }
        }
        Self::new()
    }

    /// Save to config file
    pub fn save(&self) {
        let Some(path) = Self::config_path() else {
            log::warn!("Could not determine config path; skipping recent files save");
            return;
        };
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                log::warn!("Failed to create config dir: {}", e);
                return;
            }
        }
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    log::warn!("Failed to save recent files: {}", e);
                }
            }
            Err(e) => log::warn!("Failed to serialize recent files: {}", e),
        }
    }

    fn config_path() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        Some(home.join(".config").join("scrinever").join("recent.json"))
    }

    /// Get projects opened within the last N days
    pub fn recently_active(&self, days: i64) -> Vec<&RecentProject> {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        self.projects.iter().filter(|p| p.last_opened > cutoff).collect()
    }

    /// Get the total number of unique project paths
    pub fn unique_paths(&self) -> usize {
        self.projects.iter()
            .map(|p| &p.path)
            .collect::<HashSet<_>>()
            .len()
    }

    /// Sort projects by title alphabetically
    pub fn sorted_by_title(&self) -> Vec<&RecentProject> {
        let mut sorted: Vec<&RecentProject> = self.projects.iter().collect();
        sorted.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        sorted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recent_projects_new() {
        let recent = RecentProjects::new();
        assert!(recent.projects.is_empty());
        assert_eq!(recent.count(), 0);
    }

    #[test]
    fn test_add_project() {
        let mut recent = RecentProjects::new();
        recent.add("Test Project", PathBuf::from("/tmp/test.scx"));
        assert_eq!(recent.count(), 1);
        assert_eq!(recent.projects[0].title, "Test Project");
    }

    #[test]
    fn test_add_moves_to_front() {
        let mut recent = RecentProjects::new();
        recent.add("First", PathBuf::from("/tmp/first.scx"));
        recent.add("Second", PathBuf::from("/tmp/second.scx"));
        assert_eq!(recent.projects[0].title, "Second");
    }

    #[test]
    fn test_add_deduplicates_by_path() {
        let mut recent = RecentProjects::new();
        let path = PathBuf::from("/tmp/project.scx");
        recent.add("V1", path.clone());
        recent.add("V2", path.clone());
        assert_eq!(recent.count(), 1);
        assert_eq!(recent.projects[0].title, "V2");
    }

    #[test]
    fn test_remove() {
        let mut recent = RecentProjects::new();
        let path = PathBuf::from("/tmp/test.scx");
        recent.add("Test", path.clone());
        recent.remove(&path);
        assert_eq!(recent.count(), 0);
    }

    #[test]
    fn test_clear() {
        let mut recent = RecentProjects::new();
        recent.add("A", PathBuf::from("/tmp/a.scx"));
        recent.add("B", PathBuf::from("/tmp/b.scx"));
        recent.clear();
        assert_eq!(recent.count(), 0);
    }

    #[test]
    fn test_most_recent() {
        let mut recent = RecentProjects::new();
        assert!(recent.most_recent().is_none());
        recent.add("Latest", PathBuf::from("/tmp/latest.scx"));
        assert_eq!(recent.most_recent().unwrap().title, "Latest");
    }

    #[test]
    fn test_find_by_title() {
        let mut recent = RecentProjects::new();
        recent.add("My Novel", PathBuf::from("/tmp/novel.scx"));
        assert!(recent.find_by_title("my novel").is_some());
        assert!(recent.find_by_title("MY NOVEL").is_some());
        assert!(recent.find_by_title("nope").is_none());
    }

    #[test]
    fn test_contains_path() {
        let mut recent = RecentProjects::new();
        let path = PathBuf::from("/tmp/test.scx");
        recent.add("Test", path.clone());
        assert!(recent.contains_path(&path));
        assert!(!recent.contains_path(&PathBuf::from("/tmp/other.scx")));
    }

    #[test]
    fn test_sorted_by_title() {
        let mut recent = RecentProjects::new();
        recent.add("Zebra", PathBuf::from("/tmp/z.scx"));
        recent.add("Alpha", PathBuf::from("/tmp/a.scx"));
        recent.add("Middle", PathBuf::from("/tmp/m.scx"));
        let sorted = recent.sorted_by_title();
        assert_eq!(sorted[0].title, "Alpha");
        assert_eq!(sorted[1].title, "Middle");
        assert_eq!(sorted[2].title, "Zebra");
    }

    #[test]
    fn test_max_20_entries() {
        let mut recent = RecentProjects::new();
        for i in 0..25 {
            recent.add(&format!("P{}", i), PathBuf::from(format!("/tmp/{}.scx", i)));
        }
        assert_eq!(recent.count(), 20);
    }

    #[test]
    fn test_unique_paths() {
        let mut recent = RecentProjects::new();
        recent.add("A", PathBuf::from("/tmp/a.scx"));
        recent.add("B", PathBuf::from("/tmp/b.scx"));
        assert_eq!(recent.unique_paths(), 2);
    }

    #[test]
    fn test_recent_project_age_string() {
        let project = RecentProject {
            title: "Test".to_string(),
            path: PathBuf::from("/tmp/test.scx"),
            last_opened: Utc::now(),
        };
        assert_eq!(project.age_string(), "just now");
    }

    #[test]
    fn test_recent_project_extension() {
        let project = RecentProject {
            title: "Test".to_string(),
            path: PathBuf::from("/tmp/test.scx"),
            last_opened: Utc::now(),
        };
        assert_eq!(project.extension(), "scx");
    }

    #[test]
    fn test_recent_project_directory_name() {
        let project = RecentProject {
            title: "Test".to_string(),
            path: PathBuf::from("/home/user/projects/test.scx"),
            last_opened: Utc::now(),
        };
        assert_eq!(project.directory_name(), "projects");
    }

    #[test]
    fn test_recent_project_directory_name_root() {
        let project = RecentProject {
            title: "Test".to_string(),
            path: PathBuf::from("/test.scx"),
            last_opened: Utc::now(),
        };
        // Parent of /test.scx is /, which has no file_name
        assert_eq!(project.directory_name(), "");
    }

    #[test]
    fn test_recent_project_extension_none() {
        let project = RecentProject {
            title: "Test".to_string(),
            path: PathBuf::from("/tmp/test"),
            last_opened: Utc::now(),
        };
        assert_eq!(project.extension(), "");
    }

    #[test]
    fn test_recently_active() {
        let mut recent = RecentProjects::new();
        recent.add("Today", PathBuf::from("/tmp/today.scx"));
        let active = recent.recently_active(7);
        assert_eq!(active.len(), 1);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut recent = RecentProjects::new();
        recent.add("Test", PathBuf::from("/tmp/test.scx"));
        let json = serde_json::to_string(&recent).unwrap();
        let parsed: RecentProjects = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.count(), 1);
        assert_eq!(parsed.projects[0].title, "Test");
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut recent = RecentProjects::new();
        recent.add("A", PathBuf::from("/tmp/a.scx"));
        recent.remove(&PathBuf::from("/tmp/nonexistent.scx"));
        assert_eq!(recent.count(), 1);
    }

    #[test]
    fn test_find_by_title_case_insensitive() {
        let mut recent = RecentProjects::new();
        recent.add("My Great Novel", PathBuf::from("/tmp/novel.scx"));
        assert!(recent.find_by_title("my great novel").is_some());
        assert!(recent.find_by_title("MY GREAT NOVEL").is_some());
        assert!(recent.find_by_title("My Great Novel").is_some());
    }
}

/// Wire unused recent items for compilation.
pub fn wire_unused_recent_items() {
    let proj = RecentProject {
        title: "Test".to_string(),
        path: std::path::PathBuf::from("/tmp"),
        last_opened: chrono::Utc::now(),
    };
    let _ = proj.exists();
    let _ = proj.extension();

    let mut recent = RecentProjects::new();
    let _ = recent.recently_active(7);
}
