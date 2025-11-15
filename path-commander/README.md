# Path Commander (`pc`)

A powerful Terminal User Interface (TUI) for managing Windows PATH environment variables, inspired by Midnight Commander.

## Features

- **Dual-Panel Interface**: View and manage MACHINE (system) and USER paths side-by-side
- **Intelligent Analysis**:
  - Detects duplicate entries (case-insensitive, normalized comparison)
  - Identifies "dead" paths that don't exist on the filesystem
  - Highlights non-normalized paths (short names, environment variables)
- **Color-Coded Display**:
  - 🟢 Green: Valid, unique, normalized paths
  - 🔴 Red: Dead paths (don't exist)
  - 🟡 Yellow: Duplicate paths
  - 🔵 Cyan: Non-normalized paths (can be expanded/normalized)
- **Multi-Select Operations**: Mark multiple paths with checkboxes for batch operations
- **Path Operations**:
  - Add/Edit/Delete individual paths
  - Remove all duplicates with one command
  - Remove all dead paths with one command
  - Normalize paths (expand short names and environment variables)
  - Move paths between USER and MACHINE scopes
  - Reorder paths
- **Safety Features**:
  - Staged changes (review before applying)
  - Automatic backup before applying changes
  - Manual backup/restore functionality
  - Timestamped backup files
  - Confirmation dialogs for destructive operations
- **Permission Handling**:
  - Automatic detection of administrator privileges
  - USER paths editable without admin rights
  - MACHINE paths require administrator privileges
  - Clear visual indicators of permission levels

## Installation

### Prerequisites

- Windows OS
- Rust toolchain (1.70 or later)

### Building from Source

```bash
# Clone the repository
cd path-commander

# Build release binary
cargo build --release

# The executable will be at: target/release/pc.exe
```

### Installing the Binary

```bash
# Option 1: Copy to a directory in your PATH
copy target\release\pc.exe C:\Windows\System32\

# Option 2: Add the target/release directory to your PATH
```

## Usage

### Starting Path Commander

```bash
# Run with user privileges (can only edit USER paths)
pc

# Run as administrator (can edit both USER and MACHINE paths)
# Right-click Command Prompt/PowerShell and "Run as Administrator"
pc
```

### Keyboard Shortcuts

#### Navigation
- `↑/↓`, `j/k` - Move selection up/down
- `PgUp/PgDn` - Move selection by 10 items
- `Home/End` - Jump to first/last item
- `Tab`, `←/→` - Switch between MACHINE and USER panels

#### Selection
- `Space`, `Insert`, `F2` - Toggle mark on current item

#### Actions
- `F1`, `?` - Show help screen
- `F3`, `Delete` - Delete marked items
- `F4` - Add new path
- `F5` - Move marked items to other panel (USER ↔ MACHINE)
- `F6` - Move current item up in order
- `F7` - Remove all duplicate paths
- `F8` - Remove all dead paths
- `F9` - Normalize marked paths
- `Enter` - Edit current path

#### Save/Restore
- `Ctrl+S` - Apply changes to Windows Registry
- `Ctrl+B` - Create manual backup
- `Ctrl+R` - Restore from backup

#### Other
- `Q`, `F10`, `Esc` - Exit (with confirmation if changes exist)
- `Ctrl+C` - Force quit

### Mouse Support

Path Commander has full mouse support for efficient navigation and editing:

#### Basic Mouse Operations
- **Click** - Select a path entry and switch to that panel
- **Double-click** - Edit the selected path (same as pressing Enter)
- **Scroll wheel** - Scroll through the path list
- **Click on scrollbar** - Jump to that position in the list
- **Click on checkbox** - Toggle mark on that path
- **Click on key hints** - Execute that command (F1-F9, Ctrl+S, etc.)

#### Advanced Mouse Operations
- **Ctrl+Click** - Toggle mark on an item without changing selection
- **Shift+Click** - Range select (mark all items between current selection and clicked item)

You can mix keyboard and mouse interactions seamlessly for maximum efficiency.

## How It Works

### Path Analysis

Path Commander analyzes each PATH entry to determine:

1. **Existence**: Does the path exist on the filesystem?
2. **Duplicates**: Is this path duplicated (case-insensitive)?
   - Checks within the same scope (USER or MACHINE)
   - Checks across scopes
3. **Normalization**: Can the path be normalized?
   - Expands environment variables (e.g., `%USERPROFILE%`)
   - Converts short names to long names (e.g., `PROGRA~1` → `Program Files`)
   - Removes trailing slashes

### Backup System

Backups are stored in JSON format at:
```
%LOCALAPPDATA%\PathCommander\backups\
```

Each backup contains:
- Timestamp
- Complete USER PATH string
- Complete MACHINE PATH string
- Individual path entries for both scopes

Backup filename format: `path_backup_YYYYMMDD_HHMMSS.json`

### Applying Changes

When you press `Ctrl+S`:
1. A backup is automatically created
2. Changes are written to the Windows Registry
3. A `WM_SETTINGCHANGE` message is broadcast to notify other applications
4. Path Commander detects running processes that won't pick up the changes
5. If any non-responsive processes are found, a notification dialog is shown
6. The status bar confirms success

### Process Restart Notifications

After applying changes, Path Commander automatically detects running processes that don't respond to environment variable updates. If any are found, you'll see a dialog listing:

- **Terminals**: cmd.exe, powershell.exe, pwsh.exe, bash.exe, Windows Terminal
- **Editors**: VS Code, Visual Studio, JetBrains IDEs, Sublime Text, Notepad++
- **Other**: Console Host, MinTTY, Atom

These processes load environment variables at startup and must be restarted to see the new PATH.

**Note**: New processes started after saving will automatically see the updated PATH.

## Examples

### Remove All Dead Paths

1. Press `F8`
2. Confirm with `Y`
3. Press `Ctrl+S` to apply changes

### Remove Duplicates

1. Press `F7`
2. Confirm with `Y`
3. Press `Ctrl+S` to apply changes

### Normalize Paths

1. Mark paths to normalize (they'll show in cyan)
2. Press `F9`
3. Press `Ctrl+S` to apply changes

### Move Paths from MACHINE to USER

1. Select the MACHINE panel (left side)
2. Mark the paths you want to move
3. Press `F5`
4. Paths move to USER panel
5. Press `Ctrl+S` to apply changes

### Manual Backup and Restore

```bash
# Create a backup
Ctrl+B

# Later, restore from backup
Ctrl+R
# Select backup from list
Enter
# Confirm restore
Y
# Apply changes
Ctrl+S
```

## Safety Considerations

- **Always review changes** before pressing `Ctrl+S`
- **Backups are created automatically** before applying changes
- **Critical system paths**: Be careful when deleting paths that might be needed by Windows or applications
- **Test in a VM first** if you're unsure about operations
- **Keep a manual backup**: Consider exporting your PATH to a text file before major changes

## Troubleshooting

### "Failed to open registry key" Error

- You need administrator privileges to modify MACHINE paths
- Right-click your terminal and select "Run as Administrator"

### Changes Not Reflected in Open Applications

- Most applications read PATH on startup
- Restart the application to pick up changes
- Some system components may require a reboot

### "Access Denied" When Writing MACHINE Paths

- Ensure you're running as Administrator
- Check that antivirus/security software isn't blocking registry writes

### Backup Directory Not Found

- The application creates it automatically at: `%LOCALAPPDATA%\PathCommander\backups\`
- If this fails, check disk space and permissions

## Technical Details

- **Language**: Rust
- **TUI Framework**: ratatui
- **Terminal Backend**: crossterm
- **Windows API**: windows-rs crate
- **Binary Size**: ~2-3 MB (optimized release build)
- **Dependencies**: None (single, standalone executable)

## License

MIT License - see LICENSE file for details

## Author

Jesse Slaton (github@doxazo.net)

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

For developers:
- See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines
- See [TESTING.md](TESTING.md) for information about the test suite

## Roadmap

Potential future features:
- Export/import PATH to various formats (CSV, JSON, plain text)
- Undo/redo functionality
- Search/filter paths
- Duplicate detection across similar patterns (e.g., different versions of the same tool)
- Integration with UAC elevation from within the app
- Themes/color customization (basic theme support already implemented)

## Acknowledgments

- Inspired by Midnight Commander's dual-panel interface
- Built with the excellent Rust ecosystem
- Thanks to the ratatui and crossterm communities
