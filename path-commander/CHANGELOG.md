# Changelog

All notable changes to Path Commander will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-01-16

### Added
- **Drop-down menu system** matching Midnight Commander's UX
  - Four menu categories: File, Command, Options, Help
  - Full keyboard support (Alt+letter for menu activation, arrows for navigation)
  - Full mouse support (click to open menus and select items)
  - Context-aware menu items (enabled/disabled based on application state)
  - Proper MC theme integration with menu-specific colors from .ini files
  - Underlined accelerator keys in menu bar
  - Keyboard shortcuts displayed next to menu items
- **About dialog** accessible via Help > About
  - Program name and version
  - Description and copyright information
  - MIT License display (styled as link)
  - Project GitHub URL (styled as link)
- **Menu-specific theme colors** for better MC compatibility
  - `menuinactive`: Menu bar when not selected
  - `_default_`: Dropdown menu default colors
  - `menusel`: Selected menu item (e.g., bright white on pink/purple in Dracula)
  - `menuhot`: Accelerator key colors
  - `menuhotsel`: Accelerator key when item selected

### Changed
- **Header simplified** to save vertical space
  - Removed 2-line program title ("Path Commander - Windows PATH Environment Manager")
  - Header now shows only 1 line with statistics and status
  - Remote connection indicator moved to beginning of statistics line
  - Removed border around header for cleaner look
  - Gained 2 extra lines of vertical space for path display
- **Exit behavior standardized** to match Midnight Commander
  - F10 is now the exclusive quit shortcut (works globally in all modes)
  - Removed Esc as exit trigger (now only closes dialogs/menus)
  - Removed 'q' as quit shortcut (freed up for future use)
  - Function key hints updated to show "10Quit" consistent with other numbered keys
- **Viewport height calculation** updated from `-10` to `-9` to account for new layout
  - Menu bar: 1 line
  - Header: 1 line (was 3)
  - Main content: variable
  - Status bar: 3 lines
  - Key hints: 2 lines
  - Panel borders: 2 lines

### Fixed
- Menu colors now properly use MC theme definitions for better visibility
- Prevented accidental exits when using Esc to close dialogs
- Test suite updated to match new viewport height calculation

### Technical
- Added `Mode::About` to application state
- Added `Mode::Menu { active_menu, selected_item }` to track menu state
- Made `has_marked_items()` and `has_marked_dead_paths()` public for menu state updates
- New `menu.rs` module with menu definitions and actions
- Menu dropdown calculates width based on longest item
- Proper bounds checking for menu navigation and clicks

## [0.1.0] - 2025-01-15

### Initial Release
- TUI for managing Windows PATH environment variables
- Dual-panel interface for MACHINE and USER scopes
- Path validation and analysis (dead paths, duplicates, non-normalized)
- Color-coded path status indicators
- Backup and restore functionality
- Undo/redo support
- Theme system with Midnight Commander .ini skin support
- Remote computer PATH management
- File browser for directory selection
- Comprehensive keyboard shortcuts
- Full mouse support (clicking, scrolling, shift/ctrl modifiers)
- Administrator privilege detection and elevation support
- Process detection and restart recommendations
- Filter system for viewing specific path types
- Function key display matching MC's buttonbar style

[0.2.0]: https://github.com/jesse-slaton/cli-tools/releases/tag/v0.2.0
[0.1.0]: https://github.com/jesse-slaton/cli-tools/releases/tag/v0.1.0
