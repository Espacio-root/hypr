use serde_json::Value;
use utils::*;
use std::fs::File;
use std::io::{Read, Write};

const CONF_PATH: &str = "/home/espacio/.config/hypr/deviceprefs.conf";
static DEVICES: &[&str] = &["elan1200:00-04f3:3168-touchpad", "elan9009:00-04f3:2c1b", "elan9008:00-04f3:2d55"];

#[derive(Debug)]
enum State {
    Disabled,
    Enabled,
}

fn is_enabled() -> State {
    match File::open("/tmp/hypr/writemode.txt") {
        Ok(mut file) => {
            let mut contents = String::new();
            if let Err(_) = file.read_to_string(&mut contents) {
                return State::Disabled;
            }
            match contents.trim() {
                "1" => State::Enabled,
                _ => State::Disabled,
            }
        }
        Err(_) => State::Disabled,
    }
}

fn empty_conf() {
    let mut file = File::create(CONF_PATH).unwrap();
    file.write_all(b"").unwrap();
}

fn write_to_conf(content: &str) {
    let mut file = File::options().append(true).open(CONF_PATH).unwrap();
    writeln!(&mut file, "{}", content).unwrap();
}

fn write_to_state(state: State) -> Result<(), std::io::Error> {
    let mut file = File::create("/tmp/hypr/writemode.txt").unwrap();
    match state {
        State::Enabled => writeln!(&mut file, "1"),
        State::Disabled => writeln!(&mut file, "0")
    }
}

fn disable_monitor() {
    let content_to_write = "monitor=eDP-1,disabled\nmonitor=DP-5,disabled\nmonitor=DP-1,preferred,0x0,auto";
    write_to_conf(content_to_write);
}

fn set_devices(devices: Vec<&str>, state: State) {
    let enabled = match state {
        State::Enabled => "true",
        State::Disabled => "false"
    };
    let mut content_to_write: String = String::new();
    for device in devices {
        content_to_write.push_str(format!("device {{\n  name = {}\n  enabled = {}\n}}\n", device, enabled).as_str());
    }
    write_to_conf(&content_to_write);
}

fn enable_monitor() {
    let content_to_write = "monitor=eDP-1,1280x720,0x0,1\nmonitor=DP-1,preferred,0x720,auto\nmonitor=DP-6,1920x1080,1280x0,1";
    write_to_conf(content_to_write);
}

fn get_n_monitors() -> u32 {
    let ouput = execute("hyprctl monitors -j").unwrap();
    let json: Value = serde_json::from_str(&ouput).unwrap();
    let monitors: Vec<serde_json::Value> = json.as_array().unwrap().to_vec();
    monitors.len() as u32
}

fn enable() {
    write_to_state(State::Enabled).unwrap();
    empty_conf();
    set_devices(DEVICES.to_vec(), State::Disabled);
    execute("notify-send -a \"Hypr\" \"WriteMode enabled\"").unwrap();
}

fn disable() {
    write_to_state(State::Disabled).unwrap();
    empty_conf();
    set_devices(DEVICES.to_vec(), State::Enabled);
    execute("hyprctl reload").unwrap();
    execute("notify-send -a \"Hypr\" \"WriteMode disabled\"").unwrap();
}

fn main() {
    println!("{:?}", is_enabled());
    match is_enabled() {
        State::Enabled => disable(),
        State::Disabled => enable()
    }
}
