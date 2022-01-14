use std::{
    io,
    io::Read,
    process::{Command, Stdio},
};

struct CommandObject<'a> {
    name: &'a str,
    args: Vec<&'a str>
}

impl<'a> CommandObject<'a> {
    fn parse(string: &'a str) -> Self {
        let mut split = string.split_whitespace();
        let name = split.next().unwrap();
        let args: Vec<&str> = split.collect();

        Self {
            name,
            args
        }
    }
}

fn pipe_commands<'a, I>(commands: I)
where
    I: Iterator<Item = CommandObject<'a>>
{
    let out = commands.fold(None, |mut out, command| {
        let cmd = Command::new(command.name)
            .args(command.args)
            .stdin(out.take().map(Into::into).unwrap_or_else(Stdio::piped))
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to execute command");
        cmd.stdout
    });

    let mut s = String::new();
    out.unwrap().read_to_string(&mut s).unwrap();
    println!("{}", s);
}

fn main() {
    let stdin = io::stdin();
        
    loop {
        let mut buffer = String::new();
        stdin.read_line(&mut buffer).unwrap();

        // TODO: Work on parsing pipes better and handling edge cases...
        let commands = buffer.split("|")
            .map(|s| CommandObject::parse(s));
        
        pipe_commands(commands);
    }
}
