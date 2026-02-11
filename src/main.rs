use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Terminal,
};
use std::{io, time::{Duration, Instant}};
use tokio::sync::mpsc;

mod evasion;    // Singularity: Unified Evasion
mod cleaning;   // Singularity: Unified Cleaning
mod dynamo;     // Dynamic API Loading
mod engine;     // Core Logic
mod poly;       // Polymorphic Seed
mod dark_matter; // Encrypted Strings
mod quantum;     // Direct Syscalls
pub mod structs;     // Kernel Structures

// --- 🛠️ ENGINEERING: RAII COM GUARD ---
struct ComGuard;

impl ComGuard {
    fn new() -> Result<Self> {
        unsafe {
            use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED); 
        }
        Ok(ComGuard)
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe {
            use windows::Win32::System::Com::CoUninitialize;
            CoUninitialize();
        }
    }
}

// --- 📨 MESSAGING SYSTEM ---
#[derive(Debug)]
pub enum WorkerMsg {
    Log(String, String, Color), // Level, Message, Color
    Progress(f64, String),      // Percentage, Status Text
    Done,
}

struct LogMessage {
    timestamp: String,
    level: String,
    content: String,
    color: Color,
}

// --- 🚀 MAIN ENTRY POINT ---
#[tokio::main]
async fn main() -> Result<()> {
    // 1. Auto-Elevation
    if !evasion::am_i_admin() {
        evasion::elevate_self();
        return Ok(());
    }

    // 2. Setup UI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3. Channels
    let (tx, mut rx) = mpsc::channel(100);

    // 4. SPAWN ENGINE THREAD
    std::thread::spawn(move || {
        let _guard = ComGuard::new().unwrap(); 

        // --- SINGULARITY: EVASION PROTOCOL ---
        if let Some(mut _evasion) = evasion::EvasionSystem::new() {
             let _ = tx.blocking_send(WorkerMsg::Log("SECURITY".into(), "Singularity Evasion Active".into(), Color::Green));
             let _ = tx.blocking_send(WorkerMsg::Log("ABYSS".into(), "Entropy Ocean Summoned & Ghost Heartbeat Active".into(), Color::Magenta));
        } else {
             std::process::exit(1);
        }

        // Human Verification
        let _ = tx.blocking_send(WorkerMsg::Log("SECURITY".into(), "Verifying Human Presence...".into(), Color::Yellow));
        evasion::verify_human_presence();
        let _ = tx.blocking_send(WorkerMsg::Log("ACCESS".into(), "Identity Confirmed.".into(), Color::Green));

        // RUN ENGINE
        let engine_result = tokio::runtime::Runtime::new().unwrap().block_on(engine::Engine::run(tx.clone()));
        
        if let Err(e) = engine_result {
            let _ = tx.blocking_send(WorkerMsg::Log("ERROR".into(), format!("CRITICAL FAILURE: {}", e), Color::Red));
        }

        let _ = tx.blocking_send(WorkerMsg::Done);
    });

    // 5. UI LOOP
    let start_time = Instant::now();
    let mut logs: Vec<LogMessage> = Vec::new();
    let mut progress = 0.0;
    let mut status = "INITIALIZING...".to_string();
    let mut files_scanned = 0; 
    let _bytes_cleaned = 0.0; 

    loop {
        // Handle Messages
        while let Ok(msg) = rx.try_recv() {
            match msg {
                WorkerMsg::Log(level, content, color) => {
                    logs.push(LogMessage {
                        timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
                        level,
                        content,
                        color,
                    });
                    if logs.len() > 100 { logs.remove(0); }
                    files_scanned += 1; 
                }
                WorkerMsg::Progress(p, s) => {
                    progress = p;
                    status = s;
                }
                WorkerMsg::Done => {
                    progress = 1.0;
                    status = "SYSTEM OPTIMIZED".to_string();
                }
            }
        }

        // Draw UI
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Header
                    Constraint::Min(10),    // Logs
                    Constraint::Length(3),  // Footer
                ])
                .split(size);

            // Header
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title(" ⚡ KAWAII CLEANER PRO v18.0.0 (SINGULARITY) "))
                .gauge_style(Style::default().fg(Color::Cyan).bg(Color::Black))
                .percent((progress * 100.0) as u16)
                .label(format!("{:.1}% - {}", progress * 100.0, status));
            f.render_widget(gauge, chunks[0]);

            // Logs
            let log_items: Vec<ListItem> = logs.iter().map(|l| {
                ListItem::new(format!("{} [{}] {}", l.timestamp, l.level, l.content))
                    .style(Style::default().fg(l.color))
            }).collect();
            
            let logs_list = List::new(log_items)
                .block(Block::default().borders(Borders::ALL).title(" 📜 EVENT HORIZON LOG "));
            f.render_widget(logs_list, chunks[1]);
            
            // Footer
            let elapsed = start_time.elapsed().as_secs_f64();
            let stats = format!(" 📁 Activity: {} | ⏱️ Time: {:.1}s ", files_scanned, elapsed);
            let footer = Paragraph::new(stats)
                .block(Block::default().borders(Borders::ALL).title(" 📊 STATISTICS "))
                .style(Style::default().fg(Color::Magenta));
            f.render_widget(footer, chunks[2]);

        })?;

        // Input 
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}
