# CLI Tools

A collection of command-line utilities and tools.

## Tools

### [Path Commander](./path-commander/)
A terminal user interface (TUI) application for managing Windows PATH environment variables.

**Features:**
- Visual interface for managing USER and MACHINE PATH variables
- Dead path detection and cleanup
- Duplicate path detection
- Path normalization
- Backup and restore functionality
- Full mouse support
- Directory creation for non-existent paths

**Requirements:**
- Windows 10 or later
- Rust (for building from source)

## Installation

See individual tool directories for installation instructions.

## Building from Source

Each tool has its own build instructions in its respective directory.

## Contributing

Contributions are welcome! Please read our [Contributing Guidelines](./CONTRIBUTING.md) before submitting changes.

### Quick Start for Contributors

1. **Install Git hooks** (enforces code quality):
   ```bash
   # Linux / macOS / Git Bash
   ./hooks/install-hooks.sh

   # Windows PowerShell
   .\hooks\install-hooks.ps1
   ```

2. **Make your changes** following our code quality standards

3. **Submit a pull request**

See [CONTRIBUTING.md](./CONTRIBUTING.md) for detailed guidelines.

## Code Quality

This repository uses automated pre-commit hooks to maintain code quality:
- Code formatting with `rustfmt`
- Linting with `clippy`
- Build verification
- Automated testing
- Documentation checks

See [hooks/README.md](./hooks/README.md) for more information.

## License

See individual tool directories for license information.
