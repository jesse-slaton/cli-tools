use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, Wrap},
    Frame,
};

use crate::app::{App, ConfirmAction, InputMode, Mode, Panel};
use crate::path_analyzer::PathStatus;

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
                format!("U:{}", stats.user_duplicates),
                Style::default().fg(app.theme.path_duplicate_fg),
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
                    .fg(Color::Cyan)
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

        let title = vec![
            Line::from(vec![
                Span::styled(
                    "Path Commander",
                    Style::default()
                        .fg(app.theme.header_fg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - Windows PATH Environment Manager"),
            ]),
            Line::from(second_line_spans),
        ];

        let header = Paragraph::new(title)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Left);

        f.render_widget(header, area);
    }

    fn render_panel(&self, f: &mut Frame, area: Rect, app: &App, panel: Panel) {
        let is_active = app.active_panel == panel;
        let (paths, info, selected, marked, scrollbar_state) = match panel {
            Panel::Machine => (
                &app.machine_paths,
                &app.machine_info,
                app.machine_selected,
                &app.machine_marked,
                &app.machine_scrollbar_state,
            ),
            Panel::User => (
                &app.user_paths,
                &app.user_info,
                app.user_selected,
                &app.user_marked,
                &app.user_scrollbar_state,
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

        let title = format!(
            " {} {} {}",
            panel.scope().as_str(),
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
            Style::default().fg(Color::DarkGray)
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

                let mut style = Style::default().fg(color);
                if is_selected {
                    style = style.add_modifier(Modifier::REVERSED);
                }

                ListItem::new(display).style(style)
            })
            .collect();

        let list = List::new(items).block(block).highlight_style(
            Style::default()
                .add_modifier(Modifier::REVERSED)
                .add_modifier(Modifier::BOLD),
        );

        f.render_widget(list, chunks[0]);

        // Render scrollbar
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .thumb_style(if is_active {
                Style::default().fg(app.theme.panel_border_fg)
            } else {
                Style::default().fg(Color::Gray)
            })
            .track_style(Style::default().fg(Color::DarkGray));

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
                Style::default().fg(Color::Yellow),
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
                    vec![
                        Span::styled("F1", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Help │ "),
                        Span::styled("/", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Clear Filter │ "),
                        Span::styled("Ctrl+A", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Mark All │ "),
                        Span::styled("F3", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Del │ "),
                        Span::styled("Ctrl+S", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Save │ "),
                        Span::styled("Q", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Quit"),
                    ]
                } else if total_marked > 0 {
                    // When items are marked - show bulk operations
                    vec![
                        Span::styled("F1", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Help │ "),
                        Span::styled("F3", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Delete │ "),
                        Span::styled("F5", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Move │ "),
                        Span::styled("F9", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Normalize │ "),
                        Span::styled("Ctrl+Shift+U", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Unmark All │ "),
                        Span::styled("Ctrl+S", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Save │ "),
                        Span::styled("Q", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Quit"),
                    ]
                } else {
                    // Normal mode - default hints with more discoverable features
                    vec![
                        Span::styled("F1", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Help │ "),
                        Span::styled("F2", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Mark │ "),
                        Span::styled("F3", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Del │ "),
                        Span::styled("F4", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Add │ "),
                        Span::styled("/", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Filter │ "),
                        Span::styled("Ctrl+A", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Mark All │ "),
                        Span::styled("Ctrl+S", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Save │ "),
                        Span::styled("Q", Style::default().fg(app.theme.header_fg)),
                        Span::raw(" Quit"),
                    ]
                }
            }
            _ => vec![
                Span::styled("ESC", Style::default().fg(app.theme.header_fg)),
                Span::raw(" Cancel"),
            ],
        };

        let hints_line = Line::from(hints);
        let paragraph = Paragraph::new(hints_line).alignment(Alignment::Center);

        f.render_widget(paragraph, area);
    }

    fn render_help(&self, f: &mut Frame, app: &App) {
        let help_text = vec![
            Line::from(vec![Span::styled(
                "Path Commander - Help",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Navigation:",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from("  ↑/↓, j/k        Move selection up/down"),
            Line::from("  PgUp/PgDn       Move selection by 10"),
            Line::from("  Home/End        Move to first/last item"),
            Line::from("  Tab, ←/→        Switch between panels"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Selection:",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from("  Space, Insert   Toggle mark on current item"),
            Line::from("  F2              Toggle mark (Midnight Commander style)"),
            Line::from("  Ctrl+A          Mark all visible paths in current scope"),
            Line::from("  Ctrl+Shift+A    Mark all paths in both scopes"),
            Line::from("  Ctrl+D          Mark all duplicates in current scope"),
            Line::from("  Ctrl+Shift+D    Mark all dead paths in current scope"),
            Line::from("  Ctrl+N          Mark all non-normalized paths"),
            Line::from("  Ctrl+Shift+U    Unmark all"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Filtering:",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from("  /               Open filter menu"),
            Line::from("                  • Clear filter (show all)"),
            Line::from("                  • Dead paths"),
            Line::from("                  • Duplicates"),
            Line::from("                  • Non-normalized paths"),
            Line::from("                  • Valid paths only"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Actions:",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from("  F3, Delete      Delete marked items"),
            Line::from("  F4              Add new path"),
            Line::from("  F5              Move marked items to other panel"),
            Line::from("  F6              Move item up in order"),
            Line::from("  F7              Remove all duplicates"),
            Line::from("  F8              Remove all dead paths"),
            Line::from("  F9              Normalize selected paths"),
            Line::from("  F10             Create marked dead directories"),
            Line::from("  Enter           Edit current path"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Save/Restore:",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from("  Ctrl+S          Apply changes to registry"),
            Line::from("  Ctrl+B          Create backup"),
            Line::from("  Ctrl+R          Restore from backup"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Color Legend:",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Red", Style::default().fg(Color::Red)),
                Span::raw(" - Dead path (does not exist)"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Yellow", Style::default().fg(Color::Yellow)),
                Span::raw(" - Duplicate path"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Cyan", Style::default().fg(Color::Cyan)),
                Span::raw(" - Non-normalized (contains ~, env vars)"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Green", Style::default().fg(Color::Green)),
                Span::raw(" - Valid, unique, normalized"),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Press ESC or F1 to close this help",
                Style::default().fg(Color::Yellow),
            )]),
        ];

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.dialog_border_fg)),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });

        let area = centered_rect(80, 90, f.area());
        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(help, area);
    }

    fn render_process_restart_info(&self, f: &mut Frame, app: &App) {
        let mut lines = vec![
            Line::from(vec![Span::styled(
                "PATH Changes Applied Successfully!",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Important: Some running processes need to be restarted",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from("The following processes won't pick up the new PATH until restarted:"),
            Line::from(""),
        ];

        // Add each process to the list
        for process in &app.processes_to_restart {
            lines.push(Line::from(vec![
                Span::raw("  • "),
                Span::styled(process, Style::default().fg(Color::Cyan)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Why restart?",
            Style::default().add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(
            "These processes load environment variables at startup and don't respond to",
        ));
        lines.push(Line::from(
            "WM_SETTINGCHANGE notifications. You'll need to close and reopen them to see",
        ));
        lines.push(Line::from("the updated PATH."));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "Note: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("New processes started after this point will see the updated PATH."),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Press ENTER or ESC to continue",
            Style::default().fg(Color::Yellow),
        )]));

        let info = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.dialog_border_fg))
                    .title(" Process Restart Required "),
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
                    Style::default().fg(Color::Yellow),
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
                message_lines.push(Line::from(vec![Span::raw(
                    "Create directory and add to PATH?",
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
                    Style::default().fg(Color::Gray),
                )]));
            }
        }

        message_lines.push(Line::from(""));
        message_lines.push(Line::from(vec![
            Span::styled("Y", Style::default().fg(app.theme.button_focused_bg)),
            Span::raw("es / "),
            Span::styled("N", Style::default().fg(app.theme.error_fg)),
            Span::raw("o"),
        ]));

        let text = message_lines;

        let dialog = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Confirm ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.dialog_border_fg)),
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
        };

        let text = vec![
            Line::from(""),
            Line::from(vec![Span::raw(&app.input_buffer)]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Enter to confirm, ESC to cancel",
                Style::default().fg(Color::Gray),
            )]),
        ];

        let input = Paragraph::new(text)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.dialog_border_fg)),
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

                let mut style = Style::default();
                if is_selected {
                    style = style.add_modifier(Modifier::REVERSED);
                }

                ListItem::new(filename).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Select Backup to Restore ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .add_modifier(Modifier::BOLD),
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

                let mut style = Style::default();
                if is_current {
                    style = style.fg(Color::Cyan);
                }
                if is_selected {
                    style = style.add_modifier(Modifier::REVERSED);
                }

                ListItem::new(display).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Filter Paths ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .add_modifier(Modifier::BOLD),
            );

        let area = centered_rect(60, 60, f.area());
        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(list, area);
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
