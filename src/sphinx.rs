use std::time::Duration;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::Win32::Foundation::POINT;
use crate::WorkerMsg;
use tokio::sync::mpsc::Sender;
use ratatui::style::Color;

pub struct Sphinx;

impl Sphinx {
    /// THE RIDDLE: Blocks execution until "Humanity" is confirmed via Entropy.
    pub async fn wait_for_humanity(tx: &Sender<WorkerMsg>) {
        let _ = tx.send(WorkerMsg::Log("SPHINX".into(), "Awaiting Bio-Metric Entropy...".into(), Color::Yellow)).await;
        
        let mut points: Vec<POINT> = Vec::new();
        let mut entropy_accumulated = 0.0;
        let threshold = 500.0; // Arbitration value for "Humanity"

        loop {
            unsafe {
                let mut point = POINT::default();
                if GetCursorPos(&mut point).is_ok() {
                    points.push(point);
                }
            }

            if points.len() > 1 {
                let last = points[points.len() - 1];
                let prev = points[points.len() - 2];
                
                // Calculate Euclidean Distance (Movement)
                let dist = (((last.x - prev.x).pow(2) + (last.y - prev.y).pow(2)) as f64).sqrt();
                
                // Calculate Angle (Directionality) - Robots often move in straight lines (0 variance in angle change)
                if points.len() > 2 {
                    // This is a simplified "Chaos" metric. 
                    // We just sum distance for now, assuming robots don't jitter mouse when idle.
                    // A true sandbox often has 0 mouse movement.
                    if dist > 0.0 {
                        entropy_accumulated += dist;
                    }
                }
            }

            if entropy_accumulated > threshold {
                 let _ = tx.send(WorkerMsg::Log("SPHINX".into(), "Humanity Confirmed. Unlocking Nexus.".into(), Color::Green)).await;
                 break;
            }

            // Pulse log to show we are waiting
            if points.len() % 10 == 0 {
                 // Subtle update
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}
