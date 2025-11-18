# CLI Tools

A collection of command-line utilities and tools.

## Tools

### [Path Commander](./path-commander/) (v0.6.0)
A powerful Terminal User Interface (TUI) for managing Windows PATH environment variables, inspired by Midnight Commander.

**Features:**
- **Dual-panel interface** - Visual side-by-side management of USER and MACHINE PATH variables
- **Remote computer management** - Manage PATH variables on remote Windows computers via UNC paths
- **Intelligent analysis** - Dead path detection, duplicate detection (case-insensitive), cross-scope duplicate detection
- **Color-coded display** - Green (valid), Red (dead), Yellow (duplicate), Cyan (non-normalized)
- **Multi-select operations** - Mark multiple paths with checkboxes for batch operations
- **Path cleanup tools** - Remove all duplicates, remove all dead paths, normalize paths (expand variables)
- **Safety features** - Automatic backups before changes, manual backup/restore, undo/redo support
- **Full mouse support** - Click, drag, scroll, Ctrl+Click, Shift+Click range selection
- **Theming system** - Compatible with Midnight Commander .ini themes, live preview
- **Menu system** - Drop-down menus (File, Command, Options, Help) with keyboard shortcuts
- **Permission handling** - Auto-detects admin privileges, clear visual indicators

**Requirements:**
- Windows 10 or later
- Administrator privileges (for modifying MACHINE paths)
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
