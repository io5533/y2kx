use clap::{Parser};
use y2kx::ToSmfOptions;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// input: Y2KX (y2kx)
    pub input: String,

    /// output: MIDI (mid)
    pub output: String,

    /// PPQN for MIDI
    #[arg(short, long, default_value_t = ToSmfOptions::default().ppqn)]
    pub ppqn: u16,

    /// TEMPO_US for MIDI
    #[arg(short, long, default_value_t = ToSmfOptions::default().tempo_us)]
    pub tempo_us: u32,

    /// Note length for MIDI
    #[arg(short, long, default_value_t = ToSmfOptions::default().note_len)]
    pub note_len: u64,
}