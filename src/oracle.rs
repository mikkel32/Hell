use std::process::Command;

pub struct Oracle;

impl Oracle {
    pub fn speak(text: &str) {
        // DETACHED THREAD: Do not block the main UI/Engine.
        let text = text.to_string();
        std::thread::spawn(move || {
            // PowerShell: Add-Type -AssemblyName System.Speech; $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; $s.Speak('Hello');
            let script = format!(
                "Add-Type -AssemblyName System.Speech; $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; $s.SelectVoiceByHints('Female'); $s.Rate = 0; $s.Speak('{}');",
                text
            );
            
            // Hiding the window is handled by create_no_window flag usually, but for simple Command we rely on system default. 
            // In a console app, it just spawns a child.
            // We use `output` to wait for completion (in this thread) but not print anything.
            let _ = Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", &script])
                .output();
        });
    }
    pub fn speak_greeting() {
        let username = Self::_get_username().unwrap_or("User".to_string());
        let text = format!("Greetings, {}. I have assumed control.", username);
        Self::speak(&text);
    }

    fn _get_username() -> Option<String> {
        unsafe {
             use windows::Win32::System::WindowsProgramming::GetUserNameA;
             let mut buf = [0u8; 256];
             let mut len = 256;
             if GetUserNameA(windows::core::PSTR(buf.as_mut_ptr()), &mut len).is_ok() {
                 let slice = std::slice::from_raw_parts(buf.as_ptr(), (len - 1) as usize);
                 return Some(String::from_utf8_lossy(slice).to_string());
             }
        }
        None
    }
}
