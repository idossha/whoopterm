use anyhow::Result;
use clap::Parser;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
    Frame, Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::time::{Duration, Instant};

mod api;
mod auth;
mod config;
mod data;

use api::WhoopAPI;
use data::{DashboardData, SleepScore};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REFRESH_INTERVAL: Duration = Duration::from_secs(300); // Auto-refresh every 5 minutes

#[derive(Parser)]
#[command(name = "whoopterm")]
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
    last_refresh: Option<Instant>,
    loading: bool,
}

impl App {
    fn new() -> Self {
        Self {
            data: None,
            api: WhoopAPI::new(),
            error_message: None,
            last_refresh: None,
            loading: false,
        }
    }

    async fn load_data(&mut self) -> Result<()> {
        self.loading = true;
        match self.api.load_cached_or_refresh().await {
            Ok(data) => {
                self.data = Some(data);
                self.error_message = None;
                self.last_refresh = Some(Instant::now());
            }
            Err(e) => {
                self.error_message = Some(format!("{}", e));
            }
        }
        self.loading = false;
        Ok(())
    }

    async fn refresh_data(&mut self) -> Result<()> {
        self.loading = true;
        match self.api.refresh_all_data().await {
            Ok(data) => {
                self.data = Some(data);
                self.error_message = None;
                self.last_refresh = Some(Instant::now());
            }
            Err(e) => {
                self.error_message = Some(format!("{}", e));
            }
        }
        self.loading = false;
        Ok(())
    }

    fn should_auto_refresh(&self) -> bool {
        if let Some(last) = self.last_refresh {
            last.elapsed() > REFRESH_INTERVAL
        } else {
            false
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut app = App::new();

    // Handle --auth and --test before entering TUI mode
    if cli.auth {
        if let Err(e) = app.api.authenticate().await {
            eprintln!("Authentication failed: {:#}", e);
            std::process::exit(1);
        }
        return Ok(());
    }

    if cli.test {
        match app.api.test_connection().await {
            Ok(_) => {
                println!("API test successful!");
            }
            Err(e) => {
                eprintln!("API test failed: {}", e);
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    // Load data before entering TUI
    if cli.refresh {
        let _ = app.refresh_data().await;
    } else {
        let _ = app.load_data().await;
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

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
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        terminal.draw(|f| ui(f, app))?;

        // Auto-refresh every 5 minutes
        if app.should_auto_refresh() {
            let _ = app.refresh_data().await;
        }

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

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
            last_tick = Instant::now();
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// UI Rendering
// ─────────────────────────────────────────────────────────────────────────────

fn ui(f: &mut Frame, app: &App) {
    let size = f.area();
    
    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),  // Header bar
            Constraint::Length(10), // Recovery + Sleep row
            Constraint::Min(6),     // Sleep history (flexible)
            Constraint::Min(6),     // Workouts (flexible)
            Constraint::Length(1),  // Footer
        ])
        .split(size);

    // Header
    render_header(f, chunks[0], app);

    // Error overlay if present
    if let Some(error) = &app.error_message {
        render_error_popup(f, size, error);
        return;
    }

    if let Some(data) = &app.data {
        // Recovery + Sleep side by side
        render_recovery_and_sleep(f, chunks[1], data);
        
        // Sleep history
        render_sleep_history(f, chunks[2], data);
        
        // Workouts
        render_workouts(f, chunks[3], data);
    } else if app.loading {
        let loading = Paragraph::new("Loading...")
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);
        f.render_widget(loading, chunks[1]);
    }

    // Footer
    render_footer(f, chunks[4]);
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let profile_name = app.data.as_ref()
        .and_then(|d| d.profile.as_ref())
        .map(|p| format!("{} {}", p.first_name, p.last_name))
        .unwrap_or_else(|| "WHOOPTERM".to_string());
    
    let refresh_text = if let Some(last) = app.last_refresh {
        let elapsed = last.elapsed();
        if elapsed.as_secs() < 60 {
            "just now".to_string()
        } else if elapsed.as_secs() < 3600 {
            format!("{}m ago", elapsed.as_secs() / 60)
        } else {
            format!("{}h ago", elapsed.as_secs() / 3600)
        }
    } else {
        "never".to_string()
    };

    let header_spans = vec![
        Span::styled(profile_name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("Last updated: {}", refresh_text), Style::default().fg(Color::Gray)),
        Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("v{}", VERSION), Style::default().fg(Color::DarkGray)),
    ];

    let header = Paragraph::new(Line::from(header_spans));
    f.render_widget(header, area);
}

fn render_recovery_and_sleep(f: &mut Frame, area: Rect, data: &DashboardData) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35), // Recovery
            Constraint::Percentage(65), // Sleep
        ])
        .split(area);

    render_recovery_panel(f, chunks[0], data);
    render_sleep_panel(f, chunks[1], data);
}

fn render_recovery_panel(f: &mut Frame, area: Rect, data: &DashboardData) {
    let block = Block::default()
        .title(" Recovery ")
        .title_style(Style::default().fg(Color::Cyan))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    
    let inner = block.inner(area);
    f.render_widget(block, area);

    if let Some(recovery) = data.recovery.first().and_then(|r| r.score.as_ref()) {
        let score = recovery.recovery_score as i32;
        let color = get_recovery_color(score);
        
        let bar_width = (inner.width as usize).saturating_sub(12);
        let bar = create_horizontal_bar(score, 100, bar_width);
        
        let text = vec![
            Line::from(vec![
                Span::styled(format!("{:3}%", score), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::styled(" ", Style::default()),
                Span::styled(bar, Style::default().fg(color)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("RHR  ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{:.0} bpm", recovery.resting_heart_rate), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("HRV  ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{:.1} ms", recovery.hrv_rmssd_milli), Style::default().fg(Color::White)),
            ]),
        ];
        
        let mut final_text = text;
        if let Some(spo2) = recovery.spo2_percentage {
            final_text.push(Line::from(vec![
                Span::styled("SpO₂ ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{:.0}%", spo2), Style::default().fg(Color::White)),
            ]));
        }
        if let Some(temp) = recovery.skin_temp_celsius {
            final_text.push(Line::from(vec![
                Span::styled("Skin ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{:.1}°C", temp), Style::default().fg(Color::White)),
            ]));
        }
        
        let paragraph = Paragraph::new(final_text);
        f.render_widget(paragraph, inner);
    } else {
        let no_data = Paragraph::new("No recovery data")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(no_data, inner);
    }
}

fn render_sleep_panel(f: &mut Frame, area: Rect, data: &DashboardData) {
    let block = Block::default()
        .title(" Last Night's Sleep ")
        .title_style(Style::default().fg(Color::Cyan))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    
    let inner = block.inner(area);
    f.render_widget(block, area);

    if let Some(sleep) = data.sleep.first() {
        if let Some(score) = &sleep.score {
            render_sleep_content(f, inner, score);
        } else {
            let no_data = Paragraph::new("Sleep not scored")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            f.render_widget(no_data, inner);
        }
    } else {
        let no_data = Paragraph::new("No sleep data")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(no_data, inner);
    }
}

fn render_sleep_content(f: &mut Frame, area: Rect, score: &SleepScore) {
    let stages = &score.stage_summary;
    let total_mins = stages.total_in_bed_time_milli / 60000;
    let efficiency = score.sleep_efficiency_percentage.unwrap_or(0.0);
    let performance = score.sleep_performance_percentage.unwrap_or(0.0);
    
    let awake_mins = stages.total_awake_time_milli / 60000;
    let light_mins = stages.total_light_sleep_time_milli / 60000;
    let deep_mins = stages.total_slow_wave_sleep_time_milli / 60000;
    let rem_mins = stages.total_rem_sleep_time_milli / 60000;

    // Top row: Duration, Efficiency, Performance
    let mut text = vec![
        Line::from(vec![
            Span::styled("Duration    ", Style::default().fg(Color::Gray)),
            Span::styled(format_duration(total_mins), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("     Efficiency ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.0}%", efficiency), Style::default().fg(Color::White)),
            Span::styled("     Performance ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.0}%", performance), Style::default().fg(Color::White)),
        ]),
        Line::from(""),
    ];

    // Sleep stage bars
    let bar_width = (area.width as usize).saturating_sub(18);
    if total_mins > 0 {
        text.push(create_stage_line("Awake", awake_mins, total_mins, Color::Yellow, bar_width));
        text.push(create_stage_line("Light", light_mins, total_mins, Color::Blue, bar_width));
        text.push(create_stage_line("Deep ", deep_mins, total_mins, Color::Magenta, bar_width));
        text.push(create_stage_line("REM  ", rem_mins, total_mins, Color::Cyan, bar_width));
    }

    let paragraph = Paragraph::new(text);
    f.render_widget(paragraph, area);
}

fn create_stage_line<'a>(label: &'a str, mins: i64, total: i64, color: Color, width: usize) -> Line<'a> {
    let percentage = (mins as f64 / total as f64 * 100.0) as i32;
    let bar = create_proportional_bar(mins, total, width);
    
    Line::from(vec![
        Span::styled(format!("{} ", label), Style::default().fg(Color::Gray)),
        Span::styled(format_duration(mins), Style::default().fg(Color::White)),
        Span::styled(" ", Style::default()),
        Span::styled(bar, Style::default().fg(color)),
        Span::styled(format!(" {:2}%", percentage), Style::default().fg(Color::DarkGray)),
    ])
}

fn render_sleep_history(f: &mut Frame, area: Rect, data: &DashboardData) {
    let block = Block::default()
        .title(" Sleep History (7d) ")
        .title_style(Style::default().fg(Color::Cyan))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let header_cells = vec![
        Cell::from("Date").style(Style::default().fg(Color::Gray)),
        Cell::from("Hours").style(Style::default().fg(Color::Gray)),
        Cell::from("Sleep").style(Style::default().fg(Color::Gray)),
        Cell::from("Eff.").style(Style::default().fg(Color::Gray)),
    ];
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = data
        .sleep
        .iter()
        .take(7)
        .filter(|s| s.score.is_some())
        .map(|sleep| {
            let date = format_date(&sleep.start);
            let hours = sleep.score.as_ref().map(|s| s.stage_summary.total_in_bed_time_milli as f64 / 3600000.0).unwrap_or(0.0);
            let efficiency = sleep.score.as_ref().and_then(|s| s.sleep_efficiency_percentage).unwrap_or(0.0) as i32;
            
            let bar_width = 20;
            let bar = create_horizontal_bar((hours * 10.0) as i32, 100, bar_width);
            let bar_color = if hours >= 7.0 { Color::Green } else if hours >= 6.0 { Color::Yellow } else { Color::Red };

            let cells = vec![
                Cell::from(date).style(Style::default().fg(Color::White)),
                Cell::from(format!("{:.1}h", hours)).style(Style::default().fg(Color::White)),
                Cell::from(bar).style(Style::default().fg(bar_color)),
                Cell::from(format!("{}%", efficiency)).style(Style::default().fg(Color::Gray)),
            ];
            Row::new(cells).height(1)
        })
        .collect();

    let table = Table::new(rows, vec![
        Constraint::Length(10),
        Constraint::Length(7),
        Constraint::Min(10),
        Constraint::Length(6),
    ])
    .header(header)
    .column_spacing(2);

    f.render_widget(table, inner);
}

fn render_workouts(f: &mut Frame, area: Rect, data: &DashboardData) {
    let block = Block::default()
        .title(" Recent Workouts ")
        .title_style(Style::default().fg(Color::Cyan))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let header_cells = vec![
        Cell::from("Date").style(Style::default().fg(Color::Gray)),
        Cell::from("Activity").style(Style::default().fg(Color::Gray)),
        Cell::from("Strain").style(Style::default().fg(Color::Gray)),
        Cell::from("Duration").style(Style::default().fg(Color::Gray)),
        Cell::from("Avg HR").style(Style::default().fg(Color::Gray)),
    ];
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = data
        .workouts
        .iter()
        .take(5)
        .filter(|w| w.score.is_some())
        .map(|workout| {
            let date = format_date(&workout.start);
            let activity = &workout.sport_name;
            let score = workout.score.as_ref().unwrap();
            let strain = score.strain;
            let duration_mins = (workout.end.timestamp() - workout.start.timestamp()) / 60;
            let avg_hr = score.average_heart_rate;

            let strain_bar_width = 8;
            let strain_bar = create_horizontal_bar((strain * 5.0) as i32, 100, strain_bar_width);
            let strain_color = if strain >= 15.0 { Color::Red } else if strain >= 10.0 { Color::Yellow } else { Color::Green };

            let cells = vec![
                Cell::from(date).style(Style::default().fg(Color::White)),
                Cell::from(activity.clone()).style(Style::default().fg(Color::White)),
                Cell::from(format!("{} {:.1}", strain_bar, strain)).style(Style::default().fg(strain_color)),
                Cell::from(format_duration(duration_mins)).style(Style::default().fg(Color::Gray)),
                Cell::from(format!("{}", avg_hr)).style(Style::default().fg(Color::Gray)),
            ];
            Row::new(cells).height(1)
        })
        .collect();

    let table = Table::new(rows, vec![
        Constraint::Length(10),
        Constraint::Length(16),
        Constraint::Length(14),
        Constraint::Length(10),
        Constraint::Length(8),
    ])
    .header(header)
    .column_spacing(2);

    f.render_widget(table, inner);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let footer = Paragraph::new("  r Refresh  q Quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, area);
}

fn render_error_popup(f: &mut Frame, area: Rect, error: &str) {
    let popup_area = centered_rect(80, 40, area);
    
    let error_text = format!("\n{}\n\nPress any key to continue...", error);
    let error_widget = Paragraph::new(error_text)
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Error ")
                .title_style(Style::default().fg(Color::Red))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Red))
        );
    
    f.render_widget(Clear, popup_area);
    f.render_widget(error_widget, popup_area);
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn create_horizontal_bar(value: i32, max: i32, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let ratio = (value as f64 / max as f64).min(1.0).max(0.0);
    let filled = (ratio * width as f64) as usize;
    let filled_str = "█".repeat(filled);
    let empty_str = "░".repeat(width.saturating_sub(filled));
    format!("{}{}", filled_str, empty_str)
}

fn create_proportional_bar(value: i64, total: i64, width: usize) -> String {
    if total == 0 || width == 0 {
        return "░".repeat(width);
    }
    let ratio = (value as f64 / total as f64).min(1.0).max(0.0);
    let filled = (ratio * width as f64) as usize;
    let filled_str = "█".repeat(filled);
    let empty_str = "░".repeat(width.saturating_sub(filled));
    format!("{}{}", filled_str, empty_str)
}

fn get_recovery_color(score: i32) -> Color {
    if score >= 67 {
        Color::Green
    } else if score >= 33 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn format_duration(minutes: i64) -> String {
    if minutes < 0 {
        return "--".to_string();
    }
    let hours = minutes / 60;
    let mins = minutes % 60;
    if hours > 0 {
        format!("{}h{:02}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

fn format_date(datetime: &chrono::DateTime<chrono::Utc>) -> String {
    datetime.format("%b %d").to_string()
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
