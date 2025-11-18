# Path Commander - Interactive Colorway Mapping Guide

This guide shows **which theme fields control which UI elements** for the Dracula theme.
Provide feedback on any color choices that don't look right!

## Legend
- **Theme Field**: The `Theme` struct field name
- **Dracula Color**: RGB values used in Dracula theme
- **Where Used**: Which UI element this controls

---

## 1. MAIN SCREEN - Header & Status Bar

```
┌─────────────────────────────────────────────────────────────────────────┐
│ ①Path Commander v1.0.0 - Windows PATH Editor    ②[ADMIN] ⚠            │
├──────────────────────┬──────────────────────────────────────────────────┤
│ MACHINE (System)     │ USER (Current User)              ③Filter: Dead  │
```

### Color Mapping:

| Element | Theme Field | Dracula Color | Notes |
|---------|-------------|---------------|-------|
| ① Header text | `header_fg` | Cyan #8be9fd | Main title bar text |
| ① Header background | `header_bg` | Background #282a36 | Title bar background |
| ② Admin warning | `admin_warning_fg` | Orange #ffb86c | "[ADMIN] ⚠" indicator |
| ③ Filter indicator | `filter_indicator_fg` | Cyan #8be9fd | Active filter text |

**Question**: Should admin warning be Orange, or use Yellow like other warnings?

---

## 2. MAIN SCREEN - Panel Content

```
│ MACHINE (System)     │ USER (Current User)                              │
│ ┌──────────────────┐ │ ┌──────────────────────────────────────────────┐ │
│ │①☐ C:\Windows\Sys │ │ │②☑ C:\Users\Me\App                           │ │  <- ① Normal
│ │③☐ C:\Program File│ │ │④☐ C:\Missing\Path                           │ │  <- ③ Selected
│ │⑤☑ C:\Duplicate   │ │ │⑥☐ C:\Duplicate                              │ │  <- ⑤ Marked
│ │⑦☐ C:\PROGRA~1    │ │ │⑧☐ %USERPROFILE%\tools                       │ │
│ │  ...             │ │ │  ...                                        │ │
│ └──────────────────┘ │ └──────────────────────────────────────────────┘ │
│ ⑨4 paths (1 marked) │ 4 paths (0 marked)                              │
```

### Color Mapping:

| Element | Theme Field | Dracula Color | Notes |
|---------|-------------|---------------|-------|
| ① Normal path | `panel_normal_fg` | Foreground #f8f8f2 | Unselected items |
| ② Normal background | `panel_normal_bg` | Background #282a36 | Panel background |
| ③ Selected path | `panel_selected_fg` | Foreground #f8f8f2 | Selected item text |
| ④ Selected background | `panel_selected_bg` | Current Line #44475a | Selection highlight |
| ⑤ Marked path | `panel_marked_fg` | Pink #ff79c6 | Checked items |
| ⑥ Marked background | `panel_marked_bg` | Background #282a36 | Background for marked |
| ⑦ Panel border | `panel_border_fg` | Comment #6272a4 | Box borders |
| ⑧ Panel border bg | `panel_border_bg` | Background #282a36 | Border background |
| ⑨ Status count | `status_fg` | Foreground #f8f8f2 | Bottom status text |

**Question**: Is Pink too bright for marked items? Should it be Purple instead?

---

## 3. MAIN SCREEN - Path Status Colors

```
│ ☐ C:\Windows\System32           ← Valid path (exists)
│ ☐ C:\Missing\Directory          ← Dead path (doesn't exist)
│ ☐ C:\Duplicate                  ← Duplicate path
│ ☐ C:\PROGRA~1                   ← Non-normalized path
```

### Color Mapping:

| Status | Theme Field | Dracula Color | Visual |
|--------|-------------|---------------|--------|
| Valid | `path_valid_fg` | Green #50fa7b | Bright green |
| Dead | `path_dead_fg` | Red #ff5555 | Bright red |
| Duplicate | `path_duplicate_fg` | Yellow #f1fa8c | Bright yellow |
| Non-normalized | `path_nonnormalized_fg` | Cyan #8be9fd | Cyan |

**Question**: These are the most prominent colors. Do they look distinct enough?

---

## 4. MAIN SCREEN - Scrollbar

```
│ │ ☐ Path 1        │║
│ │ ☐ Path 2        │█  ← Scrollbar thumb
│ │ ☐ Path 3        │║  ← Scrollbar track
│ │ ☐ Path 4        │║
```

### Color Mapping:

| Element | Theme Field | Dracula Color | Notes |
|---------|-------------|---------------|-------|
| Scrollbar track | `scrollbar_fg` | Comment #6272a4 | Track (║ character) |
| Scrollbar background | `scrollbar_bg` | Background #282a36 | Behind track |
| Scrollbar thumb | `scrollbar_thumb_fg` | Gray #808080 | Thumb position (█) |

**Question**: Should scrollbar be more visible, or stay subtle?

---

## 5. STATUS BAR - Bottom

```
├──────────────────────┴──────────────────────────────────────────────────┤
│ ①F1 Help  F2 Backup  ②F3 Filter  F5 Edit  Ins Add  Del Delete  F10 Quit│
└─────────────────────────────────────────────────────────────────────────┘
  ③4 marked                                                                  (if items marked)
```

### Color Mapping:

| Element | Theme Field | Dracula Color | Notes |
|---------|-------------|---------------|-------|
| ① Status bar text | `status_fg` | Foreground #f8f8f2 | Function key labels |
| ② Status bar bg | `status_bg` | Current Line #44475a | Bottom bar background |
| ③ Marked count | `panel_marked_fg` | Yellow #f1fa8c | "4 marked" text |

**Question**: Should "marked" count use Pink (marked items) or Yellow (warning)?

---

## 6. HELP SCREEN (F1)

```
┌───────────────────── ①Help ─────────────────────────┐
│                                                      │
│  ②Navigation                                         │
│    ③↑↓ / jk       Move selection                    │
│    Tab          Switch panel                        │
│                                                      │
│  Path Status Colors:                                │
│    ④Red         Dead paths (don't exist)           │
│    ⑤Yellow      Duplicate paths                    │
│                                                      │
│  ⑥Press any key to close                            │
└──────────────────────────────────────────────────────┘
```

### Color Mapping:

| Element | Theme Field | Dracula Color | Notes |
|---------|-------------|---------------|-------|
| ① Dialog title | `dialog_title_fg` | Cyan #8be9fd | "Help" title |
| ① Dialog title bg | `dialog_title_bg` | Current Line #44475a | Title background |
| ② Section headers | `help_bold_fg` | Cyan #8be9fd | "Navigation", etc. |
| ③ Normal help text | `help_fg` | Foreground #f8f8f2 | Body text |
| ④ Color examples | *Fixed colors* | Red/Yellow/etc | Shows actual colors |
| ⑤ Help links | `help_link_fg` | Purple #bd93f9 | Interactive elements |
| ⑥ Footer message | `warning_fg` | Yellow #f1fa8c | "Press any key..." |
| Dialog background | `help_bg` | Background #282a36 | Help screen bg |

**Question**: Should section headers (②) be Cyan, Yellow, or Purple?

---

## 7. CONFIRM DIALOGS (Delete, Apply, Quit, etc.)

```
┌─────────────── ①Confirm ───────────────┐
│                                         │
│  ②Apply the following changes?         │
│                                         │
│  ③⚠ USER scope (4 changes)              │
│    • Add: C:\New\Path                  │
│    • Delete: C:\Old\Path               │
│                                         │
│  ④This will update the registry        │
│                                         │
│  ⑤[Yes] [No]                            │
│                                         │
└─────────────────────────────────────────┘
```

### Color Mapping:

| Element | Theme Field | Dracula Color | Notes |
|---------|-------------|---------------|-------|
| ① Dialog border | `dialog_border_fg` | Cyan #8be9fd | Border and title |
| ② Main text | `dialog_fg` | Foreground #f8f8f2 | Dialog message |
| ② Dialog background | `dialog_bg` | Current Line #44475a | Dialog background |
| ③ Warning text | `warning_fg` | Yellow #f1fa8c | "⚠" warnings |
| ④ Info text | `info_fg` | Cyan #8be9fd | Informational notes |
| ⑤ Button (unfocused) | `button_fg` | Current Line #44475a | Button text |
| ⑤ Button (focused) | `button_focused_bg` | Green #50fa7b | Focused button bg |

**Question**: Should warnings be Yellow or Orange? Info be Cyan or Purple?

---

## 8. PROCESS RESTART INFO

```
┌─────── Process Restart Info ───────┐
│                                     │
│  ①✓ Successfully updated PATH       │
│                                     │
│  ②⚠ Running processes detected:    │
│    • ③cmd.exe (2 instances)        │
│    • ③powershell.exe               │
│                                     │
│  ④Note: Restart these processes    │
│                                     │
│  ⑤Press any key to continue        │
└─────────────────────────────────────┘
```

### Color Mapping:

| Element | Theme Field | Dracula Color | Notes |
|---------|-------------|---------------|-------|
| ① Success header | `success_fg` | Green #50fa7b | "✓ Successfully..." |
| ② Warning header | `warning_fg` | Yellow #f1fa8c | "⚠ Running processes..." |
| ③ Process names | `info_fg` | Cyan #8be9fd | Process list items |
| ④ Note labels | `warning_fg` | Yellow #f1fa8c | "Note:" prefix |
| ⑤ Footer message | `warning_fg` | Yellow #f1fa8c | "Press any key..." |

**Question**: Should process names be Cyan, Purple, or Foreground color?

---

## 9. INPUT OVERLAY (Add/Edit Path)

```
┌────── ①Add Path ──────┐
│                        │
│  ②Enter path:          │
│  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓     │  ← Input field
│                        │
│  ③Press Enter to add   │
│  ③Press Esc to cancel  │
│                        │
└────────────────────────┘
```

### Color Mapping:

| Element | Theme Field | Dracula Color | Notes |
|---------|-------------|---------------|-------|
| ① Dialog border/title | `dialog_border_fg` | Cyan #8be9fd | Border and title |
| ② Main text | `dialog_fg` | Foreground #f8f8f2 | Prompt text |
| ③ Help text | `info_fg` | Cyan #8be9fd | Instructions |
| Dialog background | `dialog_bg` | Current Line #44475a | Background |

**Question**: Should help text (③) be dimmed (Gray) or bright (Cyan)?

---

## 10. BACKUP LIST

```
┌─────── ①Backup List ───────┐
│                             │
│  ②> backup_20250115_143022  │  ← Selected
│    backup_20250114_120045   │
│    backup_20250113_095512   │
│                             │
└─────────────────────────────┘
```

### Color Mapping:

| Element | Theme Field | Dracula Color | Notes |
|---------|-------------|---------------|-------|
| ① Dialog border | `dialog_border_fg` | Cyan #8be9fd | Border currently hardcoded |
| ② Selected item | `panel_selected_bg` | Current Line #44475a | Selection highlight |
| Normal items | `dialog_fg` | Foreground #f8f8f2 | List items |

**Question**: Should backup list use same colors as main panels?

---

## Summary - Predominant Theme Fields by Frequency

Based on the analysis, here are the **most important** fields that appear most often:

### Top 10 Most Visible Fields:

1. **`panel_normal_fg/bg`** - Used for all unselected paths (most of the screen)
2. **`panel_selected_fg/bg`** - Used for current selection (always visible)
3. **`path_valid_fg`** - Most paths are green (valid) in normal use
4. **`path_dead_fg`** - Very prominent when paths are broken (red stands out)
5. **`dialog_border_fg`** - Used in all dialogs/overlays (Cyan is very visible)
6. **`dialog_fg/bg`** - Used in all dialogs (frequent)
7. **`header_fg/bg`** - Always visible at top
8. **`status_fg/bg`** - Always visible at bottom
9. **`warning_fg`** - Used frequently for important messages
10. **`panel_marked_fg`** - Very visible when marking items (Pink)

### Suggested Priorities for Feedback:

**Critical** (must look good):
- Main panel colors (normal, selected)
- Path status colors (valid=green, dead=red, duplicate=yellow, non-normalized=cyan)
- Dialog borders (currently Cyan everywhere)

**Important** (frequently seen):
- Marked items (currently Pink - is this too bright?)
- Warning messages (Yellow vs Orange?)
- Success messages (Green)

**Nice to have** (less frequent):
- Scrollbar visibility
- Help screen headers
- Process names in restart info

---

## Questions for Feedback

Please review and provide feedback on:

1. **Pink for marked items** - Too bright? Use Purple instead?
2. **Admin warning** - Orange or Yellow?
3. **Cyan for borders/titles** - Too much Cyan? Some should be Purple?
4. **Scrollbar** - Too subtle? Should thumb be brighter?
5. **"marked" count in status bar** - Yellow or Pink?
6. **Help screen headers** - Cyan, Yellow, or Purple?
7. **Info messages** - Cyan or Purple?
8. **Process names** - Cyan, Purple, or normal Foreground?
9. **Help text in dialogs** - Bright (Cyan) or dimmed (Gray)?
10. **Overall balance** - Too much of any one color?

Please provide feedback in this format:
```
Field: path_marked_fg
Current: Pink #ff79c6
Suggested: Purple #bd93f9
Reason: Pink is too bright/distracting
```

Or just note which elements look wrong and I'll suggest alternatives!
