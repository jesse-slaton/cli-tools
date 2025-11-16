use anyhow::{Context, Result};
use std::path::PathBuf;

/// Get the Path Commander configuration directory (~/.pc)
///
/// Falls back to ~/.pathcommand if ~/.pc is not available or already in use by another application
pub fn get_config_dir() -> Result<PathBuf> {
    let home_dir = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .context("Could not determine home directory (USERPROFILE or HOME not set)")?;

    let config_dir = PathBuf::from(home_dir).join(".pc");

    // Check if ~/.pc exists and has a marker file from another application
    if config_dir.exists() {
        let marker = config_dir.join(".pathcommander");
        if !marker.exists() {
            // Check if directory has files (might be used by something else)
            if let Ok(entries) = std::fs::read_dir(&config_dir) {
                if entries.count() > 0 {
                    // Directory exists with files but no marker - might be another app
                    // Use fallback directory
                    let fallback =
                        PathBuf::from(std::env::var("USERPROFILE")?).join(".pathcommand");
                    return Ok(fallback);
                }
            }
        }
    }

    Ok(config_dir)
}

/// Get the themes directory path
pub fn get_themes_dir() -> Result<PathBuf> {
    let config_dir = get_config_dir()?;
    Ok(config_dir.join("themes"))
}

/// Get the backups directory path
pub fn get_backups_dir() -> Result<PathBuf> {
    let config_dir = get_config_dir()?;
    Ok(config_dir.join("backups"))
}

/// Ensure the configuration directory structure exists
///
/// Creates:
/// - ~/.pc/
/// - ~/.pc/.pathcommander (marker file)
/// - ~/.pc/themes/
/// - ~/.pc/backups/
pub fn ensure_config_dirs() -> Result<()> {
    let config_dir = get_config_dir()?;

    // Create main config directory
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).with_context(|| {
            format!(
                "Failed to create config directory: {}",
                config_dir.display()
            )
        })?;
    }

    // Create marker file
    let marker = config_dir.join(".pathcommander");
    if !marker.exists() {
        std::fs::write(&marker, "Path Commander configuration directory\n")
            .with_context(|| format!("Failed to create marker file: {}", marker.display()))?;
    }

    // Create themes directory
    let themes_dir = get_themes_dir()?;
    if !themes_dir.exists() {
        std::fs::create_dir_all(&themes_dir).with_context(|| {
            format!(
                "Failed to create themes directory: {}",
                themes_dir.display()
            )
        })?;
    }

    // Create backups directory
    let backups_dir = get_backups_dir()?;
    if !backups_dir.exists() {
        std::fs::create_dir_all(&backups_dir).with_context(|| {
            format!(
                "Failed to create backups directory: {}",
                backups_dir.display()
            )
        })?;
    }

    Ok(())
}

/// Migrate backups from old location (%LOCALAPPDATA%\PathCommander\backups\) to new location
pub fn migrate_backups() -> Result<()> {
    let old_backups_dir = std::env::var("LOCALAPPDATA")
        .ok()
        .map(|appdata| PathBuf::from(appdata).join("PathCommander").join("backups"));

    if let Some(old_dir) = old_backups_dir {
        if old_dir.exists() {
            let new_dir = get_backups_dir()?;

            // Copy all backup files from old to new location
            let entries = std::fs::read_dir(&old_dir).with_context(|| {
                format!(
                    "Failed to read old backups directory: {}",
                    old_dir.display()
                )
            })?;

            for entry in entries {
                let entry = entry?;
                let file_name = entry.file_name();
                let old_path = entry.path();
                let new_path = new_dir.join(&file_name);

                // Only copy if file doesn't exist in new location
                if !new_path.exists() && old_path.is_file() {
                    std::fs::copy(&old_path, &new_path).with_context(|| {
                        format!("Failed to migrate backup: {}", file_name.to_string_lossy())
                    })?;
                }
            }

            // Note: We don't delete the old directory - user can do that manually
        }
    }

    Ok(())
}

/// List all available themes (built-in + custom)
///
/// Returns a list of theme names with a boolean indicating if they're built-in
pub fn list_available_themes() -> Result<Vec<(String, bool)>> {
    let mut themes = vec![("default".to_string(), true)];

    // Add custom themes from ~/.pc/themes/
    let themes_dir = get_themes_dir()?;
    if themes_dir.exists() {
        let entries = std::fs::read_dir(&themes_dir).with_context(|| {
            format!("Failed to read themes directory: {}", themes_dir.display())
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("ini") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    let name = name.to_string();
                    // Don't add if it's a built-in theme name
                    if !themes.iter().any(|(n, _)| n == &name) {
                        themes.push((name, false));
                    }
                }
            }
        }
    }

    Ok(themes)
}

/// Get the path to a theme INI file
///
/// Returns the path if it exists, or None if the theme doesn't exist
pub fn get_theme_path(theme_name: &str) -> Option<PathBuf> {
    // Check custom themes first
    if let Ok(themes_dir) = get_themes_dir() {
        let custom_path = themes_dir.join(format!("{}.ini", theme_name));
        if custom_path.exists() {
            return Some(custom_path);
        }
    }

    // Check embedded themes directory
    let embedded_path = PathBuf::from("themes").join(format!("{}.ini", theme_name));
    if embedded_path.exists() {
        return Some(embedded_path);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir_path() {
        let config_dir = get_config_dir().unwrap();
        assert!(
            config_dir.to_string_lossy().contains(".pc")
                || config_dir.to_string_lossy().contains(".pathcommand")
        );
    }

    #[test]
    fn test_themes_dir_path() {
        let themes_dir = get_themes_dir().unwrap();
        assert!(themes_dir.to_string_lossy().ends_with("themes"));
    }

    #[test]
    fn test_backups_dir_path() {
        let backups_dir = get_backups_dir().unwrap();
        assert!(backups_dir.to_string_lossy().ends_with("backups"));
    }
}
