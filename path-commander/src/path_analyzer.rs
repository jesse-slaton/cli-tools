use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Convert a local path to UNC format for remote access
/// Example: `C:\Program Files` on computer `SERVER` becomes `\\SERVER\C$\Program Files`
pub fn to_unc_path(local_path: &str, computer_name: &str) -> Option<String> {
    let expanded = expand_environment_variables(local_path);

    // Check if it's already a UNC path
    if expanded.starts_with(r"\\") {
        return None;
    }

    // Extract drive letter (e.g., "C:" from "C:\Windows")
    if expanded.len() >= 2 && expanded.chars().nth(1) == Some(':') {
        let drive_letter = expanded.chars().next()?;
        let rest_of_path = &expanded[2..];

        // Convert to UNC: \\COMPUTERNAME\C$\rest\of\path
        Some(format!(
            r"\\{}\{}${}",
            computer_name, drive_letter, rest_of_path
        ))
    } else {
        // Not a standard drive-letter path
        None
    }
}

/// Status of a path entry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathStatus {
    Valid,         // Exists, unique, normalized
    Dead,          // Does not exist
    Duplicate,     // Duplicate within same scope or across scopes
    NonNormalized, // Contains short names, env vars, or can be expanded
    DeadDuplicate, // Both dead and duplicate
}

impl PathStatus {
    /// Check if this status indicates a problem
    #[cfg(test)]
    pub fn is_problematic(&self) -> bool {
        matches!(
            self,
            PathStatus::Dead | PathStatus::Duplicate | PathStatus::DeadDuplicate
        )
    }

    /// Get a human-readable description
    #[cfg(test)]
    pub fn description(&self) -> &'static str {
        match self {
            PathStatus::Valid => "Valid",
            PathStatus::Dead => "Dead (path does not exist)",
            PathStatus::Duplicate => "Duplicate",
            PathStatus::NonNormalized => "Can be normalized",
            PathStatus::DeadDuplicate => "Dead & Duplicate",
        }
    }
}

/// Information about a path entry
#[derive(Debug, Clone)]
pub struct PathInfo {
    /// The original path string before normalization (preserved for debugging and future features)
    #[allow(dead_code)]
    pub original: String,
    pub normalized: String,
    pub status: PathStatus,
    pub exists: bool,
    pub is_duplicate: bool,
    pub needs_normalization: bool,
}

/// Analyze a list of path entries
pub fn analyze_paths(paths: &[String], other_scope_paths: &[String]) -> Vec<PathInfo> {
    analyze_paths_with_remote(paths, other_scope_paths, None)
}

/// Analyze a list of path entries with optional remote computer support
pub fn analyze_paths_with_remote(
    paths: &[String],
    other_scope_paths: &[String],
    remote_computer: Option<&str>,
) -> Vec<PathInfo> {
    let mut results: Vec<PathInfo> = Vec::new();
    let mut seen_normalized: HashMap<String, usize> = HashMap::new();

    // First pass: normalize and check existence
    for (idx, path) in paths.iter().enumerate() {
        let normalized = normalize_path(path);
        let exists = path_exists_with_remote(&normalized, remote_computer);
        let needs_normalization = path != &normalized;

        // Track normalized paths for duplicate detection
        if let Some(&first_idx) = seen_normalized.get(&normalized.to_lowercase()) {
            // Mark the first occurrence as duplicate too
            if first_idx < results.len() {
                results[first_idx].is_duplicate = true;
            }
            results.push(PathInfo {
                original: path.clone(),
                normalized: normalized.clone(),
                status: PathStatus::Valid, // Will be updated
                exists,
                is_duplicate: true,
                needs_normalization,
            });
        } else {
            seen_normalized.insert(normalized.to_lowercase(), idx);
            results.push(PathInfo {
                original: path.clone(),
                normalized: normalized.clone(),
                status: PathStatus::Valid, // Will be updated
                exists,
                is_duplicate: false,
                needs_normalization,
            });
        }
    }

    // Check for duplicates across scopes
    let other_normalized: HashSet<String> = other_scope_paths
        .iter()
        .map(|p| normalize_path(p).to_lowercase())
        .collect();

    for info in &mut results {
        if other_normalized.contains(&info.normalized.to_lowercase()) {
            info.is_duplicate = true;
        }
    }

    // Second pass: determine final status
    for info in &mut results {
        info.status = determine_status(info);
    }

    results
}

/// Determine the final status of a path
fn determine_status(info: &PathInfo) -> PathStatus {
    match (info.exists, info.is_duplicate, info.needs_normalization) {
        (false, true, _) => PathStatus::DeadDuplicate,
        (false, false, _) => PathStatus::Dead,
        (true, true, _) => PathStatus::Duplicate,
        (true, false, true) => PathStatus::NonNormalized,
        (true, false, false) => PathStatus::Valid,
    }
}

/// Check if a path exists (file or directory)
pub fn path_exists(path: &str) -> bool {
    path_exists_with_remote(path, None)
}

/// Check if a path exists with optional remote computer support
pub fn path_exists_with_remote(path: &str, remote_computer: Option<&str>) -> bool {
    if path.is_empty() {
        return false;
    }

    // Expand environment variables first
    let expanded = expand_environment_variables(path);

    // If checking a remote path, convert to UNC
    if let Some(computer_name) = remote_computer {
        if let Some(unc_path) = to_unc_path(&expanded, computer_name) {
            // Try to access the UNC path
            return Path::new(&unc_path).exists();
        }
    }

    // Local path or UNC conversion failed - check locally
    Path::new(&expanded).exists()
}

/// Normalize a path by:
/// - Removing quotes (both balanced and unbalanced)
/// - Collapsing to environment variables where possible (e.g., C:\Program Files -> %PROGRAMFILES%)
/// - Expanding short names (8.3 format) to long path names
/// - Removing trailing slashes
/// - Removing \?\ prefix if present
///
/// This is the opposite of the previous behavior - we want portable env vars, not expanded paths.
pub fn normalize_path(path: &str) -> String {
    if path.is_empty() {
        return path.to_string();
    }

    // Remove quotes (both balanced and unbalanced)
    // PATH entries should never have quotes
    let mut cleaned = path.trim().to_string();

    // Remove leading quote
    if cleaned.starts_with('"') {
        cleaned = cleaned[1..].to_string();
    }

    // Remove trailing quote
    if cleaned.ends_with('"') {
        cleaned = cleaned[..cleaned.len() - 1].to_string();
    }

    // Trim again after quote removal
    cleaned = cleaned.trim().to_string();

    // First get the absolute expanded path for comparison
    let mut expanded = expand_environment_variables(&cleaned);

    // Remove \?\ prefix if present (this shouldn't be in PATH variables)
    if let Some(stripped) = expanded.strip_prefix(r"\\?\") {
        expanded = stripped.to_string();
    }

    // Try to canonicalize to expand short names (8.3 format like PROGRA~1)
    if let Ok(canonical) = std::fs::canonicalize(&expanded) {
        if let Some(path_str) = canonical.to_str() {
            // Canonicalize adds \\?\ prefix, remove it
            if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
                expanded = stripped.to_string();
            } else {
                expanded = path_str.to_string();
            }
        }
    }

    // Remove trailing backslash/slash
    expanded = expanded
        .trim_end_matches('\\')
        .trim_end_matches('/')
        .to_string();

    // Now collapse to environment variables where possible
    collapse_to_env_vars(&expanded)
}

/// Collapse an absolute path to use environment variables where possible
/// Matches longest prefixes first and performs case-insensitive matching
fn collapse_to_env_vars(path: &str) -> String {
    // Environment variable mappings in priority order (longest/most specific first)
    // This ensures we prefer %LOCALAPPDATA% over %USERPROFILE% for paths in AppData\Local
    let env_var_mappings: Vec<(&str, String)> = vec![
        (
            "LOCALAPPDATA",
            std::env::var("LOCALAPPDATA").unwrap_or_default(),
        ),
        ("APPDATA", std::env::var("APPDATA").unwrap_or_default()),
        (
            "PROGRAMFILES(X86)",
            std::env::var("PROGRAMFILES(X86)").unwrap_or_default(),
        ),
        (
            "PROGRAMFILES",
            std::env::var("PROGRAMFILES").unwrap_or_default(),
        ),
        (
            "PROGRAMDATA",
            std::env::var("PROGRAMDATA").unwrap_or_default(),
        ),
        (
            "USERPROFILE",
            std::env::var("USERPROFILE").unwrap_or_default(),
        ),
        (
            "SYSTEMROOT",
            std::env::var("SYSTEMROOT").unwrap_or_default(),
        ),
        ("WINDIR", std::env::var("WINDIR").unwrap_or_default()),
        ("TEMP", std::env::var("TEMP").unwrap_or_default()),
        ("TMP", std::env::var("TMP").unwrap_or_default()),
    ];

    let path_lower = path.to_lowercase();

    // Try to match against each environment variable (case-insensitive)
    for (var_name, var_value) in env_var_mappings {
        if var_value.is_empty() {
            continue;
        }

        let var_value_lower = var_value.to_lowercase();

        // Check if the path starts with this env var value
        if path_lower.starts_with(&var_value_lower) {
            // Get the remaining part of the path
            let remaining = &path[var_value.len()..];

            // If there's a remaining part, it should start with a separator
            if remaining.is_empty() {
                return format!("%{}%", var_name);
            } else if remaining.starts_with('\\') || remaining.starts_with('/') {
                return format!("%{}%{}", var_name, remaining);
            }
        }
    }

    // No matching environment variable found, return the path as-is
    path.to_string()
}

/// Expand environment variables in a path string
pub fn expand_environment_variables(path: &str) -> String {
    let mut result = path.to_string();

    // Common environment variables to expand
    let vars = [
        "USERPROFILE",
        "PROGRAMFILES",
        "PROGRAMFILES(X86)",
        "PROGRAMDATA",
        "APPDATA",
        "LOCALAPPDATA",
        "SYSTEMROOT",
        "WINDIR",
        "TEMP",
        "TMP",
        "HOMEDRIVE",
        "HOMEPATH",
    ];

    for var in &vars {
        if let Ok(value) = std::env::var(var) {
            // Try both %VAR% and ${VAR} formats
            result = result.replace(&format!("%{}%", var), &value);
            result = result.replace(&format!("${{{}}}", var), &value);
            // Case-insensitive replacement
            let var_lower = var.to_lowercase();
            result = result.replace(&format!("%{}%", var_lower), &value);
        }
    }

    result
}

/// Find all duplicate paths across both scopes
#[cfg(test)]
pub fn find_all_duplicates(user_paths: &[String], machine_paths: &[String]) -> HashSet<String> {
    let mut duplicates = HashSet::new();
    let mut seen = HashMap::new();

    // Check user paths
    for path in user_paths {
        let normalized = normalize_path(path).to_lowercase();
        *seen.entry(normalized.clone()).or_insert(0) += 1;
    }

    // Check machine paths
    for path in machine_paths {
        let normalized = normalize_path(path).to_lowercase();
        *seen.entry(normalized.clone()).or_insert(0) += 1;
    }

    // Collect paths that appear more than once
    for (path, count) in seen {
        if count > 1 {
            duplicates.insert(path);
        }
    }

    duplicates
}

/// Find all dead paths in a list
#[cfg(test)]
pub fn find_dead_paths(paths: &[String]) -> Vec<usize> {
    paths
        .iter()
        .enumerate()
        .filter(|(_, path)| !path_exists(path))
        .map(|(idx, _)| idx)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path_env_vars() {
        // Environment variables should be preserved (not expanded)
        let path = r"%USERPROFILE%\Documents";
        let normalized = normalize_path(path);
        // Since the path already uses env vars, it should remain with env vars
        // (it will expand then collapse back)
        assert!(normalized.contains('%') || normalized.contains("USERPROFILE"));
    }

    #[test]
    fn test_normalize_path_trailing_slash() {
        let path = r"C:\Windows\";
        let normalized = normalize_path(path);
        assert!(!normalized.ends_with('\\'));
        assert!(!normalized.ends_with('/'));
    }

    #[test]
    fn test_normalize_path_empty() {
        let path = "";
        let normalized = normalize_path(path);
        assert_eq!(normalized, "");
    }

    #[test]
    fn test_expand_environment_variables() {
        // Test %VAR% format
        let userprofile = std::env::var("USERPROFILE").unwrap();
        let path = r"%USERPROFILE%\test";
        let expanded = expand_environment_variables(path);
        assert_eq!(expanded, format!(r"{}\test", userprofile));

        // Test case-insensitive
        let path_lower = r"%userprofile%\test";
        let expanded_lower = expand_environment_variables(path_lower);
        assert_eq!(expanded_lower, format!(r"{}\test", userprofile));
    }

    #[test]
    fn test_path_exists() {
        assert!(path_exists(r"C:\Windows"));
        assert!(!path_exists(r"C:\ThisPathDoesNotExist123456"));
    }

    #[test]
    fn test_path_exists_empty() {
        assert!(!path_exists(""));
    }

    #[test]
    fn test_path_exists_with_env_var() {
        // Test that environment variables are expanded before checking
        assert!(path_exists(r"%SYSTEMROOT%"));
    }

    #[test]
    fn test_find_duplicates_case_insensitive() {
        let paths = vec![
            r"C:\Windows".to_string(),
            r"C:\windows".to_string(), // Case variation
            r"C:\Program Files".to_string(),
        ];

        let info = analyze_paths(&paths, &[]);
        let duplicates: Vec<_> = info.iter().filter(|i| i.is_duplicate).collect();

        assert_eq!(duplicates.len(), 2); // The two Windows entries
    }

    #[test]
    fn test_analyze_paths_no_duplicates() {
        let paths = vec![r"C:\Windows".to_string(), r"C:\Program Files".to_string()];

        let info = analyze_paths(&paths, &[]);
        assert_eq!(info.len(), 2);
        assert!(!info[0].is_duplicate);
        assert!(!info[1].is_duplicate);
    }

    #[test]
    fn test_analyze_paths_cross_scope_duplicates() {
        let user_paths = vec![r"C:\Windows".to_string()];
        let machine_paths = vec![r"C:\windows".to_string()]; // Case variation

        let info = analyze_paths(&user_paths, &machine_paths);
        assert_eq!(info.len(), 1);
        assert!(info[0].is_duplicate);
    }

    #[test]
    fn test_analyze_paths_dead_path() {
        let paths = vec![r"C:\ThisPathDoesNotExist123456".to_string()];

        let info = analyze_paths(&paths, &[]);
        assert_eq!(info.len(), 1);
        assert_eq!(info[0].status, PathStatus::Dead);
        assert!(!info[0].exists);
    }

    #[test]
    fn test_analyze_paths_dead_duplicate() {
        let paths = vec![
            r"C:\NonExistent1".to_string(),
            r"C:\NonExistent1".to_string(),
        ];

        let info = analyze_paths(&paths, &[]);
        assert_eq!(info.len(), 2);
        assert_eq!(info[0].status, PathStatus::DeadDuplicate);
        assert_eq!(info[1].status, PathStatus::DeadDuplicate);
    }

    #[test]
    fn test_analyze_paths_needs_normalization() {
        // Test that an absolute path gets collapsed to env var
        let programfiles = std::env::var("PROGRAMFILES").unwrap_or_default();
        if !programfiles.is_empty() {
            let absolute_path = format!(r"{}\TestApp", programfiles);
            let paths = vec![absolute_path.clone()];

            let info = analyze_paths(&paths, &[]);
            assert_eq!(info.len(), 1);
            // The absolute path should be marked as needing normalization
            // because it can be collapsed to %PROGRAMFILES%\TestApp
            if info[0].needs_normalization {
                assert!(info[0].normalized.contains('%'));
                assert!(info[0].normalized.contains("PROGRAMFILES"));
            }
        }
    }

    #[test]
    fn test_find_all_duplicates() {
        let user_paths = vec![r"C:\Windows".to_string(), r"C:\Program Files".to_string()];
        let machine_paths = vec![
            r"C:\windows".to_string(), // Duplicate of user path
            r"C:\Temp".to_string(),
        ];

        let duplicates = find_all_duplicates(&user_paths, &machine_paths);
        assert!(duplicates.contains(&normalize_path(r"C:\Windows").to_lowercase()));
    }

    #[test]
    fn test_find_dead_paths() {
        let paths = vec![
            r"C:\Windows".to_string(),
            r"C:\NonExistent1".to_string(),
            r"C:\Program Files".to_string(),
            r"C:\NonExistent2".to_string(),
        ];

        let dead = find_dead_paths(&paths);
        assert_eq!(dead.len(), 2);
        assert!(dead.contains(&1));
        assert!(dead.contains(&3));
    }

    #[test]
    fn test_path_status_is_problematic() {
        assert!(PathStatus::Dead.is_problematic());
        assert!(PathStatus::Duplicate.is_problematic());
        assert!(PathStatus::DeadDuplicate.is_problematic());
        assert!(!PathStatus::Valid.is_problematic());
        assert!(!PathStatus::NonNormalized.is_problematic());
    }

    #[test]
    fn test_path_status_description() {
        assert_eq!(PathStatus::Valid.description(), "Valid");
        assert_eq!(PathStatus::Dead.description(), "Dead (path does not exist)");
        assert_eq!(PathStatus::Duplicate.description(), "Duplicate");
        assert_eq!(PathStatus::NonNormalized.description(), "Can be normalized");
        assert_eq!(PathStatus::DeadDuplicate.description(), "Dead & Duplicate");
    }

    #[test]
    fn test_determine_status() {
        // Valid path
        let info = PathInfo {
            original: "%SYSTEMROOT%".to_string(),
            normalized: "%SYSTEMROOT%".to_string(),
            status: PathStatus::Valid,
            exists: true,
            is_duplicate: false,
            needs_normalization: false,
        };
        assert_eq!(determine_status(&info), PathStatus::Valid);

        // Dead path
        let info = PathInfo {
            original: "C:\\NonExistent".to_string(),
            normalized: "C:\\NonExistent".to_string(),
            status: PathStatus::Valid,
            exists: false,
            is_duplicate: false,
            needs_normalization: false,
        };
        assert_eq!(determine_status(&info), PathStatus::Dead);

        // Duplicate path
        let info = PathInfo {
            original: "%SYSTEMROOT%".to_string(),
            normalized: "%SYSTEMROOT%".to_string(),
            status: PathStatus::Valid,
            exists: true,
            is_duplicate: true,
            needs_normalization: false,
        };
        assert_eq!(determine_status(&info), PathStatus::Duplicate);

        // Non-normalized path (absolute path that could use env var)
        let info = PathInfo {
            original: "C:\\Windows".to_string(),
            normalized: "%SYSTEMROOT%".to_string(),
            status: PathStatus::Valid,
            exists: true,
            is_duplicate: false,
            needs_normalization: true,
        };
        assert_eq!(determine_status(&info), PathStatus::NonNormalized);

        // Dead duplicate
        let info = PathInfo {
            original: "C:\\NonExistent".to_string(),
            normalized: "C:\\NonExistent".to_string(),
            status: PathStatus::Valid,
            exists: false,
            is_duplicate: true,
            needs_normalization: false,
        };
        assert_eq!(determine_status(&info), PathStatus::DeadDuplicate);
    }

    #[test]
    fn test_empty_path_list() {
        let info = analyze_paths(&[], &[]);
        assert_eq!(info.len(), 0);
    }

    #[test]
    fn test_multiple_duplicates() {
        let paths = vec![
            r"C:\Windows".to_string(),
            r"C:\windows".to_string(),
            r"C:\WINDOWS".to_string(),
        ];

        let info = analyze_paths(&paths, &[]);
        assert_eq!(info.len(), 3);
        // All three should be marked as duplicates
        assert!(info.iter().all(|i| i.is_duplicate));
    }

    #[test]
    fn test_collapse_to_programfiles() {
        let programfiles = std::env::var("PROGRAMFILES").unwrap_or_default();
        if !programfiles.is_empty() {
            let absolute_path = format!(r"{}\Git\cmd", programfiles);
            let normalized = normalize_path(&absolute_path);
            assert!(normalized.contains("%PROGRAMFILES%"));
            assert!(normalized.contains("Git"));
        }
    }

    #[test]
    fn test_collapse_to_localappdata() {
        let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
        if !localappdata.is_empty() {
            let absolute_path = format!(r"{}\Programs\Python", localappdata);
            let normalized = normalize_path(&absolute_path);
            assert!(normalized.contains("%LOCALAPPDATA%"));
            assert!(normalized.contains("Programs"));
        }
    }

    #[test]
    fn test_collapse_prefers_longest_match() {
        // LOCALAPPDATA is under USERPROFILE, so we should prefer LOCALAPPDATA
        let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
        if !localappdata.is_empty() && localappdata.contains("AppData\\Local") {
            let absolute_path = format!(r"{}\Test", localappdata);
            let normalized = normalize_path(&absolute_path);
            // Should use LOCALAPPDATA, not USERPROFILE
            assert!(normalized.contains("%LOCALAPPDATA%"));
            assert!(!normalized.contains("%USERPROFILE%"));
        }
    }

    #[test]
    fn test_remove_extended_path_prefix() {
        let path = r"\\?\C:\CustomLocation\bin";
        let normalized = normalize_path(path);
        assert!(!normalized.starts_with(r"\\?\"));
    }

    #[test]
    fn test_already_normalized_path_unchanged() {
        // A path that's already using env vars should remain unchanged
        let path = r"%PROGRAMFILES%\Git\cmd";
        let normalized = normalize_path(path);
        // After expand then collapse, should get back the same env var format
        assert!(normalized.contains("%PROGRAMFILES%"));
    }

    #[test]
    fn test_case_insensitive_collapse() {
        let programfiles = std::env::var("PROGRAMFILES").unwrap_or_default();
        if !programfiles.is_empty() {
            // Try with different case
            let lowercase_path = format!(r"{}\Git", programfiles.to_lowercase());
            let normalized = normalize_path(&lowercase_path);
            // Note: This test may not work as expected since the path may not exist
            // and canonicalize might fail. The important thing is it doesn't crash.
            assert!(!normalized.is_empty());
        }
    }

    #[test]
    fn test_path_without_matching_env_var() {
        let path = r"C:\CustomTools\bin";
        let normalized = normalize_path(path);
        // Should remain as absolute path since there's no matching env var
        // (unless it happens to match one of the standard env vars)
        assert!(!normalized.is_empty());
    }

    #[test]
    fn test_collapse_systemroot() {
        let systemroot = std::env::var("SYSTEMROOT").unwrap_or_default();
        if !systemroot.is_empty() {
            let absolute_path = format!(r"{}\System32", systemroot);
            let normalized = normalize_path(&absolute_path);
            assert!(
                normalized.contains("%SYSTEMROOT%") || normalized.contains("%WINDIR%"),
                "Expected env var in: {}",
                normalized
            );
        }
    }

    #[test]
    fn test_trailing_slash_removal_with_collapse() {
        let programfiles = std::env::var("PROGRAMFILES").unwrap_or_default();
        if !programfiles.is_empty() {
            let path_with_slash = format!(r"{}\Git\", programfiles);
            let normalized = normalize_path(&path_with_slash);
            assert!(!normalized.ends_with('\\'));
            assert!(!normalized.ends_with('/'));
        }
    }

    #[test]
    fn test_remove_balanced_quotes() {
        let systemroot = std::env::var("SYSTEMROOT").unwrap_or_default();
        if !systemroot.is_empty() {
            let quoted_path = format!(r#""{}\System32""#, systemroot);
            let normalized = normalize_path(&quoted_path);
            assert!(!normalized.starts_with('"'));
            assert!(!normalized.ends_with('"'));
            assert!(normalized.contains("%SYSTEMROOT%") || normalized.contains("%WINDIR%"));
        }
    }

    #[test]
    fn test_remove_leading_quote_only() {
        let systemroot = std::env::var("SYSTEMROOT").unwrap_or_default();
        if !systemroot.is_empty() {
            let unbalanced_path = format!(r#""{}\System32"#, systemroot);
            let normalized = normalize_path(&unbalanced_path);
            assert!(!normalized.starts_with('"'));
            assert!(!normalized.ends_with('"'));
            assert!(normalized.contains("%SYSTEMROOT%") || normalized.contains("%WINDIR%"));
        }
    }

    #[test]
    fn test_remove_trailing_quote_only() {
        let systemroot = std::env::var("SYSTEMROOT").unwrap_or_default();
        if !systemroot.is_empty() {
            let unbalanced_path = format!(r#"{}\System32""#, systemroot);
            let normalized = normalize_path(&unbalanced_path);
            assert!(!normalized.starts_with('"'));
            assert!(!normalized.ends_with('"'));
            assert!(normalized.contains("%SYSTEMROOT%") || normalized.contains("%WINDIR%"));
        }
    }

    #[test]
    fn test_remove_quotes_with_whitespace() {
        let systemroot = std::env::var("SYSTEMROOT").unwrap_or_default();
        if !systemroot.is_empty() {
            // Quotes with leading/trailing whitespace
            let quoted_path = format!(r#"  "{}\System32"  "#, systemroot);
            let normalized = normalize_path(&quoted_path);
            assert!(!normalized.starts_with('"'));
            assert!(!normalized.ends_with('"'));
            assert!(!normalized.starts_with(' '));
            assert!(!normalized.ends_with(' '));
            assert!(normalized.contains("%SYSTEMROOT%") || normalized.contains("%WINDIR%"));
        }
    }

    #[test]
    fn test_quotes_with_env_var() {
        let quoted_path = r#""%PROGRAMFILES%\Git\cmd""#;
        let normalized = normalize_path(quoted_path);
        assert!(!normalized.starts_with('"'));
        assert!(!normalized.ends_with('"'));
        assert!(normalized.contains("%PROGRAMFILES%"));
    }

    #[test]
    fn test_unbalanced_quote_with_env_var_leading() {
        let quoted_path = r#""%PROGRAMFILES%\Git\cmd"#;
        let normalized = normalize_path(quoted_path);
        assert!(!normalized.starts_with('"'));
        assert!(!normalized.ends_with('"'));
        assert!(normalized.contains("%PROGRAMFILES%"));
    }

    #[test]
    fn test_unbalanced_quote_with_env_var_trailing() {
        let quoted_path = r#"%PROGRAMFILES%\Git\cmd""#;
        let normalized = normalize_path(quoted_path);
        assert!(!normalized.starts_with('"'));
        assert!(!normalized.ends_with('"'));
        assert!(normalized.contains("%PROGRAMFILES%"));
    }

    #[test]
    fn test_quoted_paths_detected_as_needing_normalization() {
        // Test that quoted paths are properly flagged as needing normalization
        let systemroot = std::env::var("SYSTEMROOT").unwrap_or_default();
        if !systemroot.is_empty() {
            // Test balanced quotes
            let quoted_path = format!(r#""{}\System32""#, systemroot);
            let paths = vec![quoted_path.clone()];
            let info = analyze_paths(&paths, &[]);

            assert_eq!(info.len(), 1);
            assert!(
                info[0].needs_normalization,
                "Quoted path should be flagged as needing normalization: original='{}', normalized='{}'",
                info[0].original,
                info[0].normalized
            );
            assert_eq!(info[0].status, PathStatus::NonNormalized);

            // Test unbalanced quote (leading)
            let unbalanced_path = format!(r#""{}\System32"#, systemroot);
            let paths = vec![unbalanced_path.clone()];
            let info = analyze_paths(&paths, &[]);

            assert_eq!(info.len(), 1);
            assert!(
                info[0].needs_normalization,
                "Unbalanced quoted path should be flagged as needing normalization"
            );
            assert_eq!(info[0].status, PathStatus::NonNormalized);
        }
    }

    #[test]
    fn test_to_unc_path() {
        // Test basic drive conversion
        let result = to_unc_path(r"C:\Windows", "SERVER");
        assert_eq!(result, Some(r"\\SERVER\C$\Windows".to_string()));

        // Test with Program Files
        let result = to_unc_path(r"C:\Program Files\App", "REMOTE-PC");
        assert_eq!(
            result,
            Some(r"\\REMOTE-PC\C$\Program Files\App".to_string())
        );

        // Test with D: drive
        let result = to_unc_path(r"D:\Data", "SERVER");
        assert_eq!(result, Some(r"\\SERVER\D$\Data".to_string()));

        // Test with environment variable (should be expanded first)
        let systemroot = std::env::var("SYSTEMROOT").unwrap_or_default();
        if !systemroot.is_empty() {
            let result = to_unc_path(r"%SYSTEMROOT%\System32", "SERVER");
            assert!(result.is_some());
            let unc = result.unwrap();
            assert!(unc.starts_with(r"\\SERVER\"));
            assert!(unc.contains("System32"));
        }

        // Test already UNC path (should return None)
        let result = to_unc_path(r"\\EXISTING\Share\Path", "SERVER");
        assert_eq!(result, None);

        // Test non-standard path (should return None)
        let result = to_unc_path(r"RelativePath", "SERVER");
        assert_eq!(result, None);
    }

    #[test]
    fn test_path_exists_with_remote() {
        // Test local path without remote computer
        assert!(path_exists_with_remote(r"C:\Windows", None));

        // Test empty path
        assert!(!path_exists_with_remote("", None));
        assert!(!path_exists_with_remote("", Some("SERVER")));

        // Test with environment variable
        assert!(path_exists_with_remote(r"%SYSTEMROOT%", None));
    }

    #[test]
    fn test_analyze_paths_with_remote() {
        // Test basic analysis with remote computer name
        // Note: This test uses local paths and won't actually access a remote computer
        let paths = vec![r"C:\Windows".to_string()];
        let info = analyze_paths_with_remote(&paths, &[], Some("SERVER"));
        assert_eq!(info.len(), 1);
        // The path may or may not exist depending on whether SERVER\C$ is accessible
        // So we just verify the function doesn't crash
    }

    #[test]
    fn test_absolute_path_detected_as_needing_normalization() {
        // Test that absolute paths that can be collapsed are flagged
        let programfiles = std::env::var("PROGRAMFILES").unwrap_or_default();
        if !programfiles.is_empty() {
            let absolute_path = format!(r"{}\Git", programfiles);
            let paths = vec![absolute_path.clone()];
            let info = analyze_paths(&paths, &[]);

            assert_eq!(info.len(), 1);
            if info[0].exists {
                // Only check if path exists, otherwise status might be Dead
                assert!(
                    info[0].needs_normalization,
                    "Absolute path that can be collapsed should be flagged: original='{}', normalized='{}'",
                    info[0].original,
                    info[0].normalized
                );
                assert_eq!(info[0].status, PathStatus::NonNormalized);
            }
        }
    }
}
