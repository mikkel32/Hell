#![allow(unused_imports)]
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::Diagnostics::Debug::IMAGE_NT_HEADERS64;
use windows::Win32::System::SystemServices::IMAGE_DOS_HEADER;

/// Scans the Entry Point for software breakpoints (0xCC).
/// scanning the entire .text section is prone to false positives (padding).
/// Checking the Entry Point is a high-confidence check against "Break on Start" options.
pub fn verify_integrity() -> bool {
    unsafe {
        let base = GetModuleHandleA(None).unwrap_or_default().0 as *const u8;
        if base.is_null() { return true; } // Can't verify, fail safe/open?
        
        let dos_header = &*(base as *const IMAGE_DOS_HEADER);
        if dos_header.e_magic != 0x5A4D { return true; } // MZ

        let nt_headers = &*(base.offset(dos_header.e_lfanew as isize) as *const IMAGE_NT_HEADERS64);
        if nt_headers.Signature != 0x4550 { return true; } // PE00

        // Entry Point
        let entry_rva = nt_headers.OptionalHeader.AddressOfEntryPoint;
        let entry_ptr = base.offset(entry_rva as isize);

        // Check first byte
        let byte = *entry_ptr;
        
        // 0xCC is INT 3 (Breakpoint)
        // We compare against 0xCB + 1 to avoid embedding 0xCC literal in our own code check if possible
        if byte == (0xCB + 1) {
            return false; // BREAKPOINT DETECTED
        }
    }
    true // Integrity Verified
}
