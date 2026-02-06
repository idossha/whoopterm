use anyhow::Result;
use clap::Parser;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Clear},
    Frame, Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

mod api;
mod auth;
mod config;
mod data;

use api::WhoopAPI;
use data::{DashboardData};

#[derive(Parser)]
#[command(name = "whoop")]
#[command(about = "WHOOP fitness dashboard for the terminal")]
#[command(version)]
struct Cli {
    /// Refresh data
    #[arg(short, long)]
    refresh: bool,
    
    /// Authenticate with WHOOP
    #[arg(short, long)]
    auth: bool,
    
    /// Test API connectivity
    #[arg(long)]
    test: bool,
}

struct App {
    data: Option<DashboardData>,
    api: WhoopAPI,
    error_message: Option<String>,
}

impl App {
    fn new() -> Self {
        Self {
            data: None,
            api: WhoopAPI::new(),
            error_message: None,
        }
    }

    async fn load_data(&mut self) -> Result<()> {
        match self.api.load_cached_or_refresh().await {
            Ok(data) => {
                self.data = Some(data);
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Error: {}", e));
            }
        }
        Ok(())
    }

    async fn refresh_data(&mut self) -> Result<()> {
        match self.api.refresh_all_data().await {
            Ok(data) => {
                self.data = Some(data);
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Error: {}", e));
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    // Handle CLI arguments
    if cli.auth {
        // Run authentication
        if let Err(e) = app.api.authenticate().await {
            app.error_message = Some(format!("Authentication failed: {}", e));
        }
    } else if cli.test {
        // Test API
        match app.api.test_connection().await {
            Ok(_) => {
                app.error_message = Some("API test successful!".to_string());
            }
            Err(e) => {
                app.error_message = Some(format!("API test failed: {}", e));
            }
        }
    } else {
        // Load data
        if cli.refresh {
            let _ = app.refresh_data().await;
        } else {
            let _ = app.load_data().await;
        }
    }

    // Run main loop
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let mut last_tick = std::time::Instant::now();
    let tick_rate = std::time::Duration::from_millis(250);

    loop {
        terminal.draw(|f| ui(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| std::time::Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('r') => {
                        let _ = app.refresh_data().await;
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = std::time::Instant::now();
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();
    
    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(2),  // User info
            Constraint::Min(10),    // Main content
            Constraint::Length(1),  // Footer
        ])
        .split(size);

    // Header
    let header = Paragraph::new("WHOOP DASHBOARD")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded),
        );
    f.render_widget(header, chunks[0]);

    // User info
    if let Some(data) = &app.data {
        let user_text = if let Some(profile) = &data.profile {
            format!(
                "  {} {}  |  ID: {}",
                profile.first_name, profile.last_name, profile.user_id
            )
        } else {
            "  Not authenticated".to_string()
        };
        let user_info = Paragraph::new(user_text)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(user_info, chunks[1]);
    }

    // Error message
    if let Some(error) = &app.error_message {
        let error_widget = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Red)));
        let area = centered_rect(60, 20, size);
        f.render_widget(Clear, area);
        f.render_widget(error_widget, area);
        return;
    }

    // Main content
    if let Some(data) = &app.data {
        render_main_content(f, chunks[2], data);
    }

    // Footer
    let footer = Paragraph::new("[r] Refresh  |  [q] Quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[3]);
}

fn render_main_content(f: &mut Frame, area: Rect, data: &DashboardData) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Recovery + Sleep
            Constraint::Length(12), // Sleep history
            Constraint::Min(6),    // Workouts
        ])
        .split(area);

    // Recovery and Today's Sleep
    render_today_metrics(f, chunks[0], data);

    // Sleep history
    render_sleep_history(f, chunks[1], data);

    // Workouts
    render_workouts(f, chunks[2], data);
}

fn render_today_metrics(f: &mut Frame, area: Rect, data: &DashboardData) {
    let block = Block::default()
        .title(" Today's Metrics ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded);
    
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut text = vec![];

    // Recovery
    if let Some(recovery) = data.recovery.first() {
        let score = recovery.score.recovery_score;
        let bar = create_horizontal_bar(score, 100, 30);
        text.push(Line::from(vec![
            Span::styled("  Recovery: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:3}% ", score), Style::default().fg(get_score_color(score))),
            Span::styled(bar, Style::default().fg(Color::Gray)),
        ]));
        text.push(Line::from(vec![
            Span::styled(
                format!("    RHR: {} bpm    HRV: {:.1} ms", recovery.score.resting_heart_rate, recovery.score.hrv_rmssd_milli),
                Style::default().fg(Color::Gray),
            ),
        ]));
    }

    text.push(Line::from(""));

    // Sleep
    if let Some(sleep) = data.sleep.first() {
        let total = sleep.score.stage_summary.total_in_bed_time_milli / 60000;
        let efficiency = sleep.score.sleep_efficiency_percentage;
        text.push(Line::from(vec![
            Span::styled("  Last Night's Sleep: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}  |  Efficiency: {}%", format_duration(total), efficiency),
                Style::default().fg(Color::White),
            ),
        ]));

        // Sleep stages
        let stages = &sleep.score.stage_summary;
        let awake = stages.total_awake_time_milli / 60000;
        let light = stages.total_light_sleep_time_milli / 60000;
        let deep = stages.total_slow_wave_sleep_time_milli / 60000;
        let rem = stages.total_rem_sleep_time_milli / 60000;
        let total_all = awake + light + deep + rem;

        if total_all > 0 {
            text.push(Line::from(vec![
                Span::styled(
                    format!(
                        "    Awake: {} ({:.0}%)  Light: {} ({:.0}%)  Deep: {} ({:.0}%)  REM: {} ({:.0}%)",
                        format_duration_short(awake),
                        (awake as f64 / total_all as f64) * 100.0,
                        format_duration_short(light),
                        (light as f64 / total_all as f64) * 100.0,
                        format_duration_short(deep),
                        (deep as f64 / total_all as f64) * 100.0,
                        format_duration_short(rem),
                        (rem as f64 / total_all as f64) * 100.0
                    ),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    }

    let paragraph = Paragraph::new(text);
    f.render_widget(paragraph, inner);
}

fn render_sleep_history(f: &mut Frame, area: Rect, data: &DashboardData) {
    let block = Block::default()
        .title(" Sleep History (7 Days) ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded);

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Create table for sleep history
    let header_cells = ["Date", "Hours", "Graph", "Efficiency"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows: Vec<Row> = data
        .sleep
        .iter()
        .take(7)
        .map(|sleep| {
            let rfc_date = sleep.start.to_rfc3339();
            let date = rfc_date.split('T').next().unwrap_or("");
            let hours = sleep.score.stage_summary.total_in_bed_time_milli as f64 / 3600000.0;
            let bar = create_horizontal_bar(hours as i32, 10, 20);
            let efficiency = sleep.score.sleep_efficiency_percentage;

            let cells = vec![
                Cell::from(date.to_string()),
                Cell::from(format!("{:.1}h", hours)),
                Cell::from(bar).style(Style::default().fg(Color::Cyan)),
                Cell::from(format!("{}%", efficiency)),
            ];
            Row::new(cells).height(1)
        })
        .collect();

    let table = Table::new(rows)
        .header(header)
        .block(Block::default())
        .widths(&[
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Length(22),
            Constraint::Length(12),
        ]);

    f.render_widget(table, inner);
}

fn render_workouts(f: &mut Frame, area: Rect, data: &DashboardData) {
    let block = Block::default()
        .title(" Recent Workouts ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let header_cells = ["Date", "Activity", "Strain", "Duration", "HR"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows: Vec<Row> = data
        .workouts
        .iter()
        .take(5)
        .map(|workout| {
            let rfc_date = workout.start.to_rfc3339();
            let date = rfc_date.split('T').next().unwrap_or("");
            let activity = workout.sport_name.clone();
            let strain_bar = create_horizontal_bar((workout.score.strain * 10.0) as i32, 210, 10);
            let strain = format!("{:.1}", workout.score.strain);
            let duration = format_duration(
                ((workout.end.timestamp() - workout.start.timestamp()) / 60) as i64
            );
            let hr = format!("{} bpm", workout.score.average_heart_rate);

            let cells = vec![
                Cell::from(date.to_string()),
                Cell::from(activity),
                Cell::from(format!("{} {}", strain_bar, strain)),
                Cell::from(duration),
                Cell::from(hr),
            ];
            Row::new(cells).height(1)
        })
        .collect();

    let table = Table::new(rows)
        .header(header)
        .block(Block::default())
        .widths(&[
            Constraint::Length(12),
            Constraint::Length(16),
            Constraint::Length(16),
            Constraint::Length(10),
            Constraint::Length(10),
        ]);

    f.render_widget(table, inner);
}

fn create_horizontal_bar(value: i32, max: i32, width: usize) -> String {
    let ratio = (value as f64 / max as f64).min(1.0);
    let filled = (ratio * width as f64) as usize;
    let filled_str = "█".repeat(filled);
    let empty_str = "░".repeat(width - filled);
    format!("{}{}", filled_str, empty_str)
}

fn get_score_color(score: i32) -> Color {
    if score >= 67 {
        Color::Green
    } else if score >= 33 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn format_duration(minutes: i64) -> String {
    let hours = minutes / 60;
    let mins = minutes % 60;
    if hours > 0 {
        format!("{}h{:02}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

fn format_duration_short(minutes: i64) -> String {
    let hours = minutes / 60;
    let mins = minutes % 60;
    format!("{}:{:02}", hours, mins)
}

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
