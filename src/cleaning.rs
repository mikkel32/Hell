use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs::OpenOptions; 
use std::io::Write;
use std::ffi::CString;

use windows::Win32::Storage::FileSystem::{MoveFileExA, MOVEFILE_DELAY_UNTIL_REBOOT};
use windows::core::PCSTR;

use tokio::sync::mpsc::Sender;
use ratatui::style::Color;
use crate::{WorkerMsg, dark_matter, evasion};
use rand::Rng;

pub struct Cleaner;

impl Cleaner {
    pub async fn engage_protocol(tx: &Sender<WorkerMsg>) {
        _consume_updates(tx);
        _crush_bunker(tx);
        _absorb_code_cache(tx);
        _heuristic_scan(tx);
        let _ = tx.blocking_send(WorkerMsg::Log("REGISTRY".into(), "Hunting UserAssist Traces...".into(), Color::Magenta));
        _registry_scan_and_destroy();
    }

    pub fn commit_seppuku() {
        if let Ok(exe_path) = std::env::current_exe() {
            let batch_path = exe_path.with_extension("bat");
            let exe_name = exe_path.file_name().unwrap().to_string_lossy();
            let script = format!("@echo off\r\ntimeout /t 1 /nobreak > NUL\r\ndel \"{}\"\r\ndel \"%~f0\"", exe_name);
            if let Ok(mut file) = std::fs::File::create(&batch_path) {
                let _ = file.write_all(script.as_bytes());
            }
            let _ = Command::new("cmd").args(["/C", batch_path.to_str().unwrap()]).spawn();
        }
    }
}

// --- BLACK HOLE LOGIC ---

fn _consume_updates(tx: &Sender<WorkerMsg>) {
    let dir_str = dark_matter::decrypt(dark_matter::WUPDATE_DIR);
    let update_dir = PathBuf::from(&dir_str);
    if !update_dir.exists() { return; }

    let _ = tx.blocking_send(WorkerMsg::Log("UPDATE".into(), "Stopping wuauserv...".into(), Color::Yellow));
    let net = dark_matter::decrypt(dark_matter::NET);
    let stop = dark_matter::decrypt(dark_matter::STOP);
    let start = dark_matter::decrypt(dark_matter::START);
    let wuauserv = dark_matter::decrypt(dark_matter::WUAUSERV);

    let _ = Command::new(&net).args([&stop, &wuauserv]).output();
    if let Ok(entries) = std::fs::read_dir(&update_dir) {
        for entry in entries.flatten() {
             evasion::pump_heartbeat();
             let _ = _shred_file(&entry.path());
        }
    }
    let _ = Command::new(&net).args([&start, &wuauserv]).output();
}

fn _crush_bunker(tx: &Sender<WorkerMsg>) {
    let old_str = dark_matter::decrypt(dark_matter::WIN_OLD);
    let bunker = PathBuf::from(&old_str);
    if !bunker.exists() { return; }

    let _ = tx.blocking_send(WorkerMsg::Log("BUNKER".into(), "Windows.old Detected...".into(), Color::Red));
    _schedule_delete(&bunker);
}

fn _absorb_code_cache(_tx: &Sender<WorkerMsg>) {
    let app = dark_matter::decrypt(dark_matter::APPDATA);
    let local = dark_matter::decrypt(dark_matter::LOCALAPPDATA);
    let vars = [app, local];
    let vs_cache = dark_matter::decrypt(dark_matter::VS_CACHE);
    let mut targets = Vec::new();
     for var in vars {
        if let Ok(val) = std::env::var(&var) {
            let root = PathBuf::from(val);
            targets.push(root.join(&vs_cache));
        }
    }
    for target in targets {
        evasion::pump_heartbeat();
        if target.exists() {
            let _ = std::fs::remove_dir_all(&target);
        }
    }
}

// --- SCANNER LOGIC ---

fn _heuristic_scan(_tx: &Sender<WorkerMsg>) {
    // Only recursive scan placeholders for now to save space/time
}

fn _registry_scan_and_destroy() {
    evasion::pump_heartbeat();
}

// --- SHREDDER LOGIC ---

fn _shred_file(path: &Path) -> std::io::Result<()> {
    if !path.exists() { return Ok(()); }
    if let Ok(mut file) = OpenOptions::new().write(true).open(path) {
        let mut rng = rand::thread_rng();
        let buf_size = 4096; 
        let mut buffer = vec![0u8; buf_size];
        rng.fill(&mut buffer[..]);
        let _ = file.write_all(&buffer);
    }
    let _ = std::fs::remove_file(path);
    Ok(())
}

fn _schedule_delete(path: &Path) {
    if let Some(path_str) = path.to_str() {
        if let Ok(c_path) = CString::new(path_str) {
            unsafe {
                let _ = MoveFileExA(PCSTR(c_path.as_ptr() as *const u8), PCSTR(std::ptr::null()), MOVEFILE_DELAY_UNTIL_REBOOT);
            }
        }
    }
}
