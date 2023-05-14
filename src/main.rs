use sysinfo::{DiskExt, System, SystemExt, CpuExt};
use std::io;
use std::time::{Duration, Instant};
use std::thread;
use crossterm::{
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, Paragraph, Widget, Wrap};
use tui::{symbols, Frame};
use tui::text::{Span, Spans, Text};
use chrono::Local;
use clap::Parser;

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
/**
 * CPUChart: Class which tracks cpu utilization and returns a chart as requested
 */
pub struct CPUTracker {
    buffer: Vec<(f64, f64)>,
    buffer_size: usize,
    sys: System
}
impl CPUTracker {
    pub fn new(size:usize) -> Self {
        Self {
            buffer: Vec::new(),
            buffer_size: size,
            sys: System::new_all()
        }
    }
    /** Returns a Chart of utilization of time.
     * @param sys: System
     * @param start_time: Instant
     * @param buffer: Vec<(f64, f64)>
     * @return Chart
     */
    pub fn chart(&mut self, start_time:Instant, frame: &mut Frame<impl tui::backend::Backend>, area: Rect){ 
        self.sys.refresh_cpu();
        let cpu_usage = self.sys.global_cpu_info().cpu_usage() as f64;
        let elapsed = start_time.elapsed().as_secs() as f64;
        self.buffer.push((elapsed, cpu_usage));
        if self.buffer.len() > self.buffer_size {
            self.buffer.remove(0);
        }
        let datasets = vec![
            Dataset::default()
                .name("CPU Usage")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Red))
                .data(&self.buffer),
        ];
        //let cpu_usage_str: String = format!("CPU {:.2}%", cpu_usage);
        let cpu_header = Spans::from(vec![
            Span::styled("CPU: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:.2}%", cpu_usage), Style::default().fg(Color::Red))
        ]);
        let chart= Chart::new(datasets)
            .block(Block::default().title(cpu_header).borders(Borders::ALL))
            .x_axis(
                Axis::default()
                    //.title("Time in seconds")
                    .style(Style::default().fg(Color::White))
                    .bounds([elapsed - self.buffer_size as f64, elapsed]),
            )
            .y_axis(
                Axis::default()
                    //.title("CPU Usage")
                    .style(Style::default().fg(Color::White))
                    .bounds([0.0, 100.0]),
            );
        frame.render_widget(chart, area);
    }
}
pub struct MemTracker {
    buffer_mem: Vec<(f64, f64)>,
    buffer_swap: Vec<(f64, f64)>,
    buffer_size: usize,
    sys: System
}
impl MemTracker {
    pub fn new(size:usize) -> Self {
        Self {
            buffer_mem: Vec::new(),
            buffer_swap: Vec::new(),
            buffer_size: size,
            sys: System::new_all()
        }
    }
    /** Renders a Chart of utilization of time
     * @param sys: System
     * @param start_time: Instant
     * @param frame
     * @param area
     */
    pub fn chart(&mut self, start_time:Instant, frame: &mut Frame<impl tui::backend::Backend>, area: Rect) {
        self.sys.refresh_memory();

        let memory_usage = self.sys.used_memory() as f64;
        let memory_total = self.sys.total_memory() as f64;
        let mem_pct = memory_usage/memory_total*100.0;

        let swap_usage = self.sys.used_swap() as f64;
        let swap_total = self.sys.total_swap() as f64;
        let swap_pct = swap_usage/swap_total*100.0;

        let elapsed = start_time.elapsed().as_secs() as f64;

        self.buffer_mem.push((elapsed, mem_pct));
        if self.buffer_mem.len() > self.buffer_size {
            self.buffer_mem.remove(0);
        }
        self.buffer_swap.push((elapsed, swap_pct));
        if self.buffer_swap.len() > self.buffer_size {
            self.buffer_swap.remove(0);
        }

        // Make string for "Memory {usage} / {total}"
        //let mem_message = format!("Memory: RAM {:.2}% Swap {:.2}%", (memory_usage/memory_total*100.0), (swap_usage/swap_total*100.0));
        let mem_header = Spans::from(vec![
            Span::styled("Memory: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(format!("RAM {:.2}% ", mem_pct), Style::default().fg(Color::Cyan)),
            Span::styled(format!("Swap {:.2}%", swap_pct), Style::default().fg(Color::Yellow)),
        ]);

        let datasets = vec![
            Dataset::default()
                .name("Memory Usage")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Cyan))
                .data(&self.buffer_mem),
            Dataset::default()
                .name("Swap Usage")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Yellow))
                .data(&self.buffer_swap),
        ];
        let chart= Chart::new(datasets)
            .block(Block::default().title(mem_header).borders(Borders::ALL))
            .x_axis(
                Axis::default()
                    //.title("Time in seconds")
                    .style(Style::default().fg(Color::White))
                    .bounds([elapsed - self.buffer_size as f64, elapsed]),
            )
            .y_axis(
                Axis::default()
                    //.title("Usage")
                    .style(Style::default().fg(Color::White))
                    .bounds([0.0, 100.0]),
            );
        frame.render_widget(chart, area);
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
    let mut cpu_tracker = CPUTracker::new(60);
    let mut ram_tracker = MemTracker::new(60);

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
            
            let disk_column = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints(
                    [
                        Constraint::Percentage(100)
                    ].as_ref()
                )
                .split(top_row[1]);

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
            
            //iterate over disks
            for disk in sys.disks() {
                let disk_usage = disk.total_space() - disk.available_space();
                let total_space = disk.total_space();
                let disk_usage_ratio = disk_usage as f64 / total_space as f64 * 100.0;

                let disk_gauge = Gauge::default()
                    .block(Block::default().title("Disk").borders(Borders::ALL))
                    .gauge_style(Style::default().fg(Color::White))
                    .label(disk.name().to_str().unwrap_or_default())
                    .percent(disk_usage_ratio as u16);
                
                f.render_widget(disk_gauge, disk_column[0]);
            }

            cpu_tracker.chart(start_time, f, bottom_row[0]);
            ram_tracker.chart(start_time, f, bottom_row[1]);
        })?;

        // sleep for a bit to prevent high CPU usage
        //thread::sleep(Duration::from_millis(1000));
    }
    return Ok(());
}

