# Contributing to CLI Tools

Thank you for your interest in contributing to the CLI Tools repository! This document provides guidelines and instructions for contributing.

## Getting Started

### Prerequisites

- Git
- Rust (latest stable version)
- Windows 10 or later (for testing Windows-specific tools like Path Commander)

### Setting Up Your Development Environment

1. **Fork and clone the repository**

   ```bash
   git clone https://github.com/YOUR_USERNAME/cli-tools.git
   cd cli-tools
   ```

2. **Install Git hooks**

   We use Git hooks to maintain code quality. Install them before making changes:

   **Linux / macOS / Git Bash:**
   ```bash
   ./hooks/install-hooks.sh
   ```

   **Windows PowerShell:**
   ```powershell
   .\hooks\install-hooks.ps1
   ```

   **Windows Command Prompt:**
   ```cmd
   hooks\install-hooks.bat
   ```

3. **Install development tools (optional but recommended)**

   ```bash
   # Code formatting (required by hooks)
   rustup component add rustfmt

   # Linting (required by hooks)
   rustup component add clippy

   # Security auditing (optional)
   cargo install cargo-audit
   ```

## Code Quality Standards

All code submissions must pass the following checks (enforced by pre-commit hooks):

### 1. Code Formatting

All Rust code must be formatted with `rustfmt`:

```bash
cargo fmt
```

### 2. Linting

Code must pass all Clippy lints without warnings:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### 3. Build Verification

Code must compile without errors:

```bash
cargo check --all-targets
```

### 4. Tests

All tests must pass:

```bash
cargo test
```

### 5. Documentation

Public APIs should be documented. Documentation must build without errors:

```bash
cargo doc --no-deps
```

## Making Changes

### 1. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
```

Use descriptive branch names:
- `feature/add-xyz` for new features
- `fix/issue-123` for bug fixes
- `docs/update-readme` for documentation
- `refactor/simplify-abc` for refactoring

### 2. Make Your Changes

- Write clean, readable code
- Follow Rust naming conventions and idioms
- Add tests for new functionality
- Update documentation as needed
- Keep commits focused and atomic

### 3. Test Your Changes

Before committing, ensure:

```bash
# Run all checks
cd path-commander  # or your project directory
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

### 4. Commit Your Changes

Commits should:
- Have clear, descriptive messages
- Follow the conventional commit format (optional but recommended):
  - `feat:` for new features
  - `fix:` for bug fixes
  - `docs:` for documentation
  - `test:` for tests
  - `refactor:` for refactoring
  - `chore:` for maintenance tasks

Example:
```bash
git commit -m "feat: add duplicate detection across scopes"
```

The pre-commit hook will run automatically. If it fails:
- Fix the reported issues
- Stage your fixes: `git add .`
- Try committing again

### 5. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a pull request on GitHub with:
- Clear description of changes
- Reference to any related issues
- Screenshots (if UI changes)
- Test results (if applicable)

## Project Structure

This is a monorepo containing multiple CLI tools:

```
cli-tools/
├── hooks/              # Git hooks and installation scripts
├── path-commander/     # PATH management TUI (Windows)
│   ├── src/
│   ├── tests/
│   ├── Cargo.toml
│   └── README.md
└── README.md
```

Each tool is self-contained with its own build system and dependencies.

## Testing

### Running Tests

```bash
# Run all tests for a specific project
cd path-commander
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in release mode (faster)
cargo test --release
```

### Writing Tests

- Place unit tests in the same file as the code (`#[cfg(test)]` module)
- Place integration tests in `tests/` directory
- Use descriptive test names
- Test edge cases and error conditions

## Code Review Process

1. All submissions require review
2. Address reviewer feedback promptly
3. Keep the PR focused and reasonably sized
4. Rebase on main if requested
5. Ensure CI checks pass

## Bypassing Hooks (Emergency Only)

In rare emergency situations, you can bypass pre-commit hooks:

```bash
git commit --no-verify -m "Emergency: fix critical bug"
```

**Note:** Use this sparingly! Code that bypasses hooks may be rejected during review.

## Getting Help

- Check existing issues on GitHub
- Read tool-specific documentation in each project's directory
- Review the `CLAUDE.md` file for architectural guidance
- Ask questions in issue discussions

## Code of Conduct

- Be respectful and constructive
- Welcome newcomers
- Focus on the code, not the person
- Assume good intentions

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.

## Recognition

Contributors will be recognized in release notes and the project README.

Thank you for contributing! 🎉
