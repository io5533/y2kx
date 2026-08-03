use midly::{
    MetaMessage,
    Smf,
    Timing,
    Track,
    TrackEventKind,
};

const DEFAULT_TEMPO_US_PER_QUARTER: u32 = 500_000; // 120 BPM
const MS_PER_SEC: u64 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TempoChange {
    pub tick: u64,
    pub time_ms: u64,
    pub us_per_quarter: u32,
}

impl TempoChange {
    pub fn new(tick: u64, time_ms: u64, us_per_quarter: u32) -> Self {
        Self {
            tick,
            time_ms,
            us_per_quarter,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TempoMap {
    changes: Vec<TempoChange>,
}

impl TempoMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_smf(smf: &Smf) -> Result<Self, &'static str> {
        let ppqn = get_ppqn(smf)?;
        let tempo_track = smf
            .tracks
            .first()
            .ok_or("Track is not found in MIDI file")?;

        Ok(Self {
            changes: read_tempo_changes(tempo_track, ppqn),
        })
    }

    pub fn tick_to_ms(
        &self,
        tick: u64,
        ppqn: u64,
        cursor: &mut usize,
    ) -> u64 {
        while *cursor + 1 < self.changes.len()
            && self.changes[*cursor + 1].tick <= tick
        {
            *cursor += 1;
        }

        let change = &self.changes[*cursor];

        change.time_ms
            + (tick - change.tick)
                * change.us_per_quarter as u64
                / ppqn
                / MS_PER_SEC
    }

    pub fn changes(&self) -> &[TempoChange] {
        &self.changes
    }
}

// -----------------------------------------------------
// Internal helpers
// -----------------------------------------------------

fn get_ppqn(smf: &Smf) -> Result<u64, &'static str> {
    match smf.header.timing {
        Timing::Metrical(t) => Ok(t.as_int() as u64),
        _ => Err("SMPTE timing is not supported."),
    }
}

fn read_tempo_changes(
    track: &Track,
    ppqn: u64,
) -> Vec<TempoChange> {
    let mut changes = Vec::with_capacity(16);

    let mut current_tick = 0u64;
    let mut last_tempo_tick = 0u64;
    let mut current_time_ms = 0u64;
    let mut current_tempo = DEFAULT_TEMPO_US_PER_QUARTER;

    changes.push(TempoChange::new(
        0,
        0,
        current_tempo,
    ));

    for event in track {
        current_tick += event.delta.as_int() as u64;

        if let TrackEventKind::Meta(MetaMessage::Tempo(new_tempo)) = event.kind {
            let delta_tick = current_tick - last_tempo_tick;

            current_time_ms +=
                delta_tick * current_tempo as u64
                / ppqn
                / MS_PER_SEC;

            current_tempo = new_tempo.as_int();
            last_tempo_tick = current_tick;

            changes.push(TempoChange::new(
                current_tick,
                current_time_ms,
                current_tempo,
            ));
        }
    }

    changes
}