mod builtin;
mod helper;

use std::env;
#[allow(unused_imports)]
use std::io::{self, Write};
use crate::builtin::{call, call_builtin};

enum ShellSignal {
    Exit
}

#[derive(Default)]
struct ShellState {
    should_exit: bool
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
