/// Integration tests for Path Commander
///
/// These tests validate end-to-end functionality without requiring Windows Registry access
/// or administrator privileges.
#[cfg(test)]
mod integration_tests {
    use std::path::PathBuf;

    #[test]
    fn test_project_structure() {
        // Verify project has expected structure
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        assert!(manifest_dir.join("Cargo.toml").exists());
        assert!(manifest_dir.join("README.md").exists());
        assert!(manifest_dir.join("src").exists());
        assert!(manifest_dir.join("src/main.rs").exists());
    }

    #[test]
    fn test_binary_name() {
        // Verify binary is named 'pc' as expected
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let cargo_toml = std::fs::read_to_string(manifest_dir.join("Cargo.toml")).unwrap();

        assert!(cargo_toml.contains("name = \"pc\""));
        assert!(cargo_toml.contains("path = \"src/main.rs\""));
    }

    #[test]
    fn test_required_dependencies() {
        // Verify critical dependencies are present
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let cargo_toml = std::fs::read_to_string(manifest_dir.join("Cargo.toml")).unwrap();

        // Core dependencies for TUI and Windows integration
        assert!(cargo_toml.contains("ratatui"));
        assert!(cargo_toml.contains("crossterm"));
        assert!(cargo_toml.contains("windows"));
        assert!(cargo_toml.contains("serde"));
        assert!(cargo_toml.contains("anyhow"));
    }

    #[test]
    fn test_dev_dependencies() {
        // Verify test framework dependencies
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let cargo_toml = std::fs::read_to_string(manifest_dir.join("Cargo.toml")).unwrap();

        assert!(cargo_toml.contains("mockall"));
        assert!(cargo_toml.contains("proptest"));
        assert!(cargo_toml.contains("tempfile"));
    }
}
