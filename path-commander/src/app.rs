use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{layout::Rect, widgets::ScrollbarState};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

use crate::backup::{self, PathBackup};
use crate::path_analyzer::{
    analyze_paths, analyze_paths_with_remote, normalize_path, to_unc_path, PathInfo,
};
use crate::permissions;
use crate::registry::{self, PathScope, RemoteConnection};
use crate::theme::Theme;

/// Represents the connection mode of the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionMode {
    /// Local mode: Machine (left) and User (right) panels
    Local,
    /// Remote mode: Local Machine (left) and Remote Machine (right) panels
    Remote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Panel {
    Machine,
    User,
}

impl Panel {
    pub fn toggle(&self) -> Self {
        match self {
            Panel::Machine => Panel::User,
            Panel::User => Panel::Machine,
        }
    }

    pub fn scope(&self) -> PathScope {
        match self {
            Panel::Machine => PathScope::Machine,
            Panel::User => PathScope::User,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Help,
    About,
    Confirm(ConfirmAction),
    Input(InputMode),
    BackupList,
    ProcessRestartInfo,
    FilterMenu,
    ThemeSelection,
    FileBrowser,
    Menu {
        active_menu: usize,
        selected_item: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmAction {
    Exit,
    DeleteSelected,
    DeleteAllDead,
    DeleteAllDuplicates,
    ApplyChanges,
    RequestElevation,
    RestoreBackup,
    CreateSingleDirectory,
    CreateMarkedDirectories,
    DisconnectRemote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    AddPath,
    EditPath,
    ConnectRemote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterMode {
    None,
    Dead,
    Duplicates,
    NonNormalized,
    Valid,
}

/// Represents a directory entry in the file browser
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: PathBuf,
    #[allow(dead_code)]
    pub is_parent: bool, // True for ".." entry (kept for future use)
    pub is_drive: bool, // True for drive letter entries (C:\, D:\, etc.)
}

/// Represents an undoable operation with enough data to reverse it
#[derive(Debug, Clone)]
pub enum Operation {
    /// Delete operations - stores deleted items with their original indices
    DeletePaths {
        panel: Panel,
        deleted: Vec<(usize, String)>, // (index, path) pairs in original order
    },
    /// Add operation - stores panel and index where path was added
    AddPath {
        panel: Panel,
        index: usize,
        path: String,
    },
    /// Edit operation - stores old and new values
    EditPath {
        panel: Panel,
        index: usize,
        old_path: String,
        new_path: String,
    },
    /// Swap operation (move up/down)
    SwapPaths {
        panel: Panel,
        index1: usize,
        index2: usize,
    },
    /// Move paths from one panel to another
    MovePaths {
        from_panel: Panel,
        #[allow(dead_code)]
        to_panel: Panel,
        paths_with_indices: Vec<(usize, String)>, // Original indices in from_panel
    },
    /// Copy paths from one panel to another (used in remote mode)
    CopyPaths {
        #[allow(dead_code)]
        from_panel: Panel,
        to_panel: Panel,
        paths_with_indices: Vec<(usize, String)>, // Paths that were copied to to_panel
    },
    /// Normalize paths - stores changes made
    NormalizePaths {
        panel: Panel,
        changes: Vec<(usize, String, String)>, // (index, old_path, new_path)
    },
}

pub struct App {
    pub connection_mode: ConnectionMode, // Local or Remote mode
    pub remote_connection: Option<RemoteConnection>, // Remote connection if in Remote mode
    pub machine_paths: Vec<String>,
    pub user_paths: Vec<String>,
    pub machine_info: Vec<PathInfo>,
    pub user_info: Vec<PathInfo>,
    pub machine_original: Vec<String>,
    pub user_original: Vec<String>,
    // Remote machine paths (used when in Remote mode for the right panel)
    pub remote_machine_paths: Vec<String>,
    pub remote_machine_info: Vec<PathInfo>,
    pub remote_machine_original: Vec<String>,
    pub remote_machine_selected: usize,
    pub remote_machine_marked: HashSet<usize>,
    pub remote_scrollbar_state: ScrollbarState,
    pub active_panel: Panel,
    pub machine_selected: usize,
    pub user_selected: usize,
    pub machine_marked: HashSet<usize>,
    pub user_marked: HashSet<usize>,
    pub mode: Mode,
    pub is_admin: bool,
    pub has_changes: bool,
    pub status_message: String,
    pub input_buffer: String,
    pub backup_list: Vec<std::path::PathBuf>,
    pub backup_selected: usize,
    pub machine_scrollbar_state: ScrollbarState,
    pub user_scrollbar_state: ScrollbarState,
    pub should_exit: bool,
    pub viewport_height: u16,
    pub pending_directory: String, // Temporarily stores path for directory creation confirmation
    pub processes_to_restart: Vec<String>, // List of processes that need restarting to pick up PATH changes
    pub theme: Theme,                      // Color theme for UI rendering
    pub theme_arg: Option<String>, // Original theme argument from command line (for elevation)
    pub filter_mode: FilterMode,   // Current filter mode (None, Dead, Duplicates, etc.)
    pub filter_menu_selected: usize, // Selected item in filter menu (0-4)
    pub theme_list: Vec<(String, bool)>, // List of available themes (name, is_builtin)
    pub theme_selected: usize,     // Selected theme in the theme selector
    pub original_theme: Option<Theme>, // Theme before opening theme selector (for Esc cancellation)
    pub undo_stack: Vec<Operation>, // Stack of undoable operations
    pub redo_stack: Vec<Operation>, // Stack of redoable operations
    last_click_time: std::time::Instant, // Time of last mouse click for double-click detection
    last_click_pos: (Panel, usize), // Panel and row of last click
    mode_enter_time: std::time::Instant, // Time when current mode was entered (for buffering protection)
    // File browser state
    pub file_browser_current_path: PathBuf, // Current directory being browsed
    pub file_browser_entries: Vec<DirectoryEntry>, // Directory entries in current path
    pub file_browser_selected: usize,       // Selected entry index
    pub file_browser_scrollbar_state: ScrollbarState, // Scrollbar state for file browser
}

impl App {
    pub fn new(theme: Theme, theme_arg: Option<String>) -> Result<Self> {
        let is_admin = permissions::is_admin();

        // Read paths from registry
        let user_path_string = registry::read_path(PathScope::User)?;
        let machine_path_string = registry::read_path(PathScope::Machine)?;

        let user_paths = registry::parse_path(&user_path_string);
        let machine_paths = registry::parse_path(&machine_path_string);

        // Analyze paths
        let user_info = analyze_paths(&user_paths, &machine_paths);
        let machine_info = analyze_paths(&machine_paths, &user_paths);

        Ok(Self {
            connection_mode: ConnectionMode::Local,
            remote_connection: None,
            machine_scrollbar_state: ScrollbarState::new(machine_paths.len()).position(0),
            user_scrollbar_state: ScrollbarState::new(user_paths.len()).position(0),
            remote_scrollbar_state: ScrollbarState::new(0).position(0),
            machine_paths: machine_paths.clone(),
            user_paths: user_paths.clone(),
            machine_info,
            user_info,
            machine_original: machine_paths,
            user_original: user_paths,
            remote_machine_paths: Vec::new(),
            remote_machine_info: Vec::new(),
            remote_machine_original: Vec::new(),
            remote_machine_selected: 0,
            remote_machine_marked: HashSet::new(),
            active_panel: Panel::Machine,
            machine_selected: 0,
            user_selected: 0,
            machine_marked: HashSet::new(),
            user_marked: HashSet::new(),
            mode: Mode::Normal,
            is_admin,
            has_changes: false,
            status_message: permissions::get_privilege_message(),
            input_buffer: String::new(),
            backup_list: Vec::new(),
            backup_selected: 0,
            should_exit: false,
            viewport_height: 10, // Default, will be updated based on terminal size
            pending_directory: String::new(),
            processes_to_restart: Vec::new(),
            theme,
            theme_arg,
            filter_mode: FilterMode::None,
            filter_menu_selected: 0,
            theme_list: Vec::new(), // Will be populated when theme selector is opened
            theme_selected: 0,
            original_theme: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_click_time: std::time::Instant::now(),
            last_click_pos: (Panel::Machine, 0),
            mode_enter_time: std::time::Instant::now(),
            file_browser_current_path: std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("C:\\")),
            file_browser_entries: Vec::new(),
            file_browser_selected: 0,
            file_browser_scrollbar_state: ScrollbarState::new(0).position(0),
        })
    }

    /// Create a new App with a remote connection
    pub fn new_with_remote(
        theme: Theme,
        theme_arg: Option<String>,
        remote_computer: &str,
    ) -> Result<Self> {
        let mut app = Self::new(theme, theme_arg)?;
        app.connect_to_remote(remote_computer)?;
        Ok(app)
    }

    /// Restore App from an elevation state (after UAC elevation)
    pub fn from_elevation_state(
        theme: Theme,
        state: crate::elevation::ElevationState,
    ) -> Result<Self> {
        // Create a new app with the theme
        let mut app = Self::new(theme, state.theme_arg.clone())?;

        // Restore all state from elevation
        app.connection_mode = state.connection_mode;
        app.machine_paths = state.machine_paths;
        app.user_paths = state.user_paths;
        app.remote_machine_paths = state.remote_machine_paths;
        app.active_panel = state.active_panel;
        app.machine_selected = state.machine_selected;
        app.user_selected = state.user_selected;
        app.remote_machine_selected = state.remote_machine_selected;
        app.machine_marked = state.machine_marked;
        app.user_marked = state.user_marked;
        app.remote_machine_marked = state.remote_machine_marked;
        app.filter_mode = state.filter_mode;
        app.input_buffer = state.input_buffer;
        app.pending_directory = state.pending_directory;

        // Restore remote connection if in remote mode
        if app.connection_mode == ConnectionMode::Remote {
            if let Some(ref computer_name) = state.remote_computer_name {
                app.connect_to_remote(computer_name)?;
            }
        }

        // Mark that we have changes (since we restored edited state)
        app.has_changes =
            app.machine_paths != app.machine_original || app.user_paths != app.user_original;

        // Reanalyze paths
        app.reanalyze();

        // Update status message
        app.set_status("Elevated successfully! You can now modify MACHINE paths.");

        Ok(app)
    }

    /// Connect to a remote computer
    pub fn connect_to_remote(&mut self, computer_name: &str) -> Result<()> {
        // Establish remote connection
        let connection = RemoteConnection::connect(computer_name)?;

        // Read remote MACHINE paths
        let remote_path_string = registry::read_path_remote(PathScope::Machine, &connection)?;
        let remote_paths = registry::parse_path(&remote_path_string);

        // Analyze remote paths (compare with local machine paths for cross-scope duplicates)
        let remote_info = analyze_paths(&remote_paths, &self.machine_paths);

        // Update app state to remote mode
        self.connection_mode = ConnectionMode::Remote;
        self.remote_machine_paths = remote_paths.clone();
        self.remote_machine_info = remote_info;
        self.remote_machine_original = remote_paths.clone();
        self.remote_machine_selected = 0;
        self.remote_machine_marked = HashSet::new();
        self.remote_scrollbar_state = ScrollbarState::new(remote_paths.len()).position(0);
        self.remote_connection = Some(connection);

        // Update status message
        self.status_message = format!(
            "Connected to remote computer: {} | {}",
            computer_name,
            permissions::get_privilege_message()
        );

        Ok(())
    }

    /// Disconnect from remote computer and return to local mode
    pub fn disconnect_from_remote(&mut self) -> Result<()> {
        if self.connection_mode == ConnectionMode::Local {
            return Ok(());
        }

        // Clear remote connection and data
        self.connection_mode = ConnectionMode::Local;
        self.remote_connection = None;
        self.remote_machine_paths.clear();
        self.remote_machine_info.clear();
        self.remote_machine_original.clear();
        self.remote_machine_selected = 0;
        self.remote_machine_marked.clear();
        self.remote_scrollbar_state = ScrollbarState::new(0).position(0);

        // Switch back to Machine panel if on User panel (which was showing remote)
        if self.active_panel == Panel::User {
            self.active_panel = Panel::Machine;
        }

        // Update status message
        self.status_message = permissions::get_privilege_message();

        Ok(())
    }

    /// Update viewport height based on terminal size
    /// Calculates visible lines in panel: terminal_height - menu(1) - header(1) - status(3) - hints(2) - borders(2)
    pub fn update_viewport_height(&mut self, terminal_height: u16) {
        // Layout: Menu(1) + Header(1) + Content + Status(3) + Hints(2)
        // Panel has top and bottom borders (2)
        // Viewport = terminal_height - 1 - 1 - 3 - 2 - 2 = terminal_height - 9
        self.viewport_height = terminal_height.saturating_sub(9).max(1);
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Result<()> {
        match self.mode {
            Mode::Normal => self.handle_normal_input(key),
            Mode::Help => self.handle_help_input(key),
            Mode::About => self.handle_about_input(key),
            Mode::Confirm(action) => self.handle_confirm_input(key, action),
            Mode::Input(input_mode) => self.handle_input_mode(key, input_mode),
            Mode::BackupList => self.handle_backup_list_input(key),
            Mode::ProcessRestartInfo => self.handle_process_restart_info_input(key),
            Mode::FileBrowser => self.handle_file_browser_input(key),
            Mode::FilterMenu => self.handle_filter_menu_input(key),
            Mode::ThemeSelection => self.handle_theme_selection_input(key),
            Mode::Menu {
                active_menu,
                selected_item,
            } => self.handle_menu_input(key, active_menu, selected_item),
        }
    }

    fn handle_normal_input(&mut self, key: KeyEvent) -> Result<()> {
        match (key.code, key.modifiers) {
            // Navigation
            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => self.move_selection(-1),
            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => self.move_selection(1),
            (KeyCode::PageUp, _) => {
                // Jump by viewport height minus 1 for context (like vim Ctrl+B)
                let jump = (self.viewport_height.saturating_sub(1).max(1)) as i32;
                self.move_selection(-jump)
            }
            (KeyCode::PageDown, _) => {
                // Jump by viewport height minus 1 for context (like vim Ctrl+F)
                let jump = (self.viewport_height.saturating_sub(1).max(1)) as i32;
                self.move_selection(jump)
            }
            (KeyCode::Home, _) => self.move_selection_to(0),
            (KeyCode::End, _) => self.move_selection_to(usize::MAX),
            (KeyCode::Tab, _) | (KeyCode::Left, _) | (KeyCode::Right, _) => {
                self.active_panel = self.active_panel.toggle();
            }

            // Selection
            (KeyCode::Char(' '), _) | (KeyCode::Insert, _) => self.toggle_mark(),

            // Actions
            (KeyCode::F(2), _) => self.toggle_mark(),
            (KeyCode::F(3), _) => {
                if self.has_marked_items() {
                    self.mode = Mode::Confirm(ConfirmAction::DeleteSelected);
                }
            }
            (KeyCode::F(4), _) => self.start_add_path(),
            (KeyCode::F(5), _) => self.move_marked_to_other_panel()?,
            (KeyCode::F(6), _) => self.move_item_up(),
            (KeyCode::F(7), _) => {
                self.mode = Mode::Confirm(ConfirmAction::DeleteAllDuplicates);
            }
            (KeyCode::F(8), _) => {
                self.mode = Mode::Confirm(ConfirmAction::DeleteAllDead);
            }
            (KeyCode::F(9), _) => self.normalize_selected(),
            (KeyCode::F(10), _) => {
                // Create marked dead directories
                if self.has_marked_dead_paths() {
                    self.mode = Mode::Confirm(ConfirmAction::CreateMarkedDirectories);
                } else {
                    self.set_status("No marked dead paths to create");
                }
            }
            (KeyCode::Char('/'), _) => {
                // Open filter menu
                self.mode = Mode::FilterMenu;
                self.filter_menu_selected = 0;
            }
            (KeyCode::Char('t'), _) => {
                // Open theme selection menu
                self.open_theme_selector()?;
            }
            (KeyCode::F(1), _) | (KeyCode::Char('?'), _) => {
                self.mode = Mode::Help;
            }

            // Edit
            (KeyCode::Enter, _) => {
                // Only allow editing if the current panel has paths
                let has_paths = match self.active_panel {
                    Panel::Machine => !self.machine_paths.is_empty(),
                    Panel::User => !self.user_paths.is_empty(),
                };
                if has_paths {
                    self.start_edit_path();
                }
            }
            (KeyCode::Delete, _) => {
                if self.has_marked_items() {
                    self.mode = Mode::Confirm(ConfirmAction::DeleteSelected);
                }
            }

            // Save/Restore
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                if self.has_changes {
                    // Check if we need elevation for MACHINE path changes
                    let needs_elevation = crate::elevation::needs_elevation_for_changes(
                        self.is_admin,
                        &self.machine_paths,
                        &self.machine_original,
                        &self.remote_machine_paths,
                        &self.remote_machine_original,
                        self.connection_mode,
                    );

                    if needs_elevation {
                        self.mode = Mode::Confirm(ConfirmAction::RequestElevation);
                    } else {
                        self.mode = Mode::Confirm(ConfirmAction::ApplyChanges);
                    }
                } else {
                    self.set_status("No changes to save");
                }
            }
            (KeyCode::Char('b'), KeyModifiers::CONTROL) => self.create_backup()?,
            (KeyCode::Char('o'), KeyModifiers::CONTROL) => {
                // Connect to or disconnect from remote computer
                match self.connection_mode {
                    ConnectionMode::Local => {
                        // Open connection dialog
                        self.mode = Mode::Input(InputMode::ConnectRemote);
                        self.input_buffer.clear();
                    }
                    ConnectionMode::Remote => {
                        // Confirm disconnect
                        self.mode = Mode::Confirm(ConfirmAction::DisconnectRemote);
                    }
                }
            }

            // Undo/Redo
            (KeyCode::Char('z'), KeyModifiers::CONTROL) => self.undo()?,
            (KeyCode::Char('y'), KeyModifiers::CONTROL) => self.redo()?,
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => self.show_backup_list()?,

            // Bulk selection commands
            (KeyCode::Char('A'), KeyModifiers::CONTROL) => {
                // Shift+Ctrl+A (uppercase A means shift is pressed)
                self.mark_all_both_scopes();
            }
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                // Ctrl+A only
                self.mark_all_visible();
            }
            (KeyCode::Char('D'), KeyModifiers::CONTROL) => {
                // Shift+Ctrl+D (uppercase D means shift is pressed)
                self.mark_all_dead();
            }
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                // Ctrl+D only
                self.mark_all_duplicates();
            }
            (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                self.mark_all_non_normalized();
            }
            (KeyCode::Char('U'), KeyModifiers::CONTROL) => {
                // Shift+Ctrl+U (uppercase U means shift is pressed)
                self.unmark_all();
            }

            (KeyCode::Char('e'), KeyModifiers::CONTROL)
            | (KeyCode::Char('E'), KeyModifiers::CONTROL) => {
                // Request elevation with Ctrl+E
                if !self.is_admin {
                    self.mode = Mode::Confirm(ConfirmAction::RequestElevation);
                } else {
                    self.set_status("Already running as administrator");
                }
            }

            // Menu activation with Alt+letter
            (KeyCode::Char(c), KeyModifiers::ALT) => {
                self.activate_menu_by_char(c);
            }

            _ => {}
        }
        Ok(())
    }

    fn handle_help_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::F(1) | KeyCode::Char('q') => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_about_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_process_restart_info_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                self.mode = Mode::Normal;
                self.set_status("Changes applied successfully!");
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_confirm_input(&mut self, key: KeyEvent, action: ConfirmAction) -> Result<()> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                // For Exit action, require F10 instead of y/Enter
                if matches!(action, ConfirmAction::Exit) {
                    return Ok(());
                }
                self.mode = Mode::Normal;
                match action {
                    ConfirmAction::Exit => {
                        self.should_exit = true;
                    }
                    ConfirmAction::DeleteSelected => self.delete_marked()?,
                    ConfirmAction::DeleteAllDead => self.delete_all_dead()?,
                    ConfirmAction::DeleteAllDuplicates => self.delete_all_duplicates()?,
                    ConfirmAction::ApplyChanges => self.apply_changes()?,
                    ConfirmAction::RequestElevation => {
                        // Request UAC elevation and restart with elevated privileges
                        self.request_elevation()?;
                    }
                    ConfirmAction::RestoreBackup => self.restore_selected_backup()?,
                    ConfirmAction::CreateSingleDirectory => self.create_single_directory()?,
                    ConfirmAction::CreateMarkedDirectories => self.create_marked_directories()?,
                    ConfirmAction::DisconnectRemote => {
                        self.disconnect_from_remote()?;
                        self.set_status("Disconnected from remote computer");
                    }
                }
            }
            KeyCode::F(10) => {
                // F10 in Exit confirmation dialog = confirm exit
                if matches!(action, ConfirmAction::Exit) {
                    self.mode = Mode::Normal;
                    self.should_exit = true;
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_input_mode(&mut self, key: KeyEvent, input_mode: InputMode) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                // Prevent buffered ENTER keys from immediately confirming (100ms grace period)
                let elapsed = std::time::Instant::now().duration_since(self.mode_enter_time);
                if elapsed < std::time::Duration::from_millis(100) {
                    return Ok(());
                }

                self.mode = Mode::Normal;
                self.mode_enter_time = std::time::Instant::now();
                match input_mode {
                    InputMode::AddPath => self.add_path_from_input()?,
                    InputMode::EditPath => self.update_path_from_input()?,
                    InputMode::ConnectRemote => {
                        let computer_name = self.input_buffer.trim().to_string();
                        if !computer_name.is_empty() {
                            match self.connect_to_remote(&computer_name) {
                                Ok(()) => {
                                    self.set_status(&format!(
                                        "Successfully connected to {}",
                                        computer_name
                                    ));
                                }
                                Err(e) => {
                                    self.set_status(&format!(
                                        "Failed to connect to {}: {}",
                                        computer_name, e
                                    ));
                                }
                            }
                        }
                    }
                }
                self.input_buffer.clear();
            }
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.mode_enter_time = std::time::Instant::now();
                self.input_buffer.clear();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_backup_list_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.backup_selected > 0 {
                    self.backup_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.backup_selected + 1 < self.backup_list.len() {
                    self.backup_selected += 1;
                }
            }
            KeyCode::Enter => {
                if !self.backup_list.is_empty() {
                    self.mode = Mode::Confirm(ConfirmAction::RestoreBackup);
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_file_browser_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.file_browser_selected > 0 {
                    self.file_browser_selected -= 1;
                    self.file_browser_scrollbar_state = self
                        .file_browser_scrollbar_state
                        .position(self.file_browser_selected);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.file_browser_selected + 1 < self.file_browser_entries.len() {
                    self.file_browser_selected += 1;
                    self.file_browser_scrollbar_state = self
                        .file_browser_scrollbar_state
                        .position(self.file_browser_selected);
                }
            }
            KeyCode::PageUp => {
                let jump = self.viewport_height.saturating_sub(1).max(1) as usize;
                self.file_browser_selected = self.file_browser_selected.saturating_sub(jump);
                self.file_browser_scrollbar_state = self
                    .file_browser_scrollbar_state
                    .position(self.file_browser_selected);
            }
            KeyCode::PageDown => {
                let jump = self.viewport_height.saturating_sub(1).max(1) as usize;
                let max_idx = self.file_browser_entries.len().saturating_sub(1);
                self.file_browser_selected = (self.file_browser_selected + jump).min(max_idx);
                self.file_browser_scrollbar_state = self
                    .file_browser_scrollbar_state
                    .position(self.file_browser_selected);
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.file_browser_selected = 0;
                self.file_browser_scrollbar_state = self
                    .file_browser_scrollbar_state
                    .position(self.file_browser_selected);
            }
            KeyCode::End | KeyCode::Char('G') => {
                if !self.file_browser_entries.is_empty() {
                    self.file_browser_selected = self.file_browser_entries.len() - 1;
                    self.file_browser_scrollbar_state = self
                        .file_browser_scrollbar_state
                        .position(self.file_browser_selected);
                }
            }
            KeyCode::Enter => {
                // If an entry is selected, navigate into it
                if !self.file_browser_entries.is_empty() {
                    self.navigate_to_selected_directory();
                }
            }
            KeyCode::Char(' ') => {
                // Space key: select current directory for adding to PATH
                self.select_current_directory_for_path()?;
            }
            KeyCode::Tab => {
                // Tab: switch to manual text input mode
                self.mode = Mode::Input(InputMode::AddPath);
                self.mode_enter_time = std::time::Instant::now();
                self.input_buffer = self.file_browser_current_path.to_string_lossy().to_string();
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                // Cancel file browser
                self.mode = Mode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_filter_menu_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.filter_menu_selected > 0 {
                    self.filter_menu_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                // We have 5 filter options (0-4): None, Dead, Duplicates, NonNormalized, Valid
                if self.filter_menu_selected < 4 {
                    self.filter_menu_selected += 1;
                }
            }
            KeyCode::Enter => {
                // Apply selected filter
                let new_filter = match self.filter_menu_selected {
                    0 => FilterMode::None,
                    1 => FilterMode::Dead,
                    2 => FilterMode::Duplicates,
                    3 => FilterMode::NonNormalized,
                    4 => FilterMode::Valid,
                    _ => FilterMode::None,
                };

                // Set the filter mode directly (don't toggle)
                self.filter_mode = new_filter;
                let filter_name = match new_filter {
                    FilterMode::None => "None (showing all)",
                    FilterMode::Dead => "Dead paths",
                    FilterMode::Duplicates => "Duplicates",
                    FilterMode::NonNormalized => "Non-normalized",
                    FilterMode::Valid => "Valid paths",
                };
                self.set_status(&format!("Filter: {}", filter_name));
                self.mode = Mode::Normal;
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('/') => {
                // Close menu without changing filter
                self.mode = Mode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn open_theme_selector(&mut self) -> Result<()> {
        // Load available themes
        self.theme_list = crate::config::list_available_themes()?;

        // Find current theme in list and select it
        let current_name = &self.theme.name;
        self.theme_selected = self
            .theme_list
            .iter()
            .position(|(name, _)| name == current_name)
            .unwrap_or(0);

        // Store original theme for Esc cancellation
        self.original_theme = Some(self.theme.clone());

        self.mode = Mode::ThemeSelection;
        Ok(())
    }

    /// Load and apply the currently selected theme (for live preview)
    fn apply_selected_theme(&mut self) -> Result<()> {
        if let Some((theme_name, is_builtin)) = self.theme_list.get(self.theme_selected) {
            let new_theme = if *is_builtin {
                Theme::builtin(theme_name)?
            } else {
                // Load from custom theme file
                if let Some(theme_path) = crate::config::get_theme_path(theme_name) {
                    Theme::from_mc_skin(&theme_path)?
                } else {
                    self.set_status(&format!("Theme file not found: {}", theme_name));
                    return Ok(());
                }
            };

            self.theme = new_theme;
        }
        Ok(())
    }

    fn handle_theme_selection_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.theme_selected > 0 {
                    self.theme_selected -= 1;
                    // Apply theme immediately for live preview
                    self.apply_selected_theme()?;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.theme_selected < self.theme_list.len().saturating_sub(1) {
                    self.theme_selected += 1;
                    // Apply theme immediately for live preview
                    self.apply_selected_theme()?;
                }
            }
            KeyCode::Enter => {
                // Keep the currently selected theme and close
                if let Some((theme_name, _)) = self.theme_list.get(self.theme_selected) {
                    self.set_status(&format!("Theme changed to: {}", theme_name));
                }
                // Clear original theme since we're accepting the change
                self.original_theme = None;
                self.mode = Mode::Normal;
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('t') => {
                // Restore original theme and close
                if let Some(original) = self.original_theme.take() {
                    self.theme = original;
                }
                self.mode = Mode::Normal;
            }
            KeyCode::Char('r') => {
                // Reload theme list
                self.theme_list = crate::config::list_available_themes()?;
                self.set_status("Theme list reloaded");
            }
            _ => {}
        }
        Ok(())
    }

    // Mouse event handling
    pub fn handle_mouse(&mut self, mouse: MouseEvent, terminal_size: Rect) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Handle clicks based on current mode
                match self.mode {
                    Mode::Confirm(_) => {
                        self.handle_confirm_dialog_click(mouse.column, mouse.row, terminal_size)?;
                    }
                    Mode::Normal => {
                        // Check if click is on menu bar (top row)
                        if mouse.row == 0 {
                            self.handle_menu_bar_click(mouse.column)?;
                        }
                        // Check if click is on key hints area (bottom 2 rows)
                        else if mouse.row >= terminal_size.height.saturating_sub(2) {
                            self.handle_hints_click(mouse.column, terminal_size.width)?;
                        } else {
                            self.handle_mouse_click(
                                mouse.column,
                                mouse.row,
                                terminal_size,
                                mouse.modifiers,
                            )?;
                        }
                    }
                    Mode::Menu {
                        active_menu,
                        selected_item,
                    } => {
                        self.handle_menu_click(
                            mouse.column,
                            mouse.row,
                            active_menu,
                            selected_item,
                        )?;
                    }
                    _ => {}
                }
            }
            MouseEventKind::ScrollUp => {
                // Only scroll in Normal mode
                if self.mode == Mode::Normal {
                    self.move_selection(1);
                }
            }
            MouseEventKind::ScrollDown => {
                // Only scroll in Normal mode
                if self.mode == Mode::Normal {
                    self.move_selection(-1);
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_mouse_click(
        &mut self,
        x: u16,
        y: u16,
        terminal_size: Rect,
        modifiers: KeyModifiers,
    ) -> Result<()> {
        // Calculate layout (same as ui.rs render_main)
        let header_height = 3;
        let status_height = 3;
        let hints_height = 2;

        // Main content area
        let content_start = header_height;
        let content_end = terminal_size.height - status_height - hints_height;

        // Check if click is in main content area
        if y < content_start || y >= content_end {
            return Ok(());
        }

        // Determine which panel was clicked (50/50 split)
        let panel_width = terminal_size.width / 2;
        let clicked_panel = if x < panel_width {
            Panel::Machine
        } else {
            Panel::User
        };

        // Get panel-specific coordinates
        let panel_x_offset = if clicked_panel == Panel::Machine {
            0
        } else {
            panel_width
        };
        let relative_x = x - panel_x_offset;
        let relative_y = y - content_start;

        // Check if click is on scrollbar (second-to-last column, before right border)
        let scrollbar_x = panel_width - 2;
        if relative_x == scrollbar_x {
            // Clicked on scrollbar - jump to position
            let border_top: u16 = 1;
            let border_bottom: u16 = 1;
            let content_height = content_end - content_start;
            let scrollbar_height =
                content_height.saturating_sub(border_top + border_bottom) as usize;

            if scrollbar_height > 0 {
                let click_pos = relative_y.saturating_sub(border_top) as usize;
                let paths = match clicked_panel {
                    Panel::Machine => &self.machine_paths,
                    Panel::User => &self.user_paths,
                };

                if !paths.is_empty() && click_pos < scrollbar_height {
                    // Calculate target position as percentage
                    let target_pos = (click_pos * paths.len()) / scrollbar_height;
                    let target_pos = target_pos.min(paths.len().saturating_sub(1));

                    self.active_panel = clicked_panel;
                    self.move_selection_to(target_pos);
                }
            }
            return Ok(());
        }

        // Check if click is on border (first or last column of panel)
        if relative_x == 0 || relative_x >= panel_width - 1 {
            // Clicked on panel border - switch to this panel
            self.active_panel = clicked_panel;
            return Ok(());
        }

        // Click is inside panel content
        // Account for: top border (1) + title (0, included in border)
        let border_top = 1;

        // Check if click is on border row
        if relative_y == 0 {
            // Clicked on top border/title - switch to this panel
            self.active_panel = clicked_panel;
            return Ok(());
        }

        // Calculate list item index (0-based)
        // relative_y - border_top = row in list
        let list_row = relative_y.saturating_sub(border_top) as usize;

        // Get current panel's paths
        let paths = match clicked_panel {
            Panel::Machine => &self.machine_paths,
            Panel::User => &self.user_paths,
        };

        // Check if list_row is valid
        if list_row >= paths.len() {
            // Clicked below last item - just switch panel
            self.active_panel = clicked_panel;
            return Ok(());
        }

        // Handle Ctrl+Click: Toggle mark on clicked item without changing selection
        if modifiers.contains(KeyModifiers::CONTROL) {
            match clicked_panel {
                Panel::Machine => {
                    if self.machine_marked.contains(&list_row) {
                        self.machine_marked.remove(&list_row);
                    } else {
                        self.machine_marked.insert(list_row);
                    }
                }
                Panel::User => {
                    if self.user_marked.contains(&list_row) {
                        self.user_marked.remove(&list_row);
                    } else {
                        self.user_marked.insert(list_row);
                    }
                }
            }
            return Ok(());
        }

        // Handle Shift+Click: Range select from current selection to clicked item
        if modifiers.contains(KeyModifiers::SHIFT) {
            let current_selection = match clicked_panel {
                Panel::Machine => self.machine_selected,
                Panel::User => self.user_selected,
            };

            let start = current_selection.min(list_row);
            let end = current_selection.max(list_row);

            match clicked_panel {
                Panel::Machine => {
                    for i in start..=end {
                        self.machine_marked.insert(i);
                    }
                }
                Panel::User => {
                    for i in start..=end {
                        self.user_marked.insert(i);
                    }
                }
            }

            self.active_panel = clicked_panel;
            self.move_selection_to(list_row);
            return Ok(());
        }

        // Normal click: Switch to this panel and select the item
        self.active_panel = clicked_panel;
        self.move_selection_to(list_row);

        // Define checkbox area bounds (used for both double-click check and marking)
        // Checkbox is at relative_x = 1 (border) to 5 (border + "[ ] ")
        let checkbox_start = 1; // After left border
        let checkbox_end = 5; // "[ ] " is 4 chars

        // Check for double-click (two clicks on same item within 500ms)
        let now = std::time::Instant::now();
        let double_click_threshold = std::time::Duration::from_millis(500);
        let is_same_position = self.last_click_pos == (clicked_panel, list_row);
        let is_within_time = now.duration_since(self.last_click_time) < double_click_threshold;

        if is_same_position && is_within_time && relative_x >= checkbox_end {
            // Double-click detected outside checkbox area - edit the path
            self.start_edit_path();
            // Reset click tracking to prevent triple-click issues
            self.last_click_time = std::time::Instant::now() - double_click_threshold;
            return Ok(());
        }

        // Update click tracking for next potential double-click
        self.last_click_time = now;
        self.last_click_pos = (clicked_panel, list_row);

        // Check if click is on checkbox area

        if relative_x >= checkbox_start && relative_x < checkbox_end {
            // Clicked on checkbox - toggle mark (without auto-advance)
            match self.active_panel {
                Panel::Machine => {
                    if self.machine_marked.contains(&self.machine_selected) {
                        self.machine_marked.remove(&self.machine_selected);
                    } else {
                        self.machine_marked.insert(self.machine_selected);
                    }
                }
                Panel::User => {
                    if self.user_marked.contains(&self.user_selected) {
                        self.user_marked.remove(&self.user_selected);
                    } else {
                        self.user_marked.insert(self.user_selected);
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_hints_click(&mut self, x: u16, width: u16) -> Result<()> {
        // Key hints are centered, so calculate the starting position
        // MC-style format: "1Help | 2Mark | 3Del | 4Add | /Filter | Ctrl+SSave | QQuit"
        // Note: Actual hints vary by context (filter active, items marked, normal mode)
        // Approximate length for normal mode: ~80 characters (was 100 in old format)
        let hint_text_len = 80;
        let start_x = (width.saturating_sub(hint_text_len)) / 2;

        // Calculate relative position
        if x < start_x {
            return Ok(());
        }

        let relative_x = x - start_x;

        // Map click positions to keys (MC-style format):
        // "1" (1) + "Help" (4) + " | " (3) = 0-7
        // "2" (1) + "Mark" (4) + " | " (3) = 8-15
        // "3" (1) + "Del" (3) + " | " (3) = 16-22
        // "4" (1) + "Add" (3) + " | " (3) = 23-29
        // "/" (1) + "Filter" (6) + " | " (3) = 30-39
        // "Ctrl+Z" (6) + "Undo" (4) + " | " (3) = 40-52 (conditional)
        // "Ctrl+Y" (6) + "Redo" (4) + " | " (3) = 53-65 (conditional)
        // "Ctrl+S" (6) + "Save" (4) + " | " (3) = ~66-78
        // "Q" (1) + "Quit" (4) = ~79-83

        match relative_x {
            0..=7 => self.mode = Mode::Help, // 1Help
            8..=15 => self.toggle_mark(),    // 2Mark
            16..=22 => {
                // 3Del
                if self.has_marked_items() {
                    self.mode = Mode::Confirm(ConfirmAction::DeleteSelected);
                }
            }
            23..=29 => self.start_add_path(), // 4Add
            30..=39 => {
                // /Filter - toggle filter or something
                // This used to be F5 Move, but now it's /Filter
                // Adjust based on context
            }
            40..=90 => {
                // This region contains conditional items (Undo/Redo) and final items (Save/Quit)
                // Due to dynamic nature of hints, we'll handle Save and Quit conservatively
                // TODO: Make this more robust by calculating exact positions dynamically
                if relative_x >= 70 {
                    // Likely in the Save/Quit region
                    if relative_x >= 79 {
                        self.confirm_exit(); // QQuit
                    } else {
                        // Ctrl+SSave
                        if self.has_changes {
                            self.mode = Mode::Confirm(ConfirmAction::ApplyChanges);
                        } else {
                            self.set_status("No changes to save");
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_confirm_dialog_click(&mut self, x: u16, y: u16, terminal_size: Rect) -> Result<()> {
        // Dialog is 60% width, 30% height, centered
        let dialog_width = (terminal_size.width * 60) / 100;
        let dialog_height = (terminal_size.height * 30) / 100;
        let dialog_x = (terminal_size.width - dialog_width) / 2;
        let dialog_y = (terminal_size.height - dialog_height) / 2;

        // Check if click is within dialog bounds
        if x < dialog_x || x >= dialog_x + dialog_width {
            return Ok(());
        }
        if y < dialog_y || y >= dialog_y + dialog_height {
            return Ok(());
        }

        // Calculate relative position within dialog
        let relative_y = y - dialog_y;

        // "Yes / No" is on line 3 of the dialog (after empty line, message, empty line)
        // Dialog has border, so content starts at y=1
        // Line 0: border
        // Line 1: empty
        // Line 2: message
        // Line 3: empty
        // Line 4: "Yes / No"
        if relative_y == 4 {
            let relative_x = x - dialog_x;
            let center_x = dialog_width / 2;

            // "Yes / No" is centered
            // "Y" is approximately at center - 4
            // "N" is approximately at center + 3
            let yes_x = center_x.saturating_sub(4);
            let no_x = center_x + 3;

            if relative_x >= yes_x.saturating_sub(2) && relative_x <= yes_x + 2 {
                // Clicked on "Yes"
                if let Mode::Confirm(action) = self.mode {
                    self.mode = Mode::Normal;
                    match action {
                        ConfirmAction::Exit => {
                            self.should_exit = true;
                        }
                        ConfirmAction::DeleteSelected => self.delete_marked()?,
                        ConfirmAction::DeleteAllDead => self.delete_all_dead()?,
                        ConfirmAction::DeleteAllDuplicates => self.delete_all_duplicates()?,
                        ConfirmAction::ApplyChanges => self.apply_changes()?,
                        ConfirmAction::RequestElevation => {
                            self.request_elevation()?;
                        }
                        ConfirmAction::RestoreBackup => self.restore_selected_backup()?,
                        ConfirmAction::CreateSingleDirectory => self.create_single_directory()?,
                        ConfirmAction::CreateMarkedDirectories => {
                            self.create_marked_directories()?
                        }
                        ConfirmAction::DisconnectRemote => {
                            self.disconnect_from_remote()?;
                            self.set_status("Disconnected from remote computer");
                        }
                    }
                }
            } else if relative_x >= no_x.saturating_sub(2) && relative_x <= no_x + 2 {
                // Clicked on "No"
                self.mode = Mode::Normal;
            }
        }

        Ok(())
    }

    // Navigation helpers
    fn move_selection(&mut self, delta: i32) {
        match self.active_panel {
            Panel::Machine => {
                let new_pos = (self.machine_selected as i32 + delta)
                    .max(0)
                    .min(self.machine_paths.len().saturating_sub(1) as i32);
                self.machine_selected = new_pos as usize;
                self.machine_scrollbar_state =
                    self.machine_scrollbar_state.position(self.machine_selected);
            }
            Panel::User => {
                let new_pos = (self.user_selected as i32 + delta)
                    .max(0)
                    .min(self.user_paths.len().saturating_sub(1) as i32);
                self.user_selected = new_pos as usize;
                self.user_scrollbar_state = self.user_scrollbar_state.position(self.user_selected);
            }
        }
    }

    fn move_selection_to(&mut self, pos: usize) {
        match self.active_panel {
            Panel::Machine => {
                self.machine_selected = pos.min(self.machine_paths.len().saturating_sub(1));
                self.machine_scrollbar_state =
                    self.machine_scrollbar_state.position(self.machine_selected);
            }
            Panel::User => {
                self.user_selected = pos.min(self.user_paths.len().saturating_sub(1));
                self.user_scrollbar_state = self.user_scrollbar_state.position(self.user_selected);
            }
        }
    }

    fn toggle_mark(&mut self) {
        match self.active_panel {
            Panel::Machine => {
                if self.machine_marked.contains(&self.machine_selected) {
                    self.machine_marked.remove(&self.machine_selected);
                } else {
                    self.machine_marked.insert(self.machine_selected);
                }
                self.move_selection(1);
            }
            Panel::User => {
                if self.user_marked.contains(&self.user_selected) {
                    self.user_marked.remove(&self.user_selected);
                } else {
                    self.user_marked.insert(self.user_selected);
                }
                self.move_selection(1);
            }
        }
    }

    pub fn has_marked_items(&self) -> bool {
        !self.machine_marked.is_empty() || !self.user_marked.is_empty()
    }

    pub fn has_marked_dead_paths(&self) -> bool {
        // Check if any marked items in the active panel are dead paths
        match self.active_panel {
            Panel::Machine => self
                .machine_marked
                .iter()
                .any(|&idx| idx < self.machine_info.len() && !self.machine_info[idx].exists),
            Panel::User => self
                .user_marked
                .iter()
                .any(|&idx| idx < self.user_info.len() && !self.user_info[idx].exists),
        }
    }

    // Path modification
    fn delete_marked(&mut self) -> Result<()> {
        let mut deleted_count = 0;

        // Record machine deletions for undo
        let mut machine_deleted = Vec::new();
        for (idx, path) in self.machine_paths.iter().enumerate() {
            if self.machine_marked.contains(&idx) {
                machine_deleted.push((idx, path.clone()));
            }
        }

        // Delete from machine paths
        let mut new_machine = Vec::new();
        for (idx, path) in self.machine_paths.iter().enumerate() {
            if !self.machine_marked.contains(&idx) {
                new_machine.push(path.clone());
            } else {
                deleted_count += 1;
            }
        }
        self.machine_paths = new_machine;
        self.machine_marked.clear();

        // Record user deletions for undo
        let mut user_deleted = Vec::new();
        for (idx, path) in self.user_paths.iter().enumerate() {
            if self.user_marked.contains(&idx) {
                user_deleted.push((idx, path.clone()));
            }
        }

        // Delete from user paths
        let mut new_user = Vec::new();
        for (idx, path) in self.user_paths.iter().enumerate() {
            if !self.user_marked.contains(&idx) {
                new_user.push(path.clone());
            } else {
                deleted_count += 1;
            }
        }
        self.user_paths = new_user;
        self.user_marked.clear();

        // Clear redo stack and record undo operations
        self.clear_redo_stack();
        if !machine_deleted.is_empty() {
            self.undo_stack.push(Operation::DeletePaths {
                panel: Panel::Machine,
                deleted: machine_deleted,
            });
        }
        if !user_deleted.is_empty() {
            self.undo_stack.push(Operation::DeletePaths {
                panel: Panel::User,
                deleted: user_deleted,
            });
        }

        self.reanalyze();
        self.has_changes = true;
        self.set_status(&format!("Deleted {} path(s)", deleted_count));
        Ok(())
    }

    fn delete_all_dead(&mut self) -> Result<()> {
        // Record machine dead paths for undo
        let mut machine_deleted = Vec::new();
        for (idx, path) in self.machine_paths.iter().enumerate() {
            if !crate::path_analyzer::path_exists(path) {
                machine_deleted.push((idx, path.clone()));
            }
        }

        // Record user dead paths for undo
        let mut user_deleted = Vec::new();
        for (idx, path) in self.user_paths.iter().enumerate() {
            if !crate::path_analyzer::path_exists(path) {
                user_deleted.push((idx, path.clone()));
            }
        }

        let machine_before = self.machine_paths.len();
        let user_before = self.user_paths.len();

        self.machine_paths
            .retain(|p| crate::path_analyzer::path_exists(p));
        self.user_paths
            .retain(|p| crate::path_analyzer::path_exists(p));

        let deleted =
            (machine_before - self.machine_paths.len()) + (user_before - self.user_paths.len());

        // Clear redo stack and record undo operations
        self.clear_redo_stack();
        if !machine_deleted.is_empty() {
            self.undo_stack.push(Operation::DeletePaths {
                panel: Panel::Machine,
                deleted: machine_deleted,
            });
        }
        if !user_deleted.is_empty() {
            self.undo_stack.push(Operation::DeletePaths {
                panel: Panel::User,
                deleted: user_deleted,
            });
        }

        self.reanalyze();
        self.has_changes = true;
        self.set_status(&format!("Deleted {} dead path(s)", deleted));
        Ok(())
    }

    fn delete_all_duplicates(&mut self) -> Result<()> {
        let mut seen = HashSet::new();
        let mut deleted = 0;

        // Identify duplicates in machine paths for undo
        let mut machine_deleted = Vec::new();
        for (idx, path) in self.machine_paths.iter().enumerate() {
            let normalized = normalize_path(path).to_lowercase();
            if seen.contains(&normalized) {
                machine_deleted.push((idx, path.clone()));
            } else {
                seen.insert(normalized);
            }
        }

        // Keep first occurrence of each path (case-insensitive, normalized)
        seen.clear();
        let mut new_machine = Vec::new();
        for path in &self.machine_paths {
            let normalized = normalize_path(path).to_lowercase();
            if seen.insert(normalized) {
                new_machine.push(path.clone());
            } else {
                deleted += 1;
            }
        }
        self.machine_paths = new_machine;

        // Identify duplicates in user paths for undo
        let mut user_deleted = Vec::new();
        for (idx, path) in self.user_paths.iter().enumerate() {
            let normalized = normalize_path(path).to_lowercase();
            if seen.contains(&normalized) {
                user_deleted.push((idx, path.clone()));
            } else {
                seen.insert(normalized);
            }
        }

        let mut new_user = Vec::new();
        for path in &self.user_paths {
            let normalized = normalize_path(path).to_lowercase();
            if seen.insert(normalized) {
                new_user.push(path.clone());
            } else {
                deleted += 1;
            }
        }
        self.user_paths = new_user;

        // Clear redo stack and record undo operations
        self.clear_redo_stack();
        if !machine_deleted.is_empty() {
            self.undo_stack.push(Operation::DeletePaths {
                panel: Panel::Machine,
                deleted: machine_deleted,
            });
        }
        if !user_deleted.is_empty() {
            self.undo_stack.push(Operation::DeletePaths {
                panel: Panel::User,
                deleted: user_deleted,
            });
        }

        self.reanalyze();
        self.has_changes = true;
        self.set_status(&format!("Deleted {} duplicate path(s)", deleted));
        Ok(())
    }

    fn normalize_selected(&mut self) {
        let mut normalized_count = 0;
        let mut changes = Vec::new();

        match self.active_panel {
            Panel::Machine => {
                for idx in &self.machine_marked {
                    if let Some(path) = self.machine_paths.get_mut(*idx) {
                        let normalized = normalize_path(path);
                        if &normalized != path {
                            changes.push((*idx, path.clone(), normalized.clone()));
                            *path = normalized;
                            normalized_count += 1;
                        }
                    }
                }
                self.machine_marked.clear();

                // Clear redo stack and record undo operation for machine panel
                if !changes.is_empty() {
                    self.clear_redo_stack();
                    self.undo_stack.push(Operation::NormalizePaths {
                        panel: Panel::Machine,
                        changes,
                    });
                }
            }
            Panel::User => {
                for idx in &self.user_marked {
                    if let Some(path) = self.user_paths.get_mut(*idx) {
                        let normalized = normalize_path(path);
                        if &normalized != path {
                            changes.push((*idx, path.clone(), normalized.clone()));
                            *path = normalized;
                            normalized_count += 1;
                        }
                    }
                }
                self.user_marked.clear();

                // Clear redo stack and record undo operation for user panel
                if !changes.is_empty() {
                    self.clear_redo_stack();
                    self.undo_stack.push(Operation::NormalizePaths {
                        panel: Panel::User,
                        changes,
                    });
                }
            }
        }

        if normalized_count > 0 {
            self.reanalyze();
            self.has_changes = true;
            self.set_status(&format!("Normalized {} path(s)", normalized_count));
        }
    }

    fn move_marked_to_other_panel(&mut self) -> Result<()> {
        let from_panel = self.active_panel;
        let to_panel = from_panel.toggle();

        // In remote mode, copy instead of move (don't delete from source)
        let is_copy_mode = self.connection_mode == ConnectionMode::Remote;

        let (from_paths, to_paths, from_marked) = match from_panel {
            Panel::Machine => (
                &mut self.machine_paths,
                &mut self.user_paths,
                &mut self.machine_marked,
            ),
            Panel::User => (
                &mut self.user_paths,
                &mut self.machine_paths,
                &mut self.user_marked,
            ),
        };

        if from_marked.is_empty() {
            return Ok(());
        }

        // Record for undo: paths with their original indices
        let mut paths_with_indices = Vec::new();
        let mut indices: Vec<_> = from_marked.iter().copied().collect();
        indices.sort_unstable(); // Sort in ascending order for recording

        for idx in &indices {
            if let Some(path) = from_paths.get(*idx) {
                paths_with_indices.push((*idx, path.clone()));
            }
        }

        let mut moved = Vec::new();
        indices.sort_unstable_by(|a, b| b.cmp(a)); // Reverse order to maintain indices during removal

        for idx in indices {
            if let Some(path) = from_paths.get(idx) {
                moved.push(path.clone());
            }
        }

        // In copy mode (remote), don't remove from source
        if !is_copy_mode {
            // Remove from source (in reverse order)
            let mut new_from = Vec::new();
            for (idx, path) in from_paths.iter().enumerate() {
                if !from_marked.contains(&idx) {
                    new_from.push(path.clone());
                }
            }
            *from_paths = new_from;
        }

        // Add to destination
        to_paths.extend(moved.iter().cloned());

        let count = moved.len();
        from_marked.clear();

        // Clear redo stack and record undo operation
        if !paths_with_indices.is_empty() {
            self.clear_redo_stack();
            if is_copy_mode {
                self.undo_stack.push(Operation::CopyPaths {
                    from_panel,
                    to_panel,
                    paths_with_indices,
                });
            } else {
                self.undo_stack.push(Operation::MovePaths {
                    from_panel,
                    to_panel,
                    paths_with_indices,
                });
            }
        }

        self.reanalyze();
        self.has_changes = true;

        let action_verb = if is_copy_mode { "Copied" } else { "Moved" };
        let target = if self.connection_mode == ConnectionMode::Remote {
            if to_panel == Panel::User {
                self.remote_connection
                    .as_ref()
                    .map(|c| format!("remote ({})", c.computer_name()))
                    .unwrap_or_else(|| "remote".to_string())
            } else {
                "local".to_string()
            }
        } else {
            to_panel.scope().as_str().to_string()
        };

        self.set_status(&format!("{} {} path(s) to {}", action_verb, count, target));
        Ok(())
    }

    fn move_item_up(&mut self) {
        match self.active_panel {
            Panel::Machine => {
                if self.machine_selected > 0 {
                    let idx1 = self.machine_selected;
                    let idx2 = self.machine_selected - 1;

                    self.machine_paths.swap(idx1, idx2);
                    self.machine_selected -= 1;

                    // Clear redo stack and record undo operation
                    self.clear_redo_stack();
                    self.undo_stack.push(Operation::SwapPaths {
                        panel: Panel::Machine,
                        index1: idx1,
                        index2: idx2,
                    });

                    self.has_changes = true;
                    self.reanalyze();
                }
            }
            Panel::User => {
                if self.user_selected > 0 {
                    let idx1 = self.user_selected;
                    let idx2 = self.user_selected - 1;

                    self.user_paths.swap(idx1, idx2);
                    self.user_selected -= 1;

                    // Clear redo stack and record undo operation
                    self.clear_redo_stack();
                    self.undo_stack.push(Operation::SwapPaths {
                        panel: Panel::User,
                        index1: idx1,
                        index2: idx2,
                    });

                    self.has_changes = true;
                    self.reanalyze();
                }
            }
        }
    }

    fn start_add_path(&mut self) {
        // Open file browser instead of text input
        self.mode = Mode::FileBrowser;
        self.file_browser_selected = 0;
        // Start from current directory or C:\ if current dir unavailable
        self.file_browser_current_path =
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("C:\\"));
        self.read_current_directory();
    }

    /// Get available drive letters on Windows
    fn get_available_drives() -> Vec<DirectoryEntry> {
        let mut drives = Vec::new();

        // Add network drives option
        drives.push(DirectoryEntry {
            name: "Network...".to_string(),
            path: PathBuf::from("NETWORK"),
            is_parent: false,
            is_drive: true,
        });

        // Check drives A-Z
        for letter in b'A'..=b'Z' {
            let drive_letter = letter as char;
            let drive_path = PathBuf::from(format!("{}:\\", drive_letter));

            // Check if the drive exists
            if drive_path.exists() {
                drives.push(DirectoryEntry {
                    name: format!("{}:", drive_letter),
                    path: drive_path,
                    is_parent: false,
                    is_drive: true,
                });
            }
        }

        drives
    }

    /// Check if a path is a drive root (e.g., C:\, D:\)
    fn is_drive_root(path: &std::path::Path) -> bool {
        let path_str = path.to_string_lossy();
        // Match patterns like "C:\", "D:\", etc.
        path_str.len() == 3
            && path_str.chars().nth(1) == Some(':')
            && path_str.chars().nth(2) == Some('\\')
    }

    /// Read and populate entries for the current directory in file browser
    fn read_current_directory(&mut self) {
        self.file_browser_entries.clear();

        // Check if we're at a drive root or in "drives view"
        let path_str = self.file_browser_current_path.to_string_lossy().to_string();

        if path_str == "DRIVES" {
            // Show all available drives
            self.file_browser_entries = Self::get_available_drives();
        } else {
            // Add parent directory entry
            if Self::is_drive_root(&self.file_browser_current_path) {
                // At drive root, parent goes to drives list
                self.file_browser_entries.push(DirectoryEntry {
                    name: "..".to_string(),
                    path: PathBuf::from("DRIVES"),
                    is_parent: true,
                    is_drive: false,
                });
            } else if self.file_browser_current_path.parent().is_some() {
                // Normal parent directory
                self.file_browser_entries.push(DirectoryEntry {
                    name: "..".to_string(),
                    path: self
                        .file_browser_current_path
                        .parent()
                        .unwrap()
                        .to_path_buf(),
                    is_parent: true,
                    is_drive: false,
                });
            }

            // Read directory entries (only if not in drives view)
            if let Ok(entries) = std::fs::read_dir(&self.file_browser_current_path) {
                let mut dirs: Vec<DirectoryEntry> = entries
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| {
                        // Only include directories
                        entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                    })
                    .filter_map(|entry| {
                        let path = entry.path();
                        let name = path.file_name()?.to_string_lossy().to_string();
                        Some(DirectoryEntry {
                            name,
                            path,
                            is_parent: false,
                            is_drive: false,
                        })
                    })
                    .collect();

                // Sort directories alphabetically (case-insensitive)
                dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

                self.file_browser_entries.extend(dirs);
            }
        }

        // Ensure selection is valid
        if self.file_browser_selected >= self.file_browser_entries.len()
            && !self.file_browser_entries.is_empty()
        {
            self.file_browser_selected = self.file_browser_entries.len() - 1;
        }

        // Update scrollbar state
        self.file_browser_scrollbar_state = ScrollbarState::new(self.file_browser_entries.len())
            .position(self.file_browser_selected);
    }

    /// Navigate to the selected directory in file browser
    fn navigate_to_selected_directory(&mut self) {
        if self.file_browser_entries.is_empty() {
            return;
        }

        if let Some(entry) = self.file_browser_entries.get(self.file_browser_selected) {
            // Check if user selected the network option
            if entry.path.to_string_lossy() == "NETWORK" {
                // Switch to manual input mode for network path
                self.mode = Mode::Input(InputMode::AddPath);
                self.mode_enter_time = std::time::Instant::now();
                self.input_buffer = "\\\\".to_string(); // Start with UNC prefix
                return;
            }

            self.file_browser_current_path = entry.path.clone();
            self.file_browser_selected = 0; // Reset selection when entering new directory
            self.read_current_directory();
        }
    }

    /// Select the current directory in file browser and add it to PATH
    fn select_current_directory_for_path(&mut self) -> Result<()> {
        // Don't allow adding the "DRIVES" pseudo-directory
        let path_str = self.file_browser_current_path.to_string_lossy().to_string();
        if path_str == "DRIVES" {
            self.set_status("Cannot add drives list to PATH. Navigate to a directory first.");
            return Ok(());
        }

        // Set input buffer and add path
        self.input_buffer = path_str;
        self.mode = Mode::Normal;
        self.add_path_from_input()?;
        self.input_buffer.clear();

        Ok(())
    }

    fn start_edit_path(&mut self) {
        let current_path = match self.active_panel {
            Panel::Machine => self.machine_paths.get(self.machine_selected),
            Panel::User => self.user_paths.get(self.user_selected),
        };

        if let Some(path) = current_path {
            self.input_buffer = path.clone();
            self.mode = Mode::Input(InputMode::EditPath);
            self.mode_enter_time = std::time::Instant::now();
        }
    }

    fn add_path_from_input(&mut self) -> Result<()> {
        if self.input_buffer.is_empty() {
            return Ok(());
        }

        // Check if directory exists
        let expanded = normalize_path(&self.input_buffer);
        if !std::path::Path::new(&expanded).exists() {
            // Directory doesn't exist - check if we can create it
            if Self::can_create_directory(&self.input_buffer) {
                // Store the path and ask for confirmation
                self.pending_directory = self.input_buffer.clone();
                self.mode = Mode::Confirm(ConfirmAction::CreateSingleDirectory);
                return Ok(());
            } else {
                // Can't create (network path, invalid chars, etc.) - add anyway as dead path
                self.set_status("Warning: Path cannot be auto-created (network or invalid). Adding as dead path.");
            }
        }

        // Directory exists or can't be created - add it
        let new_path = self.input_buffer.clone();
        let (panel, index) = match self.active_panel {
            Panel::Machine => {
                let idx = self.machine_paths.len();
                self.machine_paths.push(new_path.clone());
                (Panel::Machine, idx)
            }
            Panel::User => {
                let idx = self.user_paths.len();
                self.user_paths.push(new_path.clone());
                (Panel::User, idx)
            }
        };

        // Clear redo stack and record undo operation
        self.clear_redo_stack();
        self.undo_stack.push(Operation::AddPath {
            panel,
            index,
            path: new_path,
        });

        self.reanalyze();
        self.has_changes = true;
        self.set_status("Path added");
        Ok(())
    }

    fn update_path_from_input(&mut self) -> Result<()> {
        if self.input_buffer.is_empty() {
            return Ok(());
        }

        let new_path = self.input_buffer.clone();

        match self.active_panel {
            Panel::Machine => {
                if let Some(path) = self.machine_paths.get_mut(self.machine_selected) {
                    let old_path = path.clone();
                    *path = new_path.clone();

                    // Clear redo stack and record undo operation
                    self.clear_redo_stack();
                    self.undo_stack.push(Operation::EditPath {
                        panel: Panel::Machine,
                        index: self.machine_selected,
                        old_path,
                        new_path,
                    });
                }
            }
            Panel::User => {
                if let Some(path) = self.user_paths.get_mut(self.user_selected) {
                    let old_path = path.clone();
                    *path = new_path.clone();

                    // Clear redo stack and record undo operation
                    self.clear_redo_stack();
                    self.undo_stack.push(Operation::EditPath {
                        panel: Panel::User,
                        index: self.user_selected,
                        old_path,
                        new_path,
                    });
                }
            }
        }

        self.reanalyze();
        self.has_changes = true;
        self.set_status("Path updated");
        Ok(())
    }

    // Backup/Restore
    fn create_backup(&mut self) -> Result<()> {
        let user_path = registry::join_paths(&self.user_original);
        let machine_path = registry::join_paths(&self.machine_original);

        let backup = PathBackup::new(
            user_path,
            machine_path,
            self.user_original.clone(),
            self.machine_original.clone(),
        );

        let backup_dir = backup::get_default_backup_dir();
        let filepath = backup.save(&backup_dir)?;

        self.set_status(&format!("Backup saved: {}", filepath.display()));
        Ok(())
    }

    fn show_backup_list(&mut self) -> Result<()> {
        let backup_dir = backup::get_default_backup_dir();
        self.backup_list = backup::list_backups(&backup_dir)?;
        self.backup_selected = 0;

        if self.backup_list.is_empty() {
            self.set_status("No backups found");
        } else {
            self.mode = Mode::BackupList;
        }
        Ok(())
    }

    fn restore_selected_backup(&mut self) -> Result<()> {
        if self.backup_selected < self.backup_list.len() {
            let backup_path = &self.backup_list[self.backup_selected];
            let backup = PathBackup::load(backup_path)?;

            self.user_paths = backup.user_paths;
            self.machine_paths = backup.machine_paths;

            self.reanalyze();
            self.has_changes = true;
            self.set_status("Backup restored (not yet applied)");
        }
        Ok(())
    }

    // Apply changes to registry
    fn apply_changes(&mut self) -> Result<()> {
        // Save current state as backup first
        self.create_backup()?;

        match self.connection_mode {
            ConnectionMode::Local => {
                // Apply user paths
                let user_path = registry::join_paths(&self.user_paths);
                registry::write_path(PathScope::User, &user_path)?;

                // Apply machine paths (if admin)
                if self.is_admin {
                    let machine_path = registry::join_paths(&self.machine_paths);
                    registry::write_path(PathScope::Machine, &machine_path)?;
                }

                // Update originals
                self.user_original = self.user_paths.clone();
                self.machine_original = self.machine_paths.clone();
            }
            ConnectionMode::Remote => {
                // In remote mode, only write to local MACHINE and remote MACHINE
                // (USER paths are not shown/editable in remote mode)

                // Apply local machine paths (if admin)
                if self.is_admin {
                    let machine_path = registry::join_paths(&self.machine_paths);
                    registry::write_path(PathScope::Machine, &machine_path)?;
                }

                // Apply remote machine paths (if connected and admin)
                if self.is_admin {
                    if let Some(ref connection) = self.remote_connection {
                        let remote_path = registry::join_paths(&self.remote_machine_paths);
                        registry::write_path_remote(PathScope::Machine, &remote_path, connection)?;
                    }
                }

                // Update originals
                self.machine_original = self.machine_paths.clone();
                self.remote_machine_original = self.remote_machine_paths.clone();
            }
        }

        self.has_changes = false;

        // Note: Undo/redo stacks are NOT cleared on save, allowing users to undo changes even after saving

        // Detect running processes that won't pick up the new PATH
        match crate::process_detector::detect_running_processes() {
            Ok(processes) => {
                if !processes.is_empty() {
                    self.processes_to_restart = processes;
                    self.mode = Mode::ProcessRestartInfo;
                } else {
                    // No non-responsive processes detected
                    self.set_status(
                        "Changes applied! All running processes should pick up the new PATH.",
                    );
                }
            }
            Err(e) => {
                // Process detection failed, but changes were still applied successfully
                self.set_status(&format!(
                    "Changes applied! (Process detection failed: {})",
                    e
                ));
            }
        }

        Ok(())
    }

    pub fn confirm_exit(&mut self) {
        if self.has_changes {
            self.mode = Mode::Confirm(ConfirmAction::Exit);
        } else {
            self.should_exit = true;
        }
    }

    /// Handle F10 key press - shows exit confirmation dialog
    pub fn handle_f10_press(&mut self) {
        // Always show confirmation dialog (user must press F10 again in dialog to confirm)
        self.mode = Mode::Confirm(ConfirmAction::Exit);
    }

    /// Check if a directory can be created (not network path, valid chars, etc.)
    fn can_create_directory(path: &str) -> bool {
        if path.is_empty() {
            return false;
        }

        // Check if it's a network path (starts with \\ or //)
        if path.starts_with("\\\\") || path.starts_with("//") {
            return false;
        }

        // Check for invalid characters (basic check)
        let invalid_chars = ['<', '>', '|', '\"', '?', '*'];
        if path.chars().any(|c| invalid_chars.contains(&c)) {
            return false;
        }

        // Path should have a drive letter on Windows or be a relative path
        // Simple check: if it has a colon, it should be at position 1 (like C:)
        if let Some(colon_pos) = path.find(':') {
            if colon_pos != 1 {
                return false;
            }
        }

        true
    }

    /// Create a directory and its parent directories
    /// If remote_computer is provided, creates the directory on the remote computer using UNC path
    fn create_directory_with_remote(path: &str, remote_computer: Option<&str>) -> Result<()> {
        use std::fs;
        use std::path::Path;

        if path.is_empty() {
            return Err(anyhow::anyhow!("Path is empty"));
        }

        // Expand environment variables
        let expanded = normalize_path(path);

        // Determine the actual path to create
        let actual_path = if let Some(computer_name) = remote_computer {
            // Convert to UNC path for remote creation
            to_unc_path(&expanded, computer_name)
                .ok_or_else(|| anyhow::anyhow!("Cannot convert to UNC path: {}", expanded))?
        } else {
            expanded
        };

        let path_obj = Path::new(&actual_path);

        // Create directory with parents
        fs::create_dir_all(path_obj).map_err(|e| {
            if remote_computer.is_some() {
                anyhow::anyhow!(
                    "Failed to create remote directory '{}': {}. \
                    Ensure C$ administrative shares are enabled and accessible.",
                    actual_path,
                    e
                )
            } else {
                anyhow::anyhow!("Failed to create directory '{}': {}", actual_path, e)
            }
        })?;

        Ok(())
    }

    /// Create the pending directory and add the path
    fn create_single_directory(&mut self) -> Result<()> {
        if self.pending_directory.is_empty() {
            return Ok(());
        }

        // Determine if we're creating on remote computer
        // In remote mode, User panel (right) is actually the remote machine
        let remote_computer =
            if self.connection_mode == ConnectionMode::Remote && self.active_panel == Panel::User {
                self.remote_connection.as_ref().map(|c| c.computer_name())
            } else {
                None
            };

        match Self::create_directory_with_remote(&self.pending_directory, remote_computer) {
            Ok(()) => {
                // Directory created successfully - now add the path
                match self.active_panel {
                    Panel::Machine => {
                        self.machine_paths.push(self.pending_directory.clone());
                    }
                    Panel::User => {
                        if self.connection_mode == ConnectionMode::Remote {
                            self.remote_machine_paths
                                .push(self.pending_directory.clone());
                        } else {
                            self.user_paths.push(self.pending_directory.clone());
                        }
                    }
                }

                self.reanalyze();
                self.has_changes = true;
                self.set_status(&format!(
                    "Created directory and added: {}",
                    self.pending_directory
                ));
                self.pending_directory.clear();
                Ok(())
            }
            Err(e) => {
                self.set_status(&format!("Failed to create directory: {}", e));
                self.pending_directory.clear();
                Err(e)
            }
        }
    }

    /// Create all marked dead directories
    fn create_marked_directories(&mut self) -> Result<()> {
        let mut created_count = 0;
        let mut skipped_count = 0;
        let mut failed_paths = Vec::new();

        // Determine if we're creating on remote computer
        let remote_computer =
            if self.connection_mode == ConnectionMode::Remote && self.active_panel == Panel::User {
                self.remote_connection.as_ref().map(|c| c.computer_name())
            } else {
                None
            };

        // Collect all marked dead paths
        let marked_paths: Vec<(usize, String)> = match self.active_panel {
            Panel::Machine => self
                .machine_marked
                .iter()
                .filter_map(|&idx| {
                    if idx < self.machine_paths.len() && idx < self.machine_info.len() {
                        if !self.machine_info[idx].exists {
                            Some((idx, self.machine_paths[idx].clone()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect(),
            Panel::User => {
                if self.connection_mode == ConnectionMode::Remote {
                    // In remote mode, User panel shows remote machine paths
                    self.remote_machine_marked
                        .iter()
                        .filter_map(|&idx| {
                            if idx < self.remote_machine_paths.len()
                                && idx < self.remote_machine_info.len()
                            {
                                if !self.remote_machine_info[idx].exists {
                                    Some((idx, self.remote_machine_paths[idx].clone()))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect()
                } else {
                    self.user_marked
                        .iter()
                        .filter_map(|&idx| {
                            if idx < self.user_paths.len() && idx < self.user_info.len() {
                                if !self.user_info[idx].exists {
                                    Some((idx, self.user_paths[idx].clone()))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect()
                }
            }
        };

        // Try to create each directory
        for (_idx, path) in marked_paths {
            if !Self::can_create_directory(&path) {
                skipped_count += 1;
                continue;
            }

            match Self::create_directory_with_remote(&path, remote_computer) {
                Ok(()) => created_count += 1,
                Err(_) => {
                    failed_paths.push(path);
                }
            }
        }

        // Reanalyze to update dead path status
        if created_count > 0 {
            self.reanalyze();
            self.has_changes = true;
        }

        // Show status message
        let mut msg = format!("Created {} directories", created_count);
        if skipped_count > 0 {
            msg.push_str(&format!(", skipped {} (network/invalid)", skipped_count));
        }
        if !failed_paths.is_empty() {
            msg.push_str(&format!(", failed {} paths", failed_paths.len()));
        }
        self.set_status(&msg);

        Ok(())
    }

    fn reanalyze(&mut self) {
        match self.connection_mode {
            ConnectionMode::Local => {
                self.user_info = analyze_paths(&self.user_paths, &self.machine_paths);
                self.machine_info = analyze_paths(&self.machine_paths, &self.user_paths);

                // Update scrollbar content lengths
                self.machine_scrollbar_state = self
                    .machine_scrollbar_state
                    .content_length(self.machine_paths.len());
                self.user_scrollbar_state = self
                    .user_scrollbar_state
                    .content_length(self.user_paths.len());

                // Adjust selection if out of bounds
                if self.machine_selected >= self.machine_paths.len()
                    && !self.machine_paths.is_empty()
                {
                    self.machine_selected = self.machine_paths.len() - 1;
                    self.machine_scrollbar_state =
                        self.machine_scrollbar_state.position(self.machine_selected);
                }
                if self.user_selected >= self.user_paths.len() && !self.user_paths.is_empty() {
                    self.user_selected = self.user_paths.len() - 1;
                    self.user_scrollbar_state =
                        self.user_scrollbar_state.position(self.user_selected);
                }
            }
            ConnectionMode::Remote => {
                // In remote mode: analyze local machine vs remote machine paths
                // Local paths are analyzed normally (no remote computer name)
                self.machine_info = analyze_paths(&self.machine_paths, &self.remote_machine_paths);

                // Remote paths need UNC path validation - pass the remote computer name
                let remote_computer_name = self
                    .remote_connection
                    .as_ref()
                    .map(|conn| conn.computer_name());
                self.remote_machine_info = analyze_paths_with_remote(
                    &self.remote_machine_paths,
                    &self.machine_paths,
                    remote_computer_name,
                );

                // Update scrollbar content lengths
                self.machine_scrollbar_state = self
                    .machine_scrollbar_state
                    .content_length(self.machine_paths.len());
                self.remote_scrollbar_state = self
                    .remote_scrollbar_state
                    .content_length(self.remote_machine_paths.len());

                // Adjust selection if out of bounds
                if self.machine_selected >= self.machine_paths.len()
                    && !self.machine_paths.is_empty()
                {
                    self.machine_selected = self.machine_paths.len() - 1;
                    self.machine_scrollbar_state =
                        self.machine_scrollbar_state.position(self.machine_selected);
                }
                if self.remote_machine_selected >= self.remote_machine_paths.len()
                    && !self.remote_machine_paths.is_empty()
                {
                    self.remote_machine_selected = self.remote_machine_paths.len() - 1;
                    self.remote_scrollbar_state = self
                        .remote_scrollbar_state
                        .position(self.remote_machine_selected);
                }
            }
        }
    }

    /// Request UAC elevation and restart the application with administrator privileges
    fn request_elevation(&mut self) -> Result<()> {
        // Build elevation state from current app state
        let elevation_state = crate::elevation::ElevationState {
            connection_mode: self.connection_mode,
            remote_computer_name: self
                .remote_connection
                .as_ref()
                .map(|c| c.computer_name().to_string()),
            machine_paths: self.machine_paths.clone(),
            user_paths: self.user_paths.clone(),
            remote_machine_paths: self.remote_machine_paths.clone(),
            active_panel: self.active_panel,
            machine_selected: self.machine_selected,
            user_selected: self.user_selected,
            remote_machine_selected: self.remote_machine_selected,
            machine_marked: self.machine_marked.clone(),
            user_marked: self.user_marked.clone(),
            remote_machine_marked: self.remote_machine_marked.clone(),
            filter_mode: self.filter_mode,
            input_buffer: self.input_buffer.clone(),
            pending_directory: self.pending_directory.clone(),
            theme_arg: self.theme_arg.clone(),
        };

        // Get current executable path
        let current_exe = std::env::current_exe()
            .context("Failed to get current executable path")?
            .to_string_lossy()
            .to_string();

        // Request elevation (this will trigger UAC and restart the app)
        match crate::elevation::request_elevation(&elevation_state, &current_exe) {
            Ok(()) => {
                // Elevation successful - the new elevated process is starting
                // Exit this instance
                self.should_exit = true;
                self.set_status("Restarting with elevated privileges...");
                Ok(())
            }
            Err(e) => {
                // Elevation failed or was cancelled
                self.set_status(&format!(
                    "Elevation failed: {}. Continuing without elevation.",
                    e
                ));
                Ok(())
            }
        }
    }

    fn set_status(&mut self, message: &str) {
        self.status_message = message.to_string();
    }

    // Bulk selection functions
    fn mark_all_visible(&mut self) {
        let count = match self.active_panel {
            Panel::Machine => {
                let filtered = self.get_filtered_indices(&self.machine_info);
                for idx in filtered {
                    self.machine_marked.insert(idx);
                }
                self.machine_marked.len()
            }
            Panel::User => {
                let filtered = self.get_filtered_indices(&self.user_info);
                for idx in filtered {
                    self.user_marked.insert(idx);
                }
                self.user_marked.len()
            }
        };
        self.set_status(&format!(
            "Marked {} visible paths in {} scope",
            count,
            self.active_panel.scope().as_str()
        ));
    }

    fn mark_all_both_scopes(&mut self) {
        let machine_filtered = self.get_filtered_indices(&self.machine_info);
        for idx in machine_filtered {
            self.machine_marked.insert(idx);
        }
        let user_filtered = self.get_filtered_indices(&self.user_info);
        for idx in user_filtered {
            self.user_marked.insert(idx);
        }
        let total = self.machine_marked.len() + self.user_marked.len();
        self.set_status(&format!("Marked {} paths in both scopes", total));
    }

    fn mark_all_duplicates(&mut self) {
        let count = match self.active_panel {
            Panel::Machine => {
                for (idx, info) in self.machine_info.iter().enumerate() {
                    if info.is_duplicate {
                        self.machine_marked.insert(idx);
                    }
                }
                self.machine_marked.len()
            }
            Panel::User => {
                for (idx, info) in self.user_info.iter().enumerate() {
                    if info.is_duplicate {
                        self.user_marked.insert(idx);
                    }
                }
                self.user_marked.len()
            }
        };
        self.set_status(&format!("Marked {} duplicate paths", count));
    }

    fn mark_all_dead(&mut self) {
        let count = match self.active_panel {
            Panel::Machine => {
                for (idx, info) in self.machine_info.iter().enumerate() {
                    if !info.exists {
                        self.machine_marked.insert(idx);
                    }
                }
                self.machine_marked.len()
            }
            Panel::User => {
                for (idx, info) in self.user_info.iter().enumerate() {
                    if !info.exists {
                        self.user_marked.insert(idx);
                    }
                }
                self.user_marked.len()
            }
        };
        self.set_status(&format!("Marked {} dead paths", count));
    }

    fn mark_all_non_normalized(&mut self) {
        let count = match self.active_panel {
            Panel::Machine => {
                for (idx, info) in self.machine_info.iter().enumerate() {
                    if info.needs_normalization {
                        self.machine_marked.insert(idx);
                    }
                }
                self.machine_marked.len()
            }
            Panel::User => {
                for (idx, info) in self.user_info.iter().enumerate() {
                    if info.needs_normalization {
                        self.user_marked.insert(idx);
                    }
                }
                self.user_marked.len()
            }
        };
        self.set_status(&format!("Marked {} non-normalized paths", count));
    }

    fn unmark_all(&mut self) {
        let total = self.machine_marked.len() + self.user_marked.len();
        self.machine_marked.clear();
        self.user_marked.clear();
        self.set_status(&format!("Unmarked {} paths", total));
    }

    /// Get filtered indices based on current filter mode
    pub fn get_filtered_indices(&self, info: &[PathInfo]) -> Vec<usize> {
        match self.filter_mode {
            FilterMode::None => (0..info.len()).collect(),
            FilterMode::Dead => info
                .iter()
                .enumerate()
                .filter(|(_, i)| !i.exists)
                .map(|(idx, _)| idx)
                .collect(),
            FilterMode::Duplicates => info
                .iter()
                .enumerate()
                .filter(|(_, i)| i.is_duplicate)
                .map(|(idx, _)| idx)
                .collect(),
            FilterMode::NonNormalized => info
                .iter()
                .enumerate()
                .filter(|(_, i)| i.needs_normalization)
                .map(|(idx, _)| idx)
                .collect(),
            FilterMode::Valid => info
                .iter()
                .enumerate()
                .filter(|(_, i)| i.exists && !i.is_duplicate && !i.needs_normalization)
                .map(|(idx, _)| idx)
                .collect(),
        }
    }

    pub fn get_statistics(&self) -> Statistics {
        let machine_dead = self.machine_info.iter().filter(|i| !i.exists).count();
        let user_dead = self.user_info.iter().filter(|i| !i.exists).count();

        let machine_duplicates = self.machine_info.iter().filter(|i| i.is_duplicate).count();
        let user_duplicates = self.user_info.iter().filter(|i| i.is_duplicate).count();

        let machine_non_normalized = self
            .machine_info
            .iter()
            .filter(|i| i.needs_normalization)
            .count();
        let user_non_normalized = self
            .user_info
            .iter()
            .filter(|i| i.needs_normalization)
            .count();

        Statistics {
            machine_total: self.machine_paths.len(),
            user_total: self.user_paths.len(),
            machine_dead,
            user_dead,
            machine_duplicates,
            user_duplicates,
            machine_non_normalized,
            user_non_normalized,
        }
    }

    /// Calculate the total PATH length for a given scope
    /// Returns the total character count including semicolon separators
    /// Windows PATH limit is 2047 characters
    pub fn calculate_path_length(&self, panel: Panel) -> usize {
        let paths = match (self.connection_mode, panel) {
            (ConnectionMode::Local, Panel::Machine) => &self.machine_paths,
            (ConnectionMode::Local, Panel::User) => &self.user_paths,
            (ConnectionMode::Remote, Panel::Machine) => &self.machine_paths,
            (ConnectionMode::Remote, Panel::User) => &self.remote_machine_paths,
        };

        if paths.is_empty() {
            return 0;
        }

        // Calculate total length: sum of all path lengths + (n-1) semicolons
        let total_chars: usize = paths.iter().map(|p| p.len()).sum();
        let separators = paths.len().saturating_sub(1);
        total_chars + separators
    }

    /// Undo the last operation by popping from the undo stack and reversing it
    pub fn undo(&mut self) -> Result<()> {
        if let Some(operation) = self.undo_stack.pop() {
            // Push to redo stack before reversing
            self.redo_stack.push(operation.clone());

            // Reverse the operation without recording it
            match operation {
                Operation::DeletePaths { panel, deleted } => {
                    // Restore deleted paths at their original indices
                    let paths = match panel {
                        Panel::Machine => &mut self.machine_paths,
                        Panel::User => &mut self.user_paths,
                    };

                    // Sort by index to insert in correct order
                    let mut sorted_deleted = deleted;
                    sorted_deleted.sort_by_key(|(idx, _)| *idx);

                    for (idx, path) in sorted_deleted {
                        paths.insert(idx, path);
                    }
                }

                Operation::AddPath { panel, index, .. } => {
                    // Remove the added path
                    let paths = match panel {
                        Panel::Machine => &mut self.machine_paths,
                        Panel::User => &mut self.user_paths,
                    };

                    if index < paths.len() {
                        paths.remove(index);
                    }
                }

                Operation::EditPath {
                    panel,
                    index,
                    old_path,
                    ..
                } => {
                    // Restore the old path value
                    let paths = match panel {
                        Panel::Machine => &mut self.machine_paths,
                        Panel::User => &mut self.user_paths,
                    };

                    if let Some(path) = paths.get_mut(index) {
                        *path = old_path;
                    }
                }

                Operation::SwapPaths {
                    panel,
                    index1,
                    index2,
                } => {
                    // Swap back (same operation reverses itself)
                    let paths = match panel {
                        Panel::Machine => &mut self.machine_paths,
                        Panel::User => &mut self.user_paths,
                    };

                    if index1 < paths.len() && index2 < paths.len() {
                        paths.swap(index1, index2);
                    }
                }

                Operation::MovePaths {
                    from_panel,
                    to_panel: _,
                    paths_with_indices,
                } => {
                    // Move paths back from to_panel to from_panel at their original indices
                    let (from_paths, to_paths) = match from_panel {
                        Panel::Machine => (&mut self.machine_paths, &mut self.user_paths),
                        Panel::User => (&mut self.user_paths, &mut self.machine_paths),
                    };

                    // Remove from destination panel (they were appended at the end)
                    // We need to remove the last N items where N = paths_with_indices.len()
                    let count = paths_with_indices.len();
                    let new_len = to_paths.len().saturating_sub(count);
                    to_paths.truncate(new_len);

                    // Restore to source panel at original indices
                    let mut sorted_paths = paths_with_indices;
                    sorted_paths.sort_by_key(|(idx, _)| *idx);

                    for (idx, path) in sorted_paths {
                        from_paths.insert(idx, path);
                    }
                }

                Operation::CopyPaths {
                    from_panel: _,
                    to_panel,
                    paths_with_indices,
                } => {
                    // Undo copy: remove the copied paths from to_panel
                    // Note: paths_with_indices contains the indices from the FROM panel,
                    // but we need to remove from the TO panel where they were appended
                    let to_paths = match to_panel {
                        Panel::Machine => &mut self.machine_paths,
                        Panel::User => &mut self.user_paths,
                    };

                    // Paths were appended to the end, so remove the last N paths
                    let count = paths_with_indices.len();
                    let new_len = to_paths.len().saturating_sub(count);
                    to_paths.truncate(new_len);
                }

                Operation::NormalizePaths { panel, changes } => {
                    // Restore old (non-normalized) values
                    let paths = match panel {
                        Panel::Machine => &mut self.machine_paths,
                        Panel::User => &mut self.user_paths,
                    };

                    for (idx, old_path, _) in changes {
                        if let Some(path) = paths.get_mut(idx) {
                            *path = old_path;
                        }
                    }
                }
            }

            self.reanalyze();
            self.has_changes = true;
            self.set_status("Undo successful");
            Ok(())
        } else {
            self.set_status("Nothing to undo");
            Ok(())
        }
    }

    /// Check if there are operations available to undo
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Redo the last undone operation by popping from the redo stack and re-applying it
    pub fn redo(&mut self) -> Result<()> {
        if let Some(operation) = self.redo_stack.pop() {
            // Re-apply the operation and push back to undo stack
            self.undo_stack.push(operation.clone());

            // Apply the operation
            match operation {
                Operation::DeletePaths { panel, deleted } => {
                    // Re-delete the paths
                    let paths = match panel {
                        Panel::Machine => &mut self.machine_paths,
                        Panel::User => &mut self.user_paths,
                    };

                    // Sort indices in reverse to delete from end to start
                    let mut indices: Vec<_> = deleted.iter().map(|(idx, _)| *idx).collect();
                    indices.sort_unstable_by(|a, b| b.cmp(a));

                    for idx in indices {
                        if idx < paths.len() {
                            paths.remove(idx);
                        }
                    }
                }

                Operation::AddPath {
                    panel,
                    index: _,
                    path,
                } => {
                    // Re-add the path
                    let paths = match panel {
                        Panel::Machine => &mut self.machine_paths,
                        Panel::User => &mut self.user_paths,
                    };
                    paths.push(path);
                }

                Operation::EditPath {
                    panel,
                    index,
                    old_path: _,
                    new_path,
                } => {
                    // Re-apply the edit
                    let paths = match panel {
                        Panel::Machine => &mut self.machine_paths,
                        Panel::User => &mut self.user_paths,
                    };

                    if let Some(path) = paths.get_mut(index) {
                        *path = new_path;
                    }
                }

                Operation::SwapPaths {
                    panel,
                    index1,
                    index2,
                } => {
                    // Re-swap (same as undo)
                    let paths = match panel {
                        Panel::Machine => &mut self.machine_paths,
                        Panel::User => &mut self.user_paths,
                    };

                    if index1 < paths.len() && index2 < paths.len() {
                        paths.swap(index1, index2);
                    }
                }

                Operation::MovePaths {
                    from_panel,
                    to_panel: _,
                    paths_with_indices,
                } => {
                    // Re-do: move paths from from_panel to to_panel (destination is implicit from from_panel)
                    let (from_paths, to_paths) = match from_panel {
                        Panel::Machine => (&mut self.machine_paths, &mut self.user_paths),
                        Panel::User => (&mut self.user_paths, &mut self.machine_paths),
                    };

                    // Remove from source panel (sorted in reverse)
                    let mut indices: Vec<_> =
                        paths_with_indices.iter().map(|(idx, _)| *idx).collect();
                    indices.sort_unstable_by(|a, b| b.cmp(a));

                    for idx in indices {
                        if idx < from_paths.len() {
                            from_paths.remove(idx);
                        }
                    }

                    // Add to destination panel
                    for (_, path) in paths_with_indices {
                        to_paths.push(path);
                    }
                }

                Operation::CopyPaths {
                    from_panel: _,
                    to_panel,
                    paths_with_indices,
                } => {
                    // Redo copy: add the copied paths back to to_panel
                    let to_paths = match to_panel {
                        Panel::Machine => &mut self.machine_paths,
                        Panel::User => &mut self.user_paths,
                    };

                    for (_, path) in paths_with_indices {
                        to_paths.push(path);
                    }
                }

                Operation::NormalizePaths { panel, changes } => {
                    // Re-apply normalizations
                    let paths = match panel {
                        Panel::Machine => &mut self.machine_paths,
                        Panel::User => &mut self.user_paths,
                    };

                    for (idx, _, new_path) in changes {
                        if let Some(path) = paths.get_mut(idx) {
                            *path = new_path;
                        }
                    }
                }
            }

            self.reanalyze();
            self.has_changes = true;
            self.set_status("Redo successful");
            Ok(())
        } else {
            self.set_status("Nothing to redo");
            Ok(())
        }
    }

    /// Check if there are operations available to redo
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear the redo stack - called whenever a new operation is performed
    fn clear_redo_stack(&mut self) {
        self.redo_stack.clear();
    }

    /// Activate menu by accelerator character
    fn activate_menu_by_char(&mut self, c: char) {
        let menus = crate::menu::get_menus(self.connection_mode);
        let c_lower = c.to_lowercase().next().unwrap_or(c);

        for (i, menu) in menus.iter().enumerate() {
            if menu.accelerator == c_lower {
                self.mode = Mode::Menu {
                    active_menu: i,
                    selected_item: 0,
                };
                return;
            }
        }
    }

    /// Handle click on menu bar
    fn handle_menu_bar_click(&mut self, column: u16) -> Result<()> {
        let menus = crate::menu::get_menus(self.connection_mode);
        let mut x_offset = 1; // Start with 1 for initial space

        for (i, menu) in menus.iter().enumerate() {
            let menu_start = x_offset;
            let menu_end = x_offset + menu.name.len() as u16 + 2; // +2 for spaces around menu name

            if column >= menu_start && column < menu_end {
                // Clicked on this menu
                self.mode = Mode::Menu {
                    active_menu: i,
                    selected_item: 0,
                };
                return Ok(());
            }

            x_offset = menu_end;
        }

        Ok(())
    }

    /// Handle click in menu dropdown
    fn handle_menu_click(
        &mut self,
        column: u16,
        row: u16,
        active_menu: usize,
        _selected_item: usize,
    ) -> Result<()> {
        let menus = crate::menu::get_menus(self.connection_mode);

        if active_menu >= menus.len() {
            return Ok(());
        }

        let menu = &menus[active_menu];

        // Calculate menu position
        let mut x_offset = 1;
        for menu in menus.iter().take(active_menu) {
            x_offset += menu.name.len() as u16 + 2;
        }

        // Calculate menu width
        let mut menu_width = menu.name.len();
        for item in &menu.items {
            let item_text_len =
                item.label.len() + item.shortcut.as_ref().map(|s| s.len() + 2).unwrap_or(0);
            menu_width = menu_width.max(item_text_len);
        }
        menu_width += 4;

        let menu_x = x_offset;
        let menu_y = 1; // Below menu bar
        let menu_height = menu.items.len() as u16 + 2; // +2 for borders

        // Check if click is within menu bounds
        if column >= menu_x
            && column < menu_x + menu_width as u16
            && row >= menu_y
            && row < menu_y + menu_height
        {
            // Calculate which item was clicked (accounting for border)
            let clicked_item = (row - menu_y - 1) as usize; // -1 for top border

            if clicked_item < menu.items.len() {
                let item = &menu.items[clicked_item];
                if item.enabled {
                    self.execute_menu_action(item.action)?;
                }
            }
        } else {
            // Clicked outside menu, close it
            self.mode = Mode::Normal;
        }

        Ok(())
    }

    /// Handle keyboard input in menu mode
    fn handle_menu_input(
        &mut self,
        key: KeyEvent,
        active_menu: usize,
        selected_item: usize,
    ) -> Result<()> {
        let menus = crate::menu::get_menus(self.connection_mode);

        match key.code {
            KeyCode::Esc => {
                // Close menu and return to normal mode
                self.mode = Mode::Normal;
            }
            KeyCode::Left => {
                // Move to previous menu
                if active_menu > 0 {
                    self.mode = Mode::Menu {
                        active_menu: active_menu - 1,
                        selected_item: 0,
                    };
                }
            }
            KeyCode::Right => {
                // Move to next menu
                if active_menu + 1 < menus.len() {
                    self.mode = Mode::Menu {
                        active_menu: active_menu + 1,
                        selected_item: 0,
                    };
                }
            }
            KeyCode::Up => {
                // Move to previous item in menu
                if selected_item > 0 {
                    self.mode = Mode::Menu {
                        active_menu,
                        selected_item: selected_item - 1,
                    };
                }
            }
            KeyCode::Down => {
                // Move to next item in menu
                if active_menu < menus.len() && selected_item + 1 < menus[active_menu].items.len() {
                    self.mode = Mode::Menu {
                        active_menu,
                        selected_item: selected_item + 1,
                    };
                }
            }
            KeyCode::Enter => {
                // Execute selected menu action
                if active_menu < menus.len() {
                    let menu = &menus[active_menu];
                    if selected_item < menu.items.len() {
                        let item = &menu.items[selected_item];
                        if item.enabled {
                            self.execute_menu_action(item.action)?;
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Execute a menu action
    fn execute_menu_action(&mut self, action: crate::menu::MenuAction) -> Result<()> {
        use crate::menu::MenuAction;

        // Close the menu first
        self.mode = Mode::Normal;

        match action {
            // File menu
            MenuAction::RunAsAdministrator => {
                if !self.is_admin {
                    self.mode = Mode::Confirm(ConfirmAction::RequestElevation);
                } else {
                    self.set_status("Already running as administrator");
                }
            }
            MenuAction::Exit => {
                self.confirm_exit();
            }

            // Command menu
            MenuAction::AddPath => {
                self.start_add_path();
            }
            MenuAction::EditPath => {
                let has_paths = match self.active_panel {
                    Panel::Machine => !self.machine_paths.is_empty(),
                    Panel::User => !self.user_paths.is_empty(),
                };
                if has_paths {
                    self.start_edit_path();
                }
            }
            MenuAction::DeleteMarked => {
                if self.has_marked_items() {
                    self.mode = Mode::Confirm(ConfirmAction::DeleteSelected);
                }
            }
            MenuAction::MarkItem => {
                self.toggle_mark();
            }
            MenuAction::UnmarkAll => {
                self.unmark_all();
            }
            MenuAction::MoveMarked => {
                self.move_marked_to_other_panel()?;
            }
            MenuAction::MoveItemUp => {
                self.move_item_up();
            }
            MenuAction::NormalizeSelected => {
                self.normalize_selected();
            }
            MenuAction::DeleteAllDead => {
                self.mode = Mode::Confirm(ConfirmAction::DeleteAllDead);
            }
            MenuAction::DeleteAllDuplicates => {
                self.mode = Mode::Confirm(ConfirmAction::DeleteAllDuplicates);
            }
            MenuAction::CreateMarkedDirectories => {
                if self.has_marked_dead_paths() {
                    self.mode = Mode::Confirm(ConfirmAction::CreateMarkedDirectories);
                } else {
                    self.set_status("No marked dead paths to create");
                }
            }

            // Options menu
            MenuAction::SelectTheme => {
                self.open_theme_selector()?;
            }
            MenuAction::ApplyFilter => {
                self.mode = Mode::FilterMenu;
                self.filter_menu_selected = 0;
            }
            MenuAction::ConnectRemote => {
                if self.connection_mode == ConnectionMode::Local {
                    self.mode = Mode::Input(InputMode::ConnectRemote);
                    self.input_buffer.clear();
                }
            }
            MenuAction::DisconnectRemote => {
                if self.connection_mode == ConnectionMode::Remote {
                    self.mode = Mode::Confirm(ConfirmAction::DisconnectRemote);
                }
            }
            MenuAction::CreateBackup => {
                self.create_backup()?;
            }
            MenuAction::RestoreBackup => {
                self.show_backup_list()?;
            }

            // Help menu
            MenuAction::KeyboardShortcuts => {
                self.mode = Mode::Help;
            }
            MenuAction::About => {
                self.mode = Mode::About;
            }
        }

        Ok(())
    }
}

pub struct Statistics {
    pub machine_total: usize,
    pub user_total: usize,
    pub machine_dead: usize,
    pub user_dead: usize,
    pub machine_duplicates: usize,
    pub user_duplicates: usize,
    pub machine_non_normalized: usize,
    pub user_non_normalized: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a test App without registry access
    fn create_test_app(machine_paths: Vec<String>, user_paths: Vec<String>) -> App {
        let machine_info = analyze_paths(&machine_paths, &user_paths);
        let user_info = analyze_paths(&user_paths, &machine_paths);

        App {
            connection_mode: ConnectionMode::Local,
            remote_connection: None,
            machine_paths: machine_paths.clone(),
            user_paths: user_paths.clone(),
            machine_info,
            user_info,
            machine_original: machine_paths,
            user_original: user_paths,
            remote_machine_paths: Vec::new(),
            remote_machine_info: Vec::new(),
            remote_machine_original: Vec::new(),
            remote_machine_selected: 0,
            remote_machine_marked: HashSet::new(),
            remote_scrollbar_state: ScrollbarState::default(),
            active_panel: Panel::User,
            machine_selected: 0,
            user_selected: 0,
            machine_marked: HashSet::new(),
            user_marked: HashSet::new(),
            mode: Mode::Normal,
            is_admin: false,
            has_changes: false,
            status_message: String::new(),
            input_buffer: String::new(),
            backup_list: Vec::new(),
            backup_selected: 0,
            machine_scrollbar_state: ScrollbarState::default(),
            user_scrollbar_state: ScrollbarState::default(),
            should_exit: false,
            viewport_height: 20,
            pending_directory: String::new(),
            processes_to_restart: Vec::new(),
            theme: Theme::default(),
            theme_arg: None,
            filter_mode: FilterMode::None,
            filter_menu_selected: 0,
            theme_list: Vec::new(),
            theme_selected: 0,
            original_theme: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_click_time: std::time::Instant::now(),
            last_click_pos: (Panel::Machine, 0),
            mode_enter_time: std::time::Instant::now(),
            file_browser_current_path: PathBuf::from("C:\\"),
            file_browser_entries: Vec::new(),
            file_browser_selected: 0,
            file_browser_scrollbar_state: ScrollbarState::new(0).position(0),
        }
    }

    #[test]
    fn test_panel_toggle() {
        assert_eq!(Panel::User.toggle(), Panel::Machine);
        assert_eq!(Panel::Machine.toggle(), Panel::User);
    }

    #[test]
    fn test_panel_scope() {
        assert_eq!(Panel::User.scope(), PathScope::User);
        assert_eq!(Panel::Machine.scope(), PathScope::Machine);
    }

    #[test]
    fn test_app_initial_state() {
        let app = create_test_app(
            vec![r"C:\Windows".to_string()],
            vec![r"C:\Users\Test".to_string()],
        );

        assert_eq!(app.machine_paths.len(), 1);
        assert_eq!(app.user_paths.len(), 1);
        assert_eq!(app.active_panel, Panel::User);
        assert_eq!(app.machine_selected, 0);
        assert_eq!(app.user_selected, 0);
        assert!(!app.has_changes);
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn test_move_selection_down() {
        let mut app = create_test_app(
            vec![],
            vec![
                r"C:\Path1".to_string(),
                r"C:\Path2".to_string(),
                r"C:\Path3".to_string(),
            ],
        );

        assert_eq!(app.user_selected, 0);

        app.move_selection(1);
        assert_eq!(app.user_selected, 1);

        app.move_selection(1);
        assert_eq!(app.user_selected, 2);

        // Should not go beyond last item
        app.move_selection(1);
        assert_eq!(app.user_selected, 2);
    }

    #[test]
    fn test_move_selection_up() {
        let mut app = create_test_app(
            vec![],
            vec![
                r"C:\Path1".to_string(),
                r"C:\Path2".to_string(),
                r"C:\Path3".to_string(),
            ],
        );

        app.user_selected = 2;

        app.move_selection(-1);
        assert_eq!(app.user_selected, 1);

        app.move_selection(-1);
        assert_eq!(app.user_selected, 0);

        // Should not go below 0
        app.move_selection(-1);
        assert_eq!(app.user_selected, 0);
    }

    #[test]
    fn test_move_selection_to() {
        let mut app = create_test_app(
            vec![],
            vec![
                r"C:\Path1".to_string(),
                r"C:\Path2".to_string(),
                r"C:\Path3".to_string(),
            ],
        );

        app.move_selection_to(2);
        assert_eq!(app.user_selected, 2);

        app.move_selection_to(0);
        assert_eq!(app.user_selected, 0);
    }

    #[test]
    fn test_toggle_mark() {
        let mut app = create_test_app(
            vec![],
            vec![
                r"C:\Path1".to_string(),
                r"C:\Path2".to_string(),
                r"C:\Path3".to_string(),
            ],
        );

        // Mark first item (toggle_mark also moves selection down)
        app.user_selected = 0;
        app.toggle_mark();
        assert!(app.user_marked.contains(&0));
        assert_eq!(app.user_selected, 1); // Selection moved down

        // Go back and unmark first item
        app.user_selected = 0;
        app.toggle_mark();
        assert!(!app.user_marked.contains(&0));

        // Test on machine panel
        app.active_panel = Panel::Machine;
        app.machine_paths = vec![r"C:\Windows".to_string(), r"C:\Temp".to_string()];
        app.machine_selected = 0;
        app.toggle_mark();
        assert!(app.machine_marked.contains(&0));
        assert_eq!(app.machine_selected, 1); // Selection moved down
    }

    #[test]
    fn test_has_marked_items() {
        let mut app = create_test_app(
            vec![r"C:\Windows".to_string()],
            vec![r"C:\Path1".to_string()],
        );

        assert!(!app.has_marked_items());

        app.user_marked.insert(0);
        assert!(app.has_marked_items());

        app.user_marked.clear();
        app.active_panel = Panel::Machine;
        app.machine_marked.insert(0);
        assert!(app.has_marked_items());
    }

    #[test]
    fn test_has_marked_dead_paths() {
        let mut app = create_test_app(
            vec![],
            vec![
                r"C:\Windows".to_string(),
                r"C:\NonExistentPath123456".to_string(),
            ],
        );
        app.reanalyze();

        // No marked items yet
        assert!(!app.has_marked_dead_paths());

        // Mark a valid path
        app.user_marked.insert(0);
        assert!(!app.has_marked_dead_paths());

        // Mark a dead path
        app.user_marked.insert(1);
        assert!(app.has_marked_dead_paths());
    }

    #[test]
    fn test_mode_transitions() {
        let mut app = create_test_app(vec![], vec![]);

        assert_eq!(app.mode, Mode::Normal);

        app.mode = Mode::Help;
        assert_eq!(app.mode, Mode::Help);

        app.mode = Mode::Confirm(ConfirmAction::Exit);
        assert_eq!(app.mode, Mode::Confirm(ConfirmAction::Exit));

        app.mode = Mode::Input(InputMode::AddPath);
        assert_eq!(app.mode, Mode::Input(InputMode::AddPath));

        app.mode = Mode::BackupList;
        assert_eq!(app.mode, Mode::BackupList);
    }

    #[test]
    fn test_can_create_directory() {
        // Valid paths
        assert!(App::can_create_directory(r"C:\NewFolder"));
        assert!(App::can_create_directory(r"C:\Path\To\NewFolder"));

        // Network paths should not be creatable
        assert!(!App::can_create_directory(r"\\server\share\folder"));

        // Paths with invalid characters
        assert!(!App::can_create_directory(r"C:\Invalid|Path"));
        assert!(!App::can_create_directory(r"C:\Invalid<Path"));
        assert!(!App::can_create_directory(r"C:\Invalid>Path"));
    }

    #[test]
    fn test_get_statistics() {
        let app = create_test_app(
            vec![r"C:\Windows".to_string(), r"C:\NonExistent1".to_string()],
            vec![
                r"C:\Users".to_string(),
                r"C:\windows".to_string(), // Duplicate of machine path
                r"C:\NonExistent2".to_string(),
            ],
        );

        let stats = app.get_statistics();

        assert_eq!(stats.machine_total, 2);
        assert_eq!(stats.user_total, 3);
        assert_eq!(stats.machine_dead, 1); // C:\NonExistent1
        assert_eq!(stats.user_dead, 1); // C:\NonExistent2
                                        // Duplicates: C:\windows is duplicate of C:\Windows
        assert!(stats.machine_duplicates > 0 || stats.user_duplicates > 0);
    }

    #[test]
    fn test_confirm_exit() {
        let mut app = create_test_app(vec![], vec![]);

        // When there are no changes, should exit directly
        app.confirm_exit();
        assert!(app.should_exit);

        // When there are changes, should confirm first
        let mut app = create_test_app(vec![], vec![]);
        app.has_changes = true;
        app.confirm_exit();
        assert_eq!(app.mode, Mode::Confirm(ConfirmAction::Exit));
    }

    #[test]
    fn test_delete_marked() {
        let mut app = create_test_app(
            vec![],
            vec![
                r"C:\Path1".to_string(),
                r"C:\Path2".to_string(),
                r"C:\Path3".to_string(),
            ],
        );

        // Mark items 0 and 2
        app.user_marked.insert(0);
        app.user_marked.insert(2);

        let result = app.delete_marked();
        assert!(result.is_ok());

        // Should have 1 path remaining (the middle one)
        assert_eq!(app.user_paths.len(), 1);
        assert_eq!(app.user_paths[0], r"C:\Path2");

        // Marked set should be cleared
        assert!(app.user_marked.is_empty());

        // Should have changes
        assert!(app.has_changes);
    }

    #[test]
    fn test_normalize_selected() {
        // Test with an absolute path that should be collapsed to an env var
        let systemroot = std::env::var("SYSTEMROOT").unwrap_or_default();
        if systemroot.is_empty() {
            return; // Skip test if SYSTEMROOT is not set
        }

        let absolute_path = format!(r"{}\System32", systemroot);
        let mut app = create_test_app(vec![], vec![absolute_path.clone()]);
        app.reanalyze();

        app.user_selected = 0;
        // normalize_selected works on marked paths, so mark it first
        app.user_marked.insert(0);
        app.normalize_selected();

        // Path should be normalized to use environment variables
        assert!(
            app.user_paths[0].contains('%'),
            "Expected normalized path to contain env var, got: {}",
            app.user_paths[0]
        );
        assert!(app.has_changes);
    }

    #[test]
    fn test_start_add_path() {
        let mut app = create_test_app(vec![], vec![]);

        app.start_add_path();

        // Now opens file browser instead of text input
        assert_eq!(app.mode, Mode::FileBrowser);
        assert_eq!(app.file_browser_selected, 0);
        // Should have loaded directory entries (at least ".." if not at root)
        // The exact number depends on the current directory, so we just check it's initialized
    }

    #[test]
    fn test_start_edit_path() {
        let mut app = create_test_app(vec![], vec![r"C:\ExistingPath".to_string()]);

        app.user_selected = 0;
        app.start_edit_path();

        assert_eq!(app.mode, Mode::Input(InputMode::EditPath));
        assert_eq!(app.input_buffer, r"C:\ExistingPath");
    }

    #[test]
    fn test_update_viewport_height() {
        let mut app = create_test_app(vec![], vec![]);

        // Viewport height = terminal_height - 9 (for UI elements)
        app.update_viewport_height(30);
        assert_eq!(app.viewport_height, 21); // 30 - 9

        app.update_viewport_height(50);
        assert_eq!(app.viewport_height, 41); // 50 - 9
    }

    #[test]
    fn test_reanalyze_adjusts_selection() {
        let mut app = create_test_app(
            vec![],
            vec![r"C:\Path1".to_string(), r"C:\Path2".to_string()],
        );

        // Select the last item
        app.user_selected = 1;

        // Remove the last path and reanalyze
        app.user_paths.pop();
        app.reanalyze();

        // Selection should be adjusted to 0 (last valid index)
        assert_eq!(app.user_selected, 0);
    }

    #[test]
    fn test_empty_paths() {
        let app = create_test_app(vec![], vec![]);

        assert_eq!(app.machine_paths.len(), 0);
        assert_eq!(app.user_paths.len(), 0);

        let stats = app.get_statistics();
        assert_eq!(stats.machine_total, 0);
        assert_eq!(stats.user_total, 0);
    }

    #[test]
    fn test_panel_switching() {
        let mut app = create_test_app(
            vec![r"C:\Machine".to_string()],
            vec![r"C:\User".to_string()],
        );

        assert_eq!(app.active_panel, Panel::User);

        app.active_panel = app.active_panel.toggle();
        assert_eq!(app.active_panel, Panel::Machine);

        app.active_panel = app.active_panel.toggle();
        assert_eq!(app.active_panel, Panel::User);
    }

    #[test]
    fn test_marked_items_on_both_panels() {
        let mut app = create_test_app(
            vec![r"C:\Machine1".to_string(), r"C:\Machine2".to_string()],
            vec![r"C:\User1".to_string(), r"C:\User2".to_string()],
        );

        // Mark items on user panel
        app.active_panel = Panel::User;
        app.user_marked.insert(0);
        app.user_marked.insert(1);

        // Mark items on machine panel
        app.active_panel = Panel::Machine;
        app.machine_marked.insert(0);

        // Check both panels have marked items
        app.active_panel = Panel::User;
        assert!(app.has_marked_items());

        app.active_panel = Panel::Machine;
        assert!(app.has_marked_items());
    }

    #[test]
    fn test_calculate_path_length() {
        // Test empty paths
        let app = create_test_app(vec![], vec![]);
        assert_eq!(app.calculate_path_length(Panel::Machine), 0);
        assert_eq!(app.calculate_path_length(Panel::User), 0);

        // Test single path (no separators)
        let app = create_test_app(vec![r"C:\Windows".to_string()], vec![]);
        assert_eq!(app.calculate_path_length(Panel::Machine), 10); // "C:\Windows".len()
        assert_eq!(app.calculate_path_length(Panel::User), 0);

        // Test multiple paths with separators
        // "C:\Windows" (10) + ";" (1) + "C:\Program Files" (16) = 27
        let app = create_test_app(
            vec![r"C:\Windows".to_string(), r"C:\Program Files".to_string()],
            vec![],
        );
        assert_eq!(app.calculate_path_length(Panel::Machine), 27);

        // Test both panels with different lengths
        // Machine: "C:\A" (4) + ";" + "C:\B" (4) = 9
        // User: "C:\User\Path" (12) + ";" + "C:\Another" (10) + ";" + "C:\Third" (8) = 32
        let app = create_test_app(
            vec![r"C:\A".to_string(), r"C:\B".to_string()],
            vec![
                r"C:\User\Path".to_string(),
                r"C:\Another".to_string(),
                r"C:\Third".to_string(),
            ],
        );
        assert_eq!(app.calculate_path_length(Panel::Machine), 9);
        assert_eq!(app.calculate_path_length(Panel::User), 32);

        // Test path that exceeds Windows limit
        // Create a path string that's longer than 2047 characters
        let long_path = "C:\\".to_string() + &"VeryLongDirectoryName".repeat(100);
        let app = create_test_app(vec![long_path.clone()], vec![]);
        let length = app.calculate_path_length(Panel::Machine);
        assert!(length > 2047, "Path length {} should exceed 2047", length);
        assert_eq!(length, long_path.len());
    }
}
