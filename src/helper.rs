use std::fs::DirEntry;
use std::{fs, io};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

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
