use std::path::Path;
use windows::Win32::Storage::FileSystem::{MoveFileExA, MOVEFILE_DELAY_UNTIL_REBOOT};
use windows::core::PCSTR;
use std::ffi::CString;

/// Schedules a file for deletion at the next system reboot.
/// This is used for files that are currently locked by the OS.
pub fn schedule_delete(path: &Path) {
    if let Some(path_str) = path.to_str() {
        if let Ok(c_path) = CString::new(path_str) {
            unsafe {
                // MOVEFILE_DELAY_UNTIL_REBOOT requires Administrator privileges.
                // We pass NULL as the destination to indicate deletion.
                let _ = MoveFileExA(
                    PCSTR(c_path.as_ptr() as *const u8),
                    PCSTR(std::ptr::null()),
                    MOVEFILE_DELAY_UNTIL_REBOOT,
                );
            }
        }
    }
}
