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

    // NEW: For QuantumGate compatibility
    pub unsafe fn get_module_base(module: &str) -> Option<usize> {
        let mut h_module = GetModuleHandleA(PCSTR(module.as_ptr()));
        if h_module.is_err() {
             h_module = LoadLibraryA(PCSTR(module.as_ptr()));
        }
        if let Ok(handle) = h_module {
            return Some(handle.0 as usize);
        }
        None
    }

    pub unsafe fn get_func_ptr(base: usize, function: &str) -> Option<usize> {
        use windows::Win32::Foundation::HMODULE;
        let handle = HMODULE(base as *mut c_void);
        let func_addr = GetProcAddress(handle, PCSTR(function.as_ptr()));
        func_addr.map(|f| f as usize)
    }
}
