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

mod security; // Evasion Logic
mod poly;     // Polymorphic Seed
mod obfuscation; // String Hiding
mod scanner;  // Heuristic Discovery
mod shredder; // Secure Deletion
mod grim_reaper; // Boot-time Deletion
mod registry_hunter; // Registry Cleaning
mod immolation; // Self Deletion
mod dynamo;     // Dynamic API Loading
mod engine;   // Core Logic
mod chronos;  // Temporal Integrity (NTP)
mod chaos;    // Entropy Ocean (Anti-Forensics)
mod math_trap; // Opaque Predicates (Decompiler Confusion)
mod black_hole; // Deep Clean (Windows Updates, etc.)

// --- 🛠️ ENGINEERING: RAII COM GUARD ---
struct ComGuard;

impl ComGuard {
    fn new() -> Result<Self> {
        unsafe {
            use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
            // Ignore error if already initialized
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
    // 1. Setup UI
    // AUTO-ELEVATION (Before UI init)
    if !security::am_i_admin() {
        security::elevate_self();
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Channels
    let (tx, mut rx) = mpsc::channel(100);

    // 3. SPAWN ENGINE THREAD
    std::thread::spawn(move || {
        let _guard = ComGuard::new().unwrap(); 
        
        // ANTI-ANALYSIS (Ghost Protocol)
        // 0. Parent Validation (Void)
        if !security::is_safe_parent() {
             std::process::exit(0); // Silently exit if launched by untrusted parent
        }

        let _ = tx.blocking_send(WorkerMsg::Log("SECURITY".into(), "Initiating Ghost Protocol & Abyss Checks...".into(), Color::Magenta));
        
        // 0.5. Entropy Ocean (Abyss - Anti-Forensics)
        // Allocate massive memory to confuse dumps and exhaust small VMs
        let _ocean = chaos::EntropyOcean::summon();
        let _ = tx.blocking_send(WorkerMsg::Log("ABYSS".into(), "Entropy Ocean Summoned (256MB Chaos)".into(), Color::Magenta));

        // 0.6. NTP Reality Check (Abyss - Temporal Integrity)
        if !chronos::reality_check() {
             let _ = tx.blocking_send(WorkerMsg::Log("CRITICAL".into(), "TEMPORAL ANOMALY DETECTED (Time Warp)".into(), Color::Red));
             security::crash_dummy();
             std::process::exit(1);
        }
            
        // 1. Debugger Check
        if security::check_debugger() {
                security::crash_dummy();
                std::process::exit(1);
        }

            // 2. Time-Warp Check
            if security::detect_time_warping() {
                 security::crash_dummy();
                 std::process::exit(1);
            }
            
            // 2.5. Opaque Predicates (Abyss - CFG Confusion)
            if !math_trap::verify_reality() {
                 // Mathematical impossibility occurred (Cosmic Ray? or Patching?)
                 let _ = tx.blocking_send(WorkerMsg::Log("CRITICAL".into(), "REALITY COLLAPSE DETECTED".into(), Color::Red));
                 security::crash_dummy();
                 std::process::exit(1);
            }
            // 4. Hunter Process Check (Immediate)
            if security::check_hunter_processes() {
                 security::crash_dummy();
                 std::process::exit(1);
            }
            // 5. Human Verification
            let _ = tx.blocking_send(WorkerMsg::Log("SECURITY".into(), "Verifying Human Presence... (Move Mouse)".into(), Color::Yellow));
            security::verify_human_presence();
            let _ = tx.blocking_send(WorkerMsg::Log("SECURITY".into(), "Human Verified. Access Granted.".into(), Color::Green));

        // RUN ENGINE
        let engine_result = tokio::runtime::Runtime::new().unwrap().block_on(engine::Engine::run(tx.clone()));
        
        if let Err(e) = engine_result {
            let _ = tx.blocking_send(WorkerMsg::Log("ERROR".into(), format!("CRITICAL FAILURE: {}", e), Color::Red));
        }

        let _ = tx.blocking_send(WorkerMsg::Done);
    });

    // 4. UI LOOP
    let start_time = Instant::now();
    let mut logs: Vec<LogMessage> = Vec::new();
    let mut progress = 0.0;
    let mut status = "INITIALIZING...".to_string();
    let mut files_scanned = 0;
    let bytes_cleaned = 0.0; // Mock metric for now

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
                    if logs.len() > 100 { logs.remove(0); } // Keep history bounded
                    files_scanned += 1; // Approximate activity metric
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
                .block(Block::default().borders(Borders::ALL).title(" ⚡ KAWAII CLEANER PRO v10.0.0 "))
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
                .block(Block::default().borders(Borders::ALL).title(" 📜 SYSTEM EVENT LOG "));
            f.render_widget(logs_list, chunks[1]);
            
            // Auto-scroll logic: Render the last items
            // Ratatui list auto-scrolls if we implement state, but for simplicity here we just show the list.
            // A more advanced TUI would use ListState.

            // Footer
            let elapsed = start_time.elapsed().as_secs_f64();
            let stats = format!(" 📁 Files: {} | 💾 Cleaned: {:.2} MB | ⏱️ Time: {:.1}s ", files_scanned, bytes_cleaned, elapsed);
            let footer = Paragraph::new(stats)
                .block(Block::default().borders(Borders::ALL).title(" 📊 STATISTICS "))
                .style(Style::default().fg(Color::Magenta));
            f.render_widget(footer, chunks[2]);

        })?;

        // Input Handling
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        if progress >= 1.0 && status == "SYSTEM OPTIMIZED" {
            // Keep open for user to see 'q' to exit
        }
    }

    // 5. Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}
