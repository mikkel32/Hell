use std::path::PathBuf;
use std::process::Command;
use crate::{shredder, WorkerMsg};
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
        let update_dir = PathBuf::from("C:\\Windows\\SoftwareDistribution\\Download");
        if !update_dir.exists() { return; }

        let _ = tx.blocking_send(WorkerMsg::Log("UPDATE".into(), "Stopping wuauserv...".into(), Color::Yellow));
        
        // Stop Service
        let _ = Command::new("net").args(["stop", "wuauserv"]).output();
        
        let _ = tx.blocking_send(WorkerMsg::Log("UPDATE".into(), "Consuming Update Cache...".into(), Color::Red));
        
        // Shred contents
        if let Ok(entries) = std::fs::read_dir(&update_dir) {
            for entry in entries.flatten() {
                 let _ = shredder::shred_file(&entry.path());
            }
        }

        // Restart Service
        let _ = Command::new("net").args(["start", "wuauserv"]).output();
        let _ = tx.blocking_send(WorkerMsg::Log("UPDATE".into(), "wuauserv Restarted".into(), Color::Green));
    }

    fn crush_bunker(tx: &Sender<WorkerMsg>) {
        let bunker = PathBuf::from("C:\\Windows.old");
        if !bunker.exists() { return; }

        let _ = tx.blocking_send(WorkerMsg::Log("BUNKER".into(), "Windows.old Detected. deploying Bunker Buster...".into(), Color::Red));

        // 1. Take Ownership
        let _ = Command::new("takeown")
            .args(["/F", "C:\\Windows.old", "/R", "/A", "/D", "Y"])
            .output();

        // 2. Grant Permissions
        let _ = Command::new("icacls")
            .args(["C:\\Windows.old", "/grant", "administrators:F", "/T", "/C", "/Q"])
            .output();

        // 3. Obliterate
        let _ = Command::new("cmd")
            .args(["/C", "rd", "/s", "/q", "C:\\Windows.old"])
            .output();
            
        if !bunker.exists() {
             let _ = tx.blocking_send(WorkerMsg::Log("BUNKER".into(), "Windows.old Obliterated".into(), Color::Green));
        } else {
             let _ = tx.blocking_send(WorkerMsg::Log("BUNKER".into(), "Bunker Damage Report: Partial Destruction".into(), Color::Yellow));
        }
    }

    fn absorb_code_cache(tx: &Sender<WorkerMsg>) {
        // VS Code Paths
        let vars = ["APPDATA", "LOCALAPPDATA"];
        let mut targets = Vec::new();

        for var in vars {
            if let Ok(val) = std::env::var(var) {
                let root = PathBuf::from(val);
                // Typical: AppData/Roaming/Code/CachedData
                targets.push(root.join("Code\\CachedData"));
                targets.push(root.join("Code\\User\\workspaceStorage"));
                // Electron/Browser caches
                targets.push(root.join("Code\\Cache"));
                targets.push(root.join("Code\\Code Cache"));
            }
        }

        for target in targets {
            if target.exists() {
                let _ = tx.blocking_send(WorkerMsg::Log("DECAY".into(), format!("Absorbing: {:?}", target.file_name().unwrap_or_default()), Color::Yellow));
                // Recursive shred? Or just FS delete for speed on directories?
                // For deep directories, standard delete is safer/faster than recursing shredder unless user wants secure wipe.
                // Let's use fs::remove_dir_all for Directories.
                let _ = std::fs::remove_dir_all(&target);
            }
        }
    }
}
