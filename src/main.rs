use std::{
    env,
    io,
    path::Path,
    process::{
        Command as StdCommand,
        Stdio,
        Child
    }
};

use clap::Parser;

enum CommandType<'a> {
    ShellCommand(ShellCommand),
    SystemCommand(&'a str)
}

impl<'a> CommandType<'a> {
    fn parse(program: &str) -> CommandType {
        match program {
            "echo" => CommandType::ShellCommand(ShellCommand::Echo),
            "cd" => CommandType::ShellCommand(ShellCommand::Cd),
            "exit" => CommandType::ShellCommand(ShellCommand::Exit),
            _ => CommandType::SystemCommand(program)
        }
    }
}

#[derive(PartialEq)]
enum ShellCommand
{
    Echo,
    Cd,
    Exit
}

impl ShellCommand
{
    fn execute<'a, I>(&self, mut args: I)
    where
        I: Iterator<Item = &'a str>
    {
        match self {
            ShellCommand::Echo => {
                println!("{}", args.collect::<Vec<&str>>().concat());
            }
            ShellCommand::Cd => {
                let new_dir = args.next().unwrap();
                let root = Path::new(new_dir);
    
                match env::set_current_dir(&root) {
                    Ok(_) => println!("Changed directory to {}", root.display()),
                    Err(e) => eprintln!("{}", e)
                }
            }
            _ => return
        }
    }

    fn to_string<'a>(&self) -> &'a str {
        match &self {
            ShellCommand::Echo => "echo",
            ShellCommand::Cd => "cd",
            ShellCommand::Exit => "exit"
        }
    }
}

struct Command<'a>
{
    kind: CommandType<'a>,
    args: Box<dyn Iterator<Item = &'a str> + 'a>
}

impl<'a> Command<'a>
{
    fn parse(
        mut args: Box<dyn Iterator<Item = &'a str> + 'a>
    ) -> Option<Self> {
        let kind = CommandType::parse(args.next()?);

        Some(Self {
            kind,
            args
        })
    }

    fn parse_pipes(args: &'a str) -> impl Iterator<Item = Command<'a>> {
        args.split("|").map(|s| {
            Command::parse(Box::new(s.split_whitespace())).unwrap()
        })
    }

    fn pipe_commands(
        commands: impl Iterator<Item = Command<'a>>
    ) {
        let mut peekable = commands.peekable();
        let mut previous = None;
        while let Some(command) = peekable.next() {
            previous = Some(command.execute(previous, peekable.peek().is_some()));
        }
    }

    fn execute(self, previous: Option<Child>, has_next: bool) -> Child {
        let stdin = previous
            .map_or(
                Stdio::inherit(),
                |output: Child| Stdio::from(output.stdout.unwrap())
            );
                
        let stdout = if has_next {
            Stdio::piped()
        } else {
            Stdio::inherit()
        };

        match self.kind {
            CommandType::ShellCommand(program) => {
                if program == ShellCommand::Exit {
                    std::process::exit(0);
                }
                
                let a: &str = program.to_string();
                let b = self.args.collect::<Vec<&str>>().concat();
                let command = format!("{} {}", a, b.as_str());
                let args = [
                    "-c",
                    command.as_str()
                ];

                StdCommand::new(std::env::current_exe().unwrap())
                    .args(args)
                    .stdin(stdin)
                    .stdout(stdout)
                    .spawn()
                    .expect("Failed to execute command")
            }
            CommandType::SystemCommand(program) => {
                StdCommand::new(program)
                    .args(self.args)
                    .stdin(stdin)
                    .stdout(stdout)
                    .spawn()
                    .expect("Failed to execute command")
            }
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short)]
    command: Option<String>
}

fn main() {
    let args = Cli::parse();

    if let Some(command) = args.command {
        let mut command_args = command.split_whitespace();

        if let CommandType::ShellCommand(cmd) = CommandType::parse(command_args.next().unwrap()) {
            cmd.execute(command_args);
        } else {
            eprintln!("Invalid command/arguments.");
        }
    } else {
        loop {
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            
            let parsed = Command::parse_pipes(buffer.as_str());
            Command::pipe_commands(parsed);
        }
    }
}
