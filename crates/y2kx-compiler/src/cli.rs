use clap::{Parser, ValueEnum};
use y2kx::{CompileOptions, NoteOrder};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CliNoteOrder {
    LowToHigh,
    HighToLow,
    Random,
}

impl From<NoteOrder> for CliNoteOrder {
    fn from(value: NoteOrder) -> Self {
        match value {
            NoteOrder::LowToHigh => CliNoteOrder::LowToHigh,
            NoteOrder::HighToLow => CliNoteOrder::HighToLow,
            NoteOrder::Random => CliNoteOrder::Random,
        }
    }
}

impl From<CliNoteOrder> for NoteOrder {
    fn from(value: CliNoteOrder) -> Self {
        match value {
            CliNoteOrder::LowToHigh => Self::LowToHigh,
            CliNoteOrder::HighToLow => Self::HighToLow,
            CliNoteOrder::Random => Self::Random,
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// input: MIDI (mid)
    pub input: String,

    /// output: Y2KX (y2kx)
    pub output: String,

    /// Time in milliseconds to wait before starting playback.
    #[arg(short, long, default_value_t = CompileOptions::default().preparation_time)]
    pub prepare: u16,

    /// Interval in milliseconds between notes in the same chord.
    #[arg(short, long, default_value_t = CompileOptions::default().arpeggio)]
    pub arpeggio: u16,

    /// Order of notes in each chord before applying the arpeggio.
    #[arg(short, long, value_enum, default_value_t = CliNoteOrder::from(CompileOptions::default().arpeggio_order))]
    pub order: CliNoteOrder,

    /// Key press duration in milliseconds.
    #[arg(short, long, default_value_t = CompileOptions::default().click_len)]
    pub click: u16,

    /// Merge all tracks
    #[arg(short, long, default_value_t = false)]
    pub merge_tracks: bool,
}