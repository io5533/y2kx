use crate::midi::music;
use crate::file;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowKey {
    Up = 4,
    Down = 0,
    Left = 2,
    Right = 5,
}

impl ArrowKey {
    pub fn to_y2kx(&self) -> file::Action {
        match self {
            Self::Up => file::Action::Up,
            Self::Down => file::Action::Down,
            Self::Left => file::Action::Left,
            Self::Right => file::Action::Right,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Y2kxNote {
    pub time_ms: u64,
    pub u1: bool,
    pub u7: bool,
    pub u12: bool,
    pub key: ArrowKey,
}


impl Y2kxNote {
    pub const NOTES: &'static [&'static [Self]] = &[
            &[ Self { time_ms: 0, u1: false, u7: false, u12: false, key: ArrowKey::Down } ], //C4
            &[ Self { time_ms: 0, u1: true, u7: false,u12: false, key: ArrowKey::Down } ], //C4S
            &[ Self { time_ms: 0, u1: false, u7: false,u12: false, key: ArrowKey::Left } ], //D4
            &[ Self { time_ms: 0, u1: true, u7: false,u12: false, key: ArrowKey::Left } ], //D4S
            &[ Self { time_ms: 0, u1: false, u7: false,u12: false, key: ArrowKey::Up } ], //E4
            &[ Self { time_ms: 0, u1: false, u7: false,u12: false, key: ArrowKey::Right },
                Self { time_ms: 0, u1: true, u7: false,u12: false, key: ArrowKey::Up } ], //F4
            &[ Self { time_ms: 0, u1: true, u7: false,u12: false, key: ArrowKey::Right } ], //F4S
            &[ Self { time_ms: 0, u1: false, u7: true,u12: false, key: ArrowKey::Down } ], //G4
            &[ Self { time_ms: 0, u1: true, u7: true,u12: false, key: ArrowKey::Down } ], //G4S
            &[ Self { time_ms: 0, u1: false, u7: true,u12: false, key: ArrowKey::Left } ], //A4
            &[ Self { time_ms: 0, u1: true, u7: true,u12: false, key: ArrowKey::Left } ], //A4SF$
            &[ Self { time_ms: 0, u1: false, u7: true,u12: false, key: ArrowKey::Up } ], //B4
            &[ Self { time_ms: 0, u1: false, u7: false,u12: true, key: ArrowKey::Down },
                Self { time_ms: 0, u1: false, u7: true,u12: false, key: ArrowKey::Right },
                Self { time_ms: 0, u1: true, u7: true,u12: false, key: ArrowKey::Up } ], //C5
            &[ Self { time_ms: 0, u1: true, u7: false,u12: true, key: ArrowKey::Down },
                Self { time_ms: 0, u1: true, u7: true,u12: false, key: ArrowKey::Right } ], //C5S
            &[ Self { time_ms: 0, u1: false, u7: false,u12: true, key: ArrowKey::Left } ], //D5
            &[ Self { time_ms: 0, u1: true, u7: false,u12: true, key: ArrowKey::Left } ], //D5S
            &[ Self { time_ms: 0, u1: false, u7: false,u12: true, key: ArrowKey::Up } ], //E5
            &[ Self { time_ms: 0, u1: false, u7: false,u12: true, key: ArrowKey::Right },
                Self { time_ms: 0, u1: true, u7: false,u12: true, key: ArrowKey::Up } ], //F5
            &[ Self { time_ms: 0, u1: true, u7: false,u12: true, key: ArrowKey::Right } ], //F5S
            &[ Self { time_ms: 0, u1: false, u7: true,u12: true, key: ArrowKey::Down } ], //G5
            &[ Self { time_ms: 0, u1: true, u7: true,u12: true, key: ArrowKey::Down } ], //G5S
            &[ Self { time_ms: 0, u1: false, u7: true,u12: true, key: ArrowKey::Left } ], //A5
            &[ Self { time_ms: 0, u1: true, u7: true,u12: true, key: ArrowKey::Left } ], //A5S
            &[ Self { time_ms: 0, u1: false, u7: true,u12: true, key: ArrowKey::Up } ], //B5
            &[ Self { time_ms: 0, u1: false, u7: true,u12: true, key: ArrowKey::Right },
                Self { time_ms: 0, u1: true, u7: true,u12: true, key: ArrowKey::Up } ], //C6
            &[ Self { time_ms: 0, u1: true, u7: true,u12: true, key: ArrowKey::Right } ], //x
    ];
    pub fn new() -> Self {
        Y2kxNote { time_ms: 0, u1: false, u7: false, u12: false, key: ArrowKey::Down }
    }
    pub fn from_note(note: music::Note) -> Result<Vec<Self>, &'static str> {
        if  60 <= note.pitch && note.pitch <= 84 {
            let mut out = Self::NOTES[note.pitch as usize - 60].to_vec();
            for y2kx_note in out.iter_mut() {
                y2kx_note.time_ms = note.time_ms;
            }
            Ok(out.clone())
        } else {
            Err("Invaild pitch for y2kx.")
        }
    }
    pub fn to_note(&self) -> Option<music::Note> {
        let pitch = self.get_pitch();
        if 60 <= pitch && pitch <= 84 {
            Some(music::Note::new(self.time_ms, pitch))
        } else {
            None
        }
    }
    pub fn get_next_best_from_note(&self, next_note: music::Note) -> Result<Self, &'static str> {
        let notes = Y2kxNote::from_note(next_note)?;

        notes.into_iter().max_by_key(|p| self.score(p)).ok_or("Unable to find best note")
    }
    pub fn get_pitch(&self) -> u8 {
        self.pitch_index() + 60
    }
    pub fn pitch_index(&self) -> u8 {
        let mut out = self.key as u8;
        if self.u1 { out += 1; }
        if self.u7 { out += 7; }
        if self.u12 { out += 12; }
        out
    }
    /// bigger = better
    pub fn score(&self, note: &Y2kxNote) -> u8 {
        let mut s: u8 = 0;
        if note.u1 == self.u1 { s += 1; }
        if note.u7 == self.u7 { s += 1; }
        if note.u12 == self.u12 { s += 1; }
        s
    }
}