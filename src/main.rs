use colored::{ColoredString, Colorize};
use signal_hook::{consts::SIGCHLD, iterator::Signals};
use std::io::Write;
use std::process::{exit, Child, Command};
use std::sync::{Arc, Mutex};
use std::thread;

enum ProcessState {
    Background,
    Foreground,
    Stopped,
}

impl ProcessState {
    fn to_string(&self) -> String {
        match self {
            ProcessState::Background => String::from("background"),
            ProcessState::Foreground => String::from("foreground"),
            ProcessState::Stopped => String::from("stopped"),
        }
    }
}

struct Process {
    name: String,
    handle: Child,
    state: ProcessState,
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

fn initialize_signal_handler(process_list: &HandleMutex) {
    let mut signals = Signals::new(&[SIGCHLD]).expect("Error creating signal handler");
    let process_list = Arc::clone(process_list);
    thread::spawn(move || {
        for _ in signals.forever() {
            let mut finished_indexes: Vec<usize> = Vec::new();
            let mut process_list = process_list.lock().unwrap();

            for (index, process) in process_list.iter_mut().enumerate() {
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
                process_list.remove(index);
            }
        }
    });
}

fn jobs(process_list: &HandleMutex) {
    let process_list = process_list.lock().unwrap();
    for (index, process) in process_list.iter().enumerate() {
        println!(
            "[{}] {} {} - {}",
            index + 1,
            process.name,
            colorize_pid(&process.id()),
            process.state.to_string()
        );
    }
}

fn fg(process_list: &HandleMutex, index: usize) {
    let mut process_list = process_list.lock().unwrap();
    if let Some(p) = process_list.get_mut(index) {
        p.state = ProcessState::Foreground;
        match p.handle.wait() {
            Ok(status) => {
                println!("Process {} exited with code {}", p.id(), status);
                process_list.remove(index);
            }
            Err(e) => println!("Error waiting for process {}: {}", p.id(), e),
        }
    }
}

fn main() {
    const PROMPT: &str = "λ";

    let process_list: HandleMutex = Arc::new(Mutex::new(Vec::new()));

    initialize_signal_handler(&process_list);

    loop {
        print!("{} ", PROMPT.cyan());
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).unwrap() == 0 {
            exit(0);
        }

        if let Some(input) = parse_input(&input) {
            let (name, args, background) = parse_command(input);

            if name == "exit" {
                exit(0);
            } else if name == "jobs" {
                jobs(&process_list);
                continue;
            } else if name == "fg" {
                let arg: Result<usize, _> = if let Some(arg) = args.first() {
                    arg.parse()
                } else {
                    Ok(1)
                };
                match arg {
                    Ok(index) => fg(&process_list, index - 1),
                    Err(e) => println!("Error parsing argument: {}", e),
                }
                continue;
            }

            let mut command = Command::new(&name);
            command.args(&args);
            if background {
                match command.spawn() {
                    Ok(handle) => {
                        let mut process_list = process_list.lock().unwrap();
                        process_list.push(Process {
                            name,
                            handle,
                            state: ProcessState::Background,
                        });
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
