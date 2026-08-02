mod cli;

use clap::Parser;

use y2kx::{CompileOptions, NoteOrder, Music};
use cli::Args;

use midly::Smf;

fn main() {
    let args: Args = Args::parse();

    let options = CompileOptions {
        arpeggio: args.arpeggio,
        arpeggio_order: NoteOrder::from(args.order),
        click_len: args.click,
        preparation_time: args.prepare,
        merge_tracks: args.merge_tracks,
        del_nullchar: args.del_null
    };


    let data = std::fs::read(args.input).unwrap();
    let smf = Smf::parse(&data).unwrap();

    let music = Music::from_smf_range(&smf, 60..=84).unwrap();

    let y2kx = y2kx::compile_with(&music, options).unwrap();

    println!("[INFO] {} track(s):", y2kx.track_count());
    for track in y2kx.tracks() {
        println!("[INFO] - {}",track.name);
    }

    std::fs::write(args.output, y2kx.to_bytes()).unwrap();

    println!("[INFO] y2kx is compiled and saved.");
}
