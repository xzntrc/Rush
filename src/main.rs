use std::{
    env,
    io,
    io::Read,
    path::Path,
    fs,
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

enum ShellCommand {
    cd,
    ls
}

struct Command<'a> {
    kind: CommandType<'a>,
    args: Vec<&'a str>
}

impl<'a> Command<'a> {
    fn parse(string: &'a str) -> Self {
        let mut split = string.split_whitespace();
        let kind = CommandType::SystemCommand(split.next().unwrap());
        let args: Vec<&str> = split.collect();

        Self {
            kind,
            args
        }
    }

    fn execute(&self, previous: Option<Child>, has_next: bool) -> Option<Child> {
        match &self.kind {
            CommandType::SystemCommand(cmd) => {
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

                let output = StdCommand::new(cmd)
                    .args(&self.args)
                    .stdin(stdin)
                    .stdout(stdout)
                    .spawn()
                    .expect("Failed to execute command");
                
                Some(output)
            }
            CommandType::ShellCommand(cmd) => {
                match cmd {
                    ShellCommand::cd => {
                        let new_dir = self.args[0];

                        let root = Path::new(new_dir);
                        if let Err(e) = env::set_current_dir(&root) {
                            eprintln!("{}", e);
                        } else {
                            println!("Changed directory to {}", root.display());
                        }
                    }
                    ShellCommand::ls => {
                        let paths = fs::read_dir(env::current_dir().unwrap()).unwrap();
                        
                        for path in paths {
                            println!("{}", path.unwrap().file_name().to_str().unwrap());
                        }
                    }
                }

                None
            }
        }
    }
}

fn main() {
    loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();

        // TODO: Work on parsing pipes better and handling edge cases...
        let mut commands = buffer.split("|")
            .map(|s| Command::parse(s))
            .peekable();
        
        let mut previous = None;
        while let Some(command) = commands.next() {
            previous = command.execute(previous, commands.peek().is_some());
        }
    }
}
