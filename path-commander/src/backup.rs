use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
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
    pub fn new(user_path: String, machine_path: String, user_paths: Vec<String>, machine_paths: Vec<String>) -> Self {
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
        let backup = serde_json::from_reader(reader)
            .with_context(|| "Failed to parse backup file")?;
        Ok(backup)
    }

    /// Get a formatted display string for this backup
    pub fn display_info(&self) -> String {
        let dt = DateTime::parse_from_rfc3339(&self.timestamp)
            .ok()
            .and_then(|dt| Some(dt.format("%Y-%m-%d %H:%M:%S").to_string()))
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
pub fn get_default_backup_dir() -> PathBuf {
    let mut path = dirs::data_local_dir()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    path.push("PathCommander");
    path.push("backups");
    path
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
            .map(|t| std::cmp::Reverse(t))
            .ok()
    });

    Ok(backups)
}

/// Delete old backups, keeping only the most recent N backups
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
        std::env::var("LOCALAPPDATA")
            .ok()
            .map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn test_backup_roundtrip() {
        use std::env;

        let backup = PathBackup::new(
            r"C:\User\Path".to_string(),
            r"C:\Machine\Path".to_string(),
            vec![r"C:\User\Path".to_string()],
            vec![r"C:\Machine\Path".to_string()],
        );

        let temp_dir = env::temp_dir();
        let filepath = backup.save(&temp_dir).unwrap();

        let loaded = PathBackup::load(&filepath).unwrap();
        assert_eq!(loaded.user_path, backup.user_path);
        assert_eq!(loaded.machine_path, backup.machine_path);

        // Cleanup
        let _ = fs::remove_file(filepath);
    }
}
