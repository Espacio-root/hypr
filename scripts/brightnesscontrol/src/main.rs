use clap::{Parser};
use regex::Regex;
use utils::*;

const ICODIR: &str = "/home/espacio/.config/dunst/icons/vol";

/// Control volume of the different input devices
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Increase Brightness
    #[arg(value_enum, short, long)]
    increase: bool,

    /// Decrease Brightness
    #[arg(value_enum, short, long)]
    decrease: bool,
}

fn get_brightness() -> u32 {
    let re = Regex::new(r"([0-9]+)%").unwrap();
    let output = execute("brightnessctl -m").unwrap();
    let brightness = re.captures(output.as_str()).unwrap().get(1).unwrap().as_str();
    brightness.parse::<u32>().unwrap()
}

fn send_notification() {
    let brightness = get_brightness();
    let output = execute("brightnessctl info").unwrap();
    let device = Regex::new(r"Device '([^']*)'").unwrap()
        .captures(&output).unwrap().get(1).unwrap().as_str();
    let angle = ((brightness+2)/5)*5;
    let icon = format!("{}/vol-{}.svg", ICODIR, angle);
    let cmd = format!("notify-send -a brightness -r 91190 -t 800 -i {} '{}{}' '{}'", icon, pad_progress(brightness, 5), get_progress_bar(brightness, 5), device);
    execute(&cmd).unwrap();
}

fn main() {
    let args = Args::parse();
    let cmd = if args.increase {
        if get_brightness() < 10 {
            "brightnessctl set +1%"
        } else {
            "brightnessctl set +5%"
        }
    } else if args.decrease {
        if get_brightness() <= 10 {
            "brightnessctl set 1%-"
        } else {
            "brightnessctl set 5%-"
        }
    } else {
        eprintln!("Please pass -i or -d!");
        ""
    };
    execute(cmd).unwrap();
    send_notification();
}
