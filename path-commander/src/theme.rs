use anyhow::{anyhow, Result};
use ratatui::style::Color;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Represents a color theme for Path Commander, compatible with Midnight Commander skins
#[derive(Debug, Clone)]
pub struct Theme {
    /// Theme name
    pub name: String,
    /// Color definitions from the skin
    colors: HashMap<String, Color>,
    /// Foreground/background pairs for different UI elements
    pub panel_normal_fg: Color,
    pub panel_normal_bg: Color,
    pub panel_selected_fg: Color,
    pub panel_selected_bg: Color,
    pub panel_marked_fg: Color,
    pub panel_marked_bg: Color,
    pub panel_border_fg: Color,
    pub header_fg: Color,
    pub header_bg: Color,
    pub status_fg: Color,
    pub status_bg: Color,
    pub dialog_fg: Color,
    pub dialog_bg: Color,
    pub dialog_border_fg: Color,
    pub error_fg: Color,
    pub button_fg: Color,
    pub button_focused_fg: Color,
    pub button_focused_bg: Color,
    pub path_valid_fg: Color,
    pub path_dead_fg: Color,
    pub path_duplicate_fg: Color,
    pub path_nonnormalized_fg: Color,
}

impl Theme {
    /// Load a theme from a Midnight Commander .ini skin file
    /// TODO: Implement full .ini parser for MC skin compatibility
    pub fn from_mc_skin(_path: &Path) -> Result<Self> {
        Err(anyhow!("MC skin file loading not yet implemented. Use built-in themes: default, dracula"))
    }

    /// Load a built-in theme by name
    pub fn builtin(name: &str) -> Result<Self> {
        match name.to_lowercase().as_str() {
            "dracula" => Ok(Self::dracula()),
            "default" | "classic" => Ok(Self::default()),
            _ => Err(anyhow!("Unknown built-in theme: {}", name)),
        }
    }

    /// Get the default Path Commander theme
    pub fn default() -> Self {
        Self {
            name: "default".to_string(),
            colors: HashMap::new(),
            panel_normal_fg: Color::White,
            panel_normal_bg: Color::Black,
            panel_selected_fg: Color::Black,
            panel_selected_bg: Color::Cyan,
            panel_marked_fg: Color::Yellow,
            panel_marked_bg: Color::Black,
            panel_border_fg: Color::White,
            header_fg: Color::White,
            header_bg: Color::Blue,
            status_fg: Color::Black,
            status_bg: Color::White,
            dialog_fg: Color::White,
            dialog_bg: Color::Blue,
            dialog_border_fg: Color::Cyan,
            error_fg: Color::Red,
            button_fg: Color::Black,
            button_focused_fg: Color::White,
            button_focused_bg: Color::Green,
            path_valid_fg: Color::Green,
            path_dead_fg: Color::Red,
            path_duplicate_fg: Color::Yellow,
            path_nonnormalized_fg: Color::Cyan,
        }
    }

    /// Get the Dracula theme (inspired by MC Dracula skin)
    pub fn dracula() -> Self {
        Self {
            name: "dracula".to_string(),
            colors: HashMap::new(),
            panel_normal_fg: Color::Rgb(248, 248, 242), // Foreground
            panel_normal_bg: Color::Rgb(40, 42, 54),     // Background
            panel_selected_fg: Color::Rgb(248, 248, 242),
            panel_selected_bg: Color::Rgb(68, 71, 90),   // Current Line
            panel_marked_fg: Color::Rgb(255, 121, 198),  // Pink
            panel_marked_bg: Color::Rgb(40, 42, 54),
            panel_border_fg: Color::Rgb(98, 114, 164),   // Comment
            header_fg: Color::Rgb(139, 233, 253),        // Cyan
            header_bg: Color::Rgb(40, 42, 54),
            status_fg: Color::Rgb(248, 248, 242),
            status_bg: Color::Rgb(68, 71, 90),
            dialog_fg: Color::Rgb(248, 248, 242),
            dialog_bg: Color::Rgb(68, 71, 90),
            dialog_border_fg: Color::Rgb(139, 233, 253), // Cyan
            error_fg: Color::Rgb(255, 85, 85),           // Red
            button_fg: Color::Rgb(68, 71, 90),
            button_focused_fg: Color::Rgb(40, 42, 54),
            button_focused_bg: Color::Rgb(80, 250, 123), // Green
            path_valid_fg: Color::Rgb(80, 250, 123),     // Green
            path_dead_fg: Color::Rgb(255, 85, 85),       // Red
            path_duplicate_fg: Color::Rgb(241, 250, 140), // Yellow
            path_nonnormalized_fg: Color::Rgb(139, 233, 253), // Cyan
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::default()
    }
}

/// Parse MC color notation (e.g., "rgb524", "color0", "black")
fn parse_mc_color(s: &str) -> Result<Color> {
    let s = s.trim();

    // Handle rgb notation (rgb012 where each digit is 0-5)
    if s.starts_with("rgb") && s.len() == 6 {
        let r = s.chars().nth(3).and_then(|c| c.to_digit(6)).ok_or_else(|| anyhow!("Invalid rgb"))?;
        let g = s.chars().nth(4).and_then(|c| c.to_digit(6)).ok_or_else(|| anyhow!("Invalid rgb"))?;
        let b = s.chars().nth(5).and_then(|c| c.to_digit(6)).ok_or_else(|| anyhow!("Invalid rgb"))?;

        // Convert 0-5 range to 0-255
        let r = ((r * 255) / 5) as u8;
        let g = ((g * 255) / 5) as u8;
        let b = ((b * 255) / 5) as u8;

        return Ok(Color::Rgb(r, g, b));
    }

    // Handle color names
    match s.to_lowercase().as_str() {
        "black" | "color0" => Ok(Color::Black),
        "red" | "color1" => Ok(Color::Red),
        "green" | "color2" => Ok(Color::Green),
        "yellow" | "color3" => Ok(Color::Yellow),
        "blue" | "color4" => Ok(Color::Blue),
        "magenta" | "color5" => Ok(Color::Magenta),
        "cyan" | "color6" => Ok(Color::Cyan),
        "white" | "color7" => Ok(Color::White),
        "brightblack" | "gray" | "color8" => Ok(Color::DarkGray),
        "brightred" | "color9" => Ok(Color::LightRed),
        "brightgreen" | "color10" => Ok(Color::LightGreen),
        "brightyellow" | "color11" => Ok(Color::LightYellow),
        "brightblue" | "color12" => Ok(Color::LightBlue),
        "brightmagenta" | "color13" => Ok(Color::LightMagenta),
        "brightcyan" | "color14" => Ok(Color::LightCyan),
        "brightwhite" | "color15" => Ok(Color::White),
        _ => Err(anyhow!("Unknown color: {}", s)),
    }
}

/// Parse MC color pair (e.g., "rgb555;rgb435" or "white;blue")
fn parse_mc_color_pair(s: &str, color_map: &HashMap<String, Color>) -> Option<(Color, Color)> {
    let parts: Vec<&str> = s.split(';').collect();
    if parts.len() != 2 {
        return None;
    }

    // Try to resolve as color name from map, or parse directly
    let fg = color_map
        .get(parts[0].trim())
        .cloned()
        .or_else(|| parse_mc_color(parts[0]).ok())?;

    let bg = color_map
        .get(parts[1].trim())
        .cloned()
        .or_else(|| parse_mc_color(parts[1]).ok())?;

    Some((fg, bg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mc_color_rgb() {
        let color = parse_mc_color("rgb555").unwrap();
        assert_eq!(color, Color::Rgb(255, 255, 255));

        let color = parse_mc_color("rgb000").unwrap();
        assert_eq!(color, Color::Rgb(0, 0, 0));

        let color = parse_mc_color("rgb524").unwrap();
        // 5 -> 255, 2 -> 102, 4 -> 204
        assert_eq!(color, Color::Rgb(255, 102, 204));
    }

    #[test]
    fn test_parse_mc_color_names() {
        assert_eq!(parse_mc_color("black").unwrap(), Color::Black);
        assert_eq!(parse_mc_color("white").unwrap(), Color::White);
        assert_eq!(parse_mc_color("color0").unwrap(), Color::Black);
    }

    #[test]
    fn test_builtin_themes() {
        let default_theme = Theme::default();
        assert_eq!(default_theme.name, "default");

        let dracula = Theme::dracula();
        assert_eq!(dracula.name, "dracula");
    }
}
