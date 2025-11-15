# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Structure

This is a monorepo containing multiple CLI tools. Each tool is located in its own subdirectory at the root level:

- `path-commander/` - A TUI application for managing Windows PATH environment variables (Rust)

Each tool is self-contained with its own build system, dependencies, and documentation.

## Path Commander

### Build Commands

```bash
# Navigate to the project directory
cd path-commander

# Build release binary
cargo build --release

# Build and run (for testing)
cargo run

# Run the built binary
./target/release/pc.exe

# Alternative: Use provided build scripts
./build.ps1     # PowerShell
./build.bat     # Batch file
```

### Testing Requirements

Path Commander requires Windows to run. The application:
- Only runs on Windows (enforced at compile time with `#[cfg(not(target_os = "windows"))]`)
- Requires Windows Registry access for reading/writing PATH variables
- Uses Windows-specific APIs for admin privilege detection

### Architecture Overview

**Core State Management (`app.rs`)**
- `App` struct contains all application state (paths, selections, marked items, modes)
- Dual-panel architecture: separate state for MACHINE (system) and USER scopes
- Modal interface with states: Normal, Help, Confirm, Input, BackupList
- Event-driven model: keyboard, mouse, and terminal events drive state changes

**Registry Integration (`registry.rs`)**
- Direct Windows Registry access via `windows-rs` crate
- Reads from `HKEY_CURRENT_USER\Environment` (USER paths)
- Reads from `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Session Manager\Environment` (MACHINE paths)
- Writes PATH changes back to registry and broadcasts `WM_SETTINGCHANGE` message

**Path Analysis (`path_analyzer.rs`)**
- Analyzes paths for: existence, duplicates (case-insensitive), normalization needs
- Detects dead paths (don't exist on filesystem)
- Identifies non-normalized paths (short names like `PROGRA~1`, unexpanded env vars like `%USERPROFILE%`)
- Cross-scope duplicate detection (between USER and MACHINE scopes)

**UI Rendering (`ui.rs`)**
- Ratatui-based TUI with dual panels (left=MACHINE, right=USER)
- Color-coded paths: Green (valid), Red (dead), Yellow (duplicate), Cyan (non-normalized)
- Supports checkboxes for multi-select operations
- Modal dialogs for confirmations and input
- Full mouse support (clicking, scrolling, shift/ctrl modifiers)

**Event Handling (`main.rs`)**
- Event deduplication: filters duplicate events within 200ms window (Windows/MSYS2 console buffering issue)
- Input flush on startup: clears buffered Enter keys from application launch
- Viewport-aware scrolling: PgUp/PgDn jump by visible screen height minus 1 for context

**Permission System (`permissions.rs`)**
- Detects administrator privileges at startup
- USER paths can be modified without admin rights
- MACHINE paths require administrator access
- Clear visual indicators of permission levels

**Backup System (`backup.rs`)**
- JSON-based backups stored in `%LOCALAPPDATA%\PathCommander\backups\`
- Automatic backup before applying changes
- Manual backup/restore functionality
- Timestamped backup files: `path_backup_YYYYMMDD_HHMMSS.json`

### Key Implementation Details

**Scrollbar State Management**
- Each panel maintains its own `ScrollbarState` (ratatui)
- State must be updated on: selection changes, path additions/deletions, panel switches
- Position and content length must stay in sync with actual paths

**Mouse Event Handling**
- Click regions: checkbox (cols 1-5), scrollbar (second-to-last column), content area
- Ctrl+Click: toggle mark without changing selection
- Shift+Click: range select from current selection to clicked item
- Scrollbar click: jump to position based on percentage

**Input Buffer Management**
- Single `input_buffer: String` shared across Input modes (AddPath, EditPath)
- Buffer must be cleared on mode exit (Enter/Esc)
- For EditPath mode, buffer is pre-populated with current path value

**Directory Creation Feature**
- When adding a non-existent path, prompts user to create directory
- Validates path is creatable (not network path, no invalid chars)
- Uses `std::fs::create_dir_all` to create parent directories
- Reanalyzes after creation to update "dead path" status

**Marked Items Tracking**
- Separate `HashSet<usize>` for machine_marked and user_marked
- Indices refer to positions in respective path vectors
- Must be cleared/adjusted when paths are deleted, moved, or reordered
- Used for batch operations: delete, normalize, move to other panel

### Common Pitfalls

1. **Scrollbar sync**: After modifying paths, always call `reanalyze()` which updates scrollbar content length
2. **Index bounds**: Always check indices are valid before accessing paths/info (users can click/select during transitions)
3. **Mode transitions**: Ensure `input_buffer` and `pending_directory` are cleared when exiting modes
4. **Duplicate detection**: Uses case-insensitive normalized comparison (`to_lowercase()`)
5. **Event deduplication**: Don't remove the 200ms window check - it's necessary for Windows/MSYS2 console environments

### File Dependencies

- `main.rs` → orchestrates event loop, terminal setup
- `app.rs` → core state and business logic (1200+ lines)
- `ui.rs` → rendering (300+ lines of layout/widget code)
- `registry.rs` → Windows Registry I/O
- `path_analyzer.rs` → path validation and normalization
- `backup.rs` → JSON serialization/deserialization
- `permissions.rs` → Windows privilege detection

### Release Build Optimization

The `Cargo.toml` configures aggressive release optimizations:
- `opt-level = "z"` - optimize for size
- `lto = true` - link-time optimization
- `codegen-units = 1` - single codegen unit for better optimization
- `strip = true` - strip debug symbols

Target binary size: ~2-3 MB (standalone, no dependencies)
