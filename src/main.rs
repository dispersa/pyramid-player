extern crate rodio;

use anyhow::{anyhow,Context, Result};
use clap::{App as ClapApp, AppSettings, Arg, SubCommand};
use rodio::DeviceTrait;
use std::{collections::HashMap, io::BufReader};
use sysfs_gpio::{Direction, Pin};

static CMD_PLAY: &str = "play";
static CMD_LISTDEV : &str = "listdev"; 
static ARG_FILE: &str = "file";
static ARG_LOOP_SECS: &str = "secs";
static ARG_DEVICE : &str = "dev";
static ARG_DANCE : &str = "dance";


struct GPIO{
    pins: Vec<Pin>
}
impl GPIO {
    pub fn new() -> Self {
        let mut pins = Vec::new();
        for pin_no in 11..=18 {
            pins.push(Pin::new(pin_no));
        }
        GPIO { pins }
    }
    pub fn init(&self) -> Result<()> {
        for pin in &self.pins {
            if !pin.is_exported() {
                pin.export().map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("Cannot export pin {}: {}",pin.get_pin(), err) ))?;
            }
            if pin.get_direction()? != Direction::Out {
                pin.set_direction(Direction::Out)?;
            }
            if pin.get_value()? != 1 {
                pin.set_value(1)?;
            }
        }
        Ok(())
    }
    pub fn test(&self) -> Result<()> {
        for pin in &self.pins {
            println!("Activating PIN {}",pin.get_pin());
            pin.set_value(0)?;
            std::thread::sleep( std::time::Duration::from_millis(1000) );
            pin.set_value(1)?;
        }
        Ok(())
    }

    pub fn set(&self, pin: usize, on: bool) -> Result<()> {
        println!("SIGNAL: {}={}",pin,on);
        self.pins[pin].set_value(if on {1} else {0})?;
        Ok(())
    }
}

struct Dance{
    gpio : GPIO,
    steps: HashMap<usize, Vec<bool>>
}

impl Dance {
    pub fn new(dance: &str) -> Self {
        let mut elapsed = 0;
        let mut steps = HashMap::new();
        for step in dance.split(';') {
            let mut secs_gipos = step.splitn(2,':');
            let secs = usize::from_str_radix(secs_gipos.next().unwrap(),10).unwrap();
            elapsed += secs;
            let gpios : Vec<bool> = secs_gipos.next().unwrap().split(",").map(|v| v=="1").collect();     
            steps.insert(elapsed, gpios);
        }
       Self{ gpio: GPIO::new(), steps}
    }

    pub fn init(&self) -> Result<()> {
        self.gpio.init()?;
        self.gpio.test()
    }

    pub fn step(&self, elapsed: usize) -> Result<()> {
        if let Some(step) = self.steps.get(&elapsed) {
            for (gpio,on) in step.iter().enumerate() {
                self.gpio.set(gpio,*on)?;
            }
        }
        Ok(())
    }
}

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
                    Arg::with_name(ARG_DANCE)
                        .long("dance")
                        .takes_value(true)
                        .help("seconds:gpio0,gpio1,gipo2,gipo3;seconds..."),
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
        
        let device = matches.value_of(ARG_DEVICE);
        let dance = matches.value_of(ARG_DANCE);
            
        cmd_play(file, loop_secs, device, dance)?;
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

fn cmd_play(file_path: &str, loop_secs: Option<u64>, device_name: Option<&str>, dance: Option<&str>) -> Result<()> {
    let device = if let Some(device_name) = device_name {
        rodio::output_devices()?
            .find(|d| d.name().unwrap_or("".to_string()) == device_name ) 
            .ok_or(anyhow!("cannot find output device"))?
    } else {
        rodio::default_output_device().context("opening default output device")?
    };
    println!("Using device {}",device.name()?);
    let dance = Dance::new(dance.unwrap());
    dance.init()?;
    loop {
        let mut elapsed = 0;
        let sink = rodio::Sink::new(&device);
        let file = std::fs::File::open(file_path).context("cmd_play open file")?;
        sink.append(rodio::Decoder::new(BufReader::new(file)).context("cmd_play decode file")?);
        while !sink.empty() {
            dance.step(elapsed)?;
            std::thread::sleep(std::time::Duration::from_secs(1));
            elapsed += 1;
        }
        sink.sleep_until_end();
        if let Some(secs) = loop_secs {
            std::thread::sleep(std::time::Duration::from_secs(secs));
        } else {
            break;
        }
    }

    Ok(())
}

#[test]
fn test_steps() {
    let dance = Dance::new("20:0,1,0;30:1,1");
    assert_eq!(dance.steps.len(),2);
    assert_eq!(dance.steps[&20], vec![false,true,false]);
    assert_eq!(dance.steps[&50], vec![true,true]);
}
