use std::io;

use clap::Parser;
use rush::command::{Command,CommandType};

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

        match CommandType::parse(command_args.next().unwrap()) {
            CommandType::ShellCommand(cmd) => {
                cmd.execute(command_args);
            }
            CommandType::SystemCommand(cmd) => {
                Command::parse(Box::new([cmd].into_iter()))
                    .execute(None, false);
            }
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
