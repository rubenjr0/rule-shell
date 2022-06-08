use colored::{ColoredString, Colorize};
use signal_hook::{consts::SIGCHLD, iterator::Signals};
use std::io::Write;
use std::process::{exit, Child, Command};
use std::sync::{Arc, Mutex};
use std::thread;

struct Process {
    name: String,
    handle: Child,
}

type HandleMutex = Arc<Mutex<Vec<Process>>>;

impl Process {
    fn id(&self) -> u32 {
        self.handle.id()
    }
}

fn colorize_pid(pid: &u32) -> ColoredString {
    format!("{}", pid).green()
}

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

fn initialize_signal_handler(background_processes: &HandleMutex) {
    let mut signals = Signals::new(&[SIGCHLD]).expect("Error creating signal handler");
    let background_processes = Arc::clone(background_processes);
    thread::spawn(move || {
        for _ in signals.forever() {
            let mut finished_indexes: Vec<usize> = Vec::new();
            let mut background_processes = background_processes.lock().unwrap();

            for (index, process) in background_processes.iter_mut().enumerate() {
                let id = process.id();
                match process.handle.try_wait() {
                    Ok(Some(status)) => {
                        finished_indexes.push(index);
                        println!(
                            "Process {} exited with status {}",
                            colorize_pid(&id),
                            status
                        );
                    }
                    Err(e) => panic!("Error getting process {} status: {}", colorize_pid(&id), e),
                    _ => continue,
                }
            }
            for index in finished_indexes {
                background_processes.remove(index);
            }
        }
    });
}

fn jobs(background_processes: &HandleMutex) {
    let mut background_processes = background_processes.lock().unwrap();
    for (index, process) in background_processes.iter_mut().enumerate() {
        let status = if let Ok(None) = process.handle.try_wait() {
            "Running"
        } else {
            "Background"
        };
        println!(
            "[{}] {} {} - {}",
            index + 1,
            process.name,
            colorize_pid(&process.id()),
            status
        );
    }
}

fn main() {
    const PROMPT: &str = "λ";

    let background_processes: HandleMutex = Arc::new(Mutex::new(Vec::new()));

    initialize_signal_handler(&background_processes);

    loop {
        print!("{} ", PROMPT.cyan());
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).unwrap() == 0 {
            exit(0);
        }

        if let Some(input) = parse_input(&input) {
            let (name, args, background) = parse_command(input);

            if name == "jobs" {
                jobs(&background_processes);
                continue;
            }

            let mut command = Command::new(&name);
            command.args(&args);
            if background {
                match command.spawn() {
                    Ok(handle) => {
                        let mut background_processes = background_processes.lock().unwrap();
                        background_processes.push(Process { name, handle });
                    }
                    Err(e) => panic!("Error! {}", e),
                };
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
