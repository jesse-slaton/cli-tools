use anyhow::{Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use chrono::DateTime;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

/// A backup of PATH environment variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathBackup {
    pub timestamp: String,
    pub user_path: String,
    pub machine_path: String,
    pub user_paths: Vec<String>,
    pub machine_paths: Vec<String>,
}

impl PathBackup {
    pub fn new(
        user_path: String,
        machine_path: String,
        user_paths: Vec<String>,
        machine_paths: Vec<String>,
    ) -> Self {
        Self {
            timestamp: Local::now().to_rfc3339(),
            user_path,
            machine_path,
            user_paths,
            machine_paths,
        }
    }

    /// Save this backup to a file
    pub fn save(&self, directory: &Path) -> Result<PathBuf> {
        // Create backup directory if it doesn't exist
        fs::create_dir_all(directory)?;

        // Generate filename with timestamp
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("path_backup_{}.json", timestamp);
        let filepath = directory.join(filename);

        // Write to file
        let file = File::create(&filepath)
            .with_context(|| format!("Failed to create backup file: {:?}", filepath))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self)
            .with_context(|| "Failed to write backup data")?;

        Ok(filepath)
    }

    /// Load a backup from a file
    pub fn load(filepath: &Path) -> Result<Self> {
        let file = File::open(filepath)
            .with_context(|| format!("Failed to open backup file: {:?}", filepath))?;
        let reader = BufReader::new(file);
        let backup =
            serde_json::from_reader(reader).with_context(|| "Failed to parse backup file")?;
        Ok(backup)
    }

    /// Get a formatted display string for this backup
    #[cfg(test)]
    pub fn display_info(&self) -> String {
        let dt = DateTime::parse_from_rfc3339(&self.timestamp)
            .ok()
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| self.timestamp.clone());

        format!(
            "Backup from {}\nUSER paths: {}\nMACHINE paths: {}",
            dt,
            self.user_paths.len(),
            self.machine_paths.len()
        )
    }
}

/// Get the default backup directory
///
/// Returns ~/.pc/backups/ (or ~/.pathcommand/backups/ as fallback)
pub fn get_default_backup_dir() -> PathBuf {
    crate::config::get_backups_dir().unwrap_or_else(|_| {
        // Fallback to old location if config directory can't be determined
        let mut path = dirs::data_local_dir()
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        path.push("PathCommander");
        path.push("backups");
        path
    })
}

/// List all backup files in a directory
pub fn list_backups(directory: &Path) -> Result<Vec<PathBuf>> {
    if !directory.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "json" {
                    if let Some(name) = path.file_name() {
                        if name.to_string_lossy().starts_with("path_backup_") {
                            backups.push(path);
                        }
                    }
                }
            }
        }
    }

    // Sort by modification time, newest first
    backups.sort_by_key(|path| {
        fs::metadata(path)
            .and_then(|m| m.modified())
            .map(std::cmp::Reverse)
            .ok()
    });

    Ok(backups)
}

/// Delete old backups, keeping only the most recent N backups
#[cfg(test)]
pub fn cleanup_old_backups(directory: &Path, keep_count: usize) -> Result<usize> {
    let mut backups = list_backups(directory)?;

    if backups.len() <= keep_count {
        return Ok(0);
    }

    let to_delete = backups.split_off(keep_count);
    let deleted_count = to_delete.len();

    for backup in to_delete {
        fs::remove_file(backup)?;
    }

    Ok(deleted_count)
}

// Re-export dirs crate functionality or provide alternative
mod dirs {
    use std::path::PathBuf;

    pub fn data_local_dir() -> Option<PathBuf> {
        std::env::var("LOCALAPPDATA").ok().map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_backup_create() {
        let backup = PathBackup::new(
            r"C:\User\Path".to_string(),
            r"C:\Machine\Path".to_string(),
            vec![r"C:\User\Path".to_string()],
            vec![r"C:\Machine\Path".to_string()],
        );

        assert!(!backup.timestamp.is_empty());
        assert_eq!(backup.user_paths.len(), 1);
        assert_eq!(backup.machine_paths.len(), 1);
        assert_eq!(backup.user_path, r"C:\User\Path");
        assert_eq!(backup.machine_path, r"C:\Machine\Path");
    }

    #[test]
    fn test_backup_create_empty_paths() {
        let backup = PathBackup::new(String::new(), String::new(), vec![], vec![]);

        assert!(!backup.timestamp.is_empty());
        assert_eq!(backup.user_paths.len(), 0);
        assert_eq!(backup.machine_paths.len(), 0);
    }

    #[test]
    fn test_backup_create_multiple_paths() {
        let user_paths = vec![
            r"C:\User\Path1".to_string(),
            r"C:\User\Path2".to_string(),
            r"C:\User\Path3".to_string(),
        ];
        let machine_paths = vec![r"C:\Windows".to_string(), r"C:\Program Files".to_string()];

        let backup = PathBackup::new(
            user_paths.join(";"),
            machine_paths.join(";"),
            user_paths.clone(),
            machine_paths.clone(),
        );

        assert_eq!(backup.user_paths.len(), 3);
        assert_eq!(backup.machine_paths.len(), 2);
    }

    #[test]
    fn test_backup_save_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("new_directory");

        let backup = PathBackup::new(
            r"C:\Test".to_string(),
            r"C:\Test".to_string(),
            vec![r"C:\Test".to_string()],
            vec![r"C:\Test".to_string()],
        );

        let filepath = backup.save(&backup_dir).unwrap();
        assert!(filepath.exists());
        assert!(backup_dir.exists());
    }

    #[test]
    fn test_backup_roundtrip() {
        let temp_dir = TempDir::new().unwrap();

        let backup = PathBackup::new(
            r"C:\User\Path".to_string(),
            r"C:\Machine\Path".to_string(),
            vec![r"C:\User\Path".to_string()],
            vec![r"C:\Machine\Path".to_string()],
        );

        let filepath = backup.save(temp_dir.path()).unwrap();

        let loaded = PathBackup::load(&filepath).unwrap();
        assert_eq!(loaded.user_path, backup.user_path);
        assert_eq!(loaded.machine_path, backup.machine_path);
        assert_eq!(loaded.user_paths, backup.user_paths);
        assert_eq!(loaded.machine_paths, backup.machine_paths);
    }

    #[test]
    fn test_backup_filename_format() {
        let temp_dir = TempDir::new().unwrap();

        let backup = PathBackup::new(String::new(), String::new(), vec![], vec![]);

        let filepath = backup.save(temp_dir.path()).unwrap();
        let filename = filepath.file_name().unwrap().to_string_lossy();

        // Should match pattern: path_backup_YYYYMMDD_HHMMSS.json
        assert!(filename.starts_with("path_backup_"));
        assert!(filename.ends_with(".json"));
    }

    #[test]
    fn test_backup_load_nonexistent() {
        let result = PathBackup::load(Path::new("nonexistent_file.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_backup_load_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_file = temp_dir.path().join("invalid.json");
        fs::write(&invalid_file, "not valid json").unwrap();

        let result = PathBackup::load(&invalid_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_backup_display_info() {
        let backup = PathBackup::new(
            r"C:\User\Path1;C:\User\Path2".to_string(),
            r"C:\Windows;C:\Program Files".to_string(),
            vec![r"C:\User\Path1".to_string(), r"C:\User\Path2".to_string()],
            vec![r"C:\Windows".to_string(), r"C:\Program Files".to_string()],
        );

        let info = backup.display_info();
        assert!(info.contains("Backup from"));
        assert!(info.contains("USER paths: 2"));
        assert!(info.contains("MACHINE paths: 2"));
    }

    #[test]
    fn test_list_backups_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let backups = list_backups(temp_dir.path()).unwrap();
        assert_eq!(backups.len(), 0);
    }

    #[test]
    fn test_list_backups_nonexistent_directory() {
        let nonexistent = Path::new("nonexistent_directory_12345");
        let backups = list_backups(nonexistent).unwrap();
        assert_eq!(backups.len(), 0);
    }

    #[test]
    fn test_list_backups_multiple() {
        let temp_dir = TempDir::new().unwrap();

        // Create multiple backups (filename uses second precision, so wait 1 second between)
        for _ in 0..2 {
            let backup = PathBackup::new(String::new(), String::new(), vec![], vec![]);
            backup.save(temp_dir.path()).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(1100)); // Ensure different timestamps
        }

        let backups = list_backups(temp_dir.path()).unwrap();
        assert_eq!(backups.len(), 2);

        // Verify all are valid backup files
        for backup_path in &backups {
            let filename = backup_path.file_name().unwrap().to_string_lossy();
            assert!(filename.starts_with("path_backup_"));
        }
    }

    #[test]
    fn test_list_backups_ignores_other_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create a backup
        let backup = PathBackup::new(String::new(), String::new(), vec![], vec![]);
        backup.save(temp_dir.path()).unwrap();

        // Create other files that should be ignored
        fs::write(temp_dir.path().join("other.json"), "{}").unwrap();
        fs::write(temp_dir.path().join("readme.txt"), "text").unwrap();

        let backups = list_backups(temp_dir.path()).unwrap();
        assert_eq!(backups.len(), 1); // Only the real backup
    }

    #[test]
    fn test_cleanup_old_backups_keep_all() {
        let temp_dir = TempDir::new().unwrap();

        // Create 2 backups (filename uses second precision, so wait 1 second between)
        for _ in 0..2 {
            let backup = PathBackup::new(String::new(), String::new(), vec![], vec![]);
            backup.save(temp_dir.path()).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(1100));
        }

        // Keep 5 (more than we have)
        let deleted = cleanup_old_backups(temp_dir.path(), 5).unwrap();
        assert_eq!(deleted, 0);

        let backups = list_backups(temp_dir.path()).unwrap();
        assert_eq!(backups.len(), 2);
    }

    #[test]
    fn test_cleanup_old_backups_delete_some() {
        let temp_dir = TempDir::new().unwrap();

        // Create 3 backups (filename uses second precision, so wait 1 second between)
        for _ in 0..3 {
            let backup = PathBackup::new(String::new(), String::new(), vec![], vec![]);
            backup.save(temp_dir.path()).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(1100));
        }

        // Keep only 1 most recent
        let deleted = cleanup_old_backups(temp_dir.path(), 1).unwrap();
        assert_eq!(deleted, 2);

        let backups = list_backups(temp_dir.path()).unwrap();
        assert_eq!(backups.len(), 1);
    }

    #[test]
    fn test_cleanup_old_backups_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let deleted = cleanup_old_backups(temp_dir.path(), 5).unwrap();
        assert_eq!(deleted, 0);
    }

    #[test]
    fn test_get_default_backup_dir() {
        let dir = get_default_backup_dir();
        let path_str = dir.to_string_lossy();
        // Should use new ~/.pc/backups/ location (or fallback)
        assert!(
            path_str.contains(".pc")
                || path_str.contains(".pathcommand")
                || path_str.contains("PathCommander")
        ); // Fallback
        assert!(path_str.contains("backups"));
    }

    #[test]
    fn test_backup_with_special_characters() {
        let temp_dir = TempDir::new().unwrap();

        let paths_with_special_chars = vec![
            r"C:\Program Files (x86)\Test".to_string(),
            r"C:\Users\Test User\Documents".to_string(),
            r"C:\Path with spaces\and-dashes_and_underscores".to_string(),
        ];

        let backup = PathBackup::new(
            paths_with_special_chars.join(";"),
            String::new(),
            paths_with_special_chars.clone(),
            vec![],
        );

        let filepath = backup.save(temp_dir.path()).unwrap();
        let loaded = PathBackup::load(&filepath).unwrap();

        assert_eq!(loaded.user_paths, paths_with_special_chars);
    }

    #[test]
    fn test_backup_with_very_long_paths() {
        let temp_dir = TempDir::new().unwrap();

        // Create a very long path (but within Windows limits)
        let long_path = format!(r"C:\{}\Test", "VeryLongDirectoryName".repeat(10));

        let backup = PathBackup::new(
            long_path.clone(),
            String::new(),
            vec![long_path.clone()],
            vec![],
        );

        let filepath = backup.save(temp_dir.path()).unwrap();
        let loaded = PathBackup::load(&filepath).unwrap();

        assert_eq!(loaded.user_paths[0], long_path);
    }
}
