# Dracula MC Theme Color Extraction

This document shows how Path Commander intelligently extracts colors from the unmodified Dracula Midnight Commander theme.

## Color Palette Extracted from dracula-mc.ini

### Core Colors (from `[core]` section)
- **Panel background**: `rgb111` → (51, 51, 51) - Dark gray/navy
- **Panel text**: `rgb555` → (255, 255, 255) - White
- **Selection bg**: `rgb524` → (255, 102, 204) - **PINK**
- **Marked items**: `rgb253` → (102, 255, 153) - Cyan

### Dialog Colors (from `[dialog]` section)
- **Dialog bg**: `rgb222` → (102, 102, 102) - Medium gray
- **Dialog focus bg**: `rgb435` → (204, 153, 255) - **PURPLE/LAVENDER** ✨
- **Dialog title**: `rgb355` → (153, 255, 255) - Cyan

### File Highlight Colors (from `[filehighlight]` section)
- **Directories**: `rgb335` → (153, 153, 255) - **LIGHT PURPLE** ✨
- **Symlinks**: `rgb542` → (255, 204, 102) - Orange/peach
- **Executables**: `rgb253` → (102, 255, 153) - Bright green

### Error Colors (from `[error]` section)
- **Error text**: `rgb511` → (255, 51, 51) - Bright red

### Button Bar Colors (from `[buttonbar]` section)
- **Hotkey bg**: `rgb524` → (255, 102, 204) - **PINK** ✨
- **Button text**: `rgb555` → (255, 255, 255) - White

## Intelligent Mapping to Path Commander

Path Commander uses these MC colors to create a cohesive Dracula experience:

| PC Feature | MC Source | Color Value | Visual Effect |
|------------|-----------|-------------|---------------|
| **Panel borders** | `[dialog] dfocus` bg | `rgb435` purple | Purple panel frames |
| **Valid paths** | `[filehighlight] directory` | `rgb335` light purple | Directories = valid paths |
| **Dead paths** | `[error] _default_` | `rgb511` red | Errors = dead paths |
| **Duplicate paths** | `[filehighlight] symlink` | `rgb542` orange | Symlinks = duplicates |
| **Scrollbar track** | `[dialog] dfocus` bg | `rgb435` purple | Purple scrollbar |
| **Scrollbar thumb** | `[buttonbar] hotkey` bg | `rgb524` pink | Pink scroll thumb |
| **Filter indicator** | `[dialog] dfocus` bg | `rgb435` purple | Purple filter badge |
| **Selection** | `[core] selected` | `rgb524` pink | Pink selection highlight |

## Result: Authentic Dracula Experience

By extracting these purple and pink colors from the MC theme, Path Commander automatically inherits the signature Dracula aesthetic:
- **Purple everywhere**: Borders, scrollbars, filter indicators
- **Pink highlights**: Selection, scroll thumb, button bar
- **Consistent file colors**: Directories, symlinks, errors match MC conventions

No custom `[pathcommander]` section needed - the unmodified MC theme works perfectly!

## RGB Color Notation

MC uses `rgbXYZ` notation where X, Y, Z are 0-5:
- `0` = 0/255 (darkest)
- `1` = 51/255
- `2` = 102/255
- `3` = 153/255
- `4` = 204/255
- `5` = 255/255 (brightest)

Examples:
- `rgb435` = (204, 153, 255) = Purple/lavender
- `rgb524` = (255, 102, 204) = Pink/magenta
- `rgb335` = (153, 153, 255) = Light purple/blue
