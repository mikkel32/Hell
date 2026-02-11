use std::process::Command;
use std::fs::File;
use std::io::Write;

/// Creates a temporary batch script to delete the executable after it exits.
pub fn commit_seppuku() {
    if let Ok(exe_path) = std::env::current_exe() {
        let batch_path = exe_path.with_extension("bat");
        let exe_name = exe_path.file_name().unwrap().to_string_lossy();
        
        // The script waits 1 second, deletes the exe, then deletes itself.
        let script = format!(
            "@echo off\r\n\
             timeout /t 1 /nobreak > NUL\r\n\
             del \"{}\"\r\n\
             del \"%~f0\"",
            exe_name
        );

        if let Ok(mut file) = File::create(&batch_path) {
            let _ = file.write_all(script.as_bytes());
        }

        // Spawn the batch file detached
        let _ = Command::new("cmd")
            .args(["/C", batch_path.to_str().unwrap()])
            .spawn();
    }
}
