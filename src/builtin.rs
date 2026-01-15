use crate::io::Error;
use std::result::Result::Err;
use std::{env, io};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use crate::{ShellSignal, ShellState};
use crate::helper::{is_executable};
use std::process::{Command, Stdio};
use std::thread;
use std::sync::mpsc;

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

    fn exec(&self, state: &ShellState, args: &[&str], output: &mut dyn Write)
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

    fn exec(&self, state: &ShellState, args: &[&str], output: &mut dyn Write)
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

            if let Some(external) = get_external(arg, None) {
                writeln!(output, "{} is {}", arg, external.display())?;
                continue 'arg_loop
            }

            writeln!(output, "{}: not found", arg)?;
        }

        Ok(None)
    }
}

static BUILTINS: &[&dyn ShellCommand] = &[
    &ECHO,
    &EXIT,
    &TYPECMD
];

pub fn call_builtin(state: &ShellState, cmd: &dyn ShellCommand, args: &[&str], output: &mut dyn Write)
                    -> io::Result<Option<ShellSignal>>
{
    let signal = cmd.exec(state, args, output)?;
    Ok(signal)
}

pub fn get_external(name: &str, path: Option<PathBuf>) -> Option<PathBuf> {
    let mut app_path: PathBuf = path.unwrap_or_else(|| PathBuf::from("/"));

    if name.starts_with("./") {
        app_path = app_path.join(name[2..].to_owned());
    } else if name.starts_with(std::path::MAIN_SEPARATOR) {
        app_path = PathBuf::from(name);
    } else {
        if let Some(path_var) = env::var_os("PATH") {
            let paths = env::split_paths(&path_var);
            for path in paths {
                let full_path = path.join(name);
                if is_executable(&full_path) {
                    app_path = full_path;
                    break
                }
            };
        }
    }

    if !is_executable(&app_path) {
        return None
    }

    Some(app_path)
}

pub fn call_external(_state: &ShellState, path: &Path, args: &[&str], output: &mut dyn Write)
    -> io::Result<Option<ShellSignal>>
{
    let mut cmd = Command::new(path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stdout = cmd.stdout.take().unwrap();
    let mut stderr = cmd.stderr.take().unwrap();

    let (tx, rx) = mpsc::channel::<Vec<u8>>();

    {
        let tx = tx.clone();
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            while let Ok(n) = stdout.read(&mut buf) {
                if n == 0 { break }
                tx.send(buf[..n].to_vec()).unwrap();
            }
        });
    }

    {
        let tx = tx.clone();
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            while let Ok(n) = stderr.read(&mut buf) {
                if n == 0 { break }
                tx.send(buf[..n].to_vec()).unwrap();
            }
        });
    }
    drop(tx);

    for chunk in rx {
        output.write_all(&chunk)?;
    }

    cmd.wait()?;
    Ok(None)
}

pub fn call(state: &ShellState, name: &str, args: &[&str], output: &mut dyn Write)
    -> io::Result<Option<ShellSignal>>
{
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
