use windows::core::PCSTR;
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress, LoadLibraryA};
use std::ffi::c_void;

/// Dynamically resolves a function from a DLL at runtime.
/// This prevents the function from appearing in the Import Address Table (IAT).
pub struct Dynamo;

impl Dynamo {
    pub unsafe fn get_func(module: &str, function: &str) -> Option<*const c_void> {
        // 1. Try to get handle if already loaded
        let mut h_module = GetModuleHandleA(PCSTR(module.as_ptr()));
        
        // 2. If not, LoadLibrary
        if h_module.is_err() {
            h_module = LoadLibraryA(PCSTR(module.as_ptr()));
        }

        if let Ok(handle) = h_module {
            // 3. GetProcAddress
            let func_addr = GetProcAddress(handle, PCSTR(function.as_ptr()));
            return func_addr.map(|f| f as *const c_void);
        }
        
        None
    }
}

// Helper macros could go here for easier calling, 
// but for now we'll just use raw pointer casts in security.rs
