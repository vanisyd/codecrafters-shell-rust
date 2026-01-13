#[allow(unused_imports)]
use std::io::{self, Write};

struct Echo {
    text: String
}
impl Echo {
    fn new(text: String) -> Self {
        Self {
            text
        }
    }
}
impl Command for Echo {
    fn exec(&mut self) {
        println!("{}", self.text);
        io::stdout().flush().unwrap();
    }
}

struct Exit {}
impl Command for Exit {
    fn exec(&mut self) {
        todo!()
    }
}

trait Command {
    fn exec(&mut self);
}

fn parse_command(args: Vec<&str>) -> Option<impl Command> {
    let cmd_name = *args.get(0)?;
    match cmd_name {
        "echo" => Some(Echo::new(args[1..].to_owned().join(" "))),
        _ => None
    }
}

fn main() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut input = String::new();

    loop {
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();
        if input == "exit" {
            break
        } else {
            let args: Vec<&str> = input.split_whitespace().collect();
            match parse_command(args) {
                Some(mut cmd) => cmd.exec(),
                None => {
                    println!("{input}: command not found");
                }
            }

            print!("$ ");
            io::stdout().flush().unwrap();
        }
        input.clear();
    }

    io::stdout().flush().unwrap();
}
