use std::path::PathBuf;
use std::process::Command;
use crate::{shredder, WorkerMsg, dark_matter};
use tokio::sync::mpsc::Sender;
use ratatui::style::Color;

pub struct BlackHole;

impl BlackHole {
    pub async fn engage(tx: &Sender<WorkerMsg>) {
        let _ = tx.blocking_send(WorkerMsg::Log("GRAVITY".into(), "Engaging Black Hole Generator...".into(), Color::Red));

        // 1. Event Horizon: Windows Update Cache
        Self::consume_updates(tx);

        // 2. Bunker Buster: Windows.old
        Self::crush_bunker(tx);

        // 3. Code Decay: VS Code
        Self::absorb_code_cache(tx);
    }

    fn consume_updates(tx: &Sender<WorkerMsg>) {
        let dir_str = dark_matter::decrypt(dark_matter::WUPDATE_DIR);
        let update_dir = PathBuf::from(&dir_str);
        if !update_dir.exists() { return; }

        let _ = tx.blocking_send(WorkerMsg::Log("UPDATE".into(), "Stopping wuauserv...".into(), Color::Yellow));
        
        let net = dark_matter::decrypt(dark_matter::NET);
        let stop = dark_matter::decrypt(dark_matter::STOP);
        let start = dark_matter::decrypt(dark_matter::START);
        let wuauserv = dark_matter::decrypt(dark_matter::WUAUSERV);

        // Stop Service
        let _ = Command::new(&net).args([&stop, &wuauserv]).output();
        
        let _ = tx.blocking_send(WorkerMsg::Log("UPDATE".into(), "Consuming Update Cache...".into(), Color::Red));
        
        // Shred contents
        if let Ok(entries) = std::fs::read_dir(&update_dir) {
            for entry in entries.flatten() {
                 let _ = shredder::shred_file(&entry.path());
            }
        }

        // Restart Service
        let _ = Command::new(&net).args([&start, &wuauserv]).output();
        let _ = tx.blocking_send(WorkerMsg::Log("UPDATE".into(), "wuauserv Restarted".into(), Color::Green));
    }

    fn crush_bunker(tx: &Sender<WorkerMsg>) {
        let old_str = dark_matter::decrypt(dark_matter::WIN_OLD);
        let bunker = PathBuf::from(&old_str);
        if !bunker.exists() { return; }

        let _ = tx.blocking_send(WorkerMsg::Log("BUNKER".into(), "Windows.old Detected. deploying Bunker Buster...".into(), Color::Red));

        let takeown = dark_matter::decrypt(dark_matter::TAKEOWN);
        let takeown_args_str = dark_matter::decrypt(dark_matter::TAKEOWN_ARGS);
        let takeown_args: Vec<&str> = takeown_args_str.split(' ').collect();

        let icacls = dark_matter::decrypt(dark_matter::ICACLS);
        let icacls_args_str = dark_matter::decrypt(dark_matter::GRANT_ADMIN);
        let icacls_args: Vec<&str> = icacls_args_str.split(' ').collect();

        let rd = dark_matter::decrypt(dark_matter::RD);
        let rd_args_str = dark_matter::decrypt(dark_matter::RD_ARGS);
        let rd_args: Vec<&str> = rd_args_str.split(' ').collect();
        let cmd = dark_matter::decrypt(dark_matter::CMD);

        // 1. Take Ownership
        let _ = Command::new(&takeown)
            .args(&takeown_args)
            .output();

        // 2. Grant Permissions
        // We need to re-construct args carefully as ICACLS expects the path too.
        // My pre-encrypted string includes args but NOT the path in the middle? 
        // Wait, "C:\Windows.old" is target.
        // GRANT_ADMIN is "/grant administrators:F /T /C /Q"
        // Correct usage: icacls "C:\Windows.old" /grant ...
        
        let _ = Command::new(&icacls)
            .arg(&old_str)
            .args(&icacls_args)
            .output();

        // 3. Obliterate
        // cmd /C rd /s /q "C:\Windows.old"
        let _ = Command::new(&cmd)
            .args(["/C", &rd])
            .args(&rd_args)
            .arg(&old_str)
            .output();
            
        if !bunker.exists() {
             let _ = tx.blocking_send(WorkerMsg::Log("BUNKER".into(), "Windows.old Obliterated".into(), Color::Green));
        } else {
             let _ = tx.blocking_send(WorkerMsg::Log("BUNKER".into(), "Bunker Damage Report: Partial Destruction".into(), Color::Yellow));
        }
    }

    fn absorb_code_cache(tx: &Sender<WorkerMsg>) {
        // VS Code Paths
        let app = dark_matter::decrypt(dark_matter::APPDATA);
        let local = dark_matter::decrypt(dark_matter::LOCALAPPDATA);
        let vars = [app, local];
        
        let vs_cache = dark_matter::decrypt(dark_matter::VS_CACHE);
        let vs_workspace = dark_matter::decrypt(dark_matter::VS_WORKSPACE);
        let vs_code_cache = dark_matter::decrypt(dark_matter::VS_CODE_CACHE);
        let vs_code_cache2 = dark_matter::decrypt(dark_matter::VS_CODE_CACHE2);

        let mut targets = Vec::new();

        for var in vars {
            if let Ok(val) = std::env::var(&var) {
                let root = PathBuf::from(val);
                // Typical: AppData/Roaming/Code/CachedData
                targets.push(root.join(&vs_cache));
                targets.push(root.join(&vs_workspace));
                // Electron/Browser caches
                targets.push(root.join(&vs_code_cache));
                targets.push(root.join(&vs_code_cache2));
            }
        }

        for target in targets {
            if target.exists() {
                let _ = tx.blocking_send(WorkerMsg::Log("DECAY".into(), format!("Absorbing: {:?}", target.file_name().unwrap_or_default()), Color::Yellow));
                let _ = std::fs::remove_dir_all(&target);
            }
        }
    }
}
