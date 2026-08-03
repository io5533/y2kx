mod cli;

use cli::Args;
use clap::Parser;
use y2kx::Y2kxFile;

fn main() {
    let args: Args = Args::parse();


    let data = std::fs::read(args.input).unwrap();
    let file = Y2kxFile::from_bytes(&data).unwrap();

    println!("Y2KX version {}", file.version());
    println!("title: {}", file.title());
    println!("artist: {}", file.artist());
    println!("description: {}", file.description());

    let duration: usize = file.delays().iter().map(|&x| x as usize).sum();
    println!("duration in ms: {}", duration);

    let track_names: Vec<String> = file.tracks().iter().map(|track| track.name.clone()) .collect();
    println!("tracks[{}]: \"{:?}\"", file.track_count(), track_names);
}
