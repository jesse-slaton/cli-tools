use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Status of a path entry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathStatus {
    Valid,          // Exists, unique, normalized
    Dead,           // Does not exist
    Duplicate,      // Duplicate within same scope or across scopes
    NonNormalized,  // Contains short names, env vars, or can be expanded
    DeadDuplicate,  // Both dead and duplicate
}

impl PathStatus {
    /// Check if this status indicates a problem
    pub fn is_problematic(&self) -> bool {
        matches!(
            self,
            PathStatus::Dead | PathStatus::Duplicate | PathStatus::DeadDuplicate
        )
    }

    /// Get a human-readable description
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
    pub original: String,
    pub normalized: String,
    pub status: PathStatus,
    pub exists: bool,
    pub is_duplicate: bool,
    pub needs_normalization: bool,
}

/// Analyze a list of path entries
pub fn analyze_paths(paths: &[String], other_scope_paths: &[String]) -> Vec<PathInfo> {
    let mut results = Vec::new();
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
    expanded = expanded.trim_end_matches('\\').trim_end_matches('/').to_string();

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
    fn test_normalize_path() {
        // This will use actual environment variables
        let path = r"%USERPROFILE%\AppData\Local";
        let normalized = normalize_path(path);
        assert!(!normalized.contains('%'));
    }

    #[test]
    fn test_path_exists() {
        assert!(path_exists(r"C:\Windows"));
        assert!(!path_exists(r"C:\ThisPathDoesNotExist123456"));
    }

    #[test]
    fn test_find_duplicates() {
        let paths = vec![
            r"C:\Windows".to_string(),
            r"C:\windows".to_string(), // Case variation
            r"C:\Program Files".to_string(),
        ];

        let info = analyze_paths(&paths, &[]);
        let duplicates: Vec<_> = info
            .iter()
            .filter(|i| i.is_duplicate)
            .collect();

        assert_eq!(duplicates.len(), 2); // The two Windows entries
    }
}
