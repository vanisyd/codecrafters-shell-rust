use crate::io::Error;
use std::result::Result::Err;
use std::{env, io};
use std::io::{Write};
use crate::{ShellSignal, ShellState};
use crate::external::{call_external, find_external, get_external};

pub trait ShellCommand: Sync {
    fn name(&self) -> &'static str;
    fn exec(&self, state: &ShellState, args: &[&str], output: &mut dyn Write)
        -> io::Result<Option<ShellSignal>>;
}

pub struct Echo;
static ECHO: Echo = Echo;
impl ShellCommand for Echo {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn exec(&self, _state: &ShellState, args: &[&str], output: &mut dyn Write)
        -> io::Result<Option<ShellSignal>>
    {
        writeln!(output, "{}", args.join(" "))?;

        Ok(None)
    }
}

pub struct Exit;
static EXIT: Exit = Exit;
impl ShellCommand for Exit {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn exec(&self, _state: &ShellState, _args: &[&str], _output: &mut dyn Write)
            -> io::Result<Option<ShellSignal>>
    {
        Ok(Some(ShellSignal::Exit))
    }
}

pub struct TypeCmd;
static TYPECMD: TypeCmd = TypeCmd;
impl ShellCommand for TypeCmd {
    fn name(&self) -> &'static str {
        "type"
    }

    fn exec(&self, _state: &ShellState, args: &[&str], output: &mut dyn Write)
            -> io::Result<Option<ShellSignal>>
    {
        'arg_loop: for &arg in args {
            let builtin = BUILTINS.iter().
                find(|cmd| cmd.name() == arg);
            if builtin.is_some() {
                writeln!(output, "{} is a shell builtin", arg)?;
                continue 'arg_loop
            }

            if let Some(external) = find_external(arg, None) {
                writeln!(output, "{} is {}", arg, external.display())?;
                continue 'arg_loop
            }

            writeln!(output, "{}: not found", arg)?;
        }

        Ok(None)
    }
}

pub struct Pwd;
static PWD: Pwd = Pwd;
impl ShellCommand for Pwd {
    fn name(&self) -> &'static str {
        "pwd"
    }

    fn exec(&self, state: &ShellState, args: &[&str], output: &mut dyn Write) -> io::Result<Option<ShellSignal>> {
        writeln!(output, "{}", state.current_dir.display())?;
        Ok(None)
    }
}

static BUILTINS: &[&dyn ShellCommand] = &[
    &ECHO,
    &EXIT,
    &TYPECMD,
    &PWD,
];

pub fn call_builtin(state: &ShellState, cmd: &dyn ShellCommand, args: &[&str], output: &mut dyn Write)
                    -> io::Result<Option<ShellSignal>>
{
    let signal = cmd.exec(state, args, output)?;
    Ok(signal)
}

pub fn call(state: &ShellState, name: &str, args: &[&str], output: &mut dyn Write)
    -> io::Result<Option<ShellSignal>>
{
    if let Some(tst) = call_test(state, name, args, output) {
        return tst
    }

    let builtin = BUILTINS.iter().find(|cmd| cmd.name() == name);
    if let Some(&cmd) = builtin {
        return call_builtin(state, cmd, args, output);
    }

    let external = get_external(name, None);
    if let Some(cmd) = external {
        return call_external(state, &cmd, args, output);
    }

    Err(Error::from(io::ErrorKind::InvalidFilename))
}

fn call_test(state: &ShellState, name: &str, args: &[&str], output: &mut dyn Write)
    -> Option<io::Result<Option<ShellSignal>>>
{
    if name == "~" {
        let home = env::var_os("HOME").unwrap();
        writeln!(output, "{:?}", home).unwrap();
        return Some(Ok(None))
    }

    None
}
