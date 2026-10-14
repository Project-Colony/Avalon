use anyhow::{bail, ensure, Context, Result};
use chrono::Utc;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use super::metadata::ProjectSettings;
use super::project::Project;
use super::write_atomic;

/// Backups kept per project. Older ones are deleted after each new backup.
pub const KEEP_BACKUPS: usize = 20;

/// Timestamp in backup file names, in UTC.
const TIMESTAMP_FORMAT: &str = "%Y%m%d_%H%M%S";

/// Most entries a backup archive may list on restore: one per document plus
/// `project.json` and the compile presets.
const MAX_ARCHIVE_ENTRIES: usize = 10_000;

/// Largest single file a restore inflates. A long chapter is a few MiB.
const MAX_ENTRY_BYTES: u64 = 64 * 1024 * 1024;

/// Largest total a restore inflates, so a zip bomb cannot fill the disk.
const MAX_TOTAL_BYTES: u64 = 1024 * 1024 * 1024;

/// Backup manager for Avalon projects.
///
/// A backup is one zip archive, `<project folder name>_<UTC timestamp>.zip`,
/// holding `project.json`, the `docs/` text files and `compile_presets.json`.
/// Snapshots are left out to keep backups small.
pub struct BackupManager;

impl BackupManager {
    /// The folder backups go to: `~/Scrinever Backups`.
    pub fn default_root() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home.join(super::BACKUPS_DIR_NAME))
    }

    /// Zip the project saved in `project_dir` into `root`, keep the newest
    /// [`KEEP_BACKUPS`] backups of that project, and return the new archive.
    pub fn create_backup(project_dir: &Path, root: &Path) -> Result<PathBuf> {
        let project_json = project_dir.join("project.json");
        ensure!(
            project_json.is_file(),
            "{} is not a saved project: it has no project.json",
            project_dir.display()
        );

        let mut files = vec![("project.json".to_string(), project_json)];
        let presets = project_dir.join("compile_presets.json");
        if presets.is_file() {
            files.push(("compile_presets.json".to_string(), presets));
        }
        let docs_dir = project_dir.join("docs");
        if docs_dir.is_dir() {
            let mut docs = Vec::new();
            for entry in fs::read_dir(&docs_dir).context("Failed to read the docs folder")? {
                let entry = entry?;
                let name = entry.file_name();
                // Leftover `.tmp` files from an interrupted save are not documents.
                if let Some(name) = name.to_str().filter(|n| is_doc_file_name(n)) {
                    if entry.file_type()?.is_file() {
                        docs.push((format!("docs/{name}"), entry.path()));
                    }
                }
            }
            docs.sort();
            files.extend(docs);
        }

        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        for (name, path) in &files {
            let data = fs::read(path).with_context(|| format!("Failed to read {}", path.display()))?;
            zip.start_file(name.as_str(), options)?;
            zip.write_all(&data)?;
        }
        let archive = zip.finish().context("Failed to build the backup archive")?.into_inner();

        fs::create_dir_all(root).context("Failed to create the backup folder")?;
        let name = format!(
            "{}_{}.zip",
            backup_stem(project_dir),
            Utc::now().format(TIMESTAMP_FORMAT)
        );
        let backup_path = root.join(name);
        write_atomic(&backup_path, &archive).context("Failed to write the backup")?;

        Self::prune_backups(root, project_dir, KEEP_BACKUPS)?;
        Ok(backup_path)
    }

    /// Count one save of the project in `project_dir` and back it up into
    /// `root` when its settings ask for it: automatic backups are on and
    /// `backup_interval_saves` saves happened since the last backup.
    ///
    /// `saves_since_backup` carries the count from one call to the next. It
    /// is reset once a backup is written, so a failed backup is retried on
    /// the next save. Returns the new backup, if one was due.
    pub fn backup_after_save(
        project_dir: &Path,
        root: &Path,
        settings: &ProjectSettings,
        saves_since_backup: &mut u32,
    ) -> Result<Option<PathBuf>> {
        if !settings.auto_backup {
            *saves_since_backup = 0;
            return Ok(None);
        }
        *saves_since_backup = saves_since_backup.saturating_add(1);
        if *saves_since_backup < settings.backup_interval_saves.max(1) {
            return Ok(None);
        }
        let backup = Self::create_backup(project_dir, root)?;
        *saves_since_backup = 0;
        Ok(Some(backup))
    }

    /// Unpack the backup at `backup_path` into a new folder next to
    /// `project_dir` and return that folder. The copy gets a project id of its
    /// own, and the project in `project_dir` is left as it is.
    ///
    /// The archive may only hold `project.json`, `compile_presets.json` and
    /// `docs/<document id>.json`; any other entry, too many entries or too
    /// much data rejects the whole backup and removes the new folder.
    pub fn restore_backup(backup_path: &Path, project_dir: &Path) -> Result<PathBuf> {
        let file = fs::File::open(backup_path).with_context(|| format!("Failed to open {}", backup_path.display()))?;
        let mut archive = ZipArchive::new(file).context("The backup is not a valid zip archive")?;
        ensure!(
            archive.len() <= MAX_ARCHIVE_ENTRIES,
            "The backup lists {} entries, more than the {} allowed",
            archive.len(),
            MAX_ARCHIVE_ENTRIES
        );

        let target = Self::new_restore_folder(backup_path, project_dir)?;
        let result = Self::extract(&mut archive, &target).and_then(|()| {
            // The copy is a project of its own. A new id keeps a file dialog
            // opened for the original from landing in it.
            let mut restored = Project::load(&target)?;
            restored.id = Uuid::new_v4();
            restored.save()
        });
        if let Err(e) = result {
            if let Err(cleanup) = fs::remove_dir_all(&target) {
                log::warn!("Failed to remove {}: {}", target.display(), cleanup);
            }
            return Err(e);
        }
        Ok(target)
    }

    /// Create the folder a restore writes to: `<project> restored <timestamp>`
    /// next to `project_dir`, with a number appended when that name is taken.
    fn new_restore_folder(backup_path: &Path, project_dir: &Path) -> Result<PathBuf> {
        let parent = project_dir.parent().unwrap_or(Path::new("."));
        let stamp = backup_path
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.get(s.len().saturating_sub(15)..))
            .filter(|s| is_timestamp(s))
            .map(str::to_string)
            .unwrap_or_else(|| Utc::now().format(TIMESTAMP_FORMAT).to_string());
        let extension = project_dir
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{e}"))
            .unwrap_or_default();
        let base = format!("{} restored {}", backup_stem(project_dir), stamp);

        for n in 1..=100 {
            let name = if n == 1 {
                format!("{base}{extension}")
            } else {
                format!("{base} ({n}){extension}")
            };
            let dir = parent.join(name);
            match fs::create_dir(&dir) {
                Ok(()) => return Ok(dir),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e).with_context(|| format!("Failed to create {}", dir.display())),
            }
        }
        bail!(
            "Too many restored copies of this backup next to {}",
            project_dir.display()
        )
    }

    /// Write every entry of `archive` into the empty folder `target`.
    fn extract<R: Read + std::io::Seek>(archive: &mut ZipArchive<R>, target: &Path) -> Result<()> {
        fs::create_dir(target.join("docs")).context("Failed to create the docs folder")?;
        let mut budget = MAX_TOTAL_BYTES;
        for i in 0..archive.len() {
            let entry = archive.by_index(i).context("Failed to read the backup archive")?;
            let name = entry.name().to_string();
            let Some(path) = entry_path(&name) else {
                bail!("The backup holds {:?}, which is not part of a project", name);
            };
            // Read one byte past the cap: the declared size can lie, the byte count cannot.
            let cap = MAX_ENTRY_BYTES.min(budget);
            let mut data = Vec::new();
            entry
                .take(cap + 1)
                .read_to_end(&mut data)
                .with_context(|| format!("Failed to read {name} from the backup"))?;
            if data.len() as u64 > cap {
                if cap < MAX_ENTRY_BYTES {
                    bail!("The backup holds more than {}", super::format_bytes(MAX_TOTAL_BYTES));
                }
                bail!("{name} in the backup is larger than {}", super::format_bytes(cap));
            }
            budget -= data.len() as u64;
            fs::write(target.join(&path), &data).with_context(|| format!("Failed to write {}", path.display()))?;
        }
        Ok(())
    }

    /// List the backups of the project in `project_dir` found in `root`,
    /// newest first.
    pub fn list_backups(root: &Path, project_dir: &Path) -> Result<Vec<BackupEntry>> {
        let mut entries = Vec::new();
        if !root.exists() {
            return Ok(entries);
        }

        let prefix = format!("{}_", backup_stem(project_dir));
        for entry in fs::read_dir(root).context("Failed to read backup directory")?.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let Some(timestamp) = name
                .strip_prefix(&prefix)
                .and_then(|s| s.strip_suffix(".zip"))
                .filter(|s| is_timestamp(s))
            else {
                continue;
            };
            entries.push(BackupEntry {
                timestamp: timestamp.to_string(),
                size_bytes: entry.metadata().map(|m| m.len()).unwrap_or(0),
                path,
            });
        }

        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(entries)
    }

    /// Remove old backups, keeping only the most recent `keep` backups
    fn prune_backups(root: &Path, project_dir: &Path, keep: usize) -> Result<()> {
        let mut backups = Self::list_backups(root, project_dir)?;
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

/// The name backups of the project in `project_dir` start with: its folder
/// name without the extension.
fn backup_stem(project_dir: &Path) -> &str {
    project_dir.file_stem().and_then(|s| s.to_str()).unwrap_or("project")
}

/// Whether `s` is a `YYYYMMDD_HHMMSS` timestamp.
fn is_timestamp(s: &str) -> bool {
    s.len() == 15
        && s.bytes()
            .enumerate()
            .all(|(i, b)| if i == 8 { b == b'_' } else { b.is_ascii_digit() })
}

/// Whether `name` is a document file as the project writes it:
/// `<lowercase hyphenated UUID>.json`.
fn is_doc_file_name(name: &str) -> bool {
    name.strip_suffix(".json")
        .is_some_and(|id| Uuid::try_parse(id).is_ok_and(|uuid| uuid.to_string() == id))
}

/// Where the archive entry `name` goes inside a restored project, or `None`
/// for anything that is not a project file. Only exact names are accepted, so
/// no entry can reach outside the restored folder.
fn entry_path(name: &str) -> Option<PathBuf> {
    match name.split_once('/') {
        None if name == "project.json" || name == "compile_presets.json" => Some(PathBuf::from(name)),
        Some(("docs", doc)) if is_doc_file_name(doc) => Some(Path::new("docs").join(doc)),
        _ => None,
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
    fn test_list_backups_missing_root() {
        let dir = tempfile::tempdir().unwrap();
        let result = BackupManager::list_backups(&dir.path().join("none"), Path::new("Novel.scriv"));
        assert!(result.unwrap().is_empty());
    }

    use crate::core::binder::BinderItem;

    /// Save a project holding one chapter with `text` into `<base>/Novel.scriv`.
    fn saved_project(base: &Path, text: &str) -> (PathBuf, Uuid) {
        let mut project = Project::new("Novel");
        let mut item = BinderItem::new_text("Chapter 1");
        item.document.as_mut().unwrap().content = text.to_string();
        let id = item.id;
        project.binder.draft.children.push(item);
        let dir = base.join("Novel.scriv");
        project.save_as(&dir).unwrap();
        (dir, id)
    }

    fn chapter_text(project_dir: &Path, id: Uuid) -> String {
        let project = Project::load(project_dir).unwrap();
        project
            .binder
            .find_item(&id)
            .unwrap()
            .document
            .as_ref()
            .unwrap()
            .content
            .clone()
    }

    #[test]
    fn test_backup_and_restore_round_trip_the_text() {
        let base = tempfile::tempdir().unwrap();
        let root = base.path().join("backups");
        let (dir, id) = saved_project(base.path(), "It was a dark and stormy night.");

        let backup = BackupManager::create_backup(&dir, &root).unwrap();
        assert_eq!(BackupManager::list_backups(&root, &dir).unwrap()[0].path, backup);

        // Change the live project after the backup.
        let mut live = Project::load(&dir).unwrap();
        live.binder
            .find_item_mut(&id)
            .unwrap()
            .document
            .as_mut()
            .unwrap()
            .content = "Rewritten".into();
        live.save().unwrap();

        let restored = BackupManager::restore_backup(&backup, &dir).unwrap();
        assert_ne!(restored, dir);
        assert_eq!(restored.parent(), dir.parent());
        assert_eq!(chapter_text(&restored, id), "It was a dark and stormy night.");
        assert_eq!(chapter_text(&dir, id), "Rewritten");
        assert_ne!(Project::load(&restored).unwrap().id, live.id);

        // A second restore of the same backup gets its own folder.
        let again = BackupManager::restore_backup(&backup, &dir).unwrap();
        assert_ne!(again, restored);
    }

    #[test]
    fn test_prune_keeps_the_newest_twenty() {
        let base = tempfile::tempdir().unwrap();
        let root = base.path().join("backups");
        let (dir, _) = saved_project(base.path(), "Text");
        fs::create_dir_all(&root).unwrap();
        for day in 1..=24 {
            fs::write(root.join(format!("Novel_202001{day:02}_000000.zip")), b"old").unwrap();
        }
        // Another project's backup and a stray file are not this project's to prune.
        let other = root.join("Novel 2_20200101_000000.zip");
        fs::write(&other, b"other").unwrap();
        let stray = root.join("Novel_notes.zip");
        fs::write(&stray, b"stray").unwrap();

        let newest = BackupManager::create_backup(&dir, &root).unwrap();

        let kept = BackupManager::list_backups(&root, &dir).unwrap();
        assert_eq!(kept.len(), KEEP_BACKUPS);
        assert_eq!(kept[0].path, newest);
        assert_eq!(kept.last().unwrap().timestamp, "20200106_000000");
        assert!(!root.join("Novel_20200105_000000.zip").exists());
        assert!(other.exists() && stray.exists());
    }

    #[test]
    fn test_auto_backup_honours_the_settings() {
        let base = tempfile::tempdir().unwrap();
        let root = base.path().join("backups");
        let (dir, _) = saved_project(base.path(), "Text");
        let mut settings = ProjectSettings {
            auto_backup: false,
            backup_interval_saves: 1,
            ..ProjectSettings::default()
        };
        let mut saves = 0;

        for _ in 0..5 {
            assert!(BackupManager::backup_after_save(&dir, &root, &settings, &mut saves)
                .unwrap()
                .is_none());
        }
        assert!(!root.exists());

        settings.auto_backup = true;
        settings.backup_interval_saves = 3;
        let made: Vec<_> = (0..3)
            .map(|_| BackupManager::backup_after_save(&dir, &root, &settings, &mut saves).unwrap())
            .collect();
        assert!(made[0].is_none() && made[1].is_none());
        assert!(made[2].as_ref().is_some_and(|p| p.is_file()));
        assert_eq!(saves, 0);
    }

    /// Zip `entries` into `<base>/hostile_20200101_000000.zip`.
    fn archive(base: &Path, entries: &[(&str, &[u8])]) -> PathBuf {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, data) in entries {
            zip.start_file(*name, SimpleFileOptions::default()).unwrap();
            zip.write_all(data).unwrap();
        }
        let path = base.join("hostile_20200101_000000.zip");
        fs::write(&path, zip.finish().unwrap().into_inner()).unwrap();
        path
    }

    #[test]
    fn test_restore_rejects_hostile_entry_names() {
        let base = tempfile::tempdir().unwrap();
        let (dir, _) = saved_project(base.path(), "Text");
        let doc = format!("docs/{}.json", Uuid::new_v4());
        let hostile = [
            "../evil.json",
            "/evil.json",
            "docs/../../evil.json",
            "docs/../evil.json",
            "docs\\..\\..\\evil.json",
            "C:/evil.json",
            "docs/sub/evil.json",
            "docs/evil.json",
            "snapshots/evil.json",
            "docs/",
        ];
        for name in hostile {
            let backup = archive(base.path(), &[("project.json", b"{}"), (&doc, b"{}"), (name, b"{}")]);
            let err = BackupManager::restore_backup(&backup, &dir).unwrap_err();
            assert!(err.to_string().contains("not part of a project"), "{name}: {err}");
        }

        assert!(!base.path().join("evil.json").exists());
        let mut left: Vec<_> = fs::read_dir(base.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        left.sort();
        assert_eq!(left, ["Novel.scriv", "hostile_20200101_000000.zip"]);
    }

    #[test]
    fn test_restore_needs_project_json() {
        let base = tempfile::tempdir().unwrap();
        let (dir, _) = saved_project(base.path(), "Text");
        let backup = archive(base.path(), &[("compile_presets.json", b"[]")]);
        let err = BackupManager::restore_backup(&backup, &dir).unwrap_err();
        assert!(err.to_string().contains("project.json"), "{err}");
    }

    #[test]
    fn test_restore_rejects_too_many_entries() {
        let base = tempfile::tempdir().unwrap();
        let (dir, _) = saved_project(base.path(), "Text");
        let names: Vec<String> = (0..=MAX_ARCHIVE_ENTRIES).map(|i| format!("{i}")).collect();
        let entries: Vec<(&str, &[u8])> = names.iter().map(|n| (n.as_str(), &b""[..])).collect();
        let backup = archive(base.path(), &entries);
        let err = BackupManager::restore_backup(&backup, &dir).unwrap_err();
        assert!(err.to_string().contains("entries"), "{err}");
    }

    #[test]
    fn test_doc_file_names() {
        let id = Uuid::new_v4();
        assert!(is_doc_file_name(&format!("{id}.json")));
        assert!(!is_doc_file_name(&format!("{id}.json.tmp")));
        assert!(!is_doc_file_name(&format!("{}.json", id.simple())));
        assert!(!is_doc_file_name(&format!("{}.json", id.urn())));
        assert!(!is_doc_file_name(&format!("{}.json", id.to_string().to_uppercase())));
    }
}
