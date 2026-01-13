#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut command = String::new();

    loop {
        io::stdin().read_line(&mut command).unwrap();
        command = command.trim().to_string();
        if command == "exit" {
            break
        } else {
            println!("{command}: command not found");
            print!("$ ");
            io::stdout().flush().unwrap();
        }
        command.clear();
    }

    io::stdout().flush().unwrap();
}
