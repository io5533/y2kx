use std::io;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_DOWN, VK_LEFT, VK_RIGHT,
    VK_SHIFT, VK_UP, VK_X, VK_Z,
    MapVirtualKeyW, KEYEVENTF_SCANCODE, MAPVK_VK_TO_VSC,
};

use super::KeyboardBackend;

pub struct Keyboard;

impl Keyboard {
    // Windows Virtual-Key Code 상수 정의 (u8)
    pub const SHIFT: u8 = VK_SHIFT.0 as u8; // 0x10
    pub const Z: u8 = VK_Z.0 as u8;         // 0x5A
    pub const X: u8 = VK_X.0 as u8;         // 0x58
    pub const LEFT: u8 = VK_LEFT.0 as u8;   // 0x25
    pub const UP: u8 = VK_UP.0 as u8;       // 0x26
    pub const DOWN: u8 = VK_DOWN.0 as u8;   // 0x28
    pub const RIGHT: u8 = VK_RIGHT.0 as u8; // 0x27

    pub fn new() -> io::Result<Self> {
        // Windows는 별도의 디바이스 핸들을 열 필요가 없으므로 바로 생성합니다.
        Ok(Self)
    }

    fn emit(&mut self, key: u8, is_up: bool) -> io::Result<()> {
        let mut flags = KEYBD_EVENT_FLAGS(0);

        // 키를 뗄 때는 KEYEVENTF_KEYUP 플래그 설정
        if is_up {
            flags |= KEYEVENTF_KEYUP;
        }

        // 방향키(LEFT, UP, RIGHT, DOWN)는 Extended Key 플래그가 필요합니다.
        if matches!(key, Self::LEFT | Self::UP | Self::RIGHT | Self::DOWN) {
            flags |= KEYEVENTF_EXTENDEDKEY;
        }

        let scan = unsafe {
            MapVirtualKeyW(key as u32, MAPVK_VK_TO_VSC)
        } as u16;

        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: scan,
                    dwFlags: flags | KEYEVENTF_SCANCODE,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        // SendInput 호출하여 가상 입력 주입
        let sent = unsafe {
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32)
        };

        // 0을 반환하면 입력 주입 실패 (권한 부족 또는 모드 차단)
        if sent == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

impl KeyboardBackend for Keyboard {
    fn key_down(&mut self, key: u8) -> io::Result<()> {
        self.emit(key, false)
    }

    fn key_up(&mut self, key: u8) -> io::Result<()> {
        self.emit(key, true)
    }
}
