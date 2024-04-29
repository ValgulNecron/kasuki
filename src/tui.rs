use std::io::stdout;
use std::{fs, io};

use crossterm::event::{Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, ExecutableCommand};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Span, Text};
use ratatui::widgets::{Block, Borders, LineGauge, Paragraph, Wrap};
use ratatui::{symbols, Frame, Terminal};
use sysinfo::System;

use crate::constant::{APP_VERSION, BOT_INFO, LOGS_PATH, TUI_FG_COLOR};

pub async fn create_tui() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(ui)?;
        should_quit = handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn handle_events() -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn ui(frame: &mut Frame) {
    let mut sys = System::new_all();
    sys.refresh_all();
    let total_cpu_core = sys.cpus().len();
    let processes = sys.processes();
    let pid = &sysinfo::get_current_pid().unwrap();
    let process = processes.get(&pid).unwrap();
    let app_cpu_usage = process.cpu_usage();
    let memory_usage = process.memory();
    let app_cpu_usage = app_cpu_usage / total_cpu_core as f32;
    let app_cpu_usage = format!("{:.2}%", app_cpu_usage);
    let app_memory_usage = format!("{:.2}Mb", memory_usage / 1024 / 1024);

    let mut system_cpu_usage: f64 = 0.0;
    for cpu in sys.cpus() {
        system_cpu_usage += cpu.cpu_usage() as f64
    }
    system_cpu_usage = system_cpu_usage / total_cpu_core as f64;
    let system_memory_usage = sys.used_memory();
    let system_total_memory = sys.total_memory();
    let system_memory_ratio = system_memory_usage as f64 / system_total_memory as f64;
    let system_cpu_ratio = system_cpu_usage / 100.0;
    let disk_read = process.disk_usage().total_read_bytes;
    let disk_write = process.disk_usage().total_written_bytes;

    let main_layout = Layout::new(
        Direction::Vertical,
        [Constraint::Min(3), Constraint::Fill(99)],
    )
    .split(frame.size());

    let logs = read_logs().unwrap_or_else(|e| format!("Error reading logs: {}", e));
    let lines_count = logs.lines().count() as u16; // Calculate the total number of lines
    let offset = lines_count - frame.size().height;
    let text = Text::from(logs).style(Style::default().fg(TUI_FG_COLOR));
    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .title(Span::styled("Logs", Style::default().fg(TUI_FG_COLOR)))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(TUI_FG_COLOR)),
            )
            .wrap(Wrap { trim: true })
            .scroll((offset, 0)),
        main_layout[1],
    );

    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(3),
        ])
        .split(main_layout[0]);

    frame.render_widget(
        LineGauge::default()
            .block(
                Block::default()
                    .title(Span::styled("CPU Usage", Style::default().fg(TUI_FG_COLOR)))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(TUI_FG_COLOR)),
            )
            .style(Style::default().fg(TUI_FG_COLOR))
            .gauge_style(Style::default().fg(Color::Green))
            .line_set(symbols::line::THICK)
            .ratio(system_cpu_ratio),
        inner_layout[0],
    );
    frame.render_widget(
        LineGauge::default()
            .block(
                Block::default()
                    .title(Span::styled(
                        "Memory Usage",
                        Style::default().fg(TUI_FG_COLOR),
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(TUI_FG_COLOR)),
            )
            .style(Style::default().fg(TUI_FG_COLOR))
            .gauge_style(Style::default().fg(Color::Green))
            .line_set(symbols::line::THICK)
            .ratio(system_memory_ratio),
        inner_layout[1],
    );
    let bot = unsafe { BOT_INFO.clone() };
    let name = match bot {
        Some(bot) => bot.name,
        None => process.name().to_string(),
    };
    let second_inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .split(inner_layout[2]);
    frame.render_widget(
        Paragraph::new(Span::styled(name, Style::default().fg(TUI_FG_COLOR))).block(
            Block::default()
                .title(Span::styled("Name", Style::default().fg(TUI_FG_COLOR)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(TUI_FG_COLOR)),
        ),
        second_inner_layout[0],
    );
    frame.render_widget(
        Paragraph::new(Span::styled(APP_VERSION, Style::default().fg(TUI_FG_COLOR))).block(
            Block::default()
                .title(Span::styled("Version", Style::default().fg(TUI_FG_COLOR)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(TUI_FG_COLOR)),
        ),
        second_inner_layout[1],
    );
    frame.render_widget(
        Paragraph::new(Span::styled(
            pid.to_string(),
            Style::default().fg(TUI_FG_COLOR),
        ))
        .block(
            Block::default()
                .title(Span::styled("PID", Style::default().fg(TUI_FG_COLOR)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(TUI_FG_COLOR)),
        ),
        second_inner_layout[2],
    );
    frame.render_widget(
        Paragraph::new(Span::styled(
            app_cpu_usage.to_string(),
            Style::default().fg(TUI_FG_COLOR),
        ))
        .block(
            Block::default()
                .title(Span::styled("CPU", Style::default().fg(TUI_FG_COLOR)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(TUI_FG_COLOR)),
        ),
        second_inner_layout[3],
    );
    frame.render_widget(
        Paragraph::new(Span::styled(
            app_memory_usage.to_string(),
            Style::default().fg(TUI_FG_COLOR),
        ))
        .block(
            Block::default()
                .title(Span::styled("Memory", Style::default().fg(TUI_FG_COLOR)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(TUI_FG_COLOR)),
        ),
        second_inner_layout[4],
    );
    frame.render_widget(
        Paragraph::new(Span::styled(
            disk_read.to_string(),
            Style::default().fg(TUI_FG_COLOR),
        ))
        .block(
            Block::default()
                .title(Span::styled("Disk read", Style::default().fg(TUI_FG_COLOR)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(TUI_FG_COLOR)),
        ),
        second_inner_layout[5],
    );
    frame.render_widget(
        Paragraph::new(Span::styled(
            disk_write.to_string(),
            Style::default().fg(TUI_FG_COLOR),
        ))
        .block(
            Block::default()
                .title(Span::styled(
                    "Disk write",
                    Style::default().fg(TUI_FG_COLOR),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(TUI_FG_COLOR)),
        ),
        second_inner_layout[6],
    );
}

fn read_logs() -> Result<String, io::Error> {
    // read the latest logs from the logs directory
    // first list the contents of the logs directory
    let paths = fs::read_dir(LOGS_PATH)?;
    // now get the latest log file
    let mut latest_file = None;
    let mut latest_time = 0;
    for path in paths {
        let path = path?.path();
        let metadata = fs::metadata(&path)?;
        let modified = metadata.modified().unwrap().elapsed().unwrap().as_secs();
        if latest_file.is_none() || modified > latest_time {
            latest_time = modified;
            latest_file = Some(path);
        }
    }
    // now read the contents of the latest log file
    let latest_file = latest_file.unwrap();
    let contents = fs::read_to_string(latest_file)?;
    Ok(contents)
}
