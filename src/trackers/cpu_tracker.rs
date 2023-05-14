use sysinfo::{System, SystemExt, CpuExt};
use std::time::Instant;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Axis, Block, Borders, Chart, Dataset};
use tui::{symbols, Frame};
use tui::text::{Span, Spans, Text};


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
                .style(Style::default().fg(Color::LightRed))
                .data(&self.buffer),
        ];
        //let cpu_usage_str: String = format!("CPU {:.2}%", cpu_usage);
        let cpu_header = Spans::from(vec![
            Span::styled("CPU: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:.2}%", cpu_usage), Style::default().fg(Color::LightRed))
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