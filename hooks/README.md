# Git Hooks

This directory contains Git hook templates that enforce code quality standards for the cli-tools repository.

## What are Git Hooks?

Git hooks are scripts that Git executes before or after events such as: commit, push, and receive. They're a useful way to enforce coding standards and run automated checks.

## Available Hooks

### pre-commit

Runs before each commit to ensure code quality. This hook performs the following checks on all Rust projects in the repository:

1. **Code Formatting** (`cargo fmt --check`)
   - Ensures code follows Rust formatting standards
   - Auto-fix with: `cargo fmt`

2. **Linting** (`cargo clippy`)
   - Catches common mistakes and enforces best practices
   - Treats all warnings as errors
   - Check with: `cargo clippy --all-targets --all-features -- -D warnings`

3. **Build Verification** (`cargo check`)
   - Ensures the code compiles without errors
   - Check with: `cargo check --all-targets`

4. **Unit Tests** (`cargo test`)
   - Runs all unit and integration tests
   - Ensures no breaking changes
   - Run with: `cargo test`

5. **Documentation** (`cargo doc`) - Non-blocking
   - Verifies documentation builds successfully
   - Warnings won't block commits
   - Check with: `cargo doc --no-deps`

6. **Security Audit** (`cargo audit`) - Optional, Non-blocking
   - Checks dependencies for known vulnerabilities
   - Requires `cargo-audit` to be installed
   - Install with: `cargo install cargo-audit`

## Installation

### Linux / macOS / Git Bash (Windows)

```bash
# From repository root
./hooks/install-hooks.sh
```

Or make it executable first:

```bash
chmod +x hooks/install-hooks.sh
./hooks/install-hooks.sh
```

### Windows PowerShell

```powershell
# From repository root
.\hooks\install-hooks.ps1
```

### Windows Command Prompt

```cmd
REM From repository root
hooks\install-hooks.bat
```

## Bypassing Hooks

In emergency situations, you can bypass the pre-commit hook:

```bash
git commit --no-verify -m "Emergency fix"
```

**Note:** This should be used sparingly and only when absolutely necessary. Code that bypasses hooks may be rejected during code review.

## Uninstalling Hooks

To remove the hooks:

```bash
# Linux / macOS / Git Bash
rm .git/hooks/pre-commit
```

```powershell
# Windows PowerShell
Remove-Item .git\hooks\pre-commit
```

## Performance Considerations

The pre-commit hook runs several checks which may take 10-30 seconds depending on your system and the size of changes. This is intentional to catch issues early before they reach code review.

If you're making many small commits, consider:
- Making logical, complete commits rather than many tiny ones
- Running `cargo fmt` and `cargo clippy` manually during development
- Using `--no-verify` for WIP commits on feature branches (but clean them up before merging)

## Troubleshooting

### Hook not running

- Ensure the hook is installed: `ls -l .git/hooks/pre-commit`
- Check the hook is executable (Unix-like systems): `chmod +x .git/hooks/pre-commit`
- Verify you're committing Rust files (the hook only runs for `.rs` and `.toml` files)

### Permission errors on Windows

- Run your terminal as Administrator
- Or use Git Bash instead of Command Prompt

### Clippy warnings failing the commit

Fix all clippy warnings before committing:

```bash
cd path-commander
cargo clippy --all-targets --all-features -- -D warnings
```

### Tests failing

Ensure all tests pass:

```bash
cd path-commander
cargo test
```

## Contributing

When adding new hooks:

1. Add the hook template to this directory
2. Update the installation scripts
3. Update this README
4. Test on multiple platforms (Windows, Linux, macOS)
