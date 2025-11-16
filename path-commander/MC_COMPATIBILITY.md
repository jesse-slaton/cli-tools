# Midnight Commander Theme Compatibility

Path Commander can load **real Midnight Commander .ini theme files** with intelligent fallbacks for PC-specific features.

## ✅ What Works Out-of-the-Box

These MC sections are **fully supported** and will work perfectly:

| MC Section | PC Usage | Fields Used |
|------------|----------|-------------|
| `[core]` | Main panel UI | `_default_`, `selected`, `marked`, `header` |
| `[dialog]` | All dialogs/overlays | `_default_`, `dtitle`, `dfocus`, `dhotnormal` |
| `[statusbar]` | Bottom status bar | `_default_` |
| `[help]` | Help screen (F1) | `_default_`, `helpbold`, `helplink` |
| `[menu]` | Menus (filter, theme) | `_default_`, `menusel` (parsed for future use) |
| `[buttonbar]` | Key hints bar | `hotkey`, `button` (parsed for future use) |
| `[filehighlight]` | Path status colors | `directory`, `symlink` (fallback colors) |
| `[error]` | Error/dead path colors | `_default_` (fallback for dead paths) |

**NEW**: Path Commander now extracts purple/lavender colors from `[dialog] dfocus` and `[buttonbar] hotkey` to automatically style borders, scrollbars, and indicators with the signature Dracula purple theme!

## 🎨 Intelligent Fallbacks

For Path Commander-specific features, we use **smart mappings** from MC themes:

### Path Status Colors

| PC Feature | Tries 1st | Tries 2nd (MC Fallback) | Final Default |
|------------|-----------|-------------------------|---------------|
| **Valid paths** (green) | `[pathcommander] path_valid` | `[filehighlight] directory` | Green |
| **Dead paths** (red) | `[pathcommander] path_dead` | `[error] _default_` | Red |
| **Duplicates** (yellow) | `[pathcommander] path_duplicate` | `[filehighlight] symlink` | Yellow |
| **Non-normalized** (cyan) | `[pathcommander] path_nonnormalized` | - | Cyan |

### UI Element Colors

| PC Element | Tries 1st | Tries 2nd (MC Fallback) | Final Default |
|------------|-----------|-------------------------|---------------|
| **Panel border** | `[pathcommander] border` | `[dialog] dfocus` background | Panel normal fg |
| **Scrollbar** | `[pathcommander] scrollbar` | `[dialog] dfocus` background | Dialog focus bg |
| **Scrollbar thumb** | `[pathcommander] scrollbar_thumb` | `[buttonbar] hotkey` background | Pink/selection |
| **Filter indicator** | `[pathcommander] filter_indicator` | `[dialog] dfocus` background | Purple |
| **Admin warning** | `[pathcommander] admin_warning` | `[pathcommander] warning` | Yellow |

## 📋 Real Example: Dracula MC Theme

When loading the official Dracula MC theme (`dracula256.ini`), Path Commander will:

1. **Use MC colors** for:
   - Panel background/foreground: `rgb555;default`
   - Selection: `color0;rgb524` (black on pink)
   - Marked items: `color0;rgb253` (black on yellow)
   - Header: `rgb555;color0`

2. **Map MC file colors** to path status:
   - Valid paths: `rgb335` (MC directory color = light purple)
   - Dead paths: `rgb511` (MC error color = bright red)
   - Duplicates: `rgb542` (MC symlink color = orange/peach)

3. **Use MC purple theme colors** for PC-specific UI:
   - Panel border: `rgb435` (MC dialog focus = purple/lavender)
   - Scrollbar: `rgb435` (MC dialog focus = purple/lavender)
   - Scrollbar thumb: `rgb524` (MC button hotkey = pink)
   - Filter indicator: `rgb435` (MC dialog focus = purple/lavender)

## 🔧 How to Use MC Themes

### Method 1: Download and Use
```bash
# Download any MC theme
curl https://example.com/theme.ini -o ~/.pc/themes/mytheme.ini

# Load it in Path Commander
pc --theme mytheme
# or press 't' in the app and select it
```

### Method 2: Convert MC Theme to PC-Enhanced

Take an existing MC theme and add PC-specific sections:

```ini
# ... existing MC sections ...

[pathcommander]
# Path status colors (optional - will use MC fallbacks if omitted)
path_valid = rgb335;default
path_dead = rgb511;default
path_duplicate = rgb554;default
path_nonnormalized = rgb355;default

# UI elements (optional)
scrollbar = rgb333;default
scrollbar_thumb = rgb444;default
filter_indicator = rgb355;default
admin_warning = rgb553;default

# Status messages (optional)
warning = rgb554;color0
info = rgb355;color0
success = rgb335;color0
```

## 📚 Popular MC Themes

Compatible with themes from:
- [Dracula for MC](https://draculatheme.com/midnight-commander)
- [MC Skins Collection](https://github.com/iwfmp/mc-skins)
- Any standard MC .ini theme

## ⚠️ Limitations

**What WON'T be used from MC themes:**

| MC Section | Why Not Used |
|------------|--------------|
| `[Lines]` | Path Commander uses hardcoded Unicode box characters |
| `[editor]` | PC doesn't have a text editor |
| `[viewer]` | PC doesn't have a file viewer |
| `[diffviewer]` | PC doesn't have a diff viewer |
| `[widget-panel]` | Overlaps with `[core]`, not needed |

## 🎯 Best Practices

### For Best Results:

1. **Use MC themes as-is** - they'll work with smart fallbacks
2. **Add `[pathcommander]` section** for perfect color matching
3. **Test with 't' key** - see live preview before committing

### Creating PC-Enhanced Themes:

```ini
[skin]
description = My Theme for Path Commander
version = 1.0

# Standard MC sections (required)
[core]
_default_ = white;black
selected = black;cyan
marked = yellow;black
header = white;blue

[dialog]
_default_ = white;blue
dtitle = yellow;blue

[statusbar]
_default_ = black;white

# PC enhancements (optional but recommended)
[pathcommander]
path_valid = green;black
path_dead = red;black
path_duplicate = yellow;black
path_nonnormalized = cyan;black
```

## 🔍 Testing Compatibility

To test an MC theme:

```bash
# Load the theme
pc --theme path/to/theme.ini

# Check if colors look right:
# - Do valid paths look good? (green)
# - Do dead paths stand out? (red)
# - Does selection work? (should use theme's 'selected' color)
# - Do dialogs match theme?
```

## 📊 Color Format Support

Path Commander supports MC's color notation:

- **RGB**: `rgb524` (0-5 range per channel, converted to 0-255)
- **Named**: `black`, `white`, `red`, `blue`, `cyan`, `yellow`, `green`, `magenta`
- **Bright**: `brightred`, `brightblue`, etc.
- **Indexed**: `color0` through `color15`
- **Default**: `default` (terminal default)

## 🤝 Contributing MC Themes

Found a great MC theme that works well? Add it to `themes/` and submit a PR!

We accept:
- Direct ports of popular MC themes
- PC-enhanced versions with `[pathcommander]` sections
- Original themes in MC-compatible format
