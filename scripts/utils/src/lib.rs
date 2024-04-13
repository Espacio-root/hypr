use std::process::{self, Command, Stdio};
use std::io;

fn execute_raw(cmd: &str) -> Result<process::Child, io::Error> {
    let cmds: Vec<&str> = cmd.split(" | ").collect();
    match cmds.len() {
        0 => Err(io::Error::new(io::ErrorKind::InvalidInput, "No command provided!")),
        1 => {
            let args = parse_command(cmds[0]);
            let output = Command::new(&args[0])
                .args(&args[1..])
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to spawn command");
            Ok(output)
        }
        _ => {
            let args = parse_command(cmds[cmd.len()-1]);
            let output = Command::new(&args[0])
                .args(&args[1..])
                .stdin(Stdio::from(
                    execute_raw(&cmds[1..cmds.len()-1].join("|")).unwrap().stdout.unwrap()
                ))
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to spawn command");
            Ok(output)
        }
    }
}

pub fn execute_cmd(cmd: &str) -> Result<process::Output, io::Error> {
    match execute_raw(cmd) {
        Ok(child) => Ok(child.wait_with_output().unwrap()),
        Err(e) => Err(e)
    }
}

pub fn execute(cmd: &str) -> Result<String, io::Error> {
    match execute_cmd(cmd) {
        Ok(output) => Ok(String::from_utf8_lossy(&output.stdout).trim().to_string()),
        Err(e) => Err(e)
    }
}

fn parse_command(cmd: &str) -> Vec<String> {
    let mut cmd_iter = cmd.chars().into_iter();
    let mut res: Vec<String> = Vec::new();
    let mut word = String::new();
    let mut inside_quotes = false;

    while let Some(s) = cmd_iter.next() {
        match s {
            ' ' => {
                if !inside_quotes {
                    if !word.is_empty() {
                        res.push(word.clone());
                        word.clear();
                    }
                } else {
                    word.push(s);
                }
            },
            '"' | '\'' => inside_quotes = !inside_quotes,
            _ => word.push(s)
        }
    };
    if !word.is_empty() {
        res.push(word.clone());
    }
    res
}

pub fn get_progress_bar(progress:u32, bar_length:u32) -> String {
    let num_dots = progress * bar_length / 100;
    let mut bar = String::new();

    bar.push('[');
    bar.push_str("+".repeat(num_dots as usize).as_str());
    bar.push_str(" ".repeat((bar_length - num_dots) as usize).as_str());
    bar.push(']');
    bar
}

pub fn pad_progress(progress: u32, max_length: u32) -> String {
    let padding_length = max_length - progress.to_string().len() as u32 - 1;

    if padding_length.gt(&0) {
        format!("{}%{}", progress, " ".repeat(padding_length as usize))
    } else {
        format!("{}%", progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_without_quotes() {
        let result = parse_command("ls -l -a");
        assert_eq!(result, Vec::from(["ls", "-l", "-a"]));
    }

    #[test]
    fn command_with_single_quotes() {
        let result = parse_command("grep 'search term' file.txt");
        assert_eq!(result, Vec::from(["grep", "search term", "file.txt"]));
    }

    #[test]
    fn command_with_double_quotes() {
        let result = parse_command("grep \"search term\" file.txt");
        assert_eq!(result, Vec::from(["grep", "search term", "file.txt"]));
    }

    #[test]
    fn execute_without_quotes() {
        let result = execute("echo hello").unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn execute_with_single_quotes() {
        let result = execute("echo 'hello'").unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn execute_with_double_quotes() {
        let result = execute("echo \"hello\"").unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn progress_bar_test() {
        let bar = get_progress_bar(75, 10);
        assert_eq!(bar, "[+++++++   ]");
    }

    #[test]
    fn pad_test() {
        let value = pad_progress(75, 4);
        assert_eq!(value, "75% ")
    }
}
