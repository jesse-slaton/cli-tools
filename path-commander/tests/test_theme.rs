// The crate is binary-only (no library target), so an integration test cannot
// import `theme` from it. Include the self-contained module source directly.
// Not every item in the module is used by this test.
#![allow(dead_code)]

use ratatui::style::Color;
use std::path::Path;

#[path = "../src/theme.rs"]
mod theme;
use theme::Theme;

#[test]
fn loads_bundled_dracula_skin() {
    let skin = Path::new(env!("CARGO_MANIFEST_DIR")).join("themes/dracula-pc-enhanced.ini");
    let theme = Theme::from_mc_skin(&skin).expect("bundled skin should parse");

    // The name comes from [skin] description, not the file-stem fallback.
    assert_eq!(theme.name, "Dracula Theme for Path Commander");
    // `[core] selected = white;rgb524` parses to 255/5*{5,2,4}; the default is Color::Cyan,
    // so this fails if INI parsing silently stops applying colors.
    assert_eq!(theme.panel_selected_bg, Color::Rgb(255, 102, 204));
}
