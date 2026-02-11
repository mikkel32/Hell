use windows::Win32::Foundation::{HWND, COLORREF};
use windows::Win32::System::Console::GetConsoleWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongA, SetWindowLongA, SetLayeredWindowAttributes, 
    GWL_EXSTYLE, WS_EX_LAYERED, LWA_ALPHA, LAYERED_WINDOW_ATTRIBUTES_FLAGS,
};

pub struct PhaseShift;

impl PhaseShift {
    pub fn set_opacity(alpha: u8) {
        unsafe {
            let handle = GetConsoleWindow();
            if !handle.0.is_null() {
                 // 1. Add WS_EX_LAYERED
                 let current_style = GetWindowLongA(handle, GWL_EXSTYLE);
                 if (current_style as u32 & WS_EX_LAYERED.0) == 0 {
                     SetWindowLongA(handle, GWL_EXSTYLE, current_style | WS_EX_LAYERED.0 as i32);
                 }

                 // 2. Set Alpha
                 let _ = SetLayeredWindowAttributes(handle, COLORREF(0), alpha, LWA_ALPHA);
            }
        }
    }
}
