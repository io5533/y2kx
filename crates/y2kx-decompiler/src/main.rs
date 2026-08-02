mod cli;

use cli::Args;
use clap::Parser;
use y2kx::{ToSmfOptions, Y2kxFile};

use std::fs::File;
use std::io::BufWriter;

use midly::io::IoWrap;

fn main() {
    let args: Args = Args::parse();


    let data = std::fs::read(args.input).unwrap();
    let file = Y2kxFile::from_bytes(&data).unwrap();

    let music = y2kx::decompile(&file).unwrap();
    let smf = music.to_smf(ToSmfOptions {
        ppqn: args.ppqn,
        tempo_us: args.tempo_us,
        note_len: args.note_len,
    });



    let file = File::create(args.output).unwrap();
    let writer = BufWriter::new(file);
    let mut writer = IoWrap(writer);

    smf.write(&mut writer).unwrap();
}
