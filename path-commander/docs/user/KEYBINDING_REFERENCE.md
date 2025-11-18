# Keybinding Reference for Path Commander

This document provides guidance on which keybindings are safe to use in Path Commander and which should be avoided due to conflicts with terminal emulators, Windows, and common conventions.

## Terminal Emulator Conflicts (FORBIDDEN)

These keys are commonly intercepted by terminal emulators and should **NOT** be used:

### Windows Terminal
- **F11** - Fullscreen toggle (intercepted before app sees it)
- **Ctrl+Shift+F** - Find
- **Ctrl+Shift+P** - Command palette
- **Ctrl+T** - New tab
- **Ctrl+W** - Close tab
- **Ctrl+Tab** - Switch tabs
- **Ctrl+Shift+Tab** - Switch tabs (reverse)
- **Ctrl+Plus/Minus** - Zoom in/out
- **Ctrl+0** - Reset zoom
- **Ctrl+Shift+D** - Duplicate pane
- **Alt+Shift+Plus/Minus** - Resize pane
- **Ctrl+Shift+C** - Copy (terminal-level)
- **Ctrl+Shift+V** - Paste (terminal-level)
- **Alt+Enter** - Fullscreen (some configurations)

### ConEmu / Cmder
- **F11** - Fullscreen
- **F12** - Settings (optional, configurable)
- **Ctrl+Tab** - Switch consoles
- **Win+Alt+P** - Settings
- **Ctrl+Shift+E** - Copy/paste menu

### General Terminal Emulators
- **F11** - Almost universally fullscreen
- **Ctrl+C** - SIGINT (though can be overridden in raw mode)
- **Ctrl+Z** - SIGTSTP (suspend)
- **Ctrl+D** - EOF signal (end of input)
- **Ctrl+\** - SIGQUIT
- **Ctrl+Shift+C/V** - Copy/paste in many terminals

### Browser-Based Terminals (if applicable)
- **F5** - Refresh page
- **F12** - Developer tools
- **Ctrl+F** - Browser find
- **Ctrl+L** - Focus address bar
- **Ctrl+T** - New tab
- **Ctrl+W** - Close tab

## Windows System Conflicts (FORBIDDEN)

- **Win+[Any]** - Windows system shortcuts (minimize, desktop, etc.)
- **Alt+F4** - Close window
- **Alt+Tab** - Switch windows
- **Alt+Space** - Window menu
- **Ctrl+Alt+Delete** - Security options
- **Print Screen** - Screenshot

## SAFE Keybindings for TUI Applications

### Function Keys (F1-F12)
✅ **SAFE (with notes):**
- **F1-F10** - Generally safe (F1 traditionally Help, F10 sometimes menu bar)
- **F12** - Mostly safe (may conflict with dev tools in some terminals)

❌ **AVOID:**
- **F11** - Fullscreen in most terminals

### Ctrl + Letter Combinations
✅ **SAFE:**
- **Ctrl+A** - Select/Mark all
- **Ctrl+B** - Backward/Back/Backup
- **Ctrl+E** - Edit/End
- **Ctrl+G** - Go to
- **Ctrl+H** - Help (sometimes Backspace)
- **Ctrl+J** - Enter (alternate)
- **Ctrl+K** - Kill/Delete line
- **Ctrl+L** - Clear/Refresh screen
- **Ctrl+M** - Enter (alternate)
- **Ctrl+N** - Next/New
- **Ctrl+O** - Open
- **Ctrl+P** - Previous/Print
- **Ctrl+Q** - Quit (though may conflict with XON/XOFF)
- **Ctrl+R** - Refresh/Restore/Reverse search
- **Ctrl+S** - Save (may pause terminal output in some old terminals)
- **Ctrl+U** - Undo/Unmark/Clear line
- **Ctrl+V** - Paste (application-level, not terminal-level)
- **Ctrl+X** - Cut
- **Ctrl+Y** - Redo/Yank

⚠️ **CAUTION:**
- **Ctrl+C** - Usually SIGINT, but can be caught in raw mode
- **Ctrl+D** - EOF signal, be careful
- **Ctrl+Z** - SIGTSTP, be careful
- **Ctrl+Q/S** - May trigger XON/XOFF flow control in some terminals

❌ **AVOID:**
- **Ctrl+W** - Close tab in many terminals
- **Ctrl+T** - New tab in many terminals
- **Ctrl+Tab** - Switch tabs

### Alt + Letter Combinations
✅ **VERY SAFE (Recommended for custom actions):**
- **Alt+A through Alt+Z** - Generally safe, commonly used in TUI apps
- **Alt+0 through Alt+9** - Generally safe

**Examples from other TUI apps:**
- Midnight Commander uses Alt+[letter] extensively
- Vim uses Alt combinations
- Tmux uses Alt in some configurations

✅ **Alt is the safest modifier** for custom application keybindings!

### Shift + Function Keys
✅ **SAFE:**
- **Shift+F1 through Shift+F12** - Generally safe

### Ctrl + Function Keys
✅ **SAFE:**
- **Ctrl+F1 through Ctrl+F12** - Generally safe (avoid Ctrl+F11 just to be safe)

### Navigation Keys
✅ **SAFE:**
- **Arrow keys** (Up, Down, Left, Right)
- **Home, End**
- **Page Up, Page Down**
- **Tab, Shift+Tab**
- **Enter, Backspace, Delete**
- **Insert**

### Special Keys
✅ **SAFE:**
- **Escape** - Always safe
- **Space** - Safe for application use

## Recommended Keybinding Strategy

### Priority Tiers

**Tier 1: Most Discoverable (F-Keys)**
- Use F1-F10 for primary features
- Users expect F1 = Help
- F2-F10 are fair game for main features

**Tier 2: Safe and Memorable (Alt + Letter)**
- Use Alt+[Letter] for secondary features
- Choose mnemonic letters (Alt+D = Dead, Alt+S = Save, etc.)
- Very safe from terminal conflicts

**Tier 3: Power User (Ctrl + Letter)**
- Use Ctrl+[Letter] for common operations
- Avoid Ctrl+C/D/Z/Q/S unless you understand implications
- Good for Save, Undo, Copy/Paste metaphors

**Tier 4: Advanced (Modified F-Keys)**
- Shift+F[n], Ctrl+F[n] for feature variations
- Example: F11 (filter), Shift+F11 (filter variation)
- Only use if F-key alone is already mapped

### Navigation Standards
- **↑/↓ or j/k** - Vertical movement (vim-style)
- **←/→ or h/l** - Horizontal movement or panel switch
- **PgUp/PgDn** - Page scrolling
- **Home/End** - Jump to start/end
- **Tab/Shift+Tab** - Panel switching or field navigation

### Selection Standards
- **Space or Insert** - Toggle mark/selection
- **Ctrl+A** - Select all
- **Ctrl+Shift+U** - Unselect all (if Ctrl+U is taken)

### Action Standards
- **Enter** - Confirm/Edit
- **Escape** - Cancel/Back
- **Delete or F3** - Delete (F3 from Midnight Commander)
- **F4** - Edit/Add (from Midnight Commander)
- **F5** - Copy/Move (from Midnight Commander)
- **F6** - Rename/Move (from Midnight Commander)
- **F7** - Create (from Midnight Commander)
- **F8** - Delete (from Midnight Commander)

## Current Path Commander Keybindings

### Navigation
- ↑/↓, j/k - Move selection
- PgUp/PgDn - Jump by viewport
- Home/End - Jump to start/end
- Tab, ←/→ - Switch panels

### Selection & Marking
- Space, Insert, F2 - Toggle mark
- **Ctrl+A** - Mark all visible (current scope)
- **Ctrl+Shift+A** - Mark all (both scopes)
- **Ctrl+D** - Mark all duplicates
- **Ctrl+Shift+D** - Mark all dead paths
- **Ctrl+N** - Mark all non-normalized
- **Ctrl+Shift+U** - Unmark all

### Filtering (⚠️ CONFLICT - See Issue #13)
- **F11** ❌ - Dead paths (conflicts with fullscreen)
- **F12** ⚠️ - Duplicates (may conflict)
- **Shift+F11** - Non-normalized
- **Ctrl+F11** - Valid paths

### Actions
- F1, ? - Help
- F3, Delete - Delete marked
- F4 - Add path
- F5 - Move marked to other panel
- F6 - Move item up
- F7 - Delete all duplicates
- F8 - Delete all dead
- F9 - Normalize marked
- F10 - Create marked directories
- Enter - Edit path

### File Operations
- Ctrl+S - Save/Apply changes
- Ctrl+B - Create backup
- Ctrl+R - Restore from backup

### Exit
- Q - Quit

## Recommendations for Issue #13 Resolution

Replace problematic F11/F12 filtering with Alt-based keys:

### Proposed New Filtering Keybindings
- **Alt+D** - Toggle Dead paths filter
- **Alt+U** - Toggle dUplicates filter (or Alt+P for duPlicates)
- **Alt+N** - Toggle Non-normalized filter
- **Alt+V** - Toggle Valid paths filter
- **Alt+C** or **Esc** - Clear filter (when filter active)

**Rationale:**
- No conflicts with any terminal emulators
- Mnemonic (D=Dead, U=dUplicates, N=Non-normalized, V=Valid)
- Follows pattern used by Midnight Commander
- Single keypress with modifier (convenient)
- Leaves F11/F12 unused but safe

## Testing Checklist

When adding new keybindings, test on:
- [ ] Windows Terminal
- [ ] PowerShell native terminal
- [ ] cmd.exe
- [ ] Git Bash / MSYS2
- [ ] ConEmu / Cmder
- [ ] WSL Ubuntu (Windows Terminal)
- [ ] WSL Ubuntu (native terminal)

## References

- [Windows Terminal Keybindings](https://learn.microsoft.com/en-us/windows/terminal/customize-settings/actions)
- [Midnight Commander Keybindings](https://midnight-commander.org/wiki/doc/common/keybind)
- [Vim Keybindings](https://vim.rtorr.com/)
- [Crossterm Event Documentation](https://docs.rs/crossterm/latest/crossterm/event/index.html)

## Future Considerations

### Customizable Keybindings
Consider adding a config file for user-customizable keybindings:
```toml
[keybindings]
filter_dead = "Alt+D"
filter_duplicates = "Alt+U"
mark_all = "Ctrl+A"
# ...
```

### Keybinding Profiles
- "Default" - Safe, conflict-free bindings
- "Legacy" - Match old versions
- "Vim-style" - For vim users
- "Midnight Commander" - Match MC exactly

## Summary

**Golden Rules:**
1. ✅ **Alt+Letter is safest** for custom features
2. ✅ **F1-F10 are mostly safe** (avoid F11)
3. ✅ **Ctrl+Letter is usually safe** (avoid Ctrl+C/D/Z/W/T/Tab)
4. ❌ **Never use F11** (fullscreen in terminals)
5. ❌ **Avoid Ctrl+Shift+[key]** (often terminal shortcuts)
6. 📝 **Document all keybindings** in help screen and README
7. 🧪 **Test on multiple terminals** before release
