use std::result::Result::Err;
use std::{env, io};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use crate::{ShellError, ShellSignal, ShellState};
use crate::external::{call_external, find_external, get_external};
use crate::helper::resolve_path;

pub trait ShellCommand: Sync {
    fn name(&self) -> &'static str;
    fn exec(&self, state: &ShellState, args: &[&str], output: &mut dyn Write)
        -> Result<Option<ShellSignal>, ShellError>;
}

pub struct Echo;
static ECHO: Echo = Echo;
impl ShellCommand for Echo {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn exec(&self, _state: &ShellState, args: &[&str], output: &mut dyn Write)
        -> Result<Option<ShellSignal>, ShellError>
    {
        writeln!(output, "{}", args.join(" ")).map_err(|_| ShellError::OutputError)?;

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
            -> Result<Option<ShellSignal>, ShellError>
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
            -> Result<Option<ShellSignal>, ShellError>
    {
        'arg_loop: for &arg in args {
            let builtin = BUILTINS.iter().
                find(|cmd| cmd.name() == arg);
            if builtin.is_some() {
                writeln!(output, "{} is a shell builtin", arg)
                    .map_err(|_| ShellError::OutputError)?;
                continue 'arg_loop
            }

            if let Some(external) = find_external(arg, None) {
                writeln!(output, "{} is {}", arg, external.display())
                    .map_err(|_| ShellError::OutputError)?;
                continue 'arg_loop
            }

            writeln!(output, "{}: not found", arg)
                .map_err(|_| ShellError::OutputError)?;
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

    fn exec(&self, state: &ShellState, _args: &[&str], output: &mut dyn Write)
        -> Result<Option<ShellSignal>, ShellError>
    {
        writeln!(output, "{}", state.current_dir.display())
            .map_err(|_| ShellError::OutputError)?;
        Ok(None)
    }
}

pub struct Cd;
static CD: Cd = Cd;
impl ShellCommand for Cd {
    fn name(&self) -> &'static str {
        "cd"
    }

    fn exec(&self, state: &ShellState, args: &[&str], output: &mut dyn Write)
        -> Result<Option<ShellSignal>, ShellError>
    {
        if args.len() > 1 {
            writeln!(output, "{}: too many arguments", self.name())
                .map_err(|_| ShellError::OutputError)?;
            return Err(ShellError::InvalidArgument)
        }

        if let Ok(path) = resolve_path(args[0], &state.current_dir) {
            Ok(Some(ShellSignal::ChangeDir(path)))
        } else {
            writeln!(output, "{}: {}: No such file or directory", self.name(), args[0])
                .map_err(|_| ShellError::OutputError)?;
            Err(ShellError::InvalidArgument)
        }
    }
}

static BUILTINS: &[&dyn ShellCommand] = &[
    &ECHO,
    &EXIT,
    &TYPECMD,
    &PWD,
    &CD
];

pub fn call_builtin(state: &ShellState, cmd: &dyn ShellCommand, args: &[&str], output: &mut dyn Write)
                    -> Result<Option<ShellSignal>, ShellError>
{
    let signal = cmd.exec(state, args, output)?;
    Ok(signal)
}

pub fn call(state: &ShellState, name: &str, args: &[&str], output: &mut dyn Write)
    -> Result<Option<ShellSignal>, ShellError>
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

    Err(ShellError::CommandNotFound)
}

fn call_test(_state: &ShellState, name: &str, _args: &[&str], output: &mut dyn Write)
    -> Option<Result<Option<ShellSignal>, ShellError>>
{
    if name == "q" {
        let dir = PathBuf::from("../");
        writeln!(output, "{:?}", dir.is_relative()).unwrap();
        writeln!(output, "{:?}", dir.is_absolute()).unwrap();
        writeln!(output, "{:?}", dir.has_root()).unwrap();
    }

    None
}
