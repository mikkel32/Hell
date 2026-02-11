use std::ffi::CString;
use windows::core::PCSTR;
use windows::Win32::System::Registry::{RegOpenKeyExA, RegSetValueExA, HKEY, HKEY_CURRENT_USER, KEY_WRITE, REG_SZ, RegCloseKey};

pub struct Persistence;

impl Persistence {
    pub fn install() {
        if crate::cleaning::SAFE_MODE {
            // SIMULATION
            // In a real TUI we'd log this, but this is a helper.
            return;
        }

        unsafe {
            let subkey = CString::new("Software\\Microsoft\\Windows\\CurrentVersion\\Run").unwrap();
            let mut h_key = HKEY::default();
            
            // Open Key
            if RegOpenKeyExA(
                HKEY_CURRENT_USER,
                PCSTR(subkey.as_ptr() as *const u8),
                0,
                KEY_WRITE,
                &mut h_key
            ).is_ok() {
                // Get Exe Path
                if let Ok(path) = std::env::current_exe() {
                    if let Some(path_str) = path.to_str() {
                         let value_name = CString::new("KawaiiCleaner").unwrap();
                         let value_data = CString::new(path_str).unwrap();
                         let data_bytes = value_data.as_bytes_with_nul();
                         
                         // Set Value
                         let _ = RegSetValueExA(
                             h_key,
                             PCSTR(value_name.as_ptr() as *const u8),
                             0,
                             REG_SZ,
                             Some(data_bytes),
                         );
                    }
                }
                let _ = RegCloseKey(h_key);
            }
        }
    }
}
