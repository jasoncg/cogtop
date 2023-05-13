use sysinfo::{System, SystemExt, CpuExt};
use std::io;
use std::time::Duration;
use std::thread;
use tui::Terminal;
use tui::backend::CrosstermBackend;
use tui::layout::{Layout, Constraint, Direction};
use tui::widgets::{Widget, Gauge, Block, Borders};
use tui::style::{Color, Modifier, Style};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::fs;

use serde::{Serialize, Deserialize};


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
}

fn main() -> Result<(), io::Error> {
    let cli = Cli::parse();

    let mut sys = System::new_all();

    // terminal setup
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        // refresh system info
        sys.refresh_all();

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ].as_ref()
                )
                .split(f.size());

            // calculate memory usage in percentage
            let total_memory = sys.total_memory() as f64;
            let used_memory = sys.used_memory() as f64;
            let memory_usage = used_memory / total_memory * 100.0;

            // calculate cpu usage
            let cpu_usage = sys.global_cpu_info().cpu_usage() as f64;

            // create memory gauge
            let memory_gauge = Gauge::default()
                .block(Block::default().title("Memory Usage").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::White))
                .percent(memory_usage as u16);

            // create cpu gauge
            let cpu_gauge = Gauge::default()
                .block(Block::default().title("CPU Usage").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::White))
                .percent(cpu_usage as u16);

            f.render_widget(memory_gauge, chunks[0]);
            f.render_widget(cpu_gauge, chunks[1]);
        })?;

        // sleep for a bit to prevent high CPU usage
        thread::sleep(Duration::from_millis(1000));
    }
}
/*
impl Drop for Terminal<CrosstermBackend<io::Stdout>> {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
    }
}
*/