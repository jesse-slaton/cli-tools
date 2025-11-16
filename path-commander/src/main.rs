mod app;
mod backup;
mod config;
mod path_analyzer;
mod permissions;
mod process_detector;
mod registry;
mod theme;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;

use app::App;
use theme::Theme;
use ui::UI;

/// Path Commander - Windows PATH Environment Manager
#[derive(Parser, Debug)]
#[command(name = "pc")]
#[command(about = "A TUI for managing Windows PATH environment variables", long_about = None)]
struct Args {
    /// Theme to use (built-in: default, dracula) or path to .ini skin file
    #[arg(short, long)]
    theme: Option<String>,

    /// Connect to remote computer (hostname or IP address)
    #[arg(short, long)]
    remote: Option<String>,
}

fn main() -> Result<()> {
    // Check if running on Windows
    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("This application only runs on Windows.");
        std::process::exit(1);
    }

    // Parse command-line arguments
    let args = Args::parse();

    // Initialize config directories
    config::ensure_config_dirs()?;
    config::migrate_backups().ok(); // Don't fail if migration fails

    // Load theme
    let theme = if let Some(theme_name) = args.theme {
        // Check if it's a file path
        let path = PathBuf::from(&theme_name);
        if path.exists() {
            Theme::from_mc_skin(&path)?
        } else {
            // Try loading from custom themes directory
            if let Some(custom_path) = config::get_theme_path(&theme_name) {
                Theme::from_mc_skin(&custom_path)?
            } else {
                // Try loading as built-in theme
                Theme::builtin(&theme_name)?
            }
        }
    } else {
        Theme::default()
    };

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

    // Create app state (with remote connection if specified)
    let mut app = if let Some(remote) = args.remote {
        match App::new_with_remote(theme, &remote) {
            Ok(app) => app,
            Err(e) => {
                // Restore terminal before showing error
                disable_raw_mode()?;
                execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
                eprintln!("Failed to connect to remote computer '{}': {:?}", remote, e);
                std::process::exit(1);
            }
        }
    } else {
        App::new(theme)?
    };
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

        // Update viewport height for PGUP/PGDOWN navigation
        let terminal_height = terminal.size()?.height;
        app.update_viewport_height(terminal_height);

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
