use std::ops::RangeBounds;
use rand::seq::SliceRandom;

use midly::{
    Smf,
    Timing,
    MetaMessage,
    MidiMessage,
    Track,
    TrackEventKind,

    Format,
    Header,
    TrackEvent,
    num::{u4, u7, u15, u24, u28},
};
use crate::CompileOptions;


use super::tempo::TempoMap;


/// Compile options.
#[derive(Debug, Clone, Copy)]
pub struct ToSmfOptions {
    pub ppqn: u16,

    pub tempo_us: u32,
    pub note_len: u64,
}

impl Default for ToSmfOptions {
    fn default() -> Self {
        Self {
            ppqn: 480,
            tempo_us: 500_000, // 120 BPM
            note_len: CompileOptions::default().click_len as u64
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Note {
    pub time_ms: u64,
    pub pitch: u8,
}

impl Note {
    pub fn new(time_ms: u64, pitch: u8) -> Self {
        Self { time_ms, pitch }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TrackData {
    pub name: String,
    pub notes: Vec<Note>,
}

impl TrackData {

    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            notes: Vec::new(),
        }
    }

    pub fn sort(&mut self) {
        self.notes.sort_unstable_by_key(|note| (note.time_ms, note.pitch));
    }

    pub fn merge(&mut self, other: &TrackData) {
        let mut out = Vec::with_capacity(self.notes.len() + other.notes.len());

        let mut i = 0;
        let mut j = 0;

        while i < self.notes.len() && j < other.notes.len() {
            let a = &self.notes[i];
            let b = &other.notes[j];

            match (a.time_ms, a.pitch).cmp(&(b.time_ms, b.pitch)) {
                std::cmp::Ordering::Less => {
                    out.push(a.clone());
                    i += 1;
                }
                std::cmp::Ordering::Greater => {
                    out.push(b.clone());
                    j += 1;
                }
                std::cmp::Ordering::Equal => {
                    out.push(a.clone());
                    i += 1;
                    j += 1;
                }
            }
        }

        out.extend_from_slice(&self.notes[i..]);
        out.extend_from_slice(&other.notes[j..]);

        self.notes = out;
    }

    pub fn apply_playback_speed(&mut self, speed: f64) {
        for note in &mut self.notes {
            note.time_ms = (note.time_ms as f64 / speed) as u64;
        }
    }
}

#[derive(Debug, Default)]
pub struct Music {
    tracks: Vec<TrackData>
}

#[derive(Debug, Clone, Copy)]
pub enum NoteOrder {
    LowToHigh,
    HighToLow,
    Random,
}

impl Music {
    // Constructors
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_smf(
        smf: &Smf,
    ) -> Result<Self, &'static str> {
        Self::from_smf_range(smf, 0..=255)
    }

    pub fn from_smf_range<R>(smf: &Smf, range: R) -> Result<Music, &'static str>
    where
        R: RangeBounds<u8>,
    {
        let ppqn = get_ppqn(smf)?;
        let tempo_map = TempoMap::from_smf(smf)?;

        let mut music = Music::new();

        for track in &smf.tracks {
            music.tracks.push(read_track(
                track,
                &tempo_map,
                ppqn,
                &range,
            ));
        }

        Ok(music)
    }

    // Track management
    pub fn add_track(&mut self, track: TrackData) {
        self.tracks.push(track);
    }

    pub fn remove_other_tracks(&mut self, index: usize) -> Result<(), &'static str> {
        if index >= self.tracks.len() {
            return Err("Track index out of range.");
        }

        let track = self.tracks.swap_remove(index);
        self.tracks = vec![track];
        Ok(())
    }

    // Processing
    pub fn sort(&mut self) {
        for track in &mut self.tracks {
            track.sort();
        }
    }


    pub fn apply_arpeggio(&mut self, ms: u16, order: NoteOrder) {
        for track in &mut self.tracks {
            apply_arpeggio(&mut track.notes, ms, order);
        }
    }

    // Accessors
    pub fn tracks(&self) -> &[TrackData] {
        &self.tracks
    }

    pub fn tracks_mut(&self) -> &[TrackData] {
        &self.tracks
    }

    pub fn get_mut_track(&mut self, index: usize) -> Result<&mut TrackData, &'static str> {
        self.tracks
            .get_mut(index)
            .ok_or("Track index out of range")
    }

    pub fn get_mut_notes(&mut self, index: usize) -> Result<&mut Vec<Note>, &'static str> {
       Ok(&mut self.get_mut_track(index)?.notes)
    }


    pub fn to_smf<'a>(&'a self, options: ToSmfOptions) -> Smf<'a> {
        let mut tracks = Vec::new();

        for (track_index, track) in self.tracks.iter().enumerate() {
            // (tick, priority, event) — priority로 동일 tick에서 NoteOff가 NoteOn보다 먼저 오도록 보장
            let mut events: Vec<(u64, u8, TrackEventKind<'a>)> = Vec::new();

            if track_index == 0 {
                events.push((0, 1, TrackEventKind::Meta(
                    MetaMessage::Tempo(u24::new(options.tempo_us))
                )));
            }

            events.push((0, 1, TrackEventKind::Meta(
                MetaMessage::TrackName(track.name.clone().into_bytes().leak())
            )));

            events.push((0, 1, TrackEventKind::Midi {
                channel: u4::new(0),
                message: MidiMessage::ProgramChange { program: u7::new(0) },
            }));

            // 정렬이 안 되어 있을 수 있으니 로컬로 정렬해서 다음 노트 확인
            let mut notes = track.notes.clone();
            notes.sort_unstable_by_key(|n| (n.time_ms, n.pitch));

            for (i, note) in notes.iter().enumerate() {
                let tick = note.time_ms * options.ppqn as u64 * 1000 / options.tempo_us as u64;

                // 다음 노트 시작 시각을 넘지 않도록 sustain 길이를 clamp
                let end_ms = match notes.get(i + 1) {
                    Some(next) => next.time_ms.min(note.time_ms + options.note_len),
                    None => note.time_ms + options.note_len,
                };
                let end_tick =
                    (end_ms * options.ppqn as u64 * 1000 / options.tempo_us as u64)
                        .max(tick + 1); // 최소 1틱 보장

                events.push((tick, 2, TrackEventKind::Midi {
                    channel: u4::new(0),
                    message: MidiMessage::NoteOn { key: u7::new(note.pitch), vel: u7::new(100) },
                }));

                events.push((end_tick, 0, TrackEventKind::Midi {
                    channel: u4::new(0),
                    message: MidiMessage::NoteOff { key: u7::new(note.pitch), vel: u7::new(0) },
                }));
            }

            // sort_by_key(stable) + (tick, priority) 튜플 키로 tie-break 명확화
            events.sort_by_key(|e| (e.0, e.1));

            let mut last_tick = 0u64;
            let mut midi_track = Vec::new();

            for (tick, _prio, kind) in events {
                let delta = tick - last_tick;
                last_tick = tick;
                midi_track.push(TrackEvent { delta: u28::new(delta as u32), kind });
            }

            midi_track.push(TrackEvent {
                delta: u28::new(0),
                kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
            });

            tracks.push(midi_track);
        }

        Smf {
            header: Header {
                format: if tracks.len() <= 1 { Format::SingleTrack } else { Format::Parallel },
                timing: Timing::Metrical(u15::new(options.ppqn)),
            },
            tracks,
        }
    }
    pub fn apply_playback_speed(&mut self, speed: f64) {
        for track in &mut self.tracks {
            track.apply_playback_speed(speed);
        }
    }
}

// -----------------------------------------------------
// Internal helpers
// -----------------------------------------------------

pub fn get_ppqn(smf: &Smf) -> Result<u64, &'static str> {
    match smf.header.timing {
        Timing::Metrical(t) => Ok(t.as_int() as u64),
        _ => Err("SMPTE timing is not supported."),
    }
}


fn read_track<R>(
    track: &Track,
    tempo_map: &TempoMap,
    ppqn: u64,
    range: &R,
) -> TrackData
where
    R: RangeBounds<u8>,
{
    let mut current_tick = 0u64;
    let mut tempo_cursor = 0usize;

    let mut out = TrackData::new("");

    for event in track {
        current_tick += event.delta.as_int() as u64;

        match event.kind {
            TrackEventKind::Meta(MetaMessage::TrackName(name)) => {
                out.name = String::from_utf8_lossy(name).into_owned();
            }

            TrackEventKind::Midi { message, .. } => {
                if let MidiMessage::NoteOn { key, vel } = message {
                    if vel.as_int() == 0 {
                        continue;
                    }

                    let pitch = key.as_int();

                    if !range.contains(&pitch) {
                        continue;
                    }

                    let time_ms = tempo_map.tick_to_ms(
                        current_tick,
                        ppqn,
                        &mut tempo_cursor,
                    );

                    out.notes.push(Note::new(time_ms, pitch));
                }
            }

            _ => {}
        }
    }

    out
}


fn sort_note_group(notes: &mut [Note], order: NoteOrder) {
    match order {
        NoteOrder::LowToHigh => {
            notes.sort_unstable_by_key(|n| n.pitch);
        }

        NoteOrder::HighToLow => {
            notes.sort_unstable_by(|a, b| b.pitch.cmp(&a.pitch));
        }

        NoteOrder::Random => {
            notes.shuffle(&mut rand::rng());
        }
    }
}

fn spread_notes(notes: &mut [Note], interval_ms: u16) {
    if notes.len() < 2 {
        return;
    }

    let interval = interval_ms as u64;

    let mut prev_time = notes[0].time_ms;

    for note in notes.iter_mut().skip(1) {
        if note.time_ms < prev_time + interval {
            note.time_ms = prev_time + interval;
        }

        prev_time = note.time_ms;
    }
}

// public utility
pub fn apply_arpeggio(
    notes: &mut [Note],
    interval_ms: u16,
    order: NoteOrder,
) {
    if notes.len() < 2 {
        return;
    }

    let mut begin = 0;

    for i in 1..=notes.len() {
        let end_of_group =
            i == notes.len()
            || notes[i].time_ms != notes[begin].time_ms;

        if end_of_group {
            sort_note_group(&mut notes[begin..i], order);
            begin = i;
        }
    }

    spread_notes(notes, interval_ms);
}