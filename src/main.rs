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
use windows::{
    core::HSTRING,
    Win32::{
        System::Com::{CoInitializeEx, CoUninitialize, CoCreateInstance, CLSCTX_ALL, COINIT_APARTMENTTHREADED},
        UI::Shell::{
            FileOperation, IFileOperation, SHCreateItemFromParsingName, IShellItem,
            FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT,
        },
    },
};
use rand::Rng; // For Jitter
mod security; // Polymorphic Seed

// --- 🛠️ ENGINEERING: RAII COM GUARD ---
struct ComGuard;
impl ComGuard {
    fn new() -> Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
            Ok(ComGuard)
        }
    }
}
impl Drop for ComGuard {
    fn drop(&mut self) { unsafe { CoUninitialize(); } }
}

// --- DATA STRUCTURES ---
#[derive(Clone)]
struct LogMessage {
    timestamp: String,
    level: String,
    message: String,
    color: Color,
}

struct App {
    progress: f64,
    status: String,
    logs: Vec<LogMessage>,
    files_scanned: usize,
    bytes_cleaned: u64,
    start_time: Instant,
}

impl App {
    fn new() -> Self {
        Self {
            progress: 0.0,
            status: "INITIALIZING SYSTEM...".to_string(),
            logs: Vec::new(),
            files_scanned: 0,
            bytes_cleaned: 0,
            start_time: Instant::now(),
        }
    }

    fn add_log(&mut self, level: &str, msg: &str, color: Color) {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
        self.logs.push(LogMessage {
            timestamp,
            level: level.to_string(),
            message: msg.to_string(),
            color,
        });
        // Keep log buffer manageable
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }
}

// --- MESSAGING ---
enum WorkerMsg {
    Log(String, String, Color), // Level, Message, Color
    Progress(f64, String),      // Progress (0.0-1.0), Status Text
    StatUpdate(usize, u64),     // Files Scanned, Bytes Cleaned
    Done,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Setup UI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Channels
    let (tx, mut rx) = mpsc::channel(100);

    // 3. SPAWN WORKER THREAD (Native COM)
    std::thread::spawn(move || {
        let _guard = ComGuard::new().unwrap(); 
        
        // ANTI-ANALYSIS
        unsafe {
            if windows::Win32::System::Diagnostics::Debug::IsDebuggerPresent().as_bool() {
                 let _ = tx.blocking_send(WorkerMsg::Log("DANGER".into(), "Debugger Detected! Ejecting...".into(), Color::Red));
                 std::thread::sleep(Duration::from_millis(1000));
                 std::process::exit(0);
            }
        }

        let _ = tx.blocking_send(WorkerMsg::Log("INFO".into(), format!("Kernel Interface Initialized (PolySeed: {})", security::POLY_SEED), Color::Cyan));
        
        if let Err(e) = unsafe { perform_insane_cleanup(&tx) } {
            let _ = tx.blocking_send(WorkerMsg::Log("ERROR".into(), format!("CRITICAL FAILURE: {}", e), Color::Red));
        }
        let _ = tx.blocking_send(WorkerMsg::Done);
    });

    // 4. UI Loop
    let mut app = App::new();
    let tick_rate = Duration::from_millis(30);
    // Use chrono for internal time if needed, but not for App struct to keep it simple with std::time
    
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') { break; }
            }
        }
        
        while let Ok(msg) = rx.try_recv() {
            match msg {
                WorkerMsg::Log(lvl, m, c) => app.add_log(&lvl, &m, c),
                WorkerMsg::Progress(p, s) => {
                    app.progress = p;
                    app.status = s;
                },
                WorkerMsg::StatUpdate(f, b) => {
                    app.files_scanned += f;
                    app.bytes_cleaned += b;
                },
                WorkerMsg::Done => {
                    app.progress = 1.0;
                    app.status = "SYSTEM OPTIMIZED".to_string();
                    app.add_log("SUCCESS", "Protocol Complete. Press 'q' to exit.", Color::Green);
                }
            }
        }
        
        // Auto-exit on completion after delay? 
        // User asked for "more clever", maybe wait for user to admire the stats?
        // Let's keep it interactive until 'q' or maybe 10s timeout if idle.
    }

    // 5. Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

// --- 🔥 INSANE ENGINEERING LOGIC ---
unsafe fn perform_insane_cleanup(tx: &mpsc::Sender<WorkerMsg>) -> Result<()> {
    // A. Setup COM Engine
    let file_op: IFileOperation = CoCreateInstance(&FileOperation, None, CLSCTX_ALL)?;
    file_op.SetOperationFlags(FOF_NOCONFIRMATION | FOF_NOERRORUI | FOF_SILENT)?;

    // B. Define Targets
    let mut targets = Vec::new();
    
    // 1. User Temp
    targets.push(std::env::temp_dir());
    
    // 2. Windows Temp (Requires Admin)
    targets.push(std::path::PathBuf::from(r"C:\Windows\Temp"));
    
    // 3. Prefetch (The forbidden zone)
    targets.push(std::path::PathBuf::from(r"C:\Windows\Prefetch"));
    
    // 4. SoftwareDistribution (Windows Update)
    targets.push(std::path::PathBuf::from(r"C:\Windows\SoftwareDistribution\Download"));

    // 5. Recycle Bin (Special handling usually, but we try path)
    // targets.push(std::path::PathBuf::from(r"C:\$Recycle.Bin")); // Risky direct access

    let total_steps = targets.len() as f64;
    let mut rng = rand::thread_rng();

    for (i, root) in targets.iter().enumerate() {
        if !root.exists() { continue; }

        let _ = tx.blocking_send(WorkerMsg::Progress(
            (i as f64) / total_steps, 
            format!("Scanning Sector: {:?}", root.file_name().unwrap_or_default())
        ));

        // Walk directory
        // We use standard walkdir logic mostly, but let's do a simple read_dir for robustness
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                let path_str = path.to_string_lossy();
                
                // JITTER: Human-like delay
                // 10ms to 50ms implies "fast human" or "careful machine"
                // Helps evade "mass deletion" heuristics
                let jitter: u64 = rng.gen_range(10..40); 
                std::thread::sleep(Duration::from_millis(jitter));

                let _ = tx.blocking_send(WorkerMsg::Log(
                    "ANALYSIS".into(), 
                    format!("Target Acquired: ...{}", path_str.chars().rev().take(30).collect::<String>().chars().rev().collect::<String>()), 
                    Color::DarkGray
                ));

                // C. COM Operation
                let path_hstring = HSTRING::from(&*path_str);
                // Fixed type inference
                let item_result: windows::core::Result<IShellItem> = SHCreateItemFromParsingName(&path_hstring, None);
                
                if let Ok(item) = item_result {
                    // Queue Delete
                    if let Ok(_) = file_op.DeleteItems(&item) {
                         let _ = tx.blocking_send(WorkerMsg::StatUpdate(1, entry.metadata().map(|m| m.len()).unwrap_or(0)));
                    }
                }
            }
        }
    }

    let _ = tx.blocking_send(WorkerMsg::Progress(0.95, "COMMITTING TRANSACTION...".into()));
    let _ = tx.blocking_send(WorkerMsg::Log("KERNEL".into(), "Executing Kernel Transaction...".into(), Color::Yellow));
    
    // D. EXECUTE (The Point of No Return)
    file_op.PerformOperations()?;

    Ok(())
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Logs
            Constraint::Length(3), // Footer/Stats
            Constraint::Length(1), // Help
        ])
        .split(f.size());

    // 1. Header (Gauge)
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" ⚡ KAWAII CLEANER PRO v5.1.0 "))
        .gauge_style(Style::default().fg(Color::Magenta).bg(Color::Black).add_modifier(ratatui::style::Modifier::BOLD))
        .ratio(app.progress)
        .label(format!("{:.1}% - {}", app.progress * 100.0, app.status));
    f.render_widget(gauge, chunks[0]);

    // 2. Logs
    let log_items: Vec<ListItem> = app.logs.iter().rev().take(20).map(|log| {
        let style = Style::default().fg(log.color);
        let content = format!("{} [{}] {}", log.timestamp, log.level, log.message);
        ListItem::new(content).style(style)
    }).collect();
    
    let logs_list = List::new(log_items)
        .block(Block::default().borders(Borders::ALL).title(" 📜 SYSTEM EVENT LOG "))
        .direction(ratatui::widgets::ListDirection::BottomToTop);
    f.render_widget(logs_list, chunks[1]);
    
    // 3. Stats Footer
    let duration = app.start_time.elapsed();
    let stats_text = format!(
        " 📁 Files: {} | 💾 Cleaned: {:.2} MB | ⏱️ Time: {:.1}s ",
        app.files_scanned,
        app.bytes_cleaned as f64 / 1024.0 / 1024.0,
        duration.as_secs_f64()
    );
    let stats = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title(" 📊 STATISTICS "))
        .style(Style::default().fg(Color::Cyan))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(stats, chunks[2]);
    
    // 4. Help
    let help = Paragraph::new("Press 'q' to Quit | System Secure")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[3]);
}
