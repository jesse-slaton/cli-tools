/// Menu system for Path Commander
/// Provides drop-down menus similar to Midnight Commander

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    pub label: String,
    pub shortcut: Option<String>,
    pub action: MenuAction,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    // File menu
    RunAsAdministrator,
    Exit,

    // Command menu
    AddPath,
    EditPath,
    DeleteMarked,
    MarkItem,
    UnmarkAll,
    MoveMarked,
    MoveItemUp,
    NormalizeSelected,
    DeleteAllDead,
    DeleteAllDuplicates,
    CreateMarkedDirectories,

    // Options menu
    SelectTheme,
    ApplyFilter,
    ConnectRemote,
    DisconnectRemote,
    CreateBackup,
    RestoreBackup,

    // Help menu
    KeyboardShortcuts,
    About,
}

pub struct Menu {
    pub name: String,
    pub accelerator: char, // The letter that activates this menu with Alt
    pub items: Vec<MenuItem>,
}

impl Menu {
    pub fn new(name: &str, accelerator: char) -> Self {
        Self {
            name: name.to_string(),
            accelerator,
            items: Vec::new(),
        }
    }

    pub fn add_item(&mut self, label: &str, shortcut: Option<&str>, action: MenuAction) {
        self.items.push(MenuItem {
            label: label.to_string(),
            shortcut: shortcut.map(|s| s.to_string()),
            action,
            enabled: true,
        });
    }
}

/// Get all menus for the application
pub fn get_menus(connection_mode: crate::app::ConnectionMode) -> Vec<Menu> {
    let mut menus = Vec::new();

    // File menu
    let mut file_menu = Menu::new("File", 'f');
    file_menu.add_item(
        "Run as Administrator",
        Some("Ctrl+E"),
        MenuAction::RunAsAdministrator,
    );
    file_menu.add_item("Exit", Some("F10"), MenuAction::Exit);
    menus.push(file_menu);

    // Command menu
    let mut command_menu = Menu::new("Command", 'c');
    command_menu.add_item("Add Path", Some("F4"), MenuAction::AddPath);
    command_menu.add_item("Edit Path", Some("Enter"), MenuAction::EditPath);
    command_menu.add_item("Delete Marked", Some("F3/Del"), MenuAction::DeleteMarked);
    command_menu.add_item("Mark/Unmark", Some("F2/Space"), MenuAction::MarkItem);
    command_menu.add_item("Unmark All", Some("Ctrl+Shift+U"), MenuAction::UnmarkAll);

    // Dynamic label based on connection mode
    let f5_label = if connection_mode == crate::app::ConnectionMode::Remote {
        "Copy Marked to Other Computer"
    } else {
        "Move Marked to Other Panel"
    };
    command_menu.add_item(f5_label, Some("F5"), MenuAction::MoveMarked);
    command_menu.add_item("Move Item Up", Some("F6"), MenuAction::MoveItemUp);
    command_menu.add_item(
        "Normalize Selected",
        Some("F9"),
        MenuAction::NormalizeSelected,
    );
    command_menu.add_item(
        "Delete All Dead Paths",
        Some("F8"),
        MenuAction::DeleteAllDead,
    );
    command_menu.add_item(
        "Delete All Duplicates",
        Some("F7"),
        MenuAction::DeleteAllDuplicates,
    );
    command_menu.add_item(
        "Create Marked Directories",
        Some("F10"),
        MenuAction::CreateMarkedDirectories,
    );
    menus.push(command_menu);

    // Options menu
    let mut options_menu = Menu::new("Options", 'o');
    options_menu.add_item("Select Theme", Some("t"), MenuAction::SelectTheme);
    options_menu.add_item("Apply Filter", Some("/"), MenuAction::ApplyFilter);
    options_menu.add_item(
        "Connect to Remote",
        Some("Ctrl+O"),
        MenuAction::ConnectRemote,
    );
    options_menu.add_item(
        "Disconnect Remote",
        Some("Ctrl+O"),
        MenuAction::DisconnectRemote,
    );
    options_menu.add_item("Create Backup", Some("Ctrl+B"), MenuAction::CreateBackup);
    options_menu.add_item("Restore Backup", Some("Ctrl+R"), MenuAction::RestoreBackup);
    menus.push(options_menu);

    // Help menu
    let mut help_menu = Menu::new("Help", 'h');
    help_menu.add_item("Help", Some("F1"), MenuAction::KeyboardShortcuts);
    help_menu.add_item("About", None, MenuAction::About);
    menus.push(help_menu);

    menus
}

/// Update menu item enabled states based on app state
pub fn update_menu_enabled_states(
    menus: &mut [Menu],
    is_admin: bool,
    has_marked: bool,
    has_marked_dead: bool,
    has_selection: bool,
    is_remote: bool,
    _has_changes: bool,
) {
    for menu in menus.iter_mut() {
        for item in menu.items.iter_mut() {
            item.enabled = match item.action {
                MenuAction::RunAsAdministrator => !is_admin,
                MenuAction::DeleteMarked | MenuAction::MoveMarked | MenuAction::UnmarkAll => {
                    has_marked
                }
                MenuAction::CreateMarkedDirectories => has_marked_dead,
                MenuAction::EditPath | MenuAction::NormalizeSelected | MenuAction::MoveItemUp => {
                    has_selection
                }
                MenuAction::DisconnectRemote => is_remote,
                MenuAction::ConnectRemote => !is_remote,
                _ => true,
            };
        }
    }
}
