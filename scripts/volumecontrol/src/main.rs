use clap::{Parser, ValueEnum};
use utils::*;

const ICODIR: &str = "/home/espacio/.config/dunst/icons/vol";

/// Control volume of the different input devices
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input Device
    #[arg(value_enum, short, long)]
    input: Option<Actions>,

    /// Output Device
    #[arg(value_enum, short, long)]
    output: Option<Actions>,

    /// Player Application
    #[arg(value_enum, short, long)]
    player: Option<Actions>,
}

#[derive(Debug, Clone, ValueEnum)]
enum Actions {
    /// Increase
    #[value(name = "i", alias = "inc")]
    Increase,

    /// Decrease
    #[value(name = "d", alias = "dec")]
    Decrease,

    /// Mute
    #[value(name = "m", alias = "mute")]
    Mute,
}

fn action_pamixer(nsink: &str, srce: &str, action: &str, step:u8) {
    let cmd = format!("pamixer '{}' -'{}' {}", srce, action, step);
    execute(&cmd).unwrap();

    let cmd = format!("pamixer {} --get-volume", srce);
    let volume = execute(&cmd).unwrap().parse::<u32>().unwrap();
    notify_vol(volume, nsink);
}

fn action_playerctl(srce: &str, action: &str, step: u8) {
    let cmd = format!("playerctl --player='{}' volume 0.0{}{}", srce, step, action);
    execute(&cmd).unwrap();

    let cmd = format!("playerctl --player={} volume", srce);
    let volume = execute(&cmd).unwrap().parse::<u32>().unwrap();
    notify_vol(volume, srce);
}

fn mute(nsink: &str, ctrl: &str, srce: &str) {
    let cmd = format!("{} '{}' -t", ctrl, srce);
    execute(&cmd).unwrap();
    notify_mute(srce, nsink);
}

fn notify_vol(volume: u32, nsink: &str) {
    let angle = ((volume + 2)/5)*5; 
    let icon = format!("{}/vol-{}.svg", ICODIR, angle);
    // println!("{}, {}", icon, angle);
    let cmd = format!("notify-send -a pulse -r 91190 -t 800 -i {} '{}{}' '{}'", icon, pad_progress(volume, 5), get_progress_bar(volume, 10), nsink);
    execute(&cmd).unwrap();
}

fn notify_mute(srce: &str, nsink: &str) {
    let cmd = format!("pamixer '{}' --get-mute", srce);
    let mute = execute(&cmd).unwrap().parse::<bool>().unwrap();
    let dvce = if srce == "--default-source" {"mic"} else {"speaker"};
    
    let cmd = if mute {
        format!("notify-send -a pulse -r 91190 -t 800 -i {}/muted-{}.svg muted '{}'", ICODIR, dvce, nsink)
    } else {
        format!("notify-send -a pulse -r 91190 -t 800 -i {}/unmuted-{}.svg unmuted '{}'", ICODIR, dvce, nsink)
    };
    execute(&cmd).unwrap();
}

fn main() {
    let args = Args::parse();
    let step: u8 = 5;

    if let Some(input) = args.input {
        let cmd = execute("pamixer --list-sources").unwrap();
        let last_line = cmd.lines().last().unwrap();
        let nsink = last_line.split('\"').rev().nth(1).unwrap();

        if nsink.is_empty() {
            eprintln!("ERROR: Input device not found...");
            return;
        }
        let srce = "--default-source";
        match input {
            Actions::Increase => action_pamixer(nsink, srce, "i", step),
            Actions::Decrease => action_pamixer(nsink, srce, "d", step),
            Actions::Mute => mute(nsink, "pamixer", srce),
        }
    }

    if let Some(output) = args.output {
        let cmd = execute("pamixer --get-default-sink").unwrap();
        let last_line = cmd.lines().last().unwrap();
        let nsink = last_line.split('\"').rev().nth(1).unwrap();

        if nsink.is_empty() {
            eprintln!("ERROR: Output device not found...");
            return;
        }
        match output {
            Actions::Increase => action_pamixer(nsink, "", "i", step),
            Actions::Decrease => action_pamixer(nsink, "", "d", step),
            Actions::Mute => mute(nsink, "pamixer", ""),
        }
    }

    if let Some(player) = args.player {
        let cmd = execute("playerctl --list-all | grep -w '$OPTARG'").unwrap();
        let nsink = cmd.lines().last().unwrap();

        if nsink.is_empty() {
            eprintln!("ERROR: Output device not found...");
            return;
        }
        match player {
            Actions::Increase => action_playerctl(nsink, "+", step),
            Actions::Decrease => action_playerctl(nsink, "-", step),
            Actions::Mute => mute(nsink, "playerctl", ""),
        }
    }
}
