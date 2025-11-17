use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, Wrap},
    Frame,
};

use crate::app::{App, ConfirmAction, InputMode, Mode, Panel};
use crate::menu;
use crate::path_analyzer::PathStatus;
use crate::theme::Theme;

pub struct UI;

impl UI {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, f: &mut Frame, app: &App) {
        match app.mode {
            Mode::Help => self.render_help(f, app),
            Mode::About => self.render_about(f, app),
            Mode::Confirm(action) => self.render_confirm(f, app, action),
            Mode::BackupList => self.render_backup_list(f, app),
            Mode::ProcessRestartInfo => self.render_process_restart_info(f, app),
            Mode::FilterMenu => self.render_filter_menu(f, app),
            Mode::ThemeSelection => self.render_theme_selection(f, app),
            Mode::Menu {
                active_menu,
                selected_item,
            } => {
                self.render_main(f, app);
                self.render_menu_dropdown(f, app, active_menu, selected_item);
            }
            _ => self.render_main(f, app),
        }
    }

    fn render_main(&self, f: &mut Frame, app: &App) {
        // Set overall background to match MC's blue theme
        let root_block = Block::default().style(
            Style::default()
                .fg(app.theme.panel_normal_fg)
                .bg(app.theme.panel_normal_bg),
        );
        f.render_widget(root_block, f.area());

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Menu bar
                Constraint::Length(1), // Header (statistics only)
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Status bar
                Constraint::Length(2), // Key hints
            ])
            .split(f.area());

        // Render menu bar
        self.render_menu_bar(f, chunks[0], app);

        // Render header
        self.render_header(f, chunks[1], app);

        // Split main area into two panels
        let panels = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[2]);

        // Render panels
        self.render_panel(f, panels[0], app, Panel::Machine);
        self.render_panel(f, panels[1], app, Panel::User);

        // Render status bar
        self.render_status(f, chunks[3], app);

        // Render key hints
        self.render_key_hints(f, chunks[4], app);

        // Render input overlay if in input mode
        if let Mode::Input(input_mode) = app.mode {
            self.render_input_overlay(f, app, input_mode);
        }

        // Render file browser overlay if in file browser mode
        if let Mode::FileBrowser = app.mode {
            self.render_file_browser(f, app);
        }
    }

    fn render_header(&self, f: &mut Frame, area: Rect, app: &App) {
        let stats = app.get_statistics();

        // Build filter status text
        let mut second_line_spans = vec![
            Span::raw("Total: "),
            Span::styled(
                format!("M:{} ", stats.machine_total),
                Style::default().fg(app.theme.panel_normal_fg),
            ),
            Span::styled(
                format!("U:{} ", stats.user_total),
                Style::default().fg(app.theme.panel_normal_fg),
            ),
            Span::raw("│ Dead: "),
            Span::styled(
                format!("M:{} ", stats.machine_dead),
                Style::default().fg(app.theme.path_dead_fg),
            ),
            Span::styled(
                format!("U:{} ", stats.user_dead),
                Style::default().fg(app.theme.path_dead_fg),
            ),
            Span::raw("│ Duplicates: "),
            Span::styled(
                format!("M:{} ", stats.machine_duplicates),
                Style::default().fg(app.theme.path_duplicate_fg),
            ),
            Span::styled(
                format!("U:{} ", stats.user_duplicates),
                Style::default().fg(app.theme.path_duplicate_fg),
            ),
            Span::raw("│ Non-norm: "),
            Span::styled(
                format!("M:{} ", stats.machine_non_normalized),
                Style::default().fg(app.theme.path_nonnormalized_fg),
            ),
            Span::styled(
                format!("U:{}", stats.user_non_normalized),
                Style::default().fg(app.theme.path_nonnormalized_fg),
            ),
        ];

        // Add filter status if active
        use crate::app::FilterMode;
        if app.filter_mode != FilterMode::None {
            let filter_text = match app.filter_mode {
                FilterMode::Dead => "Dead",
                FilterMode::Duplicates => "Duplicates",
                FilterMode::NonNormalized => "Non-normalized",
                FilterMode::Valid => "Valid",
                FilterMode::None => "",
            };
            second_line_spans.push(Span::raw(" │ Filter: "));
            second_line_spans.push(Span::styled(
                filter_text,
                Style::default()
                    .fg(app.theme.filter_indicator_fg)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        second_line_spans.push(Span::raw(" │ "));
        second_line_spans.push(Span::styled(
            if app.has_changes { "MODIFIED" } else { "Clean" },
            if app.has_changes {
                Style::default()
                    .fg(app.theme.path_duplicate_fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(app.theme.path_valid_fg)
            },
        ));

        // Add connection mode indicator if in remote mode
        if let Some(ref connection) = app.remote_connection {
            second_line_spans.insert(0, Span::raw(" "));
            second_line_spans.insert(
                0,
                Span::styled(
                    connection.computer_name(),
                    Style::default()
                        .fg(app.theme.path_valid_fg)
                        .add_modifier(Modifier::BOLD),
                ),
            );
            second_line_spans.insert(
                0,
                Span::styled(
                    "REMOTE: ",
                    Style::default()
                        .fg(app.theme.path_duplicate_fg)
                        .add_modifier(Modifier::BOLD),
                ),
            );
        }

        let header_line = Line::from(second_line_spans);

        let header = Paragraph::new(header_line)
            .style(
                Style::default()
                    .fg(app.theme.header_fg)
                    .bg(app.theme.header_bg),
            )
            .alignment(Alignment::Left);

        f.render_widget(header, area);
    }

    fn render_panel(&self, f: &mut Frame, area: Rect, app: &App, panel: Panel) {
        use crate::app::ConnectionMode;

        let is_active = app.active_panel == panel;

        // In Remote mode, Panel::User shows remote MACHINE paths instead of USER paths
        let (paths, info, selected, marked, scrollbar_state) = match (app.connection_mode, panel) {
            (ConnectionMode::Local, Panel::Machine) => (
                &app.machine_paths,
                &app.machine_info,
                app.machine_selected,
                &app.machine_marked,
                &app.machine_scrollbar_state,
            ),
            (ConnectionMode::Local, Panel::User) => (
                &app.user_paths,
                &app.user_info,
                app.user_selected,
                &app.user_marked,
                &app.user_scrollbar_state,
            ),
            (ConnectionMode::Remote, Panel::Machine) => (
                &app.machine_paths,
                &app.machine_info,
                app.machine_selected,
                &app.machine_marked,
                &app.machine_scrollbar_state,
            ),
            (ConnectionMode::Remote, Panel::User) => (
                &app.remote_machine_paths,
                &app.remote_machine_info,
                app.remote_machine_selected,
                &app.remote_machine_marked,
                &app.remote_scrollbar_state,
            ),
        };

        // Get filtered indices
        let filtered_indices = app.get_filtered_indices(info);

        // Split area: List (left) and Scrollbar (right 1 column)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),    // List takes remaining space
                Constraint::Length(1), // Scrollbar takes 1 column
            ])
            .split(area);

        // Build panel title based on connection mode
        let scope_label = match (app.connection_mode, panel) {
            (ConnectionMode::Local, Panel::Machine) => "MACHINE".to_string(),
            (ConnectionMode::Local, Panel::User) => "USER".to_string(),
            (ConnectionMode::Remote, Panel::Machine) => "LOCAL MACHINE".to_string(),
            (ConnectionMode::Remote, Panel::User) => {
                if let Some(ref conn) = app.remote_connection {
                    format!("REMOTE MACHINE ({})", conn.computer_name())
                } else {
                    "REMOTE MACHINE".to_string()
                }
            }
        };

        let title = format!(
            " {} {} {}",
            scope_label,
            if !app.is_admin && panel == Panel::Machine {
                "[READ-ONLY]"
            } else {
                ""
            },
            if !filtered_indices.is_empty() && filtered_indices.len() != paths.len() {
                format!("[{}/{}]", filtered_indices.len(), paths.len())
            } else {
                String::new()
            }
        );

        let border_style = if is_active {
            Style::default()
                .fg(app.theme.panel_border_fg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.panel_border_fg)
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(
                Style::default()
                    .fg(app.theme.panel_normal_fg)
                    .bg(app.theme.panel_normal_bg),
            );

        // Only show filtered paths
        let items: Vec<ListItem> = filtered_indices
            .iter()
            .map(|&idx| {
                let is_selected = idx == selected && is_active;
                let is_marked = marked.contains(&idx);

                let path = &paths[idx];
                let status = info.get(idx).map(|i| i.status).unwrap_or(PathStatus::Valid);
                let color = self.get_status_color(status, &app.theme);

                let checkbox = if is_marked { "[X] " } else { "[ ] " };
                let display = format!("{}{}", checkbox, path);

                let style = if is_selected {
                    // Use theme colors for selection
                    Style::default()
                        .fg(app.theme.panel_selected_fg)
                        .bg(app.theme.panel_selected_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    // Use status color for normal items
                    Style::default().fg(color).bg(app.theme.panel_normal_bg)
                };

                ListItem::new(display).style(style)
            })
            .collect();

        let list = List::new(items).block(block);

        f.render_widget(list, chunks[0]);

        // Render scrollbar
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .thumb_style(Style::default().fg(app.theme.scrollbar_thumb_fg))
            .track_style(Style::default().fg(app.theme.scrollbar_fg));

        // Clone state for rendering (render_stateful_widget needs &mut)
        let mut scrollbar_state_mut = *scrollbar_state;
        f.render_stateful_widget(scrollbar, chunks[1], &mut scrollbar_state_mut);
    }

    fn render_status(&self, f: &mut Frame, area: Rect, app: &App) {
        let mut status_spans = vec![];

        // Show privilege level with helpful context
        if app.is_admin {
            status_spans.push(Span::styled(
                "ADMIN ",
                Style::default().fg(app.theme.path_valid_fg),
            ));
        } else {
            status_spans.push(Span::styled(
                "USER ",
                Style::default().fg(app.theme.path_duplicate_fg),
            ));
            status_spans.push(Span::styled(
                "(MACHINE read-only, press Ctrl+E to elevate)",
                Style::default().fg(app.theme.path_duplicate_fg),
            ));
        }
        status_spans.push(Span::raw(" │ "));

        // Add marked items count if any are marked
        let total_marked = app.machine_marked.len() + app.user_marked.len();
        if total_marked > 0 {
            status_spans.push(Span::styled(
                format!("{} marked", total_marked),
                Style::default().fg(app.theme.panel_marked_fg),
            ));
            status_spans.push(Span::raw(" │ "));
        }

        status_spans.push(Span::styled(
            &app.status_message,
            Style::default().fg(app.theme.status_fg),
        ));

        let status_text = vec![Line::from(status_spans)];

        let status = Paragraph::new(status_text)
            .block(
                Block::default().borders(Borders::ALL).style(
                    Style::default()
                        .fg(app.theme.status_fg)
                        .bg(app.theme.status_bg),
                ),
            )
            .alignment(Alignment::Left);

        f.render_widget(status, area);
    }

    /// Render function keys with even spacing across terminal width (MC-style)
    fn render_evenly_spaced_keys(
        &self,
        keys: Vec<(&str, &str)>,
        area: Rect,
        theme: &Theme,
    ) -> Line<'static> {
        let available_width = area.width as usize;
        let num_keys = keys.len();

        if num_keys == 0 {
            return Line::from(vec![]);
        }

        // Calculate total content width (without spacing)
        let total_content_width: usize = keys
            .iter()
            .map(|(num, label)| num.len() + label.len())
            .sum();

        // Calculate total spacing to distribute
        let total_spacing = available_width.saturating_sub(total_content_width);

        // Distribute spacing evenly between keys
        let spacing_per_gap = if num_keys > 1 {
            total_spacing / (num_keys - 1)
        } else {
            0
        };

        let mut spans = Vec::new();
        for (idx, (key_num, label)) in keys.iter().enumerate() {
            // Add the key number span
            spans.push(Span::styled(
                key_num.to_string(),
                Style::default()
                    .fg(theme.function_key_number_fg)
                    .bg(theme.function_key_number_bg),
            ));
            // Add the label span
            spans.push(Span::styled(
                label.to_string(),
                Style::default()
                    .fg(theme.function_key_label_fg)
                    .bg(theme.function_key_label_bg),
            ));

            // Add spacing between keys (but not after the last key)
            if idx < num_keys - 1 {
                spans.push(Span::raw(" ".repeat(spacing_per_gap)));
            }
        }

        Line::from(spans)
    }

    fn render_key_hints(&self, f: &mut Frame, area: Rect, app: &App) {
        let hints_line = match app.mode {
            Mode::Normal => {
                use crate::app::FilterMode;

                // Count total marked items across both panels
                let total_marked = app.machine_marked.len() + app.user_marked.len();
                let filter_active = app.filter_mode != FilterMode::None;

                // Context-sensitive hints based on application state
                if filter_active {
                    // When filter is active - show filter-related operations
                    let mut key_pairs = vec![
                        ("1", "Help"),
                        ("/", "Clear"),
                        ("Ctrl+A", "MarkAll"),
                        ("3", "Del"),
                    ];
                    if app.can_undo() {
                        key_pairs.push(("Ctrl+Z", "Undo"));
                    }
                    if app.can_redo() {
                        key_pairs.push(("Ctrl+Y", "Redo"));
                    }
                    key_pairs.push(("Ctrl+S", "Save"));
                    if !app.is_admin {
                        key_pairs.push(("Ctrl+E", "Elevate"));
                    }
                    key_pairs.push(("10", "Quit"));
                    self.render_evenly_spaced_keys(key_pairs, area, &app.theme)
                } else if total_marked > 0 {
                    // When items are marked - show bulk operations
                    let mut key_pairs = vec![
                        ("1", "Help"),
                        ("3", "Delete"),
                        ("5", "Move"),
                        ("9", "Normalize"),
                    ];
                    if app.can_undo() {
                        key_pairs.push(("Ctrl+Z", "Undo"));
                    }
                    if app.can_redo() {
                        key_pairs.push(("Ctrl+Y", "Redo"));
                    }
                    key_pairs.push(("Ctrl+Shift+U", "Unmark"));
                    key_pairs.push(("Ctrl+S", "Save"));
                    if !app.is_admin {
                        key_pairs.push(("Ctrl+E", "Elevate"));
                    }
                    key_pairs.push(("10", "Quit"));
                    self.render_evenly_spaced_keys(key_pairs, area, &app.theme)
                } else {
                    // Normal mode - default hints with more discoverable features
                    let mut key_pairs = vec![
                        ("1", "Help"),
                        ("2", "Mark"),
                        ("3", "Del"),
                        ("4", "Add"),
                        ("/", "Filter"),
                    ];
                    if app.can_undo() {
                        key_pairs.push(("Ctrl+Z", "Undo"));
                    }
                    if app.can_redo() {
                        key_pairs.push(("Ctrl+Y", "Redo"));
                    }
                    key_pairs.push(("Ctrl+S", "Save"));
                    // Show Ctrl+E hint when not admin
                    if !app.is_admin {
                        key_pairs.push(("Ctrl+E", "Elevate"));
                    }
                    // Show Ctrl+O hint when in remote mode
                    if app.remote_connection.is_some() {
                        key_pairs.push(("Ctrl+O", "Disconnect"));
                    }
                    key_pairs.push(("10", "Quit"));

                    self.render_evenly_spaced_keys(key_pairs, area, &app.theme)
                }
            }
            _ => {
                let key_pairs = vec![("ESC", "Cancel")];
                self.render_evenly_spaced_keys(key_pairs, area, &app.theme)
            }
        };

        let paragraph = Paragraph::new(hints_line).alignment(Alignment::Left).style(
            Style::default()
                .fg(app.theme.function_key_label_fg)
                .bg(app.theme.function_key_label_bg),
        );

        f.render_widget(paragraph, area);
    }

    fn render_help(&self, f: &mut Frame, app: &App) {
        // Create a centered dialog area
        let area = centered_rect(90, 90, f.area());
        f.render_widget(ratatui::widgets::Clear, area);

        // Create outer block with title
        let outer_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.dialog_border_fg))
            .style(Style::default().fg(app.theme.help_fg).bg(app.theme.help_bg))
            .title(vec![Span::styled(
                " Path Commander - Help ",
                Style::default()
                    .fg(app.theme.dialog_title_fg)
                    .add_modifier(Modifier::BOLD),
            )]);

        let inner_area = outer_block.inner(area);
        f.render_widget(outer_block, area);

        // Split inner area into two columns with small gap
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Left column
                Constraint::Percentage(50), // Right column
            ])
            .split(inner_area);

        // Left column content
        let left_text = vec![
            Line::from(vec![Span::styled(
                "Navigation:",
                Style::default()
                    .fg(app.theme.help_bold_fg)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  ↑/↓, j/k        Move selection up/down"),
            Line::from("  PgUp/PgDn       Move selection by 10"),
            Line::from("  Home/End        Move to first/last item"),
            Line::from("  Tab, ←/→        Switch between panels"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Selection:",
                Style::default()
                    .fg(app.theme.help_bold_fg)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  Space, Insert   Toggle mark on current"),
            Line::from("  F2              Toggle mark (MC style)"),
            Line::from("  Ctrl+A          Mark all in current scope"),
            Line::from("  Ctrl+Shift+A    Mark all in both scopes"),
            Line::from("  Ctrl+D          Mark all duplicates"),
            Line::from("  Ctrl+Shift+D    Mark all dead paths"),
            Line::from("  Ctrl+N          Mark non-normalized paths"),
            Line::from("  Ctrl+Shift+U    Unmark all"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Filtering:",
                Style::default()
                    .fg(app.theme.help_bold_fg)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  /               Open filter menu"),
            Line::from("                  • Clear filter (show all)"),
            Line::from("                  • Dead paths"),
            Line::from("                  • Duplicates"),
            Line::from("                  • Non-normalized paths"),
            Line::from("                  • Valid paths only"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Appearance:",
                Style::default()
                    .fg(app.theme.help_bold_fg)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  t               Choose color theme"),
            Line::from("                  • Dracula, Classic MC, Monokai"),
            Line::from("                  • Load custom themes from ~/.pc/themes/"),
        ];

        // Right column content
        let right_text = vec![
            Line::from(vec![Span::styled(
                "Actions:",
                Style::default()
                    .fg(app.theme.help_bold_fg)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  F3, Delete      Delete marked items"),
            Line::from("  F4              Add new path"),
            Line::from("  F5              Move marked to other panel"),
            Line::from("  F6              Move item up in order"),
            Line::from("  F7              Remove all duplicates"),
            Line::from("  F8              Remove all dead paths"),
            Line::from("  F9              Normalize selected paths"),
            Line::from("  F10             Create marked dead dirs"),
            Line::from("  Enter           Edit current path"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Save/Restore:",
                Style::default()
                    .fg(app.theme.help_bold_fg)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  Ctrl+S          Apply changes to registry"),
            Line::from("  Ctrl+B          Create backup"),
            Line::from("  Ctrl+R          Restore from backup"),
            Line::from("  Ctrl+Z          Undo last operation"),
            Line::from("  Ctrl+Y          Redo last undone operation"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Privileges:",
                Style::default()
                    .fg(app.theme.help_bold_fg)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  Ctrl+E          Elevate to Administrator"),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "USER mode:",
                    Style::default().fg(app.theme.path_duplicate_fg),
                ),
                Span::raw(" MACHINE paths read-only"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("ADMIN mode:", Style::default().fg(app.theme.path_valid_fg)),
                Span::raw(" Full access to all paths"),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Remote:",
                Style::default()
                    .fg(app.theme.help_bold_fg)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  Ctrl+O          Connect to/disconnect from"),
            Line::from("                  remote computer"),
            Line::from("  --remote NAME   Connect on startup"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Color Legend:",
                Style::default()
                    .fg(app.theme.help_bold_fg)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Red", Style::default().fg(app.theme.path_dead_fg)),
                Span::raw(" - Dead path (doesn't exist)"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Yellow", Style::default().fg(app.theme.path_duplicate_fg)),
                Span::raw(" - Duplicate path"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Cyan", Style::default().fg(app.theme.path_nonnormalized_fg)),
                Span::raw(" - Non-normalized"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Green", Style::default().fg(app.theme.path_valid_fg)),
                Span::raw(" - Valid, unique, normalized"),
            ]),
        ];

        // Create paragraphs for each column
        let left_para = Paragraph::new(left_text)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });

        let right_para = Paragraph::new(right_text)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });

        // Render both columns
        f.render_widget(left_para, columns[0]);
        f.render_widget(right_para, columns[1]);

        // Render footer message at the bottom of the dialog
        let footer_area = Rect {
            x: inner_area.x,
            y: inner_area.y + inner_area.height - 1,
            width: inner_area.width,
            height: 1,
        };

        let footer = Paragraph::new(Line::from(vec![Span::styled(
            "Press ESC or F1 to close this help",
            Style::default().fg(app.theme.warning_fg),
        )]))
        .alignment(Alignment::Center);

        f.render_widget(footer, footer_area);
    }

    fn render_process_restart_info(&self, f: &mut Frame, app: &App) {
        let mut lines = vec![
            Line::from(vec![Span::styled(
                "PATH Changes Applied Successfully!",
                Style::default()
                    .fg(app.theme.success_fg)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Important: Some running processes need to be restarted",
                Style::default()
                    .fg(app.theme.warning_fg)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "The following processes won't pick up the new PATH until restarted:",
                Style::default().fg(app.theme.dialog_fg),
            )]),
            Line::from(""),
        ];

        // Add each process to the list
        for process in &app.processes_to_restart {
            lines.push(Line::from(vec![
                Span::styled("  • ", Style::default().fg(app.theme.dialog_fg)),
                Span::styled(process, Style::default().fg(app.theme.info_fg)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Why restart?",
            Style::default()
                .fg(app.theme.dialog_fg)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "These processes load environment variables at startup and don't respond to",
            Style::default().fg(app.theme.dialog_fg),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "WM_SETTINGCHANGE notifications. You'll need to close and reopen them to see",
            Style::default().fg(app.theme.dialog_fg),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "the updated PATH.",
            Style::default().fg(app.theme.dialog_fg),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "Note: ",
                Style::default()
                    .fg(app.theme.warning_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "New processes started after this point will see the updated PATH.",
                Style::default().fg(app.theme.dialog_fg),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Press ENTER or ESC to continue",
            Style::default().fg(app.theme.warning_fg),
        )]));

        let info = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.dialog_border_fg))
                    .title(vec![Span::styled(
                        " Process Restart Required ",
                        Style::default()
                            .fg(app.theme.dialog_title_fg)
                            .bg(app.theme.dialog_title_bg)
                            .add_modifier(Modifier::BOLD),
                    )])
                    .style(
                        Style::default()
                            .fg(app.theme.dialog_fg)
                            .bg(app.theme.dialog_bg),
                    ),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });

        let area = centered_rect(80, 80, f.area());
        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(info, area);
    }

    fn render_about(&self, f: &mut Frame, app: &App) {
        // Create a centered dialog area
        let area = centered_rect(60, 50, f.area());
        f.render_widget(ratatui::widgets::Clear, area);

        // Create outer block with title
        let outer_block = Block::default()
            .title("About Path Commander")
            .title_style(
                Style::default()
                    .fg(app.theme.dialog_title_fg)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(app.theme.dialog_border_fg)
                    .bg(app.theme.dialog_bg),
            )
            .style(
                Style::default()
                    .fg(app.theme.dialog_fg)
                    .bg(app.theme.dialog_bg),
            );

        let inner_area = outer_block.inner(area);
        f.render_widget(outer_block, area);

        // Build content
        let content = vec![
            Line::from(vec![
                Span::styled(
                    "Path Commander",
                    Style::default()
                        .fg(app.theme.dialog_title_fg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  v0.1.0"),
            ]),
            Line::from(""),
            Line::from("Windows PATH Environment Manager"),
            Line::from(""),
            Line::from("Copyright © 2025 Jesse Slaton"),
            Line::from(""),
            Line::from(vec![
                Span::raw("License: "),
                Span::styled(
                    "MIT License",
                    Style::default()
                        .fg(app.theme.help_link_fg)
                        .add_modifier(Modifier::UNDERLINED),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Project: "),
                Span::styled(
                    "github.com/jesse-slaton/cli-tools",
                    Style::default()
                        .fg(app.theme.help_link_fg)
                        .add_modifier(Modifier::UNDERLINED),
                ),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "Press Esc or Enter to close",
                Style::default().fg(app.theme.dialog_fg),
            )),
        ];

        let paragraph = Paragraph::new(content)
            .style(
                Style::default()
                    .fg(app.theme.dialog_fg)
                    .bg(app.theme.dialog_bg),
            )
            .alignment(Alignment::Center);

        f.render_widget(paragraph, inner_area);
    }

    fn render_confirm(&self, f: &mut Frame, app: &App, action: ConfirmAction) {
        // Build the message lines based on action
        let mut message_lines = vec![Line::from("")]; // Start with blank line

        match action {
            ConfirmAction::Exit => {
                if app.has_changes {
                    message_lines.push(Line::from(vec![Span::styled(
                        "You have unsaved changes. Exit anyway?",
                        Style::default()
                            .fg(app.theme.dialog_fg)
                            .add_modifier(Modifier::BOLD),
                    )]));
                } else {
                    message_lines.push(Line::from(vec![Span::styled(
                        "Exit Path Commander?",
                        Style::default()
                            .fg(app.theme.dialog_fg)
                            .add_modifier(Modifier::BOLD),
                    )]));
                }
            }
            ConfirmAction::DeleteSelected => {
                message_lines.push(Line::from(vec![Span::styled(
                    "Delete selected paths?",
                    Style::default()
                        .fg(app.theme.dialog_fg)
                        .add_modifier(Modifier::BOLD),
                )]));
            }
            ConfirmAction::DeleteAllDead => {
                message_lines.push(Line::from(vec![Span::styled(
                    "Delete all dead paths?",
                    Style::default()
                        .fg(app.theme.dialog_fg)
                        .add_modifier(Modifier::BOLD),
                )]));
            }
            ConfirmAction::DeleteAllDuplicates => {
                message_lines.push(Line::from(vec![Span::styled(
                    "Delete all duplicate paths?",
                    Style::default()
                        .fg(app.theme.dialog_fg)
                        .add_modifier(Modifier::BOLD),
                )]));
            }
            ConfirmAction::ApplyChanges => {
                message_lines.push(Line::from(vec![Span::styled(
                    "Apply changes to PATH?",
                    Style::default()
                        .fg(app.theme.dialog_fg)
                        .add_modifier(Modifier::BOLD),
                )]));
                message_lines.push(Line::from(""));
                // Show which scopes will be modified
                let mut scopes = Vec::new();
                if app.user_paths != app.user_original {
                    scopes.push("USER");
                }
                if app.machine_paths != app.machine_original {
                    scopes.push("MACHINE");
                }
                let scope_text = if scopes.is_empty() {
                    "No changes detected".to_string()
                } else {
                    format!("This will modify: {}", scopes.join(" and "))
                };
                message_lines.push(Line::from(vec![Span::styled(
                    scope_text,
                    Style::default().fg(app.theme.warning_fg),
                )]));
            }
            ConfirmAction::RequestElevation => {
                message_lines.push(Line::from(vec![Span::styled(
                    "Administrator Privileges Required",
                    Style::default()
                        .fg(app.theme.dialog_fg)
                        .add_modifier(Modifier::BOLD),
                )]));
                message_lines.push(Line::from(""));
                message_lines.push(Line::from(vec![Span::styled(
                    "Modifying MACHINE (system-wide) PATH requires",
                    Style::default().fg(app.theme.dialog_fg),
                )]));
                message_lines.push(Line::from(vec![Span::styled(
                    "administrator privileges.",
                    Style::default().fg(app.theme.dialog_fg),
                )]));
                message_lines.push(Line::from(""));
                message_lines.push(Line::from(vec![Span::styled(
                    "Current privileges: Standard User",
                    Style::default().fg(app.theme.info_fg),
                )]));
                message_lines.push(Line::from(vec![Span::styled(
                    "Required privileges: Administrator",
                    Style::default().fg(app.theme.warning_fg),
                )]));
                message_lines.push(Line::from(""));
                message_lines.push(Line::from(vec![Span::styled(
                    "Restart with elevated privileges?",
                    Style::default().fg(app.theme.dialog_fg),
                )]));
                message_lines.push(Line::from(""));
                message_lines.push(Line::from(vec![Span::styled(
                    "(Your current changes will be preserved)",
                    Style::default()
                        .fg(app.theme.info_fg)
                        .add_modifier(Modifier::ITALIC),
                )]));
            }
            ConfirmAction::RestoreBackup => {
                message_lines.push(Line::from(vec![Span::styled(
                    "Restore from selected backup?",
                    Style::default()
                        .fg(app.theme.dialog_fg)
                        .add_modifier(Modifier::BOLD),
                )]));
            }
            ConfirmAction::CreateSingleDirectory => {
                message_lines.push(Line::from(vec![Span::styled(
                    "Directory does not exist.",
                    Style::default()
                        .fg(app.theme.dialog_fg)
                        .add_modifier(Modifier::BOLD),
                )]));
                message_lines.push(Line::from(""));
                message_lines.push(Line::from(vec![Span::styled(
                    "Create directory and add to PATH?",
                    Style::default().fg(app.theme.dialog_fg),
                )]));
            }
            ConfirmAction::CreateMarkedDirectories => {
                message_lines.push(Line::from(vec![Span::styled(
                    "Create all marked dead directories?",
                    Style::default()
                        .fg(app.theme.dialog_fg)
                        .add_modifier(Modifier::BOLD),
                )]));
                message_lines.push(Line::from(""));
                message_lines.push(Line::from(vec![Span::styled(
                    "(Network paths and invalid paths will be skipped)",
                    Style::default().fg(app.theme.info_fg),
                )]));
            }
            ConfirmAction::DisconnectRemote => {
                let computer_name = app
                    .remote_connection
                    .as_ref()
                    .map(|c| c.computer_name())
                    .unwrap_or("unknown");
                message_lines.push(Line::from(vec![Span::styled(
                    format!("Disconnect from remote computer '{}'?", computer_name),
                    Style::default()
                        .fg(app.theme.dialog_fg)
                        .add_modifier(Modifier::BOLD),
                )]));
                if app.has_changes {
                    message_lines.push(Line::from(""));
                    message_lines.push(Line::from(vec![Span::styled(
                        "Warning: Unsaved changes will be lost!",
                        Style::default().fg(app.theme.warning_fg),
                    )]));
                }
            }
        }

        message_lines.push(Line::from(""));
        message_lines.push(Line::from(vec![
            Span::styled(
                "Y",
                Style::default()
                    .fg(app.theme.button_focused_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("es", Style::default().fg(app.theme.dialog_fg)),
            Span::styled(" / ", Style::default().fg(app.theme.dialog_fg)),
            Span::styled(
                "N",
                Style::default()
                    .fg(app.theme.button_focused_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("o", Style::default().fg(app.theme.dialog_fg)),
        ]));

        let text = message_lines;

        let dialog = Paragraph::new(text)
            .block(
                Block::default()
                    .title(vec![Span::styled(
                        " Confirm ",
                        Style::default()
                            .fg(app.theme.dialog_title_fg)
                            .bg(app.theme.dialog_title_bg)
                            .add_modifier(Modifier::BOLD),
                    )])
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.dialog_border_fg))
                    .style(
                        Style::default()
                            .fg(app.theme.dialog_fg)
                            .bg(app.theme.dialog_bg),
                    ),
            )
            .alignment(Alignment::Center);

        let area = centered_rect(60, 30, f.area());
        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(dialog, area);
    }

    fn render_input_overlay(&self, f: &mut Frame, app: &App, input_mode: InputMode) {
        let title = match input_mode {
            InputMode::AddPath => " Add Path ",
            InputMode::EditPath => " Edit Path ",
            InputMode::ConnectRemote => " Connect to Remote Computer ",
        };

        let text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                &app.input_buffer,
                Style::default().fg(app.theme.dialog_fg),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Enter to confirm, ESC to cancel",
                Style::default().fg(app.theme.info_fg),
            )]),
        ];

        let input = Paragraph::new(text)
            .block(
                Block::default()
                    .title(vec![Span::styled(
                        title,
                        Style::default()
                            .fg(app.theme.dialog_title_fg)
                            .bg(app.theme.dialog_title_bg)
                            .add_modifier(Modifier::BOLD),
                    )])
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.dialog_border_fg))
                    .style(
                        Style::default()
                            .fg(app.theme.dialog_fg)
                            .bg(app.theme.dialog_bg),
                    ),
            )
            .alignment(Alignment::Left);

        let area = centered_rect(70, 20, f.area());
        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(input, area);
    }

    fn render_file_browser(&self, f: &mut Frame, app: &App) {
        let area = centered_rect(85, 75, f.area());

        // Clear the area and render the main block
        f.render_widget(ratatui::widgets::Clear, area);

        // Render the main block
        let main_block = Block::default()
            .title(vec![Span::styled(
                " Browse Directories ",
                Style::default()
                    .fg(app.theme.dialog_title_fg)
                    .bg(app.theme.dialog_title_bg)
                    .add_modifier(Modifier::BOLD),
            )])
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.dialog_border_fg))
            .style(
                Style::default()
                    .fg(app.theme.dialog_fg)
                    .bg(app.theme.dialog_bg),
            );

        f.render_widget(main_block.clone(), area);

        // Get inner area (inside the borders)
        let inner_area = main_block.inner(area);

        // Create layout: title area, list area, help area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Current path
                Constraint::Min(0),    // Directory list
                Constraint::Length(2), // Key hints
            ])
            .split(inner_area);

        // Render current path
        let current_path = app.file_browser_current_path.to_string_lossy().to_string();
        let display_path = if current_path == "DRIVES" {
            "Available Drives".to_string()
        } else {
            format!("Current: {}", current_path)
        };
        let path_text = vec![
            Line::from(vec![Span::styled(
                display_path,
                Style::default().fg(app.theme.dialog_fg),
            )]),
            Line::from(""),
        ];
        let path_widget = Paragraph::new(path_text)
            .style(
                Style::default()
                    .fg(app.theme.dialog_fg)
                    .bg(app.theme.dialog_bg),
            )
            .alignment(Alignment::Left);

        // Render directory list
        let items: Vec<ListItem> = app
            .file_browser_entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let is_selected = idx == app.file_browser_selected;
                let display_name = if entry.name == "Network..." {
                    format!("<{}>", entry.name) // Network: "<Network...>"
                } else if entry.is_drive {
                    format!("[{}]", entry.name) // Drive: "[C:]"
                } else {
                    format!("/{}", entry.name) // Parent "/.." or Directory "/dirname"
                };

                let style = if is_selected {
                    Style::default()
                        .fg(app.theme.panel_selected_fg)
                        .bg(app.theme.panel_selected_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(app.theme.dialog_fg)
                        .bg(app.theme.dialog_bg)
                };

                ListItem::new(display_name).style(style)
            })
            .collect();

        let list = List::new(items).style(
            Style::default()
                .fg(app.theme.dialog_fg)
                .bg(app.theme.dialog_bg),
        );

        // Render key hints
        let hints_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(app.theme.info_fg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" navigate │ "),
                Span::styled(
                    "Space",
                    Style::default()
                        .fg(app.theme.info_fg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" select │ "),
                Span::styled(
                    "Tab",
                    Style::default()
                        .fg(app.theme.info_fg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" manual input │ "),
                Span::styled(
                    "ESC",
                    Style::default()
                        .fg(app.theme.info_fg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" cancel"),
            ]),
        ];
        let hints_widget = Paragraph::new(hints_text)
            .style(
                Style::default()
                    .fg(app.theme.dialog_fg)
                    .bg(app.theme.dialog_bg),
            )
            .alignment(Alignment::Center);

        // Render the inner widgets
        f.render_widget(path_widget, chunks[0]);
        f.render_widget(list, chunks[1]);
        f.render_widget(hints_widget, chunks[2]);

        // Render scrollbar
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(app.theme.scrollbar_fg))
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let mut scrollbar_state = app.file_browser_scrollbar_state;
        f.render_stateful_widget(scrollbar, chunks[1], &mut scrollbar_state);
    }

    fn render_backup_list(&self, f: &mut Frame, app: &App) {
        let items: Vec<ListItem> = app
            .backup_list
            .iter()
            .enumerate()
            .map(|(idx, path)| {
                let is_selected = idx == app.backup_selected;
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown");

                let style = if is_selected {
                    Style::default()
                        .fg(app.theme.panel_selected_fg)
                        .bg(app.theme.panel_selected_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(app.theme.dialog_fg)
                        .bg(app.theme.dialog_bg)
                };

                ListItem::new(filename).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title(" Select Backup to Restore ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.dialog_border_fg))
                .style(
                    Style::default()
                        .fg(app.theme.dialog_fg)
                        .bg(app.theme.dialog_bg),
                ),
        );

        let area = centered_rect(70, 50, f.area());
        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(list, area);
    }

    fn render_filter_menu(&self, f: &mut Frame, app: &App) {
        use crate::app::FilterMode;

        // Filter options with descriptions
        let filter_options = [
            ("Clear Filter", "Show all paths", FilterMode::None),
            (
                "Dead Paths",
                "Paths that don't exist on filesystem",
                FilterMode::Dead,
            ),
            (
                "Duplicates",
                "Paths that appear multiple times",
                FilterMode::Duplicates,
            ),
            (
                "Non-Normalized",
                "Paths with env vars or short names",
                FilterMode::NonNormalized,
            ),
            (
                "Valid Paths",
                "Paths that are valid and unique",
                FilterMode::Valid,
            ),
        ];

        let items: Vec<ListItem> = filter_options
            .iter()
            .enumerate()
            .map(|(idx, (name, description, filter_mode))| {
                let is_selected = idx == app.filter_menu_selected;
                let is_current = app.filter_mode == *filter_mode;

                // Add indicator if this is the current filter
                let current_marker = if is_current { " [ACTIVE]" } else { "" };
                let display = format!("{}{}\n  {}", name, current_marker, description);

                let style = if is_selected {
                    Style::default()
                        .fg(app.theme.panel_selected_fg)
                        .bg(app.theme.panel_selected_bg)
                        .add_modifier(Modifier::BOLD)
                } else if is_current {
                    Style::default()
                        .fg(app.theme.filter_indicator_fg)
                        .bg(app.theme.dialog_bg)
                } else {
                    Style::default()
                        .fg(app.theme.dialog_fg)
                        .bg(app.theme.dialog_bg)
                };

                ListItem::new(display).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title(" Filter Paths ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.dialog_border_fg))
                .style(
                    Style::default()
                        .fg(app.theme.dialog_fg)
                        .bg(app.theme.dialog_bg),
                ),
        );

        let area = centered_rect(60, 60, f.area());
        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(list, area);
    }

    fn render_theme_selection(&self, f: &mut Frame, app: &App) {
        let items: Vec<ListItem> = app
            .theme_list
            .iter()
            .enumerate()
            .map(|(idx, (name, is_builtin))| {
                let is_selected = idx == app.theme_selected;
                let is_current = name == &app.theme.name;

                // Add indicators
                let builtin_marker = if *is_builtin {
                    " [Built-in]"
                } else {
                    " [Custom]"
                };
                let current_marker = if is_current { " [ACTIVE]" } else { "" };
                let display = format!("{}{}{}", name, current_marker, builtin_marker);

                let style = if is_selected {
                    Style::default()
                        .fg(app.theme.panel_selected_fg)
                        .bg(app.theme.panel_selected_bg)
                        .add_modifier(Modifier::BOLD)
                } else if is_current {
                    Style::default()
                        .fg(app.theme.filter_indicator_fg)
                        .bg(app.theme.dialog_bg)
                } else {
                    Style::default()
                        .fg(app.theme.dialog_fg)
                        .bg(app.theme.dialog_bg)
                };

                ListItem::new(display).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title(" Select Theme ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.dialog_border_fg))
                .style(
                    Style::default()
                        .fg(app.theme.dialog_fg)
                        .bg(app.theme.dialog_bg),
                ),
        );

        // Add preview at the bottom
        let area = centered_rect(70, 70, f.area());
        f.render_widget(ratatui::widgets::Clear, area);

        // Split area for list and preview
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Theme list
                Constraint::Length(8), // Preview
            ])
            .split(area);

        f.render_widget(list, chunks[0]);

        // Render preview
        let preview_lines = vec![
            Line::from(vec![Span::styled(
                "Preview:",
                Style::default()
                    .fg(app.theme.help_bold_fg)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Valid", Style::default().fg(app.theme.path_valid_fg)),
                Span::raw(" • "),
                Span::styled("Dead", Style::default().fg(app.theme.path_dead_fg)),
                Span::raw(" • "),
                Span::styled(
                    "Duplicate",
                    Style::default().fg(app.theme.path_duplicate_fg),
                ),
                Span::raw(" • "),
                Span::styled(
                    "Non-norm",
                    Style::default().fg(app.theme.path_nonnormalized_fg),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "↑↓/jk Navigate  Enter Select  Esc Cancel  r Reload",
                Style::default().fg(app.theme.info_fg),
            )]),
        ];

        let preview = Paragraph::new(preview_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.dialog_border_fg))
                    .style(
                        Style::default()
                            .fg(app.theme.dialog_fg)
                            .bg(app.theme.dialog_bg),
                    ),
            )
            .alignment(Alignment::Left);

        f.render_widget(preview, chunks[1]);
    }

    fn get_status_color(&self, status: PathStatus, theme: &crate::theme::Theme) -> Color {
        match status {
            PathStatus::Valid => theme.path_valid_fg,
            PathStatus::Dead => theme.path_dead_fg,
            PathStatus::Duplicate => theme.path_duplicate_fg,
            PathStatus::NonNormalized => theme.path_nonnormalized_fg,
            PathStatus::DeadDuplicate => theme.path_dead_fg,
        }
    }
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

impl UI {
    /// Render the menu bar (top row with menu names)
    fn render_menu_bar(&self, f: &mut Frame, area: Rect, app: &App) {
        let menus = menu::get_menus();
        let mut spans = Vec::new();

        for (i, menu_item) in menus.iter().enumerate() {
            // Determine if this menu is active (only in Menu mode)
            let is_active = if let Mode::Menu { active_menu, .. } = app.mode {
                active_menu == i
            } else {
                false
            };

            // Style based on whether menu is active
            let style = if is_active {
                Style::default()
                    .fg(app.theme.menu_selected_fg)
                    .bg(app.theme.menu_selected_bg)
            } else {
                Style::default()
                    .fg(app.theme.menu_inactive_fg)
                    .bg(app.theme.menu_inactive_bg)
            };

            // Add space before menu name
            if i > 0 {
                spans.push(Span::styled("  ", style));
            } else {
                spans.push(Span::styled(" ", style));
            }

            // Add menu name with highlighted accelerator key
            let name = &menu_item.name;
            let accel_char = menu_item.accelerator.to_uppercase().to_string();

            if let Some(pos) = name.to_uppercase().find(&accel_char) {
                // Add text before accelerator
                if pos > 0 {
                    spans.push(Span::styled(&name[..pos], style));
                }
                // Add highlighted accelerator
                spans.push(Span::styled(
                    &name[pos..pos + 1],
                    style.add_modifier(Modifier::UNDERLINED),
                ));
                // Add text after accelerator
                if pos + 1 < name.len() {
                    spans.push(Span::styled(&name[pos + 1..], style));
                }
            } else {
                spans.push(Span::styled(name, style));
            }

            spans.push(Span::styled(" ", style));
        }

        // Fill the rest of the line with background color
        let menu_line = Line::from(spans);
        let menu_bar = Paragraph::new(menu_line).style(
            Style::default()
                .fg(app.theme.menu_inactive_fg)
                .bg(app.theme.menu_inactive_bg),
        );
        f.render_widget(menu_bar, area);
    }

    /// Render the drop-down menu overlay
    fn render_menu_dropdown(
        &self,
        f: &mut Frame,
        app: &App,
        active_menu: usize,
        selected_item: usize,
    ) {
        let mut menus = menu::get_menus();

        // Update enabled states based on app state
        let has_marked = app.has_marked_items();
        let has_marked_dead = app.has_marked_dead_paths();
        let has_selection = match app.active_panel {
            Panel::Machine => !app.machine_paths.is_empty(),
            Panel::User => !app.user_paths.is_empty(),
        };
        let is_remote = app.connection_mode == crate::app::ConnectionMode::Remote;

        menu::update_menu_enabled_states(
            &mut menus,
            app.is_admin,
            has_marked,
            has_marked_dead,
            has_selection,
            is_remote,
            app.has_changes,
        );

        if active_menu >= menus.len() {
            return;
        }

        let menu = &menus[active_menu];

        // Calculate menu position (under the menu name in menu bar)
        let mut x_offset = 1; // Start with 1 for initial space
        for menu_item in menus.iter().take(active_menu) {
            x_offset += menu_item.name.len() as u16 + 2; // name + 2 spaces
        }

        // Calculate menu width (longest item + padding)
        let mut menu_width = menu.name.len();
        for item in &menu.items {
            let item_text_len =
                item.label.len() + item.shortcut.as_ref().map(|s| s.len() + 2).unwrap_or(0);
            menu_width = menu_width.max(item_text_len);
        }
        menu_width += 4; // Add padding

        let menu_height = menu.items.len() as u16 + 2; // +2 for borders

        // Create menu area (positioned below menu bar)
        let area = Rect {
            x: x_offset,
            y: 1, // Below menu bar
            width: menu_width as u16,
            height: menu_height,
        };

        // Ensure menu fits on screen
        let terminal_width = f.area().width;
        let area = if area.x + area.width > terminal_width {
            Rect {
                x: terminal_width.saturating_sub(area.width),
                ..area
            }
        } else {
            area
        };

        // Build menu items
        let items: Vec<ListItem> = menu
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = i == selected_item;
                let is_enabled = item.enabled;

                let fg = if !is_enabled {
                    app.theme.button_disabled_fg
                } else if is_selected {
                    app.theme.menu_selected_fg
                } else {
                    app.theme.menu_active_fg
                };

                let bg = if is_selected {
                    app.theme.menu_selected_bg
                } else {
                    app.theme.menu_active_bg
                };

                // Format: "Label          Shortcut"
                let label = &item.label;
                let shortcut = item.shortcut.as_deref().unwrap_or("");
                let spacing =
                    " ".repeat(menu_width.saturating_sub(label.len() + shortcut.len() + 4));
                let text = format!(" {}{} {} ", label, spacing, shortcut);

                ListItem::new(text).style(Style::default().fg(fg).bg(bg))
            })
            .collect();

        let menu_list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(app.theme.menu_active_fg)
                        .bg(app.theme.menu_active_bg),
                )
                .style(
                    Style::default()
                        .fg(app.theme.menu_active_fg)
                        .bg(app.theme.menu_active_bg),
                ),
        );

        // Clear background behind menu (draw a filled rectangle)
        let clear_block = Block::default().style(
            Style::default()
                .fg(app.theme.menu_active_fg)
                .bg(app.theme.menu_active_bg),
        );
        f.render_widget(clear_block, area);

        // Render the menu
        f.render_widget(menu_list, area);
    }
}
