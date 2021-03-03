extern crate rodio;

use std::io::BufReader;
use anyhow::{Context, Result};
use clap::{Arg, App as ClapApp, SubCommand, AppSettings};

static CMD_PLAY : &str = "play";
static ARG_FILE : &str = "file";
static ARG_LOOP : &str = "loop";

fn main() -> Result<()> {
    let matches = ClapApp::new("N.b3 pyramid-player")
        .subcommand(
            SubCommand::with_name(CMD_PLAY)
            .about("Play mp3 file")
            .arg(Arg::with_name(ARG_LOOP).long("loop").help("Play'n play again"))
            .arg(Arg::with_name(ARG_FILE).required(true).takes_value(true).index(1)))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    if let Some(matches) = matches.subcommand_matches(CMD_PLAY) {
        let file = matches.value_of(ARG_FILE).unwrap();
        let repeat = matches.occurrences_of(ARG_LOOP) > 0;
        cmd_play(file, repeat)?;
    } else {
        unreachable!()
    }
    Ok(())
}

fn cmd_play(file_path: &str, repeat: bool) -> Result<()> {
    let device = rodio::default_output_device().context("opening default output device")?;
    loop {
        let sink = rodio::Sink::new(&device);
        let file = std::fs::File::open(file_path).context("cmd_play open file")?;
        sink.append(rodio::Decoder::new(BufReader::new(file)).context("cmd_play decode file")?);
        sink.sleep_until_end();
        if !repeat {
            break;
        }
    }

    Ok(())
}