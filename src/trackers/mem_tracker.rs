use sysinfo::{System, SystemExt};
use std::time::Instant;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Axis, Block, Borders, Chart, Dataset};
use tui::{symbols, Frame};
use tui::text::{Span, Spans};


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
