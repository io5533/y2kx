use std::{
    fs::OpenOptions,
    io,
};

use input_linux::{
    sys::{
        input_event, timeval, uinput_setup, BUS_USB, EV_KEY, EV_SYN, KEY_MAX, SYN_REPORT,
        KEY_LEFTSHIFT, KEY_Z, KEY_X, KEY_LEFT, KEY_UP, KEY_DOWN, KEY_RIGHT,
    },
    EventKind, Key, UInputHandle,
};

use super::KeyboardBackend;

pub struct Keyboard {
    handle: UInputHandle<std::fs::File>,
}

impl Keyboard {
    pub const SHIFT: u8 = KEY_LEFTSHIFT as u8;
    pub const Z: u8 = KEY_Z as u8;
    pub const X: u8 = KEY_X as u8;
    pub const LEFT: u8 = KEY_LEFT as u8;
    pub const UP: u8 = KEY_UP as u8;
    pub const DOWN: u8 = KEY_DOWN as u8;
    pub const RIGHT: u8 = KEY_RIGHT as u8;

    pub fn new() -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/uinput")?;

        let handle = UInputHandle::new(file);

        handle.set_evbit(EventKind::Key)?;
        handle.set_evbit(EventKind::Synchronize)?;

        // 사용 가능한 키 등록 (i32 -> u16 형변환 처리)
        for code in 1..(KEY_MAX as u16) {
            if let Ok(key) = Key::from_code(code) {
                let _ = handle.set_keybit(key);
            }
        }

        // 디바이스 정보 설정
        let mut setup: uinput_setup = unsafe { std::mem::zeroed() };
        setup.id.bustype = BUS_USB as u16;
        setup.id.vendor = 0x1234;
        setup.id.product = 0x5678;
        setup.id.version = 1;

        let name = b"y2kx Virtual Keyboard\0";
        let len = name.len().min(setup.name.len());
        for i in 0..len {
            setup.name[i] = name[i] as libc::c_char;
        }

        // 가상 디바이스 생성
        handle.dev_setup(&setup)?;
        handle.dev_create()?;

        Ok(Self { handle })
    }

    fn emit(&mut self, ty: u16, code: u16, value: i32) -> io::Result<()> {
        let ev = input_event {
            time: timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            type_: ty,
            code,
            value,
        };

        // handle.write는 &[sys::input_event]를 직접 받음
        self.handle.write(&[ev])?;
        Ok(())
    }

    fn sync(&mut self) -> io::Result<()> {
        self.emit(EV_SYN as u16, SYN_REPORT as u16, 0)
    }
}

impl KeyboardBackend for Keyboard {
    fn key_down(&mut self, key: u8) -> io::Result<()> {
        self.emit(EV_KEY as u16, key as u16, 1)?;
        self.sync()
    }

    fn key_up(&mut self, key: u8) -> io::Result<()> {
        self.emit(EV_KEY as u16, key as u16, 0)?;
        self.sync()
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        let _ = self.handle.dev_destroy();
    }
}