use std::{
    env::{self, args_os},
    io,
    path::Path,
    process::{
        Command as StdCommand,
        Stdio,
        Child
    }
};

enum CommandType<'a> {
    ShellCommand(ShellCommand),
    SystemCommand(&'a str)
}

impl<'a> CommandType<'a> {
    fn parse(program: &'a str) -> CommandType {
        match program {
            "echo" => CommandType::ShellCommand(ShellCommand::echo),
            "cd" => CommandType::ShellCommand(ShellCommand::cd),
            "exit" => CommandType::ShellCommand(ShellCommand::exit),
            _ => CommandType::SystemCommand(program)
        }
    }
}

#[derive(PartialEq)]
enum ShellCommand {
    echo,
    cd,
    exit
}

impl ShellCommand {
    fn to_string<'a>(&self) -> &'a str {
        match &self {
            ShellCommand::echo => "echo",
            ShellCommand::cd => "cd",
            ShellCommand::exit => "exit"
        }
    }
}

struct Command<'a, I: Iterator<Item = &'a str>> {
    kind: CommandType<'a>,
    args: I
}

impl<'a, I: Iterator<Item = &'a str>> Command<'a, I> {
    fn parse(mut args: I) -> Option<Self>
    {
        let kind = CommandType::parse(args.next()?);

        Some(Self {
            kind,
            args
        })
    }

    fn pipe_commands<C>(commands: C)
    where
        C: Iterator<Item = Command<'a, I>>
    {
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
                if program == ShellCommand::exit {
                    std::process::exit(0);
                }

                StdCommand::new(std::env::current_exe().unwrap())
                    .args(["-c", program.to_string()].into_iter().chain(self.args))
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

fn main() {
    let mut args = env::args().skip(1);

    if let Some(arg) = args.next() {
        match arg.as_str() {
            "-c" => {
                if let CommandType::ShellCommand(cmd) = CommandType::parse(args.next().unwrap().as_str()) {
                    match cmd {
                        ShellCommand::echo => {
                            println!("{}", args.collect::<Vec<String>>().concat());
                        }
                        ShellCommand::cd => {
                            let new_dir = args.next().take().unwrap();
                            let root = Path::new(new_dir.as_str());
                
                            if let Err(e) = env::set_current_dir(&root) {
                                eprintln!("{}", e);
                            } else {
                                println!("Changed directory to {}", root.display());
                            }
                        }
                        _ => return
                    }
                } else {
                    eprintln!("Invalid command/arguments.");
                }
            }
            _ => eprintln!("Invalid arguments.")
        }
    } else {
        loop {
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            
            // TODO: Work on parsing pipes better and handling edge cases...
            let commands = buffer.split("|")
                .map(|s| {
                    Command::parse(s.split_whitespace()).unwrap()
                })
                .peekable();

            Command::pipe_commands(commands);
        }
    }
}
