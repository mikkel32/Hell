use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::security;

pub static ATOMIC_HEARTBEAT: AtomicU64 = AtomicU64::new(0);

/// Starts the Ghost Heartbeat monitor in a detached thread.
/// If the main thread stops updating the heartbeat (e.g. paused by debugger),
/// this thread will detect the "Cardiac Arrest" and terminate the process.
pub fn start_monitor() {
    // 1. Initialize Heartbeat
    update_beat();

    // 2. Spawn Ghost Thread
    thread::spawn(|| {
        loop {
            // Check delta
            let last_beat = ATOMIC_HEARTBEAT.load(Ordering::Relaxed);
            let now = current_time_ms();
            
            // If main thread hasn't beaten in 3 seconds (3000ms), we assume pause.
            // Allow some slack for heavy operations (like massive file deletion).
            // But 5 seconds is generous.
            if now > last_beat + 5000 {
                // Cardiac Arrest detected (Debugger Paused?)
                // Immediate termination.
                security::crash_dummy(); 
                std::process::exit(1); 
            }
            
            // Sleep 1 second.
            // We use smart_sleep here too? No, regular sleep is fine for the ghost.
            // The debugger might hook sleep, but if it hooks this sleep, it just delays the check?
            // If the debugger pauses the *whole process*, this thread pauses too.
            // Wait, if the whole process is paused, this thread doesn't run, so it can't detect it?
            // TRUE. 
            // HOWEVER: Some debuggers focus on the Main Thread and let others run (rare, but possible).
            // OR: If the analyst is stepping through the main thread, this thread might still get scheduled time slices?
            // "Ghost Heartbeat" only works if the debugger is stepping/pausing specific threads or if the OS schedules this thread while main is blocked on a breakpoint.
            // It is valid anti-step evasion.
            
            thread::sleep(std::time::Duration::from_millis(1000));
        }
    });
}

pub fn beat() {
    update_beat();
}

fn update_beat() {
    let now = current_time_ms();
    ATOMIC_HEARTBEAT.store(now, Ordering::Relaxed);
}

fn current_time_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}
