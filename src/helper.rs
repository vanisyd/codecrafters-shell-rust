use std::fs::DirEntry;
use std::{env, fs, io};
use std::env::split_paths;
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::string::ToString;

pub fn visit_dirs<T, R>(dir: &Path, cb: &mut T) -> io::Result<Option<R>>
where
    T: FnMut(&DirEntry) -> Option<R>
{
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                if let Some(resp) = cb(&entry) {
                    return Ok(Some(resp));
                }
            }
        }
    }
    
    Ok(None)
}

pub fn is_executable(path: &Path) -> bool {
    if path.is_file() && let Ok(metadata) = path.metadata() {
        if metadata.permissions().mode() & 0o111 != 0 {
            return true
        }
    }

    false
}

pub fn resolve_path(dir: &str, cur_dir: &Path) -> Result<PathBuf, ()> {
    let mut path = PathBuf::from(dir);

    let new_path: Result<PathBuf, ()> = if path.is_absolute() {
        Ok(path)
    } else {
        let mut directory = if path.starts_with("~") {
            if let Some(home_dir) = env::var_os("HOME") {
                path = path.strip_prefix("~/").unwrap().to_path_buf();
                PathBuf::from(home_dir.as_os_str())
            } else {
                return Err(())
            }
        } else {
            cur_dir.to_owned()
        };

        for path_part in path.components() {
            match path_part {
                Component::CurDir => {}
                Component::ParentDir => {
                    if directory != Path::new("/") {
                        directory.pop();
                    }
                },
                _ => {
                    directory.push(path_part)
                }
            };
        }

        Ok(directory)
    };

    match new_path {
        Ok(path) => {
            if path.is_dir() {
                Ok(path)
            } else {
                Err(())
            }
        },
        Err(_) => Err(())
    }
}
