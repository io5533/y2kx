mod backend;
mod cli;
mod player;

use clap::Parser;
use y2kx::Y2kxFile;
use player::Player;

use cli::Args;

fn main() {
    let args: Args = Args::parse();


    let data = std::fs::read(args.input).unwrap();
    let file = Y2kxFile::from_bytes(&data).unwrap();

    let mut player = Player::new().unwrap();
    player.play(&file, args.track, args.mode).unwrap();
}
