use windows::Win32::System::Registry::{RegOpenKeyExA, RegEnumKeyExA, RegEnumValueA, RegCloseKey, RegDeleteValueA, HKEY_CURRENT_USER, KEY_ALL_ACCESS, HKEY};
use windows::core::PCSTR;
use std::ffi::CString;

/// ROT13 decoding for UserAssist keys
fn rot13(s: &str) -> String {
    s.chars().map(|c| {
        match c {
            'a'..='m' | 'A'..='M' => ((c as u8) + 13) as char,
            'n'..='z' | 'N'..='Z' => ((c as u8) - 13) as char,
            _ => c,
        }
    }).collect()
}

pub fn scan_and_destroy() {
    unsafe {
        let subkey = CString::new("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\UserAssist").unwrap();
        let mut h_key = HKEY::default();
        
        // Open UserAssist Key
        if RegOpenKeyExA(HKEY_CURRENT_USER, PCSTR(subkey.as_ptr() as *const u8), 0, KEY_ALL_ACCESS, &mut h_key).is_ok() {
            
            // Enumerate Subkeys (GUIDs)
            let mut index = 0;
            loop {
                let mut name_buf = [0u8; 256];
                let mut name_len = 256;
                
                if RegEnumKeyExA(h_key, index, windows::core::PSTR(name_buf.as_mut_ptr()), &mut name_len, None, windows::core::PSTR::null(), None, None).is_err() {
                    break;
                }
                index += 1;
                
                let guid_name = std::str::from_utf8(&name_buf[..name_len as usize]).unwrap_or_default();
                let count_path = format!("{}\\{}\\Count", subkey.to_str().unwrap(), guid_name);
                let c_count_path = CString::new(count_path).unwrap();
                let mut h_count_key = HKEY::default();

                // Open "Count" Subkey
                if RegOpenKeyExA(HKEY_CURRENT_USER, PCSTR(c_count_path.as_ptr() as *const u8), 0, KEY_ALL_ACCESS, &mut h_count_key).is_ok() {
                    let mut val_index = 0;
                    loop {
                        let mut val_name_buf = [0u8; 1024]; // Larger buffer for paths
                        let mut val_name_len = 1024;
                        
                        if RegEnumValueA(h_count_key, val_index, windows::core::PSTR(val_name_buf.as_mut_ptr()), &mut val_name_len, None, None, None, None).is_err() {
                            break;
                        }
                        
                        let val_name = std::str::from_utf8(&val_name_buf[..val_name_len as usize]).unwrap_or_default();
                        let decoded = rot13(val_name);
                        let lower = decoded.to_lowercase();
                        
                        if lower.contains("kawaii") || lower.contains("cleaner") || lower.contains("hell") {
                            // Obliterate
                            let c_val_name = CString::new(val_name).unwrap();
                            let _ = RegDeleteValueA(h_count_key, PCSTR(c_val_name.as_ptr() as *const u8));
                            // Do not increment index if deleted, as indices shift? 
                            // Actually documentation says indices *might* shift. 
                            // Safer to restart or decrement, but for simplicity we assume we catch most.
                        } else {
                            val_index += 1;
                        }
                    }
                    let _ = RegCloseKey(h_count_key);
                }
            }
            let _ = RegCloseKey(h_key);
        }
    }
}
