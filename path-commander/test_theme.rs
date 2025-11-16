use std::path::Path;
mod theme;
use theme::Theme;

fn main() {
    let theme = Theme::from_mc_skin(Path::new("/c/Users/jesse/.pc/themes/dracula.ini")).unwrap();
    
    println!("Theme name: {}", theme.name);
    println!("Panel border fg: {:?}", theme.panel_border_fg);
    println!("Scrollbar fg: {:?}", theme.scrollbar_fg);
    println!("Scrollbar thumb fg: {:?}", theme.scrollbar_thumb_fg);
    println!("Path valid fg: {:?}", theme.path_valid_fg);
    println!("Path dead fg: {:?}", theme.path_dead_fg);
    println!("Path duplicate fg: {:?}", theme.path_duplicate_fg);
}
