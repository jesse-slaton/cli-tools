# Testing Guide for Path Commander

This document describes the testing framework and practices for Path Commander.

## Overview

Path Commander uses Rust's built-in testing framework along with specialized testing libraries for comprehensive test coverage. The test suite includes:

- **Unit tests**: Test individual functions and modules in isolation (68 tests)
- **Integration tests**: Test project structure and dependencies (4 tests)
- **Total: 72 tests** covering core functionality

## Running Tests

### Run All Tests

```bash
cargo test
```

### Run Tests for Specific Module

```bash
# Test path analyzer
cargo test path_analyzer

# Test backup functionality
cargo test backup

# Test app state management
cargo test app
```

### Run with Output

```bash
cargo test -- --nocapture
```

### Run Integration Tests Only

```bash
cargo test --test integration_test
```

## Test Coverage

### Path Analyzer Module (`src/path_analyzer.rs`)
- ✅ Path normalization (environment variables, trailing slashes)
- ✅ Path existence checking
- ✅ Duplicate detection (case-insensitive, cross-scope)
- ✅ Dead path detection
- ✅ Path status determination
- ✅ Environment variable expansion
- ✅ Edge cases (empty paths, multiple duplicates)

**Tests**: 22 unit tests

### Backup Module (`src/backup.rs`)
- ✅ Backup creation and serialization
- ✅ Save/load roundtrip
- ✅ Backup listing and filtering
- ✅ Old backup cleanup
- ✅ Directory creation
- ✅ Error handling (invalid JSON, nonexistent files)
- ✅ Special characters and long paths
- ✅ Multiple backups with distinct timestamps

**Tests**: 19 unit tests

### App Module (`src/app.rs`)
- ✅ Panel toggling and scope conversion
- ✅ Selection movement and bounds checking
- ✅ Item marking/unmarking
- ✅ Mode transitions
- ✅ Path deletion (marked items)
- ✅ Path normalization
- ✅ Statistics generation
- ✅ Viewport height management
- ✅ Directory creation validation
- ✅ State consistency after operations

**Tests**: 27 unit tests

### Integration Tests (`tests/integration_test.rs`)
- ✅ Project structure validation
- ✅ Binary configuration
- ✅ Required dependencies
- ✅ Dev dependencies

**Tests**: 4 integration tests

## Test Dependencies

The project uses the following testing libraries (configured in `Cargo.toml`):

```toml
[dev-dependencies]
mockall = "0.13"      # Mocking framework (for future use)
proptest = "1.5"      # Property-based testing (for future use)
tempfile = "3.8"      # Temporary file/directory handling
```

## Writing Tests

### Unit Test Structure

Unit tests are co-located with the code they test using `#[cfg(test)]` modules:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = create_test_data();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected_value);
    }
}
```

### Testing Best Practices

1. **Isolation**: Tests should not depend on each other or external state
2. **Clarity**: Test names should clearly describe what is being tested
3. **Coverage**: Test both happy paths and edge cases
4. **Speed**: Keep tests fast; use `tempfile::TempDir` for file system tests
5. **Determinism**: Tests should produce consistent results

### Helper Functions

For complex modules like `app.rs`, create helper functions to set up test state:

```rust
fn create_test_app(machine_paths: Vec<String>, user_paths: Vec<String>) -> App {
    // Create app without requiring Windows Registry access
    // ...
}
```

## Continuous Integration

### GitHub Actions Workflow

Add this to `.github/workflows/test.yml` to run tests on every push:

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --verbose
```

## Known Test Limitations

### Timing-Sensitive Tests

Some backup tests (`test_list_backups_multiple`, `test_cleanup_old_backups_*`) use `std::thread::sleep` to ensure unique timestamps. These tests:

- Take ~3-4 seconds total to run
- May fail if system time resolution is very low
- Use 1100ms sleep between backups (filename precision is 1 second)

### Windows-Specific Functionality

Tests that require Windows Registry access or administrator privileges are not included in the automated test suite. These include:

- Actual registry read/write operations
- Admin privilege escalation
- WM_SETTINGCHANGE message broadcasting

## Test Metrics

Current test statistics (as of Nov 2025):

- **Total tests**: 72
- **Pass rate**: 100%
- **Execution time**: ~3-4 seconds
- **Code coverage**: High coverage of core logic modules

## Future Improvements

- [ ] Add property-based tests using `proptest`
- [ ] Add mock registry for testing Windows-specific functionality
- [ ] Increase test coverage for UI rendering logic
- [ ] Add benchmarks for performance-critical paths
- [ ] Generate code coverage reports
- [ ] Add mutation testing

## Contributing Tests

When adding new features:

1. Write tests for the new functionality
2. Ensure all existing tests still pass
3. Update this document if new test patterns are introduced
4. Follow the existing test structure and naming conventions

## Troubleshooting

### Tests Fail on File System Operations

If backup tests fail, ensure:
- You have write permissions to the temp directory
- System time is configured correctly
- No antivirus is blocking file creation

### Tests Run Slowly

- Backup tests with sleep delays are expected to take 3-4 seconds
- Other tests should complete in milliseconds
- Run specific test modules to speed up iteration

## Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Cargo Test Reference](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [Tempfile Documentation](https://docs.rs/tempfile/)
