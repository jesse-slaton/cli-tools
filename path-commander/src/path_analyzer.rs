use std::collections::{HashMap, HashSet};
use std::path::Path;

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
    #[allow(dead_code)]
    pub fn is_problematic(&self) -> bool {
        matches!(
            self,
            PathStatus::Dead | PathStatus::Duplicate | PathStatus::DeadDuplicate
        )
    }

    /// Get a human-readable description
    #[allow(dead_code)]
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
#[allow(dead_code)]
pub struct PathInfo {
    pub original: String,
    pub normalized: String,
    pub status: PathStatus,
    pub exists: bool,
    pub is_duplicate: bool,
    pub needs_normalization: bool,
}

/// Analyze a list of path entries
pub fn analyze_paths(paths: &[String], other_scope_paths: &[String]) -> Vec<PathInfo> {
    let mut results: Vec<PathInfo> = Vec::new();
    let mut seen_normalized: HashMap<String, usize> = HashMap::new();

    // First pass: normalize and check existence
    for (idx, path) in paths.iter().enumerate() {
        let normalized = normalize_path(path);
        let exists = path_exists(&normalized);
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
    if path.is_empty() {
        return false;
    }

    // Expand environment variables first
    let expanded = expand_environment_variables(path);
    Path::new(&expanded).exists()
}

/// Normalize a path by:
/// - Expanding environment variables
/// - Converting to long path names (from 8.3 format)
/// - Removing trailing slashes
/// - Resolving to canonical form if possible
pub fn normalize_path(path: &str) -> String {
    if path.is_empty() {
        return path.to_string();
    }

    // First expand environment variables
    let mut expanded = expand_environment_variables(path);

    // Convert to long path name if it exists
    if let Ok(canonical) = std::fs::canonicalize(&expanded) {
        if let Some(path_str) = canonical.to_str() {
            expanded = path_str.to_string();
        }
    }

    // Remove trailing backslash/slash
    expanded = expanded
        .trim_end_matches('\\')
        .trim_end_matches('/')
        .to_string();

    expanded
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
        // This will use actual environment variables
        let path = r"%USERPROFILE%\AppData\Local";
        let normalized = normalize_path(path);
        assert!(!normalized.contains('%'));
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
        // Create a path with environment variable
        let paths = vec![r"%SYSTEMROOT%".to_string()];

        let info = analyze_paths(&paths, &[]);
        assert_eq!(info.len(), 1);
        assert!(info[0].needs_normalization);
        assert!(!info[0].normalized.contains('%'));
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
            original: "C:\\Windows".to_string(),
            normalized: "C:\\Windows".to_string(),
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
            original: "C:\\Windows".to_string(),
            normalized: "C:\\Windows".to_string(),
            status: PathStatus::Valid,
            exists: true,
            is_duplicate: true,
            needs_normalization: false,
        };
        assert_eq!(determine_status(&info), PathStatus::Duplicate);

        // Non-normalized path
        let info = PathInfo {
            original: "%SYSTEMROOT%".to_string(),
            normalized: "C:\\Windows".to_string(),
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
}
