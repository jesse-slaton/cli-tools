mod app;
mod backup;
mod path_analyzer;
mod permissions;
mod registry;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use app::App;
use ui::UI;

fn main() -> Result<()> {
    // Check if running on Windows
    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("This application only runs on Windows.");
        std::process::exit(1);
    }

    // Check admin rights and notify user
    let is_admin = permissions::is_admin();
    if !is_admin {
        println!("Warning: Running without administrator privileges.");
        println!("You can modify USER paths, but MACHINE paths will be read-only.");
        println!("Press any key to continue...");
        event::read()?;
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // Small delay to ensure terminal is fully ready on Windows
    std::thread::sleep(std::time::Duration::from_millis(50));

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new()?;
    let mut ui = UI::new();

    // Main loop
    let result = run_app(&mut terminal, &mut app, &mut ui);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    ui: &mut UI,
) -> Result<()> {
    // Initial render to show the UI immediately
    terminal.draw(|f| ui.render(f, app))?;

    // Flush any pending keyboard events from application launch
    // Use a polling window to catch Enter key delayed by Windows/MSYS2 console buffering
    let flush_deadline = std::time::Instant::now() + std::time::Duration::from_millis(150);
    while std::time::Instant::now() < flush_deadline {
        if event::poll(std::time::Duration::from_millis(10))? {
            event::read()?;
        }
    }

    loop {
        terminal.draw(|f| ui.render(f, app))?;

        if let Event::Key(key) = event::read()? {
            // Global shortcuts
            match (key.code, key.modifiers) {
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                (KeyCode::Char('q'), KeyModifiers::NONE) => {
                    if app.confirm_exit() {
                        break;
                    }
                }
                (KeyCode::F(10), _) | (KeyCode::Esc, _) => {
                    if app.confirm_exit() {
                        break;
                    }
                }
                _ => {
                    // Handle input in app
                    app.handle_input(key)?;
                }
            }
        }
    }

    Ok(())
}
