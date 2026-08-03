mod cli;

use cli::Args;
use clap::Parser;
use y2kx::Y2kxFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();

    println!("[INFO] Reading and parsing Y2KX file. (path: `{}`)", &args.input);

    let data = match std::fs::read(&args.input) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("[ERROR] Could not read the file.");
            eprintln!("[ERROR] This error can be caused by invaild permission or path.");
            Err(error)?
        },
    };
    let file = match Y2kxFile::from_bytes(&data) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("[ERROR] Could not parse the file into Y2KX.");
            eprintln!("[ERROR] This error can be caused by corrupted Y2KX file or y2kx-compiler's BUG.");
            Err(error)?
        },
    };

    println!("Y2KX version {}", file.version());
    println!("title: {}", file.title());
    println!("artist: {}", file.artist());
    println!("description ========\n{}\n====================", file.description());

    let duration: usize = file.delays().iter().map(|&x| x as usize).sum();
    println!("duration in ms: {}", duration);

    let track_names: Vec<String> = file.tracks().iter().map(|track| track.name.clone()).collect();
    println!("tracks[{}]: \"{:?}\"", file.track_count(), track_names);

    Ok(())
}
