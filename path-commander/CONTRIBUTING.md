# Contributing to Path Commander

Thank you for your interest in contributing to Path Commander! This guide will help you get started with development.

## Table of Contents

- [Development Setup](#development-setup)
- [Building the Project](#building-the-project)
- [Testing](#testing)
- [Code Style](#code-style)
- [Submitting Changes](#submitting-changes)
- [Architecture Overview](#architecture-overview)

## Development Setup

### Prerequisites

- **Windows OS** (required for development and testing)
- **Rust toolchain** 1.70 or later
  - Install from [rustup.rs](https://rustup.rs/)
- **Git** for version control
- **Code editor** (VS Code with rust-analyzer recommended)

### Clone the Repository

```bash
git clone https://github.com/yourusername/cli-tools.git
cd cli-tools/path-commander
```

### Install Dependencies

Dependencies are managed through Cargo and will be installed automatically:

```bash
cargo build
```

## Building the Project

### Development Build

```bash
# Build with debug symbols (faster compilation)
cargo build

# Run directly (for testing)
cargo run
```

### Release Build

```bash
# Build optimized binary
cargo build --release

# The binary will be at: target/release/pc.exe
```

### Build Scripts

Convenience scripts are provided:

- `build.ps1` - PowerShell build script
- `build.bat` - Batch build script

## Testing

Path Commander has a comprehensive test suite with 72 tests. See [TESTING.md](TESTING.md) for detailed information.

### Quick Test Commands

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test path_analyzer
cargo test backup
cargo test app

# Run tests with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_test
```

### Test Requirements

- Tests run on Windows only (Windows-specific functionality)
- Some tests require ~3-4 seconds due to timestamp-based backup tests
- All tests must pass before submitting a pull request

### Writing Tests

- Add unit tests in the same file as the code using `#[cfg(test)]` modules
- Add integration tests in the `tests/` directory
- Follow existing test patterns and naming conventions
- Test both happy paths and edge cases
- Keep tests isolated and deterministic

## Code Style

### Rust Style Guidelines

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for code formatting:
  ```bash
  cargo fmt
  ```
- Run Clippy for linting:
  ```bash
  cargo clippy -- -D warnings
  ```

### Naming Conventions

- Functions: `snake_case`
- Types/Structs: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

### Documentation

- Add doc comments (`///`) for public APIs
- Include examples in doc comments where helpful
- Update CLAUDE.md if architecture changes significantly

### Error Handling

- Use `anyhow::Result` for functions that can fail
- Provide context with `.context()` or `.with_context()`
- Avoid unwrapping in production code; use proper error handling

## Submitting Changes

### Before You Submit

1. **Run tests**: `cargo test`
2. **Format code**: `cargo fmt`
3. **Check lints**: `cargo clippy`
4. **Build release**: `cargo build --release`
5. **Test manually** in both admin and non-admin modes

### Pull Request Process

1. **Create a branch** for your feature or fix:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following the code style guidelines

3. **Add tests** for new functionality

4. **Update documentation**:
   - README.md for user-facing changes
   - CLAUDE.md for architectural changes
   - Doc comments for API changes

5. **Commit your changes** with clear commit messages:
   ```bash
   git commit -m "Add feature: description of your change"
   ```

6. **Push to your fork** and create a pull request:
   ```bash
   git push origin feature/your-feature-name
   ```

7. **Describe your changes** in the PR description:
   - What problem does it solve?
   - How did you test it?
   - Are there any breaking changes?

### Commit Message Guidelines

- Use present tense ("Add feature" not "Added feature")
- Use imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit first line to 72 characters
- Reference issues with `#123` notation

Examples:
```
Add undo functionality for path operations

Fix crash when selecting empty path list

Improve performance of duplicate detection (Issue #42)
```

## Architecture Overview

### Project Structure

```
path-commander/
├── src/
│   ├── main.rs              # Entry point and event loop
│   ├── app.rs               # Core application state (1200+ lines)
│   ├── ui.rs                # UI rendering logic
│   ├── registry.rs          # Windows Registry I/O
│   ├── path_analyzer.rs     # Path validation and analysis
│   ├── backup.rs            # Backup/restore functionality
│   ├── permissions.rs       # Admin privilege detection
│   ├── process_detector.rs  # Process detection for notifications
│   └── theme.rs             # Color theme support
├── tests/                   # Integration tests
├── Cargo.toml               # Dependencies and metadata
├── README.md                # User documentation
├── TESTING.md               # Testing guide
├── CONTRIBUTING.md          # This file
└── CLAUDE.md                # Detailed architecture for AI assistants
```

### Key Components

**App State (`app.rs`)**
- Central state management
- Dual-panel architecture (MACHINE/USER)
- Modal interface (Normal, Help, Confirm, Input, BackupList)
- Event-driven model

**Registry Integration (`registry.rs`)**
- Direct Windows Registry access
- PATH reading/writing
- WM_SETTINGCHANGE broadcasting

**Path Analysis (`path_analyzer.rs`)**
- Path existence checking
- Duplicate detection (case-insensitive)
- Normalization (env vars, short names)

**UI Rendering (`ui.rs`)**
- Ratatui-based TUI
- Dual panels with color coding
- Modal dialogs
- Mouse support

See [CLAUDE.md](CLAUDE.md) for detailed architectural information.

### Common Development Tasks

#### Adding a New Feature

1. Plan the feature (consider creating an issue first)
2. Identify which module(s) need changes
3. Write tests first (TDD approach recommended)
4. Implement the feature
5. Test manually in both admin/non-admin modes
6. Update documentation

#### Debugging

```bash
# Run with debug output
RUST_LOG=debug cargo run

# Run tests with backtrace
RUST_BACKTRACE=1 cargo test

# Check for memory issues (requires nightly)
cargo +nightly miri test
```

#### Performance Profiling

```bash
# Build with profiling enabled
cargo build --release --features profiling

# Use Windows Performance Analyzer or similar tools
```

## Getting Help

- **Issues**: Check existing [GitHub Issues](https://github.com/yourusername/cli-tools/issues)
- **Discussions**: Start a discussion for questions or ideas
- **Code Review**: Ask for review on your pull request

## Code of Conduct

- Be respectful and constructive
- Welcome newcomers and help them learn
- Focus on what is best for the project
- Show empathy towards other community members

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

If you have questions about contributing, feel free to:
- Open an issue with the `question` label
- Start a discussion in GitHub Discussions
- Contact the maintainer: Jesse Slaton (github@doxazo.net)

---

Thank you for contributing to Path Commander! 🎉
