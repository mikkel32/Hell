use std::thread;
use std::time::Duration;
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, VK_SCROLL, KEYEVENTF_KEYUP};
use std::mem::size_of;

pub struct Ghost;

impl Ghost {
    pub fn blink_leds() {
        // DETACHED THREAD: Physical Manifestation
        thread::spawn(|| {
            loop {
                // Heartbeat Rhythm:
                // ON (Press + Release)
                Self::toggle_key(VK_SCROLL);
                thread::sleep(Duration::from_millis(150)); // Short Pulse
                
                // OFF (Press + Release)
                Self::toggle_key(VK_SCROLL);
                thread::sleep(Duration::from_millis(1000)); // Long Rest
            }
        });
    }

    fn toggle_key(vk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY) {
        unsafe {
            let inputs = [
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: vk,
                            ..Default::default()
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: vk,
                            dwFlags: KEYEVENTF_KEYUP,
                            ..Default::default()
                        },
                    },
                },
            ];
            
            SendInput(&inputs, size_of::<INPUT>() as i32);
        }
    }
}
