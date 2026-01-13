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

struct TypeCmd {
    arg: Vec<String>
}

impl TypeCmd {
    fn new(arg: Vec<String>) -> Self {
        Self {
            arg
        }
    }
}

impl Command for TypeCmd {
    fn exec(&mut self) {
        if parse_command(self.arg.iter().map(|s| s.as_str()).collect()).is_some() {
            println!("{} is a shell builtin", self.arg[0])
        } else {
            println!("{}: not found", self.arg.join(" "))
        }
    }
}

trait Command {
    fn exec(&mut self);
}

fn parse_command(args: Vec<&str>) -> Option<Box<dyn Command>> {
    let cmd_name = *args.get(0)?;

    match cmd_name {
        "echo" => Some(Box::new(Echo::new(args[1..].join(" ")))),
        "type" => {
            let cmd_arg = args[1..].iter().map(|&s| s.to_string()).collect();
            Some(Box::new(TypeCmd::new(cmd_arg)))
        },
        "exit" => Some(Box::new(Exit {})),
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
