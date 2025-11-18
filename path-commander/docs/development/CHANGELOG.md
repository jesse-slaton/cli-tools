# Changelog

All notable changes to Path Commander will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.2] - 2025-01-17

### Fixed
- **Keyboard input now properly supports key repeat for backspace and character input**
  - Holding down backspace now continuously deletes characters (previously only deleted once)
  - All text input fields now respond correctly to held-down keys

### Changed
- **Migrated to standard ratatui/crossterm event handling pattern**
  - Replaced custom time-based event deduplication with `KeyEventKind` filtering
  - Now filters events by `KeyEventKind::Press` (and `Repeat` for text input) instead of 200ms timing window
  - Aligns with official ratatui documentation and ecosystem best practices
  - Simplified event loop by removing manual state tracking

### Technical
- Added `KeyEventKind` to crossterm imports in `main.rs` and `app.rs`
- Updated event loop in `main.rs` to filter by `key.kind == KeyEventKind::Press`
- Updated `handle_input_mode()` in `app.rs` to accept both `Press` and `Repeat` events
- Removed time-based deduplication logic (`last_event_time`, `last_key_code` tracking)

## [0.6.0] - 2025-01-17

### Added
- **UNC path support for remote computer PATH management** (Issue #24)
  - Remote paths are now validated using UNC paths (`\\COMPUTERNAME\C$\...`)
  - Accurate dead/alive path detection on remote computers
  - Ability to create missing directories on remote systems
  - Works seamlessly with existing administrative shares (C$, D$, etc.)

### Changed
- Path validation in remote mode now uses UNC paths to check if paths exist
- Directory creation in remote mode now creates directories on the remote computer
- Enhanced error messages for UNC path access failures with helpful troubleshooting hints

### Technical
- Added `to_unc_path()` helper function to convert local paths to UNC format
- Added `path_exists_with_remote()` function for remote path validation
- Added `analyze_paths_with_remote()` function with optional remote computer parameter
- Updated `App::reanalyze()` to pass remote computer name when in remote mode
- Updated directory creation functions to support UNC paths
- Added comprehensive unit tests for UNC path functionality

### Requirements
- Requires C$ administrative shares to be enabled on remote computers (enabled by default on most Windows systems)
- Same credentials used for remote registry access are used for UNC path access

## [0.5.0] - 2025-01-17

### Changed
- **Exit confirmation dialog now requires F10 to confirm**
  - First F10 press always shows confirmation dialog
  - In the dialog, press F10 again to exit
  - Press Esc to cancel and stay in the program
  - Changed from "Yes / No" buttons to "F10 to exit / Esc to cancel"
  - Makes the exit action more deliberate and consistent with the exit trigger

## [0.4.0] - 2025-01-17

### Added
- **Live theme preview in theme selector**
  - Theme changes now apply instantly as you navigate with arrow keys (Up/Down) or vim keys (j/k)
  - No need to press Enter to see what a theme looks like
  - Press Esc to cancel and restore the original theme
  - Press Enter to accept the currently previewed theme
  - Much faster to find and select a theme you like

### Technical
- Added `original_theme` field to `App` struct to store theme before opening selector
- Added `apply_selected_theme()` helper method for live theme application
- Theme selector now clones current theme when opened and restores it on Esc

## [0.3.0] - 2025-01-17

### Added
- **Floating dialog system** matching Midnight Commander's visual style
  - Dialogs now render with main window visible underneath (proper z-ordering)
  - Drop shadow effects on right and bottom edges for visual depth
  - Bold borders for enhanced visual separation
- **Intelligent auto-sizing for dialogs** based on content
  - About dialog: auto-sized to fit ASCII logo (30×16)
  - Filter Menu: auto-sized to fit 5 options with descriptions (45×11)
  - Backup List: dynamically sizes based on number of backups (max 15 visible)
  - Theme Selection: dynamically sizes based on longest theme name
- **ASCII logo in About dialog** with "Path Commander" branding
- **MC-style gray dialog backgrounds** (black text on lightgray)
  - Matches Midnight Commander's dialog aesthetic
  - Classic theme updated to use gray dialogs
- **Theme color name support** for MC skin files
  - Added support for `lightgray`, `gray`, `brown` color names
  - MC skin files now parse correctly with named colors (not just rgb notation)
- **Help menu item** - Added "Help" (F1) to Help menu for discoverability

### Changed
- **Simplified Help dialog** - Removed redundant shortcuts already visible in menus/function keys
  - Kept only navigation, marking, undo/redo, privileges, remote mode, and color legend
  - Two-column layout retained with more focused content
- **Dialog sizes reduced** across the board for less screen obstruction
  - Help: 90%×90% → 55%×50%
  - Confirm: 60%×30% → 40%×20%
  - Input: 70%×20% → 50%×15%
  - File Browser: 85%×75% → 60%×60%
  - Process Restart Info: 80%×80% → 55%×50%
- **Theme Selection dialog improvements**
  - Removed preview section (not useful)
  - Auto-sizes based on theme name lengths
  - Can show up to 15 themes at once (increased from 12)

### Fixed
- Help dialog now uses standard dialog colors instead of separate `help_bg`
- All dialogs respect main window visibility (no black overlays)
- Theme chooser no longer unnecessarily wide

### Technical
- Added `content_sized_rect()` helper for auto-sizing dialogs
- Added `render_dialog_shadow()` helper for consistent shadow effects
- Added `create_floating_dialog_block()` for consistent dialog styling
- Refactored `render()` to always render main window first, then dialogs on top
- Updated color parser to support MC's named color conventions

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

[0.6.2]: https://github.com/jesse-slaton/cli-tools/releases/tag/v0.6.2
[0.6.0]: https://github.com/jesse-slaton/cli-tools/releases/tag/v0.6.0
[0.5.0]: https://github.com/jesse-slaton/cli-tools/releases/tag/v0.5.0
[0.4.0]: https://github.com/jesse-slaton/cli-tools/releases/tag/v0.4.0
[0.3.0]: https://github.com/jesse-slaton/cli-tools/releases/tag/v0.3.0
[0.2.0]: https://github.com/jesse-slaton/cli-tools/releases/tag/v0.2.0
[0.1.0]: https://github.com/jesse-slaton/cli-tools/releases/tag/v0.1.0
