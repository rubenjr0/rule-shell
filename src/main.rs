use std::{io::Write, path::Path, process::exit};
mod command_builder;
use command_builder::{CommandBuilder, CommandStatus};
use nix::unistd::chdir;
use std::process::Command;

fn main() {
    const PROMPT: &str = ">>";
    loop {
        print!("{} ", PROMPT);
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).unwrap() == 0 {
            exit(0);
        }
        let command = CommandBuilder::build_command(&input);
        match command {
            CommandStatus::None => continue,
            CommandStatus::Command(c) => {
                let name = c.get_name();
                let args = c.get_args();
                let background = c.get_background();

                if name.eq("cd") {
                    let path = match args.first() {
                        Some(p) => Path::new(p),
                        None => Path::new("."),
                    };
                    match chdir(path) {
                        Ok(_) => continue,
                        Err(e) => println!("Error! {}", e),
                    }
                    continue;
                }

                let mut command = Command::new(&name);
                command.args(args);

                if background {
                    command.spawn().expect("Failed on spawn");
                } else {
                    match command.status() {
                        Ok(_) => continue,
                        Err(_) => println!("Error, command `{}` not found!", name),
                    }
                }
            }
        }
    }
}
