mod builtin;
mod helper;
mod external;

use std::env;
#[allow(unused_imports)]
#[allow(dead_code)]
use std::io::{self, Write};
use std::path::PathBuf;
use crate::builtin::{call};

enum ShellSignal {
    Exit
}

struct ShellState {
    current_dir: PathBuf,
    should_exit: bool
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
            should_exit: false
        }
    }
}

impl ShellState {
    pub fn update(&mut self, signal: ShellSignal) {
        match signal {
            ShellSignal::Exit => self.should_exit = true,
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

        let parts: Vec<&str> = input.split_whitespace().collect();

        if let Some(cmd_name) = parts.first() {
            let args: &[&str] = &parts[1..];
            if let Ok(result) = call(&mut state, cmd_name, args, io::stdout().by_ref()) {
                if let Some(signal) = result {
                    state.update(signal);
                }
            } else {
                println!("{cmd_name}: command not found");
            }
        }

        io::stdout().flush().unwrap();
        input.clear();
    }
}
