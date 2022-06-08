use nix::unistd::chdir;
use std::io::Write;
use std::process::exit;
use std::process::Command;

fn parse_input(input: &str) -> Option<Vec<String>> {
    let input: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
    if input.len() == 0 {
        None
    } else {
        Some(input)
    }
}

fn parse_command(input: Vec<String>) -> (String, Vec<String>, bool) {
    let name: String = input.first().unwrap().to_string();
    let background = match input.last() {
        None => false,
        Some(c) => *c == "&",
    };
    let args_len = input.len() - if background { 1 } else { 0 };
    let args = input[1..args_len].to_vec();
    (name, args, background)
}

fn main() {
    const PROMPT: &str = "$ ";

    let background_handles: Vec<String>;

    loop {
        print!("{} ", PROMPT);
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).unwrap() == 0 {
            exit(0);
        }

        if let Some(input) = parse_input(&input) {
            let (name, args, background) = parse_command(input);
            let mut command = Command::new(&name);
            command.args(&args);
            if background {
                // background_handles.push(todo!());
                command.spawn();
            } else {
                if let Err(e) = command.status() {
                    println!("Error: {}", e);
                }
            }
            /*
            println!(
                "Running command {} with args {:?}, bg: {}",
                name, args, background
            );
            */
        }
    }
}
