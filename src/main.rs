use std::{
    io,
    io::Read,
    process::{
        Command as StdCommand,
        Stdio,
        ChildStdout
    },
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

    fn execute(&self, out: &mut Option<ChildStdout>) -> Option<ChildStdout> {
        match &self.kind {
            CommandType::SystemCommand(cmd) => {
                let std_cmd = StdCommand::new(cmd)
                    .args(&self.args)
                    .stdin(out.take().map(Into::into).unwrap_or_else(Stdio::piped))
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("Failed to execute command");
                std_cmd.stdout
            }
            CommandType::ShellCommand(cmd) => {
                todo!();
            }
            _ => None
        }
    }

    fn pipe_commands<I>(commands: I)
    where
        I: Iterator<Item = Command<'a>>
    {
        let out = commands.fold(None, |mut out, command| {
            command.execute(&mut out)
        });

        let mut s = String::new();
        out.unwrap().read_to_string(&mut s).unwrap();
        println!("{}", s);
    }
}

fn main() {
    let stdin = io::stdin();
        
    loop {
        let mut buffer = String::new();
        stdin.read_line(&mut buffer).unwrap();

        // TODO: Work on parsing pipes better and handling edge cases...
        let commands = buffer.split("|")
            .map(|s| Command::parse(s));
        
        Command::pipe_commands(commands);
    }
}
