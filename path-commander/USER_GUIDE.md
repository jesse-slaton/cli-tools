# Path Commander User Guide

**Version 0.6.0**

Path Commander is a Terminal User Interface (TUI) application for managing Windows PATH environment variables. This guide will help you understand and use all of its features effectively.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Understanding the Interface](#understanding-the-interface)
3. [Basic Operations](#basic-operations)
4. [Advanced Features](#advanced-features)
5. [Remote Computer Management](#remote-computer-management)
6. [Theming and Customization](#theming-and-customization)
7. [Safety and Backups](#safety-and-backups)
8. [Troubleshooting](#troubleshooting)
9. [Tips and Best Practices](#tips-and-best-practices)

---

## Getting Started

### Installation

1. **Download or build** the Path Commander executable (`pc.exe`)
2. **Place it in your PATH** or run from the build directory:
   ```bash
   # Option 1: Copy to a directory already in PATH
   copy target\release\pc.exe C:\Windows\System32\

   # Option 2: Run from build directory
   cd path-commander
   cargo run --release
   ```

### First Launch

Simply run `pc` from your terminal:

```bash
# Run with user privileges (can edit USER paths only)
pc

# Run as administrator (can edit both USER and MACHINE paths)
# Right-click PowerShell/Command Prompt and select "Run as Administrator"
pc
```

**Important**: To modify system-wide MACHINE paths, you must run Path Commander as administrator.

### Understanding Privileges

Path Commander shows your current privilege level in the status bar:

- **Running as User**: Can only edit USER paths (right panel)
- **Running as Administrator**: Can edit both USER and MACHINE paths

The left panel header will show:
- `MACHINE (admin required)` - if you're not an admin
- `MACHINE` - if you are an admin

---

## Understanding the Interface

### Screen Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ File  Command  Options  Help                         [Menu Bar] │
├─────────────────────────────────────────────────────────────────┤
│ Paths: 15 | Duplicates: 2 | Dead: 3            [Status Header] │
├──────────────────────────┬──────────────────────────────────────┤
│ MACHINE (System)         │ USER                                 │
│ ┌──────────────────────┐ │ ┌──────────────────────────────────┐ │
│ │☐ C:\Windows\System32 │ │ │☑ C:\Users\Me\AppData\Local\bin  │ │
│ │☐ C:\Program Files... │ │ │☐ C:\MyTools                     │ │
│ │☑ C:\OldPath (dead)   │ │ │☐ %USERPROFILE%\Scripts          │ │
│ └──────────────────────┘ │ └──────────────────────────────────┘ │
├──────────────────────────┴──────────────────────────────────────┤
│ Ready | 2 changes pending            [Status Bar - 3 lines]    │
│ Press F1 for Help                                               │
│ Ctrl+S: Apply Changes | Ctrl+Z: Undo | Ctrl+Y: Redo            │
├─────────────────────────────────────────────────────────────────┤
│ 1Help│2Mark│3Del│4Add│5Move│6Up│7DupRm│8Dead│9Norm│10Quit      │
│                                                [Function Keys]  │
└─────────────────────────────────────────────────────────────────┘
```

### Color Coding

Paths are color-coded to show their status:

- **🟢 Green** - Valid, unique, normalized path
- **🔴 Red** - Dead path (doesn't exist on filesystem)
- **🟡 Yellow** - Duplicate path (exists elsewhere in PATH)
- **🔵 Cyan** - Non-normalized path (contains `%VARIABLES%` or short names like `PROGRA~1`)

### Panels

- **Left Panel**: MACHINE (system-wide) paths
  - Affects all users on the computer
  - Requires administrator privileges to modify

- **Right Panel**: USER paths
  - Affects only the current user
  - Can be modified without admin privileges

---

## Basic Operations

### Navigation

#### Keyboard Navigation
- **↑/↓** or **j/k** - Move selection up/down
- **PgUp/PgDn** - Jump 10 items
- **Home/End** - Jump to first/last item
- **Tab** or **←/→** - Switch between MACHINE and USER panels

#### Mouse Navigation
- **Click** - Select a path and switch to that panel
- **Scroll wheel** - Scroll through paths
- **Click scrollbar** - Jump to that position

### Adding Paths

**Method 1: Using Function Keys**
1. Press **F4** (or select **Command > Add Path**)
2. Type or paste the path
3. Press **Enter**

**Method 2: Using Menu**
1. Press **Alt+C** to open Command menu
2. Select **Add Path**
3. Type the path and press **Enter**

**Tips**:
- Press **Ctrl+V** to paste paths
- Use the file browser (activated automatically) to select existing directories
- If the path doesn't exist, you'll be prompted to create it

### Editing Paths

1. Select the path you want to edit
2. Press **Enter** or **double-click**
3. Modify the path in the input field
4. Press **Enter** to save or **Esc** to cancel

### Deleting Paths

**Single Path**:
1. Select the path
2. Press **F3** or **Delete**
3. Confirm with **Enter**

**Multiple Paths**:
1. Mark the paths you want to delete (see [Marking Items](#marking-items))
2. Press **F3** or **Delete**
3. Confirm with **Enter**

### Marking Items

Mark items for batch operations using checkboxes:

- **Space**, **Insert**, or **F2** - Toggle mark on current item
- **Ctrl+Click** - Toggle mark without changing selection
- **Shift+Click** - Mark all items between current selection and clicked item
- **Click checkbox** - Toggle mark on that specific item

Once items are marked (checkboxes show ☑), you can:
- Delete them all at once (F3)
- Move them to the other panel (F5)
- Normalize them all (F9)

### Reordering Paths

The order of paths matters! Windows searches PATH entries from left to right (top to bottom in Path Commander).

To move a path up in priority:
1. Select the path
2. Press **F6** repeatedly to move it up

### Applying Changes

**IMPORTANT**: Changes are staged until you apply them!

1. Make your changes (add, edit, delete, reorder)
2. Review the changes in the panels
3. Press **Ctrl+S** to apply changes to Windows Registry
4. A backup is automatically created before applying

The status bar shows: `X changes pending` when you have unapplied changes.

---

## Advanced Features

### Removing Duplicates

Duplicates waste space and can cause confusion. Path Commander detects duplicates (case-insensitive) both within a scope and across USER/MACHINE.

**Remove all duplicates at once**:
1. Press **F7** (or **Command > Remove Duplicates**)
2. Confirm with **Enter**
3. Press **Ctrl+S** to apply changes

When duplicates are found:
- Both instances are highlighted in yellow
- The first occurrence is kept
- Duplicates are marked for removal

### Removing Dead Paths

Dead paths point to directories that don't exist. They clutter your PATH and can slow down command execution.

**Remove all dead paths**:
1. Press **F8** (or **Command > Remove Dead Paths**)
2. Confirm with **Enter**
3. Press **Ctrl+S** to apply changes

**Create missing directories** instead of removing:
1. Press **Shift+F10** (or **Command > Create Marked Directories**)
2. Directories will be created for all marked dead paths
3. Paths will turn green once directories exist

### Normalizing Paths

Non-normalized paths contain:
- Environment variables like `%USERPROFILE%` or `%ProgramFiles%`
- Short names like `PROGRA~1`
- Trailing slashes

**Normalize paths**:
1. Mark the cyan paths you want to normalize (or they'll all be selected)
2. Press **F9** (or **Command > Normalize Paths**)
3. Paths are expanded to their full form
4. Press **Ctrl+S** to apply

Example:
- Before: `%USERPROFILE%\bin`
- After: `C:\Users\YourName\bin`

**Note**: Normalization makes paths more readable but less portable. Use judgment based on your needs.

### Moving Paths Between Scopes

Move paths from MACHINE to USER or vice versa:

1. Mark the paths to move in the source panel
2. Press **F5** (or **Command > Move to Other Panel**)
3. Paths are moved to the other panel
4. Press **Ctrl+S** to apply

**Use cases**:
- Move personal tools from MACHINE to USER (cleaner system PATH)
- Move shared tools from USER to MACHINE (available to all users)

### Undo and Redo

Path Commander supports unlimited undo/redo:

- **Ctrl+Z** - Undo last change
- **Ctrl+Y** - Redo last undone change

Undo/redo works for:
- Adding, editing, deleting paths
- Moving paths between panels
- Reordering paths
- Batch operations (duplicates, dead paths, normalize)

**Note**: Undo/redo resets when you apply changes (Ctrl+S) or restart the application.

### Filtering Paths

Use filters to focus on specific path types:

1. Press **F** or select **Options > Filter**
2. Choose a filter:
   - **All Paths** - Show everything (default)
   - **Dead Paths Only** - Show only paths that don't exist
   - **Duplicate Paths Only** - Show only duplicated paths
   - **Valid Paths Only** - Show only green (valid) paths
   - **Non-Normalized Paths** - Show only cyan paths
3. The display updates to show only matching paths

Filters don't modify data, just change what's visible.

---

## Remote Computer Management

**New in v0.6.0**: Path Commander can manage PATH variables on remote Windows computers across your network.

### Prerequisites

1. **Run as Administrator** - Required for remote registry access
2. **Remote Registry Service** - Must be running on target computer
3. **Network Access** - Target computer must be reachable
4. **Administrative Shares** - C$ shares must be enabled (default on most Windows systems)
5. **Credentials** - You need admin rights on the remote computer

### Connecting to a Remote Computer

**Method 1: Command Line**
```bash
pc --remote COMPUTERNAME
pc --remote 192.168.1.100
```

**Method 2: Interactive**
1. Launch Path Commander
2. Press **Ctrl+O** (or **File > Connect to Remote**)
3. Enter computer name or IP address
4. Press **Enter**

### Remote Mode Interface

When connected, the interface changes:

- **Header shows**: `REMOTE: COMPUTERNAME`
- **Left Panel**: LOCAL MACHINE paths (your computer)
- **Right Panel**: REMOTE MACHINE paths (target computer)
- **Function Keys**: F5 now **copies** instead of moves

### Remote Operations

**What works**:
- ✅ View remote MACHINE paths
- ✅ Add, edit, delete remote paths
- ✅ Normalize remote paths
- ✅ Remove duplicates and dead paths on remote
- ✅ Copy paths between local and remote computers (F5)
- ✅ Validate path existence on remote (via UNC paths)
- ✅ Create missing directories on remote (Shift+F10)
- ✅ Undo/Redo
- ✅ Cross-computer duplicate detection

**Limitations**:
- ❌ Remote USER paths (security - only MACHINE paths accessible)
- ⚠️ WM_SETTINGCHANGE messages don't affect remote processes (restart required)

### Copying Paths Between Computers

In remote mode, **F5** copies instead of moves:

1. Switch to the panel with paths to copy (local or remote)
2. Mark the paths
3. Press **F5**
4. Paths are copied to the other computer
5. Press **Ctrl+S** to apply changes on both computers

Example: Copy development tools from your main machine to a test VM.

### UNC Path Validation

Path Commander uses UNC paths (`\\COMPUTERNAME\C$\path`) to validate remote paths:

- Dead/alive status is accurate
- You can create missing directories on remote computers
- Requires administrative shares (C$, D$, etc.) to be enabled

If you get "Access Denied" errors:
1. Verify administrative shares are enabled on remote computer
2. Ensure you have admin credentials
3. Check firewall settings

### Disconnecting

1. Press **Ctrl+O** (or **File > Disconnect**)
2. Confirm if you have pending changes
3. Interface returns to local mode (USER and MACHINE panels)

---

## Theming and Customization

Path Commander supports Midnight Commander (MC) theme files for complete visual customization.

### Built-in Themes

Path Commander includes several themes:
- **Classic** - Default theme, gray dialogs, readable colors
- **Dracula** - Dark theme with purple/pink accents
- **Monokai** - Dark theme with orange/yellow accents

### Using Themes

**Command Line**:
```bash
pc --theme ~/.pc/themes/dracula.ini
pc --theme C:\Users\Me\.pc\themes\monokai.ini
```

**Interactive Theme Selector**:
1. Press **t** while running
2. Use **↑/↓** or **j/k** to browse themes
3. Theme previews apply **live** as you navigate
4. Press **Enter** to keep the current theme
5. Press **Esc** to cancel and restore original theme

### Installing MC Themes

Any Midnight Commander `.ini` skin file works! Download themes from:

**Official Dracula Theme**:
```bash
mkdir -p ~/.pc/themes
curl -sL https://raw.githubusercontent.com/dracula/midnight-commander/master/skins/dracula256.ini -o ~/.pc/themes/dracula.ini
```

**MC Skins Collection**:
Browse https://github.com/nkulikov/mc-skins

**Other Sources**:
Search GitHub for "midnight commander skins" or "mc themes"

### Theme Directory

Themes are stored in:
- **Linux/macOS/Git Bash**: `~/.pc/themes/`
- **Windows native**: `%USERPROFILE%\.pc\themes\`

The directory is created automatically on first run.

### Creating Custom Themes

See the [MC_COMPATIBILITY.md](MC_COMPATIBILITY.md) and [THEMING_DESIGN.md](THEMING_DESIGN.md) documents for details on:
- MC `.ini` file format
- Color mappings between MC and Path Commander
- Supported color notations (RGB, indices, names)

---

## Safety and Backups

### Automatic Backups

Before applying changes (Ctrl+S), Path Commander **automatically** creates a timestamped backup:

- **Location**: `%LOCALAPPDATA%\PathCommander\backups\`
- **Format**: `path_backup_YYYYMMDD_HHMMSS.json`
- **Contains**: Complete USER and MACHINE PATH values

You can't disable automatic backups - they're your safety net!

### Manual Backups

Create a backup at any time:

1. Press **Ctrl+B** (or **File > Create Backup**)
2. A timestamped backup is saved
3. Status bar confirms: `Backup created`

Use manual backups before major changes, experiments, or when handing off a computer.

### Restoring from Backup

If something goes wrong:

1. Press **Ctrl+R** (or **File > Restore from Backup**)
2. Select a backup from the list (sorted newest first)
3. Press **Enter** to load it
4. Review the restored paths
5. Press **Ctrl+S** to apply the restoration

**Note**: Restoring loads the backup into Path Commander but doesn't apply it until you press Ctrl+S.

### What Backups Include

Each backup stores:
- Full USER PATH string
- Full MACHINE PATH string
- Individual path entries (for both scopes)
- Timestamp of backup creation

Backups are JSON files - you can inspect or edit them manually if needed.

---

## Troubleshooting

### "Failed to open registry key" Error

**Cause**: You need administrator privileges to modify MACHINE paths.

**Solution**:
1. Close Path Commander
2. Right-click PowerShell or Command Prompt
3. Select "Run as Administrator"
4. Run `pc` again

### Changes Not Reflected in Open Applications

**Cause**: Most applications read PATH on startup.

**Solution**:
- Restart the application to pick up changes
- Path Commander shows a dialog listing processes that need restart
- New processes started after saving automatically see the new PATH
- Some system components may require a reboot

### Paths Still Showing as Dead After Adding

**Cause**: Directory doesn't exist yet.

**Solution**:
1. Press **Shift+F10** (or **Command > Create Marked Directories**)
2. Path Commander creates the directory
3. Path status updates to green (valid)

### "Access Denied" When Connecting to Remote Computer

**Causes**:
- Not running as administrator
- Remote Registry service not running on target
- Don't have admin rights on remote computer
- Firewall blocking access
- Administrative shares disabled

**Solutions**:
1. Run Path Commander as administrator
2. On remote computer, ensure Remote Registry service is running:
   ```powershell
   Get-Service RemoteRegistry | Start-Service
   ```
3. Verify you have admin credentials on the remote computer
4. Check Windows Firewall settings on both computers
5. Verify administrative shares are enabled (C$, D$, etc.)

### Theme Not Loading

**Cause**: Theme file not found or invalid format.

**Solution**:
1. Check the theme file path is correct
2. Verify the file is a valid MC `.ini` format
3. Try launching with `--theme` flag to see error messages
4. Copy the theme file to `~/.pc/themes/` directory

### Undo/Redo Not Working

**Cause**: Undo history is cleared after applying changes (Ctrl+S).

**Behavior**: This is intentional - once changes are written to the registry, they can't be undone through undo/redo. Use backup/restore instead.

---

## Tips and Best Practices

### Before Making Major Changes

1. **Create a manual backup** (Ctrl+B)
2. **Take note** of critical paths (or screenshot)
3. **Test in stages** - make small changes and test before continuing

### Keeping PATH Clean

1. **Run regularly** to check for dead paths (they accumulate over time)
2. **Remove duplicates** - they waste space and provide no benefit
3. **Consider normalization** - but remember it makes paths less portable
4. **Move personal tools to USER** - keeps system PATH cleaner

### Order Matters

- Windows searches PATH from left to right (top to bottom in Path Commander)
- Put frequently-used paths near the top
- If two programs have the same name, the one in the earlier path wins
- Use **F6** to move important paths higher

### Working with Remote Computers

1. **Test locally first** - get familiar with operations before working remotely
2. **Backup before remote changes** - both local and remote
3. **Use descriptive paths** - easier to identify which computer they belong to
4. **Document changes** - keep notes on what you changed and why

### Performance Tips

- **Close unused applications** before applying changes (fewer processes to notify)
- **Use filters** when working with large PATH lists (F key)
- **Mark multiple items** for batch operations instead of one-by-one

### Using with Development Tools

- **Python virtual environments** - Add venv Scripts folder to USER PATH temporarily
- **Node.js global packages** - Usually in `%APPDATA%\npm`
- **Rust cargo binaries** - Usually in `%USERPROFILE%\.cargo\bin`
- **Go binaries** - Usually in `%USERPROFILE%\go\bin`

### Keyboard vs. Mouse

Both are fully supported! Use whatever feels natural:

- **Keyboard** - Faster for power users, better for repetitive tasks
- **Mouse** - More intuitive for beginners, good for random access
- **Mix both** - Mark items with mouse, operate with keyboard shortcuts

### Menu System

Access all features through menus if you forget shortcuts:

- **Alt+F** - File menu (backups, remote, exit)
- **Alt+C** - Command menu (add, delete, move, cleanup)
- **Alt+O** - Options menu (filter, themes)
- **Alt+H** - Help menu (help screen, about)

---

## Quick Reference

### Essential Shortcuts

| Action | Shortcut |
|--------|----------|
| **Help** | F1 or ? |
| **Add Path** | F4 |
| **Delete Marked** | F3 or Delete |
| **Move to Other Panel** | F5 |
| **Remove Duplicates** | F7 |
| **Remove Dead Paths** | F8 |
| **Normalize Paths** | F9 |
| **Apply Changes** | Ctrl+S |
| **Undo** | Ctrl+Z |
| **Redo** | Ctrl+Y |
| **Create Backup** | Ctrl+B |
| **Restore Backup** | Ctrl+R |
| **Remote Connect** | Ctrl+O |
| **Theme Selector** | t |
| **Exit** | F10 or Q |

### Color Legend

| Color | Meaning |
|-------|---------|
| 🟢 Green | Valid, unique, normalized path |
| 🔴 Red | Dead path (doesn't exist) |
| 🟡 Yellow | Duplicate path |
| 🔵 Cyan | Non-normalized path |

---

## Getting Help

- Press **F1** or **?** for in-app help
- Visit: https://github.com/jesse-slaton/cli-tools
- Report issues: https://github.com/jesse-slaton/cli-tools/issues
- Read docs: [README.md](README.md), [CHANGELOG.md](CHANGELOG.md)

---

**Path Commander** - Manage your Windows PATH with confidence!

*Copyright © 2025 Jesse Slaton - MIT License*
