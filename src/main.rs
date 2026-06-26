mod builtin;
mod external;
mod helper;
mod lexer;

use crate::builtin::call;
use crate::lexer::{Parser};
use std::env;
#[allow(unused_imports)]
#[allow(dead_code)]
use std::io::{self, Write};
use std::io::Stdout;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
enum ShellError {
    #[error("command not found")]
    CommandNotFound,
    #[error("invalid argument")]
    InvalidArgument,
    #[error("output error")]
    OutputError,
    #[error("command execution error")]
    ExecutionError,
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

        if let Some(cmd_name) = parsed_cmd.cmd() {
            match parsed_cmd.output().as_writer() {
                Ok(mut writer) => {
                    let exec_result = call(&mut state, &parsed_cmd, &mut writer);
                    match exec_result {
                        Ok(result) => {
                            if let Some(signal) = result {
                                state.update(signal);
                            }
                        }
                        Err(e) => match e {
                            ShellError::CommandNotFound => {
                                println!("{}: {}", cmd_name, e.to_string());
                            }
                            _ => {}
                        },
                    }
                },
                Err(e) => {
                    println!("{e}")
                }
            };


        }

        io::stdout().flush().unwrap();
        input.clear();
    }
}
