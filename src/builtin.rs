use std::{env, fs, io};
use std::fs::{read_dir, DirEntry, Permissions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use thiserror::__private::AsDisplay;
use crate::{ShellSignal, ShellState};
use crate::helper::visit_dirs;

pub trait Command: Sync {
    fn name(&self) -> &'static str;
    fn exec(&self, state: &ShellState, args: &[&str], output: &mut dyn Write)
        -> io::Result<Option<ShellSignal>>;
}

pub struct Echo;
static ECHO: Echo = Echo;
impl Command for Echo {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn exec(&self, state: &ShellState, args: &[&str], output: &mut dyn Write)
        -> io::Result<Option<ShellSignal>>
    {
        writeln!(output, "{}", args.join(" "))?;

        Ok(None)
    }
}

pub struct Exit;
static EXIT: Exit = Exit;
impl Command for Exit {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn exec(&self, state: &ShellState, args: &[&str], output: &mut dyn Write)
            -> io::Result<Option<ShellSignal>>
    {
        Ok(Some(ShellSignal::Exit))
    }
}

pub struct TypeCmd;
static TYPECMD: TypeCmd = TypeCmd;
impl Command for TypeCmd {
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
                writeln!(output, "{} is shell builtin", arg)?;
                continue 'arg_loop
            }

            if arg.contains(std::path::MAIN_SEPARATOR) {
                if fs::exists(arg).is_ok() && let Ok(metadata) = fs::metadata(arg) {
                    if metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0) {
                        writeln!(output, "{} is {}", arg, arg)?;
                        continue 'arg_loop
                    }
                }
            }

            if let Some(path_var) = env::var_os("PATH") {
                let paths = env::split_paths(&path_var);
                for path in paths {
                    let full_path = path.join(arg);
                    if full_path.is_file() && let Ok(metadata) = full_path.metadata() {
                        if metadata.permissions().mode() & 0o111 != 0 {
                            writeln!(output, "{} is {}", arg, full_path.display())?;
                            continue 'arg_loop
                        }
                    }
                }
            }

            writeln!(output, "{}: not found", arg)?;
        }

        Ok(None)
    }
}

static BUILTINS: &[&dyn Command] = &[
    &ECHO,
    &EXIT,
    &TYPECMD
];

pub fn call_builtin(state: &ShellState, name: &str, args: &[&str], output: &mut dyn Write)
    -> Result<Option<ShellSignal>, ()>
{
    let builtin = BUILTINS.iter().find(|cmd| cmd.name() == name);

    if let Some(cmd) = builtin {
        let signal = cmd.exec(state, args, output)
            .map_err(|_| ())?;
        Ok(signal)
    } else {
        Err(())
    }
}
