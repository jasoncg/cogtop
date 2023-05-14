mod trackers;
use std::io;
use std::time::{Duration, Instant};
use crossterm::{
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, Widget, Wrap, Gauge};
use tui::text::{Span, Spans};
use chrono::Local;
use clap::Parser;
use sysinfo::{System, SystemExt, CpuExt, DiskExt};

use trackers::cpu_tracker::CPUTracker;
use trackers::mem_tracker::MemTracker;
use trackers::storage_tracker::StorageTracker;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
}
fn format_uptime(uptime: u64) -> String {
    let secs = uptime % 60;
    let mins = (uptime / 60) % 60;
    let hours = (uptime / 3600) % 24;
    let days = uptime / 86400;

    if days > 0 {
        format!("{} days, {} hours, {} mins, {} secs", days, hours, mins, secs)
    } else if hours > 0 {
        format!("{} hours, {} mins, {} secs", hours, mins, secs)
    } else if mins > 0 {
        format!("{} mins, {} secs", mins, secs)
    } else {
        format!("{} secs", secs)
    }
}




fn main() -> Result<(), io::Error> {
    let _cli = Cli::parse();

    let mut sys = System::new_all();
    enable_raw_mode()?;
    // terminal setup
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let _tclearresult = terminal.clear();
    
    // Initialize buffers for memory and CPU usage over time
    let mut mem_buffer: Vec<(f64, f64)> = Vec::new();
    let mut cpu_buffer: Vec<(f64, f64)> = Vec::new();
    let mut start_time = Instant::now();

    // Create CPU tracker
    let mut cpu_tracker = CPUTracker::new(200);
    let mut ram_tracker = MemTracker::new(200);
    let mut storage_tracker = StorageTracker::new(200);

    loop {
        // Check for keyboard input
        if poll(Duration::from_millis(1000))? {
            let event = read()?;
            if event == Event::Key(KeyCode::Char('q').into()) {
                break;
            }   
            
        }
        // refresh system info
        sys.refresh_all();

        terminal.draw(|f| {
            let window = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(50),
                        Constraint::Percentage(50)
                    ].as_ref()
                )
                .split(f.size());

            let top_row = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ].as_ref()
                )
                .split(window[0]);

            let bottom_row = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ].as_ref()
                )
                .split(window[1]);
            
            // Block for displaying system information
            let text = vec![
                Spans::from(vec![
                    Span::styled("Hostname: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(sys.host_name().unwrap_or_default()),
                ]),
                Spans::from(vec![
                    Span::styled("CPU: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(sys.global_cpu_info().brand()),
                ]),
                // RAM info
                Spans::from(vec![
                    Span::styled("RAM: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{:.2} GB / {:.2} GB", sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0, sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0)),
                ]),
                Spans::from(vec![
                    Span::styled("Date/Time: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(Local::now().format("%m/%d/%Y %H:%M:%S").to_string()),
                ]),
                Spans::from(vec![
                    Span::styled("Uptime: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format_uptime(sys.uptime())),
                ]),
                // Add more system info here...
            ];
            let block = Paragraph::new(text)
                                    .block(Block::default()
                                        .borders(Borders::ALL)
                                        .title("System Info"))
                                    .wrap(Wrap { trim: true });
            f.render_widget(block, top_row[0]);

            // Display gauges for all disks/partitions
            storage_tracker.gauges(f, top_row[1]);
            
            cpu_tracker.chart(start_time, f, bottom_row[0]);
            ram_tracker.chart(start_time, f, bottom_row[1]);
        })?;

        // sleep for a bit to prevent high CPU usage
        //thread::sleep(Duration::from_millis(1000));
    }
    return disable_raw_mode();
}

