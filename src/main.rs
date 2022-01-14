use std::io::Read;
use std::{
    env,
    process::{Command, Stdio},
};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    // TODO: Work on parsing pipes better and handling edge cases...
    let out = args.join(" ").split("|").fold(None, |mut out, command| {
        let mut split_command = command.split_whitespace();
        let command_program = split_command.next().unwrap();
        let command_args: Vec<&str> = split_command.collect();

        let cmd = Command::new(command_program)
            .args(command_args)
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
