use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, Wrap},
    Frame,
};

use crate::app::{App, ConfirmAction, InputMode, Mode, Panel};
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
            Mode::Confirm(action) => self.render_confirm(f, app, action),
            Mode::BackupList => self.render_backup_list(f, app),
            Mode::ProcessRestartInfo => self.render_process_restart_info(f, app),
            Mode::FilterMenu => self.render_filter_menu(f, app),
            Mode::ThemeSelection => self.render_theme_selection(f, app),
            _ => self.render_main(f, app),
        }
    }

    fn render_main(&self, f: &mut Frame, app: &App) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Status bar
                Constraint::Length(2), // Key hints
            ])
            .split(f.area());

        // Render header
        self.render_header(f, chunks[0], app);

        // Split main area into two panels
        let panels = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Render panels
        self.render_panel(f, panels[0], app, Panel::Machine);
        self.render_panel(f, panels[1], app, Panel::User);

        // Render status bar
        self.render_status(f, chunks[2], app);

        // Render key hints
        self.render_key_hints(f, chunks[3], app);

        // Render input overlay if in input mode
        if let Mode::Input(input_mode) = app.mode {
            self.render_input_overlay(f, app, input_mode);
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

        // Build first line with connection mode indicator
        let mut first_line_spans = vec![
            Span::styled(
                "Path Commander",
                Style::default()
                    .fg(app.theme.header_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Windows PATH Environment Manager"),
        ];

        // Add connection mode indicator if in remote mode
        if let Some(ref connection) = app.remote_connection {
            first_line_spans.push(Span::raw(" │ "));
            first_line_spans.push(Span::styled(
                "REMOTE: ",
                Style::default()
                    .fg(app.theme.path_duplicate_fg)
                    .add_modifier(Modifier::BOLD),
            ));
            first_line_spans.push(Span::styled(
                connection.computer_name(),
                Style::default()
                    .fg(app.theme.path_valid_fg)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        let title = vec![Line::from(first_line_spans), Line::from(second_line_spans)];

        let header = Paragraph::new(title)
            .block(Block::default().borders(Borders::ALL))
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
            .border_style(border_style);

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
        let mut status_spans = vec![
            Span::styled(
                if app.is_admin { "ADMIN " } else { "USER " },
                Style::default().fg(if app.is_admin {
                    app.theme.path_valid_fg
                } else {
                    app.theme.path_duplicate_fg
                }),
            ),
            Span::raw("│ "),
        ];

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
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Left);

        f.render_widget(status, area);
    }

    /// Helper to create MC-style function key display (e.g., "3View" instead of "F3 View")
    /// Returns spans for the key number and label
    fn mc_function_key(&self, key_num: &str, label: &str, theme: &Theme) -> Vec<Span<'static>> {
        vec![
            Span::styled(
                key_num.to_string(),
                Style::default()
                    .fg(theme.function_key_number_fg)
                    .bg(theme.function_key_number_bg),
            ),
            Span::styled(
                label.to_string(),
                Style::default()
                    .fg(theme.function_key_label_fg)
                    .bg(theme.function_key_label_bg),
            ),
        ]
    }

    fn render_key_hints(&self, f: &mut Frame, area: Rect, app: &App) {
        let hints = match app.mode {
            Mode::Normal => {
                use crate::app::FilterMode;

                // Count total marked items across both panels
                let total_marked = app.machine_marked.len() + app.user_marked.len();
                let filter_active = app.filter_mode != FilterMode::None;

                // Context-sensitive hints based on application state
                if filter_active {
                    // When filter is active - show filter-related operations
                    let mut hints_vec = Vec::new();
                    hints_vec.extend(self.mc_function_key("1", "Help", &app.theme));
                    hints_vec.push(Span::raw(" | "));
                    hints_vec.push(Span::styled(
                        "/",
                        Style::default()
                            .fg(app.theme.function_key_number_fg)
                            .bg(app.theme.function_key_number_bg),
                    ));
                    hints_vec.push(Span::styled(
                        "Clear Filter",
                        Style::default()
                            .fg(app.theme.function_key_label_fg)
                            .bg(app.theme.function_key_label_bg),
                    ));
                    hints_vec.push(Span::raw(" | "));
                    hints_vec.push(Span::styled(
                        "Ctrl+A",
                        Style::default()
                            .fg(app.theme.function_key_number_fg)
                            .bg(app.theme.function_key_number_bg),
                    ));
                    hints_vec.push(Span::styled(
                        "Mark All",
                        Style::default()
                            .fg(app.theme.function_key_label_fg)
                            .bg(app.theme.function_key_label_bg),
                    ));
                    hints_vec.push(Span::raw(" | "));
                    hints_vec.extend(self.mc_function_key("3", "Del", &app.theme));
                    hints_vec.push(Span::raw(" | "));
                    if app.can_undo() {
                        hints_vec.push(Span::styled(
                            "Ctrl+Z",
                            Style::default()
                                .fg(app.theme.function_key_number_fg)
                                .bg(app.theme.function_key_number_bg),
                        ));
                        hints_vec.push(Span::styled(
                            "Undo",
                            Style::default()
                                .fg(app.theme.function_key_label_fg)
                                .bg(app.theme.function_key_label_bg),
                        ));
                        hints_vec.push(Span::raw(" | "));
                    }
                    if app.can_redo() {
                        hints_vec.push(Span::styled(
                            "Ctrl+Y",
                            Style::default()
                                .fg(app.theme.function_key_number_fg)
                                .bg(app.theme.function_key_number_bg),
                        ));
                        hints_vec.push(Span::styled(
                            "Redo",
                            Style::default()
                                .fg(app.theme.function_key_label_fg)
                                .bg(app.theme.function_key_label_bg),
                        ));
                        hints_vec.push(Span::raw(" | "));
                    }
                    hints_vec.push(Span::styled(
                        "Ctrl+S",
                        Style::default()
                            .fg(app.theme.function_key_number_fg)
                            .bg(app.theme.function_key_number_bg),
                    ));
                    hints_vec.push(Span::styled(
                        "Save",
                        Style::default()
                            .fg(app.theme.function_key_label_fg)
                            .bg(app.theme.function_key_label_bg),
                    ));
                    hints_vec.push(Span::raw(" | "));
                    hints_vec.push(Span::styled(
                        "Q",
                        Style::default()
                            .fg(app.theme.function_key_number_fg)
                            .bg(app.theme.function_key_number_bg),
                    ));
                    hints_vec.push(Span::styled(
                        "Quit",
                        Style::default()
                            .fg(app.theme.function_key_label_fg)
                            .bg(app.theme.function_key_label_bg),
                    ));
                    hints_vec
                } else if total_marked > 0 {
                    // When items are marked - show bulk operations
                    let mut hints_vec = Vec::new();
                    hints_vec.extend(self.mc_function_key("1", "Help", &app.theme));
                    hints_vec.push(Span::raw(" | "));
                    hints_vec.extend(self.mc_function_key("3", "Delete", &app.theme));
                    hints_vec.push(Span::raw(" | "));
                    hints_vec.extend(self.mc_function_key("5", "Move", &app.theme));
                    hints_vec.push(Span::raw(" | "));
                    hints_vec.extend(self.mc_function_key("9", "Normalize", &app.theme));
                    hints_vec.push(Span::raw(" | "));
                    if app.can_undo() {
                        hints_vec.push(Span::styled(
                            "Ctrl+Z",
                            Style::default()
                                .fg(app.theme.function_key_number_fg)
                                .bg(app.theme.function_key_number_bg),
                        ));
                        hints_vec.push(Span::styled(
                            "Undo",
                            Style::default()
                                .fg(app.theme.function_key_label_fg)
                                .bg(app.theme.function_key_label_bg),
                        ));
                        hints_vec.push(Span::raw(" | "));
                    }
                    if app.can_redo() {
                        hints_vec.push(Span::styled(
                            "Ctrl+Y",
                            Style::default()
                                .fg(app.theme.function_key_number_fg)
                                .bg(app.theme.function_key_number_bg),
                        ));
                        hints_vec.push(Span::styled(
                            "Redo",
                            Style::default()
                                .fg(app.theme.function_key_label_fg)
                                .bg(app.theme.function_key_label_bg),
                        ));
                        hints_vec.push(Span::raw(" | "));
                    }
                    hints_vec.push(Span::styled(
                        "Ctrl+Shift+U",
                        Style::default()
                            .fg(app.theme.function_key_number_fg)
                            .bg(app.theme.function_key_number_bg),
                    ));
                    hints_vec.push(Span::styled(
                        "Unmark All",
                        Style::default()
                            .fg(app.theme.function_key_label_fg)
                            .bg(app.theme.function_key_label_bg),
                    ));
                    hints_vec.push(Span::raw(" | "));
                    hints_vec.push(Span::styled(
                        "Ctrl+S",
                        Style::default()
                            .fg(app.theme.function_key_number_fg)
                            .bg(app.theme.function_key_number_bg),
                    ));
                    hints_vec.push(Span::styled(
                        "Save",
                        Style::default()
                            .fg(app.theme.function_key_label_fg)
                            .bg(app.theme.function_key_label_bg),
                    ));
                    hints_vec.push(Span::raw(" | "));
                    hints_vec.push(Span::styled(
                        "Q",
                        Style::default()
                            .fg(app.theme.function_key_number_fg)
                            .bg(app.theme.function_key_number_bg),
                    ));
                    hints_vec.push(Span::styled(
                        "Quit",
                        Style::default()
                            .fg(app.theme.function_key_label_fg)
                            .bg(app.theme.function_key_label_bg),
                    ));
                    hints_vec
                } else {
                    // Normal mode - default hints with more discoverable features
                    let mut hints_vec = Vec::new();
                    hints_vec.extend(self.mc_function_key("1", "Help", &app.theme));
                    hints_vec.push(Span::raw(" | "));
                    hints_vec.extend(self.mc_function_key("2", "Mark", &app.theme));
                    hints_vec.push(Span::raw(" | "));
                    hints_vec.extend(self.mc_function_key("3", "Del", &app.theme));
                    hints_vec.push(Span::raw(" | "));
                    hints_vec.extend(self.mc_function_key("4", "Add", &app.theme));
                    hints_vec.push(Span::raw(" | "));
                    hints_vec.push(Span::styled(
                        "/",
                        Style::default()
                            .fg(app.theme.function_key_number_fg)
                            .bg(app.theme.function_key_number_bg),
                    ));
                    hints_vec.push(Span::styled(
                        "Filter",
                        Style::default()
                            .fg(app.theme.function_key_label_fg)
                            .bg(app.theme.function_key_label_bg),
                    ));
                    hints_vec.push(Span::raw(" | "));
                    if app.can_undo() {
                        hints_vec.push(Span::styled(
                            "Ctrl+Z",
                            Style::default()
                                .fg(app.theme.function_key_number_fg)
                                .bg(app.theme.function_key_number_bg),
                        ));
                        hints_vec.push(Span::styled(
                            "Undo",
                            Style::default()
                                .fg(app.theme.function_key_label_fg)
                                .bg(app.theme.function_key_label_bg),
                        ));
                        hints_vec.push(Span::raw(" | "));
                    }
                    if app.can_redo() {
                        hints_vec.push(Span::styled(
                            "Ctrl+Y",
                            Style::default()
                                .fg(app.theme.function_key_number_fg)
                                .bg(app.theme.function_key_number_bg),
                        ));
                        hints_vec.push(Span::styled(
                            "Redo",
                            Style::default()
                                .fg(app.theme.function_key_label_fg)
                                .bg(app.theme.function_key_label_bg),
                        ));
                        hints_vec.push(Span::raw(" | "));
                    }
                    hints_vec.push(Span::styled(
                        "Ctrl+S",
                        Style::default()
                            .fg(app.theme.function_key_number_fg)
                            .bg(app.theme.function_key_number_bg),
                    ));
                    hints_vec.push(Span::styled(
                        "Save",
                        Style::default()
                            .fg(app.theme.function_key_label_fg)
                            .bg(app.theme.function_key_label_bg),
                    ));
                    hints_vec.push(Span::raw(" | "));
                    // Show Ctrl+O hint when in remote mode
                    if let Some(ref conn) = app.remote_connection {
                        hints_vec.push(Span::styled(
                            "Ctrl+O",
                            Style::default()
                                .fg(app.theme.function_key_number_fg)
                                .bg(app.theme.function_key_number_bg),
                        ));
                        hints_vec.push(Span::styled(
                            format!("Disconnect({})", conn.computer_name()),
                            Style::default()
                                .fg(app.theme.function_key_label_fg)
                                .bg(app.theme.function_key_label_bg),
                        ));
                        hints_vec.push(Span::raw(" | "));
                    }
                    hints_vec.push(Span::styled(
                        "Q",
                        Style::default()
                            .fg(app.theme.function_key_number_fg)
                            .bg(app.theme.function_key_number_bg),
                    ));
                    hints_vec.push(Span::styled(
                        "Quit",
                        Style::default()
                            .fg(app.theme.function_key_label_fg)
                            .bg(app.theme.function_key_label_bg),
                    ));
                    hints_vec
                }
            }
            _ => vec![
                Span::styled(
                    "ESC",
                    Style::default()
                        .fg(app.theme.function_key_number_fg)
                        .bg(app.theme.function_key_number_bg),
                ),
                Span::styled(
                    "Cancel",
                    Style::default()
                        .fg(app.theme.function_key_label_fg)
                        .bg(app.theme.function_key_label_bg),
                ),
            ],
        };

        let hints_line = Line::from(hints);
        let paragraph = Paragraph::new(hints_line).alignment(Alignment::Center);

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
                .border_style(Style::default().fg(app.theme.dialog_border_fg)),
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
                .border_style(Style::default().fg(app.theme.dialog_border_fg)),
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
                .border_style(Style::default().fg(app.theme.dialog_border_fg)),
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
                    .border_style(Style::default().fg(app.theme.dialog_border_fg)),
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
