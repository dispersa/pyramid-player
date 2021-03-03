extern crate rodio;

use anyhow::{Context, Result};
use clap::{App as ClapApp, AppSettings, Arg, SubCommand};
use std::io::BufReader;

static CMD_PLAY: &str = "play";
static ARG_FILE: &str = "file";
static ARG_LOOP_SECS: &str = "secs";

fn main() -> Result<()> {
    let matches = ClapApp::new("N.b3 pyramid-player")
        .subcommand(
            SubCommand::with_name(CMD_PLAY)
                .about("Play mp3 file")
                .arg(
                    Arg::with_name(ARG_LOOP_SECS)
                        .long("loop")
                        .takes_value(true)
                        .help("Play'n play again, adding delay in seconds"),
                )
                .arg(
                    Arg::with_name(ARG_FILE)
                        .required(true)
                        .takes_value(true)
                        .index(1),
                ),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    if let Some(matches) = matches.subcommand_matches(CMD_PLAY) {
        let file = matches.value_of(ARG_FILE).unwrap();
        let loop_secs = matches
            .value_of(ARG_LOOP_SECS)
            .map(|v| u64::from_str_radix(v, 10))
            .transpose()
            .context("seconds should be a number")?;
        cmd_play(file, loop_secs)?;
    } else {
        unreachable!()
    }
    Ok(())
}

fn cmd_play(file_path: &str, loop_secs: Option<u64>) -> Result<()> {
    let device = rodio::default_output_device().context("opening default output device")?;
    loop {
        let sink = rodio::Sink::new(&device);
        let file = std::fs::File::open(file_path).context("cmd_play open file")?;
        sink.append(rodio::Decoder::new(BufReader::new(file)).context("cmd_play decode file")?);
        sink.sleep_until_end();
        if let Some(secs) = loop_secs {
            std::thread::sleep(std::time::Duration::from_secs(secs));
        } else {
            break;
        }
    }

    Ok(())
}
