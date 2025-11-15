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

    // Event deduplication state to filter duplicate events from Windows/MSYS2 buffering
    let mut last_event_time = std::time::Instant::now();
    let mut last_key_code: Option<KeyCode> = None;

    loop {
        terminal.draw(|f| ui.render(f, app))?;

        // Check if app wants to exit
        if app.should_exit {
            break;
        }

        match event::read()? {
            Event::Key(key) => {
                let now = std::time::Instant::now();
                let elapsed = now.duration_since(last_event_time);

                // Filter duplicate events within 200ms window (Windows/MSYS2 console buffering)
                // Conservative threshold to catch all buffering delays while staying well below
                // human key repeat timing (typically 250-500ms)
                let is_duplicate = last_key_code == Some(key.code)
                    && elapsed < std::time::Duration::from_millis(200);

                if !is_duplicate {
                    last_key_code = Some(key.code);
                    last_event_time = now;

                    // Global shortcuts
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                        (KeyCode::Char('q'), KeyModifiers::NONE) => {
                            // Only handle 'q' in Normal mode, otherwise let app handle it
                            if matches!(app.mode, app::Mode::Normal) {
                                app.confirm_exit();
                            } else {
                                app.handle_input(key)?;
                            }
                        }
                        (KeyCode::F(10), _) | (KeyCode::Esc, _) => {
                            // Only handle ESC/F10 as quit in Normal mode
                            if matches!(app.mode, app::Mode::Normal) {
                                app.confirm_exit();
                            } else {
                                app.handle_input(key)?;
                            }
                        }
                        _ => {
                            // Handle input in app
                            app.handle_input(key)?;
                        }
                    }
                }
                // Duplicate events are silently dropped
            }
            Event::Mouse(mouse) => {
                // Handle mouse events (clicks, scrolling)
                let size = terminal.size()?;
                let rect = ratatui::layout::Rect::new(0, 0, size.width, size.height);
                app.handle_mouse(mouse, rect)?;
            }
            _ => {}
        }
    }

    Ok(())
}
