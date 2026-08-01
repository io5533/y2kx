use std::io;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
pub use linux::Keyboard;

#[cfg(target_os = "windows")]
pub use windows::Keyboard;

#[cfg(target_os = "macos")]
pub use macos::Keyboard;

pub trait KeyboardBackend {
    fn key_down(&mut self, key: u8) -> io::Result<()>;
    fn key_up(&mut self, key: u8) -> io::Result<()>;
}