use crate::{scanner, shredder, security, grim_reaper, registry_hunter, immolation, black_hole};
use tokio::sync::mpsc::Sender;
use ratatui::style::Color;

use crate::WorkerMsg;

pub struct Engine;

impl Engine {
    /// The core loop of the Apex Cleaner
    pub async fn run(tx: Sender<WorkerMsg>) -> anyhow::Result<()> {
        let _ = tx.blocking_send(WorkerMsg::Log("KERNEL".into(), "Apex Engine Initialized".into(), Color::Cyan));

        // 1. Shadow Watcher (Continuous Defense)
        security::start_shadow_watcher();
        let _ = tx.blocking_send(WorkerMsg::Log("SECURITY".into(), "Shadow Watcher Active (Background Monitor)".into(), Color::Magenta));

        // 2. Standard Cleaning Phase (Temp, Prefetch, etc.)
        // We re-use logic from old perform_insane_cleanup here, but simplified for brevity in this example.
        // In a full refactor, we'd move that huge function here. For now, we assume the caller handles the basic scan
        // or we just implement the new Singularity logic here.
        
        // 3. SINGULARITY PHASE (Heuristic Scan)
        let _ = tx.blocking_send(WorkerMsg::Log("SINGULARITY".into(), "Scanning Target Vectors...".into(), Color::Magenta));
        let targets = scanner::find_smart_targets();
        let _ = tx.blocking_send(WorkerMsg::Log("INTELLIGENCE".into(), format!("Tracked {} Targets", targets.len()), Color::Cyan));

        let mut pulse_t = 0.0;
        
        for target_dir in targets {
             let _ = tx.blocking_send(WorkerMsg::Log("HUNTER".into(), format!("Purging: {:?}", target_dir.file_name().unwrap_or_default()), Color::Yellow));
             
             if let Ok(entries) = std::fs::read_dir(&target_dir) {
                 for entry in entries.flatten() {
                     let path = entry.path();
                     if path.is_file() {
                          // ORGANIC PULSE
                          let delay = Engine::organic_pulse(pulse_t);
                          security::smart_sleep(delay as u32);
                          pulse_t += 0.1;

                          // SHRED
                          if shredder::shred_file(&path).is_err() {
                              // If shred fails (Locked file?), call Grim Reaper
                              grim_reaper::schedule_delete(&path);
                               let _ = tx.blocking_send(WorkerMsg::Log("REAPER".into(), format!("Marked for Death (Reboot): {:?}", path.file_name().unwrap()), Color::Red));
                          } else {
                               // Optional: too verbose to log every single file
                               // let _ = tx.blocking_send(WorkerMsg::Log("SHRED".into(), "Obliterated".into(), Color::Red));
                          }
                     }
                 }
             }

        }

        // 12. REGISTRY HUNTER
        let _ = tx.blocking_send(WorkerMsg::Log("INTELLIGENCE".into(), "Scanning Registry (UserAssist)...".into(), Color::Cyan));
        registry_hunter::scan_and_destroy();
        let _ = tx.blocking_send(WorkerMsg::Log("CLEAN".into(), "Registry Traces Obliterated".into(), Color::Green));

        // 13. BLACK HOLE (Deep Clean)
        black_hole::BlackHole::engage(&tx).await;

        // 14. SELF-IMMOLATION
        let _ = tx.blocking_send(WorkerMsg::Log("EVASION".into(), "Initiating Self-Destruct Sequence...".into(), Color::Red));
        immolation::commit_seppuku();
        
        Ok(())
    }

    fn organic_pulse(t: f64) -> u64 {
        let base = 50.0;
        let amplitude = 40.0;
        let frequency = 0.5;
        let delay = base + amplitude * (t * frequency).sin();
        delay as u64
    }
}
