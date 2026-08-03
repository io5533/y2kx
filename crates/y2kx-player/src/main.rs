mod backend;
mod cli;
mod player;

use clap::Parser;
use y2kx::Y2kxFile;
use player::Player;

use cli::Args;

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

    println!("[INFO] Preparing the player.");

    let mut player = match Player::new() {
        Ok(data) => data,
        Err(error) => {
            eprintln!("[ERROR] Could not init `Player`.");
            eprintln!("[ERROR] This error can be caused by invaild OS permission or y2kx-player's BUG.");
            Err(error)?
        },
    };

    println!("[INFO] Start playing the Y2KX file's track ID, `{}`.", args.track);
    println!("[INFO] Title: {}", file.title());
    println!("[INFO] Artist: {}", file.artist());

    match player.play(&file, args.track, args.mode) {
        Err(error) => {
            eprintln!("[ERROR] An error occurred while playing.");
            eprintln!("[ERROR] This error can be caused by invaild track ID, other OS error or y2kx-player's BUG.");
            Err(error)?
        },
        _ => {},
    };

    Ok(())
}
