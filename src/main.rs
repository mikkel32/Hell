use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use humansize::{format_size, DECIMAL};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, BorderType, Gauge, List, ListItem, ListState, Paragraph, Wrap,
    },
    Frame, Terminal,
};
use std::{
    fs,
    io::{self, Stdout},
    path::{Path, PathBuf},
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

// --- 🎨 THEME CONFIGURATION (Cyber-Pastel) ---
const COL_BG: Color = Color::Rgb(30, 30, 46);      // Dark Base
const COL_FG: Color = Color::Rgb(205, 214, 244);   // Text White
const COL_PINK: Color = Color::Rgb(245, 194, 231); // Kawaii Pink
const COL_CYAN: Color = Color::Rgb(137, 220, 235); // Cyber Cyan
const COL_PURP: Color = Color::Rgb(203, 166, 247); // Border Purple
const COL_RED: Color = Color::Rgb(243, 139, 168);  // Danger/Delete

// --- 🧠 DATA STRUCTURES ---

#[derive(Clone, Debug)]
struct TrashCategory {
    id: String,
    name: String,
    path: PathBuf,
    size: u64,
    count: usize,
    icon: &'static str,
    selected: bool,
}

#[derive(PartialEq)]
enum AppState {
    Dashboard,
    Scanning,
    Review,
    Cleaning,
    Done,
}

// Messages sent from Background Threads -> UI Thread
enum AppMessage {
    ScanUpdate(String, f64), // Task Name, Progress (0.0 - 1.0)
    CategoryFound(TrashCategory),
    ScanComplete,
    CleanUpdate(String, f64),
    CleanComplete,
    Log(String),
}

struct App {
    state: AppState,
    categories: Vec<TrashCategory>,
    list_state: ListState,
    total_size: u64,
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

    // 2. Initialize App & Channels
    let (tx, rx) = mpsc::channel(100);
    let mut app = App::new(tx, rx);

    // 3. Run Event Loop
    let res = run_loop(&mut terminal, &mut app).await;

    // 4. Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        eprintln!("Error: {:?}", e);
    }

    Ok(())
}

// --- ⚙️ LOGIC CORE ---

impl App {
    fn new(tx: mpsc::Sender<AppMessage>, rx: mpsc::Receiver<AppMessage>) -> Self {
        // Initialize System Monitor
        let mut sys = System::new_with_specifics(
            RefreshKind::new().with_cpu(CpuRefreshKind::everything()).with_memory()
        );
        sys.refresh_all(); // First refresh is usually empty, so we do it once here

        App {
            state: AppState::Dashboard,
            categories: Vec::new(),
            list_state: ListState::default(),
            total_size: 0,
            progress: 0.0,
            logs: vec!["System initialized. Waiting for command... (◕‿◕)".to_string()],
            system: sys,
            spinner_frame: 0,
            tx,
            rx,
        }
    }

    fn start_scan(&mut self) {
        self.state = AppState::Scanning;
        self.categories.clear();
        self.total_size = 0;
        self.progress = 0.0;
        
        let tx = self.tx.clone();
        
        // Spawn Background Task
        tokio::spawn(async move {
            let targets = get_scan_targets();
            let total_targets = targets.len() as f64;

            for (i, (name, path, icon)) in targets.into_iter().enumerate() {
                let _ = tx.send(AppMessage::ScanUpdate(format!("Scanning {}", name), i as f64 / total_targets)).await;
                
                // 1. Filesystem Scan
                if path.exists() && name != "Recycle Bin" {
                    let (size, count) = scan_dir_stats(&path);
                    if size > 0 {
                        let cat = TrashCategory {
                            id: name.clone(),
                            name,
                            path,
                            size,
                            count,
                            icon,
                            selected: true,
                        };
                        let _ = tx.send(AppMessage::CategoryFound(cat)).await;
                    }
                }
                
                // 2. Recycle Bin Special Case
                if name == "Recycle Bin" {
                    // Check if $Recycle.Bin exists on C: (Hidden system folder)
                    let rb_path = PathBuf::from("C:\\$Recycle.Bin");
                    if rb_path.exists() {
                        let (size, count) = scan_dir_stats(&rb_path);
                         // Even if size is 0, we might want to try emptying it via API
                        let cat = TrashCategory {
                            id: "RecycleBin".to_string(),
                            name: "Recycle Bin".to_string(),
                            path: rb_path,
                            size, // Approx size
                            count, 
                            icon: "🗑️",
                            selected: true,
                        };
                        let _ = tx.send(AppMessage::CategoryFound(cat)).await;
                    }
                }

                // Artificial delay for UI "Hacker" feel
                tokio::time::sleep(Duration::from_millis(150)).await;
            }

            let _ = tx.send(AppMessage::ScanComplete).await;
        });
    }

    fn start_clean(&mut self) {
        self.state = AppState::Cleaning;
        let tx = self.tx.clone();
        let targets: Vec<TrashCategory> = self.categories.iter().filter(|c| c.selected).cloned().collect();

        tokio::spawn(async move {
            let total = targets.len() as f64;
            
            for (i, cat) in targets.iter().enumerate() {
                let _ = tx.send(AppMessage::CleanUpdate(format!("Scrubbing {}", cat.name), i as f64 / total)).await;

                if cat.id == "RecycleBin" {
                    unsafe {
                        // NATIVE WINDOWS API CALL
                        // SHEmptyRecycleBinW(hwnd, root_path, flags)
                        let _ = SHEmptyRecycleBinW(
                            HWND(0), 
                            PCWSTR::null(), 
                            SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND
                        );
                        let _ = tx.send(AppMessage::Log("✨ Native API: Recycle Bin Emptied".to_string())).await;
                    }
                } else {
                    // Standard File Removal
                    match clean_dir_contents(&cat.path) {
                        Ok(bytes) => {
                             let _ = tx.send(AppMessage::Log(format!("Deleted {} from {}", format_size(bytes, DECIMAL), cat.name))).await;
                        }
                        Err(e) => {
                             let _ = tx.send(AppMessage::Log(format!("Error in {}: {}", cat.name, e))).await;
                        }
                    }
                }
                tokio::time::sleep(Duration::from_millis(300)).await;
            }
            let _ = tx.send(AppMessage::CleanComplete).await;
        });
    }
}

// --- 🛠️ HELPERS ---

fn get_scan_targets() -> Vec<(String, PathBuf, &'static str)> {
    let mut targets = Vec::new();
    
    // 1. User Temp
    if let Some(base) = directories::BaseDirs::new() {
        targets.push(("User Cache".to_string(), base.cache_dir().to_path_buf(), "👤"));
    }
    targets.push(("User Temp".to_string(), std::env::temp_dir(), "🌡️"));

    // 2. Windows System Paths (Best accessed as Admin)
    if let Ok(sysroot) = std::env::var("SystemRoot") {
        let root = PathBuf::from(sysroot);
        targets.push(("Windows Temp".to_string(), root.join("Temp"), "⚙️"));
        targets.push(("Prefetch".to_string(), root.join("Prefetch"), "🚀"));
        targets.push(("Windows Updates".to_string(), root.join("SoftwareDistribution").join("Download"), "📦"));
        targets.push(("Crash Dumps".to_string(), root.join("Minidump"), "💥"));
    }
    
    // 3. Old Windows Installations
    targets.push(("Old Windows".to_string(), PathBuf::from("C:\\Windows.old"), "💾"));
    
    // 4. Recycle Bin marker
    targets.push(("Recycle Bin".to_string(), PathBuf::from(""), "🗑️"));

    targets
}

fn scan_dir_stats(path: &Path) -> (u64, usize) {
    let mut size = 0;
    let mut count = 0;
    // WalkDir is recursive and fast
    for entry in WalkDir::new(path).min_depth(1).max_depth(5).into_iter().filter_map(|e| e.ok()) {
        if let Ok(meta) = entry.metadata() {
            if meta.is_file() {
                size += meta.len();
                count += 1;
            }
        }
    }
    (size, count)
}

fn clean_dir_contents(path: &Path) -> Result<u64> {
    let mut reclaimed = 0;
    for entry in WalkDir::new(path).min_depth(1).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.is_file() {
            if let Ok(meta) = p.metadata() {
                // We ignore errors (e.g., locked files) and continue
                if fs::remove_file(p).is_ok() {
                    reclaimed += meta.len();
                }
            }
        }
    }
    Ok(reclaimed)
}

// --- 🖥️ UI LOOP ---

async fn run_loop(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // 1. Handle Input (Non-blocking)
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.state {
                    AppState::Dashboard => {
                        if key.code == KeyCode::Char('s') { app.start_scan(); }
                        if key.code == KeyCode::Char('q') { return Ok(()); }
                    }
                    AppState::Review => {
                        match key.code {
                            KeyCode::Char('c') => app.start_clean(),
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Down => {
                                let i = match app.list_state.selected() {
                                    Some(i) => if i >= app.categories.len() - 1 { 0 } else { i + 1 },
                                    None => 0,
                                };
                                app.list_state.select(Some(i));
                            },
                            KeyCode::Up => {
                                let i = match app.list_state.selected() {
                                    Some(i) => if i == 0 { app.categories.len() - 1 } else { i - 1 },
                                    None => 0,
                                };
                                app.list_state.select(Some(i));
                            },
                            _ => {}
                        }
                    }
                    AppState::Done => {
                        if key.code == KeyCode::Char('q') { return Ok(()); }
                    }
                    _ => {}
                }
            }
        }

        // 2. Handle Background Messages
        while let Ok(msg) = app.rx.try_recv() {
            match msg {
                AppMessage::ScanUpdate(task, prog) => {
                    app.logs.push(task);
                    app.progress = prog;
                }
                AppMessage::CategoryFound(cat) => {
                    app.total_size += cat.size;
                    app.categories.push(cat);
                }
                AppMessage::ScanComplete => {
                    app.state = AppState::Review;
                    if !app.categories.is_empty() { app.list_state.select(Some(0)); }
                    app.logs.push(format!("Scan complete. Found {}", format_size(app.total_size, DECIMAL)));
                }
                AppMessage::CleanUpdate(task, prog) => {
                    app.logs.push(task);
                    app.progress = prog;
                }
                AppMessage::CleanComplete => {
                    app.state = AppState::Done;
                    app.categories.clear();
                    app.total_size = 0;
                    app.logs.push("Cleanup operation successful.".to_string());
                }
                AppMessage::Log(s) => app.logs.push(s),
            }
        }
        
        // Trim logs
        if app.logs.len() > 20 { app.logs.remove(0); }

        // 3. Tick (Update Animations & System Stats)
        if last_tick.elapsed() >= tick_rate {
            app.system.refresh_cpu();
            app.system.refresh_memory();
            app.spinner_frame = (app.spinner_frame + 1) % 4;
            last_tick = Instant::now();
        }
    }
}

// --- 🎨 RENDERING ENGINE ---

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Stats
            Constraint::Min(0),    // Main
            Constraint::Length(3), // Footer
        ].as_ref())
        .split(f.area());

    // 1. HEADER
    let title = Paragraph::new(Span::styled(" 🌸 KAWAII CLEANER // WIN11 PRO 🌸 ", Style::default().fg(COL_BG).bg(COL_PINK).add_modifier(Modifier::BOLD)))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(COL_PURP)));
    f.render_widget(title, chunks[0]);

    // 2. DASHBOARD (CPU/RAM)
    let sys_chunks = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(50), Constraint::Percentage(50)]).split(chunks[1]);
    
    let cpu_usage = app.system.global_cpu_info().cpu_usage();
    let ram_usage = app.system.used_memory() as f64 / app.system.total_memory() as f64;
    
    let cpu_gauge = Gauge::default()
        .block(Block::default().title(" CPU Load ").borders(Borders::ALL).border_style(Style::default().fg(COL_CYAN)))
        .gauge_style(Style::default().fg(COL_PINK))
        .ratio((cpu_usage as f64 / 100.0).clamp(0.0, 1.0))
        .label(format!("{:.1}%", cpu_usage));
    
    let ram_gauge = Gauge::default()
        .block(Block::default().title(" RAM Usage ").borders(Borders::ALL).border_style(Style::default().fg(COL_CYAN)))
        .gauge_style(Style::default().fg(COL_PURP))
        .ratio(ram_usage)
        .label(format!("{:.1}%", ram_usage * 100.0));

    f.render_widget(cpu_gauge, sys_chunks[0]);
    f.render_widget(ram_gauge, sys_chunks[1]);

    // 3. MAIN CONTENT
    match app.state {
        AppState::Dashboard => draw_dashboard(f, app, chunks[2]),
        AppState::Scanning | AppState::Cleaning => draw_scanning(f, app, chunks[2]),
        AppState::Review => draw_review(f, app, chunks[2]),
        AppState::Done => draw_done(f, app, chunks[2]),
    }

    // 4. FOOTER
    let spinner = ["⠋", "⠙", "⠹", "⠸"];
    let spin = if app.state == AppState::Scanning || app.state == AppState::Cleaning { spinner[app.spinner_frame] } else { "•" };
    let log_msg = app.logs.last().map(|s| s.as_str()).unwrap_or("");

    let footer = Paragraph::new(format!("{} {}", spin, log_msg))
        .style(Style::default().fg(COL_FG))
        .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(COL_PURP)));
    f.render_widget(footer, chunks[3]);
}

fn draw_dashboard(f: &mut Frame, _app: &App, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from(Span::styled("Ready to Optimize System", Style::default().fg(COL_CYAN))),
        Line::from(""),
        Line::from("Targets: Temp, Prefetch, Updates, Recycle Bin"),
        Line::from(""),
        Line::from(Span::styled("Press [s] to Start Scan", Style::default().fg(COL_PINK).add_modifier(Modifier::BOLD))),
    ];
    let p = Paragraph::new(text).alignment(Alignment::Center).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(p, area);
}

fn draw_scanning(f: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::default().constraints([Constraint::Length(3), Constraint::Min(0)]).margin(2).split(area);
    let gauge = Gauge::default()
        .block(Block::default().title(" Progress ").borders(Borders::ALL).border_style(Style::default().fg(COL_PINK)))
        .gauge_style(Style::default().fg(COL_CYAN).bg(COL_BG))
        .ratio(app.progress);
    f.render_widget(gauge, layout[0]);

    let logs: Vec<ListItem> = app.logs.iter().rev().take(12).map(|s| ListItem::new(Line::from(s.as_str()))).collect();
    let log_list = List::new(logs).block(Block::default().borders(Borders::ALL).title(" Activity Log "));
    f.render_widget(log_list, layout[1]);
}

fn draw_review(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app.categories.iter().map(|c| {
        let size_str = if c.size == 0 && c.id == "RecycleBin" { "Unknown".to_string() } else { format_size(c.size, DECIMAL) };
        ListItem::new(Line::from(vec![
            Span::styled(format!("{} ", c.icon), Style::default()),
            Span::styled(format!("{:<20}", c.name), Style::default().fg(COL_FG)),
            Span::styled(size_str, Style::default().fg(COL_PINK)),
        ]))
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Junk Found ").border_type(BorderType::Rounded))
        .highlight_style(Style::default().bg(COL_PURP).fg(COL_BG).add_modifier(Modifier::BOLD));
    
    f.render_stateful_widget(list, area, &mut app.list_state);

    let help_area = Rect { x: area.x + 2, y: area.y + area.height - 4, width: area.width - 4, height: 3 };
    let help = Paragraph::new("[c] CLEAN ALL • [q] QUIT").alignment(Alignment::Center).style(Style::default().fg(COL_RED).bg(COL_BG));
    f.render_widget(help, help_area);
}

fn draw_done(f: &mut Frame, _app: &App, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from(Span::styled("(ﾉ◕ヮ◕)ﾉ*:･ﾟ✧", Style::default().fg(COL_PINK))),
        Line::from(""),
        Line::from("System Cleaned Successfully!"),
        Line::from(""),
        Line::from("Press [q] to exit."),
    ];
    let p = Paragraph::new(text).alignment(Alignment::Center).block(Block::default().borders(Borders::ALL));
    f.render_widget(p, area);
}
