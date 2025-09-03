use std::{
    collections::HashMap,
    io::{self, Stdout},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table, Cell},
    Terminal, Frame,
};
use tokio::sync::mpsc;

use crate::model::{HealthStatus, MonitoredService};

#[derive(Debug, Clone)]
pub enum DashboardEvent {
    ServiceUpdate(String, MonitoredService), // URL, Updated service
}

#[derive(Debug, Clone)]
pub enum FilterMode {
    All,
    HealthyOnly,
    UnhealthyOnly,
}

#[derive(Debug, Clone)]
pub enum SortMode {
    Url,
    Status,
    ResponseTime,
    NextCheck,
}

pub struct DashboardState {
    services: HashMap<String, MonitoredService>,
    filter_mode: FilterMode,
    sort_mode: SortMode,
    selected_index: usize,
    last_update: Instant,
}

impl DashboardState {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            filter_mode: FilterMode::All,
            sort_mode: SortMode::Url,
            selected_index: 0,
            last_update: Instant::now(),
        }
    }

    pub fn update_service(&mut self, url: String, service: MonitoredService) {
        self.services.insert(url, service);
        self.last_update = Instant::now();
    }

    pub fn get_filtered_services(&self) -> Vec<&MonitoredService> {
        let mut filtered: Vec<&MonitoredService> = self.services.values()
            .filter(|service| match self.filter_mode {
                FilterMode::All => true,
                FilterMode::HealthyOnly => service.health_status == HealthStatus::Healthy,
                FilterMode::UnhealthyOnly => service.health_status == HealthStatus::Unhealthy,
            })
            .collect();

        match self.sort_mode {
            SortMode::Url => filtered.sort_by(|a, b| a.url.cmp(&b.url)),
            SortMode::Status => filtered.sort_by(|a, b| {
                match (&a.health_status, &b.health_status) {
                    (HealthStatus::Unhealthy, HealthStatus::Healthy) => std::cmp::Ordering::Less,
                    (HealthStatus::Healthy, HealthStatus::Unhealthy) => std::cmp::Ordering::Greater,
                    _ => a.url.cmp(&b.url),
                }
            }),
            SortMode::ResponseTime => filtered.sort_by(|a, b| {
                match (a.response_time, b.response_time) {
                    (Some(a_time), Some(b_time)) => a_time.cmp(&b_time),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => a.url.cmp(&b.url),
                }
            }),
            SortMode::NextCheck => filtered.sort_by(|a, b| {
                match (a.next_check, b.next_check) {
                    (Some(a_check), Some(b_check)) => a_check.cmp(&b_check),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => a.url.cmp(&b.url),
                }
            }),
        }

        filtered
    }

    pub fn toggle_filter(&mut self) {
        self.filter_mode = match self.filter_mode {
            FilterMode::All => FilterMode::HealthyOnly,
            FilterMode::HealthyOnly => FilterMode::UnhealthyOnly,
            FilterMode::UnhealthyOnly => FilterMode::All,
        };
        self.selected_index = 0; // Reset selection when filter changes
    }

    pub fn toggle_sort(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::Url => SortMode::Status,
            SortMode::Status => SortMode::ResponseTime,
            SortMode::ResponseTime => SortMode::NextCheck,
            SortMode::NextCheck => SortMode::Url,
        };
        self.selected_index = 0; // Reset selection when sort changes
    }

    pub fn select_previous(&mut self) {
        let filtered_count = self.get_filtered_services().len();
        if filtered_count > 0 {
            self.selected_index = if self.selected_index == 0 {
                filtered_count - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn select_next(&mut self) {
        let filtered_count = self.get_filtered_services().len();
        if filtered_count > 0 {
            self.selected_index = (self.selected_index + 1) % filtered_count;
        }
    }
}

pub struct Dashboard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    state: DashboardState,
    rx: mpsc::UnboundedReceiver<DashboardEvent>,
}

impl Dashboard {
    pub fn new(rx: mpsc::UnboundedReceiver<DashboardEvent>) -> Result<Self, io::Error> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            state: DashboardState::new(),
            rx,
        })
    }

    pub async fn run(&mut self) -> Result<(), io::Error> {
        loop {
            // Handle incoming events from monitoring tasks
            while let Ok(event) = self.rx.try_recv() {
                match event {
                    DashboardEvent::ServiceUpdate(url, service) => {
                        self.state.update_service(url, service);
                    }
                }
            }

            // Draw the dashboard
            let state = &self.state; // Create a reference to avoid borrowing issues
            self.terminal.draw(|f| draw_dashboard(f, state))?;

            // Handle keyboard input
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Char('f') => self.state.toggle_filter(),
                            KeyCode::Char('s') => self.state.toggle_sort(),
                            KeyCode::Up | KeyCode::Char('k') => self.state.select_previous(),
                            KeyCode::Down | KeyCode::Char('j') => self.state.select_next(),
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

fn draw_dashboard(frame: &mut Frame, state: &DashboardState) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Header
        Constraint::Min(10),   // Main table
        Constraint::Length(3), // Footer
    ]).split(frame.area());

    draw_header(frame, chunks[0], state);
    draw_services_table(frame, chunks[1], state);
    draw_footer(frame, chunks[2]);
}

fn draw_header(frame: &mut Frame, area: Rect, state: &DashboardState) {
    let title = format!(
        "Fluxa Dashboard - Filter: {:?} | Sort: {:?} | Services: {}",
        state.filter_mode,
        state.sort_mode,
        state.services.len()
    );

    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Status"));

    frame.render_widget(header, area);
}

fn draw_services_table(frame: &mut Frame, area: Rect, state: &DashboardState) {
    let filtered_services = state.get_filtered_services();
    
    let header_cells = vec![
        Cell::from("URL").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Response Time").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Next Check").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Error").style(Style::default().add_modifier(Modifier::BOLD)),
    ];
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Blue))
        .height(1);

    let rows: Vec<Row> = filtered_services
        .iter()
        .enumerate()
        .map(|(i, service)| {
            let status_style = match service.health_status {
                HealthStatus::Healthy => Style::default().fg(Color::Green),
                HealthStatus::Unhealthy => Style::default().fg(Color::Red),
            };

            let response_time_str = service.response_time
                .map(|rt| format!("{:.2}ms", rt.as_secs_f64() * 1000.0))
                .unwrap_or_else(|| "N/A".to_string());

            let next_check_str = service.next_check
                .map(|next| {
                    let now = Instant::now();
                    if next > now {
                        let remaining = next - now;
                        format_duration(remaining)
                    } else {
                        "Now".to_string()
                    }
                })
                .unwrap_or_else(|| "N/A".to_string());

            let error_str = service.error_message
                .as_ref()
                .map(|e| truncate_string(e, 30))
                .unwrap_or_else(|| "-".to_string());

            let cells = vec![
                Cell::from(truncate_string(&service.url, 40)),
                Cell::from(format!("{:?}", service.health_status)).style(status_style),
                Cell::from(response_time_str),
                Cell::from(next_check_str),
                Cell::from(error_str).style(Style::default().fg(Color::Red)),
            ];

            let row_style = if i == state.selected_index {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(cells).style(row_style)
        })
        .collect();

    let table = Table::new(rows, [
        Constraint::Percentage(30), // URL
        Constraint::Percentage(15), // Status
        Constraint::Percentage(15), // Response Time
        Constraint::Percentage(20), // Next Check
        Constraint::Percentage(20), // Error
    ])
    .header(header)
    .block(Block::default().borders(Borders::ALL).title("Monitored Services"))
    .row_highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_widget(table, area);
}

fn draw_footer(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(vec![
            Span::styled("q/Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Quit | "),
            Span::styled("f", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Filter | "),
            Span::styled("s", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Sort | "),
            Span::styled("↑/k", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Up | "),
            Span::styled("↓/j", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": Down"),
        ]),
    ];

    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Help"));

    frame.render_widget(footer, area);
}

impl Drop for Dashboard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m{}s", secs / 60, secs % 60)
    } else {
        format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}