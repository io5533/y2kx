use clap::{Parser, ValueEnum};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ButtonSetting {
    /// Shift: #x1, Z: #x7, X: #x12
    A,
    /// Shift: #x7, Z: #x12, X: #x1
    B,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// input: Y2KX (y2kx)
    pub input: String,

    /// Track ID (1=..255) to play
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(1..=255), default_value_t = 1)]
    pub track: u8,

    /// Button setting
    #[arg(short, long, value_enum, default_value_t = ButtonSetting::A)]
    pub mode: ButtonSetting,
}