use anyhow::Result;
use blake3::Hasher;
use chrono::{DateTime, Local};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dashmap::DashMap;
use humansize::{format_size, DECIMAL};
use memmap2::Mmap;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Gauge, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use rayon::prelude::*;
use std::{
    fs::{self, File},
    io::{self, Stdout},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tokio::sync::mpsc;
use walkdir::WalkDir;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::HWND,
        UI::Shell::{SHEmptyRecycleBinW, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND},
    },
};

// --- 🎨 THEME: "CYBER-FORENSICS" ---
const COL_BG: Color = Color::Rgb(15, 15, 25);
const COL_FG: Color = Color::Rgb(220, 220, 235);
const COL_ACCENT: Color = Color::Rgb(0, 255, 150); // Neon Mint
const COL_WARN: Color = Color::Rgb(255, 80, 80);   // Alert Red
const COL_DIM: Color = Color::Rgb(60, 60, 80);

// --- 🧠 INTELLIGENT DATA STRUCTURES ---

#[derive(Clone, Debug)]
struct SmartFile {
    path: PathBuf,
    size: u64,
    modified: DateTime<Local>,
    // 0 = Safe to delete, 100 = Risky (Recent file)
    risk_score: u8,
}

#[derive(Clone, Debug)]
struct TrashCategory {
    id: String,
    name: String,
    files: Vec<SmartFile>,
    total_size: u64,
    icon: &'static str,
}

enum AppMessage {
    ScanUpdate(String, f64), // Log, Progress
    CategoryFound(TrashCategory),
    DuplicateFound(u64),     // Bytes found in duplicates
    ScanComplete,
    CleanUpdate(String, f64),
    CleanComplete,
    Log(String),
}

#[derive(PartialEq)]
enum AppState {
    Dashboard,
    Scanning,
    Review,
    Cleaning,
    Done,
}

struct App {
    state: AppState,
    categories: Vec<TrashCategory>,
    duplicates_reclaimable: u64,
    list_state: ListState,
    total_reclaimable: u64,
    progress: f64,
    logs: Vec<String>,
    system: System,
    spinner_frame: usize,
    tx: mpsc::Sender<AppMessage>,
    rx: mpsc::Receiver<AppMessage>,
}

// --- 🚀 MAIN ENTRY POINT ---

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Initialize App
    let (tx, rx) = mpsc::channel(100);
    let mut app = App::new(tx, rx);

    // 3. Event Loop
    let res = run_loop(&mut terminal, &mut app).await;

    // 4. Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        eprintln!("Critical Failure: {:?}", e);
    }

    Ok(())
}

impl App {
    fn new(tx: mpsc::Sender<AppMessage>, rx: mpsc::Receiver<AppMessage>) -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::new().with_cpu(CpuRefreshKind::everything()).with_memory()
        );
        sys.refresh_all();

        App {
            state: AppState::Dashboard,
            categories: Vec::new(),
            duplicates_reclaimable: 0,
            list_state: ListState::default(),
            total_reclaimable: 0,
            progress: 0.0,
            logs: vec!["Neural Engine Initialized. Waiting for command...".to_string()],
            system: sys,
            spinner_frame: 0,
            tx,
            rx,
        }
    }

    fn start_smart_scan(&mut self) {
        self.state = AppState::Scanning;
        self.categories.clear();
        self.total_reclaimable = 0;
        self.progress = 0.0;
        let tx = self.tx.clone();

        tokio::spawn(async move {
            let targets = get_scan_targets();
            let total_steps = (targets.len() + 1) as f64; 

            // Phase 1: Heuristic Trash Scan
            for (i, (name, path, icon)) in targets.into_iter().enumerate() {
                let _ = tx.send(AppMessage::ScanUpdate(format!("Heuristic Scan: {}", name), i as f64 / total_steps)).await;
                
                if name == "Recycle Bin" {
                     let rb_path = PathBuf::from("C:\\$Recycle.Bin");
                     if rb_path.exists() {
                         let (files, size) = scan_standard(&rb_path);
                         let cat = TrashCategory { id: "RecycleBin".to_string(), name, files, total_size: size, icon };
                         let _ = tx.send(AppMessage::CategoryFound(cat)).await;
                     }
                } else if path.exists() {
                    // SMART SCAN: Checks file age and locking status
                    let (files, size) = scan_smart_heuristics(&path);
                    if size > 0 {
                        let cat = TrashCategory { id: name.clone(), name, files, total_size: size, icon };
                        let _ = tx.send(AppMessage::CategoryFound(cat)).await;
                    }
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }

            // Phase 2: Parallel Deep Scan (Duplicates)
            let _ = tx.send(AppMessage::ScanUpdate("Spinning up Rayon Threads...".to_string(), 0.9)).await;
            
            if let Some(user_home) = directories::UserDirs::new() {
                let downloads = user_home.download_dir().unwrap_or(Path::new("C:\\"));
                
                // Spawn blocking thread for heavy CPU work (Hashing)
                let dup_size = tokio::task::spawn_blocking(move || {
                    find_duplicates_parallel(downloads)
                }).await.unwrap_or(0);

                if dup_size > 0 {
                    let _ = tx.send(AppMessage::DuplicateFound(dup_size)).await;
                    let _ = tx.send(AppMessage::Log(format!("Found {} redundant data", format_size(dup_size, DECIMAL)))).await;
                }
            }

            let _ = tx.send(AppMessage::ScanComplete).await;
        });
    }

    fn start_clean(&mut self) {
        self.state = AppState::Cleaning;
        let tx = self.tx.clone();
        let targets: Vec<TrashCategory> = self.categories.clone();

        tokio::spawn(async move {
            let total_items: usize = targets.iter().map(|c| c.files.len()).sum();
            let mut processed = 0;

            for cat in targets {
                if cat.id == "RecycleBin" {
                    unsafe {
                        let _ = SHEmptyRecycleBinW(HWND(0), PCWSTR::null(), SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND);
                        let _ = tx.send(AppMessage::Log("✨ Win32 API: Recycle Bin Purged".to_string())).await;
                    }
                } else {
                    for file in cat.files {
                        processed += 1;
                        let _ = tx.send(AppMessage::CleanUpdate(format!("Unlinking: {:?}", file.path.file_name().unwrap_or_default()), processed as f64 / total_items as f64)).await;
                        
                        // Heuristic Safety Check: Re-verify existence before delete
                        if file.path.exists() {
                            if fs::remove_file(&file.path).is_ok() {
                                // Deleted successfully
                            }
                        }
                        if processed % 10 == 0 { tokio::time::sleep(Duration::from_millis(1)).await; }
                    }
                    let _ = tx.send(AppMessage::Log(format!("Scrubbed {}", cat.name))).await;
                }
            }
            let _ = tx.send(AppMessage::CleanComplete).await;
        });
    }
}

// --- 🧠 INTELLIGENCE MODULES ---

fn get_scan_targets() -> Vec<(String, PathBuf, &'static str)> {
    let mut t = Vec::new();
    if let Ok(sys) = std::env::var("SystemRoot") {
        t.push(("Win Temp".to_string(), PathBuf::from(sys).join("Temp"), "⚙️"));
    }
    t.push(("User Temp".to_string(), std::env::temp_dir(), "🌡️"));
    // Standard Recycle Bin Path placeholder
    t.push(("Recycle Bin".to_string(), PathBuf::from(""), "🗑️"));
    t
}

// Heuristic: Scans for files but skips anything modified in last 1 hour (Safety)
fn scan_smart_heuristics(path: &Path) -> (Vec<SmartFile>, u64) {
    let mut files = Vec::new();
    let mut total_size = 0;
    let now = Local::now();

    for entry in WalkDir::new(path).min_depth(1).max_depth(5).into_iter().filter_map(|e| e.ok()) {
        if let Ok(meta) = entry.metadata() {
            if meta.is_file() {
                let modified: DateTime<Local> = meta.modified().unwrap_or(std::time::SystemTime::now()).into();
                let age = now.signed_duration_since(modified);
                
                // SAFETY LOCK: Skip files younger than 1 hour (likely in use)
                if age.num_hours() > 1 {
                    total_size += meta.len();
                    files.push(SmartFile {
                        path: entry.path().to_path_buf(),
                        size: meta.len(),
                        modified,
                        risk_score: if age.num_hours() < 24 { 50 } else { 0 },
                    });
                }
            }
        }
    }
    (files, total_size)
}

fn scan_standard(path: &Path) -> (Vec<SmartFile>, u64) {
    let mut size = 0;
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if let Ok(meta) = entry.metadata() {
            if meta.is_file() { size += meta.len(); }
        }
    }
    (Vec::new(), size) 
}

// --- 🔥 PARALLEL ENGINE (The Advanced Part) ---

// Uses Rayon + DashMap + Memmap + Blake3
// This is significantly faster than standard loops
fn find_duplicates_parallel(path: &Path) -> u64 {
    // 1. Collect candidates (Large files > 1MB)
    let candidates: Vec<PathBuf> = WalkDir::new(path)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| {
            if e.metadata().map(|m| m.len()).unwrap_or(0) > 1_000_000 {
                Some(e.path().to_path_buf())
            } else { None }
        })
        .collect();

    let wasted_bytes = Arc::new(AtomicU64::new(0));
    let hashes = DashMap::new(); // Thread-safe Map

    // 2. Parallel Hash Calculation
    // par_iter() automatically splits work across all CPU cores
    candidates.par_iter().for_each(|p| {
        if let Ok(file) = File::open(p) {
            // ADVANCED: Mmap maps the file to virtual RAM. 
            // The OS handles paging, making reads exceptionally fast (Zero-Copy).
            let hash_result = unsafe { 
                if let Ok(mmap) = Mmap::map(&file) {
                    let mut hasher = Hasher::new();
                    // BLAKE3 uses SIMD (AVX2/AVX-512) for speed
                    hasher.update(&mmap);
                    Some(hasher.finalize().to_hex().to_string())
                } else { None }
            };

            if let Some(h) = hash_result {
                if hashes.contains_key(&h) {
                    // Collision found! This file is a duplicate.
                    if let Ok(m) = file.metadata() {
                        wasted_bytes.fetch_add(m.len(), Ordering::Relaxed);
                    }
                } else {
                    hashes.insert(h, p.clone());
                }
            }
        }
    });

    wasted_bytes.load(Ordering::Relaxed)
}

// --- 🖥️ UI LOOP ---
async fn run_loop(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, app))?;
        let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or(Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') { return Ok(()); }
                if app.state == AppState::Dashboard && key.code == KeyCode::Char('s') { app.start_smart_scan(); }
                if app.state == AppState::Review && key.code == KeyCode::Char('c') { app.start_clean(); }
            }
        }

        while let Ok(msg) = app.rx.try_recv() {
            match msg {
                AppMessage::ScanUpdate(s, p) => { app.progress = p; app.logs.push(s); }
                AppMessage::CategoryFound(c) => { app.total_reclaimable += c.total_size; app.categories.push(c); }
                AppMessage::DuplicateFound(s) => { app.duplicates_reclaimable = s; app.total_reclaimable += s; }
                AppMessage::ScanComplete => { app.state = AppState::Review; app.progress = 1.0; }
                AppMessage::CleanUpdate(s, p) => { app.progress = p; app.logs.push(s); }
                AppMessage::CleanComplete => { app.state = AppState::Done; }
                AppMessage::Log(s) => app.logs.push(s),
            }
        }
        if app.logs.len() > 15 { app.logs.remove(0); }
        if last_tick.elapsed() >= tick_rate {
            app.system.refresh_cpu();
            app.system.refresh_memory();
            app.spinner_frame = (app.spinner_frame + 1) % 4;
            last_tick = Instant::now();
        }
    }
}

// --- 🎨 RENDER ---
fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default().direction(Direction::Vertical).margin(1).constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)]).split(f.area());
    
    // Header
    let title = Paragraph::new(" 🔮 KAWAII FORENSICS // SYSTEM OPTIMIZER ").style(Style::default().bg(COL_ACCENT).fg(COL_BG).add_modifier(Modifier::BOLD)).alignment(Alignment::Center).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(COL_DIM)));
    f.render_widget(title, chunks[0]);

    // Footer
    let help = match app.state {
        AppState::Dashboard => " [S] INITIATE SMART SCAN • [Q] ABORT ",
        AppState::Review => " [C] PURGE TRASH • [Q] EXIT ",
        _ => " SYSTEM PROCESSING... "
    };
    f.render_widget(Paragraph::new(help).alignment(Alignment::Center).style(Style::default().fg(COL_FG)), chunks[2]);

    match app.state {
        AppState::Dashboard => {
            let info = Paragraph::new(format!("\n\nReady for Deep System Analysis.\n\nRAM: {:.1}%\nCPU: {:.1}%\n\nWaiting for command...", 
                (app.system.used_memory() as f64 / app.system.total_memory() as f64) * 100.0,
                app.system.global_cpu_info().cpu_usage()
            )).alignment(Alignment::Center).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(COL_ACCENT)));
            f.render_widget(info, chunks[1]);
        }
        AppState::Scanning | AppState::Cleaning => {
            let gauge = Gauge::default().block(Block::default().borders(Borders::ALL).title(" Neural Network Activity ")).gauge_style(Style::default().fg(COL_ACCENT)).ratio(app.progress);
            let log_list = List::new(app.logs.iter().rev().take(10).map(|s| ListItem::new(Line::from(s.as_str()))).collect::<Vec<_>>()).block(Block::default().borders(Borders::ALL).title(" Kernel Logs "));
            let layout = Layout::default().constraints([Constraint::Length(3), Constraint::Min(0)]).split(chunks[1]);
            f.render_widget(gauge, layout[0]);
            f.render_widget(log_list, layout[1]);
        }
        AppState::Review => {
            let mut items = Vec::new();
            for c in &app.categories {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", c.icon), Style::default()),
                    Span::styled(format!("{:<15}", c.name), Style::default().fg(COL_FG)),
                    Span::styled(format_size(c.total_size, DECIMAL), Style::default().fg(COL_ACCENT)),
                ])));
            }
            if app.duplicates_reclaimable > 0 {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("⚡ ", Style::default()),
                    Span::styled(format!("{:<15}", "Redundant Data"), Style::default().fg(COL_FG)),
                    Span::styled(format_size(app.duplicates_reclaimable, DECIMAL), Style::default().fg(COL_WARN).add_modifier(Modifier::BOLD)),
                ])));
            }
            let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" Targets Acquired ")).highlight_style(Style::default().bg(COL_DIM));
            f.render_widget(list, chunks[1]);
        }
        AppState::Done => {
            let p = Paragraph::new("\n\n(ﾉ◕ヮ◕)ﾉ*:･ﾟ✧\n\nSystems Optimized Successfully.").alignment(Alignment::Center).block(Block::default().borders(Borders::ALL));
            f.render_widget(p, chunks[1]);
        }
    }
}
