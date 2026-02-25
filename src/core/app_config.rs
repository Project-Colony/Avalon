#![allow(dead_code)] // Methods used by GUI code
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application-level persistent configuration.
///
/// Unlike `ProjectSettings` (which is per-project), `AppConfig` stores
/// user preferences that persist across projects and sessions, such as
/// daily writing goals, UI preferences, and last-used compile settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Daily word count goal (0 = disabled)
    #[serde(default)]
    pub daily_goal: usize,
    /// Weekly word count goal (0 = disabled)
    #[serde(default)]
    pub weekly_goal: usize,
    /// Session word count goal (0 = disabled)
    #[serde(default)]
    pub session_goal: usize,

    /// Whether composition (distraction-free) mode was last active
    #[serde(default)]
    pub composition_mode: bool,
    /// Whether the inspector panel was visible
    #[serde(default = "default_true")]
    pub show_inspector: bool,

    /// Last-used export format name
    #[serde(default)]
    pub last_export_format: String,

    /// Search preferences
    #[serde(default)]
    pub search_case_sensitive: bool,
    #[serde(default)]
    pub search_whole_word: bool,

    /// Name generator type preference
    #[serde(default = "default_name_gen_type")]
    pub name_gen_type: String,

    /// Window width (0 = system default)
    #[serde(default)]
    pub window_width: u32,
    /// Window height (0 = system default)
    #[serde(default)]
    pub window_height: u32,
}

fn default_true() -> bool {
    true
}

fn default_name_gen_type() -> String {
    "fantasy".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            daily_goal: 0,
            weekly_goal: 0,
            session_goal: 500,
            composition_mode: false,
            show_inspector: true,
            last_export_format: String::new(),
            search_case_sensitive: false,
            search_whole_word: false,
            name_gen_type: default_name_gen_type(),
            window_width: 0,
            window_height: 0,
        }
    }
}

impl AppConfig {
    /// Load config from the standard config path.
    /// Returns default config if the file doesn't exist or can't be parsed.
    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };
        if !path.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(&path) {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(config) => config,
                Err(e) => {
                    log::warn!("Failed to parse app config: {}", e);
                    Self::default()
                }
            },
            Err(e) => {
                log::warn!("Failed to read app config: {}", e);
                Self::default()
            }
        }
    }

    /// Save config to the standard config path.
    pub fn save(&self) {
        let Some(path) = Self::config_path() else {
            log::warn!("Could not determine config path; skipping app config save");
            return;
        };
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                log::warn!("Failed to create config directory: {}", e);
                return;
            }
        }
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    log::warn!("Failed to save app config: {}", e);
                }
            }
            Err(e) => log::warn!("Failed to serialize app config: {}", e),
        }
    }

    fn config_path() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        Some(home.join(".config").join("scrinever").join("config.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.daily_goal, 0);
        assert_eq!(config.weekly_goal, 0);
        assert_eq!(config.session_goal, 500);
        assert!(config.show_inspector);
        assert!(!config.composition_mode);
        assert!(!config.search_case_sensitive);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut config = AppConfig::default();
        config.daily_goal = 1000;
        config.composition_mode = true;

        let json = serde_json::to_string(&config).unwrap();
        let parsed: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.daily_goal, 1000);
        assert!(parsed.composition_mode);
    }

    #[test]
    fn test_deserialization_with_missing_fields() {
        // Simulate an older config file with fewer fields
        let json = r#"{"daily_goal": 500}"#;
        let config: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.daily_goal, 500);
        assert_eq!(config.weekly_goal, 0); // default
        assert!(config.show_inspector); // default true
    }

    #[test]
    fn test_load_nonexistent() {
        // load() should return default when file doesn't exist
        let config = AppConfig::load();
        assert_eq!(config.daily_goal, 0);
    }
}
