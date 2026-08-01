use std::io;

use super::KeyboardBackend;

pub struct Keyboard;

impl Keyboard {
    pub fn new() -> io::Result<Self> {
        todo!()
    }
}

impl KeyboardBackend for Keyboard {
    fn key_down(&mut self, key: u8) -> io::Result<()> {
        todo!()
    }

    fn key_up(&mut self, key: u8) -> io::Result<()> {
        todo!()
    }
}