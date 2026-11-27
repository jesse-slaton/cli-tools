// The crate is binary-only (no library target), so an integration test cannot
// import `theme` from it. Include the self-contained module source directly.
// Not every item in the module is used by this test.
#![allow(dead_code)]

use std::path::Path;

#[path = "../src/theme.rs"]
mod theme;
use theme::Theme;

#[test]
fn loads_bundled_dracula_skin() {
    let skin = Path::new(env!("CARGO_MANIFEST_DIR")).join("themes/dracula-pc-enhanced.ini");
    let theme = Theme::from_mc_skin(&skin).expect("bundled skin should parse");

    assert!(!theme.name.is_empty());
}
