use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, MouseButton};
use ratatui::{layout::Rect, widgets::ScrollbarState};
use std::collections::HashSet;

use crate::backup::{self, PathBackup};
use crate::path_analyzer::{analyze_paths, normalize_path, PathInfo};
use crate::permissions;
use crate::registry::{self, PathScope};

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmAction {
    Exit,
    DeleteSelected,
    DeleteAllDead,
    DeleteAllDuplicates,
    ApplyChanges,
    RestoreBackup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    AddPath,
    EditPath,
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
}

impl App {
    pub fn new() -> Result<Self> {
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
        })
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Result<()> {
        match self.mode {
            Mode::Normal => self.handle_normal_input(key),
            Mode::Help => self.handle_help_input(key),
            Mode::Confirm(action) => self.handle_confirm_input(key, action),
            Mode::Input(input_mode) => self.handle_input_mode(key, input_mode),
            Mode::BackupList => self.handle_backup_list_input(key),
        }
    }

    fn handle_normal_input(&mut self, key: KeyEvent) -> Result<()> {
        match (key.code, key.modifiers) {
            // Navigation
            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => self.move_selection(-1),
            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => self.move_selection(1),
            (KeyCode::PageUp, _) => self.move_selection(-10),
            (KeyCode::PageDown, _) => self.move_selection(10),
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

    fn handle_confirm_input(&mut self, key: KeyEvent, action: ConfirmAction) -> Result<()> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                self.mode = Mode::Normal;
                match action {
                    ConfirmAction::Exit => {
                        // Will be handled by main loop
                    }
                    ConfirmAction::DeleteSelected => self.delete_marked()?,
                    ConfirmAction::DeleteAllDead => self.delete_all_dead()?,
                    ConfirmAction::DeleteAllDuplicates => self.delete_all_duplicates()?,
                    ConfirmAction::ApplyChanges => self.apply_changes()?,
                    ConfirmAction::RestoreBackup => self.restore_selected_backup()?,
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
        // Only handle mouse in Normal mode
        if self.mode != Mode::Normal {
            return Ok(());
        }

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.handle_mouse_click(mouse.column, mouse.row, terminal_size, mouse.modifiers)?;
            }
            MouseEventKind::ScrollUp => {
                // Scroll up = content moves up = view moves down in list
                self.move_selection(1);
            }
            MouseEventKind::ScrollDown => {
                // Scroll down = content moves down = view moves up in list
                self.move_selection(-1);
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_mouse_click(&mut self, x: u16, y: u16, terminal_size: Rect, modifiers: KeyModifiers) -> Result<()> {
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
            let content_height = (content_end - content_start) as u16;
            let scrollbar_height = content_height.saturating_sub(border_top + border_bottom) as usize;

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

        // Check if click is on checkbox area
        // Checkbox is at relative_x = 1 (border) to 5 (border + "[ ] ")
        let checkbox_start = 1; // After left border
        let checkbox_end = 5; // "[ ] " is 4 chars

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

    // Navigation helpers
    fn move_selection(&mut self, delta: i32) {
        match self.active_panel {
            Panel::Machine => {
                let new_pos = (self.machine_selected as i32 + delta)
                    .max(0)
                    .min(self.machine_paths.len().saturating_sub(1) as i32);
                self.machine_selected = new_pos as usize;
                self.machine_scrollbar_state = self.machine_scrollbar_state.position(self.machine_selected);
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
                self.machine_scrollbar_state = self.machine_scrollbar_state.position(self.machine_selected);
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

        self.machine_paths.retain(|p| {
            crate::path_analyzer::path_exists(p)
        });
        self.user_paths.retain(|p| {
            crate::path_analyzer::path_exists(p)
        });

        let deleted = (machine_before - self.machine_paths.len())
            + (user_before - self.user_paths.len());

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
        self.set_status(&format!("Moved {} path(s) to {}", count, self.active_panel.toggle().scope().as_str()));
        Ok(())
    }

    fn move_item_up(&mut self) {
        match self.active_panel {
            Panel::Machine => {
                if self.machine_selected > 0 {
                    self.machine_paths.swap(self.machine_selected, self.machine_selected - 1);
                    self.machine_selected -= 1;
                    self.has_changes = true;
                    self.reanalyze();
                }
            }
            Panel::User => {
                if self.user_selected > 0 {
                    self.user_paths.swap(self.user_selected, self.user_selected - 1);
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

        self.set_status("Changes applied successfully!");
        Ok(())
    }

    pub fn confirm_exit(&mut self) -> bool {
        if self.has_changes {
            self.mode = Mode::Confirm(ConfirmAction::Exit);
            false
        } else {
            true
        }
    }

    fn reanalyze(&mut self) {
        self.user_info = analyze_paths(&self.user_paths, &self.machine_paths);
        self.machine_info = analyze_paths(&self.machine_paths, &self.user_paths);

        // Update scrollbar content lengths
        self.machine_scrollbar_state = self.machine_scrollbar_state
            .content_length(self.machine_paths.len());
        self.user_scrollbar_state = self.user_scrollbar_state
            .content_length(self.user_paths.len());

        // Adjust selection if out of bounds
        if self.machine_selected >= self.machine_paths.len() && !self.machine_paths.is_empty() {
            self.machine_selected = self.machine_paths.len() - 1;
            self.machine_scrollbar_state = self.machine_scrollbar_state.position(self.machine_selected);
        }
        if self.user_selected >= self.user_paths.len() && !self.user_paths.is_empty() {
            self.user_selected = self.user_paths.len() - 1;
            self.user_scrollbar_state = self.user_scrollbar_state.position(self.user_selected);
        }
    }

    fn set_status(&mut self, message: &str) {
        self.status_message = message.to_string();
    }

    pub fn get_statistics(&self) -> Statistics {
        let machine_dead = self.machine_info.iter().filter(|i| !i.exists).count();
        let user_dead = self.user_info.iter().filter(|i| !i.exists).count();

        let machine_duplicates = self.machine_info.iter().filter(|i| i.is_duplicate).count();
        let user_duplicates = self.user_info.iter().filter(|i| i.is_duplicate).count();

        let machine_non_normalized = self.machine_info.iter().filter(|i| i.needs_normalization).count();
        let user_non_normalized = self.user_info.iter().filter(|i| i.needs_normalization).count();

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
