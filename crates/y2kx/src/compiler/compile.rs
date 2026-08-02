use crate::ArrowKey;
use crate::file::{self, Action, Command, Y2kxFile};
use crate::midi::music::{self, TrackData};
use super::note::Y2kxNote;

/// Compile options.
#[derive(Debug, Clone, Copy)]
pub struct CompileOptions {
    pub preparation_time: u16,
    pub arpeggio: u16,
    pub arpeggio_order: music::NoteOrder,
    pub click_len: u16,
    pub merge_tracks: bool,
    pub del_nullchar: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            preparation_time: 500,
            arpeggio: 60,
            arpeggio_order: music::NoteOrder::LowToHigh,
            click_len: 50,
            merge_tracks: false,
            del_nullchar: true
        }
    }
}


/// Internal keyframe.
#[derive(Debug, Clone)]
struct TimedKeyframe {
    time_ms: u64,
    commands: Vec<Command>,
}

impl TimedKeyframe {
    fn merge(&mut self, other: Self) {
        debug_assert_eq!(self.time_ms, other.time_ms);
        self.commands.extend(other.commands);
    }
}

pub fn decompile(y2kx: &Y2kxFile) -> Result<music::Music, String> {
    let mut out = music::Music::new();

    let mut tracks: Vec<TrackData> = y2kx.tracks().iter().map(|track| music::TrackData::new(track.name.clone())).collect();

    let mut u1 = vec![false; tracks.len()];
    let mut u7 = vec![false; tracks.len()];
    let mut u12 = vec![false; tracks.len()];
    let mut up = vec![false; tracks.len()];
    let mut down = vec![false; tracks.len()];
    let mut left = vec![false; tracks.len()];
    let mut right = vec![false; tracks.len()];

    let mut current_time = 0_u64;
    for i in 0..y2kx.delays().len() {
        let keyframe: (u32, &[Command]) = y2kx.keyframe(i)?;

        current_time += keyframe.0 as u64;

        for command in keyframe.1 {
            let command = command.into_u16().to_be_bytes();
            let track_id = command[0] as usize;
            let action: Action = Action::try_from(command[1])?;
            
            match action {
                Action::PitchUp1 => u1[track_id-1] = !u1[track_id-1],
                Action::PitchUp7 => u7[track_id-1] = !u7[track_id-1],
                Action::PitchUp12 => u12[track_id-1] = !u12[track_id-1],
                Action::Up => {
                    up[track_id-1] = !up[track_id-1];
                    if up[track_id-1] {
                        if let Some(_note) = tracks[track_id-1].notes.last() {
                            //
                        } else {
                            tracks[track_id-1].notes.push((Y2kxNote { time_ms: current_time, u1: u1[track_id-1], u7: u7[track_id-1], u12: u12[track_id-1], key: ArrowKey::Up }).to_note().ok_or("Invaild note.")?);
                        }
                    }
                },
                Action::Down => {
                    down[track_id-1] = !down[track_id-1];
                    if down[track_id-1] {
                        if let Some(_note) = tracks[track_id-1].notes.last() {
                            //
                        } else {
                            tracks[track_id-1].notes.push((Y2kxNote { time_ms: current_time, u1: u1[track_id-1], u7: u7[track_id-1], u12: u12[track_id-1], key: ArrowKey::Down }).to_note().ok_or("Invaild note.")?);
                        }
                    }
                },
                Action::Left => {
                    left[track_id-1] = !left[track_id-1];
                    if down[track_id-1] {
                        if let Some(_note) = tracks[track_id-1].notes.last() {
                            //
                        } else {
                            tracks[track_id-1].notes.push((Y2kxNote { time_ms: current_time, u1: u1[track_id-1], u7: u7[track_id-1], u12: u12[track_id-1], key: ArrowKey::Left }).to_note().ok_or("Invaild note.")?);
                        }
                    }
                },
                Action::Right => {
                    right[track_id-1] = !right[track_id-1];
                    if right[track_id-1] {
                        if let Some(_note) = tracks[track_id-1].notes.last() {
                            //
                        } else {
                            tracks[track_id-1].notes.push((Y2kxNote { time_ms: current_time, u1: u1[track_id-1], u7: u7[track_id-1], u12: u12[track_id-1], key: ArrowKey::Right }).to_note().ok_or("Invaild note.")?);
                        }
                    }
                },
            }
        }

        //let mut out_track = music::TrackData::new(track.name.clone());




        //out.add_track(out_track);
    }

    for track in tracks {
        out.add_track(track);
    }

    Ok(out)
}

/// Compile using default options.
pub fn compile(music: &music::Music) -> Result<Y2kxFile, String> {
    compile_with(music, CompileOptions::default())
}

/// Compile using custom options.
pub fn compile_with(
    music: &music::Music,
    options: CompileOptions,
) -> Result<Y2kxFile, String> {
    validate_options(options)?;

    let tracks: Vec<TrackData> = if options.merge_tracks {
        let mut merged = TrackData::new("Merged");

        for track in music.tracks() {
            let mut track = track.clone();
            track.sort();
            merged.merge(&track);
        }

        vec![merged]
    } else {
        let mut tracks = music.tracks().to_vec();

        for track in &mut tracks {
            track.sort();
        }

        tracks
    };
    

    let mut notes = Vec::<Vec<music::Note>>::with_capacity(tracks.len());
    let mut names = Vec::<String>::with_capacity(tracks.len());

    for track in tracks {
        let mut arpeggio_notes = track.notes.clone();
        music::apply_arpeggio(&mut arpeggio_notes, options.arpeggio, options.arpeggio_order);
        notes.push(arpeggio_notes);
        names.push(if options.del_nullchar { track.name.clone().replace('\0',"") } else { track.name.clone() });
    }



    let mut merged_keyframes = Vec::new();

    for (track_index, notes) in notes.iter().enumerate() {
        if notes.is_empty() {
            continue;
        }

        let y2kx_notes = compile_track_notes(notes)?;
        let keyframes = emit_keyframes(
            track_index as u16 + 1,
            y2kx_notes,
            options.preparation_time,
            options.click_len,
        );

        merged_keyframes = merge_keyframes(
            merged_keyframes,
            keyframes,
        );
    }

    build_file(
        merged_keyframes,
        &names,
    )
}

// ------------------------------------------------------------
// Helpers
// ------------------------------------------------------------

fn validate_options(
    options: CompileOptions,
) -> Result<(), String> {
    if options.arpeggio <= options.click_len {
        Err("arpeggio must be bigger then click_len.".into())
    } else {
        Ok(())
    }
}

/// Convert a track's MIDI notes into the optimal Y2kx note sequence.
fn compile_track_notes(
    notes: &[music::Note],
) -> Result<Vec<Y2kxNote>, String> {
    if notes.is_empty() {
        return Ok(Vec::new());
    }

    // First note: choose the first available candidate.
    // (Later에는 전략을 바꿀 수도 있으므로 여기만 수정하면 된다.)
    let mut candidates =
        Y2kxNote::from_note(notes[0]).map_err(String::from)?;

    let mut compiled = Vec::with_capacity(notes.len());
    compiled.push(candidates.swap_remove(0));

    // Remaining notes: always choose the representation that
    // minimizes register changes from the previous note.
    for note in notes.iter().skip(1) {
        let next = compiled
            .last()
            .unwrap()
            .get_next_best_from_note(*note)
            .map_err(String::from)?;

        compiled.push(next);
    }

    Ok(compiled)
}

fn emit_keyframes(
    track_id: u16,
    notes: Vec<Y2kxNote>,
    preparation_time: u16,
    click_len: u16,
) -> Vec<TimedKeyframe> {
    let mut keyframes = Vec::new();

    // Current pitch-up register states.
    let mut u1_now = false;
    let mut u7_now = false;
    let mut u12_now = false;

    // Current note's birth time.
    let mut birth_time = 0_u64;

    // Commands executed when the previous note dies.
    let mut pending_commands = Vec::<Command>::new();

    for note in notes {
        //
        // ---------------- BORN ----------------
        //

        let perform_time = note.time_ms + preparation_time as u64;

        // Prevent the xylophone from closing.
        if perform_time.saturating_sub(birth_time) >= 3000 {
            birth_time = perform_time - 3000;
        }
        

        update_pitch_state(
            &mut pending_commands,
            &mut u1_now,
            note.u1,
            track_id,
            Action::PitchUp1,
        );

        update_pitch_state(
            &mut pending_commands,
            &mut u7_now,
            note.u7,
            track_id,
            Action::PitchUp7,
        );

        update_pitch_state(
            &mut pending_commands,
            &mut u12_now,
            note.u12,
            track_id,
            Action::PitchUp12,
        );


        keyframes.push(TimedKeyframe {
            time_ms: birth_time,
            commands: pending_commands,
        });

        //
        // ---------------- PERFORM ----------------
        //

        keyframes.push(TimedKeyframe {
            time_ms: perform_time,
            commands: vec![Command::new(track_id, note.key.to_y2kx())],
        });

        //
        // ---------------- DEAD ----------------
        //

        pending_commands = vec![Command::new(track_id, note.key.to_y2kx())];
        birth_time = perform_time + click_len as u64;
    }

    // Release the last pressed key.
    keyframes.push(TimedKeyframe {
        time_ms: birth_time,
        commands: pending_commands,
    });

    keyframes
}

fn update_pitch_state(
    commands: &mut Vec<Command>,
    current: &mut bool,
    next: bool,
    track_id: u16,
    action: Action,
) {
    if *current != next {
        commands.push(Command::new(track_id, action));
        *current = next;
    }
}

fn merge_keyframes(
    merged: Vec<TimedKeyframe>,
    keyframes: Vec<TimedKeyframe>,
) -> Vec<TimedKeyframe> {
    if merged.is_empty() {
        return keyframes
    }

    let mut result = Vec::with_capacity(merged.len() + keyframes.len());

    let mut left = merged.iter().peekable();
    let mut right = keyframes.into_iter().peekable();

    while left.peek().is_some() || right.peek().is_some() {
        match (left.peek(), right.peek()) {
            (Some(a), Some(b)) => {
                if a.time_ms < b.time_ms {
                    result.push(left.next().unwrap().clone());
                } else if a.time_ms > b.time_ms {
                    result.push(right.next().unwrap());
                } else {
                    let mut keyframe = left.next().unwrap().clone();
                    keyframe.merge(right.next().unwrap());
                    result.push(keyframe);
                }
            }

            (Some(_), None) => {
                result.extend(left.cloned());
                break;
            }

            (None, Some(_)) => {
                result.extend(right);
                break;
            }

            (None, None) => break,
        }
    }

    result
}

fn build_file(
    keyframes: Vec<TimedKeyframe>,
    track_names: &[String],
) -> Result<Y2kxFile, String> {
    let mut file = Y2kxFile::new(0).unwrap();

    // Instrument 목록 추가
    for name in track_names {
        file.add_track(file::Track::new(name)?).map_err(String::from)?;
    }

    let mut previous_time = 0_u64;

    for keyframe in keyframes {
        let delay = (keyframe.time_ms - previous_time) as u32;

        file.add_keyframe(delay, keyframe.commands)
            .map_err(String::from)?;

        previous_time = keyframe.time_ms;
    }

    Ok(file)
}