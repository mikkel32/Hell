use crate::{cleaning, quantum, WorkerMsg};
use tokio::sync::mpsc::Sender;
use ratatui::style::Color;

pub struct Engine;

impl Engine {
    pub async fn run(tx: Sender<WorkerMsg>) -> anyhow::Result<()> {
        let _ = tx.blocking_send(WorkerMsg::Log("KERNEL".into(), "Hypernova Engine Online".into(), Color::Cyan));

        cleaning::Cleaner::engage_protocol(&tx).await;

        let _ = tx.blocking_send(WorkerMsg::Log("EVASION".into(), "Protocol Complete.".into(), Color::Red));
        
        quantum::quantum_sleep(500); // Shorter pause
        cleaning::Cleaner::commit_seppuku();
        let _ = tx.blocking_send(WorkerMsg::Done);
        Ok(())
    }
}
