use anyhow::{anyhow, Context, Result};
use ratatui::style::Color;
use std::collections::HashMap;
use std::path::Path;

/// Represents a color theme for Path Commander, compatible with Midnight Commander skins
#[derive(Debug, Clone)]
#[allow(dead_code)] // Some fields reserved for future dialog theming improvements
pub struct Theme {
    /// Theme name
    pub name: String,
    /// Color definitions from the skin (reserved for MC skin file parsing)
    colors: HashMap<String, Color>,

    // Panel colors
    pub panel_normal_fg: Color,
    pub panel_normal_bg: Color,
    pub panel_selected_fg: Color,
    pub panel_selected_bg: Color,
    pub panel_marked_fg: Color,
    pub panel_marked_bg: Color,
    pub panel_border_fg: Color,
    pub panel_border_bg: Color,

    // Header and status
    pub header_fg: Color,
    pub header_bg: Color,
    pub status_fg: Color,
    pub status_bg: Color,

    // Dialog colors
    pub dialog_fg: Color,
    pub dialog_bg: Color,
    pub dialog_border_fg: Color,
    pub dialog_title_fg: Color,
    pub dialog_title_bg: Color,

    // Error/status messages
    pub error_fg: Color,
    pub warning_fg: Color,
    pub warning_bg: Color,
    pub info_fg: Color,
    pub info_bg: Color,
    pub success_fg: Color,
    pub success_bg: Color,

    // Button colors
    pub button_fg: Color,
    pub button_bg: Color,
    pub button_focused_fg: Color,
    pub button_focused_bg: Color,
    pub button_disabled_fg: Color,
    pub button_disabled_bg: Color,

    // Help screen colors
    pub help_fg: Color,
    pub help_bg: Color,
    pub help_bold_fg: Color,
    pub help_bold_bg: Color,
    pub help_link_fg: Color,
    pub help_link_bg: Color,

    // Path status colors
    pub path_valid_fg: Color,
    pub path_valid_bg: Color,
    pub path_dead_fg: Color,
    pub path_dead_bg: Color,
    pub path_duplicate_fg: Color,
    pub path_duplicate_bg: Color,
    pub path_nonnormalized_fg: Color,
    pub path_nonnormalized_bg: Color,

    // UI element colors
    pub scrollbar_fg: Color,
    pub scrollbar_bg: Color,
    pub scrollbar_thumb_fg: Color,
    pub scrollbar_thumb_bg: Color,
    pub filter_indicator_fg: Color,
    pub filter_indicator_bg: Color,
    pub admin_warning_fg: Color,
    pub admin_warning_bg: Color,

    // Function key display (MC-style buttonbar - see issue #16)
    pub function_key_number_fg: Color,
    pub function_key_number_bg: Color,
    pub function_key_label_fg: Color,
    pub function_key_label_bg: Color,
}

impl Theme {
    /// Load a theme from a Midnight Commander .ini skin file
    pub fn from_mc_skin(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read INI file: {}", path.display()))?;

        // Parse INI file into sections
        let ini_data = Self::parse_ini(&content)?;

        // Get theme name from metadata or filename
        let name = ini_data
            .get("skin")
            .and_then(|s| s.get("description"))
            .map(|s| s.as_str())
            .unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("custom")
            })
            .to_string();

        Ok(Self::from_mc_data(name, ini_data))
    }

    /// Create a theme from parsed MC INI data
    /// This is where intelligent mapping happens for MC themes
    fn from_mc_data(name: String, ini_data: HashMap<String, HashMap<String, String>>) -> Self {
        // Helper to get color pair or use default
        let get_color_pair = |section: &str, key: &str, default_fg: Color, default_bg: Color| {
            ini_data
                .get(section)
                .and_then(|s| s.get(key))
                .and_then(|v| parse_mc_color_pair(v, &HashMap::new()))
                .unwrap_or((default_fg, default_bg))
        };

        // Parse [core] section
        let (panel_normal_fg, panel_normal_bg) =
            get_color_pair("core", "_default_", Color::White, Color::Black);
        let (panel_selected_fg, panel_selected_bg) =
            get_color_pair("core", "selected", Color::Black, Color::Cyan);
        let (panel_marked_fg, panel_marked_bg) =
            get_color_pair("core", "marked", Color::Yellow, Color::Black);
        let (header_fg, header_bg) = get_color_pair("core", "header", Color::White, Color::Blue);

        // Parse [dialog] section
        let (dialog_fg, dialog_bg) =
            get_color_pair("dialog", "_default_", Color::White, Color::Blue);
        let (dialog_title_fg, dialog_title_bg) =
            get_color_pair("dialog", "dtitle", Color::Yellow, Color::Blue);
        let (_dialog_focus_fg, dialog_focus_bg) =
            get_color_pair("dialog", "dfocus", dialog_fg, dialog_bg);

        // Parse [widget-common] section
        let (button_fg, button_bg) =
            get_color_pair("widget-common", "button_default", Color::Black, Color::Cyan);
        let (button_focused_fg, button_focused_bg) =
            get_color_pair("widget-common", "button_focus", Color::White, Color::Green);
        let (button_disabled_fg, button_disabled_bg) = get_color_pair(
            "widget-common",
            "button_disabled",
            Color::DarkGray,
            Color::Black,
        );

        // Parse [statusbar] section
        let (status_fg, status_bg) =
            get_color_pair("statusbar", "_default_", Color::Black, Color::White);

        // Parse [help] section
        let (help_fg, help_bg) = get_color_pair("help", "_default_", Color::White, Color::Blue);
        let (help_bold_fg, help_bold_bg) =
            get_color_pair("help", "helpbold", Color::Yellow, Color::Blue);
        let (help_link_fg, help_link_bg) =
            get_color_pair("help", "helplink", Color::LightCyan, Color::Blue);

        // Parse [menu] section (reserved for future menu implementation)
        let (_menu_fg, _menu_bg) = get_color_pair("menu", "_default_", dialog_fg, dialog_bg);
        let (_menu_selected_fg, _menu_selected_bg) =
            get_color_pair("menu", "menusel", panel_selected_fg, panel_selected_bg);

        // Parse [buttonbar] section (MC function key display - see issue #16)
        // In MC, "hotkey" is the number (e.g., "3" in "3View") with colored background
        // and "button" is the label text (e.g., "View" in "3View")
        let (buttonbar_hotkey_fg, buttonbar_hotkey_bg) =
            get_color_pair("buttonbar", "hotkey", Color::White, panel_selected_bg);
        let (buttonbar_button_fg, buttonbar_button_bg) =
            get_color_pair("buttonbar", "button", status_fg, status_bg);

        // Parse [pathcommander] section (Path Commander specific)
        // If not present, fall back to MC's [filehighlight] and [error] sections for intelligent defaults

        // Panel border: Try PC-specific, then MC dialog focus background (purple), then fallback
        let (panel_border_fg, panel_border_bg) = if ini_data
            .get("pathcommander")
            .and_then(|s| s.get("border"))
            .is_some()
        {
            get_color_pair("pathcommander", "border", panel_normal_fg, panel_normal_bg)
        } else {
            (dialog_focus_bg, panel_normal_bg) // Use MC dialog focus color (typically purple)
        };

        // Dialog border uses same color as panel border
        let dialog_border_fg = panel_border_fg;

        // Path valid: Try PC-specific, then MC directory color, then default green
        let (path_valid_fg, path_valid_bg) = if ini_data
            .get("pathcommander")
            .and_then(|s| s.get("path_valid"))
            .is_some()
        {
            get_color_pair("pathcommander", "path_valid", Color::Green, panel_normal_bg)
        } else if let Some((fg, _)) = ini_data
            .get("filehighlight")
            .and_then(|s| s.get("directory"))
            .and_then(|v| parse_mc_color_pair(v, &HashMap::new()))
        {
            (fg, panel_normal_bg) // Use MC directory color
        } else {
            (Color::Green, panel_normal_bg)
        };

        // Path dead: Try PC-specific, then MC error color, then default red
        let (path_dead_fg, path_dead_bg) = if ini_data
            .get("pathcommander")
            .and_then(|s| s.get("path_dead"))
            .is_some()
        {
            get_color_pair("pathcommander", "path_dead", Color::Red, panel_normal_bg)
        } else if let Some((fg, _)) = ini_data
            .get("error")
            .and_then(|s| s.get("_default_"))
            .and_then(|v| parse_mc_color_pair(v, &HashMap::new()))
        {
            (fg, panel_normal_bg) // Use MC error color
        } else {
            (Color::Red, panel_normal_bg)
        };

        // Path duplicate: Try PC-specific, then MC symlink color, then default yellow
        let (path_duplicate_fg, path_duplicate_bg) = if ini_data
            .get("pathcommander")
            .and_then(|s| s.get("path_duplicate"))
            .is_some()
        {
            get_color_pair(
                "pathcommander",
                "path_duplicate",
                Color::Yellow,
                panel_normal_bg,
            )
        } else if let Some((fg, _)) = ini_data
            .get("filehighlight")
            .and_then(|s| s.get("symlink"))
            .and_then(|v| parse_mc_color_pair(v, &HashMap::new()))
        {
            (fg, panel_normal_bg) // Use MC symlink color
        } else {
            (Color::Yellow, panel_normal_bg)
        };

        // Path non-normalized: Try PC-specific, then default cyan
        let (path_nonnormalized_fg, path_nonnormalized_bg) = get_color_pair(
            "pathcommander",
            "path_nonnormalized",
            Color::Cyan,
            panel_normal_bg,
        );

        let (warning_fg, warning_bg) =
            get_color_pair("pathcommander", "warning", Color::Yellow, dialog_bg);
        let (info_fg, info_bg) =
            get_color_pair("pathcommander", "info", Color::LightCyan, dialog_bg);
        let (success_fg, success_bg) =
            get_color_pair("pathcommander", "success", Color::Green, dialog_bg);

        // Scrollbar: Try PC-specific, then MC dialog focus (purple) or button background
        let (scrollbar_fg, scrollbar_bg) = if ini_data
            .get("pathcommander")
            .and_then(|s| s.get("scrollbar"))
            .is_some()
        {
            get_color_pair(
                "pathcommander",
                "scrollbar",
                panel_border_fg,
                panel_normal_bg,
            )
        } else {
            (dialog_focus_bg, panel_normal_bg) // Use MC dialog focus color (purple)
        };

        // Scrollbar thumb: Try PC-specific, then MC selection or button hotkey background (pink)
        let (scrollbar_thumb_fg, scrollbar_thumb_bg) = if ini_data
            .get("pathcommander")
            .and_then(|s| s.get("scrollbar_thumb"))
            .is_some()
        {
            get_color_pair(
                "pathcommander",
                "scrollbar_thumb",
                panel_selected_bg,
                panel_normal_bg,
            )
        } else {
            (buttonbar_hotkey_bg, panel_normal_bg) // Use MC button hotkey color (pink)
        };

        // Filter indicator: Try PC-specific, then MC dialog focus (purple)
        let (filter_indicator_fg, filter_indicator_bg) = if ini_data
            .get("pathcommander")
            .and_then(|s| s.get("filter_indicator"))
            .is_some()
        {
            get_color_pair(
                "pathcommander",
                "filter_indicator",
                header_fg,
                panel_normal_bg,
            )
        } else {
            (dialog_focus_bg, panel_normal_bg) // Use MC dialog focus color (purple)
        };
        let (admin_warning_fg, admin_warning_bg) = get_color_pair(
            "pathcommander",
            "admin_warning",
            warning_fg,
            panel_normal_bg,
        );

        Self {
            name,
            colors: HashMap::new(),

            // Panel colors
            panel_normal_fg,
            panel_normal_bg,
            panel_selected_fg,
            panel_selected_bg,
            panel_marked_fg,
            panel_marked_bg,
            panel_border_fg,
            panel_border_bg,

            // Header and status
            header_fg,
            header_bg,
            status_fg,
            status_bg,

            // Dialog colors
            dialog_fg,
            dialog_bg,
            dialog_border_fg,
            dialog_title_fg,
            dialog_title_bg,

            // Error/status messages
            error_fg: path_dead_fg, // Reuse dead path color for errors
            warning_fg,
            warning_bg,
            info_fg,
            info_bg,
            success_fg,
            success_bg,

            // Button colors
            button_fg,
            button_bg,
            button_focused_fg,
            button_focused_bg,
            button_disabled_fg,
            button_disabled_bg,

            // Help screen colors
            help_fg,
            help_bg,
            help_bold_fg,
            help_bold_bg,
            help_link_fg,
            help_link_bg,

            // Path status colors
            path_valid_fg,
            path_valid_bg,
            path_dead_fg,
            path_dead_bg,
            path_duplicate_fg,
            path_duplicate_bg,
            path_nonnormalized_fg,
            path_nonnormalized_bg,

            // UI element colors
            scrollbar_fg,
            scrollbar_bg,
            scrollbar_thumb_fg,
            scrollbar_thumb_bg,
            filter_indicator_fg,
            filter_indicator_bg,
            admin_warning_fg,
            admin_warning_bg,

            // Function key display (MC-style buttonbar)
            function_key_number_fg: buttonbar_hotkey_fg,
            function_key_number_bg: buttonbar_hotkey_bg,
            function_key_label_fg: buttonbar_button_fg,
            function_key_label_bg: buttonbar_button_bg,
        }
    }

    /// Parse INI file content into sections and key-value pairs
    fn parse_ini(content: &str) -> Result<HashMap<String, HashMap<String, String>>> {
        let mut result: HashMap<String, HashMap<String, String>> = HashMap::new();
        let mut current_section = String::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            // Section header
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].trim().to_string();
                result.entry(current_section.clone()).or_default();
                continue;
            }

            // Key-value pair
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim().to_string();
                let value = line[eq_pos + 1..].trim().to_string();
                if !current_section.is_empty() {
                    result
                        .entry(current_section.clone())
                        .or_default()
                        .insert(key, value);
                }
            }
        }

        Ok(result)
    }

    /// Load a built-in theme by name
    pub fn builtin(name: &str) -> Result<Self> {
        match name.to_lowercase().as_str() {
            "default" | "classic" => Ok(Self::default()),
            _ => Err(anyhow!(
                "Unknown built-in theme: {}. Use 'default' or load a theme from ~/.pc/themes/",
                name
            )),
        }
    }

    /// Get the default Path Commander theme
    pub fn default() -> Self {
        Self {
            name: "default".to_string(),
            colors: HashMap::new(),

            // Panel colors
            panel_normal_fg: Color::White,
            panel_normal_bg: Color::Black,
            panel_selected_fg: Color::Black,
            panel_selected_bg: Color::Cyan,
            panel_marked_fg: Color::Yellow,
            panel_marked_bg: Color::Black,
            panel_border_fg: Color::White,
            panel_border_bg: Color::Black,

            // Header and status
            header_fg: Color::White,
            header_bg: Color::Blue,
            status_fg: Color::Black,
            status_bg: Color::White,

            // Dialog colors
            dialog_fg: Color::White,
            dialog_bg: Color::Blue,
            dialog_border_fg: Color::Cyan,
            dialog_title_fg: Color::Yellow,
            dialog_title_bg: Color::Blue,

            // Error/status messages
            error_fg: Color::Red,
            warning_fg: Color::Yellow,
            warning_bg: Color::Blue,
            info_fg: Color::LightCyan,
            info_bg: Color::Blue,
            success_fg: Color::Green,
            success_bg: Color::Blue,

            // Button colors
            button_fg: Color::Black,
            button_bg: Color::Cyan,
            button_focused_fg: Color::White,
            button_focused_bg: Color::Green,
            button_disabled_fg: Color::DarkGray,
            button_disabled_bg: Color::Black,

            // Help screen colors
            help_fg: Color::White,
            help_bg: Color::Blue,
            help_bold_fg: Color::Yellow,
            help_bold_bg: Color::Blue,
            help_link_fg: Color::LightCyan,
            help_link_bg: Color::Blue,

            // Path status colors
            path_valid_fg: Color::Green,
            path_valid_bg: Color::Black,
            path_dead_fg: Color::Red,
            path_dead_bg: Color::Black,
            path_duplicate_fg: Color::Yellow,
            path_duplicate_bg: Color::Black,
            path_nonnormalized_fg: Color::Cyan,
            path_nonnormalized_bg: Color::Black,

            // UI element colors
            scrollbar_fg: Color::DarkGray,
            scrollbar_bg: Color::Black,
            scrollbar_thumb_fg: Color::White,
            scrollbar_thumb_bg: Color::Black,
            filter_indicator_fg: Color::Cyan,
            filter_indicator_bg: Color::Black,
            admin_warning_fg: Color::Yellow,
            admin_warning_bg: Color::Black,

            // Function key display (MC-style buttonbar)
            function_key_number_fg: Color::White,
            function_key_number_bg: Color::Cyan,
            function_key_label_fg: Color::Black,
            function_key_label_bg: Color::White,
        }
    }
}

/// Parse MC color pair notation (fg;bg) supporting rgb, named colors, and color indices
fn parse_mc_color_pair(
    value: &str,
    _color_defs: &HashMap<String, Color>,
) -> Option<(Color, Color)> {
    let parts: Vec<&str> = value.split(';').collect();
    if parts.is_empty() {
        return None;
    }

    let fg = parse_mc_color(parts[0].trim())?;
    let bg = if parts.len() > 1 && !parts[1].trim().is_empty() {
        // Parse background color if present and not empty
        parse_mc_color(parts[1].trim()).unwrap_or(Color::Reset)
    } else {
        // Empty or missing background = transparent/default
        Color::Reset
    };

    Some((fg, bg))
}

/// Parse a single MC color value (rgb524, white, color0, etc.)
fn parse_mc_color(s: &str) -> Option<Color> {
    let s = s.trim();

    // Handle empty or default
    if s.is_empty() || s == "default" {
        return Some(Color::Reset);
    }

    // Handle rgb notation (e.g., rgb524 = r:5/5, g:2/5, b:4/5)
    if s.starts_with("rgb") && s.len() == 6 {
        let r = s.chars().nth(3)?.to_digit(10)? as u16;
        let g = s.chars().nth(4)?.to_digit(10)? as u16;
        let b = s.chars().nth(5)?.to_digit(10)? as u16;

        // Convert 0-5 scale to 0-255 scale
        let r_256 = ((r * 255) / 5) as u8;
        let g_256 = ((g * 255) / 5) as u8;
        let b_256 = ((b * 255) / 5) as u8;

        return Some(Color::Rgb(r_256, g_256, b_256));
    }

    // Handle indexed colors (color0-color15)
    if let Some(stripped) = s.strip_prefix("color") {
        if let Ok(idx) = stripped.parse::<u8>() {
            return Some(Color::Indexed(idx));
        }
    }

    // Handle named colors
    match s.to_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        "brightblack" | "gray" | "grey" => Some(Color::DarkGray),
        "brightred" => Some(Color::LightRed),
        "brightgreen" => Some(Color::LightGreen),
        "brightyellow" => Some(Color::LightYellow),
        "brightblue" => Some(Color::LightBlue),
        "brightmagenta" => Some(Color::LightMagenta),
        "brightcyan" => Some(Color::LightCyan),
        "brightwhite" => Some(Color::Gray),
        _ => None,
    }
}
