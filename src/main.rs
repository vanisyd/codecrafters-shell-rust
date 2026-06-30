mod builtin;
mod external;
mod helper;
mod lexer;

use crate::builtin::call;
use crate::lexer::{OutputWriter, ParsedCommand, Parser};
use std::env;
#[allow(unused_imports)]
#[allow(dead_code)]
use std::io::{self, Write};
use std::path::{PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
enum ShellError {
    #[error("{0}: command not found")]
    CommandNotFound(String),
    #[error("invalid argument")]
    InvalidArgument,
    #[error("output error")]
    OutputError,
    #[error("command execution error")]
    ExecutionError,
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

enum ShellSignal {
    Exit,
    ChangeDir(PathBuf),
}

struct ShellState {
    current_dir: PathBuf,
    should_exit: bool,
}

impl Default for ShellState {
    fn default() -> Self {
        let current_dir = if let Ok(cur_dir) = env::current_dir() {
            cur_dir
        } else {
            PathBuf::from("/")
        };

        Self {
            current_dir,
            should_exit: false,
        }
    }
}

impl ShellState {
    pub fn update(&mut self, signal: ShellSignal) {
        match signal {
            ShellSignal::Exit => self.should_exit = true,
            ShellSignal::ChangeDir(dir) => self.current_dir = dir,
        }
    }
}

fn main() {
    let mut state = ShellState::default();

    let mut input = String::new();
    while !state.should_exit {
        print!("$ ");
        io::stdout().flush().unwrap();

        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();

        let mut parser = Parser::new(&input);
        let parsed_cmd = parser.parse_command();

        match execute_cmd(&mut state, parsed_cmd) {
            Ok(_) => {},
            Err(e) => {
                match e {
                    ShellError::CommandNotFound(e) => {
                        println!("{}", e);
                    },
                    _ => {}
                }
            }
        }

        io::stdout().flush().unwrap();
        input.clear();
    }
}

fn execute_cmd(mut state: &mut ShellState, cmd: ParsedCommand) -> Result<Option<ShellSignal>, ShellError> {
    {
        let (stdout, stderr) = cmd.output();
        let (mut stdout_writer, mut stderr_writer) = match (stdout.as_writer(), stderr.as_writer()) {
            (Ok(stdout), Ok(stderr)) => (stdout, stderr),
            (Err(e), _) | (_, Err(e)) => return Err(ShellError::Unknown(anyhow::anyhow!(e))),
        };

        let signal = call(&mut state, &cmd, &mut stdout_writer, &mut stderr_writer)?;
        if let Some(signal) = signal {
            state.update(signal);
        }
    }

    Ok(None)
}
