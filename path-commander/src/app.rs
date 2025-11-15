use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{layout::Rect, widgets::ScrollbarState};
use std::collections::HashSet;

use crate::backup::{self, PathBackup};
use crate::path_analyzer::{analyze_paths, normalize_path, PathInfo};
use crate::permissions;
use crate::registry::{self, PathScope};
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Confirm(ConfirmAction),
    Input(InputMode),
    BackupList,
    ProcessRestartInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmAction {
    Exit,
    DeleteSelected,
    DeleteAllDead,
    DeleteAllDuplicates,
    ApplyChanges,
    RestoreBackup,
    CreateSingleDirectory,
    CreateMarkedDirectories,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    AddPath,
    EditPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    None,
    Dead,
    Duplicates,
    NonNormalized,
    Valid,
}

pub struct App {
    pub machine_paths: Vec<String>,
    pub user_paths: Vec<String>,
    pub machine_info: Vec<PathInfo>,
    pub user_info: Vec<PathInfo>,
    pub machine_original: Vec<String>,
    pub user_original: Vec<String>,
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
    pub filter_mode: FilterMode,           // Current filter mode (None, Dead, Duplicates, etc.)
    last_click_time: std::time::Instant,   // Time of last mouse click for double-click detection
    last_click_pos: (Panel, usize),        // Panel and row of last click
}

impl App {
    pub fn new(theme: Theme) -> Result<Self> {
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
            machine_scrollbar_state: ScrollbarState::new(machine_paths.len()).position(0),
            user_scrollbar_state: ScrollbarState::new(user_paths.len()).position(0),
            machine_paths: machine_paths.clone(),
            user_paths: user_paths.clone(),
            machine_info,
            user_info,
            machine_original: machine_paths,
            user_original: user_paths,
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
            filter_mode: FilterMode::None,
            last_click_time: std::time::Instant::now(),
            last_click_pos: (Panel::Machine, 0),
        })
    }

    /// Update viewport height based on terminal size
    /// Calculates visible lines in panel: terminal_height - header(3) - status(3) - hints(2) - borders(2)
    pub fn update_viewport_height(&mut self, terminal_height: u16) {
        // Layout: Header(3) + Content + Status(3) + Hints(2)
        // Panel has top and bottom borders (2)
        // Viewport = terminal_height - 3 - 3 - 2 - 2 = terminal_height - 10
        self.viewport_height = terminal_height.saturating_sub(10).max(1);
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Result<()> {
        match self.mode {
            Mode::Normal => self.handle_normal_input(key),
            Mode::Help => self.handle_help_input(key),
            Mode::Confirm(action) => self.handle_confirm_input(key, action),
            Mode::Input(input_mode) => self.handle_input_mode(key, input_mode),
            Mode::BackupList => self.handle_backup_list_input(key),
            Mode::ProcessRestartInfo => self.handle_process_restart_info_input(key),
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
                // Create marked dead directories (moved from F11)
                if self.has_marked_dead_paths() {
                    self.mode = Mode::Confirm(ConfirmAction::CreateMarkedDirectories);
                } else {
                    self.set_status("No marked dead paths to create");
                }
            }
            (KeyCode::F(11), KeyModifiers::SHIFT) => {
                // Toggle Non-normalized filter
                self.toggle_filter(FilterMode::NonNormalized);
            }
            (KeyCode::F(11), KeyModifiers::CONTROL) => {
                // Toggle Valid filter
                self.toggle_filter(FilterMode::Valid);
            }
            (KeyCode::F(11), _) => {
                // Toggle Dead paths filter
                self.toggle_filter(FilterMode::Dead);
            }
            (KeyCode::F(12), _) => {
                // Toggle Duplicates filter
                self.toggle_filter(FilterMode::Duplicates);
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
                    self.mode = Mode::Confirm(ConfirmAction::ApplyChanges);
                } else {
                    self.set_status("No changes to save");
                }
            }
            (KeyCode::Char('b'), KeyModifiers::CONTROL) => self.create_backup()?,
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
                self.mode = Mode::Normal;
                match action {
                    ConfirmAction::Exit => {
                        self.should_exit = true;
                    }
                    ConfirmAction::DeleteSelected => self.delete_marked()?,
                    ConfirmAction::DeleteAllDead => self.delete_all_dead()?,
                    ConfirmAction::DeleteAllDuplicates => self.delete_all_duplicates()?,
                    ConfirmAction::ApplyChanges => self.apply_changes()?,
                    ConfirmAction::RestoreBackup => self.restore_selected_backup()?,
                    ConfirmAction::CreateSingleDirectory => self.create_single_directory()?,
                    ConfirmAction::CreateMarkedDirectories => self.create_marked_directories()?,
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
                self.mode = Mode::Normal;
                match input_mode {
                    InputMode::AddPath => self.add_path_from_input()?,
                    InputMode::EditPath => self.update_path_from_input()?,
                }
                self.input_buffer.clear();
            }
            KeyCode::Esc => {
                self.mode = Mode::Normal;
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
                        // Check if click is on key hints area (bottom 2 rows)
                        let hints_start = terminal_size.height.saturating_sub(2);
                        if mouse.row >= hints_start {
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
        // Hint text: "F1 Help │ F2 Mark │ F3 Del │ F4 Add │ F5 Move │ F9 Normalize │ Ctrl+S Save │ Ctrl+B Backup │ Q Quit"
        // Total length: 100 characters
        let hint_text_len = 100;
        let start_x = (width.saturating_sub(hint_text_len)) / 2;

        // Calculate relative position
        if x < start_x {
            return Ok(());
        }

        let relative_x = x - start_x;

        // Map click positions to keys (exact character positions):
        // "F1" (2) + " Help │ " (8) = 0-9
        // "F2" (2) + " Mark │ " (8) = 10-19
        // "F3" (2) + " Del │ " (7) = 20-28
        // "F4" (2) + " Add │ " (7) = 29-37
        // "F5" (2) + " Move │ " (8) = 38-47
        // "F9" (2) + " Normalize │ " (13) = 48-62
        // "Ctrl+S" (6) + " Save │ " (8) = 63-76
        // "Ctrl+B" (6) + " Backup │ " (10) = 77-92
        // "Q" (1) + " Quit" (5) = 93-99

        match relative_x {
            0..=9 => self.mode = Mode::Help, // F1 Help
            10..=19 => self.toggle_mark(),   // F2 Mark
            20..=28 => {
                // F3 Del
                if self.has_marked_items() {
                    self.mode = Mode::Confirm(ConfirmAction::DeleteSelected);
                }
            }
            29..=37 => self.start_add_path(), // F4 Add
            38..=47 => {
                let _ = self.move_marked_to_other_panel();
            } // F5 Move
            48..=62 => self.normalize_selected(), // F9 Normalize
            63..=76 => {
                // Ctrl+S Save
                if self.has_changes {
                    self.mode = Mode::Confirm(ConfirmAction::ApplyChanges);
                } else {
                    self.set_status("No changes to save");
                }
            }
            77..=92 => {
                let _ = self.create_backup();
            } // Ctrl+B Backup
            93..=99 => self.confirm_exit(), // Q Quit
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
                        ConfirmAction::RestoreBackup => self.restore_selected_backup()?,
                        ConfirmAction::CreateSingleDirectory => self.create_single_directory()?,
                        ConfirmAction::CreateMarkedDirectories => {
                            self.create_marked_directories()?
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

    fn has_marked_items(&self) -> bool {
        !self.machine_marked.is_empty() || !self.user_marked.is_empty()
    }

    fn has_marked_dead_paths(&self) -> bool {
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

        self.reanalyze();
        self.has_changes = true;
        self.set_status(&format!("Deleted {} path(s)", deleted_count));
        Ok(())
    }

    fn delete_all_dead(&mut self) -> Result<()> {
        let machine_before = self.machine_paths.len();
        let user_before = self.user_paths.len();

        self.machine_paths
            .retain(|p| crate::path_analyzer::path_exists(p));
        self.user_paths
            .retain(|p| crate::path_analyzer::path_exists(p));

        let deleted =
            (machine_before - self.machine_paths.len()) + (user_before - self.user_paths.len());

        self.reanalyze();
        self.has_changes = true;
        self.set_status(&format!("Deleted {} dead path(s)", deleted));
        Ok(())
    }

    fn delete_all_duplicates(&mut self) -> Result<()> {
        let mut seen = HashSet::new();
        let mut deleted = 0;

        // Keep first occurrence of each path (case-insensitive, normalized)
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

        self.reanalyze();
        self.has_changes = true;
        self.set_status(&format!("Deleted {} duplicate path(s)", deleted));
        Ok(())
    }

    fn normalize_selected(&mut self) {
        let mut normalized_count = 0;

        match self.active_panel {
            Panel::Machine => {
                for idx in &self.machine_marked {
                    if let Some(path) = self.machine_paths.get_mut(*idx) {
                        let normalized = normalize_path(path);
                        if &normalized != path {
                            *path = normalized;
                            normalized_count += 1;
                        }
                    }
                }
                self.machine_marked.clear();
            }
            Panel::User => {
                for idx in &self.user_marked {
                    if let Some(path) = self.user_paths.get_mut(*idx) {
                        let normalized = normalize_path(path);
                        if &normalized != path {
                            *path = normalized;
                            normalized_count += 1;
                        }
                    }
                }
                self.user_marked.clear();
            }
        }

        if normalized_count > 0 {
            self.reanalyze();
            self.has_changes = true;
            self.set_status(&format!("Normalized {} path(s)", normalized_count));
        }
    }

    fn move_marked_to_other_panel(&mut self) -> Result<()> {
        let (from_paths, to_paths, from_marked) = match self.active_panel {
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

        let mut moved = Vec::new();
        let mut indices: Vec<_> = from_marked.iter().copied().collect();
        indices.sort_unstable_by(|a, b| b.cmp(a)); // Reverse order to maintain indices

        for idx in indices {
            if let Some(path) = from_paths.get(idx) {
                moved.push(path.clone());
            }
        }

        // Remove from source (in reverse order)
        let mut new_from = Vec::new();
        for (idx, path) in from_paths.iter().enumerate() {
            if !from_marked.contains(&idx) {
                new_from.push(path.clone());
            }
        }
        *from_paths = new_from;

        // Add to destination
        to_paths.extend(moved.iter().cloned());

        let count = moved.len();
        from_marked.clear();

        self.reanalyze();
        self.has_changes = true;
        self.set_status(&format!(
            "Moved {} path(s) to {}",
            count,
            self.active_panel.toggle().scope().as_str()
        ));
        Ok(())
    }

    fn move_item_up(&mut self) {
        match self.active_panel {
            Panel::Machine => {
                if self.machine_selected > 0 {
                    self.machine_paths
                        .swap(self.machine_selected, self.machine_selected - 1);
                    self.machine_selected -= 1;
                    self.has_changes = true;
                    self.reanalyze();
                }
            }
            Panel::User => {
                if self.user_selected > 0 {
                    self.user_paths
                        .swap(self.user_selected, self.user_selected - 1);
                    self.user_selected -= 1;
                    self.has_changes = true;
                    self.reanalyze();
                }
            }
        }
    }

    fn start_add_path(&mut self) {
        self.mode = Mode::Input(InputMode::AddPath);
        self.input_buffer.clear();
    }

    fn start_edit_path(&mut self) {
        let current_path = match self.active_panel {
            Panel::Machine => self.machine_paths.get(self.machine_selected),
            Panel::User => self.user_paths.get(self.user_selected),
        };

        if let Some(path) = current_path {
            self.input_buffer = path.clone();
            self.mode = Mode::Input(InputMode::EditPath);
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
        match self.active_panel {
            Panel::Machine => {
                if !self.is_admin {
                    self.set_status("Need admin rights to add MACHINE paths");
                    return Ok(());
                }
                self.machine_paths.push(self.input_buffer.clone());
            }
            Panel::User => {
                self.user_paths.push(self.input_buffer.clone());
            }
        }

        self.reanalyze();
        self.has_changes = true;
        self.set_status("Path added");
        Ok(())
    }

    fn update_path_from_input(&mut self) -> Result<()> {
        if self.input_buffer.is_empty() {
            return Ok(());
        }

        match self.active_panel {
            Panel::Machine => {
                if !self.is_admin {
                    self.set_status("Need admin rights to edit MACHINE paths");
                    return Ok(());
                }
                if let Some(path) = self.machine_paths.get_mut(self.machine_selected) {
                    *path = self.input_buffer.clone();
                }
            }
            Panel::User => {
                if let Some(path) = self.user_paths.get_mut(self.user_selected) {
                    *path = self.input_buffer.clone();
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
        self.has_changes = false;

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
    fn create_directory(path: &str) -> Result<()> {
        use std::fs;
        use std::path::Path;

        if path.is_empty() {
            return Err(anyhow::anyhow!("Path is empty"));
        }

        // Expand environment variables
        let expanded = normalize_path(path);
        let path_obj = Path::new(&expanded);

        // Create directory with parents
        fs::create_dir_all(path_obj)?;

        Ok(())
    }

    /// Create the pending directory and add the path
    fn create_single_directory(&mut self) -> Result<()> {
        if self.pending_directory.is_empty() {
            return Ok(());
        }

        match Self::create_directory(&self.pending_directory) {
            Ok(()) => {
                // Directory created successfully - now add the path
                match self.active_panel {
                    Panel::Machine => {
                        if !self.is_admin {
                            self.set_status("Need admin rights to add MACHINE paths");
                            return Ok(());
                        }
                        self.machine_paths.push(self.pending_directory.clone());
                    }
                    Panel::User => {
                        self.user_paths.push(self.pending_directory.clone());
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
            Panel::User => self
                .user_marked
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
                .collect(),
        };

        // Try to create each directory
        for (_idx, path) in marked_paths {
            if !Self::can_create_directory(&path) {
                skipped_count += 1;
                continue;
            }

            match Self::create_directory(&path) {
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
        if self.machine_selected >= self.machine_paths.len() && !self.machine_paths.is_empty() {
            self.machine_selected = self.machine_paths.len() - 1;
            self.machine_scrollbar_state =
                self.machine_scrollbar_state.position(self.machine_selected);
        }
        if self.user_selected >= self.user_paths.len() && !self.user_paths.is_empty() {
            self.user_selected = self.user_paths.len() - 1;
            self.user_scrollbar_state = self.user_scrollbar_state.position(self.user_selected);
        }
    }

    fn set_status(&mut self, message: &str) {
        self.status_message = message.to_string();
    }

    // Filtering functions
    fn toggle_filter(&mut self, mode: FilterMode) {
        if self.filter_mode == mode {
            // Toggle off - clear filter
            self.filter_mode = FilterMode::None;
            self.set_status("Filter cleared");
        } else {
            // Toggle on - set new filter
            self.filter_mode = mode;
            let filter_name = match mode {
                FilterMode::Dead => "Dead paths",
                FilterMode::Duplicates => "Duplicates",
                FilterMode::NonNormalized => "Non-normalized",
                FilterMode::Valid => "Valid paths",
                FilterMode::None => "None",
            };
            self.set_status(&format!("Filter: {}", filter_name));
        }
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
}

#[allow(dead_code)]
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
            machine_paths: machine_paths.clone(),
            user_paths: user_paths.clone(),
            machine_info,
            user_info,
            machine_original: machine_paths,
            user_original: user_paths,
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
            filter_mode: FilterMode::None,
            last_click_time: std::time::Instant::now(),
            last_click_pos: (Panel::Machine, 0),
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
        let mut app = create_test_app(vec![], vec![r"%SYSTEMROOT%".to_string()]);
        app.reanalyze();

        app.user_selected = 0;
        // normalize_selected works on marked paths, so mark it first
        app.user_marked.insert(0);
        app.normalize_selected();

        // Path should be normalized (no % signs)
        assert!(!app.user_paths[0].contains('%'));
        assert!(app.has_changes);
    }

    #[test]
    fn test_start_add_path() {
        let mut app = create_test_app(vec![], vec![]);

        app.start_add_path();

        assert_eq!(app.mode, Mode::Input(InputMode::AddPath));
        assert_eq!(app.input_buffer, "");
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

        // Viewport height = terminal_height - 10 (for UI elements)
        app.update_viewport_height(30);
        assert_eq!(app.viewport_height, 20); // 30 - 10

        app.update_viewport_height(50);
        assert_eq!(app.viewport_height, 40); // 50 - 10
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
}
