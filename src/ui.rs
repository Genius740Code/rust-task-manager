use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Gauge, Paragraph, Row, Sparkline, Table, Wrap,
    },
    Frame,
};

use crate::system::{SortOrder, SystemMonitor};

pub fn draw_ui(
    f: &mut Frame,
    monitor: &SystemMonitor,
    selected_process: usize,
    sort_order: &SortOrder,
    debug_mode: bool,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Length(8),  // cpu/memory info
            Constraint::Min(10),    // process table
            Constraint::Length(2),  // footer
        ])
        .split(f.size());

    draw_header(f, chunks[0], monitor);
    draw_system_stats(f, chunks[1], monitor);
    draw_process_table(f, chunks[2], monitor, selected_process, sort_order);
    draw_footer(f, chunks[3], debug_mode);
}

fn draw_header(f: &mut Frame, area: Rect, monitor: &SystemMonitor) {
    let system_info = monitor.get_system_info();
    let uptime_hours = system_info.uptime / 3600;
    let uptime_mins = (system_info.uptime % 3600) / 60;

    let header_text = vec![
        Line::from(vec![
            Span::styled("SysTop", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" - System Monitor"),
        ]),
        Line::from(vec![
            Span::raw("Host: "),
            Span::styled(&system_info.hostname, Style::default().fg(Color::Green)),
            Span::raw(" | Uptime: "),
            Span::styled(
                format!("{}h {}m", uptime_hours, uptime_mins),
                Style::default().fg(Color::Yellow),
            ),
        ]),
    ];

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)))
        .alignment(Alignment::Left);

    f.render_widget(header, area);
}

fn draw_system_stats(f: &mut Frame, area: Rect, monitor: &SystemMonitor) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_cpu_stats(f, chunks[0], monitor);
    draw_memory_stats(f, chunks[1], monitor);
}

fn draw_cpu_stats(f: &mut Frame, area: Rect, monitor: &SystemMonitor) {
    let cpu_info = monitor.get_cpu_info();
    
    let cpu_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(3); cpu_info.len().min(4)]) // show max 4 cores
        .split(area);

    for (i, cpu) in cpu_info.iter().enumerate().take(4) {
        if i < cpu_chunks.len() {
            let gauge = Gauge::default()
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(format!("CPU {}", i + 1)))
                .gauge_style(Style::default().fg(match cpu.usage as u16 {
                    0..=50 => Color::Green,
                    51..=80 => Color::Yellow,
                    _ => Color::Red,
                }))
                .percent(cpu.usage as u16)
                .label(format!("{:.1}%", cpu.usage));

            f.render_widget(gauge, cpu_chunks[i]);
        }
    }
}

fn draw_memory_stats(f: &mut Frame, area: Rect, monitor: &SystemMonitor) {
    let memory_percent = monitor.get_memory_percent();
    let used_memory = monitor.get_used_memory();
    let total_memory = monitor.get_total_memory();
    let memory_history = monitor.get_memory_history();

    let memory_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(area);

    // memory gauge
    let memory_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Memory"))
        .gauge_style(Style::default().fg(match memory_percent as u16 {
            0..=60 => Color::Green,
            61..=85 => Color::Yellow,
            _ => Color::Red,
        }))
        .percent(memory_percent as u16)
        .label(format!(
            "{:.1}% ({:.1}GB / {:.1}GB)",
            memory_percent,
            used_memory as f64 / 1024.0 / 1024.0 / 1024.0,
            total_memory as f64 / 1024.0 / 1024.0 / 1024.0
        ));

    f.render_widget(memory_gauge, memory_chunks[0]);

    // memory history sparkline
    if !memory_history.is_empty() {
        let sparkline_data: Vec<u64> = memory_history.iter().map(|&x| x as u64).collect();
        let sparkline = Sparkline::default()
            .block(Block::default().borders(Borders::ALL).title("Memory History"))
            .data(&sparkline_data)
            .style(Style::default().fg(Color::Cyan));
        
        f.render_widget(sparkline, memory_chunks[1]);
    }
}

fn draw_process_table(
    f: &mut Frame,
    area: Rect,
    monitor: &SystemMonitor,
    selected_process: usize,
    sort_order: &SortOrder,
) {
    let processes = monitor.get_processes(sort_order);
    
    let header_cells = ["PID", "Name", "CPU%", "Memory", "Mem%"]
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let style = match (i, sort_order) {
                (0, SortOrder::Pid) => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                (1, SortOrder::Name) => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                (2, SortOrder::Cpu) => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                (4, SortOrder::Memory) => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                _ => Style::default(),
            };
            Span::styled(*h, style)
        });

    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Blue))
        .height(1)
        .bottom_margin(1);

    let rows = processes.iter().enumerate().map(|(i, process)| {
        let style = if i == selected_process {
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let memory_mb = process.memory as f64 / 1024.0 / 1024.0;
        let memory_display = if memory_mb >= 1024.0 {
            format!("{:.1}GB", memory_mb / 1024.0)
        } else {
            format!("{:.0}MB", memory_mb)
        };

        Row::new(vec![
            process.pid.to_string(),
            process.name.clone(),
            format!("{:.1}", process.cpu_usage),
            memory_display,
            format!("{:.2}", process.memory_percent),
        ])
        .style(style)
    });

    let process_table = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Processes (sorted by {:?})", sort_order))
        )
        .widths(&[
            Constraint::Length(8),
            Constraint::Min(20),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(8),
        ])
        .column_spacing(1);

    f.render_widget(process_table, area);
}

fn draw_footer(f: &mut Frame, area: Rect, debug_mode: bool) {
    let mut footer_text = vec![
        Line::from("Controls: ↑/↓ or j/k (navigate) | K (kill process) | c (sort by CPU) | m (sort by memory) | q (quit)")
    ];

    if debug_mode {
        footer_text.push(Line::from(Span::styled(
            "DEBUG MODE ACTIVE", 
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        )));
    }

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    f.render_widget(footer, area);
}