mod cli;

use clap::Parser;

use y2kx::{CompileOptions, NoteOrder, Music};
use cli::Args;

use midly::Smf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();

    let options = CompileOptions {
        arpeggio: args.arpeggio,
        arpeggio_order: NoteOrder::from(args.order),
        click_len: args.click,
        preparation_time: args.prepare,
        merge_tracks: args.merge_tracks,
    };

    println!("[INFO] Reading and parsing MIDI file(SMF). (path: `{}`)", &args.input);

    let data = match std::fs::read(&args.input) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("[ERROR] Could not read the file.");
            eprintln!("[ERROR] This error can be caused by invaild permission or path.");
            Err(error)?
        },
    };
    let smf = match Smf::parse(&data) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("[ERROR] Could not parse the file into MIDI(SMF).");
            eprintln!("[ERROR] This error can be caused by corrupted MIDI(SMF) file.");
            Err(error)?
        },
    };

    let mut music = match Music::from_smf_range(&smf, 60..=84) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("[ERROR] Could not prepare MIDI(SMF).");
            eprintln!("[ERROR] This error can be caused by corrupted MIDI(SMF) or using SMPTE timing which is not supported.");
            Err(error)?
        },
    };
    music.apply_playback_speed(args.speed);

    let y2kx = match y2kx::compile_with(&music, options) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("[ERROR] Could not compile MIDI(SMF) into Y2KX.");
            eprintln!("[ERROR] This error can be caused by trying to add TOO MANY tracks (over 255).");
            Err(error)?
        },
    };

    println!("[INFO] the Y2KX file is compiled with options: {:?}", options);
    println!("[INFO] {} track(s):", y2kx.track_count());
    for track in y2kx.tracks() {
        println!("[INFO] - \"{}\"", track.name);
    }

     match std::fs::write(args.output, y2kx.to_bytes()) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("[ERROR] Could not write the file.");
            eprintln!("[ERROR] This error can be caused by invaild permission or path.");
            Err(error)?
        },
    };

    println!("[INFO] the file was saved!");

    Ok(())
}
