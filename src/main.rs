extern crate rodio;

use anyhow::{anyhow,Context, Result};
use clap::{App as ClapApp, AppSettings, Arg, SubCommand};
use rodio::DeviceTrait;
use std::io::BufReader;

static CMD_PLAY: &str = "play";
static CMD_LISTDEV : &str = "listdev"; 
static ARG_FILE: &str = "file";
static ARG_LOOP_SECS: &str = "secs";
static ARG_DEVICE : &str = "dev";

fn main() -> Result<()> {
    let matches = ClapApp::new("N.b3 pyramid-player")
        .subcommand(
            SubCommand::with_name(CMD_LISTDEV)
                .about("List available devices")
        )
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
                    Arg::with_name(ARG_DEVICE)
                        .long("device")
                        .takes_value(true)
                        .help("Device to use"),
                )
                .arg(
                    Arg::with_name(ARG_FILE)
                        .required(true)
                        .takes_value(true)
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
        
        let device = matches
            .value_of(ARG_DEVICE);
            
        cmd_play(file, loop_secs, device)?;
    } else if let Some(_) = matches.subcommand_matches(CMD_LISTDEV){
        cmd_listdev()?; 
    } else { 
        unreachable!()
    }
    Ok(())
}

fn cmd_listdev() -> Result<()> {
    println!("Found devices");
    for device in rodio::output_devices()? {
        println!("- '{}'",device.name()?);
    }
    Ok(())
}

fn cmd_play(file_path: &str, loop_secs: Option<u64>, device_name: Option<&str>) -> Result<()> {
    let device = if let Some(device_name) = device_name {
        rodio::output_devices()?
            .find(|d| d.name().unwrap_or("".to_string()) == device_name ) 
            .ok_or(anyhow!("cannot find output device"))?
    } else {
        rodio::default_output_device().context("opening default output device")?
    };
    println!("Using device {}",device.name()?);
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
