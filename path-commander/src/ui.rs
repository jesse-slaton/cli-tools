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
            Mode::Help => self.render_help(f),
            Mode::Confirm(action) => self.render_confirm(f, app, action),
            Mode::BackupList => self.render_backup_list(f, app),
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

        let title = vec![
            Line::from(vec![
                Span::styled("Path Commander", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" - Windows PATH Environment Manager"),
            ]),
            Line::from(vec![
                Span::raw("Total: "),
                Span::styled(
                    format!("M:{} ", stats.machine_total),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("U:{} ", stats.user_total),
                    Style::default().fg(Color::White),
                ),
                Span::raw("│ Dead: "),
                Span::styled(
                    format!("M:{} ", stats.machine_dead),
                    Style::default().fg(Color::Red),
                ),
                Span::styled(
                    format!("U:{} ", stats.user_dead),
                    Style::default().fg(Color::Red),
                ),
                Span::raw("│ Duplicates: "),
                Span::styled(
                    format!("M:{} ", stats.machine_duplicates),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!("U:{}", stats.user_duplicates),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(" │ "),
                Span::styled(
                    if app.has_changes { "MODIFIED" } else { "Clean" },
                    if app.has_changes {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Green)
                    },
                ),
            ]),
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

        // Split area: List (left) and Scrollbar (right 1 column)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),      // List takes remaining space
                Constraint::Length(1),   // Scrollbar takes 1 column
            ])
            .split(area);

        let title = format!(
            " {} {} ",
            panel.scope().as_str(),
            if !app.is_admin && panel == Panel::Machine {
                "[READ-ONLY]"
            } else {
                ""
            }
        );

        let border_style = if is_active {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        let items: Vec<ListItem> = paths
            .iter()
            .enumerate()
            .map(|(idx, path)| {
                let is_selected = idx == selected && is_active;
                let is_marked = marked.contains(&idx);

                let status = info.get(idx).map(|i| i.status).unwrap_or(PathStatus::Valid);
                let color = self.get_status_color(status);

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
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            })
            .track_style(Style::default().fg(Color::DarkGray));

        // Clone state for rendering (render_stateful_widget needs &mut)
        let mut scrollbar_state_mut = scrollbar_state.clone();
        f.render_stateful_widget(scrollbar, chunks[1], &mut scrollbar_state_mut);
    }

    fn render_status(&self, f: &mut Frame, area: Rect, app: &App) {
        let status_text = vec![
            Line::from(vec![
                Span::styled(
                    if app.is_admin { "ADMIN " } else { "USER " },
                    Style::default().fg(if app.is_admin { Color::Green } else { Color::Yellow }),
                ),
                Span::raw("│ "),
                Span::raw(&app.status_message),
            ]),
        ];

        let status = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Left);

        f.render_widget(status, area);
    }

    fn render_key_hints(&self, f: &mut Frame, area: Rect, app: &App) {
        let hints = match app.mode {
            Mode::Normal => vec![
                Span::styled("F1", Style::default().fg(Color::Cyan)),
                Span::raw(" Help │ "),
                Span::styled("F2", Style::default().fg(Color::Cyan)),
                Span::raw(" Mark │ "),
                Span::styled("F3", Style::default().fg(Color::Cyan)),
                Span::raw(" Del │ "),
                Span::styled("F4", Style::default().fg(Color::Cyan)),
                Span::raw(" Add │ "),
                Span::styled("F5", Style::default().fg(Color::Cyan)),
                Span::raw(" Move │ "),
                Span::styled("F9", Style::default().fg(Color::Cyan)),
                Span::raw(" Normalize │ "),
                Span::styled("Ctrl+S", Style::default().fg(Color::Cyan)),
                Span::raw(" Save │ "),
                Span::styled("Ctrl+B", Style::default().fg(Color::Cyan)),
                Span::raw(" Backup │ "),
                Span::styled("Q", Style::default().fg(Color::Cyan)),
                Span::raw(" Quit"),
            ],
            _ => vec![
                Span::styled("ESC", Style::default().fg(Color::Cyan)),
                Span::raw(" Cancel"),
            ],
        };

        let hints_line = Line::from(hints);
        let paragraph = Paragraph::new(hints_line).alignment(Alignment::Center);

        f.render_widget(paragraph, area);
    }

    fn render_help(&self, f: &mut Frame) {
        let help_text = vec![
            Line::from(vec![Span::styled(
                "Path Commander - Help",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled("Navigation:", Style::default().add_modifier(Modifier::BOLD))]),
            Line::from("  ↑/↓, j/k        Move selection up/down"),
            Line::from("  PgUp/PgDn       Move selection by 10"),
            Line::from("  Home/End        Move to first/last item"),
            Line::from("  Tab, ←/→        Switch between panels"),
            Line::from(""),
            Line::from(vec![Span::styled("Selection:", Style::default().add_modifier(Modifier::BOLD))]),
            Line::from("  Space, Insert   Toggle mark on current item"),
            Line::from("  F2              Toggle mark (Midnight Commander style)"),
            Line::from(""),
            Line::from(vec![Span::styled("Actions:", Style::default().add_modifier(Modifier::BOLD))]),
            Line::from("  F3, Delete      Delete marked items"),
            Line::from("  F4              Add new path"),
            Line::from("  F5              Move marked items to other panel"),
            Line::from("  F6              Move item up in order"),
            Line::from("  F7              Remove all duplicates"),
            Line::from("  F8              Remove all dead paths"),
            Line::from("  F9              Normalize selected paths"),
            Line::from("  Enter           Edit current path"),
            Line::from(""),
            Line::from(vec![Span::styled("Save/Restore:", Style::default().add_modifier(Modifier::BOLD))]),
            Line::from("  Ctrl+S          Apply changes to registry"),
            Line::from("  Ctrl+B          Create backup"),
            Line::from("  Ctrl+R          Restore from backup"),
            Line::from(""),
            Line::from(vec![Span::styled("Color Legend:", Style::default().add_modifier(Modifier::BOLD))]),
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
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });

        let area = centered_rect(80, 90, f.area());
        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(help, area);
    }

    fn render_confirm(&self, f: &mut Frame, app: &App, action: ConfirmAction) {
        let message = match action {
            ConfirmAction::Exit => {
                if app.has_changes {
                    "You have unsaved changes. Exit anyway?"
                } else {
                    "Exit Path Commander?"
                }
            }
            ConfirmAction::DeleteSelected => "Delete selected paths?",
            ConfirmAction::DeleteAllDead => "Delete all dead paths?",
            ConfirmAction::DeleteAllDuplicates => "Delete all duplicate paths?",
            ConfirmAction::ApplyChanges => {
                "Apply changes to Windows Registry?\n\nThis will modify your PATH environment variables."
            }
            ConfirmAction::RestoreBackup => "Restore from selected backup?",
            ConfirmAction::CreateSingleDirectory => {
                "Directory does not exist.\n\nCreate directory and add to PATH?"
            }
            ConfirmAction::CreateMarkedDirectories => {
                "Create all marked dead directories?\n\n(Network paths and invalid paths will be skipped)"
            }
        };

        let text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                message,
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Y", Style::default().fg(Color::Green)),
                Span::raw("es / "),
                Span::styled("N", Style::default().fg(Color::Red)),
                Span::raw("o"),
            ]),
        ];

        let dialog = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Confirm ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
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
                    .border_style(Style::default().fg(Color::Cyan)),
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

    fn get_status_color(&self, status: PathStatus) -> Color {
        match status {
            PathStatus::Valid => Color::Green,
            PathStatus::Dead => Color::Red,
            PathStatus::Duplicate => Color::Yellow,
            PathStatus::NonNormalized => Color::Cyan,
            PathStatus::DeadDuplicate => Color::Red,
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
