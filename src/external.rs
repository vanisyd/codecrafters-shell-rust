use crate::helper::is_executable;
use crate::{ShellError, ShellSignal, ShellState};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::{env, io, thread};

pub fn find_external(name: &str, path: Option<PathBuf>) -> Option<PathBuf> {
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
                    break;
                }
            }
        }
    }

    if !is_executable(&app_path) {
        return None;
    }

    Some(app_path)
}

//Codecrafters tests don't work if app that in $PATH executed by full path
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
                    return Some(PathBuf::from(name));
                }
            }
        }
    }

    if !is_executable(&app_path) {
        return None;
    }

    Some(app_path)
}

pub fn call_external<'a>(
    state: &ShellState,
    path: &Path,
    args: &mut dyn Iterator<Item = &'a str>,
    output: &mut dyn Write,
) -> Result<Option<ShellSignal>, ShellError> {
    let mut cmd = Command::new(path)
        .args(args)
        .current_dir(state.current_dir.to_path_buf())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|_| ShellError::ExecutionError)?;

    let mut stdout = cmd.stdout.take().unwrap();
    let mut stderr = cmd.stderr.take().unwrap();

    let (tx, rx) = mpsc::channel::<Vec<u8>>();

    {
        let tx = tx.clone();
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            while let Ok(n) = stdout.read(&mut buf) {
                if n == 0 {
                    break;
                }
                tx.send(buf[..n].to_vec()).unwrap();
            }
        });
    }

    {
        let tx = tx.clone();
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            while let Ok(n) = stderr.read(&mut buf) {
                if n == 0 {
                    break;
                }
                tx.send(buf[..n].to_vec()).unwrap();
            }
        });
    }
    drop(tx);

    for chunk in rx {
        output
            .write_all(&chunk)
            .map_err(|_| ShellError::OutputError)?;
    }

    cmd.wait().map_err(|_| ShellError::ExecutionError)?;
    Ok(None)
}
