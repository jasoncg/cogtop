use sysinfo::{System, SystemExt, DiskExt};
use std::convert::TryInto;
use std::time::Instant;
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, Gauge};
use tui::{symbols, Frame};
use tui::text::{Span, Spans, Text};
use tui::layout::{Constraint, Direction, Layout, Rect};
use std::collections::HashSet;

pub struct StorageTracker {
    buffer: Vec<(f64, f64)>,
    buffer_size: usize,
    sys: System
}
impl StorageTracker {
    pub fn new(size:usize) -> Self {
        Self {
            buffer: Vec::new(),
            buffer_size: size,
            sys: System::new_all()
        }
    }
    

// ...

pub fn gauges(&mut self, frame: &mut Frame<impl tui::backend::Backend>, area: Rect) {
    self.sys.refresh_all();
    let disks = self.sys.disks();

    let mut unique_disk_names: Vec<String> = Vec::new(); // List to store unique disk names

    // Iterate over disks
    for disk in disks.iter() {
        let disk_name = disk.name().to_str().unwrap_or_default().to_owned();

        // Check if the disk name is already in the list of unique names
        if !unique_disk_names.contains(&disk_name) {
            unique_disk_names.push(disk_name);
        }
    }

    let unique_disk_count = unique_disk_names.len();
    if unique_disk_count == 0 {
        return;
    }

    // Calculate the available height for each gauge
    let gauge_height = (100.0 / unique_disk_count as f64).floor() as u16;

    let disk_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(
            vec![Constraint::Percentage(gauge_height) ; unique_disk_count]
        )
        .split(area);

    // Iterate over unique disk names
    for (i, disk_name) in unique_disk_names.iter().enumerate() {
        // Find the first occurrence of the disk with the unique name
        let disk = disks.iter().find(|d| d.name().to_str().unwrap_or_default() == *disk_name);
        if let Some(disk) = disk {
            let disk_usage = disk.total_space() - disk.available_space();   // bytes
            let total_space = disk.total_space();                           // bytes
            let disk_usage_ratio = disk_usage as f64 / total_space as f64 * 100.0;
            //let title = format!("{} {:.2} / {:.2}GB", disk_name, disk_usage as f64 / 1_000_000_000.0, total_space as f64 / 1_000_000_000.0);
            let label = format!("{:.2}% ({:.2} / {:.2}GB)", disk_usage_ratio, disk_usage as f64 / 1_000_000_000.0, total_space as f64 / 1_000_000_000.0);

            let disk_gauge = Gauge::default()
                .block(Block::default().title(disk_name.to_string()).borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Blue).bg(Color::Black).add_modifier(Modifier::BOLD))
                .label(label)
                .percent(disk_usage_ratio as u16);

            frame.render_widget(disk_gauge, disk_layout[i]);
        }
    }
}

    

    /*
    pub fn chart(&mut self, start_time:Instant, frame: &mut Frame<impl tui::backend::Backend>, area: Rect){ 
        
        frame.render_widget(chart, area);
    }*/
}