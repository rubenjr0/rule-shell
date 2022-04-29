use std::fmt;

pub enum CommandStatus<'a> {
    None,
    Command(CommandBuilder<'a>),
}

pub struct CommandBuilder<'a> {
    name: &'a str,
    args: Vec<&'a str>,
    background: bool,
}

impl<'a> CommandBuilder<'a> {
    pub fn get_name(&self) -> &str {
        self.name
    }

    pub fn get_args(&self) -> &Vec<&str> {
        &self.args
    }

    pub fn get_background(&self) -> bool {
        self.background
    }

    pub fn build_command(input: &'a str) -> CommandStatus {
        let input = input.trim();
        if input.len() == 0 {
            return CommandStatus::None;
        }
        let input: Vec<&str> = input.split_whitespace().collect();
        let name = input.first().unwrap();
        let mut args = input[1..].to_vec();
        let mut background = false;
        if let Some(c) = args.last() {
            background = if (*c).eq("&") {
                args.pop();
                true
            } else {
                false
            }
        }
        let command = CommandBuilder {
            name,
            args,
            background,
        };
        CommandStatus::Command(command)
    }
}

impl<'a> fmt::Display for CommandBuilder<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {:?}, bg: {}", self.name, self.args, self.background)
    }
}
