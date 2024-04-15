use std::collections::HashMap;

use clap::{Parser};
use maplit::hashmap;
use regex::Regex;
use serde_json::{Result, Value};
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

fn get_brightness(device: &str) -> u32 {
    let re = Regex::new(r"([0-9]+)%").unwrap();
    let output = execute(format!("brightnessctl -d {} -m", device).as_str()).unwrap();
    let brightness = re.captures(output.as_str()).unwrap().get(1).unwrap().as_str();
    brightness.parse::<u32>().unwrap()
}

fn send_notification(device: &str) {
    let brightness = get_brightness(device);
    let output = execute("brightnessctl info").unwrap();
    let device = Regex::new(r"Device '([^']*)'").unwrap()
        .captures(&output).unwrap().get(1).unwrap().as_str();
    let angle = ((brightness+2)/5)*5;
    let icon = format!("{}/vol-{}.svg", ICODIR, angle);
    let cmd = format!("notify-send -a brightness -r 91190 -t 800 -i {} '{}{}' '{}'", icon, pad_progress(brightness, 5), get_progress_bar(brightness, 5), device);
    execute(&cmd).unwrap();
}

fn get_device() -> Result<String> {
    let output = execute("hyprctl monitors -j").unwrap();
    let json: Value = serde_json::from_str(&output)?;

    let device = match json {
        Value::Array(arr) => {
            let focused_monitor = arr.into_iter().find(|x| x["focused"] == Value::Bool(true));
            match focused_monitor {
                Some(monitor) => monitor.get("name").unwrap().to_string(),
                None => "eDP-1".to_string()
            }
        },
        _ => "eDP-1".to_string()
    };
    Ok(device)
}

fn main() {
    let args = Args::parse();
    let map: HashMap<&str, &str> = hashmap! {
        "eDP-1" => "intel_backlight",
        "DP-1" => "asus_screenpad"
    };
    let raw_device = get_device().unwrap_or("eDP-1".to_string()).replace("\"", "");
    let device = map.get(raw_device.as_str()).unwrap_or(&"intel_backlight");
    let cmd = if args.increase {
        if get_brightness(device) < 10 {
            "set +1%"
        } else {
            "set +5%"
        }
    } else if args.decrease {
        if get_brightness(device) <= 10 {
            "set 1%-"
        } else {
            "set 5%-"
        }
    } else {
        eprintln!("Please pass -i or -d!");
        ""
    };
    execute(format!("brightnessctl -d {} {}", device, cmd).as_str()).unwrap();
    send_notification(device);
}
