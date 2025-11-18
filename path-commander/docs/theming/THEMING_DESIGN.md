# Path Commander Theming Design

## Overview
This document outlines the theming system for Path Commander, inspired by Midnight Commander's skin format.

## Configuration Directory
Theme files will be stored in:
- **Primary**: `~/.pc/themes/` (Windows: `%USERPROFILE%\.pc\themes\`)
- **Fallback**: `~/.pathcommand/themes/` (if ~/.pc is unavailable)

Built-in themes will be embedded in the application but can be exported to the config directory.

## INI File Format

### Section Structure
Path Commander theme INI files use these sections:

```ini
[skin]
description = Theme Name
version = 1.0

[core]
# Main panel colors
_default_ = foreground;background
selected = foreground;background
marked = foreground;background
header = foreground;background

[dialog]
# Dialog and overlay colors
_default_ = foreground;background
dfocus = focused_foreground;focused_background  # Focused element
dhotnormal = hotkey_foreground;background       # Hotkey (unfocused)
dhotfocus = hotkey_foreground;focused_background # Hotkey (focused)
dtitle = title_foreground;background            # Dialog title

[widget-common]
# Common widget colors
button_default = foreground;background
button_focus = foreground;background
button_disabled = foreground;background

[statusbar]
# Bottom status bar
_default_ = foreground;background

[help]
# Help screen colors
_default_ = foreground;background
helpbold = bold_foreground;background
helpitalic = italic_foreground;background
helplink = link_foreground;background

[pathcommander]
# Path Commander specific colors
border = foreground;background          # Panel borders
path_valid = foreground;background      # Valid paths (exist)
path_dead = foreground;background       # Dead paths (don't exist)
path_duplicate = foreground;background  # Duplicate paths
path_nonnormalized = foreground;background  # Non-normalized paths
warning = foreground;background         # Warning messages
info = foreground;background            # Info messages
success = foreground;background         # Success messages
scrollbar = foreground;background       # Scrollbar track
scrollbar_thumb = foreground;background # Scrollbar thumb
filter_indicator = foreground;background # Active filter indicator
admin_warning = foreground;background   # Admin privilege warning
```

### INI Field → Theme Struct Mapping

| INI Section | INI Field | Theme Struct Field |
|-------------|-----------|-------------------|
| `[core]` | `_default_` | `panel_normal_fg`, `panel_normal_bg` |
| `[core]` | `selected` | `panel_selected_fg`, `panel_selected_bg` |
| `[core]` | `marked` | `panel_marked_fg`, `panel_marked_bg` |
| `[core]` | `header` | `header_fg`, `header_bg` |
| `[dialog]` | `_default_` | `dialog_fg`, `dialog_bg` |
| `[dialog]` | `dtitle` | `dialog_title_fg`, `dialog_title_bg` |
| `[widget-common]` | `button_default` | `button_fg`, `button_bg` |
| `[widget-common]` | `button_focus` | `button_focused_fg`, `button_focused_bg` |
| `[statusbar]` | `_default_` | `status_fg`, `status_bg` |
| `[help]` | `_default_` | `help_fg`, `help_bg` |
| `[help]` | `helpbold` | `help_bold_fg`, `help_bold_bg` |
| `[help]` | `helplink` | `help_link_fg`, `help_link_bg` |
| `[pathcommander]` | `border` | `panel_border_fg`, `panel_border_bg` |
| `[pathcommander]` | `path_valid` | `path_valid_fg`, `path_valid_bg` |
| `[pathcommander]` | `path_dead` | `path_dead_fg`, `path_dead_bg` |
| `[pathcommander]` | `path_duplicate` | `path_duplicate_fg`, `path_duplicate_bg` |
| `[pathcommander]` | `path_nonnormalized` | `path_nonnormalized_fg`, `path_nonnormalized_bg` |
| `[pathcommander]` | `warning` | `warning_fg`, `warning_bg` |
| `[pathcommander]` | `info` | `info_fg`, `info_bg` |
| `[pathcommander]` | `success` | `success_fg`, `success_bg` |
| `[pathcommander]` | `scrollbar` | `scrollbar_fg`, `scrollbar_bg` |
| `[pathcommander]` | `scrollbar_thumb` | `scrollbar_thumb_fg`, `scrollbar_thumb_bg` |
| `[pathcommander]` | `filter_indicator` | `filter_indicator_fg`, `filter_indicator_bg` |
| `[pathcommander]` | `admin_warning` | `admin_warning_fg`, `admin_warning_bg` |

## Color Mock-ups

### 1. Dracula Theme

**Reference**: Based on https://draculatheme.com/contribute (official Dracula spec)

**Color Palette**:
```
Background:    #282a36  (rgb: 40, 42, 54)
Current Line:  #44475a  (rgb: 68, 71, 90)
Foreground:    #f8f8f2  (rgb: 248, 248, 242)
Comment:       #6272a4  (rgb: 98, 114, 164)
Cyan:          #8be9fd  (rgb: 139, 233, 253)
Green:         #50fa7b  (rgb: 80, 250, 123)
Orange:        #ffb86c  (rgb: 255, 184, 108)
Pink:          #ff79c6  (rgb: 255, 121, 198)
Purple:        #bd93f9  (rgb: 189, 147, 249)
Red:           #ff5555  (rgb: 255, 85, 85)
Yellow:        #f1fa8c  (rgb: 241, 250, 140)
```

**INI File** (`dracula.ini`):
```ini
[skin]
description = Dracula Theme for Path Commander
version = 1.0

[core]
_default_ = rgb555;rgb111          # Foreground on Background
selected = rgb555;rgb222           # Foreground on Current Line
marked = rgb554;rgb111             # Pink on Background
header = rgb355;rgb111             # Cyan on Background

[dialog]
_default_ = rgb555;rgb222          # Foreground on Current Line
dtitle = rgb355;rgb222             # Cyan on Current Line
dhotnormal = rgb535;rgb222         # Purple on Current Line
dhotfocus = rgb252;rgb222          # Green on Current Line

[widget-common]
button_default = rgb222;rgb111     # Current Line on Background
button_focus = rgb111;rgb252       # Background on Green
button_disabled = rgb333;rgb111    # Comment on Background

[statusbar]
_default_ = rgb555;rgb222          # Foreground on Current Line

[help]
_default_ = rgb555;rgb111          # Foreground on Background
helpbold = rgb355;rgb111           # Cyan on Background
helplink = rgb535;rgb111           # Purple on Background

[pathcommander]
border = rgb333;rgb111             # Comment on Background
path_valid = rgb252;rgb111         # Green on Background
path_dead = rgb555;rgb111          # Red on Background
path_duplicate = rgb554;rgb111     # Yellow on Background
path_nonnormalized = rgb355;rgb111 # Cyan on Background
warning = rgb554;rgb111            # Yellow on Background
info = rgb355;rgb111               # Cyan on Background
success = rgb252;rgb111            # Green on Background
scrollbar = rgb333;rgb111          # Comment on Background
scrollbar_thumb = rgb444;rgb111    # Lighter gray on Background
filter_indicator = rgb355;rgb111   # Cyan on Background
admin_warning = rgb553;rgb111      # Orange on Background
```

**Visual Mock-up** (text representation):
```
┌─────────────────────────────────────────────────────────────────────────┐
│ Path Commander v1.0.0 - Windows PATH Editor              [ADMIN] ⚠     │  <- Cyan on Background, Orange warning
├──────────────────────┬──────────────────────────────────────────────────┤
│ MACHINE (System)     │ USER (Current User)                              │  <- Comment border
│ ┌──────────────────┐ │ ┌──────────────────────────────────────────────┐ │
│ │☐ C:\Windows\Sys  │ │ │☑ C:\Users\Me\App                            │ │  <- Green (valid)
│ │☐ C:\Program File │ │ │☐ C:\Missing\Path                            │ │  <- Red (dead)
│ │☑ C:\Duplicate    │ │ │☐ C:\Duplicate                               │ │  <- Yellow (duplicate)
│ │☐ C:\PROGRA~1     │ │ │☐ %USERPROFILE%\tools                        │ │  <- Cyan (non-normalized)
│ │  ...             │ │ │  ...                                        │ │
│ └──────────────────┘ │ └──────────────────────────────────────────────┘ │
│ 4 paths (1 marked)   │ 4 paths (0 marked)                              │
├──────────────────────┴──────────────────────────────────────────────────┤
│ F1 Help  F2 Backup  F3 Filter  F5 Edit  Ins Add  Del Delete  F10 Quit  │  <- Foreground on Current Line
└─────────────────────────────────────────────────────────────────────────┘
```

### 2. Classic MC (Default Blue) Theme

**Color Palette**:
```
Based on traditional Midnight Commander blue theme
Uses standard 16-color terminal palette
```

**INI File** (`classic.ini`):
```ini
[skin]
description = Classic Midnight Commander Blue Theme
version = 1.0

[core]
_default_ = white;black
selected = black;cyan
marked = yellow;black
header = white;blue

[dialog]
_default_ = white;blue
dtitle = yellow;blue
dhotnormal = brightcyan;blue
dhotfocus = yellow;cyan

[widget-common]
button_default = black;cyan
button_focus = white;green
button_disabled = gray;black

[statusbar]
_default_ = black;white

[help]
_default_ = white;blue
helpbold = yellow;blue
helplink = brightcyan;blue

[pathcommander]
border = white;black
path_valid = green;black
path_dead = red;black
path_duplicate = yellow;black
path_nonnormalized = cyan;black
warning = yellow;blue
info = brightcyan;blue
success = green;blue
scrollbar = gray;black
scrollbar_thumb = white;black
filter_indicator = cyan;black
admin_warning = yellow;black
```

**Visual Mock-up**:
```
┌─────────────────────────────────────────────────────────────────────────┐
│ Path Commander v1.0.0 - Windows PATH Editor              [ADMIN] ⚠     │  <- White on Blue, Yellow warning
├──────────────────────┬──────────────────────────────────────────────────┤
│ MACHINE (System)     │ USER (Current User)                              │  <- White border
│ ┌──────────────────┐ │ ┌──────────────────────────────────────────────┐ │
│ │☐ C:\Windows\Sys  │ │ │☑ C:\Users\Me\App                            │ │  <- Green (valid)
│ │☐ C:\Program File │ │ │☐ C:\Missing\Path                            │ │  <- Red (dead)
│ │☑ C:\Duplicate    │ │ │☐ C:\Duplicate                               │ │  <- Yellow (duplicate)
│ │☐ C:\PROGRA~1     │ │ │☐ %USERPROFILE%\tools                        │ │  <- Cyan (non-normalized)
│ │  ...             │ │ │  ...                                        │ │
│ └──────────────────┘ │ └──────────────────────────────────────────────┘ │
│ 4 paths (1 marked)   │ 4 paths (0 marked)                              │
├──────────────────────┴──────────────────────────────────────────────────┤
│ F1 Help  F2 Backup  F3 Filter  F5 Edit  Ins Add  Del Delete  F10 Quit  │  <- Black on White
└─────────────────────────────────────────────────────────────────────────┘
```

### 3. Additional Theme: Monokai

**Color Palette**:
```
Background:    #272822  (rgb: 39, 40, 34)
Foreground:    #f8f8f2  (rgb: 248, 248, 242)
Selection:     #49483e  (rgb: 73, 72, 62)
Comment:       #75715e  (rgb: 117, 113, 94)
Green:         #a6e22e  (rgb: 166, 226, 46)
Orange:        #fd971f  (rgb: 253, 151, 31)
Pink:          #f92672  (rgb: 249, 38, 114)
Purple:        #ae81ff  (rgb: 174, 129, 255)
Yellow:        #e6db74  (rgb: 230, 219, 116)
Cyan:          #66d9ef  (rgb: 102, 217, 239)
```

**INI File** (`monokai.ini`):
```ini
[skin]
description = Monokai Theme for Path Commander
version = 1.0

[core]
_default_ = rgb555;rgb111          # Foreground on Background
selected = rgb555;rgb222           # Foreground on Selection
marked = rgb554;rgb111             # Pink on Background
header = rgb245;rgb111             # Cyan on Background

[dialog]
_default_ = rgb555;rgb222          # Foreground on Selection
dtitle = rgb245;rgb222             # Cyan on Selection
dhotnormal = rgb454;rgb222         # Purple on Selection
dhotfocus = rgb253;rgb222          # Orange on Selection

[widget-common]
button_default = rgb111;rgb333     # Background on Comment
button_focus = rgb111;rgb335       # Background on Green
button_disabled = rgb333;rgb111    # Comment on Background

[statusbar]
_default_ = rgb555;rgb222          # Foreground on Selection

[help]
_default_ = rgb555;rgb111          # Foreground on Background
helpbold = rgb245;rgb111           # Cyan on Background
helplink = rgb454;rgb111           # Purple on Background

[pathcommander]
border = rgb333;rgb111             # Comment on Background
path_valid = rgb335;rgb111         # Green on Background
path_dead = rgb531;rgb111          # Pink on Background
path_duplicate = rgb554;rgb111     # Yellow on Background
path_nonnormalized = rgb245;rgb111 # Cyan on Background
warning = rgb554;rgb111            # Yellow on Background
info = rgb245;rgb111               # Cyan on Background
success = rgb335;rgb111            # Green on Background
scrollbar = rgb333;rgb111          # Comment on Background
scrollbar_thumb = rgb444;rgb111    # Lighter gray on Background
filter_indicator = rgb245;rgb111   # Cyan on Background
admin_warning = rgb553;rgb111      # Orange on Background
```

## Theme Selection UI

### Command-line Usage
```bash
# Use built-in theme
pc --theme dracula
pc --theme classic
pc --theme monokai

# Use custom theme from config directory
pc --theme mytheme        # Loads ~/.pc/themes/mytheme.ini

# Use theme from specific path
pc --theme /path/to/custom.ini
```

### In-App Theme Selector (F4 or 't' key)

```
┌─────────────────── Select Theme ────────────────────────┐
│                                                          │
│   > Dracula          (Built-in)                         │  <- Selected
│     Classic MC       (Built-in)                         │
│     Monokai          (Built-in)                         │
│   ─────────────────────────────────────────────         │
│     my-theme         (~/.pc/themes/my-theme.ini)        │
│     corporate        (~/.pc/themes/corporate.ini)       │
│                                                          │
│  ↑↓ Navigate  Enter Select  Esc Cancel  r Reload        │
│                                                          │
│  Preview:                                                │
│  ┌────────────────────────────────────────────────────┐ │
│  │☐ C:\Windows\System32               <- path_valid  │ │
│  │☐ C:\Missing\Path                   <- path_dead   │ │
│  │☐ C:\Duplicate                      <- path_dup    │ │
│  │☐ %USERPROFILE%\tools               <- path_nonorm │ │
│  └────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘
```

## Implementation Details

### Theme Struct Expansion

Add these new fields to `Theme`:

```rust
pub struct Theme {
    // Existing fields...

    // Dialog-specific
    pub dialog_title_fg: Color,
    pub dialog_title_bg: Color,

    // Buttons
    pub button_bg: Color,
    pub button_disabled_fg: Color,
    pub button_disabled_bg: Color,

    // Help screen
    pub help_fg: Color,
    pub help_bg: Color,
    pub help_bold_fg: Color,
    pub help_bold_bg: Color,
    pub help_link_fg: Color,
    pub help_link_bg: Color,

    // Path Commander specific
    pub panel_border_bg: Color,
    pub path_valid_bg: Color,
    pub path_dead_bg: Color,
    pub path_duplicate_bg: Color,
    pub path_nonnormalized_bg: Color,
    pub warning_fg: Color,
    pub warning_bg: Color,
    pub info_fg: Color,
    pub info_bg: Color,
    pub success_fg: Color,
    pub success_bg: Color,
    pub scrollbar_fg: Color,
    pub scrollbar_bg: Color,
    pub scrollbar_thumb_fg: Color,
    pub scrollbar_thumb_bg: Color,
    pub filter_indicator_fg: Color,
    pub filter_indicator_bg: Color,
    pub admin_warning_fg: Color,
    pub admin_warning_bg: Color,
}
```

### Config Directory Structure

```
~/.pc/
├── config.toml           # General config (default theme, etc.)
├── themes/
│   ├── dracula.ini       # User can export/modify built-ins
│   ├── classic.ini
│   ├── monokai.ini
│   └── my-custom.ini
└── backups/              # Existing backup directory
    └── path_backup_*.json
```

## Testing Colorways

To help choose colors, here's a side-by-side comparison:

| Element | Dracula | Classic MC | Monokai |
|---------|---------|------------|---------|
| Background | Dark purple-gray (#282a36) | Black | Dark gray (#272822) |
| Foreground | Off-white (#f8f8f2) | White | Off-white (#f8f8f2) |
| Selection | Purple-gray (#44475a) | Cyan bg | Gray (#49483e) |
| Valid path | Green (#50fa7b) | Green | Green (#a6e22e) |
| Dead path | Red (#ff5555) | Red | Pink (#f92672) |
| Duplicate | Yellow (#f1fa8c) | Yellow | Yellow (#e6db74) |
| Non-normalized | Cyan (#8be9fd) | Cyan | Cyan (#66d9ef) |
| Warning | Yellow (#f1fa8c) | Yellow | Yellow (#e6db74) |
| Success | Green (#50fa7b) | Green | Green (#a6e22e) |
| Border | Comment gray (#6272a4) | White | Comment gray (#75715e) |

## Questions for Feedback

1. **Config directory**: Prefer `~/.pc/` or `~/.pathcommand/`?
2. **Color choices**: Are the Dracula colors correct? Any adjustments needed?
3. **Additional themes**: Want any other popular themes (Gruvbox, Nord, Solarized)?
4. **Theme hotkey**: F4 for theme selector, or different key?
5. **Preview depth**: Should theme preview show more UI elements?
