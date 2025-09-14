use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::system::{SystemMonitor, SortOrder};
use crate::ui;

pub struct App {
    system_monitor: Arc<RwLock<SystemMonitor>>,
    selected_process: usize,
    sort_order: SortOrder,
    last_update: Instant,
    update_interval: Duration,
    should_quit: bool,
    debug_mode: bool,
}

impl App {
    pub fn new(update_interval: Duration, debug: bool) -> Result<Self> {
        let system_monitor = Arc::new(RwLock::new(SystemMonitor::new()));
        
        Ok(Self {
            system_monitor,
            selected_process: 0,
            sort_order: SortOrder::Cpu,
            last_update: Instant::now(),
            update_interval,
            should_quit: false,
            debug_mode: debug,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // spawn background task for system updates
        let monitor_clone = Arc::clone(&self.system_monitor);
        let interval = self.update_interval;
        tokio::spawn(async move {
            loop {
                {
                    let mut monitor = monitor_clone.write().await;
                    monitor.refresh();
                }
                tokio::time::sleep(interval).await;
            }
        });

        // main event loop
        let res = self.run_app(&mut terminal).await;

        // restore terminal
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

    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            self.draw(terminal).await?;

            // handle events with timeout to allow for regular redraws
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => {
                            self.should_quit = true;
                        }
                        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                            self.should_quit = true;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.move_selection_up().await;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.move_selection_down().await;
                        }
                        KeyCode::Char('K') => {
                            self.kill_selected_process().await?;
                        }
                        KeyCode::Char('c') => {
                            self.sort_order = SortOrder::Cpu;
                            self.selected_process = 0;
                        }
                        KeyCode::Char('m') => {
                            self.sort_order = SortOrder::Memory;
                            self.selected_process = 0;
                        }
                        _ => {}
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    async fn draw<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let monitor = self.system_monitor.read().await;
        
        terminal.draw(|f| {
            ui::draw_ui(
                f,
                &monitor,
                self.selected_process,
                &self.sort_order,
                self.debug_mode,
            );
        })?;
        
        Ok(())
    }

    async fn move_selection_up(&mut self) {
        if self.selected_process > 0 {
            self.selected_process -= 1;
        }
    }

    async fn move_selection_down(&mut self) {
        let monitor = self.system_monitor.read().await;
        let processes = monitor.get_processes(&self.sort_order);
        if self.selected_process < processes.len().saturating_sub(1) {
            self.selected_process += 1;
        }
    }

    async fn kill_selected_process(&mut self) -> Result<()> {
        let monitor = self.system_monitor.read().await;
        let processes = monitor.get_processes(&self.sort_order);
        
        if let Some(process) = processes.get(self.selected_process) {
            // attempt to kill the process (requires appropriate permissions)
            #[cfg(unix)]
            {
                use std::process::Command;
                let _ = Command::new("kill")
                    .arg("-9")
                    .arg(process.pid.to_string())
                    .output();
            }
            
            #[cfg(windows)]
            {
                use std::process::Command;
                let _ = Command::new("taskkill")
                    .args(&["/F", "/PID", &process.pid.to_string()])
                    .output();
            }
        }
        
        Ok(())
    }
}