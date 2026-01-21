use crate::external::{call_external, find_external, get_external};
use crate::helper::resolve_path;
use crate::lexer::ParsedCommand;
use crate::{ShellError, ShellSignal, ShellState};
use std::io::Write;
use std::path::PathBuf;
use std::result::Result::Err;

pub trait ShellCommand: Sync {
    fn name(&self) -> &'static str;
    fn exec<'a>(
        &self,
        state: &ShellState,
        args: &mut dyn Iterator<Item = &'a str>,
        output: &mut dyn Write,
    ) -> Result<Option<ShellSignal>, ShellError>;
}

pub struct Echo;
static ECHO: Echo = Echo;
impl ShellCommand for Echo {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn exec<'a>(
        &self,
        _state: &ShellState,
        args: &mut dyn Iterator<Item = &'a str>,
        output: &mut dyn Write,
    ) -> Result<Option<ShellSignal>, ShellError> {
        for (i, arg) in args.enumerate() {
            if i > 0 {
                write!(output, " ").map_err(|_| ShellError::OutputError)?;
            }
            write!(output, "{}", arg).map_err(|_| ShellError::OutputError)?;
        }
        write!(output, "\n").map_err(|_| ShellError::OutputError)?;

        Ok(None)
    }
}

pub struct Exit;
static EXIT: Exit = Exit;
impl ShellCommand for Exit {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn exec<'a>(
        &self,
        _state: &ShellState,
        _args: &mut dyn Iterator<Item = &'a str>,
        _output: &mut dyn Write,
    ) -> Result<Option<ShellSignal>, ShellError> {
        Ok(Some(ShellSignal::Exit))
    }
}

pub struct TypeCmd;
static TYPECMD: TypeCmd = TypeCmd;
impl ShellCommand for TypeCmd {
    fn name(&self) -> &'static str {
        "type"
    }

    fn exec<'a>(
        &self,
        _state: &ShellState,
        args: &mut dyn Iterator<Item = &'a str>,
        output: &mut dyn Write,
    ) -> Result<Option<ShellSignal>, ShellError> {
        'arg_loop: for arg in args {
            let builtin = BUILTINS.iter().find(|cmd| cmd.name() == arg);
            if builtin.is_some() {
                writeln!(output, "{} is a shell builtin", arg)
                    .map_err(|_| ShellError::OutputError)?;
                continue 'arg_loop;
            }

            if let Some(external) = find_external(arg, None) {
                writeln!(output, "{} is {}", arg, external.display())
                    .map_err(|_| ShellError::OutputError)?;
                continue 'arg_loop;
            }

            writeln!(output, "{}: not found", arg).map_err(|_| ShellError::OutputError)?;
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

    fn exec<'a>(
        &self,
        state: &ShellState,
        _args: &mut dyn Iterator<Item = &'a str>,
        output: &mut dyn Write,
    ) -> Result<Option<ShellSignal>, ShellError> {
        writeln!(output, "{}", state.current_dir.display()).map_err(|_| ShellError::OutputError)?;
        Ok(None)
    }
}

pub struct Cd;
static CD: Cd = Cd;
impl ShellCommand for Cd {
    fn name(&self) -> &'static str {
        "cd"
    }

    fn exec<'a>(
        &self,
        state: &ShellState,
        args: &mut dyn Iterator<Item = &'a str>,
        output: &mut dyn Write,
    ) -> Result<Option<ShellSignal>, ShellError> {
        let dir = args.next().unwrap_or("");
        if args.next().is_some() {
            writeln!(output, "{}: too many arguments", self.name())
                .map_err(|_| ShellError::OutputError)?;
            return Err(ShellError::InvalidArgument);
        }

        if let Ok(path) = resolve_path(dir, &state.current_dir) {
            Ok(Some(ShellSignal::ChangeDir(path)))
        } else {
            writeln!(
                output,
                "{}: {}: No such file or directory",
                self.name(),
                dir
            )
            .map_err(|_| ShellError::OutputError)?;
            Err(ShellError::InvalidArgument)
        }
    }
}

static BUILTINS: &[&dyn ShellCommand] = &[&ECHO, &EXIT, &TYPECMD, &PWD, &CD];

pub fn call_builtin(
    state: &ShellState,
    cmd: &dyn ShellCommand,
    command: &ParsedCommand,
    output: &mut dyn Write,
) -> Result<Option<ShellSignal>, ShellError> {
    let mut args = command.args();
    let signal = cmd.exec(state, &mut args, output)?;
    Ok(signal)
}

pub fn call(
    state: &ShellState,
    command: &ParsedCommand,
    output: &mut dyn Write,
) -> Result<Option<ShellSignal>, ShellError> {
    /*if let Some(tst) = call_test(state, name, args, output) {
        return tst
    }*/
    if let Some(cmd_name) = command.cmd() {
        let builtin = BUILTINS.iter().find(|cmd| cmd.name() == cmd_name);
        if let Some(&cmd) = builtin {
            return call_builtin(state, cmd, command, output);
        }

        let external = get_external(cmd_name, None);
        let mut args = command.args();
        if let Some(cmd) = external {
            return call_external(state, &cmd, &mut args, output);
        }
    }

    Err(ShellError::CommandNotFound)
}

fn call_test(
    _state: &ShellState,
    name: &str,
    _args: &[&str],
    output: &mut dyn Write,
) -> Option<Result<Option<ShellSignal>, ShellError>> {
    if name == "q" {
        let dir = PathBuf::from("../");
        writeln!(output, "{:?}", dir.is_relative()).unwrap();
        writeln!(output, "{:?}", dir.is_absolute()).unwrap();
        writeln!(output, "{:?}", dir.has_root()).unwrap();
    }

    None
}
